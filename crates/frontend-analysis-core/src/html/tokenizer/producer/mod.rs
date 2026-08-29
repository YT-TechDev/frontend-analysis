//! The crate-private bounded lossless HTML tokenizer production engine.
//!
//! Implements the #109-approved Data-context state subset, TC-S9's selected
//! RAWTEXT extension, and TC-S10's selected RCDATA + Named Character
//! Reference extension over a known-valid-UTF-8 `SourceText` using an
//! iterative, non-recursive, single-forward-owner cursor. Transition
//! accounting, resource accounting, diagnostics, and completion follow the
//! corrected #111/#112 contracts.

mod builder;
mod cursor;
mod named_character_reference;
mod session;
mod state;

pub(crate) use self::session::{
    HtmlTokenizerSession, HtmlTokenizerSessionBoundary, HtmlTokenizerSessionControlError,
};

use crate::html::token::{
    HtmlCharacterToken, HtmlEndOfFileToken, HtmlPreprocessingEvidence, HtmlTagKind, HtmlToken,
};
use crate::{SourceAnchor, SourceText};

use self::builder::{
    AttributeNameBuilder, AttributeValueBuilder, PendingEmissionDiagnostic, TagBuilder,
};
use self::cursor::{Cursor, InputUnit};
use self::state::State;

use super::diagnostic::{
    HtmlTokenizerDiagnostic, HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticContext,
    HtmlTokenizerDiagnosticHandling, HtmlTokenizerDiagnosticSubject, HtmlTokenizerRecoveryKind,
};
use super::resource::{
    HtmlTokenizerConfigurationFailure, HtmlTokenizerInvariantFailure, HtmlTokenizerLimits,
    HtmlTokenizerResource, HtmlTokenizerResourceLimit, HtmlTokenizerUsage,
};
use super::result::{
    HtmlTokenizerCompletion, HtmlTokenizerCoverage, HtmlTokenizerIncompleteCause,
    HtmlTokenizerRunResult,
};

/// Produces the collected run result for one synchronous tokenizer pass.
///
/// The public-in-crate batch contract is intentionally unchanged. It drains
/// the same private resumable session used by coordinated tree construction;
/// without tree feedback, the first context-dependent boundary publishes the
/// exact predecessor deferred-unsupported result.
pub(crate) fn tokenize(source: &SourceText, limits: HtmlTokenizerLimits) -> HtmlTokenizerRunResult {
    HtmlTokenizerSession::new(source, limits).finish_batch_compatible()
}

fn invalid_configuration_result(
    source: &SourceText,
    limits: HtmlTokenizerLimits,
    failure: HtmlTokenizerConfigurationFailure,
) -> HtmlTokenizerRunResult {
    let empty = source.anchor(0, 0).expect("valid empty anchor");
    HtmlTokenizerRunResult::new(
        source,
        Vec::new(),
        HtmlPreprocessingEvidence::new(source, None).expect("valid preprocessing evidence"),
        Vec::new(),
        HtmlTokenizerCoverage::new(
            source,
            empty.clone(),
            source
                .anchor(0, source.as_str().len())
                .expect("valid coverage"),
        )
        .expect("valid empty coverage"),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::InvalidConfiguration(
            failure,
        )),
        limits,
        HtmlTokenizerUsage::new(source.as_str().len(), 0, 0, 0, 0, 0, 0),
    )
    .expect("valid invalid-configuration run result")
}

fn source_bytes_limit_result(
    source: &SourceText,
    limits: HtmlTokenizerLimits,
    source_len: usize,
) -> HtmlTokenizerRunResult {
    let empty = source.anchor(0, 0).expect("valid empty anchor");
    let resource_limit = HtmlTokenizerResourceLimit::new(
        source,
        HtmlTokenizerResource::SourceBytes,
        limits.max_source_bytes(),
        source_len,
        empty.clone(),
    )
    .expect("valid source-bytes resource limit evidence");
    HtmlTokenizerRunResult::new(
        source,
        Vec::new(),
        HtmlPreprocessingEvidence::new(source, None).expect("valid preprocessing evidence"),
        Vec::new(),
        HtmlTokenizerCoverage::new(
            source,
            empty.clone(),
            source.anchor(0, source_len).expect("valid coverage"),
        )
        .expect("valid empty coverage"),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )),
        limits,
        HtmlTokenizerUsage::new(source_len, 0, 0, 0, 0, 0, 0),
    )
    .expect("valid source-bytes-limit run result")
}

/// A contiguous run of character output under construction.
struct DataRun {
    start: usize,
    end: usize,
    interpreted: String,
}

/// The outcome of one dispatch: keep going, complete the run, or stop with
/// an incomplete cause.
enum Step {
    Continue,
    Complete,
    Stop(HtmlTokenizerIncompleteCause),
}

struct Engine<'a> {
    source: &'a SourceText,
    limits: HtmlTokenizerLimits,
    cursor: Cursor<'a>,
    state: State,
    current: InputUnit,
    pending_reconsume: bool,
    processed_end: usize,
    /// The strongest lower bound `processed_end` may ever be rolled back to:
    /// the highest end position among all tokens and diagnostics committed
    /// so far. A markup-declaration-style rollback must never drop coverage
    /// below evidence that already exists in the run's output.
    min_processed_end: usize,
    skipped_leading_bom: Option<(usize, usize)>,
    tag_open_start: usize,
    /// The exact raw end of the opening delimiter captured as it was
    /// consumed: the end of `<` alone, or the end of `/` once a `</` end-tag
    /// opener is recognized. Never reconstructed from tag kind afterward.
    tag_open_delimiter_end: usize,
    pending_solidus: Option<(usize, usize)>,
    data_run: Option<DataRun>,
    tag: Option<TagBuilder>,
    tokens: Vec<HtmlToken>,
    diagnostics: Vec<HtmlTokenizerDiagnostic>,
    committed_retained_bytes: usize,
    peak_attributes_per_tag: usize,
    transition_steps: usize,
    /// Retained emitted start-tag index that owns appropriate-end-tag meaning
    /// during the selected RAWTEXT (TC-S9) or RCDATA (TC-S10) episode. Never
    /// supplied by tree state. Appropriate-end-tag identity is one shared
    /// lexical concept across both text modes, so it is one field rather than
    /// two parallel ones; which mode is active remains `self.state`.
    text_mode_start_tag_index: Option<usize>,
    /// True only after RAWTEXT/RCDATA end-tag-name recognition proved an
    /// appropriate candidate and ordinary tag finalization is finishing that
    /// exact token.
    text_mode_closing_tag: bool,
    /// The exact authored `&` span that entered the selected character
    /// reference state. Retained only for the duration of one reference: it
    /// is authored evidence, never interpreted output or a matching buffer.
    character_reference_start: (usize, usize),
    /// The selected maximum match whose remaining authored scalars the
    /// [`State::NamedCharacterReference`] state is consuming, together with
    /// the still-unconsumed byte offset within its generated identifier.
    /// Holds no copied candidate: the identifier is a `&'static str` owned by
    /// the compiler-sealed canonical generated data.
    pending_named_reference: Option<PendingNamedReference>,
}

/// The bounded plan for one already-selected maximum match.
///
/// This is a decision, not a buffer: it retains the generated identifier by
/// static reference plus authored offsets, so it contributes nothing to
/// `TemporaryBufferBytes`.
#[derive(Debug, Clone, Copy)]
struct PendingNamedReference {
    selected: named_character_reference::NamedCharacterReferenceMatch,
    /// Bytes of the generated identifier already consumed authoritatively.
    consumed_name_bytes: usize,
}

impl<'a> Engine<'a> {
    fn new(source: &'a SourceText, limits: HtmlTokenizerLimits) -> Self {
        let (cursor, bom) = Cursor::new(source);
        Self {
            source,
            limits,
            cursor,
            state: State::Data,
            current: InputUnit::Eof { at: 0 },
            pending_reconsume: false,
            processed_end: bom.map_or(0, |(_, end)| end),
            min_processed_end: bom.map_or(0, |(_, end)| end),
            skipped_leading_bom: bom,
            tag_open_start: 0,
            tag_open_delimiter_end: 0,
            pending_solidus: None,
            data_run: None,
            tag: None,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
            committed_retained_bytes: 0,
            peak_attributes_per_tag: 0,
            transition_steps: 0,
            text_mode_start_tag_index: None,
            text_mode_closing_tag: false,
            character_reference_start: (0, 0),
            pending_named_reference: None,
        }
    }

