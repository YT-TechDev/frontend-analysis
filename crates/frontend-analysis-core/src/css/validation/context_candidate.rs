//! Test-only production candidate adapter comparing the #167 context-aware
//! producer against the #166 candidate-independent context/order/resource
//! gold model (`context_gold.rs` / `context_fixtures.rs`).
//!
//! Gold is never generated from or rewritten by production output; this
//! module only reads production `CssParserRunResult` values to compare them
//! against the independently authored fixtures. Production code must never
//! import this module or the gold model.
//!
//! # Execution classification
//!
//! Not every one of the 16 independent context fixtures is an ordinary
//! source-driven complete parser run:
//!
//! - ordinary source-driven fixtures execute the real tokenizer then the
//!   real parser under generous finite limits;
//! - the two `UpstreamTokenizerIncomplete` fixtures whose terminal a normal
//!   generous tokenizer cannot naturally reproduce (the boundary sits at, or
//!   short of, what a real tokenizer would otherwise complete past) use a
//!   parser-level contract-valid synthetic upstream result, mirroring
//!   `parser_candidate::run_fixture_with_synthetic_upstream_incomplete`;
//! - the two `PeakContextDepth` / `ContextRecords` refusal fixtures execute
//!   the real parser under a tight limit for the resource kind their own
//!   `resource_expectation` names;
//! - the two generic parser-resource partial-ancestry fixtures
//!   (`CSS-CONTEXT-PARSER-RESOURCE-LIMITED-TERMINATION-001` and
//!   `CSS-CONTEXT-NESTED-PARSER-RESOURCE-LIMITED-ANCESTRY-001`) intentionally
//!   do not prescribe a particular resource trigger and remain contract/gold
//!   self-validation fixtures only; this module never claims a production
//!   execution recipe for them.

use crate::css::parser::producer::run;
use crate::css::parser::resource::{CssParserLimits, CssParserResourceKind};
use crate::css::parser::result::{
    CssParserExecutionCompletion, CssParserRunResult, CssParserTermination,
};
use crate::css::token::{CssLexicalItem, CssToken, CssTokenKind};
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::css::tokenizer::resource::{CssTokenizerLimits, CssTokenizerResourceUsage};
use crate::css::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerRunResult, CssTokenizerTermination,
};
use crate::{SourceId, SourceText};

use super::context_gold::{ContextGoldFixture, ContextGoldResourceKind, ContextGoldTermination};

/// Fixture IDs intentionally excluded from production execution: generic
/// parser-resource partial-ancestry gold that does not prescribe a specific
/// resource trigger (see module docs).
pub(super) const CONTRACT_ONLY_FIXTURE_IDS: &[&str] = &[
    "CSS-CONTEXT-PARSER-RESOURCE-LIMITED-TERMINATION-001",
    "CSS-CONTEXT-NESTED-PARSER-RESOURCE-LIMITED-ANCESTRY-001",
];

/// Fixture IDs whose exact partial terminal a normal generous tokenizer
/// cannot naturally reproduce, requiring a parser-level contract-valid
/// synthetic upstream result instead.
const SYNTHETIC_UPSTREAM_FIXTURE_IDS: &[&str] = &[
    "CSS-CONTEXT-UPSTREAM-INCOMPLETE-TERMINATION-001",
    "CSS-CONTEXT-NESTED-UPSTREAM-INCOMPLETE-ANCESTRY-001",
];

fn generous_tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

