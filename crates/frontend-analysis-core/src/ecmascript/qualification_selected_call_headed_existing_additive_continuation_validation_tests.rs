//! Candidate-independent bounded call-headed existing additive continuation
//! source-shape and fact-preservation validation for Issue #853 (durable
//! research: Issue #688 comment `5908350749` post-#852 zero-base frontier
//! authority; accepted call-head authority: Issue #849 / PR #850 Oracle and
//! Issue #851 / PR #852 production; accepted additive authority: Issue #752
//! two-`IdentifierReference` initializer, #764 two-`IdentifierReference`
//! free-standing, #777 right-unary, #789 one-reference-one-Decimal, #795
//! three-`IdentifierReference` Oracles and their later composed widenings).
//!
//! This oracle qualifies only the owner-specific composition
//!
//! ```text
//! SelectedCallHeadedAdditive(owner) ::=
//!     SelectedZeroArgumentIdentifierReferenceCall
//!     ONE-OR-MORE ExistingSelectedAdditiveContinuation(owner)
//! ```
//!
//! where `SelectedZeroArgumentIdentifierReferenceCall` is exactly the
//! accepted Issue #849 head
//!
//! ```text
//! SelectedAcceptedIdentifierReference SelectedCallTrivia* "(" SelectedCallTrivia* ")"
//! ```
//!
//! and `ExistingSelectedAdditiveContinuation(owner)` is exactly the
//! continuation language the frozen baseline already accepts for that owner
//! after a first `IdentifierReference` atom. The only novelty proven here is
//! that the first additive atom may be the call head. No new additive
//! grammar, precedence layer, or expression parser is introduced: there is
//! deliberately **no** shared additive-tail recognizer.
//!
//! ## Owner-specific continuation theorem
//!
//! The initializer owner and the free-standing whole-body owner accept the
//! same *link* language on the frozen baseline,
//!
//! ```text
//! Link ::= Trivia ("+" | "-") Trivia ( PlainDecimal
//!                                    | ("+" | "-") Trivia IdentifierReference
//!                                    | IdentifierReference )
//! ```
//!
//! with the load-bearing punctuator boundary that an equal sign adjacent to
//! the binary sign with zero trivia (`++`, `--`) is never split into a binary
//! plus a unary sign, and with exactly one right-unary sign, never recursive
//! and never before a Decimal. They differ in their *failure contract*, which
//! is the load-bearing distinction this Oracle keeps observable:
//!
//! - **Initializer** (`scan_initializer_call_headed`): staged/local recovery.
//!   A failing link restores the latest completed initializer prefix and
//!   leaves the untouched suffix to the enclosing declaration owner, which
//!   alone owns `;` / EOF / before-`}` termination. With no complete link the
//!   call head stays with the call-only theorem.
//! - **Free-standing** (`scan_free_standing_call_headed`): whole-body
//!   transaction. The body owns the trivia after each atom; once an authored
//!   `+` / `-` has been consumed, any failure rolls the *whole* body back,
//!   with no prefix and no call-only fallback. Only an absent operator leaves
//!   the head to the call-only theorem.
//!
//! The two recognizers are written independently; they share only the leaf
//! atoms (the head scan, the `IdentifierReference` atom scan, the plain
//! Decimal atom scan, and the selected trivia scan). Neither is derived from
//! the other, and agreement of their accepted language is asserted by fixture,
//! not assumed.
//!
//! ## Evidence theorem
//!
//! For every accepted source: the callee is the first retained fact with its
//! exact authored range, decoded name, and Direct/EscapedNonReserved
//! provenance; every later reference operand contributes exactly one fact in
//! authored order (a right-unary operand retains only the inner reference);
//! Decimal operands, operators, delimiters, and trivia contribute no fact; and
//! no call identity, call result, arithmetic value, runtime binding,
//! callability, or property relation is retained or implied. The retained
//! evidence type is exactly a list of reference facts.
//!
//! ## Source-owned expectations
//!
//! Every expected range, fragment, name, provenance, link count, expression
//! end, and terminator is authored literally in a fixture (or composed piece
//! by piece from known piece widths for the trivia sweeps). Recognition never
//! feeds expectations: no source search, rescan, retokenization,
//! decoded-length inference, or endpoint reconstruction locates anything, and
//! the source-independence test below rejects the search vocabulary.
//!
//! ## Local static-semantics conclusion (hypothesis, falsified locally)
//!
//! Section "E. Early Error reachability model" restates, against the frozen
//! inventory, that the admitted syntax -- an `IdentifierReference` callee,
//! empty `Arguments`, selected trivia, binary and unary `+` / `-`, plain
//! Decimal operands, and tail `IdentifierReference` operands -- supplies no
//! trigger element of any frozen Early Error identity. The single identity
//! family that names the call cover, `AsyncArrowFunction :
//! CoverCallExpressionAndAsyncArrowHead => AsyncConciseBody` (frozen
//! `EE-35`), requires the `=>` token; an accepted source's token after the
//! call head is always `+` / `-`, and `async()+g=>x` stays outside. The
//! update identities (`EE-10`) need `++` / `--`, which the same-sign boundary
//! never produces. This is a *local* claim about this bounded composition
//! only; it is not an aggregate selected completion theorem and updates no
//! frozen inventory.
//!
//! ## Vocabulary
//!
//! `Matched`, `NotSelected`, `ResourceLimited`, and `InternalFailure` are the
//! Issue's four outcomes. This Oracle's recognizers are total,
//! allocation-bounded functions over one candidate and never produce the two
//! processing failures; they remain distinct identities and are never folded
//! into ordinary non-selection. Valid-but-outside ECMAScript and
//! malformed/incomplete candidates alike are `NotSelected`; this Oracle never
//! upgrades a decline into a normative `SyntaxError`.
//!
//! `SelectedCallTrivia` independently restates the same already-accepted
//! selected-slice trivia contract used by the #849 Oracle: `TAB`, `VT`, `FF`,
//! `BOM`, `LF`, `CR`, `LINE SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen
//! Unicode 17 `Space_Separator` property. Comments remain outside.
//!
//! This is a validation-only leaf: it does not call production lexical,
//! static-semantics, correspondence, Binding/Scope, aggregate, or runtime
//! evaluation code, so every positive fixture is independent Oracle evidence
//! for a future, separately authorized production decision, never a claim that
//! production already accepts it. No runtime, direct-eval, `ResolveBinding`,
//! `IsCallable`, `GetValue`, arithmetic, coercion, or value-flow claim is
//! made: `eval()` is only source syntax here.
//!
//! Production is explicitly NOT authorized by Issue #853.

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_CALL_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_zero_argument_identifier_reference_call_expression_validation_tests.rs"
);
const PREVIOUS_TWO_REFERENCE_INITIALIZER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_initializer_validation_tests.rs"
);
const PREVIOUS_TWO_REFERENCE_FREE_STANDING_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_expression_statement_use_site_validation_tests.rs"
);
const PREVIOUS_ONE_REFERENCE_ONE_DECIMAL_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_one_reference_one_plain_decimal_additive_initializer_validation_tests.rs"
);
const PREVIOUS_THREE_REFERENCE_INITIALIZER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_three_identifier_reference_additive_initializer_validation_tests.rs"
);
const PREVIOUS_RIGHT_UNARY_INITIALIZER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_identifier_reference_right_unary_additive_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_call_headed_existing_additive_continuation_validation_tests.rs"
);
const COMPLETION_AUTHORITY_SOURCE: &str =
    include_str!("qualification_validation_tests/selected_slice_completion.rs");
const INVENTORY_SOURCE: &str = include_str!("qualification_validation_tests/inventory.rs");
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "call-headed existing additive continuation frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "a call in the additive tail, non-empty Arguments, chained/member/optional/",
    "new/super calls, multiplicative/assignment/update/conditional/logical ",
    "expressions, grouping, comments, AsyncArrowFunction/cover-grammar ",
    "reinterpretation, runtime call or arithmetic semantics, or deeper Blocks"
);

const UNCONDITIONALLY_RESERVED_WORDS: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
];

/// Every code point of the selected trivia contract that this Oracle sweeps
/// in both trivia positions: the eight explicit selected code points plus a
/// representative spread of the frozen Unicode 17 `Space_Separator` property.
const SELECTED_TRIVIA_SWEEP: &[char] = &[
    '\u{0009}', '\u{000B}', '\u{000C}', '\u{0020}', '\u{00A0}', '\u{FEFF}', '\u{000A}', '\u{000D}',
    '\u{2028}', '\u{2029}', '\u{1680}', '\u{2000}', '\u{200A}', '\u{202F}', '\u{205F}', '\u{3000}',
];

/// Code points that look like separators but are not selected trivia under
/// the frozen contract (`Cf` format characters, `NEL`, and the Unicode 6.3
/// reclassified `U+180E`). None of them is also an `IdentifierPart`.
const NON_TRIVIA_SEPARATOR_LOOKALIKES: &[char] = &['\u{0085}', '\u{180E}', '\u{200B}', '\u{2060}'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentifierReferenceProvenance {
    Direct,
    EscapedNonReserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodeFailure {
    MalformedEscape,
    NonCodePoint,
    InvalidStart,
    InvalidPart,
    DecodedReserved,
}

/// Atom failures, kept exactly as the already-accepted `IdentifierReference`
/// policy classifies them; never re-labelled as call- or additive-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CalleeFailure {
    NoIdentifierStart,
    DirectReservedWord,
    Decode(DecodeFailure),
}

/// Why the accepted #849 call head is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclineReason {
    Callee(CalleeFailure),
    /// After the callee and selected trivia the next code point is not `(`.
    MissingCallOpener,
    /// The candidate ends before the paired `)`.
    IncompleteSuffix,
    /// A code point other than selected trivia or `)` follows `(`
    /// (non-empty arguments, comments, spread, ...).
    UnownedInsideArguments,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CalleeEvidence {
    range: Range,
    name: String,
    provenance: IdentifierReferenceProvenance,
}

/// Oracle-side head evidence with candidate-relative ranges. `close.1` is the
/// end of the complete zero-argument call head. It creates no requirement that
/// a future production consumer store a call anchor, delimiter anchors, an
/// `Arguments` identity, a call operator, or a call result.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CallEvidence {
    callee: CalleeEvidence,
    close: Range,
}

fn slice(text: &str, Range(start, end): Range) -> &str {
    text.get(start..end)
        .unwrap_or_else(|| panic!("invalid UTF-8 range ({start},{end}) in {text:?}"))
}

fn authored_anchor(source_id: u64, text: &str, range: Range) -> String {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    let Range(start, end) = range;
    source
        .anchor(start, end)
        .expect("fixture range must be a valid authored anchor")
        .fragment()
        .to_owned()
}

fn shift(range: Range, base: usize) -> Range {
    Range(range.0 + base, range.1 + base)
}

/// Independently restates only the already-accepted Issue #237 direct
/// `IdentifierStart` code-point shape.
fn is_direct_identifier_start(code_point: char) -> bool {
    matches!(code_point, '$' | '_') || is_id_start(code_point as u32)
}

/// Independently restates only the already-accepted Issue #237 direct
/// `IdentifierPart` code-point shape.
fn is_direct_identifier_part(code_point: char) -> bool {
    code_point == '$' || is_id_continue(code_point as u32)
}

/// Independently restates only the already-accepted Issue #237 direct
/// escape-free `IdentifierName` code-point shape: a pure code-point-family
/// check, never a reserved-word policy on its own.
fn is_escape_free_identifier_name(spelling: &str) -> bool {
    let mut code_points = spelling.chars();
    let Some(first) = code_points.next() else {
        return false;
    };
    is_direct_identifier_start(first) && code_points.all(is_direct_identifier_part)
}

/// `SelectedDirectIdentifierReference`: the already-accepted direct
/// escape-free `IdentifierName` shape, minus the unconditionally reserved
/// words that Script/non-strict/`Yield=false`/`Await=false` never admits as
/// an `IdentifierReference`.
fn is_selected_direct_identifier_reference(candidate: &str) -> bool {
    is_escape_free_identifier_name(candidate)
        && !UNCONDITIONALLY_RESERVED_WORDS.contains(&candidate)
}

fn ascii_hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

/// Parses the hex digits inside a braced `\u{HexDigits}` escape, independently
/// restating only the already-accepted Issue #241 braced-form boundary:
/// leading zeros are permitted without limit, but at most six significant hex
/// digits are, and the resulting value must be a valid Unicode code point
/// (`<= 0x10FFFF`).
fn parse_braced_code_point(digits: &[u8]) -> Result<u32, DecodeFailure> {
    if digits.is_empty() || digits.iter().any(|byte| ascii_hex_value(*byte).is_none()) {
        return Err(DecodeFailure::MalformedEscape);
    }

    let significant = match digits.iter().position(|byte| *byte != b'0') {
        Some(index) => &digits[index..],
        None => return Ok(0),
    };

    if significant.len() > 6 {
        return Err(DecodeFailure::NonCodePoint);
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }

    (value <= 0x10_FFFF)
        .then_some(value)
        .ok_or(DecodeFailure::NonCodePoint)
}

/// Decodes exactly one `UnicodeEscapeSequence` starting at `start` -- either
/// the fixed-form `\uXXXX` or the braced `\u{HexDigits}` form -- returning
/// its code point and the byte offset immediately after it.
fn decode_unicode_escape_at(bytes: &[u8], start: usize) -> Result<(u32, usize), DecodeFailure> {
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return Err(DecodeFailure::MalformedEscape);
    }
    let payload = start + 2;

    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload + 1;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(u8::is_ascii_hexdigit) {
            end += 1;
        }
        if end == digits_start || bytes.get(end) != Some(&b'}') {
            return Err(DecodeFailure::MalformedEscape);
        }
        let code_point = parse_braced_code_point(&bytes[digits_start..end])?;
        return Ok((code_point, end + 1));
    }

    let digits = bytes
        .get(payload..payload + 4)
        .ok_or(DecodeFailure::MalformedEscape)?;
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return Err(DecodeFailure::MalformedEscape);
    }
    let mut value = 0_u32;
    for byte in digits {
        value = value * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }
    Ok((value, payload + 4))
}

fn is_selected_identifier_start_code_point(code_point: u32) -> bool {
    char::from_u32(code_point).is_some_and(is_direct_identifier_start)
}

fn is_selected_identifier_part_code_point(code_point: u32) -> bool {
    char::from_u32(code_point).is_some_and(is_direct_identifier_part)
}

