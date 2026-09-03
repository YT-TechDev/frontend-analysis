use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontVariantCapsQualificationOutcome, CssFontVariantCapsUnsupportedReason,
    CssFontVariantCapsValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFontVariantCapsQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::Normal)
        }
        ExpectedOutcome::SmallCaps => {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::SmallCaps)
        }
        ExpectedOutcome::AllSmallCaps => {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::AllSmallCaps)
        }
        ExpectedOutcome::PetiteCaps => {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::PetiteCaps)
        }
        ExpectedOutcome::AllPetiteCaps => CssFontVariantCapsQualificationOutcome::Qualified(
            CssFontVariantCapsValue::AllPetiteCaps,
        ),
        ExpectedOutcome::Unicase => {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::Unicase)
        }
        ExpectedOutcome::TitlingCaps => {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::TitlingCaps)
        }
        ExpectedOutcome::Invalid => {
            CssFontVariantCapsQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFontVariantCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantCapsUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFontVariantCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantCapsUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .font_variant_caps_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_font_variant_caps_grammar() {
    let result = qualify(
        2400,
        concat!(
            "a{font-variant-caps:normal;}",
            "b{font-variant-caps:small-caps;}",
            "c{font-variant-caps:all-small-caps;}",
            "d{font-variant-caps:petite-caps;}",
            "e{font-variant-caps:all-petite-caps;}",
            "f{font-variant-caps:unicase;}",
            "g{font-variant-caps:titling-caps;}",
            "h{FONT-VARIANT-CAPS:SmAlL-CaPs;}",
            r"i{font-variant-caps:\73 mall-caps;}",
            r"j{font-variant-caps:\75 nicase;}",
            "k{font-variant-caps:auto;}",
            "l{font-variant-caps:none;}",
            "m{font-variant-caps:normal unicase;}",
            "n{font-variant-caps:small-caps,unicase;}",
            "o{font-variant-caps:;}",
            "p{font-variant-caps:1;}",
            "q{font-variant-caps:1px;}",
            "r{font-variant-caps:\"small-caps\";}",
            "s{color:small-caps;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::SmallCaps,
            ExpectedOutcome::AllSmallCaps,
            ExpectedOutcome::PetiteCaps,
            ExpectedOutcome::AllPetiteCaps,
            ExpectedOutcome::Unicase,
            ExpectedOutcome::TitlingCaps,
            ExpectedOutcome::SmallCaps,
            ExpectedOutcome::SmallCaps,
            ExpectedOutcome::Unicase,
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
        2401,
        concat!(
            "a{font-variant-caps:/**/small-caps/**/!important;}",
            "b{font-variant-caps:/**/titling-caps/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::SmallCaps, ExpectedOutcome::TitlingCaps],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2402,
        concat!(
            "a{font-variant-caps:initial;}",
            "b{font-variant-caps:inherit;}",
            "c{font-variant-caps:unset;}",
            "d{font-variant-caps:revert;}",
            "e{font-variant-caps:revert-layer;}",
            "f{font-variant-caps:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2403,
        concat!(
            "a{font-variant-caps:var(--caps);}",
            "b{font-variant-caps:env(caps);}",
            "c{font-variant-caps:attr(data-caps);}",
            "d{font-variant-caps:--caps();}",
            "e{font-variant-caps:first-valid(small-caps,unicase);}",
            "f{font-variant-caps:cycle(petite-caps,titling-caps);}",
            "g{font-variant-caps:interpolate(0%,0:normal,1:unicase);}",
            "h{font-variant-caps:foo();}",
            "i{font-variant-caps:calc(1);}",
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
        2404,
        concat!(
            "a{font-variant-caps:normal first-valid(unicase);}",
            "b{font-variant-caps:first-valid(unicase) normal;}",
            "c{font-variant-caps:small-caps foo();}",
            "d{font-variant-caps:foo() titling-caps;}",
            "e{font-variant-caps:normal var(--caps);}",
            "f{font-variant-caps:foo(var(--caps));}",
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
fn one_run_interleaves_font_variant_caps_with_every_accepted_leaf() {
    let result = qualify(
        2405,
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
    assert_eq!(
        result.font_variant_caps_observations()[0].occurrence_index(),
        31
    );
    assert_expected(&result, &[ExpectedOutcome::Unicase]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2406,
        "a{font-variant-caps:small-caps;}b{font-variant-caps:small-caps;}",
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::SmallCaps, ExpectedOutcome::SmallCaps],
    );
    assert_eq!(
        result.font_variant_caps_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.font_variant_caps_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.font_variant_caps_observations()[0]
            .placement()
            .context_id(),
        result.font_variant_caps_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2410, "@font-face{font-variant-caps:small-caps;}"),
        (2411, "@page{font-variant-caps:small-caps;}"),
        (2412, "@page{@top-left{font-variant-caps:small-caps;}}"),
        (2413, "@keyframes k{from{font-variant-caps:small-caps;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_variant_caps_observations().is_empty(),
            "nonordinary declaration context produced a font-variant-caps observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2420,
        "a{font-variant-caps:normal;font-variant-caps:unicase;}",
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
        "a{font-variant-caps:normal;}",
        "b{font-variant-caps:inherit;}",
        "c{font-variant-caps:auto;}",
        "d{font-variant-caps:var(--caps);}",
    );
    let first = qualify(2430, css);
    let repeated = qualify(2430, css);
    let another_source = qualify(2431, css);

    assert_eq!(
        first.font_variant_caps_observations(),
        repeated.font_variant_caps_observations()
    );
    assert_eq!(
        first.font_variant_caps_observations(),
        another_source.font_variant_caps_observations()
    );
}
