//! Candidate-independent ordered 2..N heterogeneous `IdentifierReference` /
//! plain-Decimal additive-chain fact-preservation validation for Issue #815
//! (durable research: Issue #688 comment `5811369636`; accepted predecessor
//! authorities: Issue #789 / PR #790 candidate-independent exactly-two
//! heterogeneous one-`IdentifierReference`, one-plain-Decimal-atom additive
//! theorem; Issue #791 / PR #792 and Issue #793 / PR #794 production
//! composition of that theorem; Issue #801 / PR #802 candidate-independent
//! ordered plain-`IdentifierReference` additive-chain `2..N` theorem with
//! genuine iterative continuation; Issue #809 / PR #810 accepted precedent
//! for composing one bounded independent theorem with one arbitrary-`N`
//! theorem into a new arbitrary-`N` Oracle; Issue #813 / PR #814 latest
//! optional-leading-`+`/`-` `IdentifierReference` additive-chain production).
//!
//! This oracle qualifies only the unbounded expression-composition family:
//!
//! ```text
//! SelectedHeterogeneousReferenceDecimalAdditiveChain ::=
//!     SelectedHeterogeneousOperand
//!     (
//!         SelectedAdditiveTrivia
//!         ("+" | "-")
//!         SelectedAdditiveTrivia
//!         SelectedHeterogeneousOperand
//!     )+
//!
//! SelectedHeterogeneousOperand ::=
//!       SelectedAcceptedIdentifierReference
//!     | SelectedAcceptedPlainDecimalAtom
//!
//! SelectedAcceptedIdentifierReference ::=
//!       SelectedDirectIdentifierReference
//!     | SelectedEscapedNonReservedIdentifierReference
//!
//! SelectedAcceptedPlainDecimalAtom ::=
//!       SelectedPlainExponentDecimalLiteral
//!     | SelectedPlainFractionalDecimalLiteral
//!     | SelectedDecimalInteger
//! ```
//!
//! with whole-candidate acceptance additionally requiring `operand_count >=
//! 2`, at least one retained `IdentifierReference` fact, and at least one
//! recognized (but never retained) plain Decimal atom, over the entire naked
//! candidate.
//!
//! ## Load-bearing novelty: composing a bounded heterogeneous theorem with an
//! arbitrary-`N` continuation
//!
//! Issue #789 independently proved that a single `IdentifierReference`
//! operand and a single plain Decimal atom operand compose heterogeneously in
//! either order, but only for an exactly-two-operand candidate. Issue #801
//! independently proved a genuine arbitrary-cardinality iterative additive
//! continuation, but only for plain `IdentifierReference` operands. Neither
//! predecessor proves their composition: that at every continuation of an
//! arbitrary finite chain, each operand may independently be either family,
//! that the exact bounded exponent-sign-ownership rule Issue #789 already
//! proved for a Decimal atom applies unchanged no matter which position in
//! the chain that atom occupies, and that the retained
//! `IdentifierReference` fact sequence tracks only the reference-family
//! operands, in exact authored order, while every Decimal-family operand
//! remains correspondence-silent and presence-only. This Oracle proves
//! exactly that composed theorem, and nothing more, matching the accepted
//! composition precedent already set by Issue #809/#810.
//!
//! `recognize_selected_heterogeneous_reference_decimal_additive_chain` is a
//! genuine `loop`, never an unrolled fixed sequence of named
//! `first`/`second`/`third`/`fourth` fields, and is deliberately exercised
//! beyond four and five operands (an eight-operand sentinel below) precisely
//! so an accidental fixed cap cannot silently satisfy the suite.
//!
//! ## Operand family dispatch and Decimal exponent-sign ownership
//!
//! Dispatch is purely syntactic and never depends on decoded semantic
//! identity: a candidate operand is attempted as
//! `SelectedAcceptedPlainDecimalAtom` only when its first authored byte is an
//! ASCII digit or `.`; every other first byte attempts
//! `SelectedAcceptedIdentifierReference`. In particular an escaped
//! `IdentifierReference` such as `\u0030` is never reclassified as Decimal
//! merely because its decoded value resembles a digit, because its authored
//! first byte is `\`, not a digit or `.`.
//!
//! `operand_interior_run_end` is the same bounded, single left-to-right
//! forward pass already established by Issue #752/#795/#801 for the
//! `IdentifierReference` family (consuming one direct code point or one
//! syntactically well-formed `UnicodeEscapeSequence` element at a time,
//! stopping only at the first selected-trivia code point or at `+`/`-`).
//! `decimal_atom_run_end` independently restates, unchanged, the Issue #789
//! bounded left-to-right forward scan for the Decimal family: it
//! unconditionally extends over an ASCII digit, `.`, `e`, or `E`, and extends
//! over a `+`/`-` byte *only* when that byte immediately follows a
//! just-consumed `e`/`E` byte -- the sole rule that keeps an
//! exponent-internal sign owned by the Decimal atom distinct from a
//! following binary additive operator, without rescanning, without a general
//! tokenizer, and without ever looking beyond the one byte just consumed.
//! Neither scan ever rejects, rescans, or reinterprets a byte it has already
//! judged; grammar validity of the collected run is judged afterward, once
//! per whole operand slice, by `is_selected_accepted_plain_decimal_atom` or
//! `recognize_accepted_identifier_reference`. A structurally invalid Decimal
//! run (for example `1e+` with nothing after the exponent sign) fails the
//! whole candidate outright and is never reinterpreted as a shorter accepted
//! numeric prefix followed by a binary operator.
//!
//! `SelectedAdditiveTrivia` independently restates the complete
//! already-accepted selected-slice trivia contract established by Issue
//! #742/#743 and restated again by Issue #746/#747, Issue #752/#753, Issue
//! #789/#790, Issue #795/#796, and Issue #801/#802 (the same code-point set
//! the production selected-trivia recognizer accepts): `TAB`, `VT`, `FF`,
//! `BOM`, `LF`, `CR`, `LINE SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen
//! Unicode 17 `Space_Separator` property. Comments remain outside this
//! contract and are never accepted as trivia.
//!
//! `SelectedDirectIdentifierReference` independently restates the
//! already-accepted Issue #237 direct escape-free `IdentifierName`
//! code-point shape (`is_id_start`/`is_id_continue` plus `$`/`_`) minus the
//! unconditionally reserved words. `SelectedEscapedNonReservedIdentifierReference`
//! independently restates the already-accepted Issue #241
//! `UnicodeEscapeSequence` decode/position/decoded-reserved-word theorem.
//! `SelectedDecimalInteger`, `SelectedPlainFractionalDecimalLiteral`, and
//! `SelectedPlainExponentDecimalLiteral` independently restate the
//! already-accepted Issue #727/#735 plain Decimal atom grammar. None of these
//! restatements imports another Oracle's code: this leaf owns its own copy of
//! every already-accepted primitive it consumes, matching the convention
//! already set by every predecessor listed above.
//!
//! ## What is retained, and what is not
//!
//! The Oracle's only retained representation is
//! `HeterogeneousAdditiveChainEvidence`, holding an ordered
//! `Vec<RecognizedReference>` of every `IdentifierReference`-family operand's
//! own exact authored `SourceAnchor`/fragment, decoded semantic name, and
//! Direct/EscapedNonReserved spelling state (in exact authored left-to-right
//! order, duplicates never deduplicated), plus the total `operand_count`
//! across both families. This is a minimal private test-only shape chosen
//! only because the theorem itself is `2..N` and load-bearingly
//! heterogeneous:
//!
//! ```text
//! ORACLE DYNAMIC STORAGE (private Vec<RecognizedReference> + operand_count)
//! !=
//! PRODUCTION STORAGE AUTHORITY
//! ```
//!
//! It explicitly does **not** freeze future production representation, does
//! not retain any Decimal `SourceAnchor`, Decimal literal kind, or Decimal
//! value, does not retain a binary-operator enum or `SourceAnchor`, does not
//! retain orientation (`ReferenceLeft`/`ReferenceRight`), does not retain a
//! whole-expression `SourceAnchor`, and performs no runtime evaluation
//! (`GetValue`/`ResolveBinding`/`ToPrimitive`/`ToNumeric`/`ToNumber`/`ToString`).
//! `references.len() < operand_count` is the direct, by-construction proof
//! that at least one Decimal operand existed without retaining Decimal
//! evidence merely to prove heterogeneity.
//!
//! This is a validation-only leaf: production supports no heterogeneous
//! arbitrary-cardinality additive family at the #815 baseline, so every
//! positive fixture below remains `UnsupportedCoverage` under current
//! production and exists only as independent Oracle evidence for a future,
//! separately authorized production decision. No completion successor file
//! accompanies this leaf: this Issue adds zero production capability, so the
//! frozen `193 / 10 / 183 / {}` completion partition cannot move, matching
//! the precedent set by #742/#743, #746/#747, #752/#753, #789/#790,
//! #795/#796, #801/#802, and #809/#810.

