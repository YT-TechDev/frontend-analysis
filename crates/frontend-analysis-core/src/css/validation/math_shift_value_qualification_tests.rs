use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssMathShiftQualificationOutcome, CssMathShiftUnsupportedReason, CssMathShiftValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Compact,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssMathShiftQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssMathShiftQualificationOutcome::Qualified(CssMathShiftValue::Normal)
        }
        ExpectedOutcome::Compact => {
            CssMathShiftQualificationOutcome::Qualified(CssMathShiftValue::Compact)
        }
        ExpectedOutcome::Invalid => {
            CssMathShiftQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssMathShiftQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMathShiftUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssMathShiftQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMathShiftUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .math_shift_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_math_shift_grammar() {
    let result = qualify(
        3100,
        concat!(
            "a{math-shift:normal;}",
            "b{math-shift:compact;}",
            "c{MATH-SHIFT:CoMpAcT;}",
            r"d{math-shift:\63 ompact;}",
            r"e{math-\73 hift:normal;}",
            "f{math-shift:auto;}",
            "g{math-shift:normal compact;}",
            "h{math-shift:;}",
            "i{math-shift:1;}",
            "j{math-shift:1px;}",
            "k{math-shift:\"normal\";}",
            "l{math-style:compact;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Compact,
            ExpectedOutcome::Compact,
            ExpectedOutcome::Compact,
            ExpectedOutcome::Normal,
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
        3101,
        concat!(
            "a{math-shift:/**/normal/**/!important;}",
            "b{math-shift:/**/compact/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Normal, ExpectedOutcome::Compact],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3102,
        concat!(
            "a{math-shift:initial;}",
            "b{math-shift:inherit;}",
            "c{math-shift:unset;}",
            "d{math-shift:revert;}",
            "e{math-shift:revert-layer;}",
            "f{math-shift:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3103,
        concat!(
            "a{math-shift:var(--style);}",
            "b{math-shift:env(style);}",
            "c{math-shift:attr(data-style);}",
            "d{math-shift:--style();}",
            "e{math-shift:first-valid(normal,compact);}",
            "f{math-shift:cycle(normal,compact);}",
            "g{math-shift:interpolate(0%,0:normal,1:compact);}",
            "h{math-shift:foo();}",
            "i{math-shift:calc(1);}",
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
        3104,
        concat!(
            "a{math-shift:normal first-valid(compact);}",
            "b{math-shift:first-valid(normal) compact;}",
            "c{math-shift:normal foo();}",
            "d{math-shift:foo() compact;}",
            "e{math-shift:compact var(--style);}",
            "f{math-shift:foo(var(--style));}",
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
fn element_kind_and_mathml_presentation_hints_are_not_inputs_to_qualification() {
    let result = qualify(
        3105,
        concat!(
            "math[displaystyle=true]{math-shift:compact;}",
            "div{math-shift:compact;}",
            "svg{math-shift:compact;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Compact,
            ExpectedOutcome::Compact,
            ExpectedOutcome::Compact,
        ],
    );
    assert_eq!(result.math_shift_observations()[0].occurrence_index(), 0);
    assert_eq!(result.math_shift_observations()[1].occurrence_index(), 1);
    assert_eq!(result.math_shift_observations()[2].occurrence_index(), 2);
    assert_eq!(
        result.math_shift_observations()[0].outcome(),
        result.math_shift_observations()[1].outcome()
    );
    assert_eq!(
        result.math_shift_observations()[1].outcome(),
        result.math_shift_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_math_shift_with_every_accepted_leaf() {
    let result = qualify(
        3106,
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
            "aq{text-align-last:match-parent;}",
            "ar{math-style:compact;}",
            "as{math-shift:compact;}",
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
    assert_eq!(result.text_align_last_observations().len(), 1);
    assert_eq!(result.math_style_observations().len(), 1);
    assert_eq!(result.math_shift_observations().len(), 1);
    assert_eq!(result.math_shift_observations()[0].occurrence_index(), 44);
    assert_expected(&result, &[ExpectedOutcome::Compact]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(3107, "a{math-shift:normal;}b{math-shift:normal;}");

    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Normal]);
    assert_eq!(result.math_shift_observations()[0].occurrence_index(), 0);
    assert_eq!(result.math_shift_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.math_shift_observations()[0].placement().context_id(),
        result.math_shift_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3110, "@font-face{math-shift:compact;}"),
        (3111, "@page{math-shift:compact;}"),
        (3112, "@page{@top-left{math-shift:compact;}}"),
        (3113, "@keyframes k{from{math-shift:compact;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.math_shift_observations().is_empty(),
            "nonordinary declaration context produced a math-shift observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3120,
        "a{math-shift:normal;math-shift:compact;}",
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
        "a{math-shift:normal;}",
        "b{math-shift:inherit;}",
        "c{math-shift:auto;}",
        "d{math-shift:var(--style);}",
        "e{math-style:compact;}",
    );
    let first = qualify(3130, css);
    let repeated = qualify(3130, css);
    let another_source = qualify(3131, css);

    assert_eq!(
        first.math_shift_observations(),
        repeated.math_shift_observations()
    );
    assert_eq!(
        first.math_shift_observations(),
        another_source.math_shift_observations()
    );
}
