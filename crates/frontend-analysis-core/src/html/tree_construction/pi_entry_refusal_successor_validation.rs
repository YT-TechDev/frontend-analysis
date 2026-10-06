//! Candidate-independent validation for Issue #910.
//!
//! This module validates the corrected bounded semantics of Processing
//! Instruction (PI) **entry refusal** before any production placement or
//! implementation:
//!
//! ```text
//! authored Data-context "<?" opener
//!     -> recognize PI entry through the forward tokenizer lifecycle
//!     -> preserve already-committed preceding evidence
//!     -> no PI-owned syntax diagnostic, no PI token, no comment, no node,
//!        no interpreted PI output
//!     -> remain incomplete as Deferred ProcessingInstruction
//!     -> stop before observing PI target / data / EOF grammar
//! ```
//!
//! It is **test-only** and changes no production behavior. It does not
//! authorize a production correction, and it is not an independent
//! acceptance of itself. Capability support is unchanged: the correction
//! concerns diagnostic truth and refusal ownership, never PI support.
//!
//! # Independent oracle boundary
//!
//! The model and every expected value below import nothing from the
//! production HTML tokenizer, producer, session, result, corpus, tree
//! construction, report, or CLI. Expected values are hand-authored from the
//! pinned WHATWG text and the accepted result/resource contracts. They are
//! never produced by running production, never derived from `ERR-006`,
//! `UNSUP-004`, or their same-dispatch test, never reconstructed by rescanning
//! source, and never inferred from decoded lengths. The only crate items used
//! by the model are the `SourceText` / `SourceId` anchoring primitives, so
//! authored evidence ranges are real byte ranges.
//!
//! The single exception is [`live_vocabulary_witness`]: it names two live
//! enum variants (`ProcessingInstruction`, `Deferred`) so that "the exact live
//! typed equivalent" cannot silently drift into generic text. It reads no
//! behavior.
//!
//! # Pins (both inspected as immutable sources; blob ids re-derived by hashing
//! # the fetched `source` files while authoring this leaf)
//!
//! - Current: WHATWG HTML `ba3400bb4f3d86477a5c8cfc07ff91fc0caaf21c`, source
//!   blob `7875de93bb63ebd6c6aa16437147b9b7f6a51283`.
//! - Original #109 tokenizer pin: `24c5e48bf66ea61bc199ec6338c81258275ba9c6`,
//!   source blob `90c6259281b1e457f53c2bf01e85aba7a27c0d19`.
//!
//! The Data state, the Tag open state, and the five Processing Instruction
//! states (open, target, after target, data, questionable) were compared
//! between the two pins after whitespace normalization and are identical.
//!
//! # Pinned obligations the gold is derived from
//!
//! - **Data**, U+003C: switch to Tag open. Nothing is emitted by `<` itself.
//! - **Tag open**, U+003F (`?`): *set the temporary buffer to the empty string
//!   and switch to the processing instruction open state.* There is **no**
//!   parse error on this transition. The previously reported
//!   `unexpected-question-mark-instead-of-tag-name` is absent from both pins.
//! - Tag open `!`, `/`, ASCII alpha, EOF, and "anything else" are different
//!   branches. Of these, only "anything else" is modelled (it is the
//!   production-reachable source of an earlier genuine diagnostic):
//!   `invalid-first-character-of-tag-name`, emit `<`, reconsume in Data.
//! - **PI open / target / after target / data / questionable** own every
//!   later PI parse error (`eof-in-processing-instruction`,
//!   `invalid-first-character-of-processing-instruction-target`,
//!   `invalid-processing-instruction-target`,
//!   `disallowed-processing-instruction-target`) and some paths convert the
//!   buffer to a bogus comment. A bounded profile that refuses *before*
//!   entering those states cannot claim any of them, and cannot claim a
//!   comment either. This model never enters them.
//!
//! "Unsupported" is not a parse error: it records that the profile did not
//! examine further input, not that the source is invalid HTML.
//!
//! # Coverage / trigger policy under validation
//!
//! This is project bounded-profile policy, not a WHATWG rule. Two geometries
//! are compared executably in [`policy_challenge`]:
//!
//! - **Selected (floor-bounded full opener)**: the processed prefix ends
//!   before the opening `<` (never below the end of committed evidence); the
//!   unsupported trigger is the exact cursor-owned opener `<?`, narrowed only
//!   by any prefix of it already owned by committed evidence.
//! - **Alternative (question unit)**: the processed prefix includes `<`; the
//!   trigger is `?` alone.
//!
//! The accepted result contract does not discriminate (both satisfy it); the
//! selection rests on: (1) owned recognition - the trigger plus committed
//! evidence tile the recognized opener without consumer reconstruction, while
//! the alternative leaves `<` processed with no owner; (2) `<` is provisional
//! until Tag open classifies it, so no committed evidence is rolled back; (3)
//! the accepted `<!` MarkupDeclaration unsupported boundary (`UNSUP-003`) has
//! exactly this geometry; (4) the empty-marker trigger used by the historical
//! PI shape exists only to avoid "re-claiming already-diagnosed territory",
//! and that reason disappears once no diagnostic is committed.
//!
//! ## Finding: the full-opener policy needs a committed-evidence floor
//!
//! For `<<?`, the second `<` is the offending unit of a committed
//! `invalid-first-character-of-tag-name` diagnostic *and* the first byte of
//! the `<?` opener. The accepted contract requires diagnostic locations to lie
//! inside the processed prefix and an `Input` trigger to start exactly at the
//! prefix end, so the trigger for this input cannot be the whole `<?`: it is
//! narrowed to the unowned remainder `?`. No committed evidence is rolled back
//! and nothing is reconstructed; the diagnostic and trigger together tile the
//! opener. The policy is therefore stated and validated as
//! `prefix = max(opener_start, end of committed evidence)`. Maintainers
//! should treat "exact `<?` opener" as this floor-bounded rule.
//!
//! # Deliberately bounded model
//!
//! The model is a Data / Tag open recognizer, not a second tokenizer. It
//! models Data character runs, `<`, NUL, control-character preprocessing
//! diagnostics, a leading BOM, and Tag open dispatch. Every other branch
//! (`&`, CR, noncharacters, end/start tags, `!`, EOF) is an explicit
//! [`Branch`] boundary outside the theorem. It models no PI grammar, comment,
//! bogus comment, MarkupDeclaration, tree, report, or CLI behavior, and it is
//! not a proposed production state layout or API.
//!
//! # Historical contradiction (preserved, never edited here)
//!
//! Current production, `ERR-006`, `UNSUP-004`, their same-dispatch test, and
//! the tokenizer validation documentation share the falsified premise. They
//! remain untouched by this leaf. [`SUPERSESSION_PRESSURE`] records what a
//! later accepted successor must reconcile; historical ids must not be reused
//! for a different theorem.

use crate::{SourceId, SourceText};

const WHATWG_CURRENT_COMMIT: &str = "ba3400bb4f3d86477a5c8cfc07ff91fc0caaf21c";
const WHATWG_CURRENT_BLOB: &str = "7875de93bb63ebd6c6aa16437147b9b7f6a51283";
const WHATWG_109_COMMIT: &str = "24c5e48bf66ea61bc199ec6338c81258275ba9c6";
const WHATWG_109_BLOB: &str = "90c6259281b1e457f53c2bf01e85aba7a27c0d19";

/// What a later accepted successor must reconcile. Recorded, not acted on.
const SUPERSESSION_PRESSURE: &[(&str, &str)] = &[
    (
        "producer TagOpen('?') diagnostic",
        "production defect; correct only after successor authority",
    ),
    (
        "ERR-006",
        "contradicted theorem; explicit supersession, retirement, or reclassification",
    ),
    (
        "UNSUP-004",
        "current correspondence to replace after an accepted successor",
    ),
    (
        "same-dispatch ERR-006/UNSUP-004 test",
        "future correspondence maintenance",
    ),
    (
        "tokenizer validation docs",
        "claim Standard authority for the question-mark error; focused correction",
    ),
    (
        "#109 / #112 discussion",
        "history; the mistaken premise is superseded later, not rewritten",
    ),
];

// ---------------------------------------------------------------------------
// Authored evidence and vocabulary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evidence {
    source_id: SourceId,
    start: usize,
    end: usize,
    raw: String,
}

