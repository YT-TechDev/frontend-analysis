use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBorderCollapseQualificationOutcome, CssBorderCollapseUnsupportedReason,
    CssBorderCollapseValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Separate,
    Collapse,
    Invalid,
    UnsupportedCssWide,
    UnsupportedFunction,
}

fn tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(4096, 100_000, 8192, 1024, 8192, 8192).unwrap()
}

fn parser_limits() -> CssParserLimits {
    parser_limits_with_occurrences(8192)
}

fn parser_limits_with_occurrences(max_declaration_occurrences: usize) -> CssParserLimits {
    CssParserLimits::new(
        100_000,
        256,
        256,
        max_declaration_occurrences,
        1024,
        1024,
        1024,
        1024,
        8192,
    )
    .unwrap()
}

fn qualify(source_id: u64, css: &str) -> CssValueQualificationRunResult {
    qualify_with_limits(source_id, css, parser_limits())
}

fn qualify_with_limits(
    source_id: u64,
    css: &str,
    parser_limits: CssParserLimits,
) -> CssValueQualificationRunResult {
    let source = SourceText::new(SourceId::new(source_id), css.to_owned());
    let parser_result = analyze_css_source(&source, tokenizer_limits(), parser_limits).unwrap();
    run(parser_result).unwrap()
}

