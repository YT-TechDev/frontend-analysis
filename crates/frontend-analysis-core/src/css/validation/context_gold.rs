//! Candidate-independent CSS parser context/order/resource gold data model
//! (#166).
//!
//! This is contract/gold data for the future #167 nested-context producer.
//! #166 implements no nested-context producer: fixtures here describe what
//! #167 must eventually produce, never what the current producer already
//! emits (the current producer's `context_records` is always empty; see
//! `css::parser::producer`). This module and its fixtures validate the gold
//! representation itself; producer-versus-gold comparison begins in #167.
//!
//! Structurally independent from production: fixtures are authored here
//! from the approved #166 architecture record, never from production
//! `CssParserContextRecord`, `CssParserContextId`, or `CssParserContextKind`,
//! a future producer, an external parser, CSSOM, or browser output. This
//! module must never import those production types.

use super::gold::GoldRange;
use crate::{SourceId, SourceText};

fn anchor_ok(source: &SourceText, range: GoldRange) -> bool {
    source.anchor(range.start, range.end).is_ok()
}

fn fragment_is(source: &SourceText, range: GoldRange, expected: &str) -> bool {
    source
        .anchor(range.start, range.end)
        .is_ok_and(|anchor| anchor.fragment() == expected)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ContextGoldGroup {
    TopLevel,
    DeclarationOnly,
    MixedDeclarationsAndRules,
    RecursiveNesting,
    Termination,
    DuplicateSpelling,
    CustomProperty,
    Malformed,
    ResourceLimit,
}

/// Independent context-kind vocabulary. Mirrors only the #166 production
/// enum's sole variant; not the production type itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContextGoldKind {
    QualifiedRuleBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContextGoldTermination {
    AuthoredRightCurly(GoldRange),
    EndOfInput(GoldRange),
    UpstreamTokenizerIncomplete(GoldRange),
    ParserResourceLimit(GoldRange),
}

impl ContextGoldTermination {
    pub(super) const fn evidence_range(self) -> GoldRange {
        match self {
            Self::AuthoredRightCurly(range)
            | Self::EndOfInput(range)
            | Self::UpstreamTokenizerIncomplete(range)
            | Self::ParserResourceLimit(range) => range,
        }
    }
}

/// One future materialized declaration item's expected placement within its
/// owning context: parent-local direct-item ordinal, declaration-run
/// ordinal (scoped to that context), and an informational source span used
/// only to validate ordering/containment in this independent model. Mirrors
/// the #166-approved rule that declaration-run identity owns no authored
/// `SourceAnchor` of its own; `span` here is gold-only bookkeeping, not a
/// claimed production evidence field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ContextGoldDeclarationItem {
    pub(super) item_ordinal: usize,
    pub(super) run_ordinal: usize,
    pub(super) span: GoldRange,
}

/// One independent context record: identity, parent id, the direct-item
/// ordinal scoped to that parent (or to the implicit stylesheet root when
/// `parent` is `None`), kind, exact header/opener/body ranges, termination,
/// and the future materialized declaration items it directly owns.
///
/// Every record carries exactly one `item_ordinal`, including top-level
/// records: the implicit root is a genuine ordinal scope, never an
/// unordered special case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContextGoldRecord {
    pub(super) id: usize,
    pub(super) parent: Option<usize>,
    pub(super) item_ordinal: usize,
    pub(super) kind: ContextGoldKind,
    pub(super) header: GoldRange,
    pub(super) block_opener: GoldRange,
    pub(super) body: GoldRange,
    pub(super) termination: ContextGoldTermination,
    pub(super) declarations: Vec<ContextGoldDeclarationItem>,
}

