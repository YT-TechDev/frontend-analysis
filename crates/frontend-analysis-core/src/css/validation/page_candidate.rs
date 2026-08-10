//! Test-only production candidate adapter comparing the #170 producer
//! against the candidate-independent `@page`/page-margin gold model
//! (`page_gold.rs` / `page_fixtures.rs`).
//!
//! Gold is never generated from or rewritten by production output; this
//! module only reads production `CssParserRunResult` values to compare them
//! against the independently authored fixtures. Production code must never
//! import this module or the gold model.

use crate::css::declaration::{
    CssDeclarationOccurrence, CssDeclarationTermination, CssDescriptorOccurrence,
    CssPageDeclarationOccurrence, CssPageMarginDeclarationOccurrence,
};
use crate::css::parser::context::{
    CssParserContextKind, CssParserContextRecord, CssParserContextTermination,
    CssParserDescriptorRuleKind, CssParserPageMarginRuleKind,
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

use super::gold::GoldRange;
use super::page_gold::{
    PageGoldContext, PageGoldFixture, PageGoldMarginKind, PageGoldOccurrence,
    PageGoldOccurrenceTermination, PageGoldRecoveryTermination, PageGoldResourceKind, PageGoldRoot,
    PageGoldTermination,
};

/// No page fixture requires a contract-only (production-execution-skipped)
/// recipe: unlike #169's generic-parser-resource-stop category, every #170
/// category in this inventory has a deterministic, architecture-specified
/// exact terminal reachable through ordinary gold-fixture execution. The
/// generic post-entry resource-stop categories (34/35) are instead proved by
/// dedicated tests in `page_lifecycle_validation_tests.rs`, mirroring #169's
/// own precedent for its analogous category 17.
pub(super) const PAGE_CONTRACT_ONLY_FIXTURE_IDS: &[&str] = &[];

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

pub(super) fn tight_limits_for(
    expectation: super::page_gold::PageGoldResourceExpectation,
) -> CssParserLimits {
    let mut max_algorithm_steps = 1 << 20;
    let mut max_peak_context_depth = 1 << 12;
    let mut max_declaration_occurrences = 1 << 16;
    let mut max_recovery_records = 1 << 16;
    let mut max_unsupported_regions = 1 << 16;
    let mut max_context_records = 1 << 16;
    match expectation.kind {
        PageGoldResourceKind::AlgorithmSteps => max_algorithm_steps = expectation.limit.max(1),
        PageGoldResourceKind::PeakContextDepth => max_peak_context_depth = expectation.limit,
        PageGoldResourceKind::ContextRecords => max_context_records = expectation.limit,
        PageGoldResourceKind::DeclarationOccurrences => {
            max_declaration_occurrences = expectation.limit;
        }
        PageGoldResourceKind::RecoveryRecords => max_recovery_records = expectation.limit,
        PageGoldResourceKind::UnsupportedRegions => max_unsupported_regions = expectation.limit,
    }
    CssParserLimits::new(
        max_algorithm_steps,
        1 << 12,
        max_peak_context_depth,
        max_declaration_occurrences,
        1 << 16,
        max_recovery_records,
        max_unsupported_regions,
        1 << 16,
        max_context_records,
    )
    .unwrap()
}

/// Runs `fixture` against real production, selecting the execution recipe
/// its own resource expectation requires.
pub(super) fn run_fixture(fixture: &PageGoldFixture) -> CssParserRunResult {
    assert!(
        !PAGE_CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id),
        "{}: contract-only fixture has no production execution recipe",
        fixture.id
    );

    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits())
        .unwrap_or_else(|error| panic!("{}: production tokenizer error: {error:?}", fixture.id));

    let limits = match fixture.resource_expectation {
        Some(expectation) => tight_limits_for(expectation),
        None => generous_parser_limits(),
    };

    run(&source, tokenizer_result, limits)
        .unwrap_or_else(|error| panic!("{}: production parser error: {error:?}", fixture.id))
}