    fn run(&mut self) -> HtmlTokenizerCompletion {
        loop {
            let mut preprocessing_failure = None;
            let unit = if self.pending_reconsume {
                self.pending_reconsume = false;
                self.current
            } else {
                let (unit, diagnostic_code) = self.cursor.advance();
                self.current = unit;
                if let Some(code) = diagnostic_code
                    && let Err(stop) = self.append_preprocessing_diagnostic(code, unit)
                {
                    preprocessing_failure = Some(stop);
                }
                unit
            };

            let step = match preprocessing_failure {
                Some(stop) => stop,
                None => match self.commit_transition_step(unit.start()) {
                    Ok(()) => self.dispatch(unit),
                    Err(stop) => stop,
                },
            };

            match step {
                Step::Continue => {
                    if !self.pending_reconsume {
                        self.processed_end = unit.end();
                    }
                }
                Step::Complete => return HtmlTokenizerCompletion::Complete,
                Step::Stop(cause) => {
                    self.best_effort_flush_on_stop();
                    // An unsupported-capability discovery releases any
                    // active tag builder (established #109 architecture:
                    // "a deliberate stopping point... not a forced
                    // mid-operation refusal"). Every other stop cause
                    // (resource limit, internal invariant failure) freezes
                    // it instead: the builder and its already-retained
                    // bytes stay exactly as they were, visible through
                    // `active_retained_bytes` at `into_result` time. Either
                    // way, any diagnostic that provisionally
                    // forward-referenced this tag's future token index is
                    // retargeted, since the tag will never emit.
                    if matches!(
                        cause,
                        HtmlTokenizerIncompleteCause::UnsupportedCapability(_)
                    ) {
                        self.release_active_tag();
                    } else {
                        self.retarget_active_tag_diagnostics();
                    }
                    return HtmlTokenizerCompletion::Incomplete(cause);
                }
            }
        }
    }

    fn into_result(self, completion: HtmlTokenizerCompletion) -> HtmlTokenizerRunResult {
        let source_len = self.source.as_str().len();
        let processed_end = if matches!(completion, HtmlTokenizerCompletion::Complete) {
            source_len
        } else {
            self.processed_end
        };
        let processed_prefix = self
            .source
            .anchor(0, processed_end)
            .expect("valid processed prefix");
        let unprocessed_suffix = self
            .source
            .anchor(processed_end, source_len)
            .expect("valid unprocessed suffix");
        let coverage =
            HtmlTokenizerCoverage::new(self.source, processed_prefix, unprocessed_suffix)
                .expect("valid coverage");
        let skipped_leading_bom = self
            .skipped_leading_bom
            .map(|(start, end)| self.source.anchor(start, end).expect("valid bom anchor"));
        let preprocessing = HtmlPreprocessingEvidence::new(self.source, skipped_leading_bom)
            .expect("valid preprocessing evidence");
        let peak_attributes_per_tag = self.peak_attributes_per_tag;
        let retained_interpreted_bytes = self
            .committed_retained_bytes
            .checked_add(
                self.active_retained_bytes()
                    .expect("retained bytes bounded by the enforced limit throughout the run"),
            )
            .expect("retained bytes bounded by the enforced limit throughout the run");
        let usage = HtmlTokenizerUsage::new(
            source_len,
            self.transition_steps,
            self.tokens.len(),
            self.diagnostics.len(),
            peak_attributes_per_tag,
            retained_interpreted_bytes,
            0,
        );
        HtmlTokenizerRunResult::new(
            self.source,
            self.tokens,
            preprocessing,
            self.diagnostics,
            coverage,
            completion,
            self.limits,
            usage,
        )
        .expect("valid run result")
    }

    fn best_effort_flush_on_stop(&mut self) {
        if self.data_run.is_none() {
            return;
        }
        let _ = self.flush_data_run();
    }

    /// Retargets a still-active tag's provisional diagnostic subjects
    /// without releasing the builder itself: used for resource-limit and
    /// internal-invariant stops, where the tag remains frozen (matching
    /// every other frozen-builder resource-limit stop) and its retained
    /// bytes stay exactly as they were, visible through
    /// `active_retained_bytes` at `into_result` time.
    fn retarget_active_tag_diagnostics(&mut self) {
        let Some(tag) = self.tag.as_ref() else {
            return;
        };
        let diagnostics_start = tag.diagnostics_start;
        let region = self.anchor(tag.tag_start, self.processed_end);
        self.resolve_abandoned_tag_diagnostics(diagnostics_start, &region);
    }

    /// Releases a still-active tag builder that will never become a token:
    /// retargets its provisional diagnostic subjects, then discards the
    /// builder (its retained bytes are not folded into the committed count,
    /// matching the established released-builder-loses-its-evidence
    /// behavior of a deliberate, non-resource-limited stop).
    fn release_active_tag(&mut self) {
        let Some(tag) = self.tag.take() else {
            return;
        };
        let region = self.anchor(tag.tag_start, self.processed_end);
        self.resolve_abandoned_tag_diagnostics(tag.diagnostics_start, &region);
    }

    /// Retargets every diagnostic appended since `diagnostics_start` whose
    /// subject provisionally forward-referenced the (never emitted) active
    /// tag's future token index, to contract-valid `AbandonedInput`
    /// evidence bounded by `region`. Mutating a diagnostic's subject in
    /// place is not a fallible/resource-limited operation: it changes no
    /// counted resource and cannot itself be refused.
    fn resolve_abandoned_tag_diagnostics(
        &mut self,
        diagnostics_start: usize,
        region: &SourceAnchor,
    ) {
        let future_index = self.tokens.len();
        for diagnostic in &mut self.diagnostics[diagnostics_start..] {
            let needs_retarget = matches!(
                diagnostic.subject(),
                HtmlTokenizerDiagnosticSubject::EmittedToken { token_index } if *token_index == future_index
            );
            if needs_retarget {
                *diagnostic = HtmlTokenizerDiagnostic::new(
                    self.source,
                    diagnostic.code(),
                    diagnostic.location().clone(),
                    diagnostic.context(),
                    diagnostic.handling(),
                    HtmlTokenizerDiagnosticSubject::AbandonedInput {
                        region: region.clone(),
                    },
                )
                .expect("valid retargeted diagnostic");
            }
        }
    }

    fn dispatch(&mut self, unit: InputUnit) -> Step {
        match self.state {
            State::Data => self.step_data(unit),
            State::TagOpen => self.step_tag_open(unit),
            State::EndTagOpen => self.step_end_tag_open(unit),
            State::TagName => self.step_tag_name(unit),
            State::BeforeAttributeName => self.step_before_attribute_name(unit),
            State::AttributeName => self.step_attribute_name(unit),
            State::AfterAttributeName => self.step_after_attribute_name(unit),
            State::BeforeAttributeValue => self.step_before_attribute_value(unit),
            State::AttributeValueDoubleQuoted => self.step_attribute_value_quoted(unit, true),
            State::AttributeValueSingleQuoted => self.step_attribute_value_quoted(unit, false),
            State::AttributeValueUnquoted => self.step_attribute_value_unquoted(unit),
            State::AfterAttributeValueQuoted => self.step_after_attribute_value_quoted(unit),
            State::SelfClosingStartTag => self.step_self_closing_start_tag(unit),
            State::RawText => self.step_raw_text(unit),
            State::RawTextLessThanSign => self.step_raw_text_less_than_sign(unit),
            State::RawTextEndTagOpen => self.step_raw_text_end_tag_open(unit),
            State::RawTextEndTagName => self.step_raw_text_end_tag_name(unit),
            State::Rcdata => self.step_rcdata(unit),
            State::RcdataLessThanSign => self.step_rcdata_less_than_sign(unit),
            State::RcdataEndTagOpen => self.step_rcdata_end_tag_open(unit),
            State::RcdataEndTagName => self.step_rcdata_end_tag_name(unit),
            State::CharacterReference => self.step_character_reference(unit),
            State::NamedCharacterReference => self.step_named_character_reference(unit),
            State::AmbiguousAmpersand => self.step_ambiguous_ampersand(unit),
        }
    }

    // ---- Data ----------------------------------------------------------

