//! Core-integrated CSS source analysis boundary (#140/#172).
//!
//! Connects retained `&SourceText` through the project-owned CSS tokenizer
//! and parser, then revalidates parser-projected source/relationship evidence
//! against that exact supplied source before returning the same
//! `CssParserRunResult`. Tokenizer/parser semantics, diagnostics, recovery,
//! discard, unsupported, completion, termination, and resource ownership stay
//! lower-layer owned; Core adds reconciliation only.
//!
//! #172 extends the original declaration-only #140 reconciliation through the
//! retained context graph and every distinct declaration-shaped occurrence
//! category. It does not search source text, rescan delimiters, retokenize,
//! reparse, reconstruct context boundaries, infer parentage from raw ranges,
//! or rerun descriptor/page/keyframe qualification. No second successful
//! result hierarchy or public CSS API is introduced.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRangeError, SourceText};

use super::declaration::{
    CssDeclarationEvidenceRole, CssDeclarationOccurrence, CssDeclarationTermination,
};
use super::parser::producer::run as run_parser;
use super::parser::resource::CssParserLimits;
use super::parser::result::{CssParserRunError, CssParserRunResult};
use super::tokenizer::producer::run as run_tokenizer;
use super::tokenizer::resource::CssTokenizerLimits;
use super::tokenizer::result::CssTokenizerRunError;

mod context;

#[cfg(test)]
mod tests;

