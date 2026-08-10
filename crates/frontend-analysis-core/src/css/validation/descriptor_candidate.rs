//! Test-only production candidate adapter comparing the #169 producer
//! against the candidate-independent descriptor-context gold model
//! (`descriptor_gold.rs` / `descriptor_fixtures.rs`).
//!
//! Gold is never generated from or rewritten by production output; this
//! module only reads production `CssParserRunResult` values to compare them
//! against the independently authored fixtures. Production code must never
//! import this module or the gold model.

use crate::css::declaration::{CssDeclarationOccurrence, CssDeclarationTermination};
use crate::css::parser::context::{
    CssParserContextKind, CssParserContextTermination, CssParserDescriptorRuleKind,
};
use crate::css::parser::evidence::{CssParserRecoveryTermination, CssParserUnsupportedRegion};
use crate::css::parser::producer::run;
use crate::css::parser::resource::{CssParserLimits, CssParserResourceKind};
use crate::css::parser::result::{
    CssParserExecutionCompletion, CssParserRunResult, CssParserTermination,
};
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::{SourceId, SourceText};

use super::descriptor_gold::{
    DescriptorGoldContext, DescriptorGoldFixture, DescriptorGoldOccurrence,
    DescriptorGoldOccurrenceTermination, DescriptorGoldRecoveryTermination,
    DescriptorGoldResourceKind, DescriptorGoldRoot, DescriptorGoldRuleKind,
    DescriptorGoldTermination,
};
use super::gold::GoldRange;

/// Fixture IDs intentionally excluded from production execution: the
/// generic-parser-resource-stop category (17) whose exact internal terminal
/// is not an architecture-specified contract, mirroring the fixed 16-fixture
/// inventory's own established `CONTRACT_ONLY_FIXTURE_IDS` precedent
/// (`context_candidate.rs`).
pub(super) const DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS: &[&str] =
    &["CSS-DESCRIPTOR-GENERIC-PARSER-RESOURCE-LIMITED-TERMINATION-001"];

fn generous_tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

fn generous_parser_limits() -> CssParserLimits {
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

/// Runs `fixture` against real production, selecting the execution recipe
/// its own resource expectation requires. Panics (as test-only wiring) if
/// called for a fixture listed in [`DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS`].
pub(super) fn run_fixture(fixture: &DescriptorGoldFixture) -> CssParserRunResult {
    assert!(
        !DESCRIPTOR_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id),
        "{}: contract-only fixture has no production execution recipe",
        fixture.id
    );

    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits())
        .unwrap_or_else(|error| panic!("{}: production tokenizer error: {error:?}", fixture.id));

    let limits = match fixture.resource_expectation {
        Some(expectation) => {
            let mut max_algorithm_steps = 1 << 20;
            let mut max_peak_context_depth = 1 << 12;
            let mut max_declaration_occurrences = 1 << 16;
            let mut max_unsupported_regions = 1 << 16;
            let mut max_context_records = 1 << 16;
            match expectation.kind {
                DescriptorGoldResourceKind::AlgorithmSteps => {
                    max_algorithm_steps = expectation.limit.max(1)
                }
                DescriptorGoldResourceKind::PeakContextDepth => {
                    max_peak_context_depth = expectation.limit
                }
                DescriptorGoldResourceKind::ContextRecords => {
                    max_context_records = expectation.limit
                }
                DescriptorGoldResourceKind::DeclarationOccurrences => {
                    max_declaration_occurrences = expectation.limit
                }
                DescriptorGoldResourceKind::UnsupportedRegions => {
                    max_unsupported_regions = expectation.limit
                }
            }
            CssParserLimits::new(
                max_algorithm_steps,
                1 << 12,
                max_peak_context_depth,
                max_declaration_occurrences,
                1 << 16,
                1 << 16,
                max_unsupported_regions,
                1 << 16,
                max_context_records,
            )
            .unwrap()
        }
        None => generous_parser_limits(),
    };

    run(&source, tokenizer_result, limits)
        .unwrap_or_else(|error| panic!("{}: production parser error: {error:?}", fixture.id))
}

