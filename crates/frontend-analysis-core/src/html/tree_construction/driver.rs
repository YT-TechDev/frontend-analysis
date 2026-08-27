//! The Core-owned HTML tree-construction coordinator.
//!
//! The driver owns exactly one operation:
//! [`construct_html_document_shell`]. TC-S1 through TC-S8 could drain the
//! project-owned tokenizer in batch before tree construction because none of
//! those proved cells required tree-directed lexical control. TC-S9 is the
//! first bounded production capability that exercises Candidate C's causal
//! feedback promise.
//!
//! The coordinator now pulls from the same private tokenizer Engine through a
//! crate-private resumable session. It applies a tree semantic
//! [`HtmlTreeTokenizerFeedback::EnterRawText`] request at the tokenizer's
//! existing post-`<style>` suspension before any later source production, then
//! acknowledges that applied feedback back to the tree session. Neither side
//! owns the other's private state representation.
//!
//! The existing batch tokenizer and the predecessor [`drive_token`] helper
//! remain compatible sibling entry points. TC-S9 feedback is represented only
//! by a coordinator-private outcome so predecessor production tests and
//! consumers do not need to learn a new terminal outcome they can never
//! observe under their batch-tokenizer contract.
//!
//! The driver also owns same-token redispatch: the token-driving core retains
//! one admitted token and its trigger across every session dispatch that token
//! needs, so the session itself performs only one insertion-mode rule
//! evaluation per call. TC-S9 reuses that exact mechanism for Text-mode EOF:
//! no second EOF token is requested.

use std::error::Error;
use std::fmt;

use crate::SourceText;

use super::super::tokenizer::producer::{
    HtmlTokenizerSession, HtmlTokenizerSessionBoundary, HtmlTokenizerSessionControlError,
};
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::{HtmlTokenizerMode, HtmlTokenizerRunResult};
use super::result::{
    HtmlDocumentShellAnalysis, HtmlTreeCompletion, HtmlTreeFreezeError, HtmlTreeIncompleteCause,
    HtmlTreeTokenTrigger, HtmlTreeUnsupportedCapability, freeze,
};
use super::session::{
    AdmittedToken, DispatchOutcome, HtmlTreeSession, HtmlTreeSessionError,
    HtmlTreeTokenizerFeedback, InsertionMode, TokenOutcome, admit, token_trigger,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlDocumentShellConstructionError {
    Session(HtmlTreeSessionError),
    Tokenizer(HtmlTokenizerSessionControlError),
    Coordination(HtmlTreeCoordinatorError),
    Freeze(HtmlTreeFreezeError),
}

impl fmt::Display for HtmlDocumentShellConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HTML document shell construction boundary violation: {self:?}"
        )
    }
}

impl Error for HtmlDocumentShellConstructionError {}

impl From<HtmlTreeSessionError> for HtmlDocumentShellConstructionError {
    fn from(error: HtmlTreeSessionError) -> Self {
        Self::Session(error)
    }
}

impl From<HtmlTokenizerSessionControlError> for HtmlDocumentShellConstructionError {
    fn from(error: HtmlTokenizerSessionControlError) -> Self {
        Self::Tokenizer(error)
    }
}

