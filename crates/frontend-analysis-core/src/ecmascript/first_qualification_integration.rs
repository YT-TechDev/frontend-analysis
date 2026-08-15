//! First selected aggregate qualification integration for Issue #213.
//!
//! This module composes the existing #204 selected lexical/syntax owner with the
//! existing #210 selected static-semantics owner. It preserves staged
//! implementation coverage explicitly and does not construct aggregate
//! `QualificationOutcome::Qualified`.

use crate::SourceText;

use super::first_lexical_slice::{FirstLexicalSliceOutcome, recognize_first_lexical_slice};
use super::first_static_semantics::{
    SelectedStaticSemanticsOutcome, evaluate_first_static_semantics,
    selected_rejection_to_qualification,
};
use super::qualification::QualificationOutcome;

/// Aggregate meaning for the currently implemented selected qualification path.
///
/// `SelectedAcceptedIncomplete` means every obligation implemented by the
/// selected #204 + #210 path accepted the authoritative source, while broader
/// first-envelope aggregate completeness remains intentionally unproven.
#[derive(Debug)]
pub(super) enum FirstSelectedQualificationAttempt {
    UnsupportedCoverage,
    SelectedAcceptedIncomplete,
    Outcome(QualificationOutcome),
}

/// Runs the first selected qualification integration over authoritative Core
/// `SourceText` without adding a second source interpretation path.
pub(super) fn attempt_first_selected_qualification(
    source: &SourceText,
) -> FirstSelectedQualificationAttempt {
    match recognize_first_lexical_slice(source) {
        FirstLexicalSliceOutcome::RecognizedSelectedSlice(script) => {
            match evaluate_first_static_semantics(&script) {
                SelectedStaticSemanticsOutcome::Accepted => {
                    FirstSelectedQualificationAttempt::SelectedAcceptedIncomplete
                }
                SelectedStaticSemanticsOutcome::Rejected(rejection) => {
                    FirstSelectedQualificationAttempt::Outcome(
                        selected_rejection_to_qualification(source, &rejection),
                    )
                }
                SelectedStaticSemanticsOutcome::ResourceLimited => {
                    FirstSelectedQualificationAttempt::Outcome(
                        QualificationOutcome::resource_limited(),
                    )
                }
            }
        }
        FirstLexicalSliceOutcome::UnsupportedCoverage => {
            FirstSelectedQualificationAttempt::UnsupportedCoverage
        }
        FirstLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence => {
            // The current #204 marker carries no truthful rejection payload and
            // currently has no producer. Fail closed rather than fabricating a
            // whole-source SyntaxRejected verdict.
            FirstSelectedQualificationAttempt::Outcome(QualificationOutcome::internal_failure())
        }
        FirstLexicalSliceOutcome::ResourceLimited => FirstSelectedQualificationAttempt::Outcome(
            QualificationOutcome::resource_limited(),
        ),
        FirstLexicalSliceOutcome::InternalFailure => FirstSelectedQualificationAttempt::Outcome(
            QualificationOutcome::internal_failure(),
        ),
    }
}
