//! Candidate-independent validation for Issue #904.
//!
//! This module validates, before any production placement or implementation,
//! the AttributeValue character-reference successor semantics for the single
//! authored attribute of the exact SelectedOrdinary domain:
//!
//! ```text
//! element names: div | section | article | aside | footer | header | main | nav
//! attributes:    exactly one authored attribute (the one-attribute case)
//! ```
//!
//! It is **test-only** and changes no production behavior. It does not
//! authorize a production implementation, a public API, a new resource
//! dimension, or any change to the accepted #900 / PR #901 validator.
//!
//! # Selection and predecessor authority
//!
//! Selected by Issue #348 comment `6008703400`. Accepted and reused, not
//! re-proved here: #900 / PR #901 (exact tokenizer-complete authored attribute
//! evidence to exact SelectedOrdinary node identity), #902 / PR #903
//! (reference-free attribute projection), #392 / #396 / #398 (canonical Named
//! Character Reference data ownership), #874 / #876 (Data-state named
//! references), #878 / #880 (Data/RCDATA numeric references), and #390 / #394
//! (selected Title RCDATA named references).
//!
//! # The #901 split
//!
//! The #901 positive association theorem is **reused** compositionally and is
//! not duplicated. The #901 AttributeValue `&` negative control
//! (`UnsupportedCapability(CharacterReference(AttributeValue))`) is a
//! **historical boundary selected for future supersession**. It is left byte
//! for byte unchanged; `negative_control_in_901_is_preserved` pins that, and
//! the observation tests below record that current production still stops
//! there. Nothing here makes production pass the successor fixtures.
//!
//! # What is new, and how the expected meaning is owned
//!
//! The new theorem is composition in AttributeValue context:
//!
//! ```text
//! accepted reference recognition/data
//!   -> AttributeValue return context (three originating states)
//!   -> attribute-only semicolonless named exception / ambiguous behavior
//!   -> decoded output routed to the active attribute value (never Character
//!      tokens)
//!   -> exact completed authored attribute evidence
//!   -> existing #901 association (no new identity theorem)
//! ```
//!
//! Expected meaning has exactly two sources, both validator-owned:
//!
//! 1. the private, deliberately small [`Machine`] below, a unit-driven model of
//!    exactly one start tag with one attribute plus a text-only harness for the
//!    Data/RCDATA controls; it is not a second HTML tokenizer or tree builder;
//! 2. hand-authored GOLD: fixture bytes, hand-counted byte ranges, explicit
//!    expected Unicode strings and ordered diagnostics.
//!
//! Everything before `mod observation` imports only the `SourceText` /
//! `SourceId` anchoring primitives. In particular it does not name the
//! production tokenizer, its generated Named data or matcher, the tree session,
//! result, freeze, projector, or the CLI.
//! `model_does_not_import_production_semantics` enforces this on this file's own
//! text. The only code that touches the production tokenizer is the
//! `observation` module, which records historical-boundary and unchanged-control
//! observations and is never an oracle for new semantics.
//!
//! # Named corpus
//!
//! [`NAMED`] is a small hand-authored **test-local** subset of the WHATWG table,
//! as in the accepted #874 validator. It is faithful for every authored cell:
//! for each tested input, every WHATWG name that is a prefix of the remaining
//! input is present in the subset. It is not a production table, is never
//! generated from one, and never calls the production matcher.
//!
//! # Pinned state shape (WHATWG HTML, hand-transcribed)
//!
//! - Attribute value (double-quoted, single-quoted, unquoted) `&` sets the
//!   return state to the originating state and switches to Character Reference.
//! - Character Reference: ASCII alphanumeric reconsumes in Named; `#` selects
//!   Numeric; anything else flushes `&` and reconsumes in the return state.
//!   So `&;` is a literal `&` followed by an ordinary `;` with **no**
//!   diagnostic; only the ambiguous-ampersand state reports a `;`.
//! - Named: the maximum match. When consumed as part of an attribute, a match
//!   not ending in `;` that is followed by `=` or an ASCII alphanumeric is
//!   flushed literally and gives **no** diagnostic. Otherwise a match not ending
//!   in `;` reports `missing-semicolon-after-character-reference`, and the
//!   decoded value replaces the consumed text. No match flushes `&` and enters
//!   the ambiguous ampersand state.
//! - Ambiguous ampersand: ASCII alphanumerics are appended to the active
//!   attribute value; `;` reports `unknown-named-character-reference` and is
//!   reconsumed in the return state; anything else is reconsumed.
//! - Numeric: `x`/`X` selects hexadecimal; a digit of the radix reconsumes in
//!   the digit state; anything else is `absence-of-digits-in-numeric-character-
//!   reference` and flushes the consumed prefix literally. Digit states
//!   accumulate; `;` or any other unit ends the run (a missing `;` reports
//!   `missing-semicolon-after-character-reference` first) and Numeric End maps
//!   the value: zero, out-of-range and surrogate become U+FFFD with their
//!   diagnostics, noncharacters keep their value with a diagnostic, CR and
//!   controls report `control-character-reference`, and the C1 override table
//!   remaps 27 code points.
//! - Decoded output is output only. It never re-enters tokenizer input.
//!
//! WHATWG HTML `main` observed at prompt time: see [`WHATWG_PROMPT_TIME`].
//! WPT and html5lib are challenge/corroboration pins only and define nothing.
//! These markers are freshness markers recorded from the prompt; the upstream
//! text could not be re-fetched in the authoring environment, so the state shape
//! above is transcribed from the accepted predecessor validators (#874, #878)
//! and Issue #348 rather than from a fresh upstream read.
//!
//! # Resource theorem (no new dimension)
//!
//! The successor reuses `RetainedInterpretedBytes` and the fixed `Diagnostics`
//! budget. Retained bytes are the run-cumulative count: committed bytes of
//! earlier tokens, plus the tag name, the attribute name and the interpreted
//! value so far. A reference's whole effect (decoded output append, its
//! diagnostics, and consumption of its terminal unit) is **prepared at the
//! unit where its outcome becomes known and committed atomically**:
//!
//! - Named: prepared before the matched name is consumed; refusal leaves the
//!   cursor immediately after `&`.
//! - Numeric: digit units only accumulate in a fixed-size accumulator (no
//!   retained effect). Preparation happens at the terminal unit, before `;` or
//!   the following unit is consumed and before any End diagnostic or output.
//! - Multi-scalar output is all or nothing; the cost is the decoded UTF-8
//!   length, not the authored length and not the scalar count.
//!
//! The model keeps no temporary buffer (a fixed accumulator, and a forward
//! prefix walk over borrowed remaining input), so `TemporaryBufferBytes` stays
//! zero. If correct behavior could not fit these limits, this validation would
//! have stopped; it did not. Transition-step budgeting is the unchanged
//! lower-layer rule and is not modeled.
//!
//! # Adversarial sealing
//!
//! [`Probe`] names deliberately wrong designs. `every_wrong_design_is_caught`
//! shows each one is rejected by a named GOLD group. Probes live only in this
//! test-local model and never touch production.
//!
//! # Deliberately excluded
//!
//! Second attributes (modeled only as the accepted
//! `ResourceLimit(AttributesPerTag, 1, 2)` fact), other element families,
//! self-closing tags, valueless attributes, CR normalization, EOF inside a tag,
//! per-reference public evidence, new node kinds and any tree recovery.

use crate::{SourceId, SourceText};

/// WHATWG HTML `main` observed at prompt time (freshness marker only).
const WHATWG_PROMPT_TIME: &str = "bf84f3c5a9cf87ac0a9f6461f32151971ad2279b";
/// WPT challenge pin observed at prompt time (challenge only, never an oracle).
const WPT_PROMPT_TIME: &str = "c271c10de4c682ac8377dc00977d19dbb62f4a7d";
/// html5lib-tests challenge pin (challenge only, never an oracle).
const HTML5LIB_PROMPT_TIME: &str = "c777c408b61078ea2eb4acefc2535f54dbc8b28a";
/// Issue #348 selection comment.
const SELECTION_AUTHORITY_COMMENT: &str = "6008703400";

/// Product profile: exactly one authored attribute per tag.
const ATTRIBUTES_PER_TAG: usize = 1;

/// The selected SelectedOrdinary element names.
const SELECTED: [&str; 8] = [
    "div", "section", "article", "aside", "footer", "header", "main", "nav",
];

// ---------------------------------------------------------------------------
// Test-local Named Character Reference subset (hand-authored)
// ---------------------------------------------------------------------------

/// `(identifier without the authored '&', decoded value)`.
const NAMED: &[(&str, &str)] = &[
    ("NewLine;", "\u{000a}"),
    ("Not;", "\u{2aec}"),
    ("NotEqualTilde;", "\u{2242}\u{0338}"),
    ("Tab;", "\u{0009}"),
    ("amp", "\u{0026}"),
    ("amp;", "\u{0026}"),
    ("bne;", "\u{003d}\u{20e5}"),
    ("copy", "\u{00a9}"),
    ("copy;", "\u{00a9}"),
    ("gt", "\u{003e}"),
    ("gt;", "\u{003e}"),
    ("lt", "\u{003c}"),
    ("lt;", "\u{003c}"),
    ("nbsp", "\u{00a0}"),
    ("nbsp;", "\u{00a0}"),
    ("not", "\u{00ac}"),
    ("not;", "\u{00ac}"),
    ("notin;", "\u{2209}"),
];

/// Maximum (or, for the wrong-design probe, shortest) complete identifier that
/// is a prefix of `rest`. A pure forward walk over borrowed input; no buffer.
fn named_match(rest: &str, shortest: bool) -> Option<(&'static str, &'static str)> {
    let candidates = NAMED.iter().filter(|(name, _)| rest.starts_with(name));
    if shortest {
        candidates.min_by_key(|(name, _)| name.len()).copied()
    } else {
        candidates.max_by_key(|(name, _)| name.len()).copied()
    }
}

// ---------------------------------------------------------------------------
// Numeric End: the normative mapping (hand-authored, input-free)
// ---------------------------------------------------------------------------

/// Saturation sentinel above U+10FFFF. Appending a digit never decreases the
/// accumulated code, so clamping preserves the out-of-range comparison for
/// every digit run, however long, with no host integer parse.
const SATURATED: u32 = 0x11_0000;

/// The pinned C1 override rows. 0x81, 0x8D, 0x8F, 0x90 and 0x9D have no row.
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

/// Numeric Character Reference End, transcribed bullet by bullet. The bullets
/// are sequential, so the result is an ordered list.
fn numeric_end(code: u32, probe: Probe) -> (char, Vec<Kind>) {
    let mut code = code;
    let mut kinds = Vec::new();
    let replace = probe != Probe::NoInvalidReplacement;
    if code == 0 {
        kinds.push(Kind::NullCharacterReference);
        if replace {
            code = 0xFFFD;
        }
    }
    if code > 0x10_FFFF {
        kinds.push(Kind::CharacterReferenceOutsideUnicodeRange);
        code = 0xFFFD;
    }
    if (0xD800..=0xDFFF).contains(&code) {
        kinds.push(Kind::SurrogateCharacterReference);
        code = 0xFFFD;
    }
    if is_noncharacter(code) {
        kinds.push(Kind::NoncharacterCharacterReference);
    }
    if code == 0x0D || (is_control(code) && !matches!(code, 0x09 | 0x0A | 0x0C | 0x20)) {
        kinds.push(Kind::ControlCharacterReference);
        if probe != Probe::NoControlRemap
            && let Some((_, mapped)) = C1_OVERRIDES.iter().find(|(from, _)| *from == code)
        {
            code = *mapped;
        }
    }
    (char::from_u32(code).unwrap_or('\u{fffd}'), kinds)
}

// ---------------------------------------------------------------------------
// Model vocabulary
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    DoubleQuoted,
    SingleQuoted,
    Unquoted,
}

const SYNTAXES: [Syntax; 3] = [Syntax::DoubleQuoted, Syntax::SingleQuoted, Syntax::Unquoted];

/// The originating (return) state of a character reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Return {
    Data,
    Rcdata,
    Attribute(Syntax),
}

impl Return {
    fn is_attribute(self) -> bool {
        matches!(self, Return::Attribute(_))
    }

    fn sink(self) -> Sink {
        if self.is_attribute() {
            Sink::AttributeValue
        } else {
            Sink::CharacterToken
        }
    }
}

/// Where decoded output goes. Routing is observable: attribute output is
/// appended to the active attribute value, never emitted as document Character
/// tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sink {
    AttributeValue,
    CharacterToken,
}

/// Test-local diagnostic vocabulary (the accepted categories reached here).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    UnexpectedNullCharacter,
    UnexpectedCharacterInUnquotedAttributeValue,
    MissingSemicolonAfterCharacterReference,
    UnknownNamedCharacterReference,
    AbsenceOfDigitsInNumericCharacterReference,
    NullCharacterReference,
    CharacterReferenceOutsideUnicodeRange,
    SurrogateCharacterReference,
    NoncharacterCharacterReference,
    ControlCharacterReference,
}

const MISSING: Kind = Kind::MissingSemicolonAfterCharacterReference;
const UNKNOWN: Kind = Kind::UnknownNamedCharacterReference;
const ABSENCE: Kind = Kind::AbsenceOfDigitsInNumericCharacterReference;
const NULL_REF: Kind = Kind::NullCharacterReference;
const OUTSIDE_RANGE: Kind = Kind::CharacterReferenceOutsideUnicodeRange;
const SURROGATE: Kind = Kind::SurrogateCharacterReference;
const NONCHARACTER: Kind = Kind::NoncharacterCharacterReference;
const CONTROL_REF: Kind = Kind::ControlCharacterReference;
const UNEXPECTED_NULL: Kind = Kind::UnexpectedNullCharacter;
const UNEXPECTED_UNQUOTED: Kind = Kind::UnexpectedCharacterInUnquotedAttributeValue;

