use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssMaskTypeQualificationOutcome, CssMaskTypeUnsupportedReason, CssMaskTypeValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Luminance,
    Alpha,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssMaskTypeQualificationOutcome {
    match expected {
        ExpectedOutcome::Luminance => {
            CssMaskTypeQualificationOutcome::Qualified(CssMaskTypeValue::Luminance)
        }
        ExpectedOutcome::Alpha => {
            CssMaskTypeQualificationOutcome::Qualified(CssMaskTypeValue::Alpha)
        }
        ExpectedOutcome::Invalid => {
            CssMaskTypeQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssMaskTypeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMaskTypeUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssMaskTypeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMaskTypeUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .mask_type_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_mask_type_grammar() {
    let result = qualify(
        2780,
        concat!(
            "a{mask-type:luminance;}",
            "b{mask-type:alpha;}",
            "c{MASK-TYPE:LuMiNaNcE;}",
            r"d{mask-type:\61 lpha;}",
            r"e{mask-\74 ype:luminance;}",
            "f{mask-type:auto;}",
            "g{mask-type:luminance alpha;}",
            "h{mask-type:alpha, luminance;}",
            "i{mask-type:;}",
            "j{mask-type:1;}",
            "k{mask-type:1px;}",
            "l{mask-type:\"alpha\";}",
            "m{color:alpha;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Luminance,
            ExpectedOutcome::Alpha,
            ExpectedOutcome::Luminance,
            ExpectedOutcome::Alpha,
            ExpectedOutcome::Luminance,
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
        2781,
        concat!(
            "a{mask-type:/**/luminance/**/!important;}",
            "b{mask-type:/**/alpha/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Luminance, ExpectedOutcome::Alpha],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2782,
        concat!(
            "a{mask-type:initial;}",
            "b{mask-type:inherit;}",
            "c{mask-type:unset;}",
            "d{mask-type:revert;}",
            "e{mask-type:revert-layer;}",
            "f{mask-type:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2783,
        concat!(
            "a{mask-type:var(--mask-type);}",
            "b{mask-type:env(mask-type);}",
            "c{mask-type:attr(data-mask-type);}",
            "d{mask-type:--mask-type();}",
            "e{mask-type:first-valid(luminance,alpha);}",
            "f{mask-type:cycle(luminance,alpha);}",
            "g{mask-type:interpolate(0%,0:luminance,1:alpha);}",
            "h{mask-type:foo();}",
            "i{mask-type:calc(1);}",
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
        2784,
        concat!(
            "a{mask-type:luminance first-valid(alpha);}",
            "b{mask-type:first-valid(alpha) luminance;}",
            "c{mask-type:alpha foo();}",
            "d{mask-type:foo() alpha;}",
            "e{mask-type:luminance var(--mask-type);}",
            "f{mask-type:foo(var(--mask-type));}",
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
fn element_applicability_is_not_an_input_to_mask_type_qualification() {
    let result = qualify(
        2785,
        concat!(
            "mask{mask-type:luminance;}",
            "div{mask-type:luminance;}",
            "svg mask{mask-type:luminance;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Luminance,
            ExpectedOutcome::Luminance,
            ExpectedOutcome::Luminance,
        ],
    );
    assert_eq!(result.mask_type_observations()[0].occurrence_index(), 0);
    assert_eq!(result.mask_type_observations()[1].occurrence_index(), 1);
    assert_eq!(result.mask_type_observations()[2].occurrence_index(), 2);
    assert_eq!(
        result.mask_type_observations()[0].outcome(),
        result.mask_type_observations()[1].outcome()
    );
    assert_eq!(
        result.mask_type_observations()[1].outcome(),
        result.mask_type_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_mask_type_with_every_accepted_leaf() {
    let result = qualify(
        2786,
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
    assert_eq!(result.mask_type_observations()[0].occurrence_index(), 36);
    assert_expected(&result, &[ExpectedOutcome::Alpha]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(2787, "a{mask-type:alpha;}b{mask-type:alpha;}");

    assert_expected(&result, &[ExpectedOutcome::Alpha, ExpectedOutcome::Alpha]);
    assert_eq!(result.mask_type_observations()[0].occurrence_index(), 0);
    assert_eq!(result.mask_type_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.mask_type_observations()[0].placement().context_id(),
        result.mask_type_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2790, "@font-face{mask-type:alpha;}"),
        (2791, "@page{mask-type:alpha;}"),
        (2792, "@page{@top-left{mask-type:alpha;}}"),
        (2793, "@keyframes k{from{mask-type:alpha;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.mask_type_observations().is_empty(),
            "nonordinary declaration context produced a mask-type observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2800,
        "a{mask-type:luminance;mask-type:alpha;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Luminance]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{mask-type:luminance;}",
        "b{mask-type:inherit;}",
        "c{mask-type:auto;}",
        "d{mask-type:var(--mask-type);}",
        "e{color:alpha;}",
    );
    let first = qualify(2810, css);
    let repeated = qualify(2810, css);
    let another_source = qualify(2811, css);

    assert_eq!(
        first.mask_type_observations(),
        repeated.mask_type_observations()
    );
    assert_eq!(
        first.mask_type_observations(),
        another_source.mask_type_observations()
    );
}
