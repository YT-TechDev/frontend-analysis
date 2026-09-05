use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontVariantLigaturesComponent, CssFontVariantLigaturesQualificationOutcome,
    CssFontVariantLigaturesUnsupportedReason, CssFontVariantLigaturesValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    None,
    Components(&'static [CssFontVariantLigaturesComponent]),
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferredFunction,
    UnsupportedWholeValueFunction,
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

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual = result.font_variant_ligatures_observations();
    assert_eq!(actual.len(), expected.len());

    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssFontVariantLigaturesQualificationOutcome::Qualified(
                    CssFontVariantLigaturesValue::Normal,
                ),
                ExpectedOutcome::Normal,
            )
            | (
                CssFontVariantLigaturesQualificationOutcome::Qualified(
                    CssFontVariantLigaturesValue::None,
                ),
                ExpectedOutcome::None,
            ) => {}
            (
                CssFontVariantLigaturesQualificationOutcome::Qualified(
                    CssFontVariantLigaturesValue::Components(components),
                ),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssFontVariantLigaturesQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssFontVariantLigaturesQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantLigaturesUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssFontVariantLigaturesQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantLigaturesUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssFontVariantLigaturesQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantLigaturesUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => panic!(
                "unexpected font-variant-ligatures outcome: {actual:?}, expected {expected:?}"
            ),
        }
    }
}

