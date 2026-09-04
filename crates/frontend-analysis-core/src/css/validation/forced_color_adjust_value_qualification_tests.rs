use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssForcedColorAdjustQualificationOutcome, CssForcedColorAdjustUnsupportedReason,
    CssForcedColorAdjustValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    None,
    PreserveParentColor,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssForcedColorAdjustQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => CssForcedColorAdjustQualificationOutcome::Qualified(
            CssForcedColorAdjustValue::Auto,
        ),
        ExpectedOutcome::None => CssForcedColorAdjustQualificationOutcome::Qualified(
            CssForcedColorAdjustValue::None,
        ),
        ExpectedOutcome::PreserveParentColor => {
            CssForcedColorAdjustQualificationOutcome::Qualified(
                CssForcedColorAdjustValue::PreserveParentColor,
            )
        }
        ExpectedOutcome::Invalid => {
            CssForcedColorAdjustQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssForcedColorAdjustQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssForcedColorAdjustUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssForcedColorAdjustQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssForcedColorAdjustUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .forced_color_adjust_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_forced_color_adjust_grammar() {
    let result = qualify(
        2980,
        concat!(
            "a{forced-color-adjust:auto;}",
            "b{forced-color-adjust:none;}",
            "c{forced-color-adjust:preserve-parent-color;}",
            "d{FORCED-COLOR-ADJUST:PrEsErVe-PaReNt-CoLoR;}",
            r"e{forced-color-adjust:preserve-\70 arent-color;}",
            r"f{forced-\63 olor-adjust:none;}",
            "g{forced-color-adjust:default;}",
            "h{forced-color-adjust:auto none;}",
            "i{forced-color-adjust:;}",
            "j{forced-color-adjust:1;}",
            "k{forced-color-adjust:1px;}",
            "l{forced-color-adjust:\"none\";}",
            "m{color:auto;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::None,
            ExpectedOutcome::PreserveParentColor,
            ExpectedOutcome::PreserveParentColor,
            ExpectedOutcome::PreserveParentColor,
            ExpectedOutcome::None,
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
        2981,
        concat!(
            "a{forced-color-adjust:/**/auto/**/!important;}",
            "b{forced-color-adjust:/**/none/**/!important;}",
            "c{forced-color-adjust:/**/preserve-parent-color/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::None,
            ExpectedOutcome::PreserveParentColor,
        ],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2982,
        concat!(
            "a{forced-color-adjust:initial;}",
            "b{forced-color-adjust:inherit;}",
            "c{forced-color-adjust:unset;}",
            "d{forced-color-adjust:revert;}",
            "e{forced-color-adjust:revert-layer;}",
            "f{forced-color-adjust:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2983,
        concat!(
            "a{forced-color-adjust:var(--adjust);}",
            "b{forced-color-adjust:env(adjust);}",
            "c{forced-color-adjust:attr(data-adjust);}",
            "d{forced-color-adjust:--adjust();}",
            "e{forced-color-adjust:first-valid(auto,none);}",
            "f{forced-color-adjust:cycle(auto,none);}",
            "g{forced-color-adjust:interpolate(0%,0:auto,1:none);}",
            "h{forced-color-adjust:foo();}",
            "i{forced-color-adjust:calc(1);}",
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
        2984,
        concat!(
            "a{forced-color-adjust:auto first-valid(none);}",
            "b{forced-color-adjust:first-valid(none) auto;}",
            "c{forced-color-adjust:none foo();}",
            "d{forced-color-adjust:foo() none;}",
            "e{forced-color-adjust:preserve-parent-color var(--adjust);}",
            "f{forced-color-adjust:foo(var(--adjust));}",
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
fn forced_colors_runtime_state_and_element_kind_are_not_inputs_to_qualification() {
    let result = qualify(
        2985,
        concat!(
            "html{forced-color-adjust:none;}",
            "div{forced-color-adjust:none;}",
            "svg{forced-color-adjust:none;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::None,
            ExpectedOutcome::None,
            ExpectedOutcome::None,
        ],
    );
    assert_eq!(
        result.forced_color_adjust_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.forced_color_adjust_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(
        result.forced_color_adjust_observations()[2].occurrence_index(),
        2
    );
    assert_eq!(
        result.forced_color_adjust_observations()[0].outcome(),
        result.forced_color_adjust_observations()[1].outcome()
    );
    assert_eq!(
        result.forced_color_adjust_observations()[1].outcome(),
        result.forced_color_adjust_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_forced_color_adjust_with_every_accepted_leaf() {
    let result = qualify(
        2986,
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
            "ao{text-anchor:middle;}",
            "ap{forced-color-adjust:preserve-parent-color;}",
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
    assert_eq!(result.text_anchor_observations().len(), 1);
    assert_eq!(result.forced_color_adjust_observations().len(), 1);
    assert_eq!(
        result.forced_color_adjust_observations()[0].occurrence_index(),
        41
    );
    assert_expected(&result, &[ExpectedOutcome::PreserveParentColor]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2987,
        "a{forced-color-adjust:none;}b{forced-color-adjust:none;}",
    );

    assert_expected(&result, &[ExpectedOutcome::None, ExpectedOutcome::None]);
    assert_eq!(
        result.forced_color_adjust_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.forced_color_adjust_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.forced_color_adjust_observations()[0]
            .placement()
            .context_id(),
        result.forced_color_adjust_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2990, "@font-face{forced-color-adjust:none;}"),
        (2991, "@page{forced-color-adjust:none;}"),
        (2992, "@page{@top-left{forced-color-adjust:none;}}"),
        (2993, "@keyframes k{from{forced-color-adjust:none;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.forced_color_adjust_observations().is_empty(),
            "nonordinary declaration context produced a forced-color-adjust observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3000,
        "a{forced-color-adjust:auto;forced-color-adjust:none;}",
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
        "a{forced-color-adjust:auto;}",
        "b{forced-color-adjust:inherit;}",
        "c{forced-color-adjust:default;}",
        "d{forced-color-adjust:var(--adjust);}",
        "e{color:none;}",
    );
    let first = qualify(3010, css);
    let repeated = qualify(3010, css);
    let another_source = qualify(3011, css);

    assert_eq!(
        first.forced_color_adjust_observations(),
        repeated.forced_color_adjust_observations()
    );
    assert_eq!(
        first.forced_color_adjust_observations(),
        another_source.forced_color_adjust_observations()
    );
}