    fn step_data(&mut self, unit: InputUnit) -> Step {
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
                ..
            } => {
                if let Err(stop) = self.flush_data_run() {
                    return stop;
                }
                self.tag_open_start = start;
                self.tag_open_delimiter_end = end;
                self.state = State::TagOpen;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '&',
                start,
                end,
            } => {
                if let Err(stop) = self.flush_data_run() {
                    return stop;
                }
                let trigger = self.discovery_trigger((start, end));
                self.unsupported_input_stop(
                    super::result::HtmlTokenizerCapability::CharacterReference {
                        context: super::result::HtmlCharacterReferenceContext::Data,
                    },
                    trigger,
                )
            }
            InputUnit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                match self.recover_null_character(
                    HtmlTokenizerDiagnosticContext::Data,
                    (start, end),
                    None,
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                if let Err(stop) = self.push_data_char('\u{fffd}', start, end) {
                    return stop;
                }
                Step::Continue
            }
            InputUnit::Scalar { ch, start, end } => {
                if let Err(stop) = self.push_data_char(ch, start, end) {
                    return stop;
                }
                Step::Continue
            }
        }
    }

    fn push_data_char(&mut self, ch: char, start: usize, end: usize) -> Result<(), Step> {
        let delta = ch.len_utf8();
        self.try_reserve_retained(delta, start)?;
        match &mut self.data_run {
            Some(run) => {
                run.interpreted.push(ch);
                run.end = end;
            }
            None => {
                let mut interpreted = String::new();
                interpreted.push(ch);
                self.data_run = Some(DataRun {
                    start,
                    end,
                    interpreted,
                });
            }
        }
        Ok(())
    }

    fn flush_data_run(&mut self) -> Result<(), Step> {
        let Some(run) = self.data_run.as_ref() else {
            return Ok(());
        };
        // Preflight before destroying the active run: a refused emission
        // must leave the pending Data run exactly as it was, not silently
        // lost.
        let committed =
            self.preflight_token_emission(run.interpreted.len(), (run.start, run.start))?;
        let run = self.data_run.take().expect("data run active");
        let anchor = self.anchor(run.start, run.end);
        let token = HtmlToken::Character(
            HtmlCharacterToken::new(anchor, run.interpreted).expect("valid character token"),
        );
        self.commit_token(token, committed);
        Ok(())
    }

    // ---- Tag open --------------------------------------------------------

    fn step_tag_open(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Eof { at } => {
                // The retained-byte cost of the literal `<` recovery is
                // reserved before the diagnostic claims `Recovered`: a
                // resource refusal must not follow a diagnostic that
                // already asserted the recovery happened.
                if let Err(stop) = self.try_reserve_retained(1, self.tag_open_start) {
                    return stop;
                }
                if let Err(stop) = self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::EofBeforeTagName,
                    (at, at),
                    HtmlTokenizerDiagnosticContext::TagOpen,
                    HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::EmittedLiteralMarkupPrefix,
                    ),
                    HtmlTokenizerDiagnosticSubject::InputLocation,
                ) {
                    return stop;
                }
                let literal = self.anchor(self.tag_open_start, self.tag_open_delimiter_end);
                let token = HtmlToken::Character(
                    HtmlCharacterToken::new(literal, "<".to_owned())
                        .expect("valid literal character token"),
                );
                if let Err(stop) =
                    self.try_push_token(token, (self.tag_open_start, self.tag_open_start))
                {
                    return stop;
                }
                self.emit_eof(at)
            }
            InputUnit::Scalar {
                ch: '/',
                end: solidus_end,
                ..
            } => {
                self.tag_open_delimiter_end = solidus_end;
                self.state = State::EndTagOpen;
                Step::Continue
            }
            InputUnit::Scalar { ch, .. } if ch.is_ascii_alphabetic() => {
                self.tag = Some(TagBuilder::new(
                    HtmlTagKind::Start,
                    self.tag_open_start,
                    (self.tag_open_start, self.tag_open_delimiter_end),
                    unit.start(),
                    self.diagnostics.len(),
                ));
                self.state = State::TagName;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar { ch: '!', end, .. } => {
                self.processed_end = self.tag_open_start.max(self.min_processed_end);
                let trigger = self.discovery_trigger((self.tag_open_start, end));
                self.unsupported_input_stop(
                    super::result::HtmlTokenizerCapability::MarkupDeclaration,
                    trigger,
                )
            }
            InputUnit::Scalar {
                ch: '?',
                start,
                end,
            } => {
                match self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::UnexpectedQuestionMarkInsteadOfTagName,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::TagOpen,
                    HtmlTokenizerDiagnosticHandling::Stopped,
                    HtmlTokenizerDiagnosticSubject::InputLocation,
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                self.processed_end = end;
                self.unsupported_input_stop(
                    super::result::HtmlTokenizerCapability::ProcessingInstruction,
                    (end, end),
                )
            }
            InputUnit::Scalar { start, end, .. } => {
                // Reserve the literal `<` recovery's retained-byte cost
                // before the diagnostic claims `Recovered`.
                if let Err(stop) = self.try_reserve_retained(1, self.tag_open_start) {
                    return stop;
                }
                if let Err(stop) = self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::InvalidFirstCharacterOfTagName,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::TagOpen,
                    HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::EmittedLiteralMarkupPrefix,
                    ),
                    HtmlTokenizerDiagnosticSubject::InputLocation,
                ) {
                    return stop;
                }
                if let Err(stop) =
                    self.push_data_char('<', self.tag_open_start, self.tag_open_delimiter_end)
                {
                    return stop;
                }
                self.state = State::Data;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    // ---- End tag open ------------------------------------------------

    fn step_end_tag_open(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if ch.is_ascii_alphabetic() => {
                self.tag = Some(TagBuilder::new(
                    HtmlTagKind::End,
                    self.tag_open_start,
                    (self.tag_open_start, self.tag_open_delimiter_end),
                    unit.start(),
                    self.diagnostics.len(),
                ));
                self.state = State::TagName;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '>',
                start,
                end,
            } => {
                match self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::MissingEndTagName,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::EndTagOpen,
                    HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::IgnoredUnexpectedInput,
                    ),
                    HtmlTokenizerDiagnosticSubject::InputLocation,
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                self.state = State::Data;
                Step::Continue
            }
            InputUnit::Eof { at } => {
                // Reserve the literal `</` recovery's retained-byte cost
                // before the diagnostic claims `Recovered`.
                if let Err(stop) = self.try_reserve_retained(2, self.tag_open_start) {
                    return stop;
                }
                if let Err(stop) = self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::EofBeforeTagName,
                    (at, at),
                    HtmlTokenizerDiagnosticContext::EndTagOpen,
                    HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::EmittedLiteralMarkupPrefix,
                    ),
                    HtmlTokenizerDiagnosticSubject::InputLocation,
                ) {
                    return stop;
                }
                let literal = self.anchor(self.tag_open_start, self.tag_open_delimiter_end);
                let token = HtmlToken::Character(
                    HtmlCharacterToken::new(literal, "</".to_owned())
                        .expect("valid literal character token"),
                );
                if let Err(stop) =
                    self.try_push_token(token, (self.tag_open_start, self.tag_open_start))
                {
                    return stop;
                }
                self.emit_eof(at)
            }
            InputUnit::Scalar { start, end, .. } => {
                match self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::InvalidFirstCharacterOfTagName,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::EndTagOpen,
                    HtmlTokenizerDiagnosticHandling::Stopped,
                    HtmlTokenizerDiagnosticSubject::InputLocation,
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                let trigger = self.discovery_trigger((start, end));
                self.unsupported_input_stop(
                    super::result::HtmlTokenizerCapability::BogusCommentRecovery,
                    trigger,
                )
            }
        }
    }

    // ---- Tag name ------------------------------------------------------

    fn step_tag_name(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Eof { at } => {
                self.abandon_tag_at_eof(HtmlTokenizerDiagnosticContext::TagName, at)
            }
            InputUnit::Scalar { ch, start, end } if is_html_whitespace(ch) => {
                let _ = (start, end);
                self.state = State::BeforeAttributeName;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '/',
                start,
                end,
            } => {
                self.pending_solidus = Some((start, end));
                self.state = State::SelfClosingStartTag;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '>',
                start,
                end,
            } => self.finish_tag((start, end)),
            InputUnit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                match self.recover_null_character(
                    HtmlTokenizerDiagnosticContext::TagName,
                    (start, end),
                    Some(self.tokens.len()),
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                if let Err(stop) = self.try_reserve_retained('\u{fffd}'.len_utf8(), start) {
                    return stop;
                }
                let tag = self.tag.as_mut().expect("tag active");
                tag.push_name('\u{fffd}', end);
                Step::Continue
            }
            InputUnit::Scalar { ch, start, end } => {
                let normalized = ch.to_ascii_lowercase();
                if let Err(stop) = self.try_reserve_retained(normalized.len_utf8(), start) {
                    return stop;
                }
                let tag = self.tag.as_mut().expect("tag active");
                tag.push_name(normalized, end);
                Step::Continue
            }
        }
    }

    // ---- Before attribute name ------------------------------------------

    fn step_before_attribute_name(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if is_html_whitespace(ch) => Step::Continue,
            InputUnit::Scalar { ch: '/', .. }
            | InputUnit::Scalar { ch: '>', .. }
            | InputUnit::Eof { .. } => {
                self.state = State::AfterAttributeName;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '=',
                start,
                end,
            } => {
                if let Err(stop) = self.try_start_attribute(start) {
                    return stop;
                }
                // Preflight the retained-byte cost of the literal `=` that
                // recovery is about to push into the new attribute name
                // before either diagnostic claims `Recovered`/observes the
                // recovered evidence, so a resource refusal cannot follow a
                // diagnostic that already asserted the recovery happened.
                if let Err(stop) = self.try_reserve_retained('='.len_utf8(), start) {
                    return stop;
                }
                match self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::UnexpectedEqualsSignBeforeAttributeName,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::BeforeAttributeName,
                    HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::StartedAttributeAtUnexpectedEqualsSign,
                    ),
                    HtmlTokenizerDiagnosticSubject::EmittedToken {
                        token_index: self.tokens.len(),
                    },
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                match self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::UnexpectedCharacterInAttributeName,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::AttributeName,
                    HtmlTokenizerDiagnosticHandling::Continued,
                    HtmlTokenizerDiagnosticSubject::EmittedToken {
                        token_index: self.tokens.len(),
                    },
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                let mut name = AttributeNameBuilder::new(start);
                name.push('=', end);
                self.tag.as_mut().expect("tag active").pending_name = Some(name);
                self.state = State::AttributeName;
                Step::Continue
            }
            InputUnit::Scalar { start, .. } => {
                if let Err(stop) = self.try_start_attribute(start) {
                    return stop;
                }
                self.tag.as_mut().expect("tag active").pending_name =
                    Some(AttributeNameBuilder::new(start));
                self.state = State::AttributeName;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    // ---- Attribute name --------------------------------------------------

    fn step_attribute_name(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if is_html_whitespace(ch) => {
                if let Err(stop) = self.on_attribute_name_complete() {
                    return stop;
                }
                self.state = State::AfterAttributeName;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar { ch: '/', .. }
            | InputUnit::Scalar { ch: '>', .. }
            | InputUnit::Eof { .. } => {
                if let Err(stop) = self.on_attribute_name_complete() {
                    return stop;
                }
                self.state = State::AfterAttributeName;
                self.pending_reconsume = true;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '=',
                start,
                end,
            } => {
                if let Err(stop) = self.on_attribute_name_complete() {
                    return stop;
                }
                self.tag.as_mut().expect("tag active").pending_equals = Some((start, end));
                self.state = State::BeforeAttributeValue;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                match self.recover_null_character(
                    HtmlTokenizerDiagnosticContext::AttributeName,
                    (start, end),
                    Some(self.tokens.len()),
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                self.push_attribute_name_char('\u{fffd}', start, end)
            }
            InputUnit::Scalar {
                ch: ch @ ('"' | '\'' | '<'),
                start,
                end,
            } => {
                match self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::UnexpectedCharacterInAttributeName,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::AttributeName,
                    HtmlTokenizerDiagnosticHandling::Continued,
                    HtmlTokenizerDiagnosticSubject::EmittedToken {
                        token_index: self.tokens.len(),
                    },
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                self.push_attribute_name_char(ch, start, end)
            }
            InputUnit::Scalar { ch, start, end } => {
                let normalized = ch.to_ascii_lowercase();
                self.push_attribute_name_char(normalized, start, end)
            }
        }
    }

    fn push_attribute_name_char(&mut self, ch: char, start: usize, end: usize) -> Step {
        if let Err(stop) = self.try_reserve_retained(ch.len_utf8(), start) {
            return stop;
        }
        let tag = self.tag.as_mut().expect("tag active");
        tag.pending_name
            .as_mut()
            .expect("pending attribute name active")
            .push(ch, end);
        Step::Continue
    }

    /// Runs once, at the exact moment the tokenizer leaves the
    /// attribute-name state for the current pending attribute, per the
    /// pinned WHATWG snapshot ("When the user agent leaves the attribute
    /// name state ... the complete attribute's name must be compared to the
    /// other attributes"). Determines and stores the duplicate-vs-effective
    /// disposition so later value finalization reads it back rather than
    /// recomputing it, and commits `DuplicateAttribute` (an ordinary
    /// observation-conditioned diagnostic) here — chronologically before any
    /// diagnostic the pending value could still produce — so the
    /// diagnostics vector never places an earlier-positioned diagnostic
    /// after a later-positioned one. `EndTagWithAttributes` is
    /// emission-conditioned (#111): its evidence is recorded as pending
    /// evidence on the tag builder here rather than committed immediately,
    /// since its underlying parse-error fact exists only if the end-tag
    /// token actually emits (see [`finish_tag`](Self::finish_tag)). It is
    /// recorded at most once per tag, for the first attribute only,
    /// matching the pinned "end-tag token emitted with attributes" single
    /// parse error.
    fn on_attribute_name_complete(&mut self) -> Result<(), Step> {
        let tag = self.tag.as_ref().expect("tag active");
        let name = tag
            .pending_name
            .as_ref()
            .expect("pending attribute name active");
        let disposition = tag.duplicate_disposition(&name.interpreted);
        let is_duplicate = matches!(
            disposition,
            crate::html::token::HtmlAttributeDisposition::DuplicateOf { .. }
        );
        let is_first_attribute = tag.attributes.is_empty();
        let is_end_tag = matches!(tag.kind, HtmlTagKind::End);
        let name_location = (name.start, name.end);

        self.tag
            .as_mut()
            .expect("tag active")
            .pending_name
            .as_mut()
            .expect("pending attribute name active")
            .disposition = Some(disposition);

        if is_duplicate {
            self.append_diagnostic(
                HtmlTokenizerDiagnosticCode::DuplicateAttribute,
                name_location,
                HtmlTokenizerDiagnosticContext::AttributeName,
                HtmlTokenizerDiagnosticHandling::Recovered(
                    HtmlTokenizerRecoveryKind::PreservedDuplicateAttributeOccurrence,
                ),
                HtmlTokenizerDiagnosticSubject::EmittedToken {
                    token_index: self.tokens.len(),
                },
            )?;
        }
        if is_end_tag && is_first_attribute {
            let insertion_index = self.diagnostics.len();
            self.tag
                .as_mut()
                .expect("tag active")
                .pending_emission_diagnostics
                .push(PendingEmissionDiagnostic {
                    code: HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
                    location: name_location,
                    context: HtmlTokenizerDiagnosticContext::AttributeName,
                    handling: HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
                    ),
                    insertion_index,
                });
        }
        Ok(())
    }

    // ---- After attribute name ------------------------------------------

    fn step_after_attribute_name(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if is_html_whitespace(ch) => Step::Continue,
            InputUnit::Scalar { ch: '/', .. } => {
                self.commit_pending_as_missing();
                self.pending_solidus = Some((unit.start(), unit.end()));
                self.state = State::SelfClosingStartTag;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '=',
                start,
                end,
            } => {
                self.tag.as_mut().expect("tag active").pending_equals = Some((start, end));
                self.state = State::BeforeAttributeValue;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '>',
                start,
                end,
                ..
            } => {
                self.commit_pending_as_missing();
                self.finish_tag((start, end))
            }
            InputUnit::Eof { at } => {
                self.commit_pending_as_missing();
                self.abandon_tag_at_eof(HtmlTokenizerDiagnosticContext::AfterAttributeName, at)
            }
            InputUnit::Scalar { start, .. } => {
                self.commit_pending_as_missing();
                if let Err(stop) = self.try_start_attribute(start) {
                    return stop;
                }
                self.tag.as_mut().expect("tag active").pending_name =
                    Some(AttributeNameBuilder::new(start));
                self.state = State::AttributeName;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    /// Finalizes the current pending attribute (if any) as a `Missing`
    /// value and pushes it onto the tag builder's attribute list. The
    /// duplicate/end-tag diagnostics were already committed by
    /// [`Self::on_attribute_name_complete`] when the name state was left,
    /// so this is infallible: nothing here can still be refused.
    fn commit_pending_as_missing(&mut self) {
        let tag = self.tag.as_mut().expect("tag active");
        let Some(name) = tag.pending_name.take() else {
            return;
        };
        let disposition = name
            .disposition
            .expect("disposition determined when leaving attribute-name state");
        let attribute = builder::finalize_missing_attribute(self.source, &name, disposition);
        self.push_attribute(attribute);
    }

    /// Pushes a fully finalized attribute onto the active tag. Infallible:
    /// `DuplicateAttribute` and `EndTagWithAttributes` are decided and
    /// committed earlier, in [`Self::on_attribute_name_complete`].
    fn push_attribute(&mut self, attribute: crate::html::token::HtmlAttributeEvidence) {
        let tag = self.tag.as_mut().expect("tag active");
        tag.attributes.push(attribute);
        self.peak_attributes_per_tag = self.peak_attributes_per_tag.max(tag.attributes.len());
    }

    // ---- Before attribute value ------------------------------------------

    fn step_before_attribute_value(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if is_html_whitespace(ch) => Step::Continue,
            InputUnit::Scalar {
                ch: '"',
                start,
                end,
            } => {
                self.begin_quoted_value(true, (start, end));
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '\'',
                start,
                end,
            } => {
                self.begin_quoted_value(false, (start, end));
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '>',
                start,
                end,
            } => {
                // The required diagnostic is checked before the pending
                // attribute name is taken: a diagnostics-limit refusal must
                // leave it untouched (still active) rather than destroying
                // it first.
                if let Err(stop) = self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::MissingAttributeValue,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::BeforeAttributeValue,
                    HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::CompletedTagWithMissingAttributeValue,
                    ),
                    HtmlTokenizerDiagnosticSubject::EmittedToken {
                        token_index: self.tokens.len(),
                    },
                ) {
                    return stop;
                }
                let equals = self.equals_span();
                let name = self
                    .tag
                    .as_mut()
                    .expect("tag active")
                    .pending_name
                    .take()
                    .expect("pending attribute name active");
                let disposition = name
                    .disposition
                    .expect("disposition determined when leaving attribute-name state");
                let attribute = builder::finalize_missing_after_equals_attribute(
                    self.source,
                    &name,
                    equals,
                    start,
                    disposition,
                );
                self.push_attribute(attribute);
                self.finish_tag((start, end))
            }
            InputUnit::Scalar { .. } | InputUnit::Eof { .. } => {
                let equals = self.equals_span();
                let value_start = unit.start();
                self.tag.as_mut().expect("tag active").pending_value =
                    Some(AttributeValueBuilder::new_unquoted(equals, value_start));
                self.state = State::AttributeValueUnquoted;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    fn equals_span(&self) -> (usize, usize) {
        self.tag
            .as_ref()
            .expect("tag active")
            .pending_equals
            .expect("equals span captured immediately before entering BeforeAttributeValue")
    }

    fn begin_quoted_value(&mut self, double: bool, open_quote: (usize, usize)) {
        let equals = self.equals_span();
        let builder = AttributeValueBuilder::new_quoted(double, equals, open_quote);
        self.tag.as_mut().expect("tag active").pending_value = Some(builder);
        self.state = if double {
            State::AttributeValueDoubleQuoted
        } else {
            State::AttributeValueSingleQuoted
        };
    }

    // ---- Attribute value (quoted) ----------------------------------------

    fn step_attribute_value_quoted(&mut self, unit: InputUnit, double: bool) -> Step {
        let quote = if double { '"' } else { '\'' };
        let context = if double {
            HtmlTokenizerDiagnosticContext::AttributeValueDoubleQuoted
        } else {
            HtmlTokenizerDiagnosticContext::AttributeValueSingleQuoted
        };
        match unit {
            InputUnit::Scalar { ch, start, end } if ch == quote => {
                self.commit_quoted_attribute((start, end));
                self.state = State::AfterAttributeValueQuoted;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '&',
                start,
                end,
            } => {
                let trigger = self.discovery_trigger((start, end));
                self.unsupported_input_stop(
                    super::result::HtmlTokenizerCapability::CharacterReference {
                        context: super::result::HtmlCharacterReferenceContext::AttributeValue,
                    },
                    trigger,
                )
            }
            InputUnit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                match self.recover_null_character(context, (start, end), Some(self.tokens.len())) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                self.push_attribute_value_char('\u{fffd}', start, end)
            }
            InputUnit::Eof { at } => self.abandon_tag_at_eof(context, at),
            InputUnit::Scalar { ch, start, end } => self.push_attribute_value_char(ch, start, end),
        }
    }

    fn push_attribute_value_char(&mut self, ch: char, start: usize, end: usize) -> Step {
        if let Err(stop) = self.try_reserve_retained(ch.len_utf8(), start) {
            return stop;
        }
        let tag = self.tag.as_mut().expect("tag active");
        tag.pending_value
            .as_mut()
            .expect("pending attribute value active")
            .push(ch, end);
        Step::Continue
    }

    fn commit_quoted_attribute(&mut self, close_quote: (usize, usize)) {
        let tag = self.tag.as_mut().expect("tag active");
        let name = tag
            .pending_name
            .take()
            .expect("pending attribute name active");
        let value = tag
            .pending_value
            .take()
            .expect("pending attribute value active");
        let disposition = name
            .disposition
            .expect("disposition determined when leaving attribute-name state");
        let attribute = builder::finalize_valued_attribute(
            self.source,
            &name,
            &value,
            Some(close_quote),
            disposition,
        );
        self.push_attribute(attribute);
    }

    // ---- Attribute value (unquoted) --------------------------------------

    fn step_attribute_value_unquoted(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, start, end } if is_html_whitespace(ch) => {
                let _ = (start, end);
                self.commit_unquoted_attribute();
                self.state = State::BeforeAttributeName;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '&',
                start,
                end,
            } => {
                let trigger = self.discovery_trigger((start, end));
                self.unsupported_input_stop(
                    super::result::HtmlTokenizerCapability::CharacterReference {
                        context: super::result::HtmlCharacterReferenceContext::AttributeValue,
                    },
                    trigger,
                )
            }
            InputUnit::Scalar {
                ch: '>',
                start,
                end,
            } => {
                self.commit_unquoted_attribute();
                self.finish_tag((start, end))
            }
            InputUnit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                match self.recover_null_character(
                    HtmlTokenizerDiagnosticContext::AttributeValueUnquoted,
                    (start, end),
                    Some(self.tokens.len()),
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                self.push_attribute_value_char('\u{fffd}', start, end)
            }
            InputUnit::Scalar {
                ch: ch @ ('"' | '\'' | '<' | '=' | '`'),
                start,
                end,
            } => {
                match self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::UnexpectedCharacterInUnquotedAttributeValue,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::AttributeValueUnquoted,
                    HtmlTokenizerDiagnosticHandling::Continued,
                    HtmlTokenizerDiagnosticSubject::EmittedToken {
                        token_index: self.tokens.len(),
                    },
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                self.push_attribute_value_char(ch, start, end)
            }
            InputUnit::Eof { at } => {
                self.abandon_tag_at_eof(HtmlTokenizerDiagnosticContext::AttributeValueUnquoted, at)
            }
            InputUnit::Scalar { ch, start, end } => self.push_attribute_value_char(ch, start, end),
        }
    }

    fn commit_unquoted_attribute(&mut self) {
        let tag = self.tag.as_mut().expect("tag active");
        let name = tag
            .pending_name
            .take()
            .expect("pending attribute name active");
        let value = tag
            .pending_value
            .take()
            .expect("pending attribute value active");
        let disposition = name
            .disposition
            .expect("disposition determined when leaving attribute-name state");
        let attribute =
            builder::finalize_valued_attribute(self.source, &name, &value, None, disposition);
        self.push_attribute(attribute);
    }

    // ---- After attribute value (quoted) ----------------------------------

    fn step_after_attribute_value_quoted(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar { ch, .. } if is_html_whitespace(ch) => {
                self.state = State::BeforeAttributeName;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '/',
                start,
                end,
            } => {
                self.pending_solidus = Some((start, end));
                self.state = State::SelfClosingStartTag;
                Step::Continue
            }
            InputUnit::Scalar {
                ch: '>',
                start,
                end,
                ..
            } => self.finish_tag((start, end)),
            InputUnit::Eof { at } => self.abandon_tag_at_eof(
                HtmlTokenizerDiagnosticContext::AfterAttributeValueQuoted,
                at,
            ),
            InputUnit::Scalar { start, end, .. } => {
                match self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::MissingWhitespaceBetweenAttributes,
                    (start, end),
                    HtmlTokenizerDiagnosticContext::AfterAttributeValueQuoted,
                    HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::ReconsumedBeforeAttributeName,
                    ),
                    HtmlTokenizerDiagnosticSubject::EmittedToken {
                        token_index: self.tokens.len(),
                    },
                ) {
                    Ok(()) => {}
                    Err(stop) => return stop,
                }
                self.state = State::BeforeAttributeName;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    // ---- Self-closing start tag -------------------------------------------

    fn step_self_closing_start_tag(&mut self, unit: InputUnit) -> Step {
        match unit {
            InputUnit::Scalar {
                ch: '>',
                start,
                end,
                ..
            } => {
                let solidus = self.pending_solidus;
                let is_end_tag = matches!(
                    self.tag.as_ref().expect("tag active").kind,
                    HtmlTagKind::End
                );
                // `EndTagWithTrailingSolidus` is emission-conditioned
                // (#111): recording pending evidence on the tag builder is
                // infallible (it does not touch the Diagnostics resource
                // counter), so unlike the diagnostics this replaced, no
                // refusal path can leave `pending_solidus` half-consumed.
                if is_end_tag && let Some(span) = solidus {
                    let insertion_index = self.diagnostics.len();
                    self.tag
                        .as_mut()
                        .expect("tag active")
                        .pending_emission_diagnostics
                        .push(PendingEmissionDiagnostic {
                            code: HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
                            location: span,
                            context: HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
                            handling: HtmlTokenizerDiagnosticHandling::Recovered(
                                HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
                            ),
                            insertion_index,
                        });
                }
                self.pending_solidus = None;
                self.tag.as_mut().expect("tag active").self_closing_solidus = solidus;
                self.finish_tag((start, end))
            }
            InputUnit::Eof { at } => {
                self.abandon_tag_at_eof(HtmlTokenizerDiagnosticContext::SelfClosingStartTag, at)
            }
            InputUnit::Scalar { start, .. } => {
                let solidus = self.pending_solidus.expect("solidus recorded");
                if let Err(stop) = self.append_diagnostic(
                    HtmlTokenizerDiagnosticCode::UnexpectedSolidusInTag,
                    solidus,
                    HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
                    HtmlTokenizerDiagnosticHandling::Recovered(
                        HtmlTokenizerRecoveryKind::ReconsumedBeforeAttributeName,
                    ),
                    HtmlTokenizerDiagnosticSubject::EmittedToken {
                        token_index: self.tokens.len(),
                    },
                ) {
                    return stop;
                }
                self.pending_solidus = None;
                let _ = start;
                self.state = State::BeforeAttributeName;
                self.pending_reconsume = true;
                Step::Continue
            }
        }
    }

    // ---- Shared helpers ----------------------------------------------------

    fn finish_tag(&mut self, close_delimiter: (usize, usize)) -> Step {
        // Preflight before destroying the active tag: a refused emission
        // must leave the tag builder exactly as it was, not silently lost.
        // The run loop's generic `Step::Stop` handling then resolves the
        // still-active tag (provisional diagnostics, retained bytes).
        let token_bytes = match self
            .tag
            .as_ref()
            .expect("tag active")
            .retained_interpreted_bytes()
        {
            Some(bytes) => bytes,
            None => return counter_overflow_stop(),
        };
        // The resource-limit location is the current committed coverage
        // boundary, not a fixed offset derived from the close delimiter: a
        // diagnostic committed earlier in this same dispatch (for example
        // `MissingAttributeValue`'s `BeforeAttributeValue('>')` recovery)
        // may already have advanced `processed_end` to include the close
        // delimiter before this preflight ever runs, and the refusal must
        // report that already-committed boundary, not re-derive one.
        let at = (self.processed_end, self.processed_end);
        let committed = match self.preflight_token_emission(token_bytes, at) {
            Ok(committed) => committed,
            Err(stop) => return stop,
        };
        // The end-tag token's emission and any pending emission-conditioned
        // diagnostic evidence it carries are one compound operation: both
        // resources are preflighted here, before anything is mutated, so a
        // refusal cannot leave an emitted tag without its required
        // diagnostic or vice versa.
        let pending_count = self
            .tag
            .as_ref()
            .expect("tag active")
            .pending_emission_diagnostics
            .len();
        if pending_count > 0
            && let Err(stop) = self.preflight_pending_emission_diagnostics(pending_count, at)
        {
            return stop;
        }
        let mut tag = self.tag.take().expect("tag active");
        let pending_diagnostics = std::mem::take(&mut tag.pending_emission_diagnostics);
        let is_start_tag = matches!(tag.kind, HtmlTagKind::Start);
        let context_mode = if is_start_tag {
            context_dependent_mode(&tag.interpreted_name)
        } else {
            None
        };
        let finishing_text_mode_close = self.text_mode_closing_tag && !is_start_tag;
        let close_end = close_delimiter.1;
        let token = HtmlToken::Tag(builder::finalize_tag(self.source, tag, close_delimiter));
        self.commit_token(token, committed);
        if !pending_diagnostics.is_empty() {
            let token_index = self.tokens.len() - 1;
            self.materialize_pending_emission_diagnostics(pending_diagnostics, token_index);
        }
        self.state = State::Data;
        if finishing_text_mode_close {
            self.processed_end = close_end;
            return self.text_mode_close_yield(close_end);
        }
        match context_mode {
            Some(mode) => {
                self.processed_end = close_end;
                self.context_mode_stop(mode, close_end)
            }
            None => Step::Continue,
        }
    }

    /// Checks that `count` additional emission-conditioned diagnostics
    /// (pending evidence about to be materialized alongside a successful
    /// end-tag emission) fit within the `Diagnostics` limit, without
    /// mutating any state. `attempted` is the would-be total `Diagnostics`
    /// usage of the rejected atomic operation: one compound operation may
    /// require multiple pending diagnostics to become valid together, so
    /// `attempted` is `committed + count`, not a fixed single-unit
    /// increment. The ordinary one-pending-diagnostic case still naturally
    /// reports `attempted == committed + 1`.
    fn preflight_pending_emission_diagnostics(
        &self,
        count: usize,
        at: (usize, usize),
    ) -> Result<(), Step> {
        let required = checked_counter_add(self.diagnostics.len(), count)?;
        if required > self.limits.max_diagnostics() {
            return Err(self.resource_limit_stop(HtmlTokenizerResource::Diagnostics, required, at));
        }
        Ok(())
    }

    /// Converts pending emission-conditioned diagnostic evidence for a tag
    /// whose token just committed at `token_index` into real diagnostics,
    /// inserting each at its recorded chronological position rather than
    /// appending at the end: diagnostics committed after the evidence was
    /// recognized, with equal or later source starts, must not be
    /// reordered. Processed in reverse recognition order so repeated
    /// inserts at the same recorded index still land in original
    /// chronological order relative to each other.
    fn materialize_pending_emission_diagnostics(
        &mut self,
        pending: Vec<PendingEmissionDiagnostic>,
        token_index: usize,
    ) {
        for evidence in pending.into_iter().rev() {
            let anchor = self.anchor(evidence.location.0, evidence.location.1);
            let diagnostic = HtmlTokenizerDiagnostic::new(
                self.source,
                evidence.code,
                anchor,
                evidence.context,
                evidence.handling,
                HtmlTokenizerDiagnosticSubject::EmittedToken { token_index },
            )
            .expect("valid emission-conditioned diagnostic evidence");
            self.diagnostics
                .insert(evidence.insertion_index, diagnostic);
        }
    }

    fn context_mode_stop(
        &self,
        mode: super::result::HtmlTokenizerMode,
        boundary_at: usize,
    ) -> Step {
        let boundary = self.anchor(boundary_at, boundary_at);
        let unsupported = super::result::HtmlTokenizerUnsupportedCapability::new(
            self.source,
            super::result::HtmlTokenizerCapability::ContextDependentTokenizerMode { mode },
            super::result::HtmlTokenizerCapabilityAvailability::Deferred,
            super::result::HtmlTokenizerUnsupportedTrigger::EmittedToken {
                token_index: self.tokens.len() - 1,
                boundary,
            },
        )
        .expect("valid context-mode unsupported evidence");
        Step::Stop(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        ))
    }

    fn abandon_tag_at_eof(
        &mut self,
        context: HtmlTokenizerDiagnosticContext,
        eof_at: usize,
    ) -> Step {
        // The required `EofInTag` diagnostic is checked before the tag
        // builder is taken: a diagnostics-limit refusal must leave the tag
        // untouched (still active) rather than destroying it first. On
        // refusal, the run loop's generic `Step::Stop` handling resolves
        // the still-active tag instead.
        let tag_start = self.tag.as_ref().expect("tag active").tag_start;
        let region = self.anchor(tag_start, eof_at);
        if let Err(stop) = self.append_diagnostic_with_subject(
            HtmlTokenizerDiagnosticCode::EofInTag,
            (eof_at, eof_at),
            context,
            HtmlTokenizerDiagnosticHandling::Recovered(
                HtmlTokenizerRecoveryKind::AbandonedIncompleteTagAtEof,
            ),
            HtmlTokenizerDiagnosticSubject::AbandonedInput {
                region: region.clone(),
            },
        ) {
            return stop;
        }
        let tag = self.tag.take().expect("tag active");
        self.resolve_abandoned_tag_diagnostics(tag.diagnostics_start, &region);
        self.emit_eof(eof_at)
    }

    fn emit_eof(&mut self, at: usize) -> Step {
        let anchor = self.anchor(at, at);
        let token = HtmlToken::EndOfFile(
            HtmlEndOfFileToken::new(self.source, anchor).expect("valid eof token"),
        );
        match self.try_push_token(token, (at, at)) {
            Ok(()) => Step::Complete,
            Err(stop) => stop,
        }
    }

    /// Computes a contract-valid `Input` trigger span for a capability
    /// discovered while examining `natural` (typically the triggering
    /// unit's own span). An `Input` trigger's start must exactly equal the
    /// final coverage boundary: when a diagnostic already committed on this
    /// same unit has advanced `processed_end` past `natural`'s start, the
    /// trigger narrows to an empty marker at that boundary instead of
    /// re-claiming already-diagnosed territory.
    fn discovery_trigger(&self, natural: (usize, usize)) -> (usize, usize) {
        let start = self.processed_end;
        let end = natural.1.max(start);
        (start, end)
    }

    fn unsupported_input_stop(
        &mut self,
        capability: super::result::HtmlTokenizerCapability,
        trigger: (usize, usize),
    ) -> Step {
        // Discovering an unsupported capability is a deliberate stopping
        // point reached through normal successful transitions, not a forced
        // mid-operation refusal. Any active tag builder is released
        // generically by the run loop's `Step::Stop` handling (retargeting
        // its provisional diagnostics, then discarding the builder) rather
        // than silently cleared here. `data_run` is always already `None`
        // at every call site (Data-context discovery flushes it first;
        // attribute-value-context discovery has no active data run).
        let anchor = self.anchor(trigger.0, trigger.1);
        let unsupported = super::result::HtmlTokenizerUnsupportedCapability::new(
            self.source,
            capability,
            super::result::HtmlTokenizerCapabilityAvailability::Deferred,
            super::result::HtmlTokenizerUnsupportedTrigger::Input(anchor),
        )
        .expect("valid unsupported-capability evidence");
        Step::Stop(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        ))
    }

    fn anchor(&self, start: usize, end: usize) -> SourceAnchor {
        self.source
            .anchor(start, end)
            .expect("valid internal anchor")
    }

    fn commit_transition_step(&mut self, at: usize) -> Result<(), Step> {
        let attempted = checked_increment(self.transition_steps)?;
        if attempted > self.limits.max_transition_steps() {
            return Err(self.resource_limit_stop(
                HtmlTokenizerResource::TransitionSteps,
                attempted,
                (at, at),
            ));
        }
        self.transition_steps = attempted;
        Ok(())
    }

    /// Checks that one more token may be emitted, without mutating any
    /// state: neither the emitted-token count nor the committed
    /// retained-byte count are applied until [`Self::commit_token`] is
    /// called. Callers that must not destroy a pre-existing builder (the
    /// active Data run, the active tag) call this first and only then take
    /// ownership of that builder's content.
    fn preflight_token_emission(
        &self,
        token_bytes: usize,
        at: (usize, usize),
    ) -> Result<usize, Step> {
        let attempted = checked_increment(self.tokens.len())?;
        if attempted > self.limits.max_emitted_tokens() {
            return Err(self.resource_limit_stop(
                HtmlTokenizerResource::EmittedTokens,
                attempted,
                at,
            ));
        }
        checked_counter_add(self.committed_retained_bytes, token_bytes)
    }

    /// Applies a token emission already validated by
    /// [`Self::preflight_token_emission`]. Infallible: the checked
    /// `committed_retained_bytes` value is supplied by the caller's
    /// preflight, not recomputed.
    fn commit_token(&mut self, token: HtmlToken, committed_retained_bytes: usize) {
        self.committed_retained_bytes = committed_retained_bytes;
        self.min_processed_end = self.min_processed_end.max(token_span_end(&token));
        self.tokens.push(token);
    }

    fn try_push_token(&mut self, token: HtmlToken, at: (usize, usize)) -> Result<(), Step> {
        let token_bytes =
            checked_token_interpreted_bytes(&token).ok_or_else(counter_overflow_stop)?;
        let committed = self.preflight_token_emission(token_bytes, at)?;
        self.commit_token(token, committed);
        Ok(())
    }

    fn append_diagnostic(
        &mut self,
        code: HtmlTokenizerDiagnosticCode,
        at: (usize, usize),
        context: HtmlTokenizerDiagnosticContext,
        handling: HtmlTokenizerDiagnosticHandling,
        subject: HtmlTokenizerDiagnosticSubject,
    ) -> Result<(), Step> {
        self.append_diagnostic_with_subject(code, at, context, handling, subject)
    }

    fn append_diagnostic_with_subject(
        &mut self,
        code: HtmlTokenizerDiagnosticCode,
        at: (usize, usize),
        context: HtmlTokenizerDiagnosticContext,
        handling: HtmlTokenizerDiagnosticHandling,
        subject: HtmlTokenizerDiagnosticSubject,
    ) -> Result<(), Step> {
        let attempted = checked_increment(self.diagnostics.len())?;
        if attempted > self.limits.max_diagnostics() {
            return Err(self.resource_limit_stop(
                HtmlTokenizerResource::Diagnostics,
                attempted,
                at,
            ));
        }
        let anchor = self.anchor(at.0, at.1);
        let diagnostic =
            HtmlTokenizerDiagnostic::new(self.source, code, anchor, context, handling, subject)
                .expect("valid diagnostic evidence");
        self.diagnostics.push(diagnostic);
        // A successfully recorded diagnostic commits its own location to the
        // processed prefix: the position has been observed and explained,
        // even when the input unit it describes is later reconsumed into a
        // dispatch that discovers a different unsupported capability.
        self.processed_end = self.processed_end.max(at.1);
        self.min_processed_end = self.min_processed_end.max(at.1);
        Ok(())
    }

    fn append_preprocessing_diagnostic(
        &mut self,
        code: HtmlTokenizerDiagnosticCode,
        unit: InputUnit,
    ) -> Result<(), Step> {
        self.append_diagnostic_with_subject(
            code,
            (unit.start(), unit.end()),
            HtmlTokenizerDiagnosticContext::InputPreprocessing,
            HtmlTokenizerDiagnosticHandling::Continued,
            HtmlTokenizerDiagnosticSubject::InputLocation,
        )
    }

    fn append_null_diagnostic(
        &mut self,
        context: HtmlTokenizerDiagnosticContext,
        at: (usize, usize),
        emitted_token_index: Option<usize>,
    ) -> Result<(), Step> {
        let subject = match emitted_token_index {
            Some(token_index) => HtmlTokenizerDiagnosticSubject::EmittedToken { token_index },
            None => HtmlTokenizerDiagnosticSubject::InputLocation,
        };
        self.append_diagnostic_with_subject(
            HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter,
            at,
            context,
            HtmlTokenizerDiagnosticHandling::Recovered(
                HtmlTokenizerRecoveryKind::ReplacedNullWithReplacementCharacter,
            ),
            subject,
        )
    }

    /// Reserves the replacement character's retained-byte cost and appends
    /// `UnexpectedNullCharacter` with `Recovered`, in that order, so the
    /// diagnostic never claims the replacement happened before its resource
    /// cost is guaranteed available. Callers still perform the actual
    /// replacement push afterward (its own internal reservation then
    /// trivially re-succeeds, since nothing else can commit in between).
    fn recover_null_character(
        &mut self,
        context: HtmlTokenizerDiagnosticContext,
        at: (usize, usize),
        emitted_token_index: Option<usize>,
    ) -> Result<(), Step> {
        self.try_reserve_retained('\u{fffd}'.len_utf8(), at.0)?;
        self.append_null_diagnostic(context, at, emitted_token_index)
    }

    fn try_start_attribute(&mut self, at: usize) -> Result<(), Step> {
        let attempted = checked_increment(self.tag.as_ref().expect("tag active").attributes.len())?;
        if attempted > self.limits.max_attributes_per_tag() {
            return Err(self.resource_limit_stop(
                HtmlTokenizerResource::AttributesPerTag,
                attempted,
                (at, at),
            ));
        }
        Ok(())
    }

    fn try_reserve_retained(&mut self, delta: usize, at: usize) -> Result<(), Step> {
        let active = self
            .active_retained_bytes()
            .ok_or_else(counter_overflow_stop)?;
        let attempted = checked_counter_add(
            checked_counter_add(self.committed_retained_bytes, active)?,
            delta,
        )?;
        if attempted > self.limits.max_retained_interpreted_bytes() {
            return Err(self.resource_limit_stop(
                HtmlTokenizerResource::RetainedInterpretedBytes,
                attempted,
                (at, at),
            ));
        }
        Ok(())
    }

    /// Interpreted bytes currently retained by the active (not yet
    /// committed) character run and tag builder. Returns `None` on checked
    /// accumulation overflow rather than panicking or wrapping.
    fn active_retained_bytes(&self) -> Option<usize> {
        let data = self
            .data_run
            .as_ref()
            .map_or(0, |run| run.interpreted.len());
        let tag = match &self.tag {
            Some(tag) => tag.retained_interpreted_bytes()?,
            None => 0,
        };
        data.checked_add(tag)
    }

    fn resource_limit_stop(
        &self,
        resource: HtmlTokenizerResource,
        attempted: usize,
        at: (usize, usize),
    ) -> Step {
        let limit = self.limits.limit_for(resource);
        let anchor = self.anchor(at.0, at.1);
        let evidence =
            HtmlTokenizerResourceLimit::new(self.source, resource, limit, attempted, anchor)
                .expect("valid resource limit evidence");
        Step::Stop(HtmlTokenizerIncompleteCause::ResourceLimit(evidence))
    }
}

