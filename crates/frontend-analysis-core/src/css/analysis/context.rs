//! #172 Core reconciliation for parser-owned CSS context evidence.
//!
//! This module validates only already-projected parser evidence against the
//! exact supplied `SourceText` and against other retained result rows. It
//! never rediscovers syntax from source bytes and never reruns parser semantic
//! qualification.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::css::parser::context::{
    CssParserContextKind, CssParserContextRecord, CssParserContextTermination,
    CssParserDescriptorRuleKind,
};
use crate::css::parser::evidence::CssParserUnsupportedRegion;
use crate::css::parser::resource::CssParserResourceKind;
use crate::css::parser::result::{
    CssParserExecutionCompletion, CssParserRunResult, CssParserTermination,
};
use crate::css::tokenizer::result::{CssTokenizerCompletion, CssTokenizerTermination};
use crate::{SourceAnchor, SourceId, SourceRangeError, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssContextOccurrenceKind {
    Ordinary,
    Descriptor,
    Page,
    PageMargin,
    Keyframe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssContextEvidenceRole {
    ParserTerminal,
    ParserResourceLimit,
    ContextHeader { index: usize },
    ContextBlockOpener { index: usize },
    ContextBody { index: usize },
    ContextAtKeyword { index: usize },
    ContextDescriptorPropertyName { index: usize },
    ContextPageSelectorList { index: usize },
    ContextKeyframesName { index: usize },
    ContextKeyframeSelectorList { index: usize },
    ContextTermination { index: usize },
    UnsupportedComplete { index: usize },
    UnsupportedAtKeyword { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssContextRelationship {
    ParserTerminalMustBePoint,
    ParserResourceLimitTerminalMismatch,
    ContextIdIndexMismatch {
        index: usize,
    },
    ContextParentNotBefore {
        index: usize,
    },
    ContextChildOutsideParentBody {
        index: usize,
    },
    ContextParentKindMismatch {
        index: usize,
    },
    ContextNearestQualifiedAncestorMismatch {
        index: usize,
    },
    ContextSiblingOrdinalOrderViolation {
        index: usize,
    },
    ContextSiblingSourceOrderViolation {
        index: usize,
    },
    ContextBoundaryMismatch {
        index: usize,
    },
    ContextEvidenceShapeMismatch {
        index: usize,
    },
    ContextBeyondParserTerminal {
        index: usize,
    },
    ContextTerminationLifecycleMismatch {
        index: usize,
    },
    OccurrenceUnknownOwner {
        kind: CssContextOccurrenceKind,
        index: usize,
    },
    OccurrenceWrongOwnerKind {
        kind: CssContextOccurrenceKind,
        index: usize,
    },
    OccurrenceOutsideOwnerBody {
        kind: CssContextOccurrenceKind,
        index: usize,
    },
    DirectItemOrdinalViolation {
        context_id: usize,
    },
    DirectItemSourceOrderViolation {
        context_id: usize,
    },
    DeclarationRunOrdinalViolation {
        index: usize,
    },
    UnsupportedUnknownOwner {
        index: usize,
    },
    UnsupportedOutsideOwnerBody {
        index: usize,
    },
    UnsupportedWrongOwnerKind {
        index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssContextAnalysisError {
    SourceIdentityMismatch {
        role: CssContextEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    SourceRangeInvalid {
        role: CssContextEvidenceRole,
        error: SourceRangeError,
    },
    SourceContentMismatch {
        role: CssContextEvidenceRole,
    },
    RelationshipViolation(CssContextRelationship),
    ResourceUsageMismatch {
        kind: CssParserResourceKind,
        expected: usize,
        actual: usize,
    },
    ResourceAccountingOverflow {
        kind: CssParserResourceKind,
    },
}

impl fmt::Display for CssContextAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS Core context reconciliation failure: {self:?}"
        )
    }
}

impl Error for CssContextAnalysisError {}

#[derive(Debug, Clone, Copy)]
struct ContextGraphRow {
    id: usize,
    parent: Option<usize>,
    item_ordinal: usize,
    kind: CssParserContextKind,
    nearest_qualified_ancestor: Option<usize>,
    extent_start: usize,
    extent_end: usize,
    body_start: usize,
    body_end: usize,
}

#[derive(Debug, Clone, Copy)]
enum DirectItemKind {
    OrdinaryDeclaration {
        occurrence_index: usize,
        run_ordinal: usize,
    },
    Other,
}

#[derive(Debug, Clone, Copy)]
struct DirectItem {
    ordinal: usize,
    start: usize,
    end: usize,
    kind: DirectItemKind,
}

pub(super) fn validate_context_evidence(
    source: &SourceText,
    result: &CssParserRunResult,
) -> Result<(), CssContextAnalysisError> {
    validate_anchor(
        source,
        CssContextEvidenceRole::ParserTerminal,
        result.terminal(),
    )?;
    if !result.terminal().range().is_empty() {
        return relationship(CssContextRelationship::ParserTerminalMustBePoint);
    }

    if let CssParserTermination::ParserResourceLimit(evidence) = result.termination() {
        validate_anchor(
            source,
            CssContextEvidenceRole::ParserResourceLimit,
            evidence.location(),
        )?;
        if evidence.location().range() != result.terminal().range() {
            return relationship(CssContextRelationship::ParserResourceLimitTerminalMismatch);
        }
    }

    let peak_depth = validate_context_records(source, result)?;
    validate_placements_and_unsupported(source, result)?;
    validate_resource_counts(result, peak_depth)?;
    Ok(())
}

fn validate_context_records(
    source: &SourceText,
    result: &CssParserRunResult,
) -> Result<usize, CssContextAnalysisError> {
    let mut rows = Vec::with_capacity(result.context_records().len());

    for (index, record) in result.context_records().iter().enumerate() {
        validate_anchor(
            source,
            CssContextEvidenceRole::ContextHeader { index },
            record.header(),
        )?;
        validate_anchor(
            source,
            CssContextEvidenceRole::ContextBlockOpener { index },
            record.block_opener(),
        )?;
        validate_anchor(
            source,
            CssContextEvidenceRole::ContextBody { index },
            record.body(),
        )?;
        if let Some(anchor) = record.at_keyword() {
            validate_anchor(
                source,
                CssContextEvidenceRole::ContextAtKeyword { index },
                anchor,
            )?;
        }
        if let Some(anchor) = record.descriptor_property_name() {
            validate_anchor(
                source,
                CssContextEvidenceRole::ContextDescriptorPropertyName { index },
                anchor,
            )?;
        }
        if let Some(anchor) = record.page_selector_list() {
            validate_anchor(
                source,
                CssContextEvidenceRole::ContextPageSelectorList { index },
                anchor,
            )?;
        }
        if let Some(anchor) = record.keyframes_name() {
            validate_anchor(
                source,
                CssContextEvidenceRole::ContextKeyframesName { index },
                anchor,
            )?;
        }
        if let Some(anchor) = record.keyframe_selector_list() {
            validate_anchor(
                source,
                CssContextEvidenceRole::ContextKeyframeSelectorList { index },
                anchor,
            )?;
        }

        validate_context_shape(index, record)?;
        validate_context_termination(source, result, index, record)?;

        if record.extent_end() > result.terminal().range().start() {
            return relationship(CssContextRelationship::ContextBeyondParserTerminal { index });
        }

        rows.push(ContextGraphRow {
            id: record.id().index(),
            parent: record.parent().map(|parent| parent.index()),
            item_ordinal: record.item_ordinal().value(),
            kind: record.kind(),
            nearest_qualified_ancestor: record
                .nearest_qualified_ancestor()
                .map(|ancestor| ancestor.index()),
            extent_start: record.extent_start(),
            extent_end: record.extent_end(),
            body_start: record.body().range().start(),
            body_end: record.body().range().end(),
        });
    }

    validate_context_graph(&rows)
}

fn validate_context_shape(
    index: usize,
    record: &CssParserContextRecord,
) -> Result<(), CssContextAnalysisError> {
    let header = record.header().range();
    let opener = record.block_opener().range();
    let body = record.body().range();
    if opener.is_empty()
        || record.block_opener().fragment() != "{"
        || header.end() != opener.start()
        || body.start() != opener.end()
    {
        return relationship(CssContextRelationship::ContextBoundaryMismatch { index });
    }

    if let Some(at_keyword) = record.at_keyword() {
        let range = at_keyword.range();
        if !at_keyword.fragment().starts_with('@')
            || range.start() != header.start()
            || range.end() > header.end()
        {
            return relationship(CssContextRelationship::ContextEvidenceShapeMismatch { index });
        }
    }

    let in_header = |anchor: &SourceAnchor| {
        anchor.range().start() >= header.start() && anchor.range().end() <= header.end()
    };
    for anchor in [
        record.descriptor_property_name(),
        record.page_selector_list(),
        record.keyframes_name(),
        record.keyframe_selector_list(),
    ]
    .into_iter()
    .flatten()
    {
        if anchor.range().is_empty() || !in_header(anchor) {
            return relationship(CssContextRelationship::ContextEvidenceShapeMismatch { index });
        }
    }

    let at = record.at_keyword().is_some();
    let descriptor_name = record.descriptor_property_name().is_some();
    let page_selector = record.page_selector_list().is_some();
    let keyframes_name = record.keyframes_name().is_some();
    let keyframe_selector = record.keyframe_selector_list().is_some();
    let shape_ok = match record.kind() {
        CssParserContextKind::QualifiedRuleBlock => {
            !at && !descriptor_name && !page_selector && !keyframes_name && !keyframe_selector
        }
        CssParserContextKind::GroupRuleBlock(_) => {
            at && !descriptor_name && !page_selector && !keyframes_name && !keyframe_selector
        }
        CssParserContextKind::DescriptorRuleBlock(CssParserDescriptorRuleKind::FontFace) => {
            at && !descriptor_name && !page_selector && !keyframes_name && !keyframe_selector
        }
        CssParserContextKind::DescriptorRuleBlock(CssParserDescriptorRuleKind::Property) => {
            at && descriptor_name && !page_selector && !keyframes_name && !keyframe_selector
        }
        CssParserContextKind::PageRuleBlock => {
            at && !descriptor_name && !keyframes_name && !keyframe_selector
        }
        CssParserContextKind::PageMarginRuleBlock(_) => {
            at && !descriptor_name && !page_selector && !keyframes_name && !keyframe_selector
        }
        CssParserContextKind::KeyframesRuleBlock => {
            at && !descriptor_name && !page_selector && keyframes_name && !keyframe_selector
        }
        CssParserContextKind::KeyframeBlock => {
            !at && !descriptor_name && !page_selector && !keyframes_name && keyframe_selector
        }
    };
    if !shape_ok {
        return relationship(CssContextRelationship::ContextEvidenceShapeMismatch { index });
    }

    if let (Some(at_keyword), Some(name)) = (record.at_keyword(), record.descriptor_property_name())
        && name.range().start() < at_keyword.range().end()
    {
        return relationship(CssContextRelationship::ContextEvidenceShapeMismatch { index });
    }
    if let (Some(at_keyword), Some(selector)) = (record.at_keyword(), record.page_selector_list())
        && selector.range().start() < at_keyword.range().end()
    {
        return relationship(CssContextRelationship::ContextEvidenceShapeMismatch { index });
    }
    if let (Some(at_keyword), Some(name)) = (record.at_keyword(), record.keyframes_name())
        && name.range().start() < at_keyword.range().end()
    {
        return relationship(CssContextRelationship::ContextEvidenceShapeMismatch { index });
    }

    Ok(())
}

fn validate_context_termination(
    source: &SourceText,
    result: &CssParserRunResult,
    index: usize,
    record: &CssParserContextRecord,
) -> Result<(), CssContextAnalysisError> {
    let body_end = record.body().range().end();
    match record.termination() {
        CssParserContextTermination::AuthoredRightCurly { right_curly } => {
            validate_anchor(
                source,
                CssContextEvidenceRole::ContextTermination { index },
                right_curly,
            )?;
            if right_curly.fragment() != "}"
                || right_curly.range().is_empty()
                || right_curly.range().start() != body_end
            {
                return relationship(CssContextRelationship::ContextBoundaryMismatch { index });
            }
        }
        CssParserContextTermination::EndOfInput { terminal } => {
            validate_eof_context_boundary(source, index, body_end, terminal)?;
            if !upstream_true_eof(result)
                || !matches!(
                    result.termination(),
                    CssParserTermination::EndOfTokenizerInput
                )
            {
                return relationship(
                    CssContextRelationship::ContextTerminationLifecycleMismatch { index },
                );
            }
        }
        CssParserContextTermination::UpstreamTokenizerIncomplete { terminal } => {
            validate_anchor(
                source,
                CssContextEvidenceRole::ContextTermination { index },
                terminal,
            )?;
            if !terminal.range().is_empty()
                || terminal.range().start() != body_end
                || terminal.range() != result.terminal().range()
                || !matches!(
                    result.termination(),
                    CssParserTermination::UpstreamTokenizerIncomplete
                )
                || result.execution_completion() != CssParserExecutionCompletion::Incomplete
            {
                return relationship(
                    CssContextRelationship::ContextTerminationLifecycleMismatch { index },
                );
            }
        }
        CssParserContextTermination::ParserResourceLimit { terminal } => {
            validate_anchor(
                source,
                CssContextEvidenceRole::ContextTermination { index },
                terminal,
            )?;
            if !terminal.range().is_empty()
                || terminal.range().start() != body_end
                || terminal.range() != result.terminal().range()
                || !matches!(
                    result.termination(),
                    CssParserTermination::ParserResourceLimit(_)
                )
                || result.execution_completion() != CssParserExecutionCompletion::Incomplete
            {
                return relationship(
                    CssContextRelationship::ContextTerminationLifecycleMismatch { index },
                );
            }
        }
    }
    Ok(())
}

fn validate_eof_context_boundary(
    source: &SourceText,
    index: usize,
    body_end: usize,
    terminal: &SourceAnchor,
) -> Result<(), CssContextAnalysisError> {
    validate_anchor(
        source,
        CssContextEvidenceRole::ContextTermination { index },
        terminal,
    )?;
    if !terminal.range().is_empty()
        || terminal.range().start() != body_end
        || terminal.range().start() != source.as_str().len()
    {
        return relationship(CssContextRelationship::ContextTerminationLifecycleMismatch { index });
    }
    Ok(())
}

fn validate_context_graph(rows: &[ContextGraphRow]) -> Result<usize, CssContextAnalysisError> {
    let mut depths = Vec::with_capacity(rows.len());
    let mut last_sibling: BTreeMap<Option<usize>, (usize, usize)> = BTreeMap::new();

    for (index, row) in rows.iter().enumerate() {
        if row.id != index {
            return relationship(CssContextRelationship::ContextIdIndexMismatch { index });
        }

        let depth = if let Some(parent) = row.parent {
            if parent >= index {
                return relationship(CssContextRelationship::ContextParentNotBefore { index });
            }
            let parent_row = &rows[parent];
            if row.extent_start < parent_row.body_start || row.extent_end > parent_row.body_end {
                return relationship(CssContextRelationship::ContextChildOutsideParentBody {
                    index,
                });
            }
            if !parent_kind_allows(parent_row.kind, row.kind) {
                return relationship(CssContextRelationship::ContextParentKindMismatch { index });
            }
            depths[parent] + 1
        } else {
            if !root_kind_allowed(row.kind) {
                return relationship(CssContextRelationship::ContextParentKindMismatch { index });
            }
            1
        };
        depths.push(depth);

        let expected_nearest = match row.parent {
            None => None,
            Some(parent) => match rows[parent].kind {
                CssParserContextKind::QualifiedRuleBlock => Some(parent),
                CssParserContextKind::GroupRuleBlock(_) => rows[parent].nearest_qualified_ancestor,
                CssParserContextKind::PageRuleBlock | CssParserContextKind::KeyframesRuleBlock => {
                    None
                }
                _ => None,
            },
        };
        if row.nearest_qualified_ancestor != expected_nearest {
            return relationship(
                CssContextRelationship::ContextNearestQualifiedAncestorMismatch { index },
            );
        }
        if let Some(nearest) = row.nearest_qualified_ancestor
            && (nearest >= index
                || !matches!(rows[nearest].kind, CssParserContextKind::QualifiedRuleBlock))
        {
            return relationship(
                CssContextRelationship::ContextNearestQualifiedAncestorMismatch { index },
            );
        }

        let scope = row.parent;
        if let Some((previous_ordinal, previous_end)) = last_sibling.get(&scope).copied() {
            if row.item_ordinal <= previous_ordinal {
                return relationship(
                    CssContextRelationship::ContextSiblingOrdinalOrderViolation { index },
                );
            }
            if row.extent_start < previous_end {
                return relationship(CssContextRelationship::ContextSiblingSourceOrderViolation {
                    index,
                });
            }
        }
        last_sibling.insert(scope, (row.item_ordinal, row.extent_end));
    }

    Ok(depths.into_iter().max().unwrap_or(0))
}

fn root_kind_allowed(kind: CssParserContextKind) -> bool {
    matches!(
        kind,
        CssParserContextKind::QualifiedRuleBlock
            | CssParserContextKind::DescriptorRuleBlock(_)
            | CssParserContextKind::PageRuleBlock
            | CssParserContextKind::KeyframesRuleBlock
    )
}

fn parent_kind_allows(parent: CssParserContextKind, child: CssParserContextKind) -> bool {
    match parent {
        CssParserContextKind::QualifiedRuleBlock | CssParserContextKind::GroupRuleBlock(_) => {
            matches!(
                child,
                CssParserContextKind::QualifiedRuleBlock | CssParserContextKind::GroupRuleBlock(_)
            )
        }
        CssParserContextKind::PageRuleBlock => {
            matches!(child, CssParserContextKind::PageMarginRuleBlock(_))
        }
        CssParserContextKind::KeyframesRuleBlock => {
            matches!(child, CssParserContextKind::KeyframeBlock)
        }
        CssParserContextKind::DescriptorRuleBlock(_)
        | CssParserContextKind::PageMarginRuleBlock(_)
        | CssParserContextKind::KeyframeBlock => false,
    }
}

fn validate_placements_and_unsupported(
    source: &SourceText,
    result: &CssParserRunResult,
) -> Result<(), CssContextAnalysisError> {
    let records = result.context_records();
    let mut items: BTreeMap<usize, Vec<DirectItem>> = BTreeMap::new();

    for record in records {
        if let Some(parent) = record.parent() {
            items.entry(parent.index()).or_default().push(DirectItem {
                ordinal: record.item_ordinal().value(),
                start: record.extent_start(),
                end: record.extent_end(),
                kind: DirectItemKind::Other,
            });
        }
    }

    for (index, occurrence) in result.occurrences().iter().enumerate() {
        let placement = occurrence.placement();
        add_occurrence_item(
            records,
            &mut items,
            CssContextOccurrenceKind::Ordinary,
            index,
            placement.context_id().index(),
            placement.item_ordinal().value(),
            Some(placement.run_ordinal().value()),
            occurrence.complete(),
        )?;
    }
    for (index, occurrence) in result.descriptor_occurrences().iter().enumerate() {
        let placement = occurrence.placement();
        add_occurrence_item(
            records,
            &mut items,
            CssContextOccurrenceKind::Descriptor,
            index,
            placement.context_id().index(),
            placement.item_ordinal().value(),
            None,
            occurrence.complete(),
        )?;
    }
    for (index, occurrence) in result.page_occurrences().iter().enumerate() {
        let placement = occurrence.placement();
        add_occurrence_item(
            records,
            &mut items,
            CssContextOccurrenceKind::Page,
            index,
            placement.context_id().index(),
            placement.item_ordinal().value(),
            None,
            occurrence.complete(),
        )?;
    }
    for (index, occurrence) in result.page_margin_occurrences().iter().enumerate() {
        let placement = occurrence.placement();
        add_occurrence_item(
            records,
            &mut items,
            CssContextOccurrenceKind::PageMargin,
            index,
            placement.context_id().index(),
            placement.item_ordinal().value(),
            None,
            occurrence.complete(),
        )?;
    }
    for (index, occurrence) in result.keyframe_occurrences().iter().enumerate() {
        let placement = occurrence.placement();
        add_occurrence_item(
            records,
            &mut items,
            CssContextOccurrenceKind::Keyframe,
            index,
            placement.context_id().index(),
            placement.item_ordinal().value(),
            None,
            occurrence.complete(),
        )?;
    }

    for (index, unsupported) in result.unsupported_regions().iter().enumerate() {
        match unsupported {
            CssParserUnsupportedRegion::TopLevelAtRule {
                complete,
                at_keyword,
            } => {
                validate_anchor(
                    source,
                    CssContextEvidenceRole::UnsupportedComplete { index },
                    complete,
                )?;
                validate_anchor(
                    source,
                    CssContextEvidenceRole::UnsupportedAtKeyword { index },
                    at_keyword,
                )?;
            }
            CssParserUnsupportedRegion::NestedContentRemainder { region } => {
                validate_anchor(
                    source,
                    CssContextEvidenceRole::UnsupportedComplete { index },
                    region,
                )?;
            }
            CssParserUnsupportedRegion::NestedAtRule {
                complete,
                at_keyword,
                context_id,
                item_ordinal,
            } => {
                validate_anchor(
                    source,
                    CssContextEvidenceRole::UnsupportedComplete { index },
                    complete,
                )?;
                validate_anchor(
                    source,
                    CssContextEvidenceRole::UnsupportedAtKeyword { index },
                    at_keyword,
                )?;
                if at_keyword.range().start() != complete.range().start()
                    || at_keyword.range().end() > complete.range().end()
                {
                    return relationship(CssContextRelationship::UnsupportedOutsideOwnerBody {
                        index,
                    });
                }
                add_unsupported_item(
                    records,
                    &mut items,
                    index,
                    context_id.index(),
                    item_ordinal.value(),
                    complete,
                    None,
                )?;
            }
            CssParserUnsupportedRegion::UnqualifiedKeyframeBlock {
                complete,
                context_id,
                item_ordinal,
            } => {
                validate_anchor(
                    source,
                    CssContextEvidenceRole::UnsupportedComplete { index },
                    complete,
                )?;
                add_unsupported_item(
                    records,
                    &mut items,
                    index,
                    context_id.index(),
                    item_ordinal.value(),
                    complete,
                    Some(CssParserContextKind::KeyframesRuleBlock),
                )?;
            }
        }
    }

    validate_direct_items(&mut items)
}

#[allow(clippy::too_many_arguments)]
fn add_occurrence_item(
    records: &[CssParserContextRecord],
    items: &mut BTreeMap<usize, Vec<DirectItem>>,
    kind: CssContextOccurrenceKind,
    index: usize,
    context_id: usize,
    ordinal: usize,
    run_ordinal: Option<usize>,
    complete: &SourceAnchor,
) -> Result<(), CssContextAnalysisError> {
    let Some(owner) = records.get(context_id) else {
        return relationship(CssContextRelationship::OccurrenceUnknownOwner { kind, index });
    };
    validate_occurrence_owner_kind(kind, index, owner.kind())?;
    let body = owner.body().range();
    if complete.range().start() < body.start() || complete.range().end() > body.end() {
        return relationship(CssContextRelationship::OccurrenceOutsideOwnerBody { kind, index });
    }
    items.entry(context_id).or_default().push(DirectItem {
        ordinal,
        start: complete.range().start(),
        end: complete.range().end(),
        kind: match (kind, run_ordinal) {
            (CssContextOccurrenceKind::Ordinary, Some(run_ordinal)) => {
                DirectItemKind::OrdinaryDeclaration {
                    occurrence_index: index,
                    run_ordinal,
                }
            }
            _ => DirectItemKind::Other,
        },
    });
    Ok(())
}

fn add_unsupported_item(
    records: &[CssParserContextRecord],
    items: &mut BTreeMap<usize, Vec<DirectItem>>,
    index: usize,
    context_id: usize,
    ordinal: usize,
    complete: &SourceAnchor,
    required_kind: Option<CssParserContextKind>,
) -> Result<(), CssContextAnalysisError> {
    let Some(owner) = records.get(context_id) else {
        return relationship(CssContextRelationship::UnsupportedUnknownOwner { index });
    };
    if let Some(required_kind) = required_kind
        && owner.kind() != required_kind
    {
        return relationship(CssContextRelationship::UnsupportedWrongOwnerKind { index });
    }
    let body = owner.body().range();
    if complete.range().start() < body.start() || complete.range().end() > body.end() {
        return relationship(CssContextRelationship::UnsupportedOutsideOwnerBody { index });
    }
    items.entry(context_id).or_default().push(DirectItem {
        ordinal,
        start: complete.range().start(),
        end: complete.range().end(),
        kind: DirectItemKind::Other,
    });
    Ok(())
}

fn occurrence_owner_kind_allowed(
    occurrence: CssContextOccurrenceKind,
    owner: CssParserContextKind,
) -> bool {
    match occurrence {
        CssContextOccurrenceKind::Ordinary => matches!(
            owner,
            CssParserContextKind::QualifiedRuleBlock | CssParserContextKind::GroupRuleBlock(_)
        ),
        CssContextOccurrenceKind::Descriptor => {
            matches!(owner, CssParserContextKind::DescriptorRuleBlock(_))
        }
        CssContextOccurrenceKind::Page => matches!(owner, CssParserContextKind::PageRuleBlock),
        CssContextOccurrenceKind::PageMargin => {
            matches!(owner, CssParserContextKind::PageMarginRuleBlock(_))
        }
        CssContextOccurrenceKind::Keyframe => matches!(owner, CssParserContextKind::KeyframeBlock),
    }
}

fn validate_occurrence_owner_kind(
    occurrence: CssContextOccurrenceKind,
    index: usize,
    owner: CssParserContextKind,
) -> Result<(), CssContextAnalysisError> {
    if !occurrence_owner_kind_allowed(occurrence, owner) {
        return relationship(CssContextRelationship::OccurrenceWrongOwnerKind {
            kind: occurrence,
            index,
        });
    }
    Ok(())
}

fn validate_direct_items(
    items: &mut BTreeMap<usize, Vec<DirectItem>>,
) -> Result<(), CssContextAnalysisError> {
    for (&context_id, context_items) in items.iter_mut() {
        context_items.sort_by_key(|item| item.ordinal);
        let mut previous_end = None;
        let mut next_run = 0usize;
        let mut open_run = None;

        for (expected_ordinal, item) in context_items.iter().enumerate() {
            if item.ordinal != expected_ordinal {
                return relationship(CssContextRelationship::DirectItemOrdinalViolation {
                    context_id,
                });
            }
            if previous_end.is_some_and(|end| item.start < end) {
                return relationship(CssContextRelationship::DirectItemSourceOrderViolation {
                    context_id,
                });
            }
            previous_end = Some(item.end);

            match item.kind {
                DirectItemKind::OrdinaryDeclaration {
                    occurrence_index,
                    run_ordinal,
                } => match open_run {
                    Some(current) if current == run_ordinal => {}
                    Some(_) => {
                        return relationship(
                            CssContextRelationship::DeclarationRunOrdinalViolation {
                                index: occurrence_index,
                            },
                        );
                    }
                    None => {
                        if run_ordinal != next_run {
                            return relationship(
                                CssContextRelationship::DeclarationRunOrdinalViolation {
                                    index: occurrence_index,
                                },
                            );
                        }
                        open_run = Some(run_ordinal);
                        next_run += 1;
                    }
                },
                DirectItemKind::Other => open_run = None,
            }
        }
    }
    Ok(())
}

fn validate_resource_counts(
    result: &CssParserRunResult,
    peak_depth: usize,
) -> Result<(), CssContextAnalysisError> {
    validate_resource_count(
        result,
        CssParserResourceKind::ContextRecords,
        result.context_records().len(),
    )?;
    validate_resource_count(result, CssParserResourceKind::PeakContextDepth, peak_depth)?;

    let declaration_count = [
        result.occurrences().len(),
        result.descriptor_occurrences().len(),
        result.page_occurrences().len(),
        result.page_margin_occurrences().len(),
        result.keyframe_occurrences().len(),
    ]
    .into_iter()
    .try_fold(0usize, |current, additional| {
        current.checked_add(additional)
    })
    .ok_or(CssContextAnalysisError::ResourceAccountingOverflow {
        kind: CssParserResourceKind::DeclarationOccurrences,
    })?;
    validate_resource_count(
        result,
        CssParserResourceKind::DeclarationOccurrences,
        declaration_count,
    )
}

fn validate_resource_count(
    result: &CssParserRunResult,
    kind: CssParserResourceKind,
    expected: usize,
) -> Result<(), CssContextAnalysisError> {
    validate_resource_count_values(kind, expected, result.resources().value(kind))
}

fn validate_resource_count_values(
    kind: CssParserResourceKind,
    expected: usize,
    actual: usize,
) -> Result<(), CssContextAnalysisError> {
    if actual != expected {
        return Err(CssContextAnalysisError::ResourceUsageMismatch {
            kind,
            expected,
            actual,
        });
    }
    Ok(())
}

fn validate_anchor(
    source: &SourceText,
    role: CssContextEvidenceRole,
    anchor: &SourceAnchor,
) -> Result<(), CssContextAnalysisError> {
    if anchor.source_id() != source.id() {
        return Err(CssContextAnalysisError::SourceIdentityMismatch {
            role,
            expected: source.id(),
            actual: anchor.source_id(),
        });
    }
    let range = anchor.range();
    source
        .anchor(range.start(), range.end())
        .map_err(|error| CssContextAnalysisError::SourceRangeInvalid { role, error })?;
    if !source.retains_exact_anchor_source(anchor) {
        return Err(CssContextAnalysisError::SourceContentMismatch { role });
    }
    Ok(())
}

fn upstream_true_eof(result: &CssParserRunResult) -> bool {
    let upstream = result.upstream_tokenizer_result();
    upstream.completion() == CssTokenizerCompletion::Complete
        && matches!(upstream.termination(), CssTokenizerTermination::EndOfInput)
}

fn relationship<T>(relationship: CssContextRelationship) -> Result<T, CssContextAnalysisError> {
    Err(CssContextAnalysisError::RelationshipViolation(relationship))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::context::CssParserGroupRuleKind;

    fn source(id: u64, text: &str) -> SourceText {
        SourceText::new(SourceId::new(id), text.to_owned())
    }

    #[test]
    fn corruption_foreign_source_anchor_is_rejected() {
        let expected = source(1, "abc");
        let foreign = source(2, "abc");
        let error = validate_anchor(
            &expected,
            CssContextEvidenceRole::ContextHeader { index: 0 },
            &foreign.anchor(0, 1).unwrap(),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            CssContextAnalysisError::SourceIdentityMismatch { .. }
        ));
    }

    #[test]
    fn corruption_same_source_id_different_retained_bytes_is_rejected() {
        let expected = source(3, "abc");
        let different = source(3, "abd");
        let error = validate_anchor(
            &expected,
            CssContextEvidenceRole::ContextHeader { index: 0 },
            &different.anchor(0, 1).unwrap(),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            CssContextAnalysisError::SourceContentMismatch { .. }
        ));
    }

    #[test]
    fn corruption_range_invalid_for_supplied_source_is_rejected() {
        let short = source(4, "a");
        let long = source(4, "abcd");
        let error = validate_anchor(
            &short,
            CssContextEvidenceRole::ContextHeader { index: 0 },
            &long.anchor(0, 4).unwrap(),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            CssContextAnalysisError::SourceRangeInvalid { .. }
        ));
    }

    #[test]
    fn corruption_forward_parent_is_rejected_without_source_rediscovery() {
        let rows = [
            ContextGraphRow {
                id: 0,
                parent: Some(1),
                item_ordinal: 0,
                kind: CssParserContextKind::QualifiedRuleBlock,
                nearest_qualified_ancestor: None,
                extent_start: 0,
                extent_end: 10,
                body_start: 1,
                body_end: 9,
            },
            ContextGraphRow {
                id: 1,
                parent: None,
                item_ordinal: 0,
                kind: CssParserContextKind::QualifiedRuleBlock,
                nearest_qualified_ancestor: None,
                extent_start: 10,
                extent_end: 20,
                body_start: 11,
                body_end: 19,
            },
        ];
        assert!(matches!(
            validate_context_graph(&rows),
            Err(CssContextAnalysisError::RelationshipViolation(
                CssContextRelationship::ContextParentNotBefore { index: 0 }
            ))
        ));
    }

    #[test]
    fn corruption_child_outside_parent_body_is_rejected() {
        let rows = [
            ContextGraphRow {
                id: 0,
                parent: None,
                item_ordinal: 0,
                kind: CssParserContextKind::QualifiedRuleBlock,
                nearest_qualified_ancestor: None,
                extent_start: 0,
                extent_end: 10,
                body_start: 2,
                body_end: 8,
            },
            ContextGraphRow {
                id: 1,
                parent: Some(0),
                item_ordinal: 0,
                kind: CssParserContextKind::QualifiedRuleBlock,
                nearest_qualified_ancestor: Some(0),
                extent_start: 1,
                extent_end: 7,
                body_start: 2,
                body_end: 6,
            },
        ];
        assert!(matches!(
            validate_context_graph(&rows),
            Err(CssContextAnalysisError::RelationshipViolation(
                CssContextRelationship::ContextChildOutsideParentBody { index: 1 }
            ))
        ));
    }

    #[test]
    fn corruption_impossible_parent_kind_is_rejected() {
        let rows = [
            ContextGraphRow {
                id: 0,
                parent: None,
                item_ordinal: 0,
                kind: CssParserContextKind::QualifiedRuleBlock,
                nearest_qualified_ancestor: None,
                extent_start: 0,
                extent_end: 20,
                body_start: 1,
                body_end: 19,
            },
            ContextGraphRow {
                id: 1,
                parent: Some(0),
                item_ordinal: 0,
                kind: CssParserContextKind::KeyframeBlock,
                nearest_qualified_ancestor: Some(0),
                extent_start: 2,
                extent_end: 10,
                body_start: 3,
                body_end: 9,
            },
        ];
        assert!(matches!(
            validate_context_graph(&rows),
            Err(CssContextAnalysisError::RelationshipViolation(
                CssContextRelationship::ContextParentKindMismatch { index: 1 }
            ))
        ));
    }

    #[test]
    fn corruption_nearest_qualified_ancestor_is_rejected() {
        let rows = [
            ContextGraphRow {
                id: 0,
                parent: None,
                item_ordinal: 0,
                kind: CssParserContextKind::QualifiedRuleBlock,
                nearest_qualified_ancestor: None,
                extent_start: 0,
                extent_end: 30,
                body_start: 1,
                body_end: 29,
            },
            ContextGraphRow {
                id: 1,
                parent: Some(0),
                item_ordinal: 0,
                kind: CssParserContextKind::GroupRuleBlock(CssParserGroupRuleKind::Media),
                nearest_qualified_ancestor: None,
                extent_start: 2,
                extent_end: 20,
                body_start: 3,
                body_end: 19,
            },
        ];
        assert!(matches!(
            validate_context_graph(&rows),
            Err(CssContextAnalysisError::RelationshipViolation(
                CssContextRelationship::ContextNearestQualifiedAncestorMismatch { index: 1 }
            ))
        ));
    }

    #[test]
    fn corruption_direct_item_ordinal_gap_is_rejected() {
        let mut items = BTreeMap::from([(
            0,
            vec![
                DirectItem {
                    ordinal: 0,
                    start: 1,
                    end: 2,
                    kind: DirectItemKind::Other,
                },
                DirectItem {
                    ordinal: 2,
                    start: 3,
                    end: 4,
                    kind: DirectItemKind::Other,
                },
            ],
        )]);
        assert!(matches!(
            validate_direct_items(&mut items),
            Err(CssContextAnalysisError::RelationshipViolation(
                CssContextRelationship::DirectItemOrdinalViolation { context_id: 0 }
            ))
        ));
    }

    #[test]
    fn corruption_declaration_run_reuse_after_child_is_rejected() {
        let mut items = BTreeMap::from([(
            0,
            vec![
                DirectItem {
                    ordinal: 0,
                    start: 1,
                    end: 2,
                    kind: DirectItemKind::OrdinaryDeclaration {
                        occurrence_index: 0,
                        run_ordinal: 0,
                    },
                },
                DirectItem {
                    ordinal: 1,
                    start: 2,
                    end: 3,
                    kind: DirectItemKind::Other,
                },
                DirectItem {
                    ordinal: 2,
                    start: 3,
                    end: 4,
                    kind: DirectItemKind::OrdinaryDeclaration {
                        occurrence_index: 1,
                        run_ordinal: 0,
                    },
                },
            ],
        )]);
        assert!(matches!(
            validate_direct_items(&mut items),
            Err(CssContextAnalysisError::RelationshipViolation(
                CssContextRelationship::DeclarationRunOrdinalViolation { index: 1 }
            ))
        ));
    }

    #[test]
    fn occurrence_owner_kind_matrix_rejects_semantic_masquerading() {
        assert!(!occurrence_owner_kind_allowed(
            CssContextOccurrenceKind::Keyframe,
            CssParserContextKind::QualifiedRuleBlock,
        ));
        assert!(!occurrence_owner_kind_allowed(
            CssContextOccurrenceKind::Descriptor,
            CssParserContextKind::PageRuleBlock,
        ));
        assert!(occurrence_owner_kind_allowed(
            CssContextOccurrenceKind::Ordinary,
            CssParserContextKind::GroupRuleBlock(CssParserGroupRuleKind::Media),
        ));
    }

    #[test]
    fn corruption_context_eof_terminal_before_source_end_is_rejected() {
        let text = source(8, "abcd");
        let terminal = text.anchor(3, 3).unwrap();
        assert!(matches!(
            validate_eof_context_boundary(&text, 0, 3, &terminal),
            Err(CssContextAnalysisError::RelationshipViolation(
                CssContextRelationship::ContextTerminationLifecycleMismatch { index: 0 }
            ))
        ));
    }

    #[test]
    fn corruption_context_resource_count_mismatch_is_rejected() {
        assert!(matches!(
            validate_resource_count_values(CssParserResourceKind::ContextRecords, 2, 1),
            Err(CssContextAnalysisError::ResourceUsageMismatch {
                kind: CssParserResourceKind::ContextRecords,
                expected: 2,
                actual: 1,
            })
        ));
        assert!(matches!(
            validate_resource_count_values(CssParserResourceKind::DeclarationOccurrences, 4, 3),
            Err(CssContextAnalysisError::ResourceUsageMismatch {
                kind: CssParserResourceKind::DeclarationOccurrences,
                expected: 4,
                actual: 3,
            })
        ));
    }

    #[test]
    fn corruption_wrong_occurrence_owner_kind_is_rejected_by_core_helper() {
        assert!(matches!(
            validate_occurrence_owner_kind(
                CssContextOccurrenceKind::Keyframe,
                5,
                CssParserContextKind::QualifiedRuleBlock,
            ),
            Err(CssContextAnalysisError::RelationshipViolation(
                CssContextRelationship::OccurrenceWrongOwnerKind {
                    kind: CssContextOccurrenceKind::Keyframe,
                    index: 5,
                }
            ))
        ));
    }

    #[test]
    fn error_debug_and_display_do_not_disclose_source_content() {
        const SECRET: &str = "context-secret-authored-css";
        let expected = source(9, SECRET);
        let foreign = source(10, SECRET);
        let error = validate_anchor(
            &expected,
            CssContextEvidenceRole::ContextHeader { index: 0 },
            &foreign.anchor(0, SECRET.len()).unwrap(),
        )
        .unwrap_err();
        assert!(!format!("{error:?}").contains(SECRET));
        assert!(!error.to_string().contains(SECRET));
    }
}
