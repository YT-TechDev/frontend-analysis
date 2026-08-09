//! First Core-integrated CSS declaration analysis vertical slice (#140).
//!
//! Connects retained `&SourceText` through the project-owned CSS tokenizer
//! (`super::tokenizer::producer::run`) and the project-owned bounded
//! declaration parser (`super::parser::producer::run`) into one synchronous,
//! crate-private Core operation, per the architecture approved on #140.
//! Tokenization and declaration parsing remain internal implementation
//! stages: this boundary exposes neither tokenizer/parser cursor or
//! checkpoint internals nor a generic parser abstraction. The tokenizer and
//! parser remain the semantic owners of lexical, diagnostic, recovery,
//! discard, unsupported, completion, termination, and resource-run
//! invariants; this module does not duplicate that validation and does not
//! claim complete CSS parsing, CSSOM membership, cascade participation, or
//! browser support. On success it returns the exact, unmodified `Core`
//! upstream result type `CssParserRunResult` already produced by
//! `super::parser::producer::run`: no second result hierarchy is introduced.
//!
//! This module adds exactly one additional responsibility: revalidating the
//! parser-projected declaration occurrence evidence against the exact
//! supplied [`SourceText`], per [`validate_occurrence_evidence`]. Revalidation
//! reconciles already-projected [`SourceAnchor`] evidence (source identity,
//! range, and retained fragment) and checks the minimal approved structural
//! relationships among occurrence anchors; it never searches source text,
//! scans for delimiters, retokenizes, reparses, or reconstructs an endpoint
//! from a decoded value. CR/LF/CRLF/FF/NUL/BOM preprocessing and raw-to-
//! semantic mapping remain entirely tokenizer-owned; this module trusts that
//! already-validated boundary rather than duplicating it.

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

#[cfg(test)]
mod tests;

/// The narrow structural relationship a [`CssDeclarationAnalysisError::OccurrenceRelationshipViolation`]
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

/// A Core-integration boundary failure for the first CSS declaration
/// analysis vertical slice.
///
/// Distinct from tokenizer/parser diagnostics (authored-input evidence) and
/// from [`CssTokenizerRunError`] / [`CssParserRunError`] (lower-layer
/// internal contract failures, wrapped here unchanged): this vocabulary
/// covers only failures owned by this Core boundary, validating
/// parser-projected declaration occurrence evidence against the exact
/// supplied [`SourceText`]. A parser/Core contract failure always becomes
/// `Err`, never clean success, clean absence, unsupported coverage, or
/// resource-limit completion.
///
/// `Debug`/`Display` output intentionally carries only structural evidence
/// (occurrence indices, evidence roles, relationship kinds, [`SourceId`],
/// and [`SourceRangeError`]) and never arbitrary authored source content or
/// decoded user-controlled strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssDeclarationAnalysisError {
    /// The project-owned tokenizer reported its own run failure.
    TokenizerRun(CssTokenizerRunError),
    /// The project-owned bounded declaration parser reported its own run
    /// failure.
    ParserRun(CssParserRunError),
    /// The occurrence evidence for the given role is bound to a source
    /// identity other than the exact supplied [`SourceText`].
    OccurrenceSourceIdentityMismatch {
        occurrence_index: usize,
        role: CssDeclarationEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    /// The occurrence evidence's already-projected range could not be
    /// revalidated through the exact supplied [`SourceText`].
    OccurrenceSourceRangeInvalid {
        occurrence_index: usize,
        role: CssDeclarationEvidenceRole,
        error: SourceRangeError,
    },
    /// The occurrence evidence's range revalidated, but the exact supplied
    /// [`SourceText`] carries different content at that range.
    OccurrenceSourceContentMismatch {
        occurrence_index: usize,
        role: CssDeclarationEvidenceRole,
    },
    /// The occurrence's independently-valid anchors violate one of the
    /// minimal approved structural relationships among them.
    OccurrenceRelationshipViolation {
        occurrence_index: usize,
        relationship: CssOccurrenceRelationship,
    },
}

impl From<CssTokenizerRunError> for CssDeclarationAnalysisError {
    fn from(error: CssTokenizerRunError) -> Self {
        Self::TokenizerRun(error)
    }
}

impl From<CssParserRunError> for CssDeclarationAnalysisError {
    fn from(error: CssParserRunError) -> Self {
        Self::ParserRun(error)
    }
}