/// Compares production `result` against `fixture`'s independent `@page`/
/// page-margin expectations. Panics (as test-only wiring) with a descriptive
/// message on any mismatch.
pub(super) fn assert_matches_page_gold(fixture: &PageGoldFixture, result: &CssParserRunResult) {
    let id = fixture.id;
    let records = result.context_records();

    let expected_context_count: usize = fixture
        .roots
        .iter()
        .map(|root| match root {
            PageGoldRoot::Page(context) => 1 + context.margins.len(),
            PageGoldRoot::Qualified(_) | PageGoldRoot::Descriptor(_) => 1,
            PageGoldRoot::UnsupportedAtRule { .. } => 0,
        })
        .sum();
    assert_eq!(
        records.len(),
        expected_context_count,
        "{id}: context record count"
    );

    for root in &fixture.roots {
        match root {
            PageGoldRoot::Page(context) => {
                assert_page_context_matches(id, records, context);
            }
            PageGoldRoot::Qualified(companion) => {
                let actual = &records[companion.id];
                assert!(
                    actual.parent().is_none(),
                    "{id}: companion {} root",
                    companion.id
                );
                assert_eq!(
                    actual.kind(),
                    CssParserContextKind::QualifiedRuleBlock,
                    "{id}: companion {} kind",
                    companion.id
                );
                assert_eq!(
                    actual.item_ordinal().value(),
                    companion.root_item_ordinal,
                    "{id}"
                );
                assert_range(
                    id,
                    "companion header",
                    companion.id,
                    actual.header(),
                    companion.header,
                );
                assert_range(
                    id,
                    "companion block_opener",
                    companion.id,
                    actual.block_opener(),
                    companion.block_opener,
                );
                assert_range(
                    id,
                    "companion body",
                    companion.id,
                    actual.body(),
                    companion.body,
                );
                assert_termination_matches(
                    id,
                    companion.id,
                    actual.termination(),
                    companion.termination,
                );
            }
            PageGoldRoot::Descriptor(companion) => {
                let actual = &records[companion.id];
                assert!(
                    actual.parent().is_none(),
                    "{id}: descriptor companion {} root",
                    companion.id
                );
                assert_eq!(
                    actual.kind(),
                    CssParserContextKind::DescriptorRuleBlock(
                        CssParserDescriptorRuleKind::FontFace
                    ),
                    "{id}: descriptor companion {} kind",
                    companion.id
                );
                assert_eq!(
                    actual.item_ordinal().value(),
                    companion.root_item_ordinal,
                    "{id}"
                );
                let at_keyword = actual.at_keyword().unwrap_or_else(|| {
                    panic!(
                        "{id}: descriptor companion {} missing at_keyword",
                        companion.id
                    )
                });
                assert_range(
                    id,
                    "descriptor at_keyword",
                    companion.id,
                    at_keyword,
                    companion.at_keyword,
                );
                assert_range(
                    id,
                    "descriptor header",
                    companion.id,
                    actual.header(),
                    companion.header,
                );
                assert_range(
                    id,
                    "descriptor block_opener",
                    companion.id,
                    actual.block_opener(),
                    companion.block_opener,
                );
                assert_range(
                    id,
                    "descriptor body",
                    companion.id,
                    actual.body(),
                    companion.body,
                );
                assert_termination_matches(
                    id,
                    companion.id,
                    actual.termination(),
                    companion.termination,
                );
            }
            PageGoldRoot::UnsupportedAtRule { .. } => {}
        }
    }

    // Top-level unsupported at-rules.
    let expected_top_level: Vec<(GoldRange, GoldRange)> = fixture
        .roots
        .iter()
        .filter_map(|root| match root {
            PageGoldRoot::UnsupportedAtRule {
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

    // Ordinary declarations, grouped by owning context (Qualified companions).
    let mut ordinary_by_context: std::collections::BTreeMap<usize, Vec<&CssDeclarationOccurrence>> =
        std::collections::BTreeMap::new();
    for occurrence in result.occurrences() {
        ordinary_by_context
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
        if let PageGoldRoot::Qualified(companion) = root {
            let actual = ordinary_by_context
                .remove(&companion.id)
                .unwrap_or_default();
            assert_declarations_match(
                id,
                "companion",
                companion.id,
                &actual,
                &companion.declarations,
            );
            let actual_nested = nested_unsupported_by_context
                .remove(&companion.id)
                .unwrap_or_default();
            assert_nested_unsupported_matches(
                id,
                "companion",
                companion.id,
                &actual_nested,
                &companion.nested_unsupported,
            );
        }
    }
    assert!(
        ordinary_by_context.is_empty(),
        "{id}: production emitted ordinary declarations in unexpected contexts: {ordinary_by_context:?}"
    );

    // Descriptor occurrences, grouped by owning context (Descriptor companions).
    let mut descriptor_by_context: std::collections::BTreeMap<
        usize,
        Vec<&CssDescriptorOccurrence>,
    > = std::collections::BTreeMap::new();
    for occurrence in result.descriptor_occurrences() {
        descriptor_by_context
            .entry(occurrence.placement().context_id().index())
            .or_default()
            .push(occurrence);
    }
    for root in &fixture.roots {
        if let PageGoldRoot::Descriptor(companion) = root {
            let actual = descriptor_by_context
                .remove(&companion.id)
                .unwrap_or_default();
            assert_declarations_match(
                id,
                "descriptor companion",
                companion.id,
                &actual,
                &companion.occurrences,
            );
        }
    }
    assert!(
        descriptor_by_context.is_empty(),
        "{id}: production emitted descriptor occurrences in unexpected contexts: {descriptor_by_context:?}"
    );

    // Page declarations, grouped by owning context.
    let mut page_by_context: std::collections::BTreeMap<usize, Vec<&CssPageDeclarationOccurrence>> =
        std::collections::BTreeMap::new();
    for occurrence in result.page_occurrences() {
        page_by_context
            .entry(occurrence.placement().context_id().index())
            .or_default()
            .push(occurrence);
    }
    // Page-margin declarations, grouped by owning context.
    let mut page_margin_by_context: std::collections::BTreeMap<
        usize,
        Vec<&CssPageMarginDeclarationOccurrence>,
    > = std::collections::BTreeMap::new();
    for occurrence in result.page_margin_occurrences() {
        page_margin_by_context
            .entry(occurrence.placement().context_id().index())
            .or_default()
            .push(occurrence);
    }
    for root in &fixture.roots {
        if let PageGoldRoot::Page(context) = root {
            let actual_declarations = page_by_context.remove(&context.id).unwrap_or_default();
            assert_declarations_match(
                id,
                "page context",
                context.id,
                &actual_declarations,
                &context.declarations,
            );

            let actual_nested = nested_unsupported_by_context
                .remove(&context.id)
                .unwrap_or_default();
            assert_nested_unsupported_matches(
                id,
                "page context",
                context.id,
                &actual_nested,
                &context.nested_unsupported,
            );

            for margin in &context.margins {
                let actual_margin_declarations = page_margin_by_context
                    .remove(&margin.id)
                    .unwrap_or_default();
                assert_declarations_match(
                    id,
                    "margin context",
                    margin.id,
                    &actual_margin_declarations,
                    &margin.declarations,
                );
                let actual_margin_nested = nested_unsupported_by_context
                    .remove(&margin.id)
                    .unwrap_or_default();
                assert_nested_unsupported_matches(
                    id,
                    "margin context",
                    margin.id,
                    &actual_margin_nested,
                    &margin.nested_unsupported,
                );
            }
        }
    }
    assert!(
        page_by_context.is_empty(),
        "{id}: production emitted page declarations in unexpected contexts: {page_by_context:?}"
    );
    assert!(
        page_margin_by_context.is_empty(),
        "{id}: production emitted page-margin declarations in unexpected contexts: {page_margin_by_context:?}"
    );
    assert!(
        nested_unsupported_by_context.is_empty(),
        "{id}: production emitted nested unsupported at-rules in unexpected contexts: {nested_unsupported_by_context:?}"
    );

    // Diagnostics / recovery (shared, pre-#170 vocabulary).
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

    // Resource usage.
    let resources = result.resources();
    let expected_declaration_occurrences: usize = fixture
        .roots
        .iter()
        .map(|root| match root {
            PageGoldRoot::Page(context) => {
                context.declarations.len()
                    + context
                        .margins
                        .iter()
                        .map(|margin| margin.declarations.len())
                        .sum::<usize>()
            }
            PageGoldRoot::Qualified(companion) => companion.declarations.len(),
            PageGoldRoot::Descriptor(companion) => companion.occurrences.len(),
            PageGoldRoot::UnsupportedAtRule { .. } => 0,
        })
        .sum();
    let expected_nested_unsupported_regions: usize = fixture
        .roots
        .iter()
        .map(|root| match root {
            PageGoldRoot::Page(context) => {
                context.nested_unsupported.len()
                    + context
                        .margins
                        .iter()
                        .map(|margin| margin.nested_unsupported.len())
                        .sum::<usize>()
            }
            PageGoldRoot::Qualified(companion) => companion.nested_unsupported.len(),
            PageGoldRoot::Descriptor(_) | PageGoldRoot::UnsupportedAtRule { .. } => 0,
        })
        .sum();

    match fixture.resource_expectation {
        Some(expectation) => {
            let CssParserTermination::ParserResourceLimit(evidence) = result.termination() else {
                panic!("{id}: expected ParserResourceLimit termination");
            };
            let expected_kind = match expectation.kind {
                PageGoldResourceKind::AlgorithmSteps => CssParserResourceKind::AlgorithmSteps,
                PageGoldResourceKind::PeakContextDepth => CssParserResourceKind::PeakContextDepth,
                PageGoldResourceKind::ContextRecords => CssParserResourceKind::ContextRecords,
                PageGoldResourceKind::DeclarationOccurrences => {
                    CssParserResourceKind::DeclarationOccurrences
                }
                PageGoldResourceKind::RecoveryRecords => CssParserResourceKind::RecoveryRecords,
                PageGoldResourceKind::UnsupportedRegions => {
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
            let expects_upstream_incomplete = fixture.roots.iter().any(|root| match root {
                PageGoldRoot::Page(context) => {
                    matches!(
                        context.termination,
                        PageGoldTermination::UpstreamTokenizerIncomplete(_)
                    ) || context.margins.iter().any(|margin| {
                        matches!(
                            margin.termination,
                            PageGoldTermination::UpstreamTokenizerIncomplete(_)
                        )
                    })
                }
                _ => false,
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
                    "{id}"
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
                    "{id}"
                );
            }
        }
    }

    assert_eq!(
        resources.value(CssParserResourceKind::ContextRecords),
        expected_context_count,
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

fn assert_page_context_matches(
    id: &str,
    records: &[CssParserContextRecord],
    context: &PageGoldContext,
) {
    let actual = &records[context.id];
    assert!(
        actual.parent().is_none(),
        "{id}: page context {} must be root-owned",
        context.id
    );
    assert_eq!(
        actual.kind(),
        CssParserContextKind::PageRuleBlock,
        "{id}: page context {} kind",
        context.id
    );
    assert_eq!(
        actual.item_ordinal().value(),
        context.root_item_ordinal,
        "{id}: page context {} root ordinal",
        context.id
    );
    let at_keyword = actual
        .at_keyword()
        .unwrap_or_else(|| panic!("{id}: page context {} missing at_keyword", context.id));
    assert_range(
        id,
        "page at_keyword",
        context.id,
        at_keyword,
        context.at_keyword,
    );
    match (actual.page_selector_list(), context.page_selector_list) {
        (None, None) => {}
        (Some(actual_list), Some(expected_list)) => {
            assert_range(
                id,
                "page_selector_list",
                context.id,
                actual_list,
                expected_list,
            );
        }
        _ => panic!(
            "{id}: page context {} page_selector_list presence mismatch",
            context.id
        ),
    }
    assert_range(
        id,
        "page header",
        context.id,
        actual.header(),
        context.header,
    );
    assert_range(
        id,
        "page block_opener",
        context.id,
        actual.block_opener(),
        context.block_opener,
    );
    assert_range(id, "page body", context.id, actual.body(), context.body);
    assert_termination_matches(id, context.id, actual.termination(), context.termination);

    for margin in &context.margins {
        let actual_margin = &records[margin.id];
        assert_eq!(
            actual_margin.parent(),
            Some(actual.id()),
            "{id}: margin {} parent",
            margin.id
        );
        assert_margin_kind_matches(id, margin.id, actual_margin.kind(), margin.kind);
        assert_eq!(
            actual_margin.item_ordinal().value(),
            margin.item_ordinal,
            "{id}: margin {} item_ordinal",
            margin.id
        );
        let margin_at_keyword = actual_margin
            .at_keyword()
            .unwrap_or_else(|| panic!("{id}: margin {} missing at_keyword", margin.id));
        assert_range(
            id,
            "margin at_keyword",
            margin.id,
            margin_at_keyword,
            margin.at_keyword,
        );
        assert!(
            actual_margin.page_selector_list().is_none(),
            "{id}: margin {} carries no selector list",
            margin.id
        );
        assert_range(
            id,
            "margin header",
            margin.id,
            actual_margin.header(),
            margin.header,
        );
        assert_range(
            id,
            "margin block_opener",
            margin.id,
            actual_margin.block_opener(),
            margin.block_opener,
        );
        assert_range(
            id,
            "margin body",
            margin.id,
            actual_margin.body(),
            margin.body,
        );
        assert_termination_matches(
            id,
            margin.id,
            actual_margin.termination(),
            margin.termination,
        );
    }
}

fn assert_declarations_match<T>(
    id: &str,
    label: &str,
    context_id: usize,
    actual: &[&T],
    expected: &[PageGoldOccurrence],
) where
    T: PageDeclarationLike,
{
    assert_eq!(
        actual.len(),
        expected.len(),
        "{id}: {label} {context_id} declaration count"
    );
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.item_ordinal(),
            expected.item_ordinal,
            "{id}: {label} {context_id} declaration {index} item_ordinal"
        );
        assert_occurrence_syntax_matches(
            id,
            &format!("{label} {context_id} declaration {index}"),
            actual.complete(),
            actual.name(),
            actual.colon(),
            actual.value(),
            actual.priority(),
            actual.termination(),
            expected,
        );
    }
}

/// Shared accessor surface for the four semantically distinct
/// `CssDeclarationSyntaxEvidence`-sharing occurrence types, used only to let
/// [`assert_declarations_match`] compare any of them against one
/// [`PageGoldOccurrence`] shape without duplicating its body four times.
trait PageDeclarationLike {
    fn item_ordinal(&self) -> usize;
    fn complete(&self) -> &crate::SourceAnchor;
    fn name(&self) -> &crate::SourceAnchor;
    fn colon(&self) -> &crate::SourceAnchor;
    fn value(&self) -> &crate::SourceAnchor;
    fn priority(&self) -> Option<&crate::css::declaration::CssDeclarationPriorityEvidence>;
    fn termination(&self) -> &CssDeclarationTermination;
}

impl PageDeclarationLike for CssDeclarationOccurrence {
    fn item_ordinal(&self) -> usize {
        self.placement().item_ordinal().value()
    }
    fn complete(&self) -> &crate::SourceAnchor {
        CssDeclarationOccurrence::complete(self)
    }
    fn name(&self) -> &crate::SourceAnchor {
        self.property_name()
    }
    fn colon(&self) -> &crate::SourceAnchor {
        CssDeclarationOccurrence::colon(self)
    }
    fn value(&self) -> &crate::SourceAnchor {
        CssDeclarationOccurrence::value(self)
    }
    fn priority(&self) -> Option<&crate::css::declaration::CssDeclarationPriorityEvidence> {
        CssDeclarationOccurrence::priority(self)
    }
    fn termination(&self) -> &CssDeclarationTermination {
        CssDeclarationOccurrence::termination(self)
    }
}

impl PageDeclarationLike for CssDescriptorOccurrence {
    fn item_ordinal(&self) -> usize {
        self.placement().item_ordinal().value()
    }
    fn complete(&self) -> &crate::SourceAnchor {
        CssDescriptorOccurrence::complete(self)
    }
    fn name(&self) -> &crate::SourceAnchor {
        CssDescriptorOccurrence::name(self)
    }
    fn colon(&self) -> &crate::SourceAnchor {
        CssDescriptorOccurrence::colon(self)
    }
    fn value(&self) -> &crate::SourceAnchor {
        CssDescriptorOccurrence::value(self)
    }
    fn priority(&self) -> Option<&crate::css::declaration::CssDeclarationPriorityEvidence> {
        CssDescriptorOccurrence::priority(self)
    }
    fn termination(&self) -> &CssDeclarationTermination {
        CssDescriptorOccurrence::termination(self)
    }
}

impl PageDeclarationLike for CssPageDeclarationOccurrence {
    fn item_ordinal(&self) -> usize {
        self.placement().item_ordinal().value()
    }
    fn complete(&self) -> &crate::SourceAnchor {
        CssPageDeclarationOccurrence::complete(self)
    }
    fn name(&self) -> &crate::SourceAnchor {
        CssPageDeclarationOccurrence::name(self)
    }
    fn colon(&self) -> &crate::SourceAnchor {
        CssPageDeclarationOccurrence::colon(self)
    }
    fn value(&self) -> &crate::SourceAnchor {
        CssPageDeclarationOccurrence::value(self)
    }
    fn priority(&self) -> Option<&crate::css::declaration::CssDeclarationPriorityEvidence> {
        CssPageDeclarationOccurrence::priority(self)
    }
    fn termination(&self) -> &CssDeclarationTermination {
        CssPageDeclarationOccurrence::termination(self)
    }
}

impl PageDeclarationLike for CssPageMarginDeclarationOccurrence {
    fn item_ordinal(&self) -> usize {
        self.placement().item_ordinal().value()
    }
    fn complete(&self) -> &crate::SourceAnchor {
        CssPageMarginDeclarationOccurrence::complete(self)
    }
    fn name(&self) -> &crate::SourceAnchor {
        CssPageMarginDeclarationOccurrence::name(self)
    }
    fn colon(&self) -> &crate::SourceAnchor {
        CssPageMarginDeclarationOccurrence::colon(self)
    }
    fn value(&self) -> &crate::SourceAnchor {
        CssPageMarginDeclarationOccurrence::value(self)
    }
    fn priority(&self) -> Option<&crate::css::declaration::CssDeclarationPriorityEvidence> {
        CssPageMarginDeclarationOccurrence::priority(self)
    }
    fn termination(&self) -> &CssDeclarationTermination {
        CssPageMarginDeclarationOccurrence::termination(self)
    }
}

fn assert_nested_unsupported_matches(
    id: &str,
    label: &str,
    context_id: usize,
    actual: &[(&crate::SourceAnchor, &crate::SourceAnchor, usize)],
    expected: &[super::page_gold::PageGoldNestedUnsupportedAtRule],
) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{id}: {label} {context_id} nested unsupported count"
    );
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        let (complete, at_keyword, item_ordinal) = *actual;
        assert_range(
            id,
            "nested unsupported complete",
            index,
            complete,
            expected.complete,
        );
        assert_range(
            id,
            "nested unsupported at_keyword",
            index,
            at_keyword,
            expected.at_keyword,
        );
        assert_eq!(
            item_ordinal, expected.item_ordinal,
            "{id}: {label} {context_id} nested unsupported {index} item_ordinal"
        );
    }
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
    expected: &PageGoldOccurrence,
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
            PageGoldOccurrenceTermination::AuthoredSemicolon(range),
        ) => assert_range(id, &format!("{label} termination"), 0, semicolon, range),
        (
            CssDeclarationTermination::OmittedBeforeRightCurly { right_curly },
            PageGoldOccurrenceTermination::OmittedBeforeRightCurly(range),
        ) => assert_range(id, &format!("{label} termination"), 0, right_curly, range),
        (
            CssDeclarationTermination::OmittedAtEndOfInput { terminal },
            PageGoldOccurrenceTermination::OmittedAtEndOfInput(range),
        ) => assert_range(id, &format!("{label} termination"), 0, terminal, range),
        _ => panic!("{id}: {label} termination kind mismatch"),
    }
}

