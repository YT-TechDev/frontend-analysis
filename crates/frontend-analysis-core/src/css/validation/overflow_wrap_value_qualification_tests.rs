use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssOverflowWrapQualificationOutcome, CssOverflowWrapUnsupportedReason, CssOverflowWrapValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    BreakWord,
    Anywhere,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssOverflowWrapQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssOverflowWrapQualificationOutcome::Qualified(CssOverflowWrapValue::Normal)
        }
        ExpectedOutcome::BreakWord => {
            CssOverflowWrapQualificationOutcome::Qualified(CssOverflowWrapValue::BreakWord)
        }
        ExpectedOutcome::Anywhere => {
            CssOverflowWrapQualificationOutcome::Qualified(CssOverflowWrapValue::Anywhere)
        }
        ExpectedOutcome::Invalid => {
            CssOverflowWrapQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssOverflowWrapQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverflowWrapUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssOverflowWrapQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverflowWrapUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .overflow_wrap_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_overflow_wrap_grammar() {
    let result = qualify(
        2700,
        concat!(
            "a{overflow-wrap:normal;}",
            "b{overflow-wrap:break-word;}",
            "c{overflow-wrap:anywhere;}",
            "d{OVERFLOW-WRAP:AnYwHeRe;}",
            r"e{overflow-wrap:\62 reak-word;}",
            r"f{overflow-\77 rap:anywhere;}",
            "g{overflow-wrap:auto;}",
            "h{overflow-wrap:normal break-word;}",
            "i{overflow-wrap:anywhere anywhere;}",
            "j{overflow-wrap:;}",
            "k{overflow-wrap:1;}",
            "l{overflow-wrap:1px;}",
            "m{overflow-wrap:\"anywhere\";}",
            "n{word-wrap:anywhere;}",
            "o{color:anywhere;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::BreakWord,
            ExpectedOutcome::Anywhere,
            ExpectedOutcome::Anywhere,
            ExpectedOutcome::BreakWord,
            ExpectedOutcome::Anywhere,
            ExpectedOutcome::Invalid,
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
fn legacy_word_wrap_alias_is_explicitly_outside_selected_property_boundary() {
    let result = qualify(
        2701,
        concat!(
            "a{word-wrap:normal;}",
            "b{word-wrap:break-word;}",
            "c{word-wrap:anywhere;}",
            "d{overflow-wrap:anywhere;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Anywhere]);
    assert_eq!(result.overflow_wrap_observations()[0].occurrence_index(), 3);
}

#[test]
fn comments_and_priority_preserve_decoded_keyword_meaning() {
    let result = qualify(
        2702,
        concat!(
            "a{overflow-wrap:/**/normal/**/!important;}",
            "b{overflow-wrap:/**/break-word/**/!important;}",
            "c{overflow-wrap:/**/anywhere/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::BreakWord,
            ExpectedOutcome::Anywhere,
        ],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2703,
        concat!(
            "a{overflow-wrap:initial;}",
            "b{overflow-wrap:inherit;}",
            "c{overflow-wrap:unset;}",
            "d{overflow-wrap:revert;}",
            "e{overflow-wrap:revert-layer;}",
            "f{overflow-wrap:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2704,
        concat!(
            "a{overflow-wrap:var(--wrap);}",
            "b{overflow-wrap:env(wrap);}",
            "c{overflow-wrap:attr(data-wrap);}",
            "d{overflow-wrap:--wrap();}",
            "e{overflow-wrap:first-valid(normal,anywhere);}",
            "f{overflow-wrap:cycle(normal,anywhere);}",
            "g{overflow-wrap:interpolate(0%,0:normal,1:anywhere);}",
            "h{overflow-wrap:foo();}",
            "i{overflow-wrap:calc(1);}",
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
        2705,
        concat!(
            "a{overflow-wrap:normal first-valid(anywhere);}",
            "b{overflow-wrap:first-valid(anywhere) normal;}",
            "c{overflow-wrap:anywhere foo();}",
            "d{overflow-wrap:foo() anywhere;}",
            "e{overflow-wrap:normal var(--wrap);}",
            "f{overflow-wrap:foo(var(--wrap));}",
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
fn one_run_interleaves_overflow_wrap_with_every_accepted_leaf() {
    let result = qualify(
        2706,
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
            "z{font-kerning:normal;}",
            "aa{font-variant-position:super;}",
            "ab{font-synthesis-weight:none;}",
            "ac{font-synthesis-small-caps:none;}",
            "ad{font-synthesis-position:none;}",
            "ae{font-variant-emoji:emoji;}",
            "af{font-variant-caps:unicase;}",
            "ag{line-break:anywhere;}",
            "ah{print-color-adjust:exact;}",
            "ai{overflow-wrap:anywhere;}",
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
    assert_eq!(result.font_kerning_observations().len(), 1);
    assert_eq!(result.font_variant_position_observations().len(), 1);
    assert_eq!(result.font_synthesis_weight_observations().len(), 1);
    assert_eq!(result.font_synthesis_small_caps_observations().len(), 1);
    assert_eq!(result.font_synthesis_position_observations().len(), 1);
    assert_eq!(result.font_variant_emoji_observations().len(), 1);
    assert_eq!(result.font_variant_caps_observations().len(), 1);
    assert_eq!(result.line_break_observations().len(), 1);
    assert_eq!(result.print_color_adjust_observations().len(), 1);
    assert_eq!(result.overflow_wrap_observations().len(), 1);
    assert_eq!(result.overflow_wrap_observations()[0].occurrence_index(), 34);
    assert_expected(&result, &[ExpectedOutcome::Anywhere]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2707,
        "a{overflow-wrap:anywhere;}b{overflow-wrap:anywhere;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Anywhere, ExpectedOutcome::Anywhere]);
    assert_eq!(result.overflow_wrap_observations()[0].occurrence_index(), 0);
    assert_eq!(result.overflow_wrap_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.overflow_wrap_observations()[0].placement().context_id(),
        result.overflow_wrap_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2710, "@font-face{overflow-wrap:anywhere;}"),
        (2711, "@page{overflow-wrap:anywhere;}"),
        (2712, "@page{@top-left{overflow-wrap:anywhere;}}"),
        (2713, "@keyframes k{from{overflow-wrap:anywhere;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.overflow_wrap_observations().is_empty(),
            "nonordinary declaration context produced an overflow-wrap observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2720,
        "a{overflow-wrap:normal;overflow-wrap:anywhere;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Normal]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{overflow-wrap:normal;}",
        "b{overflow-wrap:inherit;}",
        "c{overflow-wrap:auto;}",
        "d{overflow-wrap:var(--wrap);}",
        "e{word-wrap:anywhere;}",
    );
    let first = qualify(2730, css);
    let repeated = qualify(2730, css);
    let another_source = qualify(2731, css);

    assert_eq!(
        first.overflow_wrap_observations(),
        repeated.overflow_wrap_observations()
    );
    assert_eq!(
        first.overflow_wrap_observations(),
        another_source.overflow_wrap_observations()
    );
}