impl ContextGoldRecord {
    fn extent_end(&self) -> usize {
        self.termination.evidence_range().end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContextGoldResourceKind {
    PeakContextDepth,
    ContextRecords,
}

/// Documents an expected `PeakContextDepth`/`ContextRecords` refusal: the
/// resource-limit fixtures (#166 categories 12/13) commit only the contexts
/// that survive, and record what the refused attempt would have been.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ContextGoldResourceExpectation {
    pub(super) kind: ContextGoldResourceKind,
    pub(super) limit: usize,
    pub(super) attempted: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContextGoldFixture {
    pub(super) id: &'static str,
    pub(super) group: ContextGoldGroup,
    pub(super) source_id: u64,
    pub(super) source: &'static str,
    pub(super) byte_len: usize,
    pub(super) contexts: Vec<ContextGoldRecord>,
    pub(super) expected_context_record_count: usize,
    pub(super) expected_peak_context_depth: usize,
    pub(super) resource_expectation: Option<ContextGoldResourceExpectation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContextGoldValidationError {
    ByteLengthMismatch,
    InvalidRange,
    IdIndexMismatch,
    OpenerNotExact,
    AuthoredCloserNotExact,
    HeaderOpenerBoundaryMismatch,
    BodyOpenerBoundaryMismatch,
    TerminationBoundaryMismatch,
    TerminationMustBeEmpty,
    EndOfInputNotAtSourceEnd,
    ParentNotBefore,
    ChildOutsideParentBody,
    DuplicateSiblingItemOrdinal,
    SiblingOrdinalOrderViolation,
    SiblingSourceOrderViolation,
    ContextRecordCountMismatch,
    PeakContextDepthMismatch,
    DeclarationOutsideBody,
    DuplicateDeclarationItemOrdinal,
    DeclarationOrderViolation,
    DeclarationRunOrdinalNotMonotonic,
    ResourceExpectationAttemptDidNotExceedLimit,
}

pub(super) fn validate_fixture(
    fixture: &ContextGoldFixture,
) -> Result<(), ContextGoldValidationError> {
    if fixture.source.len() != fixture.byte_len {
        return Err(ContextGoldValidationError::ByteLengthMismatch);
    }
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());

    if fixture.contexts.len() != fixture.expected_context_record_count {
        return Err(ContextGoldValidationError::ContextRecordCountMismatch);
    }

    let mut depths: Vec<usize> = Vec::with_capacity(fixture.contexts.len());
    // Keyed by ordinal scope: `Some(parent_id)` for a real parent, `None`
    // for the implicit stylesheet root. Applied uniformly to both so a
    // top-level sibling set is validated exactly like a real parent's
    // children.
    let mut last_sibling_ordinal: std::collections::BTreeMap<Option<usize>, usize> =
        std::collections::BTreeMap::new();
    let mut last_sibling_extent_end: std::collections::BTreeMap<Option<usize>, usize> =
        std::collections::BTreeMap::new();

    for (index, record) in fixture.contexts.iter().enumerate() {
        if record.id != index {
            return Err(ContextGoldValidationError::IdIndexMismatch);
        }

        validate_local_shape(&source, record)?;

        let extent_end = record.extent_end();
        let depth = if let Some(parent_id) = record.parent {
            if parent_id >= index {
                return Err(ContextGoldValidationError::ParentNotBefore);
            }
            let parent_record = &fixture.contexts[parent_id];
            if record.header.start < parent_record.body.start || extent_end > parent_record.body.end
            {
                return Err(ContextGoldValidationError::ChildOutsideParentBody);
            }

            depths[parent_id] + 1
        } else {
            1
        };
        depths.push(depth);

        let scope_key = record.parent;
        if let Some(&previous_ordinal) = last_sibling_ordinal.get(&scope_key) {
            if record.item_ordinal == previous_ordinal {
                return Err(ContextGoldValidationError::DuplicateSiblingItemOrdinal);
            }
            if record.item_ordinal < previous_ordinal {
                return Err(ContextGoldValidationError::SiblingOrdinalOrderViolation);
            }
        }
        if let Some(&previous_extent_end) = last_sibling_extent_end.get(&scope_key)
            && record.header.start < previous_extent_end
        {
            return Err(ContextGoldValidationError::SiblingSourceOrderViolation);
        }
        last_sibling_ordinal.insert(scope_key, record.item_ordinal);
        last_sibling_extent_end.insert(scope_key, extent_end);

        validate_declarations(&source, record)?;
    }

    let achieved_peak = depths.into_iter().max().unwrap_or(0);
    if achieved_peak != fixture.expected_peak_context_depth {
        return Err(ContextGoldValidationError::PeakContextDepthMismatch);
    }

    if let Some(expectation) = fixture.resource_expectation
        && expectation.attempted <= expectation.limit
    {
        return Err(ContextGoldValidationError::ResourceExpectationAttemptDidNotExceedLimit);
    }

    Ok(())
}

fn validate_local_shape(
    source: &SourceText,
    record: &ContextGoldRecord,
) -> Result<(), ContextGoldValidationError> {
    if !anchor_ok(source, record.header) {
        return Err(ContextGoldValidationError::InvalidRange);
    }
    if !anchor_ok(source, record.block_opener) || record.block_opener.is_empty() {
        return Err(ContextGoldValidationError::InvalidRange);
    }
    if !fragment_is(source, record.block_opener, "{") {
        return Err(ContextGoldValidationError::OpenerNotExact);
    }
    if record.header.end != record.block_opener.start {
        return Err(ContextGoldValidationError::HeaderOpenerBoundaryMismatch);
    }
    if !anchor_ok(source, record.body) {
        return Err(ContextGoldValidationError::InvalidRange);
    }
    if record.body.start != record.block_opener.end {
        return Err(ContextGoldValidationError::BodyOpenerBoundaryMismatch);
    }

    match record.termination {
        ContextGoldTermination::AuthoredRightCurly(right_curly) => {
            if !anchor_ok(source, right_curly) || right_curly.is_empty() {
                return Err(ContextGoldValidationError::InvalidRange);
            }
            if !fragment_is(source, right_curly, "}") {
                return Err(ContextGoldValidationError::AuthoredCloserNotExact);
            }
            if right_curly.start != record.body.end {
                return Err(ContextGoldValidationError::TerminationBoundaryMismatch);
            }
        }
        ContextGoldTermination::EndOfInput(terminal) => {
            if !anchor_ok(source, terminal) || !terminal.is_empty() {
                return Err(ContextGoldValidationError::TerminationMustBeEmpty);
            }
            if terminal.start != record.body.end {
                return Err(ContextGoldValidationError::TerminationBoundaryMismatch);
            }
            if terminal.start != source.as_str().len() {
                return Err(ContextGoldValidationError::EndOfInputNotAtSourceEnd);
            }
        }
        ContextGoldTermination::UpstreamTokenizerIncomplete(terminal)
        | ContextGoldTermination::ParserResourceLimit(terminal) => {
            if !anchor_ok(source, terminal) || !terminal.is_empty() {
                return Err(ContextGoldValidationError::TerminationMustBeEmpty);
            }
            if terminal.start != record.body.end {
                return Err(ContextGoldValidationError::TerminationBoundaryMismatch);
            }
        }
    }

    Ok(())
}

fn validate_declarations(
    source: &SourceText,
    record: &ContextGoldRecord,
) -> Result<(), ContextGoldValidationError> {
    let mut previous: Option<(usize, usize, usize)> = None; // (item_ordinal, run_ordinal, span.end)
    for declaration in &record.declarations {
        if !anchor_ok(source, declaration.span) {
            return Err(ContextGoldValidationError::InvalidRange);
        }
        if declaration.span.start < record.body.start || declaration.span.end > record.body.end {
            return Err(ContextGoldValidationError::DeclarationOutsideBody);
        }
        if let Some((previous_item, previous_run, previous_end)) = previous {
            if declaration.item_ordinal == previous_item {
                return Err(ContextGoldValidationError::DuplicateDeclarationItemOrdinal);
            }
            if declaration.item_ordinal < previous_item || declaration.span.start < previous_end {
                return Err(ContextGoldValidationError::DeclarationOrderViolation);
            }
            if declaration.run_ordinal < previous_run {
                return Err(ContextGoldValidationError::DeclarationRunOrdinalNotMonotonic);
            }
        }
        previous = Some((
            declaration.item_ordinal,
            declaration.run_ordinal,
            declaration.span.end,
        ));
    }
    Ok(())
}
