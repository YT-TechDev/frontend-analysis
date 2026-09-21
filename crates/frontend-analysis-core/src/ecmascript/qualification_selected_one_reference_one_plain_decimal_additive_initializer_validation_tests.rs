//! Candidate-independent heterogeneous one-`IdentifierReference`,
//! one-plain-Decimal additive initializer fact-preservation validation for
//! Issue #789 (durable research: Issue #688 comment `5726598010`;
//! post-#788 zero-base frontier selection / falsification: Issue #688
//! comment `5760704995`; closest structural precedent: Issue #752 / PR #753
//! ordered two-`IdentifierReference` additive initializer Oracle; separator-
//! free plain Decimal atom family authority: Issue #735 / PR the plain
//! exponent `DecimalLiteral` Oracle, itself built on the plain fractional
//! `DecimalLiteral` Oracle, Issue #727; direct/escaped non-Reserved
//! `IdentifierReference` authored-source/decoded-name theorem: Issue #241).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedOneReferenceOnePlainDecimalAdditiveInitializer ::=
//!     SelectedAcceptedIdentifierReference
//!     SelectedAdditiveTrivia
//!     SelectedAdditiveOperator
//!     SelectedAdditiveTrivia
//!     SelectedAcceptedPlainDecimalAtom
//!   |
//!     SelectedAcceptedPlainDecimalAtom
//!     SelectedAdditiveTrivia
//!     SelectedAdditiveOperator
//!     SelectedAdditiveTrivia
//!     SelectedAcceptedIdentifierReference
//!
//! SelectedAdditiveOperator ::= "+" | "-"
//!
//! SelectedAcceptedIdentifierReference ::=
//!     SelectedDirectIdentifierReference
//!   | SelectedEscapedNonReservedIdentifierReference
//!
//! SelectedAcceptedPlainDecimalAtom ::=
//!     SelectedPlainExponentDecimalLiteral
//!   | SelectedPlainFractionalDecimalLiteral
//!   | SelectedDecimalInteger
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice. It
//! does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! The load-bearing new invariant this leaf adds beyond Issue #752 is:
//!
//! ```text
//! expression operand cardinality != retained semantic-evidence cardinality
//! ```
//!
//! `a + 1` and `1 + a` each have two syntactic operands, but only one of
//! them -- the `IdentifierReference` operand -- is a currently-consumed
//! source-backed semantic fact. The plain Decimal operand remains
//! correspondence-silent and presence-only: it is independently recognized
//! (so full-candidate transactionality is genuinely proved), but it is never
//! materialized into the returned fact.
//!
//! `SelectedAdditiveTrivia`, `SelectedDirectIdentifierReference`, and
//! `SelectedEscapedNonReservedIdentifierReference` independently restate the
//! same already-accepted primitives Issue #752 already restates a second
//! time from Issue #742/#743/#746/#747/#241 -- this leaf restates them a
//! third time rather than importing Issue #752's copy, matching the
//! project's per-leaf restatement convention. `SelectedDecimalInteger`,
//! `SelectedPlainFractionalDecimalLiteral`, and
//! `SelectedPlainExponentDecimalLiteral` independently restate the
//! already-accepted Issue #735 grammar a second time, the same way Issue
//! #735 itself restates Issue #727's predecessor grammar rather than
//! importing it.
//!
//! The only capability this Oracle adds beyond its two predecessor families
//! is dispatching on, and bounding, whichever operand comes first. When the
//! `IdentifierReference` operand is first, `left_operand_run_end` restates
//! Issue #752's bounded forward scan unchanged. When the plain Decimal atom
//! is first, `decimal_atom_run_end` performs the analogous bounded
//! left-to-right forward scan for a Decimal atom shape, whose only
//! non-trivial rule is that a `+`/`-` byte extends the run exclusively when
//! it immediately follows a just-consumed `e`/`E` byte -- the exact rule
//! that keeps an exponent-internal sign (owned by the Decimal atom) distinct
//! from a following binary additive operator, without rescanning, without a
//! general tokenizer, and without ever looking beyond the one byte just
//! consumed. Whichever operand is second is always the entire remaining
//! slice: because it must consume every remaining byte to succeed, no
//! boundary-finding is needed for it, and no valid shorter prefix of a
//! longer candidate (`a + 1 + b`, `1 + a + 2`, ...) can ever be accepted.
//!
//! This is a validation-only leaf: production supports no heterogeneous
//! additive expression family at the #789 baseline, so every positive
//! fixture below remains `UnsupportedCoverage` under current production and
//! exists only as independent Oracle evidence for a future, separately
//! authorized production representation/ownership decision. No completion
//! successor file accompanies this leaf: this Issue adds zero production
//! capability, so the frozen `193 / 10 / 183 / {}` completion partition
//! cannot move, matching the precedent set by #742/#743, #746/#747, and
//! #752/#753.

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 789;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_INITIALIZER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_initializer_validation_tests.rs"
);
const PREVIOUS_PLAIN_EXPONENT_DECIMAL_LITERAL_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_plain_exponent_decimal_literal_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_one_reference_one_plain_decimal_additive_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "heterogeneous exactly-two-operand, one-IdentifierReference, ",
    "one-plain-Decimal-atom additive initializer frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "three-or-more operand chains, unary operands, non-IdentifierReference/",
    "non-Decimal operands, numeric separator/BigInt/non-decimal numeric ",
    "forms, parenthesized/member/call operands, richer expression tails, ",
    "comments, or top-level/Block var placement"
);
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

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

