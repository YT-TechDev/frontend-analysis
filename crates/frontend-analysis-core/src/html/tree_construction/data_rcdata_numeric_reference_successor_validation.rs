//! Candidate-independent validation for Issue #878.
//!
//! This module validates the Numeric Character Reference successor semantics
//! reachable from exactly two return contexts, before any production placement
//! or implementation:
//!
//! ```text
//! Data            '&' -> Character Reference -> Numeric -> Data
//! selected Title  '&' -> Character Reference -> Numeric -> RCDATA
//! ```
//!
//! It is **test-only** and changes no production behavior. It does not
//! authorize a Numeric production implementation.
//!
//! # Independent oracle boundary
//!
//! This module imports nothing from the production HTML tokenizer, the
//! tree-construction driver, session, or result, the Named Character Reference
//! production matcher, the CLI, or any browser, WPT, or html5lib output. Every
//! expected value is hand-authored from the pinned WHATWG text. The only crate
//! items used are the `SourceText` / `SourceId` anchoring primitives, so that
//! authored evidence ranges are real.
//!
//! # Pins
//!
//! WHATWG HTML `0cd32204c6d9408be0a42cb15c86145e21deab9d`, source blob
//! `05319af4659e15aafb3f5137362b7d9561db8812` (the blob identity was
//! re-derived from the fetched pinned `source` file while authoring this
//! leaf, and the Numeric states below were transcribed from that file).
//! Challenge/corroboration only, never the oracle: html5lib-tests
//! `c777c408b61078ea2eb4acefc2535f54dbc8b28a`, WPT
//! `7108b8dedbf4f7e6e3b21fe82b84f904f529b75f`.
//!
//! # Pinned state shape
//!
//! The 2026 editorial change removed the redundant Decimal Character Reference
//! Start state, so the pinned shape is:
//!
//! - **Character Reference**: ASCII alphanumeric reconsumes in Named; `#`
//!   appends to the temporary buffer and switches to Numeric; anything else
//!   (including EOF) flushes and reconsumes in the return state.
//! - **Numeric**: `x`/`X` switches to Hexadecimal Start; an ASCII digit
//!   *reconsumes* in Decimal; anything else (including EOF) is
//!   `absence-of-digits-in-numeric-character-reference`, flushes the consumed
//!   prefix, and reconsumes in the return state.
//! - **Hexadecimal Start**: an ASCII hex digit reconsumes in Hexadecimal;
//!   anything else is the same absence-of-digits recovery.
//! - **Hexadecimal / Decimal**: a digit accumulates; `;` switches to Numeric
//!   End; anything else is `missing-semicolon-after-character-reference` and
//!   *reconsumes* in Numeric End.
//! - **Numeric End**: consumes **no** input. It maps the accumulated code and
//!   flushes the decoded scalar, then switches to the return state.
//!
//! # Falsified shortcuts (kept executable below)
//!
//! 1. Numeric is *not* one Named-style transition: every digit is a state
//!    transition ([`k1_transition_traces_are_hand_derived_per_state_dispatch`],
//!    [`t2_long_digit_runs_are_never_compressed_into_one_transition`]).
//! 2. Overflow is *not* a function of digit-run length: 10 000 leading zeros
//!    still decode, seven short digits can overflow
//!    ([`o1_digit_count_does_not_decide_overflow`]).
//! 3. A host integer parse is not harmless: it wraps or fails
//!    ([`o2_host_integer_parsing_is_falsified`]).
//! 4. Decoded Numeric output is never retokenized or re-decoded
//!    ([`n1_decoded_syntax_is_character_data_and_never_markup`]).
//! 5. Decoded CR is a value, never re-preprocessed
//!    ([`n2_decoded_cr_is_a_value_and_authored_cr_is_preprocessing`]).
//! 6. Data and RCDATA keep distinct return ownership
//!    ([`r1_return_state_ownership_is_not_erased`]).
//! 7. Absence of digits is not numeric zero
//!    ([`z1_absence_of_digits_is_not_numeric_zero`]).
//! 8. No tree insertion mode is added ([`t6_no_new_insertion_mode_is_needed`]).
//! 9. No new provenance model is needed
//!    ([`p1_existing_provenance_shape_is_sufficient`]).
//! 10. No new resource dimension and no copied digit buffer is needed
//!     ([`a5_no_copied_digit_buffer_and_no_new_resource_dimension`]).
//! 11. Authored Data U+0000 and Numeric zero are different problems
//!     ([`s3_numeric_zero_is_not_the_authored_nul_defect`]).
//! 12. Enabling Numeric leaves the `&#65;` sentinels invalid
//!     ([`s1_the_current_numeric_sentinels_become_obsolete`]).
//! 13. UNSUP-001 cannot be silently rewritten
//!     ([`s2_unsup_001_cannot_be_silently_rewritten`]).
//!
//! # Findings carried to the placement review
//!
//! - **Numeric End charge.** The accepted #109/#111 rule charges one step per
//!   normalized unit (or EOF) examined under one state, plus one per
//!   reconsume. Numeric End examines no unit, so the literal reading charges
//!   it zero. The phrase "attempted specification-state dispatch" in
//!   `HTML_TOKENIZER_VALIDATION.md` could be read as charging it one. This
//!   leaf records the examination count (definite) and the number of End
//!   executions separately, models the literal reading, and asserts nothing
//!   that depends on the difference ([`t3_end_is_input_free_and_charged_by_no_unit`],
//!   [`a1_transition_refusal_inside_a_digit_run_keeps_honest_partial_progress`]).
//!   The placement review must decide it.
//! - **Coverage across `;`.** Whether the `;` that switches to Numeric End is
//!   committed before End's own resource preflight is not determined by the
//!   pinned text; refusal tests assert only the bounds that hold under either
//!   answer.
//! - **Diagnostic order vs. the run-result contract.** The normative order can
//!   coexist with nondecreasing diagnostic source starts, provided
//!   dispatch-time diagnostics are anchored at the unit being examined (or
//!   EOF) and not at the reference start
//!   ([`d1_semicolonless_termination_orders_preprocessing_before_dispatch`]).
//! - **Sentinel blast radius.** At the observed base, `&#65;` (the Data Numeric
//!   unsupported boundary) appears in: the initial tokenizer corpus
//!   (`UNSUP-001`, `tokenizer/validation/corpus/unsupported_resources.rs`,
//!   plus the transition audit and `tokenizer/validation/tests.rs`),
//!   `tree/tests.rs`, `analysis/tests.rs`, `parser/tests.rs`,
//!   `tree_construction/validation.rs`, the TC-S3/TC-S4 div/section
//!   validation and production leaves, the After-Body validation leaf, the
//!   Data and Title Named validation and production leaves, and
//!   `frontend-analysis-cli/tests/cli_process.rs`. None is touched here; the
//!   later placement review must decide the #112 initial-corpus supersession
//!   and audit that blast radius before implementation.
//!
//! # Deliberately bounded model
//!
//! The model is a unit-driven tokenizer (Data and RCDATA only) plus a small
//! tree. It recognizes only the exact tags `<body>`, `</body>`, `</html>`,
//! `<title>`, and (in RCDATA) `</title>`; every other markup shape is refused
//! as an explicit boundary. It is not a second HTML parser and selects no
//! production state layout, cursor API, diagnostic enum, anchor encoding,
//! resource representation, or mechanism for executing the input-free Numeric
//! End state.
//!
//! Character-reference diagnostics carry a test-local *site*: the unit being
//! dispatched when the diagnostic was raised (or EOF), plus the reference
//! ordinal. The site is a semantic position, not a frozen production anchor.
//!
//! # Separate existing defect
//!
//! Authored Data U+0000 and authored RCDATA U+0000 handling are separate
//! project concerns and are neither corrected nor absorbed. The model refuses
//! authored NUL as outside the candidate. Numeric `&#0;` is a different
//! thing: it is the Numeric End `null-character-reference` replacement.

use crate::{SourceId, SourceText};

const WHATWG_HEAD: &str = "0cd32204c6d9408be0a42cb15c86145e21deab9d";
const WHATWG_SOURCE_BLOB: &str = "05319af4659e15aafb3f5137362b7d9561db8812";
const WPT_CHALLENGE_HEAD: &str = "7108b8dedbf4f7e6e3b21fe82b84f904f529b75f";
const HTML5LIB_CHALLENGE_HEAD: &str = "c777c408b61078ea2eb4acefc2535f54dbc8b28a";

// ---------------------------------------------------------------------------
// Authored evidence and input-stream units
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evidence {
    source_id: SourceId,
    start: usize,
    end: usize,
    raw: String,
}

fn evidence(source: &SourceText, start: usize, end: usize) -> Evidence {
    let anchor = source.anchor(start, end).expect("candidate byte range");
    Evidence {
        source_id: anchor.source_id(),
        start: anchor.range().start(),
        end: anchor.range().end(),
        raw: anchor.fragment().to_owned(),
    }
}

/// One normalized input unit. Authored CR and CRLF become one LF unit that
/// still spans its authored bytes. This is input preprocessing and applies
/// only to authored source, never to decoded output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Unit {
    ch: char,
    start: usize,
    end: usize,
}

fn normalize(text: &str) -> Vec<Unit> {
    let mut units = Vec::new();
    let mut iter = text.char_indices().peekable();
    while let Some((start, ch)) = iter.next() {
        if ch == '\r' {
            if let Some(&(next, '\n')) = iter.peek() {
                iter.next();
                units.push(Unit {
                    ch: '\n',
                    start,
                    end: next + 1,
                });
            } else {
                units.push(Unit {
                    ch: '\n',
                    start,
                    end: start + 1,
                });
            }
        } else {
            units.push(Unit {
                ch,
                start,
                end: start + ch.len_utf8(),
            });
        }
    }
    units
}

fn is_html_whitespace(scalar: char) -> bool {
    matches!(
        scalar,
        '\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}'
    )
}

/// `control-character-in-input-stream`: a control that is not ASCII
/// whitespace and not U+0000 (NUL has its own, unselected, handling).
fn is_input_control_diagnostic(scalar: char) -> bool {
    let code = scalar as u32;
    let c0 = code <= 0x1f && !matches!(scalar, '\0' | '\t' | '\n' | '\u{000c}' | '\r');
    let c1 = (0x7f..=0x9f).contains(&code);
    c0 || c1
}

// ---------------------------------------------------------------------------
// Numeric End: the normative mapping (hand-authored, input-free)
// ---------------------------------------------------------------------------

/// Saturation sentinel. The pinned text compares the accumulated code with
/// 0x10FFFF but defines no integer width. Appending a digit never decreases the
/// code, so clamping at any value above 0x10FFFF preserves the comparison
/// outcome for every digit run, however long. No unbounded host integer is
/// needed.
const SATURATED: u32 = 0x11_0000;

/// The pinned `table-charref-overrides` rows, transcribed verbatim. Code
/// points 0x81, 0x8D, 0x8F, 0x90, and 0x9D have no row and map to themselves.
const C1_OVERRIDES: [(u32, u32); 27] = [
    (0x80, 0x20AC),
    (0x82, 0x201A),
    (0x83, 0x0192),
    (0x84, 0x201E),
    (0x85, 0x2026),
    (0x86, 0x2020),
    (0x87, 0x2021),
    (0x88, 0x02C6),
    (0x89, 0x2030),
    (0x8A, 0x0160),
    (0x8B, 0x2039),
    (0x8C, 0x0152),
    (0x8E, 0x017D),
    (0x91, 0x2018),
    (0x92, 0x2019),
    (0x93, 0x201C),
    (0x94, 0x201D),
    (0x95, 0x2022),
    (0x96, 0x2013),
    (0x97, 0x2014),
    (0x98, 0x02DC),
    (0x99, 0x2122),
    (0x9A, 0x0161),
    (0x9B, 0x203A),
    (0x9C, 0x0153),
    (0x9E, 0x017E),
    (0x9F, 0x0178),
];

fn is_noncharacter(code: u32) -> bool {
    (0xFDD0..=0xFDEF).contains(&code) || (code <= 0x10_FFFF && (code & 0xFFFE) == 0xFFFE)
}

fn is_control(code: u32) -> bool {
    code <= 0x1f || (0x7f..=0x9f).contains(&code)
}

/// Numeric Character Reference End, transcribed bullet by bullet from the
/// pinned text. The bullets are sequential `if`s, so the result is a list; the
/// exhaustive sweep proves the list never holds more than one diagnostic.
fn end_semantics(code: u32) -> (char, Vec<DiagnosticKind>) {
    let mut code = code;
    let mut kinds = Vec::new();
    if code == 0 {
        kinds.push(NULL);
        code = 0xFFFD;
    }
    if code > 0x10_FFFF {
        kinds.push(OUTSIDE);
        code = 0xFFFD;
    }
    if (0xD800..=0xDFFF).contains(&code) {
        kinds.push(SURROGATE);
        code = 0xFFFD;
    }
    if is_noncharacter(code) {
        kinds.push(NONCHARACTER);
    }
    if code == 0x0D || (is_control(code) && !matches!(code, 0x09 | 0x0A | 0x0C | 0x20)) {
        kinds.push(CONTROL_REFERENCE);
        if let Some((_, mapped)) = C1_OVERRIDES.iter().find(|(from, _)| *from == code) {
            code = *mapped;
        }
    }
    (
        char::from_u32(code).expect("Numeric End always yields a scalar"),
        kinds,
    )
}

// ---------------------------------------------------------------------------
// Diagnostics (test-local vocabulary)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagnosticKind {
    ControlCharacterInInputStream,
    MissingSemicolonAfterCharacterReference,
    AbsenceOfDigitsInNumericCharacterReference,
    NullCharacterReference,
    CharacterReferenceOutsideUnicodeRange,
    SurrogateCharacterReference,
    NoncharacterCharacterReference,
    ControlCharacterReference,
}

const INPUT_CONTROL: DiagnosticKind = DiagnosticKind::ControlCharacterInInputStream;
const MISSING: DiagnosticKind = DiagnosticKind::MissingSemicolonAfterCharacterReference;
const ABSENCE: DiagnosticKind = DiagnosticKind::AbsenceOfDigitsInNumericCharacterReference;
const NULL: DiagnosticKind = DiagnosticKind::NullCharacterReference;
const OUTSIDE: DiagnosticKind = DiagnosticKind::CharacterReferenceOutsideUnicodeRange;
const SURROGATE: DiagnosticKind = DiagnosticKind::SurrogateCharacterReference;
const NONCHARACTER: DiagnosticKind = DiagnosticKind::NoncharacterCharacterReference;
const CONTROL_REFERENCE: DiagnosticKind = DiagnosticKind::ControlCharacterReference;

/// The semantic site of a diagnostic: the unit being dispatched when it was
/// raised, or EOF. This is the *emission position*, not a production anchor.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Pos {
    Unit(Evidence),
    Eof(usize),
}

