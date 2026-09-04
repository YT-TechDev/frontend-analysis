use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssUnicodeBidiQualificationOutcome, CssUnicodeBidiUnsupportedReason, CssUnicodeBidiValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Embed,
    Isolate,
    BidiOverride,
    IsolateOverride,
    Plaintext,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssUnicodeBidiQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::Normal)
        }
        ExpectedOutcome::Embed => {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::Embed)
        }
        ExpectedOutcome::Isolate => {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::Isolate)
        }
        ExpectedOutcome::BidiOverride => {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::BidiOverride)
        }
        ExpectedOutcome::IsolateOverride => {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::IsolateOverride)
        }
        ExpectedOutcome::Plaintext => {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::Plaintext)
        }
        ExpectedOutcome::Invalid => {
            CssUnicodeBidiQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssUnicodeBidiQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssUnicodeBidiUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssUnicodeBidiQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssUnicodeBidiUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .unicode_bidi_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_unicode_bidi_grammar() {
    let result = qualify(
        2740,
        concat!(
            "a{unicode-bidi:normal;}",
            "b{unicode-bidi:embed;}",
            "c{unicode-bidi:isolate;}",
            "d{unicode-bidi:bidi-override;}",
            "e{unicode-bidi:isolate-override;}",
            "f{unicode-bidi:plaintext;}",
            "g{UNICODE-BIDI:PlAiNtExT;}",
            r"h{unicode-bidi:\70 laintext;}",
            r"i{unicode-\62 idi:isolate-override;}",
            "j{unicode-bidi:auto;}",
            "k{unicode-bidi:isolate plaintext;}",
            "l{unicode-bidi:plaintext plaintext;}",
            "m{unicode-bidi:;}",
            "n{unicode-bidi:1;}",
            "o{unicode-bidi:1px;}",
            "p{unicode-bidi:\"isolate\";}",
            "q{direction:rtl;}",
            "r{color:plaintext;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Embed,
            ExpectedOutcome::Isolate,
            ExpectedOutcome::BidiOverride,
            ExpectedOutcome::IsolateOverride,
            ExpectedOutcome::Plaintext,
            ExpectedOutcome::Plaintext,
            ExpectedOutcome::Plaintext,
            ExpectedOutcome::IsolateOverride,
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
        2741,
        concat!(
            "a{unicode-bidi:/**/normal/**/!important;}",
            "b{unicode-bidi:/**/isolate-override/**/!important;}",
            "c{unicode-bidi:/**/plaintext/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::IsolateOverride,
            ExpectedOutcome::Plaintext,
        ],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2742,
        concat!(
            "a{unicode-bidi:initial;}",
            "b{unicode-bidi:inherit;}",
            "c{unicode-bidi:unset;}",
            "d{unicode-bidi:revert;}",
            "e{unicode-bidi:revert-layer;}",
            "f{unicode-bidi:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2743,
        concat!(
            "a{unicode-bidi:var(--bidi);}",
            "b{unicode-bidi:env(bidi);}",
            "c{unicode-bidi:attr(data-bidi);}",
            "d{unicode-bidi:--bidi();}",
            "e{unicode-bidi:first-valid(normal,isolate);}",
            "f{unicode-bidi:cycle(normal,plaintext);}",
            "g{unicode-bidi:interpolate(0%,0:normal,1:plaintext);}",
            "h{unicode-bidi:foo();}",
            "i{unicode-bidi:calc(1);}",
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
        2744,
        concat!(
            "a{unicode-bidi:normal first-valid(isolate);}",
            "b{unicode-bidi:first-valid(isolate) normal;}",
            "c{unicode-bidi:plaintext foo();}",
            "d{unicode-bidi:foo() plaintext;}",
            "e{unicode-bidi:normal var(--bidi);}",
            "f{unicode-bidi:foo(var(--bidi));}",
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
fn direction_is_not_an_input_to_unicode_bidi_qualification() {
    let result = qualify(
        2745,
        concat!(
            "a{direction:rtl;unicode-bidi:isolate;}",
            "b{direction:ltr;unicode-bidi:isolate;}",
            "c{direction:rtl;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Isolate, ExpectedOutcome::Isolate],
    );
    assert_eq!(result.unicode_bidi_observations()[0].occurrence_index(), 1);
    assert_eq!(result.unicode_bidi_observations()[1].occurrence_index(), 3);
    assert_eq!(
        result.unicode_bidi_observations()[0].outcome(),
        result.unicode_bidi_observations()[1].outcome()
    );
}

#[test]
fn one_run_interleaves_unicode_bidi_with_every_accepted_leaf() {
    let result = qualify(
        2746,
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
    assert_eq!(result.unicode_bidi_observations()[0].occurrence_index(), 35);
    assert_expected(&result, &[ExpectedOutcome::Plaintext]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(2747, "a{unicode-bidi:plaintext;}b{unicode-bidi:plaintext;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::Plaintext, ExpectedOutcome::Plaintext],
    );
    assert_eq!(result.unicode_bidi_observations()[0].occurrence_index(), 0);
    assert_eq!(result.unicode_bidi_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.unicode_bidi_observations()[0]
            .placement()
            .context_id(),
        result.unicode_bidi_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2750, "@font-face{unicode-bidi:plaintext;}"),
        (2751, "@page{unicode-bidi:plaintext;}"),
        (2752, "@page{@top-left{unicode-bidi:plaintext;}}"),
        (2753, "@keyframes k{from{unicode-bidi:plaintext;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.unicode_bidi_observations().is_empty(),
            "nonordinary declaration context produced a unicode-bidi observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2760,
        "a{unicode-bidi:normal;unicode-bidi:plaintext;}",
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
        "a{unicode-bidi:normal;}",
        "b{unicode-bidi:inherit;}",
        "c{unicode-bidi:auto;}",
        "d{unicode-bidi:var(--bidi);}",
        "e{direction:rtl;}",
    );
    let first = qualify(2770, css);
    let repeated = qualify(2770, css);
    let another_source = qualify(2771, css);

    assert_eq!(
        first.unicode_bidi_observations(),
        repeated.unicode_bidi_observations()
    );
    assert_eq!(
        first.unicode_bidi_observations(),
        another_source.unicode_bidi_observations()
    );
}