#[test]
fn handwritten_fixed_slot_boundary_matches_pinned_wpt_and_derived_theorem() {
    use CssFontVariantLigaturesComponent::{
        CommonLigatures, Contextual, DiscretionaryLigatures, HistoricalLigatures,
        NoCommonLigatures, NoContextual, NoDiscretionaryLigatures, NoHistoricalLigatures,
    };

    let result = qualify(
        85_500,
        concat!(
            "a{font-variant-ligatures:normal;}",
            "b{font-variant-ligatures:none;}",
            "c{font-variant-ligatures:common-ligatures;}",
            "d{font-variant-ligatures:no-common-ligatures;}",
            "e{font-variant-ligatures:discretionary-ligatures;}",
            "f{font-variant-ligatures:no-discretionary-ligatures;}",
            "g{font-variant-ligatures:historical-ligatures;}",
            "h{font-variant-ligatures:no-historical-ligatures;}",
            "i{font-variant-ligatures:contextual;}",
            "j{font-variant-ligatures:no-contextual;}",
            "k{font-variant-ligatures:common-ligatures contextual;}",
            "l{font-variant-ligatures:no-discretionary-ligatures historical-ligatures no-common-ligatures no-contextual;}",
            "m{font-variant-ligatures:common-ligatures no-common-ligatures;}",
            "n{font-variant-ligatures:discretionary-ligatures no-discretionary-ligatures;}",
            "o{font-variant-ligatures:historical-ligatures no-historical-ligatures;}",
            "p{font-variant-ligatures:contextual no-contextual;}",
            "q{font-variant-ligatures:common-ligatures common-ligatures;}",
            "r{font-variant-ligatures:none normal;}",
            "s{font-variant-ligatures:normal common-ligatures;}",
            "t{font-variant-ligatures:auto;}",
            "u{font-variant-ligatures:0;}",
            "v{font-variant-ligatures:\"common-ligatures\";}",
            "w{font-variant-ligatures:;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::None,
            ExpectedOutcome::Components(&[CommonLigatures]),
            ExpectedOutcome::Components(&[NoCommonLigatures]),
            ExpectedOutcome::Components(&[DiscretionaryLigatures]),
            ExpectedOutcome::Components(&[NoDiscretionaryLigatures]),
            ExpectedOutcome::Components(&[HistoricalLigatures]),
            ExpectedOutcome::Components(&[NoHistoricalLigatures]),
            ExpectedOutcome::Components(&[Contextual]),
            ExpectedOutcome::Components(&[NoContextual]),
            ExpectedOutcome::Components(&[CommonLigatures, Contextual]),
            ExpectedOutcome::Components(&[
                NoDiscretionaryLigatures,
                HistoricalLigatures,
                NoCommonLigatures,
                NoContextual,
            ]),
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
fn authored_order_and_standalone_identity_are_preserved_without_normalization() {
    use CssFontVariantLigaturesComponent::{
        CommonLigatures, Contextual, DiscretionaryLigatures, HistoricalLigatures,
    };

    let result = qualify(
        85_501,
        concat!(
            "a{font-variant-ligatures:common-ligatures contextual;}",
            "b{font-variant-ligatures:contextual common-ligatures;}",
            "c{font-variant-ligatures:historical-ligatures discretionary-ligatures;}",
            "d{font-variant-ligatures:discretionary-ligatures historical-ligatures;}",
            "e{font-variant-ligatures:normal;}",
            "f{font-variant-ligatures:none;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[CommonLigatures, Contextual]),
            ExpectedOutcome::Components(&[Contextual, CommonLigatures]),
            ExpectedOutcome::Components(&[HistoricalLigatures, DiscretionaryLigatures]),
            ExpectedOutcome::Components(&[DiscretionaryLigatures, HistoricalLigatures]),
            ExpectedOutcome::Normal,
            ExpectedOutcome::None,
        ],
    );
}

#[test]
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssFontVariantLigaturesComponent::{CommonLigatures, Contextual, NoHistoricalLigatures};

    let result = qualify(
        85_502,
        concat!(
            "a{FONT-VARIANT-LIGATURES:COMMON-LIGATURES CONTEXTUAL;}",
            r"b{font-variant-ligatures:common-ligatur\65 s;}",
            r"c{font-variant-ligatur\65 s:no-historical-ligatures;}",
            "d{font-variant-ligatures:/**/contextual/**/common-ligatures/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[CommonLigatures, Contextual]),
            ExpectedOutcome::Components(&[CommonLigatures]),
            ExpectedOutcome::Components(&[NoHistoricalLigatures]),
            ExpectedOutcome::Components(&[Contextual, CommonLigatures]),
        ],
    );

    let priority_observation = &result.font_variant_ligatures_observations()[3];
    let occurrence =
        &result.upstream_parser_result().occurrences()[priority_observation.occurrence_index()];
    assert_eq!(priority_observation.placement(), occurrence.placement());
    assert!(occurrence.priority().is_some());
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        85_503,
        concat!(
            "a{font-variant-ligatures:initial;}",
            "b{font-variant-ligatures:inherit;}",
            "c{font-variant-ligatures:unset;}",
            "d{font-variant-ligatures:revert;}",
            "e{font-variant-ligatures:revert-layer;}",
            "f{font-variant-ligatures:revert-rule;}",
            "g{font-variant-ligatures:inherit contextual;}",
            "h{font-variant-ligatures:contextual inherit;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn deferred_functions_fail_open_anywhere_while_other_functions_keep_existing_boundaries() {
    let result = qualify(
        85_504,
        concat!(
            "a{font-variant-ligatures:var(--ligatures);}",
            "b{font-variant-ligatures:common-ligatures var(--ligatures);}",
            "c{font-variant-ligatures:var(--ligatures) contextual;}",
            "d{font-variant-ligatures:foo(var(--ligatures));}",
            "e{font-variant-ligatures:first-valid(common-ligatures,contextual);}",
            "f{font-variant-ligatures:cycle(common-ligatures,contextual);}",
            "g{font-variant-ligatures:interpolate(0%,0:common-ligatures,1:contextual);}",
            "h{font-variant-ligatures:common-ligatures first-valid(contextual);}",
            "i{font-variant-ligatures:foo();}",
            "j{font-variant-ligatures:common-ligatures foo();}",
            "k{font-variant-ligatures:calc(1);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn applicability_is_not_an_input_to_authored_value_qualification() {
    use CssFontVariantLigaturesComponent::{CommonLigatures, Contextual, HistoricalLigatures};

    let result = qualify(
        85_505,
        concat!(
            "span{font-variant-ligatures:common-ligatures contextual;}",
            "table{font-variant-ligatures:historical-ligatures;}",
            "svg{font-variant-ligatures:contextual;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[CommonLigatures, Contextual]),
            ExpectedOutcome::Components(&[HistoricalLigatures]),
            ExpectedOutcome::Components(&[Contextual]),
        ],
    );
}

#[test]
fn cross_dispatch_with_accepted_font_variant_and_contain_leaves_remains_isolated() {
    use CssFontVariantLigaturesComponent::{CommonLigatures, Contextual};

    let result = qualify(
        85_506,
        concat!(
            "a{font-variant-caps:small-caps;}",
            "b{font-variant-position:sub;}",
            "c{font-variant-emoji:emoji;}",
            "d{font-variant-ligatures:common-ligatures contextual;}",
            "e{contain:layout size;}",
        ),
    );

    assert_eq!(result.font_variant_caps_observations().len(), 1);
    assert_eq!(result.font_variant_position_observations().len(), 1);
    assert_eq!(result.font_variant_emoji_observations().len(), 1);
    assert_eq!(result.font_variant_ligatures_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(
        result.font_variant_ligatures_observations()[0].occurrence_index(),
        3
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[CommonLigatures, Contextual])],
    );
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    use CssFontVariantLigaturesComponent::Contextual;

    let result = qualify(
        85_507,
        "a{font-variant-ligatures:contextual;}b{font-variant-ligatures:contextual;}",
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Contextual]),
            ExpectedOutcome::Components(&[Contextual]),
        ],
    );
    assert_ne!(
        result.font_variant_ligatures_observations()[0]
            .placement()
            .context_id(),
        result.font_variant_ligatures_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (
            85_510,
            "@font-face{font-variant-ligatures:common-ligatures;}",
        ),
        (85_511, "@page{font-variant-ligatures:common-ligatures;}"),
        (
            85_512,
            "@page{@top-left{font-variant-ligatures:common-ligatures;}}",
        ),
        (
            85_513,
            "@keyframes k{from{font-variant-ligatures:common-ligatures;}}",
        ),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_variant_ligatures_observations().is_empty(),
            "nonordinary declaration context produced a font-variant-ligatures observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssFontVariantLigaturesComponent::{CommonLigatures, Contextual};

    let result = qualify_with_limits(
        85_520,
        concat!(
            "a{font-variant-ligatures:common-ligatures contextual;",
            "font-variant-ligatures:contextual common-ligatures;}"
        ),
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[CommonLigatures, Contextual])],
    );
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{font-variant-ligatures:contextual common-ligatures;}",
        "b{font-variant-ligatures:none;}",
        "c{font-variant-ligatures:inherit;}",
        "d{font-variant-ligatures:var(--ligatures);}",
        "e{contain:layout size;}",
    );
    let first = qualify(85_530, css);
    let repeated = qualify(85_530, css);
    let another_source = qualify(85_531, css);

    assert_eq!(
        first.font_variant_ligatures_observations(),
        repeated.font_variant_ligatures_observations()
    );
    assert_eq!(
        first.font_variant_ligatures_observations(),
        another_source.font_variant_ligatures_observations()
    );
}