// --- Direct IdentifierReference policy: independently restated a third
// time from Issue #237/#746/#752, never imported from any predecessor. ---

fn is_direct_identifier_start(code_point: char) -> bool {
    matches!(code_point, '$' | '_') || is_id_start(code_point as u32)
}

fn is_direct_identifier_part(code_point: char) -> bool {
    code_point == '$' || is_id_continue(code_point as u32)
}

fn is_escape_free_identifier_name(spelling: &str) -> bool {
    let mut code_points = spelling.chars();
    let Some(first) = code_points.next() else {
        return false;
    };
    is_direct_identifier_start(first) && code_points.all(is_direct_identifier_part)
}

/// `SelectedDirectIdentifierReference`: the already-accepted direct
/// escape-free `IdentifierName` shape, minus the unconditionally reserved
/// words -- never a `BindingIdentifier` policy.
fn is_selected_direct_identifier_reference(candidate: &str) -> bool {
    is_escape_free_identifier_name(candidate)
        && !UNCONDITIONALLY_RESERVED_WORDS.contains(&candidate)
}

// --- Escaped IdentifierReference decode: independently restated a third
// time from Issue #241/#752, never imported from any predecessor. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodeFailure {
    MalformedEscape,
    NonCodePoint,
    InvalidStart,
    InvalidPart,
    DecodedReserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedIdentifier {
    string_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EscapeFormation {
    end: usize,
    code_point: u32,
}

fn ascii_hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

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

/// Forms exactly one `UnicodeEscapeSequence` element at byte offset `start`
/// -- syntactic well-formedness only, never position validity, which is
/// judged afterward once the whole operand slice is known.
fn formed_escape_at(spelling: &str, start: usize) -> Result<EscapeFormation, DecodeFailure> {
    let bytes = spelling.as_bytes();
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return Err(DecodeFailure::MalformedEscape);
    }

    let payload = start + 2;
    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload + 1;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit()) {
            end += 1;
        }
        if end == digits_start || bytes.get(end) != Some(&b'}') {
            return Err(DecodeFailure::MalformedEscape);
        }
        let code_point = parse_braced_code_point(&bytes[digits_start..end])?;
        return Ok(EscapeFormation {
            end: end + 1,
            code_point,
        });
    }

    let end = payload
        .checked_add(4)
        .ok_or(DecodeFailure::MalformedEscape)?;
    let digits = bytes
        .get(payload..end)
        .ok_or(DecodeFailure::MalformedEscape)?;
    if !digits.iter().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DecodeFailure::MalformedEscape);
    }

    let mut code_point = 0_u32;
    for byte in digits {
        code_point =
            code_point * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }

    Ok(EscapeFormation { end, code_point })
}

fn is_selected_identifier_start_for_oracle(code_point: u32) -> bool {
    code_point == u32::from(b'$') || code_point == u32::from(b'_') || is_id_start(code_point)
}

fn is_selected_identifier_part_for_oracle(code_point: u32) -> bool {
    is_selected_identifier_start_for_oracle(code_point)
        || is_id_continue(code_point)
        || matches!(code_point, 0x200C | 0x200D)
}

fn is_unconditionally_reserved_word_for_oracle(spelling: &str) -> bool {
    UNCONDITIONALLY_RESERVED_WORDS.contains(&spelling)
}

/// `SelectedEscapedNonReservedIdentifierReference`: decodes a whole operand
/// slice that must contain at least one authored `UnicodeEscapeSequence`,
/// judging start/part position validity per decoded element and rejecting a
/// decoded-reserved whole name -- independently restated from Issue #241.
fn decode_selected_escaped_identifier(spelling: &str) -> Result<DecodedIdentifier, DecodeFailure> {
    let mut offset = 0_usize;
    let mut element_index = 0_usize;
    let mut saw_escape = false;
    let mut code_points: Vec<u32> = Vec::new();

    while offset < spelling.len() {
        let (code_point, end, escaped) = if spelling.as_bytes().get(offset) == Some(&b'\\') {
            let formation = formed_escape_at(spelling, offset)?;
            (formation.code_point, formation.end, true)
        } else {
            let scalar = spelling[offset..]
                .chars()
                .next()
                .ok_or(DecodeFailure::MalformedEscape)?;
            (scalar as u32, offset + scalar.len_utf8(), false)
        };

        let valid_position = if element_index == 0 {
            is_selected_identifier_start_for_oracle(code_point)
        } else {
            is_selected_identifier_part_for_oracle(code_point)
        };
        if !valid_position {
            return Err(if element_index == 0 {
                DecodeFailure::InvalidStart
            } else {
                DecodeFailure::InvalidPart
            });
        }

        code_points.push(code_point);
        saw_escape |= escaped;
        offset = end;
        element_index += 1;
    }

    if !saw_escape || code_points.is_empty() {
        return Err(DecodeFailure::MalformedEscape);
    }

    let mut string_value = String::new();
    for code_point in &code_points {
        let scalar = char::from_u32(*code_point)
            .expect("position-valid identifier code point must be a Unicode scalar value");
        string_value.push(scalar);
    }

    if is_unconditionally_reserved_word_for_oracle(&string_value) {
        return Err(DecodeFailure::DecodedReserved);
    }

    Ok(DecodedIdentifier { string_value })
}