/// A diagnostic's semantic site. `Unit` is the start offset of the unit being
/// dispatched when it was raised. `Reference` is the ordinal of the reference
/// episode the diagnostic belongs to. Neither is a production anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Site {
    Unit(usize),
    Reference(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Diag {
    kind: Kind,
    site: Site,
}

/// How one reference episode resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Res {
    /// `&` followed by a non-alphanumeric, non-`#` unit: flushed literally.
    Bare,
    /// Named maximum match resolved to its decoded value.
    Named,
    /// Named semicolonless match blocked by the attribute-only exception and
    /// flushed literally.
    Blocked,
    /// No named match: `&` plus the alphanumeric run, literally.
    Ambiguous,
    NumDec,
    NumHex,
    /// `&#` / `&#x` with no digit: the consumed prefix, literally.
    Absent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Episode {
    entry: usize,
    /// Start of the unit handed back to `return_to` (reconsumed or next).
    consumed_end: usize,
    return_to: Return,
    res: Res,
    sink: Sink,
    produced: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Resource {
    AttributesPerTag,
    RetainedInterpretedBytes,
    Diagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Refusal {
    resource: Resource,
    limit: usize,
    attempted: usize,
    /// Start of the first unconsumed unit when the refused effect was prepared.
    at: usize,
}

/// Shapes the model deliberately does not cover. Distinct from a resource
/// refusal and from a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outside {
    TagShape,
    UnsupportedElement,
    SelfClosing,
    ValuelessAttribute,
    EofInTag,
    TrailingInput,
    CarriageReturn,
    Markup,
    DataNul,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    Resource(Refusal),
    Outside(Outside),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Complete,
    Resource(Refusal),
    Outside(Outside),
}

#[derive(Debug, Clone, Copy, Default)]
struct Limits {
    retained: Option<usize>,
    diagnostics: Option<usize>,
}

const UNLIMITED: Limits = Limits {
    retained: None,
    diagnostics: None,
};

/// Deliberately wrong designs the GOLD must reject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Probe {
    None,
    /// Quoted states return to the Unquoted state after a reference.
    WrongReturnState,
    /// Reference output is emitted as document Character tokens.
    EmitAsCharacterToken,
    /// The attribute-only semicolonless exception is omitted.
    NoAttributeException,
    /// The attribute-only exception is also applied outside attributes.
    ExceptionEverywhere,
    /// The first (shortest) match is taken instead of the maximum match.
    ShortestMatch,
    /// The missing-semicolon diagnostic is omitted.
    NoMissingSemicolonDiag,
    /// The unit after a reference is swallowed.
    SwallowDelimiter,
    /// Multi-scalar output is appended in reverse order.
    SwapScalars,
    /// Null is not replaced by U+FFFD.
    NoInvalidReplacement,
    /// Numeric accumulation wraps a 32-bit integer instead of saturating.
    WrappingNumeric,
    /// The C1 override table is not applied.
    NoControlRemap,
    /// The reference is consumed first and the resource refusal found after.
    ConsumeThenRefuse,
    /// Multi-scalar output is appended scalar by scalar with per-scalar checks.
    PartialAppend,
    /// Retained cost is the authored length, not the decoded UTF-8 length.
    RetainedByAuthoredLength,
    /// Retained cost is the scalar count, not the decoded UTF-8 length.
    RetainedByScalarCount,
    /// The `;` ending an unknown name is consumed.
    SwallowAmbiguousSemicolon,
    /// `&;` reports unknown-named-character-reference.
    SemicolonDiagOnBareAmpersand,
}

// ---------------------------------------------------------------------------
// Authored evidence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evidence {
    source_id: SourceId,
    start: usize,
    end: usize,
    raw: String,
}

fn evidence(source: &SourceText, start: usize, end: usize) -> Evidence {
    let anchor = source.anchor(start, end).expect("fixture byte range");
    Evidence {
        source_id: anchor.source_id(),
        start: anchor.range().start(),
        end: anchor.range().end(),
        raw: anchor.fragment().to_owned(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttributeEvidence {
    complete: Evidence,
    name: Evidence,
    interpreted_name: String,
    syntax: Syntax,
    equals: Evidence,
    open_quote: Option<Evidence>,
    value: Evidence,
    close_quote: Option<Evidence>,
    interpreted_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TagEvidence {
    element: String,
    tag: Evidence,
    attribute: Option<AttributeEvidence>,
}

/// A tiny composition witness: one model-local identity plus the completed tag
/// evidence. It owns no stack, recovery, insertion mode, or dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ModelNode {
    id: usize,
    element: String,
    attribute: Option<AttributeEvidence>,
}

fn associate(id: usize, tag: &TagEvidence) -> ModelNode {
    ModelNode {
        id,
        element: tag.element.clone(),
        attribute: tag.attribute.clone(),
    }
}

#[derive(Debug, Clone)]
struct Lexed {
    outcome: Outcome,
    tag: Option<TagEvidence>,
    /// The active attribute value builder content at stop.
    value: String,
    /// Document Character-token text at stop (Data/RCDATA harness, or a wrong
    /// design).
    document_text: String,
    diags: Vec<Diag>,
    episodes: Vec<Episode>,
    /// Value lengths at every commit boundary.
    boundaries: Vec<usize>,
    /// End of the last consumed unit.
    coverage_end: usize,
    /// Retained bytes owned outside the value (committed prior + names).
    base: usize,
    /// Retained bytes at stop.
    retained: usize,
}

// ---------------------------------------------------------------------------
// The model
// ---------------------------------------------------------------------------

struct Plan {
    entry: usize,
    consumed_end: usize,
    ret: Return,
    res: Res,
    produced: String,
    diags: Vec<Diag>,
    at: usize,
}

struct Machine<'a> {
    text: &'a str,
    units: Vec<(usize, char)>,
    idx: usize,
    probe: Probe,
    limits: Limits,
    base: usize,
    value: String,
    document_text: String,
    diags: Vec<Diag>,
    episodes: Vec<Episode>,
    boundaries: Vec<usize>,
    coverage_end: usize,
}

impl<'a> Machine<'a> {
    fn new(text: &'a str, limits: Limits, probe: Probe) -> Self {
        Self {
            text,
            units: text.char_indices().collect(),
            idx: 0,
            probe,
            limits,
            base: 0,
            value: String::new(),
            document_text: String::new(),
            diags: Vec::new(),
            episodes: Vec::new(),
            boundaries: vec![0],
            coverage_end: 0,
        }
    }

    fn peek(&self) -> Option<(usize, char)> {
        self.units.get(self.idx).copied()
    }

    /// Start of the first unconsumed unit, or the end of input.
    fn offset(&self) -> usize {
        self.units.get(self.idx).map_or(self.text.len(), |u| u.0)
    }

    fn consume(&mut self) {
        let (start, ch) = self.units[self.idx];
        self.idx += 1;
        self.coverage_end = start + ch.len_utf8();
    }

    fn consume_to(&mut self, end: usize) {
        while self.units.get(self.idx).is_some_and(|u| u.0 < end) {
            self.consume();
        }
    }

    /// Next unit for a reference lookahead. EOF inside a tag is outside the
    /// model; EOF in the Data/RCDATA harness is a legitimate end of input.
    fn next_unit(&self, ret: Return) -> Result<Option<(usize, char)>, Stop> {
        match self.peek() {
            None if ret.is_attribute() => Err(Stop::Outside(Outside::EofInTag)),
            other => Ok(other),
        }
    }

    fn retained_now(&self) -> usize {
        self.base + self.value.len() + self.document_text.len()
    }

    /// Prepares the refusable cost of one effect. Mutates nothing.
    fn reserve(&self, bytes: usize, diags: usize, at: usize) -> Result<(), Stop> {
        if diags > 0
            && let Some(limit) = self.limits.diagnostics
        {
            let attempted = self.diags.len() + diags;
            if attempted > limit {
                return Err(Stop::Resource(Refusal {
                    resource: Resource::Diagnostics,
                    limit,
                    attempted,
                    at,
                }));
            }
        }
        if bytes > 0
            && let Some(limit) = self.limits.retained
        {
            let attempted = self.retained_now() + bytes;
            if attempted > limit {
                return Err(Stop::Resource(Refusal {
                    resource: Resource::RetainedInterpretedBytes,
                    limit,
                    attempted,
                    at,
                }));
            }
        }
        Ok(())
    }

    fn append(&mut self, sink: Sink, text: &str) {
        match sink {
            Sink::AttributeValue => self.value.push_str(text),
            Sink::CharacterToken => self.document_text.push_str(text),
        }
    }

    fn sink_for_reference(&self, ret: Return) -> Sink {
        if self.probe == Probe::EmitAsCharacterToken {
            Sink::CharacterToken
        } else {
            ret.sink()
        }
    }

    fn append_char(&mut self, ret: Return, ch: char) -> Result<(), Stop> {
        let at = self.offset();
        self.reserve(ch.len_utf8(), 0, at)?;
        self.consume();
        let mut buffer = [0u8; 4];
        self.append(ret.sink(), ch.encode_utf8(&mut buffer));
        self.boundaries.push(self.value.len());
        Ok(())
    }

    /// Commits one reference episode atomically: prepare, then consume, append
    /// and record. The wrong-design probes break exactly one of those steps.
    fn commit(&mut self, plan: Plan) -> Result<(), Stop> {
        let sink = self.sink_for_reference(plan.ret);
        let cost = match self.probe {
            Probe::RetainedByAuthoredLength => plan.consumed_end - plan.entry,
            Probe::RetainedByScalarCount => plan.produced.chars().count(),
            _ => plan.produced.len(),
        };
        if self.probe == Probe::ConsumeThenRefuse {
            self.consume_to(plan.consumed_end);
        }
        if self.probe == Probe::PartialAppend {
            self.reserve(0, plan.diags.len(), plan.at)?;
            for scalar in plan.produced.chars() {
                let mut buffer = [0u8; 4];
                self.reserve(scalar.len_utf8(), 0, plan.at)?;
                self.append(sink, scalar.encode_utf8(&mut buffer));
            }
            self.consume_to(plan.consumed_end);
        } else {
            self.reserve(cost, plan.diags.len(), plan.at)?;
            self.consume_to(plan.consumed_end);
            self.append(sink, &plan.produced);
        }
        self.diags.extend(plan.diags);
        self.boundaries.push(self.value.len());
        self.episodes.push(Episode {
            entry: plan.entry,
            consumed_end: plan.consumed_end,
            return_to: plan.ret,
            res: plan.res,
            sink,
            produced: plan.produced,
        });
        Ok(())
    }

    /// Character Reference, entered at an authored `&`. Returns the state the
    /// caller resumes in.
    fn reference(&mut self, ret: Return) -> Result<Return, Stop> {
        let (entry, _) = self.units[self.idx];
        self.consume();
        let ordinal = self.episodes.len();
        let resume = match (self.probe, ret) {
            (
                Probe::WrongReturnState,
                Return::Attribute(Syntax::DoubleQuoted | Syntax::SingleQuoted),
            ) => Return::Attribute(Syntax::Unquoted),
            _ => ret,
        };
        match self.next_unit(ret)? {
            Some((_, ch)) if ch.is_ascii_alphanumeric() => self.named(ret, entry, ordinal)?,
            Some((_, '#')) => self.numeric(ret, entry, ordinal)?,
            next => {
                let mut diags = Vec::new();
                if self.probe == Probe::SemicolonDiagOnBareAmpersand
                    && let Some((start, ';')) = next
                {
                    diags.push(Diag {
                        kind: UNKNOWN,
                        site: Site::Unit(start),
                    });
                }
                self.commit(Plan {
                    entry,
                    consumed_end: entry + 1,
                    ret,
                    res: Res::Bare,
                    produced: "&".to_owned(),
                    diags,
                    at: entry + 1,
                })?;
            }
        }
        if self.probe == Probe::SwallowDelimiter && self.peek().is_some() {
            self.consume();
        }
        Ok(resume)
    }

    fn named(&mut self, ret: Return, entry: usize, ordinal: usize) -> Result<(), Stop> {
        let start = self.offset();
        let found = named_match(&self.text[start..], self.probe == Probe::ShortestMatch);
        let Some((name, value)) = found else {
            return self.ambiguous(ret, entry);
        };
        let name_end = start + name.len();
        let after = self.text[name_end..].chars().next();
        if ret.is_attribute() && after.is_none() {
            return Err(Stop::Outside(Outside::EofInTag));
        }
        let semicolon = name.ends_with(';');
        let rule_applies = match self.probe {
            Probe::NoAttributeException => false,
            Probe::ExceptionEverywhere => true,
            _ => ret.is_attribute(),
        };
        let blocked = !semicolon
            && rule_applies
            && after.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '=');
        let plan = if blocked {
            Plan {
                entry,
                consumed_end: name_end,
                ret,
                res: Res::Blocked,
                produced: format!("&{name}"),
                diags: Vec::new(),
                at: start,
            }
        } else {
            let produced = if self.probe == Probe::SwapScalars {
                value.chars().rev().collect()
            } else {
                value.to_owned()
            };
            let mut diags = Vec::new();
            if !semicolon && self.probe != Probe::NoMissingSemicolonDiag {
                diags.push(Diag {
                    kind: MISSING,
                    site: Site::Reference(ordinal),
                });
            }
            Plan {
                entry,
                consumed_end: name_end,
                ret,
                res: Res::Named,
                produced,
                diags,
                at: start,
            }
        };
        self.commit(plan)
    }

    /// No named match: `&` is flushed, then the ASCII alphanumeric run is
    /// appended unit by unit like ordinary characters.
    fn ambiguous(&mut self, ret: Return, entry: usize) -> Result<(), Stop> {
        let sink = self.sink_for_reference(ret);
        let at = self.offset();
        self.reserve(1, 0, at)?;
        self.append(sink, "&");
        self.boundaries.push(self.value.len());
        while let Some((start, ch)) = self.peek() {
            if !ch.is_ascii_alphanumeric() {
                break;
            }
            self.reserve(ch.len_utf8(), 0, start)?;
            self.consume();
            let mut buffer = [0u8; 4];
            self.append(sink, ch.encode_utf8(&mut buffer));
            self.boundaries.push(self.value.len());
        }
        let terminal = self.next_unit(ret)?;
        let consumed_end = self.offset();
        if let Some((start, ';')) = terminal {
            if self.probe == Probe::SwallowAmbiguousSemicolon {
                self.consume();
            } else {
                self.reserve(0, 1, start)?;
                self.diags.push(Diag {
                    kind: UNKNOWN,
                    site: Site::Unit(start),
                });
            }
        }
        self.episodes.push(Episode {
            entry,
            consumed_end,
            return_to: ret,
            res: Res::Ambiguous,
            sink,
            produced: self.text[entry..consumed_end].to_owned(),
        });
        Ok(())
    }

    fn numeric(&mut self, ret: Return, entry: usize, ordinal: usize) -> Result<(), Stop> {
        self.consume();
        let mut base = 10u32;
        let mut res = Res::NumDec;
        if let Some((_, 'x' | 'X')) = self.next_unit(ret)? {
            self.consume();
            base = 16;
            res = Res::NumHex;
        }
        let has_digit = self
            .next_unit(ret)?
            .is_some_and(|(_, ch)| ch.is_digit(base));
        if !has_digit {
            let at = self.offset();
            return self.commit(Plan {
                entry,
                consumed_end: at,
                ret,
                res: Res::Absent,
                produced: self.text[entry..at].to_owned(),
                diags: vec![Diag {
                    kind: ABSENCE,
                    site: Site::Unit(at),
                }],
                at,
            });
        }
        let mut code: u32 = 0;
        while let Some((_, ch)) = self.peek() {
            let Some(digit) = ch.to_digit(base) else {
                break;
            };
            code = if self.probe == Probe::WrappingNumeric {
                code.wrapping_mul(base).wrapping_add(digit)
            } else {
                (code * base + digit).min(SATURATED)
            };
            self.consume();
        }
        let terminal = self.next_unit(ret)?;
        let at = self.offset();
        let semicolon = matches!(terminal, Some((_, ';')));
        let (scalar, kinds) = numeric_end(code, self.probe);
        let mut diags = Vec::new();
        if !semicolon {
            diags.push(Diag {
                kind: MISSING,
                site: Site::Unit(at),
            });
        }
        diags.extend(kinds.into_iter().map(|kind| Diag {
            kind,
            site: Site::Reference(ordinal),
        }));
        self.commit(Plan {
            entry,
            consumed_end: at + usize::from(semicolon),
            ret,
            res,
            produced: scalar.to_string(),
            diags,
            at,
        })
    }

    fn finish(self, outcome: Outcome, tag: Option<TagEvidence>) -> Lexed {
        let retained = self.retained_now();
        Lexed {
            outcome,
            tag,
            value: self.value,
            document_text: self.document_text,
            diags: self.diags,
            episodes: self.episodes,
            boundaries: self.boundaries,
            coverage_end: self.coverage_end,
            base: self.base,
            retained,
        }
    }
}

fn is_html_whitespace(ch: char) -> bool {
    matches!(ch, '\t' | '\n' | '\u{000c}' | ' ')
}

/// The attribute value states. The close quote / terminator is not consumed.
fn lex_value(m: &mut Machine<'_>, syntax: Syntax) -> Result<(), Stop> {
    let mut state = Return::Attribute(syntax);
    loop {
        let Return::Attribute(current) = state else {
            unreachable!("attribute lexing never leaves attribute states");
        };
        let Some((start, ch)) = m.peek() else {
            return Err(Stop::Outside(Outside::EofInTag));
        };
        match (current, ch) {
            (_, '\r') => return Err(Stop::Outside(Outside::CarriageReturn)),
            (Syntax::DoubleQuoted, '"') | (Syntax::SingleQuoted, '\'') => return Ok(()),
            (Syntax::Unquoted, ch) if is_html_whitespace(ch) || ch == '>' => return Ok(()),
            (_, '&') => state = m.reference(state)?,
            (_, '\0') => {
                m.reserve('\u{fffd}'.len_utf8(), 1, start)?;
                m.consume();
                m.diags.push(Diag {
                    kind: UNEXPECTED_NULL,
                    site: Site::Unit(start),
                });
                m.append(Sink::AttributeValue, "\u{fffd}");
                m.boundaries.push(m.value.len());
            }
            (Syntax::Unquoted, '"' | '\'' | '<' | '=' | '`') => {
                m.reserve(ch.len_utf8(), 1, start)?;
                m.consume();
                m.diags.push(Diag {
                    kind: UNEXPECTED_UNQUOTED,
                    site: Site::Unit(start),
                });
                let mut buffer = [0u8; 4];
                m.append(Sink::AttributeValue, ch.encode_utf8(&mut buffer));
                m.boundaries.push(m.value.len());
            }
            (_, ch) => m.append_char(state, ch)?,
        }
    }
}

/// Lexes exactly: an optional `<body>` prefix, then one selected start tag with
/// zero or one authored attribute, then end of input.
fn run_tag(m: &mut Machine<'_>, source: &SourceText) -> Result<TagEvidence, Stop> {
    let outside = |kind| Err(Stop::Outside(kind));
    let mut committed = 0;
    if m.text.starts_with("<body>") {
        m.consume_to(6);
        committed = "body".len();
    }
    let tag_start = m.offset();
    if m.peek().map(|u| u.1) != Some('<') {
        return outside(Outside::TagShape);
    }
    m.consume();
    let name_start = m.offset();
    while m.peek().is_some_and(|u| u.1.is_ascii_alphabetic()) {
        m.consume();
    }
    let name_end = m.offset();
    let element = m.text[name_start..name_end].to_ascii_lowercase();
    if element.is_empty() {
        return outside(Outside::TagShape);
    }
    if !SELECTED.contains(&element.as_str()) {
        return outside(Outside::UnsupportedElement);
    }
    m.base = committed + element.len();
    let finish_tag = |m: &mut Machine<'_>, attribute: Option<AttributeEvidence>| {
        m.consume();
        if m.peek().is_some() {
            return Err(Stop::Outside(Outside::TrailingInput));
        }
        Ok(TagEvidence {
            element: element.clone(),
            tag: evidence(source, tag_start, m.coverage_end),
            attribute,
        })
    };
    match m.peek().map(|u| u.1) {
        Some('>') => return finish_tag(m, None),
        Some('/') => return outside(Outside::SelfClosing),
        Some(ch) if is_html_whitespace(ch) => {}
        _ => return outside(Outside::TagShape),
    }
    while m.peek().is_some_and(|u| is_html_whitespace(u.1)) {
        m.consume();
    }
    match m.peek().map(|u| u.1) {
        Some('>') => return finish_tag(m, None),
        Some('/') => return outside(Outside::SelfClosing),
        Some(ch) if ch.is_ascii_alphabetic() => {}
        _ => return outside(Outside::TagShape),
    }
    let attr_start = m.offset();
    while m
        .peek()
        .is_some_and(|u| u.1.is_ascii_alphabetic() || u.1 == '-')
    {
        m.consume();
    }
    let attr_name_end = m.offset();
    let interpreted_name = m.text[attr_start..attr_name_end].to_ascii_lowercase();
    m.base += interpreted_name.len();
    if m.peek().map(|u| u.1) != Some('=') {
        return outside(Outside::ValuelessAttribute);
    }
    let equals_start = m.offset();
    m.consume();
    let equals_end = m.coverage_end;
    let syntax = match m.peek().map(|u| u.1) {
        Some('"') => Syntax::DoubleQuoted,
        Some('\'') => Syntax::SingleQuoted,
        Some(ch) if is_html_whitespace(ch) || ch == '>' => {
            return outside(Outside::ValuelessAttribute);
        }
        None => return outside(Outside::EofInTag),
        _ => Syntax::Unquoted,
    };
    let quoted = syntax != Syntax::Unquoted;
    let open_quote = if quoted {
        let start = m.offset();
        m.consume();
        Some((start, m.coverage_end))
    } else {
        None
    };
    let value_start = m.offset();
    lex_value(m, syntax)?;
    let value_end = m.offset();
    let close_quote = if quoted {
        let start = m.offset();
        m.consume();
        Some((start, m.coverage_end))
    } else {
        None
    };
    let complete_end = m.coverage_end.max(value_end);
    let mut separated = !quoted;
    loop {
        match m.peek() {
            None => return outside(Outside::EofInTag),
            Some((_, ch)) if is_html_whitespace(ch) => {
                separated = true;
                m.consume();
            }
            Some((_, '>')) => break,
            Some((_, '/')) => return outside(Outside::SelfClosing),
            Some((at, ch)) if separated && ch.is_ascii_alphabetic() => {
                return Err(Stop::Resource(Refusal {
                    resource: Resource::AttributesPerTag,
                    limit: ATTRIBUTES_PER_TAG,
                    attempted: ATTRIBUTES_PER_TAG + 1,
                    at,
                }));
            }
            Some(_) => return outside(Outside::TagShape),
        }
    }
    let interpreted_value = m.value.clone();
    let attribute = AttributeEvidence {
        complete: evidence(source, attr_start, complete_end),
        name: evidence(source, attr_start, attr_name_end),
        interpreted_name,
        syntax,
        equals: evidence(source, equals_start, equals_end),
        open_quote: open_quote.map(|(start, end)| evidence(source, start, end)),
        value: evidence(source, value_start, value_end),
        close_quote: close_quote.map(|(start, end)| evidence(source, start, end)),
        interpreted_value,
    };
    finish_tag(m, Some(attribute))
}

