//! Crate-private resumable lifecycle around the existing tokenizer Engine.
//!
//! TC-S9 adds one causal coordination seam without creating a second
//! tokenizer. The existing [`Engine`] remains the single owner of source
//! progression, preprocessing, lexical state, token emission, diagnostics,
//! resource accounting, and final run validation. This module only wraps that
//! Engine with a private suspend/resume lifecycle and provides the selected
//! RAWTEXT lexical state handlers, plus TC-S10's selected RCDATA and
//! Character Reference handlers.
//!
//! TC-S10 adds a second, deliberately separate control ([`
//! HtmlTokenizerSession::apply_rcdata`]) rather than one mode-parameterized
//! entry point: the selected capability is a bounded Title discharge, and a
//! generic tokenizer-mode control surface is exactly what the accepted
//! placement refuses.
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
use crate::html::token::{HtmlCharacterToken, HtmlTagKind, HtmlToken};

use super::super::diagnostic::{
    HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticContext, HtmlTokenizerDiagnosticHandling,
    HtmlTokenizerDiagnosticSubject,
};
use super::super::resource::HtmlTokenizerLimits;
use super::super::result::{
    HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability, HtmlTokenizerCompletion,
    HtmlTokenizerIncompleteCause, HtmlTokenizerMode, HtmlTokenizerRunResult,
    HtmlTokenizerUnsupportedCapability, HtmlTokenizerUnsupportedTrigger,
};
use super::builder::TagBuilder;
use super::cursor::InputUnit;
use super::named_character_reference::{self, NamedMatch};
use super::state::State;
use super::{
    DataRun, Engine, Step, context_dependent_mode, invalid_configuration_result,
    source_bytes_limit_result, source_evidence_stop,
};