/// `SelectedEscapedNonReservedIdentifierReference`: independently restates
/// the fixed-form `\uXXXX` and braced `\u{HexDigits}` subset of the
/// already-accepted Issue #241 escaped `IdentifierReference`
/// decode/position/ReservedWord boundary. Returns the decoded semantic name,
/// never the authored spelling.
fn decode_selected_escaped_identifier(spelling: &str) -> Result<String, DecodeFailure> {
    let bytes = spelling.as_bytes();
    let mut offset = 0_usize;
    let mut element_index = 0_usize;
    let mut code_points: Vec<u32> = Vec::new();

    while offset < bytes.len() {
        let (code_point, end) = if bytes[offset] == b'\\' {
            decode_unicode_escape_at(bytes, offset)?
        } else {
            let scalar = spelling[offset..]
                .chars()
                .next()
                .ok_or(DecodeFailure::MalformedEscape)?;
            (scalar as u32, offset + scalar.len_utf8())
        };

        let valid_position = if element_index == 0 {
            is_selected_identifier_start_code_point(code_point)
        } else {
            is_selected_identifier_part_code_point(code_point)
        };
        if !valid_position {
            return Err(if element_index == 0 {
                DecodeFailure::InvalidStart
            } else {
                DecodeFailure::InvalidPart
            });
        }

        code_points.push(code_point);
        offset = end;
        element_index += 1;
    }

    if code_points.is_empty() {
        return Err(DecodeFailure::MalformedEscape);
    }

    let mut string_value = String::new();
    for code_point in &code_points {
        string_value.push(
            char::from_u32(*code_point)
                .expect("validated position-valid code point must be a Unicode scalar value"),
        );
    }

    if UNCONDITIONALLY_RESERVED_WORDS.contains(&string_value.as_str()) {
        return Err(DecodeFailure::DecodedReserved);
    }

    Ok(string_value)
}

/// `SelectedAcceptedIdentifierReference` over one *whole* spelling: Direct
/// first, EscapedNonReserved otherwise. This is the bare-atom authority; it
/// owns `a` and `\u0061` on their own and never sees call punctuation.
fn classify_selected_accepted_identifier_reference(
    operand: &str,
) -> Option<(String, IdentifierReferenceProvenance)> {
    if is_selected_direct_identifier_reference(operand) {
        return Some((operand.to_owned(), IdentifierReferenceProvenance::Direct));
    }
    decode_selected_escaped_identifier(operand)
        .ok()
        .map(|name| (name, IdentifierReferenceProvenance::EscapedNonReserved))
}

/// `SelectedCallTrivia` code point: the frozen `is_selected_trivia`
/// code-point set plus the frozen Unicode 17 `Space_Separator` property. Not
/// a call into the production selected-trivia recognizer. Comments are not
/// members.
fn is_selected_call_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

/// Consumes zero or more selected trivia code points starting at `from` and
/// returns the offset after the last one.
fn skip_call_trivia(text: &str, from: usize) -> usize {
    let mut end = from;
    for code_point in text[from..].chars() {
        if !is_selected_call_trivia(code_point) {
            break;
        }
        end += code_point.len_utf8();
    }
    end
}

/// Oracle-owned maximal-`IdentifierName` callee scan over the head of
/// `text`: one forward pass consuming identifier-part code points and whole
/// `UnicodeEscapeSequence`s, then classification of exactly the consumed
/// spelling through the restated bare-atom policy. The callee range is what
/// the scan consumed -- never an endpoint reconstructed from a decoded
/// length or a later search.
fn scan_callee(text: &str) -> Result<CalleeEvidence, CalleeFailure> {
    let bytes = text.as_bytes();
    let mut offset = 0_usize;
    let mut has_escape = false;

    while offset < bytes.len() {
        if bytes[offset] == b'\\' {
            let (_, end) =
                decode_unicode_escape_at(bytes, offset).map_err(CalleeFailure::Decode)?;
            has_escape = true;
            offset = end;
            continue;
        }
        let scalar = text[offset..]
            .chars()
            .next()
            .expect("offset is on a code point boundary inside the text");
        if !is_direct_identifier_part(scalar) {
            break;
        }
        offset += scalar.len_utf8();
    }

    if offset == 0 {
        return Err(CalleeFailure::NoIdentifierStart);
    }
    let spelling = &text[..offset];

    let (name, provenance) = if has_escape {
        let name = decode_selected_escaped_identifier(spelling).map_err(CalleeFailure::Decode)?;
        (name, IdentifierReferenceProvenance::EscapedNonReserved)
    } else if !is_escape_free_identifier_name(spelling) {
        return Err(CalleeFailure::Decode(DecodeFailure::InvalidStart));
    } else if UNCONDITIONALLY_RESERVED_WORDS.contains(&spelling) {
        return Err(CalleeFailure::DirectReservedWord);
    } else {
        (spelling.to_owned(), IdentifierReferenceProvenance::Direct)
    };

    Ok(CalleeEvidence {
        range: Range(0, offset),
        name,
        provenance,
    })
}

/// Central Issue #849 theorem, *prefix* form: the head of `text` is exactly
/// one accepted callee, zero or more selected trivia, exactly one `(`, zero
/// or more selected trivia, and exactly one paired `)`. The scan is a single
/// left-to-right pass of this Oracle's own; it never searches, rescans, or
/// reparses. A locally complete prefix is *not* a whole-candidate match: see
/// [`recognize_call`].
fn scan_zero_argument_call_prefix(text: &str) -> Result<CallEvidence, DeclineReason> {
    let callee = scan_callee(text).map_err(DeclineReason::Callee)?;

    let open_at = skip_call_trivia(text, callee.range.1);
    if !text[open_at..].starts_with('(') {
        return Err(DeclineReason::MissingCallOpener);
    }
    let close_at = skip_call_trivia(text, open_at + 1);
    match text[close_at..].chars().next() {
        None => Err(DeclineReason::IncompleteSuffix),
        Some(')') => Ok(CallEvidence {
            callee,
            close: Range(close_at, close_at + 1),
        }),
        Some(_) => Err(DeclineReason::UnownedInsideArguments),
    }
}

// ---------------------------------------------------------------------------
// A. Oracle-owned, owner-specific call-headed additive recognition
// ---------------------------------------------------------------------------

/// One retained `IdentifierReference` fact: the authored range, the decoded
/// semantic name, and Direct/EscapedNonReserved provenance. It is the only
/// evidence kind this Oracle ever retains; the callee and every reference
/// operand share it.
type ReferenceFact = CalleeEvidence;

/// The Issue's four-way failure separation. This Oracle's recognizers are
/// total, allocation-bounded functions over one candidate and never produce
/// `ResourceLimited` / `InternalFailure`; the identities exist so a decline is
/// never confused with a processing failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    Matched,
    NotSelected,
    ResourceLimited,
    InternalFailure,
}

/// The two ownership models that keep their own, different failure contracts
/// after the shared call head.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Owner {
    /// Initializer-owned continuation: staged/local recovery to the latest
    /// completed initializer prefix.
    Initializer,
    /// Free-standing whole-body continuation: whole-body transaction that
    /// rolls back entirely once an authored additive operator has started.
    FreeStanding,
}

/// Why an authored additive operator did not complete a link, for the
/// free-standing whole-body rollback only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RollbackReason {
    /// Authored `++` / `--` (an equal sign adjacent with zero trivia) is never
    /// split into a binary sign plus a unary sign.
    SameSignAdjacency,
    /// No accepted operand followed the operator (and optional single unary
    /// sign): unselected, malformed, escaped-ReservedWord, or absent.
    OperandNotSelected,
}

/// A call head followed by one-or-more complete additive links. `end` is
/// owner-specific: the initializer stops exactly at the end of its latest
/// completed operand; the free-standing body also owns the trivia after it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CallHeadedPrefix {
    /// Callee first, then one fact per reference operand in authored order.
    /// Decimal operands, operators, delimiters, and trivia add nothing.
    facts: Vec<ReferenceFact>,
    /// Authored end of the complete call head `)`.
    head_end: usize,
    /// Number of complete additive links (always at least one).
    links: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InitializerRecognition {
    /// Call head plus at least one complete link. Anything after `end` is
    /// left, untouched, for the enclosing declaration owner to judge.
    Matched(CallHeadedPrefix),
    /// No complete link exists. The call-only theorem keeps ownership of the
    /// call head, and the untouched suffix stays with the declaration owner;
    /// no call-headed-additive result is produced.
    CallOnlyRemains {
        call_only_end: usize,
    },
    Head(DeclineReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FreeStandingRecognition {
    Matched(CallHeadedPrefix),
    /// No authored additive operator follows the call head at all: the
    /// call-only theorem keeps ownership.
    CallOnlyRemains {
        call_only_end: usize,
    },
    /// An authored additive operator started and failed: the whole body rolls
    /// back. No fact list, prefix, or call-only fallback escapes.
    WholeBodyRollback(RollbackReason),
    Head(DeclineReason),
}

/// `+` or `-` at `at`, if any.
fn sign_at(text: &str, at: usize) -> Option<char> {
    match text[at..].chars().next() {
        Some(sign @ ('+' | '-')) => Some(sign),
        _ => None,
    }
}

/// Independently restated plain-Decimal continuation atom (the exact family
/// admitted by Issue #789 / #817 / #825 initializer and #764 free-standing
/// authority): `DecimalIntegerLiteral`, `DecimalIntegerLiteral . DecimalDigits?`,
/// `. DecimalDigits`, each with an optional `ExponentPart`, where a leading
/// `0` is never followed by another digit. No separators, no `n` suffix, no
/// non-decimal radix, and no legacy octal-like spelling. Returns the authored
/// end of the maximal match. An `ExponentPart` without digits is not consumed.
fn scan_plain_decimal(text: &str, at: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let digits_from = |mut offset: usize| {
        while bytes.get(offset).is_some_and(u8::is_ascii_digit) {
            offset += 1;
        }
        offset
    };

    let mut end = at;
    let has_integer_part = match bytes.get(end) {
        Some(b'0') => {
            end += 1;
            if bytes.get(end).is_some_and(u8::is_ascii_digit) {
                return None;
            }
            true
        }
        Some(b'1'..=b'9') => {
            end = digits_from(end + 1);
            true
        }
        _ => false,
    };

    let mut has_fraction_digits = false;
    if bytes.get(end) == Some(&b'.') {
        let fraction_start = end + 1;
        end = digits_from(fraction_start);
        has_fraction_digits = end > fraction_start;
    }
    if !has_integer_part && !has_fraction_digits {
        return None;
    }

    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let mut exponent = end + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        let exponent_end = digits_from(exponent);
        if exponent_end > exponent {
            end = exponent_end;
        }
    }
    Some(end)
}

/// One complete initializer-owned additive link starting at `from`
/// (immediately after the latest completed operand, before any trivia):
///
/// ```text
/// InitializerLink ::= Trivia "+"|"-" Trivia
///                     ( Decimal | ("+"|"-") Trivia Reference | Reference )
/// ```
///
/// with the same-sign zero-trivia firewall. `None` means there is no complete
/// link here (absent operator, firewall, or unselected/incomplete operand);
/// the caller then recovers to `from`, the latest completed prefix, and the
/// untouched suffix is the declaration owner's to judge.
fn scan_initializer_link(text: &str, from: usize) -> Option<(Option<ReferenceFact>, usize)> {
    let mut at = skip_call_trivia(text, from);
    let binary = sign_at(text, at)?;
    at += 1;
    let after_operator = at;
    at = skip_call_trivia(text, at);

    if let Some(unary) = sign_at(text, at) {
        if unary == binary && at == after_operator {
            return None;
        }
        let operand_at = skip_call_trivia(text, at + 1);
        let fact = scan_reference_at(text, operand_at).ok()?;
        let end = fact.range.1;
        return Some((Some(fact), end));
    }
    if let Ok(fact) = scan_reference_at(text, at) {
        let end = fact.range.1;
        return Some((Some(fact), end));
    }
    scan_plain_decimal(text, at).map(|end| (None, end))
}

/// `SelectedCallHeadedAdditive(Initializer)`: the exact #852 call head, then
/// one-or-more links of the already-accepted initializer-owned continuation.
/// A failing link recovers to the latest completed prefix, exactly as the
/// initializer owner already treats every incomplete continuation.
fn scan_initializer_call_headed(text: &str) -> InitializerRecognition {
    let head = match scan_zero_argument_call_prefix(text) {
        Ok(head) => head,
        Err(reason) => return InitializerRecognition::Head(reason),
    };
    let head_end = head.close.1;
    let mut facts = vec![head.callee];
    let mut completed_end = head_end;
    let mut links = 0_usize;

    while let Some((fact, end)) = scan_initializer_link(text, completed_end) {
        facts.extend(fact);
        completed_end = end;
        links += 1;
    }

    if links == 0 {
        return InitializerRecognition::CallOnlyRemains {
            call_only_end: head_end,
        };
    }
    InitializerRecognition::Matched(CallHeadedPrefix {
        facts,
        head_end,
        links,
        end: completed_end,
    })
}

/// `SelectedCallHeadedAdditive(FreeStanding)`: the exact #852 call head, then
/// one-or-more links of the already-accepted free-standing whole-body
/// continuation. The body owns the trivia after every atom. Once an authored
/// `+` / `-` has been consumed, any failure rolls the whole body back.
fn scan_free_standing_call_headed(text: &str) -> FreeStandingRecognition {
    let head = match scan_zero_argument_call_prefix(text) {
        Ok(head) => head,
        Err(reason) => return FreeStandingRecognition::Head(reason),
    };
    let head_end = head.close.1;
    let mut cursor = skip_call_trivia(text, head_end);
    if sign_at(text, cursor).is_none() {
        return FreeStandingRecognition::CallOnlyRemains {
            call_only_end: head_end,
        };
    }

    let mut facts = vec![head.callee];
    let mut links = 0_usize;
    while let Some(binary) = sign_at(text, cursor) {
        let after_operator = cursor + 1;
        let mut at = skip_call_trivia(text, after_operator);

        let operand_end = if let Some(unary) = sign_at(text, at) {
            if unary == binary && at == after_operator {
                return FreeStandingRecognition::WholeBodyRollback(
                    RollbackReason::SameSignAdjacency,
                );
            }
            at = skip_call_trivia(text, at + 1);
            match scan_reference_at(text, at) {
                Ok(fact) => {
                    let end = fact.range.1;
                    facts.push(fact);
                    end
                }
                Err(_) => {
                    return FreeStandingRecognition::WholeBodyRollback(
                        RollbackReason::OperandNotSelected,
                    );
                }
            }
        } else if let Ok(fact) = scan_reference_at(text, at) {
            let end = fact.range.1;
            facts.push(fact);
            end
        } else if let Some(end) = scan_plain_decimal(text, at) {
            end
        } else {
            return FreeStandingRecognition::WholeBodyRollback(RollbackReason::OperandNotSelected);
        };

        links += 1;
        cursor = skip_call_trivia(text, operand_end);
    }

    FreeStandingRecognition::Matched(CallHeadedPrefix {
        facts,
        head_end,
        links,
        end: cursor,
    })
}

/// One reference operand or callee atom at `at`, shifted from the atom scan's
/// own text-relative start.
fn scan_reference_at(text: &str, at: usize) -> Result<ReferenceFact, CalleeFailure> {
    let fact = scan_callee(&text[at..])?;
    Ok(ReferenceFact {
        range: shift(fact.range, at),
        ..fact
    })
}

// ---------------------------------------------------------------------------
// A2. Placement carriers (fixture-owned) and whole-source recognition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Placement {
    LexicalInitializer,
    TopLevelVarInitializer,
    BlockVarInitializer,
    BlockLexicalInitializer,
    TopLevelFreeStanding,
    BlockFreeStanding,
}

/// Placement-owned termination. The additive continuation never owns `;`,
/// EOF, or before-`}` termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Terminator {
    AuthoredSemicolon,
    AutomaticAtEof,
    AutomaticBeforeBlockClose,
}