fn lex_with(text: &str, source_id: u64, limits: Limits, probe: Probe) -> Lexed {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    let mut machine = Machine::new(text, limits, probe);
    match run_tag(&mut machine, &source) {
        Ok(tag) => machine.finish(Outcome::Complete, Some(tag)),
        Err(Stop::Resource(refusal)) => machine.finish(Outcome::Resource(refusal), None),
        Err(Stop::Outside(outside)) => machine.finish(Outcome::Outside(outside), None),
    }
}

fn lex(text: &str) -> Lexed {
    lex_with(text, 1, UNLIMITED, Probe::None)
}

/// The text-only harness for the Data and RCDATA controls. It shares the same
/// reference engine; only the return state, the sink, and the absence of the
/// attribute-only exception differ.
fn lex_text(text: &str, ret: Return, limits: Limits, probe: Probe) -> Lexed {
    fn run(m: &mut Machine<'_>, ret: Return) -> Result<(), Stop> {
        while let Some((_, ch)) = m.peek() {
            match ch {
                '&' => {
                    m.reference(ret)?;
                }
                '<' => return Err(Stop::Outside(Outside::Markup)),
                '\0' => return Err(Stop::Outside(Outside::DataNul)),
                '\r' => return Err(Stop::Outside(Outside::CarriageReturn)),
                ch => m.append_char(ret, ch)?,
            }
        }
        Ok(())
    }
    let mut machine = Machine::new(text, limits, probe);
    match run(&mut machine, ret) {
        Ok(()) => machine.finish(Outcome::Complete, None),
        Err(Stop::Resource(refusal)) => machine.finish(Outcome::Resource(refusal), None),
        Err(Stop::Outside(outside)) => machine.finish(Outcome::Outside(outside), None),
    }
}

// ---------------------------------------------------------------------------
// Observation of current production (never an oracle)
// ---------------------------------------------------------------------------

mod observation {
    use crate::html::token::{HtmlAttributeValueSyntax, HtmlToken};
    use crate::html::tokenizer::diagnostic::HtmlTokenizerDiagnosticCode;
    use crate::html::tokenizer::producer::tokenize;
    use crate::html::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerResource};
    use crate::html::tokenizer::result::{
        HtmlCharacterReferenceContext, HtmlTokenizerCapability, HtmlTokenizerCompletion,
        HtmlTokenizerIncompleteCause, HtmlTokenizerRunResult, HtmlTokenizerUnsupportedTrigger,
    };
    use crate::{SourceId, SourceText};

    /// The frozen Product tokenizer vector, restated as a lower-layer resource
    /// fact: source 36_864, steps 79_872, tokens 6_144, diagnostics 256,
    /// attributes per tag 1, retained bytes 49_152, temporary buffer 0.
    pub(super) fn product_limits() -> HtmlTokenizerLimits {
        HtmlTokenizerLimits::new(36_864, 79_872, 6_144, 256, 1, 49_152, 0)
    }

    pub(super) fn run(text: &str) -> HtmlTokenizerRunResult {
        tokenize(
            &SourceText::new(SourceId::new(1), text.to_owned()),
            product_limits(),
        )
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum Stop {
        Complete,
        /// `UnsupportedCapability(CharacterReference(AttributeValue))` at the
        /// given authored `&` range.
        AttributeValueReference {
            at: (usize, usize),
        },
        AttributesPerTag {
            limit: usize,
            attempted: usize,
            at: (usize, usize),
        },
        Other,
    }

    pub(super) fn stop(run: &HtmlTokenizerRunResult) -> Stop {
        match run.completion() {
            HtmlTokenizerCompletion::Complete => Stop::Complete,
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported),
            ) => match (unsupported.capability(), unsupported.trigger()) {
                (
                    HtmlTokenizerCapability::CharacterReference {
                        context: HtmlCharacterReferenceContext::AttributeValue,
                    },
                    HtmlTokenizerUnsupportedTrigger::Input(anchor),
                ) => Stop::AttributeValueReference {
                    at: (anchor.range().start(), anchor.range().end()),
                },
                _ => Stop::Other,
            },
            HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
                limit,
            )) if limit.resource() == HtmlTokenizerResource::AttributesPerTag => {
                Stop::AttributesPerTag {
                    limit: limit.limit(),
                    attempted: limit.attempted(),
                    at: (limit.at().range().start(), limit.at().range().end()),
                }
            }
            HtmlTokenizerCompletion::Incomplete(_) => Stop::Other,
        }
    }

    /// `(interpreted tag name, interpreted attribute name, interpreted value,
    /// complete range, name range, value range, syntax label)` of the last
    /// attributed start tag.
    pub(super) type AttributeView = (
        String,
        String,
        String,
        (usize, usize),
        (usize, usize),
        (usize, usize),
        &'static str,
    );

    pub(super) fn attributed_start_tag(run: &HtmlTokenizerRunResult) -> Option<AttributeView> {
        run.tokens().iter().rev().find_map(|token| {
            let HtmlToken::Tag(tag) = token else {
                return None;
            };
            let attribute = tag.attributes().first()?;
            let (value, label) = match attribute.value_syntax() {
                HtmlAttributeValueSyntax::Unquoted { value, .. } => (value, "unquoted"),
                HtmlAttributeValueSyntax::DoubleQuoted { value, .. } => (value, "double"),
                HtmlAttributeValueSyntax::SingleQuoted { value, .. } => (value, "single"),
                _ => return None,
            };
            let range =
                |anchor: &crate::SourceAnchor| (anchor.range().start(), anchor.range().end());
            Some((
                tag.name().interpreted().to_owned(),
                attribute.name().interpreted().to_owned(),
                attribute.interpreted_value().to_owned(),
                range(attribute.complete()),
                range(attribute.name().source()),
                range(value),
                label,
            ))
        })
    }

    pub(super) fn character_text(run: &HtmlTokenizerRunResult) -> String {
        run.tokens()
            .iter()
            .filter_map(|token| match token {
                HtmlToken::Character(character) => Some(character.interpreted()),
                _ => None,
            })
            .collect()
    }

    pub(super) fn missing_semicolon_count(run: &HtmlTokenizerRunResult) -> usize {
        run.diagnostics()
            .iter()
            .filter(|d| {
                d.code() == HtmlTokenizerDiagnosticCode::MissingSemicolonAfterCharacterReference
            })
            .count()
    }

    pub(super) fn diagnostic_count(run: &HtmlTokenizerRunResult) -> usize {
        run.diagnostics().len()
    }
}

// ---------------------------------------------------------------------------
// Layout: hand-authored fixture geometry computed only from authored pieces
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Layout {
    source: String,
    tag: (usize, usize),
    complete: (usize, usize),
    name: (usize, usize),
    equals: (usize, usize),
    open_quote: Option<(usize, usize)>,
    value: (usize, usize),
    close_quote: Option<(usize, usize)>,
}

/// `<body>` + `<element aname=VALUE tail>`; offsets come only from piece
/// lengths, never from model or production output.
fn build_layout(element: &str, aname: &str, syntax: Syntax, value: &str, tail: &str) -> Layout {
    let prefix = "<body>";
    let tag_start = prefix.len();
    let name_start = tag_start + 1 + element.len() + 1;
    let name_end = name_start + aname.len();
    let equals = (name_end, name_end + 1);
    let quoted = syntax != Syntax::Unquoted;
    let value_start = name_end + if quoted { 2 } else { 1 };
    let value_end = value_start + value.len();
    let open_quote = quoted.then_some((name_end + 1, name_end + 2));
    let close_quote = quoted.then_some((value_end, value_end + 1));
    let complete_end = value_end + usize::from(quoted);
    let tag_end = complete_end + tail.len() + 1;
    let quote = match syntax {
        Syntax::DoubleQuoted => "\"",
        Syntax::SingleQuoted => "'",
        Syntax::Unquoted => "",
    };
    Layout {
        source: format!("{prefix}<{element} {aname}={quote}{value}{quote}{tail}>"),
        tag: (tag_start, tag_end),
        complete: (name_start, complete_end),
        name: (name_start, name_end),
        equals,
        open_quote,
        value: (value_start, value_end),
        close_quote,
    }
}

