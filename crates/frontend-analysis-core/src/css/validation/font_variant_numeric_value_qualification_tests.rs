use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontVariantNumericComponent, CssFontVariantNumericQualificationOutcome,
    CssFontVariantNumericUnsupportedReason, CssFontVariantNumericValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Components(&'static [CssFontVariantNumericComponent]),
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
    let actual = result.font_variant_numeric_observations();
    assert_eq!(actual.len(), expected.len());

    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssFontVariantNumericQualificationOutcome::Qualified(
                    CssFontVariantNumericValue::Normal,
                ),
                ExpectedOutcome::Normal,
            ) => {}
            (
                CssFontVariantNumericQualificationOutcome::Qualified(
                    CssFontVariantNumericValue::Components(components),
                ),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssFontVariantNumericQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantNumericUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantNumericUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantNumericUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => {
                panic!("unexpected font-variant-numeric outcome: {actual:?}, expected {expected:?}")
            }
        }
    }
}

#[test]
fn handwritten_fixed_slot_boundary_matches_pinned_wpt_and_derived_theorem() {
    use CssFontVariantNumericComponent::{
        DiagonalFractions, LiningNums, OldstyleNums, Ordinal, ProportionalNums, SlashedZero,
        StackedFractions, TabularNums,
    };

    let result = qualify(
        85_600,
        concat!(
            "a{font-variant-numeric:normal;}",
            "b{font-variant-numeric:lining-nums;}",
            "c{font-variant-numeric:oldstyle-nums;}",
            "d{font-variant-numeric:proportional-nums;}",
            "e{font-variant-numeric:tabular-nums;}",
            "f{font-variant-numeric:diagonal-fractions;}",
            "g{font-variant-numeric:stacked-fractions;}",
            "h{font-variant-numeric:ordinal;}",
            "i{font-variant-numeric:slashed-zero;}",
            "j{font-variant-numeric:proportional-nums slashed-zero diagonal-fractions oldstyle-nums ordinal;}",
            "k{font-variant-numeric:lining-nums oldstyle-nums;}",
            "l{font-variant-numeric:proportional-nums tabular-nums;}",
            "m{font-variant-numeric:diagonal-fractions stacked-fractions;}",
            "n{font-variant-numeric:ordinal ordinal;}",
            "o{font-variant-numeric:slashed-zero slashed-zero;}",
            "p{font-variant-numeric:normal lining-nums;}",
            "q{font-variant-numeric:auto;}",
            "r{font-variant-numeric:0;}",
            "s{font-variant-numeric:\"lining-nums\";}",
            "t{font-variant-numeric:;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Components(&[LiningNums]),
            ExpectedOutcome::Components(&[OldstyleNums]),
            ExpectedOutcome::Components(&[ProportionalNums]),
            ExpectedOutcome::Components(&[TabularNums]),
            ExpectedOutcome::Components(&[DiagonalFractions]),
            ExpectedOutcome::Components(&[StackedFractions]),
            ExpectedOutcome::Components(&[Ordinal]),
            ExpectedOutcome::Components(&[SlashedZero]),
            ExpectedOutcome::Components(&[
                ProportionalNums,
                SlashedZero,
                DiagonalFractions,
                OldstyleNums,
                Ordinal,
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
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn authored_order_is_preserved_without_canonicalization() {
    use CssFontVariantNumericComponent::{
        OldstyleNums, ProportionalNums, SlashedZero, TabularNums,
    };

    let result = qualify(
        85_601,
        concat!(
            "a{font-variant-numeric:proportional-nums oldstyle-nums;}",
            "b{font-variant-numeric:oldstyle-nums proportional-nums;}",
            "c{font-variant-numeric:slashed-zero tabular-nums;}",
            "d{font-variant-numeric:tabular-nums slashed-zero;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[ProportionalNums, OldstyleNums]),
            ExpectedOutcome::Components(&[OldstyleNums, ProportionalNums]),
            ExpectedOutcome::Components(&[SlashedZero, TabularNums]),
            ExpectedOutcome::Components(&[TabularNums, SlashedZero]),
        ],
    );
}

#[test]
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssFontVariantNumericComponent::{LiningNums, OldstyleNums, SlashedZero, TabularNums};

    let result = qualify(
        85_602,
        concat!(
            "a{FONT-VARIANT-NUMERIC:LINING-NUMS TABULAR-NUMS;}",
            r"b{font-variant-numeric:lining-num\73;}",
            r"c{font-variant-numeri\63:oldstyle-nums;}",
            "d{font-variant-numeric:/**/slashed-zero/**/tabular-nums/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[LiningNums, TabularNums]),
            ExpectedOutcome::Components(&[LiningNums]),
            ExpectedOutcome::Components(&[OldstyleNums]),
            ExpectedOutcome::Components(&[SlashedZero, TabularNums]),
        ],
    );

    let priority_observation = &result.font_variant_numeric_observations()[3];
    let occurrence =
        &result.upstream_parser_result().occurrences()[priority_observation.occurrence_index()];
    assert_eq!(priority_observation.placement(), occurrence.placement());
    assert!(occurrence.priority().is_some());
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        85_603,
        concat!(
            "a{font-variant-numeric:initial;}",
            "b{font-variant-numeric:inherit;}",
            "c{font-variant-numeric:unset;}",
            "d{font-variant-numeric:revert;}",
            "e{font-variant-numeric:revert-layer;}",
            "f{font-variant-numeric:revert-rule;}",
            "g{font-variant-numeric:inherit ordinal;}",
            "h{font-variant-numeric:ordinal inherit;}",
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
        85_604,
        concat!(
            "a{font-variant-numeric:var(--numeric);}",
            "b{font-variant-numeric:lining-nums var(--numeric);}",
            "c{font-variant-numeric:var(--numeric) ordinal;}",
            "d{font-variant-numeric:foo(var(--numeric));}",
            "e{font-variant-numeric:first-valid(lining-nums,ordinal);}",
            "f{font-variant-numeric:cycle(lining-nums,ordinal);}",
            "g{font-variant-numeric:interpolate(0%,0:lining-nums,1:ordinal);}",
            "h{font-variant-numeric:lining-nums first-valid(ordinal);}",
            "i{font-variant-numeric:foo();}",
            "j{font-variant-numeric:lining-nums foo();}",
            "k{font-variant-numeric:calc(1);}",
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
fn applicability_and_cross_dispatch_remain_isolated() {
    use CssFontVariantNumericComponent::{OldstyleNums, TabularNums};

    let result = qualify(
        85_605,
        concat!(
            "a{font-variant-ligatures:common-ligatures;}",
            "b{contain:layout size;}",
            "table{font-variant-numeric:oldstyle-nums tabular-nums;}",
            "d{font-variant-position:sub;}",
        ),
    );

    assert_eq!(result.font_variant_ligatures_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(result.font_variant_numeric_observations().len(), 1);
    assert_eq!(result.font_variant_position_observations().len(), 1);
    assert_eq!(
        result.font_variant_numeric_observations()[0].occurrence_index(),
        2
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[OldstyleNums, TabularNums])],
    );
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    use CssFontVariantNumericComponent::Ordinal;

    let result = qualify(
        85_606,
        "a{font-variant-numeric:ordinal;}b{font-variant-numeric:ordinal;}",
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Ordinal]),
            ExpectedOutcome::Components(&[Ordinal]),
        ],
    );
    assert_ne!(
        result.font_variant_numeric_observations()[0]
            .placement()
            .context_id(),
        result.font_variant_numeric_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (85_610, "@font-face{font-variant-numeric:ordinal;}"),
        (85_611, "@page{font-variant-numeric:ordinal;}"),
        (85_612, "@page{@top-left{font-variant-numeric:ordinal;}}"),
        (85_613, "@keyframes k{from{font-variant-numeric:ordinal;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_variant_numeric_observations().is_empty(),
            "unexpected observation for {css}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssFontVariantNumericComponent::{OldstyleNums, TabularNums};

    let result = qualify_with_limits(
        85_620,
        concat!(
            "a{font-variant-numeric:oldstyle-nums tabular-nums;",
            "font-variant-numeric:tabular-nums oldstyle-nums;}"
        ),
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[OldstyleNums, TabularNums])],
    );
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{font-variant-numeric:oldstyle-nums tabular-nums ordinal;}",
        "b{font-variant-numeric:normal;}",
        "c{font-variant-numeric:inherit;}",
        "d{font-variant-numeric:var(--numeric);}",
        "e{font-variant-numeric:lining-nums oldstyle-nums;}",
    );

    let first = qualify(85_621, css);
    let repeated = qualify(85_621, css);
    assert_eq!(
        first.font_variant_numeric_observations(),
        repeated.font_variant_numeric_observations()
    );

    let other_source = qualify(85_622, css);
    let first_outcomes: Vec<_> = first
        .font_variant_numeric_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let other_outcomes: Vec<_> = other_source
        .font_variant_numeric_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(first_outcomes, other_outcomes);
}