/// A fixture-owned carrier: `open` is the literal source before the
/// expression region, `base` the literal number of bytes it occupies (a
/// fixture constant, asserted against `open`, never measured to locate
/// anything), and `block` whether the placement is the one-level Block.
struct PlacementSpec {
    placement: Placement,
    owner: Owner,
    open: &'static str,
    base: usize,
    block: bool,
    /// The terminations this placement's owner accepts. A Block lexical
    /// declaration owns only an authored `;`; the Block `var` and Block
    /// free-standing owners also accept before-`}` automatic termination.
    terminators: &'static [Terminator],
}

const TOP_LEVEL_TERMINATORS: &[Terminator] =
    &[Terminator::AuthoredSemicolon, Terminator::AutomaticAtEof];
const BLOCK_TERMINATORS: &[Terminator] = &[
    Terminator::AuthoredSemicolon,
    Terminator::AutomaticBeforeBlockClose,
];
const BLOCK_LEXICAL_TERMINATORS: &[Terminator] = &[Terminator::AuthoredSemicolon];

const PLACEMENTS: &[PlacementSpec] = &[
    PlacementSpec {
        placement: Placement::LexicalInitializer,
        owner: Owner::Initializer,
        open: "const x = ",
        base: 10,
        block: false,
        terminators: TOP_LEVEL_TERMINATORS,
    },
    PlacementSpec {
        placement: Placement::TopLevelVarInitializer,
        owner: Owner::Initializer,
        open: "var x = ",
        base: 8,
        block: false,
        terminators: TOP_LEVEL_TERMINATORS,
    },
    PlacementSpec {
        placement: Placement::BlockVarInitializer,
        owner: Owner::Initializer,
        open: "{ var x = ",
        base: 10,
        block: true,
        terminators: BLOCK_TERMINATORS,
    },
    PlacementSpec {
        placement: Placement::BlockLexicalInitializer,
        owner: Owner::Initializer,
        open: "{ let x = ",
        base: 10,
        block: true,
        terminators: BLOCK_LEXICAL_TERMINATORS,
    },
    PlacementSpec {
        placement: Placement::TopLevelFreeStanding,
        owner: Owner::FreeStanding,
        open: "",
        base: 0,
        block: false,
        terminators: TOP_LEVEL_TERMINATORS,
    },
    PlacementSpec {
        placement: Placement::BlockFreeStanding,
        owner: Owner::FreeStanding,
        open: "{ ",
        base: 2,
        block: true,
        terminators: BLOCK_TERMINATORS,
    },
];

/// The placement-owned terminations of a carrier, as literal source text
/// following the expression region.
fn closers(spec: &PlacementSpec) -> Vec<(&'static str, Terminator)> {
    let (authored, automatic, automatic_terminator) = if spec.block {
        ("; }", " }", Terminator::AutomaticBeforeBlockClose)
    } else {
        (";", "", Terminator::AutomaticAtEof)
    };
    [
        (authored, Terminator::AuthoredSemicolon),
        (automatic, automatic_terminator),
    ]
    .into_iter()
    .filter(|(_, terminator)| spec.terminators.contains(terminator))
    .collect()
}

/// Bytes of selected trivia that a carrier terminator authors before its own
/// terminator token. The free-standing body owns that trivia; the initializer
/// leaves it to the declaration owner. Fixture-owned: the only such closer is
/// the Block `" }"`.
fn closer_lead(close: &str) -> usize {
    usize::from(close.starts_with(' '))
}

/// Placement-owned terminator scan at `at`. A Block ends exactly at its `}`.
fn scan_terminator(text: &str, at: usize, block: bool) -> Option<Terminator> {
    let at = skip_call_trivia(text, at);
    let authored = text[at..].starts_with(';');
    if block {
        let after = if authored {
            skip_call_trivia(text, at + 1)
        } else {
            at
        };
        return (&text[after..] == "}").then_some(if authored {
            Terminator::AuthoredSemicolon
        } else {
            Terminator::AutomaticBeforeBlockClose
        });
    }
    if authored {
        return (at + 1 == text.len()).then_some(Terminator::AuthoredSemicolon);
    }
    (at == text.len()).then_some(Terminator::AutomaticAtEof)
}

/// Source-absolute call-headed additive evidence for a whole carrier source.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceEvidence {
    facts: Vec<ReferenceFact>,
    links: usize,
    expression_end: usize,
    terminator: Terminator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NotSelectedReason {
    CarrierMismatch,
    Head(DeclineReason),
    /// No complete additive link: call-only ownership remains.
    CallOnlyRemains {
        call_only_end: usize,
    },
    /// Free-standing whole-body rollback: no fact list, prefix, or call-only
    /// fallback escapes.
    WholeBodyRollback(RollbackReason),
    /// A complete call-headed prefix exists, but the placement owner finds
    /// no terminator after it. The prefix is never promoted to a result.
    LeftoverAfterPrefix {
        prefix_end: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SourceRecognition {
    Matched(SourceEvidence),
    NotSelected(NotSelectedReason),
}

fn spec_of(placement: Placement) -> &'static PlacementSpec {
    let spec = match placement {
        Placement::LexicalInitializer => &PLACEMENTS[0],
        Placement::TopLevelVarInitializer => &PLACEMENTS[1],
        Placement::BlockVarInitializer => &PLACEMENTS[2],
        Placement::BlockLexicalInitializer => &PLACEMENTS[3],
        Placement::TopLevelFreeStanding => &PLACEMENTS[4],
        Placement::BlockFreeStanding => &PLACEMENTS[5],
    };
    assert_eq!(spec.placement, placement);
    spec
}

fn absolute_facts(prefix: CallHeadedPrefix, base: usize) -> Vec<ReferenceFact> {
    prefix
        .facts
        .into_iter()
        .map(|fact| ReferenceFact {
            range: shift(fact.range, base),
            ..fact
        })
        .collect()
}

/// Whole-source recognition through the owner-specific path of `placement`.
/// Transactional: a `Matched` result exists only when the carrier prefix, the
/// call-headed additive expression, and the placement-owned terminator all
/// hold; every other shape is `NotSelected` and carries no fact list.
fn recognize_source(placement: Placement, source: &str) -> SourceRecognition {
    let spec = spec_of(placement);
    if !source.starts_with(spec.open) {
        return SourceRecognition::NotSelected(NotSelectedReason::CarrierMismatch);
    }
    let text = &source[spec.base..];

    let (prefix, expression_end) = match spec.owner {
        Owner::Initializer => match scan_initializer_call_headed(text) {
            InitializerRecognition::Matched(prefix) => {
                let end = prefix.end;
                (prefix, end)
            }
            InitializerRecognition::CallOnlyRemains { call_only_end } => {
                return SourceRecognition::NotSelected(NotSelectedReason::CallOnlyRemains {
                    call_only_end: spec.base + call_only_end,
                });
            }
            InitializerRecognition::Head(reason) => {
                return SourceRecognition::NotSelected(NotSelectedReason::Head(reason));
            }
        },
        Owner::FreeStanding => match scan_free_standing_call_headed(text) {
            FreeStandingRecognition::Matched(prefix) => {
                let end = prefix.end;
                (prefix, end)
            }
            FreeStandingRecognition::CallOnlyRemains { call_only_end } => {
                return SourceRecognition::NotSelected(NotSelectedReason::CallOnlyRemains {
                    call_only_end: spec.base + call_only_end,
                });
            }
            FreeStandingRecognition::WholeBodyRollback(reason) => {
                return SourceRecognition::NotSelected(NotSelectedReason::WholeBodyRollback(
                    reason,
                ));
            }
            FreeStandingRecognition::Head(reason) => {
                return SourceRecognition::NotSelected(NotSelectedReason::Head(reason));
            }
        },
    };

    let Some(terminator) = scan_terminator(text, expression_end, spec.block)
        .filter(|terminator| spec.terminators.contains(terminator))
    else {
        return SourceRecognition::NotSelected(NotSelectedReason::LeftoverAfterPrefix {
            prefix_end: spec.base + expression_end,
        });
    };
    let links = prefix.links;
    SourceRecognition::Matched(SourceEvidence {
        facts: absolute_facts(prefix, spec.base),
        links,
        expression_end: spec.base + expression_end,
        terminator,
    })
}

fn classify_source(placement: Placement, source: &str) -> FrontierOutcome {
    match recognize_source(placement, source) {
        SourceRecognition::Matched(_) => FrontierOutcome::Matched,
        SourceRecognition::NotSelected(_) => FrontierOutcome::NotSelected,
    }
}

// ---------------------------------------------------------------------------
// B. Source-composition fixtures (fixture-owned literal ranges)
// ---------------------------------------------------------------------------

const DIRECT: IdentifierReferenceProvenance = IdentifierReferenceProvenance::Direct;
const ESCAPED: IdentifierReferenceProvenance = IdentifierReferenceProvenance::EscapedNonReserved;

/// One expected retained fact, authored directly in the fixture:
/// expression-relative range, exact authored fragment, decoded semantic name,
/// and provenance. Nothing here is derived from recognition.
#[derive(Debug, Clone, Copy)]
struct Fx {
    range: Range,
    fragment: &'static str,
    name: &'static str,
    provenance: IdentifierReferenceProvenance,
}

const fn dir(start: usize, end: usize, name: &'static str) -> Fx {
    Fx {
        range: Range(start, end),
        fragment: name,
        name,
        provenance: DIRECT,
    }
}

const fn esc(start: usize, end: usize, fragment: &'static str, name: &'static str) -> Fx {
    Fx {
        range: Range(start, end),
        fragment,
        name,
        provenance: ESCAPED,
    }
}

/// A positive call-headed additive expression: the expected retained fact
/// list (callee first), the fixture-owned count of complete additive links,
/// and the fixture-owned expression-relative end of the last operand.
struct ExprRow {
    expr: &'static str,
    facts: &'static [Fx],
    links: usize,
    end: usize,
}

const fn row(expr: &'static str, facts: &'static [Fx], links: usize, end: usize) -> ExprRow {
    ExprRow {
        expr,
        facts,
        links,
        end,
    }
}

const BASIC_ROWS: &[ExprRow] = &[
    row("f()+g", &[dir(0, 1, "f"), dir(4, 5, "g")], 1, 5),
    row("f()-g", &[dir(0, 1, "f"), dir(4, 5, "g")], 1, 5),
    row("foo()+bar", &[dir(0, 3, "foo"), dir(6, 9, "bar")], 1, 9),
    row(
        "\u{03C0}()+g",
        &[dir(0, 2, "\u{03C0}"), dir(5, 6, "g")],
        1,
        6,
    ),
    row(
        "f()+\u{03C0}",
        &[dir(0, 1, "f"), dir(4, 6, "\u{03C0}")],
        1,
        6,
    ),
    row(
        "\u{1D49C}()+g",
        &[dir(0, 4, "\u{1D49C}"), dir(7, 8, "g")],
        1,
        8,
    ),
    row(
        "f()+\u{1D49C}",
        &[dir(0, 1, "f"), dir(4, 8, "\u{1D49C}")],
        1,
        8,
    ),
];

const ESCAPED_HEAD_ROWS: &[ExprRow] = &[
    row(
        r"\u0066()+g",
        &[esc(0, 6, r"\u0066", "f"), dir(9, 10, "g")],
        1,
        10,
    ),
    row(
        r"\u{66}()+g",
        &[esc(0, 6, r"\u{66}", "f"), dir(9, 10, "g")],
        1,
        10,
    ),
    row(
        r"f\u006Fo()+g",
        &[esc(0, 8, r"f\u006Fo", "foo"), dir(11, 12, "g")],
        1,
        12,
    ),
    row(
        r"\u{1D49C}()+g",
        &[esc(0, 9, r"\u{1D49C}", "\u{1D49C}"), dir(12, 13, "g")],
        1,
        13,
    ),
    row(
        r"\u{0066}()-g",
        &[esc(0, 8, r"\u{0066}", "f"), dir(11, 12, "g")],
        1,
        12,
    ),
    row(
        r"f\u0031()+g",
        &[esc(0, 7, r"f\u0031", "f1"), dir(10, 11, "g")],
        1,
        11,
    ),
];

const ESCAPED_TAIL_ROWS: &[ExprRow] = &[
    row(
        r"f()+\u0067",
        &[dir(0, 1, "f"), esc(4, 10, r"\u0067", "g")],
        1,
        10,
    ),
    row(
        r"f()+\u{67}",
        &[dir(0, 1, "f"), esc(4, 10, r"\u{67}", "g")],
        1,
        10,
    ),
    row(
        r"f()+g\u0031",
        &[dir(0, 1, "f"), esc(4, 11, r"g\u0031", "g1")],
        1,
        11,
    ),
    row(
        r"\u0066()+\u0067",
        &[esc(0, 6, r"\u0066", "f"), esc(9, 15, r"\u0067", "g")],
        1,
        15,
    ),
    row(
        r"f()-g+\u0068",
        &[dir(0, 1, "f"), dir(4, 5, "g"), esc(6, 12, r"\u0068", "h")],
        2,
        12,
    ),
    row(
        r"f()+\u{1D49C}",
        &[dir(0, 1, "f"), esc(4, 13, r"\u{1D49C}", "\u{1D49C}")],
        1,
        13,
    ),
];

/// Longer chains, only through the owner's already-accepted continuation. No
/// new maximum is claimed: the last row simply shows that a call head may
/// enter a chain longer than the earlier rows.
const LONGER_ROWS: &[ExprRow] = &[
    row(
        "f()-g+h",
        &[dir(0, 1, "f"), dir(4, 5, "g"), dir(6, 7, "h")],
        2,
        7,
    ),
    row(
        "f()+g+h-i",
        &[
            dir(0, 1, "f"),
            dir(4, 5, "g"),
            dir(6, 7, "h"),
            dir(8, 9, "i"),
        ],
        3,
        9,
    ),
    row(
        "f()+g+h+i+j",
        &[
            dir(0, 1, "f"),
            dir(4, 5, "g"),
            dir(6, 7, "h"),
            dir(8, 9, "i"),
            dir(10, 11, "j"),
        ],
        4,
        11,
    ),
    row(
        "f()+g-h+i-j+k-l",
        &[
            dir(0, 1, "f"),
            dir(4, 5, "g"),
            dir(6, 7, "h"),
            dir(8, 9, "i"),
            dir(10, 11, "j"),
            dir(12, 13, "k"),
            dir(14, 15, "l"),
        ],
        6,
        15,
    ),
];

/// Decimal operands contribute a link but never a fact.
const DECIMAL_ROWS: &[ExprRow] = &[
    row("f()+1", &[dir(0, 1, "f")], 1, 5),
    row("f()-1", &[dir(0, 1, "f")], 1, 5),
    row("f()+0", &[dir(0, 1, "f")], 1, 5),
    row("f()+1.5", &[dir(0, 1, "f")], 1, 7),
    row("f()+.5", &[dir(0, 1, "f")], 1, 6),
    row("f()+1e3", &[dir(0, 1, "f")], 1, 7),
    row("f()+1.", &[dir(0, 1, "f")], 1, 6),
    row("f()+1.5e-3", &[dir(0, 1, "f")], 1, 10),
    row("f()+1+2", &[dir(0, 1, "f")], 2, 7),
    row("f()+g+1", &[dir(0, 1, "f"), dir(4, 5, "g")], 2, 7),
    row("f()+1+g", &[dir(0, 1, "f"), dir(6, 7, "g")], 2, 7),
    row(
        "f()+1-g+2+h",
        &[dir(0, 1, "f"), dir(6, 7, "g"), dir(10, 11, "h")],
        4,
        11,
    ),
    row(
        "f()+g+1+h",
        &[dir(0, 1, "f"), dir(4, 5, "g"), dir(8, 9, "h")],
        3,
        9,
    ),
    row("f()+1+-g", &[dir(0, 1, "f"), dir(7, 8, "g")], 2, 8),
    row("f()+1+ +g", &[dir(0, 1, "f"), dir(8, 9, "g")], 2, 9),
];

/// Exactly-one right-unary `+` / `-` IdentifierReference operands, only in the
/// forms the existing owners already accept (opposite signs may be adjacent;
/// equal signs need selected trivia between them).
const UNARY_ROWS: &[ExprRow] = &[
    row("f()+-g", &[dir(0, 1, "f"), dir(5, 6, "g")], 1, 6),
    row("f()-+g", &[dir(0, 1, "f"), dir(5, 6, "g")], 1, 6),
    row("f()+ +g", &[dir(0, 1, "f"), dir(6, 7, "g")], 1, 7),
    row("f()- -g", &[dir(0, 1, "f"), dir(6, 7, "g")], 1, 7),
    row("f()+- g", &[dir(0, 1, "f"), dir(6, 7, "g")], 1, 7),
    row(
        "f()+g+-h",
        &[dir(0, 1, "f"), dir(4, 5, "g"), dir(7, 8, "h")],
        2,
        8,
    ),
    row(
        "f()+g-+h",
        &[dir(0, 1, "f"), dir(4, 5, "g"), dir(7, 8, "h")],
        2,
        8,
    ),
    row(
        "f()+g+ +h",
        &[dir(0, 1, "f"), dir(4, 5, "g"), dir(8, 9, "h")],
        2,
        9,
    ),
    row(
        "f()+g- -h",
        &[dir(0, 1, "f"), dir(4, 5, "g"), dir(8, 9, "h")],
        2,
        9,
    ),
    row(
        "f()+-g+-h+-i",
        &[
            dir(0, 1, "f"),
            dir(5, 6, "g"),
            dir(8, 9, "h"),
            dir(11, 12, "i"),
        ],
        3,
        12,
    ),
    row(
        r"f()+-\u0067",
        &[dir(0, 1, "f"), esc(5, 11, r"\u0067", "g")],
        1,
        11,
    ),
];

/// Selected trivia around the callee and `(`, inside `()`, around binary
/// `+` / `-`, and before the tail operand, including the load-bearing
/// LineTerminator placements.
const TRIVIA_ROWS: &[ExprRow] = &[
    row("f ()+g", &[dir(0, 1, "f"), dir(5, 6, "g")], 1, 6),
    row("f( )+g", &[dir(0, 1, "f"), dir(5, 6, "g")], 1, 6),
    row("f\n()+g", &[dir(0, 1, "f"), dir(5, 6, "g")], 1, 6),
    row("f()\n+g", &[dir(0, 1, "f"), dir(5, 6, "g")], 1, 6),
    row("f()+\ng", &[dir(0, 1, "f"), dir(5, 6, "g")], 1, 6),
    row("f() + g", &[dir(0, 1, "f"), dir(6, 7, "g")], 1, 7),
    row("f ( ) + g", &[dir(0, 1, "f"), dir(8, 9, "g")], 1, 9),
    row("f()\n+\ng", &[dir(0, 1, "f"), dir(6, 7, "g")], 1, 7),
    row(
        "f()+g\n+h",
        &[dir(0, 1, "f"), dir(4, 5, "g"), dir(7, 8, "h")],
        2,
        8,
    ),
    row(
        "f()+g +\nh",
        &[dir(0, 1, "f"), dir(4, 5, "g"), dir(8, 9, "h")],
        2,
        9,
    ),
    row("f()\n-\n1", &[dir(0, 1, "f")], 1, 7),
    row("f()+g\n+1", &[dir(0, 1, "f"), dir(4, 5, "g")], 2, 8),
    row("f\n(\n)\n+\ng", &[dir(0, 1, "f"), dir(8, 9, "g")], 1, 9),
    row("f\u{00A0}()+g", &[dir(0, 1, "f"), dir(6, 7, "g")], 1, 7),
    row(
        "f() \t\u{2028}\u{00A0}+\u{FEFF} \r\ng",
        &[dir(0, 1, "f"), dir(17, 18, "g")],
        1,
        18,
    ),
];

/// Callee names with special surrounding grammar or runtime lore. Each is a
/// plain `IdentifierReference` here: source syntax only. `async()+g` is the
/// AsyncArrow cover neighbor whose `+` continuation is never `=>`.
const SPECIAL_HEAD_ROWS: &[ExprRow] = &[
    row("eval()+g", &[dir(0, 4, "eval"), dir(7, 8, "g")], 1, 8),
    row(
        "arguments()+g",
        &[dir(0, 9, "arguments"), dir(12, 13, "g")],
        1,
        13,
    ),
    row("async()+g", &[dir(0, 5, "async"), dir(8, 9, "g")], 1, 9),
    row("async\n()+g", &[dir(0, 5, "async"), dir(9, 10, "g")], 1, 10),
    row("yield()+g", &[dir(0, 5, "yield"), dir(8, 9, "g")], 1, 9),
    row("await()+g", &[dir(0, 5, "await"), dir(8, 9, "g")], 1, 9),
];

fn all_positive_rows() -> impl Iterator<Item = &'static ExprRow> {
    BASIC_ROWS
        .iter()
        .chain(ESCAPED_HEAD_ROWS)
        .chain(ESCAPED_TAIL_ROWS)
        .chain(LONGER_ROWS)
        .chain(DECIMAL_ROWS)
        .chain(UNARY_ROWS)
        .chain(TRIVIA_ROWS)
        .chain(SPECIAL_HEAD_ROWS)
}

