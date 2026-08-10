//! Test-only production adapter for the independent #171 keyframes gold.
//!
//! Gold is never generated from production output. This module only executes
//! the real tokenizer/parser and compares retained production evidence with
//! the separately authored `keyframe_gold.rs` / `keyframe_fixtures.rs` data.

use crate::css::declaration::{
    CssDeclarationOccurrence, CssDeclarationPriorityEvidence, CssDeclarationTermination,
    CssKeyframeDeclarationOccurrence,
};
use crate::css::parser::context::{
    CssParserContextKind, CssParserContextRecord, CssParserContextTermination,
};
use crate::css::parser::evidence::{
    CssParserRecoveryKind, CssParserRecoveryTermination, CssParserUnsupportedRegion,
};
use crate::css::parser::producer::run;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::{
    CssParserCoverage, CssParserExecutionCompletion, CssParserRunResult, CssParserTermination,
};
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::{SourceAnchor, SourceId, SourceText};

use super::gold::GoldRange;
use super::keyframe_gold::{
    KeyframeGoldContext, KeyframeGoldContextTermination, KeyframeGoldFixture,
    KeyframeGoldOccurrence, KeyframeGoldOccurrenceTermination, KeyframeGoldPriority,
    KeyframeGoldRecoveryTermination, KeyframeGoldUnsupported,
};

fn generous_tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

pub(super) fn generous_parser_limits() -> CssParserLimits {
    CssParserLimits::new(
        1 << 20,
        1 << 12,
        1 << 12,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
    )
    .unwrap()
}

pub(super) fn run_source(source_id: u64, text: &str) -> CssParserRunResult {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    let tokenizer = run_tokenizer(&source, generous_tokenizer_limits()).unwrap();
    run(&source, tokenizer, generous_parser_limits()).unwrap()
}

pub(super) fn run_fixture(fixture: &KeyframeGoldFixture) -> CssParserRunResult {
    run_source(fixture.source_id, fixture.source)
}

fn assert_range(id: &str, label: &str, actual: &SourceAnchor, expected: GoldRange) {
    assert_eq!(
        actual.range().start(),
        expected.start,
        "{id}: {label} start"
    );
    assert_eq!(actual.range().end(), expected.end, "{id}: {label} end");
}

fn assert_context_termination(
    id: &str,
    actual: &CssParserContextTermination,
    expected: KeyframeGoldContextTermination,
) {
    match (actual, expected) {
        (
            CssParserContextTermination::AuthoredRightCurly { right_curly },
            KeyframeGoldContextTermination::AuthoredRightCurly(range),
        ) => assert_range(id, "context right curly", right_curly, range),
        (
            CssParserContextTermination::EndOfInput { terminal },
            KeyframeGoldContextTermination::EndOfInput(range),
        ) => assert_range(id, "context EOF terminal", terminal, range),
        _ => panic!("{id}: context termination mismatch: {actual:?} vs {expected:?}"),
    }
}

fn assert_priority(
    id: &str,
    actual: Option<&CssDeclarationPriorityEvidence>,
    expected: Option<KeyframeGoldPriority>,
) {
    match (actual, expected) {
        (None, None) => {}
        (Some(actual), Some(expected)) => {
            assert_range(
                id,
                "priority complete",
                actual.complete(),
                expected.complete,
            );
            assert_range(id, "priority bang", actual.bang(), expected.bang);
            assert_range(
                id,
                "priority important ident",
                actual.important_ident(),
                expected.important_ident,
            );
        }
        _ => panic!("{id}: priority presence mismatch"),
    }
}

fn assert_occurrence_termination(
    id: &str,
    actual: &CssDeclarationTermination,
    expected: KeyframeGoldOccurrenceTermination,
) {
    match (actual, expected) {
        (
            CssDeclarationTermination::AuthoredSemicolon { semicolon },
            KeyframeGoldOccurrenceTermination::AuthoredSemicolon(range),
        ) => assert_range(id, "occurrence semicolon", semicolon, range),
        (
            CssDeclarationTermination::OmittedBeforeRightCurly { right_curly },
            KeyframeGoldOccurrenceTermination::OmittedBeforeRightCurly(range),
        ) => assert_range(id, "occurrence right curly", right_curly, range),
        (
            CssDeclarationTermination::OmittedAtEndOfInput { terminal },
            KeyframeGoldOccurrenceTermination::OmittedAtEndOfInput(range),
        ) => assert_range(id, "occurrence EOF terminal", terminal, range),
        _ => panic!("{id}: occurrence termination mismatch: {actual:?} vs {expected:?}"),
    }
}