fn evidence(source: &SourceText, start: usize, end: usize) -> Evidence {
    let anchor = source.anchor(start, end).expect("authored byte range");
    Evidence {
        source_id: anchor.source_id(),
        start: anchor.range().start(),
        end: anchor.range().end(),
        raw: anchor.fragment().to_owned(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Resource {
    SourceBytes,
    TransitionSteps,
    EmittedTokens,
    Diagnostics,
    AttributesPerTag,
    RetainedInterpretedBytes,
    TemporaryBufferBytes,
}

impl Resource {
    /// The fixed, unchanged seven-dimension vector. No dimension is added.
    const ALL: [Self; 7] = [
        Self::SourceBytes,
        Self::TransitionSteps,
        Self::EmittedTokens,
        Self::Diagnostics,
        Self::AttributesPerTag,
        Self::RetainedInterpretedBytes,
        Self::TemporaryBufferBytes,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Limits {
    source_bytes: usize,
    transition_steps: usize,
    emitted_tokens: usize,
    diagnostics: usize,
    attributes_per_tag: usize,
    retained_interpreted_bytes: usize,
    temporary_buffer_bytes: usize,
}

impl Limits {
    const fn generous() -> Self {
        Self {
            source_bytes: 4_096,
            transition_steps: 4_096,
            emitted_tokens: 4_096,
            diagnostics: 256,
            attributes_per_tag: 256,
            retained_interpreted_bytes: 16_384,
            temporary_buffer_bytes: 4_096,
        }
    }

    fn limit_for(self, resource: Resource) -> usize {
        match resource {
            Resource::SourceBytes => self.source_bytes,
            Resource::TransitionSteps => self.transition_steps,
            Resource::EmittedTokens => self.emitted_tokens,
            Resource::Diagnostics => self.diagnostics,
            Resource::AttributesPerTag => self.attributes_per_tag,
            Resource::RetainedInterpretedBytes => self.retained_interpreted_bytes,
            Resource::TemporaryBufferBytes => self.temporary_buffer_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Usage {
    source_bytes: usize,
    transition_steps: usize,
    emitted_tokens: usize,
    diagnostics: usize,
    peak_attributes_per_tag: usize,
    retained_interpreted_bytes: usize,
    peak_temporary_buffer_bytes: usize,
}

impl Usage {
    fn value_of(self, resource: Resource) -> usize {
        match resource {
            Resource::SourceBytes => self.source_bytes,
            Resource::TransitionSteps => self.transition_steps,
            Resource::EmittedTokens => self.emitted_tokens,
            Resource::Diagnostics => self.diagnostics,
            Resource::AttributesPerTag => self.peak_attributes_per_tag,
            Resource::RetainedInterpretedBytes => self.retained_interpreted_bytes,
            Resource::TemporaryBufferBytes => self.peak_temporary_buffer_bytes,
        }
    }
}

/// Hand-authored usage: attribute and temporary-buffer peaks are zero.
const fn usage(
    source_bytes: usize,
    transition_steps: usize,
    emitted_tokens: usize,
    diagnostics: usize,
    retained_interpreted_bytes: usize,
) -> Usage {
    Usage {
        source_bytes,
        transition_steps,
        emitted_tokens,
        diagnostics,
        peak_attributes_per_tag: 0,
        retained_interpreted_bytes,
        peak_temporary_buffer_bytes: 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagCode {
    // Modelled, genuinely earlier evidence.
    ControlCharacterInInputStream,
    UnexpectedNullCharacter,
    InvalidFirstCharacterOfTagName,
    // Vocabulary the corrected PI entry must never claim. The correct model
    // never constructs these; wrong models do.
    UnexpectedQuestionMarkInsteadOfTagName,
    EofInProcessingInstruction,
    InvalidFirstCharacterOfProcessingInstructionTarget,
    InvalidProcessingInstructionTarget,
    DisallowedProcessingInstructionTarget,
    UnsupportedReportedAsParseError,
}

impl DiagCode {
    /// Diagnostics a PI-entry refusal must not own.
    fn is_pi_owned(self) -> bool {
        matches!(
            self,
            Self::UnexpectedQuestionMarkInsteadOfTagName
                | Self::EofInProcessingInstruction
                | Self::InvalidFirstCharacterOfProcessingInstructionTarget
                | Self::InvalidProcessingInstructionTarget
                | Self::DisallowedProcessingInstructionTarget
                | Self::UnsupportedReportedAsParseError
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagContext {
    InputPreprocessing,
    Data,
    TagOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Handling {
    Continued,
    RecoveredLiteralMarkupPrefix,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diag {
    code: DiagCode,
    at: Evidence,
    context: DiagContext,
    handling: Handling,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Character { at: Evidence, interpreted: String },
    // Never produced by the correct model; wrong models only.
    ProcessingInstruction { at: Evidence, target: String },
    Comment { at: Evidence },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Capability {
    ProcessingInstruction,
    MarkupDeclaration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Availability {
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigFailure {
    ZeroTransitionStepLimit,
    ZeroEmittedTokenLimit,
}

/// A branch this leaf deliberately does not model. Reaching one is not a
/// claim about that branch's output; it proves only that the PI theorem was
/// not widened onto it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Branch {
    CharacterReference,
    CarriageReturn,
    Noncharacter,
    EndTagOpen,
    StartTagName,
    MarkupDeclaration,
    EofAfterLessThan,
    EofWithoutOpener,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Completion {
    Unsupported {
        capability: Capability,
        availability: Availability,
        trigger: Evidence,
    },
    ResourceLimit {
        resource: Resource,
        limit: usize,
        attempted: usize,
        /// `None`: this leaf makes no claim about where a run-flush refusal
        /// is anchored; it is not part of the theorem.
        at: Option<(usize, usize)>,
    },
    InvalidConfiguration(ConfigFailure),
    Outside(Branch),
    /// Never produced. Present only so an upgrade to complete is expressible
    /// and rejectable.
    Complete,
}

/// Cursor-recorded recognition of the authored `<?` opener, kept by the model
/// as a witness. It is not a proposed production field.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Recognition {
    opener: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    source_id: SourceId,
    source_len: usize,
    bom_end: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diag>,
    processed_end: usize,
    completion: Completion,
    usage: Usage,
    recognition: Option<Recognition>,
}

// ---------------------------------------------------------------------------
// The bounded Data / Tag open recognizer
// ---------------------------------------------------------------------------

/// Selected behavior plus deliberately wrong behaviors used as executable
/// falsifiers. Only `Selected` is the model under validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dialect {
    Selected,
    QuestionUnit,
    HistoricalDiagnostic,
    EmptyTrigger,
    AttemptDiagnostic,
    AttemptToken,
    AttemptRetained,
    AttemptTemporaryBuffer,
    ParseTarget,
    WidenBang,
}

#[derive(Debug, Clone, Copy)]
enum Unit {
    Scalar { ch: char, start: usize, end: usize },
    Eof { at: usize },
}

impl Unit {
    fn start(self) -> usize {
        match self {
            Self::Scalar { start, .. } => start,
            Self::Eof { at } => at,
        }
    }

    fn end(self) -> usize {
        match self {
            Self::Scalar { end, .. } => end,
            Self::Eof { at } => at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Data,
    TagOpen,
}

enum Flow {
    Continue,
    Reconsume,
}

struct Run {
    start: usize,
    end: usize,
    text: String,
}

struct Machine<'a> {
    source: &'a SourceText,
    limits: Limits,
    dialect: Dialect,
    state: State,
    tokens: Vec<Token>,
    diagnostics: Vec<Diag>,
    run: Option<Run>,
    steps: usize,
    processed_end: usize,
    bom_end: usize,
    tag_open_start: usize,
    lt_end: usize,
    recognition: Option<Recognition>,
}

fn is_control(ch: char) -> bool {
    matches!(ch, '\u{1}'..='\u{8}' | '\u{b}' | '\u{e}'..='\u{1f}' | '\u{7f}'..='\u{9f}')
}

fn is_noncharacter(ch: char) -> bool {
    let value = ch as u32;
    (0xfdd0..=0xfdef).contains(&value) || (value & 0xfffe) == 0xfffe
}

fn run(source: &SourceText, limits: Limits) -> Observation {
    run_dialect(source, limits, Dialect::Selected)
}

fn run_dialect(source: &SourceText, limits: Limits, dialect: Dialect) -> Observation {
    Machine {
        source,
        limits,
        dialect,
        state: State::Data,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
        run: None,
        steps: 0,
        processed_end: 0,
        bom_end: 0,
        tag_open_start: 0,
        lt_end: 0,
        recognition: None,
    }
    .drive()
}

impl Machine<'_> {
    fn text(&self) -> &str {
        self.source.as_str()
    }

    fn drive(mut self) -> Observation {
        let len = self.text().len();
        // Configuration and source-size preflight precede any processing.
        if self.limits.transition_steps == 0 {
            return self.finish(Completion::InvalidConfiguration(
                ConfigFailure::ZeroTransitionStepLimit,
            ));
        }
        if self.limits.emitted_tokens == 0 {
            return self.finish(Completion::InvalidConfiguration(
                ConfigFailure::ZeroEmittedTokenLimit,
            ));
        }
        if len > self.limits.source_bytes {
            let limit = self.limits.source_bytes;
            return self.finish(Completion::ResourceLimit {
                resource: Resource::SourceBytes,
                limit,
                attempted: len,
                at: Some((0, 0)),
            });
        }
        let mut position = 0;
        if self.text().starts_with('\u{feff}') {
            self.bom_end = '\u{feff}'.len_utf8();
            self.processed_end = self.bom_end;
            position = self.bom_end;
        }
        let mut pending: Option<Unit> = None;
        let completion = loop {
            let (unit, fresh) = match pending.take() {
                Some(unit) => (unit, false),
                None => match self.text()[position..].chars().next() {
                    Some(ch) => {
                        let unit = Unit::Scalar {
                            ch,
                            start: position,
                            end: position + ch.len_utf8(),
                        };
                        position = unit.end();
                        (unit, true)
                    }
                    None => (Unit::Eof { at: len }, true),
                },
            };
            match self.step(unit, fresh) {
                Ok(Flow::Continue) => {
                    self.processed_end = self.processed_end.max(unit.end());
                }
                Ok(Flow::Reconsume) => pending = Some(unit),
                Err(completion) => break completion,
            }
        };
        // A terminal stop first tries to close a pending Data run; a refused
        // close leaves it pending and is not retried.
        if self.run.is_some() {
            let _ = self.flush_run();
        }
        self.finish(completion)
    }

    fn finish(self, completion: Completion) -> Observation {
        let retained = self.retained();
        let source_len = self.source.as_str().len();
        Observation {
            source_id: self.source.id(),
            source_len,
            bom_end: self.bom_end,
            usage: Usage {
                source_bytes: source_len,
                transition_steps: self.steps,
                emitted_tokens: self.tokens.len(),
                diagnostics: self.diagnostics.len(),
                peak_attributes_per_tag: 0,
                retained_interpreted_bytes: retained,
                peak_temporary_buffer_bytes: 0,
            },
            tokens: self.tokens,
            diagnostics: self.diagnostics,
            processed_end: self.processed_end,
            completion,
            recognition: self.recognition,
        }
    }

    /// Bytes owned by emitted tokens plus the pending Data run.
    fn retained(&self) -> usize {
        let emitted: usize = self
            .tokens
            .iter()
            .map(|token| match token {
                Token::Character { interpreted, .. } => interpreted.len(),
                Token::ProcessingInstruction { target, .. } => target.len(),
                Token::Comment { .. } => 0,
            })
            .sum();
        emitted + self.run.as_ref().map_or(0, |run| run.text.len())
    }

    /// The end of all committed evidence: the lower bound a terminal coverage
    /// boundary may never cross.
    fn floor(&self) -> usize {
        let tokens = self.tokens.iter().map(|token| match token {
            Token::Character { at, .. }
            | Token::ProcessingInstruction { at, .. }
            | Token::Comment { at } => at.end,
        });
        let diagnostics = self.diagnostics.iter().map(|diag| diag.at.end);
        tokens.chain(diagnostics).fold(self.bom_end, usize::max)
    }

    fn append_diag(
        &mut self,
        code: DiagCode,
        range: (usize, usize),
        context: DiagContext,
        handling: Handling,
    ) -> Result<(), Completion> {
        let attempted = self.diagnostics.len() + 1;
        if attempted > self.limits.diagnostics {
            return Err(Completion::ResourceLimit {
                resource: Resource::Diagnostics,
                limit: self.limits.diagnostics,
                attempted,
                at: Some(range),
            });
        }
        let at = evidence(self.source, range.0, range.1);
        self.diagnostics.push(Diag {
            code,
            at,
            context,
            handling,
        });
        // A committed diagnostic commits its location to the processed prefix.
        self.processed_end = self.processed_end.max(range.1);
        Ok(())
    }

    fn reserve_retained(&mut self, delta: usize, at: usize) -> Result<(), Completion> {
        let attempted = self.retained() + delta;
        if attempted > self.limits.retained_interpreted_bytes {
            return Err(Completion::ResourceLimit {
                resource: Resource::RetainedInterpretedBytes,
                limit: self.limits.retained_interpreted_bytes,
                attempted,
                at: Some((at, at)),
            });
        }
        Ok(())
    }

    fn preflight_token(&self, at: Option<(usize, usize)>) -> Result<(), Completion> {
        let attempted = self.tokens.len() + 1;
        if attempted > self.limits.emitted_tokens {
            return Err(Completion::ResourceLimit {
                resource: Resource::EmittedTokens,
                limit: self.limits.emitted_tokens,
                attempted,
                at,
            });
        }
        Ok(())
    }

    fn flush_run(&mut self) -> Result<(), Completion> {
        if self.run.is_none() {
            return Ok(());
        }
        // Preflight before destroying the pending run. Where a refused close
        // is anchored is not part of this theorem.
        self.preflight_token(None)?;
        let run = self.run.take().expect("pending run");
        let at = evidence(self.source, run.start, run.end);
        self.tokens.push(Token::Character {
            at,
            interpreted: run.text,
        });
        Ok(())
    }

    fn step(&mut self, unit: Unit, fresh: bool) -> Result<Flow, Completion> {
        if let (true, Unit::Scalar { ch, start, end }) = (fresh, unit) {
            if ch == '\r' {
                return Err(Completion::Outside(Branch::CarriageReturn));
            }
            if is_noncharacter(ch) {
                return Err(Completion::Outside(Branch::Noncharacter));
            }
            if is_control(ch) {
                // Preprocessing diagnostics precede, and cost no, transition.
                self.append_diag(
                    DiagCode::ControlCharacterInInputStream,
                    (start, end),
                    DiagContext::InputPreprocessing,
                    Handling::Continued,
                )?;
            }
        }
        // One transition per attempted dispatch, including a reconsume.
        let attempted = self.steps + 1;
        if attempted > self.limits.transition_steps {
            return Err(Completion::ResourceLimit {
                resource: Resource::TransitionSteps,
                limit: self.limits.transition_steps,
                attempted,
                at: Some((unit.start(), unit.start())),
            });
        }
        self.steps = attempted;
        match self.state {
            State::Data => self.data(unit),
            State::TagOpen => self.tag_open(unit),
        }
    }

    fn data(&mut self, unit: Unit) -> Result<Flow, Completion> {
        match unit {
            Unit::Eof { .. } => {
                self.flush_run()?;
                Err(Completion::Outside(Branch::EofWithoutOpener))
            }
            Unit::Scalar {
                ch: '<',
                start,
                end,
            } => {
                self.flush_run()?;
                self.tag_open_start = start;
                self.lt_end = end;
                self.state = State::TagOpen;
                Ok(Flow::Continue)
            }
            Unit::Scalar { ch: '&', .. } => Err(Completion::Outside(Branch::CharacterReference)),
            Unit::Scalar {
                ch: '\0',
                start,
                end,
            } => {
                self.flush_run()?;
                self.append_diag(
                    DiagCode::UnexpectedNullCharacter,
                    (start, end),
                    DiagContext::Data,
                    Handling::Continued,
                )?;
                self.preflight_token(Some((start, start)))?;
                self.reserve_retained(1, start)?;
                let at = evidence(self.source, start, end);
                self.tokens.push(Token::Character {
                    at,
                    interpreted: "\0".to_owned(),
                });
                Ok(Flow::Continue)
            }
            Unit::Scalar { ch, start, end } => {
                self.reserve_retained(ch.len_utf8(), start)?;
                self.push_run_char(ch, start, end);
                Ok(Flow::Continue)
            }
        }
    }

    fn push_run_char(&mut self, ch: char, start: usize, end: usize) {
        match &mut self.run {
            Some(run) => {
                run.text.push(ch);
                run.end = end;
            }
            None => {
                self.run = Some(Run {
                    start,
                    end,
                    text: ch.to_string(),
                });
            }
        }
    }

    fn tag_open(&mut self, unit: Unit) -> Result<Flow, Completion> {
        match unit {
            Unit::Scalar { ch: '?', .. } => self.pi_entry(unit),
            Unit::Scalar { ch: '!', .. } if self.dialect == Dialect::WidenBang => {
                self.pi_entry(unit)
            }
            Unit::Scalar { ch: '!', .. } => Err(Completion::Outside(Branch::MarkupDeclaration)),
            Unit::Scalar { ch: '/', .. } => Err(Completion::Outside(Branch::EndTagOpen)),
            Unit::Scalar { ch, .. } if ch.is_ascii_alphabetic() => {
                Err(Completion::Outside(Branch::StartTagName))
            }
            Unit::Eof { .. } => Err(Completion::Outside(Branch::EofAfterLessThan)),
            Unit::Scalar { start, end, .. } => {
                // invalid-first-character-of-tag-name: emit `<`, reconsume in
                // Data. The retained cost is reserved before the diagnostic
                // claims a recovery.
                self.reserve_retained(1, self.tag_open_start)?;
                self.append_diag(
                    DiagCode::InvalidFirstCharacterOfTagName,
                    (start, end),
                    DiagContext::TagOpen,
                    Handling::RecoveredLiteralMarkupPrefix,
                )?;
                self.push_run_char('<', self.tag_open_start, self.lt_end);
                self.state = State::Data;
                Ok(Flow::Reconsume)
            }
        }
    }

    /// Tag open `?` (or, for a wrong dialect, another unit): PI entry.
    ///
    /// The selected behavior commits no diagnostic, token, comment, or
    /// buffer, enters no PI grammar state, and reports Deferred
    /// ProcessingInstruction with a floor-bounded full-opener geometry.
    fn pi_entry(&mut self, unit: Unit) -> Result<Flow, Completion> {
        let (opener_start, question_end) = (self.tag_open_start, unit.end());
        let question_start = opener_start + 1;
        self.recognition = Some(Recognition {
            opener: evidence(self.source, opener_start, question_end),
        });
        let floor = self.floor();
        let mut prefix = opener_start.max(floor);
        match self.dialect {
            Dialect::Selected | Dialect::WidenBang => {}
            Dialect::QuestionUnit => prefix = self.lt_end.max(floor),
            Dialect::HistoricalDiagnostic => {
                self.append_diag(
                    DiagCode::UnexpectedQuestionMarkInsteadOfTagName,
                    (question_start, question_end),
                    DiagContext::TagOpen,
                    Handling::Stopped,
                )?;
                prefix = question_end;
            }
            Dialect::EmptyTrigger => prefix = question_end,
            Dialect::AttemptDiagnostic => {
                self.append_diag(
                    DiagCode::UnexpectedQuestionMarkInsteadOfTagName,
                    (question_start, question_end),
                    DiagContext::TagOpen,
                    Handling::Stopped,
                )?;
            }
            Dialect::AttemptToken => {
                self.preflight_token(Some((question_end, question_end)))?;
                let at = evidence(self.source, opener_start, question_end);
                self.tokens.push(Token::ProcessingInstruction {
                    at,
                    target: String::new(),
                });
            }
            Dialect::AttemptRetained => {
                self.reserve_retained(question_end - opener_start, opener_start)?;
            }
            Dialect::AttemptTemporaryBuffer => {
                let target = self.text()[question_end..]
                    .chars()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
                    .count();
                if target > self.limits.temporary_buffer_bytes {
                    return Err(Completion::ResourceLimit {
                        resource: Resource::TemporaryBufferBytes,
                        limit: self.limits.temporary_buffer_bytes,
                        attempted: target,
                        at: Some((question_end, question_end)),
                    });
                }
            }
            Dialect::ParseTarget => {
                // Enters PI grammar: consumes target and data to `>` or EOF.
                let rest = self.text()[question_end..].to_owned();
                let target: String = rest
                    .chars()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
                    .collect();
                self.steps += rest.chars().count() + 1;
                let at = evidence(self.source, opener_start, self.text().len());
                self.tokens
                    .push(Token::ProcessingInstruction { at, target });
                self.processed_end = self.text().len();
                return Err(Completion::Complete);
            }
        }
        self.processed_end = prefix;
        let trigger = match self.dialect {
            Dialect::EmptyTrigger | Dialect::HistoricalDiagnostic => {
                evidence(self.source, question_end, question_end)
            }
            _ => evidence(self.source, prefix, question_end),
        };
        Err(Completion::Unsupported {
            capability: Capability::ProcessingInstruction,
            availability: Availability::Deferred,
            trigger,
        })
    }
}

// ---------------------------------------------------------------------------
// Validators: accepted result-contract invariants (R) and theorem rules (T)
// ---------------------------------------------------------------------------

const R_SOURCE: &str = "R: evidence must belong to the run's source with exact bytes";
const R_BOUNDS: &str = "R: processed prefix must lie in source and cover the BOM";
const R_TOKENS: &str = "R: tokens must be ordered and inside the processed prefix";
const R_DIAGNOSTICS: &str = "R: diagnostics must be ordered and inside the processed prefix";
const R_TRIGGER: &str = "R: an Input trigger must start exactly at the coverage boundary";
const R_RESOURCE: &str = "R: refusal must have attempted > limit and a boundary inside the prefix";
const R_USAGE: &str = "R: usage must match evidence and respect every limit";
const R_COMPLETE: &str = "R: Complete requires the whole source processed";

/// Accepted result-contract invariants this leaf relies on. Both candidate
/// geometries (and the historical one) satisfy them: the contract alone does
/// not select the policy.
fn validate_contract(
    observation: &Observation,
    source: &SourceText,
    limits: Limits,
) -> Result<(), &'static str> {
    let ranges_match = |at: &Evidence| {
        at.source_id == source.id()
            && source
                .anchor(at.start, at.end)
                .map(|anchor| anchor.fragment() == at.raw)
                .unwrap_or(false)
    };
    let trigger = match &observation.completion {
        Completion::Unsupported { trigger, .. } => Some(trigger),
        _ => None,
    };
    let evidences = observation
        .tokens
        .iter()
        .map(|token| match token {
            Token::Character { at, .. }
            | Token::ProcessingInstruction { at, .. }
            | Token::Comment { at } => at,
        })
        .chain(observation.diagnostics.iter().map(|diag| &diag.at))
        .chain(trigger)
        .chain(observation.recognition.iter().map(|rec| &rec.opener));
    for at in evidences {
        if !ranges_match(at) {
            return Err(R_SOURCE);
        }
    }
    if observation.source_id != source.id()
        || observation.source_len != source.as_str().len()
        || observation.processed_end > observation.source_len
        || observation.bom_end > observation.processed_end
    {
        return Err(R_BOUNDS);
    }
    let mut previous_end = 0;
    for token in &observation.tokens {
        let at = match token {
            Token::Character { at, .. }
            | Token::ProcessingInstruction { at, .. }
            | Token::Comment { at } => at,
        };
        if at.start < previous_end
            || at.end > observation.processed_end
                && !matches!(token, Token::ProcessingInstruction { .. })
        {
            return Err(R_TOKENS);
        }
        previous_end = at.end;
    }
    let mut previous_start = 0;
    for diag in &observation.diagnostics {
        if diag.at.start < previous_start || diag.at.end > observation.processed_end {
            return Err(R_DIAGNOSTICS);
        }
        previous_start = diag.at.start;
    }
    match &observation.completion {
        Completion::Unsupported { trigger, .. } => {
            if trigger.start != observation.processed_end {
                return Err(R_TRIGGER);
            }
        }
        Completion::ResourceLimit {
            limit,
            attempted,
            at,
            ..
        } => {
            if attempted <= limit || at.is_some_and(|(start, _)| start > observation.processed_end)
            {
                return Err(R_RESOURCE);
            }
        }
        Completion::Complete => {
            if observation.processed_end != observation.source_len {
                return Err(R_COMPLETE);
            }
        }
        Completion::InvalidConfiguration(_) | Completion::Outside(_) => {}
    }
    let refused = match &observation.completion {
        Completion::ResourceLimit { resource, .. } => Some(*resource),
        _ => None,
    };
    let character_bytes: usize = observation
        .tokens
        .iter()
        .map(|token| match token {
            Token::Character { interpreted, .. } => interpreted.len(),
            _ => 0,
        })
        .sum();
    if observation.usage.source_bytes != observation.source_len
        || observation.usage.emitted_tokens != observation.tokens.len()
        || observation.usage.diagnostics != observation.diagnostics.len()
        || observation.usage.retained_interpreted_bytes < character_bytes
        || Resource::ALL.iter().any(|resource| {
            Some(*resource) != refused
                && observation.usage.value_of(*resource) > limits.limit_for(*resource)
        })
    {
        return Err(R_USAGE);
    }
    Ok(())
}

const T_COMPLETION: &str = "T: PI entry must remain Unsupported(ProcessingInstruction, Deferred)";
const T_NO_PI_DIAGNOSTIC: &str = "T: PI entry must own no syntax diagnostic";
const T_NO_PI_OUTPUT: &str = "T: PI entry must emit no PI token, comment, or interpreted output";
const T_NO_RESOURCE_USE: &str = "T: PI entry must consume no buffer, retained, or token resource";
const T_OPENER: &str = "T: the recognized opener must be the exact cursor-owned `<?`";
const T_GEOMETRY: &str = "T: coverage/trigger must be the floor-bounded full-opener geometry";
const T_TRIGGER_EMPTY: &str = "T: the trigger must carry non-empty opener evidence";
const T_TILING: &str = "T: committed evidence plus trigger must tile the opener without gaps";

/// The corrected PI-entry theorem, applied to a run that reached Tag open `?`.
fn validate_theorem(observation: &Observation) -> Result<(), &'static str> {
    let Completion::Unsupported {
        capability: Capability::ProcessingInstruction,
        availability: Availability::Deferred,
        trigger,
    } = &observation.completion
    else {
        return Err(T_COMPLETION);
    };
    let Some(recognition) = &observation.recognition else {
        return Err(T_OPENER);
    };
    let opener = &recognition.opener;
    if opener.raw != "<?" || opener.end != opener.start + 2 {
        return Err(T_OPENER);
    }
    // No PI-owned diagnostic, and nothing diagnosed at or after the `?`.
    if observation
        .diagnostics
        .iter()
        .any(|diag| diag.code.is_pi_owned() || diag.at.end > opener.start + 1)
    {
        return Err(T_NO_PI_DIAGNOSTIC);
    }
    if observation
        .tokens
        .iter()
        .any(|token| !matches!(token, Token::Character { .. }))
    {
        return Err(T_NO_PI_OUTPUT);
    }
    let character_bytes: usize = observation
        .tokens
        .iter()
        .map(|token| match token {
            Token::Character { interpreted, .. } => interpreted.len(),
            _ => 0,
        })
        .sum();
    if observation.usage.peak_temporary_buffer_bytes != 0
        || observation.usage.peak_attributes_per_tag != 0
        || observation.usage.retained_interpreted_bytes != character_bytes
        || observation.usage.emitted_tokens != observation.tokens.len()
    {
        return Err(T_NO_RESOURCE_USE);
    }
    let expected_prefix = opener.start.max(committed_floor(observation));
    if observation.processed_end != expected_prefix
        || trigger.start != expected_prefix
        || trigger.end != opener.end
    {
        return Err(T_GEOMETRY);
    }
    if trigger.start == trigger.end {
        return Err(T_TRIGGER_EMPTY);
    }
    if !opener_tiled(observation) {
        return Err(T_TILING);
    }
    Ok(())
}

/// The end of all committed evidence in an observation: the lower bound a
/// terminal coverage boundary may never cross.
fn committed_floor(observation: &Observation) -> usize {
    observation
        .tokens
        .iter()
        .map(|token| match token {
            Token::Character { at, .. }
            | Token::ProcessingInstruction { at, .. }
            | Token::Comment { at } => at.end,
        })
        .chain(observation.diagnostics.iter().map(|diag| diag.at.end))
        .fold(observation.bom_end, usize::max)
}

/// Every byte of the recognized opener is owned by a committed diagnostic or
/// by the trigger. No byte is left to consumer reconstruction.
fn opener_tiled(observation: &Observation) -> bool {
    let (Some(recognition), Completion::Unsupported { trigger, .. }) =
        (&observation.recognition, &observation.completion)
    else {
        return false;
    };
    let opener = &recognition.opener;
    (opener.start..opener.end).all(|byte| {
        (trigger.start..trigger.end).contains(&byte)
            || observation
                .diagnostics
                .iter()
                .any(|diag| (diag.at.start..diag.at.end).contains(&byte))
    })
}

/// Processed bytes no committed token, diagnostic, or BOM owns. Measurement
/// only: the accepted contract tolerates such bytes (abandoned input), which
/// is why it cannot select the policy by itself.
fn unowned_processed_bytes(observation: &Observation) -> Vec<usize> {
    (observation.bom_end..observation.processed_end)
        .filter(|byte| {
            !observation.tokens.iter().any(|token| match token {
                Token::Character { at, .. }
                | Token::ProcessingInstruction { at, .. }
                | Token::Comment { at } => (at.start..at.end).contains(byte),
            }) && !observation
                .diagnostics
                .iter()
                .any(|diag| (diag.at.start..diag.at.end).contains(byte))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Hand-authored gold builders
// ---------------------------------------------------------------------------

fn text(id: u64, content: &str) -> SourceText {
    SourceText::new(SourceId::new(id), content.to_owned())
}

fn character(source: &SourceText, start: usize, end: usize, interpreted: &str) -> Token {
    Token::Character {
        at: evidence(source, start, end),
        interpreted: interpreted.to_owned(),
    }
}

fn diag(
    source: &SourceText,
    code: DiagCode,
    range: (usize, usize),
    context: DiagContext,
    handling: Handling,
) -> Diag {
    Diag {
        code,
        at: evidence(source, range.0, range.1),
        context,
        handling,
    }
}

fn pi_unsupported(source: &SourceText, trigger: (usize, usize)) -> Completion {
    Completion::Unsupported {
        capability: Capability::ProcessingInstruction,
        availability: Availability::Deferred,
        trigger: evidence(source, trigger.0, trigger.1),
    }
}

struct Gold<'a> {
    source: &'a SourceText,
    bom_end: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diag>,
    processed_end: usize,
    completion: Completion,
    usage: Usage,
    opener: Option<(usize, usize)>,
}

impl Gold<'_> {
    fn build(self) -> Observation {
        Observation {
            source_id: self.source.id(),
            source_len: self.source.as_str().len(),
            bom_end: self.bom_end,
            tokens: self.tokens,
            diagnostics: self.diagnostics,
            processed_end: self.processed_end,
            completion: self.completion,
            usage: self.usage,
            recognition: self.opener.map(|(start, end)| Recognition {
                opener: evidence(self.source, start, end),
            }),
        }
    }
}

/// A PI-entry refusal with no committed evidence other than `tokens` and
/// `diagnostics`. `processed_end` and the trigger are authored, never derived.
fn pi_gold<'a>(
    source: &'a SourceText,
    tokens: Vec<Token>,
    diagnostics: Vec<Diag>,
    processed_end: usize,
    opener: (usize, usize),
    trigger: (usize, usize),
    usage: Usage,
) -> Gold<'a> {
    Gold {
        source,
        bom_end: 0,
        tokens,
        diagnostics,
        processed_end,
        completion: pi_unsupported(source, trigger),
        usage,
        opener: Some(opener),
    }
}

fn refusal<'a>(
    source: &'a SourceText,
    tokens: Vec<Token>,
    diagnostics: Vec<Diag>,
    processed_end: usize,
    completion: Completion,
    usage: Usage,
) -> Gold<'a> {
    Gold {
        source,
        bom_end: 0,
        tokens,
        diagnostics,
        processed_end,
        completion,
        usage,
        opener: None,
    }
}

fn assert_run(source: &SourceText, limits: Limits, gold: Gold<'_>) {
    let expected = gold.build();
    let observed = run(source, limits);
    assert_eq!(observed, expected, "{:?}", source.as_str());
    assert_eq!(
        validate_contract(&observed, source, limits),
        Ok(()),
        "{:?}",
        source.as_str()
    );
}

// ---------------------------------------------------------------------------
// Key fixtures
// ---------------------------------------------------------------------------

/// Every PI-entry fixture the theorem covers, as plain source text.
const PI_FIXTURES: &[&str] = &[
    "<?",
    "<?probe>",
    "a<?probe>",
    "abc<?probe>",
    "\u{e9}\u{1f600}<?x",
    "\u{feff}<?x",
    "<1<?x",
    "\u{1}<?",
    "\0a<?",
    "<<?x",
];

#[test]
fn a1_bare_opener_is_refused_without_grammar_or_eof_claims() {
    // Data('<'), TagOpen('?') = 2 transitions. Nothing follows to examine.
    let source = text(1, "<?");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![],
            vec![],
            0,
            (0, 2),
            (0, 2),
            usage(2, 2, 0, 0, 0),
        ),
    );
}

#[test]
fn b1_legal_looking_target_is_refused_before_target_grammar() {
    // `<?probe>` is the key counterexample to an unconditional question-mark
    // error: the target is well formed, yet the profile stops at TagOpen('?').
    // Still exactly 2 transitions: `probe>` is never examined.
    let source = text(1, "<?probe>");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![],
            vec![],
            0,
            (0, 2),
            (0, 2),
            usage(8, 2, 0, 0, 0),
        ),
    );
    let observed = run(&source, Limits::generous());
    assert_eq!(observed.usage.transition_steps, 2);
    assert_eq!(
        source.as_str().len() - observed.processed_end,
        8,
        "the whole source stays unprocessed; the target is never examined"
    );
}

#[test]
fn c1_preceding_text_survives_and_no_pi_error_is_added() {
    // Data('a'), Data('<') closes the run, TagOpen('?') = 3.
    let source = text(1, "a<?probe>");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![character(&source, 0, 1, "a")],
            vec![],
            1,
            (1, 3),
            (1, 3),
            usage(9, 3, 1, 0, 1),
        ),
    );
}

#[test]
fn d1_a_longer_run_stays_one_token_and_the_boundary_stays_exact() {
    // a, b, c, '<', '?' = 5 transitions; one aggregated Character token.
    let source = text(1, "abc<?probe>");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![character(&source, 0, 3, "abc")],
            vec![],
            3,
            (3, 5),
            (3, 5),
            usage(11, 5, 1, 0, 3),
        ),
    );
}

#[test]
fn d2_ranges_are_utf8_bytes_not_scalar_counts() {
    // U+00E9 is 2 bytes, U+1F600 is 4: the run is [0,6), the opener [6,8).
    let source = text(1, "\u{e9}\u{1f600}<?x");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![character(&source, 0, 6, "\u{e9}\u{1f600}")],
            vec![],
            6,
            (6, 8),
            (6, 8),
            usage(9, 4, 1, 0, 6),
        ),
    );
}

#[test]
fn d3_a_leading_bom_stays_inside_coverage_and_costs_no_transition() {
    // BOM is skipped by preprocessing: [0,3) is processed, no transition.
    let source = text(1, "\u{feff}<?x");
    let mut gold = pi_gold(
        &source,
        vec![],
        vec![],
        3,
        (3, 5),
        (3, 5),
        usage(6, 2, 0, 0, 0),
    );
    gold.bom_end = 3;
    assert_run(&source, Limits::generous(), gold);
}

#[test]
fn e1_an_earlier_tag_open_diagnostic_survives_and_pi_entry_adds_none() {
    // `<1` is accepted: invalid-first-character-of-tag-name at [1,2), `<`
    // emitted as text, '1' reconsumed in Data. Then a fresh `<?` opener.
    // Data('<'), TagOpen('1'), Data('1' reconsumed), Data('<'),
    // TagOpen('?') = 5.
    let source = text(1, "<1<?x");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![character(&source, 0, 2, "<1")],
            vec![diag(
                &source,
                DiagCode::InvalidFirstCharacterOfTagName,
                (1, 2),
                DiagContext::TagOpen,
                Handling::RecoveredLiteralMarkupPrefix,
            )],
            2,
            (2, 4),
            (2, 4),
            usage(5, 5, 1, 1, 2),
        ),
    );
}