fn assert_margin_kind_matches(
    id: &str,
    index: usize,
    actual: CssParserContextKind,
    expected: PageGoldMarginKind,
) {
    let matches = matches!(
        (actual, expected),
        (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::TopLeftCorner),
            PageGoldMarginKind::TopLeftCorner
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::TopLeft),
            PageGoldMarginKind::TopLeft
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::TopCenter),
            PageGoldMarginKind::TopCenter
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::TopRight),
            PageGoldMarginKind::TopRight
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::TopRightCorner),
            PageGoldMarginKind::TopRightCorner
        ) | (
            CssParserContextKind::PageMarginRuleBlock(
                CssParserPageMarginRuleKind::BottomLeftCorner
            ),
            PageGoldMarginKind::BottomLeftCorner
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::BottomLeft),
            PageGoldMarginKind::BottomLeft
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::BottomCenter),
            PageGoldMarginKind::BottomCenter
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::BottomRight),
            PageGoldMarginKind::BottomRight
        ) | (
            CssParserContextKind::PageMarginRuleBlock(
                CssParserPageMarginRuleKind::BottomRightCorner
            ),
            PageGoldMarginKind::BottomRightCorner
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::LeftTop),
            PageGoldMarginKind::LeftTop
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::LeftMiddle),
            PageGoldMarginKind::LeftMiddle
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::LeftBottom),
            PageGoldMarginKind::LeftBottom
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::RightTop),
            PageGoldMarginKind::RightTop
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::RightMiddle),
            PageGoldMarginKind::RightMiddle
        ) | (
            CssParserContextKind::PageMarginRuleBlock(CssParserPageMarginRuleKind::RightBottom),
            PageGoldMarginKind::RightBottom
        )
    );
    assert!(
        matches,
        "{id}: margin {index} kind mismatch: actual {actual:?}, expected {expected:?}"
    );
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