fn assert_fixture_is_self_consistent(row: &ExprRow) {
    assert!(!row.facts.is_empty(), "{:?}", row.expr);
    assert_eq!(row.facts[0].range.0, 0, "{:?}", row.expr);
    assert!(row.links >= 1, "{:?}", row.expr);
    assert!(row.facts.len() <= 1 + row.links, "{:?}", row.expr);

    let mut previous_end = 0;
    for fact in row.facts {
        assert_eq!(slice(row.expr, fact.range), fact.fragment, "{:?}", row.expr);
        assert!(fact.range.0 >= previous_end, "{:?}", row.expr);
        previous_end = fact.range.1;
        // Editing transport must not have decoded authored escapes away.
        if fact.provenance == ESCAPED {
            assert!(fact.fragment.contains('\\'), "{:?}", row.expr);
            assert_ne!(fact.fragment, fact.name, "{:?}", row.expr);
        } else {
            assert!(!fact.fragment.contains('\\'), "{:?}", row.expr);
            assert_eq!(fact.fragment, fact.name, "{:?}", row.expr);
        }
        // Bare-atom authority: every fact fragment is exactly one accepted
        // IdentifierReference.
        assert_eq!(
            classify_selected_accepted_identifier_reference(fact.fragment),
            Some((fact.name.to_owned(), fact.provenance)),
            "{:?}",
            row.expr
        );
    }
    assert!(row.end >= previous_end, "{:?}", row.expr);
    assert!(row.end <= row.expr.len(), "{:?}", row.expr);
}