#[test]
fn e2_an_earlier_preprocessing_diagnostic_survives() {
    // The control character diagnostic costs no transition of its own:
    // Data(U+0001), Data('<'), TagOpen('?') = 3.
    let source = text(1, "\u{1}<?");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![character(&source, 0, 1, "\u{1}")],
            vec![diag(
                &source,
                DiagCode::ControlCharacterInInputStream,
                (0, 1),
                DiagContext::InputPreprocessing,
                Handling::Continued,
            )],
            1,
            (1, 3),
            (1, 3),
            usage(3, 3, 1, 1, 1),
        ),
    );
}

#[test]
fn e3_an_earlier_null_diagnostic_and_token_survive_and_runs_stay_separate() {
    // Data(NUL) emits its own token; 'a' is a separate run closed by '<'.
    // NUL, 'a', '<', '?' = 4.
    let source = text(1, "\0a<?");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![
                character(&source, 0, 1, "\0"),
                character(&source, 1, 2, "a"),
            ],
            vec![diag(
                &source,
                DiagCode::UnexpectedNullCharacter,
                (0, 1),
                DiagContext::Data,
                Handling::Continued,
            )],
            2,
            (2, 4),
            (2, 4),
            usage(4, 4, 2, 1, 2),
        ),
    );
}

#[test]
fn g1_a_diagnostic_owned_less_than_narrows_the_trigger_without_rollback() {
    // `<<?x`: the second `<` is the offending unit of a committed
    // invalid-first-character-of-tag-name diagnostic [1,2) AND the first byte
    // of the opener [1,3). Diagnostics must lie inside the processed prefix
    // and an Input trigger must start at the prefix end, so the prefix is 2
    // and the trigger is the unowned remainder `?` [2,3). Nothing committed
    // is rolled back; diagnostic + trigger tile the opener. The question-unit
    // alternative gives the same geometry here.
    // Data('<'), TagOpen('<'), Data('<' reconsumed, closes "<"),
    // TagOpen('?') = 4.
    let source = text(1, "<<?x");
    assert_run(
        &source,
        Limits::generous(),
        pi_gold(
            &source,
            vec![character(&source, 0, 1, "<")],
            vec![diag(
                &source,
                DiagCode::InvalidFirstCharacterOfTagName,
                (1, 2),
                DiagContext::TagOpen,
                Handling::RecoveredLiteralMarkupPrefix,
            )],
            2,
            (1, 3),
            (2, 3),
            usage(4, 4, 1, 1, 1),
        ),
    );
    let observed = run(&source, Limits::generous());
    assert!(opener_tiled(&observed));
    assert_eq!(validate_theorem(&observed), Ok(()));
}

