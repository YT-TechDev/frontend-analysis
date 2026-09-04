use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssColorInterpolationFiltersQualificationOutcome,
    CssColorInterpolationFiltersUnsupportedReason, CssColorInterpolationFiltersValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Srgb,
    LinearRgb,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssColorInterpolationFiltersQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => CssColorInterpolationFiltersQualificationOutcome::Qualified(
            CssColorInterpolationFiltersValue::Auto,
        ),
        ExpectedOutcome::Srgb => CssColorInterpolationFiltersQualificationOutcome::Qualified(
            CssColorInterpolationFiltersValue::Srgb,
        ),
        ExpectedOutcome::LinearRgb => CssColorInterpolationFiltersQualificationOutcome::Qualified(
            CssColorInterpolationFiltersValue::LinearRgb,
        ),
        ExpectedOutcome::Invalid => {
            CssColorInterpolationFiltersQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssColorInterpolationFiltersQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColorInterpolationFiltersUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssColorInterpolationFiltersQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColorInterpolationFiltersUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .color_interpolation_filters_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_color_interpolation_filters_grammar() {
    let result = qualify(
        2820,
        concat!(
            "a{color-interpolation-filters:auto;}",
            "b{color-interpolation-filters:sRGB;}",
            "c{color-interpolation-filters:linearRGB;}",
            "d{COLOR-INTERPOLATION-FILTERS:LiNeArRgB;}",
            r"e{color-interpolation-filters:s\52 GB;}",
            r"f{color-interpolation-filt\65 rs:auto;}",
            "g{color-interpolation-filters:none;}",
            "h{color-interpolation-filters:linearRGB sRGB;}",
            "i{color-interpolation-filters:auto sRGB linearRGB;}",
            "j{color-interpolation-filters:;}",
            "k{color-interpolation-filters:1;}",
            "l{color-interpolation-filters:1px;}",
            "m{color-interpolation-filters:\"sRGB\";}",
            "n{color:sRGB;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Srgb,
            ExpectedOutcome::LinearRgb,
            ExpectedOutcome::LinearRgb,
            ExpectedOutcome::Srgb,
            ExpectedOutcome::Auto,
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
        2821,
        concat!(
            "a{color-interpolation-filters:/**/auto/**/!important;}",
            "b{color-interpolation-filters:/**/sRGB/**/!important;}",
            "c{color-interpolation-filters:/**/linearRGB/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Srgb,
            ExpectedOutcome::LinearRgb,
        ],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2822,
        concat!(
            "a{color-interpolation-filters:initial;}",
            "b{color-interpolation-filters:inherit;}",
            "c{color-interpolation-filters:unset;}",
            "d{color-interpolation-filters:revert;}",
            "e{color-interpolation-filters:revert-layer;}",
            "f{color-interpolation-filters:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2823,
        concat!(
            "a{color-interpolation-filters:var(--space);}",
            "b{color-interpolation-filters:env(space);}",
            "c{color-interpolation-filters:attr(data-space);}",
            "d{color-interpolation-filters:--space();}",
            "e{color-interpolation-filters:first-valid(sRGB,linearRGB);}",
            "f{color-interpolation-filters:cycle(sRGB,linearRGB);}",
            "g{color-interpolation-filters:interpolate(0%,0:sRGB,1:linearRGB);}",
            "h{color-interpolation-filters:foo();}",
            "i{color-interpolation-filters:calc(1);}",
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
        2824,
        concat!(
            "a{color-interpolation-filters:sRGB first-valid(linearRGB);}",
            "b{color-interpolation-filters:first-valid(linearRGB) sRGB;}",
            "c{color-interpolation-filters:auto foo();}",
            "d{color-interpolation-filters:foo() auto;}",
            "e{color-interpolation-filters:linearRGB var(--space);}",
            "f{color-interpolation-filters:foo(var(--space));}",
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
fn filter_primitive_applicability_is_not_an_input_to_qualification() {
    let result = qualify(
        2825,
        concat!(
            "filter{color-interpolation-filters:sRGB;}",
            "div{color-interpolation-filters:sRGB;}",
            "svg feGaussianBlur{color-interpolation-filters:sRGB;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Srgb,
            ExpectedOutcome::Srgb,
            ExpectedOutcome::Srgb,
        ],
    );
    assert_eq!(
        result.color_interpolation_filters_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.color_interpolation_filters_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(
        result.color_interpolation_filters_observations()[2].occurrence_index(),
        2
    );
    assert_eq!(
        result.color_interpolation_filters_observations()[0].outcome(),
        result.color_interpolation_filters_observations()[1].outcome()
    );
    assert_eq!(
        result.color_interpolation_filters_observations()[1].outcome(),
        result.color_interpolation_filters_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_color_interpolation_filters_with_every_accepted_leaf() {
    let result = qualify(
        2826,
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
    assert_eq!(
        result.color_interpolation_filters_observations()[0].occurrence_index(),
        37
    );
    assert_expected(&result, &[ExpectedOutcome::LinearRgb]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2827,
        "a{color-interpolation-filters:sRGB;}b{color-interpolation-filters:sRGB;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Srgb, ExpectedOutcome::Srgb]);
    assert_eq!(
        result.color_interpolation_filters_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.color_interpolation_filters_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.color_interpolation_filters_observations()[0]
            .placement()
            .context_id(),
        result.color_interpolation_filters_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2830, "@font-face{color-interpolation-filters:sRGB;}"),
        (2831, "@page{color-interpolation-filters:sRGB;}"),
        (2832, "@page{@top-left{color-interpolation-filters:sRGB;}}"),
        (
            2833,
            "@keyframes k{from{color-interpolation-filters:sRGB;}}",
        ),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.color_interpolation_filters_observations().is_empty(),
            "nonordinary declaration context produced a color-interpolation-filters observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2840,
        "a{color-interpolation-filters:auto;color-interpolation-filters:sRGB;}",
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
        "a{color-interpolation-filters:auto;}",
        "b{color-interpolation-filters:inherit;}",
        "c{color-interpolation-filters:none;}",
        "d{color-interpolation-filters:var(--space);}",
        "e{color:sRGB;}",
    );
    let first = qualify(2850, css);
    let repeated = qualify(2850, css);
    let another_source = qualify(2851, css);

    assert_eq!(
        first.color_interpolation_filters_observations(),
        repeated.color_interpolation_filters_observations()
    );
    assert_eq!(
        first.color_interpolation_filters_observations(),
        another_source.color_interpolation_filters_observations()
    );
}