// ---------------------------------------------------------------------------
// GOLD: the reference matrix. Value-relative offsets, hand counted.
// ---------------------------------------------------------------------------

type GoldDiags = &'static [(Kind, Site)];
type GoldEpisodes = &'static [(Res, usize, &'static str)];

struct Case {
    name: &'static str,
    /// Authored value text valid in all three syntaxes.
    value: &'static str,
    interpreted: &'static str,
    /// `Site::Unit` offsets are relative to the start of the authored value.
    diags: GoldDiags,
    /// Replaces `diags` for the unquoted syntax when an authored `=` is handled
    /// by the unquoted state itself.
    diags_unquoted: Option<GoldDiags>,
    /// `(resolution, authored bytes consumed, produced text)`.
    episodes: GoldEpisodes,
}

fn case(
    name: &'static str,
    value: &'static str,
    interpreted: &'static str,
    diags: GoldDiags,
    episodes: GoldEpisodes,
) -> Case {
    Case {
        name,
        value,
        interpreted,
        diags,
        diags_unquoted: None,
        episodes,
    }
}

const FFFD: &str = "\u{fffd}";

fn cases() -> Vec<Case> {
    let mut blocked_equals = case(
        "blocked-equals",
        "&amp=1",
        "&amp=1",
        &[],
        &[(Res::Blocked, 4, "&amp")],
    );
    blocked_equals.diags_unquoted = Some(&[(UNEXPECTED_UNQUOTED, Site::Unit(4))]);
    vec![
        case(
            "named-semicolon",
            "a&amp;b",
            "a&b",
            &[],
            &[(Res::Named, 5, "&")],
        ),
        case(
            "named-lt-gt",
            "&lt;x&gt;",
            "<x>",
            &[],
            &[(Res::Named, 4, "<"), (Res::Named, 4, ">")],
        ),
        case(
            "multi-scalar",
            "&NotEqualTilde;",
            "\u{2242}\u{0338}",
            &[],
            &[(Res::Named, 15, "\u{2242}\u{0338}")],
        ),
        case(
            "multi-scalar-yields-equals-sign",
            "p&bne;q",
            "p=\u{20e5}q",
            &[],
            &[(Res::Named, 5, "=\u{20e5}")],
        ),
        case(
            "max-match-notin",
            "&notin;",
            "\u{2209}",
            &[],
            &[(Res::Named, 7, "\u{2209}")],
        ),
        case(
            "case-sensitive-Not",
            "&Not;",
            "\u{2aec}",
            &[],
            &[(Res::Named, 5, "\u{2aec}")],
        ),
        case(
            "case-sensitive-not",
            "&not;",
            "\u{ac}",
            &[],
            &[(Res::Named, 5, "\u{ac}")],
        ),
        case(
            "max-match-then-blocked-notit",
            "&notit;",
            "&notit;",
            &[],
            &[(Res::Blocked, 4, "&not")],
        ),
        case(
            "max-match-then-blocked-notinx",
            "&notinx",
            "&notinx",
            &[],
            &[(Res::Blocked, 4, "&not")],
        ),
        case(
            "blocked-alphanumeric",
            "&ampx",
            "&ampx",
            &[],
            &[(Res::Blocked, 4, "&amp")],
        ),
        case(
            "blocked-digit",
            "&gt2",
            "&gt2",
            &[],
            &[(Res::Blocked, 3, "&gt")],
        ),
        blocked_equals,
        case(
            "semicolonless-at-delimiter",
            "x&amp",
            "x&",
            &[(MISSING, Site::Reference(0))],
            &[(Res::Named, 4, "&")],
        ),
        case(
            "semicolonless-before-dash",
            "&amp-z",
            "&-z",
            &[(MISSING, Site::Reference(0))],
            &[(Res::Named, 4, "&")],
        ),
        case(
            "semicolonless-not",
            "&not",
            "\u{ac}",
            &[(MISSING, Site::Reference(0))],
            &[(Res::Named, 4, "\u{ac}")],
        ),
        case(
            "semicolonless-copy-before-dot",
            "&copy.",
            "\u{a9}.",
            &[(MISSING, Site::Reference(0))],
            &[(Res::Named, 5, "\u{a9}")],
        ),
        case(
            "unknown-name-semicolon",
            "&zzz;",
            "&zzz;",
            &[(UNKNOWN, Site::Unit(4))],
            &[(Res::Ambiguous, 4, "&zzz")],
        ),
        case(
            "unknown-name-with-digit",
            "&zz9;",
            "&zz9;",
            &[(UNKNOWN, Site::Unit(4))],
            &[(Res::Ambiguous, 4, "&zz9")],
        ),
        case(
            "unknown-single-digit",
            "&9;",
            "&9;",
            &[(UNKNOWN, Site::Unit(2))],
            &[(Res::Ambiguous, 2, "&9")],
        ),
        case(
            "ambiguous-run-without-semicolon",
            "&zzz",
            "&zzz",
            &[],
            &[(Res::Ambiguous, 4, "&zzz")],
        ),
        case(
            "bare-ampersand-at-delimiter",
            "a&",
            "a&",
            &[],
            &[(Res::Bare, 1, "&")],
        ),
        case(
            "bare-ampersand-before-dash",
            "&-",
            "&-",
            &[],
            &[(Res::Bare, 1, "&")],
        ),
        case(
            "ampersand-semicolon",
            "&;",
            "&;",
            &[],
            &[(Res::Bare, 1, "&")],
        ),
        case(
            "ampersand-then-reference",
            "&&amp;",
            "&&",
            &[],
            &[(Res::Bare, 1, "&"), (Res::Named, 5, "&")],
        ),
        case(
            "two-named-references",
            "&amp;&amp;",
            "&&",
            &[],
            &[(Res::Named, 5, "&"), (Res::Named, 5, "&")],
        ),
        case(
            "decoded-whitespace",
            "&Tab;x&NewLine;",
            "\tx\n",
            &[],
            &[(Res::Named, 5, "\t"), (Res::Named, 9, "\n")],
        ),
        case(
            "nbsp",
            "&nbsp;",
            "\u{a0}",
            &[],
            &[(Res::Named, 6, "\u{a0}")],
        ),
        case(
            "composition-prefix-reference-suffix",
            "p&amp;q&lt;r&#65;s",
            "p&q<rAs",
            &[],
            &[
                (Res::Named, 5, "&"),
                (Res::Named, 4, "<"),
                (Res::NumDec, 5, "A"),
            ],
        ),
        case(
            "multibyte-offsets",
            "\u{e9}&zzz;",
            "\u{e9}&zzz;",
            &[(UNKNOWN, Site::Unit(6))],
            &[(Res::Ambiguous, 4, "&zzz")],
        ),
        case(
            "multibyte-around-named",
            "\u{e9}&amp;\u{fc}",
            "\u{e9}&\u{fc}",
            &[],
            &[(Res::Named, 5, "&")],
        ),
        case("decimal", "&#65;", "A", &[], &[(Res::NumDec, 5, "A")]),
        case("hexadecimal", "&#x41;", "A", &[], &[(Res::NumHex, 6, "A")]),
        case(
            "hexadecimal-uppercase-x",
            "&#X4a;",
            "J",
            &[],
            &[(Res::NumHex, 6, "J")],
        ),
        case(
            "decimal-without-semicolon-letter",
            "&#65b",
            "Ab",
            &[(MISSING, Site::Unit(4))],
            &[(Res::NumDec, 4, "A")],
        ),
        case(
            "hexadecimal-without-semicolon-nonhex",
            "&#x41g",
            "Ag",
            &[(MISSING, Site::Unit(5))],
            &[(Res::NumHex, 5, "A")],
        ),
        case(
            "hexadecimal-letter-is-a-digit",
            "&#x41b;",
            "\u{41b}",
            &[],
            &[(Res::NumHex, 7, "\u{41b}")],
        ),
        case(
            "decimal-without-semicolon-at-delimiter",
            "&#65",
            "A",
            &[(MISSING, Site::Unit(4))],
            &[(Res::NumDec, 4, "A")],
        ),
        case(
            "absence-of-digits-letter",
            "&#z",
            "&#z",
            &[(ABSENCE, Site::Unit(2))],
            &[(Res::Absent, 2, "&#")],
        ),
        case(
            "absence-of-digits-semicolon",
            "&#;",
            "&#;",
            &[(ABSENCE, Site::Unit(2))],
            &[(Res::Absent, 2, "&#")],
        ),
        case(
            "absence-of-hex-digits-semicolon",
            "&#x;",
            "&#x;",
            &[(ABSENCE, Site::Unit(3))],
            &[(Res::Absent, 3, "&#x")],
        ),
        case(
            "absence-of-hex-digits-uppercase-x",
            "&#Xg",
            "&#Xg",
            &[(ABSENCE, Site::Unit(3))],
            &[(Res::Absent, 3, "&#X")],
        ),
        case(
            "absence-of-digits-at-delimiter",
            "a&#",
            "a&#",
            &[(ABSENCE, Site::Unit(3))],
            &[(Res::Absent, 2, "&#")],
        ),
        case(
            "zero-decimal",
            "&#0;",
            FFFD,
            &[(NULL_REF, Site::Reference(0))],
            &[(Res::NumDec, 4, FFFD)],
        ),
        case(
            "zero-hexadecimal",
            "&#x0;",
            FFFD,
            &[(NULL_REF, Site::Reference(0))],
            &[(Res::NumHex, 5, FFFD)],
        ),
        case(
            "surrogate-hexadecimal",
            "&#xD800;",
            FFFD,
            &[(SURROGATE, Site::Reference(0))],
            &[(Res::NumHex, 8, FFFD)],
        ),
        case(
            "surrogate-decimal",
            "&#57343;",
            FFFD,
            &[(SURROGATE, Site::Reference(0))],
            &[(Res::NumDec, 8, FFFD)],
        ),
        case(
            "outside-range-hexadecimal",
            "&#x110000;",
            FFFD,
            &[(OUTSIDE_RANGE, Site::Reference(0))],
            &[(Res::NumHex, 10, FFFD)],
        ),
        case(
            "outside-range-decimal",
            "&#1114112;",
            FFFD,
            &[(OUTSIDE_RANGE, Site::Reference(0))],
            &[(Res::NumDec, 10, FFFD)],
        ),
        case(
            "wraparound-trap-decimal",
            "&#4294967361;",
            FFFD,
            &[(OUTSIDE_RANGE, Site::Reference(0))],
            &[(Res::NumDec, 13, FFFD)],
        ),
        case(
            "wraparound-trap-hexadecimal",
            "&#x100000041;",
            FFFD,
            &[(OUTSIDE_RANGE, Site::Reference(0))],
            &[(Res::NumHex, 13, FFFD)],
        ),
        case(
            "c1-remap-euro",
            "&#x80;",
            "\u{20ac}",
            &[(CONTROL_REF, Site::Reference(0))],
            &[(Res::NumHex, 6, "\u{20ac}")],
        ),
        case(
            "c1-remap-y-diaeresis",
            "&#159;",
            "\u{178}",
            &[(CONTROL_REF, Site::Reference(0))],
            &[(Res::NumDec, 6, "\u{178}")],
        ),
        case(
            "c1-without-override-row",
            "&#x81;",
            "\u{81}",
            &[(CONTROL_REF, Site::Reference(0))],
            &[(Res::NumHex, 6, "\u{81}")],
        ),
        case(
            "carriage-return-is-a-value",
            "&#x0D;",
            "\r",
            &[(CONTROL_REF, Site::Reference(0))],
            &[(Res::NumHex, 6, "\r")],
        ),
        case(
            "delete-control",
            "&#127;",
            "\u{7f}",
            &[(CONTROL_REF, Site::Reference(0))],
            &[(Res::NumDec, 6, "\u{7f}")],
        ),
        case(
            "c0-control",
            "&#1;",
            "\u{1}",
            &[(CONTROL_REF, Site::Reference(0))],
            &[(Res::NumDec, 4, "\u{1}")],
        ),
        case(
            "ascii-whitespace-has-no-diagnostic",
            "&#x9;&#10;&#x20;",
            "\t\n ",
            &[],
            &[
                (Res::NumHex, 5, "\t"),
                (Res::NumDec, 5, "\n"),
                (Res::NumHex, 6, " "),
            ],
        ),
        case(
            "noncharacter-fdd0",
            "&#xFDD0;",
            "\u{fdd0}",
            &[(NONCHARACTER, Site::Reference(0))],
            &[(Res::NumHex, 8, "\u{fdd0}")],
        ),
        case(
            "noncharacter-fffe",
            "&#xFFFE;",
            "\u{fffe}",
            &[(NONCHARACTER, Site::Reference(0))],
            &[(Res::NumHex, 8, "\u{fffe}")],
        ),
        case(
            "noncharacter-10ffff",
            "&#x10FFFF;",
            "\u{10ffff}",
            &[(NONCHARACTER, Site::Reference(0))],
            &[(Res::NumHex, 10, "\u{10ffff}")],
        ),
        case(
            "bmp-value",
            "&#x20AC;",
            "\u{20ac}",
            &[],
            &[(Res::NumHex, 8, "\u{20ac}")],
        ),
        case(
            "non-bmp-hexadecimal",
            "&#x1F600;",
            "\u{1f600}",
            &[],
            &[(Res::NumHex, 9, "\u{1f600}")],
        ),
        case(
            "non-bmp-decimal",
            "&#128512;",
            "\u{1f600}",
            &[],
            &[(Res::NumDec, 9, "\u{1f600}")],
        ),
        case(
            "leading-zeros",
            "&#x0000000041;",
            "A",
            &[],
            &[(Res::NumHex, 14, "A")],
        ),
        case(
            "two-numeric",
            "&#65;&#x42;",
            "AB",
            &[],
            &[(Res::NumDec, 5, "A"), (Res::NumHex, 6, "B")],
        ),
        case(
            "decoded-delimiter-like-scalars",
            "&#34;&#39;&#32;&#62;&#38;",
            "\"' >&",
            &[],
            &[
                (Res::NumDec, 5, "\""),
                (Res::NumDec, 5, "'"),
                (Res::NumDec, 5, " "),
                (Res::NumDec, 5, ">"),
                (Res::NumDec, 5, "&"),
            ],
        ),
        case(
            "decoded-ampersand-is-not-redecoded",
            "&#38;amp;",
            "&amp;",
            &[],
            &[(Res::NumDec, 5, "&")],
        ),
        case(
            "raw-nul",
            "a\0b",
            "a\u{fffd}b",
            &[(UNEXPECTED_NULL, Site::Unit(1))],
            &[],
        ),
        case(
            "raw-nul-after-bare-ampersand",
            "x&\0y",
            "x&\u{fffd}y",
            &[(UNEXPECTED_NULL, Site::Unit(2))],
            &[(Res::Bare, 1, "&")],
        ),
    ]
}

const ATTRIBUTE_NAMES: [&str; 5] = ["id", "class", "title", "data-x", "aria-label"];
const TAILS: [&str; 2] = ["", " "];