impl From<HtmlTreeFreezeError> for HtmlDocumentShellConstructionError {
    fn from(error: HtmlTreeFreezeError) -> Self {
        Self::Freeze(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeCoordinatorError {
    FeedbackRequestedWithoutMatchingTokenizerSuspension {
        boundary: HtmlTokenizerSessionBoundary,
    },
    TokenizerSuspendedWithoutTreeFeedback {
        mode: HtmlTokenizerMode,
    },
    FeedbackTokenWasNotLastProducedAtBoundary,
}

enum Stop {
    Parsing { processed_tokens: usize },
    Unsupported(HtmlTreeUnsupportedCapability),
}

/// The outcome vocabulary that only the coordinated production loop observes.
///
/// Deliberately separate from the frozen predecessor [`TokenOutcome`].
enum CoordinatedTokenOutcome {
    Consumed,
    TokenizerFeedbackRequested(HtmlTreeTokenizerFeedback),
    StoppedParsing,
    Unsupported(super::result::HtmlTreeCapability),
}

pub(crate) fn construct_html_document_shell(
    source: &SourceText,
    limits: HtmlTokenizerLimits,
) -> Result<HtmlDocumentShellAnalysis, HtmlDocumentShellConstructionError> {
    let mut tokenizer = HtmlTokenizerSession::new(source, limits);
    let mut session = HtmlTreeSession::new()?;
    let mut next_token_index = 0usize;
    let mut stop = None;
    let mut coordinated_raw_text_entry_tokens = Vec::new();
    let mut coordinated_raw_text_close_tokens = Vec::new();

    let tokenizer_run: HtmlTokenizerRunResult = 'production: loop {
        let boundary = tokenizer.drive_to_boundary();
        let produced_len = tokenizer.tokens().len();

        while next_token_index < produced_len {
            let token = &tokenizer.tokens()[next_token_index];
            let trigger = token_trigger(token, next_token_index);
            let admitted = match admit(token) {
                Ok(admitted) => admitted,
                Err(capability) => {
                    stop = Some(Stop::Unsupported(HtmlTreeUnsupportedCapability::new(
                        capability, trigger,
                    )));
                    break;
                }
            };

            match drive_coordinated_token(&mut session, &admitted, &trigger)? {
                CoordinatedTokenOutcome::Consumed => {
                    next_token_index += 1;
                }
                CoordinatedTokenOutcome::TokenizerFeedbackRequested(feedback) => {
                    if next_token_index + 1 != produced_len {
                        return Err(HtmlDocumentShellConstructionError::Coordination(
                            HtmlTreeCoordinatorError::FeedbackTokenWasNotLastProducedAtBoundary,
                        ));
                    }
                    let expected_boundary = match feedback {
                        HtmlTreeTokenizerFeedback::EnterRawText => {
                            HtmlTokenizerSessionBoundary::Suspended(HtmlTokenizerMode::RawText)
                        }
                    };
                    if boundary != expected_boundary {
                        return Err(HtmlDocumentShellConstructionError::Coordination(
                            HtmlTreeCoordinatorError::FeedbackRequestedWithoutMatchingTokenizerSuspension {
                                boundary,
                            },
                        ));
                    }

                    match feedback {
                        HtmlTreeTokenizerFeedback::EnterRawText => tokenizer.apply_raw_text()?,
                    }
                    coordinated_raw_text_entry_tokens.push(next_token_index);
                    session.acknowledge_tokenizer_feedback(feedback)?;
                    next_token_index += 1;
                    continue 'production;
                }
                CoordinatedTokenOutcome::StoppedParsing => {
                    next_token_index += 1;
                    stop = Some(Stop::Parsing {
                        processed_tokens: next_token_index,
                    });
                    break;
                }
                CoordinatedTokenOutcome::Unsupported(capability) => {
                    stop = Some(Stop::Unsupported(HtmlTreeUnsupportedCapability::new(
                        capability, trigger,
                    )));
                    break;
                }
            }
        }

        if stop.is_some() {
            break 'production tokenizer.finish_batch_compatible();
        }

        match boundary {
            HtmlTokenizerSessionBoundary::TokenAvailable => {
                let close_token_index = produced_len.checked_sub(1).ok_or(
                    HtmlDocumentShellConstructionError::Coordination(
                        HtmlTreeCoordinatorError::FeedbackTokenWasNotLastProducedAtBoundary,
                    ),
                )?;
                coordinated_raw_text_close_tokens.push(close_token_index);
            }
            HtmlTokenizerSessionBoundary::Suspended(mode) => {
                return Err(HtmlDocumentShellConstructionError::Coordination(
                    HtmlTreeCoordinatorError::TokenizerSuspendedWithoutTreeFeedback { mode },
                ));
            }
            HtmlTokenizerSessionBoundary::Terminal => {
                break 'production tokenizer.into_result()?;
            }
        }
    };

    let completion = effective_completion(&stop, &tokenizer_run);
    let mut parts = session.finish(completion);
    parts.coordinated_raw_text_entry_tokens = coordinated_raw_text_entry_tokens;
    parts.coordinated_raw_text_close_tokens = coordinated_raw_text_close_tokens;
    Ok(freeze(source, tokenizer_run, parts)?)
}

/// Frozen predecessor helper used by TC-S1–TC-S8 production correspondence.
///
/// A TC-S9 feedback request is an invariant mismatch for this non-coordinated
/// helper, not a new [`TokenOutcome`] variant. Under the predecessor batch
/// tokenizer contract that condition is unreachable because tokenization stops
/// at the context-dependent boundary first.
pub(super) fn drive_token(
    session: &mut HtmlTreeSession,
    admitted: &AdmittedToken<'_>,
    trigger: &HtmlTreeTokenTrigger,
) -> Result<TokenOutcome, HtmlTreeSessionError> {
    match drive_coordinated_token(session, admitted, trigger)? {
        CoordinatedTokenOutcome::Consumed => Ok(TokenOutcome::Consumed),
        CoordinatedTokenOutcome::StoppedParsing => Ok(TokenOutcome::StoppedParsing),
        CoordinatedTokenOutcome::Unsupported(capability) => {
            Ok(TokenOutcome::Unsupported(capability))
        }
        CoordinatedTokenOutcome::TokenizerFeedbackRequested(_) => {
            Err(HtmlTreeSessionError::TokenizerFeedbackRequiresCoordinator)
        }
    }
}

fn drive_coordinated_token(
    session: &mut HtmlTreeSession,
    admitted: &AdmittedToken<'_>,
    trigger: &HtmlTreeTokenTrigger,
) -> Result<CoordinatedTokenOutcome, HtmlTreeSessionError> {
    let mut evaluated_modes: Vec<InsertionMode> = Vec::new();
    loop {
        let mode = session.insertion_mode();
        if evaluated_modes.contains(&mode) {
            return Err(HtmlTreeSessionError::RepeatedInsertionModeEvaluation);
        }
        evaluated_modes.push(mode);
        match session.dispatch(admitted, trigger)? {
            DispatchOutcome::Consumed => return Ok(CoordinatedTokenOutcome::Consumed),
            DispatchOutcome::TokenizerFeedbackRequested(feedback) => {
                return Ok(CoordinatedTokenOutcome::TokenizerFeedbackRequested(
                    feedback,
                ));
            }
            DispatchOutcome::ReprocessSameToken => {}
            DispatchOutcome::StoppedParsing => return Ok(CoordinatedTokenOutcome::StoppedParsing),
            DispatchOutcome::Unsupported(capability) => {
                return Ok(CoordinatedTokenOutcome::Unsupported(capability));
            }
        }
    }
}

fn effective_completion(
    stop: &Option<Stop>,
    tokenizer_run: &HtmlTokenizerRunResult,
) -> HtmlTreeCompletion {
    match stop {
        Some(Stop::Unsupported(unsupported)) => HtmlTreeCompletion::Incomplete(
            HtmlTreeIncompleteCause::UnsupportedCapability(unsupported.clone()),
        ),
        Some(Stop::Parsing { processed_tokens })
            if !tokenizer_run.is_incomplete()
                && *processed_tokens == tokenizer_run.tokens().len() =>
        {
            HtmlTreeCompletion::Complete
        }
        Some(Stop::Parsing { .. }) | None => {
            HtmlTreeCompletion::Incomplete(HtmlTreeIncompleteCause::LowerLayerIncomplete)
        }
    }
}
