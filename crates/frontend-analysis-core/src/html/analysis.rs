//! First Core-integrated HTML analysis vertical slice.
//!
//! Connects retained `&SourceText` through the project-owned HTML tokenizer
//! (`super::tokenizer::producer::tokenize`) and the project-owned
//! explicit-start-tag analysis parser
//! (`super::parser::analyze_explicit_start_tags`) into one synchronous,
//! crate-visible Core operation, per the architecture approved on #116.
//!
//! Tokenization and analysis parsing remain internal implementation stages:
//! this boundary exposes neither tokenizer state, parser mutation state, nor
//! an external parser abstraction. The tokenizer and parser remain the
//! semantic owners of token, diagnostic, coverage, completion, unsupported,
//! and resource-run invariants; this module does not duplicate that
//! validator. It adds exactly one additional responsibility: validating the
//! parser-projected occurrence evidence against the exact supplied
//! [`SourceText`], per [`validate_occurrence_evidence`].

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRangeError, SourceText};

use super::parser::{
    HtmlAnalysisParserContractError, HtmlExplicitStartTagAnalysis, HtmlExplicitStartTagOccurrence,
    analyze_explicit_start_tags,
};
use super::tokenizer::producer::tokenize;
use super::tokenizer::resource::HtmlTokenizerLimits;

#[cfg(test)]
mod tests;

/// Which projected occurrence evidence a [`HtmlStartTagAnalysisError`]
/// source-evidence variant concerns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlOccurrenceEvidenceRole {
    /// The complete authored start-tag evidence.
    CompleteTag,
    /// The raw tag-name evidence.
    RawName,
}

/// A Core-integration boundary failure for the first HTML analysis vertical
/// slice.
///
/// Distinct from tokenizer diagnostics (authored-input evidence) and from
/// [`HtmlAnalysisParserContractError`] (a parser-internal invariant
/// failure): this vocabulary covers only failures owned by this Core
/// boundary, validating parser-projected occurrence evidence against the
/// exact supplied [`SourceText`]. A parser/Core contract failure always
/// becomes `Err`, never clean success or clean absence.
///
/// `Debug` output intentionally carries only structural evidence (occurrence
/// indices, roles, [`SourceId`], and [`SourceRangeError`]) and never
/// arbitrary authored source content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlStartTagAnalysisError {
    /// The retained analysis parser reported its own internal contract
    /// violation. See [`HtmlAnalysisParserContractError`].
    ParserContract(HtmlAnalysisParserContractError),
    /// The occurrence evidence for the given role is bound to a source
    /// identity other than the exact supplied [`SourceText`].
    OccurrenceSourceIdentityMismatch {
        occurrence_index: usize,
        role: HtmlOccurrenceEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    /// The occurrence evidence's already-projected range could not be
    /// revalidated through the exact supplied [`SourceText`].
    OccurrenceSourceRangeInvalid {
        occurrence_index: usize,
        role: HtmlOccurrenceEvidenceRole,
        error: SourceRangeError,
    },
    /// The occurrence evidence's range revalidated, but the exact supplied
    /// [`SourceText`] carries different content at that range.
    OccurrenceSourceContentMismatch {
        occurrence_index: usize,
        role: HtmlOccurrenceEvidenceRole,
    },
    /// The occurrence's raw tag-name evidence is not contained within its
    /// complete authored start-tag evidence.
    OccurrenceContainmentViolation { occurrence_index: usize },
}

impl fmt::Display for HtmlStartTagAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HTML start-tag analysis boundary violation: {self:?}"
        )
    }
}

impl Error for HtmlStartTagAnalysisError {}

