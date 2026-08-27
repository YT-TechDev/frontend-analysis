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
//! The existing batch tokenizer remains a compatible sibling entry point; the
//! coordinator does not silently change its contract.
//!
//! The driver also owns same-token redispatch: [`drive_token`] retains one
//! admitted token and its trigger across every session dispatch that token
//! needs, so the session itself performs only one insertion-mode rule
//! evaluation per call. TC-S9 reuses that exact mechanism for Text-mode EOF:
//! no second EOF token is requested.
//!
//! # Fixed configuration
//!
//! The supported construction program remains ordinary document parsing with
//! parser scripting mode **Disabled** and tokenizer initial state **Data**.
//! There is still no public parse-control protocol or browser/runtime input.
//!
//! # Effective completion
//!
//! `Complete` requires all three of: the retained tokenizer run is
//! `Complete`; every emitted token was processed through end of file by
//! supported actions; and freeze succeeded. Lower-layer incompleteness is
//! never upgraded, and the tokenizer's exact incomplete meaning stays
//! authoritative on the retained run.

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

/// A construction operation/boundary failure.
///
/// Distinct from HTML parse diagnostics (authored-input evidence) and from
/// unsupported capability evidence (a missing proved rule): every variant here
/// means the coordination boundary produced something it must never produce.
/// All variants carry structural evidence only; `Debug`/`Display` never expose
/// arbitrary authored source content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlDocumentShellConstructionError {
    /// The private construction session violated one of its own invariants.
    Session(HtmlTreeSessionError),
    /// The private tokenizer session violated a coordinator-control invariant.
    Tokenizer(HtmlTokenizerSessionControlError),
    /// Tokenizer and tree reached mutually inconsistent coordination states.
    Coordination(HtmlTreeCoordinatorError),
    /// Validated freeze rejected the construction output.
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

/// Structural coordinator mismatches for the private TC-S9 feedback seam.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTreeCoordinatorError {
    FeedbackRequestedWithoutMatchingTokenizerSuspension {
        requested: HtmlTreeTokenizerFeedback,
        boundary: HtmlTokenizerSessionBoundary,
    },
    TokenizerSuspendedWithoutTreeFeedback {
        mode: HtmlTokenizerMode,
    },
    FeedbackTokenWasNotLastProducedAtBoundary,
}

/// Why tree processing stopped.
enum Stop {
    /// A supported rule stopped document parsing at the given token index.
    Parsing { processed_tokens: usize },
    /// The current closed production theorem does not prove the reached cell.
    Unsupported(HtmlTreeUnsupportedCapability),
}

/// Constructs the disabled-scripting document tree for the currently accepted
/// bounded HTML production capabilities through TC-S9.
///
/// The caller retains ownership of `source`; this operation borrows it and
/// does not clone the complete source. The returned analysis may outlive the
/// caller's handle through the existing [`crate::SourceAnchor`] ownership
/// contract.
///
/// Returns `Err` only for a tokenizer/session/coordinator/freeze invariant
/// failure. Unsupported input, tokenizer incompleteness, and HTML parse
/// diagnostics are ordinary `Ok` results carrying their own evidence.
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

            match drive_token(&mut session, &admitted, &trigger)? {
                TokenOutcome::Consumed => {
                    next_token_index += 1;
                }
                TokenOutcome::TokenizerFeedbackRequested(feedback) => {
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
                                requested: feedback,
                                boundary,
                            },
                        ));
                    }

                    // Causal two-phase Style start:
                    // tree request -> tokenizer apply -> tree acknowledgement.
                    // Only after acknowledgement may the next outer drive
                    // consume later source. The token index is retained only
                    // as private freeze evidence that this exact transition
                    // really crossed the coordinator boundary.
                    match feedback {
                        HtmlTreeTokenizerFeedback::EnterRawText => tokenizer.apply_raw_text()?,
                    }
                    coordinated_raw_text_entry_tokens.push(next_token_index);
                    session.acknowledge_tokenizer_feedback(feedback)?;
                    next_token_index += 1;
                    continue 'production;
                }
                TokenOutcome::StoppedParsing => {
                    next_token_index += 1;
                    stop = Some(Stop::Parsing {
                        processed_tokens: next_token_index,
                    });
                    break;
                }
                TokenOutcome::Unsupported(capability) => {
                    stop = Some(Stop::Unsupported(HtmlTreeUnsupportedCapability::new(
                        capability, trigger,
                    )));
                    break;
                }
            }
        }

        if stop.is_some() {
            // Preserve lower-layer meaning after an independent tree stop.
            // This drains only under the existing batch-compatible no-tree-
            // feedback policy and never guesses a future context transition.
            break 'production tokenizer.finish_batch_compatible();
        }

        match boundary {
            HtmlTokenizerSessionBoundary::TokenAvailable => {
                // This boundary exists only after the tokenizer itself
                // recognized and emitted the selected appropriate RAWTEXT end
                // tag, returned its lexical state to Data, and yielded before
                // post-close source. Tree dispatch above has now consumed that
                // exact close and restored InHead.
                let close_token_index = produced_len
                    .checked_sub(1)
                    .ok_or(HtmlDocumentShellConstructionError::Coordination(
                        HtmlTreeCoordinatorError::FeedbackTokenWasNotLastProducedAtBoundary,
                    ))?;
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

/// Drives one admitted token to a terminal [`TokenOutcome`].
///
/// The driver owns same-token redispatch here: it retains the same admitted
/// token and trigger across every [`DispatchOutcome::ReprocessSameToken`]
/// result, never re-admitting the token or reconstructing the trigger, and
/// never advancing the tokenizer cursor until this returns.
///
/// Per-token termination is structural rather than a numeric budget: this
/// tracks which insertion modes have already been evaluated while processing
/// this one token, and treats evaluating any mode again as an invariant
/// failure. Because [`InsertionMode`] is a small finite domain, this proves one
/// token cannot cycle and is not a resource limit.
pub(super) fn drive_token(
    session: &mut HtmlTreeSession,
    admitted: &AdmittedToken<'_>,
    trigger: &HtmlTreeTokenTrigger,
) -> Result<TokenOutcome, HtmlTreeSessionError> {
    let mut evaluated_modes: Vec<InsertionMode> = Vec::new();
    loop {
        let mode = session.insertion_mode();
        if evaluated_modes.contains(&mode) {
            return Err(HtmlTreeSessionError::RepeatedInsertionModeEvaluation);
        }
        evaluated_modes.push(mode);
        match session.dispatch(admitted, trigger)? {
            DispatchOutcome::Consumed => return Ok(TokenOutcome::Consumed),
            DispatchOutcome::TokenizerFeedbackRequested(feedback) => {
                return Ok(TokenOutcome::TokenizerFeedbackRequested(feedback));
            }
            DispatchOutcome::ReprocessSameToken => {}
            DispatchOutcome::StoppedParsing => return Ok(TokenOutcome::StoppedParsing),
            DispatchOutcome::Unsupported(capability) => {
                return Ok(TokenOutcome::Unsupported(capability));
            }
        }
    }
}

/// Resolves effective completion from the tree's own stop and the retained
/// tokenizer run without ever upgrading lower-layer incompleteness.
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