fn assert_termination_matches(
    id: &str,
    index: usize,
    actual: &CssParserContextTermination,
    expected: PageGoldTermination,
) {
    match (actual, expected) {
        (
            CssParserContextTermination::AuthoredRightCurly { right_curly },
            PageGoldTermination::AuthoredRightCurly(range),
        ) => assert_range(id, "context termination", index, right_curly, range),
        (
            CssParserContextTermination::EndOfInput { terminal },
            PageGoldTermination::EndOfInput(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        (
            CssParserContextTermination::UpstreamTokenizerIncomplete { terminal },
            PageGoldTermination::UpstreamTokenizerIncomplete(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        (
            CssParserContextTermination::ParserResourceLimit { terminal },
            PageGoldTermination::ParserResourceLimit(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        _ => panic!("{id}: context {index} termination kind mismatch"),
    }
}

fn assert_recovery_termination_matches(
    id: &str,
    index: usize,
    actual: &CssParserRecoveryTermination,
    expected: PageGoldRecoveryTermination,
) {
    match (actual, expected) {
        (
            CssParserRecoveryTermination::AuthoredSemicolon { semicolon },
            PageGoldRecoveryTermination::AuthoredSemicolon(range),
        ) => assert_range(id, "recovery termination", index, semicolon, range),
        (
            CssParserRecoveryTermination::EnclosingBlockEnd { right_curly },
            PageGoldRecoveryTermination::EnclosingBlockEnd(range),
        ) => assert_range(id, "recovery termination", index, right_curly, range),
        (
            CssParserRecoveryTermination::EndOfInput { terminal },
            PageGoldRecoveryTermination::EndOfInput(range),
        ) => assert_range(id, "recovery termination", index, terminal, range),
        _ => panic!("{id}: recovery {index} termination kind mismatch"),
    }
}