/// The narrow structural relationship a [`CssAnalysisError::OccurrenceRelationshipViolation`]
/// concerns, beyond independently-valid per-anchor evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOccurrenceRelationship {
    /// `complete.start` must equal `property_name.start`.
    CompleteStartMatchesPropertyName,
    /// `property_name` must be contained in `complete`.
    PropertyNameContained,
    /// `colon` must be contained in `complete`.
    ColonContained,
    /// `property_name.end` must be `<= colon.start`.
    PropertyNameEndsAtOrBeforeColonStart,
    /// A non-empty `value` must be contained in `complete`.
    ValueContained,
    /// A zero-length `value` must sit exactly at `colon.end`.
    ZeroLengthValueAtColonEnd,
    /// `colon.end` must be `<=` a non-empty `value.start`.
    ColonEndsAtOrBeforeValueStart,
    /// `priority.complete` must be contained in `complete`.
    PriorityContained,
    /// `priority.complete.start` must equal `bang.start`.
    PriorityStartsAtBang,
    /// `priority.complete.end` must equal `important_ident.end`.
    PriorityEndsAtImportantIdent,
    /// `bang.end` must be `<= important_ident.start`.
    BangEndsAtOrBeforeImportantIdentStart,
    /// `value` must not overlap `priority.complete`.
    ValueDoesNotOverlapPriority,
    /// An authored semicolon must be contained in `complete`.
    AuthoredSemicolonContained,
    /// An authored semicolon must not precede the last retained
    /// value/priority evidence.
    AuthoredSemicolonNotBeforeLastEvidence,
    /// `complete.end` must equal the authored semicolon's end.
    AuthoredSemicolonEndsComplete,
    /// An omitted-before-right-curly boundary must not precede `complete.end`.
    RightCurlyAtOrAfterCompleteEnd,
    /// An omitted-at-end-of-input terminal must be an empty anchor.
    EofTerminalEmpty,
    /// An omitted-at-end-of-input terminal must sit at the exact retained
    /// source end.
    EofTerminalAtSourceEnd,
    /// An omitted-at-end-of-input terminal must not precede `complete.end`.
    EofTerminalAtOrAfterCompleteEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnalysisOccurrenceKind {
    Ordinary,
    Descriptor,
    Page,
    PageMargin,
    Keyframe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssAnalysisOccurrenceRef {
    pub(crate) kind: CssAnalysisOccurrenceKind,
    pub(crate) index: usize,
}

impl CssAnalysisOccurrenceRef {
    const fn ordinary(index: usize) -> Self {
        Self {
            kind: CssAnalysisOccurrenceKind::Ordinary,
            index,
        }
    }

    const fn descriptor(index: usize) -> Self {
        Self {
            kind: CssAnalysisOccurrenceKind::Descriptor,
            index,
        }
    }

    const fn page(index: usize) -> Self {
        Self {
            kind: CssAnalysisOccurrenceKind::Page,
            index,
        }
    }

    const fn page_margin(index: usize) -> Self {
        Self {
            kind: CssAnalysisOccurrenceKind::PageMargin,
            index,
        }
    }

    const fn keyframe(index: usize) -> Self {
        Self {
            kind: CssAnalysisOccurrenceKind::Keyframe,
            index,
        }
    }
}

/// A Core-integration boundary failure for CSS source analysis.
///
/// Distinct from tokenizer/parser diagnostics (authored-input evidence) and
/// from [`CssTokenizerRunError`] / [`CssParserRunError`] (lower-layer
/// internal contract failures, wrapped here unchanged): this vocabulary
/// covers only failures owned by this Core boundary while reconciling
/// parser-projected occurrence and context evidence against the exact
/// supplied [`SourceText`]. A parser/Core contract failure always becomes
/// `Err`, never clean success, clean absence, unsupported coverage, or
/// resource-limit completion.
///
/// `Debug`/`Display` output intentionally carries only structural evidence
/// (occurrence indices, evidence roles, relationship kinds, [`SourceId`],
/// and [`SourceRangeError`]) and never arbitrary authored source content or
/// decoded user-controlled strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssAnalysisError {
    /// The project-owned tokenizer reported its own run failure.
    TokenizerRun(CssTokenizerRunError),
    /// The project-owned bounded CSS parser reported its own run failure.
    ParserRun(CssParserRunError),
    /// #172 context/placement/resource reconciliation failed.
    Context(context::CssContextAnalysisError),
    /// The occurrence evidence for the given role is bound to a source
    /// identity other than the exact supplied [`SourceText`].
    OccurrenceSourceIdentityMismatch {
        occurrence: CssAnalysisOccurrenceRef,
        role: CssDeclarationEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    /// The occurrence evidence's already-projected range could not be
    /// revalidated through the exact supplied [`SourceText`].
    OccurrenceSourceRangeInvalid {
        occurrence: CssAnalysisOccurrenceRef,
        role: CssDeclarationEvidenceRole,
        error: SourceRangeError,
    },
    /// The occurrence evidence's range revalidated, but the exact supplied
    /// [`SourceText`] carries different content at that range.
    OccurrenceSourceContentMismatch {
        occurrence: CssAnalysisOccurrenceRef,
        role: CssDeclarationEvidenceRole,
    },
    /// The occurrence's independently-valid anchors violate one of the
    /// minimal approved structural relationships among them.
    OccurrenceRelationshipViolation {
        occurrence: CssAnalysisOccurrenceRef,
        relationship: CssOccurrenceRelationship,
    },
}

impl From<CssTokenizerRunError> for CssAnalysisError {
    fn from(error: CssTokenizerRunError) -> Self {
        Self::TokenizerRun(error)
    }
}

impl From<CssParserRunError> for CssAnalysisError {
    fn from(error: CssParserRunError) -> Self {
        Self::ParserRun(error)
    }
}

impl From<context::CssContextAnalysisError> for CssAnalysisError {
    fn from(error: context::CssContextAnalysisError) -> Self {
        Self::Context(error)
    }
}

impl fmt::Display for CssAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS analysis boundary violation: {self:?}")
    }
}

impl Error for CssAnalysisError {}

