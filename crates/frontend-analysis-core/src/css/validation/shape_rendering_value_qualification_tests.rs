use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssShapeRenderingQualificationOutcome, CssShapeRenderingUnsupportedReason,
    CssShapeRenderingValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    OptimizeSpeed,
    CrispEdges,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssShapeRenderingQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssShapeRenderingQualificationOutcome::Qualified(CssShapeRenderingValue::Auto)
        }
        ExpectedOutcome::OptimizeSpeed => {
            CssShapeRenderingQualificationOutcome::Qualified(CssShapeRenderingValue::OptimizeSpeed)
        }
        ExpectedOutcome::CrispEdges => {
            CssShapeRenderingQualificationOutcome::Qualified(CssShapeRenderingValue::CrispEdges)
        }
        ExpectedOutcome::GeometricPrecision => CssShapeRenderingQualificationOutcome::Qualified(
            CssShapeRenderingValue::GeometricPrecision,
        ),
        ExpectedOutcome::Invalid => {
            CssShapeRenderingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssShapeRenderingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeRenderingUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssShapeRenderingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeRenderingUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .shape_rendering_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_shape_rendering_grammar() {
    let result = qualify(
        2860,
        concat!(
            "a{shape-rendering:auto;}",
            "b{shape-rendering:optimizeSpeed;}",
            "c{shape-rendering:crispEdges;}",
            "d{shape-rendering:geometricPrecision;}",
            "e{SHAPE-RENDERING:GeOmEtRiCpReCiSiOn;}",
            r"f{shape-rendering:optimize\53 peed;}",
            r"g{shape-\72 endering:crispEdges;}",
            "h{shape-rendering:optimizeLegibility;}",
            "i{shape-rendering:auto optimizeSpeed;}",
            "j{shape-rendering:;}",
            "k{shape-rendering:1;}",
            "l{shape-rendering:1px;}",
            "m{shape-rendering:\"crispEdges\";}",
            "n{color:crispEdges;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::OptimizeSpeed,
            ExpectedOutcome::CrispEdges,
            ExpectedOutcome::GeometricPrecision,
            ExpectedOutcome::GeometricPrecision,
            ExpectedOutcome::OptimizeSpeed,
            ExpectedOutcome::CrispEdges,
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
        2861,
        concat!(
            "a{shape-rendering:/**/auto/**/!important;}",
            "b{shape-rendering:/**/optimizeSpeed/**/!important;}",
            "c{shape-rendering:/**/crispEdges/**/!important;}",
            "d{shape-rendering:/**/geometricPrecision/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::OptimizeSpeed,
            ExpectedOutcome::CrispEdges,
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
        2862,
        concat!(
            "a{shape-rendering:initial;}",
            "b{shape-rendering:inherit;}",
            "c{shape-rendering:unset;}",
            "d{shape-rendering:revert;}",
            "e{shape-rendering:revert-layer;}",
            "f{shape-rendering:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2863,
        concat!(
            "a{shape-rendering:var(--hint);}",
            "b{shape-rendering:env(hint);}",
            "c{shape-rendering:attr(data-hint);}",
            "d{shape-rendering:--hint();}",
            "e{shape-rendering:first-valid(crispEdges,auto);}",
            "f{shape-rendering:cycle(auto,crispEdges);}",
            "g{shape-rendering:interpolate(0%,0:auto,1:crispEdges);}",
            "h{shape-rendering:foo();}",
            "i{shape-rendering:calc(1);}",
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
        2864,
        concat!(
            "a{shape-rendering:auto first-valid(crispEdges);}",
            "b{shape-rendering:first-valid(crispEdges) auto;}",
            "c{shape-rendering:crispEdges foo();}",
            "d{shape-rendering:foo() crispEdges;}",
            "e{shape-rendering:geometricPrecision var(--hint);}",
            "f{shape-rendering:foo(var(--hint));}",
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
fn svg_shape_applicability_is_not_an_input_to_qualification() {
    let result = qualify(
        2865,
        concat!(
            "path{shape-rendering:crispEdges;}",
            "div{shape-rendering:crispEdges;}",
            "svg rect{shape-rendering:crispEdges;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::CrispEdges,
            ExpectedOutcome::CrispEdges,
            ExpectedOutcome::CrispEdges,
        ],
    );
    assert_eq!(result.shape_rendering_observations()[0].occurrence_index(), 0);
    assert_eq!(result.shape_rendering_observations()[1].occurrence_index(), 1);
    assert_eq!(result.shape_rendering_observations()[2].occurrence_index(), 2);
    assert_eq!(
        result.shape_rendering_observations()[0].outcome(),
        result.shape_rendering_observations()[1].outcome()
    );
    assert_eq!(
        result.shape_rendering_observations()[1].outcome(),
        result.shape_rendering_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_shape_rendering_with_every_accepted_leaf() {
    let result = qualify(
        2866,
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
    assert_eq!(result.shape_rendering_observations()[0].occurrence_index(), 38);
    assert_expected(&result, &[ExpectedOutcome::GeometricPrecision]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2867,
        "a{shape-rendering:crispEdges;}b{shape-rendering:crispEdges;}",
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::CrispEdges, ExpectedOutcome::CrispEdges],
    );
    assert_eq!(result.shape_rendering_observations()[0].occurrence_index(), 0);
    assert_eq!(result.shape_rendering_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.shape_rendering_observations()[0]
            .placement()
            .context_id(),
        result.shape_rendering_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2870, "@font-face{shape-rendering:crispEdges;}"),
        (2871, "@page{shape-rendering:crispEdges;}"),
        (2872, "@page{@top-left{shape-rendering:crispEdges;}}"),
        (2873, "@keyframes k{from{shape-rendering:crispEdges;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.shape_rendering_observations().is_empty(),
            "nonordinary declaration context produced a shape-rendering observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2880,
        "a{shape-rendering:auto;shape-rendering:crispEdges;}",
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
        "a{shape-rendering:auto;}",
        "b{shape-rendering:inherit;}",
        "c{shape-rendering:optimizeLegibility;}",
        "d{shape-rendering:var(--hint);}",
        "e{color:crispEdges;}",
    );
    let first = qualify(2890, css);
    let repeated = qualify(2890, css);
    let another_source = qualify(2891, css);

    assert_eq!(
        first.shape_rendering_observations(),
        repeated.shape_rendering_observations()
    );
    assert_eq!(
        first.shape_rendering_observations(),
        another_source.shape_rendering_observations()
    );
}