use crate::{SourceId, SourceText};

use super::super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};
use super::inventory::RULE_UNITS;

const ISSUE_ID: u64 = 815;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("model.rs");
const PREVIOUS_ONE_REFERENCE_ONE_PLAIN_DECIMAL_ADDITIVE_ORACLE_SOURCE: &str = include_str!(
    "../qualification_selected_one_reference_one_plain_decimal_additive_initializer_validation_tests.rs"
);
const PREVIOUS_PLAIN_IDENTIFIER_REFERENCE_ADDITIVE_CHAIN_ORACLE_SOURCE: &str =
    include_str!("selected_plain_identifier_reference_additive_chain_frontier.rs");
const THIS_ORACLE_SOURCE: &str =
    include_str!("selected_heterogeneous_reference_decimal_additive_chain_frontier.rs");
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "ordered 2..N heterogeneous IdentifierReference/plain-Decimal additive-chain ",
    "frontier only; later independently qualified owners may strengthen classification ",
    "for unary operands, non-IdentifierReference/non-Decimal operands, numeric ",
    "separator/radix/BigInt forms, parenthesized/member/call operands, richer ",
    "expression tails, comments, or top-level/Block var placement"
);
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];
const SELECTED_OPERAND_CARDINALITY: &str = "2..N";

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

// --- Direct IdentifierReference policy: independently restated from Issue
// #237/#752/#789/#801, never imported from any of those Oracles. ---

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

// --- Escaped IdentifierReference decode: independently restated from Issue
// #241, never imported from that Oracle. ---

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

// --- Selected trivia: independently restated from Issue
// #742/#743/#746/#747/#752/#789/#801. ---

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

// --- Plain Decimal atom grammar: independently restated from Issue
// #727/#735/#789, never imported from any of those Oracles. ---

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

/// Finds where a plain Decimal atom operand ends, wherever in the chain it
/// occurs: one left-to-right forward pass consuming an ASCII digit, `.`,
/// `e`, or `E` unconditionally, and consuming a `+`/`-` byte only when it
/// immediately follows a just-consumed `e`/`E` byte -- the sole rule that
/// keeps an exponent-internal sign owned by the Decimal atom rather than
/// mistaken for a following binary additive operator. Stops at the first
/// byte that cannot extend a plain Decimal atom shape (in particular: at
/// selected trivia, and at a `+`/`-` that does not immediately follow
/// `e`/`E`). Position/grammar validity of the collected run is judged
/// afterward by `is_selected_accepted_plain_decimal_atom`, never here --
/// this scan never rejects, rescans, or looks beyond the one byte just
/// consumed. Independently restated, unchanged, from Issue #789's
/// `decimal_atom_run_end`.
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