// --- Selected trivia: independently restated a third time from
// Issue #742/#743/#746/#747/#752. ---

fn is_selected_additive_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

fn trivia_run_end(candidate: &str) -> usize {
    let mut offset = 0_usize;
    for code_point in candidate.chars() {
        if !is_selected_additive_trivia(code_point) {
            break;
        }
        offset += code_point.len_utf8();
    }
    offset
}

// --- Plain Decimal atom grammar: independently restated a second time from
// Issue #727/#735, never imported from that Oracle. ---

/// `SelectedDecimalInteger`.
fn is_selected_decimal_integer(candidate: &str) -> bool {
    if candidate == "0" {
        return true;
    }
    let mut bytes = candidate.bytes();
    match bytes.next() {
        Some(b'1'..=b'9') => bytes.all(|byte| byte.is_ascii_digit()),
        _ => false,
    }
}

fn is_decimal_digits(candidate: &str) -> bool {
    !candidate.is_empty() && candidate.bytes().all(|byte| byte.is_ascii_digit())
}

/// `SelectedPlainFractionalDecimalLiteral ::=
/// SelectedDecimalInteger "." DecimalDigits? | "." DecimalDigits`.
fn is_selected_plain_fractional_decimal_literal(candidate: &str) -> bool {
    let Some((integer_part, fraction_part)) = candidate.split_once('.') else {
        return false;
    };
    if integer_part.is_empty() {
        is_decimal_digits(fraction_part)
    } else {
        is_selected_decimal_integer(integer_part)
            && (fraction_part.is_empty() || is_decimal_digits(fraction_part))
    }
}

fn split_at_first_exponent_marker(candidate: &str) -> Option<(&str, &str)> {
    let marker_position = candidate
        .as_bytes()
        .iter()
        .position(|byte| matches!(byte, b'e' | b'E'))?;
    Some((
        &candidate[..marker_position],
        &candidate[marker_position + 1..],
    ))
}

/// `SelectedPlainExponentPart ::= ("e" | "E") ("+" | "-")? DecimalDigits`
/// (the exponent marker byte itself is already consumed by the caller).
fn is_selected_plain_exponent_part(candidate: &str) -> bool {
    let digits = match candidate.as_bytes().first() {
        Some(b'+') | Some(b'-') => &candidate[1..],
        _ => candidate,
    };
    is_decimal_digits(digits)
}

/// `SelectedPlainExponentDecimalLiteral ::= SelectedDecimalInteger
/// SelectedPlainExponentPart | SelectedPlainFractionalDecimalLiteral
/// SelectedPlainExponentPart`. Whole-string match: any leftover content
/// after a structurally valid mantissa/exponent split disqualifies the
/// candidate.
fn is_selected_plain_exponent_decimal_literal(candidate: &str) -> bool {
    let Some((mantissa, exponent_tail)) = split_at_first_exponent_marker(candidate) else {
        return false;
    };
    if !is_selected_plain_exponent_part(exponent_tail) {
        return false;
    }
    is_selected_decimal_integer(mantissa) || is_selected_plain_fractional_decimal_literal(mantissa)
}

/// `SelectedAcceptedPlainDecimalAtom`: exactly one of the three disjoint
/// whole-string Decimal atom grammars, over the complete slice, never a
/// prefix or a later-reconstructed sub-range.
fn is_selected_accepted_plain_decimal_atom(candidate: &str) -> bool {
    is_selected_plain_exponent_decimal_literal(candidate)
        || is_selected_plain_fractional_decimal_literal(candidate)
        || is_selected_decimal_integer(candidate)
}

// --- The Issue #789 theorem itself. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperandSpellingKind {
    Direct,
    EscapedNonReserved,
}

/// One retained `IdentifierReference` operand fact. A private test-only
/// shape, not a claim about future production storage.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedOperand<'a> {
    authored: &'a str,
    semantic_name: String,
    spelling: OperandSpellingKind,
}

/// The Oracle's only retained representation: exactly one source-backed
/// `IdentifierReference` fact. There is no field for the Decimal operand --
/// its recognition is required for the whole candidate to succeed, but it
/// is never materialized into this struct, which is the direct, by-
/// construction proof of the Issue #789 cardinality-mismatch theorem. This
/// struct shape is a minimal private test representation, never a claim
/// that future production must use this exact shape, an `Option`, a
/// `Vec`, or any other specific storage.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedOneReferenceOnePlainDecimalAdditiveFact<'a> {
    reference: RecognizedOperand<'a>,
}

