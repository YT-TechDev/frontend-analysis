//! Crate-private resumable lifecycle around the existing tokenizer Engine.
//!
//! TC-S9 adds one causal coordination seam without creating a second
//! tokenizer. The existing [`Engine`] remains the single owner of source
//! progression, preprocessing, lexical state, token emission, diagnostics,
//! resource accounting, and final run validation. This module only wraps that
//! Engine with a private suspend/resume lifecycle and provides the selected
//! RAWTEXT (TC-S9) and RCDATA + Named Character Reference (TC-S10) lexical
//! state handlers.
//!
//! The existing batch `tokenize()` entry point drains this same session under
//! the predecessor no-tree-feedback policy. A context-dependent start tag
//! therefore still materializes the exact established deferred unsupported
//! result when no tree coordination is present.
//!
//! The two selected episodes are separately owned. RAWTEXT and RCDATA each
//! retain their own start tag, their own appropriate-close marker, and their
//! own activation control, so neither lifecycle can be opened, closed, or
//! validated under the other's theorem. Only helpers that carry no episode
//! identity — the `</` literal push, the end-tag-candidate fallback, and the
//! private close-yield marker — are shared.
//!
//! This module is crate-private implementation infrastructure. It is not a
//! public iterator, async stream, parser-control protocol, serialization
//! contract, browser-adapter boundary, or product-level cancellation API.

use crate::SourceText;
use crate::html::token::{HtmlCharacterToken, HtmlTagKind, HtmlToken};