/// One private production boundary reached while driving the tokenizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerSessionBoundary {
    /// The selected appropriate RAWTEXT or RCDATA end tag committed and the
    /// Engine yielded before consuming any post-close source.
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
    RcdataRequestedWithoutSuspension,
    RcdataRequestedForDifferentMode(HtmlTokenizerMode),
    RcdataActivationInvariant,
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

        if is_private_text_mode_close_yield(&completion) {
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

    /// Applies the one TC-S10 tree-directed lexical control.
    ///
    /// Deliberately a separate operation from [`Self::apply_raw_text`], with
    /// no tokenizer-mode operand: the tree asks for the selected Title RCDATA
    /// entry it owns the semantics of, and the coordinator maps that request
    /// onto exactly this control. Appropriate-end-tag identity again comes
    /// from the Engine's own retained emitted start tag; no expected close
    /// spelling, source range, or source slice is accepted from tree
    /// construction.
    pub(crate) fn apply_rcdata(&mut self) -> Result<(), HtmlTokenizerSessionControlError> {
        let Some(mode) = self.suspended_mode else {
            return Err(HtmlTokenizerSessionControlError::RcdataRequestedWithoutSuspension);
        };
        if mode != HtmlTokenizerMode::Rcdata {
            return Err(HtmlTokenizerSessionControlError::RcdataRequestedForDifferentMode(mode));
        }
        if !self
            .engine
            .as_mut()
            .expect("suspended tokenizer engine")
            .activate_rcdata_from_suspended_start()
        {
            return Err(HtmlTokenizerSessionControlError::RcdataActivationInvariant);
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

/// TC-S9 and TC-S10 use the already-existing private
/// `TreeConstructionControlledState` capability only as an internal
/// Engine-yield marker after an appropriate RAWTEXT or RCDATA end tag has
/// emitted. It is consumed here and never frozen into a tokenizer result.
fn is_private_text_mode_close_yield(completion: &HtmlTokenizerCompletion) -> bool {
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
                if let Err(stop) = self.try_reserve_retained(ch.len_utf8(), start) {
                    return stop;
                }
                self.tag
                    .as_mut()
                    .expect("RAWTEXT end-tag candidate active")
                    .push_name(ch, end);
                Step::Continue
            }
            InputUnit::Scalar { ch, start, end }
                if self.raw_text_candidate_is_appropriate()
                    && matches!(ch, '\t' | '\n' | '\u{000c}' | ' ' | '/' | '>') =>
            {
                if let Err(stop) = self.flush_data_run() {
                    return stop;
                }
                self.tag
                    .as_mut()
                    .expect("appropriate RAWTEXT end-tag candidate active")
                    .interpreted_name
                    .make_ascii_lowercase();
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
    /// selected appropriate RAWTEXT or RCDATA end-tag token has committed.
    pub(super) fn text_mode_close_yield(&mut self, boundary_at: usize) -> Step {
        self.raw_text_start_tag_index = None;
        self.raw_text_closing_tag = false;
        self.rcdata_start_tag_index = None;
        self.rcdata_closing_tag = false;
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
        .expect("valid private text-mode close yield");
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
            && candidate
                .interpreted_name
                .eq_ignore_ascii_case(start.name().interpreted())
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
    /// using only exact offsets and the ASCII spelling retained while the
    /// single forward cursor examined the candidate. No source is searched,
    /// sliced, rescanned, or retokenized.
    fn fallback_raw_text_end_tag_candidate(&mut self) -> Result<(), Step> {
        let tag = self.tag.as_ref().expect("RAWTEXT end-tag candidate active");
        let start = tag.tag_start;
        let end = tag.name_end;

        // The active builder already accounts for the ASCII name bytes.
        // Fallback replaces it with the retained literal spelling and adds
        // only the two `</` delimiter bytes. Preflight that delta before
        // destroying the candidate so a resource refusal preserves state.
        self.try_reserve_retained(2, start)?;
        let tag = self.tag.take().expect("RAWTEXT end-tag candidate active");
        let mut raw = String::with_capacity(2 + tag.interpreted_name.len());
        raw.push_str("</");
        raw.push_str(&tag.interpreted_name);

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

    /// The one element whose RCDATA lifecycle TC-S10 selects.
    ///
    /// `HtmlTokenizerMode::Rcdata` is a durable *mode* vocabulary shared with
    /// `textarea`, so mode classification alone is not the selected boundary.
    const SELECTED_RCDATA_ELEMENT: &'static str = "title";

    /// Activates the selected RCDATA episode only from the exact existing
    /// post-start-tag suspended boundary. The retained emitted start tag
    /// remains the tokenizer-owned appropriate-end-tag authority for this
    /// episode.
    ///
    /// The tokenizer checks the selected element itself rather than trusting
    /// the caller's request. The tree's `EnterRcdataForTitle` boundary already
    /// refuses every other element, but a durable RCDATA mode is shared with
    /// `textarea`, so a suspended `<textarea>` must be rejected here too. Both
    /// halves of that dual boundary are independently enforced, and neither is
    /// a generic mode switch.
    pub(super) fn activate_rcdata_from_suspended_start(&mut self) -> bool {
        let Some(token_index) = self.tokens.len().checked_sub(1) else {
            return false;
        };
        let Some(HtmlToken::Tag(tag)) = self.tokens.get(token_index) else {
            return false;
        };
        if tag.kind() != HtmlTagKind::Start
            || tag.name().interpreted() != Self::SELECTED_RCDATA_ELEMENT
            || context_dependent_mode(tag.name().interpreted()) != Some(HtmlTokenizerMode::Rcdata)
            || self.state != State::Data
            || self.tag.is_some()
            || self.pending_reconsume
        {
            return false;
        }
        self.rcdata_start_tag_index = Some(token_index);
        self.rcdata_closing_tag = false;
        self.state = State::Rcdata;
        true
    }

    pub(super) fn step_rcdata(&mut self, unit: InputUnit) -> Step {
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
                self.state = State::RcdataLessThanSign;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '&',
                start,
                end,
            } => {
                // Each authored contribution keeps its own exact source
                // evidence: flushing here is what lets one coalesced final
                // text node still expose `a`, `&amp;`, `b` as three ordered
                // contributions instead of one fabricated span.
                if let Err(stop) = self.flush_data_run() {
                    return stop;
                }
                self.character_reference_ampersand = (start, end);
                self.state = State::CharacterReference;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                // RCDATA NUL recovery is deliberately outside TC-S10. Prior
                // valid evidence is preserved by the flush inside the stop;
                // no U+FFFD is produced and no recovery diagnostic is
                // claimed.
                self.rcdata_null_unsupported_stop((start, end))
            }
            InputUnit::Scalar { ch, start, end } => {
                if let Err(stop) = self.push_data_char(ch, start, end) {
                    return stop;
                }
                Step::Continue
            }
        }
    }

    pub(super) fn step_rcdata_less_than_sign(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch: '/', end, .. } => {
                self.tag_open_delimiter_end = end;
                self.state = State::RcdataEndTagOpen;
                Step::Continue
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                let less_than_end = self.tag_open_start + '<'.len_utf8();
                if let Err(stop) = self.push_data_char('<', self.tag_open_start, less_than_end) {
                    return stop;
                }
                self.state = State::Rcdata;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    pub(super) fn step_rcdata_end_tag_open(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if ch.is_ascii_alphabetic() => {
                self.tag = Some(TagBuilder::new(
                    HtmlTagKind::End,
                    self.tag_open_start,
                    (self.tag_open_start, self.tag_open_delimiter_end),
                    unit.start(),
                    self.diagnostics.len(),
                ));
                self.state = State::RcdataEndTagName;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                if let Err(stop) = self.push_raw_text_opening_literal() {
                    return stop;
                }
                self.state = State::Rcdata;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    pub(super) fn step_rcdata_end_tag_name(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, start, end } if ch.is_ascii_alphabetic() => {
                if let Err(stop) = self.try_reserve_retained(ch.len_utf8(), start) {
                    return stop;
                }
                self.tag
                    .as_mut()
                    .expect("RCDATA end-tag candidate active")
                    .push_name(ch, end);
                Step::Continue
            }
            InputUnit::Scalar { ch, start, end }
                if self.rcdata_candidate_is_appropriate()
                    && matches!(ch, '\t' | '\n' | '\u{000c}' | ' ' | '/' | '>') =>
            {
                if let Err(stop) = self.flush_data_run() {
                    return stop;
                }
                self.tag
                    .as_mut()
                    .expect("appropriate RCDATA end-tag candidate active")
                    .interpreted_name
                    .make_ascii_lowercase();
                self.rcdata_closing_tag = true;
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
                    _ => unreachable!("guarded RCDATA delimiter"),
                }
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                if let Err(stop) = self.fallback_raw_text_end_tag_candidate() {
                    return stop;
                }
                self.state = State::Rcdata;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    fn rcdata_candidate_is_appropriate(&self) -> bool {
        let Some(start_index) = self.rcdata_start_tag_index else {
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
            && candidate
                .interpreted_name
                .eq_ignore_ascii_case(start.name().interpreted())
    }

    // ---- Character Reference ---------------------------------------------

    pub(super) fn step_character_reference(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if ch.is_ascii_alphanumeric() => {
                self.state = State::NamedCharacterReference;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '#',
                start,
                end,
            } => {
                // Character Reference entry already succeeded and the
                // authored `&` is committed coverage; the `#` alone is the
                // narrow Numeric-branch trigger.
                self.selected_rcdata_unsupported_stop(
                    HtmlTokenizerCapability::NumericCharacterReferenceInRcdata,
                    (start, end),
                )
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                let (start, end) = self.character_reference_ampersand;
                if let Err(stop) = self.push_data_char('&', start, end) {
                    return stop;
                }
                self.state = State::Rcdata;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    /// One dispatch, one transition: the whole maximum-match discovery is a
    /// single attempted specification-state transition, and the authoritative
    /// consumption that commits an already-selected match never becomes one
    /// transition per consumed byte.
    pub(super) fn step_named_character_reference(&mut self, unit: InputUnit) -> Step {
        let first = match unit {
            InputUnit::Scalar { ch, .. } if ch.is_ascii() => ch as u8,
            // Only an ASCII alphanumeric dispatches here. Anything else can
            // match no identifier at all, which is exactly the ambiguous
            // path — no guess, no fabricated match.
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                self.state = State::AmbiguousAmpersand;
                self.pending_reconsume = true;
                return Step::Continue;
            }
        };
        // The already materialized first identifier unit plus a bounded
        // borrowed view of raw source the cursor has not consumed. Nothing
        // here advances, preprocesses, retains, or diagnoses.
        let window = named_character_reference::LOOKAHEAD_BYTES;
        let rest = self.cursor.unconsumed_bytes(window);
        let Some(found) = named_character_reference::maximum_match(first, rest) else {
            let (start, end) = self.character_reference_ampersand;
            if let Err(stop) = self.push_data_char('&', start, end) {
                return stop;
            }
            self.state = State::AmbiguousAmpersand;
            self.pending_reconsume = true;
            return Step::Continue;
        };
        self.commit_named_character_reference(found, unit.start())
    }

    /// Commits one resolved Named Character Reference as a single
    /// resource-atomic semantic effect.
    ///
    /// Every fallible resource decision happens *before* the remaining
    /// matched source is committed, in the accepted deterministic order
    /// `RetainedInterpretedBytes -> EmittedTokens -> Diagnostics`, and the
    /// diagnostics check is made only when the semantic result actually
    /// requires a diagnostic. A refusal therefore commits no matched source,
    /// no resolved token and no missing-semicolon diagnostic, and leaves
    /// prior valid evidence and the already-committed coverage boundary
    /// exactly as they were. Once consumption succeeds, no fallible step
    /// remains.
    fn commit_named_character_reference(&mut self, found: NamedMatch, first_at: usize) -> Step {
        let (ampersand_start, _) = self.character_reference_ampersand;
        let value_bytes = found.value.len();
        // Identifier bytes are ASCII by construction, so the match end is
        // exact arithmetic over consumed units, not a reconstructed endpoint.
        let match_end = first_at + found.name_len;
        let boundary = (self.processed_end, self.processed_end);

        // Phase 1 — every fallible resource decision, in the accepted order.
        if let Err(stop) = self.try_reserve_retained(value_bytes, ampersand_start) {
            return stop;
        }
        let committed = match self.preflight_token_emission(value_bytes, boundary) {
            Ok(committed) => committed,
            Err(stop) => return stop,
        };
        if !found.ends_with_semicolon
            && let Err(stop) = self.preflight_pending_emission_diagnostics(1, boundary)
        {
            return stop;
        }

        // Phase 2 — every fallible *evidence construction*, still before any
        // authoritative source is committed. Decoded scalars are output only:
        // they are never fed back as tokenizer input, so a decoded `<` or
        // `</title>` stays interpreted text and recursive decoding cannot
        // occur.
        let Ok(anchor) = self.source.anchor(ampersand_start, match_end) else {
            return source_evidence_stop();
        };
        let Ok(character) = HtmlCharacterToken::new(anchor, found.value.to_owned()) else {
            return source_evidence_stop();
        };
        // The resolved token's index is already determined: nothing else can
        // emit between here and its commit below, so the diagnostic can name
        // it before either exists.
        let resolved_token_index = self.tokens.len();
        // Anchored to the last authored ASCII scalar of the *matched*
        // identifier — never to the later source the match deliberately did
        // not consume.
        let missing_semicolon_at = (match_end.saturating_sub(1), match_end);
        let missing_semicolon = if found.ends_with_semicolon {
            None
        } else {
            let Some(diagnostic) = self.prepare_diagnostic(
                HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference,
                missing_semicolon_at,
                HtmlTokenizerDiagnosticContext::NamedCharacterReference,
                HtmlTokenizerDiagnosticHandling::Continued,
                HtmlTokenizerDiagnosticSubject::EmittedToken {
                    token_index: resolved_token_index,
                },
            ) else {
                return source_evidence_stop();
            };
            Some(diagnostic)
        };

        // Phase 3 — authoritative consumption of the already-selected match.
        // The first identifier unit is the current unit; every remaining one
        // advances through the ordinary single-forward-owner cursor lifecycle,
        // one unit at a time. There is no multi-unit rollback, no endpoint
        // jump, no source search, and no reparse.
        for _ in 1..found.name_len {
            let (consumed, diagnostic) = self.cursor.advance();
            debug_assert!(
                diagnostic.is_none(),
                "identifier bytes are ASCII alphanumerics or ';' and carry no \
                 preprocessing observation"
            );
            self.current = consumed;
        }
        self.processed_end = self.processed_end.max(match_end);

        // Phase 4 — commit already-built evidence. Nothing here can fail.
        if let Some(diagnostic) = missing_semicolon {
            self.commit_prepared_diagnostic(diagnostic, missing_semicolon_at);
        }
        self.commit_token(HtmlToken::Character(character), committed);
        debug_assert_eq!(self.tokens.len() - 1, resolved_token_index);
        self.state = State::Rcdata;
        Step::Continue
    }

    pub(super) fn step_ambiguous_ampersand(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, start, end } if ch.is_ascii_alphanumeric() => {
                if let Err(stop) = self.push_data_char(ch, start, end) {
                    return stop;
                }
                Step::Continue
            }
            InputUnit::Scalar {
                ch: ';',
                start,
                end,
            } => {
                // The authored `;` observation is what makes the reference
                // unknown, and that observation is complete on its own: this
                // diagnostic is observation-conditioned, so it commits first
                // and stays valid even if the later unresolved-run emission is
                // refused. Only after a successful flush — which keeps the
                // unresolved contribution's own exact boundary — does the same
                // `;` reconsume as ordinary RCDATA text. Tree-level coalescing
                // may merge the final text; the two contributions stay
                // separately inspectable.
                let boundary = (self.processed_end, self.processed_end);
                if let Err(stop) = self.preflight_pending_emission_diagnostics(1, boundary) {
                    return stop;
                }
                let Some(diagnostic) = self.prepare_diagnostic(
                    HtmlTokenizerDiagnosticCode::UnknownNamedCharacterReference,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::AmbiguousAmpersand,
                    HtmlTokenizerDiagnosticHandling::Continued,
                    HtmlTokenizerDiagnosticSubject::InputLocation,
                ) else {
                    return source_evidence_stop();
                };
                self.commit_prepared_diagnostic(diagnostic, (start, end));
                if let Err(stop) = self.flush_data_run() {
                    return stop;
                }
                self.state = State::Rcdata;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                self.state = State::Rcdata;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    fn rcdata_null_unsupported_stop(&mut self, natural: (usize, usize)) -> Step {
        // Prior valid evidence is preserved before the refusal is published.
        if let Err(stop) = self.flush_data_run() {
            return stop;
        }
        self.selected_rcdata_unsupported_stop(HtmlTokenizerCapability::RcdataNullRecovery, natural)
    }

    /// Publishes one of the two narrow boundaries TC-S10 deliberately does not
    /// select, at the exact authored trigger.
    ///
    /// Both are `Unsupported`, not `Deferred`: this bounded machine refuses the
    /// branch outright rather than promising a later resumption of the same
    /// coordinated run. That is a different claim from the standalone
    /// `ContextDependentTokenizerMode` boundaries, which remain `Deferred`
    /// because tree feedback really can discharge them.
    fn selected_rcdata_unsupported_stop(
        &mut self,
        capability: HtmlTokenizerCapability,
        natural: (usize, usize),
    ) -> Step {
        let trigger = self.discovery_trigger(natural);
        let anchor = self.anchor(trigger.0, trigger.1);
        let unsupported = HtmlTokenizerUnsupportedCapability::new(
            self.source,
            capability,
            HtmlTokenizerCapabilityAvailability::Unsupported,
            HtmlTokenizerUnsupportedTrigger::Input(anchor),
        )
        .expect("valid selected RCDATA unsupported evidence");
        Step::Stop(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        ))
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