fn generous_context_parser_limits() -> CssParserLimits {
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
/// its own resource expectation and termination shape require. Panics (as
/// test-only wiring) if called for a fixture listed in
/// [`CONTRACT_ONLY_FIXTURE_IDS`].
pub(super) fn run_fixture(fixture: &ContextGoldFixture) -> CssParserRunResult {
    assert!(
        !CONTRACT_ONLY_FIXTURE_IDS.contains(&fixture.id),
        "{}: contract-only fixture has no production execution recipe",
        fixture.id
    );

    if let Some(expectation) = fixture.resource_expectation {
        return run_with_tight_resource_limit(fixture, expectation.kind, expectation.limit);
    }

    if SYNTHETIC_UPSTREAM_FIXTURE_IDS.contains(&fixture.id) {
        let terminal_offset = fixture
            .contexts
            .iter()
            .find_map(|record| match record.termination {
                ContextGoldTermination::UpstreamTokenizerIncomplete(terminal) => {
                    Some(terminal.start)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("{}: expected an UpstreamTokenizerIncomplete context to derive the synthetic terminal", fixture.id));
        return run_with_synthetic_upstream_incomplete(fixture, terminal_offset);
    }

    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits())
        .unwrap_or_else(|error| panic!("{}: production tokenizer error: {error:?}", fixture.id));
    run(&source, tokenizer_result, generous_context_parser_limits())
        .unwrap_or_else(|error| panic!("{}: production parser error: {error:?}", fixture.id))
}

fn run_with_tight_resource_limit(
    fixture: &ContextGoldFixture,
    kind: ContextGoldResourceKind,
    limit: usize,
) -> CssParserRunResult {
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits())
        .unwrap_or_else(|error| panic!("{}: production tokenizer error: {error:?}", fixture.id));

    let (max_peak_context_depth, max_context_records) = match kind {
        ContextGoldResourceKind::PeakContextDepth => (limit, 1 << 16),
        ContextGoldResourceKind::ContextRecords => (1 << 12, limit),
    };
    let limits = CssParserLimits::new(
        1 << 20,
        1 << 12,
        max_peak_context_depth,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        max_context_records,
    )
    .unwrap();

    run(&source, tokenizer_result, limits)
        .unwrap_or_else(|error| panic!("{}: production parser error: {error:?}", fixture.id))
}

/// Mirrors `parser_candidate::run_fixture_with_synthetic_upstream_incomplete`:
/// constructs a contract-valid `CssTokenizerRunResult` truncated exactly at
/// `terminal_offset` (which a normal generous tokenizer run over this
/// fixture's own source would not naturally produce as an incomplete
/// terminal), then feeds it into the production parser.
fn run_with_synthetic_upstream_incomplete(
    fixture: &ContextGoldFixture,
    terminal_offset: usize,
) -> CssParserRunResult {
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());

    let mut lexical_items = Vec::new();
    let mut cursor = 0usize;
    for boundary in synthetic_lexical_boundaries(fixture.source, terminal_offset) {
        let fragment = &fixture.source[cursor..boundary];
        let kind = if fragment == "{" {
            CssTokenKind::LeftCurlyBracket
        } else if fragment == "}" {
            CssTokenKind::RightCurlyBracket
        } else {
            CssTokenKind::Ident(fragment.to_owned())
        };
        let anchor = source.anchor(cursor, boundary).unwrap();
        lexical_items.push(CssLexicalItem::SemanticToken(
            CssToken::new(&source, anchor, kind).unwrap(),
        ));
        cursor = boundary;
    }
    assert_eq!(
        cursor, terminal_offset,
        "{}: synthetic lexical coverage must reach the terminal exactly",
        fixture.id
    );

    let item_count = lexical_items.len();
    let resource_limit = crate::css::tokenizer::resource::CssTokenizerResourceLimitEvidence::new(
        &source,
        crate::css::tokenizer::resource::CssTokenizerResourceKind::AlgorithmSteps,
        1,
        2,
        source.anchor(terminal_offset, terminal_offset).unwrap(),
    )
    .unwrap();
    let tokenizer_result = CssTokenizerRunResult::new(
        &source,
        None,
        lexical_items,
        Vec::new(),
        source.anchor(0, terminal_offset).unwrap(),
        source
            .anchor(terminal_offset, fixture.source.len())
            .unwrap(),
        source.anchor(terminal_offset, terminal_offset).unwrap(),
        CssTokenizerCompletion::Incomplete,
        CssTokenizerTermination::ResourceLimit(resource_limit),
        CssTokenizerResourceUsage::new(fixture.source.len(), 1, item_count, 0, 0, 0),
    )
    .unwrap();

    run(&source, tokenizer_result, generous_context_parser_limits()).unwrap_or_else(|error| {
        panic!(
            "{}: production parser error on synthetic upstream: {error:?}",
            fixture.id
        )
    })
}

fn synthetic_lexical_boundaries(source: &str, terminal_offset: usize) -> Vec<usize> {
    let mut boundaries = Vec::new();
    let mut position = 0usize;
    while position < terminal_offset {
        let mut next = position + 1;
        while next < terminal_offset && !source.is_char_boundary(next) {
            next += 1;
        }
        boundaries.push(next);
        position = next;
    }
    boundaries
}