fn is_html_whitespace(ch: char) -> bool {
    matches!(ch, '\t' | '\n' | '\u{000c}' | ' ')
}

/// Maps an interpreted start-tag name to the context-dependent tokenizer
/// mode it would require, per the established unsupported boundary. TC-S9
/// implements the selected coordinated `<style>` RAWTEXT path while the batch
/// entry point preserves this map as deferred evidence for every member.
fn context_dependent_mode(interpreted_name: &str) -> Option<super::result::HtmlTokenizerMode> {
    use super::result::HtmlTokenizerMode;
    match interpreted_name {
        "title" | "textarea" => Some(HtmlTokenizerMode::Rcdata),
        "style" | "xmp" | "iframe" | "noembed" | "noframes" => Some(HtmlTokenizerMode::RawText),
        "script" => Some(HtmlTokenizerMode::ScriptData),
        "noscript" => Some(HtmlTokenizerMode::Noscript),
        "plaintext" => Some(HtmlTokenizerMode::Plaintext),
        _ => None,
    }
}

/// Constructs the typed `Step::Stop` for an internal invariant the engine
/// itself guarantees, so a violation stops honestly instead of panicking.
///
/// Reserved for conditions no authored input can produce: TC-S10 uses it only
/// where the authoritative cursor would have to disagree with a maximum match
/// already verified byte-for-byte against the very same source.
fn internal_invariant_stop(failure: HtmlTokenizerInvariantFailure) -> Step {
    Step::Stop(HtmlTokenizerIncompleteCause::InternalInvariantFailure(
        failure,
    ))
}