#[test]
fn every_fixture_satisfies_the_corrected_theorem_under_every_source_id() {
    for id in [1, 7, u64::MAX] {
        for fixture in PI_FIXTURES {
            let source = text(id, fixture);
            let observed = run(&source, Limits::generous());
            assert_eq!(validate_theorem(&observed), Ok(()), "{fixture:?}");
            assert_eq!(
                validate_contract(&observed, &source, Limits::generous()),
                Ok(()),
                "{fixture:?}"
            );
            assert_eq!(observed.source_id, SourceId::new(id));
            let Completion::Unsupported { trigger, .. } = &observed.completion else {
                panic!("{fixture:?} must be an unsupported refusal");
            };
            assert_eq!(trigger.source_id, SourceId::new(id), "{fixture:?}");
            assert!(
                observed
                    .tokens
                    .iter()
                    .all(|token| matches!(token, Token::Character { .. }))
            );
        }
    }
}

#[test]
fn source_identity_changes_provenance_and_nothing_else() {
    for fixture in PI_FIXTURES {
        let first = run(&text(1, fixture), Limits::generous());
        let second = run(&text(7, fixture), Limits::generous());
        assert_ne!(first.source_id, second.source_id);
        let mut normalized = second.clone();
        normalized.source_id = first.source_id;
        let rewrite = |at: &mut Evidence| at.source_id = first.source_id;
        for token in &mut normalized.tokens {
            match token {
                Token::Character { at, .. }
                | Token::ProcessingInstruction { at, .. }
                | Token::Comment { at } => rewrite(at),
            }
        }
        for diag in &mut normalized.diagnostics {
            rewrite(&mut diag.at);
        }
        if let Completion::Unsupported { trigger, .. } = &mut normalized.completion {
            rewrite(trigger);
        }
        if let Some(recognition) = &mut normalized.recognition {
            rewrite(&mut recognition.opener);
        }
        assert_eq!(first, normalized, "{fixture:?}");
    }
}

