use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFillRuleQualificationOutcome, CssFillRuleUnsupportedReason, CssFillRuleValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Nonzero,
    Evenodd,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFillRuleQualificationOutcome {
    match expected {
        ExpectedOutcome::Nonzero => {
            CssFillRuleQualificationOutcome::Qualified(CssFillRuleValue::Nonzero)
        }
        ExpectedOutcome::Evenodd => {
            CssFillRuleQualificationOutcome::Qualified(CssFillRuleValue::Evenodd)
        }
        ExpectedOutcome::Invalid => CssFillRuleQualificationOutcome::InvalidForSelectedValueGrammar,
        ExpectedOutcome::UnsupportedCssWide => {
            CssFillRuleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillRuleUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFillRuleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillRuleUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .fill_rule_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_pinned_wpt() {
    let result = qualify(
        3260,
        concat!(
            "a{fill-rule:nonzero;}",
            "b{fill-rule:evenodd;}",
            "c{FILL-RULE:EvEnOdD;}",
            r"d{fill-rule:\6e onzero;}",
            r"e{fill-\72 ule:evenodd;}",
            "f{fill-rule:auto;}",
            "g{fill-rule:nonzero evenodd;}",
            "h{fill-rule:1;}",
            "i{fill-rule:1px;}",
            "j{fill-rule:\"evenodd\";}",
            "k{fill-rule:;}",
            "l{clip-rule:evenodd;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Nonzero,
            ExpectedOutcome::Evenodd,
            ExpectedOutcome::Evenodd,
            ExpectedOutcome::Nonzero,
            ExpectedOutcome::Evenodd,
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
        3261,
        concat!(
            "a{fill-rule:/**/nonzero/**/!important;}",
            "b{fill-rule:/**/evenodd/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Nonzero, ExpectedOutcome::Evenodd],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3262,
        concat!(
            "a{fill-rule:initial;}",
            "b{fill-rule:inherit;}",
            "c{fill-rule:unset;}",
            "d{fill-rule:revert;}",
            "e{fill-rule:revert-layer;}",
            "f{fill-rule:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3263,
        concat!(
            "a{fill-rule:var(--rule);}",
            "b{fill-rule:env(rule);}",
            "c{fill-rule:attr(data-rule);}",
            "d{fill-rule:--rule();}",
            "e{fill-rule:first-valid(nonzero,evenodd);}",
            "f{fill-rule:cycle(nonzero,evenodd);}",
            "g{fill-rule:interpolate(0%,0:nonzero,1:evenodd);}",
            "h{fill-rule:foo();}",
            "i{fill-rule:calc(1);}",
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
        3264,
        concat!(
            "a{fill-rule:nonzero first-valid(evenodd);}",
            "b{fill-rule:first-valid(nonzero) evenodd;}",
            "c{fill-rule:nonzero foo();}",
            "d{fill-rule:foo() evenodd;}",
            "e{fill-rule:evenodd var(--rule);}",
            "f{fill-rule:foo(var(--rule));}",
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
fn element_kind_and_svg_applicability_are_not_inputs_to_qualification() {
    let result = qualify(
        3265,
        concat!(
            "path{fill-rule:evenodd;}",
            "div{fill-rule:evenodd;}",
            "svg{fill-rule:evenodd;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Evenodd,
            ExpectedOutcome::Evenodd,
            ExpectedOutcome::Evenodd,
        ],
    );
    assert_eq!(result.fill_rule_observations()[0].occurrence_index(), 0);
    assert_eq!(result.fill_rule_observations()[1].occurrence_index(), 1);
    assert_eq!(result.fill_rule_observations()[2].occurrence_index(), 2);
}

#[test]
fn one_run_interleaves_fill_rule_with_every_accepted_leaf() {
    let result = qualify(
        3266,
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
            "at{ruby-merge:merge;}",
            "au{ruby-overhang:spaces;}",
            "av{clip-rule:evenodd;}",
            "aw{fill-rule:evenodd;}",
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
    assert_eq!(result.ruby_merge_observations().len(), 1);
    assert_eq!(result.ruby_overhang_observations().len(), 1);
    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.fill_rule_observations().len(), 1);
    assert_eq!(result.fill_rule_observations()[0].occurrence_index(), 48);
    assert_expected(&result, &[ExpectedOutcome::Evenodd]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(3267, "a{fill-rule:nonzero;}b{fill-rule:nonzero;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::Nonzero, ExpectedOutcome::Nonzero],
    );
    assert_eq!(result.fill_rule_observations()[0].occurrence_index(), 0);
    assert_eq!(result.fill_rule_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.fill_rule_observations()[0].placement().context_id(),
        result.fill_rule_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3270, "@font-face{fill-rule:evenodd;}"),
        (3271, "@page{fill-rule:evenodd;}"),
        (3272, "@page{@top-left{fill-rule:evenodd;}}"),
        (3273, "@keyframes k{from{fill-rule:evenodd;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.fill_rule_observations().is_empty(),
            "nonordinary declaration context produced a fill-rule observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3280,
        "a{fill-rule:nonzero;fill-rule:evenodd;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Nonzero]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{fill-rule:nonzero;}",
        "b{fill-rule:inherit;}",
        "c{fill-rule:auto;}",
        "d{fill-rule:var(--rule);}",
        "e{clip-rule:evenodd;}",
    );
    let first = qualify(3290, css);
    let repeated = qualify(3290, css);
    let another_source = qualify(3291, css);

    assert_eq!(
        first.fill_rule_observations(),
        repeated.fill_rule_observations()
    );
    assert_eq!(
        first.fill_rule_observations(),
        another_source.fill_rule_observations()
    );
}