// --- The Issue #815 theorem itself. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperandSpellingKind {
    Direct,
    EscapedNonReserved,
}

/// One retained `IdentifierReference`-family operand fact. A private
/// test-only shape, not a claim about future production storage.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedReference<'a> {
    authored: &'a str,
    semantic_name: String,
    spelling: OperandSpellingKind,
}

/// The Oracle's only retained representation: an ordered sequence of
/// `IdentifierReference`-family operand facts, plus the total syntax operand
/// count across both families. `references.len() < operand_count` is the
/// by-construction proof that at least one Decimal-family operand existed
/// without ever materializing Decimal evidence. This `Vec` shape is a
/// minimal private test representation chosen only because the theorem
/// itself is `2..N`; it is never a claim that future production must use
/// this exact shape.
#[derive(Debug, Clone, PartialEq, Eq)]
struct HeterogeneousAdditiveChainEvidence<'a> {
    references: Vec<RecognizedReference<'a>>,
    operand_count: usize,
}

/// Validates one whole operand slice against `SelectedAcceptedIdentifierReference`:
/// exactly one of the direct or escaped-non-Reserved routes, over the
/// complete slice, never a prefix or a later-reconstructed sub-range.
fn recognize_accepted_identifier_reference(candidate: &str) -> Option<RecognizedReference<'_>> {
    if candidate.contains('\\') {
        let decoded = decode_selected_escaped_identifier(candidate).ok()?;
        Some(RecognizedReference {
            authored: candidate,
            semantic_name: decoded.string_value,
            spelling: OperandSpellingKind::EscapedNonReserved,
        })
    } else if is_selected_direct_identifier_reference(candidate) {
        Some(RecognizedReference {
            authored: candidate,
            semantic_name: candidate.to_owned(),
            spelling: OperandSpellingKind::Direct,
        })
    } else {
        None
    }
}

/// Finds where an `IdentifierReference`-family operand ends, wherever in the
/// chain it occurs: one left-to-right forward pass consuming either one
/// direct code point or one syntactically well-formed `UnicodeEscapeSequence`
/// element, stopping at the first selected-trivia code point or at `+`/`-`,
/// or at the end of the candidate. A malformed escape aborts the whole
/// candidate immediately (`None`) rather than silently truncating the run at
/// that point. Position validity of the collected run is judged afterward by
/// `recognize_accepted_identifier_reference`, never here. Independently
/// restated, unchanged, from Issue #801's `operand_interior_run_end`.
fn operand_interior_run_end(candidate: &str) -> Option<usize> {
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

/// Central Issue #815 theorem: a genuine left-to-right iterative recognizer,
/// not an unrolled fixed-cardinality theorem. Whole-candidate,
/// all-or-nothing: no evidence is ever returned unless the complete family --
/// one accepted heterogeneous operand, then one-or-more repetitions of
/// (trivia, exactly one authored `+`/`-`, trivia, one more accepted
/// heterogeneous operand) -- is recognized over the entire candidate, with
/// nothing left over, `operand_count >= 2`, at least one retained reference,
/// and at least one recognized Decimal atom. Dispatch per operand is purely
/// syntactic: an operand beginning with an ASCII digit or `.` is attempted
/// only as a Decimal atom; every other operand is attempted only as an
/// `IdentifierReference`. Reference facts are only ever appended to a local
/// `Vec` during the loop and combined into the returned struct in a single
/// final expression, so a failure partway through the chain can never
/// publish an earlier prepared prefix as a partial result (`?` returns
/// `None` from every fallible step before that final expression is reached).
/// Every authored binary operator is recognized only to prove the exact
/// bounded continuation family; none is ever retained in the returned
/// evidence, and neither is any Decimal atom.
fn recognize_selected_heterogeneous_reference_decimal_additive_chain(
    candidate: &str,
) -> Option<HeterogeneousAdditiveChainEvidence<'_>> {
    let mut references: Vec<RecognizedReference<'_>> = Vec::new();
    let mut operand_count = 0_usize;
    let mut saw_decimal = false;
    let mut remaining = candidate;

    loop {
        let starts_decimal = remaining
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_digit() || first == '.');

        let after_operand = if starts_decimal {
            let operand_end = decimal_atom_run_end(remaining)?;
            let (operand_slice, after_operand) = remaining.split_at(operand_end);
            if !is_selected_accepted_plain_decimal_atom(operand_slice) {
                return None;
            }
            saw_decimal = true;
            after_operand
        } else {
            let operand_end = operand_interior_run_end(remaining)?;
            let (operand_slice, after_operand) = remaining.split_at(operand_end);
            let reference = recognize_accepted_identifier_reference(operand_slice)?;
            references.push(reference);
            after_operand
        };
        operand_count += 1;

        if after_operand.is_empty() {
            break;
        }

        let after_trivia = &after_operand[trivia_run_end(after_operand)..];
        match after_trivia.chars().next() {
            Some('+') | Some('-') => {
                let after_operator = &after_trivia[1..];
                remaining = &after_operator[trivia_run_end(after_operator)..];
            }
            // Either trailing trivia with no operator behind it (outer
            // trivia does not belong to this naked theorem), or unexpected
            // trailing content that the operand scan stopped short of
            // absorbing: both remain unaccepted, never a partial prefix.
            _ => return None,
        }
    }

    (operand_count >= 2 && !references.is_empty() && saw_decimal).then_some(
        HeterogeneousAdditiveChainEvidence {
            references,
            operand_count,
        },
    )
}