/// Compares production `result` against `fixture`'s independent descriptor
/// expectations. Panics (as test-only wiring) with a descriptive message on
/// any mismatch.
pub(super) fn assert_matches_descriptor_gold(
    fixture: &DescriptorGoldFixture,
    result: &CssParserRunResult,
) {
    let id = fixture.id;

    let context_roots: Vec<&DescriptorGoldRoot> = fixture
        .roots
        .iter()
        .filter(|root| !matches!(root, DescriptorGoldRoot::UnsupportedAtRule { .. }))
        .collect();

    assert_eq!(
        result.context_records().len(),
        context_roots.len(),
        "{id}: context record count"
    );

    for (index, (actual, expected_root)) in result
        .context_records()
        .iter()
        .zip(&context_roots)
        .enumerate()
    {
        match expected_root {
            DescriptorGoldRoot::Descriptor(context) => {
                assert!(
                    actual.parent().is_none(),
                    "{id}: context {index} descriptor context must be stylesheet-root-only"
                );
                assert_eq!(actual.id().index(), context.id, "{id}: context {index} id");
                assert_eq!(
                    actual.item_ordinal().value(),
                    context.root_item_ordinal,
                    "{id}: context {index} root item_ordinal"
                );
                assert_descriptor_kind_matches(id, index, actual.kind(), context.kind);
                assert_range(
                    id,
                    "context at_keyword",
                    index,
                    actual
                        .at_keyword()
                        .unwrap_or_else(|| panic!("{id}: context {index} missing at_keyword")),
                    context.at_keyword,
                );
                match (actual.descriptor_property_name(), context.property_name) {
                    (None, None) => {}
                    (Some(actual_name), Some(expected_name)) => {
                        assert_range(
                            id,
                            "context descriptor_property_name",
                            index,
                            actual_name,
                            expected_name,
                        );
                    }
                    _ => panic!("{id}: context {index} descriptor_property_name presence mismatch"),
                }
                assert_range(id, "context header", index, actual.header(), context.header);
                assert_range(
                    id,
                    "context block_opener",
                    index,
                    actual.block_opener(),
                    context.block_opener,
                );
                assert_range(id, "context body", index, actual.body(), context.body);
                assert_descriptor_termination_matches(
                    id,
                    index,
                    actual.termination(),
                    context.termination,
                );
            }
            DescriptorGoldRoot::Qualified(companion) => {
                assert!(
                    actual.parent().is_none(),
                    "{id}: context {index} qualified companion must be stylesheet-root"
                );
                assert_eq!(
                    actual.kind(),
                    CssParserContextKind::QualifiedRuleBlock,
                    "{id}: context {index} expected QualifiedRuleBlock"
                );
                assert_eq!(
                    actual.id().index(),
                    companion.id,
                    "{id}: context {index} id"
                );
                assert_eq!(
                    actual.item_ordinal().value(),
                    companion.root_item_ordinal,
                    "{id}: context {index} root item_ordinal"
                );
                assert_range(
                    id,
                    "companion header",
                    index,
                    actual.header(),
                    companion.header,
                );
                assert_range(
                    id,
                    "companion block_opener",
                    index,
                    actual.block_opener(),
                    companion.block_opener,
                );
                assert_range(id, "companion body", index, actual.body(), companion.body);
                assert_descriptor_termination_matches(
                    id,
                    index,
                    actual.termination(),
                    companion.termination,
                );
            }
            DescriptorGoldRoot::UnsupportedAtRule { .. } => unreachable!(),
        }
    }

    // Top-level unsupported at-rules.
    let expected_top_level: Vec<(GoldRange, GoldRange)> = fixture
        .roots
        .iter()
        .filter_map(|root| match root {
            DescriptorGoldRoot::UnsupportedAtRule {
                complete,
                at_keyword,
                ..
            } => Some((*complete, *at_keyword)),
            _ => None,
        })
        .collect();
    let actual_top_level: Vec<_> = result
        .unsupported_regions()
        .iter()
        .filter_map(|region| match region {
            CssParserUnsupportedRegion::TopLevelAtRule {
                complete,
                at_keyword,
            } => Some((complete, at_keyword)),
            _ => None,
        })
        .collect();
    assert_eq!(
        actual_top_level.len(),
        expected_top_level.len(),
        "{id}: top-level unsupported at-rule count"
    );
    for (index, (actual, expected)) in actual_top_level.iter().zip(&expected_top_level).enumerate()
    {
        assert_range(
            id,
            "top-level unsupported complete",
            index,
            actual.0,
            expected.0,
        );
        assert_range(
            id,
            "top-level unsupported at_keyword",
            index,
            actual.1,
            expected.1,
        );
    }

    // Ordinary declarations, grouped by owning context.
    let mut ordinary_by_context: std::collections::BTreeMap<usize, Vec<&CssDeclarationOccurrence>> =
        std::collections::BTreeMap::new();
    for occurrence in result.occurrences() {
        ordinary_by_context
            .entry(occurrence.placement().context_id().index())
            .or_default()
            .push(occurrence);
    }
    for root in &fixture.roots {
        if let DescriptorGoldRoot::Qualified(companion) = root {
            let actual = ordinary_by_context
                .remove(&companion.id)
                .unwrap_or_default();
            assert_eq!(
                actual.len(),
                companion.declarations.len(),
                "{id}: companion {} declaration count",
                companion.id
            );
            for (decl_index, (actual, expected)) in
                actual.iter().zip(&companion.declarations).enumerate()
            {
                assert_eq!(
                    actual.placement().item_ordinal().value(),
                    expected.item_ordinal,
                    "{id}: companion {} declaration {decl_index} item_ordinal",
                    companion.id
                );
                assert_occurrence_syntax_matches(
                    id,
                    &format!("companion {} declaration {decl_index}", companion.id),
                    actual.complete(),
                    actual.property_name(),
                    actual.colon(),
                    actual.value(),
                    actual.priority(),
                    actual.termination(),
                    expected,
                );
            }
        }
    }
    assert!(
        ordinary_by_context.is_empty(),
        "{id}: production emitted ordinary declarations in unexpected contexts: {ordinary_by_context:?}"
    );

    // Descriptor occurrences, grouped by owning context.
    let mut descriptor_by_context: std::collections::BTreeMap<
        usize,
        Vec<&crate::css::declaration::CssDescriptorOccurrence>,
    > = std::collections::BTreeMap::new();
    for occurrence in result.descriptor_occurrences() {
        descriptor_by_context
            .entry(occurrence.placement().context_id().index())
            .or_default()
            .push(occurrence);
    }
    // Nested unsupported at-rules, grouped by owning context.
    let mut nested_unsupported_by_context: std::collections::BTreeMap<
        usize,
        Vec<(&crate::SourceAnchor, &crate::SourceAnchor, usize)>,
    > = std::collections::BTreeMap::new();
    for region in result.unsupported_regions() {
        if let CssParserUnsupportedRegion::NestedAtRule {
            complete,
            at_keyword,
            context_id,
            item_ordinal,
        } = region
        {
            nested_unsupported_by_context
                .entry(context_id.index())
                .or_default()
                .push((complete, at_keyword, item_ordinal.value()));
        }
    }

    for root in &fixture.roots {
        if let DescriptorGoldRoot::Descriptor(context) = root {
            let actual_occurrences = descriptor_by_context
                .remove(&context.id)
                .unwrap_or_default();
            assert_eq!(
                actual_occurrences.len(),
                context.occurrences.len(),
                "{id}: descriptor context {} occurrence count",
                context.id
            );
            for (occ_index, (actual, expected)) in actual_occurrences
                .iter()
                .zip(&context.occurrences)
                .enumerate()
            {
                assert_eq!(
                    actual.placement().item_ordinal().value(),
                    expected.item_ordinal,
                    "{id}: descriptor context {} occurrence {occ_index} item_ordinal",
                    context.id
                );
                assert_occurrence_syntax_matches(
                    id,
                    &format!("descriptor context {} occurrence {occ_index}", context.id),
                    actual.complete(),
                    actual.name(),
                    actual.colon(),
                    actual.value(),
                    actual.priority(),
                    actual.termination(),
                    expected,
                );
            }

            let actual_nested = nested_unsupported_by_context
                .remove(&context.id)
                .unwrap_or_default();
            assert_eq!(
                actual_nested.len(),
                context.nested_unsupported.len(),
                "{id}: descriptor context {} nested unsupported count",
                context.id
            );
            for (nested_index, (actual, expected)) in actual_nested
                .iter()
                .zip(&context.nested_unsupported)
                .enumerate()
            {
                let (complete, at_keyword, item_ordinal) = *actual;
                assert_range(
                    id,
                    "nested unsupported complete",
                    nested_index,
                    complete,
                    expected.complete,
                );
                assert_range(
                    id,
                    "nested unsupported at_keyword",
                    nested_index,
                    at_keyword,
                    expected.at_keyword,
                );
                assert_eq!(
                    item_ordinal, expected.item_ordinal,
                    "{id}: descriptor context {} nested unsupported {nested_index} item_ordinal",
                    context.id
                );
            }
        }
    }
    assert!(
        descriptor_by_context.is_empty(),
        "{id}: production emitted descriptor occurrences in unexpected contexts: {descriptor_by_context:?}"
    );
    assert!(
        nested_unsupported_by_context.is_empty(),
        "{id}: production emitted nested unsupported at-rules in unexpected contexts: {nested_unsupported_by_context:?}"
    );

    // Diagnostics / recovery (shared, pre-#169 vocabulary).
    assert_eq!(
        result.parser_diagnostics().len(),
        fixture.diagnostics.len(),
        "{id}: diagnostic count"
    );
    for (index, (actual, expected)) in result
        .parser_diagnostics()
        .iter()
        .zip(&fixture.diagnostics)
        .enumerate()
    {
        assert_eq!(
            actual.code(),
            crate::css::parser::diagnostic::CssParserDiagnosticCode::InvalidBlockItem,
            "{id}: diagnostic {index} code"
        );
        assert_range(
            id,
            "diagnostic location",
            index,
            actual.location(),
            *expected,
        );
    }

    assert_eq!(
        result.recovery_records().len(),
        fixture.recovery.len(),
        "{id}: recovery count"
    );
    for (index, (actual, expected)) in result
        .recovery_records()
        .iter()
        .zip(&fixture.recovery)
        .enumerate()
    {
        assert_eq!(
            actual.kind(),
            crate::css::parser::evidence::CssParserRecoveryKind::MalformedBlockItem,
            "{id}: recovery {index} kind"
        );
        assert_range(
            id,
            "recovery region",
            index,
            actual.region(),
            expected.region,
        );
        assert_recovery_termination_matches(id, index, actual.termination(), expected.termination);
    }

    // Resource usage: descriptor-relevant dimensions.
    let resources = result.resources();
    let expected_context_records = context_roots.len();
    let expected_declaration_occurrences: usize = fixture
        .roots
        .iter()
        .map(|root| match root {
            DescriptorGoldRoot::Descriptor(context) => context.occurrences.len(),
            DescriptorGoldRoot::Qualified(companion) => companion.declarations.len(),
            DescriptorGoldRoot::UnsupportedAtRule { .. } => 0,
        })
        .sum();
    // Nested-unsupported-at-rule count only; top-level unsupported at-rules
    // are already counted via `expected_top_level.len()` below, and double-
    // counting them here would count each once per root instead of once
    // total.
    let expected_nested_unsupported_regions: usize = fixture
        .roots
        .iter()
        .map(|root| match root {
            DescriptorGoldRoot::Descriptor(context) => context.nested_unsupported.len(),
            DescriptorGoldRoot::Qualified(_) | DescriptorGoldRoot::UnsupportedAtRule { .. } => 0,
        })
        .sum();

    match fixture.resource_expectation {
        Some(expectation) => {
            let CssParserTermination::ParserResourceLimit(evidence) = result.termination() else {
                panic!("{id}: expected ParserResourceLimit termination");
            };
            let expected_kind = match expectation.kind {
                DescriptorGoldResourceKind::AlgorithmSteps => CssParserResourceKind::AlgorithmSteps,
                DescriptorGoldResourceKind::PeakContextDepth => {
                    CssParserResourceKind::PeakContextDepth
                }
                DescriptorGoldResourceKind::ContextRecords => CssParserResourceKind::ContextRecords,
                DescriptorGoldResourceKind::DeclarationOccurrences => {
                    CssParserResourceKind::DeclarationOccurrences
                }
                DescriptorGoldResourceKind::UnsupportedRegions => {
                    CssParserResourceKind::UnsupportedRegions
                }
            };
            assert_eq!(evidence.kind(), expected_kind, "{id}: resource limit kind");
            assert_eq!(evidence.limit(), expectation.limit, "{id}: resource limit");
            assert_eq!(
                evidence.attempted(),
                expectation.attempted,
                "{id}: resource attempted"
            );
            assert_eq!(
                result.execution_completion(),
                CssParserExecutionCompletion::Incomplete,
                "{id}: execution completion"
            );
        }
        None => {
            let expects_upstream_incomplete = fixture.roots.iter().any(|root| {
                matches!(
                    root,
                    DescriptorGoldRoot::Descriptor(DescriptorGoldContext {
                        termination: DescriptorGoldTermination::UpstreamTokenizerIncomplete(_),
                        ..
                    })
                )
            });
            if expects_upstream_incomplete {
                assert!(
                    matches!(
                        result.termination(),
                        CssParserTermination::UpstreamTokenizerIncomplete
                    ),
                    "{id}: expected UpstreamTokenizerIncomplete termination"
                );
                assert_eq!(
                    result.execution_completion(),
                    CssParserExecutionCompletion::Incomplete,
                    "{id}: execution completion"
                );
            } else {
                assert!(
                    matches!(
                        result.termination(),
                        CssParserTermination::EndOfTokenizerInput
                    ),
                    "{id}: expected EndOfTokenizerInput termination"
                );
                assert_eq!(
                    result.execution_completion(),
                    CssParserExecutionCompletion::Complete,
                    "{id}: execution completion"
                );
            }
        }
    }

    assert_eq!(
        resources.value(CssParserResourceKind::ContextRecords),
        expected_context_records,
        "{id}: ContextRecords usage"
    );
    assert_eq!(
        resources.value(CssParserResourceKind::DeclarationOccurrences),
        expected_declaration_occurrences,
        "{id}: DeclarationOccurrences aggregate usage"
    );
    assert_eq!(
        resources.value(CssParserResourceKind::UnsupportedRegions),
        expected_top_level.len() + expected_nested_unsupported_regions,
        "{id}: UnsupportedRegions usage"
    );
}

