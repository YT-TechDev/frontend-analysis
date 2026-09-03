use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTableLayoutQualificationOutcome, CssTableLayoutUnsupportedReason, CssTableLayoutValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Fixed,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssTableLayoutQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssTableLayoutQualificationOutcome::Qualified(CssTableLayoutValue::Auto)
        }
        ExpectedOutcome::Fixed => {
            CssTableLayoutQualificationOutcome::Qualified(CssTableLayoutValue::Fixed)
        }
        ExpectedOutcome::Invalid => {
            CssTableLayoutQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssTableLayoutQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTableLayoutUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssTableLayoutQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTableLayoutUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .table_layout_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_two_keyword_grammar() {
    let result = qualify(
        1920,
        concat!(
            "a{table-layout:auto;}",
            "b{table-layout:fixed;}",
            "c{table-layout:AUTO;}",
            "d{TABLE-LAYOUT:FiXeD;}",
            r"e{table-layout:\61 uto;}",
            r"f{table-layout:f\69 xed;}",
            "g{table-layout:none;}",
            "h{table-layout:auto fixed;}",
            "i{table-layout:;}",
            "j{table-layout:1;}",
            "k{table-layout:1px;}",
            "l{table-layout:\"auto\";}",
            "m{color:fixed;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Fixed,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Fixed,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Fixed,
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
        1921,
        concat!(
            "a{table-layout:/**/auto/**/!important;}",
            "b{table-layout:/**/fixed/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Fixed]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        1922,
        concat!(
            "a{table-layout:initial;}",
            "b{table-layout:inherit;}",
            "c{table-layout:unset;}",
            "d{table-layout:revert;}",
            "e{table-layout:revert-layer;}",
            "f{table-layout:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        1923,
        concat!(
            "a{table-layout:var(--layout);}",
            "b{table-layout:env(layout);}",
            "c{table-layout:attr(data-layout);}",
            "d{table-layout:--layout();}",
            "e{table-layout:first-valid(auto,fixed);}",
            "f{table-layout:cycle(auto,fixed);}",
            "g{table-layout:interpolate(0%,0:auto,1:fixed);}",
            "h{table-layout:foo();}",
            "i{table-layout:calc(1);}",
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
        1924,
        concat!(
            "a{table-layout:auto first-valid(fixed);}",
            "b{table-layout:first-valid(fixed) auto;}",
            "c{table-layout:fixed foo();}",
            "d{table-layout:foo() auto;}",
            "e{table-layout:fixed var(--layout);}",
            "f{table-layout:foo(var(--layout));}",
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
fn one_run_interleaves_table_layout_with_every_accepted_leaf() {
    let result = qualify(
        1925,
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
    assert_eq!(result.table_layout_observations()[0].occurrence_index(), 22);
    assert_expected(&result, &[ExpectedOutcome::Fixed]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(1926, "a{table-layout:auto;}b{table-layout:auto;}");

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_eq!(result.table_layout_observations()[0].occurrence_index(), 0);
    assert_eq!(result.table_layout_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.table_layout_observations()[0]
            .placement()
            .context_id(),
        result.table_layout_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (1930, "@font-face{table-layout:fixed;}"),
        (1931, "@page{table-layout:fixed;}"),
        (1932, "@page{@top-left{table-layout:fixed;}}"),
        (1933, "@keyframes k{from{table-layout:fixed;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.table_layout_observations().is_empty(),
            "nonordinary declaration context produced a table-layout observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        1940,
        "a{table-layout:auto;table-layout:fixed;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Auto]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{table-layout:auto;}",
        "b{table-layout:inherit;}",
        "c{table-layout:none;}",
        "d{table-layout:var(--layout);}",
    );
    let first = qualify(1950, css);
    let repeated = qualify(1950, css);
    let another_source = qualify(1951, css);

    assert_eq!(
        first.table_layout_observations(),
        repeated.table_layout_observations()
    );
    assert_eq!(
        first.table_layout_observations(),
        another_source.table_layout_observations()
    );
}
