//! Crate-private resumable lifecycle around the existing tokenizer Engine.
//!
//! TC-S9 adds one causal coordination seam without creating a second
//! tokenizer. The existing [`Engine`] remains the single owner of source
//! progression, preprocessing, lexical state, token emission, diagnostics,
//! resource accounting, and final run validation. This module only wraps that
//! Engine with a private suspend/resume lifecycle and provides the selected
//! RAWTEXT lexical state handlers.
//!
//! The existing batch `tokenize()` entry point drains this same session under
//! the predecessor no-tree-feedback policy. A context-dependent start tag
//! therefore still materializes the exact established deferred unsupported
//! result when no tree coordination is present.
//!
//! This module is crate-private implementation infrastructure. It is not a
//! public iterator, async stream, parser-control protocol, serialization
//! contract, browser-adapter boundary, or product-level cancellation API.

use crate::SourceText;
use crate::html::token::{HtmlTagKind, HtmlToken};

use super::super::resource::HtmlTokenizerLimits;
use super::super::result::{
    HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability, HtmlTokenizerCompletion,
    HtmlTokenizerIncompleteCause, HtmlTokenizerMode, HtmlTokenizerRunResult,
    HtmlTokenizerUnsupportedCapability, HtmlTokenizerUnsupportedTrigger,
};
use super::builder::TagBuilder;
use super::cursor::InputUnit;
use super::state::State;
use super::{
    DataRun, Engine, Step, context_dependent_mode, invalid_configuration_result,
    source_bytes_limit_result,
};

/// One private production boundary reached while driving the tokenizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerSessionBoundary {
    /// The selected appropriate RAWTEXT end tag committed and the Engine
    /// yielded before consuming any post-close source.
    TokenAvailable,
    /// A context-dependent start tag committed at its exact post-tag boundary;
    /// no later source has been consumed.
    Suspended(HtmlTokenizerMode),
    /// The Engine reached a durable complete or incomplete terminal state.
    Terminal,
}

/// A private coordination invariant failure.
///
/// These are operation-boundary errors, never authored-input diagnostics or
/// unsupported-capability evidence. They carry no arbitrary source content.
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
    /// Configuration/source-size preflight results use the exact established
    /// batch constructors and never instantiate or advance an Engine.
    immediate_result: Option<HtmlTokenizerRunResult>,
    terminal_completion: Option<HtmlTokenizerCompletion>,
    /// The exact predecessor completion produced by the Engine at a
    /// context-dependent boundary. Batch mode publishes it unchanged;
    /// coordinated mode discards it only after successfully applying the
    /// selected tree feedback to the still-live Engine.
    suspended_completion: Option<HtmlTokenizerCompletion>,
    suspended_mode: Option<HtmlTokenizerMode>,
}

impl<'a> HtmlTokenizerSession<'a> {
    pub(crate) fn new(source: &'a SourceText, limits: HtmlTokenizerLimits) -> Self {
        if let Some(failure) = limits.configuration_failure() {
            return Self {
                engine: None,
                immediate_result: Some(invalid_configuration_result(source, limits, failure)),
                terminal_completion: None,
                suspended_completion: None,
                suspended_mode: None,
            };
        }
        let source_len = source.as_str().len();
        if source_len > limits.max_source_bytes() {
            return Self {
                engine: None,
                immediate_result: Some(source_bytes_limit_result(source, limits, source_len)),
                terminal_completion: None,
                suspended_completion: None,
                suspended_mode: None,
            };
        }

        Self {
            engine: Some(Engine::new(source, limits)),
            immediate_result: None,
            terminal_completion: None,
            suspended_completion: None,
            suspended_mode: None,
        }
    }

