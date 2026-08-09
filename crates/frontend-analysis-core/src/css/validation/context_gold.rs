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

/// A retained context's relationship to its parent: the parent's gold `id`
/// and this context's parent-local direct-item ordinal within it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ContextGoldParent {
    pub(super) parent_id: usize,
    pub(super) item_ordinal: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContextGoldTermination {
    AuthoredRightCurly(GoldRange),
    EndOfInput(GoldRange),
    UpstreamTokenizerIncomplete(GoldRange),
    ParserResourceLimit(GoldRange),
}

impl ContextGoldTermination {
    const fn evidence_range(self) -> GoldRange {
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

/// One independent context record: identity, parent relationship, kind,
/// exact header/opener/body ranges, termination, and the future
/// materialized declaration items it directly owns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContextGoldRecord {
    pub(super) id: usize,
    pub(super) parent: Option<ContextGoldParent>,
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
    DuplicateChildItemOrdinal,
    ChildOrderViolation,
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
    let mut last_child_ordinal: std::collections::BTreeMap<usize, usize> =
        std::collections::BTreeMap::new();

    for (index, record) in fixture.contexts.iter().enumerate() {
        if record.id != index {
            return Err(ContextGoldValidationError::IdIndexMismatch);
        }

        validate_local_shape(&source, record)?;

        let depth = if let Some(parent) = record.parent {
            if parent.parent_id >= index {
                return Err(ContextGoldValidationError::ParentNotBefore);
            }
            let parent_record = &fixture.contexts[parent.parent_id];
            if record.header.start < parent_record.body.start
                || record.extent_end() > parent_record.body.end
            {
                return Err(ContextGoldValidationError::ChildOutsideParentBody);
            }

            if let Some(&previous) = last_child_ordinal.get(&parent.parent_id) {
                if parent.item_ordinal == previous {
                    return Err(ContextGoldValidationError::DuplicateChildItemOrdinal);
                }
                if parent.item_ordinal < previous {
                    return Err(ContextGoldValidationError::ChildOrderViolation);
                }
            }
            last_child_ordinal.insert(parent.parent_id, parent.item_ordinal);

            depths[parent.parent_id] + 1
        } else {
            1
        };
        depths.push(depth);

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
