//! Selected aggregate qualification integration for Issue #218.
//!
//! This module composes the selected lexical owner with the selected
//! static-semantics owner while preserving staged implementation coverage. It
//! does not construct aggregate `QualificationOutcome::Qualified`.

use crate::SourceText;

use super::qualification::QualificationOutcome;
use super::selected_lexical_slice::{
    SelectedLexicalSliceOutcome, recognize_selected_lexical_slice,
};
use super::selected_static_semantics::{
    SelectedStaticSemanticsOutcome, evaluate_selected_static_semantics,
    selected_rejection_to_qualification,
};

#[derive(Debug)]
pub(super) enum SelectedQualificationAttempt {
    UnsupportedCoverage,
    SelectedAcceptedIncomplete,
    Outcome(QualificationOutcome),
}

pub(super) fn attempt_selected_qualification(source: &SourceText) -> SelectedQualificationAttempt {
    match recognize_selected_lexical_slice(source) {
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(script) => {
            match evaluate_selected_static_semantics(&script) {
                SelectedStaticSemanticsOutcome::Accepted => {
                    SelectedQualificationAttempt::SelectedAcceptedIncomplete
                }
                SelectedStaticSemanticsOutcome::Rejected(rejection) => {
                    SelectedQualificationAttempt::Outcome(selected_rejection_to_qualification(
                        source, &rejection,
                    ))
                }
                SelectedStaticSemanticsOutcome::ResourceLimited => {
                    SelectedQualificationAttempt::Outcome(QualificationOutcome::resource_limited())
                }
                SelectedStaticSemanticsOutcome::InternalFailure => {
                    SelectedQualificationAttempt::Outcome(QualificationOutcome::internal_failure())
                }
            }
        }
        SelectedLexicalSliceOutcome::UnsupportedCoverage => {
            SelectedQualificationAttempt::UnsupportedCoverage
        }
        SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence => {
            SelectedQualificationAttempt::Outcome(QualificationOutcome::internal_failure())
        }
        SelectedLexicalSliceOutcome::ResourceLimited => {
            SelectedQualificationAttempt::Outcome(QualificationOutcome::resource_limited())
        }
        SelectedLexicalSliceOutcome::InternalFailure => {
            SelectedQualificationAttempt::Outcome(QualificationOutcome::internal_failure())
        }
    }
}
