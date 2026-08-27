//! Crate-private resumable lifecycle around the existing tokenizer Engine.
//!
//! TC-S9 needs one causal coordination seam: after a context-dependent start
//! tag has emitted, tree construction must be able to apply the selected
//! tokenizer control before later source production. The lexical Engine stays
//! the sole source/cursor/state owner; this wrapper only owns suspension,
//! resumption, terminalization, and the batch-compatibility policy.
//!
//! This is not a public iterator, stream, parser-control protocol, or
//! serialization boundary. The existing batch `tokenize()` entry point drains
//! this same session under the predecessor policy and therefore preserves the
//! established deferred-unsupported behavior when no tree coordination exists.

use crate::SourceText;
use crate::html::token::HtmlToken;

use super::super::resource::HtmlTokenizerLimits;
use super::super::result::{HtmlTokenizerCompletion, HtmlTokenizerMode, HtmlTokenizerRunResult};
use super::{Engine, EngineBoundary, invalid_configuration_result, source_bytes_limit_result};

/// One private production boundary reached while driving the tokenizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerSessionBoundary {
    /// At least one new token committed and the Engine yielded before
    /// consuming more source. No tree-directed control is owed by this
    /// boundary itself.
    TokenAvailable,
    /// A context-dependent start tag committed at its exact post-tag boundary.
    /// Later source has not been consumed yet.
    Suspended(HtmlTokenizerMode),
    /// The Engine reached a final complete or incomplete tokenizer result.
    Terminal,
}

/// A private coordination invariant failure.
///
/// These are operation-boundary errors, never authored-input diagnostics and
/// never unsupported-capability evidence. They carry no source contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerSessionControlError {
    RawTextRequestedWithoutSuspension,
    RawTextRequestedForDifferentMode(HtmlTokenizerMode),
    RawTextActivationInvariant,
    ResultRequestedBeforeTerminal,
    ResultRequestedWithOutstandingSuspension(HtmlTokenizerMode),
}

impl std::fmt::Display for HtmlTokenizerSessionControlError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "HTML tokenizer session coordination invariant violation: {self:?}"
        )
    }
}

impl std::error::Error for HtmlTokenizerSessionControlError {}

/// One resumable tokenizer run over the existing lexical Engine.
pub(crate) struct HtmlTokenizerSession<'a> {
    engine: Option<Engine<'a>>,
    immediate_result: Option<HtmlTokenizerRunResult>,
    terminal_completion: Option<HtmlTokenizerCompletion>,
    suspended_mode: Option<HtmlTokenizerMode>,
}

impl<'a> HtmlTokenizerSession<'a> {
    /// Creates a session without consuming source beyond the existing
    /// configuration/source-size preflight.
    pub(crate) fn new(source: &'a SourceText, limits: HtmlTokenizerLimits) -> Self {
        if let Some(failure) = limits.configuration_failure() {
            return Self {
                engine: None,
                immediate_result: Some(invalid_configuration_result(source, limits, failure)),
                terminal_completion: None,
                suspended_mode: None,
            };
        }

        let source_len = source.as_str().len();
        if source_len > limits.max_source_bytes() {
            return Self {
                engine: None,
                immediate_result: Some(source_bytes_limit_result(source, limits, source_len)),
                terminal_completion: None,
                suspended_mode: None,
            };
        }

        Self {
            engine: Some(Engine::new(source, limits)),
            immediate_result: None,
            terminal_completion: None,
            suspended_mode: None,
        }
    }

