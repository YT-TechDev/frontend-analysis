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

/// Independent finite group-rule-kind vocabulary (#168). Mirrors only the
/// approved five-member registry; not the production enum itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ContextGoldGroupRuleKind {
    Media,
    Supports,
    Container,
    Layer,
    Scope,
}

/// Independent context-kind vocabulary. Mirrors the #166/#168 production
/// enum's variants; not the production type itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ContextGoldKind {
    QualifiedRuleBlock,
    GroupRuleBlock(ContextGoldGroupRuleKind),
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
    /// The exact authored at-keyword range, present only for a
    /// `GroupRuleBlock` record (#168). Always `None` for `QualifiedRuleBlock`.
    pub(super) at_keyword: Option<GoldRange>,
    /// The nearest ancestor `QualifiedRuleBlock` context id, or `None` at
    /// the implicit stylesheet root's direct children (#141/#168).
    pub(super) nearest_qualified_ancestor: Option<usize>,
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
    /// #168: a refused commit for a nested unsupported at-rule.
    UnsupportedRegions,
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

/// One expected context-aware nested unsupported at-rule (#168): a
/// materialized direct item sharing its owning context's direct-item
/// ordinal space with declarations and retained child contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ContextGoldUnsupportedAtRule {
    pub(super) complete: GoldRange,
    pub(super) at_keyword: GoldRange,
    pub(super) owner_context: usize,
    pub(super) item_ordinal: usize,
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
    /// Context-aware nested unsupported at-rules (#168), flat across the
    /// whole run, mirroring production's flat `unsupported_regions()`.
    pub(super) unsupported: Vec<ContextGoldUnsupportedAtRule>,
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
    GroupAtKeywordPresenceMismatch,
    GroupAtKeywordShapeInvalid,
    NearestQualifiedAncestorInvalidReference,
    NearestQualifiedAncestorMismatch,
    UnsupportedAtRuleShapeInvalid,
    UnsupportedAtRuleUnknownOwner,
    UnsupportedAtRuleOutsideOwnerBody,
    UnsupportedAtRuleOrderViolation,
    DirectItemDuplicateOrdinal,
    DirectItemOrderViolation,
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

        validate_group_at_keyword_presence(&source, record)?;
        validate_nearest_qualified_ancestor(&fixture.contexts, index, record)?;

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

    validate_unsupported_and_direct_items(&source, fixture)?;

    Ok(())
}

/// Validates a `GroupRuleBlock` record's `at_keyword` shape and a
/// `QualifiedRuleBlock` record's absence of one (#168).
fn validate_group_at_keyword_presence(
    source: &SourceText,
    record: &ContextGoldRecord,
) -> Result<(), ContextGoldValidationError> {
    match (record.kind, record.at_keyword) {
        (ContextGoldKind::QualifiedRuleBlock, None) => Ok(()),
        (ContextGoldKind::GroupRuleBlock(_), Some(at_keyword)) => {
            if !anchor_ok(source, at_keyword) || at_keyword.is_empty() {
                return Err(ContextGoldValidationError::GroupAtKeywordShapeInvalid);
            }
            if at_keyword.start != record.header.start || !record.header.contains(at_keyword) {
                return Err(ContextGoldValidationError::GroupAtKeywordShapeInvalid);
            }
            let starts_with_at = source
                .anchor(at_keyword.start, at_keyword.end)
                .is_ok_and(|anchor| anchor.fragment().starts_with('@'));
            if !starts_with_at {
                return Err(ContextGoldValidationError::GroupAtKeywordShapeInvalid);
            }
            Ok(())
        }
        _ => Err(ContextGoldValidationError::GroupAtKeywordPresenceMismatch),
    }
}

/// Validates one record's `nearest_qualified_ancestor` against the retained
/// parent table (#141/#168), mirroring production's own result-level proof:
/// a `Some` reference must be strictly earlier and itself a
/// `QualifiedRuleBlock`, and the value must equal the one structurally
/// derivable from the parent's own kind and nearest-ancestor evidence.
fn validate_nearest_qualified_ancestor(
    contexts: &[ContextGoldRecord],
    index: usize,
    record: &ContextGoldRecord,
) -> Result<(), ContextGoldValidationError> {
    if let Some(ancestor_id) = record.nearest_qualified_ancestor {
        let valid_reference = ancestor_id < index
            && matches!(
                contexts[ancestor_id].kind,
                ContextGoldKind::QualifiedRuleBlock
            );
        if !valid_reference {
            return Err(ContextGoldValidationError::NearestQualifiedAncestorInvalidReference);
        }
    }

    let expected = match record.parent {
        None => None,
        Some(parent_id) => {
            let parent_record = &contexts[parent_id];
            match parent_record.kind {
                ContextGoldKind::QualifiedRuleBlock => Some(parent_id),
                ContextGoldKind::GroupRuleBlock(_) => parent_record.nearest_qualified_ancestor,
            }
        }
    };
    if record.nearest_qualified_ancestor != expected {
        return Err(ContextGoldValidationError::NearestQualifiedAncestorMismatch);
    }

    Ok(())
}