#[allow(clippy::too_many_arguments)]
fn assert_occurrence_syntax_matches(
    id: &str,
    label: &str,
    complete: &crate::SourceAnchor,
    name: &crate::SourceAnchor,
    colon: &crate::SourceAnchor,
    value: &crate::SourceAnchor,
    priority: Option<&crate::css::declaration::CssDeclarationPriorityEvidence>,
    termination: &CssDeclarationTermination,
    expected: &DescriptorGoldOccurrence,
) {
    assert_eq!(
        (complete.range().start(), complete.range().end()),
        (expected.complete.start, expected.complete.end),
        "{id}: {label} complete"
    );
    assert_eq!(
        (name.range().start(), name.range().end()),
        (expected.name.start, expected.name.end),
        "{id}: {label} name"
    );
    assert_eq!(
        (colon.range().start(), colon.range().end()),
        (expected.colon.start, expected.colon.end),
        "{id}: {label} colon"
    );
    assert_eq!(
        (value.range().start(), value.range().end()),
        (expected.value.start, expected.value.end),
        "{id}: {label} value"
    );
    match (priority, expected.priority) {
        (None, None) => {}
        (Some(actual), Some(expected_priority)) => {
            assert_eq!(
                (
                    actual.complete().range().start(),
                    actual.complete().range().end()
                ),
                (
                    expected_priority.complete.start,
                    expected_priority.complete.end
                ),
                "{id}: {label} priority complete"
            );
        }
        _ => panic!("{id}: {label} priority presence mismatch"),
    }
    match (termination, expected.termination) {
        (
            CssDeclarationTermination::AuthoredSemicolon { semicolon },
            DescriptorGoldOccurrenceTermination::AuthoredSemicolon(range),
        ) => assert_range(id, &format!("{label} termination"), 0, semicolon, range),
        (
            CssDeclarationTermination::OmittedBeforeRightCurly { right_curly },
            DescriptorGoldOccurrenceTermination::OmittedBeforeRightCurly(range),
        ) => assert_range(id, &format!("{label} termination"), 0, right_curly, range),
        (
            CssDeclarationTermination::OmittedAtEndOfInput { terminal },
            DescriptorGoldOccurrenceTermination::OmittedAtEndOfInput(range),
        ) => assert_range(id, &format!("{label} termination"), 0, terminal, range),
        _ => panic!("{id}: {label} termination kind mismatch"),
    }
}