use super::super::diagnostic::{
    HtmlTokenizerDiagnostic, HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticContext,
    HtmlTokenizerDiagnosticHandling, HtmlTokenizerDiagnosticSubject,
};
use super::super::resource::{HtmlTokenizerInvariantFailure, HtmlTokenizerLimits};
use super::super::result::{
    HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability, HtmlTokenizerCompletion,
    HtmlTokenizerIncompleteCause, HtmlTokenizerMode, HtmlTokenizerRunResult,
    HtmlTokenizerUnsupportedCapability, HtmlTokenizerUnsupportedTrigger,
};
use super::builder::TagBuilder;
use super::cursor::InputUnit;
use super::named_character_reference;
use super::state::State;
use super::{
    DataRun, Engine, Step, context_dependent_mode, internal_invariant_stop,
    invalid_configuration_result, source_bytes_limit_result,
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
    TitleRcdataRequestedWithoutSuspension,
    TitleRcdataRequestedForDifferentMode(HtmlTokenizerMode),
    TitleRcdataActivationInvariant,
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
    /// Deliberately Title-specific rather than a generic
    /// `switch_tokenizer(mode)`: the durable `Rcdata` mode is shared with
    /// `textarea`, so mode classification alone is not the selected boundary.
    /// The tokenizer therefore refuses activation independently of the tree
    /// unless its own retained suspended start tag is exactly interpreted
    /// `title`. Appropriate-end-tag identity likewise stays derived from that
    /// retained start tag; no close spelling, range, or source slice is
    /// accepted from tree construction.
    pub(crate) fn apply_title_rcdata(&mut self) -> Result<(), HtmlTokenizerSessionControlError> {
        let Some(mode) = self.suspended_mode else {
            return Err(HtmlTokenizerSessionControlError::TitleRcdataRequestedWithoutSuspension);
        };
        if mode != HtmlTokenizerMode::Rcdata {
            return Err(
                HtmlTokenizerSessionControlError::TitleRcdataRequestedForDifferentMode(mode),
            );
        }
        if !self
            .engine
            .as_mut()
            .expect("suspended tokenizer engine")
            .activate_title_rcdata_from_suspended_start()
        {
            return Err(HtmlTokenizerSessionControlError::TitleRcdataActivationInvariant);
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
        if self.raw_text_start_tag_index.is_some() || self.rcdata_start_tag_index.is_some() {
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
                if let Err(stop) = self.push_text_mode_opening_literal() {
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
                if let Err(stop) = self.fallback_text_mode_end_tag_candidate() {
                    return stop;
                }
                self.state = State::RawText;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    /// Called by the parent Engine's ordinary `finish_tag()` after the
    /// selected appropriate RAWTEXT (TC-S9) or RCDATA (TC-S10) end-tag token
    /// has committed. The tokenizer has already returned to Data; this only
    /// yields so the coordinator can attribute the close to its episode.
    pub(super) fn raw_text_close_yield(&mut self, boundary_at: usize) -> Step {
        self.raw_text_start_tag_index = None;
        self.raw_text_closing_tag = false;
        self.text_mode_close_yield_marker(boundary_at)
    }

    /// The TC-S10 RCDATA counterpart. Separate from the RAWTEXT yield so each
    /// episode clears only its own lexical lifecycle state.
    pub(super) fn rcdata_close_yield(&mut self, boundary_at: usize) -> Step {
        self.rcdata_start_tag_index = None;
        self.rcdata_closing_tag = false;
        self.text_mode_close_yield_marker(boundary_at)
    }

    /// Builds the private Engine-yield marker both selected episodes use.
    ///
    /// A pure constructor over already-committed token evidence: it owns no
    /// episode identity, which is exactly why it can be shared.
    fn text_mode_close_yield_marker(&mut self, boundary_at: usize) -> Step {
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
        self.candidate_matches_episode_start(self.raw_text_start_tag_index)
    }

    /// The TC-S10 RCDATA episode's own appropriate-end-tag identity.
    fn rcdata_candidate_is_appropriate(&self) -> bool {
        self.candidate_matches_episode_start(self.rcdata_start_tag_index)
    }

    /// Whether the active end-tag candidate is the appropriate close for the
    /// episode whose retained start tag is `start_tag_index`.
    ///
    /// A pure comparison against retained token evidence. The *identity* it is
    /// asked about is supplied by the caller's own episode, so a Style episode
    /// can never be closed by asking the Title question or the reverse.
    fn candidate_matches_episode_start(&self, start_tag_index: Option<usize>) -> bool {
        let Some(start_index) = start_tag_index else {
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
    fn push_text_mode_opening_literal(&mut self) -> Result<(), Step> {
        let less_than_end = self.tag_open_start + '<'.len_utf8();
        self.push_data_char('<', self.tag_open_start, less_than_end)?;
        self.push_data_char('/', less_than_end, self.tag_open_delimiter_end)?;
        Ok(())
    }

    /// Reclassifies the current RAWTEXT end-tag candidate as character data
    /// using only exact offsets and the ASCII spelling retained while the
    /// single forward cursor examined the candidate. No source is searched,
    /// sliced, rescanned, or retokenized.
    fn fallback_text_mode_end_tag_candidate(&mut self) -> Result<(), Step> {
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

    /// Activates the selected Title RCDATA episode only from the exact
    /// existing post-start-tag suspended boundary, and only for `title`.
    ///
    /// The interpreted-name check is the selected boundary itself, not a
    /// convenience: `textarea` shares the durable `Rcdata` mode, so accepting
    /// any suspended RCDATA start tag would silently turn TC-S10 into general
    /// RCDATA support.
    pub(super) fn activate_title_rcdata_from_suspended_start(&mut self) -> bool {
        let Some(token_index) = self.tokens.len().checked_sub(1) else {
            return false;
        };
        let Some(HtmlToken::Tag(tag)) = self.tokens.get(token_index) else {
            return false;
        };
        if tag.kind() != HtmlTagKind::Start
            || tag.name().interpreted() != "title"
            || context_dependent_mode(tag.name().interpreted()) != Some(HtmlTokenizerMode::Rcdata)
            || self.state != State::Data
            || self.tag.is_some()
            || self.pending_reconsume
        {
            return false;
        }
        if self.raw_text_start_tag_index.is_some() || self.rcdata_start_tag_index.is_some() {
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
                // The authored `&` is consumed by the ordinary run loop, so
                // committed coverage never lags the entry it explains. It is
                // not yet interpreted: whether it becomes ordinary text or a
                // resolved reference is decided by the next unit, and the
                // pending Data run is deliberately left intact so the
                // ordinary-text outcome costs no extra token.
                self.character_reference_start = (start, end);
                self.state = State::CharacterReference;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                // RCDATA NUL recovery is reached but not selected by TC-S10.
                // Refuse before committing the scalar, so no replacement
                // output and no recovery diagnostic is ever claimed.
                self.rcdata_unsupported_input_stop(
                    HtmlTokenizerCapability::RcdataNullRecovery,
                    (start, end),
                )
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
                if let Err(stop) = self.push_text_mode_opening_literal() {
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
                if let Err(stop) = self.fallback_text_mode_end_tag_candidate() {
                    return stop;
                }
                self.state = State::Rcdata;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    /// The character reference state, with the RCDATA return state.
    ///
    /// `unit` is the already-materialized scalar following the authored `&`.
    /// This dispatch decides *which* branch the reference takes and costs one
    /// transition; it deliberately performs no discovery and consumes no
    /// further source, so the whole selected Named operation stays one
    /// transition of its own.
    pub(super) fn step_character_reference(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar {
                ch: '#',
                start,
                end,
            } => {
                // The Numeric branch is reached but not selected by TC-S10.
                // The authored `&` stays committed; the `#` travels only as
                // the trigger identifying the refused branch.
                self.rcdata_unsupported_input_stop(
                    HtmlTokenizerCapability::NumericCharacterReferenceInRcdata,
                    (start, end),
                )
            }
            InputUnit::Scalar { ch, .. } if ch.is_ascii_alphanumeric() => {
                // Reconsume the same scalar in the Named state, so discovery,
                // preparation, matched-source consumption and commit are one
                // indivisible transition rather than one per authored scalar.
                self.state = State::NamedCharacterReference;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                // Nothing that can begin a reference follows: the authored
                // `&` is ordinary RCDATA text and this unit is reconsumed
                // there unchanged.
                if let Err(stop) = self.flush_character_reference_ampersand() {
                    return stop;
                }
                self.state = State::Rcdata;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    /// The whole selected Named Character Reference operation, as one
    /// transition-level step.
    ///
    /// The lifecycle is fixed and ordered:
    ///
    /// ```text
    /// bounded non-committing discovery
    ///         ↓
    /// semantic/resource preparation      (every fallible resource decision)
    ///         ↓
    /// candidate evidence construction    (every fallible construction)
    ///         ↓
    /// authoritative matched-source consumption
    ///         ↓
    /// infallible semantic commit
    /// ```
    ///
    /// Nothing after consumption can refuse: the token and any required
    /// diagnostic already exist as values, and their capacity was reserved
    /// before a single scalar of the identifier was consumed. A refusal
    /// therefore always happens with the identifier wholly unconsumed, so no
    /// resource exhaustion can expose authored identifier bytes inside
    /// committed coverage with no evidence to explain them.
    ///
    /// `unit` is the reconsumed first identifier scalar; the cursor sits
    /// immediately after it.
    pub(super) fn step_named_character_reference(&mut self, unit: InputUnit) -> Step {
        let InputUnit::Scalar { ch: first, .. } = unit else {
            return internal_invariant_stop(HtmlTokenizerInvariantFailure::CursorState);
        };

        // 1. Bounded, non-committing discovery.
        let borrowed = self
            .cursor
            .peek_unconsumed_bytes(named_character_reference::maximum_lookahead_bytes());
        let Some(selected) = named_character_reference::maximum_match(first, borrowed) else {
            return self.begin_ambiguous_ampersand_run(unit);
        };
        if !selected.is_plainly_consumable() {
            // Checked, not assumed: the infallible consumption below relies on
            // every matched byte being an ASCII scalar the authoritative
            // cursor returns unchanged and without a preprocessing
            // diagnostic. Nothing is consumed or committed at this point.
            return internal_invariant_stop(HtmlTokenizerInvariantFailure::CursorState);
        }
        let ampersand_start = self.character_reference_start.0;
        let name_end = self.character_reference_start.1 + selected.name.len();
        let at = (self.processed_end, self.processed_end);

        // 2. Preparation: every fallible resource decision, in the accepted
        //    order, before any authored identifier scalar is consumed.
        if let Err(stop) = self.try_reserve_retained(selected.value.len(), ampersand_start) {
            return stop;
        }
        // Ordinary RCDATA text observed before the `&` is *prior* evidence,
        // not part of this entity: it ends at the `&`, the stop path would
        // flush it anyway, and emitting it separately keeps every
        // `EmittedTokens` refusal a single one-token attempt.
        if let Err(stop) = self.flush_data_run() {
            return stop;
        }
        let committed = match self.preflight_token_emission(selected.value.len(), at) {
            Ok(committed) => committed,
            Err(stop) => return stop,
        };
        if !selected.ends_with_semicolon()
            && let Err(stop) = self.preflight_pending_emission_diagnostics(1, at)
        {
            return stop;
        }

        // 3. Candidate evidence construction: every fallible construction.
        let anchor = self.anchor(ampersand_start, name_end);
        let Ok(character) = HtmlCharacterToken::new(anchor, selected.value.to_owned()) else {
            return internal_invariant_stop(
                HtmlTokenizerInvariantFailure::SourceEvidenceConstruction,
            );
        };
        let token = HtmlToken::Character(character);
        let missing_semicolon = if selected.ends_with_semicolon() {
            None
        } else {
            // The resolved reference will occupy exactly this index, because
            // the pending run was already flushed above and nothing else can
            // emit before the commit below.
            let token_index = self.tokens.len();
            let location = (name_end - 1, name_end);
            let Ok(diagnostic) = HtmlTokenizerDiagnostic::new(
                self.source,
                HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference,
                self.anchor(location.0, location.1),
                HtmlTokenizerDiagnosticContext::NamedCharacterReference,
                HtmlTokenizerDiagnosticHandling::Continued,
                HtmlTokenizerDiagnosticSubject::EmittedToken { token_index },
            ) else {
                return internal_invariant_stop(
                    HtmlTokenizerInvariantFailure::SourceEvidenceConstruction,
                );
            };
            Some((diagnostic, location))
        };

        // 4. Authoritative matched-source consumption. Charged to this one
        //    transition, never per scalar.
        let consumed_end = self.consume_discovered_source(selected.name.len() - 1);
        if consumed_end != name_end {
            return internal_invariant_stop(HtmlTokenizerInvariantFailure::CursorState);
        }

        // 5. Infallible semantic commit.
        self.processed_end = self.processed_end.max(name_end);
        self.commit_token(token, committed);
        if let Some((diagnostic, location)) = missing_semicolon {
            self.commit_prepared_diagnostic(diagnostic, location);
        }
        self.state = State::Rcdata;
        Step::Continue
    }

    /// Opens the unresolved Ambiguous Ampersand candidate as its own run.
    ///
    /// The candidate is a distinct semantic unit, so any ordinary RCDATA text
    /// observed before the `&` is flushed first and the candidate starts a
    /// fresh run at the authored `&`. That is what lets the run close at its
    /// own boundary, before the delimiter is reconsumed in RCDATA.
    fn begin_ambiguous_ampersand_run(&mut self, unit: InputUnit) -> Step {
        let InputUnit::Scalar { ch, start, end } = unit else {
            return internal_invariant_stop(HtmlTokenizerInvariantFailure::CursorState);
        };
        if let Err(stop) = self.flush_data_run() {
            return stop;
        }
        if let Err(stop) = self.flush_character_reference_ampersand() {
            return stop;
        }
        if let Err(stop) = self.push_data_char(ch, start, end) {
            return stop;
        }
        self.state = State::AmbiguousAmpersand;
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
                // Observation-conditioned, and recorded before the candidate
                // run is closed: the parse error is true once the run is
                // observed to complete no generated identifier, and it must
                // survive a refused flush of that run.
                if let Err(stop) = self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::UnknownNamedCharacterReference,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::AmbiguousAmpersand,
                    HtmlTokenizerDiagnosticHandling::Continued,
                    HtmlTokenizerDiagnosticSubject::InputLocation,
                ) {
                    return stop;
                }
                self.close_ambiguous_ampersand_run()
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                self.close_ambiguous_ampersand_run()
            }
        }
    }

    /// Closes the unresolved candidate at its own boundary and reconsumes the
    /// delimiter in RCDATA.
    ///
    /// The delimiter is never consumed as part of the candidate: it stays
    /// authored input and belongs to whatever contribution follows. A refused
    /// close leaves the candidate run intact and consumes nothing.
    fn close_ambiguous_ampersand_run(&mut self) -> Step {
        if let Err(stop) = self.flush_data_run() {
            return stop;
        }
        self.state = State::Rcdata;
        self.pending_reconsume = true;
        Step::Continue
    }

    /// Flushes the authored `&` that entered the character reference state
    /// into the pending RCDATA run as ordinary text.
    fn flush_character_reference_ampersand(&mut self) -> Result<(), Step> {
        let (start, end) = self.character_reference_start;
        self.push_data_char('&', start, end)
    }

    fn rcdata_unsupported_input_stop(
        &mut self,
        capability: HtmlTokenizerCapability,
        trigger: (usize, usize),
    ) -> Step {
        let trigger = self.discovery_trigger(trigger);
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