fn names<'b>(evidence: &'b HeterogeneousAdditiveChainEvidence<'_>) -> Vec<&'b str> {
    evidence
        .references
        .iter()
        .map(|reference| reference.semantic_name.as_str())
        .collect()
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 815);
    assert_eq!(SELECTED_OPERAND_CARDINALITY, "2..N");
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(
        PREVIOUS_ONE_REFERENCE_ONE_PLAIN_DECIMAL_ADDITIVE_ORACLE_SOURCE
            .contains("ISSUE_ID: u64 = 789")
    );
    assert!(
        PREVIOUS_ONE_REFERENCE_ONE_PLAIN_DECIMAL_ADDITIVE_ORACLE_SOURCE.contains(
            "heterogeneous exactly-two-operand, one-IdentifierReference, one-plain-Decimal-atom additive initializer frontier only"
        )
    );
    assert!(
        PREVIOUS_PLAIN_IDENTIFIER_REFERENCE_ADDITIVE_CHAIN_ORACLE_SOURCE
            .contains("ISSUE_ID: u64 = 801")
    );
    assert!(
        PREVIOUS_PLAIN_IDENTIFIER_REFERENCE_ADDITIVE_CHAIN_ORACLE_SOURCE
            .contains("ordered 2..N plain-IdentifierReference additive-chain frontier only")
    );
    assert!(FRONTIER_SCOPE_NOTE.contains(
        "ordered 2..N heterogeneous IdentifierReference/plain-Decimal additive-chain frontier only"
    ));

    for forbidden in [
        concat!("use super::super::", "selected_lexical_slice"),
        concat!("use super::super::", "selected_static_semantics"),
        concat!("use super::super::", "selected_qualification_integration"),
        concat!(
            "use super::super::",
            "selected_variable_statement_name_correspondence"
        ),
        concat!("use super::super::", "selected_binding_scope"),
        concat!(
            "use super::super::",
            "selected_one_level_block_binding_scope"
        ),
        concat!("use super::super::", "selected_binding_identifier"),
        concat!("use super::super::", "qualification;"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!(
            "recognize_selected_one_reference_one_plain_decimal_",
            "additive_initializer("
        ),
        concat!(
            "recognize_selected_plain_identifier_reference_",
            "additive_chain("
        ),
        concat!("consume_selected_", "identifier_reference"),
        concat!("analyze_selected_binding_", "scope("),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_selected_block_var_", "statement"),
        concat!("parse_unary_", "expression"),
        concat!("parse_additive_", "expression"),
        // Candidate independence for trivia recognition specifically: only
        // the stable normative Unicode `Space_Separator` property primitive
        // is reused, never the production selected-trivia recognizer.
        concat!("is_selected_", "trivia("),
        concat!("skip_selected_", "trivia("),
        concat!(
            "use super::super::",
            "selected_lexical_slice::is_selected_trivia"
        ),
        // No cross-Oracle import: this leaf restates rather than imports.
        concat!(
            "use super::qualification_selected_one_reference_one_plain_",
            "decimal_additive_initializer_validation_tests"
        ),
        concat!(
            "use super::selected_plain_identifier_reference_",
            "additive_chain_frontier"
        ),
        // Forbidden reconstruction of expected ranges/endpoints.
        concat!(".", "find("),
        concat!(".", "rfind("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // `is_id_start` / `is_id_continue` / `is_space_separator` are stable,
    // frozen normative Unicode 17 primitives this oracle is independently
    // entitled to consult, already used the same way by every predecessor
    // accepted oracle at the same `super::super::unicode` path.
    assert!(
        THIS_ORACLE_SOURCE.contains(
            "use super::super::unicode::{is_id_continue, is_id_start, is_space_separator}"
        )
    );
}

/// Genuine iterative model, not an unrolled fixed-cardinality theorem: the
/// recognizer is defined with a real `loop`, never named
/// `first`/`second`/`third`/`fourth` fields, and no hardcoded four-or-five
/// operand cap exists anywhere in the source (W2).
#[test]
fn recognition_is_a_genuine_loop_never_an_unrolled_fixed_cardinality_theorem() {
    assert!(THIS_ORACLE_SOURCE.contains("loop {"));
    for forbidden in [
        concat!("four", "th: RecognizedReference"),
        concat!("fif", "th: RecognizedReference"),
        concat!("struct", " Four"),
        concat!("struct", " Five"),
        concat!("Operand", "Four"),
        concat!("Operand", "Five"),
        concat!("enum ", "ExactlyFour"),
        concat!("operand_count", " == 4"),
        concat!("operand_count", " == 5"),
        concat!("operand_count", " >= 4 &&"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }
    assert!(THIS_ORACLE_SOURCE.contains("operand_count >= 2"));
}

/// Required cardinality witnesses (2, 3, 4, 5, and a long 8-operand
/// sentinel), each genuinely heterogeneous, all recognize with the exact
/// syntax operand count and exact ordered reference names. The 8-operand
/// sentinel exists to falsify an accidental fixed cap, not as a stress
/// benchmark.
#[test]
fn cardinality_witnesses_from_two_through_a_long_sentinel_all_recognize() {
    let two = recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1")
        .expect("a+1 must recognize the theorem");
    assert_eq!(two.operand_count, 2);
    assert_eq!(names(&two), ["a"]);

    let three = recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1+b")
        .expect("a+1+b must recognize the theorem");
    assert_eq!(three.operand_count, 3);
    assert_eq!(names(&three), ["a", "b"]);

    let four = recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1+b+2")
        .expect("a+1+b+2 must recognize the theorem");
    assert_eq!(four.operand_count, 4);
    assert_eq!(names(&four), ["a", "b"]);

    let five = recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1+b+2+c")
        .expect("a+1+b+2+c must recognize the theorem");
    assert_eq!(five.operand_count, 5);
    assert_eq!(names(&five), ["a", "b", "c"]);

    let long_sentinel =
        recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1+b+2+c+3+d+4")
            .expect("a+1+b+2+c+3+d+4 must recognize the theorem");
    assert_eq!(long_sentinel.operand_count, 8);
    assert_eq!(names(&long_sentinel), ["a", "b", "c", "d"]);

    let long_sentinel_other_order =
        recognize_selected_heterogeneous_reference_decimal_additive_chain("1+a+2+b+3+c+4+d")
            .expect("1+a+2+b+3+c+4+d must recognize the theorem");
    assert_eq!(long_sentinel_other_order.operand_count, 8);
    assert_eq!(names(&long_sentinel_other_order), ["a", "b", "c", "d"]);
}

/// Required positive matrix (acceptance criteria matrix from Issue #815):
/// exactly-two both orientations, three-operand family, one-reference-only,
/// multiple silent Decimal positions, interleaved forms, 4/5/8 operands,
/// escaped references, duplicates, and exponent-internal-sign chains all
/// recognize with the exact syntax operand count and exact ordered reference
/// names (W6).
#[test]
fn required_positive_matrix_pins_operand_count_and_ordered_reference_names() {
    struct Case {
        candidate: &'static str,
        expected_operand_count: usize,
        expected_reference_names: &'static [&'static str],
    }

    let cases = [
        Case {
            candidate: "a+1",
            expected_operand_count: 2,
            expected_reference_names: &["a"],
        },
        Case {
            candidate: "1+a",
            expected_operand_count: 2,
            expected_reference_names: &["a"],
        },
        Case {
            candidate: "a+1+b",
            expected_operand_count: 3,
            expected_reference_names: &["a", "b"],
        },
        Case {
            candidate: "1+a+2",
            expected_operand_count: 3,
            expected_reference_names: &["a"],
        },
        Case {
            candidate: "a+b+1",
            expected_operand_count: 3,
            expected_reference_names: &["a", "b"],
        },
        Case {
            candidate: "1+a+b",
            expected_operand_count: 3,
            expected_reference_names: &["a", "b"],
        },
        Case {
            candidate: "1+2+a+3+4",
            expected_operand_count: 5,
            expected_reference_names: &["a"],
        },
        Case {
            candidate: "a+1+2+b+3",
            expected_operand_count: 5,
            expected_reference_names: &["a", "b"],
        },
        Case {
            candidate: "a+1+b+2+c",
            expected_operand_count: 5,
            expected_reference_names: &["a", "b", "c"],
        },
        Case {
            candidate: "a+1+b+2+c+3+d+4",
            expected_operand_count: 8,
            expected_reference_names: &["a", "b", "c", "d"],
        },
        Case {
            candidate: "1+a+2+b+3+c+4+d",
            expected_operand_count: 8,
            expected_reference_names: &["a", "b", "c", "d"],
        },
        Case {
            candidate: concat!("1+", "\\", "u0061", "+2"),
            expected_operand_count: 3,
            expected_reference_names: &["a"],
        },
        Case {
            candidate: concat!("a+1+b", "\\", "u0061", "r+2"),
            expected_operand_count: 4,
            expected_reference_names: &["a", "bar"],
        },
        Case {
            candidate: "a+1+a+2+a",
            expected_operand_count: 5,
            expected_reference_names: &["a", "a", "a"],
        },
        Case {
            candidate: "a+1e-2+b",
            expected_operand_count: 3,
            expected_reference_names: &["a", "b"],
        },
        Case {
            candidate: "1e+2+a+3",
            expected_operand_count: 3,
            expected_reference_names: &["a"],
        },
        Case {
            candidate: "a+1e-2+b-3e+4+c",
            expected_operand_count: 5,
            expected_reference_names: &["a", "b", "c"],
        },
    ];

    for case in cases {
        let evidence =
            recognize_selected_heterogeneous_reference_decimal_additive_chain(case.candidate)
                .unwrap_or_else(|| panic!("{:?} must recognize the theorem", case.candidate));
        assert_eq!(
            evidence.operand_count, case.expected_operand_count,
            "{:?}",
            case.candidate
        );
        assert_eq!(
            names(&evidence).as_slice(),
            case.expected_reference_names,
            "{:?}",
            case.candidate
        );
        assert!(
            evidence.references.len() < evidence.operand_count,
            "{:?}",
            case.candidate
        );
    }
}

/// Required positive matrix with fixture-owned literal byte ranges: every
/// retained reference fact's authored range comes from the fixture's own
/// literal source, never derived by search, rescan, or reparse; every
/// skipped Decimal position is independently confirmed as a whole accepted
/// Decimal atom without ever being retained (W1/W7/W25).
#[test]
fn positive_matrix_pins_source_backed_reference_facts_with_exact_provenance() {
    struct Fixture {
        text: &'static str,
        whole: Range,
        reference_ranges: &'static [Range],
        reference_names: &'static [&'static str],
        decimal_ranges: &'static [Range],
        decimal_atoms: &'static [&'static str],
        expected_operand_count: usize,
    }

    let fixtures = [
        Fixture {
            text: "const x = a + 1 + b + 2 + c;",
            whole: Range(10, 27),
            reference_ranges: &[Range(10, 11), Range(18, 19), Range(26, 27)],
            reference_names: &["a", "b", "c"],
            decimal_ranges: &[Range(14, 15), Range(22, 23)],
            decimal_atoms: &["1", "2"],
            expected_operand_count: 5,
        },
        Fixture {
            text: "const x = 1 + a + 2 + b;",
            whole: Range(10, 23),
            reference_ranges: &[Range(14, 15), Range(22, 23)],
            reference_names: &["a", "b"],
            decimal_ranges: &[Range(10, 11), Range(18, 19)],
            decimal_atoms: &["1", "2"],
            expected_operand_count: 4,
        },
    ];

    for (fixture_index, fixture) in fixtures.iter().enumerate() {
        let whole_text = slice(fixture.text, fixture.whole);
        let evidence =
            recognize_selected_heterogeneous_reference_decimal_additive_chain(whole_text)
                .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(evidence.operand_count, fixture.expected_operand_count);
        assert_eq!(evidence.references.len(), fixture.reference_ranges.len());

        for (index, (range, expected_name)) in fixture
            .reference_ranges
            .iter()
            .zip(fixture.reference_names)
            .enumerate()
        {
            assert_eq!(slice(fixture.text, *range), *expected_name);
            assert_eq!(evidence.references[index].authored, *expected_name);
            assert_eq!(evidence.references[index].semantic_name, *expected_name);
            assert_eq!(
                evidence.references[index].spelling,
                OperandSpellingKind::Direct
            );
            assert_eq!(
                authored_anchor(
                    815_000 + fixture_index as u64 * 10 + index as u64,
                    fixture.text,
                    *range
                ),
                *expected_name
            );
        }

        for (index, (range, expected_atom)) in fixture
            .decimal_ranges
            .iter()
            .zip(fixture.decimal_atoms)
            .enumerate()
        {
            assert_eq!(slice(fixture.text, *range), *expected_atom);
            assert!(is_selected_accepted_plain_decimal_atom(expected_atom));
            assert_eq!(
                authored_anchor(
                    815_500 + fixture_index as u64 * 10 + index as u64,
                    fixture.text,
                    *range
                ),
                *expected_atom
            );
        }
    }
}

