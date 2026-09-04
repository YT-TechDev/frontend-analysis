use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextRenderingQualificationOutcome, CssTextRenderingUnsupportedReason, CssTextRenderingValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssTextRenderingQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssTextRenderingQualificationOutcome::Qualified(CssTextRenderingValue::Auto)
        }
        ExpectedOutcome::OptimizeSpeed => {
            CssTextRenderingQualificationOutcome::Qualified(CssTextRenderingValue::OptimizeSpeed)
        }
        ExpectedOutcome::OptimizeLegibility => CssTextRenderingQualificationOutcome::Qualified(
            CssTextRenderingValue::OptimizeLegibility,
        ),
        ExpectedOutcome::GeometricPrecision => CssTextRenderingQualificationOutcome::Qualified(
            CssTextRenderingValue::GeometricPrecision,
        ),
        ExpectedOutcome::Invalid => {
            CssTextRenderingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssTextRenderingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextRenderingUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssTextRenderingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextRenderingUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .text_rendering_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_text_rendering_grammar() {
    let result = qualify(
        2900,
        concat!(
            "a{text-rendering:auto;}",
            "b{text-rendering:optimizeSpeed;}",
            "c{text-rendering:optimizeLegibility;}",
            "d{text-rendering:geometricPrecision;}",
            "e{TEXT-RENDERING:GeOmEtRiCpReCiSiOn;}",
            r"f{text-rendering:optimize\53 peed;}",
            r"g{text-\72 endering:optimizeLegibility;}",
            "h{text-rendering:crispEdges;}",
            "i{text-rendering:auto optimizeSpeed;}",
            "j{text-rendering:;}",
            "k{text-rendering:1;}",
            "l{text-rendering:1px;}",
            "m{text-rendering:\"optimizeLegibility\";}",
            "n{color:optimizeLegibility;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::OptimizeSpeed,
            ExpectedOutcome::OptimizeLegibility,
            ExpectedOutcome::GeometricPrecision,
            ExpectedOutcome::GeometricPrecision,
            ExpectedOutcome::OptimizeSpeed,
            ExpectedOutcome::OptimizeLegibility,
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
        2901,
        concat!(
            "a{text-rendering:/**/auto/**/!important;}",
            "b{text-rendering:/**/optimizeSpeed/**/!important;}",
            "c{text-rendering:/**/optimizeLegibility/**/!important;}",
            "d{text-rendering:/**/geometricPrecision/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::OptimizeSpeed,
            ExpectedOutcome::OptimizeLegibility,
            ExpectedOutcome::GeometricPrecision,
        ],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2902,
        concat!(
            "a{text-rendering:initial;}",
            "b{text-rendering:inherit;}",
            "c{text-rendering:unset;}",
            "d{text-rendering:revert;}",
            "e{text-rendering:revert-layer;}",
            "f{text-rendering:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2903,
        concat!(
            "a{text-rendering:var(--hint);}",
            "b{text-rendering:env(hint);}",
            "c{text-rendering:attr(data-hint);}",
            "d{text-rendering:--hint();}",
            "e{text-rendering:first-valid(optimizeLegibility,auto);}",
            "f{text-rendering:cycle(auto,optimizeLegibility);}",
            "g{text-rendering:interpolate(0%,0:auto,1:optimizeLegibility);}",
            "h{text-rendering:foo();}",
            "i{text-rendering:calc(1);}",
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
        2904,
        concat!(
            "a{text-rendering:auto first-valid(optimizeLegibility);}",
            "b{text-rendering:first-valid(optimizeLegibility) auto;}",
            "c{text-rendering:optimizeLegibility foo();}",
            "d{text-rendering:foo() optimizeLegibility;}",
            "e{text-rendering:geometricPrecision var(--hint);}",
            "f{text-rendering:foo(var(--hint));}",
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
fn svg_text_applicability_is_not_an_input_to_qualification() {
    let result = qualify(
        2905,
        concat!(
            "text{text-rendering:optimizeLegibility;}",
            "div{text-rendering:optimizeLegibility;}",
            "svg text{text-rendering:optimizeLegibility;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::OptimizeLegibility,
            ExpectedOutcome::OptimizeLegibility,
            ExpectedOutcome::OptimizeLegibility,
        ],
    );
    assert_eq!(
        result.text_rendering_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.text_rendering_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(
        result.text_rendering_observations()[2].occurrence_index(),
        2
    );
    assert_eq!(
        result.text_rendering_observations()[0].outcome(),
        result.text_rendering_observations()[1].outcome()
    );
    assert_eq!(
        result.text_rendering_observations()[1].outcome(),
        result.text_rendering_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_text_rendering_with_every_accepted_leaf() {
    let result = qualify(
        2906,
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
            "aj{unicode-bidi:plaintext;}",
            "ak{mask-type:alpha;}",
            "al{color-interpolation-filters:linearRGB;}",
            "am{shape-rendering:geometricPrecision;}",
            "an{text-rendering:optimizeLegibility;}",
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
    assert_eq!(result.unicode_bidi_observations().len(), 1);
    assert_eq!(result.mask_type_observations().len(), 1);
    assert_eq!(result.color_interpolation_filters_observations().len(), 1);
    assert_eq!(result.shape_rendering_observations().len(), 1);
    assert_eq!(result.text_rendering_observations().len(), 1);
    assert_eq!(
        result.text_rendering_observations()[0].occurrence_index(),
        39
    );
    assert_expected(&result, &[ExpectedOutcome::OptimizeLegibility]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2907,
        "a{text-rendering:optimizeLegibility;}b{text-rendering:optimizeLegibility;}",
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::OptimizeLegibility,
            ExpectedOutcome::OptimizeLegibility,
        ],
    );
    assert_eq!(
        result.text_rendering_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.text_rendering_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.text_rendering_observations()[0]
            .placement()
            .context_id(),
        result.text_rendering_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2910, "@font-face{text-rendering:optimizeLegibility;}"),
        (2911, "@page{text-rendering:optimizeLegibility;}"),
        (2912, "@page{@top-left{text-rendering:optimizeLegibility;}}"),
        (
            2913,
            "@keyframes k{from{text-rendering:optimizeLegibility;}}",
        ),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_rendering_observations().is_empty(),
            "nonordinary declaration context produced a text-rendering observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2920,
        "a{text-rendering:auto;text-rendering:optimizeLegibility;}",
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
        "a{text-rendering:auto;}",
        "b{text-rendering:inherit;}",
        "c{text-rendering:crispEdges;}",
        "d{text-rendering:var(--hint);}",
        "e{color:optimizeLegibility;}",
    );
    let first = qualify(2930, css);
    let repeated = qualify(2930, css);
    let another_source = qualify(2931, css);

    assert_eq!(
        first.text_rendering_observations(),
        repeated.text_rendering_observations()
    );
    assert_eq!(
        first.text_rendering_observations(),
        another_source.text_rendering_observations()
    );
}