fn failure(failures: &mut Vec<String>, label: &str, message: impl std::fmt::Display) {
    failures.push(format!("{label}: {message}"));
}

fn shift_diags(gold: GoldDiags, value_start: usize) -> Vec<Diag> {
    gold.iter()
        .map(|&(kind, site)| Diag {
            kind,
            site: match site {
                Site::Unit(offset) => Site::Unit(value_start + offset),
                Site::Reference(ordinal) => Site::Reference(ordinal),
            },
        })
        .collect()
}

/// Checks one matrix cell against GOLD. Returns failures.
fn check_cell(probe: Probe, index: usize, case: &Case, syntax: Syntax, tail: &str) -> Vec<String> {
    let mut failures = Vec::new();
    let element = SELECTED[index % SELECTED.len()];
    let aname = ATTRIBUTE_NAMES[index % ATTRIBUTE_NAMES.len()];
    let source_id = 3 + (index % 7) as u64;
    let layout = build_layout(element, aname, syntax, case.value, tail);
    let label = format!("{} / {syntax:?} / tail {tail:?}", case.name);
    let lexed = lex_with(&layout.source, source_id, UNLIMITED, probe);
    if lexed.outcome != Outcome::Complete {
        failure(
            &mut failures,
            &label,
            format!("outcome {:?}", lexed.outcome),
        );
        return failures;
    }
    let tag = lexed.tag.as_ref().expect("complete tag");
    let id = SourceId::new(source_id);
    let ev = |range: (usize, usize), raw: &str| Evidence {
        source_id: id,
        start: range.0,
        end: range.1,
        raw: raw.to_owned(),
    };
    let quote = |syntax: Syntax| match syntax {
        Syntax::DoubleQuoted => "\"",
        Syntax::SingleQuoted => "'",
        Syntax::Unquoted => "",
    };
    let expected_tag = ev(layout.tag, &layout.source[layout.tag.0..layout.tag.1]);
    if tag.element != element || tag.tag != expected_tag {
        failure(&mut failures, &label, "tag evidence differs");
    }
    let attribute = tag.attribute.as_ref().expect("one attribute");
    let q = quote(syntax);
    let complete_raw = format!("{aname}={q}{}{q}", case.value);
    let expected_attribute = AttributeEvidence {
        complete: ev(layout.complete, &complete_raw),
        name: ev(layout.name, aname),
        interpreted_name: aname.to_owned(),
        syntax,
        equals: ev(layout.equals, "="),
        open_quote: layout.open_quote.map(|range| ev(range, q)),
        value: ev(layout.value, case.value),
        close_quote: layout.close_quote.map(|range| ev(range, q)),
        interpreted_value: case.interpreted.to_owned(),
    };
    if attribute != &expected_attribute {
        failure(
            &mut failures,
            &label,
            format!("attribute evidence {attribute:?} != {expected_attribute:?}"),
        );
    }
    let gold_diags = match (syntax, case.diags_unquoted) {
        (Syntax::Unquoted, Some(unquoted)) => unquoted,
        _ => case.diags,
    };
    let expected_diags = shift_diags(gold_diags, layout.value.0);
    if lexed.diags != expected_diags {
        failure(
            &mut failures,
            &label,
            format!("diagnostics {:?} != {expected_diags:?}", lexed.diags),
        );
    }
    let projected: Vec<(Res, usize, &str)> = lexed
        .episodes
        .iter()
        .map(|e| (e.res, e.consumed_end - e.entry, e.produced.as_str()))
        .collect();
    if projected != case.episodes {
        failure(
            &mut failures,
            &label,
            format!("episodes {projected:?} != {:?}", case.episodes),
        );
    }
    for episode in &lexed.episodes {
        if episode.return_to != Return::Attribute(syntax) {
            failure(&mut failures, &label, "wrong originating return state");
        }
        if episode.sink != Sink::AttributeValue {
            failure(&mut failures, &label, "reference output left the attribute");
        }
    }
    if !lexed.document_text.is_empty() {
        failure(&mut failures, &label, "document Character-token leakage");
    }
    let expected_retained = 4 + element.len() + aname.len() + case.interpreted.len();
    if lexed.retained != expected_retained {
        failure(
            &mut failures,
            &label,
            format!("retained {} != {expected_retained}", lexed.retained),
        );
    }
    failures
}

fn matrix_failures(probe: Probe) -> Vec<String> {
    let mut failures = Vec::new();
    for (index, case) in cases().iter().enumerate() {
        for syntax in SYNTAXES {
            for tail in TAILS {
                failures.extend(check_cell(probe, index, case, syntax, tail));
            }
        }
    }
    failures
}

// ---------------------------------------------------------------------------
// GOLD: explicit hand-counted fixtures (anchors, return state, delimiters)
// ---------------------------------------------------------------------------

struct Explicit {
    name: &'static str,
    source: &'static str,
    source_id: u64,
    syntax: Syntax,
    element: &'static str,
    interpreted_name: &'static str,
    tag: (usize, usize),
    complete: (usize, usize),
    name_range: (usize, usize),
    value: (usize, usize),
    interpreted: &'static str,
    /// `(entry, consumed_end, resolution)` as absolute offsets.
    episodes: &'static [(usize, usize, Res)],
    /// Absolute diagnostic sites.
    diags: &'static [(Kind, Site)],
}

fn explicit_fixtures() -> Vec<Explicit> {
    use Syntax::{DoubleQuoted as Dq, SingleQuoted as Sq, Unquoted as Uq};
    vec![
        Explicit {
            name: "double quoted named",
            source: "<body><div id=\"a&amp;b\">",
            source_id: 1,
            syntax: Dq,
            element: "div",
            interpreted_name: "id",
            tag: (6, 24),
            complete: (11, 23),
            name_range: (11, 13),
            value: (15, 22),
            interpreted: "a&b",
            episodes: &[(16, 21, Res::Named)],
            diags: &[],
        },
        Explicit {
            name: "single quoted numeric",
            source: "<body><nav class='&#x41;'>",
            source_id: 1,
            syntax: Sq,
            element: "nav",
            interpreted_name: "class",
            tag: (6, 26),
            complete: (11, 25),
            name_range: (11, 16),
            value: (18, 24),
            interpreted: "A",
            episodes: &[(18, 24, Res::NumHex)],
            diags: &[],
        },
        Explicit {
            name: "unquoted named",
            source: "<body><main data-x=a&lt;b>",
            source_id: 1,
            syntax: Uq,
            element: "main",
            interpreted_name: "data-x",
            tag: (6, 26),
            complete: (12, 25),
            name_range: (12, 18),
            value: (19, 25),
            interpreted: "a<b",
            episodes: &[(20, 24, Res::Named)],
            diags: &[],
        },
        Explicit {
            name: "multi-scalar output keeps authored value spelling",
            source: "<body><header title=\"&NotEqualTilde;\">",
            source_id: 1,
            syntax: Dq,
            element: "header",
            interpreted_name: "title",
            tag: (6, 38),
            complete: (14, 37),
            name_range: (14, 19),
            value: (21, 36),
            interpreted: "\u{2242}\u{0338}",
            episodes: &[(21, 36, Res::Named)],
            diags: &[],
        },
        Explicit {
            name: "different SourceId, ambiguous run, semicolon reconsumed",
            source: "<body><aside id='x&zzz;y'>",
            source_id: 9,
            syntax: Sq,
            element: "aside",
            interpreted_name: "id",
            tag: (6, 26),
            complete: (13, 25),
            name_range: (13, 15),
            value: (17, 24),
            interpreted: "x&zzz;y",
            episodes: &[(18, 22, Res::Ambiguous)],
            diags: &[(UNKNOWN, Site::Unit(22))],
        },
        Explicit {
            name: "double quoted: other quote is ordinary after a bare ampersand",
            source: "<body><footer id=\"x&'\">",
            source_id: 1,
            syntax: Dq,
            element: "footer",
            interpreted_name: "id",
            tag: (6, 23),
            complete: (14, 22),
            name_range: (14, 16),
            value: (18, 21),
            interpreted: "x&'",
            episodes: &[(19, 20, Res::Bare)],
            diags: &[],
        },
        Explicit {
            name: "single quoted: other quote is ordinary after a bare ampersand",
            source: "<body><footer id='x&\"'>",
            source_id: 1,
            syntax: Sq,
            element: "footer",
            interpreted_name: "id",
            tag: (6, 23),
            complete: (14, 22),
            name_range: (14, 16),
            value: (18, 21),
            interpreted: "x&\"",
            episodes: &[(19, 20, Res::Bare)],
            diags: &[],
        },
        Explicit {
            name: "unquoted: quote after a bare ampersand is handled by the unquoted state",
            source: "<body><footer id=x&\">",
            source_id: 1,
            syntax: Uq,
            element: "footer",
            interpreted_name: "id",
            tag: (6, 21),
            complete: (14, 20),
            name_range: (14, 16),
            value: (17, 20),
            interpreted: "x&\"",
            episodes: &[(18, 19, Res::Bare)],
            diags: &[(UNEXPECTED_UNQUOTED, Site::Unit(19))],
        },
        Explicit {
            name: "double quoted: space after a bare ampersand is value text",
            source: "<body><header id=\"x& y\">",
            source_id: 1,
            syntax: Dq,
            element: "header",
            interpreted_name: "id",
            tag: (6, 24),
            complete: (14, 23),
            name_range: (14, 16),
            value: (18, 22),
            interpreted: "x& y",
            episodes: &[(19, 20, Res::Bare)],
            diags: &[],
        },
        Explicit {
            name: "single quoted: space after a bare ampersand is value text",
            source: "<body><header id='x& y'>",
            source_id: 1,
            syntax: Sq,
            element: "header",
            interpreted_name: "id",
            tag: (6, 24),
            complete: (14, 23),
            name_range: (14, 16),
            value: (18, 22),
            interpreted: "x& y",
            episodes: &[(19, 20, Res::Bare)],
            diags: &[],
        },
        Explicit {
            name: "unquoted: space after a bare ampersand ends the value",
            source: "<body><header id=x& >",
            source_id: 1,
            syntax: Uq,
            element: "header",
            interpreted_name: "id",
            tag: (6, 21),
            complete: (14, 19),
            name_range: (14, 16),
            value: (17, 19),
            interpreted: "x&",
            episodes: &[(18, 19, Res::Bare)],
            diags: &[],
        },
        Explicit {
            name: "double quoted: greater-than after a bare ampersand is value text",
            source: "<body><main id=\"x&>\">",
            source_id: 1,
            syntax: Dq,
            element: "main",
            interpreted_name: "id",
            tag: (6, 21),
            complete: (12, 20),
            name_range: (12, 14),
            value: (16, 19),
            interpreted: "x&>",
            episodes: &[(17, 18, Res::Bare)],
            diags: &[],
        },
        Explicit {
            name: "unquoted: greater-than after a bare ampersand ends the tag",
            source: "<body><main id=x&>",
            source_id: 1,
            syntax: Uq,
            element: "main",
            interpreted_name: "id",
            tag: (6, 18),
            complete: (12, 17),
            name_range: (12, 14),
            value: (15, 17),
            interpreted: "x&",
            episodes: &[(16, 17, Res::Bare)],
            diags: &[],
        },
        Explicit {
            name: "double quoted: close quote is not swallowed by a semicolonless named",
            source: "<body><nav id=\"&amp\">",
            source_id: 1,
            syntax: Dq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 21),
            complete: (11, 20),
            name_range: (11, 13),
            value: (15, 19),
            interpreted: "&",
            episodes: &[(15, 19, Res::Named)],
            diags: &[(MISSING, Site::Reference(0))],
        },
        Explicit {
            name: "unquoted: tag end is not swallowed by a semicolonless named",
            source: "<body><nav id=&amp>",
            source_id: 1,
            syntax: Uq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 19),
            complete: (11, 18),
            name_range: (11, 13),
            value: (14, 18),
            interpreted: "&",
            episodes: &[(14, 18, Res::Named)],
            diags: &[(MISSING, Site::Reference(0))],
        },
        Explicit {
            name: "unquoted: blocked by equals sign, equals handled by the unquoted state",
            source: "<body><nav id=&amp=1>",
            source_id: 1,
            syntax: Uq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 21),
            complete: (11, 20),
            name_range: (11, 13),
            value: (14, 20),
            interpreted: "&amp=1",
            episodes: &[(14, 18, Res::Blocked)],
            diags: &[(UNEXPECTED_UNQUOTED, Site::Unit(18))],
        },
        Explicit {
            name: "double quoted: blocked by equals sign",
            source: "<body><nav id=\"&amp=1\">",
            source_id: 1,
            syntax: Dq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 23),
            complete: (11, 22),
            name_range: (11, 13),
            value: (15, 21),
            interpreted: "&amp=1",
            episodes: &[(15, 19, Res::Blocked)],
            diags: &[],
        },
        Explicit {
            name: "single quoted: blocked by alphanumeric",
            source: "<body><nav id='&ampx'>",
            source_id: 1,
            syntax: Sq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 22),
            complete: (11, 21),
            name_range: (11, 13),
            value: (15, 20),
            interpreted: "&ampx",
            episodes: &[(15, 19, Res::Blocked)],
            diags: &[],
        },
        Explicit {
            name: "unquoted: semicolonless decimal then ordinary letter",
            source: "<body><nav id=&#65b>",
            source_id: 1,
            syntax: Uq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 20),
            complete: (11, 19),
            name_range: (11, 13),
            value: (14, 19),
            interpreted: "Ab",
            episodes: &[(14, 18, Res::NumDec)],
            diags: &[(MISSING, Site::Unit(18))],
        },
        Explicit {
            name: "double quoted: absence of digits before the close quote",
            source: "<body><nav id=\"&#\">",
            source_id: 1,
            syntax: Dq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 19),
            complete: (11, 18),
            name_range: (11, 13),
            value: (15, 17),
            interpreted: "&#",
            episodes: &[(15, 17, Res::Absent)],
            diags: &[(ABSENCE, Site::Unit(17))],
        },
        Explicit {
            name: "unquoted: absence of digits before the tag end",
            source: "<body><nav id=&#>",
            source_id: 1,
            syntax: Uq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 17),
            complete: (11, 16),
            name_range: (11, 13),
            value: (14, 16),
            interpreted: "&#",
            episodes: &[(14, 16, Res::Absent)],
            diags: &[(ABSENCE, Site::Unit(16))],
        },
        Explicit {
            name: "unquoted: decoded delimiter-like scalars never end the value or tag",
            source: "<body><nav id=&#34;&#62;&#32;&#38;>",
            source_id: 1,
            syntax: Uq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 35),
            complete: (11, 34),
            name_range: (11, 13),
            value: (14, 34),
            interpreted: "\"> &",
            episodes: &[
                (14, 19, Res::NumDec),
                (19, 24, Res::NumDec),
                (24, 29, Res::NumDec),
                (29, 34, Res::NumDec),
            ],
            diags: &[],
        },
        Explicit {
            name: "single quoted: decoded apostrophe never closes the value",
            source: "<body><nav id='&#39;'>",
            source_id: 1,
            syntax: Sq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 22),
            complete: (11, 21),
            name_range: (11, 13),
            value: (15, 20),
            interpreted: "'",
            episodes: &[(15, 20, Res::NumDec)],
            diags: &[],
        },
        Explicit {
            name: "double quoted: decoded quote never closes the value",
            source: "<body><nav id=\"&#34;\">",
            source_id: 1,
            syntax: Dq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 22),
            complete: (11, 21),
            name_range: (11, 13),
            value: (15, 20),
            interpreted: "\"",
            episodes: &[(15, 20, Res::NumDec)],
            diags: &[],
        },
        Explicit {
            name: "raw U+0000 stays authored while interpreted is U+FFFD",
            source: "<body><nav id='a\0&amp;'>",
            source_id: 1,
            syntax: Sq,
            element: "nav",
            interpreted_name: "id",
            tag: (6, 24),
            complete: (11, 23),
            name_range: (11, 13),
            value: (15, 22),
            interpreted: "a\u{fffd}&",
            episodes: &[(17, 22, Res::Named)],
            diags: &[(UNEXPECTED_NULL, Site::Unit(16))],
        },
        Explicit {
            name: "mixed-case authored names keep authored spelling",
            source: "<body><DiV Title='&lt;'>",
            source_id: 1,
            syntax: Sq,
            element: "div",
            interpreted_name: "title",
            tag: (6, 24),
            complete: (11, 23),
            name_range: (11, 16),
            value: (18, 22),
            interpreted: "<",
            episodes: &[(18, 22, Res::Named)],
            diags: &[],
        },
    ]
}

