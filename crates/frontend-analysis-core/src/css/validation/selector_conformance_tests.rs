use super::selector_gold::{
    SelectorGoldFixture, SelectorGoldIndeterminateReason, SelectorGoldInvalidReason,
    SelectorGoldMode, SelectorGoldOutcome, SelectorGoldUnsupportedFeature, fixtures,
};
use crate::SourceId;
use crate::SourceText;
use crate::css::parser::resource::{CssParserLimits, CssParserResourceKind};
use crate::css::parser::result::{CssParserExecutionCompletion, CssParserTermination};
use crate::css::selector::analysis::analyze_css_selectors;
use crate::css::selector::context::CssSelectorGrammarContext;
use crate::css::selector::resource::{CssSelectorLimits, CssSelectorResourceKind};
use crate::css::selector::result::{
    CssSelectorExecutionCompletion, CssSelectorIndeterminateReason, CssSelectorInvalidReason,
    CssSelectorQualificationOutcome, CssSelectorTermination, CssSelectorUnsupportedFeature,
};
use crate::css::tokenizer::resource::{CssTokenizerLimits, CssTokenizerResourceKind};
use crate::css::tokenizer::result::{CssTokenizerCompletion, CssTokenizerTermination};

fn tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(16 * 1024, 200_000, 16 * 1024, 2048, 16 * 1024, 16 * 1024).unwrap()
}

fn parser_limits() -> CssParserLimits {
    CssParserLimits::new(
        200_000,
        512,
        512,
        16 * 1024,
        2048,
        2048,
        2048,
        2048,
        16 * 1024,
    )
    .unwrap()
}

fn selector_limits() -> CssSelectorLimits {
    CssSelectorLimits::new(200_000, 128, 16 * 1024, 128 * 1024).unwrap()
}

fn expected_mode(mode: SelectorGoldMode, actual: CssSelectorGrammarContext) -> bool {
    matches!(
        (mode, actual),
        (
            SelectorGoldMode::NormalSelectorList,
            CssSelectorGrammarContext::NormalSelectorList
        ) | (
            SelectorGoldMode::NestedRelativeSelectorList,
            CssSelectorGrammarContext::NestedRelativeSelectorList { .. }
        ) | (
            SelectorGoldMode::ScopedRelativeSelectorList,
            CssSelectorGrammarContext::ScopedRelativeSelectorList { .. }
        )
    )
}

fn expected_outcome(
    fixture: &SelectorGoldFixture,
    actual: &CssSelectorQualificationOutcome,
) -> bool {
    match (fixture.outcome, actual) {
        (
            SelectorGoldOutcome::Qualified,
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar,
        ) => true,
        (
            SelectorGoldOutcome::Invalid(expected),
            CssSelectorQualificationOutcome::InvalidForSelectedGrammar { reason, .. },
        ) => matches!(
            (expected, reason),
            (
                SelectorGoldInvalidReason::EmptySelectorList,
                CssSelectorInvalidReason::EmptySelectorList,
            ) | (
                SelectorGoldInvalidReason::UnexpectedComma,
                CssSelectorInvalidReason::UnexpectedComma,
            ) | (
                SelectorGoldInvalidReason::UnexpectedCombinator,
                CssSelectorInvalidReason::UnexpectedCombinator,
            ) | (
                SelectorGoldInvalidReason::InvalidCompoundOrder,
                CssSelectorInvalidReason::InvalidCompoundOrder,
            ) | (
                SelectorGoldInvalidReason::InvalidAttributeSelector,
                CssSelectorInvalidReason::InvalidAttributeSelector,
            ) | (
                SelectorGoldInvalidReason::InvalidNestingSelectorPlacement,
                CssSelectorInvalidReason::InvalidNestingSelectorPlacement,
            ) | (
                SelectorGoldInvalidReason::NestedHasNotAllowed,
                CssSelectorInvalidReason::NestedHasNotAllowed,
            ) | (
                SelectorGoldInvalidReason::UnexpectedToken,
                CssSelectorInvalidReason::UnexpectedToken,
            )
        ),
        (
            SelectorGoldOutcome::Unsupported(expected),
            CssSelectorQualificationOutcome::UnsupportedBySelectedGrammarProfile {
                feature, ..
            },
        ) => matches!(
            (expected, feature),
            (
                SelectorGoldUnsupportedFeature::IdentifierPseudoClass,
                CssSelectorUnsupportedFeature::IdentifierPseudoClass
            ) | (
                SelectorGoldUnsupportedFeature::FunctionalPseudoClass,
                CssSelectorUnsupportedFeature::FunctionalPseudoClass
            ) | (
                SelectorGoldUnsupportedFeature::PseudoElement,
                CssSelectorUnsupportedFeature::PseudoElement
            )
        ),
        (
            SelectorGoldOutcome::Indeterminate(
                SelectorGoldIndeterminateReason::MissingNamespaceEnvironment,
            ),
            CssSelectorQualificationOutcome::Indeterminate {
                reason: CssSelectorIndeterminateReason::MissingNamespaceEnvironment,
                ..
            },
        ) => true,
        _ => false,
    }
}

