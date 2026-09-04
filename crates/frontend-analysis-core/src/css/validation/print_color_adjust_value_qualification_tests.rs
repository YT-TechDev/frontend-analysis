use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssPrintColorAdjustQualificationOutcome, CssPrintColorAdjustUnsupportedReason,
    CssPrintColorAdjustValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Economy,
    Exact,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssPrintColorAdjustQualificationOutcome {
    match expected {
        ExpectedOutcome::Economy => {
            CssPrintColorAdjustQualificationOutcome::Qualified(CssPrintColorAdjustValue::Economy)
        }
        ExpectedOutcome::Exact => {
            CssPrintColorAdjustQualificationOutcome::Qualified(CssPrintColorAdjustValue::Exact)
        }
        ExpectedOutcome::Invalid => {
            CssPrintColorAdjustQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssPrintColorAdjustQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPrintColorAdjustUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssPrintColorAdjustQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPrintColorAdjustUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .print_color_adjust_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_print_color_adjust_grammar() {
    let result = qualify(
        2600,
        concat!(
            "a{print-color-adjust:economy;}",
            "b{print-color-adjust:exact;}",
            "c{PRINT-COLOR-ADJUST:ExAcT;}",
            r"d{print-color-adjust:\65 conomy;}",
            r"e{print-color-adjust:\65 xact;}",
            "f{print-color-adjust:normal;}",
            "g{print-color-adjust:economy exact;}",
            "h{print-color-adjust:exact economy;}",
            "i{print-color-adjust:exact, economy;}",
            "j{print-color-adjust:;}",
            "k{print-color-adjust:1;}",
            "l{print-color-adjust:1px;}",
            "m{print-color-adjust:\"exact\";}",
            "n{color:exact;}",
            "o{color-adjust:exact;}",
            "p{-webkit-print-color-adjust:exact;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Economy,
            ExpectedOutcome::Exact,
            ExpectedOutcome::Exact,
            ExpectedOutcome::Economy,
            ExpectedOutcome::Exact,
            ExpectedOutcome::Invalid,
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
fn comments_and_priority_preserve_decoded_keyword_meaning() {
    let result = qualify(
        2601,
        concat!(
            "a{print-color-adjust:/**/economy/**/!important;}",
            "b{print-color-adjust:/**/exact/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Economy, ExpectedOutcome::Exact]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2602,
        concat!(
            "a{print-color-adjust:initial;}",
            "b{print-color-adjust:inherit;}",
            "c{print-color-adjust:unset;}",
            "d{print-color-adjust:revert;}",
            "e{print-color-adjust:revert-layer;}",
            "f{print-color-adjust:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2603,
        concat!(
            "a{print-color-adjust:var(--print-color-adjust);}",
            "b{print-color-adjust:env(print-color-adjust);}",
            "c{print-color-adjust:attr(data-print-color-adjust);}",
            "d{print-color-adjust:--print-color-adjust();}",
            "e{print-color-adjust:first-valid(exact,economy);}",
            "f{print-color-adjust:cycle(economy,exact);}",
            "g{print-color-adjust:interpolate(0%,0:economy,1:exact);}",
            "h{print-color-adjust:foo();}",
            "i{print-color-adjust:calc(1);}",
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
        2604,
        concat!(
            "a{print-color-adjust:economy first-valid(exact);}",
            "b{print-color-adjust:first-valid(exact) economy;}",
            "c{print-color-adjust:exact foo();}",
            "d{print-color-adjust:foo() exact;}",
            "e{print-color-adjust:economy var(--print-color-adjust);}",
            "f{print-color-adjust:foo(var(--print-color-adjust));}",
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
fn one_run_interleaves_print_color_adjust_with_every_accepted_leaf() {
    let result = qualify(
        2605,
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
    assert_eq!(
        result.print_color_adjust_observations()[0].occurrence_index(),
        33
    );
    assert_expected(&result, &[ExpectedOutcome::Exact]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2606,
        "a{print-color-adjust:exact;}b{print-color-adjust:exact;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Exact, ExpectedOutcome::Exact]);
    assert_eq!(
        result.print_color_adjust_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.print_color_adjust_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.print_color_adjust_observations()[0]
            .placement()
            .context_id(),
        result.print_color_adjust_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2610, "@font-face{print-color-adjust:exact;}"),
        (2611, "@page{print-color-adjust:exact;}"),
        (2612, "@page{@top-left{print-color-adjust:exact;}}"),
        (2613, "@keyframes k{from{print-color-adjust:exact;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.print_color_adjust_observations().is_empty(),
            "nonordinary declaration context produced a print-color-adjust observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2620,
        "a{print-color-adjust:economy;print-color-adjust:exact;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Economy]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{print-color-adjust:economy;}",
        "b{print-color-adjust:inherit;}",
        "c{print-color-adjust:normal;}",
        "d{print-color-adjust:var(--print-color-adjust);}",
    );
    let first = qualify(2630, css);
    let repeated = qualify(2630, css);
    let another_source = qualify(2631, css);

    assert_eq!(
        first.print_color_adjust_observations(),
        repeated.print_color_adjust_observations()
    );
    assert_eq!(
        first.print_color_adjust_observations(),
        another_source.print_color_adjust_observations()
    );
}