fn explicit_failures(probe: Probe) -> Vec<String> {
    let mut failures = Vec::new();
    for fixture in explicit_fixtures() {
        let label = fixture.name;
        let lexed = lex_with(fixture.source, fixture.source_id, UNLIMITED, probe);
        if lexed.outcome != Outcome::Complete {
            failure(&mut failures, label, format!("outcome {:?}", lexed.outcome));
            continue;
        }
        let tag = lexed.tag.as_ref().expect("complete tag");
        let id = SourceId::new(fixture.source_id);
        let raw = |range: (usize, usize)| fixture.source[range.0..range.1].to_owned();
        let ev = |range: (usize, usize)| Evidence {
            source_id: id,
            start: range.0,
            end: range.1,
            raw: raw(range),
        };
        if tag.element != fixture.element || tag.tag != ev(fixture.tag) {
            failure(&mut failures, label, "tag evidence differs");
        }
        let attribute = tag.attribute.as_ref().expect("one attribute");
        if attribute.complete != ev(fixture.complete)
            || attribute.name != ev(fixture.name_range)
            || attribute.value != ev(fixture.value)
            || attribute.syntax != fixture.syntax
            || attribute.interpreted_name != fixture.interpreted_name
            || attribute.interpreted_value != fixture.interpreted
        {
            failure(
                &mut failures,
                label,
                format!("attribute evidence {attribute:?}"),
            );
        }
        let episodes: Vec<(usize, usize, Res)> = lexed
            .episodes
            .iter()
            .map(|e| (e.entry, e.consumed_end, e.res))
            .collect();
        if episodes != fixture.episodes {
            failure(&mut failures, label, format!("episodes {episodes:?}"));
        }
        let diags: Vec<(Kind, Site)> = lexed.diags.iter().map(|d| (d.kind, d.site)).collect();
        if diags != fixture.diags {
            failure(&mut failures, label, format!("diagnostics {diags:?}"));
        }
        if !lexed.document_text.is_empty() {
            failure(&mut failures, label, "document Character-token leakage");
        }
        // Every episode returns to the originating state; the unit at its
        // consumed end is handled by that state, never swallowed.
        for episode in &lexed.episodes {
            if episode.return_to != Return::Attribute(fixture.syntax) {
                failure(&mut failures, label, "wrong originating return state");
            }
        }
    }
    failures
}

// ---------------------------------------------------------------------------
// GOLD: Data and RCDATA controls (accepted semantics, unchanged)
// ---------------------------------------------------------------------------

struct TextCase {
    name: &'static str,
    text: &'static str,
    interpreted: &'static str,
    diags: GoldDiags,
    episodes: GoldEpisodes,
}

fn text_cases() -> Vec<TextCase> {
    vec![
        TextCase {
            name: "named with semicolon",
            text: "a&amp;b",
            interpreted: "a&b",
            diags: &[],
            episodes: &[(Res::Named, 5, "&")],
        },
        TextCase {
            name: "semicolonless not resolves where an attribute would block it",
            text: "&notit;",
            interpreted: "\u{ac}it;",
            diags: &[(MISSING, Site::Reference(0))],
            episodes: &[(Res::Named, 4, "\u{ac}")],
        },
        TextCase {
            name: "semicolonless amp before equals resolves",
            text: "&amp=1",
            interpreted: "&=1",
            diags: &[(MISSING, Site::Reference(0))],
            episodes: &[(Res::Named, 4, "&")],
        },
        TextCase {
            name: "semicolonless amp before letter resolves",
            text: "&ampx",
            interpreted: "&x",
            diags: &[(MISSING, Site::Reference(0))],
            episodes: &[(Res::Named, 4, "&")],
        },
        TextCase {
            name: "unknown name",
            text: "&zzz;",
            interpreted: "&zzz;",
            diags: &[(UNKNOWN, Site::Unit(4))],
            episodes: &[(Res::Ambiguous, 4, "&zzz")],
        },
        TextCase {
            name: "ampersand semicolon",
            text: "&;",
            interpreted: "&;",
            diags: &[],
            episodes: &[(Res::Bare, 1, "&")],
        },
        TextCase {
            name: "numeric with c1 remap",
            text: "&#65;&#x80;",
            interpreted: "A\u{20ac}",
            diags: &[(CONTROL_REF, Site::Reference(1))],
            episodes: &[(Res::NumDec, 5, "A"), (Res::NumHex, 6, "\u{20ac}")],
        },
        TextCase {
            name: "numeric without semicolon at end of input",
            text: "&#65",
            interpreted: "A",
            diags: &[(MISSING, Site::Unit(4))],
            episodes: &[(Res::NumDec, 4, "A")],
        },
        TextCase {
            name: "bare ampersand at end of input",
            text: "a&",
            interpreted: "a&",
            diags: &[],
            episodes: &[(Res::Bare, 1, "&")],
        },
        TextCase {
            name: "semicolonless named at end of input",
            text: "&amp",
            interpreted: "&",
            diags: &[(MISSING, Site::Reference(0))],
            episodes: &[(Res::Named, 4, "&")],
        },
        TextCase {
            name: "multi-scalar named",
            text: "&NotEqualTilde;",
            interpreted: "\u{2242}\u{0338}",
            diags: &[],
            episodes: &[(Res::Named, 15, "\u{2242}\u{0338}")],
        },
    ]
}

fn data_control_failures(probe: Probe) -> Vec<String> {
    let mut failures = Vec::new();
    for ret in [Return::Data, Return::Rcdata] {
        for case in text_cases() {
            let label = format!("{} / {ret:?}", case.name);
            let lexed = lex_text(case.text, ret, UNLIMITED, probe);
            if lexed.outcome != Outcome::Complete {
                failure(
                    &mut failures,
                    &label,
                    format!("outcome {:?}", lexed.outcome),
                );
                continue;
            }
            if lexed.document_text != case.interpreted {
                failure(
                    &mut failures,
                    &label,
                    format!("text {:?} != {:?}", lexed.document_text, case.interpreted),
                );
            }
            let diags: Vec<(Kind, Site)> = lexed.diags.iter().map(|d| (d.kind, d.site)).collect();
            if diags != case.diags {
                failure(&mut failures, &label, format!("diagnostics {diags:?}"));
            }
            let projected: Vec<(Res, usize, &str)> = lexed
                .episodes
                .iter()
                .map(|e| (e.res, e.consumed_end - e.entry, e.produced.as_str()))
                .collect();
            if projected != case.episodes {
                failure(&mut failures, &label, format!("episodes {projected:?}"));
            }
            if lexed
                .episodes
                .iter()
                .any(|e| e.sink != Sink::CharacterToken || e.return_to != ret)
            {
                failure(&mut failures, &label, "Data/RCDATA routing changed");
            }
            if !lexed.value.is_empty() {
                failure(&mut failures, &label, "attribute builder used in Data");
            }
        }
    }
    failures
}

// ---------------------------------------------------------------------------
// GOLD: resource and preflight theorem
// ---------------------------------------------------------------------------

fn retained(limit: usize) -> Limits {
    Limits {
        retained: Some(limit),
        diagnostics: None,
    }
}

fn diagnostics_limit(limit: usize) -> Limits {
    Limits {
        retained: None,
        diagnostics: Some(limit),
    }
}

/// `(source, retained limit, expected attempted, expected refusal offset,
/// expected value kept, expected committed episodes)`.
type RetainedRefusal = (&'static str, usize, usize, usize, &'static str, usize);

const RETAINED_REFUSALS: &[RetainedRefusal] = &[
    // Base is 4 (`body`) + 3 (`div`) + 2 (`id`) = 9.
    ("<body><div id=\"&amp;\">", 9, 10, 16, "", 0),
    ("<body><div id=\"&NotEqualTilde;\">", 13, 14, 16, "", 0),
    ("<body><div id=\"&NotEqualTilde;\">", 11, 14, 16, "", 0),
    ("<body><div id=\"a&amp;&amp;\">", 11, 12, 22, "a&", 1),
    ("<body><div id=\"&#x1F600;\">", 12, 13, 23, "", 0),
    ("<body><div id=\"&#x1F600\">", 12, 13, 23, "", 0),
    ("<body><div id=\"&#\">", 10, 11, 17, "", 0),
    ("<body><div id=\"ab\">", 10, 11, 16, "a", 0),
    ("<body><div id=\"&notit;\">", 12, 13, 16, "", 0),
];