    /// Drives until the existing Engine reaches one private coordination
    /// boundary.
    ///
    /// `Engine::run()` already stops exactly after a context-dependent start
    /// tag, before later source. The session interprets that existing durable
    /// completion as a private suspension while retaining the completion
    /// unchanged for batch-compatible finalization.
    pub(crate) fn drive_to_boundary(&mut self) -> HtmlTokenizerSessionBoundary {
        if self.immediate_result.is_some() || self.terminal_completion.is_some() {
            return HtmlTokenizerSessionBoundary::Terminal;
        }
        if let Some(mode) = self.suspended_mode {
            return HtmlTokenizerSessionBoundary::Suspended(mode);
        }

        let engine = self.engine.as_mut().expect("live tokenizer engine");
        let completion = engine.run();

        if is_private_raw_text_close_yield(&completion) {
            return HtmlTokenizerSessionBoundary::TokenAvailable;
        }

        if let Some(mode) = context_dependent_suspension(&completion) {
            self.suspended_mode = Some(mode);
            self.suspended_completion = Some(completion);
            HtmlTokenizerSessionBoundary::Suspended(mode)
        } else {
            self.terminal_completion = Some(completion);
            HtmlTokenizerSessionBoundary::Terminal
        }
    }

    /// All tokens committed so far, in retained emission order.
    pub(crate) fn tokens(&self) -> &[HtmlToken] {
        match (&self.immediate_result, &self.engine) {
            (Some(result), _) => result.tokens(),
            (None, Some(engine)) => &engine.tokens,
            (None, None) => &[],
        }
    }

    /// Applies the one TC-S9 tree-directed lexical control.
    ///
    /// Appropriate-end-tag identity is derived from the Engine's own retained
    /// emitted start tag. No expected close spelling, source range, or source
    /// slice is accepted from tree construction.
    pub(crate) fn apply_raw_text(&mut self) -> Result<(), HtmlTokenizerSessionControlError> {
        let Some(mode) = self.suspended_mode else {
            return Err(HtmlTokenizerSessionControlError::RawTextRequestedWithoutSuspension);
        };
        if mode != HtmlTokenizerMode::RawText {
            return Err(HtmlTokenizerSessionControlError::RawTextRequestedForDifferentMode(mode));
        }
        if !self
            .engine
            .as_mut()
            .expect("suspended tokenizer engine")
            .activate_raw_text_from_suspended_start()
        {
            return Err(HtmlTokenizerSessionControlError::RawTextActivationInvariant);
        }
        self.suspended_mode = None;
        self.suspended_completion = None;
        Ok(())
    }

    /// Drains with the established no-tree-feedback policy.
    ///
    /// If the session is suspended, the exact predecessor completion is
    /// published unchanged. A private appropriate-close yield is never a
    /// durable tokenizer result and is simply drained until the next real
    /// boundary. This preserves the existing batch tokenizer contract.
    pub(crate) fn finish_batch_compatible(mut self) -> HtmlTokenizerRunResult {
        if let Some(result) = self.immediate_result.take() {
            return result;
        }
        loop {
            if let Some(completion) = self.terminal_completion.take() {
                return self
                    .engine
                    .take()
                    .expect("terminal tokenizer engine")
                    .into_result(completion);
            }
            if self.suspended_mode.is_some() {
                let completion = self
                    .suspended_completion
                    .take()
                    .expect("suspended mode retains predecessor completion");
                return self
                    .engine
                    .take()
                    .expect("suspended tokenizer engine")
                    .into_result(completion);
            }

            match self.drive_to_boundary() {
                HtmlTokenizerSessionBoundary::TokenAvailable => {}
                HtmlTokenizerSessionBoundary::Suspended(_) => {}
                HtmlTokenizerSessionBoundary::Terminal => {}
            }
        }
    }

    /// Finalizes only a real tokenizer terminal state reached by coordinated
    /// driving.
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
            .expect("terminal tokenizer engine")
            .into_result(completion))
    }
}

/// Recognizes only the existing context-dependent deferred stop. This is
/// classification of retained tokenizer completion evidence, not source/token
/// reinterpretation.
fn context_dependent_suspension(completion: &HtmlTokenizerCompletion) -> Option<HtmlTokenizerMode> {
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
        unsupported,
    )) = completion
    else {
        return None;
    };
    if unsupported.availability() != HtmlTokenizerCapabilityAvailability::Deferred {
        return None;
    }
    match unsupported.capability() {
        HtmlTokenizerCapability::ContextDependentTokenizerMode { mode } => Some(mode),
        _ => None,
    }
}