/// Validates one whole operand slice against `SelectedAcceptedIdentifierReference`:
/// exactly one of the direct or escaped-non-Reserved routes, over the
/// complete slice, never a prefix or a later-reconstructed sub-range.
fn recognize_accepted_identifier_reference(candidate: &str) -> Option<RecognizedOperand<'_>> {
    if candidate.contains('\\') {
        let decoded = decode_selected_escaped_identifier(candidate).ok()?;
        Some(RecognizedOperand {
            authored: candidate,
            semantic_name: decoded.string_value,
            spelling: OperandSpellingKind::EscapedNonReserved,
        })
    } else if is_selected_direct_identifier_reference(candidate) {
        Some(RecognizedOperand {
            authored: candidate,
            semantic_name: candidate.to_owned(),
            spelling: OperandSpellingKind::Direct,
        })
    } else {
        None
    }
}

/// Finds where an `IdentifierReference` operand ends when it is the first
/// (left) operand of the candidate: independently restated, unchanged, from
/// Issue #752's `left_operand_run_end`. One left-to-right forward pass
/// consuming either one direct code point or one syntactically well-formed
/// `UnicodeEscapeSequence` element, stopping at the first selected-trivia
/// code point or at `+`/`-`. A malformed escape aborts the whole candidate
/// immediately (`None`) rather than silently truncating the run at that
/// point. Position validity of the collected run is judged afterward by
/// `recognize_accepted_identifier_reference`, never here.
fn left_operand_run_end(candidate: &str) -> Option<usize> {
    let mut offset = 0_usize;
    while offset < candidate.len() {
        if candidate.as_bytes()[offset] == b'\\' {
            let formation = formed_escape_at(candidate, offset).ok()?;
            offset = formation.end;
            continue;
        }
        let code_point = candidate[offset..].chars().next()?;
        if is_selected_additive_trivia(code_point) || code_point == '+' || code_point == '-' {
            break;
        }
        offset += code_point.len_utf8();
    }
    (offset > 0).then_some(offset)
}

/// Finds where a plain Decimal atom operand ends when it is the first
/// (left) operand of the candidate: one left-to-right forward pass
/// consuming an ASCII digit, `.`, `e`, or `E` unconditionally, and
/// consuming a `+`/`-` byte only when it immediately follows a
/// just-consumed `e`/`E` byte -- the sole rule that keeps an
/// exponent-internal sign owned by the Decimal atom rather than mistaken
/// for a following binary additive operator. Stops at the first byte that
/// cannot extend a plain Decimal atom shape (in particular: at selected
/// trivia, and at a `+`/`-` that does not immediately follow `e`/`E`).
/// Position/grammar validity of the collected run is judged afterward by
/// `is_selected_accepted_plain_decimal_atom`, never here -- this scan never
/// rejects, rescans, or looks beyond the one byte just consumed.
fn decimal_atom_run_end(candidate: &str) -> Option<usize> {
    let bytes = candidate.as_bytes();
    let mut offset = 0_usize;
    let mut previous_was_exponent_marker = false;
    while offset < bytes.len() {
        let byte = bytes[offset];
        let extends = match byte {
            b'0'..=b'9' | b'.' | b'e' | b'E' => true,
            b'+' | b'-' => previous_was_exponent_marker,
            _ => false,
        };
        if !extends {
            break;
        }
        previous_was_exponent_marker = matches!(byte, b'e' | b'E');
        offset += 1;
    }
    (offset > 0).then_some(offset)
}

