//! The Core-owned TC-S1 coordinator.
//!
//! The driver owns exactly one operation:
//! [`construct_html_document_shell`]. It invokes the existing project-owned
//! batch tokenizer unchanged, feeds the validated emitted tokens to the
//! private [`session`](super::session) in order, owns effective completion
//! orchestration, and performs the consuming finalization that turns the
//! session into an immutable, validated result.
//!
//! # Fixed configuration
//!
//! TC-S1 is fixed to ordinary document parsing, parser scripting mode
//! **Disabled**, and the tokenizer's initial **Data** state. There is no
//! parse-request, parse-context, or capability-configuration parameter, so an
//! unsupported configuration cannot be selected — silently or otherwise. The
//! only caller-supplied configuration is the existing
//! [`HtmlTokenizerLimits`], which stays the tokenizer's own contract and is
//! not reinterpreted as a tree-construction budget.
//!
//! Parser scripting is configuration, not JavaScript execution: nothing here
//! executes script. TC-S1's proved action set moreover contains no cell whose
//! behavior differs between scripting enabled and disabled, because every
//! scripting-sensitive element (`script`, `noscript`, `template`) lies outside
//! the `html`/`head`/`body` shell and is therefore explicitly unsupported.
//!
//! # Batch tokenization is sufficient here
//!
//! TC-S1's theorem requires no tokenizer feedback: none of its proved cells
//! changes tokenizer state, and every tokenizer state that tree construction
//! would have to control is already the tokenizer's own explicit unsupported
//! capability. The completed token vector is therefore correct input for this
//! slice, and no resumable-tokenizer seam is introduced.
//!
//! # Effective completion
//!
//! `Complete` requires all three of: the retained tokenizer run is
//! `Complete`; every emitted token was processed through end of file by
//! supported TC-S1 actions; and freeze succeeded. Lower-layer incompleteness
//! is never upgraded, and the tokenizer's exact
//! `UnsupportedCapability`/`ResourceLimit`/`InvalidConfiguration`/
//! `InternalInvariantFailure` meaning is never copied into a lossy duplicate:
//! it stays authoritative on the retained run.

use std::error::Error;
use std::fmt;

use crate::SourceText;

use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::result::{
    HtmlDocumentShellAnalysis, HtmlTreeCompletion, HtmlTreeFreezeError, HtmlTreeIncompleteCause,
    HtmlTreeUnsupportedCapability, freeze,
};
use super::session::{HtmlTreeSession, HtmlTreeSessionError, TokenOutcome, admit, token_trigger};

/// A TC-S1 operation/boundary failure.
///
/// Distinct from HTML parse diagnostics (authored-input evidence) and from
/// unsupported capability evidence (a missing proved rule): both variants here
/// mean the construction boundary produced something it must never produce.
/// Both carry only structural evidence; `Debug` and `Display` never expose
/// arbitrary authored source content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlDocumentShellConstructionError {
    /// The private construction session violated one of its own invariants.
    Session(HtmlTreeSessionError),
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

impl From<HtmlTreeFreezeError> for HtmlDocumentShellConstructionError {
    fn from(error: HtmlTreeFreezeError) -> Self {
        Self::Freeze(error)
    }
}

/// Why token processing stopped.
enum Stop {
    /// A supported rule stopped document parsing at the given token index.
    Parsing { processed_tokens: usize },
    /// TC-S1 does not prove the reached cell. Nothing was mutated for it.
    Unsupported(HtmlTreeUnsupportedCapability),
}

/// Constructs the TC-S1 disabled-scripting document shell for `source`.
///
/// The caller retains ownership of `source`; this operation borrows it and
/// does not clone the complete source. The returned analysis may outlive the
/// caller's handle through the existing [`SourceAnchor`](crate::SourceAnchor)
/// ownership contract.
///
/// Returns `Err` only for a session or freeze invariant failure. Unsupported
/// input, tokenizer incompleteness, and HTML parse diagnostics are all
/// ordinary `Ok` results carrying their own honest evidence.
pub(crate) fn construct_html_document_shell(
    source: &SourceText,
    limits: HtmlTokenizerLimits,
) -> Result<HtmlDocumentShellAnalysis, HtmlDocumentShellConstructionError> {
    let tokenizer_run = tokenize(source, limits);
    let mut session = HtmlTreeSession::new()?;

    let mut stop = None;
    for (token_index, token) in tokenizer_run.tokens().iter().enumerate() {
        let trigger = token_trigger(token, token_index);
        let admitted = match admit(token) {
            Ok(admitted) => admitted,
            Err(capability) => {
                stop = Some(Stop::Unsupported(HtmlTreeUnsupportedCapability::new(
                    capability, trigger,
                )));
                break;
            }
        };
        match session.process(&admitted, trigger.clone())? {
            TokenOutcome::Consumed => {}
            TokenOutcome::StoppedParsing => {
                stop = Some(Stop::Parsing {
                    processed_tokens: token_index + 1,
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

    let completion = effective_completion(&stop, &tokenizer_run);
    let parts = session.finish(completion);
    Ok(freeze(source, tokenizer_run, parts)?)
}

/// Resolves effective completion from the tree's own stop and the retained
/// run, without ever upgrading lower-layer incompleteness.
fn effective_completion(
    stop: &Option<Stop>,
    tokenizer_run: &super::super::tokenizer::result::HtmlTokenizerRunResult,
) -> HtmlTreeCompletion {
    match stop {
        // A tree stop is reported as the tree's own unsupported evidence. The
        // retained run's completion stays separately authoritative and may
        // additionally be incomplete; either way this result is not Complete,
        // so no lower-layer meaning is upgraded.
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