/// TC-S9 uses the already-existing private `TreeConstructionControlledState`
/// capability only as an internal Engine-yield marker after an appropriate
/// RAWTEXT end tag has emitted. It is consumed here and never frozen into a
/// tokenizer result.
fn is_private_raw_text_close_yield(completion: &HtmlTokenizerCompletion) -> bool {
    let HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
        unsupported,
    )) = completion
    else {
        return false;
    };
    unsupported.capability() == HtmlTokenizerCapability::TreeConstructionControlledState
        && unsupported.availability() == HtmlTokenizerCapabilityAvailability::Deferred
        && matches!(
            unsupported.trigger(),
            HtmlTokenizerUnsupportedTrigger::EmittedToken { .. }
        )
}

impl<'a> Engine<'a> {
    /// Activates RAWTEXT only from the exact existing post-start-tag suspended
    /// boundary. The retained emitted start tag remains the tokenizer-owned
    /// appropriate-end-tag authority for this RAWTEXT episode.
    pub(super) fn activate_raw_text_from_suspended_start(&mut self) -> bool {
        let Some(token_index) = self.tokens.len().checked_sub(1) else {
            return false;
        };
        let Some(HtmlToken::Tag(tag)) = self.tokens.get(token_index) else {
            return false;
        };
        if tag.kind() != HtmlTagKind::Start
            || context_dependent_mode(tag.name().interpreted()) != Some(HtmlTokenizerMode::RawText)
            || self.state != State::Data
            || self.tag.is_some()
            || self.pending_reconsume
        {
            return false;
        }
        self.raw_text_start_tag_index = Some(token_index);
        self.raw_text_closing_tag = false;
        self.state = State::RawText;
        true
    }