/// Connects retained `&SourceText` through the project-owned CSS tokenizer
/// and parser into the Core-owned CSS source analysis operation established
/// by #140 and extended through context evidence by #172.
///
/// The caller retains ownership of `source`; this operation borrows it for
/// execution and does not clone the complete source. The returned
/// [`CssParserRunResult`] may outlive the caller's `source` handle through
/// the existing [`SourceAnchor`] ownership contract.
///
/// Tokenizer completion, diagnostics, resources, parser diagnostics,
/// recovery, discard, unsupported regions, completion, termination, and
/// resources all remain authoritative and unchanged through the returned
/// [`CssParserRunResult`]; this operation never upgrades or reinterprets
/// that meaning (an `Incomplete` run never becomes `Complete`, and
/// unsupported/recovery/discard evidence is never erased). A tokenizer or
/// parser run failure, or a Core source-evidence/relationship validation
/// failure, all become `Err`.
pub(crate) fn analyze_css_source(
    source: &SourceText,
    tokenizer_limits: CssTokenizerLimits,
    parser_limits: CssParserLimits,
) -> Result<CssParserRunResult, CssAnalysisError> {
    let tokenizer_result = run_tokenizer(source, tokenizer_limits)?;
    let result = run_parser(source, tokenizer_result, parser_limits)?;

    for (index, occurrence) in result.occurrences().iter().enumerate() {
        validate_occurrence_against_source(source, index, occurrence)?;
    }
    for (index, occurrence) in result.descriptor_occurrences().iter().enumerate() {
        validate_occurrence_evidence_for(
            source,
            CssAnalysisOccurrenceRef::descriptor(index),
            occurrence.complete(),
            occurrence.name(),
            occurrence.colon(),
            occurrence.value(),
            occurrence.priority().map(|priority| {
                (
                    priority.complete(),
                    priority.bang(),
                    priority.important_ident(),
                )
            }),
            occurrence.termination(),
        )?;
    }
    for (index, occurrence) in result.page_occurrences().iter().enumerate() {
        validate_occurrence_evidence_for(
            source,
            CssAnalysisOccurrenceRef::page(index),
            occurrence.complete(),
            occurrence.name(),
            occurrence.colon(),
            occurrence.value(),
            occurrence.priority().map(|priority| {
                (
                    priority.complete(),
                    priority.bang(),
                    priority.important_ident(),
                )
            }),
            occurrence.termination(),
        )?;
    }
    for (index, occurrence) in result.page_margin_occurrences().iter().enumerate() {
        validate_occurrence_evidence_for(
            source,
            CssAnalysisOccurrenceRef::page_margin(index),
            occurrence.complete(),
            occurrence.name(),
            occurrence.colon(),
            occurrence.value(),
            occurrence.priority().map(|priority| {
                (
                    priority.complete(),
                    priority.bang(),
                    priority.important_ident(),
                )
            }),
            occurrence.termination(),
        )?;
    }
    for (index, occurrence) in result.keyframe_occurrences().iter().enumerate() {
        validate_occurrence_evidence_for(
            source,
            CssAnalysisOccurrenceRef::keyframe(index),
            occurrence.complete(),
            occurrence.name(),
            occurrence.colon(),
            occurrence.value(),
            occurrence.priority().map(|priority| {
                (
                    priority.complete(),
                    priority.bang(),
                    priority.important_ident(),
                )
            }),
            occurrence.termination(),
        )?;
    }

    context::validate_context_evidence(source, &result)?;
    Ok(result)
}

/// Validates one projected occurrence's evidence against the exact supplied
/// `source`. Delegates to [`validate_occurrence_evidence_for`], which stays
/// production-relevant while accepting the anchors directly so tests may
/// exercise it without widening `CssDeclarationOccurrence`'s private fields.
fn validate_occurrence_against_source(
    source: &SourceText,
    occurrence_index: usize,
    occurrence: &CssDeclarationOccurrence,
) -> Result<(), CssAnalysisError> {
    validate_occurrence_evidence_for(
        source,
        CssAnalysisOccurrenceRef::ordinary(occurrence_index),
        occurrence.complete(),
        occurrence.property_name(),
        occurrence.colon(),
        occurrence.value(),
        occurrence.priority().map(|priority| {
            (
                priority.complete(),
                priority.bang(),
                priority.important_ident(),
            )
        }),
        occurrence.termination(),
    )
}