fn expected_outcome(expected: ExpectedOutcome) -> CssBorderCollapseQualificationOutcome {
    match expected {
        ExpectedOutcome::Separate => {
            CssBorderCollapseQualificationOutcome::Qualified(CssBorderCollapseValue::Separate)
        }
        ExpectedOutcome::Collapse => {
            CssBorderCollapseQualificationOutcome::Qualified(CssBorderCollapseValue::Collapse)
        }
        ExpectedOutcome::Invalid => {
            CssBorderCollapseQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssBorderCollapseQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderCollapseUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssBorderCollapseQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderCollapseUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .border_collapse_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_two_keyword_grammar() {
    let result = qualify(
        1960,
        concat!(
            "a{border-collapse:separate;}",
            "b{border-collapse:collapse;}",
            "c{border-collapse:SEPARATE;}",
            "d{BORDER-COLLAPSE:CoLlApSe;}",
            r"e{border-collapse:\73 eparate;}",
            r"f{border-collapse:c\6f llapse;}",
            "g{border-collapse:none;}",
            "h{border-collapse:separate collapse;}",
            "i{border-collapse:;}",
            "j{border-collapse:1;}",
            "k{border-collapse:1px;}",
            "l{border-collapse:\"collapse\";}",
            "m{color:collapse;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Separate,
            ExpectedOutcome::Collapse,
            ExpectedOutcome::Separate,
            ExpectedOutcome::Collapse,
            ExpectedOutcome::Separate,
            ExpectedOutcome::Collapse,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn comments_and_priority_preserve_decoded_keyword_meaning() {
    let result = qualify(
        1961,
        concat!(
            "a{border-collapse:/**/separate/**/!important;}",
            "b{border-collapse:/**/collapse/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Separate, ExpectedOutcome::Collapse],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        1962,
        concat!(
            "a{border-collapse:initial;}",
            "b{border-collapse:inherit;}",
            "c{border-collapse:unset;}",
            "d{border-collapse:revert;}",
            "e{border-collapse:revert-layer;}",
            "f{border-collapse:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        1963,
        concat!(
            "a{border-collapse:var(--collapse);}",
            "b{border-collapse:env(collapse);}",
            "c{border-collapse:attr(data-collapse);}",
            "d{border-collapse:--collapse();}",
            "e{border-collapse:first-valid(separate,collapse);}",
            "f{border-collapse:cycle(separate,collapse);}",
            "g{border-collapse:interpolate(0%,0:separate,1:collapse);}",
            "h{border-collapse:foo();}",
            "i{border-collapse:calc(1);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn function_placement_preserves_keyword_leaf_fail_open_boundary() {
    let result = qualify(
        1964,
        concat!(
            "a{border-collapse:separate first-valid(collapse);}",
            "b{border-collapse:first-valid(collapse) separate;}",
            "c{border-collapse:collapse foo();}",
            "d{border-collapse:foo() separate;}",
            "e{border-collapse:collapse var(--collapse);}",
            "f{border-collapse:foo(var(--collapse));}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
        ],
    );
}

#[test]
fn one_run_interleaves_border_collapse_with_every_accepted_leaf() {
    let result = qualify(
        1965,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:isolate;}",
            "d{backface-visibility:hidden;}",
            "e{order:1;}",
            "f{column-count:2;}",
            "g{flex-grow:1;}",
            "h{flex-shrink:1;}",
            "i{opacity:.5;}",
            "j{shape-image-threshold:.5;}",
            "k{shape-margin:1px;}",
            "l{line-height:1;}",
            "m{word-spacing:-1px;}",
            "n{text-underline-offset:-10%;}",
            "o{scroll-margin-top:-1px;}",
            "p{border-top-width:thin;}",
            "q{perspective:1px;}",
            "r{z-index:1;}",
            "s{scroll-snap-align:center;}",
            "t{scroll-snap-stop:always;}",
            "u{empty-cells:hide;}",
            "v{text-decoration-style:wavy;}",
            "w{table-layout:fixed;}",
            "x{border-collapse:collapse;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.backface_visibility_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
    assert_eq!(result.column_count_observations().len(), 1);
    assert_eq!(result.flex_grow_observations().len(), 1);
    assert_eq!(result.flex_shrink_observations().len(), 1);
    assert_eq!(result.opacity_observations().len(), 1);
    assert_eq!(result.shape_image_threshold_observations().len(), 1);
    assert_eq!(result.shape_margin_observations().len(), 1);
    assert_eq!(result.line_height_observations().len(), 1);
    assert_eq!(result.word_spacing_observations().len(), 1);
    assert_eq!(result.text_underline_offset_observations().len(), 1);
    assert_eq!(result.scroll_margin_top_observations().len(), 1);
    assert_eq!(result.border_top_width_observations().len(), 1);
    assert_eq!(result.perspective_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.scroll_snap_align_observations().len(), 1);
    assert_eq!(result.scroll_snap_stop_observations().len(), 1);
    assert_eq!(result.empty_cells_observations().len(), 1);
    assert_eq!(result.text_decoration_style_observations().len(), 1);
    assert_eq!(result.table_layout_observations().len(), 1);
    assert_eq!(result.border_collapse_observations().len(), 1);
    assert_eq!(
        result.border_collapse_observations()[0].occurrence_index(),
        23
    );
    assert_expected(&result, &[ExpectedOutcome::Collapse]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        1966,
        "a{border-collapse:separate;}b{border-collapse:separate;}",
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Separate, ExpectedOutcome::Separate],
    );
    assert_eq!(
        result.border_collapse_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.border_collapse_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.border_collapse_observations()[0]
            .placement()
            .context_id(),
        result.border_collapse_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (1970, "@font-face{border-collapse:collapse;}"),
        (1971, "@page{border-collapse:collapse;}"),
        (1972, "@page{@top-left{border-collapse:collapse;}}"),
        (1973, "@keyframes k{from{border-collapse:collapse;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.border_collapse_observations().is_empty(),
            "nonordinary declaration context produced a border-collapse observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        1980,
        "a{border-collapse:separate;border-collapse:collapse;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Separate]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{border-collapse:separate;}",
        "b{border-collapse:inherit;}",
        "c{border-collapse:none;}",
        "d{border-collapse:var(--collapse);}",
    );
    let first = qualify(1990, css);
    let repeated = qualify(1990, css);
    let another_source = qualify(1991, css);

    assert_eq!(
        first.border_collapse_observations(),
        repeated.border_collapse_observations()
    );
    assert_eq!(
        first.border_collapse_observations(),
        another_source.border_collapse_observations()
    );
}