/// Central Issue #815 divergence theorem at both extremes: syntax operand
/// cardinality is never assumed equal to retained semantic-evidence
/// cardinality, in either direction.
#[test]
fn syntax_and_semantic_cardinality_diverge_at_both_extremes() {
    let one_reference_only =
        recognize_selected_heterogeneous_reference_decimal_additive_chain("1+2+3+a+4+5")
            .expect("1+2+3+a+4+5 must recognize the theorem");
    assert_eq!(one_reference_only.operand_count, 6);
    assert_eq!(names(&one_reference_only), ["a"]);

    let mostly_references =
        recognize_selected_heterogeneous_reference_decimal_additive_chain("a+b+c+1+d+e")
            .expect("a+b+c+1+d+e must recognize the theorem");
    assert_eq!(mostly_references.operand_count, 6);
    assert_eq!(names(&mostly_references), ["a", "b", "c", "d", "e"]);
}

/// Ordering and duplicate theorem: authored left-to-right order is
/// load-bearing and never reordered by semantic name, and equal semantic
/// names are never deduplicated into fewer facts.
#[test]
fn authored_order_is_preserved_and_equal_semantic_names_are_never_deduplicated() {
    let reordered = recognize_selected_heterogeneous_reference_decimal_additive_chain("c+1+a+2+b")
        .expect("c+1+a+2+b must recognize the theorem");
    assert_eq!(names(&reordered), ["c", "a", "b"]);

    let all_duplicate =
        recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1+a+2+a")
            .expect("a+1+a+2+a must recognize the theorem");
    assert_eq!(all_duplicate.references.len(), 3);
    for reference in &all_duplicate.references {
        assert_eq!(reference.semantic_name, "a");
        assert_eq!(reference.authored, "a");
    }
}