fn assert_descriptor_kind_matches(
    id: &str,
    index: usize,
    actual: CssParserContextKind,
    expected: DescriptorGoldRuleKind,
) {
    let matches = matches!(
        (actual, expected),
        (
            CssParserContextKind::DescriptorRuleBlock(CssParserDescriptorRuleKind::FontFace),
            DescriptorGoldRuleKind::FontFace
        ) | (
            CssParserContextKind::DescriptorRuleBlock(CssParserDescriptorRuleKind::Property),
            DescriptorGoldRuleKind::Property
        )
    );
    assert!(matches, "{id}: context {index} descriptor kind mismatch");
}

fn assert_range(
    id: &str,
    label: &str,
    index: usize,
    actual: &crate::SourceAnchor,
    expected: GoldRange,
) {
    let range = actual.range();
    assert_eq!(
        (range.start(), range.end()),
        (expected.start, expected.end),
        "{id}: {label} {index}"
    );
}

fn assert_descriptor_termination_matches(
    id: &str,
    index: usize,
    actual: &CssParserContextTermination,
    expected: DescriptorGoldTermination,
) {
    match (actual, expected) {
        (
            CssParserContextTermination::AuthoredRightCurly { right_curly },
            DescriptorGoldTermination::AuthoredRightCurly(range),
        ) => assert_range(id, "context termination", index, right_curly, range),
        (
            CssParserContextTermination::EndOfInput { terminal },
            DescriptorGoldTermination::EndOfInput(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        (
            CssParserContextTermination::UpstreamTokenizerIncomplete { terminal },
            DescriptorGoldTermination::UpstreamTokenizerIncomplete(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        (
            CssParserContextTermination::ParserResourceLimit { terminal },
            DescriptorGoldTermination::ParserResourceLimit(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        _ => panic!("{id}: context {index} termination kind mismatch"),
    }
}

fn assert_recovery_termination_matches(
    id: &str,
    index: usize,
    actual: &CssParserRecoveryTermination,
    expected: DescriptorGoldRecoveryTermination,
) {
    match (actual, expected) {
        (
            CssParserRecoveryTermination::AuthoredSemicolon { semicolon },
            DescriptorGoldRecoveryTermination::AuthoredSemicolon(range),
        ) => assert_range(id, "recovery termination", index, semicolon, range),
        (
            CssParserRecoveryTermination::EnclosingBlockEnd { right_curly },
            DescriptorGoldRecoveryTermination::EnclosingBlockEnd(range),
        ) => assert_range(id, "recovery termination", index, right_curly, range),
        (
            CssParserRecoveryTermination::EndOfInput { terminal },
            DescriptorGoldRecoveryTermination::EndOfInput(range),
        ) => assert_range(id, "recovery termination", index, terminal, range),
        _ => panic!("{id}: recovery {index} termination kind mismatch"),
    }
}