/// Asserts the whole-source result of one positive expression in every
/// placement and termination, against fixture-owned expectations only.
fn assert_positive_row(id_base: u64, row: &ExprRow) {
    assert_fixture_is_self_consistent(row);

    for (placement_index, spec) in PLACEMENTS.iter().enumerate() {
        assert_eq!(spec.open.len(), spec.base, "{:?}", spec.placement);
        for (closer_index, (close, terminator)) in closers(spec).into_iter().enumerate() {
            let source = format!("{}{}{}", spec.open, row.expr, close);
            let id = id_base + (placement_index * 2 + closer_index) as u64;

            let SourceRecognition::Matched(evidence) = recognize_source(spec.placement, &source)
            else {
                panic!(
                    "{source:?} unexpectedly not selected in {:?}",
                    spec.placement
                );
            };
            assert_eq!(evidence.terminator, terminator, "{source:?}");
            assert_eq!(evidence.links, row.links, "{source:?}");
            let lead = match spec.owner {
                Owner::Initializer => 0,
                Owner::FreeStanding => closer_lead(close),
            };
            assert_eq!(
                evidence.expression_end,
                spec.base + row.end + lead,
                "{source:?}"
            );
            assert_eq!(
                classify_source(spec.placement, &source),
                FrontierOutcome::Matched,
                "{source:?}"
            );

            // Retained facts: exactly the expected list, in authored order.
            assert_eq!(evidence.facts.len(), row.facts.len(), "{source:?}");
            for (actual, expected) in evidence.facts.iter().zip(row.facts) {
                let expected_range = shift(expected.range, spec.base);
                assert_eq!(actual.range, expected_range, "{source:?}");
                assert_eq!(actual.name, expected.name, "{source:?}");
                assert_eq!(actual.provenance, expected.provenance, "{source:?}");
                // Exact authored anchor over the whole carrier source.
                assert_eq!(
                    authored_anchor(id, &source, expected_range),
                    expected.fragment,
                    "{source:?}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// C. Firewall fixtures (owner-specific expectations)
// ---------------------------------------------------------------------------

/// The expected owner-specific recognition of an outside/richer/incomplete
/// candidate, at the level *below* whole-source selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    /// A complete call-headed prefix that is never promoted to a whole-source
    /// result: the fixture-owned owner-specific `end` and the exact retained
    /// name order of the prefix.
    Prefix {
        end: usize,
        names: &'static [&'static str],
    },
    /// The call-only theorem retains ownership up to `end`.
    CallOnly { end: usize },
    /// Free-standing whole-body rollback.
    Rollback(RollbackReason),
    /// The accepted call head itself is not present.
    Head(DeclineReason),
}

const fn prefix(end: usize, names: &'static [&'static str]) -> Expect {
    Expect::Prefix { end, names }
}

const fn call_only(end: usize) -> Expect {
    Expect::CallOnly { end }
}

const OPERAND: Expect = Expect::Rollback(RollbackReason::OperandNotSelected);
const SAME_SIGN: Expect = Expect::Rollback(RollbackReason::SameSignAdjacency);
const FG: &[&str] = &["f", "g"];
const F_ONLY: &[&str] = &["f"];
const FGH: &[&str] = &["f", "g", "h"];
const FGF: &[&str] = &["f", "g", "f"];
const ASYNC_G: &[&str] = &["async", "g"];

const fn head(reason: DeclineReason) -> Expect {
    Expect::Head(reason)
}

const NO_START: DeclineReason = DeclineReason::Callee(CalleeFailure::NoIdentifierStart);
const RESERVED: DeclineReason = DeclineReason::Callee(CalleeFailure::DirectReservedWord);
const NO_OPENER: DeclineReason = DeclineReason::MissingCallOpener;
const UNOWNED_ARGS: DeclineReason = DeclineReason::UnownedInsideArguments;
const fn decode(failure: DecodeFailure) -> DeclineReason {
    DeclineReason::Callee(CalleeFailure::Decode(failure))
}

struct OutsideRow<'a> {
    expr: &'a str,
    /// Initializer-owned expectation (staged/local recovery).
    init: Expect,
    /// Free-standing-owned expectation (whole-body transaction).
    free: Expect,
}

const fn both(expr: &'static str, expect: Expect) -> OutsideRow<'static> {
    OutsideRow {
        expr,
        init: expect,
        free: expect,
    }
}

const fn split(expr: &'static str, init: Expect, free: Expect) -> OutsideRow<'static> {
    OutsideRow { expr, init, free }
}

const CALL_ONLY_AND_CALL_CHAIN_ROWS: &[OutsideRow<'static>] = &[
    // The call-only theorem is not call-headed additive: at least one
    // additive link is required.
    both("f()", call_only(3)),
    both("f() ", call_only(3)),
    both("f()\n", call_only(3)),
    both("f ()", call_only(4)),
    both("f\n()", call_only(4)),
    // Nested call / MemberExpression / OptionalChain / template neighbors.
    both("f()()", call_only(3)),
    both("f()()()", call_only(3)),
    both("f().x", call_only(3)),
    both("f()[x]", call_only(3)),
    both("f()?.x", call_only(3)),
    both("f()`t`", call_only(3)),
    both("f()\n()", call_only(3)),
    both("f()()+g", call_only(3)),
    both("f().x+g", call_only(3)),
    both("f()[x]+g", call_only(3)),
    both("f()?.x+g", call_only(3)),
    both("f()=g", call_only(3)),
    both("f()=>x", call_only(3)),
];

const MULTIPLICATIVE_AND_RICHER_ROWS: &[OutsideRow<'static>] = &[
    // No multiplicative precedence layer: nothing follows the head as an
    // additive link.
    both("f()*g", call_only(3)),
    both("f()/g", call_only(3)),
    both("f()%g", call_only(3)),
    both("f()**g", call_only(3)),
    both("f()*g+h", call_only(3)),
    both("f()/g-h", call_only(3)),
    // A complete additive prefix followed by a richer operator is never
    // promoted: the placement owner finds the leftover.
    both("f()+g*h", prefix(5, FG)),
    both("f()+g/h", prefix(5, FG)),
    both("f()+g%h", prefix(5, FG)),
    both("f()+g**h", prefix(5, FG)),
    both("f()+g*h+i", prefix(5, FG)),
    both("f()-g/h-i", prefix(5, FG)),
    both("f()+1*g", prefix(5, F_ONLY)),
    both("f()+1/g", prefix(5, F_ONLY)),
    both("f()+1%g", prefix(5, F_ONLY)),
    both("f()+g?x:y", prefix(5, FG)),
    both("f()+g=h", prefix(5, FG)),
    both("f()+g==h", prefix(5, FG)),
    both("f()+g===h", prefix(5, FG)),
    both("f()+g!=h", prefix(5, FG)),
    both("f()+g<h", prefix(5, FG)),
    both("f()+g>h", prefix(5, FG)),
    both("f()+g<=h", prefix(5, FG)),
    both("f()+g&&h", prefix(5, FG)),
    both("f()+g||h", prefix(5, FG)),
    both("f()+g??h", prefix(5, FG)),
    both("f()+g&h", prefix(5, FG)),
    both("f()+g|h", prefix(5, FG)),
    both("f()+g^h", prefix(5, FG)),
    // The free-standing body also owns the selected trivia before the
    // leftover; the initializer stops at its last operand.
    split("f()+g in h", prefix(5, FG), prefix(6, FG)),
    split("f()+g instanceof h", prefix(5, FG), prefix(6, FG)),
    both("f()+g=>x", prefix(5, FG)),
    both("f()+g.x", prefix(5, FG)),
    both("f()+g[x]", prefix(5, FG)),
    both("f()+g?.x", prefix(5, FG)),
    both("f()+g`t`", prefix(5, FG)),
    // Compound assignment spellings are never additive links.
    split("f()+=g", call_only(3), OPERAND),
    split("f()-=g", call_only(3), OPERAND),
    split("f()+g+=h", prefix(5, FG), OPERAND),
    split("f()+g-=h", prefix(5, FG), OPERAND),
    split("f()+1+=h", prefix(5, F_ONLY), OPERAND),
];

/// A call is authorized only in head position: any call in the tail leaves a
/// leftover the placement owner rejects, and the prefix is never promoted.
const CALL_IN_TAIL_ROWS: &[OutsideRow<'static>] = &[
    both("f()+g()", prefix(5, FG)),
    both("f()+g()+h", prefix(5, FG)),
    both("f()+g(a)", prefix(5, FG)),
    both("f()+g+f()", prefix(7, FGF)),
    both("f()+g+f()+h", prefix(7, FGF)),
    both("f()+g+h()", prefix(7, FGH)),
    both("f()+1()", prefix(5, F_ONLY)),
    both("a+f()", head(NO_OPENER)),
    both("a+f()+g", head(NO_OPENER)),
    both("a\n+f()", head(NO_OPENER)),
    both("g+f()", head(NO_OPENER)),
];

const GROUPING_AND_HEAD_SHAPE_ROWS: &[OutsideRow<'static>] = &[
    both("(f())+g", head(NO_START)),
    both("(f)()+g", head(NO_START)),
    both("!f()+g", head(NO_START)),
    both("+f()+g", head(NO_START)),
    both("-f()+g", head(NO_START)),
    both("~f()+g", head(NO_START)),
    both("typeof f()+g", head(RESERVED)),
    both("void f()+g", head(RESERVED)),
    both("delete f()+g", head(RESERVED)),
    both("new f()+g", head(RESERVED)),
    both("obj.f()+g", head(NO_OPENER)),
    both("a?.()+g", head(NO_OPENER)),
    split("f()+(g)", call_only(3), OPERAND),
    split("f()+ (g)", call_only(3), OPERAND),
    split("f()+(g)+h", call_only(3), OPERAND),
    split("f()+g+(h)", prefix(5, FG), OPERAND),
    split("f()+(1)", call_only(3), OPERAND),
];

const ARGUMENT_ROWS: &[OutsideRow<'static>] = &[
    both("f(a)", head(UNOWNED_ARGS)),
    both("f(a)+g", head(UNOWNED_ARGS)),
    both("f(a,b)+g", head(UNOWNED_ARGS)),
    both("f(...a)+g", head(UNOWNED_ARGS)),
    both("f(a,)+g", head(UNOWNED_ARGS)),
    both("f( a )+g", head(UNOWNED_ARGS)),
    both("f(+g)+h", head(UNOWNED_ARGS)),
    both("f(()+g)", head(UNOWNED_ARGS)),
];

/// Comments remain outside the selected trivia contract everywhere.
const COMMENT_ROWS: &[OutsideRow<'static>] = &[
    both("f/*c*/()+g", head(NO_OPENER)),
    both("f//c\n()+g", head(NO_OPENER)),
    both("f(/*c*/)+g", head(UNOWNED_ARGS)),
    both("f( /*c*/ )+g", head(UNOWNED_ARGS)),
    both("f(//c\n)+g", head(UNOWNED_ARGS)),
    both("f()/*c*/+g", call_only(3)),
    both("f()//c\n+g", call_only(3)),
    both("f() /*c*/ +g", call_only(3)),
    split("f()+/*c*/g", call_only(3), OPERAND),
    split("f()+//c\ng", call_only(3), OPERAND),
    split("f()+ /*c*/ g", call_only(3), OPERAND),
    split("f()+g+/*c*/h", prefix(5, FG), OPERAND),
    both("f()+g/*c*/+h", prefix(5, FG)),
    both("f()+g//c\n+h", prefix(5, FG)),
    // The initializer stops at its last operand; the free-standing body also
    // owns the selected trivia before the comment.
    split("f()+g /*c*/", prefix(5, FG), prefix(6, FG)),
];

/// Incomplete continuations: the initializer recovers to the latest complete
/// prefix (the call-only head when no link is complete); the free-standing
/// body rolls back entirely. No call-headed-additive result escapes either.
const INCOMPLETE_ROWS: &[OutsideRow<'static>] = &[
    split("f()+", call_only(3), OPERAND),
    split("f()-", call_only(3), OPERAND),
    split("f() +", call_only(3), OPERAND),
    split("f()+ ", call_only(3), OPERAND),
    split("f()+\n", call_only(3), OPERAND),
    split("f()+g+", prefix(5, FG), OPERAND),
    split("f()+g -", prefix(5, FG), OPERAND),
    split("f()+g+\n", prefix(5, FG), OPERAND),
    split("f()-g+h+", prefix(7, FGH), OPERAND),
    split("f()+1+", prefix(5, F_ONLY), OPERAND),
    split("f()+1+g+", prefix(7, FG), OPERAND),
];

/// Same-sign adjacency, and the exactly-one right-unary limits.
const PUNCTUATOR_ROWS: &[OutsideRow<'static>] = &[
    split("f()++", call_only(3), SAME_SIGN),
    split("f()--", call_only(3), SAME_SIGN),
    split("f()++g", call_only(3), SAME_SIGN),
    split("f()--g", call_only(3), SAME_SIGN),
    split("f()+++g", call_only(3), SAME_SIGN),
    split("f()+g++", prefix(5, FG), SAME_SIGN),
    split("f()+g--", prefix(5, FG), SAME_SIGN),
    split("f()+g++h", prefix(5, FG), SAME_SIGN),
    split("f()-g--h", prefix(5, FG), SAME_SIGN),
    split("f()+g+++h", prefix(5, FG), SAME_SIGN),
    split("f()+1++g", prefix(5, F_ONLY), SAME_SIGN),
    split("f()+1--g", prefix(5, F_ONLY), SAME_SIGN),
    // Exactly one right-unary sign; never recursive, never before Decimal.
    split("f()+-+g", call_only(3), OPERAND),
    split("f()+ + +g", call_only(3), OPERAND),
    split("f()+ ++g", call_only(3), OPERAND),
    split("f()-+-g", call_only(3), OPERAND),
    split("f()+ +", call_only(3), OPERAND),
    split("f()+-", call_only(3), OPERAND),
    split("f()+-1", call_only(3), OPERAND),
    split("f()- -1", call_only(3), OPERAND),
    split("f()+g+-+h", prefix(5, FG), OPERAND),
    split("f()+g+-1", prefix(5, FG), OPERAND),
    split("f()+1+-1", prefix(5, F_ONLY), OPERAND),
    split("f()+1+ +1", prefix(5, F_ONLY), OPERAND),
];

/// The AsyncArrow / cover firewall. `async()` followed by `=>` is a cover
/// reinterpretation this theorem never owns; an additive `+` / `-` is never
/// the immediate `=>` continuation.
const ASYNC_ARROW_ROWS: &[OutsideRow<'static>] = &[
    both("async()=>x", call_only(7)),
    both("async() => x", call_only(7)),
    both("async ()=>x", call_only(8)),
    both("async\n()=>x", call_only(8)),
    both("async ( ) => x", call_only(9)),
    both("async(a)=>x", head(UNOWNED_ARGS)),
    both("async a=>x", head(NO_OPENER)),
    both("async()+g=>x", prefix(9, ASYNC_G)),
    both("async()-g=>x", prefix(9, ASYNC_G)),
    split("async() + g => x", prefix(11, ASYNC_G), prefix(12, ASYNC_G)),
];

/// IdentifierReference policy boundaries at the head: never re-labelled and
/// never reinterpreted by the additive tail.
const HEAD_BOUNDARY_ROWS: &[OutsideRow<'static>] = &[
    both(r"\u0069f()+g", head(decode(DecodeFailure::DecodedReserved))),
    both(r"\u{69}f()+g", head(decode(DecodeFailure::DecodedReserved))),
    both(
        r"\u0074his()+g",
        head(decode(DecodeFailure::DecodedReserved)),
    ),
    both(r"\u006()+g", head(decode(DecodeFailure::MalformedEscape))),
    both(r"\u0G61()+g", head(decode(DecodeFailure::MalformedEscape))),
    both(r"\u{}()+g", head(decode(DecodeFailure::MalformedEscape))),
    both(r"\u{G}()+g", head(decode(DecodeFailure::MalformedEscape))),
    both(r"\x61()+g", head(decode(DecodeFailure::MalformedEscape))),
    both(r"\u{110000}()+g", head(decode(DecodeFailure::NonCodePoint))),
    both(r"\u0030()+g", head(decode(DecodeFailure::InvalidStart))),
    both("1a()+g", head(decode(DecodeFailure::InvalidStart))),
    both(r"a\u0020()+g", head(decode(DecodeFailure::InvalidPart))),
    both("if()+g", head(RESERVED)),
    both("this()+g", head(RESERVED)),
    both("true()+g", head(RESERVED)),
    both("null()+g", head(RESERVED)),
    both("typeof()+g", head(RESERVED)),
    both("void()+g", head(RESERVED)),
    both("delete()+g", head(RESERVED)),
    both("()+g", head(NO_START)),
    both(".a()+g", head(NO_START)),
    both("a b()+g", head(NO_OPENER)),
    both("a\u{00A0}b()+g", head(NO_OPENER)),
    both("a.b()+g", head(NO_OPENER)),
];

fn all_outside_row_groups() -> [&'static [OutsideRow<'static>]; 9] {
    [
        CALL_ONLY_AND_CALL_CHAIN_ROWS,
        MULTIPLICATIVE_AND_RICHER_ROWS,
        CALL_IN_TAIL_ROWS,
        GROUPING_AND_HEAD_SHAPE_ROWS,
        ARGUMENT_ROWS,
        COMMENT_ROWS,
        INCOMPLETE_ROWS,
        PUNCTUATOR_ROWS,
        ASYNC_ARROW_ROWS,
    ]
}

/// Tails that are not an accepted operand in the existing owner continuation:
/// unselected atoms, escaped/malformed/non-CodePoint/invalid IdentifierName
/// spellings (including a valid prefix that a bad escape makes non-maximal),
/// and legacy numeric spellings.
const UNSELECTED_OPERAND_TAILS: &[&str] = &[
    "@",
    "#x",
    "'s'",
    "\"s\"",
    "`t`",
    "true",
    "null",
    "this",
    "typeof",
    "new",
    "!g",
    "~g",
    "typeof g",
    "void g",
    "delete g",
    "*",
    "/",
    "=",
    ")",
    "}",
    "?",
    ":",
    ".",
    ".e3",
    "01",
    "08",
    "09.5",
    "0777",
    r"\u0074his",
    r"\u{74}his",
    r"\u0069f",
    r"\u{}",
    r"\u12",
    r"\u{G}",
    r"\x67",
    r"\u{110000}",
    r"\u0031",
    r"\u{31}",
    r"g\u{}",
    r"g\u0020",
    r"g\u{110000}",
    "\u{200B}g",
    "\u{0085}g",
    "\u{2060}g",
];

/// A complete Decimal operand followed by a spelling that is not an operator:
/// the Decimal is a complete link, the leftover is never absorbed, and the
/// placement owner rejects the whole source. `(tail, expression-relative end)`.
const DECIMAL_LEFTOVER_TAILS: &[(&str, usize)] = &[
    ("1a", 5),
    ("1n", 5),
    ("0x1F", 5),
    ("1_0", 5),
    ("0b1", 5),
    ("1e", 5),
    ("1e+", 5),
    ("1.a", 6),
    ("1.5e", 7),
    ("1.5.3", 7),
];

// ---------------------------------------------------------------------------
// D. Tests: authority, positives, owner-specific behavior, firewalls
// ---------------------------------------------------------------------------

/// Owner-neutral view of a scanner result, for comparison against the
/// fixture-owned [`Expect`] (names are owned so the comparison is total).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Observed {
    Prefix { end: usize, names: Vec<String> },
    CallOnly { end: usize },
    Rollback(RollbackReason),
    Head(DeclineReason),
}

fn names_of(prefix: &CallHeadedPrefix) -> Vec<String> {
    prefix.facts.iter().map(|fact| fact.name.clone()).collect()
}

fn observe_initializer(text: &str) -> Observed {
    match scan_initializer_call_headed(text) {
        InitializerRecognition::Matched(prefix) => Observed::Prefix {
            end: prefix.end,
            names: names_of(&prefix),
        },
        InitializerRecognition::CallOnlyRemains { call_only_end } => {
            Observed::CallOnly { end: call_only_end }
        }
        InitializerRecognition::Head(reason) => Observed::Head(reason),
    }
}

fn observe_free_standing(text: &str) -> Observed {
    match scan_free_standing_call_headed(text) {
        FreeStandingRecognition::Matched(prefix) => Observed::Prefix {
            end: prefix.end,
            names: names_of(&prefix),
        },
        FreeStandingRecognition::CallOnlyRemains { call_only_end } => {
            Observed::CallOnly { end: call_only_end }
        }
        FreeStandingRecognition::WholeBodyRollback(reason) => Observed::Rollback(reason),
        FreeStandingRecognition::Head(reason) => Observed::Head(reason),
    }
}

fn observe_owner(owner: Owner, text: &str) -> Observed {
    match owner {
        Owner::Initializer => observe_initializer(text),
        Owner::FreeStanding => observe_free_standing(text),
    }
}

fn observed_from(expect: Expect) -> Observed {
    match expect {
        Expect::Prefix { end, names } => Observed::Prefix {
            end,
            names: names.iter().map(|name| (*name).to_owned()).collect(),
        },
        Expect::CallOnly { end } => Observed::CallOnly { end },
        Expect::Rollback(reason) => Observed::Rollback(reason),
        Expect::Head(reason) => Observed::Head(reason),
    }
}

/// The whole-source reason an owner-specific expectation must produce once a
/// placement carrier of `base` bytes precedes the expression.
fn expected_reason(expect: Expect, base: usize) -> NotSelectedReason {
    match expect {
        Expect::Prefix { end, .. } => NotSelectedReason::LeftoverAfterPrefix {
            prefix_end: base + end,
        },
        Expect::CallOnly { end } => NotSelectedReason::CallOnlyRemains {
            call_only_end: base + end,
        },
        Expect::Rollback(reason) => NotSelectedReason::WholeBodyRollback(reason),
        Expect::Head(reason) => NotSelectedReason::Head(reason),
    }
}

fn expect_for(owner: Owner, row: &OutsideRow<'_>) -> Expect {
    match owner {
        Owner::Initializer => row.init,
        Owner::FreeStanding => row.free,
    }
}

/// Every outside row, in every placement and termination: the owner-specific
/// scanner result is exactly the fixture's, and the whole source is never
/// selected -- a locally complete prefix or call head is never promoted.
fn assert_outside_rows(rows: &[OutsideRow<'_>]) {
    for row in rows {
        for spec in PLACEMENTS {
            let expect = expect_for(spec.owner, row);
            for (close, _) in closers(spec) {
                let text = format!("{}{}", row.expr, close);
                assert_eq!(
                    observe_owner(spec.owner, &text),
                    observed_from(expect),
                    "{:?} in {:?}",
                    text,
                    spec.placement
                );

                let source = format!("{}{}", spec.open, text);
                assert_eq!(
                    recognize_source(spec.placement, &source),
                    SourceRecognition::NotSelected(expected_reason(expect, spec.base)),
                    "{source:?}"
                );
                assert_eq!(
                    classify_source(spec.placement, &source),
                    FrontierOutcome::NotSelected,
                    "{source:?}"
                );
            }
        }
    }
}

const ISSUE_ID: u64 = 853;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 853);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_CALL_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 849"));
    assert!(PREVIOUS_TWO_REFERENCE_INITIALIZER_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 752"));
    assert!(PREVIOUS_TWO_REFERENCE_FREE_STANDING_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 764"));
    assert!(PREVIOUS_ONE_REFERENCE_ONE_DECIMAL_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 789"));
    assert!(PREVIOUS_THREE_REFERENCE_INITIALIZER_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 795"));
    assert!(PREVIOUS_RIGHT_UNARY_INITIALIZER_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 777"));
    assert!(
        FRONTIER_SCOPE_NOTE.contains("call-headed existing additive continuation frontier only")
    );
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));
}

/// Placement table sanity: exactly the six existing owner placements (four
/// initializer-owned, two free-standing), no additional whole-source owner,
/// and fixture-owned carrier constants that match their literal carrier text.
#[test]
fn placements_are_exactly_the_existing_owner_placements() {
    assert_eq!(PLACEMENTS.len(), 6);
    let initializer = PLACEMENTS
        .iter()
        .filter(|spec| spec.owner == Owner::Initializer)
        .count();
    let free_standing = PLACEMENTS
        .iter()
        .filter(|spec| spec.owner == Owner::FreeStanding)
        .count();
    assert_eq!((initializer, free_standing), (4, 2));
    for spec in PLACEMENTS {
        assert_eq!(spec.open.len(), spec.base, "{:?}", spec.placement);
        assert_eq!(
            spec_of(spec.placement).open,
            spec.open,
            "{:?}",
            spec.placement
        );
        assert_eq!(
            spec.open.starts_with('{'),
            spec.block,
            "{:?}",
            spec.placement
        );
    }
}

#[test]
fn direct_head_with_reference_tail_matrix() {
    assert_eq!(BASIC_ROWS.len(), 7);
    for (index, row) in BASIC_ROWS.iter().enumerate() {
        assert_positive_row(853_000 + 20 * index as u64, row);
    }
}

#[test]
fn escaped_head_matrix_pins_authored_spelling_range_and_provenance() {
    assert_eq!(ESCAPED_HEAD_ROWS.len(), 6);
    for (index, row) in ESCAPED_HEAD_ROWS.iter().enumerate() {
        assert_positive_row(853_200 + 20 * index as u64, row);
        assert_eq!(row.facts[0].provenance, ESCAPED, "{:?}", row.expr);
        assert_eq!(row.facts[1].provenance, DIRECT, "{:?}", row.expr);
    }
}

#[test]
fn escaped_tail_matrix_pins_authored_spelling_range_and_provenance() {
    assert_eq!(ESCAPED_TAIL_ROWS.len(), 6);
    for (index, row) in ESCAPED_TAIL_ROWS.iter().enumerate() {
        assert_positive_row(853_400 + 20 * index as u64, row);
        assert!(
            row.facts
                .iter()
                .skip(1)
                .any(|fact| fact.provenance == ESCAPED),
            "{:?}",
            row.expr
        );
    }
}

/// Longer chains enter only the owner's existing continuation: the callee is
/// first, every later reference is appended once, in authored order.
#[test]
fn existing_longer_chains_retain_the_callee_first_then_authored_order() {
    assert_eq!(LONGER_ROWS.len(), 4);
    for (index, row) in LONGER_ROWS.iter().enumerate() {
        assert_positive_row(853_600 + 20 * index as u64, row);
        assert_eq!(row.facts.len(), 1 + row.links, "{:?}", row.expr);
    }
}

/// Decimal operands contribute no IdentifierReference fact and no identity.
#[test]
fn decimal_operands_contribute_links_but_never_facts() {
    assert_eq!(DECIMAL_ROWS.len(), 15);
    for (index, row) in DECIMAL_ROWS.iter().enumerate() {
        assert_positive_row(853_700 + 20 * index as u64, row);
    }
    // `f()+1` retains the callee alone.
    let source = "const x = f()+1;";
    let SourceRecognition::Matched(evidence) =
        recognize_source(Placement::LexicalInitializer, source)
    else {
        panic!("{source:?}");
    };
    assert_eq!(evidence.facts.len(), 1);
    assert_eq!(evidence.facts[0].range, Range(10, 11));
    assert_eq!(evidence.facts[0].name, "f");
    assert_eq!(evidence.links, 1);

    // No fact ever carries a numeric spelling.
    for row in DECIMAL_ROWS {
        for fact in row.facts {
            assert!(
                !fact
                    .fragment
                    .starts_with(|c: char| c.is_ascii_digit() || c == '.'),
                "{:?}",
                row.expr
            );
        }
    }
}

#[test]
fn exactly_one_right_unary_reference_tails_follow_existing_boundaries() {
    assert_eq!(UNARY_ROWS.len(), 11);
    for (index, row) in UNARY_ROWS.iter().enumerate() {
        assert_positive_row(853_1000 + 20 * index as u64, row);
        // The unary sign is recognition-time control only: no operator or
        // sign character is ever inside a retained fragment.
        for fact in row.facts {
            assert!(!fact.fragment.contains(['+', '-']), "{:?}", row.expr);
        }
    }
}

#[test]
fn selected_trivia_matrix_keeps_call_continuation_and_additive_boundaries() {
    assert_eq!(TRIVIA_ROWS.len(), 15);
    for (index, row) in TRIVIA_ROWS.iter().enumerate() {
        assert_positive_row(853_1300 + 20 * index as u64, row);
    }

    // The LineTerminator between callee and `(` is one call head, never an
    // ASI split, and composes with the additive continuation.
    let source = "f\n()+g";
    let SourceRecognition::Matched(evidence) =
        recognize_source(Placement::TopLevelFreeStanding, source)
    else {
        panic!("{source:?}");
    };
    assert_eq!(evidence.facts.len(), 2);
    assert_eq!(evidence.facts[0].range, Range(0, 1));
    assert_eq!(evidence.facts[1].range, Range(5, 6));
}

#[test]
fn special_head_names_are_plain_identifier_references() {
    assert_eq!(SPECIAL_HEAD_ROWS.len(), 6);
    for (index, row) in SPECIAL_HEAD_ROWS.iter().enumerate() {
        assert_positive_row(853_1700 + 20 * index as u64, row);
    }
}

/// Asserts a dynamically composed positive expression (facts are authored by
/// piecewise construction, never located by search): all direct, one link
/// count, one expression end.
fn assert_positive_dynamic(
    id_base: u64,
    expr: &str,
    facts: &[(Range, &str)],
    links: usize,
    end: usize,
) {
    for (placement_index, spec) in PLACEMENTS.iter().enumerate() {
        for (closer_index, (close, terminator)) in closers(spec).into_iter().enumerate() {
            let source = format!("{}{}{}", spec.open, expr, close);
            let id = id_base + (placement_index * 2 + closer_index) as u64;
            let SourceRecognition::Matched(evidence) = recognize_source(spec.placement, &source)
            else {
                panic!(
                    "{source:?} unexpectedly not selected in {:?}",
                    spec.placement
                );
            };
            assert_eq!(evidence.terminator, terminator, "{source:?}");
            assert_eq!(evidence.links, links, "{source:?}");
            let lead = match spec.owner {
                Owner::Initializer => 0,
                Owner::FreeStanding => closer_lead(close),
            };
            assert_eq!(
                evidence.expression_end,
                spec.base + end + lead,
                "{source:?}"
            );
            assert_eq!(evidence.facts.len(), facts.len(), "{source:?}");
            for (actual, (range, fragment)) in evidence.facts.iter().zip(facts) {
                let expected_range = shift(*range, spec.base);
                assert_eq!(actual.range, expected_range, "{source:?}");
                assert_eq!(actual.name, *fragment, "{source:?}");
                assert_eq!(actual.provenance, DIRECT, "{source:?}");
                assert_eq!(
                    authored_anchor(id, &source, expected_range),
                    *fragment,
                    "{source:?}"
                );
            }
        }
    }
}

/// One expression per trivia position, built piece by piece so the expected
/// fact ranges are authored from known piece widths.
/// `(expression, expected facts, link count, expression end)`.
type TriviaCase = (String, Vec<(Range, &'static str)>, usize, usize);

fn trivia_position_cases(run: &str, n: usize) -> Vec<TriviaCase> {
    let f = (Range(0, 1), "f");
    vec![
        // between callee and `(`
        (
            format!("f{run}()+g"),
            vec![f, (Range(n + 4, n + 5), "g")],
            1,
            n + 5,
        ),
        // inside the empty Arguments
        (
            format!("f({run})+g"),
            vec![f, (Range(n + 4, n + 5), "g")],
            1,
            n + 5,
        ),
        // before the binary operator
        (
            format!("f(){run}+g"),
            vec![f, (Range(n + 4, n + 5), "g")],
            1,
            n + 5,
        ),
        // after the binary operator / before the tail operand
        (
            format!("f()+{run}g"),
            vec![f, (Range(n + 4, n + 5), "g")],
            1,
            n + 5,
        ),
        // before a later operator
        (
            format!("f()+g{run}+h"),
            vec![f, (Range(4, 5), "g"), (Range(n + 6, n + 7), "h")],
            2,
            n + 7,
        ),
        // after a later operator
        (
            format!("f()+g+{run}h"),
            vec![f, (Range(4, 5), "g"), (Range(n + 6, n + 7), "h")],
            2,
            n + 7,
        ),
        // after a right-unary sign
        (
            format!("f()+-{run}g"),
            vec![f, (Range(n + 5, n + 6), "g")],
            1,
            n + 6,
        ),
        // between binary and equal-sign unary (the trivia is what permits it)
        (
            format!("f()+{run}+g"),
            vec![f, (Range(n + 5, n + 6), "g")],
            1,
            n + 6,
        ),
        // between a Decimal operand and the next operator
        (
            format!("f()+1{run}+g"),
            vec![f, (Range(n + 6, n + 7), "g")],
            2,
            n + 7,
        ),
    ]
}

/// Full selected-trivia sweep at every trivia position, including every
/// LineTerminator code point, plus the separator look-alikes the frozen
/// contract does not own.
#[test]
fn selected_trivia_sweep_and_non_trivia_lookalikes() {
    for (trivia_index, trivia) in SELECTED_TRIVIA_SWEEP.iter().enumerate() {
        assert!(is_selected_call_trivia(*trivia), "{trivia:?}");
        let width = trivia.len_utf8();
        for count in [1_usize, 2] {
            let run = trivia.to_string().repeat(count);
            let n = width * count;
            for (case_index, (expr, facts, links, end)) in
                trivia_position_cases(&run, n).into_iter().enumerate()
            {
                let id = 853_2000 + (trivia_index * 200 + count * 100 + case_index * 10) as u64;
                assert_positive_dynamic(id, &expr, &facts, links, end);
            }
        }
    }

    for lookalike in NON_TRIVIA_SEPARATOR_LOOKALIKES {
        assert!(!is_selected_call_trivia(*lookalike), "{lookalike:?}");
        assert!(!is_direct_identifier_part(*lookalike), "{lookalike:?}");
        let run = lookalike.to_string();
        for (expr, _, _, _) in trivia_position_cases(&run, lookalike.len_utf8()) {
            for spec in PLACEMENTS {
                for (close, _) in closers(spec) {
                    let source = format!("{}{}{}", spec.open, expr, close);
                    assert_eq!(
                        classify_source(spec.placement, &source),
                        FrontierOutcome::NotSelected,
                        "{source:?}"
                    );
                }
            }
        }
    }
}

/// The trivia after the last operand: the initializer stops at the operand
/// and lets the declaration owner skip it; the free-standing body owns it.
/// Both are terminated only by the placement.
#[test]
fn trailing_trivia_ownership_differs_by_owner_and_termination_stays_with_placement() {
    for (expr, facts, links, operand_end, trivia_end) in [
        ("f()+g \t", 2_usize, 1_usize, 5_usize, 7_usize),
        ("f()+1 \n", 1, 1, 5, 7),
        ("f()+g\u{00A0}", 2, 1, 5, 7),
        ("f()+g+h ", 3, 2, 7, 8),
    ] {
        for spec in PLACEMENTS {
            for (close, terminator) in closers(spec) {
                let source = format!("{}{}{}", spec.open, expr, close);
                let SourceRecognition::Matched(evidence) =
                    recognize_source(spec.placement, &source)
                else {
                    panic!("{source:?}");
                };
                let expected_end = match spec.owner {
                    Owner::Initializer => operand_end,
                    Owner::FreeStanding => trivia_end + closer_lead(close),
                };
                assert_eq!(
                    evidence.expression_end,
                    spec.base + expected_end,
                    "{source:?}"
                );
                assert_eq!(evidence.facts.len(), facts, "{source:?}");
                assert_eq!(evidence.links, links, "{source:?}");
                assert_eq!(evidence.terminator, terminator, "{source:?}");
            }
        }
    }
}

#[test]
fn call_only_control_is_not_call_headed_additive() {
    for spec in PLACEMENTS {
        for (close, _) in closers(spec) {
            let source = format!("{}f(){}", spec.open, close);
            assert_eq!(
                recognize_source(spec.placement, &source),
                SourceRecognition::NotSelected(NotSelectedReason::CallOnlyRemains {
                    call_only_end: spec.base + 3
                }),
                "{source:?}"
            );
        }
    }
    // The composition declines exactly at the accepted #849 call prefix, so
    // call-only ownership is left intact for every call-only row.
    for group in all_outside_row_groups() {
        for row in group {
            for expect in [row.init, row.free] {
                if let Expect::CallOnly { end } = expect {
                    let call = scan_zero_argument_call_prefix(row.expr)
                        .unwrap_or_else(|reason| panic!("{:?}: {reason:?}", row.expr));
                    assert_eq!(call.close.1, end, "{:?}", row.expr);
                }
            }
        }
    }
    // A call head with one-or-more links is what this theorem adds.
    assert!(all_positive_rows().all(|row| row.links >= 1));
}

#[test]
fn call_and_member_neighbors_remain_outside() {
    assert_outside_rows(CALL_ONLY_AND_CALL_CHAIN_ROWS);
}

#[test]
fn multiplicative_and_richer_expression_tails_remain_outside_without_promotion() {
    assert_outside_rows(MULTIPLICATIVE_AND_RICHER_ROWS);
    // A complete additive prefix followed by a richer operator is retained
    // nowhere: every whole-source result is NotSelected with no fact list.
    for row in MULTIPLICATIVE_AND_RICHER_ROWS {
        for spec in PLACEMENTS {
            for (close, _) in closers(spec) {
                let source = format!("{}{}{}", spec.open, row.expr, close);
                assert!(
                    matches!(
                        recognize_source(spec.placement, &source),
                        SourceRecognition::NotSelected(_)
                    ),
                    "{source:?}"
                );
            }
        }
    }
}

#[test]
fn a_call_is_authorized_only_in_head_position() {
    assert_outside_rows(CALL_IN_TAIL_ROWS);
}

#[test]
fn grouping_and_non_head_shapes_remain_outside() {
    assert_outside_rows(GROUPING_AND_HEAD_SHAPE_ROWS);
}

#[test]
fn non_empty_arguments_keep_the_head_zero_argument() {
    assert_outside_rows(ARGUMENT_ROWS);
}

#[test]
fn comments_are_not_selected_trivia_anywhere_in_the_composition() {
    assert_outside_rows(COMMENT_ROWS);
}

/// Incomplete continuations: owner-specific recovery vs rollback, and no
/// call-headed-additive result in either.
#[test]
fn incomplete_continuations_never_leak_a_call_headed_additive_result() {
    assert_outside_rows(INCOMPLETE_ROWS);
}

#[test]
fn same_sign_adjacency_and_right_unary_limits_follow_owner_rules() {
    assert_outside_rows(PUNCTUATOR_ROWS);
}

#[test]
fn identifier_reference_head_boundaries_are_not_reinterpreted() {
    assert_outside_rows(HEAD_BOUNDARY_ROWS);
    let distinct: std::collections::BTreeSet<String> = HEAD_BOUNDARY_ROWS
        .iter()
        .map(|row| format!("{:?}", row.init))
        .collect();
    assert!(distinct.len() >= 8, "{distinct:?}");
}

/// Later reference positions apply the same accepted policy: each unselected
/// tail leaves the initializer at its latest completed prefix and rolls the
/// free-standing body back entirely, at the first and at a later operand.
#[test]
fn unselected_tail_operands_decline_owner_specifically() {
    for tail in UNSELECTED_OPERAND_TAILS {
        let first = format!("f()+{tail}");
        let later = format!("f()+g+{tail}");
        let first_and_later = [
            (first.as_str(), call_only(3), OPERAND),
            (later.as_str(), prefix(5, FG), OPERAND),
        ];
        for (expr, init, free) in first_and_later {
            let row = OutsideRow { expr, init, free };
            assert_outside_rows(std::slice::from_ref(&row));
        }
    }
}

/// Maximal `IdentifierName` ownership: a valid prefix followed by a bad
/// escape is never split into an accepted shorter name.
#[test]
fn maximal_identifier_name_is_never_split_at_the_tail() {
    for tail in [r"g\u{}", r"g\u0020", r"g\u{110000}", r"g\u12"] {
        let expr = format!("f()+{tail}");
        assert_eq!(
            observe_initializer(&expr),
            Observed::CallOnly { end: 3 },
            "{expr:?}"
        );
        assert_eq!(
            observe_free_standing(&expr),
            Observed::Rollback(RollbackReason::OperandNotSelected),
            "{expr:?}"
        );
    }
    // The maximal name, not a shorter prefix, is what the tail retains.
    let evidence = match scan_initializer_call_headed("f()+ab") {
        InitializerRecognition::Matched(prefix) => prefix,
        other => panic!("{other:?}"),
    };
    assert_eq!(evidence.facts[1].range, Range(4, 6));
    assert_eq!(evidence.facts[1].name, "ab");
}

/// Plain Decimal followed by a non-operator spelling: the Decimal is one
/// complete link, the leftover is never absorbed, and no whole-source result
/// exists.
#[test]
fn decimal_followed_by_leftover_is_never_absorbed() {
    for (tail, end) in DECIMAL_LEFTOVER_TAILS {
        let expr = format!("f(){}{tail}", '+');
        let row = OutsideRow {
            expr: &expr,
            init: prefix(*end, F_ONLY),
            free: prefix(*end, F_ONLY),
        };
        assert_outside_rows(std::slice::from_ref(&row));
    }
}

/// The candidate ends at true end-of-text: an unfinished call head is never
/// a call head.
#[test]
fn unfinished_head_at_end_of_text_is_not_a_call_head() {
    for text in ["f(", "f (", "f( ", "f(\n", "f(\t "] {
        assert_eq!(
            observe_initializer(text),
            Observed::Head(DeclineReason::IncompleteSuffix),
            "{text:?}"
        );
        assert_eq!(
            observe_free_standing(text),
            Observed::Head(DeclineReason::IncompleteSuffix),
            "{text:?}"
        );
    }
    assert_eq!(
        observe_initializer("f"),
        Observed::Head(DeclineReason::MissingCallOpener)
    );
    assert_eq!(observe_free_standing(""), Observed::Head(NO_START));
}

/// The two ownership models differ observably, and neither is derived from
/// the other.
#[test]
fn initializer_recovery_and_free_standing_rollback_remain_observably_distinct() {
    // The initializer keeps the latest completed call-headed prefix; the
    // free-standing body keeps nothing.
    for (text, names, end) in [
        ("f()+g+", vec!["f", "g"], 5_usize),
        ("f()+1+", vec!["f"], 5),
        ("f()-g+h+", vec!["f", "g", "h"], 7),
        ("f()+g+-1", vec!["f", "g"], 5),
        ("f()+g++h", vec!["f", "g"], 5),
        ("f()+g+@", vec!["f", "g"], 5),
    ] {
        let InitializerRecognition::Matched(prefix) = scan_initializer_call_headed(text) else {
            panic!("{text:?}");
        };
        assert_eq!(
            prefix
                .facts
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            names,
            "{text:?}"
        );
        assert_eq!(prefix.end, end, "{text:?}");
        assert!(prefix.links >= 1, "{text:?}");

        let FreeStandingRecognition::WholeBodyRollback(_) = scan_free_standing_call_headed(text)
        else {
            panic!("{text:?} must roll back the whole body");
        };
    }

    // No complete link at all: the initializer leaves the call head to the
    // call-only theorem; the free-standing body still rolls back entirely
    // once an authored operator has started, but leaves call-only alone when
    // no operator follows.
    for text in ["f()+", "f()-", "f()+-", "f()+@"] {
        assert_eq!(
            scan_initializer_call_headed(text),
            InitializerRecognition::CallOnlyRemains { call_only_end: 3 },
            "{text:?}"
        );
        assert!(matches!(
            scan_free_standing_call_headed(text),
            FreeStandingRecognition::WholeBodyRollback(_)
        ));
    }
    for text in ["f()", "f() ", "f()*g", "f()()"] {
        assert!(matches!(
            scan_initializer_call_headed(text),
            InitializerRecognition::CallOnlyRemains { .. }
        ));
        assert!(matches!(
            scan_free_standing_call_headed(text),
            FreeStandingRecognition::CallOnlyRemains { .. }
        ));
    }

    // A complete chain is identical in both owners.
    for row in all_positive_rows() {
        let InitializerRecognition::Matched(init) = scan_initializer_call_headed(row.expr) else {
            panic!("{:?}", row.expr);
        };
        let FreeStandingRecognition::Matched(free) = scan_free_standing_call_headed(row.expr)
        else {
            panic!("{:?}", row.expr);
        };
        assert_eq!(init.facts, free.facts, "{:?}", row.expr);
        assert_eq!(init.links, free.links, "{:?}", row.expr);
        assert_eq!(init.head_end, free.head_end, "{:?}", row.expr);
    }
}

/// Placement owns the terminator: leftover after a complete prefix never
/// selects the source, and a carrier of another placement never matches.
#[test]
fn placement_termination_and_carrier_are_owned_by_the_placement() {
    for (placement, source, reason) in [
        (
            Placement::LexicalInitializer,
            "const x = f()+g h;",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 15 },
        ),
        (
            Placement::LexicalInitializer,
            "const x = f()+g }",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 15 },
        ),
        (
            Placement::TopLevelVarInitializer,
            "var x = f()+g;;",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 13 },
        ),
        (
            Placement::TopLevelFreeStanding,
            "f()+g h;",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 6 },
        ),
        (
            Placement::TopLevelFreeStanding,
            "f()+g }",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 6 },
        ),
        (
            Placement::BlockFreeStanding,
            "{ f()+g",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 7 },
        ),
        (
            Placement::BlockFreeStanding,
            "{ f()+g;",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 7 },
        ),
        (
            Placement::BlockVarInitializer,
            "{ var x = f()+g;",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 15 },
        ),
        (
            Placement::BlockLexicalInitializer,
            "{ let x = f()+g }",
            NotSelectedReason::LeftoverAfterPrefix { prefix_end: 15 },
        ),
        (
            Placement::LexicalInitializer,
            "var x = f()+g;",
            NotSelectedReason::CarrierMismatch,
        ),
        (
            Placement::BlockFreeStanding,
            "f()+g;",
            NotSelectedReason::CarrierMismatch,
        ),
    ] {
        assert_eq!(
            recognize_source(placement, source),
            SourceRecognition::NotSelected(reason),
            "{source:?}"
        );
    }
}