/// Compares production `result` against `fixture`'s independent context/
/// declaration/resource expectations. Panics (as test-only wiring) with a
/// descriptive message on any mismatch.
pub(super) fn assert_matches_context_gold(
    fixture: &ContextGoldFixture,
    result: &CssParserRunResult,
) {
    let id = fixture.id;

    assert_eq!(
        result.context_records().len(),
        fixture.contexts.len(),
        "{id}: context record count"
    );

    for (index, (actual, expected)) in result
        .context_records()
        .iter()
        .zip(&fixture.contexts)
        .enumerate()
    {
        assert_eq!(actual.id().index(), expected.id, "{id}: context {index} id");
        assert_eq!(
            actual.parent().map(|parent| parent.index()),
            expected.parent,
            "{id}: context {index} parent"
        );
        assert_eq!(
            actual.item_ordinal().value(),
            expected.item_ordinal,
            "{id}: context {index} item_ordinal"
        );
        assert_range(
            id,
            "context header",
            index,
            actual.header(),
            expected.header,
        );
        assert_range(
            id,
            "context block_opener",
            index,
            actual.block_opener(),
            expected.block_opener,
        );
        assert_range(id, "context body", index, actual.body(), expected.body);
        assert_termination_matches(id, index, actual.termination(), expected.termination);
    }

    // Reconcile declarations against the per-context expected declaration
    // list: production is a flat `occurrences()` vector, so group it by
    // owning context first.
    let mut by_context: std::collections::BTreeMap<
        usize,
        Vec<&crate::css::declaration::CssDeclarationOccurrence>,
    > = std::collections::BTreeMap::new();
    for occurrence in result.occurrences() {
        by_context
            .entry(occurrence.placement().context_id().index())
            .or_default()
            .push(occurrence);
    }

    for (context_index, expected_context) in fixture.contexts.iter().enumerate() {
        let actual_declarations = by_context.remove(&context_index).unwrap_or_default();
        assert_eq!(
            actual_declarations.len(),
            expected_context.declarations.len(),
            "{id}: context {context_index} declaration count"
        );
        for (decl_index, (actual, expected)) in actual_declarations
            .iter()
            .zip(&expected_context.declarations)
            .enumerate()
        {
            let placement = actual.placement();
            assert_eq!(
                placement.context_id().index(),
                context_index,
                "{id}: context {context_index} declaration {decl_index} context_id"
            );
            assert_eq!(
                placement.item_ordinal().value(),
                expected.item_ordinal,
                "{id}: context {context_index} declaration {decl_index} item_ordinal"
            );
            assert_eq!(
                placement.run_ordinal().value(),
                expected.run_ordinal,
                "{id}: context {context_index} declaration {decl_index} run_ordinal"
            );
            let range = actual.complete().range();
            assert_eq!(
                (range.start(), range.end()),
                (expected.span.start, expected.span.end),
                "{id}: context {context_index} declaration {decl_index} span"
            );
        }
    }
    assert!(
        by_context.is_empty(),
        "{id}: production emitted declarations in unexpected contexts: {by_context:?}",
    );

    let resources = result.resources();
    assert_eq!(
        resources.value(CssParserResourceKind::PeakContextDepth),
        fixture.expected_peak_context_depth,
        "{id}: PeakContextDepth usage"
    );
    assert_eq!(
        resources.value(CssParserResourceKind::ContextRecords),
        fixture.expected_context_record_count,
        "{id}: ContextRecords usage"
    );

    match fixture.resource_expectation {
        Some(expectation) => {
            let CssParserTermination::ParserResourceLimit(evidence) = result.termination() else {
                panic!("{id}: expected ParserResourceLimit termination");
            };
            let expected_kind = match expectation.kind {
                ContextGoldResourceKind::PeakContextDepth => {
                    CssParserResourceKind::PeakContextDepth
                }
                ContextGoldResourceKind::ContextRecords => CssParserResourceKind::ContextRecords,
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
            let expected_upstream_incomplete = fixture.contexts.iter().any(|record| {
                matches!(
                    record.termination,
                    ContextGoldTermination::UpstreamTokenizerIncomplete(_)
                )
            });
            if expected_upstream_incomplete {
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
}

fn assert_range(
    id: &str,
    label: &str,
    index: usize,
    actual: &crate::SourceAnchor,
    expected: super::gold::GoldRange,
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
    actual: &crate::css::parser::context::CssParserContextTermination,
    expected: ContextGoldTermination,
) {
    use crate::css::parser::context::CssParserContextTermination;
    match (actual, expected) {
        (
            CssParserContextTermination::AuthoredRightCurly { right_curly },
            ContextGoldTermination::AuthoredRightCurly(range),
        ) => assert_range(id, "context termination", index, right_curly, range),
        (
            CssParserContextTermination::EndOfInput { terminal },
            ContextGoldTermination::EndOfInput(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        (
            CssParserContextTermination::UpstreamTokenizerIncomplete { terminal },
            ContextGoldTermination::UpstreamTokenizerIncomplete(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        (
            CssParserContextTermination::ParserResourceLimit { terminal },
            ContextGoldTermination::ParserResourceLimit(range),
        ) => assert_range(id, "context termination", index, terminal, range),
        _ => panic!("{id}: context {index} termination kind mismatch"),
    }
}