/// Validates already-projected occurrence evidence against the exact
/// supplied `source`. This is reconciliation of already-projected evidence,
/// never source discovery: it performs no source search, delimiter scan,
/// endpoint reconstruction, retokenization, or reparsing. For each present
/// anchor it revalidates the already-projected range through
/// [`SourceText::anchor`] and reconciles the anchor's complete retained-source
/// identity/content through the crate-private Core source helper, then checks
/// the minimal approved structural relationships among the anchors.
///
/// Property/colon/semicolon/right-curly fixed spelling and the decoded
/// `important` identifier match remain owned by the declaration domain
/// (`super::declaration`), which already validated them at construction
/// time; this boundary does not re-derive or re-check spelling or decoded
/// semantics.
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn validate_occurrence_evidence(
    source: &SourceText,
    occurrence_index: usize,
    complete: &SourceAnchor,
    property_name: &SourceAnchor,
    colon: &SourceAnchor,
    value: &SourceAnchor,
    priority: Option<(&SourceAnchor, &SourceAnchor, &SourceAnchor)>,
    termination: &CssDeclarationTermination,
) -> Result<(), CssAnalysisError> {
    validate_occurrence_evidence_for(
        source,
        CssAnalysisOccurrenceRef::ordinary(occurrence_index),
        complete,
        property_name,
        colon,
        value,
        priority,
        termination,
    )
}

#[allow(clippy::too_many_arguments)]
fn validate_occurrence_evidence_for(
    source: &SourceText,
    occurrence: CssAnalysisOccurrenceRef,
    complete: &SourceAnchor,
    property_name: &SourceAnchor,
    colon: &SourceAnchor,
    value: &SourceAnchor,
    priority: Option<(&SourceAnchor, &SourceAnchor, &SourceAnchor)>,
    termination: &CssDeclarationTermination,
) -> Result<(), CssAnalysisError> {
    validate_role_evidence(
        source,
        occurrence,
        CssDeclarationEvidenceRole::Complete,
        complete,
    )?;
    validate_role_evidence(
        source,
        occurrence,
        CssDeclarationEvidenceRole::PropertyName,
        property_name,
    )?;
    validate_role_evidence(source, occurrence, CssDeclarationEvidenceRole::Colon, colon)?;
    validate_role_evidence(source, occurrence, CssDeclarationEvidenceRole::Value, value)?;

    if complete.range().start() != property_name.range().start() {
        return Err(relationship_violation(
            occurrence,
            CssOccurrenceRelationship::CompleteStartMatchesPropertyName,
        ));
    }
    require_contained(
        complete,
        property_name,
        occurrence,
        CssOccurrenceRelationship::PropertyNameContained,
    )?;
    require_contained(
        complete,
        colon,
        occurrence,
        CssOccurrenceRelationship::ColonContained,
    )?;
    if property_name.range().end() > colon.range().start() {
        return Err(relationship_violation(
            occurrence,
            CssOccurrenceRelationship::PropertyNameEndsAtOrBeforeColonStart,
        ));
    }

    let last_semantic_end = if value.range().is_empty() {
        if value.range().start() != colon.range().end() {
            return Err(relationship_violation(
                occurrence,
                CssOccurrenceRelationship::ZeroLengthValueAtColonEnd,
            ));
        }
        colon.range().end()
    } else {
        require_contained(
            complete,
            value,
            occurrence,
            CssOccurrenceRelationship::ValueContained,
        )?;
        if colon.range().end() > value.range().start() {
            return Err(relationship_violation(
                occurrence,
                CssOccurrenceRelationship::ColonEndsAtOrBeforeValueStart,
            ));
        }
        value.range().end()
    };

    let last_semantic_end = if let Some((priority_complete, bang, important_ident)) = priority {
        validate_role_evidence(
            source,
            occurrence,
            CssDeclarationEvidenceRole::PriorityComplete,
            priority_complete,
        )?;
        validate_role_evidence(
            source,
            occurrence,
            CssDeclarationEvidenceRole::PriorityBang,
            bang,
        )?;
        validate_role_evidence(
            source,
            occurrence,
            CssDeclarationEvidenceRole::PriorityImportantIdent,
            important_ident,
        )?;

        require_contained(
            complete,
            priority_complete,
            occurrence,
            CssOccurrenceRelationship::PriorityContained,
        )?;
        if priority_complete.range().start() != bang.range().start() {
            return Err(relationship_violation(
                occurrence,
                CssOccurrenceRelationship::PriorityStartsAtBang,
            ));
        }
        if priority_complete.range().end() != important_ident.range().end() {
            return Err(relationship_violation(
                occurrence,
                CssOccurrenceRelationship::PriorityEndsAtImportantIdent,
            ));
        }
        if bang.range().end() > important_ident.range().start() {
            return Err(relationship_violation(
                occurrence,
                CssOccurrenceRelationship::BangEndsAtOrBeforeImportantIdentStart,
            ));
        }

        let value_end_or_point = if value.range().is_empty() {
            value.range().start()
        } else {
            value.range().end()
        };
        if value_end_or_point > priority_complete.range().start() {
            return Err(relationship_violation(
                occurrence,
                CssOccurrenceRelationship::ValueDoesNotOverlapPriority,
            ));
        }

        priority_complete.range().end()
    } else {
        last_semantic_end
    };

    match termination {
        CssDeclarationTermination::AuthoredSemicolon { semicolon } => {
            validate_role_evidence(
                source,
                occurrence,
                CssDeclarationEvidenceRole::Semicolon,
                semicolon,
            )?;
            require_contained(
                complete,
                semicolon,
                occurrence,
                CssOccurrenceRelationship::AuthoredSemicolonContained,
            )?;
            if semicolon.range().start() < last_semantic_end {
                return Err(relationship_violation(
                    occurrence,
                    CssOccurrenceRelationship::AuthoredSemicolonNotBeforeLastEvidence,
                ));
            }
            if complete.range().end() != semicolon.range().end() {
                return Err(relationship_violation(
                    occurrence,
                    CssOccurrenceRelationship::AuthoredSemicolonEndsComplete,
                ));
            }
        }
        CssDeclarationTermination::OmittedBeforeRightCurly { right_curly } => {
            validate_role_evidence(
                source,
                occurrence,
                CssDeclarationEvidenceRole::RightCurly,
                right_curly,
            )?;
            if right_curly.range().start() < complete.range().end() {
                return Err(relationship_violation(
                    occurrence,
                    CssOccurrenceRelationship::RightCurlyAtOrAfterCompleteEnd,
                ));
            }
        }
        CssDeclarationTermination::OmittedAtEndOfInput { terminal } => {
            validate_role_evidence(
                source,
                occurrence,
                CssDeclarationEvidenceRole::EofTerminal,
                terminal,
            )?;
            if !terminal.range().is_empty() {
                return Err(relationship_violation(
                    occurrence,
                    CssOccurrenceRelationship::EofTerminalEmpty,
                ));
            }
            if terminal.range().start() != source.as_str().len() {
                return Err(relationship_violation(
                    occurrence,
                    CssOccurrenceRelationship::EofTerminalAtSourceEnd,
                ));
            }
            if terminal.range().start() < complete.range().end() {
                return Err(relationship_violation(
                    occurrence,
                    CssOccurrenceRelationship::EofTerminalAtOrAfterCompleteEnd,
                ));
            }
        }
    }

    Ok(())
}