    /// Drives until exactly one private pull boundary is reached.
    ///
    /// If a context-dependent mode is already suspended, calling this again
    /// does not consume source; the same suspension is returned until the
    /// coordinator either applies the selected control or terminalizes the run
    /// with the batch-compatible policy.
    pub(crate) fn drive_to_boundary(&mut self) -> HtmlTokenizerSessionBoundary {
        if self.immediate_result.is_some() || self.terminal_completion.is_some() {
            return HtmlTokenizerSessionBoundary::Terminal;
        }
        if let Some(mode) = self.suspended_mode {
            return HtmlTokenizerSessionBoundary::Suspended(mode);
        }

        let engine = self.engine.as_mut().expect("live tokenizer engine");
        match engine.run_until_boundary() {
            EngineBoundary::TokenAvailable => HtmlTokenizerSessionBoundary::TokenAvailable,
            EngineBoundary::Suspended(mode) => {
                self.suspended_mode = Some(mode);
                HtmlTokenizerSessionBoundary::Suspended(mode)
            }
            EngineBoundary::Terminal(completion) => {
                self.terminal_completion = Some(completion);
                HtmlTokenizerSessionBoundary::Terminal
            }
        }
    }

    /// All tokens committed so far, in retained emission order.
    pub(crate) fn tokens(&self) -> &[HtmlToken] {
        if let Some(result) = &self.immediate_result {
            result.tokens()
        } else {
            self.engine.as_ref().expect("live tokenizer engine").tokens()
        }
    }

    /// Applies the one TC-S9 tree-directed lexical control.
    ///
    /// The Engine itself derives appropriate-end-tag identity from its own
    /// emitted start-tag history. No expected closing spelling/range/source
    /// slice enters through this API.
    pub(crate) fn apply_raw_text(&mut self) -> Result<(), HtmlTokenizerSessionControlError> {
        let Some(mode) = self.suspended_mode else {
            return Err(HtmlTokenizerSessionControlError::RawTextRequestedWithoutSuspension);
        };
        if mode != HtmlTokenizerMode::RawText {
            return Err(
                HtmlTokenizerSessionControlError::RawTextRequestedForDifferentMode(mode),
            );
        }
        let activated = self
            .engine
            .as_mut()
            .expect("suspended session has live engine")
            .activate_raw_text_from_suspended_start();
        if !activated {
            return Err(HtmlTokenizerSessionControlError::RawTextActivationInvariant);
        }
        self.suspended_mode = None;
        Ok(())
    }

    /// Consumes the session using the established no-tree-feedback policy.
    ///
    /// A pending or later context-dependent boundary becomes the exact same
    /// deferred unsupported capability the batch tokenizer produced before
    /// TC-S9. Ordinary token yields are drained immediately. This is used by
    /// both `tokenize()` and coordinator finalization after a tree-side stop.
    pub(crate) fn finish_batch_compatible(mut self) -> HtmlTokenizerRunResult {
        if let Some(result) = self.immediate_result.take() {
            return result;
        }

        loop {
            if let Some(completion) = self.terminal_completion.take() {
                return self
                    .engine
                    .take()
                    .expect("terminal session has engine")
                    .into_result(completion);
            }

            if let Some(mode) = self.suspended_mode.take() {
                let completion = self
                    .engine
                    .as_mut()
                    .expect("suspended session has engine")
                    .batch_completion_for_suspended_context(mode);
                return self
                    .engine
                    .take()
                    .expect("batch-terminal session has engine")
                    .into_result(completion);
            }

            match self.drive_to_boundary() {
                HtmlTokenizerSessionBoundary::TokenAvailable => {}
                HtmlTokenizerSessionBoundary::Suspended(_) => {
                    // The next iteration materializes the predecessor deferred
                    // unsupported boundary without consuming later source.
                }
                HtmlTokenizerSessionBoundary::Terminal => {}
            }
        }
    }

    /// Finalizes a session that the coordinated driver has already driven to a
    /// real terminal tokenizer completion.
    pub(crate) fn into_result(
        mut self,
    ) -> Result<HtmlTokenizerRunResult, HtmlTokenizerSessionControlError> {
        if let Some(mode) = self.suspended_mode {
            return Err(
                HtmlTokenizerSessionControlError::ResultRequestedWithOutstandingSuspension(mode),
            );
        }
        if let Some(result) = self.immediate_result.take() {
            return Ok(result);
        }
        let Some(completion) = self.terminal_completion.take() else {
            return Err(HtmlTokenizerSessionControlError::ResultRequestedBeforeTerminal);
        };
        Ok(self
            .engine
            .take()
            .expect("terminal session has engine")
            .into_result(completion))
    }
}