/// Escaped operands at first, interior, and final positions preserve
/// authored spelling distinct from decoded semantic name, in an otherwise
/// heterogeneous chain (W11).
#[test]
fn escaped_references_at_first_interior_and_final_positions_preserve_authored_identity() {
    const ESCAPED_A: &str = concat!("\\", "u0061");
    const ESCAPED_B: &str = concat!("\\", "u0062");
    const ESCAPED_C: &str = concat!("\\", "u0063");
    const FIRST_SOURCE: &str = concat!("\\", "u0061", "+1+b");
    const INTERIOR_SOURCE: &str = concat!("a+1+", "\\", "u0062", "+2");
    const LAST_SOURCE: &str = concat!("a+1+b+2+", "\\", "u0063");

    let first = recognize_selected_heterogeneous_reference_decimal_additive_chain(FIRST_SOURCE)
        .expect("escaped-first candidate must recognize the theorem");
    assert_eq!(first.operand_count, 3);
    assert_eq!(first.references[0].authored, ESCAPED_A);
    assert_eq!(first.references[0].semantic_name, "a");
    assert_eq!(
        first.references[0].spelling,
        OperandSpellingKind::EscapedNonReserved
    );
    assert_ne!(
        first.references[0].authored,
        first.references[0].semantic_name
    );

    let interior =
        recognize_selected_heterogeneous_reference_decimal_additive_chain(INTERIOR_SOURCE)
            .expect("escaped-interior candidate must recognize the theorem");
    assert_eq!(interior.operand_count, 4);
    assert_eq!(names(&interior), ["a", "b"]);
    assert_eq!(interior.references[1].authored, ESCAPED_B);
    assert_eq!(
        interior.references[1].spelling,
        OperandSpellingKind::EscapedNonReserved
    );

    let last = recognize_selected_heterogeneous_reference_decimal_additive_chain(LAST_SOURCE)
        .expect("escaped-final candidate must recognize the theorem");
    assert_eq!(last.operand_count, 5);
    assert_eq!(names(&last), ["a", "b", "c"]);
    assert_eq!(last.references[2].authored, ESCAPED_C);
    assert_eq!(
        last.references[2].spelling,
        OperandSpellingKind::EscapedNonReserved
    );
}