impl fmt::Display for CssDeclarationAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS declaration analysis boundary violation: {self:?}"
        )
    }
}

impl Error for CssDeclarationAnalysisError {}

/// Connects retained `&SourceText` through the project-owned CSS tokenizer
/// and bounded declaration parser into the first Core-owned end-to-end CSS
/// declaration analysis operation, per the architecture approved on #140.
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
pub(crate) fn analyze_css_declarations(
    source: &SourceText,
    tokenizer_limits: CssTokenizerLimits,
    parser_limits: CssParserLimits,
) -> Result<CssParserRunResult, CssDeclarationAnalysisError> {
    let tokenizer_result = run_tokenizer(source, tokenizer_limits)?;
    let result = run_parser(source, tokenizer_result, parser_limits)?;

    for (occurrence_index, occurrence) in result.occurrences().iter().enumerate() {
        validate_occurrence_against_source(source, occurrence_index, occurrence)?;
    }

    Ok(result)
}

/// Validates one projected occurrence's evidence against the exact supplied
/// `source`. Delegates to [`validate_occurrence_evidence`], which stays
/// production-relevant while accepting the anchors directly so tests may
/// exercise it without widening `CssDeclarationOccurrence`'s private fields.
fn validate_occurrence_against_source(
    source: &SourceText,
    occurrence_index: usize,
    occurrence: &CssDeclarationOccurrence,
) -> Result<(), CssDeclarationAnalysisError> {
    validate_occurrence_evidence(
        source,
        occurrence_index,
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
/// [`SourceText::anchor`] and reconciles the revalidated fragment against
/// the occurrence's own retained fragment, then checks the minimal approved
/// structural relationships among the anchors.
///
/// Property/colon/semicolon/right-curly fixed spelling and the decoded
/// `important` identifier match remain owned by the declaration domain
/// (`super::declaration`), which already validated them at construction
/// time; this boundary does not re-derive or re-check spelling or decoded
/// semantics.
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
) -> Result<(), CssDeclarationAnalysisError> {
    validate_role_evidence(
        source,
        occurrence_index,
        CssDeclarationEvidenceRole::Complete,
        complete,
    )?;
    validate_role_evidence(
        source,
        occurrence_index,
        CssDeclarationEvidenceRole::PropertyName,
        property_name,
    )?;
    validate_role_evidence(
        source,
        occurrence_index,
        CssDeclarationEvidenceRole::Colon,
        colon,
    )?;
    validate_role_evidence(
        source,
        occurrence_index,
        CssDeclarationEvidenceRole::Value,
        value,
    )?;

    if complete.range().start() != property_name.range().start() {
        return Err(relationship_violation(
            occurrence_index,
            CssOccurrenceRelationship::CompleteStartMatchesPropertyName,
        ));
    }
    require_contained(
        complete,
        property_name,
        occurrence_index,
        CssOccurrenceRelationship::PropertyNameContained,
    )?;
    require_contained(
        complete,
        colon,
        occurrence_index,
        CssOccurrenceRelationship::ColonContained,
    )?;
    if property_name.range().end() > colon.range().start() {
        return Err(relationship_violation(
            occurrence_index,
            CssOccurrenceRelationship::PropertyNameEndsAtOrBeforeColonStart,
        ));
    }

    let last_semantic_end = if value.range().is_empty() {
        if value.range().start() != colon.range().end() {
            return Err(relationship_violation(
                occurrence_index,
                CssOccurrenceRelationship::ZeroLengthValueAtColonEnd,
            ));
        }
        colon.range().end()
    } else {
        require_contained(
            complete,
            value,
            occurrence_index,
            CssOccurrenceRelationship::ValueContained,
        )?;
        if colon.range().end() > value.range().start() {
            return Err(relationship_violation(
                occurrence_index,
                CssOccurrenceRelationship::ColonEndsAtOrBeforeValueStart,
            ));
        }
        value.range().end()
    };

    let last_semantic_end = if let Some((priority_complete, bang, important_ident)) = priority {
        validate_role_evidence(
            source,
            occurrence_index,
            CssDeclarationEvidenceRole::PriorityComplete,
            priority_complete,
        )?;
        validate_role_evidence(
            source,
            occurrence_index,
            CssDeclarationEvidenceRole::PriorityBang,
            bang,
        )?;
        validate_role_evidence(
            source,
            occurrence_index,
            CssDeclarationEvidenceRole::PriorityImportantIdent,
            important_ident,
        )?;

        require_contained(
            complete,
            priority_complete,
            occurrence_index,
            CssOccurrenceRelationship::PriorityContained,
        )?;
        if priority_complete.range().start() != bang.range().start() {
            return Err(relationship_violation(
                occurrence_index,
                CssOccurrenceRelationship::PriorityStartsAtBang,
            ));
        }
        if priority_complete.range().end() != important_ident.range().end() {
            return Err(relationship_violation(
                occurrence_index,
                CssOccurrenceRelationship::PriorityEndsAtImportantIdent,
            ));
        }
        if bang.range().end() > important_ident.range().start() {
            return Err(relationship_violation(
                occurrence_index,
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
                occurrence_index,
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
                occurrence_index,
                CssDeclarationEvidenceRole::Semicolon,
                semicolon,
            )?;
            require_contained(
                complete,
                semicolon,
                occurrence_index,
                CssOccurrenceRelationship::AuthoredSemicolonContained,
            )?;
            if semicolon.range().start() < last_semantic_end {
                return Err(relationship_violation(
                    occurrence_index,
                    CssOccurrenceRelationship::AuthoredSemicolonNotBeforeLastEvidence,
                ));
            }
            if complete.range().end() != semicolon.range().end() {
                return Err(relationship_violation(
                    occurrence_index,
                    CssOccurrenceRelationship::AuthoredSemicolonEndsComplete,
                ));
            }
        }
        CssDeclarationTermination::OmittedBeforeRightCurly { right_curly } => {
            validate_role_evidence(
                source,
                occurrence_index,
                CssDeclarationEvidenceRole::RightCurly,
                right_curly,
            )?;
            if right_curly.range().start() < complete.range().end() {
                return Err(relationship_violation(
                    occurrence_index,
                    CssOccurrenceRelationship::RightCurlyAtOrAfterCompleteEnd,
                ));
            }
        }
        CssDeclarationTermination::OmittedAtEndOfInput { terminal } => {
            validate_role_evidence(
                source,
                occurrence_index,
                CssDeclarationEvidenceRole::EofTerminal,
                terminal,
            )?;
            if !terminal.range().is_empty() {
                return Err(relationship_violation(
                    occurrence_index,
                    CssOccurrenceRelationship::EofTerminalEmpty,
                ));
            }
            if terminal.range().start() != source.as_str().len() {
                return Err(relationship_violation(
                    occurrence_index,
                    CssOccurrenceRelationship::EofTerminalAtSourceEnd,
                ));
            }
            if terminal.range().start() < complete.range().end() {
                return Err(relationship_violation(
                    occurrence_index,
                    CssOccurrenceRelationship::EofTerminalAtOrAfterCompleteEnd,
                ));
            }
        }
    }

    Ok(())
}

fn validate_role_evidence(
    source: &SourceText,
    occurrence_index: usize,
    role: CssDeclarationEvidenceRole,
    anchor: &SourceAnchor,
) -> Result<(), CssDeclarationAnalysisError> {
    if anchor.source_id() != source.id() {
        return Err(
            CssDeclarationAnalysisError::OccurrenceSourceIdentityMismatch {
                occurrence_index,
                role,
                expected: source.id(),
                actual: anchor.source_id(),
            },
        );
    }

    let range = anchor.range();
    let revalidated = source.anchor(range.start(), range.end()).map_err(|error| {
        CssDeclarationAnalysisError::OccurrenceSourceRangeInvalid {
            occurrence_index,
            role,
            error,
        }
    })?;

    if revalidated.fragment() != anchor.fragment() {
        return Err(
            CssDeclarationAnalysisError::OccurrenceSourceContentMismatch {
                occurrence_index,
                role,
            },
        );
    }

    Ok(())
}

fn require_contained(
    container: &SourceAnchor,
    nested: &SourceAnchor,
    occurrence_index: usize,
    relationship: CssOccurrenceRelationship,
) -> Result<(), CssDeclarationAnalysisError> {
    if container.source_id() != nested.source_id()
        || nested.range().start() < container.range().start()
        || nested.range().end() > container.range().end()
    {
        return Err(relationship_violation(occurrence_index, relationship));
    }
    Ok(())
}

const fn relationship_violation(
    occurrence_index: usize,
    relationship: CssOccurrenceRelationship,
) -> CssDeclarationAnalysisError {
    CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
        occurrence_index,
        relationship,
    }
}
