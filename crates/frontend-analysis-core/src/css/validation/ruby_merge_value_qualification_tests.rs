use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssRubyMergeQualificationOutcome, CssRubyMergeUnsupportedReason, CssRubyMergeValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Separate,
    Merge,
    Auto,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssRubyMergeQualificationOutcome {
    match expected {
        ExpectedOutcome::Separate => {
            CssRubyMergeQualificationOutcome::Qualified(CssRubyMergeValue::Separate)
        }
        ExpectedOutcome::Merge => {
            CssRubyMergeQualificationOutcome::Qualified(CssRubyMergeValue::Merge)
        }
        ExpectedOutcome::Auto => {
            CssRubyMergeQualificationOutcome::Qualified(CssRubyMergeValue::Auto)
        }
        ExpectedOutcome::Invalid => {
            CssRubyMergeQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssRubyMergeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyMergeUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssRubyMergeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyMergeUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .ruby_merge_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_ruby_merge_grammar() {
    let result = qualify(
        3140,
        concat!(
            "a{ruby-merge:separate;}",
            "b{ruby-merge:merge;}",
            "c{ruby-merge:auto;}",
            "d{RUBY-MERGE:MeRgE;}",
            r"e{ruby-merge:\73 eparate;}",
            r"f{ruby-\6d erge:auto;}",
            "g{ruby-merge:none;}",
            "h{ruby-merge:collapse;}",
            "i{ruby-merge:merge separate;}",
            "j{ruby-merge:;}",
            "k{ruby-merge:10px;}",
            "l{ruby-merge:\"merge\";}",
            "m{math-shift:compact;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Separate,
            ExpectedOutcome::Merge,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Merge,
            ExpectedOutcome::Separate,
            ExpectedOutcome::Auto,
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
        3141,
        concat!(
            "a{ruby-merge:/**/merge/**/!important;}",
            "b{ruby-merge:/**/auto/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Merge, ExpectedOutcome::Auto]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3142,
        concat!(
            "a{ruby-merge:initial;}",
            "b{ruby-merge:inherit;}",
            "c{ruby-merge:unset;}",
            "d{ruby-merge:revert;}",
            "e{ruby-merge:revert-layer;}",
            "f{ruby-merge:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3143,
        concat!(
            "a{ruby-merge:var(--style);}",
            "b{ruby-merge:env(style);}",
            "c{ruby-merge:attr(data-style);}",
            "d{ruby-merge:--style();}",
            "e{ruby-merge:first-valid(separate,merge);}",
            "f{ruby-merge:cycle(separate,merge);}",
            "g{ruby-merge:interpolate(0%,0:separate,1:merge);}",
            "h{ruby-merge:foo();}",
            "i{ruby-merge:calc(1);}",
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
        3144,
        concat!(
            "a{ruby-merge:separate first-valid(merge);}",
            "b{ruby-merge:first-valid(separate) merge;}",
            "c{ruby-merge:separate foo();}",
            "d{ruby-merge:foo() merge;}",
            "e{ruby-merge:merge var(--style);}",
            "f{ruby-merge:foo(var(--style));}",
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
        3145,
        concat!(
            "ruby{ruby-merge:merge;}",
            "div{ruby-merge:merge;}",
            "svg{ruby-merge:merge;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Merge,
            ExpectedOutcome::Merge,
            ExpectedOutcome::Merge,
        ],
    );
    assert_eq!(result.ruby_merge_observations()[0].occurrence_index(), 0);
    assert_eq!(result.ruby_merge_observations()[1].occurrence_index(), 1);
    assert_eq!(result.ruby_merge_observations()[2].occurrence_index(), 2);
    assert_eq!(
        result.ruby_merge_observations()[0].outcome(),
        result.ruby_merge_observations()[1].outcome()
    );
    assert_eq!(
        result.ruby_merge_observations()[1].outcome(),
        result.ruby_merge_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_ruby_merge_with_every_accepted_leaf() {
    let result = qualify(
        3146,
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
    assert_eq!(result.ruby_merge_observations()[0].occurrence_index(), 45);
    assert_expected(&result, &[ExpectedOutcome::Merge]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(3147, "a{ruby-merge:separate;}b{ruby-merge:separate;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::Separate, ExpectedOutcome::Separate],
    );
    assert_eq!(result.ruby_merge_observations()[0].occurrence_index(), 0);
    assert_eq!(result.ruby_merge_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.ruby_merge_observations()[0].placement().context_id(),
        result.ruby_merge_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3150, "@font-face{ruby-merge:merge;}"),
        (3151, "@page{ruby-merge:merge;}"),
        (3152, "@page{@top-left{ruby-merge:merge;}}"),
        (3153, "@keyframes k{from{ruby-merge:merge;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.ruby_merge_observations().is_empty(),
            "nonordinary declaration context produced a ruby-merge observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3160,
        "a{ruby-merge:separate;ruby-merge:merge;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Separate]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{ruby-merge:separate;}",
        "b{ruby-merge:inherit;}",
        "c{ruby-merge:collapse;}",
        "d{ruby-merge:var(--style);}",
        "e{math-shift:compact;}",
    );
    let first = qualify(3170, css);
    let repeated = qualify(3170, css);
    let another_source = qualify(3171, css);

    assert_eq!(
        first.ruby_merge_observations(),
        repeated.ruby_merge_observations()
    );
    assert_eq!(
        first.ruby_merge_observations(),
        another_source.ruby_merge_observations()
    );
}