fn assert_keyframe_occurrence(
    id: &str,
    actual: &CssKeyframeDeclarationOccurrence,
    expected: &KeyframeGoldOccurrence,
) {
    assert_eq!(
        actual.placement().context_id().index(),
        expected.context_id,
        "{id}: keyframe occurrence owner"
    );
    assert_eq!(
        actual.placement().item_ordinal().value(),
        expected.item_ordinal,
        "{id}: keyframe occurrence item ordinal"
    );
    assert_range(
        id,
        "keyframe occurrence complete",
        actual.complete(),
        expected.complete,
    );
    assert_range(id, "keyframe occurrence name", actual.name(), expected.name);
    assert_range(
        id,
        "keyframe occurrence colon",
        actual.colon(),
        expected.colon,
    );
    assert_range(
        id,
        "keyframe occurrence value",
        actual.value(),
        expected.value,
    );
    assert_priority(id, actual.priority(), expected.priority);
    assert_occurrence_termination(id, actual.termination(), expected.termination);
}

fn assert_ordinary_occurrence(
    id: &str,
    actual: &CssDeclarationOccurrence,
    expected: &KeyframeGoldOccurrence,
) {
    assert_eq!(
        actual.placement().context_id().index(),
        expected.context_id,
        "{id}: ordinary occurrence owner"
    );
    assert_eq!(
        actual.placement().item_ordinal().value(),
        expected.item_ordinal,
        "{id}: ordinary occurrence item ordinal"
    );
    assert_range(
        id,
        "ordinary occurrence complete",
        actual.complete(),
        expected.complete,
    );
    assert_range(
        id,
        "ordinary occurrence name",
        actual.property_name(),
        expected.name,
    );
    assert_range(
        id,
        "ordinary occurrence colon",
        actual.colon(),
        expected.colon,
    );
    assert_range(
        id,
        "ordinary occurrence value",
        actual.value(),
        expected.value,
    );
    assert_priority(id, actual.priority(), expected.priority);
    assert_occurrence_termination(id, actual.termination(), expected.termination);
}

fn assert_context(id: &str, actual: &CssParserContextRecord, expected: &KeyframeGoldContext) {
    match expected {
        KeyframeGoldContext::Keyframes {
            id: expected_id,
            root_item_ordinal,
            at_keyword,
            keyframes_name,
            header,
            block_opener,
            body,
            termination,
        } => {
            assert_eq!(actual.id().index(), *expected_id, "{id}: keyframes id");
            assert!(
                actual.parent().is_none(),
                "{id}: keyframes must be root-owned"
            );
            assert_eq!(
                actual.item_ordinal().value(),
                *root_item_ordinal,
                "{id}: keyframes root item ordinal"
            );
            assert_eq!(actual.kind(), CssParserContextKind::KeyframesRuleBlock);
            assert_range(
                id,
                "keyframes at-keyword",
                actual.at_keyword().unwrap(),
                *at_keyword,
            );
            assert_range(
                id,
                "keyframes name",
                actual.keyframes_name().unwrap(),
                *keyframes_name,
            );
            assert!(actual.keyframe_selector_list().is_none());
            assert!(actual.nearest_qualified_ancestor().is_none());
            assert_range(id, "keyframes header", actual.header(), *header);
            assert_range(id, "keyframes opener", actual.block_opener(), *block_opener);
            assert_range(id, "keyframes body", actual.body(), *body);
            assert_context_termination(id, actual.termination(), *termination);
        }
        KeyframeGoldContext::KeyframeBlock {
            id: expected_id,
            parent_id,
            item_ordinal,
            selector_list,
            header,
            block_opener,
            body,
            termination,
        } => {
            assert_eq!(actual.id().index(), *expected_id, "{id}: keyframe block id");
            assert_eq!(
                actual.parent().map(|parent| parent.index()),
                Some(*parent_id),
                "{id}: keyframe block parent"
            );
            assert_eq!(
                actual.item_ordinal().value(),
                *item_ordinal,
                "{id}: keyframe block item ordinal"
            );
            assert_eq!(actual.kind(), CssParserContextKind::KeyframeBlock);
            assert!(actual.at_keyword().is_none());
            assert!(actual.keyframes_name().is_none());
            assert_range(
                id,
                "keyframe selector list",
                actual.keyframe_selector_list().unwrap(),
                *selector_list,
            );
            assert!(actual.nearest_qualified_ancestor().is_none());
            assert_range(id, "keyframe block header", actual.header(), *header);
            assert_range(
                id,
                "keyframe block opener",
                actual.block_opener(),
                *block_opener,
            );
            assert_range(id, "keyframe block body", actual.body(), *body);
            assert_context_termination(id, actual.termination(), *termination);
        }
    }
}