/// Constructs the typed `Step::Stop` for a checked counter/resource
/// accounting addition that would overflow `usize`, per the #111
/// `HtmlTokenizerInvariantFailure::CounterOverflow` contract.
fn counter_overflow_stop() -> Step {
    Step::Stop(HtmlTokenizerIncompleteCause::InternalInvariantFailure(
        HtmlTokenizerInvariantFailure::CounterOverflow,
    ))
}

/// Checked `value + 1` for a logical resource/counter attempted count.
/// Returns the typed `CounterOverflow` stop rather than panicking or
/// wrapping when `value` is already `usize::MAX`.
fn checked_increment(value: usize) -> Result<usize, Step> {
    checked_counter_add(value, 1)
}

/// Checked `base + delta` for logical resource/counter accounting. Returns
/// the typed `CounterOverflow` stop rather than panicking or wrapping.
fn checked_counter_add(base: usize, delta: usize) -> Result<usize, Step> {
    base.checked_add(delta).ok_or_else(counter_overflow_stop)
}

/// Checked per-token interpreted-byte aggregation for the
/// `RetainedInterpretedBytes` accounting performed on commit. Returns
/// `None` on checked accumulation overflow rather than panicking or
/// wrapping.
fn checked_token_interpreted_bytes(token: &HtmlToken) -> Option<usize> {
    match token {
        HtmlToken::Character(character) => Some(character.interpreted().len()),
        HtmlToken::Tag(tag) => {
            let mut total = tag.name().interpreted().len();
            for attribute in tag.attributes() {
                total = total.checked_add(attribute.name().interpreted().len())?;
                total = total.checked_add(attribute.interpreted_value().len())?;
            }
            Some(total)
        }
        HtmlToken::EndOfFile(_) => Some(0),
    }
}

