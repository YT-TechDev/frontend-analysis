use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextAlignLastQualificationOutcome, CssTextAlignLastUnsupportedReason, CssTextAlignLastValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
    MatchParent,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssTextAlignLastQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Auto)
        }
        ExpectedOutcome::Start => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Start)
        }
        ExpectedOutcome::End => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::End)
        }
        ExpectedOutcome::Left => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Left)
        }
        ExpectedOutcome::Right => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Right)
        }
        ExpectedOutcome::Center => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Center)
        }
        ExpectedOutcome::Justify => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Justify)
        }
        ExpectedOutcome::MatchParent => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::MatchParent)
        }
        ExpectedOutcome::Invalid => {
            CssTextAlignLastQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssTextAlignLastQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextAlignLastUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssTextAlignLastQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextAlignLastUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .text_align_last_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_text_align_last_grammar() {
    let result = qualify(
        3020,
        concat!(
            "a{text-align-last:auto;}",
            "b{text-align-last:start;}",
            "c{text-align-last:end;}",
            "d{text-align-last:left;}",
            "e{text-align-last:right;}",
            "f{text-align-last:center;}",
            "g{text-align-last:justify;}",
            "h{text-align-last:match-parent;}",
            "i{TEXT-ALIGN-LAST:MaTcH-PaReNt;}",
            r"j{text-align-last:match-\70 arent;}",
            r"k{text-align-\6c ast:center;}",
            "l{text-align-last:none;}",
            "m{text-align-last:auto start;}",
            "n{text-align-last:;}",
            "o{text-align-last:1;}",
            "p{text-align-last:1px;}",
            "q{text-align-last:\"left\";}",
            "r{text-align:auto;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Start,
            ExpectedOutcome::End,
            ExpectedOutcome::Left,
            ExpectedOutcome::Right,
            ExpectedOutcome::Center,
            ExpectedOutcome::Justify,
            ExpectedOutcome::MatchParent,
            ExpectedOutcome::MatchParent,
            ExpectedOutcome::MatchParent,
            ExpectedOutcome::Center,
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
        3021,
        concat!(
            "a{text-align-last:/**/auto/**/!important;}",
            "b{text-align-last:/**/justify/**/!important;}",
            "c{text-align-last:/**/match-parent/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Justify,
            ExpectedOutcome::MatchParent,
        ],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3022,
        concat!(
            "a{text-align-last:initial;}",
            "b{text-align-last:inherit;}",
            "c{text-align-last:unset;}",
            "d{text-align-last:revert;}",
            "e{text-align-last:revert-layer;}",
            "f{text-align-last:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3023,
        concat!(
            "a{text-align-last:var(--align);}",
            "b{text-align-last:env(align);}",
            "c{text-align-last:attr(data-align);}",
            "d{text-align-last:--align();}",
            "e{text-align-last:first-valid(start,end);}",
            "f{text-align-last:cycle(start,end);}",
            "g{text-align-last:interpolate(0%,0:start,1:end);}",
            "h{text-align-last:foo();}",
            "i{text-align-last:calc(1);}",
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
        3024,
        concat!(
            "a{text-align-last:auto first-valid(start);}",
            "b{text-align-last:first-valid(start) auto;}",
            "c{text-align-last:start foo();}",
            "d{text-align-last:foo() start;}",
            "e{text-align-last:match-parent var(--align);}",
            "f{text-align-last:foo(var(--align));}",
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
fn direction_layout_state_and_element_kind_are_not_inputs_to_qualification() {
    let result = qualify(
        3025,
        concat!(
            "html[dir=ltr]{text-align-last:end;}",
            "html[dir=rtl]{text-align-last:end;}",
            "svg{text-align-last:end;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::End,
            ExpectedOutcome::End,
            ExpectedOutcome::End,
        ],
    );
    assert_eq!(
        result.text_align_last_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.text_align_last_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(
        result.text_align_last_observations()[2].occurrence_index(),
        2
    );
    assert_eq!(
        result.text_align_last_observations()[0].outcome(),
        result.text_align_last_observations()[1].outcome()
    );
    assert_eq!(
        result.text_align_last_observations()[1].outcome(),
        result.text_align_last_observations()[2].outcome()
    );
}

#[test]
fn one_run_interleaves_text_align_last_with_every_accepted_leaf() {
    let result = qualify(
        3026,
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
    assert_eq!(
        result.text_align_last_observations()[0].occurrence_index(),
        42
    );
    assert_expected(&result, &[ExpectedOutcome::MatchParent]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(3027, "a{text-align-last:end;}b{text-align-last:end;}");

    assert_expected(&result, &[ExpectedOutcome::End, ExpectedOutcome::End]);
    assert_eq!(
        result.text_align_last_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.text_align_last_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.text_align_last_observations()[0]
            .placement()
            .context_id(),
        result.text_align_last_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3030, "@font-face{text-align-last:end;}"),
        (3031, "@page{text-align-last:end;}"),
        (3032, "@page{@top-left{text-align-last:end;}}"),
        (3033, "@keyframes k{from{text-align-last:end;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_align_last_observations().is_empty(),
            "nonordinary declaration context produced a text-align-last observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3040,
        "a{text-align-last:auto;text-align-last:justify;}",
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
        "a{text-align-last:auto;}",
        "b{text-align-last:inherit;}",
        "c{text-align-last:none;}",
        "d{text-align-last:var(--align);}",
        "e{text-align:center;}",
    );
    let first = qualify(3050, css);
    let repeated = qualify(3050, css);
    let another_source = qualify(3051, css);

    assert_eq!(
        first.text_align_last_observations(),
        repeated.text_align_last_observations()
    );
    assert_eq!(
        first.text_align_last_observations(),
        another_source.text_align_last_observations()
    );
}