    pub(super) fn step_raw_text(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Eof { at } => {
                if let Err(stop) = self.flush_data_run() {
                    return stop;
                }
                self.emit_eof(at)
            }
            InputUnit::Scalar {
                ch: '<',
                start,
                end,
            } => {
                self.tag_open_start = start;
                self.tag_open_delimiter_end = end;
                self.state = State::RawTextLessThanSign;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                // The accepted 8-path theorem deliberately does not widen
                // tokenizer diagnostic vocabulary with a RAWTEXT-specific
                // NULL context. Stop honestly at an existing controlled-state
                // capability rather than mislabeling the diagnostic as Data
                // or silently omitting the required parse error.
                self.raw_text_unproved_input_stop((start, end))
            }
            InputUnit::Scalar { ch, start, end } => {
                if let Err(stop) = self.push_data_char(ch, start, end) {
                    return stop;
                }
                Step::Continue
            }
        }
    }

    pub(super) fn step_raw_text_less_than_sign(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch: '/', end, .. } => {
                self.tag_open_delimiter_end = end;
                self.state = State::RawTextEndTagOpen;
                Step::Continue
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                let less_than_end = self.tag_open_start + '<'.len_utf8();
                if let Err(stop) = self.push_data_char('<', self.tag_open_start, less_than_end) {
                    return stop;
                }
                self.state = State::RawText;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    pub(super) fn step_raw_text_end_tag_open(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if ch.is_ascii_alphabetic() => {
                self.tag = Some(TagBuilder::new(
                    HtmlTagKind::End,
                    self.tag_open_start,
                    (self.tag_open_start, self.tag_open_delimiter_end),
                    unit.start(),
                    self.diagnostics.len(),
                ));
                self.state = State::RawTextEndTagName;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                if let Err(stop) = self.push_raw_text_opening_literal() {
                    return stop;
                }
                self.state = State::RawText;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    pub(super) fn step_raw_text_end_tag_name(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, start, end } if ch.is_ascii_alphabetic() => {
                let normalized = ch.to_ascii_lowercase();
                if let Err(stop) = self.try_reserve_retained(normalized.len_utf8(), start) {
                    return stop;
                }
                self.tag
                    .as_mut()
                    .expect("RAWTEXT end-tag candidate active")
                    .push_name(normalized, end);
                Step::Continue
            }
            InputUnit::Scalar { ch, start, end }
                if self.raw_text_candidate_is_appropriate()
                    && matches!(ch, '\t' | '\n' | '\u{000c}' | ' ' | '/' | '>') =>
            {
                if let Err(stop) = self.flush_data_run() {
                    return stop;
                }
                self.raw_text_closing_tag = true;
                match ch {
                    '\t' | '\n' | '\u{000c}' | ' ' => {
                        self.state = State::BeforeAttributeName;
                        Step::Continue
                    }
                    '/' => {
                        self.pending_solidus = Some((start, end));
                        self.state = State::SelfClosingStartTag;
                        Step::Continue
                    }
                    '>' => self.finish_tag((start, end)),
                    _ => unreachable!("guarded RAWTEXT delimiter"),
                }
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                if let Err(stop) = self.fallback_raw_text_end_tag_candidate() {
                    return stop;
                }
                self.state = State::RawText;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    /// Called by the parent Engine's ordinary `finish_tag()` after the
    /// selected appropriate RAWTEXT end-tag token has committed.
    pub(super) fn raw_text_close_yield(&mut self, boundary_at: usize) -> Step {
        self.raw_text_start_tag_index = None;
        self.raw_text_closing_tag = false;
        let boundary = self.anchor(boundary_at, boundary_at);
        let unsupported = HtmlTokenizerUnsupportedCapability::new(
            self.source,
            HtmlTokenizerCapability::TreeConstructionControlledState,
            HtmlTokenizerCapabilityAvailability::Deferred,
            HtmlTokenizerUnsupportedTrigger::EmittedToken {
                token_index: self.tokens.len() - 1,
                boundary,
            },
        )
        .expect("valid private RAWTEXT close yield");
        Step::Stop(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        ))
    }

    fn raw_text_candidate_is_appropriate(&self) -> bool {
        let Some(start_index) = self.raw_text_start_tag_index else {
            return false;
        };
        let Some(HtmlToken::Tag(start)) = self.tokens.get(start_index) else {
            return false;
        };
        let Some(candidate) = self.tag.as_ref() else {
            return false;
        };
        start.kind() == HtmlTagKind::Start
            && candidate.kind == HtmlTagKind::End
            && candidate.interpreted_name == start.name().interpreted()
    }

    /// Emits literal `</` fallback without rescanning. The delimiter is known
    /// ASCII source consumed by the single forward cursor.
    fn push_raw_text_opening_literal(&mut self) -> Result<(), Step> {
        let less_than_end = self.tag_open_start + '<'.len_utf8();
        self.push_data_char('<', self.tag_open_start, less_than_end)?;
        self.push_data_char('/', less_than_end, self.tag_open_delimiter_end)?;
        Ok(())
    }

    /// Reclassifies the current RAWTEXT end-tag candidate as character data
    /// using only exact offsets and temporary state captured while the
    /// tokenizer itself examined that candidate. This is not downstream
    /// source search/rescan/re-tokenization.
    fn fallback_raw_text_end_tag_candidate(&mut self) -> Result<(), Step> {
        let tag = self.tag.as_ref().expect("RAWTEXT end-tag candidate active");
        let start = tag.tag_start;
        let end = tag.name_end;

        // The active builder already accounts for the ASCII name bytes.
        // Fallback replaces it with the exact literal source span, adding
        // only the two `</` delimiter bytes. Preflight that delta before
        // destroying the candidate so a resource refusal preserves state.
        self.try_reserve_retained(2, start)?;
        let raw = self.anchor(start, end).fragment().to_owned();
        self.tag.take().expect("RAWTEXT end-tag candidate active");

        match &mut self.data_run {
            Some(run) => {
                run.interpreted.push_str(&raw);
                run.end = end;
            }
            None => {
                self.data_run = Some(DataRun {
                    start,
                    end,
                    interpreted: raw,
                });
            }
        }
        Ok(())
    }

    fn raw_text_unproved_input_stop(&mut self, trigger: (usize, usize)) -> Step {
        if let Err(stop) = self.flush_data_run() {
            return stop;
        }
        let anchor = self.anchor(trigger.0, trigger.1);
        let unsupported = HtmlTokenizerUnsupportedCapability::new(
            self.source,
            HtmlTokenizerCapability::TreeConstructionControlledState,
            HtmlTokenizerCapabilityAvailability::Unsupported,
            HtmlTokenizerUnsupportedTrigger::Input(anchor),
        )
        .expect("valid selected RAWTEXT unsupported evidence");
        Step::Stop(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        ))
    }
}