pub(super) fn assert_matches_keyframe_gold(
    fixture: &KeyframeGoldFixture,
    result: &CssParserRunResult,
) {
    let id = fixture.id;
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete,
        "{id}: source-driven fixture must complete"
    );
    assert_eq!(
        result.termination(),
        &CssParserTermination::EndOfTokenizerInput,
        "{id}: source-driven fixture termination"
    );

    let actual_contexts: Vec<_> = result
        .context_records()
        .iter()
        .filter(|record| {
            matches!(
                record.kind(),
                CssParserContextKind::KeyframesRuleBlock | CssParserContextKind::KeyframeBlock
            )
        })
        .collect();
    assert_eq!(
        actual_contexts.len(),
        fixture.contexts.len(),
        "{id}: keyframe context count"
    );
    for (actual, expected) in actual_contexts.iter().zip(&fixture.contexts) {
        assert_context(id, actual, expected);
    }

    assert_eq!(
        result.keyframe_occurrences().len(),
        fixture.keyframe_occurrences.len(),
        "{id}: keyframe occurrence count"
    );
    for (actual, expected) in result
        .keyframe_occurrences()
        .iter()
        .zip(&fixture.keyframe_occurrences)
    {
        assert_keyframe_occurrence(id, actual, expected);
    }

    assert_eq!(
        result.occurrences().len(),
        fixture.ordinary_occurrences.len(),
        "{id}: ordinary occurrence count"
    );
    for (actual, expected) in result
        .occurrences()
        .iter()
        .zip(&fixture.ordinary_occurrences)
    {
        assert_ordinary_occurrence(id, actual, expected);
    }

    assert_eq!(
        result.unsupported_regions().len(),
        fixture.unsupported.len(),
        "{id}: unsupported count"
    );
    for (actual, expected) in result
        .unsupported_regions()
        .iter()
        .zip(&fixture.unsupported)
    {
        match (actual, expected) {
            (
                CssParserUnsupportedRegion::TopLevelAtRule {
                    complete,
                    at_keyword,
                },
                KeyframeGoldUnsupported::TopLevelAtRule {
                    complete: expected_complete,
                    at_keyword: expected_at_keyword,
                },
            ) => {
                assert_range(
                    id,
                    "top-level unsupported complete",
                    complete,
                    *expected_complete,
                );
                assert_range(
                    id,
                    "top-level unsupported at-keyword",
                    at_keyword,
                    *expected_at_keyword,
                );
            }
            (
                CssParserUnsupportedRegion::NestedAtRule {
                    complete,
                    at_keyword,
                    context_id,
                    item_ordinal,
                },
                KeyframeGoldUnsupported::NestedAtRule {
                    complete: expected_complete,
                    at_keyword: expected_at_keyword,
                    context_id: expected_context,
                    item_ordinal: expected_ordinal,
                },
            ) => {
                assert_range(
                    id,
                    "nested unsupported complete",
                    complete,
                    *expected_complete,
                );
                assert_range(
                    id,
                    "nested unsupported at-keyword",
                    at_keyword,
                    *expected_at_keyword,
                );
                assert_eq!(
                    context_id.index(),
                    *expected_context,
                    "{id}: nested unsupported owner"
                );
                assert_eq!(
                    item_ordinal.value(),
                    *expected_ordinal,
                    "{id}: nested unsupported item ordinal"
                );
            }
            (
                CssParserUnsupportedRegion::UnqualifiedKeyframeBlock {
                    complete,
                    context_id,
                    item_ordinal,
                },
                KeyframeGoldUnsupported::UnqualifiedKeyframeBlock {
                    complete: expected_complete,
                    context_id: expected_context,
                    item_ordinal: expected_ordinal,
                },
            ) => {
                assert_range(
                    id,
                    "unqualified keyframe block",
                    complete,
                    *expected_complete,
                );
                assert_eq!(
                    context_id.index(),
                    *expected_context,
                    "{id}: unqualified block owner"
                );
                assert_eq!(
                    item_ordinal.value(),
                    *expected_ordinal,
                    "{id}: unqualified block item ordinal"
                );
            }
            _ => panic!("{id}: unsupported variant mismatch: {actual:?} vs {expected:?}"),
        }
    }

    assert_eq!(
        result.recovery_records().len(),
        fixture.recoveries.len(),
        "{id}: recovery count"
    );
    for (actual, expected) in result.recovery_records().iter().zip(&fixture.recoveries) {
        assert_eq!(actual.kind(), CssParserRecoveryKind::MalformedBlockItem);
        assert_range(id, "recovery region", actual.region(), expected.region);
        match (actual.termination(), expected.termination) {
            (
                CssParserRecoveryTermination::AuthoredSemicolon { semicolon },
                KeyframeGoldRecoveryTermination::AuthoredSemicolon(range),
            ) => assert_range(id, "recovery semicolon", semicolon, range),
            (
                CssParserRecoveryTermination::EndOfInput { terminal },
                KeyframeGoldRecoveryTermination::EndOfInput(range),
            ) => assert_range(id, "recovery EOF", terminal, range),
            _ => panic!("{id}: recovery termination mismatch"),
        }
    }

    assert_eq!(
        result.parser_diagnostics().len(),
        fixture.diagnostics.len(),
        "{id}: diagnostic count"
    );
    for (actual, expected) in result.parser_diagnostics().iter().zip(&fixture.diagnostics) {
        assert_range(id, "diagnostic location", actual.location(), *expected);
    }

    assert_eq!(
        result.coverage(),
        if fixture.unsupported.is_empty() {
            CssParserCoverage::SupportedForSelectedQuestion
        } else {
            CssParserCoverage::ContainsUnsupportedContexts
        },
        "{id}: coverage"
    );
}
