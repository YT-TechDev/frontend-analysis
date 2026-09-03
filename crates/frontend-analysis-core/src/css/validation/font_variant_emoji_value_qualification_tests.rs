use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontVariantEmojiQualificationOutcome, CssFontVariantEmojiUnsupportedReason,
    CssFontVariantEmojiValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Text,
    Emoji,
    Unicode,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFontVariantEmojiQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => CssFontVariantEmojiQualificationOutcome::Qualified(
            CssFontVariantEmojiValue::Normal,
        ),
        ExpectedOutcome::Text => {
            CssFontVariantEmojiQualificationOutcome::Qualified(CssFontVariantEmojiValue::Text)
        }
        ExpectedOutcome::Emoji => {
            CssFontVariantEmojiQualificationOutcome::Qualified(CssFontVariantEmojiValue::Emoji)
        }
        ExpectedOutcome::Unicode => CssFontVariantEmojiQualificationOutcome::Qualified(
            CssFontVariantEmojiValue::Unicode,
        ),
        ExpectedOutcome::Invalid => {
            CssFontVariantEmojiQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFontVariantEmojiQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantEmojiUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFontVariantEmojiQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantEmojiUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .font_variant_emoji_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_normal_text_emoji_unicode_grammar() {
    let result = qualify(
        2300,
        concat!(
            "a{font-variant-emoji:normal;}",
            "b{font-variant-emoji:text;}",
            "c{font-variant-emoji:emoji;}",
            "d{font-variant-emoji:unicode;}",
            "e{FONT-VARIANT-EMOJI:NoRmAl;}",
            "f{font-variant-emoji:EMOJI;}",
            r"g{font-variant-emoji:\6e ormal;}",
            r"h{font-variant-emoji:\74 ext;}",
            r"i{font-variant-emoji:\65 moji;}",
            r"j{font-variant-emoji:\75 nicode;}",
            "k{font-variant-emoji:auto;}",
            "l{font-variant-emoji:none;}",
            "m{font-variant-emoji:color;}",
            "n{font-variant-emoji:normal text;}",
            "o{font-variant-emoji:text emoji;}",
            "p{font-variant-emoji:normal,unicode;}",
            "q{font-variant-emoji:unicode,emoji;}",
            "r{font-variant-emoji:;}",
            "s{font-variant-emoji:1;}",
            "t{font-variant-emoji:1px;}",
            "u{font-variant-emoji:\"emoji\";}",
            "v{color:emoji;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Text,
            ExpectedOutcome::Emoji,
            ExpectedOutcome::Unicode,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Emoji,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Text,
            ExpectedOutcome::Emoji,
            ExpectedOutcome::Unicode,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
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
}

#[test]
fn comments_and_priority_preserve_decoded_keyword_meaning() {
    let result = qualify(
        2301,
        concat!(
            "a{font-variant-emoji:/**/normal/**/!important;}",
            "b{font-variant-emoji:/**/emoji/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Emoji]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2302,
        concat!(
            "a{font-variant-emoji:initial;}",
            "b{font-variant-emoji:inherit;}",
            "c{font-variant-emoji:unset;}",
            "d{font-variant-emoji:revert;}",
            "e{font-variant-emoji:revert-layer;}",
            "f{font-variant-emoji:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2303,
        concat!(
            "a{font-variant-emoji:var(--emoji);}",
            "b{font-variant-emoji:env(emoji);}",
            "c{font-variant-emoji:attr(data-emoji);}",
            "d{font-variant-emoji:--emoji();}",
            "e{font-variant-emoji:first-valid(normal,emoji);}",
            "f{font-variant-emoji:cycle(text,unicode);}",
            "g{font-variant-emoji:interpolate(0%,0:normal,1:emoji);}",
            "h{font-variant-emoji:foo();}",
            "i{font-variant-emoji:calc(1);}",
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
        2304,
        concat!(
            "a{font-variant-emoji:normal first-valid(text);}",
            "b{font-variant-emoji:first-valid(text) normal;}",
            "c{font-variant-emoji:emoji foo();}",
            "d{font-variant-emoji:foo() unicode;}",
            "e{font-variant-emoji:normal var(--emoji);}",
            "f{font-variant-emoji:foo(var(--emoji));}",
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
fn one_run_interleaves_font_variant_emoji_with_every_accepted_leaf() {
    let result = qualify(
        2305,
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
    assert_eq!(
        result.font_variant_emoji_observations()[0].occurrence_index(),
        30
    );
    assert_expected(&result, &[ExpectedOutcome::Emoji]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2306,
        "a{font-variant-emoji:text;}b{font-variant-emoji:text;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Text, ExpectedOutcome::Text]);
    assert_eq!(result.font_variant_emoji_observations()[0].occurrence_index(), 0);
    assert_eq!(result.font_variant_emoji_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.font_variant_emoji_observations()[0]
            .placement()
            .context_id(),
        result.font_variant_emoji_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2310, "@font-face{font-variant-emoji:emoji;}"),
        (2311, "@page{font-variant-emoji:emoji;}"),
        (2312, "@page{@top-left{font-variant-emoji:emoji;}}"),
        (2313, "@keyframes k{from{font-variant-emoji:emoji;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_variant_emoji_observations().is_empty(),
            "nonordinary declaration context produced a font-variant-emoji observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2320,
        "a{font-variant-emoji:normal;font-variant-emoji:unicode;}",
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
        "a{font-variant-emoji:normal;}",
        "b{font-variant-emoji:inherit;}",
        "c{font-variant-emoji:auto;}",
        "d{font-variant-emoji:var(--emoji);}",
    );
    let first = qualify(2330, css);
    let repeated = qualify(2330, css);
    let another_source = qualify(2331, css);

    assert_eq!(
        first.font_variant_emoji_observations(),
        repeated.font_variant_emoji_observations()
    );
    assert_eq!(
        first.font_variant_emoji_observations(),
        another_source.font_variant_emoji_observations()
    );
}