/// One materialized direct block-content item's ordering data for the
/// combined #168 three-category ordinal-space check.
enum GoldDirectItem {
    Declaration { run_ordinal: usize },
    ChildContext,
    NestedUnsupportedAtRule,
}

/// Validates every `unsupported` nested-at-rule expectation's local shape
/// and owner containment, then reconciles the combined direct-item ordinal
/// space (declarations, retained child contexts, and nested unsupported
/// at-rules) per owning context (#168), mirroring production's own
/// result-level reconciliation.
fn validate_unsupported_and_direct_items(
    source: &SourceText,
    fixture: &ContextGoldFixture,
) -> Result<(), ContextGoldValidationError> {
    let mut items_by_context: std::collections::BTreeMap<
        usize,
        Vec<(usize, usize, usize, GoldDirectItem)>,
    > = std::collections::BTreeMap::new();

    for (context_index, record) in fixture.contexts.iter().enumerate() {
        for declaration in &record.declarations {
            items_by_context.entry(context_index).or_default().push((
                declaration.item_ordinal,
                declaration.span.start,
                declaration.span.end,
                GoldDirectItem::Declaration {
                    run_ordinal: declaration.run_ordinal,
                },
            ));
        }
        if let Some(parent_id) = record.parent {
            items_by_context.entry(parent_id).or_default().push((
                record.item_ordinal,
                record.header.start,
                record.extent_end(),
                GoldDirectItem::ChildContext,
            ));
        }
    }

    let mut previous_key = None;
    for unsupported in &fixture.unsupported {
        if !anchor_ok(source, unsupported.complete) || unsupported.complete.is_empty() {
            return Err(ContextGoldValidationError::UnsupportedAtRuleShapeInvalid);
        }
        if !anchor_ok(source, unsupported.at_keyword) || unsupported.at_keyword.is_empty() {
            return Err(ContextGoldValidationError::UnsupportedAtRuleShapeInvalid);
        }
        if unsupported.at_keyword.start != unsupported.complete.start
            || !unsupported.complete.contains(unsupported.at_keyword)
        {
            return Err(ContextGoldValidationError::UnsupportedAtRuleShapeInvalid);
        }
        let starts_with_at = source
            .anchor(unsupported.at_keyword.start, unsupported.at_keyword.end)
            .is_ok_and(|anchor| anchor.fragment().starts_with('@'));
        if !starts_with_at {
            return Err(ContextGoldValidationError::UnsupportedAtRuleShapeInvalid);
        }

        let owner = fixture
            .contexts
            .get(unsupported.owner_context)
            .ok_or(ContextGoldValidationError::UnsupportedAtRuleUnknownOwner)?;
        if unsupported.complete.start < owner.body.start
            || unsupported.complete.end > owner.body.end
        {
            return Err(ContextGoldValidationError::UnsupportedAtRuleOutsideOwnerBody);
        }

        let key = (unsupported.complete.start, unsupported.complete.end);
        if previous_key.is_some_and(|previous| previous > key) {
            return Err(ContextGoldValidationError::UnsupportedAtRuleOrderViolation);
        }
        previous_key = Some(key);

        items_by_context
            .entry(unsupported.owner_context)
            .or_default()
            .push((
                unsupported.item_ordinal,
                unsupported.complete.start,
                unsupported.complete.end,
                GoldDirectItem::NestedUnsupportedAtRule,
            ));
    }

    for (_, mut items) in items_by_context {
        items.sort_by_key(|(ordinal, ..)| *ordinal);
        let mut expected_next_ordinal = 0usize;
        let mut previous_ordinal: Option<usize> = None;
        let mut previous_end: Option<usize> = None;
        let mut current_run: Option<usize> = None;
        let mut expected_next_run = 0usize;

        for (ordinal, start, end, kind) in items {
            if ordinal != expected_next_ordinal {
                if previous_ordinal == Some(ordinal) {
                    return Err(ContextGoldValidationError::DirectItemDuplicateOrdinal);
                }
                return Err(ContextGoldValidationError::DirectItemOrderViolation);
            }
            if let Some(previous_end) = previous_end
                && start < previous_end
            {
                return Err(ContextGoldValidationError::DirectItemOrderViolation);
            }
            previous_ordinal = Some(ordinal);
            previous_end = Some(end);
            expected_next_ordinal = ordinal + 1;

            match kind {
                GoldDirectItem::Declaration { run_ordinal } => match current_run {
                    None => {
                        if run_ordinal != expected_next_run {
                            return Err(
                                ContextGoldValidationError::DeclarationRunOrdinalNotMonotonic,
                            );
                        }
                        current_run = Some(run_ordinal);
                        expected_next_run = run_ordinal + 1;
                    }
                    Some(open_run) => {
                        if run_ordinal != open_run {
                            return Err(
                                ContextGoldValidationError::DeclarationRunOrdinalNotMonotonic,
                            );
                        }
                    }
                },
                GoldDirectItem::ChildContext | GoldDirectItem::NestedUnsupportedAtRule => {
                    current_run = None;
                }
            }
        }
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