/// Async/cover local falsification: in every accepted source the token after
/// the call head (past selected trivia) is an additive `+` / `-`, never `=`,
/// and no accepted source contains an arrow or update spelling.
#[test]
fn additive_continuation_is_never_the_immediate_arrow_continuation() {
    for row in all_positive_rows() {
        let call = scan_zero_argument_call_prefix(row.expr)
            .unwrap_or_else(|reason| panic!("{:?}: {reason:?}", row.expr));
        let after = skip_call_trivia(row.expr, call.close.1);
        assert!(sign_at(row.expr, after).is_some(), "{:?}", row.expr);
        assert!(!row.expr.contains('='), "{:?}", row.expr);
        assert!(!row.expr.contains("=>"), "{:?}", row.expr);
        assert!(!row.expr.contains("++"), "{:?}", row.expr);
        assert!(!row.expr.contains("--"), "{:?}", row.expr);
    }

    // The cover neighbors stay outside, and `async()+g` alone stays a call
    // head with an additive tail.
    assert_outside_rows(ASYNC_ARROW_ROWS);
    assert!(
        SPECIAL_HEAD_ROWS
            .iter()
            .any(|row| row.expr == "async()+g" && row.facts[0].name == "async")
    );
}

/// Update, arrow, assignment, and conditional spellings adjacent to the
/// additive punctuators are never accepted in any placement.
#[test]
fn update_arrow_assignment_and_conditional_spellings_are_never_accepted() {
    for expr in [
        "f()++",
        "f()--",
        "f()+g++",
        "f()-g--",
        "f()++g",
        "f()--g",
        "f()=>g",
        "f()+g=>h",
        "async()+g=>x",
        "f()+g=h",
        "f()+=g",
        "f()-=g",
        "f()+g?x:y",
        "f()?g:h",
    ] {
        for spec in PLACEMENTS {
            for (close, _) in closers(spec) {
                let source = format!("{}{}{}", spec.open, expr, close);
                assert_eq!(
                    classify_source(spec.placement, &source),
                    FrontierOutcome::NotSelected,
                    "{source:?}"
                );
            }
        }
    }
}