impl Pos {
    fn start(&self) -> usize {
        match self {
            Pos::Unit(evidence) => evidence.start,
            Pos::Eof(offset) => *offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    kind: DiagnosticKind,
    source_id: SourceId,
    /// The character-reference ordinal this diagnostic belongs to, if any.
    reference: Option<usize>,
    at: Pos,
}

// ---------------------------------------------------------------------------
// Tokens and provenance
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Radix {
    Decimal,
    Hexadecimal,
}

impl Radix {
    fn base(self) -> u32 {
        match self {
            Radix::Decimal => 10,
            Radix::Hexadecimal => 16,
        }
    }

    fn digit(self, ch: char) -> Option<u32> {
        ch.to_digit(self.base())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Origin {
    /// Ordinary characters. Interpreted text is the authored text after
    /// input-stream newline normalization.
    Literal,
    /// A consumed reference prefix (`&`, `&#`, `&#x`) flushed literally by a
    /// recovery path. Interpreted text equals authored text.
    FlushedReferencePrefix,
    /// A resolved Numeric reference. Interpreted text is one decoded scalar.
    ResolvedNumeric { radix: Radix },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Contribution {
    origin: Origin,
    authored: Evidence,
    interpreted: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    StartBody(Evidence),
    EndBody(Evidence),
    EndHtml(Evidence),
    StartTitle(Evidence),
    EndTitle(Evidence),
    Characters(Contribution),
    EndOfFile,
}

type Projection = (Origin, usize, usize, String);

/// `(kind, reference ordinal, emission position start)`.
type DiagnosticView = (DiagnosticKind, Option<usize>, usize);

fn lit(start: usize, end: usize, text: &str) -> Projection {
    (Origin::Literal, start, end, text.to_owned())
}

fn flushed(start: usize, end: usize, text: &str) -> Projection {
    (Origin::FlushedReferencePrefix, start, end, text.to_owned())
}

fn dec(start: usize, end: usize, text: &str) -> Projection {
    (
        Origin::ResolvedNumeric {
            radix: Radix::Decimal,
        },
        start,
        end,
        text.to_owned(),
    )
}

fn hex(start: usize, end: usize, text: &str) -> Projection {
    (
        Origin::ResolvedNumeric {
            radix: Radix::Hexadecimal,
        },
        start,
        end,
        text.to_owned(),
    )
}

// ---------------------------------------------------------------------------
// Resource model (test-local; the typed meanings are the accepted ones)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
struct Limits {
    steps: Option<usize>,
    tokens: Option<usize>,
    diagnostics: Option<usize>,
    /// Interpreted bytes of one produced character contribution.
    retained: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Resource {
    TransitionSteps,
    EmittedTokens,
    Diagnostics,
    RetainedInterpretedBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Refusal {
    resource: Resource,
    limit: usize,
    attempted: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Usage {
    steps: usize,
    tokens: usize,
    diagnostics: usize,
    /// The model keeps no copied temporary buffer: the consumed reference
    /// prefix is an authored range, and digits live only in the fixed-size
    /// accumulator. This stays zero for every run.
    peak_temporary_buffer_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outside {
    /// Any `<` shape other than the exact selected tags, including every tag
    /// with attributes (AttributeValue references live there).
    TagShape,
    /// Any RCDATA `<` that is not the appropriate `</title>`.
    RcdataMarkup,
    /// The Named branch of Character Reference: unchanged and not modelled.
    NamedBranch,
    /// Authored Data U+0000: a separate existing defect.
    DataNul,
    /// Authored RCDATA U+0000: unselected recovery.
    RcdataNul,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    Outside(Outside),
    Resource(Refusal),
}

// ---------------------------------------------------------------------------
// Unit-driven Data / RCDATA tokenizer with the Numeric states
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Data,
    Rcdata,
}

/// States that examine a unit. Numeric End is deliberately absent: it
/// examines nothing, so it never appears in a dispatch trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum St {
    Data,
    Rcdata,
    CharacterReference,
    Numeric,
    HexadecimalStart,
    Hexadecimal,
    Decimal,
    /// Tag states, counted by a hand-derived formula that is not the subject
    /// of this candidate.
    Tag,
}

/// `(state, start of the examined unit; None = conceptual EOF)`.
type Dispatch = (St, Option<usize>);

fn at(state: St, start: usize) -> Dispatch {
    (state, Some(start))
}

fn at_eof(state: St) -> Dispatch {
    (state, None)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum At {
    Unit(usize),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Entry(usize),
    Diagnostic(usize),
    Token(usize),
    /// The input-free Numeric End state executed.
    EndEntered(usize),
    /// Numeric End passed its preparation and committed its output.
    EndCommitted(usize),
}

/// Fixed-size digit accumulation. No digit string is ever copied.
#[derive(Debug, Clone, Copy)]
struct Accumulator {
    radix: Radix,
    code: u32,
}

impl Accumulator {
    fn new(radix: Radix) -> Self {
        Self { radix, code: 0 }
    }

    fn push(&mut self, digit: u32) {
        self.code = (self.code * self.radix.base() + digit).min(SATURATED);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Lexed {
    source_id: SourceId,
    source_text: String,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    events: Vec<Event>,
    entries: Vec<Evidence>,
    trace: Vec<Dispatch>,
    /// Entries whose Numeric End committed output.
    finalized: Vec<usize>,
    /// Entries whose Numeric End state executed at all.
    end_entered: Vec<usize>,
    usage: Usage,
    coverage_end: usize,
    stop: Option<Stop>,
}

impl Lexed {
    fn contributions(&self) -> Vec<&Contribution> {
        self.tokens
            .iter()
            .filter_map(|token| match token {
                Token::Characters(contribution) => Some(contribution),
                _ => None,
            })
            .collect()
    }

    fn interpreted(&self) -> String {
        self.contributions()
            .iter()
            .map(|contribution| contribution.interpreted.as_str())
            .collect()
    }

    fn projection(&self) -> Vec<Projection> {
        self.contributions()
            .iter()
            .map(|c| {
                (
                    c.origin.clone(),
                    c.authored.start,
                    c.authored.end,
                    c.interpreted.clone(),
                )
            })
            .collect()
    }

    fn diagnostic_kinds(&self) -> Vec<DiagnosticKind> {
        self.diagnostics.iter().map(|d| d.kind).collect()
    }

    /// `(kind, reference ordinal, emission position start)`.
    fn diagnostic_view(&self) -> Vec<DiagnosticView> {
        self.diagnostics
            .iter()
            .map(|d| (d.kind, d.reference, d.at.start()))
            .collect()
    }

    fn reached_eof(&self) -> bool {
        self.tokens.last() == Some(&Token::EndOfFile)
    }

    fn count_tokens(&self, predicate: impl Fn(&Token) -> bool) -> usize {
        self.tokens.iter().filter(|token| predicate(token)).count()
    }

    fn has_numeric_output(&self) -> bool {
        self.contributions()
            .iter()
            .any(|c| matches!(c.origin, Origin::ResolvedNumeric { .. }))
    }
}

struct Machine {
    source: SourceText,
    units: Vec<Unit>,
    idx: usize,
    preprocessed: usize,
    ctx: Ctx,
    return_override: Option<Ctx>,
    limits: Limits,
    usage: Usage,
    /// End of the last consumed unit. Reference spans are cut here and never
    /// from `coverage_end`, which a diagnostic site can lead.
    consumed_end: usize,
    /// Processed coverage: the consumed prefix, and through the site of every
    /// successfully committed diagnostic.
    coverage_end: usize,
    last: At,
    run_start: Option<usize>,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    events: Vec<Event>,
    entries: Vec<Evidence>,
    trace: Vec<Dispatch>,
    finalized: Vec<usize>,
    end_entered: Vec<usize>,
}

fn lex(text: &str) -> Lexed {
    lex_ctx(text, Ctx::Data)
}

fn lex_ctx(text: &str, ctx: Ctx) -> Lexed {
    lex_with(
        &SourceText::new(SourceId::new(1), text.to_owned()),
        ctx,
        Limits::default(),
        None,
    )
}

fn lex_limited(text: &str, ctx: Ctx, limits: Limits) -> Lexed {
    lex_with(
        &SourceText::new(SourceId::new(1), text.to_owned()),
        ctx,
        limits,
        None,
    )
}

fn lex_with(source: &SourceText, ctx: Ctx, limits: Limits, return_override: Option<Ctx>) -> Lexed {
    let mut machine = Machine {
        source: source.clone(),
        units: normalize(source.as_str()),
        idx: 0,
        preprocessed: 0,
        ctx,
        return_override,
        limits,
        usage: Usage::default(),
        consumed_end: 0,
        coverage_end: 0,
        last: At::Eof,
        run_start: None,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
        events: Vec::new(),
        entries: Vec::new(),
        trace: Vec::new(),
        finalized: Vec::new(),
        end_entered: Vec::new(),
    };
    let stop = machine.run().err();
    Lexed {
        source_id: machine.source.id(),
        source_text: machine.source.as_str().to_owned(),
        tokens: machine.tokens,
        diagnostics: machine.diagnostics,
        events: machine.events,
        entries: machine.entries,
        trace: machine.trace,
        finalized: machine.finalized,
        end_entered: machine.end_entered,
        usage: machine.usage,
        coverage_end: machine.coverage_end,
        stop,
    }
}

impl Machine {
    fn pos(&self, at: At) -> Pos {
        match at {
            At::Unit(index) => {
                let unit = self.units[index];
                Pos::Unit(evidence(&self.source, unit.start, unit.end))
            }
            At::Eof => Pos::Eof(self.source.as_str().len()),
        }
    }

    fn check(
        &self,
        resource: Resource,
        used: usize,
        add: usize,
        limit: Option<usize>,
    ) -> Result<(), Stop> {
        match limit {
            Some(limit) if used + add > limit => Err(Stop::Resource(Refusal {
                resource,
                limit,
                attempted: used + add,
            })),
            _ => Ok(()),
        }
    }

    /// Every fallible preparation of one semantic effect, before any of it is
    /// committed. Nothing is mutated here.
    fn preflight(&self, diagnostics: usize, tokens: usize, retained: usize) -> Result<(), Stop> {
        self.check(
            Resource::Diagnostics,
            self.usage.diagnostics,
            diagnostics,
            self.limits.diagnostics,
        )?;
        self.check(
            Resource::EmittedTokens,
            self.usage.tokens,
            tokens,
            self.limits.tokens,
        )?;
        self.check(
            Resource::RetainedInterpretedBytes,
            0,
            retained,
            self.limits.retained,
        )
    }

    /// A committed diagnostic is evidence that its authored site was observed
    /// and explained, so processed coverage reaches the end of that site. A
    /// refused diagnostic never reaches this function.
    fn commit_diagnostic(&mut self, kind: DiagnosticKind, reference: Option<usize>, at: Pos) {
        let site_end = match &at {
            Pos::Unit(evidence) => evidence.end,
            Pos::Eof(offset) => *offset,
        };
        self.coverage_end = self.coverage_end.max(site_end);
        self.diagnostics.push(Diagnostic {
            kind,
            source_id: self.source.id(),
            reference,
            at,
        });
        self.usage.diagnostics += 1;
        self.events
            .push(Event::Diagnostic(self.diagnostics.len() - 1));
    }

    fn push_diagnostic(
        &mut self,
        kind: DiagnosticKind,
        reference: Option<usize>,
        at: Pos,
    ) -> Result<(), Stop> {
        self.preflight(1, 0, 0)?;
        self.commit_diagnostic(kind, reference, at);
        Ok(())
    }

    fn commit_token(&mut self, token: Token) {
        self.tokens.push(token);
        self.usage.tokens += 1;
        self.events.push(Event::Token(self.tokens.len() - 1));
    }

    fn push_token(&mut self, token: Token) -> Result<(), Stop> {
        let retained = match &token {
            Token::Characters(contribution) => contribution.interpreted.len(),
            _ => 0,
        };
        self.preflight(0, 1, retained)?;
        self.commit_token(token);
        Ok(())
    }

    fn contribution(
        &self,
        origin: Origin,
        start: usize,
        end: usize,
        interpreted: String,
    ) -> Contribution {
        Contribution {
            origin,
            authored: evidence(&self.source, start, end),
            interpreted,
        }
    }

    /// One transition: examine the current unit (or EOF) under `state`.
    ///
    /// Lifecycle, in order: a newly materialized authored unit is input
    /// preprocessed (once; a reconsume of the same unit never repeats it), and
    /// its diagnostic, if any, is prepared and committed; only then is the
    /// transition attempted against `TransitionSteps`. A step refusal can
    /// therefore follow an already-committed preprocessing diagnostic, while a
    /// refused preprocessing diagnostic stops before any transition is
    /// attempted. An attempted transition rejected by the limit does not
    /// increment committed usage.
    fn dispatch(&mut self, state: St) -> Result<Option<Unit>, Stop> {
        let unit = self.units.get(self.idx).copied();
        if let Some(unit) = unit
            && self.idx >= self.preprocessed
        {
            if is_input_control_diagnostic(unit.ch) {
                let site = self.pos(At::Unit(self.idx));
                self.push_diagnostic(INPUT_CONTROL, None, site)?;
            }
            self.preprocessed = self.idx + 1;
        }
        if let Some(limit) = self.limits.steps
            && self.usage.steps >= limit
        {
            return Err(Stop::Resource(Refusal {
                resource: Resource::TransitionSteps,
                limit,
                attempted: self.usage.steps + 1,
            }));
        }
        self.usage.steps += 1;
        self.trace.push((state, unit.map(|u| u.start)));
        self.last = match unit {
            Some(_) => At::Unit(self.idx),
            None => At::Eof,
        };
        Ok(unit)
    }

    fn consume(&mut self, unit: Unit) {
        self.idx += 1;
        self.consumed_end = unit.end;
        self.coverage_end = self.coverage_end.max(unit.end);
    }

    fn matches_after_lt(&self, pattern: &str) -> bool {
        pattern.chars().enumerate().all(|(offset, expected)| {
            self.units.get(self.idx + 1 + offset).map(|u| u.ch) == Some(expected)
        })
    }

    fn flush_run(&mut self) -> Result<(), Stop> {
        let Some(first) = self.run_start.take() else {
            return Ok(());
        };
        let last = self.idx - 1;
        let start = self.units[first].start;
        let end = self.units[last].end;
        let interpreted: String = self.units[first..=last].iter().map(|u| u.ch).collect();
        let contribution = self.contribution(Origin::Literal, start, end, interpreted);
        self.push_token(Token::Characters(contribution))
    }

    fn run(&mut self) -> Result<(), Stop> {
        loop {
            let ctx = self.ctx;
            let state = match ctx {
                Ctx::Data => St::Data,
                Ctx::Rcdata => St::Rcdata,
            };
            let Some(unit) = self.dispatch(state)? else {
                self.flush_run()?;
                self.push_token(Token::EndOfFile)?;
                return Ok(());
            };
            match unit.ch {
                '&' => {
                    self.flush_run()?;
                    self.reference(unit, ctx)?;
                }
                '<' => {
                    self.flush_run()?;
                    self.tag(unit, ctx)?;
                }
                '\0' => {
                    self.flush_run()?;
                    return Err(Stop::Outside(match ctx {
                        Ctx::Data => Outside::DataNul,
                        Ctx::Rcdata => Outside::RcdataNul,
                    }));
                }
                _ => {
                    self.consume(unit);
                    if self.run_start.is_none() {
                        self.run_start = Some(self.idx - 1);
                    }
                }
            }
        }
    }

    /// Exact selected tags only. Step charges are the hand-derived tag-state
    /// dispatches: the opening state examines the first name letter without
    /// consuming it and the name state then reconsumes it.
    fn tag(&mut self, lt: Unit, ctx: Ctx) -> Result<(), Stop> {
        let (pattern, closing, make): (&str, bool, fn(Evidence) -> Token) = match ctx {
            Ctx::Data if self.matches_after_lt("body>") => ("body>", false, Token::StartBody),
            Ctx::Data if self.matches_after_lt("/body>") => ("/body>", true, Token::EndBody),
            Ctx::Data if self.matches_after_lt("/html>") => ("/html>", true, Token::EndHtml),
            Ctx::Data if self.matches_after_lt("title>") => ("title>", false, Token::StartTitle),
            Ctx::Rcdata if self.matches_after_lt("/title>") => ("/title>", true, Token::EndTitle),
            Ctx::Data => return Err(Stop::Outside(Outside::TagShape)),
            Ctx::Rcdata => return Err(Stop::Outside(Outside::RcdataMarkup)),
        };
        self.consume(lt);
        let name_start = usize::from(closing);
        let mut last_end = lt.end;
        for index in 0..pattern.len() {
            if index == name_start {
                self.dispatch(St::Tag)?;
            }
            let unit = self.dispatch(St::Tag)?.expect("matched tag unit");
            self.consume(unit);
            last_end = unit.end;
        }
        let token = make(evidence(&self.source, lt.start, last_end));
        // Tree-directed tokenizer feedback for the selected Title lifecycle.
        let next_ctx = match &token {
            Token::StartTitle(_) => Ctx::Rcdata,
            Token::EndTitle(_) => Ctx::Data,
            _ => ctx,
        };
        self.push_token(token)?;
        self.ctx = next_ctx;
        Ok(())
    }

    /// Return state is `ret`; the `&` was examined by that state and is
    /// consumed here. Character Reference examines the next unit.
    fn reference(&mut self, amp: Unit, ret: Ctx) -> Result<(), Stop> {
        let ret = self.return_override.unwrap_or(ret);
        let amp_idx = self.idx;
        self.consume(amp);
        self.entries
            .push(evidence(&self.source, amp.start, amp.end));
        let entry = self.entries.len() - 1;
        self.events.push(Event::Entry(entry));

        match self.dispatch(St::CharacterReference)? {
            Some(hash) if hash.ch == '#' => {
                self.consume(hash);
                self.numeric(entry, amp_idx, ret)
            }
            Some(next) if next.ch.is_ascii_alphanumeric() => {
                Err(Stop::Outside(Outside::NamedBranch))
            }
            _ => {
                self.flush_prefix(amp_idx, None)?;
                self.ctx = ret;
                Ok(())
            }
        }
    }

    fn numeric(&mut self, entry: usize, amp_idx: usize, ret: Ctx) -> Result<(), Stop> {
        let (radix, digit_state) = match self.dispatch(St::Numeric)? {
            Some(x) if x.ch == 'x' || x.ch == 'X' => {
                self.consume(x);
                match self.dispatch(St::HexadecimalStart)? {
                    Some(digit) if digit.ch.is_ascii_hexdigit() => {
                        (Radix::Hexadecimal, St::Hexadecimal)
                    }
                    _ => return self.absence_of_digits(entry, amp_idx, ret),
                }
            }
            Some(digit) if digit.ch.is_ascii_digit() => (Radix::Decimal, St::Decimal),
            _ => return self.absence_of_digits(entry, amp_idx, ret),
        };

        let mut accumulator = Accumulator::new(radix);
        loop {
            // The first pass is the reconsume examination of the digit that
            // the previous state looked at without consuming.
            let unit = self.dispatch(digit_state)?;
            let digit = unit.and_then(|u| radix.digit(u.ch).map(|value| (u, value)));
            match (unit, digit) {
                (_, Some((unit, value))) => {
                    accumulator.push(value);
                    self.consume(unit);
                }
                (Some(semicolon), None) if semicolon.ch == ';' => {
                    self.consume(semicolon);
                    break;
                }
                _ => {
                    // Reconsume in Numeric End: the terminator is not consumed.
                    let site = self.pos(self.last);
                    self.push_diagnostic(MISSING, Some(entry), site)?;
                    break;
                }
            }
        }
        self.numeric_end(entry, amp_idx, accumulator, ret)
    }

    fn absence_of_digits(&mut self, entry: usize, amp_idx: usize, ret: Ctx) -> Result<(), Stop> {
        self.flush_prefix(amp_idx, Some((ABSENCE, entry)))?;
        self.ctx = ret;
        Ok(())
    }

    /// Flush the consumed reference prefix literally, with an optional
    /// diagnostic that is raised only if the whole effect can be prepared.
    fn flush_prefix(
        &mut self,
        amp_idx: usize,
        diagnostic: Option<(DiagnosticKind, usize)>,
    ) -> Result<(), Stop> {
        let start = self.units[amp_idx].start;
        let end = self.consumed_end;
        let raw = self.source.as_str()[start..end].to_owned();
        let contribution = self.contribution(Origin::FlushedReferencePrefix, start, end, raw);
        self.preflight(
            usize::from(diagnostic.is_some()),
            1,
            contribution.interpreted.len(),
        )?;
        if let Some((kind, entry)) = diagnostic {
            let site = self.pos(self.last);
            self.commit_diagnostic(kind, Some(entry), site);
        }
        self.commit_token(Token::Characters(contribution));
        Ok(())
    }

    /// Numeric Character Reference End. It consumes no input and examines no
    /// unit; it prepares every fallible effect, then commits non-refusingly.
    fn numeric_end(
        &mut self,
        entry: usize,
        amp_idx: usize,
        accumulator: Accumulator,
        ret: Ctx,
    ) -> Result<(), Stop> {
        self.end_entered.push(entry);
        self.events.push(Event::EndEntered(entry));
        let (scalar, kinds) = end_semantics(accumulator.code);
        let start = self.units[amp_idx].start;
        let end = self.consumed_end;
        let contribution = self.contribution(
            Origin::ResolvedNumeric {
                radix: accumulator.radix,
            },
            start,
            end,
            scalar.to_string(),
        );
        self.preflight(kinds.len(), 1, contribution.interpreted.len())?;

        // Non-refusing commit: no `Result` below this line.
        self.finalized.push(entry);
        self.events.push(Event::EndCommitted(entry));
        for kind in kinds {
            let site = self.pos(self.last);
            self.commit_diagnostic(kind, Some(entry), site);
        }
        self.commit_token(Token::Characters(contribution));
        self.ctx = ret;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Bounded tree over the existing selected document-tree positions
// ---------------------------------------------------------------------------

/// The existing selected positions only. `Early` collapses the unsupported
/// early positions for character data. No insertion mode is introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Early,
    InHead,
    TitleText,
    InBody,
    AfterBody,
    AfterAfterBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Recovery {
    /// A non-whitespace scalar in After Body switched to In Body and was
    /// reprocessed there. Classified from the interpreted scalar only.
    AfterBodyNonWhitespaceToInBody { scalar: char },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextNode {
    text: String,
    contributions: Vec<Contribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Title {
    start: Evidence,
    node: Option<TextNode>,
    close: Option<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Body {
    start: Evidence,
    text_nodes: Vec<TextNode>,
    close: Option<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tree {
    /// Source-less `html` and `head` synthesized by the existing shell rules.
    shell_synthesized: bool,
    titles: Vec<Title>,
    body: Option<Body>,
    html_close: Option<Evidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreeBoundary {
    /// Character data before the body exists: early whitespace-sensitive
    /// positions stay unsupported.
    CharacterDataBeforeBody,
    /// Character data after `</html>`: the existing AfterAfterBody boundary.
    CharacterDataInAfterAfterBody,
    /// EOF inside Title text is the existing explicit non-complete checkpoint.
    EofInTitleText,
    OutsideModelledCells,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Unsupported {
    Outside(Outside),
    Tree(TreeBoundary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Completion {
    Complete,
    ResourceLimit(Refusal),
    Unsupported(Unsupported),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    lexed: Lexed,
    tree: Tree,
    mode: Mode,
    recoveries: Vec<Recovery>,
    completion: Completion,
}

fn append(node: &mut Option<TextNode>, contribution: &Contribution) {
    match node {
        Some(existing) => {
            existing.text.push_str(&contribution.interpreted);
            existing.contributions.push(contribution.clone());
        }
        None => {
            *node = Some(TextNode {
                text: contribution.interpreted.clone(),
                contributions: vec![contribution.clone()],
            });
        }
    }
}

fn insert_body(tree: &mut Tree, contribution: &Contribution) {
    let body = tree.body.as_mut().expect("body");
    match body.text_nodes.last_mut() {
        Some(node) => {
            node.text.push_str(&contribution.interpreted);
            node.contributions.push(contribution.clone());
        }
        None => body.text_nodes.push(TextNode {
            text: contribution.interpreted.clone(),
            contributions: vec![contribution.clone()],
        }),
    }
}

fn construct(lexed: Lexed) -> Observation {
    let mut tree = Tree {
        shell_synthesized: false,
        titles: Vec::new(),
        body: None,
        html_close: None,
    };
    let mut mode = Mode::Early;
    let mut recoveries = Vec::new();
    let mut refusal: Option<TreeBoundary> = None;

    for token in &lexed.tokens {
        let outcome = match (mode, token) {
            (Mode::Early | Mode::InHead, Token::StartTitle(start)) => {
                tree.shell_synthesized = true;
                tree.titles.push(Title {
                    start: start.clone(),
                    node: None,
                    close: None,
                });
                mode = Mode::TitleText;
                Ok(())
            }
            (Mode::TitleText, Token::Characters(contribution)) => {
                let title = tree.titles.last_mut().expect("title");
                append(&mut title.node, contribution);
                Ok(())
            }
            (Mode::TitleText, Token::EndTitle(close)) => {
                tree.titles.last_mut().expect("title").close = Some(close.clone());
                mode = Mode::InHead;
                Ok(())
            }
            (Mode::TitleText, Token::EndOfFile) => Err(TreeBoundary::EofInTitleText),
            (Mode::Early | Mode::InHead, Token::StartBody(start)) => {
                tree.shell_synthesized = true;
                tree.body = Some(Body {
                    start: start.clone(),
                    text_nodes: Vec::new(),
                    close: None,
                });
                mode = Mode::InBody;
                Ok(())
            }
            (Mode::Early | Mode::InHead, Token::Characters(_)) => {
                Err(TreeBoundary::CharacterDataBeforeBody)
            }
            (Mode::InHead, Token::EndOfFile) => Ok(()),
            (Mode::InBody, Token::Characters(contribution)) => {
                insert_body(&mut tree, contribution);
                Ok(())
            }
            (Mode::InBody, Token::EndBody(close)) => {
                tree.body.as_mut().expect("body").close = Some(close.clone());
                mode = Mode::AfterBody;
                Ok(())
            }
            (Mode::InBody | Mode::AfterBody, Token::EndOfFile) => Ok(()),
            (Mode::AfterBody, Token::Characters(contribution)) => {
                for scalar in contribution.interpreted.chars() {
                    if mode == Mode::AfterBody && !is_html_whitespace(scalar) {
                        recoveries.push(Recovery::AfterBodyNonWhitespaceToInBody { scalar });
                        mode = Mode::InBody;
                    }
                }
                insert_body(&mut tree, contribution);
                Ok(())
            }
            (Mode::AfterBody, Token::EndHtml(close)) => {
                tree.html_close = Some(close.clone());
                mode = Mode::AfterAfterBody;
                Ok(())
            }
            (Mode::AfterAfterBody, Token::EndOfFile) => Ok(()),
            (Mode::AfterAfterBody, Token::Characters(_)) => {
                Err(TreeBoundary::CharacterDataInAfterAfterBody)
            }
            _ => Err(TreeBoundary::OutsideModelledCells),
        };
        if let Err(boundary) = outcome {
            refusal = Some(boundary);
            break;
        }
    }

    let completion = match (refusal, lexed.stop) {
        (Some(boundary), _) => Completion::Unsupported(Unsupported::Tree(boundary)),
        (None, Some(Stop::Outside(outside))) => {
            Completion::Unsupported(Unsupported::Outside(outside))
        }
        (None, Some(Stop::Resource(refusal))) => Completion::ResourceLimit(refusal),
        (None, None) => Completion::Complete,
    };

    Observation {
        lexed,
        tree,
        mode,
        recoveries,
        completion,
    }
}

fn run(text: &str) -> Observation {
    construct(lex(text))
}

fn run_limited(text: &str, limits: Limits) -> Observation {
    construct(lex_limited(text, Ctx::Data, limits))
}

impl Observation {
    fn body_text(&self) -> String {
        self.tree
            .body
            .as_ref()
            .map(|body| body.text_nodes.iter().map(|n| n.text.as_str()).collect())
            .unwrap_or_default()
    }

    fn title_text(&self) -> String {
        self.tree
            .titles
            .iter()
            .filter_map(|title| title.node.as_ref())
            .map(|node| node.text.as_str())
            .collect()
    }

    fn body_text_node_count(&self) -> usize {
        self.tree
            .body
            .as_ref()
            .map_or(0, |body| body.text_nodes.len())
    }

    fn placed_contributions(&self) -> Vec<&Contribution> {
        let titles = self
            .tree
            .titles
            .iter()
            .filter_map(|title| title.node.as_ref())
            .flat_map(|node| node.contributions.iter());
        let body = self
            .tree
            .body
            .iter()
            .flat_map(|body| body.text_nodes.iter())
            .flat_map(|node| node.contributions.iter());
        titles.chain(body).collect()
    }
}

// ---------------------------------------------------------------------------
// Freeze validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum FreezeError {
    CompleteWithoutFullCoverage,
    NonCompleteHasEndOfFile,
    ContributionOutOfOrderOrOverlapping,
    ContributionRawMismatch,
    ContributionBeyondCoverage,
    LiteralInterpretationMismatch,
    NumericShapeMismatch,
    NumericOutputWithoutFinalization,
    DiagnosticOrderRegression,
    DiagnosticReferenceOutOfRange,
    DiagnosticSourceIdentityMismatch,
    DiagnosticBeyondCoverage,
    TreeContributionNotFromLexed,
    TextNodeDoesNotEqualContributions,
}

/// Independent lexical-shape check: `&#` `x|X`? digits `;`?
fn numeric_shape(raw: &str, radix: Radix) -> bool {
    let Some(rest) = raw.strip_prefix("&#") else {
        return false;
    };
    let (marker_ok, rest) = match rest.strip_prefix(['x', 'X']) {
        Some(after) => (radix == Radix::Hexadecimal, after),
        None => (radix == Radix::Decimal, rest),
    };
    let digits = rest.strip_suffix(';').unwrap_or(rest);
    marker_ok && !digits.is_empty() && digits.chars().all(|c| radix.digit(c).is_some())
}

/// End of a diagnostic's authored site; a zero-width EOF site ends at EOF.
fn diagnostic_site_end(diagnostic: &Diagnostic) -> usize {
    match &diagnostic.at {
        Pos::Unit(evidence) => evidence.end,
        Pos::Eof(offset) => *offset,
    }
}

fn validate_freeze(observation: &Observation) -> Result<(), FreezeError> {
    let lexed = &observation.lexed;
    let complete = observation.completion == Completion::Complete;
    if complete && (lexed.coverage_end != lexed.source_text.len() || !lexed.reached_eof()) {
        return Err(FreezeError::CompleteWithoutFullCoverage);
    }
    if lexed.stop.is_some() && lexed.reached_eof() {
        return Err(FreezeError::NonCompleteHasEndOfFile);
    }

    let mut previous_end = 0usize;
    let mut numeric_outputs = 0usize;
    for contribution in lexed.contributions() {
        let span = &contribution.authored;
        if span.start < previous_end || span.start >= span.end {
            return Err(FreezeError::ContributionOutOfOrderOrOverlapping);
        }
        previous_end = span.end;
        if span.end > lexed.coverage_end {
            return Err(FreezeError::ContributionBeyondCoverage);
        }
        if lexed.source_text[span.start..span.end] != span.raw {
            return Err(FreezeError::ContributionRawMismatch);
        }
        match contribution.origin {
            Origin::Literal => {
                let normalized: String = normalize(&span.raw).iter().map(|u| u.ch).collect();
                if contribution.interpreted != normalized {
                    return Err(FreezeError::LiteralInterpretationMismatch);
                }
            }
            Origin::FlushedReferencePrefix => {
                if contribution.interpreted != span.raw {
                    return Err(FreezeError::LiteralInterpretationMismatch);
                }
            }
            Origin::ResolvedNumeric { radix } => {
                numeric_outputs += 1;
                if !numeric_shape(&span.raw, radix) || contribution.interpreted.chars().count() != 1
                {
                    return Err(FreezeError::NumericShapeMismatch);
                }
            }
        }
    }
    if numeric_outputs != lexed.finalized.len() {
        return Err(FreezeError::NumericOutputWithoutFinalization);
    }

    let mut previous_start = 0usize;
    for diagnostic in &lexed.diagnostics {
        if diagnostic.at.start() < previous_start {
            return Err(FreezeError::DiagnosticOrderRegression);
        }
        previous_start = diagnostic.at.start();
        let site_source_id = match &diagnostic.at {
            Pos::Unit(evidence) => evidence.source_id,
            Pos::Eof(_) => diagnostic.source_id,
        };
        if diagnostic.source_id != lexed.source_id || site_source_id != lexed.source_id {
            return Err(FreezeError::DiagnosticSourceIdentityMismatch);
        }
        if diagnostic_site_end(diagnostic) > lexed.coverage_end {
            return Err(FreezeError::DiagnosticBeyondCoverage);
        }
        if diagnostic
            .reference
            .is_some_and(|reference| reference >= lexed.entries.len())
        {
            return Err(FreezeError::DiagnosticReferenceOutOfRange);
        }
    }

    let lexed_contributions = lexed.contributions();
    let placed = observation.placed_contributions();
    let mut next_lexed = 0usize;
    for placed_contribution in &placed {
        while next_lexed < lexed_contributions.len()
            && lexed_contributions[next_lexed] != *placed_contribution
        {
            next_lexed += 1;
        }
        if next_lexed == lexed_contributions.len() {
            return Err(FreezeError::TreeContributionNotFromLexed);
        }
        next_lexed += 1;
    }
    if complete && placed.len() != lexed_contributions.len() {
        return Err(FreezeError::TreeContributionNotFromLexed);
    }

    let nodes = observation
        .tree
        .titles
        .iter()
        .filter_map(|title| title.node.as_ref())
        .chain(
            observation
                .tree
                .body
                .iter()
                .flat_map(|body| body.text_nodes.iter()),
        );
    for node in nodes {
        let joined: String = node
            .contributions
            .iter()
            .map(|c| c.interpreted.as_str())
            .collect();
        if joined != node.text {
            return Err(FreezeError::TextNodeDoesNotEqualContributions);
        }
    }
    Ok(())
}

fn assert_valid(observation: &Observation) {
    assert_eq!(validate_freeze(observation), Ok(()));
}

// ---------------------------------------------------------------------------
// Falsification probes: deliberately wrong designs the gold must reject
// ---------------------------------------------------------------------------

/// Probe: Numeric collapsed into one Named-style whole-reference transition.
/// `&` plus `#` plus one lump for the entire remainder, with no per-digit step.
fn named_style_numeric_steps(_digits: usize) -> usize {
    3
}

/// Probe: overflow decided by the number of digits.
fn length_based_overflow(digits: &str, radix: Radix) -> bool {
    digits.len() > if radix == Radix::Decimal { 7 } else { 6 }
}

/// Probe: an unchecked 32-bit accumulator that wraps.
fn wrapping_u32_decode(digits: &str, radix: Radix) -> u32 {
    digits.chars().fold(0u32, |code, ch| {
        code.wrapping_mul(radix.base())
            .wrapping_add(radix.digit(ch).expect("digit"))
    })
}

/// Probe: a host integer parse that fails on overflow and is then treated as
/// "not a reference".
fn host_parse_decode(digits: &str, radix: Radix) -> Option<char> {
    u32::from_str_radix(digits, radix.base())
        .ok()
        .and_then(char::from_u32)
}

/// Probe: decoded output re-enters the tokenizer.
fn retokenized_end_body_count(text: &str) -> usize {
    let decoded = lex(text).interpreted();
    lex(&decoded).count_tokens(|token| matches!(token, Token::EndBody(_)))
}

/// Probe: decoded output is run through authored CR/CRLF preprocessing again.
fn renormalized(text: &str) -> String {
    normalize(text).iter().map(|u| u.ch).collect()
}

/// Probe: a recovery that treats "no digits" as the numeric value zero.
fn absence_as_zero(text: &str) -> Option<(char, Vec<DiagnosticKind>)> {
    (text.starts_with("&#") && !text[2..].starts_with(|c: char| c.is_ascii_digit()))
        .then(|| end_semantics(0))
}

/// Probe: anchor every character-reference diagnostic at the reference start.
fn reference_start_anchored_starts(lexed: &Lexed) -> Vec<usize> {
    lexed
        .diagnostics
        .iter()
        .map(|diagnostic| match diagnostic.reference {
            Some(reference) => lexed.entries[reference].start,
            None => diagnostic.at.start(),
        })
        .collect()
}

/// Probe: classify decoded whitespace by authored spelling.
fn spelling_based_is_whitespace(raw: &str) -> bool {
    !raw.is_empty() && !raw.starts_with('&') && raw.chars().all(is_html_whitespace)
}

fn nondecreasing(values: &[usize]) -> bool {
    values.windows(2).all(|pair| pair[0] <= pair[1])
}

/// Independent overflow oracle: strip leading zeros, then compare by length
/// and a small integer. It shares no code with the saturating accumulator.
fn exact_or_exceeds(digits: &str, radix: Radix) -> Option<u32> {
    let trimmed = digits.trim_start_matches('0');
    let max_len = if radix == Radix::Decimal { 7 } else { 6 };
    if trimmed.len() > max_len {
        return None;
    }
    if trimmed.is_empty() {
        return Some(0);
    }
    let value = u64::from_str_radix(trimmed, radix.base()).expect("short digit run");
    (value <= 0x10_FFFF).then_some(value as u32)
}

// ---------------------------------------------------------------------------
// Numeric End semantics
// ---------------------------------------------------------------------------

#[test]
fn e1_numeric_end_matches_the_pinned_bullets_for_every_required_value() {
    // (source, decoded scalar, required diagnostic)
    let cases: &[(&str, char, Option<DiagnosticKind>)] = &[
        ("&#0;", '\u{fffd}', Some(NULL)),
        ("&#x0;", '\u{fffd}', Some(NULL)),
        ("&#xD800;", '\u{fffd}', Some(SURROGATE)),
        ("&#xDFFF;", '\u{fffd}', Some(SURROGATE)),
        ("&#55296;", '\u{fffd}', Some(SURROGATE)),
        ("&#1114112;", '\u{fffd}', Some(OUTSIDE)),
        ("&#x110000;", '\u{fffd}', Some(OUTSIDE)),
        ("&#xFDD0;", '\u{fdd0}', Some(NONCHARACTER)),
        ("&#xFDEF;", '\u{fdef}', Some(NONCHARACTER)),
        ("&#xFFFE;", '\u{fffe}', Some(NONCHARACTER)),
        ("&#xFFFF;", '\u{ffff}', Some(NONCHARACTER)),
        ("&#x1FFFE;", '\u{1fffe}', Some(NONCHARACTER)),
        ("&#x10FFFE;", '\u{10fffe}', Some(NONCHARACTER)),
        ("&#x10FFFF;", '\u{10ffff}', Some(NONCHARACTER)),
        ("&#128;", '\u{20ac}', Some(CONTROL_REFERENCE)),
        ("&#x80;", '\u{20ac}', Some(CONTROL_REFERENCE)),
        ("&#159;", '\u{0178}', Some(CONTROL_REFERENCE)),
        ("&#129;", '\u{0081}', Some(CONTROL_REFERENCE)),
        ("&#13;", '\u{000d}', Some(CONTROL_REFERENCE)),
        ("&#1;", '\u{0001}', Some(CONTROL_REFERENCE)),
        ("&#127;", '\u{007f}', Some(CONTROL_REFERENCE)),
        ("&#9;", '\u{0009}', None),
        ("&#10;", '\u{000a}', None),
        ("&#12;", '\u{000c}', None),
        ("&#32;", '\u{0020}', None),
        ("&#160;", '\u{00a0}', None),
        ("&#65;", 'A', None),
        ("&#xD7FF;", '\u{d7ff}', None),
        ("&#xE000;", '\u{e000}', None),
        ("&#xFDCF;", '\u{fdcf}', None),
        ("&#xFDF0;", '\u{fdf0}', None),
        ("&#x1F600;", '\u{1f600}', None),
        ("&#x10FFFD;", '\u{10fffd}', None),
    ];
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        for &(text, scalar, kind) in cases {
            let lexed = lex_ctx(text, ctx);
            let radix = if text.contains(['x', 'X']) {
                Radix::Hexadecimal
            } else {
                Radix::Decimal
            };
            let expected = (
                Origin::ResolvedNumeric { radix },
                0,
                text.len(),
                scalar.to_string(),
            );
            assert_eq!(lexed.projection(), vec![expected], "{text} in {ctx:?}");
            // The End diagnostic is raised while the `;` is the unit last
            // dispatched.
            let expected_diagnostics: Vec<_> = kind
                .into_iter()
                .map(|k| (k, Some(0), text.len() - 1))
                .collect();
            assert_eq!(lexed.diagnostic_view(), expected_diagnostics, "{text}");
            assert!(lexed.stop.is_none() && lexed.reached_eof(), "{text}");
            assert_eq!(lexed.finalized, vec![0]);
        }
    }
}

#[test]
fn e2_every_c1_row_and_hole_is_covered_from_the_transcribed_table() {
    let mut mapped = 0;
    for code in 0x80u32..=0x9f {
        let expected = C1_OVERRIDES
            .iter()
            .find(|(from, _)| *from == code)
            .map_or(code, |(_, to)| *to);
        if expected != code {
            mapped += 1;
        }
        for text in [format!("&#{code};"), format!("&#x{code:X};")] {
            let lexed = lex(&text);
            assert_eq!(
                lexed.interpreted(),
                char::from_u32(expected).unwrap().to_string(),
                "{text}"
            );
            assert_eq!(lexed.diagnostic_kinds(), vec![CONTROL_REFERENCE], "{text}");
        }
    }
    assert_eq!(mapped, 27, "27 transcribed override rows");
    assert_eq!(C1_OVERRIDES.len(), 27);
    // The holes map to themselves while still being control references.
    for hole in [0x81u32, 0x8d, 0x8f, 0x90, 0x9d] {
        assert!(C1_OVERRIDES.iter().all(|(from, _)| *from != hole));
    }
}

#[test]
fn e3_exhaustive_end_sweep_has_hand_counted_classes() {
    let (mut null, mut outside, mut surrogate, mut noncharacter, mut control) = (0, 0, 0, 0, 0);
    for code in 0..=SATURATED {
        let (scalar, kinds) = end_semantics(code);
        assert!(kinds.len() <= 1, "bullets are disjoint at {code:#x}");
        for kind in kinds {
            match kind {
                NULL => null += 1,
                OUTSIDE => outside += 1,
                SURROGATE => surrogate += 1,
                NONCHARACTER => noncharacter += 1,
                CONTROL_REFERENCE => control += 1,
                other => panic!("unexpected Numeric End diagnostic {other:?}"),
            }
        }
        let _ = scalar;
    }
    assert_eq!(null, 1);
    assert_eq!(
        outside, 1,
        "the single saturated value stands for all larger"
    );
    assert_eq!(surrogate, 0x800);
    // 32 in U+FDD0..U+FDEF plus two per plane over 17 planes.
    assert_eq!(noncharacter, 32 + 34);
    // C0 1..=0x1F minus TAB/LF/FF (CR stays) = 28, plus 0x7F..=0x9F = 33.
    assert_eq!(control, 28 + 33);
}

// ---------------------------------------------------------------------------
// Decimal / hexadecimal / semicolon lexical cells
// ---------------------------------------------------------------------------

#[test]
fn k0_the_matrix_of_required_lexical_cells_in_both_contexts() {
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        let lexed = lex_ctx("&#65;", ctx);
        assert_eq!(lexed.projection(), vec![dec(0, 5, "A")]);
        assert!(lexed.diagnostics.is_empty());

        let lexed = lex_ctx("&#65", ctx);
        assert_eq!(lexed.projection(), vec![dec(0, 4, "A")]);
        assert_eq!(lexed.diagnostic_view(), vec![(MISSING, Some(0), 4)]);
        match &lexed.diagnostics[0].at {
            Pos::Eof(offset) => assert_eq!(*offset, 4),
            other => panic!("terminated at EOF, not at a unit: {other:?}"),
        }

        for text in ["&#x41;", "&#X41;"] {
            let lexed = lex_ctx(text, ctx);
            assert_eq!(lexed.projection(), vec![hex(0, 6, "A")], "{text}");
            assert!(lexed.diagnostics.is_empty(), "{text}");
        }
        for text in ["&#x41", "&#X41"] {
            let lexed = lex_ctx(text, ctx);
            assert_eq!(lexed.projection(), vec![hex(0, 5, "A")], "{text}");
            assert_eq!(lexed.diagnostic_view(), vec![(MISSING, Some(0), 5)]);
        }
        // Lower- and upper-case hex digits both accumulate.
        assert_eq!(lex_ctx("&#xaB;", ctx).interpreted(), "\u{ab}");
        assert_eq!(lex_ctx("&#Xe9;", ctx).interpreted(), "\u{e9}");
    }
}

#[test]
fn k1_transition_traces_are_hand_derived_per_state_dispatch() {
    use St::*;
    // One transition per unit examined under one state; reconsume is another
    // examination; EOF is examined; Numeric End examines nothing.
    let cases: Vec<(&str, Vec<Dispatch>)> = vec![
        (
            "&#65;",
            vec![
                at(Data, 0),
                at(CharacterReference, 1),
                at(Numeric, 2),
                at(Decimal, 2),
                at(Decimal, 3),
                at(Decimal, 4),
                at_eof(Data),
            ],
        ),
        (
            "&#65",
            vec![
                at(Data, 0),
                at(CharacterReference, 1),
                at(Numeric, 2),
                at(Decimal, 2),
                at(Decimal, 3),
                at_eof(Decimal),
                at_eof(Data),
            ],
        ),
        (
            "&#x41;",
            vec![
                at(Data, 0),
                at(CharacterReference, 1),
                at(Numeric, 2),
                at(HexadecimalStart, 3),
                at(Hexadecimal, 3),
                at(Hexadecimal, 4),
                at(Hexadecimal, 5),
                at_eof(Data),
            ],
        ),
        (
            "&#;",
            vec![
                at(Data, 0),
                at(CharacterReference, 1),
                at(Numeric, 2),
                at(Data, 2),
                at_eof(Data),
            ],
        ),
        (
            "&#xZ",
            vec![
                at(Data, 0),
                at(CharacterReference, 1),
                at(Numeric, 2),
                at(HexadecimalStart, 3),
                at(Data, 3),
                at_eof(Data),
            ],
        ),
        (
            "&#",
            vec![
                at(Data, 0),
                at(CharacterReference, 1),
                at_eof(Numeric),
                at_eof(Data),
            ],
        ),
        (
            // The terminator `x` is examined by Decimal and again, after the
            // reconsume through the input-free End, by Data.
            "&#65x",
            vec![
                at(Data, 0),
                at(CharacterReference, 1),
                at(Numeric, 2),
                at(Decimal, 2),
                at(Decimal, 3),
                at(Decimal, 4),
                at(Data, 4),
                at_eof(Data),
            ],
        ),
    ];
    for (text, expected) in cases {
        let lexed = lex(text);
        assert_eq!(lexed.trace, expected, "{text}");
        assert_eq!(lexed.usage.steps, expected.len(), "{text}");
    }
    // The RCDATA variant differs only in the return-state label.
    let rcdata = lex_ctx("&#65;", Ctx::Rcdata);
    assert_eq!(rcdata.trace[0], at(Rcdata, 0));
    assert_eq!(rcdata.trace[6], at_eof(Rcdata));
}

#[test]
fn k2_hex_and_decimal_start_decisions_use_the_pinned_branches() {
    // `a` is a hex digit but Numeric accepts only ASCII digits or x/X.
    let lexed = lex("&#a;");
    assert_eq!(
        lexed.projection(),
        vec![flushed(0, 2, "&#"), lit(2, 4, "a;")]
    );
    assert_eq!(lexed.diagnostic_view(), vec![(ABSENCE, Some(0), 2)]);
    // `x` followed by `x`: Hexadecimal Start flushes `&#x` and the second
    // `x` is ordinary Data.
    let lexed = lex("&#xx41;");
    assert_eq!(
        lexed.projection(),
        vec![flushed(0, 3, "&#x"), lit(3, 7, "x41;")]
    );
    assert_eq!(lexed.diagnostic_view(), vec![(ABSENCE, Some(0), 3)]);
    // `&#` then `#`: a second `#` is not a digit.
    let lexed = lex("&##65;");
    assert_eq!(
        lexed.projection(),
        vec![flushed(0, 2, "&#"), lit(2, 6, "#65;")]
    );
    // A decimal terminator that is a hex letter: Decimal does not accept it.
    let lexed = lex("&#65a;");
    assert_eq!(lexed.projection(), vec![dec(0, 4, "A"), lit(4, 6, "a;")]);
    assert_eq!(lexed.diagnostic_view(), vec![(MISSING, Some(0), 4)]);
    // No Named lookup is ever started after the Numeric branch.
    let lexed = lex("&#65amp;");
    assert_eq!(lexed.projection(), vec![dec(0, 4, "A"), lit(4, 8, "amp;")]);
    // Hex terminators that are not hex digits.
    let lexed = lex("&#x41g");
    assert_eq!(lexed.projection(), vec![hex(0, 5, "A"), lit(5, 6, "g")]);
    assert_eq!(lexed.diagnostic_view(), vec![(MISSING, Some(0), 5)]);
}

#[test]
fn k3_semicolonless_terminator_is_reconsumed_exactly_once() {
    for (text, resolved, rest_start, rest) in [
        ("&#65 b", dec(0, 4, "A"), 4, " b"),
        ("&#x41<", hex(0, 5, "A"), 5, "<"),
    ] {
        if rest == "<" {
            // `<` is a tag-shape boundary in this bounded model; the
            // reconsume still hands it to the return state.
            let lexed = lex(text);
            assert_eq!(lexed.projection(), vec![resolved]);
            assert_eq!(lexed.stop, Some(Stop::Outside(Outside::TagShape)));
            assert_eq!(lexed.trace.last(), Some(&at(St::Data, rest_start)));
            continue;
        }
        let lexed = lex(text);
        assert_eq!(
            lexed.projection(),
            vec![resolved, lit(rest_start, text.len(), rest)]
        );
        let examinations_of_terminator = lexed
            .trace
            .iter()
            .filter(|(_, start)| *start == Some(rest_start))
            .count();
        assert_eq!(examinations_of_terminator, 2, "one unit, two transitions");
        assert_eq!(
            lexed
                .projection()
                .iter()
                .map(|p| p.3.clone())
                .collect::<String>(),
            lexed.interpreted()
        );
    }
}

// ---------------------------------------------------------------------------
// No-digits recovery
// ---------------------------------------------------------------------------

#[test]
fn z0_no_digits_recovery_flushes_the_formed_prefix_and_reconsumes() {
    let cases: &[(&str, Vec<Projection>, (usize, &str))] = &[
        ("&#;", vec![flushed(0, 2, "&#"), lit(2, 3, ";")], (2, ";")),
        ("&#A", vec![flushed(0, 2, "&#"), lit(2, 3, "A")], (2, "A")),
        ("&#x;", vec![flushed(0, 3, "&#x"), lit(3, 4, ";")], (3, ";")),
        ("&#xZ", vec![flushed(0, 3, "&#x"), lit(3, 4, "Z")], (3, "Z")),
        ("&#X;", vec![flushed(0, 3, "&#X"), lit(3, 4, ";")], (3, ";")),
        ("&# ", vec![flushed(0, 2, "&#"), lit(2, 3, " ")], (2, " ")),
    ];
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        for (text, projection, (offset, unit)) in cases {
            let lexed = lex_ctx(text, ctx);
            assert_eq!(&lexed.projection(), projection, "{text}");
            assert_eq!(lexed.diagnostic_kinds(), vec![ABSENCE], "{text}");
            match &lexed.diagnostics[0].at {
                Pos::Unit(span) => {
                    assert_eq!((span.start, span.raw.as_str()), (*offset, *unit), "{text}");
                }
                other => panic!("{text}: the offending unit anchors it, got {other:?}"),
            }
            // The offending unit is flushed by the *return state*, not by the
            // recovery, and no decoded value exists.
            assert!(lexed.finalized.is_empty() && lexed.end_entered.is_empty());
            assert!(!lexed.has_numeric_output());
            assert_eq!(lexed.interpreted(), *text);
        }
        // EOF is "anything else" in both Numeric and Hexadecimal Start.
        let lexed = lex_ctx("&#", ctx);
        assert_eq!(lexed.projection(), vec![flushed(0, 2, "&#")]);
        assert_eq!(lexed.diagnostic_view(), vec![(ABSENCE, Some(0), 2)]);
        assert!(matches!(lexed.diagnostics[0].at, Pos::Eof(2)));
        let lexed = lex_ctx("&#x", ctx);
        assert_eq!(lexed.projection(), vec![flushed(0, 3, "&#x")]);
        assert_eq!(lexed.diagnostic_view(), vec![(ABSENCE, Some(0), 3)]);
        assert!(lexed.reached_eof() && lexed.stop.is_none());
    }
}

#[test]
fn z1_absence_of_digits_is_not_numeric_zero() {
    for text in ["&#;", "&#A", "&#x;", "&#xZ"] {
        let lexed = lex(text);
        let probe = absence_as_zero(text).expect("probe applies to a no-digits shape");
        // The falsified design would fabricate U+FFFD and a null diagnostic.
        assert_eq!(probe, ('\u{fffd}', vec![NULL]));
        assert!(!lexed.interpreted().contains('\u{fffd}'), "{text}");
        assert!(!lexed.diagnostic_kinds().contains(&NULL), "{text}");
        assert!(lexed.diagnostic_kinds().contains(&ABSENCE), "{text}");
        assert!(lexed.finalized.is_empty(), "{text}");
        assert!(lexed.interpreted().starts_with("&#"), "{text}");
    }
    // Real zero is a different, finalized outcome.
    for text in ["&#0;", "&#x0;", "&#000;"] {
        let lexed = lex(text);
        assert_eq!(lexed.interpreted(), "\u{fffd}", "{text}");
        assert_eq!(lexed.diagnostic_kinds(), vec![NULL], "{text}");
        assert_eq!(lexed.finalized, vec![0], "{text}");
    }
}

// ---------------------------------------------------------------------------
// Long runs and overflow
// ---------------------------------------------------------------------------

#[test]
fn o1_digit_count_does_not_decide_overflow() {
    let zeros = "0".repeat(10_000);
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        let hex_run = format!("&#x{zeros}41;");
        let lexed = lex_ctx(&hex_run, ctx);
        assert_eq!(lexed.projection(), vec![hex(0, hex_run.len(), "A")]);
        assert!(lexed.diagnostics.is_empty());

        let dec_run = format!("&#{zeros}65;");
        let lexed = lex_ctx(&dec_run, ctx);
        assert_eq!(lexed.projection(), vec![dec(0, dec_run.len(), "A")]);
        assert!(lexed.diagnostics.is_empty());

        // Semicolonless long leading-zero run at EOF.
        let bare = format!("&#x{zeros}41");
        let lexed = lex_ctx(&bare, ctx);
        assert_eq!(lexed.projection(), vec![hex(0, bare.len(), "A")]);
        assert_eq!(lexed.diagnostic_kinds(), vec![MISSING]);

        // A very long nonzero run exceeds U+10FFFF.
        let big_dec = format!("&#1{zeros};");
        let lexed = lex_ctx(&big_dec, ctx);
        assert_eq!(lexed.projection(), vec![dec(0, big_dec.len(), "\u{fffd}")]);
        assert_eq!(lexed.diagnostic_kinds(), vec![OUTSIDE]);

        let big_hex = format!("&#x{}41;", "F".repeat(10_000));
        let lexed = lex_ctx(&big_hex, ctx);
        assert_eq!(lexed.projection(), vec![hex(0, big_hex.len(), "\u{fffd}")]);
        assert_eq!(lexed.diagnostic_kinds(), vec![OUTSIDE]);
    }
    // Overflow stays sticky through a huge run that would wrap a u32 or u64
    // back to a small value: 2^32 + 65 and 2^64 + 65.
    for (text, base) in [
        ("&#4294967361;", Radix::Decimal),
        ("&#18446744073709551681;", Radix::Decimal),
        ("&#x100000041;", Radix::Hexadecimal),
        ("&#x10000000000000041;", Radix::Hexadecimal),
    ] {
        let lexed = lex(text);
        assert_eq!(lexed.interpreted(), "\u{fffd}", "{text}");
        assert_eq!(lexed.diagnostic_kinds(), vec![OUTSIDE], "{text}");
        let digits = text
            .trim_start_matches("&#x")
            .trim_start_matches("&#")
            .trim_end_matches(';');
        assert_eq!(wrapping_u32_decode(digits, base), 65, "{text} would wrap");
    }
    // Short runs can overflow too: the run length is not the decision.
    for (text, expected, diagnostic) in [
        ("&#1114111;", '\u{10ffff}', Some(NONCHARACTER)),
        ("&#1114112;", '\u{fffd}', Some(OUTSIDE)),
        ("&#x10FFFF;", '\u{10ffff}', Some(NONCHARACTER)),
        ("&#x110000;", '\u{fffd}', Some(OUTSIDE)),
        ("&#x0110000;", '\u{fffd}', Some(OUTSIDE)),
        ("&#9999999;", '\u{fffd}', Some(OUTSIDE)),
    ] {
        let lexed = lex(text);
        assert_eq!(lexed.interpreted(), expected.to_string(), "{text}");
        assert_eq!(
            lexed.diagnostic_kinds(),
            diagnostic.into_iter().collect::<Vec<_>>()
        );
    }
    assert!(length_based_overflow(&format!("{zeros}65"), Radix::Decimal));
    assert!(!length_based_overflow("9999999", Radix::Decimal));
}

#[test]
fn o2_host_integer_parsing_is_falsified() {
    let zeros = "0".repeat(100);
    // A parse that fails on overflow cannot tell "too big" from "not a
    // reference", and a wrapping one reports a small value.
    assert_eq!(host_parse_decode("4294967361", Radix::Decimal), None);
    assert_eq!(
        host_parse_decode(&format!("{zeros}65"), Radix::Decimal),
        Some('A')
    );
    assert_eq!(host_parse_decode("1114112", Radix::Decimal), None);
    assert_eq!(wrapping_u32_decode("4294967361", Radix::Decimal), 65);
    let lexed = lex("&#4294967361;");
    assert_eq!(lexed.interpreted(), "\u{fffd}");
    assert_eq!(lexed.diagnostic_kinds(), vec![OUTSIDE]);
    // Both gold behaviors are required: the failure path is a decoded U+FFFD
    // with a diagnostic, never a literal flush of the digits.
    assert!(!lexed.interpreted().contains("4294967361"));
}

#[test]
fn o3_the_saturating_model_agrees_with_an_independent_length_oracle() {
    let decimal_tails = [
        "0",
        "65",
        "1114111",
        "1114112",
        "4294967361",
        "99999999999999999999",
        "18446744073709551681",
    ];
    let hex_tails = [
        "0",
        "41",
        "10FFFF",
        "110000",
        "100000041",
        "10000000000000041",
        "FFFFFFFFFFFFFFFFFFFF",
    ];
    for zeros in [0usize, 1, 6, 7, 8, 50, 2_000] {
        let prefix = "0".repeat(zeros);
        for (radix, tails, marker) in [
            (Radix::Decimal, &decimal_tails[..], ""),
            (Radix::Hexadecimal, &hex_tails[..], "x"),
        ] {
            for tail in tails {
                let digits = format!("{prefix}{tail}");
                let text = format!("&#{marker}{digits};");
                let lexed = lex(&text);
                let (scalar, kinds) = match exact_or_exceeds(&digits, radix) {
                    Some(code) => end_semantics(code),
                    None => ('\u{fffd}', vec![OUTSIDE]),
                };
                assert_eq!(
                    lexed.interpreted(),
                    scalar.to_string(),
                    "{radix:?} {digits}"
                );
                assert_eq!(lexed.diagnostic_kinds(), kinds, "{radix:?} {digits}");
                assert_valid(&construct(lexed));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Transition accounting
// ---------------------------------------------------------------------------

#[test]
fn t1_transition_counts_follow_the_pinned_dispatch_formula() {
    // Hex: Data, CharacterReference, Numeric, HexadecimalStart, then one
    // Hexadecimal dispatch per digit (the first is the reconsume), one for `;`,
    // and EOF in Data.
    for zeros in [0usize, 1, 7, 100, 10_000] {
        let digits = zeros + 2;
        let hex_text = format!("&#x{}41;", "0".repeat(zeros));
        let lexed = lex(&hex_text);
        assert_eq!(lexed.usage.steps, 4 + digits + 1 + 1, "{zeros}");
        assert_eq!(lexed.trace.len(), lexed.usage.steps);
        let dec_text = format!("&#{}65;", "0".repeat(zeros));
        let lexed = lex(&dec_text);
        // Decimal: no start state; Numeric examines the first digit, Decimal
        // reconsumes it.
        assert_eq!(lexed.usage.steps, 3 + digits + 1 + 1, "{zeros}");
    }
}

#[test]
fn t2_long_digit_runs_are_never_compressed_into_one_transition() {
    let few = lex(&format!("&#{}1;", "0".repeat(3))).usage.steps;
    let many = lex(&format!("&#{}1;", "0".repeat(3_000))).usage.steps;
    assert_eq!(
        many - few,
        3_000 - 3,
        "exactly one transition per extra digit"
    );
    for digits in [1usize, 50, 5_000] {
        let lexed = lex(&format!("&#{};", "7".repeat(digits)));
        assert_ne!(lexed.usage.steps, named_style_numeric_steps(digits) + 2);
        assert!(lexed.usage.steps >= digits, "at least one per digit");
    }
    // The count is independent of the digit values.
    assert_eq!(
        lex(&format!("&#x{};", "0".repeat(500))).usage.steps,
        lex(&format!("&#x{};", "F".repeat(500))).usage.steps
    );
}

#[test]
fn t3_end_is_input_free_and_charged_by_no_unit() {
    let lexed = lex("&#65;");
    assert_eq!(lexed.end_entered, vec![0]);
    assert_eq!(lexed.finalized, vec![0]);
    // Numeric End never examined a unit: no dispatch names it, and no
    // dispatch refers to a position after the last consumed unit except the
    // return state's own EOF examination.
    assert!(lexed.trace.iter().all(|(state, _)| *state != St::Tag));
    assert_eq!(lexed.trace.len(), 7);
    // End events sit strictly between the terminator dispatch and the return
    // state's next examination, in the event log.
    let end_event = lexed
        .events
        .iter()
        .position(|e| *e == Event::EndCommitted(0))
        .expect("committed");
    let token_event = lexed
        .events
        .iter()
        .position(|e| matches!(e, Event::Token(_)))
        .expect("token");
    assert!(end_event < token_event);
    // Two readings of the accepted rule give two totals; the leaf freezes
    // only the examination count and records the finalization separately.
    let literal_reading = lexed.usage.steps;
    let dispatch_reading = lexed.usage.steps + lexed.end_entered.len();
    assert_eq!((literal_reading, dispatch_reading), (7, 8));
    // A no-digits recovery executes no End at all.
    assert!(lex("&#;").end_entered.is_empty());
}

#[test]
fn t4_eof_and_reconsume_examinations_are_counted() {
    // `&#65` = 5 unit examinations (& # 6 5 and the Decimal examination of
    // EOF) + the Decimal re-examination of `6` + Data EOF.
    let lexed = lex("&#65");
    assert_eq!(lexed.usage.steps, 7);
    let eof_examinations = lexed.trace.iter().filter(|(_, s)| s.is_none()).count();
    assert_eq!(eof_examinations, 2, "Decimal EOF then Data EOF");
    // Plain text is one transition per unit plus EOF.
    assert_eq!(lex("abc").usage.steps, 4);
}

// ---------------------------------------------------------------------------
// Diagnostic ordering
// ---------------------------------------------------------------------------

#[test]
fn d1_semicolonless_termination_orders_preprocessing_before_dispatch() {
    let cases: &[(&str, Vec<DiagnosticView>)] = &[
        (
            "&#65\u{1}",
            vec![(INPUT_CONTROL, None, 4), (MISSING, Some(0), 4)],
        ),
        (
            "&#x41\u{1}",
            vec![(INPUT_CONTROL, None, 5), (MISSING, Some(0), 5)],
        ),
        (
            "&#0\u{1}",
            vec![
                (INPUT_CONTROL, None, 3),
                (MISSING, Some(0), 3),
                (NULL, Some(0), 3),
            ],
        ),
        (
            "&#128\u{7f}",
            vec![
                (INPUT_CONTROL, None, 5),
                (MISSING, Some(0), 5),
                (CONTROL_REFERENCE, Some(0), 5),
            ],
        ),
        (
            "&#65\u{85}",
            vec![(INPUT_CONTROL, None, 4), (MISSING, Some(0), 4)],
        ),
        (
            "&#\u{1}",
            vec![(INPUT_CONTROL, None, 2), (ABSENCE, Some(0), 2)],
        ),
        (
            "&#x\u{85}",
            vec![(INPUT_CONTROL, None, 3), (ABSENCE, Some(0), 3)],
        ),
        (
            // With `;` the End diagnostic is raised while `;` is current, and
            // the next unit's preprocessing diagnostic follows it.
            "&#0;\u{1}",
            vec![(NULL, Some(0), 3), (INPUT_CONTROL, None, 4)],
        ),
    ];
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        for (text, expected) in cases {
            let lexed = lex_ctx(text, ctx);
            assert_eq!(&lexed.diagnostic_view(), expected, "{text} in {ctx:?}");
            let starts: Vec<usize> = lexed.diagnostics.iter().map(|d| d.at.start()).collect();
            assert!(nondecreasing(&starts), "{text}: {starts:?}");
            // The reconsumed terminator is preprocessed exactly once.
            let controls = lexed
                .diagnostic_kinds()
                .iter()
                .filter(|k| **k == INPUT_CONTROL)
                .count();
            assert_eq!(controls, 1, "{text}");
        }
    }
    // Authored CRLF terminator: one LF unit, and the site is its first byte.
    let lexed = lex("&#65\r\n");
    assert_eq!(lexed.diagnostic_view(), vec![(MISSING, Some(0), 4)]);
    assert_eq!(lexed.projection(), vec![dec(0, 4, "A"), lit(4, 6, "\n")]);
    // EOF terminator.
    let lexed = lex("&#0");
    assert_eq!(
        lexed.diagnostic_view(),
        vec![(MISSING, Some(0), 3), (NULL, Some(0), 3)]
    );
}

#[test]
fn d2_reference_start_anchoring_contradicts_the_run_result_contract() {
    // Falsified anchor choice: anchoring character-reference diagnostics at
    // the reference start makes the normative order decrease.
    let lexed = lex("&#65\u{1}");
    assert_eq!(reference_start_anchored_starts(&lexed), vec![4, 0]);
    assert!(!nondecreasing(&reference_start_anchored_starts(&lexed)));
    // The emission-position sites satisfy the same contract. The normative
    // order therefore coexists with the contract; only the anchor choice
    // would break it, and that choice stays a placement question.
    let starts: Vec<usize> = lexed.diagnostics.iter().map(|d| d.at.start()).collect();
    assert!(nondecreasing(&starts));
}

#[test]
fn d3_every_terminator_and_prefix_yields_nondecreasing_diagnostics() {
    let prefixes = [
        "&#65",
        "&#x41",
        "&#0",
        "&#xD800",
        "&#128",
        "&#1114112",
        "&#",
        "&#x",
        "&#xZ",
        "&#65;",
        "&#0;",
    ];
    let terminators = [
        "",
        ";",
        " ",
        "\u{1}",
        "\u{7f}",
        "\u{85}",
        "x",
        "\r\n",
        "\r",
        "\u{1};\u{1}",
    ];
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        for prefix in prefixes {
            for terminator in terminators {
                let text = format!("a{prefix}{terminator}b&#1{prefix}");
                let observation = construct(lex_ctx(&text, ctx));
                let starts: Vec<usize> = observation
                    .lexed
                    .diagnostics
                    .iter()
                    .map(|d| d.at.start())
                    .collect();
                assert!(nondecreasing(&starts), "{text:?}: {starts:?}");
                assert_valid(&observation);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Decoded output is never tokenizer input
// ---------------------------------------------------------------------------

#[test]
fn n1_decoded_syntax_is_character_data_and_never_markup() {
    let observation = run("<body>&#60;/body>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "</body>");
    assert_eq!(observation.mode, Mode::InBody, "no end tag closed the body");
    assert!(observation.tree.body.as_ref().unwrap().close.is_none());
    assert_eq!(
        observation
            .lexed
            .count_tokens(|t| matches!(t, Token::EndBody(_))),
        0
    );
    assert_eq!(
        observation.lexed.projection(),
        vec![dec(6, 11, "<"), lit(11, 17, "/body>")]
    );
    assert_eq!(retokenized_end_body_count("<body>&#60;/body>"), 1, "probe");

    // `&#38;amp;` is `&` followed by the ordinary text `amp;`: no recursive
    // Named or Numeric decoding.
    let observation = run("<body>&#38;amp;</body>");
    assert_eq!(observation.body_text(), "&amp;");
    assert_eq!(observation.completion, Completion::Complete);
    let observation = run("<body>&#38;#65;</body>");
    assert_eq!(observation.body_text(), "&#65;");
    assert_eq!(
        observation.lexed.finalized,
        vec![0],
        "exactly one reference"
    );
    let observation = run("<body>&#x26;#x26;</body>");
    assert_eq!(observation.body_text(), "&#x26;");

    // Decoded `</title>` inside the selected Title is text, not a close.
    let observation = run("<title>&#60;/title></title>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.title_text(), "</title>");
    assert_eq!(observation.tree.titles.len(), 1);
    assert_eq!(
        observation
            .lexed
            .count_tokens(|t| matches!(t, Token::EndTitle(_))),
        1
    );
    let observation = run("<title>&#38;lt;</title>");
    assert_eq!(observation.title_text(), "&lt;");
}

#[test]
fn n2_decoded_cr_is_a_value_and_authored_cr_is_preprocessing() {
    // Decoded U+000D is kept as U+000D.
    let observation = run("<body>&#13;</body>");
    assert_eq!(observation.body_text(), "\r");
    assert_eq!(
        observation.lexed.diagnostic_kinds(),
        vec![CONTROL_REFERENCE]
    );
    // A decoded CR followed by an authored LF is two scalars; CRLF collapsing
    // applies only to authored source.
    let observation = run("<body>&#13;\n</body>");
    assert_eq!(observation.body_text(), "\r\n");
    assert_eq!(observation.body_text().chars().count(), 2);
    // Authored CR / CRLF become one LF.
    assert_eq!(run("<body>\r\n</body>").body_text(), "\n");
    assert_eq!(run("<body>\r</body>").body_text(), "\n");
    // A decoded CR next to an authored CR keeps both.
    assert_eq!(run("<body>&#13;\r</body>").body_text(), "\r\n");
    assert_eq!(run("<body>\r&#13;</body>").body_text(), "\n\r");
    // Falsified design: re-preprocessing the decoded output.
    assert_eq!(renormalized("\r\n"), "\n");
    assert_ne!(
        renormalized(&run("<body>&#13;\n</body>").body_text()),
        "\r\n"
    );
    // Decoded CR is HTML whitespace in the tree and stays After Body.
    let observation = run("<body></body>&#13;");
    assert_eq!(observation.mode, Mode::AfterBody);
    assert!(observation.recoveries.is_empty());
    assert_eq!(observation.body_text(), "\r");
}

// ---------------------------------------------------------------------------
// Return-state ownership
// ---------------------------------------------------------------------------

#[test]
fn r1_return_state_ownership_is_not_erased() {
    // Data resumes Data: the tag after the reference is recognized.
    let data = run("<body>&#65;</body>");
    assert_eq!(data.completion, Completion::Complete);
    assert_eq!(data.mode, Mode::AfterBody);
    assert_eq!(
        data.lexed
            .trace
            .iter()
            .filter(|(s, _)| *s == St::Rcdata)
            .count(),
        0
    );

    // RCDATA resumes RCDATA: the appropriate end tag is recognized there.
    let title = run("<title>&#65;</title>");
    assert_eq!(title.completion, Completion::Complete);
    assert_eq!(title.title_text(), "A");
    assert!(title.lexed.trace.iter().any(|(s, _)| *s == St::Rcdata));

    // In RCDATA an authored `</body>` after a reference is not a body end tag:
    // the bounded model refuses RCDATA markup rather than closing anything.
    let rcdata_markup = run("<title>&#65;</body></title>");
    assert_eq!(
        rcdata_markup.completion,
        Completion::Unsupported(Unsupported::Outside(Outside::RcdataMarkup))
    );
    // In Data, `</title>` after a reference is not a title close.
    let data_markup = run("<body>&#65;</title>");
    assert_eq!(
        data_markup.completion,
        Completion::Unsupported(Unsupported::Outside(Outside::TagShape))
    );

    // Falsified designs: an erased return state would resume the wrong one.
    let source = SourceText::new(SourceId::new(1), "<title>&#65;</title>".to_owned());
    let erased = lex_with(&source, Ctx::Data, Limits::default(), Some(Ctx::Data));
    assert_eq!(erased.stop, Some(Stop::Outside(Outside::TagShape)));
    let source = SourceText::new(SourceId::new(1), "<body>&#65;</body>".to_owned());
    let erased = lex_with(&source, Ctx::Data, Limits::default(), Some(Ctx::Rcdata));
    assert_eq!(erased.stop, Some(Stop::Outside(Outside::RcdataMarkup)));

    // The recovery path returns to the same owner.
    let title = run("<title>&#;</title>");
    assert_eq!(title.completion, Completion::Complete);
    assert_eq!(title.title_text(), "&#;");
    let body = run("<body>&#x;</body>");
    assert_eq!(body.completion, Completion::Complete);
    assert_eq!(body.body_text(), "&#x;");
}

// ---------------------------------------------------------------------------
// Resource refusal and partial progress
// ---------------------------------------------------------------------------

fn steps(limit: usize) -> Limits {
    Limits {
        steps: Some(limit),
        ..Limits::default()
    }
}

#[test]
fn a1_transition_refusal_inside_a_digit_run_keeps_honest_partial_progress() {
    // `&#123;` examinations: Data&, CharRef#, Numeric 1, Decimal 1, Decimal 2,
    // Decimal 3, Decimal ;, Data EOF. After L successful dispatches the
    // committed coverage is hand-derived below.
    let expected_coverage = [(1usize, 1usize), (2, 2), (3, 2), (4, 3), (5, 4), (6, 5)];
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        for (limit, coverage) in expected_coverage {
            let lexed = lex_limited("&#123;", ctx, steps(limit));
            assert_eq!(
                lexed.stop,
                Some(Stop::Resource(Refusal {
                    resource: Resource::TransitionSteps,
                    limit,
                    attempted: limit + 1,
                })),
                "limit {limit}"
            );
            assert_eq!(
                lexed.usage.steps, limit,
                "a rejected attempt is not counted"
            );
            assert_eq!(lexed.coverage_end, coverage, "limit {limit}");
            // No decoded character exists before Numeric End finalizes, and
            // End was never even entered.
            assert!(lexed.tokens.is_empty() && !lexed.has_numeric_output());
            assert!(lexed.finalized.is_empty() && lexed.end_entered.is_empty());
            assert!(lexed.diagnostics.is_empty());
            assert!(!lexed.reached_eof());
            assert_eq!(lexed.entries.len(), 1, "the entry is committed coverage");
        }
        // Enough budget completes with the hand-derived total of 8.
        let lexed = lex_limited("&#123;", ctx, steps(8));
        assert!(lexed.stop.is_none() && lexed.reached_eof());
        assert_eq!(lexed.projection(), vec![dec(0, 6, "{")]);
        // Limit 7 sits exactly on the Numeric End charge question (see the
        // module docs). Only reading-independent invariants are asserted.
        let boundary = lex_limited("&#123;", ctx, steps(7));
        assert!(boundary.coverage_end >= 5);
        assert_eq!(
            boundary.has_numeric_output(),
            !boundary.finalized.is_empty()
        );
        assert!(!boundary.reached_eof());
    }
    // A refusal deep inside a long run: dispatches 4.. each consume a digit.
    let text = format!("&#{};", "1".repeat(1_000));
    let lexed = lex_limited(&text, Ctx::Data, steps(500));
    assert_eq!(lexed.usage.steps, 500);
    assert_eq!(lexed.coverage_end, 2 + (500 - 3));
    assert!(lexed.tokens.is_empty() && lexed.finalized.is_empty());
    assert!(matches!(
        lexed.stop,
        Some(Stop::Resource(Refusal {
            resource: Resource::TransitionSteps,
            ..
        }))
    ));
}

#[test]
fn a2_lower_layer_incomplete_never_becomes_complete_at_tree_level() {
    // `<body>` is 7 dispatches. Refusals inside the following reference.
    for limit in 7usize..=13 {
        let observation = run_limited("<body>&#123;", steps(limit));
        assert!(
            matches!(
                observation.completion,
                Completion::ResourceLimit(Refusal {
                    resource: Resource::TransitionSteps,
                    ..
                })
            ),
            "limit {limit}"
        );
        assert_eq!(observation.body_text(), "", "no decoded character yet");
        assert_eq!(observation.mode, Mode::InBody);
        assert_valid(&observation);
    }
    let observation = run_limited("<body>&#123;", steps(15));
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "{");
    // Title: `<title>` is 8 dispatches.
    for limit in 8usize..=14 {
        let observation = run_limited("<title>&#123;", steps(limit));
        assert!(matches!(
            observation.completion,
            Completion::ResourceLimit(_)
        ));
        assert_eq!(observation.title_text(), "");
        assert_valid(&observation);
    }
}

#[test]
fn a3_end_preparation_precedes_every_end_effect() {
    // Token budget: `a` takes the only token, so the reference's token is
    // refused, and the End diagnostic must not be claimed for it.
    let limits = Limits {
        tokens: Some(1),
        ..Limits::default()
    };
    let lexed = lex_limited("a&#0;", Ctx::Data, limits);
    assert_eq!(
        lexed.stop,
        Some(Stop::Resource(Refusal {
            resource: Resource::EmittedTokens,
            limit: 1,
            attempted: 2,
        }))
    );
    assert_eq!(lexed.projection(), vec![lit(0, 1, "a")]);
    assert!(lexed.diagnostics.is_empty(), "no claim of a failed effect");
    assert!(lexed.finalized.is_empty());
    assert_eq!(lexed.end_entered, vec![0]);
    // Honest partial progress: the digits are committed, the decoded character
    // is not. Either answer to the `;` question satisfies these bounds.
    assert!((4..=5).contains(&lexed.coverage_end));
    assert!(!lexed.reached_eof());

    // Diagnostic budget: a required End diagnostic cannot be recorded, so
    // neither the diagnostic nor the replacement exists.
    let limits = Limits {
        diagnostics: Some(0),
        ..Limits::default()
    };
    for text in [
        "&#0;",
        "&#xD800;",
        "&#128;",
        "&#1114112;",
        "&#xFDD0;",
        "&#13;",
    ] {
        let lexed = lex_limited(text, Ctx::Data, limits);
        assert!(
            matches!(
                lexed.stop,
                Some(Stop::Resource(Refusal {
                    resource: Resource::Diagnostics,
                    limit: 0,
                    attempted: 1,
                }))
            ),
            "{text}"
        );
        assert!(
            !lexed.has_numeric_output() && lexed.diagnostics.is_empty(),
            "{text}"
        );
        assert!(lexed.finalized.is_empty(), "{text}");
    }
    // A reference with no diagnostic needs none: the same budget completes.
    let lexed = lex_limited("&#65;", Ctx::Data, limits);
    assert!(lexed.stop.is_none());
    assert_eq!(lexed.projection(), vec![dec(0, 5, "A")]);

    // Semicolonless: whatever the missing-semicolon diagnostic does, the
    // refused End diagnostic is never claimed and no output exists.
    let limits = Limits {
        diagnostics: Some(1),
        ..Limits::default()
    };
    let lexed = lex_limited("&#0 ", Ctx::Data, limits);
    assert!(matches!(
        lexed.stop,
        Some(Stop::Resource(Refusal {
            resource: Resource::Diagnostics,
            ..
        }))
    ));
    assert!(!lexed.diagnostic_kinds().contains(&NULL));
    assert!(!lexed.has_numeric_output() && lexed.finalized.is_empty());
    assert!(lexed.diagnostics.len() <= 1);
}

#[test]
fn a4_recovery_diagnostic_and_flush_prepare_together() {
    // The absence diagnostic and the flushed prefix are one effect.
    let tokens_exhausted = Limits {
        tokens: Some(1),
        ..Limits::default()
    };
    let lexed = lex_limited("a&#;", Ctx::Data, tokens_exhausted);
    assert!(matches!(
        lexed.stop,
        Some(Stop::Resource(Refusal {
            resource: Resource::EmittedTokens,
            ..
        }))
    ));
    assert_eq!(lexed.projection(), vec![lit(0, 1, "a")]);
    assert!(
        lexed.diagnostics.is_empty(),
        "no diagnostic without its flush"
    );

    let diagnostics_exhausted = Limits {
        diagnostics: Some(0),
        ..Limits::default()
    };
    let lexed = lex_limited("&#;", Ctx::Rcdata, diagnostics_exhausted);
    assert!(matches!(
        lexed.stop,
        Some(Stop::Resource(Refusal {
            resource: Resource::Diagnostics,
            ..
        }))
    ));
    assert!(lexed.tokens.is_empty(), "no flush without its diagnostic");

    // Retained interpreted bytes are about the produced interpretation, not
    // the authored span: U+FFFD is 3 bytes whatever spelled it.
    for (text, bytes_needed) in [
        ("&#0;", 3usize),
        ("&#x0000000000000000;", 3),
        ("&#1114112;", 3),
        ("&#65;", 1),
        ("&#x41;", 1),
        ("&#x20AC;", 3),
        ("&#x1F600;", 4),
    ] {
        let refused = lex_limited(
            text,
            Ctx::Data,
            Limits {
                retained: Some(bytes_needed - 1),
                ..Limits::default()
            },
        );
        assert!(
            matches!(
                refused.stop,
                Some(Stop::Resource(Refusal {
                    resource: Resource::RetainedInterpretedBytes,
                    ..
                }))
            ),
            "{text}"
        );
        assert!(
            !refused.has_numeric_output() && refused.diagnostics.is_empty(),
            "{text}"
        );
        let accepted = lex_limited(
            text,
            Ctx::Data,
            Limits {
                retained: Some(bytes_needed),
                ..Limits::default()
            },
        );
        assert!(accepted.stop.is_none(), "{text}");
    }
    // The flushed prefix is retained output too.
    let lexed = lex_limited(
        "&#;",
        Ctx::Data,
        Limits {
            retained: Some(1),
            ..Limits::default()
        },
    );
    assert!(matches!(
        lexed.stop,
        Some(Stop::Resource(Refusal {
            resource: Resource::RetainedInterpretedBytes,
            ..
        }))
    ));
    assert!(lexed.diagnostics.is_empty());
}

#[test]
fn a5_no_copied_digit_buffer_and_no_new_resource_dimension() {
    // A 50 000-digit reference completes under limits that are tiny in every
    // existing dimension except transition steps: nothing proportional to the
    // digit run is retained, buffered, emitted, or reported.
    let digits = 50_000;
    let text = format!("&#x{}41;", "0".repeat(digits));
    let tight = Limits {
        steps: None,
        tokens: Some(2),
        diagnostics: Some(0),
        retained: Some(1),
    };
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        let lexed = lex_limited(&text, ctx, tight);
        assert!(lexed.stop.is_none(), "{ctx:?}");
        assert_eq!(lexed.projection(), vec![hex(0, text.len(), "A")]);
        assert_eq!(lexed.usage.peak_temporary_buffer_bytes, 0);
        assert_eq!(lexed.usage.tokens, 2, "the character token and EOF");
        // Only the one existing dimension that is proportional to the input
        // grows with the run.
        // Data, CharacterReference, Numeric, HexadecimalStart, one
        // Hexadecimal dispatch per digit, `;`, and EOF.
        assert_eq!(lexed.usage.steps, 4 + (digits + 2) + 1 + 1);
    }
    // The accumulator is a fixed-size value regardless of the run length.
    assert!(std::mem::size_of::<Accumulator>() <= 8);
    // TemporaryBufferBytes = 0 is semantically viable for every recovery too.
    for text in ["&#;", "&#x;", "&#0", "&#xZ"] {
        assert_eq!(lex(text).usage.peak_temporary_buffer_bytes, 0, "{text}");
    }
}

// ---------------------------------------------------------------------------
// Preprocessing x TransitionSteps x diagnostic commitment x processed coverage
// ---------------------------------------------------------------------------
//
// Accepted lifecycle for a newly materialized authored unit:
// preprocess (once) -> prepare and commit its diagnostic -> attempt the
// `TransitionSteps` transition -> dispatch. A reconsume of the same unit
// repeats none of the preprocessing. A committed diagnostic makes processed
// coverage reach its site; a refused one changes nothing.

fn diagnostics_limit(limit: usize) -> Limits {
    Limits {
        diagnostics: Some(limit),
        ..Limits::default()
    }
}

fn step_refusal(limit: usize) -> Option<Stop> {
    Some(Stop::Resource(Refusal {
        resource: Resource::TransitionSteps,
        limit,
        attempted: limit + 1,
    }))
}

#[test]
fn b1_preprocessing_commits_before_a_transition_step_refusal() {
    // &0 #1 6@2 5@3 U+0001@4. Five committed transitions precede the
    // examination of U+0001: Data, CharacterReference, Numeric, Decimal
    // (reconsume), Decimal('5').
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        let lexed = lex_limited("&#65\u{1}", ctx, steps(5));
        assert_eq!(lexed.stop, step_refusal(5), "{ctx:?}");
        assert_eq!(lexed.usage.steps, 5, "the rejected attempt is not counted");
        // The newly materialized U+0001 was preprocessed first and that
        // diagnostic stays committed, at its exact authored site.
        assert_eq!(lexed.diagnostic_view(), vec![(INPUT_CONTROL, None, 4)]);
        match &lexed.diagnostics[0].at {
            Pos::Unit(span) => {
                assert_eq!((span.start, span.end, span.raw.as_str()), (4, 5, "\u{1}"))
            }
            other => panic!("expected the authored unit site, got {other:?}"),
        }
        assert_eq!(lexed.coverage_end, 5, "processed coverage reaches 0..5");
        // Decimal never successfully examined the terminator.
        assert!(!lexed.diagnostic_kinds().contains(&MISSING));
        assert!(lexed.end_entered.is_empty() && lexed.finalized.is_empty());
        assert!(!lexed.has_numeric_output() && lexed.tokens.is_empty());
        assert!(!lexed.reached_eof());
        assert!(
            lexed.trace.iter().all(|(_, start)| *start != Some(4)),
            "U+0001 was never examined under any state"
        );
        let observation = construct(lexed);
        assert!(matches!(
            observation.completion,
            Completion::ResourceLimit(_)
        ));
        assert_valid(&observation);
    }
    // The same lifecycle through the document tree: `<body>` is 7 dispatches.
    let observation = run_limited("<body>&#65\u{1}", steps(12));
    assert!(matches!(
        observation.completion,
        Completion::ResourceLimit(Refusal {
            resource: Resource::TransitionSteps,
            limit: 12,
            attempted: 13,
        })
    ));
    assert_eq!(
        observation.lexed.diagnostic_view(),
        vec![(INPUT_CONTROL, None, 10)]
    );
    assert_eq!(observation.lexed.coverage_end, 11);
    assert_eq!(observation.body_text(), "", "no decoded character yet");
    assert_valid(&observation);
}

#[test]
fn b2_a_refused_preprocessing_diagnostic_stops_before_any_transition() {
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        let lexed = lex_limited("&#65\u{1}", ctx, diagnostics_limit(0));
        assert_eq!(
            lexed.stop,
            Some(Stop::Resource(Refusal {
                resource: Resource::Diagnostics,
                limit: 0,
                attempted: 1,
            })),
            "{ctx:?}"
        );
        // Five transitions committed; the Decimal(U+0001) transition was not
        // attempted, so it is neither counted nor in the trace.
        assert_eq!(lexed.usage.steps, 5);
        assert_eq!(lexed.trace.len(), 5);
        assert!(lexed.trace.iter().all(|(_, start)| *start != Some(4)));
        assert!(lexed.diagnostics.is_empty());
        assert_eq!(lexed.usage.diagnostics, 0, "a refusal advances no usage");
        // The refused site is not claimed as processed: coverage is only the
        // consumed digits.
        assert_eq!(lexed.coverage_end, 4);
        assert!(!lexed.diagnostic_kinds().contains(&MISSING));
        assert!(lexed.end_entered.is_empty() && lexed.finalized.is_empty());
        assert!(!lexed.has_numeric_output() && lexed.tokens.is_empty());
        assert_valid(&construct(lexed));
    }
    // The same distinction on the recovery path: the control diagnostic fits,
    // the absence diagnostic and its flush are one effect and refuse together.
    let lexed = lex_limited("&#\u{1}", Ctx::Data, diagnostics_limit(1));
    assert_eq!(lexed.diagnostic_view(), vec![(INPUT_CONTROL, None, 2)]);
    assert!(matches!(
        lexed.stop,
        Some(Stop::Resource(Refusal {
            resource: Resource::Diagnostics,
            limit: 1,
            attempted: 2,
        }))
    ));
    assert!(lexed.tokens.is_empty());
    assert_eq!(
        lexed.coverage_end, 3,
        "only the committed site is processed"
    );
    let lexed = lex_limited("&#\u{1}", Ctx::Data, diagnostics_limit(0));
    assert!(lexed.diagnostics.is_empty());
    assert_eq!(lexed.coverage_end, 2, "the refused site is not processed");
    assert_valid(&construct(lexed));
}

#[test]
fn b3_end_diagnostic_refusal_keeps_earlier_commits_and_fabricates_nothing() {
    // `&#0` + U+0001. Control (1) and missing-semicolon (2) fit a limit of 2;
    // the Numeric End null-character-reference (3) does not.
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        let lexed = lex_limited("&#0\u{1}", ctx, diagnostics_limit(2));
        assert_eq!(
            lexed.diagnostic_kinds(),
            vec![INPUT_CONTROL, MISSING],
            "{ctx:?}"
        );
        assert_eq!(lexed.diagnostic_view()[1], (MISSING, Some(0), 3));
        assert_eq!(
            lexed.stop,
            Some(Stop::Resource(Refusal {
                resource: Resource::Diagnostics,
                limit: 2,
                attempted: 3,
            }))
        );
        assert!(!lexed.diagnostic_kinds().contains(&NULL));
        assert!(!lexed.has_numeric_output() && lexed.tokens.is_empty());
        assert!(lexed.finalized.is_empty());
        assert_eq!(lexed.end_entered, vec![0], "End ran and refused its output");
        assert_eq!(lexed.usage.diagnostics, 2);
        // The committed U+0001 site is processed even though the unit itself
        // was not consumed.
        assert_eq!(lexed.coverage_end, 4);
        // Data, CharacterReference, Numeric, Decimal(reconsume), Decimal(U+0001);
        // the Numeric End charge is the open placement question.
        assert!((5..=6).contains(&lexed.usage.steps));
        let observation = construct(lexed);
        assert!(matches!(
            observation.completion,
            Completion::ResourceLimit(Refusal {
                resource: Resource::Diagnostics,
                ..
            })
        ));
        assert_valid(&observation);
    }
}

#[test]
fn b4_preprocessing_happens_exactly_once_across_every_step_boundary() {
    // Terminators that preprocess to a diagnostic, and ones that do not.
    let controlled = ["\u{1}", "\u{7f}", "\u{85}"];
    let plain = ["\r", "\r\n", " "];
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        for terminator in controlled.into_iter().chain(plain) {
            let text = format!("&#65{terminator}");
            let expected_controls = usize::from(controlled.contains(&terminator));
            for limit in 1usize..=10 {
                let lexed = lex_limited(&text, ctx, steps(limit));
                let controls = lexed
                    .diagnostic_kinds()
                    .iter()
                    .filter(|kind| **kind == INPUT_CONTROL)
                    .count();
                // The terminator is materialized by the sixth dispatch attempt,
                // whether or not that transition is then refused.
                let expected = if limit >= 5 { expected_controls } else { 0 };
                assert_eq!(controls, expected, "{text:?} limit {limit} {ctx:?}");
                if limit < 5 {
                    assert!(lexed.diagnostics.is_empty(), "{text:?} limit {limit}");
                } else if expected_controls == 1 {
                    assert_eq!(lexed.diagnostics[0].kind, INPUT_CONTROL);
                    assert_eq!(lexed.diagnostics[0].at.start(), 4);
                }
                // The missing-semicolon diagnostic follows the terminator's
                // committed transition and never precedes its preprocessing.
                let missing = lexed.diagnostic_kinds().contains(&MISSING);
                assert_eq!(missing, limit >= 6, "{text:?} limit {limit}");
                if missing && expected_controls == 1 {
                    assert_eq!(lexed.diagnostic_kinds()[..2], [INPUT_CONTROL, MISSING]);
                }
                // Decoded output exists only if Numeric End committed it.
                assert_eq!(lexed.has_numeric_output(), !lexed.finalized.is_empty());
                if limit <= 5 {
                    assert!(lexed.end_entered.is_empty(), "{text:?} limit {limit}");
                }
                if limit <= 7 {
                    assert!(lexed.stop.is_some(), "{text:?} limit {limit}");
                    assert!(!lexed.reached_eof());
                }
                assert_valid(&construct(lexed));
            }
            // Enough budget under either End-charge reading: the unit was
            // examined by Decimal and again by the return state, with one
            // preprocessing diagnostic at most.
            let lexed = lex_limited(&text, ctx, steps(9));
            assert!(lexed.stop.is_none() && lexed.reached_eof(), "{text:?}");
            assert_eq!(lexed.projection()[0], dec(0, 4, "A"));
            let examinations = lexed
                .trace
                .iter()
                .filter(|(_, start)| *start == Some(4))
                .count();
            assert_eq!(examinations, 2, "one unit, two transitions");
            let controls = lexed
                .diagnostic_kinds()
                .iter()
                .filter(|kind| **kind == INPUT_CONTROL)
                .count();
            assert_eq!(controls, expected_controls, "{text:?}");
        }
    }
}

#[test]
fn b5_freeze_rejects_a_diagnostic_site_outside_processed_coverage() {
    // A resource-limited candidate with no contributions, so no other error
    // can mask the diagnostic-coverage theorem. The committed diagnostic's
    // site ends exactly at processed coverage.
    let good = construct(lex_limited("&#65\u{1}", Ctx::Data, steps(5)));
    assert_valid(&good);
    assert!(matches!(good.completion, Completion::ResourceLimit(_)));
    assert!(good.lexed.contributions().is_empty());
    assert_eq!(diagnostic_site_end(&good.lexed.diagnostics[0]), 5);
    assert_eq!(good.lexed.coverage_end, 5);

    let mut bad = good.clone();
    bad.lexed.coverage_end = 4;
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::DiagnosticBeyondCoverage)
    );
    let mut bad = good.clone();
    bad.lexed.coverage_end = 0;
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::DiagnosticBeyondCoverage)
    );
    // Source identity of the diagnostic and of its site are both checked.
    let mut bad = good.clone();
    bad.lexed.diagnostics[0].source_id = SourceId::new(9);
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::DiagnosticSourceIdentityMismatch)
    );
    let mut bad = good;
    if let Pos::Unit(evidence) = &mut bad.lexed.diagnostics[0].at {
        evidence.source_id = SourceId::new(9);
    }
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::DiagnosticSourceIdentityMismatch)
    );

    // A zero-width EOF site ends at EOF, which processed coverage has reached.
    let eof = lex("&#65");
    assert!(matches!(eof.diagnostics[0].at, Pos::Eof(4)));
    assert_eq!(diagnostic_site_end(&eof.diagnostics[0]), 4);
    assert_eq!(eof.coverage_end, 4);
    assert_valid(&construct(eof));
}

#[test]
fn b6_committed_diagnostics_are_always_inside_coverage_across_resource_sweeps() {
    let corpus = [
        "&#65\u{1}",
        "&#0\u{1}",
        "&#x41\u{85}",
        "&#\u{1}",
        "&#x\u{7f}",
        "&#0;\u{1}",
        "&#128\r\n",
        "a&#xD800\u{1}b",
        "&#1114112\u{1}&#65\u{1}",
    ];
    for ctx in [Ctx::Data, Ctx::Rcdata] {
        for text in corpus {
            for limit in 1usize..=16 {
                let lexed = lex_limited(text, ctx, steps(limit));
                assert_valid(&construct(lexed));
            }
            for limit in 0usize..=4 {
                let lexed = lex_limited(text, ctx, diagnostics_limit(limit));
                assert!(lexed.usage.diagnostics <= limit, "{text:?} {limit}");
                assert_eq!(lexed.diagnostics.len(), lexed.usage.diagnostics);
                let furthest = lexed
                    .diagnostics
                    .iter()
                    .map(diagnostic_site_end)
                    .max()
                    .unwrap_or(0);
                assert!(furthest <= lexed.coverage_end, "{text:?} {limit}");
                assert_valid(&construct(lexed));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tree and provenance
// ---------------------------------------------------------------------------

#[test]
fn t5_selected_tree_consequences_follow_interpreted_scalars() {
    // <body>a&#38;b</body>
    let observation = run("<body>a&#38;b</body>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "a&b");
    assert_eq!(
        observation.body_text_node_count(),
        1,
        "one coalesced text node"
    );
    assert_eq!(
        observation.lexed.projection(),
        vec![lit(6, 7, "a"), dec(7, 12, "&"), lit(12, 13, "b")]
    );
    assert_eq!(observation.mode, Mode::AfterBody);
    assert_valid(&observation);

    // <body>&#9;</body>: TAB is HTML whitespace, retained as body text.
    let observation = run("<body>&#9;</body>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "\t");
    assert_valid(&observation);

    // <body></body>&#10;: LF is whitespace and stays After Body.
    let observation = run("<body></body>&#10;");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.mode, Mode::AfterBody);
    assert!(observation.recoveries.is_empty());
    assert_eq!(observation.body_text(), "\n");
    assert_valid(&observation);

    // <body></body>&#160;: NBSP is not HTML whitespace, so the existing
    // After Body recovery switches to In Body and reprocesses it.
    let observation = run("<body></body>&#160;");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.mode, Mode::InBody);
    assert_eq!(
        observation.recoveries,
        vec![Recovery::AfterBodyNonWhitespaceToInBody { scalar: '\u{a0}' }]
    );
    assert_eq!(observation.body_text(), "\u{a0}");
    assert_valid(&observation);

    // <title>&#38;</title>
    let observation = run("<title>&#38;</title>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.title_text(), "&");
    assert_eq!(observation.mode, Mode::InHead);
    assert!(observation.tree.shell_synthesized);
    assert_eq!(
        observation.tree.titles[0].close.as_ref().map(|c| c.start),
        Some(12)
    );
    assert_valid(&observation);

    // A mixed title keeps ordered authored contributions in one text node.
    let observation = run("<title>x&#x26;&#60;y</title>");
    assert_eq!(observation.title_text(), "x&<y");
    assert_eq!(observation.placed_contributions().len(), 4);
}

#[test]
fn t6_no_new_insertion_mode_is_needed() {
    // Whitespace classification follows the decoded scalar, not the spelling.
    for (reference, whitespace) in [
        ("&#9;", true),
        ("&#10;", true),
        ("&#12;", true),
        ("&#13;", true),
        ("&#32;", true),
        ("&#160;", false),
        ("&#x2003;", false),
        ("&#65;", false),
        ("&#0;", false),
        ("&#128;", false),
    ] {
        let observation = run(&format!("<body></body>{reference}"));
        assert_eq!(observation.completion, Completion::Complete, "{reference}");
        let expected_mode = if whitespace {
            Mode::AfterBody
        } else {
            Mode::InBody
        };
        assert_eq!(observation.mode, expected_mode, "{reference}");
        assert!(
            !spelling_based_is_whitespace(reference),
            "a spelling-based classifier would call every reference non-whitespace"
        );
        assert_valid(&observation);
    }
    // Literal whitespace and a whitespace reference agree.
    assert_eq!(run("<body></body>\t").mode, run("<body></body>&#9;").mode);
    // The model's tree has exactly the existing positions.
    let all = [
        Mode::Early,
        Mode::InHead,
        Mode::TitleText,
        Mode::InBody,
        Mode::AfterBody,
        Mode::AfterAfterBody,
    ];
    assert_eq!(all.len(), 6);
}

#[test]
fn t7_unselected_tree_frontiers_are_not_widened() {
    // Early whitespace-sensitive positions stay unsupported even for a
    // whitespace reference.
    for text in ["&#9;", "&#10;", "&#65;", "<title></title>&#9;"] {
        let observation = run(text);
        assert_eq!(
            observation.completion,
            Completion::Unsupported(Unsupported::Tree(TreeBoundary::CharacterDataBeforeBody)),
            "{text}"
        );
        assert!(observation.tree.body.is_none());
    }
    // AfterAfterBody character data stays unsupported, for both spellings.
    for text in ["<body></body></html>&#10;", "<body></body></html>\n"] {
        let observation = run(text);
        assert_eq!(
            observation.completion,
            Completion::Unsupported(Unsupported::Tree(
                TreeBoundary::CharacterDataInAfterAfterBody
            )),
            "{text}"
        );
    }
    // EOF inside Title text remains the existing non-complete checkpoint.
    let observation = run("<title>&#38;");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::Tree(TreeBoundary::EofInTitleText))
    );
    assert_eq!(observation.title_text(), "&");
    // A Title outside the selected positions is a tree boundary.
    let observation = run("<body><title>&#38;</title>");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::Tree(TreeBoundary::OutsideModelledCells))
    );
}

#[test]
fn p1_existing_provenance_shape_is_sufficient() {
    let text = "<body>a&#x41;&#66;&#;&#xZ;\u{1}&#67</body>";
    let observation = run(text);
    assert_eq!(observation.completion, Completion::Complete);
    assert_valid(&observation);
    let source = SourceText::new(SourceId::new(1), text.to_owned());
    for contribution in observation.lexed.contributions() {
        // One contiguous authored source range per contribution, real, and
        // anchored in the same source: no new provenance domain.
        let authored = &contribution.authored;
        assert_eq!(authored.source_id, source.id());
        assert_eq!(authored.raw, text[authored.start..authored.end]);
        assert_eq!(*authored, evidence(&source, authored.start, authored.end));
        if let Origin::ResolvedNumeric { .. } = contribution.origin {
            assert_ne!(
                contribution.interpreted, authored.raw,
                "authored != interpreted"
            );
            assert_eq!(contribution.interpreted.chars().count(), 1);
        }
    }
    // Recovering a span from the decoded length or by searching for the
    // decoded text is wrong, so evidence is carried, never reconstructed.
    let lexed = lex("a&#x41;");
    let resolved = lexed.contributions()[1].authored.clone();
    assert_eq!((resolved.start, resolved.end), (1, 7));
    assert_ne!((1, 1 + "A".len()), (resolved.start, resolved.end));
    assert_ne!(lexed.source_text.find('A'), Some(resolved.start));
    // The reference ordinal relates diagnostics to references without raw
    // ranges.
    let lexed = lex("&#0;&#65&#1;");
    assert_eq!(
        lexed.diagnostic_view(),
        vec![
            (NULL, Some(0), 3),
            (MISSING, Some(1), 8),
            (CONTROL_REFERENCE, Some(2), 11),
        ]
    );
    assert_eq!(lexed.entries.len(), 3);
}

#[test]
fn p2_source_identity_perturbation_changes_provenance_not_semantics() {
    let text = "<body>a&#x41;b&#66</body>";
    let first = construct(lex_with(
        &SourceText::new(SourceId::new(1), text.to_owned()),
        Ctx::Data,
        Limits::default(),
        None,
    ));
    let second = construct(lex_with(
        &SourceText::new(SourceId::new(77), text.to_owned()),
        Ctx::Data,
        Limits::default(),
        None,
    ));
    assert_ne!(first.lexed.source_id, second.lexed.source_id);
    assert_eq!(first.body_text(), second.body_text());
    assert_eq!(first.lexed.projection(), second.lexed.projection());
    assert_eq!(
        first.lexed.diagnostic_view(),
        second.lexed.diagnostic_view()
    );
    assert_eq!(first.lexed.usage, second.lexed.usage);
    assert!(
        second
            .lexed
            .diagnostics
            .iter()
            .all(|d| d.source_id == second.lexed.source_id)
    );
}

// ---------------------------------------------------------------------------
// Negative controls and compatibility pressure
// ---------------------------------------------------------------------------

struct RecordedSentinel {
    id: &'static str,
    source: &'static str,
    capability: &'static str,
    steps: usize,
}

/// The predecessor expectation, recorded here and never edited by this leaf.
const UNSUP_001: RecordedSentinel = RecordedSentinel {
    id: "UNSUP-001",
    source: "&#65;",
    capability: "numeric character reference in Data, Unsupported at `#`",
    steps: 2,
};

#[test]
fn s1_the_current_numeric_sentinels_become_obsolete() {
    // Every live `&#65;` sentinel expects Numeric-in-Data to be unsupported.
    // Under the selected semantics the same source completes with decoded
    // text, so those expectations cannot remain valid once Numeric is
    // enabled. This leaf records the pressure and migrates nothing.
    assert_eq!(UNSUP_001.source, "&#65;");
    for text in ["<body>&#65;", "<body></body>&#65;"] {
        let observation = run(text);
        assert_eq!(observation.completion, Completion::Complete, "{text}");
        assert_eq!(observation.lexed.finalized, vec![0], "{text}");
        assert!(observation.body_text().ends_with('A'), "{text}");
    }
    let observation = run("<title>&#65;</title>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.title_text(), "A");
}

#[test]
fn s2_unsup_001_cannot_be_silently_rewritten() {
    assert_eq!(UNSUP_001.id, "UNSUP-001");
    assert!(UNSUP_001.capability.contains("Unsupported"));
    let lexed = lex(UNSUP_001.source);
    // Its source, its refusal, and its committed transition count all change
    // meaning together: 2 recorded steps become 7 and the stop disappears.
    assert_eq!(UNSUP_001.steps, 2);
    assert_eq!(lexed.usage.steps, 7);
    assert!(lexed.stop.is_none() && lexed.reached_eof());
    assert_ne!(lexed.usage.steps, UNSUP_001.steps);
    // A third unrelated meaning would also need a different source; here the
    // honest statement is that the fixture's meaning is superseded and needs
    // explicit initial-inventory review (#112) in the placement gate.
    assert_eq!(lex("&#65;").trace.len(), 7);
}

#[test]
fn s3_numeric_zero_is_not_the_authored_nul_defect() {
    // Numeric zero is a normal finalized reference with a replacement.
    let observation = run("<body>&#0;</body>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.body_text(), "\u{fffd}");
    assert_eq!(observation.lexed.diagnostic_kinds(), vec![NULL]);
    let observation = run("<title>&#0;</title>");
    assert_eq!(observation.completion, Completion::Complete);
    assert_eq!(observation.title_text(), "\u{fffd}");
    // Authored NUL stays a separate, unmodelled, uncorrected boundary.
    let observation = run("<body>\u{0}</body>");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::Outside(Outside::DataNul))
    );
    assert!(observation.lexed.contributions().is_empty());
    assert!(!observation.body_text().contains('\u{fffd}'));
    assert!(!observation.body_text().contains('\u{0}'));
    let observation = run("<title>\u{0}</title>");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::Outside(Outside::RcdataNul))
    );
    assert!(observation.lexed.contributions().is_empty());
    // Authored NUL is not misread as the Numeric null diagnostic either.
    assert!(!observation.lexed.diagnostic_kinds().contains(&NULL));
}

#[test]
fn s4_attribute_value_general_rcdata_and_named_remain_outside() {
    // AttributeValue references: tag shapes with attributes are refused at
    // the tag, before any reference lifecycle starts.
    for text in [
        "<body class=\"&#65;\">",
        "<body class=&#x41;>",
        "<a x=&#65;>",
        "<body><div title=\"&#38;\">",
    ] {
        let observation = run(text);
        assert!(
            matches!(
                observation.completion,
                Completion::Unsupported(Unsupported::Outside(Outside::TagShape))
                    | Completion::Unsupported(Unsupported::Tree(_))
            ),
            "{text}"
        );
        assert!(observation.lexed.entries.is_empty(), "{text}");
        assert!(observation.lexed.finalized.is_empty(), "{text}");
    }
    // General RCDATA elements are not selected.
    let observation = run("<textarea>&#65;</textarea>");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::Outside(Outside::TagShape))
    );
    assert!(observation.lexed.entries.is_empty());
    // An authored `<` in selected RCDATA is not general RCDATA recovery.
    let observation = run("<title>&#65;<b</title>");
    assert_eq!(
        observation.completion,
        Completion::Unsupported(Unsupported::Outside(Outside::RcdataMarkup))
    );
    // Named references are unchanged by this leaf: their branch is neither
    // modelled nor claimed.
    for text in ["&amp;", "&notin;", "&x;"] {
        let lexed = lex(text);
        assert_eq!(
            lexed.stop,
            Some(Stop::Outside(Outside::NamedBranch)),
            "{text}"
        );
        assert!(lexed.contributions().is_empty() && lexed.finalized.is_empty());
    }
    // A bare ampersand still flushes literally and reconsumes.
    let lexed = lex("& ;");
    assert_eq!(
        lexed.projection(),
        vec![flushed(0, 1, "&"), lit(1, 3, " ;")]
    );
    assert!(lexed.diagnostics.is_empty());
}

// ---------------------------------------------------------------------------
// Controls
// ---------------------------------------------------------------------------

#[test]
fn c1_repeated_runs_are_deterministic() {
    for text in [
        "<body>a&#65;&#x42&#;&#xZ</body>",
        "<title>&#38;&#60;</title>",
    ] {
        assert_eq!(run(text), run(text));
    }
}

#[test]
fn c2_freeze_rejects_impossible_observations() {
    // Diagnostics: INPUT_CONTROL@11, MISSING@11, NULL@15 (the `;`).
    let good = run("<body>a&#65\u{1}&#0;</body>");
    assert_valid(&good);
    assert_eq!(
        good.lexed.diagnostic_view(),
        vec![
            (INPUT_CONTROL, None, 11),
            (MISSING, Some(0), 11),
            (NULL, Some(1), 15)
        ]
    );

    let mut bad = good.clone();
    bad.lexed.diagnostics.swap(0, 2);
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::DiagnosticOrderRegression)
    );

    let mut bad = good.clone();
    bad.lexed.finalized.clear();
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::NumericOutputWithoutFinalization)
    );

    let mut bad = good.clone();
    for token in &mut bad.lexed.tokens {
        if let Token::Characters(c) = token
            && matches!(c.origin, Origin::ResolvedNumeric { .. })
        {
            c.interpreted.push('B');
        }
    }
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::NumericShapeMismatch)
    );

    let mut bad = good.clone();
    bad.completion = Completion::ResourceLimit(Refusal {
        resource: Resource::TransitionSteps,
        limit: 1,
        attempted: 2,
    });
    bad.lexed.coverage_end = 3;
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::ContributionBeyondCoverage)
    );

    let mut bad = good.clone();
    bad.lexed.coverage_end -= 1;
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::CompleteWithoutFullCoverage)
    );

    let mut bad = good.clone();
    bad.lexed.tokens.pop();
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::CompleteWithoutFullCoverage)
    );

    let mut bad = good.clone();
    bad.completion = Completion::Unsupported(Unsupported::Outside(Outside::TagShape));
    bad.lexed.stop = Some(Stop::Outside(Outside::TagShape));
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::NonCompleteHasEndOfFile)
    );

    let mut bad = good.clone();
    bad.lexed.diagnostics[1].reference = Some(99);
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::DiagnosticReferenceOutOfRange)
    );

    let mut bad = good.clone();
    if let Some(Token::Characters(c)) = bad
        .lexed
        .tokens
        .iter_mut()
        .find(|t| matches!(t, Token::Characters(c) if c.origin == Origin::Literal))
    {
        c.interpreted = "z".to_owned();
    }
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::LiteralInterpretationMismatch)
    );

    let mut bad = good.clone();
    bad.tree.body.as_mut().unwrap().text_nodes.clear();
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::TreeContributionNotFromLexed)
    );

    let mut bad = good;
    bad.tree.body.as_mut().unwrap().text_nodes[0].text.push('!');
    assert_eq!(
        validate_freeze(&bad),
        Err(FreezeError::TextNodeDoesNotEqualContributions)
    );
}

