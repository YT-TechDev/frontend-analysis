use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssMathStyleQualificationOutcome, CssMathStyleUnsupportedReason, CssMathStyleValue,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssMathStyleQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssMathStyleQualificationOutcome::Qualified(CssMathStyleValue::Normal)
        }
        ExpectedOutcome::Compact => {
            CssMathStyleQualificationOutcome::Qualified(CssMathStyleValue::Compact)
        }
        ExpectedOutcome::Invalid => {
            CssMathStyleQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssMathStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMathStyleUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssMathStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMathStyleUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .math_style_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_math_style_grammar() {
    let result = qualify(
        3060,
        concat!(
            "a{math-style:normal;}",
            "b{math-style:compact;}",
            "c{MATH-STYLE:CoMpAcT;}",
            r"d{math-style:\63 ompact;}",
            r"e{math-\73 tyle:normal;}",
            "f{math-style:auto;}",
            "g{math-style:normal compact;}",
            "h{math-style:;}",
            "i{math-style:1;}",
            "j{math-style:1px;}",
            "k{math-style:\"normal\";}",
            "l{math-shift:compact;}",
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
        3061,
        concat!(
            "a{math-style:/**/normal/**/!important;}",
            "b{math-style:/**/compact/**/!important;}",
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
        3062,
        concat!(
            "a{math-style:initial;}",
            "b{math-style:inherit;}",
            "c{math-style:unset;}",
            "d{math-style:revert;}",
            "e{math-style:revert-layer;}",
            "f{math-style:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3063,
        concat!(
            "a{math-style:var(--style);}",
            "b{math-style:env(style);}",
            "c{math-style:attr(data-style);}",
            "d{math-style:--style();}",
            "e{math-style:first-valid(normal,compact);}",
            "f{math-style:cycle(normal,compact);}",
            "g{math-style:interpolate(0%,0:normal,1:compact);}",
            "h{math-style:foo();}",
            "i{math-style:calc(1);}",
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
        3064,
        concat!(
            "a{math-style:normal first-valid(compact);}",
            "b{math-style:first-valid(normal) compact;}",
            "c{math-style:normal foo();}",
            "d{math-style:foo() compact;}",
            "e{math-style:compact var(--style);}",
            "f{math-style:foo(var(--style));}",
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
        3065,
        concat!(
            "math[displaystyle=true]{math-style:compact;}",
            "div{math-style:compact;}",
            "svg{math-style:compact;}",
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
    assert_eq!(result.math_style_observations()[0].occurrence_index(), 0);
    assert_eq!(result.math_style_observations()[1].occurrence_index(), 1);
    assert_eq!(result.math_style_observations()[2].occurrence_index(), 2);
    assert_eq!(
        result.math_style_observations()[0].outcome(),
        result.math_style_observations()[1].outcome()
    );
    assert_eq!(
        result.math_style_observations()[1].outcome(),
        result.math_style_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_math_style_with_every_accepted_leaf() {
    let result = qualify(
        3066,
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
    assert_eq!(result.math_style_observations()[0].occurrence_index(), 43);
    assert_expected(&result, &[ExpectedOutcome::Compact]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(3067, "a{math-style:normal;}b{math-style:normal;}");

    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Normal]);
    assert_eq!(result.math_style_observations()[0].occurrence_index(), 0);
    assert_eq!(result.math_style_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.math_style_observations()[0].placement().context_id(),
        result.math_style_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3070, "@font-face{math-style:compact;}"),
        (3071, "@page{math-style:compact;}"),
        (3072, "@page{@top-left{math-style:compact;}}"),
        (3073, "@keyframes k{from{math-style:compact;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.math_style_observations().is_empty(),
            "nonordinary declaration context produced a math-style observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3080,
        "a{math-style:normal;math-style:compact;}",
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
        "a{math-style:normal;}",
        "b{math-style:inherit;}",
        "c{math-style:auto;}",
        "d{math-style:var(--style);}",
        "e{math-shift:compact;}",
    );
    let first = qualify(3090, css);
    let repeated = qualify(3090, css);
    let another_source = qualify(3091, css);

    assert_eq!(
        first.math_style_observations(),
        repeated.math_style_observations()
    );
    assert_eq!(
        first.math_style_observations(),
        another_source.math_style_observations()
    );
}
