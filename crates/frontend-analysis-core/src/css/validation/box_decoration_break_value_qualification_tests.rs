use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxDecorationBreakQualificationOutcome, CssBoxDecorationBreakUnsupportedReason,
    CssBoxDecorationBreakValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Slice,
    Clone,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssBoxDecorationBreakQualificationOutcome {
    match expected {
        ExpectedOutcome::Slice => {
            CssBoxDecorationBreakQualificationOutcome::Qualified(CssBoxDecorationBreakValue::Slice)
        }
        ExpectedOutcome::Clone => {
            CssBoxDecorationBreakQualificationOutcome::Qualified(CssBoxDecorationBreakValue::Clone)
        }
        ExpectedOutcome::Invalid => {
            CssBoxDecorationBreakQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssBoxDecorationBreakQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBoxDecorationBreakUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssBoxDecorationBreakQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBoxDecorationBreakUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .box_decoration_break_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_two_keyword_grammar() {
    let result = qualify(
        2000,
        concat!(
            "a{box-decoration-break:slice;}",
            "b{box-decoration-break:clone;}",
            "c{box-decoration-break:SLICE;}",
            "d{BOX-DECORATION-BREAK:ClOnE;}",
            r"e{box-decoration-break:\73 lice;}",
            r"f{box-decoration-break:c\6c one;}",
            "g{box-decoration-break:auto;}",
            "h{box-decoration-break:slice clone;}",
            "i{box-decoration-break:;}",
            "j{box-decoration-break:1;}",
            "k{box-decoration-break:1px;}",
            "l{box-decoration-break:\"clone\";}",
            "m{color:clone;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Slice,
            ExpectedOutcome::Clone,
            ExpectedOutcome::Slice,
            ExpectedOutcome::Clone,
            ExpectedOutcome::Slice,
            ExpectedOutcome::Clone,
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
        2001,
        concat!(
            "a{box-decoration-break:/**/slice/**/!important;}",
            "b{box-decoration-break:/**/clone/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Slice, ExpectedOutcome::Clone]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2002,
        concat!(
            "a{box-decoration-break:initial;}",
            "b{box-decoration-break:inherit;}",
            "c{box-decoration-break:unset;}",
            "d{box-decoration-break:revert;}",
            "e{box-decoration-break:revert-layer;}",
            "f{box-decoration-break:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2003,
        concat!(
            "a{box-decoration-break:var(--clone);}",
            "b{box-decoration-break:env(clone);}",
            "c{box-decoration-break:attr(data-clone);}",
            "d{box-decoration-break:--clone();}",
            "e{box-decoration-break:first-valid(slice,clone);}",
            "f{box-decoration-break:cycle(slice,clone);}",
            "g{box-decoration-break:interpolate(0%,0:slice,1:clone);}",
            "h{box-decoration-break:foo();}",
            "i{box-decoration-break:calc(1);}",
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
        2004,
        concat!(
            "a{box-decoration-break:slice first-valid(clone);}",
            "b{box-decoration-break:first-valid(clone) slice;}",
            "c{box-decoration-break:clone foo();}",
            "d{box-decoration-break:foo() slice;}",
            "e{box-decoration-break:clone var(--clone);}",
            "f{box-decoration-break:foo(var(--clone));}",
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
fn one_run_interleaves_box_decoration_break_with_every_accepted_leaf() {
    let result = qualify(
        2005,
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
            "y{box-decoration-break:clone;}",
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
    assert_eq!(result.box_decoration_break_observations().len(), 1);
    assert_eq!(
        result.box_decoration_break_observations()[0].occurrence_index(),
        24
    );
    assert_expected(&result, &[ExpectedOutcome::Clone]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2006,
        "a{box-decoration-break:slice;}b{box-decoration-break:slice;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Slice, ExpectedOutcome::Slice]);
    assert_eq!(
        result.box_decoration_break_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.box_decoration_break_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.box_decoration_break_observations()[0]
            .placement()
            .context_id(),
        result.box_decoration_break_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2010, "@font-face{box-decoration-break:clone;}"),
        (2011, "@page{box-decoration-break:clone;}"),
        (2012, "@page{@top-left{box-decoration-break:clone;}}"),
        (2013, "@keyframes k{from{box-decoration-break:clone;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.box_decoration_break_observations().is_empty(),
            "nonordinary declaration context produced a box-decoration-break observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2020,
        "a{box-decoration-break:slice;box-decoration-break:clone;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Slice]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{box-decoration-break:slice;}",
        "b{box-decoration-break:inherit;}",
        "c{box-decoration-break:auto;}",
        "d{box-decoration-break:var(--clone);}",
    );
    let first = qualify(2030, css);
    let repeated = qualify(2030, css);
    let another_source = qualify(2031, css);

    assert_eq!(
        first.box_decoration_break_observations(),
        repeated.box_decoration_break_observations()
    );
    assert_eq!(
        first.box_decoration_break_observations(),
        another_source.box_decoration_break_observations()
    );
}