#[test]
fn c3_gold_is_hand_authored_and_external_heads_are_markers_only() {
    for pin in [
        WHATWG_HEAD,
        WHATWG_SOURCE_BLOB,
        WPT_CHALLENGE_HEAD,
        HTML5LIB_CHALLENGE_HEAD,
    ] {
        assert_eq!(pin.len(), 40);
        assert!(pin.bytes().all(|b| b.is_ascii_hexdigit()));
    }
    // This module's source admits exactly the two source-identity primitives
    // and no production HTML path, production Numeric capability, or
    // production resource vocabulary.
    let this_file = include_str!("data_rcdata_numeric_reference_successor_validation.rs");
    let imports: Vec<&str> = this_file
        .lines()
        .map(str::trim_start)
        .filter(|line| *line == "use" || line.starts_with("use "))
        .collect();
    // The only other `use` is the function-local state-name import in k1.
    assert_eq!(
        imports,
        ["use crate::{SourceId, SourceText};", "use St::*;"]
    );

    let normalized: String = this_file
        .chars()
        .filter(|ch| !ch.is_ascii_whitespace())
        .collect();
    for forbidden in [
        ["html", "::tokenizer"].concat(),
        ["tree_construction", "::driver"].concat(),
        ["tree_construction", "::session"].concat(),
        ["tree_construction", "::result"].concat(),
        ["super", "::driver"].concat(),
        ["super", "::session"].concat(),
        ["super", "::result"].concat(),
        ["super", "::super"].concat(),
        ["frontend_analysis", "_cli"].concat(),
        ["Html", "Tokenizer"].concat(),
        ["Capa", "bility::"].concat(),
        ["Named", "CharacterReferenceMatch"].concat(),
    ] {
        assert!(
            !normalized.contains(&forbidden),
            "production path escaped the oracle boundary: {forbidden}"
        );
    }
}