/// Selected additive trivia composes at every internal boundary in a
/// heterogeneous chain, and comments never compose as operand trivia at any
/// boundary.
#[test]
fn selected_additive_trivia_matrix_across_every_internal_boundary() {
    for candidate in [
        "a+1+b+2",
        "a + 1 + b + 2",
        "a\t+\t1\t+\tb\t+\t2",
        "a\t+\u{00A0}1\u{2028}-\u{2029}b\u{FEFF}+\u{000B}2",
    ] {
        let evidence = recognize_selected_heterogeneous_reference_decimal_additive_chain(candidate)
            .unwrap_or_else(|| panic!("{candidate:?} must recognize the theorem"));
        assert_eq!(names(&evidence), ["a", "b"]);
    }

    for comment_control in [
        "a/*c*/+1+b+2",
        "a+/*c*/1+b+2",
        "a+1/*c*/+b+2",
        "a+1+/*c*/b+2",
        "a+1+b/*c*/+2",
        "a+1+b+/*c*/2",
    ] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(comment_control)
                .is_none(),
            "{comment_control:?}"
        );
    }

    assert!(!is_selected_additive_trivia('\u{200B}'));
    assert!(is_selected_additive_trivia('\u{00A0}'));
    assert!(
        recognize_selected_heterogeneous_reference_decimal_additive_chain("a+\u{200B}1+b")
            .is_none()
    );
}

/// Outer trivia firewall: leading and trailing statement/declaration-level
/// trivia does not belong to the naked theorem and is never silently
/// absorbed.
#[test]
fn outer_leading_and_trailing_trivia_remain_outside_the_naked_theorem() {
    assert!(recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1+b").is_some());
    for control in [" a+1+b", "a+1+b ", " a+1+b ", "\ta+1+b", "a+1+b\n"] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
}

/// Whole-candidate transactionality and no-prefix-truncation: an incomplete
/// or over-extended chain never publishes a prepared prefix, whatever
/// otherwise-valid-looking prefix precedes the failure.
#[test]
fn whole_candidate_transactionality_never_commits_a_partial_or_truncated_result() {
    for control in [
        "a+1+b+",
        concat!("a+1+b+", "true"),
        concat!("a+1+b+", "\\", "u0069f"),
        concat!("a+1+b+", "\\", "u{}"),
        "a+1+b.c",
        "a+1+b()",
        "a+1+b=c",
        "a+1+b?c:d",
        "1+2+a+3+4+",
        concat!("1+2+a+3+", "true"),
    ] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }

    assert!(recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1+b").is_some());
    assert!(
        recognize_selected_heterogeneous_reference_decimal_additive_chain("1+2+a+3+4").is_some()
    );
}

/// Decimal exponent-sign ownership matrix: an exponent-internal `+`/`-`
/// remains owned by the Decimal atom and distinct from a following binary
/// additive operator, at every position in the chain, including a candidate
/// with two independently owned exponent signs (section 7/14).
#[test]
fn decimal_exponent_sign_ownership_matrix_distinguishes_from_binary_operator() {
    for atom in ["1e-2", "1e+2", "3e+4"] {
        assert!(is_selected_accepted_plain_decimal_atom(atom), "{atom:?}");
    }

    let internal_minus =
        recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1e-2+b")
            .expect("a+1e-2+b must recognize the theorem");
    assert_eq!(internal_minus.operand_count, 3);
    assert_eq!(names(&internal_minus), ["a", "b"]);

    let internal_plus =
        recognize_selected_heterogeneous_reference_decimal_additive_chain("1e+2+a+3")
            .expect("1e+2+a+3 must recognize the theorem");
    assert_eq!(internal_plus.operand_count, 3);
    assert_eq!(names(&internal_plus), ["a"]);

    let multiple_exponent_sign_ownership =
        recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1e-2+b-3e+4+c")
            .expect("a+1e-2+b-3e+4+c must recognize the theorem");
    assert_eq!(multiple_exponent_sign_ownership.operand_count, 5);
    assert_eq!(names(&multiple_exponent_sign_ownership), ["a", "b", "c"]);
}

/// Invalid exponent boundary firewall: a structurally invalid exponent sign
/// never leaks a shorter accepted numeric prefix, and the failed Decimal
/// candidate never reinterprets its own owned sign as a binary operator
/// (section 8; W9/W10).
#[test]
fn invalid_exponent_boundary_never_reinterprets_owned_sign_as_binary_operator() {
    for control in ["1e++a", "1e--a", "a+1e+", "a+1e-", "a+1e++b", "a+1e--b"] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
}

/// All-reference and all-Decimal firewalls: chains composed entirely of one
/// family remain outside this exact heterogeneous theorem, owned instead by
/// already-accepted #801 (all-reference) or remaining unowned (all-Decimal)
/// authority (W4/W5).
#[test]
fn all_reference_firewall_and_all_decimal_firewall_remain_outside_the_theorem() {
    for control in ["a+b", "a+b+c", "a-b+c-d"] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
    for control in ["1+2", "1+2+3", "1e-2+3+4"] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
    assert!(recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1").is_some());
}

/// Optional unary firewall: leading, right-unary, doubled-punctuator, and
/// keyword-unary (`!`/`~`/`typeof`/`void`/`delete`) forms all remain outside
/// this plain-heterogeneous theorem, at any boundary, despite already-
/// accepted unary-wrapper production existing elsewhere (section 17; W3).
#[test]
fn optional_unary_firewall_keeps_signed_and_keyword_unary_forms_outside() {
    for control in [
        "+a+1+b",
        "-a+1+b",
        "a+-b+1",
        "a-+b+1",
        "a+-1+b",
        "a-+1+b",
        "a+1+-b",
        "a+1-+b",
        "-1+a+2",
        "+1+a+2",
        "1+-a+2",
        "1-+a+2",
        "!a+1",
        "~a+1",
        "typeof a+1",
        "void a+1",
        "delete a+1",
    ] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
}

/// `IdentifierReference` firewalls: escaped `ReservedWord`, malformed
/// escapes, invalid escaped positions, and non-`CodePoint` values reject the
/// whole heterogeneous candidate at the first, an interior, and the final
/// position, without leaking a sibling reference or Decimal fact (section
/// 18; W11/W12).
#[test]
fn identifier_reference_firewalls_reject_malformed_reserved_and_invalid_escapes_at_any_position() {
    for control in [
        concat!("\\", "u0069f+1+a"),
        concat!("1+", "\\", "u0069f+a"),
        concat!("1+a+", "\\", "u0069f"),
        concat!("\\", "u{}+1+a"),
        concat!("1+", "\\", "u{}+a"),
        concat!("1+a+", "\\", "u{}"),
        concat!("\\", "u{110000}+1+a"),
        concat!("1+", "\\", "u{110000}+a"),
        concat!("\\", "uD800+1+a"),
        concat!("1+", "\\", "uD800+a"),
        concat!("\\", "u0030+1+a"),
        concat!("1+", "\\", "u0030+a"),
    ] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }

    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0069f")),
        Err(DecodeFailure::DecodedReserved)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u{}")),
        Err(DecodeFailure::MalformedEscape)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u{110000}")),
        Err(DecodeFailure::NonCodePoint)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "uD800")),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0030")),
        Err(DecodeFailure::InvalidStart)
    );
}