/// Central Issue #789 theorem. Whole-candidate, all-or-nothing: the
/// reference fact is never returned unless the complete five-part bounded
/// family -- first operand, trivia, exactly one authored `+`/`-`, trivia,
/// second operand -- is recognized over the entire candidate, with nothing
/// left over. Orientation is dispatched, never guessed by trying both: a
/// candidate whose first character is an ASCII digit or `.` can only be a
/// Decimal-first candidate (no accepted `IdentifierReference` spelling
/// starts with either), so every other first character attempts the
/// `IdentifierReference`-first route. Because the second operand in each
/// orientation must consume the *entire* remaining slice to succeed (never
/// merely a prefix of it), a longer chain such as `a + 1 + b` can never
/// yield a valid two-operand prefix: its remaining slice after the first
/// operator is `1 + b`, which is neither a whole Decimal atom nor a whole
/// `IdentifierReference`. The operator is recognized only to prove the
/// exact bounded source family; it is never retained in the returned fact,
/// and neither is the Decimal operand.
fn recognize_selected_one_reference_one_plain_decimal_additive_initializer(
    candidate: &str,
) -> Option<SelectedOneReferenceOnePlainDecimalAdditiveFact<'_>> {
    let starts_decimal = candidate
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit() || first == '.');

    if starts_decimal {
        let decimal_end = decimal_atom_run_end(candidate)?;
        let (decimal_slice, after_decimal) = candidate.split_at(decimal_end);
        if !is_selected_accepted_plain_decimal_atom(decimal_slice) {
            return None;
        }

        let after_decimal_trivia = &after_decimal[trivia_run_end(after_decimal)..];
        match after_decimal_trivia.chars().next() {
            Some('+') | Some('-') => {}
            _ => return None,
        }
        let after_operator = &after_decimal_trivia[1..];

        let reference_slice = &after_operator[trivia_run_end(after_operator)..];
        let reference = recognize_accepted_identifier_reference(reference_slice)?;
        return Some(SelectedOneReferenceOnePlainDecimalAdditiveFact { reference });
    }

    let reference_end = left_operand_run_end(candidate)?;
    let (reference_slice, after_reference) = candidate.split_at(reference_end);
    let reference = recognize_accepted_identifier_reference(reference_slice)?;

    let after_reference_trivia = &after_reference[trivia_run_end(after_reference)..];
    match after_reference_trivia.chars().next() {
        Some('+') | Some('-') => {}
        _ => return None,
    }
    let after_operator = &after_reference_trivia[1..];

    let decimal_slice = &after_operator[trivia_run_end(after_operator)..];
    if !is_selected_accepted_plain_decimal_atom(decimal_slice) {
        return None;
    }

    Some(SelectedOneReferenceOnePlainDecimalAdditiveFact { reference })
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 789);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(
        PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_INITIALIZER_ORACLE_SOURCE
            .contains("ISSUE_ID: u64 = 752")
    );
    assert!(
        PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_INITIALIZER_ORACLE_SOURCE
            .contains("ordered two-IdentifierReference additive initializer frontier only")
    );
    assert!(PREVIOUS_PLAIN_EXPONENT_DECIMAL_LITERAL_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 735"));
    assert!(
        PREVIOUS_PLAIN_EXPONENT_DECIMAL_LITERAL_ORACLE_SOURCE
            .contains("plain exponent DecimalLiteral frontier only")
    );
    assert!(
        FRONTIER_SCOPE_NOTE.contains(
            "heterogeneous exactly-two-operand, one-IdentifierReference, one-plain-Decimal-atom additive initializer frontier only"
        )
    );
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));

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
        concat!("use super::", "selected_binding_identifier"),
        concat!("use super::", "qualification;"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!(
            "consume_selected_leading_plus_minus_decimal_",
            "unary_expression"
        ),
        concat!("consume_selected_", "identifier_reference"),
        concat!("analyze_selected_binding_", "scope("),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_selected_block_var_", "statement"),
        concat!("parse_unary_", "expression"),
        concat!("parse_additive_", "expression"),
        concat!("identifier_reference_initializer_", "facts("),
        concat!("SelectedIdentifierReference", "Initializer"),
        // Candidate independence for trivia recognition specifically: only
        // the stable normative Unicode `Space_Separator` property primitive
        // is reused, never the production selected-trivia recognizer.
        concat!("is_selected_", "trivia("),
        concat!("skip_selected_", "trivia("),
        concat!("use super::", "selected_lexical_slice::is_selected_trivia"),
        // No cross-Oracle import: this leaf restates rather than imports.
        concat!(
            "use super::qualification_selected_two_identifier_reference_",
            "additive_initializer_validation_tests"
        ),
        concat!(
            "use super::qualification_selected_plain_exponent_decimal_",
            "literal_initializer_validation_tests"
        ),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // `is_id_start` / `is_id_continue` / `is_space_separator` are stable,
    // frozen normative Unicode 17 primitives this oracle is independently
    // entitled to consult, already used the same way by both predecessor
    // accepted oracle families at the same `super::unicode` path.
    assert!(
        THIS_ORACLE_SOURCE
            .contains("use super::unicode::{is_id_continue, is_id_start, is_space_separator}")
    );
}