/// Every accepted source retains reference facts only: no fragment contains an
/// operator, delimiter, or trivia code point, and the retained evidence type
/// has no call, arithmetic, or value field.
#[test]
fn retained_evidence_is_only_authored_reference_facts() {
    for row in all_positive_rows() {
        for fact in row.facts {
            assert!(
                fact.fragment
                    .chars()
                    .all(|c| is_direct_identifier_part(c) || matches!(c, '\\' | '{' | '}')),
                "{:?}",
                row.expr
            );
            assert!(
                !fact.fragment.starts_with(|c: char| c.is_ascii_digit()),
                "{:?}",
                row.expr
            );
            assert!(!fact.fragment.chars().any(is_selected_call_trivia));
        }
        // The callee is the first fact at the head of the expression.
        assert_eq!(row.facts[0].range.0, 0, "{:?}", row.expr);
    }

    let evidence = SourceEvidence {
        facts: Vec::new(),
        links: 0,
        expression_end: 0,
        terminator: Terminator::AutomaticAtEof,
    };
    // Exhaustive destructuring: adding a call/value field breaks this test.
    let SourceEvidence {
        facts: _,
        links: _,
        expression_end: _,
        terminator: _,
    } = evidence;
}

/// Failure separation: ordinary decline is `NotSelected`; processing failures
/// are distinct identities that this total, allocation-bounded Oracle never
/// produces and that must never be folded into ordinary non-selection.
///
/// Limitation (reported honestly): this test-only recognizer performs no
/// fallible allocation and takes no resource budget, so a `ResourceLimited`
/// / `InternalFailure` injection through the recognizer is not practical
/// here. The Oracle proves the separation of identities and that no ordinary
/// decline maps onto them; forwarding of a processing failure through the
/// composition is a production-implementation obligation.
#[test]
fn processing_failures_remain_distinct_from_ordinary_decline() {
    let processing = [
        FrontierOutcome::ResourceLimited,
        FrontierOutcome::InternalFailure,
    ];
    for failure in processing {
        assert_ne!(failure, FrontierOutcome::NotSelected);
        assert_ne!(failure, FrontierOutcome::Matched);
    }
    assert_ne!(processing[0], processing[1]);

    for group in all_outside_row_groups() {
        for row in group {
            for spec in PLACEMENTS {
                let source = format!("{}{}", spec.open, row.expr);
                let outcome = classify_source(spec.placement, &source);
                assert!(!processing.contains(&outcome), "{source:?}");
            }
        }
    }
    for row in all_positive_rows() {
        for spec in PLACEMENTS {
            for (close, _) in closers(spec) {
                let source = format!("{}{}{}", spec.open, row.expr, close);
                assert_eq!(
                    classify_source(spec.placement, &source),
                    FrontierOutcome::Matched
                );
            }
        }
    }
}

/// Every positive fixture is reached by exactly one matrix; this guards a
/// matrix being silently emptied or duplicated.
#[test]
fn positive_fixture_inventory_is_complete_and_unique() {
    let expressions: Vec<&str> = all_positive_rows().map(|row| row.expr).collect();
    let mut unique = expressions.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(expressions.len(), unique.len());
    assert_eq!(expressions.len(), 7 + 6 + 6 + 4 + 15 + 11 + 15 + 6);
    for required in [
        "f()+g",
        "f()-g",
        r"\u0066()+g",
        r"\u{66}()+g",
        r"f()+\u0067",
        "f()-g+h",
        "f()+1",
        "f()+-g",
        "f()-+g",
        "f()+ +g",
        "f\n()+g",
        "async()+g",
    ] {
        assert!(expressions.contains(&required), "{required:?}");
    }

    let outside: usize = all_outside_row_groups()
        .iter()
        .map(|group| group.len())
        .sum();
    assert_eq!(outside, 18 + 41 + 11 + 17 + 8 + 15 + 11 + 24 + 10);
    let mut outside_expressions: Vec<&str> = all_outside_row_groups()
        .iter()
        .flat_map(|group| group.iter().map(|row| row.expr))
        .collect();
    outside_expressions.extend(HEAD_BOUNDARY_ROWS.iter().map(|row| row.expr));
    let count = outside_expressions.len();
    outside_expressions.sort_unstable();
    outside_expressions.dedup();
    assert_eq!(count, outside_expressions.len());
}

// ---------------------------------------------------------------------------
// E. Early Error reachability model
// ---------------------------------------------------------------------------

