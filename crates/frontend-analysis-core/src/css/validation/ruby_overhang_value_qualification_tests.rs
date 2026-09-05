use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssRubyOverhangQualificationOutcome, CssRubyOverhangUnsupportedReason, CssRubyOverhangValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Spaces,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssRubyOverhangQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssRubyOverhangQualificationOutcome::Qualified(CssRubyOverhangValue::Auto)
        }
        ExpectedOutcome::Spaces => {
            CssRubyOverhangQualificationOutcome::Qualified(CssRubyOverhangValue::Spaces)
        }
        ExpectedOutcome::Invalid => {
            CssRubyOverhangQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssRubyOverhangQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyOverhangUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssRubyOverhangQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyOverhangUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .ruby_overhang_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_keyword_and_legacy_alias_boundary_matches_pinned_wpt() {
    let result = qualify(
        3180,
        concat!(
            "a{ruby-overhang:auto;}",
            "b{ruby-overhang:spaces;}",
            "c{ruby-overhang:none;}",
            "d{RUBY-OVERHANG:SpAcEs;}",
            r"e{ruby-overhang:\61 uto;}",
            r"f{ruby-\6f verhang:none;}",
            "g{ruby-overhang:simple;}",
            "h{ruby-overhang:auto none;}",
            "i{ruby-overhang:none auto;}",
            "j{ruby-overhang:auto auto;}",
            "k{ruby-overhang:spaces spaces;}",
            "l{ruby-overhang:auto 2px;}",
            "m{ruby-overhang:\"spaces\";}",
            "n{ruby-overhang:;}",
            "o{ruby-overhang:2px;}",
            "p{ruby-merge:merge;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Spaces,
            ExpectedOutcome::Spaces,
            ExpectedOutcome::Spaces,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Spaces,
            ExpectedOutcome::Invalid,
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
    assert_eq!(
        result.ruby_overhang_observations()[1].outcome(),
        result.ruby_overhang_observations()[2].outcome()
    );
}

#[test]
fn comments_and_priority_preserve_keyword_and_alias_meaning() {
    let result = qualify(
        3181,
        concat!(
            "a{ruby-overhang:/**/spaces/**/!important;}",
            "b{ruby-overhang:/**/none/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Spaces, ExpectedOutcome::Spaces]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3182,
        concat!(
            "a{ruby-overhang:initial;}",
            "b{ruby-overhang:inherit;}",
            "c{ruby-overhang:unset;}",
            "d{ruby-overhang:revert;}",
            "e{ruby-overhang:revert-layer;}",
            "f{ruby-overhang:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3183,
        concat!(
            "a{ruby-overhang:var(--style);}",
            "b{ruby-overhang:env(style);}",
            "c{ruby-overhang:attr(data-style);}",
            "d{ruby-overhang:--style();}",
            "e{ruby-overhang:first-valid(auto,spaces);}",
            "f{ruby-overhang:cycle(auto,spaces);}",
            "g{ruby-overhang:interpolate(0%,0:auto,1:spaces);}",
            "h{ruby-overhang:foo();}",
            "i{ruby-overhang:calc(1);}",
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
        3184,
        concat!(
            "a{ruby-overhang:auto first-valid(spaces);}",
            "b{ruby-overhang:first-valid(auto) spaces;}",
            "c{ruby-overhang:auto foo();}",
            "d{ruby-overhang:foo() spaces;}",
            "e{ruby-overhang:spaces var(--style);}",
            "f{ruby-overhang:foo(var(--style));}",
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
fn element_kind_and_ruby_applicability_are_not_inputs_to_qualification() {
    let result = qualify(
        3185,
        concat!(
            "rt{ruby-overhang:spaces;}",
            "div{ruby-overhang:spaces;}",
            "svg{ruby-overhang:spaces;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Spaces,
            ExpectedOutcome::Spaces,
            ExpectedOutcome::Spaces,
        ],
    );
    assert_eq!(result.ruby_overhang_observations()[0].occurrence_index(), 0);
    assert_eq!(result.ruby_overhang_observations()[1].occurrence_index(), 1);
    assert_eq!(result.ruby_overhang_observations()[2].occurrence_index(), 2);
}

#[test]
fn one_run_interleaves_ruby_overhang_with_every_accepted_leaf() {
    let result = qualify(
        3186,
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
    assert_eq!(
        result.ruby_overhang_observations()[0].occurrence_index(),
        46
    );
    assert_expected(&result, &[ExpectedOutcome::Spaces]);
}

#[test]
fn duplicate_aliases_keep_distinct_run_local_placement() {
    let result = qualify(3187, "a{ruby-overhang:none;}b{ruby-overhang:spaces;}");

    assert_expected(&result, &[ExpectedOutcome::Spaces, ExpectedOutcome::Spaces]);
    assert_eq!(result.ruby_overhang_observations()[0].occurrence_index(), 0);
    assert_eq!(result.ruby_overhang_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.ruby_overhang_observations()[0]
            .placement()
            .context_id(),
        result.ruby_overhang_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3190, "@font-face{ruby-overhang:spaces;}"),
        (3191, "@page{ruby-overhang:spaces;}"),
        (3192, "@page{@top-left{ruby-overhang:spaces;}}"),
        (3193, "@keyframes k{from{ruby-overhang:spaces;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.ruby_overhang_observations().is_empty(),
            "nonordinary declaration context produced a ruby-overhang observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3200,
        "a{ruby-overhang:none;ruby-overhang:auto;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Spaces]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{ruby-overhang:auto;}",
        "b{ruby-overhang:none;}",
        "c{ruby-overhang:inherit;}",
        "d{ruby-overhang:simple;}",
        "e{ruby-overhang:var(--style);}",
        "f{ruby-merge:merge;}",
    );
    let first = qualify(3210, css);
    let repeated = qualify(3210, css);
    let another_source = qualify(3211, css);

    assert_eq!(
        first.ruby_overhang_observations(),
        repeated.ruby_overhang_observations()
    );
    assert_eq!(
        first.ruby_overhang_observations(),
        another_source.ruby_overhang_observations()
    );
}