/// Required positive matrix (both operators, all three accepted Decimal
/// atom families, both orientations): every candidate retains exactly one
/// `IdentifierReference` fact with its own exact authored anchor, and the
/// Decimal operand's fixture-owned range is independently confirmed as a
/// whole accepted Decimal atom without ever being retained in the returned
/// fact (W1/W2/W3/W4/W6).
#[test]
fn positive_matrix_pins_exactly_one_reference_fact_with_decimal_operand_presence_only() {
    struct PositiveFixture {
        text: &'static str,
        reference: Range,
        decimal: Range,
        expected_reference: &'static str,
        expected_decimal: &'static str,
    }

    let fixtures = [
        PositiveFixture {
            text: "const x = a + 1;",
            reference: Range(10, 11),
            decimal: Range(14, 15),
            expected_reference: "a",
            expected_decimal: "1",
        },
        PositiveFixture {
            text: "const x = a - 1;",
            reference: Range(10, 11),
            decimal: Range(14, 15),
            expected_reference: "a",
            expected_decimal: "1",
        },
        PositiveFixture {
            text: "const x = a + 1.0;",
            reference: Range(10, 11),
            decimal: Range(14, 17),
            expected_reference: "a",
            expected_decimal: "1.0",
        },
        PositiveFixture {
            text: "const x = a - .5;",
            reference: Range(10, 11),
            decimal: Range(14, 16),
            expected_reference: "a",
            expected_decimal: ".5",
        },
        PositiveFixture {
            text: "const x = a + 1e2;",
            reference: Range(10, 11),
            decimal: Range(14, 17),
            expected_reference: "a",
            expected_decimal: "1e2",
        },
        PositiveFixture {
            text: "const x = a - 1e-2;",
            reference: Range(10, 11),
            decimal: Range(14, 18),
            expected_reference: "a",
            expected_decimal: "1e-2",
        },
        PositiveFixture {
            text: "const x = 1 + a;",
            decimal: Range(10, 11),
            reference: Range(14, 15),
            expected_decimal: "1",
            expected_reference: "a",
        },
        PositiveFixture {
            text: "const x = 1.0 - a;",
            decimal: Range(10, 13),
            reference: Range(16, 17),
            expected_decimal: "1.0",
            expected_reference: "a",
        },
        PositiveFixture {
            text: "const x = .5 + a;",
            decimal: Range(10, 12),
            reference: Range(15, 16),
            expected_decimal: ".5",
            expected_reference: "a",
        },
        PositiveFixture {
            text: "const x = 1e2 - a;",
            decimal: Range(10, 13),
            reference: Range(16, 17),
            expected_decimal: "1e2",
            expected_reference: "a",
        },
        PositiveFixture {
            text: "const x = 1e-2 + a;",
            decimal: Range(10, 14),
            reference: Range(17, 18),
            expected_decimal: "1e-2",
            expected_reference: "a",
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        assert_eq!(
            slice(fixture.text, fixture.reference),
            fixture.expected_reference
        );
        assert_eq!(
            slice(fixture.text, fixture.decimal),
            fixture.expected_decimal
        );
        assert!(
            is_selected_accepted_plain_decimal_atom(fixture.expected_decimal),
            "{:?}",
            fixture.expected_decimal
        );

        let whole_start = fixture.reference.0.min(fixture.decimal.0);
        let whole_end = fixture.reference.1.max(fixture.decimal.1);
        let whole_text = slice(fixture.text, Range(whole_start, whole_end));
        let fact =
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(whole_text)
                .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(fact.reference.authored, fixture.expected_reference);
        assert_eq!(fact.reference.semantic_name, fixture.expected_reference);
        assert_eq!(fact.reference.spelling, OperandSpellingKind::Direct);

        assert_eq!(
            authored_anchor(789_000 + index as u64, fixture.text, fixture.reference),
            fixture.expected_reference
        );
        assert_eq!(
            authored_anchor(789_050 + index as u64, fixture.text, fixture.decimal),
            fixture.expected_decimal
        );
    }
}

/// Direct/escaped `IdentifierReference` provenance (both orientations):
/// preserves exact authored UTF-8 range, exact authored fragment,
/// Direct|Escaped semantic-name state, and decoded semantic name, with no
/// Unicode normalization (W5).
#[test]
fn direct_escaped_reference_provenance_matrix_pins_authored_vs_decoded_identity() {
    struct ProvenanceFixture {
        text: &'static str,
        reference: Range,
        expected_authored: &'static str,
        expected_semantic: &'static str,
        expected_spelling: OperandSpellingKind,
    }

    let fixtures = [
        ProvenanceFixture {
            text: concat!("const x = ", "\\", "u0061", " + 1;"),
            reference: Range(10, 16),
            expected_authored: concat!("\\", "u0061"),
            expected_semantic: "a",
            expected_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        ProvenanceFixture {
            text: concat!("const x = 1 + ", "\\", "u0061", ";"),
            reference: Range(14, 20),
            expected_authored: concat!("\\", "u0061"),
            expected_semantic: "a",
            expected_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        ProvenanceFixture {
            text: concat!("const x = f", "\\", "u006F", "o - 1e2;"),
            reference: Range(10, 18),
            expected_authored: concat!("f", "\\", "u006F", "o"),
            expected_semantic: "foo",
            expected_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        ProvenanceFixture {
            text: concat!("const x = .5 + b", "\\", "u0061", "r;"),
            reference: Range(15, 23),
            expected_authored: concat!("b", "\\", "u0061", "r"),
            expected_semantic: "bar",
            expected_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        // The load-bearing Issue #789 escaped-provenance witness.
        ProvenanceFixture {
            text: concat!("const x = 1e2 + f", "\\", "u006F", "o;"),
            reference: Range(16, 24),
            expected_authored: concat!("f", "\\", "u006F", "o"),
            expected_semantic: "foo",
            expected_spelling: OperandSpellingKind::EscapedNonReserved,
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        assert_eq!(
            slice(fixture.text, fixture.reference),
            fixture.expected_authored
        );

        let fact = recognize_from_declaration_rhs(fixture.text)
            .unwrap_or_else(|| panic!("{:?} must recognize the theorem", fixture.text));

        assert_eq!(fact.reference.authored, fixture.expected_authored);
        assert_eq!(fact.reference.semantic_name, fixture.expected_semantic);
        assert_eq!(fact.reference.spelling, fixture.expected_spelling);
        assert_ne!(fact.reference.authored, fact.reference.semantic_name);

        assert_eq!(
            authored_anchor(789_100 + index as u64, fixture.text, fixture.reference),
            fixture.expected_authored
        );
    }
}

/// Recognizes the theorem directly from a `const x = <candidate>;`
/// declaration by fixture-owned literal RHS bounds (`"const x = "` is 10
/// bytes; the trailing `;` is stripped), never by `.find`/`.rfind`/search.
fn recognize_from_declaration_rhs(
    text: &str,
) -> Option<SelectedOneReferenceOnePlainDecimalAdditiveFact<'_>> {
    let rhs = text.strip_prefix("const x = ")?.strip_suffix(';')?;
    recognize_selected_one_reference_one_plain_decimal_additive_initializer(rhs)
}

/// Selected trivia around the binary operator (both orientations): the
/// already-accepted selected trivia set composes on both sides of the
/// operator, comments never compose as operand trivia, and U+200B remains
/// outside (W6 adjacent).
#[test]
fn selected_additive_trivia_matrix_around_binary_operator() {
    let fixtures: &[(&str, &str)] = &[
        ("a+1", "a"),
        ("a + 1", "a"),
        ("a\t+\t1", "a"),
        ("1\u{00A0}-\u{00A0}a", "a"),
        ("a\u{2028}+\u{2029}1", "a"),
    ];

    for (candidate, expected_reference) in fixtures {
        let fact =
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(candidate)
                .unwrap_or_else(|| panic!("{candidate:?} must recognize the theorem"));
        assert_eq!(fact.reference.semantic_name, *expected_reference);
    }

    // Comments never compose as operand trivia.
    for comment_control in ["a+/*c*/1", "a/*c*/+1", "a+//c\n1", "1+/*c*/a", "1/*c*/+a"] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(
                comment_control
            )
            .is_none(),
            "{comment_control:?}"
        );
    }

    // Adversarial boundary: ZERO WIDTH SPACE (U+200B) sits immediately
    // outside the frozen `Space_Separator` range and is never accepted
    // merely because it visually resembles spacing.
    assert!(!is_selected_additive_trivia('\u{200B}'));
    assert!(is_selected_additive_trivia('\u{00A0}'));
    assert!(
        recognize_selected_one_reference_one_plain_decimal_additive_initializer("a+\u{200B}1")
            .is_none()
    );
    assert!(
        recognize_selected_one_reference_one_plain_decimal_additive_initializer("1+\u{200B}a")
            .is_none()
    );
}

/// Full-candidate transactionality (both orientations, both operators): no
/// valid shorter prefix of a longer chain is ever accepted, a trailing
/// operator or unexpected tail never yields a partial result, and the
/// complete bounded candidate remains independently valid (W8/W9).
#[test]
fn transactionality_never_commits_partial_evidence_and_chains_never_truncate() {
    for control in [
        "a + 1 + b",
        "1 + a + 2",
        "a - 1 - b",
        "1 - a - 2",
        "a +",
        "1 +",
        "a + 1 unexpected",
        "1 + a unexpected",
    ] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    // The complete bounded candidates remain independently valid; only the
    // incomplete/overextended forms above are rejected.
    assert!(
        recognize_selected_one_reference_one_plain_decimal_additive_initializer("a + 1").is_some()
    );
    assert!(
        recognize_selected_one_reference_one_plain_decimal_additive_initializer("1 + a").is_some()
    );
}

/// Unary operand firewall: leading `+`/`-` on either operand, in either
/// orientation, keeps the candidate outside this theorem, whatever
/// already-accepted unary-wrapper production exists elsewhere. The
/// exponent-internal sign in `1e-2`/`1e+2` remains valid and distinct from a
/// leading `UnaryExpression` sign.
#[test]
fn unary_operand_firewall_keeps_signed_operands_outside() {
    for control in [
        "+a + 1", "-a + 1", "a + +1", "a + -1", "+1 + a", "-1 + a", "1 + +a", "1 + -a",
    ] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    for exponent_internal_sign in ["a + 1e-2", "1e-2 + a", "a - 1e+2", "1e+2 - a"] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(
                exponent_internal_sign
            )
            .is_some(),
            "{exponent_internal_sign:?}"
        );
    }
}

/// Operand-family firewall: reference/reference, numeric/numeric, and
/// Boolean/null/this/String operands remain outside this heterogeneous
/// theorem, whether on the left or right position (W11/W12/W13).
#[test]
fn operand_family_firewall_keeps_non_heterogeneous_pairs_outside() {
    for control in [
        "a + b",
        "1 + 2",
        "a + true",
        "true + a",
        "a + null",
        "null + a",
        "a + this",
        "this + a",
        "a + \"x\"",
        "\"x\" + a",
    ] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }
}