fn token_span_end(token: &HtmlToken) -> usize {
    match token {
        HtmlToken::Character(character) => character.source().range().end(),
        HtmlToken::Tag(tag) => tag.complete().range().end(),
        HtmlToken::EndOfFile(eof) => eof.source().range().end(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Proves the checked counter-accounting helper maps `usize::MAX`
    /// overflow to the typed `CounterOverflow` invariant failure rather than
    /// panicking (debug builds) or silently wrapping (release builds), per
    /// the #111 contract. No large allocation is used or required: the
    /// helper operates on plain `usize` counters.
    #[test]
    fn checked_counter_add_reports_counter_overflow_at_usize_max() {
        let result = checked_counter_add(usize::MAX, 1);
        assert!(matches!(
            result,
            Err(Step::Stop(
                HtmlTokenizerIncompleteCause::InternalInvariantFailure(
                    HtmlTokenizerInvariantFailure::CounterOverflow
                )
            ))
        ));
    }

    #[test]
    fn checked_counter_add_reports_counter_overflow_near_usize_max() {
        let result = checked_counter_add(usize::MAX - 1, 5);
        assert!(matches!(
            result,
            Err(Step::Stop(
                HtmlTokenizerIncompleteCause::InternalInvariantFailure(
                    HtmlTokenizerInvariantFailure::CounterOverflow
                )
            ))
        ));
    }

    #[test]
    fn checked_counter_add_succeeds_below_the_boundary() {
        assert!(matches!(checked_counter_add(41, 1), Ok(42)));
        assert!(matches!(
            checked_counter_add(usize::MAX - 1, 1),
            Ok(value) if value == usize::MAX
        ));
    }

    #[test]
    fn checked_increment_reports_counter_overflow_at_usize_max() {
        let result = checked_increment(usize::MAX);
        assert!(matches!(
            result,
            Err(Step::Stop(
                HtmlTokenizerIncompleteCause::InternalInvariantFailure(
                    HtmlTokenizerInvariantFailure::CounterOverflow
                )
            ))
        ));
    }

    #[test]
    fn checked_increment_succeeds_below_usize_max() {
        assert!(matches!(checked_increment(41), Ok(42)));
    }
}