/// Connects retained `&SourceText` through the project-owned HTML tokenizer
/// and explicit-start-tag analysis parser into the first Core-owned
/// end-to-end HTML analysis operation, per the architecture approved on
/// #116.
///
/// The caller retains ownership of `source`; this operation borrows it for
/// execution and does not clone the complete source. The returned
/// [`HtmlExplicitStartTagAnalysis`] may outlive the caller's `source` handle
/// through the existing [`SourceAnchor`] ownership contract.
///
/// Tokenizer completion, diagnostics, coverage, unsupported, and resource
/// evidence remain authoritative and unchanged through
/// [`HtmlExplicitStartTagAnalysis::tokenizer_run`]; this operation never
/// upgrades or reinterprets that meaning. A parser-internal contract failure
/// or a Core source-evidence validation failure both become `Err`.
pub(crate) fn analyze_html_explicit_start_tags(
    source: &SourceText,
    limits: HtmlTokenizerLimits,
) -> Result<HtmlExplicitStartTagAnalysis, HtmlStartTagAnalysisError> {
    let tokenizer_run = tokenize(source, limits);
    let analysis = analyze_explicit_start_tags(tokenizer_run)
        .map_err(HtmlStartTagAnalysisError::ParserContract)?;

    for (occurrence_index, occurrence) in analysis.occurrences().iter().enumerate() {
        validate_occurrence_against_source(source, occurrence_index, occurrence)?;
    }

    Ok(analysis)
}

/// Validates one projected occurrence's evidence against the exact supplied
/// `source`. Delegates to [`validate_occurrence_evidence`], which stays
/// production-relevant while accepting the anchors directly so tests may
/// exercise it without widening `HtmlExplicitStartTagOccurrence`'s private
/// fields.
fn validate_occurrence_against_source(
    source: &SourceText,
    occurrence_index: usize,
    occurrence: &HtmlExplicitStartTagOccurrence,
) -> Result<(), HtmlStartTagAnalysisError> {
    validate_occurrence_evidence(
        source,
        occurrence_index,
        occurrence.complete(),
        occurrence.raw_name(),
    )
}

/// Validates already-projected occurrence evidence against the exact
/// supplied `source`. This is validation of projected evidence, not source
/// discovery: it performs no source search, delimiter scan, endpoint
/// reconstruction, or retokenization. For each role it revalidates the
/// already-projected range through [`SourceText::anchor`] and reconciles
/// the revalidated fragment against the occurrence's own retained fragment,
/// then checks that `raw_name` is contained within `complete`.
fn validate_occurrence_evidence(
    source: &SourceText,
    occurrence_index: usize,
    complete: &SourceAnchor,
    raw_name: &SourceAnchor,
) -> Result<(), HtmlStartTagAnalysisError> {
    validate_role_evidence(
        source,
        occurrence_index,
        HtmlOccurrenceEvidenceRole::CompleteTag,
        complete,
    )?;
    validate_role_evidence(
        source,
        occurrence_index,
        HtmlOccurrenceEvidenceRole::RawName,
        raw_name,
    )?;

    if complete.range().start() > raw_name.range().start()
        || raw_name.range().end() > complete.range().end()
    {
        return Err(HtmlStartTagAnalysisError::OccurrenceContainmentViolation { occurrence_index });
    }
    Ok(())
}

fn validate_role_evidence(
    source: &SourceText,
    occurrence_index: usize,
    role: HtmlOccurrenceEvidenceRole,
    anchor: &SourceAnchor,
) -> Result<(), HtmlStartTagAnalysisError> {
    if anchor.source_id() != source.id() {
        return Err(
            HtmlStartTagAnalysisError::OccurrenceSourceIdentityMismatch {
                occurrence_index,
                role,
                expected: source.id(),
                actual: anchor.source_id(),
            },
        );
    }

    let range = anchor.range();
    let revalidated = source.anchor(range.start(), range.end()).map_err(|error| {
        HtmlStartTagAnalysisError::OccurrenceSourceRangeInvalid {
            occurrence_index,
            role,
            error,
        }
    })?;

    if revalidated.fragment() != anchor.fragment() {
        return Err(HtmlStartTagAnalysisError::OccurrenceSourceContentMismatch {
            occurrence_index,
            role,
        });
    }

    Ok(())
}