/// Richer-expression/precedence firewall: no dedicated precedence parser is
/// built; a valid bounded prefix never authorizes a richer expression tail
/// (W14).
#[test]
fn richer_expression_precedence_firewall_never_prefix_commits() {
    for control in [
        "a + 1 * b",
        "a * 1 + b",
        "a + 1 ** b",
        "1 ** 2 + a",
        "a = 1 + b",
        "a += 1",
        "a ? 1 : b",
        "a || 1",
        "a && 1",
        "a ?? 1",
        "(a) + 1",
        "1 + (a)",
        "a.b + 1",
        "1 + a.b",
        "a() + 1",
        "1 + a()",
    ] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }
}

/// Numeric frontier firewall: `NumericLiteralSeparator`, `NonDecimalIntegerLiteral`,
/// and `BigIntLiteral` forms never leak into this theorem, whichever operand
/// position they occupy (W15).
#[test]
fn numeric_frontier_firewall_remains_unowned_in_either_position() {
    for control in [
        "1_0 + a", "0x10 + a", "1n + a", "a + 1_0", "a + 0x10", "a + 1n",
    ] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }
}

/// Escaped `ReservedWord` firewall: a decoded unconditionally reserved word
/// never becomes a selected `IdentifierReference` operand, whether it is
/// the left or the right operand (W16).
#[test]
fn escaped_reserved_word_firewall_keeps_decoded_reserved_operands_outside() {
    for control in [
        concat!("\\", "u0069f", " + 1"),
        concat!("1 + ", "\\", "u0069f"),
    ] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0069f")),
        Err(DecodeFailure::DecodedReserved)
    );
}