fn resource_failures(probe: Probe) -> Vec<String> {
    let mut failures = Vec::new();

    for &(source, limit, attempted, at, kept, committed) in RETAINED_REFUSALS {
        let label = format!("{source:?} @ retained {limit}");
        let lexed = lex_with(source, 1, retained(limit), probe);
        let expected = Outcome::Resource(Refusal {
            resource: Resource::RetainedInterpretedBytes,
            limit,
            attempted,
            at,
        });
        if lexed.outcome != expected {
            failure(
                &mut failures,
                &label,
                format!("outcome {:?}", lexed.outcome),
            );
        }
        if lexed.value != kept || lexed.episodes.len() != committed || lexed.tag.is_some() {
            failure(&mut failures, &label, "partial state escaped a refusal");
        }
        if lexed.coverage_end != at {
            failure(
                &mut failures,
                &label,
                format!(
                    "coverage {} reached past the refusal site {at}",
                    lexed.coverage_end
                ),
            );
        }
    }

    // The same sources complete exactly at their final retained size.
    for (source, limit) in [
        ("<body><div id=\"&amp;\">", 10),
        ("<body><div id=\"&NotEqualTilde;\">", 14),
        ("<body><div id=\"a&amp;&amp;\">", 12),
        ("<body><div id=\"&#x1F600;\">", 13),
        ("<body><div id=\"&#\">", 11),
    ] {
        let lexed = lex_with(source, 1, retained(limit), probe);
        if lexed.outcome != Outcome::Complete {
            failure(
                &mut failures,
                &format!("{source:?} @ retained {limit}"),
                format!("final size refused: {:?}", lexed.outcome),
            );
        }
    }

    // Diagnostics are prepared together with the output they explain.
    for source in ["<body><div id=\"&amp\">", "<body><div id=\"&#65\">"] {
        let lexed = lex_with(source, 1, diagnostics_limit(0), probe);
        let expected = Outcome::Resource(Refusal {
            resource: Resource::Diagnostics,
            limit: 0,
            attempted: 1,
            at: 16,
        });
        let label = format!("{source:?} @ diagnostics 0");
        if source.contains("amp") && lexed.outcome != expected {
            failure(
                &mut failures,
                &label,
                format!("outcome {:?}", lexed.outcome),
            );
        }
        if !lexed.value.is_empty() || !lexed.diags.is_empty() {
            failure(&mut failures, &label, "output without its diagnostic");
        }
    }

    // Sweep every retained and diagnostics limit below the final size for the
    // whole reference matrix, in every syntax.
    for (index, case) in cases().iter().enumerate() {
        for syntax in SYNTAXES {
            let element = SELECTED[index % SELECTED.len()];
            let aname = ATTRIBUTE_NAMES[index % ATTRIBUTE_NAMES.len()];
            let source = build_layout(element, aname, syntax, case.value, "").source;
            let label = format!("{} / {syntax:?}", case.name);
            let full = lex_with(&source, 1, UNLIMITED, Probe::None);
            for limit in full.base..full.retained {
                let lexed = lex_with(&source, 1, retained(limit), probe);
                match lexed.outcome {
                    Outcome::Resource(refusal)
                        if refusal.resource == Resource::RetainedInterpretedBytes
                            && refusal.limit == limit
                            && refusal.attempted > limit
                            && refusal.attempted <= full.retained =>
                    {
                        if lexed.coverage_end != refusal.at {
                            failure(
                                &mut failures,
                                &label,
                                format!("limit {limit}: consumed past the refusal site"),
                            );
                        }
                        if !full.value.starts_with(&lexed.value)
                            || !lexed.boundaries.contains(&lexed.value.len())
                        {
                            failure(
                                &mut failures,
                                &label,
                                format!(
                                    "limit {limit}: value {:?} is not on a commit boundary",
                                    lexed.value
                                ),
                            );
                        }
                    }
                    other => failure(
                        &mut failures,
                        &label,
                        format!("limit {limit}: expected a retained refusal, got {other:?}"),
                    ),
                }
                if lexed.tag.is_some() || lexed.retained > limit {
                    failure(
                        &mut failures,
                        &label,
                        format!("limit {limit}: state escaped"),
                    );
                }
            }
            let at_final = lex_with(&source, 1, retained(full.retained), probe);
            if at_final.outcome != Outcome::Complete || at_final.value != full.value {
                failure(&mut failures, &label, "final retained size was refused");
            }
            for limit in 0..full.diags.len() {
                let lexed = lex_with(&source, 1, diagnostics_limit(limit), probe);
                match lexed.outcome {
                    Outcome::Resource(refusal)
                        if refusal.resource == Resource::Diagnostics
                            && refusal.limit == limit
                            && refusal.attempted > limit =>
                    {
                        if lexed.coverage_end != refusal.at || lexed.diags.len() > limit {
                            failure(
                                &mut failures,
                                &label,
                                format!("diagnostics limit {limit}: partial episode"),
                            );
                        }
                    }
                    other => failure(
                        &mut failures,
                        &label,
                        format!("diagnostics limit {limit}: got {other:?}"),
                    ),
                }
            }
        }
    }
    failures
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_clean(group: &str, failures: Vec<String>) {
        assert!(
            failures.is_empty(),
            "{group}: {} failure(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    #[test]
    fn pins_and_authority_markers_are_well_formed() {
        for pin in [WHATWG_PROMPT_TIME, WPT_PROMPT_TIME, HTML5LIB_PROMPT_TIME] {
            assert_eq!(pin.len(), 40);
            assert!(pin.bytes().all(|b| b.is_ascii_hexdigit()));
        }
        assert_eq!(SELECTION_AUTHORITY_COMMENT, "6008703400");
        assert_eq!(SELECTED.len(), 8);
        assert_eq!(ATTRIBUTES_PER_TAG, 1);
    }

    // -- the reference matrix ---------------------------------------------

    #[test]
    fn reference_matrix_matches_hand_authored_gold() {
        assert_clean("matrix", matrix_failures(Probe::None));
    }

    #[test]
    fn matrix_covers_every_selected_element_and_attribute_name() {
        let total = cases().len();
        assert!(total >= SELECTED.len().max(ATTRIBUTE_NAMES.len()));
        let elements: std::collections::BTreeSet<&str> =
            (0..total).map(|i| SELECTED[i % SELECTED.len()]).collect();
        assert_eq!(elements.len(), SELECTED.len());
    }

    #[test]
    fn every_selected_element_accepts_a_reference_valued_attribute() {
        for element in SELECTED {
            for syntax in SYNTAXES {
                let layout = build_layout(element, "id", syntax, "a&amp;b&#x41;", "");
                let lexed = lex(&layout.source);
                assert_eq!(lexed.outcome, Outcome::Complete, "{element} {syntax:?}");
                let tag = lexed.tag.as_ref().expect("tag");
                assert_eq!(tag.element, element);
                assert_eq!(
                    tag.attribute.as_ref().expect("attribute").interpreted_value,
                    "a&bA"
                );
            }
        }
    }

    #[test]
    fn the_matrix_exercises_every_resolution_and_diagnostic_kind() {
        let mut resolutions = Vec::new();
        let mut kinds = Vec::new();
        for case in cases() {
            for &(res, _, _) in case.episodes {
                if !resolutions.contains(&res) {
                    resolutions.push(res);
                }
            }
            for &(kind, _) in case.diags {
                if !kinds.contains(&kind) {
                    kinds.push(kind);
                }
            }
        }
        for res in [
            Res::Bare,
            Res::Named,
            Res::Blocked,
            Res::Ambiguous,
            Res::NumDec,
            Res::NumHex,
            Res::Absent,
        ] {
            assert!(resolutions.contains(&res), "{res:?}");
        }
        for kind in [
            MISSING,
            UNKNOWN,
            ABSENCE,
            NULL_REF,
            OUTSIDE_RANGE,
            SURROGATE,
            NONCHARACTER,
            CONTROL_REF,
            UNEXPECTED_NULL,
        ] {
            assert!(kinds.contains(&kind), "{kind:?}");
        }
    }

    // -- explicit anchors, return state and delimiters --------------------

    #[test]
    fn explicit_fixtures_match_hand_counted_anchors_and_delimiters() {
        assert_clean("explicit", explicit_failures(Probe::None));
    }

    #[test]
    fn every_originating_state_is_exercised_by_the_explicit_gold() {
        for syntax in SYNTAXES {
            assert!(
                explicit_fixtures().iter().any(|f| f.syntax == syntax),
                "{syntax:?}"
            );
        }
    }

    #[test]
    fn the_unquoted_return_state_differs_from_the_quoted_ones() {
        // The same authored `x& y` has three different meanings because the
        // return state, not the reference, decides the delimiter.
        let double = lex("<body><div id=\"x& y\">");
        let single = lex("<body><div id='x& y'>");
        assert_eq!(double.outcome, Outcome::Complete);
        assert_eq!(single.outcome, Outcome::Complete);
        assert_eq!(
            double.tag.unwrap().attribute.unwrap().interpreted_value,
            "x& y"
        );
        assert_eq!(
            single.tag.unwrap().attribute.unwrap().interpreted_value,
            "x& y"
        );
        // Unquoted: the space ends the value, and `y` starts a second
        // attribute, which is the accepted Product resource fact.
        let unquoted = lex("<body><div id=x& y>");
        assert_eq!(
            unquoted.outcome,
            Outcome::Resource(Refusal {
                resource: Resource::AttributesPerTag,
                limit: 1,
                attempted: 2,
                at: 17,
            })
        );
        assert_eq!(unquoted.value, "x&");
    }

    // -- output routing -----------------------------------------------------

    #[test]
    fn attribute_output_is_never_a_document_character_token() {
        for syntax in SYNTAXES {
            let layout = build_layout("div", "id", syntax, "a&amp;b&#65;&NotEqualTilde;", "");
            let lexed = lex(&layout.source);
            assert_eq!(lexed.outcome, Outcome::Complete);
            assert_eq!(lexed.document_text, "");
            assert!(
                lexed
                    .episodes
                    .iter()
                    .all(|e| e.sink == Sink::AttributeValue)
            );
        }
        // The same references in Data are Character-token text, and the
        // attribute builder is untouched.
        let data = lex_text("a&amp;b&#65;", Return::Data, UNLIMITED, Probe::None);
        assert_eq!(data.document_text, "a&bA");
        assert_eq!(data.value, "");
        assert!(data.episodes.iter().all(|e| e.sink == Sink::CharacterToken));
    }

    #[test]
    fn a_character_token_routing_design_is_observably_wrong() {
        let layout = build_layout("div", "id", Syntax::DoubleQuoted, "a&amp;b", "");
        let wrong = lex_with(&layout.source, 1, UNLIMITED, Probe::EmitAsCharacterToken);
        assert_eq!(wrong.document_text, "&");
        assert_eq!(wrong.value, "ab");
        let right = lex(&layout.source);
        assert_eq!(right.document_text, "");
        assert_eq!(right.value, "a&b");
    }

    // -- the attribute-only named exception ---------------------------------

    #[test]
    fn attribute_only_exception_blocks_alphanumeric_and_equals_followers() {
        for (value, interpreted) in [
            ("&ampx", "&ampx"),
            ("&amp9", "&amp9"),
            ("&amp=", "&amp="),
            ("&notit;", "&notit;"),
            ("&ltZ", "&ltZ"),
        ] {
            for syntax in SYNTAXES {
                let layout = build_layout("main", "id", syntax, value, "");
                let lexed = lex(&layout.source);
                let attribute = lexed.tag.expect("tag").attribute.expect("attribute");
                assert_eq!(
                    attribute.interpreted_value, interpreted,
                    "{value:?} {syntax:?}"
                );
                // Blocked output is literal and raises no reference diagnostic.
                assert!(
                    lexed.diags.iter().all(|d| d.kind == UNEXPECTED_UNQUOTED),
                    "{value:?} {syntax:?}"
                );
            }
        }
    }

    #[test]
    fn the_same_named_data_resolves_when_the_blocking_condition_is_absent() {
        for (value, interpreted) in [
            ("&amp.", "&."),
            ("&amp-", "&-"),
            ("&amp", "&"),
            ("&lt", "<"),
            ("&amp;x", "&x"),
        ] {
            for syntax in SYNTAXES {
                let layout = build_layout("nav", "id", syntax, value, "");
                let lexed = lex(&layout.source);
                let attribute = lexed.tag.expect("tag").attribute.expect("attribute");
                assert_eq!(
                    attribute.interpreted_value, interpreted,
                    "{value:?} {syntax:?}"
                );
            }
        }
    }

    #[test]
    fn the_exception_applies_only_to_semicolonless_matches() {
        // A semicolon-terminated match is never blocked, whatever follows.
        for value in ["&amp;x", "&amp;=", "&lt;9"] {
            let layout = build_layout("div", "id", Syntax::DoubleQuoted, value, "");
            let lexed = lex(&layout.source);
            assert_eq!(lexed.episodes[0].res, Res::Named, "{value}");
            assert!(lexed.diags.is_empty(), "{value}");
        }
    }

    #[test]
    fn maximum_match_precedes_the_exception_decision() {
        // `notin;` is the maximum match and is semicolon terminated, so the
        // exception never sees it. `notit;` and `notinx` fall back to `not`,
        // which is then blocked by the following alphanumeric.
        let longest = lex("<body><div id=\"&notin;\">");
        assert_eq!(longest.episodes[0].res, Res::Named);
        assert_eq!(longest.value, "\u{2209}");
        for source in ["<body><div id=\"&notit;\">", "<body><div id=\"&notinx\">"] {
            let lexed = lex(source);
            assert_eq!(lexed.episodes[0].res, Res::Blocked, "{source}");
            assert_eq!(lexed.episodes[0].produced, "&not", "{source}");
        }
    }

    // -- ambiguous ampersand ------------------------------------------------

    #[test]
    fn ampersand_semicolon_is_a_bare_ampersand_not_an_unknown_name() {
        for syntax in SYNTAXES {
            let layout = build_layout("div", "id", syntax, "&;", "");
            let lexed = lex(&layout.source);
            assert_eq!(lexed.value, "&;");
            assert!(lexed.diags.is_empty());
            assert_eq!(lexed.episodes[0].res, Res::Bare);
        }
        let unknown = lex("<body><div id=\"&zz;\">");
        assert_eq!(unknown.diags.len(), 1);
        assert_eq!(unknown.diags[0].kind, UNKNOWN);
    }

    #[test]
    fn unknown_semicolon_is_reconsumed_by_the_return_state() {
        // `;` is ordinary value text afterwards, in all three states.
        for syntax in SYNTAXES {
            let layout = build_layout("div", "id", syntax, "&zz;;q", "");
            let lexed = lex(&layout.source);
            assert_eq!(lexed.value, "&zz;;q");
            assert_eq!(lexed.diags.len(), 1);
        }
    }

    // -- numeric references --------------------------------------------------

    #[test]
    fn numeric_overflow_is_saturating_not_wrapping() {
        let long_zeros = format!("&#{}65;", "0".repeat(10_000));
        let layout = build_layout("div", "id", Syntax::DoubleQuoted, &long_zeros, "");
        let lexed = lex(&layout.source);
        assert_eq!(lexed.outcome, Outcome::Complete);
        assert_eq!(lexed.value, "A");
        assert!(lexed.diags.is_empty());
        // Seven short digits overflow while thousands of zeros do not.
        let overflow = lex("<body><div id=\"&#x1100000;\">");
        assert_eq!(overflow.value, FFFD);
        assert_eq!(overflow.diags[0].kind, OUTSIDE_RANGE);
        let huge = format!("&#x{};", "F".repeat(40));
        let layout = build_layout("div", "id", Syntax::SingleQuoted, &huge, "");
        let lexed = lex(&layout.source);
        assert_eq!(lexed.value, FFFD);
        assert_eq!(lexed.diags.len(), 1);
    }

    #[test]
    fn numeric_diagnostic_order_puts_missing_semicolon_before_end_diagnostics() {
        // `&#x80` + delimiter: the missing semicolon is raised in the digit
        // state, then Numeric End raises the control diagnostic.
        for syntax in SYNTAXES {
            let layout = build_layout("div", "id", syntax, "&#x80", "");
            let lexed = lex(&layout.source);
            let kinds: Vec<Kind> = lexed.diags.iter().map(|d| d.kind).collect();
            assert_eq!(kinds, [MISSING, CONTROL_REF], "{syntax:?}");
            assert_eq!(lexed.value, "\u{20ac}");
        }
        let layout = build_layout("div", "id", Syntax::DoubleQuoted, "&#0", "");
        let kinds: Vec<Kind> = lex(&layout.source).diags.iter().map(|d| d.kind).collect();
        assert_eq!(kinds, [MISSING, NULL_REF]);
    }

    #[test]
    fn numeric_end_has_at_most_one_end_diagnostic_over_a_code_point_sweep() {
        // Hand-authored invariant: the sequential bullets never produce two
        // End diagnostics, and a diagnostic never coexists with a different
        // class. Swept exhaustively over the interesting ranges.
        let ranges = [
            0u32..=0x9f,
            0xD7F0..=0xE010,
            0xFDC0..=0xFDF0,
            0xFFF0..=0x1_0001,
            0x10_FFF0..=0x11_0010,
        ];
        for range in ranges {
            for code in range {
                let (scalar, kinds) = numeric_end(code, Probe::None);
                assert!(kinds.len() <= 1, "{code:#x}: {kinds:?}");
                if kinds.is_empty() {
                    assert_eq!(scalar as u32, code, "{code:#x}");
                }
            }
        }
    }

    // -- delimiter return / reconsume --------------------------------------

    #[test]
    fn failed_or_literal_references_never_swallow_the_delimiter() {
        for (value, interpreted) in [
            ("a&", "a&"),
            ("&zz", "&zz"),
            ("&#", "&#"),
            ("&#x", "&#x"),
            ("&amp", "&"),
            ("&#65", "A"),
        ] {
            for syntax in SYNTAXES {
                for tail in TAILS {
                    let layout = build_layout("footer", "id", syntax, value, tail);
                    let lexed = lex(&layout.source);
                    assert_eq!(lexed.outcome, Outcome::Complete, "{value:?} {syntax:?}");
                    let tag = lexed.tag.expect("tag");
                    let attribute = tag.attribute.expect("attribute");
                    assert_eq!(attribute.interpreted_value, interpreted);
                    // The terminator (close quote or tag end) is still exactly
                    // where the layout puts it.
                    assert_eq!(attribute.value.end, layout.value.1);
                    assert_eq!(tag.tag.end, layout.tag.1);
                }
            }
        }
    }

    // -- authored evidence and composition ---------------------------------

    #[test]
    fn reference_decoding_changes_only_the_interpreted_value() {
        for syntax in SYNTAXES {
            let with_reference = build_layout("aside", "title", syntax, "a&amp;b", "");
            let reference_free = build_layout("aside", "title", syntax, "a12345b", "");
            assert_eq!(with_reference.value, reference_free.value);
            let a = lex_with(&with_reference.source, 5, UNLIMITED, Probe::None);
            let b = lex_with(&reference_free.source, 5, UNLIMITED, Probe::None);
            let (a, b) = (a.tag.expect("a"), b.tag.expect("b"));
            let (aa, bb) = (
                a.attribute.clone().expect("a attribute"),
                b.attribute.clone().expect("b attribute"),
            );
            // Authored anchors are identical; only spelling and interpreted
            // value differ, and the model identity payload is unchanged.
            for (x, y) in [
                (&aa.complete, &bb.complete),
                (&aa.name, &bb.name),
                (&aa.equals, &bb.equals),
                (&aa.value, &bb.value),
            ] {
                assert_eq!((x.source_id, x.start, x.end), (y.source_id, y.start, y.end));
            }
            assert_eq!(aa.syntax, bb.syntax);
            assert_eq!(aa.interpreted_name, bb.interpreted_name);
            assert_eq!(aa.value.raw, "a&amp;b");
            assert_eq!(aa.interpreted_value, "a&b");
            assert_eq!(bb.interpreted_value, "a12345b");
            let (na, nb) = (associate(4, &a), associate(4, &b));
            assert_eq!((na.id, &na.element), (nb.id, &nb.element));
            assert_eq!(a.tag.start, b.tag.start);
            assert_eq!(a.tag.end, b.tag.end);
        }
    }

    #[test]
    fn same_name_nesting_needs_no_new_identity_theorem() {
        // Two selected tags with equal names and equal attributes are told
        // apart only by model identity assigned at construction, exactly as
        // #901 already proves; reference interpretation adds nothing to it.
        let first = lex("<body><div id=\"&amp;\">");
        let second = lex("<body><div id=\"&amp;\">");
        let (a, b) = (first.tag.unwrap(), second.tag.unwrap());
        assert_eq!(a, b);
        assert_ne!(associate(4, &a), associate(5, &b));
        assert_eq!(associate(4, &a).attribute, associate(5, &b).attribute);
    }

    #[test]
    fn source_identity_is_carried_by_every_anchor() {
        let layout = build_layout("header", "id", Syntax::SingleQuoted, "&lt;&#65;", "");
        let lexed = lex_with(&layout.source, 77, UNLIMITED, Probe::None);
        let attribute = lexed.tag.as_ref().unwrap().attribute.as_ref().unwrap();
        for evidence in [
            &lexed.tag.as_ref().unwrap().tag,
            &attribute.complete,
            &attribute.name,
            &attribute.equals,
            &attribute.value,
        ] {
            assert_eq!(evidence.source_id, SourceId::new(77));
        }
        assert_eq!(
            attribute.open_quote.as_ref().unwrap().source_id,
            SourceId::new(77)
        );
    }

    // -- resource and preflight --------------------------------------------

    #[test]
    fn resource_preflight_precedes_irreversible_reference_consumption() {
        assert_clean("resource", resource_failures(Probe::None));
    }

    #[test]
    fn retained_cost_is_the_decoded_utf8_length() {
        // `&NotEqualTilde;` is 15 authored bytes and 2 scalars but 5 decoded
        // UTF-8 bytes. Only the decoded length fits the limit exactly.
        let exact = lex_with(
            "<body><div id=\"&NotEqualTilde;\">",
            1,
            retained(14),
            Probe::None,
        );
        assert_eq!(exact.outcome, Outcome::Complete);
        assert_eq!(exact.retained, 14);
        let short = lex_with(
            "<body><div id=\"&NotEqualTilde;\">",
            1,
            retained(13),
            Probe::None,
        );
        assert!(matches!(
            short.outcome,
            Outcome::Resource(Refusal { attempted: 14, .. })
        ));
        assert_eq!(short.value, "");
    }

    #[test]
    fn the_model_keeps_no_temporary_buffer() {
        // Matching walks borrowed remaining input and numeric references use a
        // fixed accumulator, so a very long digit run holds nothing.
        let digits = format!("&#{};", "9".repeat(50_000));
        let layout = build_layout("div", "id", Syntax::Unquoted, &digits, "");
        let lexed = lex(&layout.source);
        assert_eq!(lexed.outcome, Outcome::Complete);
        assert_eq!(lexed.value, FFFD);
        assert_eq!(lexed.retained, lexed.base + FFFD.len());
    }

    #[test]
    fn resource_exhaustion_is_not_unsupported_and_not_a_diagnostic() {
        let diagnostic = lex("<body><div id=\"&zzz;\">");
        assert_eq!(diagnostic.outcome, Outcome::Complete);
        assert_eq!(diagnostic.diags.len(), 1);
        let refused = lex_with("<body><div id=\"&amp;\">", 1, retained(9), Probe::None);
        assert!(matches!(refused.outcome, Outcome::Resource(_)));
        let outside = lex("<body><span id=\"&amp;\">");
        assert_eq!(
            outside.outcome,
            Outcome::Outside(Outside::UnsupportedElement)
        );
        assert_ne!(refused.outcome, outside.outcome);
        assert!(refused.diags.is_empty());
    }

    // -- negative controls ---------------------------------------------------

    #[test]
    fn data_and_rcdata_controls_are_unchanged() {
        assert_clean("data", data_control_failures(Probe::None));
    }

    #[test]
    fn data_and_rcdata_share_semantics_but_not_the_attribute_exception() {
        for ret in [Return::Data, Return::Rcdata] {
            let lexed = lex_text("&notit;", ret, UNLIMITED, Probe::None);
            assert_eq!(lexed.document_text, "\u{ac}it;");
        }
        let attribute = lex("<body><div id=\"&notit;\">");
        assert_eq!(attribute.value, "&notit;");
    }

    #[test]
    fn selected_boundary_controls_stay_outside_the_successor() {
        // Second attribute: the accepted Product resource fact, at the offset
        // where the second attribute begins.
        let second = lex("<body><div id=\"&amp;\" class=\"x\">");
        assert_eq!(
            second.outcome,
            Outcome::Resource(Refusal {
                resource: Resource::AttributesPerTag,
                limit: 1,
                attempted: 2,
                at: 22,
            })
        );
        assert!(second.tag.is_none());
        // Unsupported element families remain outside.
        for element in ["span", "p", "textarea", "em"] {
            let source = format!("<body><{element} id=\"&amp;\">");
            assert_eq!(
                lex(&source).outcome,
                Outcome::Outside(Outside::UnsupportedElement),
                "{element}"
            );
        }
        // Self-closing SelectedOrdinary stays outside this frontier.
        for source in [
            "<body><div id=\"&amp;\"/>",
            "<body><div id=\"&amp;\" />",
            "<body><div/>",
        ] {
            assert_eq!(
                lex(source).outcome,
                Outcome::Outside(Outside::SelfClosing),
                "{source}"
            );
        }
        // Valueless attributes and EOF in a tag are not part of the theorem.
        assert_eq!(
            lex("<body><div id>").outcome,
            Outcome::Outside(Outside::ValuelessAttribute)
        );
        assert_eq!(
            lex("<body><div id=\"&amp;").outcome,
            Outcome::Outside(Outside::EofInTag)
        );
    }

    #[test]
    fn reference_free_attributes_keep_their_modeled_meaning() {
        for (value, syntax) in [
            ("a", Syntax::DoubleQuoted),
            ("b c", Syntax::SingleQuoted),
            ("d", Syntax::Unquoted),
            ("", Syntax::DoubleQuoted),
        ] {
            let layout = build_layout("div", "id", syntax, value, "");
            let lexed = lex(&layout.source);
            assert_eq!(lexed.outcome, Outcome::Complete, "{value:?}");
            let attribute = lexed.tag.unwrap().attribute.unwrap();
            assert_eq!(attribute.interpreted_value, value);
            assert!(lexed.episodes.is_empty());
            assert!(lexed.diags.is_empty());
        }
    }

    // -- sealing: every wrong design is rejected ---------------------------

    const WRONG_DESIGNS: &[(Probe, &str)] = &[
        (Probe::WrongReturnState, "explicit"),
        (Probe::EmitAsCharacterToken, "matrix"),
        (Probe::NoAttributeException, "matrix"),
        (Probe::ExceptionEverywhere, "data"),
        (Probe::ShortestMatch, "matrix"),
        (Probe::NoMissingSemicolonDiag, "matrix"),
        (Probe::SwallowDelimiter, "explicit"),
        (Probe::SwapScalars, "matrix"),
        (Probe::NoInvalidReplacement, "matrix"),
        (Probe::WrappingNumeric, "matrix"),
        (Probe::NoControlRemap, "matrix"),
        (Probe::ConsumeThenRefuse, "resource"),
        (Probe::PartialAppend, "resource"),
        (Probe::RetainedByAuthoredLength, "resource"),
        (Probe::RetainedByScalarCount, "resource"),
        (Probe::SwallowAmbiguousSemicolon, "matrix"),
        (Probe::SemicolonDiagOnBareAmpersand, "matrix"),
    ];

    fn group(name: &str, probe: Probe) -> Vec<String> {
        match name {
            "matrix" => matrix_failures(probe),
            "explicit" => explicit_failures(probe),
            "data" => data_control_failures(probe),
            "resource" => resource_failures(probe),
            other => panic!("unknown group {other}"),
        }
    }

    #[test]
    fn the_correct_model_passes_every_group() {
        for name in ["matrix", "explicit", "data", "resource"] {
            assert_clean(name, group(name, Probe::None));
        }
    }

    #[test]
    fn every_wrong_design_is_caught() {
        for &(probe, expected_group) in WRONG_DESIGNS {
            let failures = group(expected_group, probe);
            assert!(
                !failures.is_empty(),
                "{probe:?} escaped the {expected_group} group"
            );
        }
    }

    #[test]
    fn every_probe_is_listed_for_sealing() {
        // Guard against adding a probe without a catching group.
        assert_eq!(WRONG_DESIGNS.len(), 17);
    }

    // -- observation of current production (never an oracle) ----------------

    #[test]
    fn production_still_stops_at_the_historical_attribute_value_boundary() {
        // Historical boundary selected for future supersession, not the new
        // theorem. `(source, hand-counted offset of the stopping '&')`.
        for (source, ampersand) in [
            ("<body><div id=\"a&amp;b\"></div>", 16),
            ("<body><div id='a&b'></div>", 16),
            ("<body><div id=a&b></div>", 15),
            ("<body><nav class=\"&#x41;\"></nav>", 18),
        ] {
            let run = observation::run(source);
            assert_eq!(
                observation::stop(&run),
                observation::Stop::AttributeValueReference {
                    at: (ampersand, ampersand + 1)
                },
                "{source}"
            );
            // The successor model completes the same authored shapes: the
            // model is not derived from, and does not agree with, production
            // at this boundary.
            let tag_end = source[6..].find('>').expect("a start tag") + 7;
            let model = lex(&source[..tag_end]);
            assert_eq!(model.outcome, Outcome::Complete, "{source}");
        }
    }

    #[test]
    fn model_agrees_with_accepted_tokenizer_evidence_on_reference_free_input() {
        for (source, label) in [
            ("<body><div id=\"a\">", "double"),
            ("<body><nav class='b c'>", "single"),
            ("<body><main data-x=abc>", "unquoted"),
            ("<body><DiV Title=\"x\">", "double"),
        ] {
            let run = observation::run(source);
            assert_eq!(observation::stop(&run), observation::Stop::Complete);
            let (tag_name, name, value, complete, name_range, value_range, syntax) =
                observation::attributed_start_tag(&run).expect("attributed start tag");
            let model = lex(source);
            let tag = model.tag.expect("model tag");
            let attribute = tag.attribute.expect("model attribute");
            assert_eq!(tag.element, tag_name, "{source}");
            assert_eq!(attribute.interpreted_name, name, "{source}");
            assert_eq!(attribute.interpreted_value, value, "{source}");
            assert_eq!(
                (attribute.complete.start, attribute.complete.end),
                complete,
                "{source}"
            );
            assert_eq!(
                (attribute.name.start, attribute.name.end),
                name_range,
                "{source}"
            );
            assert_eq!(
                (attribute.value.start, attribute.value.end),
                value_range,
                "{source}"
            );
            assert_eq!(
                match attribute.syntax {
                    Syntax::DoubleQuoted => "double",
                    Syntax::SingleQuoted => "single",
                    Syntax::Unquoted => "unquoted",
                },
                syntax,
                "{label}"
            );
        }
    }

    #[test]
    fn production_second_attribute_is_the_accepted_resource_fact() {
        let run = observation::run("<body><div a=1 b=2></div>");
        assert_eq!(
            observation::stop(&run),
            observation::Stop::AttributesPerTag {
                limit: 1,
                attempted: 2,
                at: (15, 15)
            }
        );
        // The model states the same fact at the same hand-counted offset.
        assert_eq!(
            lex("<body><div a=1 b=2>").outcome,
            Outcome::Resource(Refusal {
                resource: Resource::AttributesPerTag,
                limit: 1,
                attempted: 2,
                at: 15,
            })
        );
    }

    #[test]
    fn production_data_references_match_the_model_controls() {
        let text = "a&amp;b&notit;&#65;&amp=&ampx";
        let run = observation::run(text);
        assert_eq!(observation::stop(&run), observation::Stop::Complete);
        let model = lex_text(text, Return::Data, UNLIMITED, Probe::None);
        assert_eq!(observation::character_text(&run), model.document_text);
        assert_eq!(model.document_text, "a&b\u{ac}it;A&=&x");
        assert_eq!(observation::missing_semicolon_count(&run), 3);
        assert_eq!(observation::diagnostic_count(&run), model.diags.len());
        assert_eq!(model.diags.len(), 3);
    }

    #[test]
    fn product_resource_vector_keeps_temporary_buffer_zero() {
        let limits = observation::product_limits();
        assert_eq!(limits.max_temporary_buffer_bytes(), 0);
        assert_eq!(limits.max_attributes_per_tag(), ATTRIBUTES_PER_TAG);
    }

    // -- scope and firewall ---------------------------------------------------

    #[test]
    fn model_does_not_import_production_semantics() {
        let source = include_str!(
            "in_body_selected_ordinary_attribute_value_reference_successor_validation.rs"
        );
        let split = source
            .find(&["mod ", "observation {"].concat())
            .expect("observation module marker");
        let model = &source[..split];
        let forbidden = [
            ["super::", "super::"].concat(),
            ["crate::", "html"].concat(),
            ["use super::", "driver"].concat(),
            ["use super::", "session"].concat(),
            ["use super::", "result"].concat(),
            ["tree_construction", "::driver"].concat(),
            ["tree_construction", "::session"].concat(),
            ["tree_construction", "::result"].concat(),
            ["named_character_", "reference_data"].concat(),
            ["named_character_references_", "generated"].concat(),
            ["producer::", "named"].concat(),
            ["tokenize", "("].concat(),
            ["HtmlTree", "Report"].concat(),
            ["frontend_analysis_", "cli"].concat(),
        ];
        for needle in forbidden {
            assert!(
                !model.contains(&needle),
                "the model must not name production semantics: {needle}"
            );
        }
        // The only crate items the model uses are the anchoring primitives.
        assert!(model.contains("use crate::{SourceId, SourceText};"));
    }

    #[test]
    fn negative_control_in_901_is_preserved() {
        let validator = include_str!("in_body_selected_ordinary_attribute_successor_validation.rs");
        assert!(validator.contains(
            "fn ampersand_value_stays_at_the_attribute_value_character_reference_boundary"
        ));
        assert!(validator.contains("HtmlCharacterReferenceContext::AttributeValue"));
    }

    #[test]
    fn named_subset_is_hand_authored_and_closed_under_legacy_siblings() {
        // Every semicolonless entry is one of the pinned legacy names this
        // corpus relies on, and every entry has a decoded value.
        let semicolonless: Vec<&str> = NAMED
            .iter()
            .map(|(name, _)| *name)
            .filter(|name| !name.ends_with(';'))
            .collect();
        assert_eq!(semicolonless, ["amp", "copy", "gt", "lt", "nbsp", "not"]);
        for (name, value) in NAMED {
            assert!(!value.is_empty(), "{name}");
            assert!(name.is_ascii(), "{name}");
        }
        // A legacy name always has its semicolon-terminated sibling.
        for name in semicolonless {
            assert!(
                NAMED.iter().any(|(n, _)| *n == format!("{name};")),
                "{name}"
            );
        }
    }
}