#[test]
fn repeated_runs_are_deterministic() {
    for fixture in PI_FIXTURES {
        let source = text(1, fixture);
        assert_eq!(
            run(&source, Limits::generous()),
            run(&source, Limits::generous())
        );
    }
}

#[test]
fn negative_semantics_the_selected_profile_makes_no_pi_claims() {
    let forbidden = [
        DiagCode::UnexpectedQuestionMarkInsteadOfTagName,
        DiagCode::EofInProcessingInstruction,
        DiagCode::InvalidFirstCharacterOfProcessingInstructionTarget,
        DiagCode::InvalidProcessingInstructionTarget,
        DiagCode::DisallowedProcessingInstructionTarget,
        DiagCode::UnsupportedReportedAsParseError,
    ];
    for fixture in PI_FIXTURES {
        let source = text(1, fixture);
        let observed = run(&source, Limits::generous());
        assert!(
            observed
                .diagnostics
                .iter()
                .all(|diag| !forbidden.contains(&diag.code)),
            "{fixture:?}: a PI-owned parse error was claimed"
        );
        assert!(
            observed
                .diagnostics
                .iter()
                .all(|diag| diag.handling != Handling::Stopped),
            "{fixture:?}: unsupported was reported as a stopped parse error"
        );
        assert!(
            observed
                .tokens
                .iter()
                .all(|token| matches!(token, Token::Character { .. })),
            "{fixture:?}: PI token, comment, or interpreted PI output claimed"
        );
        assert_ne!(observed.completion, Completion::Complete, "{fixture:?}");
        assert!(
            observed.processed_end < observed.source_len,
            "{fixture:?}: refusal must leave unprocessed input"
        );
        assert!(matches!(
            observed.completion,
            Completion::Unsupported {
                capability: Capability::ProcessingInstruction,
                availability: Availability::Deferred,
                ..
            }
        ));
    }
}

#[test]
fn earlier_evidence_is_monotone_and_never_reclassified() {
    // Each prefix of the source before the opener, run alone to its own
    // refusal, commits the same earlier evidence the full run keeps.
    for (with_opener, prefix_only) in [
        ("a<?probe>", "a"),
        ("abc<?probe>", "abc"),
        ("\u{e9}\u{1f600}<?x", "\u{e9}\u{1f600}"),
    ] {
        let source = text(1, with_opener);
        let observed = run(&source, Limits::generous());
        let committed: String = observed
            .tokens
            .iter()
            .map(|token| match token {
                Token::Character { interpreted, .. } => interpreted.as_str(),
                _ => "",
            })
            .collect();
        assert_eq!(committed, prefix_only);
        assert_eq!(observed.diagnostics, Vec::new());
    }
    let e1 = run(&text(1, "<1<?x"), Limits::generous());
    assert_eq!(e1.diagnostics.len(), 1);
    assert_eq!(
        e1.diagnostics[0].code,
        DiagCode::InvalidFirstCharacterOfTagName
    );
    assert_eq!(
        e1.diagnostics[0].handling,
        Handling::RecoveredLiteralMarkupPrefix
    );
    assert!(e1.diagnostics[0].at.end <= e1.processed_end);
}

// ---------------------------------------------------------------------------
// Nearby non-PI controls: no accidental widening
// ---------------------------------------------------------------------------

fn outside_branch(content: &str) -> Branch {
    match run(&text(1, content), Limits::generous()).completion {
        Completion::Outside(branch) => branch,
        other => panic!("{content:?} must stay outside the PI theorem, got {other:?}"),
    }
}

#[test]
fn f1_non_pi_tag_open_branches_are_never_pi_entry() {
    assert_eq!(outside_branch("<a?>"), Branch::StartTagName);
    assert_eq!(outside_branch("</?>"), Branch::EndTagOpen);
    assert_eq!(outside_branch("<!?>"), Branch::MarkupDeclaration);
    assert_eq!(outside_branch("<!--?-->"), Branch::MarkupDeclaration);
    assert_eq!(outside_branch("<"), Branch::EofAfterLessThan);
}