fn validate_role_evidence(
    source: &SourceText,
    occurrence: CssAnalysisOccurrenceRef,
    role: CssDeclarationEvidenceRole,
    anchor: &SourceAnchor,
) -> Result<(), CssAnalysisError> {
    if anchor.source_id() != source.id() {
        return Err(CssAnalysisError::OccurrenceSourceIdentityMismatch {
            occurrence,
            role,
            expected: source.id(),
            actual: anchor.source_id(),
        });
    }

    let range = anchor.range();
    source.anchor(range.start(), range.end()).map_err(|error| {
        CssAnalysisError::OccurrenceSourceRangeInvalid {
            occurrence,
            role,
            error,
        }
    })?;

    if !source.retains_exact_anchor_source(anchor) {
        return Err(CssAnalysisError::OccurrenceSourceContentMismatch { occurrence, role });
    }

    Ok(())
}

fn require_contained(
    container: &SourceAnchor,
    nested: &SourceAnchor,
    occurrence: CssAnalysisOccurrenceRef,
    relationship: CssOccurrenceRelationship,
) -> Result<(), CssAnalysisError> {
    if container.source_id() != nested.source_id()
        || nested.range().start() < container.range().start()
        || nested.range().end() > container.range().end()
    {
        return Err(relationship_violation(occurrence, relationship));
    }
    Ok(())
}

const fn relationship_violation(
    occurrence: CssAnalysisOccurrenceRef,
    relationship: CssOccurrenceRelationship,
) -> CssAnalysisError {
    CssAnalysisError::OccurrenceRelationshipViolation {
        occurrence,
        relationship,
    }
}