/// Numeric frontier firewall: `NumericLiteralSeparator`, radix-prefixed
/// integers, and `BigIntLiteral` forms never leak a shorter accepted
/// prefix, whichever position they occupy in the chain (section 19; W14).
#[test]
fn numeric_frontier_firewall_remains_unowned_at_any_position() {
    for control in [
        "a+1_0+b", "a+0x10+b", "a+1n+b", "1_0+a+2", "0x10+a+2", "1n+a+2",
    ] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
}

/// Other operand family firewall: Boolean, `null`, `this`, and
/// string-literal operands remain outside this theorem at any position
/// (section 20; W15).
#[test]
fn other_operand_family_firewall_keeps_boolean_null_this_and_string_outside() {
    for control in [
        "a+true+b",
        "true+a+1",
        "a+null+b",
        "null+a+1",
        "this+a+1",
        "a+this+1",
        "\"a\"+b+1",
        "a+\"x\"+b",
    ] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
}

/// Richer expression / precedence / grouping / comment firewall: grouping,
/// member access, call, multiplicative precedence, assignment, conditional,
/// comma, and comment tails all remain outside without any dedicated
/// grouping/member/call/precedence/comment recognition code (section 21/22;
/// W16/W17).
#[test]
fn richer_expression_precedence_grouping_and_comment_firewall_remains_unowned() {
    for control in [
        "(a)+1+b",
        "a+(1)+b",
        "a*1+b",
        "a+1*b",
        "a.b+1",
        "a+1+b.c",
        "a()+1",
        "a+1+b()",
        "a=1+b",
        "a+1=b",
        "a+1?b:c",
        "a+1,b",
        "a/*x*/+1+b",
        "a+/*x*/1+b",
        "a+1/*x*/+b",
        "a+1+/*x*/b",
    ] {
        assert!(
            recognize_selected_heterogeneous_reference_decimal_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
    assert!(recognize_selected_heterogeneous_reference_decimal_additive_chain("a+1+b").is_some());
}

/// Resource / failure semantics: the project's
/// `ResourceLimited`/`InternalFailure` lifecycle states remain distinct from
/// `UnsupportedCoverage`, matching the minimal symbolic model already
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
/// operator enum, orientation enum, Decimal evidence field, operator/Decimal/
/// whole-expression `SourceAnchor`, AST/CST, token tape, runtime/static
/// vocabulary, or frozen production collection-representation choice is
/// introduced.
#[test]
fn handoff_remains_validation_only_with_no_forbidden_retained_representation() {
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
        concat!("decimal", ": RecognizedReference"),
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

    assert!(THIS_ORACLE_SOURCE.contains("struct HeterogeneousAdditiveChainEvidence"));
    assert!(THIS_ORACLE_SOURCE.contains("references: Vec<RecognizedReference"));
    assert!(THIS_ORACLE_SOURCE.contains("operand_count: usize"));
    assert!(THIS_ORACLE_SOURCE.contains("ORACLE DYNAMIC STORAGE"));
    assert!(THIS_ORACLE_SOURCE.contains("PRODUCTION STORAGE AUTHORITY"));
    assert!(THIS_ORACLE_SOURCE.contains("references.len() < operand_count"));
    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}

/// Completion hard-zero: this validation-only leaf adds zero production
/// capability, so it must not alter the frozen `193 / 10 / 183` rule-unit
/// partition. The authoritative count assertions already live in
/// `qualification_validation_tests::mod`
/// (`frozen_rule_counts_match_the_independent_per_container_universe`); this
/// is a focused local restatement of the same frozen totals so this leaf
/// carries its own explicit non-movement evidence.
#[test]
fn completion_partition_remains_the_frozen_193_10_183_totals() {
    const EXPECTED_TOTAL_RULE_UNITS: usize = 193;
    const EXPECTED_ENVELOPE_INACTIVE_RULE_UNITS: usize = 10;
    const EXPECTED_ACTIVE_RULE_UNITS: usize = 183;
    assert_eq!(
        EXPECTED_ACTIVE_RULE_UNITS + EXPECTED_ENVELOPE_INACTIVE_RULE_UNITS,
        EXPECTED_TOTAL_RULE_UNITS
    );
    assert_eq!(RULE_UNITS.len(), EXPECTED_TOTAL_RULE_UNITS);
    // Built with `concat!` so the assembled forbidden literal does not
    // appear as one contiguous run inside this very assertion.
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("RuleUnitKind::", "NormativeRule")));
}