#[test]
fn f2_a_question_mark_that_is_not_the_tag_open_unit_is_not_pi_entry() {
    assert_eq!(outside_branch("?"), Branch::EofWithoutOpener);
    assert_eq!(outside_branch("a?b"), Branch::EofWithoutOpener);
    assert_eq!(outside_branch("< ?x"), Branch::EofWithoutOpener);
    assert_eq!(outside_branch("<1?"), Branch::EofWithoutOpener);
}

#[test]
fn f3_decoded_less_than_is_not_an_authored_opener() {
    // A character reference is a different, unmodelled branch; decoded `<`
    // would never re-enter tokenizer input, so `&lt;?` is not PI entry.
    assert_eq!(outside_branch("&lt;?"), Branch::CharacterReference);
    assert_eq!(outside_branch("a&lt;?probe>"), Branch::CharacterReference);
}

#[test]
fn f4_unmodelled_input_preprocessing_is_a_refused_boundary_not_a_claim() {
    assert_eq!(outside_branch("\r<?"), Branch::CarriageReturn);
    assert_eq!(outside_branch("\u{fdd0}<?"), Branch::Noncharacter);
}

#[test]
fn f5_pi_entry_does_not_widen_to_markup_declaration_or_comments() {
    for fixture in ["<!?>", "<!--?-->", "<!DOCTYPE html><?x>"] {
        let observed = run(&text(1, fixture), Limits::generous());
        assert!(
            !matches!(
                observed.completion,
                Completion::Unsupported {
                    capability: Capability::ProcessingInstruction,
                    ..
                }
            ),
            "{fixture:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Resource and refusal ordering matrix (no new dimension)
// ---------------------------------------------------------------------------

#[test]
fn r0_the_resource_vector_is_the_existing_seven_dimensions() {
    assert_eq!(Resource::ALL.len(), 7);
    let limits = Limits::generous();
    for resource in Resource::ALL {
        assert!(limits.limit_for(resource) > 0);
    }
    let observed = run(&text(1, "<?probe>"), limits);
    for resource in [
        Resource::AttributesPerTag,
        Resource::TemporaryBufferBytes,
        Resource::RetainedInterpretedBytes,
        Resource::EmittedTokens,
        Resource::Diagnostics,
    ] {
        assert_eq!(observed.usage.value_of(resource), 0, "{resource:?}");
    }
}

#[test]
fn ra_source_bytes_refusal_precedes_recognition() {
    let source = text(1, "<?");
    let mut limits = Limits::generous();
    limits.source_bytes = 1;
    assert_run(
        &source,
        limits,
        refusal(
            &source,
            vec![],
            vec![],
            0,
            Completion::ResourceLimit {
                resource: Resource::SourceBytes,
                limit: 1,
                attempted: 2,
                at: Some((0, 0)),
            },
            usage(2, 0, 0, 0, 0),
        ),
    );
}

#[test]
fn rb_transition_refusal_before_data_less_than_never_reaches_pi() {
    // Data('a') commits; the attempted Data('<') is transition 2 > 1. The
    // pending run is closed on the way out.
    let source = text(1, "a<?");
    let mut limits = Limits::generous();
    limits.transition_steps = 1;
    assert_run(
        &source,
        limits,
        refusal(
            &source,
            vec![character(&source, 0, 1, "a")],
            vec![],
            1,
            Completion::ResourceLimit {
                resource: Resource::TransitionSteps,
                limit: 1,
                attempted: 2,
                at: Some((1, 1)),
            },
            usage(3, 1, 1, 0, 1),
        ),
    );
}

#[test]
fn rc_transition_refusal_before_tag_open_question_dispatch_observes_no_pi_entry() {
    // `<?` with 1 step: Data('<') commits, TagOpen('?') is attempted 2 > 1.
    // The `<` was dispatched, so ordinary refusal coverage includes it (the
    // same shape any Tag open refusal has); no PI entry is observed.
    let source = text(1, "<?");
    let mut limits = Limits::generous();
    limits.transition_steps = 1;
    assert_run(
        &source,
        limits,
        refusal(
            &source,
            vec![],
            vec![],
            1,
            Completion::ResourceLimit {
                resource: Resource::TransitionSteps,
                limit: 1,
                attempted: 2,
                at: Some((1, 1)),
            },
            usage(2, 1, 0, 0, 0),
        ),
    );
    let observed = run(&source, limits);
    assert!(
        observed.recognition.is_none(),
        "PI entry must not be observed"
    );

    // With a preceding run: 2 steps are Data('a'), Data('<'); '?' is the third.
    let source = text(1, "a<?");
    limits.transition_steps = 2;
    assert_run(
        &source,
        limits,
        refusal(
            &source,
            vec![character(&source, 0, 1, "a")],
            vec![],
            2,
            Completion::ResourceLimit {
                resource: Resource::TransitionSteps,
                limit: 2,
                attempted: 3,
                at: Some((2, 2)),
            },
            usage(3, 2, 1, 0, 1),
        ),
    );
}

#[test]
fn rc0_exactly_two_transitions_are_enough_and_no_third_is_needed() {
    let source = text(1, "<?probe>");
    let mut limits = Limits::generous();
    limits.transition_steps = 2;
    assert_run(
        &source,
        limits,
        pi_gold(
            &source,
            vec![],
            vec![],
            0,
            (0, 2),
            (0, 2),
            usage(8, 2, 0, 0, 0),
        ),
    );
}

#[test]
fn rd_pending_run_resource_refusals_win_before_pi_discovery() {
    // Retained refusal at the run's own first character.
    let source = text(1, "a<?");
    let mut limits = Limits::generous();
    limits.retained_interpreted_bytes = 0;
    assert_run(
        &source,
        limits,
        refusal(
            &source,
            vec![],
            vec![],
            0,
            Completion::ResourceLimit {
                resource: Resource::RetainedInterpretedBytes,
                limit: 0,
                attempted: 1,
                at: Some((0, 0)),
            },
            usage(3, 1, 0, 0, 0),
        ),
    );

    // Emission refusal when `<` tries to close the pending run: NUL used one
    // token slot, so closing "a" is attempted 2 > 1. PI is never discovered.
    // Where a run-close refusal is anchored is not part of this theorem.
    let source = text(1, "\0a<?");
    let mut limits = Limits::generous();
    limits.emitted_tokens = 1;
    assert_run(
        &source,
        limits,
        refusal(
            &source,
            vec![character(&source, 0, 1, "\0")],
            vec![diag(
                &source,
                DiagCode::UnexpectedNullCharacter,
                (0, 1),
                DiagContext::Data,
                Handling::Continued,
            )],
            2,
            Completion::ResourceLimit {
                resource: Resource::EmittedTokens,
                limit: 1,
                attempted: 2,
                at: None,
            },
            usage(4, 3, 1, 1, 2),
        ),
    );
}

#[test]
fn re_zero_diagnostic_capacity_still_reaches_pi_because_none_is_attempted() {
    let mut limits = Limits::generous();
    limits.diagnostics = 0;
    for (fixture, gold_text) in [("<?", "<?"), ("abc<?probe>", "abc<?probe>")] {
        let source = text(1, fixture);
        let observed = run(&source, limits);
        assert_eq!(observed, run(&text(1, gold_text), Limits::generous()));
        assert_eq!(validate_theorem(&observed), Ok(()));
        assert_eq!(observed.usage.diagnostics, 0);
    }
    // Contrast: an earlier genuine diagnostic still needs capacity and its
    // real refusal wins first (the accepted NUL shape, `\0` + suffix).
    let source = text(1, "\0<?");
    assert_run(
        &source,
        limits,
        refusal(
            &source,
            vec![],
            vec![],
            0,
            Completion::ResourceLimit {
                resource: Resource::Diagnostics,
                limit: 0,
                attempted: 1,
                at: Some((0, 1)),
            },
            usage(3, 1, 0, 0, 0),
        ),
    );
}

#[test]
fn rf_token_capacity_has_no_pi_token_to_refuse() {
    // Zero is rejected as configuration before any processing, never as a PI
    // token refusal.
    let source = text(1, "<?");
    let mut limits = Limits::generous();
    limits.emitted_tokens = 0;
    assert_run(
        &source,
        limits,
        refusal(
            &source,
            vec![],
            vec![],
            0,
            Completion::InvalidConfiguration(ConfigFailure::ZeroEmittedTokenLimit),
            usage(2, 0, 0, 0, 0),
        ),
    );
    // Exactly the slots earlier evidence needs are enough: no PI token or
    // end-of-file token needs a further slot.
    let mut limits = Limits::generous();
    limits.emitted_tokens = 1;
    assert_eq!(
        run(&text(1, "<?"), limits),
        run(&text(1, "<?"), Limits::generous())
    );
    assert_eq!(
        run(&text(1, "a<?probe>"), limits),
        run(&text(1, "a<?probe>"), Limits::generous())
    );
    let mut limits = Limits::generous();
    limits.transition_steps = 0;
    assert_eq!(
        run(&text(1, "<?"), limits).completion,
        Completion::InvalidConfiguration(ConfigFailure::ZeroTransitionStepLimit)
    );
}

#[test]
fn rg_rh_no_retained_output_and_no_temporary_buffer_are_opened() {
    let mut limits = Limits::generous();
    limits.retained_interpreted_bytes = 0;
    limits.temporary_buffer_bytes = 0;
    for fixture in ["<?", "<?probe>", "<?x"] {
        let observed = run(&text(1, fixture), limits);
        assert_eq!(validate_theorem(&observed), Ok(()), "{fixture:?}");
        assert_eq!(observed.usage.retained_interpreted_bytes, 0);
        assert_eq!(observed.usage.peak_temporary_buffer_bytes, 0);
    }
    // Retained bytes are exactly the preceding run, no more.
    let mut limits = Limits::generous();
    limits.retained_interpreted_bytes = 1;
    let observed = run(&text(1, "a<?probe>"), limits);
    assert_eq!(validate_theorem(&observed), Ok(()));
    assert_eq!(observed.usage.retained_interpreted_bytes, 1);
}

#[test]
fn rm_the_minimal_limit_vector_for_the_theorem_reaches_pi() {
    let limits = Limits {
        source_bytes: 8,
        transition_steps: 2,
        emitted_tokens: 1,
        diagnostics: 0,
        attributes_per_tag: 0,
        retained_interpreted_bytes: 0,
        temporary_buffer_bytes: 0,
    };
    let observed = run(&text(1, "<?probe>"), limits);
    assert_eq!(validate_theorem(&observed), Ok(()));
    assert_eq!(
        validate_contract(&observed, &text(1, "<?probe>"), limits),
        Ok(())
    );
    assert_eq!(observed.usage.transition_steps, 2);
}

// ---------------------------------------------------------------------------
// Coverage / trigger policy challenge
// ---------------------------------------------------------------------------

mod policy_challenge {
    use super::*;

    /// `UNSUP-003` (`<!x>`): the accepted MarkupDeclaration unsupported
    /// boundary. Hand-authored here as precedent geometry: processed prefix 0,
    /// trigger = the exact `<!` opener [0,2). A different capability
    /// discovered at the same Tag open dispatch.
    const MARKUP_DECLARATION_PRECEDENT: (&str, usize, (usize, usize)) = ("<!x>", 0, (0, 2));

    struct Geometry {
        processed_end: usize,
        trigger: (usize, usize),
    }

    fn selected(opener: (usize, usize), floor: usize) -> Geometry {
        let processed_end = opener.0.max(floor);
        Geometry {
            processed_end,
            trigger: (processed_end, opener.1),
        }
    }

    fn question_unit(opener: (usize, usize), floor: usize) -> Geometry {
        let processed_end = (opener.0 + 1).max(floor);
        Geometry {
            processed_end,
            trigger: (processed_end, opener.1),
        }
    }

    fn adjacency(fixture: &str) -> bool {
        fixture == "<<?x"
    }

    #[test]
    fn p1_the_accepted_contract_alone_does_not_select_a_policy() {
        // Honest negative: every candidate geometry satisfies the accepted
        // result contract, including the historical one. Selection must come
        // from project invariants.
        for fixture in PI_FIXTURES {
            let source = text(1, fixture);
            for dialect in [
                Dialect::Selected,
                Dialect::QuestionUnit,
                Dialect::HistoricalDiagnostic,
                Dialect::EmptyTrigger,
            ] {
                let observed = run_dialect(&source, Limits::generous(), dialect);
                assert_eq!(
                    validate_contract(&observed, &source, Limits::generous()),
                    Ok(()),
                    "{fixture:?} {dialect:?}"
                );
            }
        }
    }

    #[test]
    fn p2_owned_recognition_selected_tiles_the_opener_question_unit_does_not() {
        for fixture in PI_FIXTURES {
            let source = text(1, fixture);
            let chosen = run_dialect(&source, Limits::generous(), Dialect::Selected);
            let alternative = run_dialect(&source, Limits::generous(), Dialect::QuestionUnit);
            assert!(opener_tiled(&chosen), "{fixture:?}");
            let opener = chosen
                .recognition
                .as_ref()
                .expect("recognized")
                .opener
                .start;
            if adjacency(fixture) {
                // Both geometries coincide where a diagnostic owns the `<`.
                assert_eq!(chosen, alternative);
                continue;
            }
            assert!(!opener_tiled(&alternative), "{fixture:?}");
            assert!(
                unowned_processed_bytes(&alternative).contains(&opener),
                "{fixture:?}: the alternative leaves `<` processed with no owner"
            );
            assert!(
                unowned_processed_bytes(&chosen).is_empty(),
                "{fixture:?}: the selected prefix is fully owned"
            );
        }
    }

    #[test]
    fn p3_the_trigger_names_the_opener_without_consumer_reconstruction() {
        for fixture in PI_FIXTURES {
            let source = text(1, fixture);
            let chosen = run_dialect(&source, Limits::generous(), Dialect::Selected);
            let alternative = run_dialect(&source, Limits::generous(), Dialect::QuestionUnit);
            let (
                Completion::Unsupported { trigger, .. },
                Completion::Unsupported { trigger: alt, .. },
            ) = (&chosen.completion, &alternative.completion)
            else {
                panic!("{fixture:?}");
            };
            if adjacency(fixture) {
                assert_eq!(trigger.raw, "?");
                continue;
            }
            assert_eq!(trigger.raw, "<?", "{fixture:?}");
            assert_eq!(alt.raw, "?", "{fixture:?}");
            assert_eq!(alt.start, trigger.start + 1);
        }
    }

    #[test]
    fn p4_no_committed_evidence_is_rolled_back_by_the_selected_prefix() {
        for fixture in PI_FIXTURES {
            let source = text(1, fixture);
            let observed = run(&source, Limits::generous());
            assert!(
                observed.processed_end >= committed_floor(&observed),
                "{fixture:?}"
            );
            let opener = observed
                .recognition
                .as_ref()
                .expect("recognized")
                .opener
                .start;
            if adjacency(fixture) {
                // The `<` is committed as a diagnostic location: the prefix
                // keeps it and the trigger narrows instead of rolling back.
                assert_eq!(observed.processed_end, opener + 1);
            } else {
                // `<` is provisional until Tag open classifies it.
                assert_eq!(observed.processed_end, opener);
            }
        }
    }

    #[test]
    fn p5_parity_with_the_accepted_markup_declaration_boundary() {
        let (_, prefix, trigger) = MARKUP_DECLARATION_PRECEDENT;
        let opener = (0, 2);
        let chosen = selected(opener, 0);
        assert_eq!((chosen.processed_end, chosen.trigger), (prefix, trigger));
        let alternative = question_unit(opener, 0);
        assert_ne!(
            (alternative.processed_end, alternative.trigger),
            (prefix, trigger),
            "the question-unit alternative departs from the accepted precedent"
        );
        let (precedent_source, _, (start, end)) = MARKUP_DECLARATION_PRECEDENT;
        assert_eq!(&precedent_source[start..end], "<!");
    }

    #[test]
    fn p6_the_model_geometry_functions_agree_with_the_machine() {
        for fixture in PI_FIXTURES {
            let source = text(1, fixture);
            let chosen = run_dialect(&source, Limits::generous(), Dialect::Selected);
            let alternative = run_dialect(&source, Limits::generous(), Dialect::QuestionUnit);
            let opener = chosen
                .recognition
                .as_ref()
                .expect("recognized")
                .opener
                .clone();
            let floor = committed_floor(&chosen);
            let geometry = selected((opener.start, opener.end), floor);
            assert_eq!(chosen.processed_end, geometry.processed_end, "{fixture:?}");
            let alt = question_unit((opener.start, opener.end), floor);
            assert_eq!(alternative.processed_end, alt.processed_end, "{fixture:?}");
            let Completion::Unsupported { trigger, .. } = &alternative.completion else {
                panic!("{fixture:?}");
            };
            assert_eq!((trigger.start, trigger.end), alt.trigger, "{fixture:?}");
        }
    }
}

// ---------------------------------------------------------------------------
// Wrong-model sealing: each wrong behavior is run and must be rejected
// ---------------------------------------------------------------------------

mod wrong_models {
    use super::*;

    fn rejected_by_theorem(dialect: Dialect, fixture: &str, limits: Limits) -> &'static str {
        let source = text(1, fixture);
        let observed = run_dialect(&source, limits, dialect);
        assert_ne!(observed, run(&source, limits), "{dialect:?} {fixture:?}");
        validate_theorem(&observed).expect_err("wrong model must be rejected")
    }

    #[test]
    fn w1_copying_the_false_question_mark_diagnostic_is_rejected() {
        for fixture in ["<?", "<?probe>", "a<?probe>"] {
            assert_eq!(
                rejected_by_theorem(Dialect::HistoricalDiagnostic, fixture, Limits::generous()),
                T_NO_PI_DIAGNOSTIC
            );
        }
    }

    #[test]
    fn w2_unsupported_reported_as_a_parse_error_is_rejected() {
        let source = text(1, "<?probe>");
        let mut observed = run(&source, Limits::generous());
        observed.diagnostics.push(diag(
            &source,
            DiagCode::UnsupportedReportedAsParseError,
            (0, 2),
            DiagContext::TagOpen,
            Handling::Stopped,
        ));
        observed.usage.diagnostics = 1;
        assert_eq!(validate_theorem(&observed), Err(T_NO_PI_DIAGNOSTIC));
        let mut observed = run(&source, Limits::generous());
        observed.diagnostics.push(diag(
            &source,
            DiagCode::InvalidFirstCharacterOfTagName,
            (1, 2),
            DiagContext::TagOpen,
            Handling::Stopped,
        ));
        observed.usage.diagnostics = 1;
        assert_eq!(
            validate_theorem(&observed),
            Err(T_NO_PI_DIAGNOSTIC),
            "even a modelled code cannot be attached to the `?` itself"
        );
    }

    #[test]
    fn w3_inspecting_the_target_or_eof_enters_pi_grammar_and_is_rejected() {
        for fixture in ["<?probe>", "<?"] {
            let source = text(1, fixture);
            let observed = run_dialect(&source, Limits::generous(), Dialect::ParseTarget);
            assert_ne!(observed.usage.transition_steps, 2, "{fixture:?}");
            assert_eq!(observed.completion, Completion::Complete);
            assert_eq!(validate_theorem(&observed), Err(T_COMPLETION));
            assert!(
                observed
                    .tokens
                    .iter()
                    .any(|token| matches!(token, Token::ProcessingInstruction { .. }))
            );
        }
        // An EOF-in-PI claim after the bare opener.
        let source = text(1, "<?");
        let mut observed = run(&source, Limits::generous());
        observed.diagnostics.push(diag(
            &source,
            DiagCode::EofInProcessingInstruction,
            (2, 2),
            DiagContext::TagOpen,
            Handling::Stopped,
        ));
        observed.usage.diagnostics = 1;
        assert_eq!(validate_theorem(&observed), Err(T_NO_PI_DIAGNOSTIC));
    }

    #[test]
    fn w4_erasing_preceding_evidence_is_rejected() {
        for fixture in ["a<?probe>", "abc<?probe>", "<1<?x", "\0a<?"] {
            let source = text(1, fixture);
            let correct = run(&source, Limits::generous());
            let mut erased = correct.clone();
            erased.tokens.clear();
            erased.usage.emitted_tokens = 0;
            erased.usage.retained_interpreted_bytes = 0;
            assert_ne!(erased, correct);
            assert!(
                !unowned_processed_bytes(&erased).is_empty(),
                "{fixture:?}: erased evidence leaves processed bytes unowned"
            );
            assert!(
                unowned_processed_bytes(&correct).is_empty(),
                "{fixture:?}: the correct run owns every processed byte"
            );
        }
    }

    #[test]
    fn w5_an_empty_trigger_is_rejected() {
        for fixture in ["<?", "<?probe>", "a<?probe>"] {
            let tag = rejected_by_theorem(Dialect::EmptyTrigger, fixture, Limits::generous());
            assert!(
                tag == T_GEOMETRY || tag == T_TRIGGER_EMPTY || tag == T_TILING,
                "{fixture:?}: {tag}"
            );
            let source = text(1, fixture);
            let observed = run_dialect(&source, Limits::generous(), Dialect::EmptyTrigger);
            assert!(!opener_tiled(&observed), "{fixture:?}");
        }
    }

    #[test]
    fn w6_reconstructing_the_opener_from_coverage_or_decoded_length_is_rejected() {
        // From coverage: [processed_end, processed_end + 2) over-reads the
        // narrowed adjacency trigger into unrelated source.
        let source = text(1, "<<?x");
        let observed = run(&source, Limits::generous());
        let reconstructed = evidence(&source, observed.processed_end, observed.processed_end + 2);
        assert_eq!(reconstructed.raw, "?x");
        let Completion::Unsupported { trigger, .. } = &observed.completion else {
            panic!("adjacency fixture must be unsupported");
        };
        assert_eq!(trigger.raw, "?");
        assert_ne!(&reconstructed, trigger);

        // From decoded length: scalar count is not a byte offset. The scalar
        // count of the preceding run is 2, but the opener starts at byte 6.
        let source = text(1, "\u{e9}\u{1f600}<?x");
        let observed = run(&source, Limits::generous());
        let scalars = "\u{e9}\u{1f600}".chars().count();
        assert_eq!(scalars, 2);
        assert!(
            source.anchor(scalars, scalars + 2).is_err(),
            "a decoded-length endpoint is not even a valid byte boundary"
        );
        let Completion::Unsupported { trigger, .. } = &observed.completion else {
            panic!("multibyte fixture must be unsupported");
        };
        assert_eq!((trigger.start, trigger.end), (6, 8));
    }

    #[test]
    fn w7_historical_coverage_through_the_question_mark_is_rejected() {
        // The historical geometry (prefix through `?`, empty trigger, one
        // stopped diagnostic) is run only as a falsifier, never as an oracle.
        for fixture in ["<?", "<?x>"] {
            let source = text(1, fixture);
            let historical =
                run_dialect(&source, Limits::generous(), Dialect::HistoricalDiagnostic);
            assert_eq!(historical.processed_end, 2);
            assert_eq!(historical.diagnostics.len(), 1);
            assert_eq!(validate_theorem(&historical), Err(T_NO_PI_DIAGNOSTIC));
            assert_ne!(historical, run(&source, Limits::generous()));
        }
    }

    #[test]
    fn w8_ignoring_the_source_identity_is_rejected() {
        let source = text(7, "a<?probe>");
        let mut observed = run(&source, Limits::generous());
        let hard_coded = SourceId::new(1);
        if let Completion::Unsupported { trigger, .. } = &mut observed.completion {
            trigger.source_id = hard_coded;
        }
        assert_eq!(
            validate_contract(&observed, &source, Limits::generous()),
            Err(R_SOURCE)
        );
        let mut observed = run(&source, Limits::generous());
        observed.source_id = hard_coded;
        assert_eq!(
            validate_contract(&observed, &source, Limits::generous()),
            Err(R_BOUNDS)
        );
    }

    #[test]
    fn w9_attempting_a_pi_diagnostic_becomes_a_false_resource_refusal() {
        let mut limits = Limits::generous();
        limits.diagnostics = 0;
        let source = text(1, "<?");
        let wrong = run_dialect(&source, limits, Dialect::AttemptDiagnostic);
        assert!(matches!(
            wrong.completion,
            Completion::ResourceLimit {
                resource: Resource::Diagnostics,
                ..
            }
        ));
        assert_ne!(wrong, run(&source, limits));
        assert_eq!(validate_theorem(&wrong), Err(T_COMPLETION));
    }

    #[test]
    fn w10_attempting_a_pi_token_becomes_a_false_resource_refusal() {
        // With the single slot spent by the preceding run, a fabricated PI
        // token is attempted 2 > 1 and would refuse the correct refusal.
        let mut limits = Limits::generous();
        limits.emitted_tokens = 1;
        let source = text(1, "a<?probe>");
        let wrong = run_dialect(&source, limits, Dialect::AttemptToken);
        assert!(matches!(
            wrong.completion,
            Completion::ResourceLimit {
                resource: Resource::EmittedTokens,
                ..
            }
        ));
        assert_ne!(wrong, run(&source, limits));
        // And with room it fabricates a PI token the theorem forbids.
        let roomy = run_dialect(&source, Limits::generous(), Dialect::AttemptToken);
        assert_eq!(validate_theorem(&roomy), Err(T_NO_PI_OUTPUT));
    }

    #[test]
    fn w11_retaining_pi_bytes_or_opening_a_buffer_is_rejected() {
        let mut limits = Limits::generous();
        limits.retained_interpreted_bytes = 0;
        let source = text(1, "<?probe>");
        let wrong = run_dialect(&source, limits, Dialect::AttemptRetained);
        assert!(matches!(
            wrong.completion,
            Completion::ResourceLimit {
                resource: Resource::RetainedInterpretedBytes,
                ..
            }
        ));
        assert_ne!(wrong, run(&source, limits));

        let mut limits = Limits::generous();
        limits.temporary_buffer_bytes = 0;
        let wrong = run_dialect(&source, limits, Dialect::AttemptTemporaryBuffer);
        assert!(matches!(
            wrong.completion,
            Completion::ResourceLimit {
                resource: Resource::TemporaryBufferBytes,
                ..
            }
        ));
        assert_ne!(wrong, run(&source, limits));
    }

    #[test]
    fn w12_upgrading_the_refusal_to_complete_is_rejected() {
        for fixture in PI_FIXTURES {
            let source = text(1, fixture);
            let mut observed = run(&source, Limits::generous());
            observed.completion = Completion::Complete;
            assert_eq!(
                validate_contract(&observed, &source, Limits::generous()),
                Err(R_COMPLETE),
                "{fixture:?}"
            );
            assert_eq!(validate_theorem(&observed), Err(T_COMPLETION));
        }
    }

    #[test]
    fn w13_widening_to_markup_declaration_or_comments_is_rejected() {
        for fixture in ["<!?>", "<!--?-->", "<!x>"] {
            let source = text(1, fixture);
            let widened = run_dialect(&source, Limits::generous(), Dialect::WidenBang);
            assert!(
                matches!(
                    widened.completion,
                    Completion::Unsupported {
                        capability: Capability::ProcessingInstruction,
                        ..
                    }
                ),
                "{fixture:?}: the widened wrong model claims PI for `<!`"
            );
            assert_eq!(
                widened
                    .recognition
                    .as_ref()
                    .map(|rec| rec.opener.raw.as_str()),
                Some("<!"),
                "{fixture:?}"
            );
            assert_eq!(validate_theorem(&widened), Err(T_OPENER), "{fixture:?}");
            assert_eq!(outside_branch(fixture), Branch::MarkupDeclaration);
        }
    }

    #[test]
    fn w14_choosing_the_question_unit_policy_is_rejected_on_tiling_not_convenience() {
        for fixture in PI_FIXTURES {
            if *fixture == "<<?x" {
                continue;
            }
            let source = text(1, fixture);
            let observed = run_dialect(&source, Limits::generous(), Dialect::QuestionUnit);
            assert_eq!(validate_theorem(&observed), Err(T_GEOMETRY), "{fixture:?}");
        }
    }

    #[test]
    fn w15_the_validator_rejects_every_listed_mutation_of_a_good_observation() {
        let source = text(1, "a<?probe>");
        let good = run(&source, Limits::generous());
        assert_eq!(validate_theorem(&good), Ok(()));

        let mut shifted = good.clone();
        shifted.processed_end += 1;
        assert_eq!(validate_theorem(&shifted), Err(T_GEOMETRY));

        let mut stretched = good.clone();
        if let Completion::Unsupported { trigger, .. } = &mut stretched.completion {
            *trigger = evidence(&source, 1, 4);
        }
        assert_eq!(validate_theorem(&stretched), Err(T_GEOMETRY));

        let mut buffered = good.clone();
        buffered.usage.peak_temporary_buffer_bytes = 5;
        assert_eq!(validate_theorem(&buffered), Err(T_NO_RESOURCE_USE));

        let mut retained = good.clone();
        retained.usage.retained_interpreted_bytes += 2;
        assert_eq!(validate_theorem(&retained), Err(T_NO_RESOURCE_USE));

        let mut commented = good.clone();
        commented.tokens.push(Token::Comment {
            at: evidence(&source, 1, 3),
        });
        assert_eq!(validate_theorem(&commented), Err(T_NO_PI_OUTPUT));

        let mut wrong_capability = good;
        if let Completion::Unsupported { capability, .. } = &mut wrong_capability.completion {
            *capability = Capability::MarkupDeclaration;
        }
        assert_eq!(validate_theorem(&wrong_capability), Err(T_COMPLETION));
    }
}

// ---------------------------------------------------------------------------
// Pins, vocabulary, and historical records
// ---------------------------------------------------------------------------

#[test]
fn pins_are_recorded_as_freshness_markers_only() {
    for pin in [
        WHATWG_CURRENT_COMMIT,
        WHATWG_CURRENT_BLOB,
        WHATWG_109_COMMIT,
        WHATWG_109_BLOB,
    ] {
        assert_eq!(pin.len(), 40);
        assert!(pin.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
    assert_ne!(WHATWG_CURRENT_COMMIT, WHATWG_109_COMMIT);
    assert_ne!(WHATWG_CURRENT_BLOB, WHATWG_109_BLOB);
}

#[test]
fn historical_contradiction_is_recorded_not_resolved() {
    let names: Vec<&str> = SUPERSESSION_PRESSURE
        .iter()
        .map(|(name, _)| *name)
        .collect();
    for required in ["ERR-006", "UNSUP-004"] {
        assert!(names.contains(&required));
    }
    // This leaf neither reuses nor redefines the historical ids: it defines
    // no fixture id of its own.
    assert!(
        SUPERSESSION_PRESSURE
            .iter()
            .all(|(_, note)| !note.is_empty())
    );
}

/// Names the live capability vocabulary so "Deferred ProcessingInstruction"
/// cannot drift into generic text. Reads no behavior and runs no tokenizer.
mod live_vocabulary_witness {
    use super::{Availability, Capability};
    use crate::html::tokenizer::result::{
        HtmlTokenizerCapability, HtmlTokenizerCapabilityAvailability,
    };

    fn live_capability(capability: Capability) -> HtmlTokenizerCapability {
        match capability {
            Capability::ProcessingInstruction => HtmlTokenizerCapability::ProcessingInstruction,
            Capability::MarkupDeclaration => HtmlTokenizerCapability::MarkupDeclaration,
        }
    }

    fn live_availability(availability: Availability) -> HtmlTokenizerCapabilityAvailability {
        match availability {
            Availability::Deferred => HtmlTokenizerCapabilityAvailability::Deferred,
        }
    }

    #[test]
    fn the_model_vocabulary_maps_onto_the_live_typed_equivalents() {
        assert_eq!(
            live_capability(Capability::ProcessingInstruction),
            HtmlTokenizerCapability::ProcessingInstruction
        );
        assert_eq!(
            live_availability(Availability::Deferred),
            HtmlTokenizerCapabilityAvailability::Deferred
        );
        assert_ne!(
            live_capability(Capability::ProcessingInstruction),
            live_capability(Capability::MarkupDeclaration)
        );
    }
}