fn assert_fixture(fixture: &SelectorGoldFixture, source_id: SourceId) {
    let source = SourceText::new(source_id, fixture.source.to_owned());
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        selector_limits(),
    )
    .unwrap_or_else(|error| panic!("{} failed selector execution: {error:?}", fixture.id));

    let observation = result
        .observations()
        .iter()
        .find(|observation| {
            let context = &result.upstream_parser_result().context_records()
                [observation.context_id().index()];
            context.header().range().start() == fixture.header.start
                && context.header().range().end() == fixture.header.end
        })
        .unwrap_or_else(|| panic!("{} did not retain expected selector context", fixture.id));

    assert!(
        expected_mode(fixture.mode, observation.grammar_context()),
        "{} grammar mode mismatch: {:?}",
        fixture.id,
        observation.grammar_context()
    );
    assert!(
        expected_outcome(fixture, observation.outcome()),
        "{} outcome mismatch: {:?}",
        fixture.id,
        observation.outcome()
    );
}

#[test]
fn production_corev1_matches_every_independent_selector_gold_fixture() {
    for (index, fixture) in fixtures().iter().enumerate() {
        assert_fixture(fixture, SourceId::new(10_000 + index as u64));
    }
}

#[test]
fn selector_execution_is_repeat_deterministic_and_source_id_independent() {
    for (index, fixture) in fixtures().iter().enumerate() {
        assert_fixture(fixture, SourceId::new(20_000 + index as u64));
        assert_fixture(fixture, SourceId::new(30_000 + index as u64));
    }
}

#[test]
fn selector_observation_resource_refusal_preserves_committed_prefix() {
    let source = SourceText::new(SourceId::new(40_000), "a{}b{}".to_owned());
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        CssSelectorLimits::new(200_000, 128, 1, 128 * 1024).unwrap(),
    )
    .unwrap();

    assert_eq!(result.observations().len(), 1);
    assert_eq!(
        result.execution_completion(),
        CssSelectorExecutionCompletion::Incomplete
    );
    assert!(matches!(
        result.termination(),
        CssSelectorTermination::ResourceLimit(evidence)
            if evidence.kind() == CssSelectorResourceKind::Observations
    ));
}

#[test]
fn selector_depth_refusal_commits_no_partial_observation() {
    let source = SourceText::new(SourceId::new(40_001), ":is(.a){}".to_owned());
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        CssSelectorLimits::new(200_000, 0, 16, 128 * 1024).unwrap(),
    )
    .unwrap();

    assert!(result.observations().is_empty());
    assert!(matches!(
        result.termination(),
        CssSelectorTermination::ResourceLimit(evidence)
            if evidence.kind() == CssSelectorResourceKind::PeakSelectorDepth
    ));
}

#[test]
fn selector_algorithm_step_refusal_commits_no_partial_observation() {
    let source = SourceText::new(SourceId::new(40_002), ".a{}".to_owned());
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        CssSelectorLimits::new(1, 128, 16, 128 * 1024).unwrap(),
    )
    .unwrap();

    assert!(result.observations().is_empty());
    assert!(matches!(
        result.termination(),
        CssSelectorTermination::ResourceLimit(evidence)
            if evidence.kind() == CssSelectorResourceKind::AlgorithmSteps
    ));
}

#[test]
fn true_eof_after_retained_header_does_not_block_selector_qualification() {
    let source = SourceText::new(SourceId::new(40_003), "a{p:v;".to_owned());
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits(),
        selector_limits(),
    )
    .unwrap();

    assert!(matches!(
        result.observations()[0].outcome(),
        CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
    ));
}

#[test]
fn upstream_tokenizer_incomplete_after_header_preserves_selector_qualification() {
    let source = SourceText::new(SourceId::new(40_004), "a{p:v;}".to_owned());
    let tokenizer_limits =
        CssTokenizerLimits::new(16 * 1024, 200_000, 2, 2048, 16 * 1024, 16 * 1024).unwrap();
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits,
        parser_limits(),
        selector_limits(),
    )
    .unwrap();

    let parser = result.upstream_parser_result();
    assert_eq!(
        parser.upstream_tokenizer_result().completion(),
        CssTokenizerCompletion::Incomplete
    );
    assert!(matches!(
        parser.upstream_tokenizer_result().termination(),
        CssTokenizerTermination::ResourceLimit(evidence)
            if evidence.kind() == CssTokenizerResourceKind::LexicalItems
    ));
    assert_eq!(
        parser.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(
        parser.termination(),
        &CssParserTermination::UpstreamTokenizerIncomplete
    );
    assert_eq!(result.observations().len(), 1);
    assert_eq!(
        result.execution_completion(),
        CssSelectorExecutionCompletion::Complete
    );
    assert!(matches!(
        result.termination(),
        CssSelectorTermination::AllRetainedQualifiedContextsProcessed
    ));
    assert!(matches!(
        result.observations()[0].outcome(),
        CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
    ));
}

#[test]
fn parser_resource_limit_after_header_preserves_selector_qualification() {
    let source = SourceText::new(SourceId::new(40_005), "a{p:v;}".to_owned());
    let parser_limits =
        CssParserLimits::new(200_000, 512, 512, 0, 2048, 2048, 2048, 2048, 16 * 1024).unwrap();
    let result = analyze_css_selectors(
        &source,
        tokenizer_limits(),
        parser_limits,
        selector_limits(),
    )
    .unwrap();

    let parser = result.upstream_parser_result();
    assert_eq!(
        parser.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert!(matches!(
        parser.termination(),
        CssParserTermination::ParserResourceLimit(evidence)
            if evidence.kind() == CssParserResourceKind::DeclarationOccurrences
    ));
    assert_eq!(result.observations().len(), 1);
    assert_eq!(
        result.execution_completion(),
        CssSelectorExecutionCompletion::Complete
    );
    assert!(matches!(
        result.termination(),
        CssSelectorTermination::AllRetainedQualifiedContextsProcessed
    ));
    assert!(matches!(
        result.observations()[0].outcome(),
        CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
    ));
}