/// Malformed / position-invalid escaped reference firewall (both
/// orientations): a malformed or position-invalid escape never leaks a
/// truncated prefix or the Decimal sibling's success -- the whole candidate
/// is rejected (W17).
#[test]
fn malformed_and_position_invalid_escape_firewall_never_leaks_sibling_success() {
    for control in [
        concat!("\\", "u0030", " + 1"),
        concat!("1 + ", "\\", "u0030"),
        concat!("a", "\\", "u002D", "b + 1"),
        concat!("1 + b", "\\", "u002D", "c"),
        concat!("\\", "uD800", " + 1"),
        concat!("1 + ", "\\", "u{110000}"),
        concat!("\\", "u{}", " + 1"),
        concat!("1 + ", "\\", "u{}"),
    ] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    // The valid Decimal sibling never publishes success when the reference
    // evidence is incomplete: confirmed directly against the same malformed
    // escapes used above.
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0030")),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("a", "\\", "u002D")),
        Err(DecodeFailure::InvalidPart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "uD800")),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u{110000}")),
        Err(DecodeFailure::NonCodePoint)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u{}")),
        Err(DecodeFailure::MalformedEscape)
    );
}

/// Cardinality firewall: a single operand alone remains outside this exact
/// bounded theorem.
#[test]
fn cardinality_firewall_keeps_single_operand_outside() {
    for control in ["a", "1", "1e2", ".5", ""] {
        assert!(
            recognize_selected_one_reference_one_plain_decimal_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }
}

/// Resource / failure semantics (W18): the project's
/// `ResourceLimited`/`InternalFailure` lifecycle states remain distinct
/// from `UnsupportedCoverage`, matching the minimal symbolic model already
/// accepted by the predecessor candidate-independent oracles -- this leaf
/// adds no further resource machinery beyond that already-accepted minimum.
#[test]
fn resource_and_internal_failure_lifecycle_states_remain_distinct_from_unsupported_coverage() {
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
    assert!(THIS_ORACLE_SOURCE.contains("ResourceLimited"));
    assert!(THIS_ORACLE_SOURCE.contains("InternalFailure"));
    assert_ne!(PROCESSING_FAILURES[0], "UnsupportedCoverage");
    assert_ne!(PROCESSING_FAILURES[1], "UnsupportedCoverage");
}

/// Freezes the presence-only, unqualified, validation-only handoff: no
/// `ReferenceLeft`/`ReferenceRight` enum, operator enum, operator/Decimal/
/// whole-expression `SourceAnchor`, Decimal atom kind, binary-expression
/// node, AST/CST, token tape, numeric value, or runtime/static vocabulary is
/// introduced; the returned fact type carries exactly one field (W3/W7 and
/// the "existing `One` reuse is not authorized" boundary -- this Oracle
/// itself makes no claim about the production initializer carrier type at
/// all, so it cannot freeze that production hypothesis).
#[test]
fn handoff_remains_validation_only_with_no_orientation_operator_or_runtime_representation() {
    for forbidden in [
        concat!("enum Reference", "Left"),
        concat!("enum Reference", "Right"),
        concat!("enum Selected", "AdditiveOperatorKind"),
        concat!("enum Operator", "Kind"),
        concat!("enum Plus", "Minus"),
        concat!("struct Operator", "SourceAnchor"),
        concat!("struct Decimal", "SourceAnchor"),
        concat!("enum Decimal", "Kind"),
        concat!("struct WholeExpression", "SourceAnchor"),
        concat!("struct Binary", "ExpressionNode"),
        concat!("struct Additive", "ExpressionNode"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("f64", "Value"),
        concat!("Get", "Value("),
        concat!("Resolve", "Binding("),
        concat!("Reference", "Record"),
        concat!("Environment", "Record"),
        concat!("To", "Numeric("),
        concat!("To", "Number("),
        concat!("To", "Primitive("),
        concat!("To", "String("),
        concat!("Bound", "Names("),
        concat!("VarDeclared", "Names("),
        concat!("LexicallyDeclared", "Names("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // The returned fact type carries exactly one field -- the reference
    // fact -- and no Decimal field, which is the structural proof that the
    // Decimal operand is presence-only.
    assert!(THIS_ORACLE_SOURCE.contains("struct SelectedOneReferenceOnePlainDecimalAdditiveFact"));
    assert!(THIS_ORACLE_SOURCE.contains("reference: RecognizedOperand"));
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("decimal", ": RecognizedOperand")));

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