/// Source syntax an Early Error identity's trigger requires, at the
/// granularity needed to decide whether an `IdentifierReference` call head
/// plus an existing additive continuation can supply it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyntaxElement {
    // Elements this theorem admits.
    CalleeIdentifierReference,
    EmptyArguments,
    SelectedTrivia,
    AdditiveBinaryOperator,
    RightUnaryPlusMinusOperator,
    PlainDecimalLiteral,
    TailIdentifierReference,
    // Elements it does not.
    BindingIdentifier,
    LegacyNumericLiteral,
    LegacyStringEscape,
    ObjectLiteral,
    RegExpLiteral,
    TemplateLiteral,
    ParenthesizedGrouping,
    OptionalChain,
    ImportMeta,
    UpdateOperator,
    DeleteOperator,
    AssignmentOperator,
    DestructuringPattern,
    ControlStatement,
    ParameterList,
    FunctionOrClassBoundary,
    ArrowToken,
    SuperKeyword,
    NewTargetMeta,
    PrivateName,
    LabelledStatement,
    BreakOrContinueStatement,
    RegExpPatternSyntax,
}

const ADMITTED_SYNTAX: &[SyntaxElement] = &[
    SyntaxElement::CalleeIdentifierReference,
    SyntaxElement::EmptyArguments,
    SyntaxElement::SelectedTrivia,
    SyntaxElement::AdditiveBinaryOperator,
    SyntaxElement::RightUnaryPlusMinusOperator,
    SyntaxElement::PlainDecimalLiteral,
    SyntaxElement::TailIdentifierReference,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reachability {
    /// The trigger is the callee atom itself, already owned by the accepted
    /// bare `IdentifierReference` authority whether or not a call follows.
    CalleeAtomAlreadyOwned,
    /// The trigger is carrier structure (declaration/statement lists) that is
    /// independent of the initializer expression.
    CarrierOwned,
    /// The trigger needs syntax outside the admitted set.
    RequiresAbsent(SyntaxElement),
}

/// Fixture-authored default reachability per frozen Early Error container.
const CONTAINER_REACHABILITY: &[(&str, Reachability)] = &[
    ("EE-01", Reachability::CalleeAtomAlreadyOwned),
    (
        "EE-02",
        Reachability::RequiresAbsent(SyntaxElement::LegacyNumericLiteral),
    ),
    (
        "EE-03",
        Reachability::RequiresAbsent(SyntaxElement::LegacyStringEscape),
    ),
    (
        "EE-04",
        Reachability::RequiresAbsent(SyntaxElement::BindingIdentifier),
    ),
    (
        "EE-05",
        Reachability::RequiresAbsent(SyntaxElement::ObjectLiteral),
    ),
    (
        "EE-06",
        Reachability::RequiresAbsent(SyntaxElement::RegExpLiteral),
    ),
    (
        "EE-07",
        Reachability::RequiresAbsent(SyntaxElement::TemplateLiteral),
    ),
    (
        "EE-08",
        Reachability::RequiresAbsent(SyntaxElement::ParenthesizedGrouping),
    ),
    (
        "EE-09",
        Reachability::RequiresAbsent(SyntaxElement::OptionalChain),
    ),
    (
        "EE-10",
        Reachability::RequiresAbsent(SyntaxElement::UpdateOperator),
    ),
    (
        "EE-11",
        Reachability::RequiresAbsent(SyntaxElement::DeleteOperator),
    ),
    (
        "EE-12",
        Reachability::RequiresAbsent(SyntaxElement::AssignmentOperator),
    ),
    (
        "EE-13",
        Reachability::RequiresAbsent(SyntaxElement::DestructuringPattern),
    ),
    ("EE-14", Reachability::CarrierOwned),
    ("EE-15", Reachability::CarrierOwned),
    (
        "EE-16",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-17",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-18",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-19",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-20",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-21",
        Reachability::RequiresAbsent(SyntaxElement::BreakOrContinueStatement),
    ),
    (
        "EE-22",
        Reachability::RequiresAbsent(SyntaxElement::BreakOrContinueStatement),
    ),
    (
        "EE-23",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-24",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-25",
        Reachability::RequiresAbsent(SyntaxElement::LabelledStatement),
    ),
    (
        "EE-26",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-27",
        Reachability::RequiresAbsent(SyntaxElement::ParameterList),
    ),
    (
        "EE-28",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-29",
        Reachability::RequiresAbsent(SyntaxElement::ArrowToken),
    ),
    (
        "EE-30",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-31",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-32",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-33",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-34",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-35",
        Reachability::RequiresAbsent(SyntaxElement::ArrowToken),
    ),
    ("EE-36", Reachability::CarrierOwned),
    (
        "EE-37",
        Reachability::RequiresAbsent(SyntaxElement::RegExpPatternSyntax),
    ),
];

/// Per-rule refinements of the container defaults.
const RULE_REACHABILITY_OVERRIDES: &[(&str, Reachability)] = &[
    // Callee-atom identities: identical trigger with or without a call.
    ("EE-04-R02", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-R05", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-R06", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-R07", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-R08", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-I01", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-I02", Reachability::CalleeAtomAlreadyOwned),
    // Same container, different trigger syntax.
    (
        "EE-09-R02",
        Reachability::RequiresAbsent(SyntaxElement::ImportMeta),
    ),
    (
        "EE-33-R10",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-36-R03",
        Reachability::RequiresAbsent(SyntaxElement::SuperKeyword),
    ),
    (
        "EE-36-R04",
        Reachability::RequiresAbsent(SyntaxElement::NewTargetMeta),
    ),
    (
        "EE-36-R05",
        Reachability::RequiresAbsent(SyntaxElement::LabelledStatement),
    ),
    (
        "EE-36-R06",
        Reachability::RequiresAbsent(SyntaxElement::BreakOrContinueStatement),
    ),
    (
        "EE-36-R07",
        Reachability::RequiresAbsent(SyntaxElement::BreakOrContinueStatement),
    ),
    (
        "EE-36-R08",
        Reachability::RequiresAbsent(SyntaxElement::PrivateName),
    ),
];

/// Locator vocabulary that marks a frozen identity as call/cover/super/async
/// neighborhood, independently of the reachability table above.
const CALL_NEIGHBORHOOD_VOCABULARY: &[&str] = &[
    "CallExpression",
    "SuperCall",
    "SuperProperty",
    "HasDirectSuper",
    "Contains super",
    "Contains NewTarget",
    "ContainsArguments",
    "AsyncArrow",
    "AssignmentTargetType",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct InventoryRule {
    id: String,
    container: String,
    active: bool,
    locator: String,
}

/// Reads the frozen inventory *source text* (never the production or
/// completion code) into rule identities.
fn parse_inventory_rules() -> Vec<InventoryRule> {
    let lines: Vec<&str> = INVENTORY_SOURCE.lines().map(str::trim).collect();
    let unquote = |line: &str| line.trim_end_matches(',').trim_matches('"').to_owned();
    let mut rules = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let active = match *line {
            "active_rule(" => true,
            "inactive_rule(" => false,
            _ => continue,
        };
        rules.push(InventoryRule {
            id: unquote(lines[index + 1]),
            container: unquote(lines[index + 2]),
            active,
            locator: unquote(lines[index + 3]),
        });
    }
    rules
}

fn reachability_of(rule: &InventoryRule) -> Reachability {
    if let Some(index) = RULE_REACHABILITY_OVERRIDES
        .iter()
        .position(|(id, _)| *id == rule.id)
    {
        return RULE_REACHABILITY_OVERRIDES[index].1;
    }
    let index = CONTAINER_REACHABILITY
        .iter()
        .position(|(container, _)| *container == rule.container)
        .unwrap_or_else(|| panic!("unaudited container for {rule:?}"));
    CONTAINER_REACHABILITY[index].1
}

fn newly_reachable(rules: &[InventoryRule], admitted: &[SyntaxElement]) -> Vec<String> {
    rules
        .iter()
        .filter(|rule| match reachability_of(rule) {
            Reachability::RequiresAbsent(element) => admitted.contains(&element),
            Reachability::CalleeAtomAlreadyOwned | Reachability::CarrierOwned => false,
        })
        .map(|rule| rule.id.clone())
        .collect()
}

/// Local Early Error question: does this exact call-headed, owner-specific
/// additive composition make any frozen identity newly required/reachable?
/// Falsifiable: the same model flips identities when a trigger element is
/// admitted (controls below). Hypothesis from #688 comment `5908350749`:
/// none; treated here as a hypothesis, not assumed.
#[test]
fn no_frozen_early_error_identity_becomes_newly_reachable() {
    let rules = parse_inventory_rules();

    // The frozen inventory shape is read, not assumed.
    assert_eq!(rules.len(), 193);
    assert_eq!(rules.iter().filter(|rule| rule.active).count(), 183);
    assert_eq!(rules.iter().filter(|rule| !rule.active).count(), 10);
    let mut ids: Vec<&str> = rules.iter().map(|rule| rule.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 193);
    let mut containers: Vec<&str> = rules.iter().map(|rule| rule.container.as_str()).collect();
    containers.sort_unstable();
    containers.dedup();
    assert_eq!(containers.len(), 37);
    assert_eq!(CONTAINER_REACHABILITY.len(), 37);
    for (container, _) in CONTAINER_REACHABILITY {
        assert!(containers.contains(container), "{container}");
    }
    for (id, _) in RULE_REACHABILITY_OVERRIDES {
        assert!(ids.contains(id), "stale override {id}");
    }
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(seen.len(), 193);"));
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(active, 183);"));
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(inactive, 10);"));

    // No admitted trigger syntax is required by any identity as "absent".
    assert!(newly_reachable(&rules, ADMITTED_SYNTAX).is_empty());

    // Every identity in the call/cover/super/async neighborhood is classified
    // as requiring absent syntax (never silently owned), and the named
    // identities the Issue calls out are among them.
    let neighborhood: Vec<&InventoryRule> = rules
        .iter()
        .filter(|rule| {
            CALL_NEIGHBORHOOD_VOCABULARY
                .iter()
                .any(|term| rule.locator.contains(term))
        })
        .collect();
    for rule in &neighborhood {
        assert!(
            matches!(reachability_of(rule), Reachability::RequiresAbsent(_)),
            "{rule:?}"
        );
    }
    for required in [
        "EE-35-R02",
        "EE-33-R10",
        "EE-36-R03",
        "EE-12-R03",
        "EE-10-R01",
    ] {
        assert!(
            neighborhood.iter().any(|rule| rule.id == required),
            "{required}"
        );
    }
    // The only container that owns CallExpression-adjacent syntax directly
    // (Left-Hand-Side Expressions, EE-09) requires `?.` templates or
    // `import.meta`, neither of which is admitted.
    for rule in rules.iter().filter(|rule| rule.container == "EE-09") {
        assert!(matches!(
            reachability_of(rule),
            Reachability::RequiresAbsent(SyntaxElement::OptionalChain | SyntaxElement::ImportMeta)
        ));
    }
    // Callee-atom identities are attributed to the atom, not to the call.
    for id in ["EE-01-R01", "EE-01-R02", "EE-04-R08"] {
        let index = rules.iter().position(|rule| rule.id == id).expect(id);
        let rule = &rules[index];
        assert_eq!(reachability_of(rule), Reachability::CalleeAtomAlreadyOwned);
    }
}

/// Model teeth: admitting the corresponding trigger syntax flips exactly the
/// identities that need it, so the empty result above is a finding, not a
/// tautology.
#[test]
fn reachability_model_flips_when_a_trigger_element_is_admitted() {
    let rules = parse_inventory_rules();
    let with = |element: SyntaxElement| {
        let mut admitted = ADMITTED_SYNTAX.to_vec();
        admitted.push(element);
        newly_reachable(&rules, &admitted)
    };

    let arrow = with(SyntaxElement::ArrowToken);
    for id in ["EE-35-R02", "EE-29-R05"] {
        assert!(arrow.iter().any(|flipped| flipped == id), "{id}");
    }
    assert!(with(SyntaxElement::AssignmentOperator).contains(&"EE-12-R03".to_owned()));
    assert!(with(SyntaxElement::UpdateOperator).contains(&"EE-10-R01".to_owned()));
    assert!(with(SyntaxElement::SuperKeyword).contains(&"EE-36-R03".to_owned()));
    assert!(with(SyntaxElement::DeleteOperator).contains(&"EE-11-R01".to_owned()));
    assert!(with(SyntaxElement::OptionalChain).contains(&"EE-09-R01".to_owned()));
    assert!(with(SyntaxElement::ImportMeta).contains(&"EE-09-R02".to_owned()));
    // The legacy/non-octal numeric spellings are exactly what the plain
    // Decimal tail excludes; admitting them would newly reach EE-02.
    assert!(with(SyntaxElement::LegacyNumericLiteral).contains(&"EE-02-R01".to_owned()));
}

// ---------------------------------------------------------------------------
// F. Independence and handoff
// ---------------------------------------------------------------------------

/// Oracle meta-independence firewall: rejects imports/calls into production
/// lexical, static-semantics, Binding/Scope, correspondence, or aggregate
/// qualification code, runtime-evaluation vocabulary, and the forbidden
/// search/reconstruction patterns that would let expected ranges be derived
/// rather than fixture-owned.
#[test]
fn validation_source_has_no_dependency_on_selected_production_or_runtime_evaluation() {
    for forbidden in [
        concat!("use super::", "selected_lexical_slice"),
        concat!("use super::", "selected_static_semantics"),
        concat!("use super::", "selected_qualification_integration"),
        concat!(
            "use super::",
            "selected_variable_statement_name_correspondence"
        ),
        concat!("use super::", "selected_binding_scope"),
        concat!("use super::", "selected_one_level_block_binding_scope"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("consume_selected_", "identifier_reference"),
        concat!("attempt_selected_", "qualification"),
        concat!("Resolve", "Binding("),
        concat!("Get", "Value("),
        concat!("Is", "Callable("),
        concat!("Perform", "Eval("),
        concat!("Evaluate", "Call("),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
        concat!("match_", "indices("),
        concat!(".", "char_indices("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(THIS_ORACLE_SOURCE.contains(concat!(
        "use super::",
        "unicode::{is_id_continue, is_id_start, is_space_separator}"
    )));
}

/// Freezes the presence-only, unqualified, validation-only handoff: this leaf
/// composes to `Matched` / `NotSelected`; `ResourceLimited` / `InternalFailure`
/// remain a distinct processing-failure identity; and no retained production
/// representation, call identity, runtime, additive-value, or general-parser
/// vocabulary is introduced.
#[test]
fn handoff_remains_presence_only_unqualified_and_validation_only() {
    for forbidden in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("struct Call", "SourceAnchor"),
        concat!("struct Call", "Node"),
        concat!("struct Arguments", "Node"),
        concat!("struct Call", "Fact"),
        concat!("enum Call", "Expression"),
        concat!("struct Call", "Result"),
        concat!("struct Additive", "Node"),
        concat!("enum Additive", "Expression"),
        concat!("struct Generic", "AdditiveTail"),
        concat!("enum Left", "HandSideExpression"),
        concat!("enum Member", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("enum Primary", "Expression"),
        concat!("struct Async", "ArrowFunction"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("struct Reference", "Record"),
        concat!("struct Environment", "Record"),
        concat!("struct Precedence", "Table"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
