//! Candidate-independent ordered 2..N optional-leading-`+`/`-`
//! `IdentifierReference` x heterogeneous plain-Decimal additive-chain
//! fact-preservation validation for Issue #821 (durable research: Issue #688
//! comment `5816917566`; accepted predecessor authorities: Issue #809 / PR
//! #810 candidate-independent ordered 2..N optional-leading-`+`/`-`
//! `IdentifierReference` additive-chain Oracle; Issue #815 / PR #816
//! candidate-independent ordered 2..N heterogeneous plain-`IdentifierReference`
//! / plain-Decimal additive-chain Oracle). Latest heterogeneous production
//! predecessor #819 / PR #820 exists but is explicitly NOT expected-answer
//! authority for this Oracle.
//!
//! This oracle qualifies only the combined expression-composition family:
//!
//! ```text
//! SelectedOptionalPlusMinusHeterogeneousReferenceDecimalAdditiveChain ::=
//!     SelectedCombinedOperand
//!     (
//!         SelectedAdditiveContinuation
//!         SelectedCombinedOperand
//!     )+
//!
//! SelectedCombinedOperand ::=
//!       SelectedAcceptedPlainDecimalAtom
//!     | SelectedOptionalPlusMinusIdentifierReferenceOperand
//!
//! SelectedOptionalPlusMinusIdentifierReferenceOperand ::=
//!       SelectedAcceptedIdentifierReference
//!     | SelectedUnaryPlusMinus
//!       SelectedUnaryOperandTrivia
//!       SelectedAcceptedIdentifierReference
//!
//! SelectedUnaryPlusMinus ::= "+" | "-"
//! ```
//!
//! with whole-candidate acceptance additionally requiring `operand_count >=
//! 2`, at least one retained `IdentifierReference` fact, and at least one
//! recognized (but never retained) plain Decimal atom, over the entire naked
//! candidate.
//!
//! ## Load-bearing novelty: three distinct `+`/`-` ownership classes
//!
//! Neither #809 nor #815 independently proves this combined theorem. #809
//! proves that an arbitrary-`N` additive chain may compose plain and
//! optionally leading-`+`/`-`-wrapped `IdentifierReference` operands at every
//! position, but it explicitly keeps Decimal/numeric operands outside its
//! theorem. #815 proves that an arbitrary-`N` additive chain may compose
//! plain `IdentifierReference` and plain Decimal atom operands
//! heterogeneously at every position, but it explicitly keeps optional
//! leading unary operands outside its theorem. Their composition requires
//! distinguishing, at arbitrary positions and arbitrary finite cardinality,
//! three distinct `+`/`-` ownership classes: binary additive `+`/`-`,
//! optional leading unary `+`/`-` owned by an `IdentifierReference` operand,
//! and exponent-internal `+`/`-` owned by a Decimal atom. For example
//! `1e-2+-a` must decompose as one Decimal atom `1e-2` (its `-` exponent-
//! owned), one binary additive `+`, and one unary `-` wrapping the
//! `IdentifierReference` `a` -- no sign may migrate between ownership classes
//! after recognition. This Oracle proves exactly that composed theorem, and
//! nothing more, matching the accepted composition precedent already set by
//! Issue #809/#810 (which itself composed Issue #777's bounded punctuator-
//! boundary theorem with Issue #801's arbitrary-`N` continuation theorem).
//!
//! `recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain`
//! is a genuine `loop`, never an unrolled fixed sequence of named
//! `first`/`second`/`third`/`fourth` fields, and is deliberately exercised
//! beyond four and five operands (an eight-operand sentinel below) precisely
//! so an accidental fixed cap cannot silently satisfy the suite.
//!
//! ## Operand family dispatch, unary-sign eligibility, and Decimal
//! exponent-sign ownership
//!
//! `recognize_combined_operand` dispatches purely syntactically, exactly as
//! Issue #815 already established: an operand is attempted as
//! `SelectedAcceptedPlainDecimalAtom` only when its first authored byte is an
//! ASCII digit or `.`; every other first byte -- including `+` and `-` --
//! attempts `SelectedOptionalPlusMinusIdentifierReferenceOperand` via
//! `recognize_optional_signed_operand`, independently restated unchanged from
//! Issue #809. Because dispatch happens before any optional leading sign is
//! consumed, an operand beginning with `+`/`-` immediately followed by an
//! ASCII digit (`+1`, `-1`) is *never* attempted as Decimal: it is attempted
//! only as a signed `IdentifierReference` operand, and since
//! `operand_interior_run_end` requires the character(s) after the consumed
//! sign to form an accepted `IdentifierReference` body -- and a bare ASCII
//! digit never does -- such an operand structurally fails, keeping signed
//! Decimal outside this theorem without any dedicated signed-Decimal check.
//! `decimal_atom_run_end` independently restates, unchanged, Issue #789's
//! (via Issue #815's restatement) bounded left-to-right forward scan for the
//! Decimal family: it unconditionally extends over an ASCII digit, `.`, `e`,
//! or `E`, and extends over a `+`/`-` byte *only* when that byte immediately
//! follows a just-consumed `e`/`E` byte -- the sole rule that keeps an
//! exponent-internal sign owned by the Decimal atom distinct from a
//! following binary additive operator or a following unary sign, without
//! rescanning, without a general tokenizer, and without ever looking beyond
//! the one byte just consumed. A structurally invalid Decimal run (for
//! example `1e+` with nothing after the exponent sign) fails the whole
//! candidate outright via `?` and is never reinterpreted as a shorter
//! accepted numeric prefix followed by a binary/unary operator.
//!
//! The accepted Issue #777/#809 `SelectedBinaryUnaryBoundary` rule --
//! same-sign binary/unary adjacency requires non-empty separating trivia;
//! opposite-sign adjacency is accepted whether or not trivia separates them
//! -- is applied independently at every continuation, but *only* when the
//! continuation operand resolves to the `IdentifierReference` family and
//! itself carries a leading sign. A Decimal-family continuation operand never
//! carries its own leading sign (dispatch already routed any `+`/`-`-first
//! continuation content to the reference family), so no boundary check
//! applies when the continuation is a plain Decimal atom: the intervening
//! trivia composes freely as ordinary `SelectedAdditiveContinuation` trivia,
//! exactly as Issue #815 already established for its own bounded/unbounded
//! Decimal continuations.
//!
//! `operand_interior_run_end` is the same bounded, single left-to-right
//! forward pass already established by Issue #752/#795/#801/#809 (consuming
//! one direct code point or one syntactically well-formed
//! `UnicodeEscapeSequence` element at a time, stopping only at the first
//! selected-trivia code point or at `+`/`-`), invoked once per
//! `IdentifierReference`-family operand's identifier body. This is never a
//! general tokenizer: any other character (`.`, `(`, `=`, `,`, `?`, and so
//! on) is silently absorbed into the same run and is judged afterward, at
//! whole-operand-slice granularity, by `recognize_accepted_identifier_reference`.
//! This is never `.find`/`.rfind`, later substring search, source rescanning
//! after recognition, reparse, retokenization, operator-relative endpoint
//! reconstruction, or decoded-length endpoint inference.
//!
//! `SelectedAdditiveTrivia`-family functions, `SelectedDirectIdentifierReference`,
//! `SelectedEscapedNonReservedIdentifierReference`, and the plain Decimal
//! atom grammar (`SelectedDecimalInteger`, `SelectedPlainFractionalDecimalLiteral`,
//! `SelectedPlainExponentDecimalLiteral`) independently restate the same
//! already-accepted primitives Issue #809 and Issue #815 each independently
//! restate. Neither restatement imports another Oracle's code: this leaf owns
//! its own copy of every already-accepted primitive it consumes, matching the
//! convention already set by every predecessor listed above.
//!
//! ## What is retained, and what is not
//!
//! The Oracle's only retained representation is
//! `CombinedAdditiveChainEvidence`, holding an ordered
//! `Vec<RecognizedReference>` of every `IdentifierReference`-family operand's
//! own exact authored `SourceAnchor`/fragment, decoded semantic name, and
//! Direct/EscapedNonReserved spelling state (in exact authored left-to-right
//! order, duplicates never deduplicated), plus the total `operand_count`
//! across both families. This is a minimal private test-only shape chosen
//! only because the theorem itself is `2..N` and load-bearingly
//! heterogeneous, matching the shape Issue #815 already established:
//!
//! ```text
//! ORACLE DYNAMIC STORAGE (private Vec<RecognizedReference> + operand_count)
//! !=
//! PRODUCTION STORAGE AUTHORITY
//! ```
//!
//! It explicitly does **not** freeze future production representation, does
//! not retain the leading unary sign (which is consumed and discarded by
//! `recognize_optional_signed_operand` before a `RecognizedReference` is
//! built -- an authored `-a` retains only `a`, never `-a`), does not retain
//! any Decimal `SourceAnchor`, Decimal literal kind, or Decimal value, does
//! not retain a binary-operator enum or `SourceAnchor`, does not retain
//! orientation, does not retain a whole-expression `SourceAnchor`, and
//! performs no runtime evaluation. `references.len() < operand_count` is the
//! direct, by-construction proof that at least one Decimal operand existed
//! without retaining Decimal evidence merely to prove heterogeneity.
//!
//! This is a validation-only leaf: production supports no combined
//! optional-leading-`+`/`-` `IdentifierReference` / heterogeneous-Decimal
//! arbitrary-cardinality family at the #821 baseline, so every positive
//! fixture below remains `UnsupportedCoverage` under current production and
//! exists only as independent Oracle evidence for a future, separately
//! authorized production decision. No completion successor file accompanies
//! this leaf: this Issue adds zero production capability, so the frozen `193
//! / 10 / 183 / {}` completion partition cannot move, matching the precedent
//! set by #742/#743, #746/#747, #752/#753, #789/#790, #795/#796, #801/#802,
//! #809/#810, and #815/#816.

use crate::{SourceId, SourceText};

use super::super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};
use super::inventory::RULE_UNITS;

const ISSUE_ID: u64 = 821;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("model.rs");
const PREVIOUS_OPTIONAL_PLUS_MINUS_IDENTIFIER_REFERENCE_ADDITIVE_CHAIN_ORACLE_SOURCE: &str =
    include_str!("selected_optional_plus_minus_identifier_reference_additive_chain_frontier.rs");
const PREVIOUS_HETEROGENEOUS_REFERENCE_DECIMAL_ADDITIVE_CHAIN_ORACLE_SOURCE: &str =
    include_str!("selected_heterogeneous_reference_decimal_additive_chain_frontier.rs");
const THIS_ORACLE_SOURCE: &str = include_str!(
    "selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain_frontier.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "optional-leading-+/- IdentifierReference x heterogeneous plain-Decimal ",
    "additive-chain 2..N frontier only; later independently qualified owners ",
    "may strengthen classification for recursive unary, both-unary ",
    "composition beyond exactly-one wrapper, signed Decimal, other unary ",
    "operator families, other non-IdentifierReference/non-Decimal operands, ",
    "numeric separator/radix/BigInt forms, parenthesized/member/call/richer- ",
    "expression operands, comments, general ASI, or production ",
    "representation/placement"
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
// #237/#752/#777/#789/#801/#809/#815, never imported from any of those
// Oracles. ---

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
// #742/#743/#746/#747/#752/#777/#789/#801/#809/#815. Reused unchanged at
// every trivia position this theorem owns. ---

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
// #727/#735/#789/#815, never imported from any of those Oracles. ---

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
/// mistaken for a following binary additive operator or unary sign. Stops
/// at the first byte that cannot extend a plain Decimal atom shape.
/// Position/grammar validity of the collected run is judged afterward by
/// `is_selected_accepted_plain_decimal_atom`, never here -- this scan never
/// rejects, rescans, or looks beyond the one byte just consumed.
/// Independently restated, unchanged, from Issue #789/#815's
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

// --- The Issue #821 theorem itself. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperandSpellingKind {
    Direct,
    EscapedNonReserved,
}

/// One retained `IdentifierReference`-family operand fact. A private
/// test-only shape, not a claim about future production storage. The
/// leading unary sign, when one was present, is never part of `authored`:
/// it is consumed and discarded by `recognize_optional_signed_operand`
/// before this struct is built.
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
/// `Vec<Fact>`, `first: Fact, rest: Vec<Fact>`, `One | Two | Three | Many`
/// (initializer or free-standing), a recursive occurrence representation, a
/// generic non-empty collection, a generic `Expression` node, or an AST/CST.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CombinedAdditiveChainEvidence<'a> {
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

/// Finds where one `IdentifierReference`-family operand's identifier body
/// ends: one left-to-right forward pass consuming either one direct code
/// point or one syntactically well-formed `UnicodeEscapeSequence` element,
/// stopping at the first selected-trivia code point or at `+`/`-`, or at the
/// end of the candidate. A malformed escape aborts the whole candidate
/// immediately (`None`) rather than silently truncating the run at that
/// point. Position validity of the collected run is judged afterward by
/// `recognize_accepted_identifier_reference`, never here. Independently
/// restated, unchanged, from Issue #752/#795/#801/#809's identically named
/// function, never imported.
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

/// Recognizes one `SelectedOptionalPlusMinusIdentifierReferenceOperand` at
/// the start of `slice`: at most one leading `+`/`-`, then, only when that
/// sign was present, `SelectedUnaryOperandTrivia`, then one accepted
/// `IdentifierReference` body. Returns the operand fact, the remainder of
/// `slice` after the operand, and the leading sign actually consumed (`None`
/// when the operand was plain). The caller applies the
/// `SelectedBinaryUnaryBoundary` rule using the returned sign; this function
/// itself knows nothing about any preceding binary operator, which is
/// exactly why it can be reused unchanged for the chain's leading operand
/// (which has no preceding binary operator at all) and for every
/// continuation operand that dispatch has already routed to the reference
/// family. A second leading sign immediately following the first (`+-`,
/// `-+`, and interior equivalents such as `a+-+b`) is rejected structurally
/// rather than by a dedicated recursive-unary check: after consuming the
/// first sign and its trivia, `operand_interior_run_end` immediately meets
/// the second sign as its very first code point and therefore returns
/// `None` (zero bytes consumed), which this function propagates. Likewise a
/// leading sign immediately followed by an ASCII digit (`+1`, `-1`) is
/// rejected structurally: `operand_interior_run_end` happily consumes the
/// digit run, but `recognize_accepted_identifier_reference` then rejects it
/// because a bare digit sequence is never an accepted `IdentifierReference`
/// -- this is the entire signed-Decimal firewall, with no dedicated check.
/// Independently restated, unchanged, from Issue #809's identically named
/// function, never imported.
fn recognize_optional_signed_operand(
    slice: &str,
) -> Option<(RecognizedReference<'_>, &str, Option<char>)> {
    let (leading_sign, after_sign) = match slice.chars().next() {
        Some(sign @ ('+' | '-')) => (Some(sign), &slice[1..]),
        _ => (None, slice),
    };

    let after_unary_operand_trivia = match leading_sign {
        Some(_) => &after_sign[trivia_run_end(after_sign)..],
        None => after_sign,
    };

    let operand_end = operand_interior_run_end(after_unary_operand_trivia)?;
    let (operand_slice, after_operand) = after_unary_operand_trivia.split_at(operand_end);
    let operand = recognize_accepted_identifier_reference(operand_slice)?;
    Some((operand, after_operand, leading_sign))
}

/// One recognized `SelectedCombinedOperand`: either a Decimal atom (recorded
/// only by presence -- never retained as a fact) or an `IdentifierReference`
/// fact together with the leading sign actually consumed for that operand
/// (`None` when the operand was plain), which the caller uses to apply the
/// `SelectedBinaryUnaryBoundary` rule at continuation boundaries.
enum RecognizedCombinedOperand<'a> {
    Reference(RecognizedReference<'a>, Option<char>),
    Decimal,
}

/// Recognizes one `SelectedCombinedOperand` at the start of `slice`.
/// Dispatch is purely syntactic and never depends on decoded semantic
/// identity, exactly as Issue #815 already established: a candidate operand
/// is attempted as `SelectedAcceptedPlainDecimalAtom` only when its first
/// authored byte is an ASCII digit or `.`; every other first byte -- `+`,
/// `-`, and every `IdentifierReference`-eligible byte alike -- attempts
/// `SelectedOptionalPlusMinusIdentifierReferenceOperand` via
/// `recognize_optional_signed_operand`. In particular an escaped
/// `IdentifierReference` such as `0` is never reclassified as Decimal
/// merely because its decoded value resembles a digit, because its authored
/// first byte is `\`, not a digit or `.`.
fn recognize_combined_operand(slice: &str) -> Option<(RecognizedCombinedOperand<'_>, &str)> {
    let starts_decimal = slice
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_digit() || first == '.');

    if starts_decimal {
        let operand_end = decimal_atom_run_end(slice)?;
        let (operand_slice, after_operand) = slice.split_at(operand_end);
        if !is_selected_accepted_plain_decimal_atom(operand_slice) {
            return None;
        }
        Some((RecognizedCombinedOperand::Decimal, after_operand))
    } else {
        let (operand, after_operand, leading_sign) = recognize_optional_signed_operand(slice)?;
        Some((
            RecognizedCombinedOperand::Reference(operand, leading_sign),
            after_operand,
        ))
    }
}

/// Central Issue #821 theorem: a genuine left-to-right iterative recognizer,
/// not an unrolled fixed-cardinality theorem. Whole-candidate,
/// all-or-nothing: reference facts are only ever appended to a local `Vec`
/// during the loop and combined into the returned struct in a single final
/// expression, so a failure partway through the chain can never publish an
/// earlier prepared prefix as a partial result (every fallible step uses `?`
/// and returns `None` before that final expression is reached).
///
/// At every continuation, `SelectedAdditiveContinuationTrivia` is consumed
/// before the binary operator, then the trivia between the binary operator
/// and the next operand is measured (`boundary_trivia_is_empty`) before that
/// next operand is dispatched. The accepted Issue #777/#809
/// `SelectedBinaryUnaryBoundary` rule is applied *only* when the dispatched
/// continuation operand resolves to the `IdentifierReference` family and
/// itself carries a leading sign: same-sign adjacency with the just-consumed
/// binary operator then requires the intervening trivia run to be non-empty
/// (otherwise the source is one `++`/`--` `UpdateExpression`-adjacent
/// punctuator, never two selected operators); opposite-sign adjacency is
/// accepted whether or not trivia separates them. A Decimal-family
/// continuation operand carries no leading sign of its own (dispatch already
/// routed any `+`/`-`-first content to the reference family instead), so no
/// boundary check ever applies to it, and the measured boundary trivia
/// simply composes as ordinary continuation trivia -- exactly as Issue #815
/// already established.
fn recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
    candidate: &str,
) -> Option<CombinedAdditiveChainEvidence<'_>> {
    let mut references: Vec<RecognizedReference<'_>> = Vec::new();
    let mut operand_count = 0_usize;
    let mut saw_decimal = false;

    let (leading_operand, after_leading) = recognize_combined_operand(candidate)?;
    match leading_operand {
        RecognizedCombinedOperand::Reference(operand, _leading_operand_sign) => {
            references.push(operand);
        }
        RecognizedCombinedOperand::Decimal => saw_decimal = true,
    }
    operand_count += 1;
    let mut remaining = after_leading;

    loop {
        if remaining.is_empty() {
            break;
        }

        // SelectedAdditiveTriviaBeforeBinary.
        let after_leading_trivia = &remaining[trivia_run_end(remaining)..];
        let binary = match after_leading_trivia.chars().next() {
            Some(sign @ ('+' | '-')) => sign,
            // Either trailing trivia with no operator behind it (outer
            // trivia does not belong to this naked theorem), or unexpected
            // trailing content that the operand scan stopped short of
            // absorbing: both remain unaccepted, never a partial prefix.
            _ => return None,
        };
        let after_binary = &after_leading_trivia[1..];

        // SelectedBinaryUnaryBoundary trivia: the run between the binary
        // operator and the next operand's own optional leading sign, when
        // the next operand turns out to carry one.
        let boundary_trivia_end = trivia_run_end(after_binary);
        let boundary_trivia_is_empty = boundary_trivia_end == 0;
        let after_boundary_trivia = &after_binary[boundary_trivia_end..];

        let (operand, after_operand) = recognize_combined_operand(after_boundary_trivia)?;
        match operand {
            RecognizedCombinedOperand::Reference(operand, leading_sign) => {
                if let Some(unary) = leading_sign
                    && binary == unary
                    && boundary_trivia_is_empty
                {
                    return None;
                }
                references.push(operand);
            }
            RecognizedCombinedOperand::Decimal => saw_decimal = true,
        }

        operand_count += 1;
        remaining = after_operand;
    }

    (operand_count >= 2 && !references.is_empty() && saw_decimal).then_some(
        CombinedAdditiveChainEvidence {
            references,
            operand_count,
        },
    )
}

fn names<'b>(evidence: &'b CombinedAdditiveChainEvidence<'_>) -> Vec<&'b str> {
    evidence
        .references
        .iter()
        .map(|reference| reference.semantic_name.as_str())
        .collect()
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 821);
    assert_eq!(SELECTED_OPERAND_CARDINALITY, "2..N");
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(
        PREVIOUS_OPTIONAL_PLUS_MINUS_IDENTIFIER_REFERENCE_ADDITIVE_CHAIN_ORACLE_SOURCE
            .contains("ISSUE_ID: u64 = 809")
    );
    assert!(
        PREVIOUS_OPTIONAL_PLUS_MINUS_IDENTIFIER_REFERENCE_ADDITIVE_CHAIN_ORACLE_SOURCE
            .contains("optional-leading-+/- IdentifierReference additive-chain 2..N frontier only")
    );
    assert!(
        PREVIOUS_HETEROGENEOUS_REFERENCE_DECIMAL_ADDITIVE_CHAIN_ORACLE_SOURCE
            .contains("ISSUE_ID: u64 = 815")
    );
    assert!(PREVIOUS_HETEROGENEOUS_REFERENCE_DECIMAL_ADDITIVE_CHAIN_ORACLE_SOURCE.contains(
        "ordered 2..N heterogeneous IdentifierReference/plain-Decimal additive-chain frontier only"
    ));
    assert!(FRONTIER_SCOPE_NOTE.contains(
        "optional-leading-+/- IdentifierReference x heterogeneous plain-Decimal additive-chain 2..N frontier only"
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
            "recognize_selected_optional_plus_minus_identifier_",
            "reference_additive_chain("
        ),
        concat!(
            "recognize_selected_heterogeneous_reference_decimal_",
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
        // No cross-Oracle import: this leaf restates rather than imports
        // from either directly composed predecessor Oracle.
        concat!(
            "use super::selected_optional_plus_minus_identifier_",
            "reference_additive_chain_frontier"
        ),
        concat!(
            "use super::selected_heterogeneous_reference_decimal_",
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

/// Genuine iterative model, not an unrolled fixed-cardinality theorem (W20):
/// the recognizer is defined with a real `loop`, never named
/// `first`/`second`/`third`/`fourth` fields, and no hardcoded four-or-five
/// operand cap exists anywhere in the source.
#[test]
fn recognition_is_a_genuine_loop_never_an_unrolled_fixed_cardinality_theorem() {
    assert!(THIS_ORACLE_SOURCE.contains("loop {"));
    // Defined exactly once as a real function; the second match is this
    // assertion's own search-string literal.
    assert_eq!(
        THIS_ORACLE_SOURCE
            .matches("fn operand_interior_run_end(")
            .count(),
        2
    );
    assert_eq!(
        THIS_ORACLE_SOURCE
            .matches("fn decimal_atom_run_end(")
            .count(),
        2
    );
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

/// Required cardinality witnesses (acceptance criteria 1, 2): 2, 3, 4, 5, and
/// a long 8-operand sentinel -- exactly the Issue's own long sentinel -- all
/// recognize with the exact syntax operand count and exact ordered reference
/// names, mixing plain and optionally-wrapped `IdentifierReference` operands
/// with Decimal operands (including an exponent-bearing one) at various
/// positions. The 8-operand sentinel exists to falsify an accidental fixed
/// cap, not as a stress benchmark.
#[test]
fn cardinality_witnesses_from_two_through_a_long_sentinel_all_recognize() {
    // 2.
    for (candidate, expected_names) in [
        ("+a+1", ["a"].as_slice()),
        ("1+-a", &["a"]),
        ("1-+a", &["a"]),
        ("1+ +a", &["a"]),
        ("1- -a", &["a"]),
        ("a+1", &["a"]),
        ("1+a", &["a"]),
    ] {
        let evidence =
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                candidate,
            )
            .unwrap_or_else(|| panic!("{candidate:?} must recognize the theorem"));
        assert_eq!(evidence.operand_count, 2);
        assert_eq!(names(&evidence), expected_names);
    }

    // 3.
    for (candidate, expected_names) in [
        ("a+-b+1", ["a", "b"].as_slice()),
        ("a+1+-b", &["a", "b"]),
        ("1+-a+2", &["a"]),
    ] {
        let evidence =
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                candidate,
            )
            .unwrap_or_else(|| panic!("{candidate:?} must recognize the theorem"));
        assert_eq!(evidence.operand_count, 3);
        assert_eq!(names(&evidence), expected_names);
    }

    // 4.
    let four =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+1+b+ +c",
        )
        .expect("a+1+b+ +c must recognize the theorem");
    assert_eq!(four.operand_count, 4);
    assert_eq!(names(&four), ["a", "b", "c"]);

    // 5+.
    for (candidate, expected_names) in [
        ("1e-2+-a", ["a"].as_slice()),
        ("1e+2-+a", &["a"]),
        ("a+-b+1e+2+c", &["a", "b", "c"]),
        ("a+1e-2+b+ +c", &["a", "b", "c"]),
    ] {
        let evidence =
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                candidate,
            )
            .unwrap_or_else(|| panic!("{candidate:?} must recognize the theorem"));
        assert_eq!(names(&evidence), expected_names);
    }

    // 8-operand long sentinel, exactly matching Issue #821's own example:
    // falsifies any accidental fixed cap while simultaneously exercising
    // leading unary reference, right-unary reference, silent Decimal
    // operands, an exponent-internal sign, and same/opposite binary-unary
    // punctuator boundaries.
    let long_sentinel =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "+a+1+-b+2+ +c-3e-2+d+4",
        )
        .expect("+a+1+-b+2+ +c-3e-2+d+4 must recognize the theorem");
    assert_eq!(long_sentinel.operand_count, 8);
    assert_eq!(names(&long_sentinel), ["a", "b", "c", "d"]);
    assert!(long_sentinel.references.len() < long_sentinel.operand_count);
}

/// Required positive matrix with fixture-owned literal byte ranges for every
/// retained reference fact -- never derived by search, rescan, or reparse --
/// using the Issue's own long-sentinel candidate embedded in a larger
/// literal (acceptance criteria 9, 10, 13, 20; W10/W23).
#[test]
fn positive_matrix_pins_source_backed_reference_facts_with_exact_provenance() {
    // "const x = +a+1+-b+2+ +c-3e-2+d+4;"
    //            0123456789...
    // Byte offsets (verified against the literal below, prefix "const x = "
    // occupies bytes 0..10): '+'=10 'a'=11 '+'=12 '1'=13 '+'=14 '-'=15
    // 'b'=16 '+'=17 '2'=18 '+'=19 ' '=20 '+'=21 'c'=22 '-'=23 '3'=24 'e'=25
    // '-'=26 '2'=27 '+'=28 'd'=29 '+'=30 '4'=31 ';'=32.
    // operand1 = "a" wrapped by leading "+" (whole-candidate leading operand,
    // no boundary rule applies).
    // operand2 = "1" (plain Decimal integer).
    // operand3 = "b" wrapped by leading "-" (opposite sign vs. preceding
    // binary "+", zero boundary trivia accepted).
    // operand4 = "2" (plain Decimal integer).
    // operand5 = "c" wrapped by leading "+" (same sign as preceding binary
    // "+", non-empty boundary trivia -- one space -- required and present).
    // operand6 = "3e-2" (Decimal atom; exponent-internal "-" is atom-owned,
    // never a binary/unary operator).
    // operand7 = "d" plain.
    // operand8 = "4" (plain Decimal integer).
    let text = "const x = +a+1+-b+2+ +c-3e-2+d+4;";
    let whole = Range(10, 32);
    let reference_ranges = [Range(11, 12), Range(16, 17), Range(22, 23), Range(29, 30)];
    let reference_names = ["a", "b", "c", "d"];
    let decimal_ranges = [Range(13, 14), Range(18, 19), Range(24, 28), Range(31, 32)];
    let decimal_atoms = ["1", "2", "3e-2", "4"];

    assert_eq!(slice(text, whole), "+a+1+-b+2+ +c-3e-2+d+4");
    for (range, expected) in reference_ranges.iter().zip(reference_names) {
        assert_eq!(slice(text, *range), expected);
    }
    for (range, expected) in decimal_ranges.iter().zip(decimal_atoms) {
        assert_eq!(slice(text, *range), expected);
        assert!(is_selected_accepted_plain_decimal_atom(expected));
    }

    let whole_text = slice(text, whole);
    let evidence =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            whole_text,
        )
        .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

    assert_eq!(evidence.operand_count, 8);
    assert_eq!(evidence.references.len(), reference_ranges.len());
    for (index, (range, expected_name)) in reference_ranges.iter().zip(reference_names).enumerate()
    {
        assert_eq!(evidence.references[index].authored, expected_name);
        assert_eq!(evidence.references[index].semantic_name, expected_name);
        assert_eq!(
            evidence.references[index].spelling,
            OperandSpellingKind::Direct
        );
        assert_eq!(
            authored_anchor(821_000 + index as u64, text, *range),
            expected_name
        );
    }
    for (index, (range, expected_atom)) in decimal_ranges.iter().zip(decimal_atoms).enumerate() {
        assert_eq!(
            authored_anchor(821_500 + index as u64, text, *range),
            expected_atom
        );
    }
    assert!(evidence.references.len() < evidence.operand_count);
}

/// Load-bearing three-way `+`/`-` ownership theorem (core of Issue #821):
/// `1e-2+-a` must decompose as one Decimal atom `1e-2` (its `-`
/// exponent-owned), one binary additive `+`, and one unary `-` wrapping the
/// `IdentifierReference` `a` -- the retained fact contains only `a`, never
/// `-a`, and the Decimal atom is never split into a shorter accepted prefix
/// plus a reinterpreted operator (W5, W10).
#[test]
fn three_way_plus_minus_ownership_classes_do_not_leak_across_boundaries() {
    assert!(is_selected_accepted_plain_decimal_atom("1e-2"));

    let evidence =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "1e-2+-a",
        )
        .expect("1e-2+-a must recognize the theorem");
    assert_eq!(evidence.operand_count, 2);
    assert_eq!(evidence.references.len(), 1);
    assert_eq!(evidence.references[0].authored, "a");
    assert_eq!(evidence.references[0].semantic_name, "a");
    assert_ne!(evidence.references[0].authored, "-a");

    // The corresponding exponent-plus variant preserves the same three-way
    // ownership in the opposite orientation (unary reference first, then
    // exponent-bearing Decimal).
    let opposite_orientation =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "1e+2-+a",
        )
        .expect("1e+2-+a must recognize the theorem");
    assert_eq!(opposite_orientation.operand_count, 2);
    assert_eq!(opposite_orientation.references[0].authored, "a");

    // A long chain exercising all three ownership classes at once, with two
    // independently owned exponent signs and two independently wrapped
    // unary references.
    let mixed =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+-b+1e+2+c",
        )
        .expect("a+-b+1e+2+c must recognize the theorem");
    assert_eq!(mixed.operand_count, 4);
    assert_eq!(names(&mixed), ["a", "b", "c"]);
}

/// Load-bearing binary/unary punctuator-boundary matrix (acceptance
/// criteria 6, 7; core of Issue #821): the accepted Issue #777/#809 boundary
/// rule applies independently at the first, an interior, the final, and a
/// long-chain continuation of the *combined heterogeneous* chain, and only
/// when the continuation operand resolves to the reference family.
/// Opposite-sign adjacency is always accepted whether or not trivia
/// separates the two operators; same-sign adjacency is accepted only with
/// non-empty separating trivia and rejected with none (W6, W7, W8).
#[test]
fn binary_unary_punctuator_boundary_matrix_applies_independently_at_every_continuation() {
    // Opposite-sign adjacency, zero trivia.
    for accepted in ["a+1+-b", "a+1-+b", "1+a+-b", "1+a-+b"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                accepted
            )
            .is_some(),
            "{accepted:?}"
        );
    }

    // Same-sign adjacency, non-empty trivia.
    for accepted in ["a+1+ +b", "a+1- -b", "1+a+\t+b", "1+a-\n-b"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                accepted
            )
            .is_some(),
            "{accepted:?}"
        );
    }

    // Same-sign adjacency, zero trivia: rejected.
    for rejected in ["a+1++b", "a+1--b", "1+a++b", "1+a--b"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                rejected
            )
            .is_none(),
            "{rejected:?}"
        );
    }

    // Interior continuation, both a preceding and following plain operand.
    for accepted in ["a+1+-b+2+c", "a+1-+b+2+c", "a+1+ +b+2+c", "a+1- -b+2+c"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                accepted
            )
            .is_some(),
            "{accepted:?}"
        );
    }
    for rejected in ["a+1++b+2+c", "a+1--b+2+c"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                rejected
            )
            .is_none(),
            "{rejected:?}"
        );
    }

    // Final continuation.
    for accepted in ["a+1+2+-b", "a+1+2-+b", "a+1+2+ +b", "a+1+2- -b"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                accepted
            )
            .is_some(),
            "{accepted:?}"
        );
    }
    for rejected in ["a+1+2++b", "a+1+2--b"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                rejected
            )
            .is_none(),
            "{rejected:?}"
        );
    }

    // Long-chain candidate applying the rule at every one of several
    // boundaries at once, mixing Decimal-continuation boundaries (which
    // never trigger the rule) with reference-continuation boundaries.
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "+a+1+-b+2+ +c-3e-2+d+4"
        )
        .is_some()
    );
    // The corresponding zero-trivia same-sign violation ("+ +" tightened to
    // "++") at that same long chain's interior boundary is rejected.
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "+a+1+-b+2++c-3e-2+d+4"
        )
        .is_none()
    );
}

/// Decimal exponent-sign ownership matrix, reused from Issue #815's own
/// theorem: an exponent-internal `+`/`-` remains owned by the Decimal atom
/// and distinct from a following binary additive operator or unary sign, at
/// every position in the combined chain (acceptance criteria 7, 8).
#[test]
fn decimal_exponent_sign_ownership_matrix_distinguishes_from_binary_and_unary_operators() {
    for atom in ["1e-2", "1e+2", "3e+4"] {
        assert!(is_selected_accepted_plain_decimal_atom(atom), "{atom:?}");
    }

    let internal_minus_before_reference =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+1e-2+b",
        )
        .expect("a+1e-2+b must recognize the theorem");
    assert_eq!(internal_minus_before_reference.operand_count, 3);
    assert_eq!(names(&internal_minus_before_reference), ["a", "b"]);

    let internal_plus_before_unary_reference =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "1e+2+-a",
        )
        .expect("1e+2+-a must recognize the theorem");
    assert_eq!(internal_plus_before_unary_reference.operand_count, 2);
    assert_eq!(names(&internal_plus_before_unary_reference), ["a"]);

    let multiple_exponent_sign_ownership =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+-b+1e+2+c",
        )
        .expect("a+-b+1e+2+c must recognize the theorem");
    assert_eq!(multiple_exponent_sign_ownership.operand_count, 4);
    assert_eq!(names(&multiple_exponent_sign_ownership), ["a", "b", "c"]);
}

/// Invalid exponent boundary firewall, reused from Issue #815's own theorem:
/// a structurally invalid exponent sign never leaks a shorter accepted
/// numeric prefix, and the failed Decimal candidate never reinterprets its
/// own owned sign as a binary or unary operator (W9).
#[test]
fn invalid_exponent_boundary_never_reinterprets_owned_sign_as_binary_or_unary_operator() {
    for control in [
        "1e++a", "1e--a", "1e+-a", "1e-+a", "a+1e+", "a+1e-", "a+1e++b", "a+1e--b",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
}

/// All-reference and all-Decimal firewalls: chains composed entirely of one
/// family remain outside this exact combined theorem, owned instead by
/// already-accepted #809/#810 (all-reference) or remaining unowned
/// (all-Decimal) authority (acceptance criteria 3, 4; W1).
#[test]
fn all_reference_firewall_and_all_decimal_firewall_remain_outside_the_theorem() {
    for control in ["+a+-b+c", "a+b+c", "a-b+c-d", "+a-b+ +c-d"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
    for control in ["1+2", "1+2+3", "1e-2+3+4"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+1"
        )
        .is_some()
    );
}

/// Signed-Decimal firewall (acceptance criterion 5; W4): the optional
/// leading `+`/`-` wrapper applies only to `IdentifierReference` operands;
/// a sign immediately followed by a bare Decimal digit run is never an
/// accepted operand, at the leading position or any continuation, whatever
/// otherwise-valid heterogeneous content surrounds it.
#[test]
fn signed_decimal_firewall_keeps_a_leading_sign_off_bare_decimal_operands() {
    for control in ["+1+a", "-1+a", "a+-1", "a+ -1", "1++2", "1- -2"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "1+-a"
        )
        .is_some()
    );
}

/// UpdateExpression / longest-input-element firewall (W6): authored
/// `++`/`--` is never split into a binary sign and a following unary sign,
/// at the first, an interior, or a final continuation boundary of the
/// combined heterogeneous chain.
#[test]
fn update_expression_and_longest_input_element_firewall_remains_outside() {
    for control in [
        "1++a", "1--a", "a++b+1", "a--b+1", "a+++b+1", "a---b+1", "a+1++b", "a+1--b",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
}

/// Exactly-one unary wrapper / recursive-unary firewall (W14): a second
/// leading sign immediately following the first is rejected at the leading
/// operand and at any continuation operand of the combined chain.
#[test]
fn recursive_unary_wrapper_firewall_remains_outside() {
    for control in ["+-a+1", "-+a+1", "1+-+a", "1-+-a", "a+-+b+1", "a+1-+-b"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+1+-b"
        )
        .is_some()
    );
}

/// Other unary operator families remain outside (W15): `!`, `~`, `typeof`,
/// `void`, `delete` never leak into this bounded `+`/`-` theorem at any
/// operand position, whatever otherwise-valid heterogeneous content
/// surrounds them.
#[test]
fn other_unary_operator_families_remain_outside() {
    for control in [
        "!a+1",
        "~a+1",
        "a+!b+1",
        "a+1+~b",
        "typeof a+1",
        "void a+1",
        "delete a+1",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
}

/// Numeric frontier firewall (W16): `NumericLiteralSeparator`, radix-
/// prefixed integers, and `BigIntLiteral` forms never leak a shorter
/// accepted prefix, whichever position they occupy in the combined chain.
#[test]
fn numeric_frontier_firewall_remains_unowned_at_any_position() {
    for control in [
        "a+1_0+b", "a+0x10+b", "a+0b10+b", "a+0o10+b", "a+1n+b", "1_0+a+2", "0x10+a+2", "1n+a+2",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
}

/// Other operand family firewall (W17): Boolean, `null`, `this`, and
/// string-literal operands remain outside this theorem at any position,
/// plain or in a position a unary sign could otherwise wrap.
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
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
}

/// Richer expression / precedence / grouping / comment firewall (W18, W19):
/// grouping, member access, call, multiplicative precedence, assignment,
/// conditional, comma, and comment tails all remain outside without any
/// dedicated recognition code for any of them, and comments/U+200B never
/// compose as trivia; an already-selected `LineTerminator` composes without
/// implying general ASI.
#[test]
fn richer_expression_precedence_grouping_and_comment_firewall_remains_unowned() {
    for control in [
        "(a)+1",
        "a+(b)+1",
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
        "a/*x*/+1",
        "a+/*x*/1",
        "a+1/*x*/+b",
        "a+1+/*x*/b",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
    assert!(!is_selected_additive_trivia('\u{200B}'));
    assert!(is_selected_additive_trivia('\u{00A0}'));
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+\u{200B}+1"
        )
        .is_none()
    );
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+\n1+b"
        )
        .is_some()
    );
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+1+b"
        )
        .is_some()
    );
}

/// Duplicate preservation theorem (acceptance criterion 9): authored
/// left-to-right order is load-bearing and never reordered by semantic
/// name; equal semantic names are never deduplicated into fewer facts,
/// whether the duplicated reference is plain, opposite-sign wrapped, or
/// same-sign wrapped with trivia, matching the Issue's own
/// `+a+1+-a+2+a` / `1+a+-a+a` examples.
#[test]
fn authored_order_is_preserved_and_equal_semantic_names_are_never_deduplicated() {
    let reordered =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "c+1+a+2+b",
        )
        .expect("c+1+a+2+b must recognize the theorem");
    assert_eq!(names(&reordered), ["c", "a", "b"]);

    let all_duplicate =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "+a+1+-a+2+a",
        )
        .expect("+a+1+-a+2+a must recognize the theorem");
    assert_eq!(all_duplicate.operand_count, 5);
    assert_eq!(all_duplicate.references.len(), 3);
    for reference in &all_duplicate.references {
        assert_eq!(reference.semantic_name, "a");
        assert_eq!(reference.authored, "a");
    }

    // Built with `concat!` over individually escaped fragments so the
    // literal bytes `\`, `u`, `0`, `0`, `6`, `1` survive intact rather than
    // being collapsed into a decoded Unicode scalar.
    const MIXED_SPELLING_DUPLICATE_SOURCE: &str =
        concat!("1+", "\\", "u0061", "+-a+", "\\", "u0061");
    const ESCAPED_A: &str = concat!("\\", "u0061");
    let mixed_spelling_duplicate =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            MIXED_SPELLING_DUPLICATE_SOURCE,
        )
        .expect("mixed direct/escaped duplicate source must recognize the theorem");
    assert_eq!(mixed_spelling_duplicate.operand_count, 4);
    assert_eq!(mixed_spelling_duplicate.references.len(), 3);
    let expected_authored = [ESCAPED_A, "a", ESCAPED_A];
    let expected_spelling = [
        OperandSpellingKind::EscapedNonReserved,
        OperandSpellingKind::Direct,
        OperandSpellingKind::EscapedNonReserved,
    ];
    for (index, reference) in mixed_spelling_duplicate.references.iter().enumerate() {
        assert_eq!(reference.authored, expected_authored[index]);
        assert_eq!(reference.spelling, expected_spelling[index]);
        assert_eq!(reference.semantic_name, "a");
    }
    assert_ne!(
        mixed_spelling_duplicate.references[0].authored,
        mixed_spelling_duplicate.references[1].authored
    );
}

/// Escaped `IdentifierReference` coverage at first, interior, final,
/// unary-wrapped, before-Decimal, after-Decimal, and after-exponent-bearing-
/// Decimal positions, each preserving authored spelling distinct from
/// decoded semantic name -- and the leading unary sign never entering the
/// retained inner anchor (acceptance criteria 9, 10, 13; W10).
#[test]
fn escaped_references_at_every_required_position_preserve_authored_identity() {
    const ESCAPED_A: &str = concat!("\\", "u0061");
    const ESCAPED_B: &str = concat!("\\", "u0062");
    const ESCAPED_C: &str = concat!("\\", "u0063");
    const ESCAPED_D: &str = concat!("\\", "u0064");

    // Wrapped escaped first operand, before a Decimal: `+a+1+b`.
    let first =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            concat!("+", "\\", "u0061", "+1+b"),
        )
        .expect("wrapped escaped first operand must recognize the theorem");
    assert_eq!(first.operand_count, 3);
    assert_eq!(first.references[0].authored, ESCAPED_A);
    assert_eq!(first.references[0].semantic_name, "a");
    assert_eq!(
        first.references[0].spelling,
        OperandSpellingKind::EscapedNonReserved
    );
    assert_ne!(first.references[0].authored, "-a");
    assert_ne!(
        first.references[0].authored,
        first.references[0].semantic_name
    );

    // Opposite-sign wrapped escaped interior operand, after a Decimal:
    // `1+-b+2`.
    let interior_after_decimal =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            concat!("1+-", "\\", "u0062", "+2"),
        )
        .expect("interior escaped operand after Decimal must recognize the theorem");
    assert_eq!(interior_after_decimal.operand_count, 3);
    assert_eq!(interior_after_decimal.references[0].authored, ESCAPED_B);
    assert_eq!(
        interior_after_decimal.references[0].spelling,
        OperandSpellingKind::EscapedNonReserved
    );

    // Same-sign wrapped (non-empty trivia) escaped final operand:
    // `a+1+ +c`.
    let last =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            concat!("a+1+ +", "\\", "u0063"),
        )
        .expect("wrapped escaped final operand must recognize the theorem");
    assert_eq!(last.operand_count, 3);
    assert_eq!(names(&last), ["a", "c"]);
    assert_eq!(last.references[1].authored, ESCAPED_C);
    assert_eq!(
        last.references[1].spelling,
        OperandSpellingKind::EscapedNonReserved
    );

    // Opposite-sign wrapped escaped operand immediately after an
    // exponent-bearing Decimal: `1e-2+-d`.
    let after_exponent_decimal =
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            concat!("1e-2+-", "\\", "u0064"),
        )
        .expect("escaped operand after exponent-bearing Decimal must recognize the theorem");
    assert_eq!(after_exponent_decimal.operand_count, 2);
    assert_eq!(after_exponent_decimal.references[0].authored, ESCAPED_D);
    assert_ne!(after_exponent_decimal.references[0].authored, "-\\u0064");
}

/// Escaped `ReservedWord` firewall at first, interior, and final positions,
/// both plain and unary-wrapped, in an otherwise heterogeneous chain
/// (W13): a decoded unconditionally reserved word never becomes a selected
/// operand anywhere.
#[test]
fn escaped_reserved_word_firewall_at_every_position_remains_outside() {
    for control in [
        concat!("\\", "u0069f+1+a"),
        concat!("+", "\\", "u0069f+1+a"),
        concat!("1+", "\\", "u0069f+a"),
        concat!("1+-", "\\", "u0069f+a"),
        concat!("a+1+", "\\", "u0069f"),
        concat!("a+1-", "\\", "u0069f"),
    ] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0069", "f")),
        Err(DecodeFailure::DecodedReserved)
    );
}

/// Malformed escape firewall at first, interior, and final positions in an
/// otherwise heterogeneous chain: a malformed escape never leaks a
/// truncated prefix or a valid sibling fact -- the whole candidate is
/// rejected (W23).
#[test]
fn malformed_escape_firewall_never_leaks_a_sibling_fact() {
    for control in [
        r"\u{}+1+a",
        r"+\u{}+1+a",
        r"1+\u{}+a",
        r"1+-\u{}+a",
        r"a+1+\u{}",
        r"a+1-\u{}",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{}"),
        Err(DecodeFailure::MalformedEscape)
    );
}

/// Whole-candidate transactionality and no-prefix-truncation (W24): an
/// incomplete or richer-tail chain never publishes a prepared prefix, and a
/// longer, otherwise-valid-looking candidate with a malformed or richer
/// final segment never truncates to a shorter accepted result. Because the
/// return type is `Option`, a rejected candidate structurally cannot yield
/// a partial `Vec` of fewer facts.
#[test]
fn whole_candidate_transactionality_never_commits_a_partial_or_truncated_result() {
    for control in [
        "+a+1+-b+",
        concat!("+a+1+-b+", "\\", "u0069f"),
        r"+a+1+-b+\u{}",
        "+a+1+-b.c",
        "+a+1+-b()",
        "+a+1+-b=c",
        "+a+1+-b,c",
        "+a+1+-b?c:d",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }

    // The complete valid prefix itself remains independently valid, and a
    // rejected longer candidate never truncates to it.
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "+a+1+-b"
        )
        .is_some()
    );
}

/// Outer trivia firewall: leading and trailing statement/declaration-level
/// trivia does not belong to the naked theorem and is never silently
/// absorbed, whatever internal trivia the same candidate otherwise accepts.
#[test]
fn outer_leading_and_trailing_trivia_remain_outside_the_naked_theorem() {
    assert!(
        recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
            "a+1+b"
        )
        .is_some()
    );
    for control in [" a+1+b", "a+1+b ", " a+1+b ", "\ta+1+b", "a+1+b\n"] {
        assert!(
            recognize_selected_optional_plus_minus_heterogeneous_reference_decimal_additive_chain(
                control
            )
            .is_none(),
            "{control:?}"
        );
    }
}

/// Resource / failure semantics: the project's
/// `ResourceLimited`/`InternalFailure` lifecycle states remain distinct from
/// `UnsupportedCoverage`, matching the minimal symbolic model already
/// accepted by every predecessor candidate-independent oracle -- this leaf
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
/// whole-expression `SourceAnchor`, no binary/unary/additive expression
/// node, no AST/CST, no token tape, no runtime/static vocabulary, and no
/// frozen production collection-representation choice is introduced.
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
        concat!("struct Unary", "SourceAnchor"),
        concat!("struct WholeExpression", "SourceAnchor"),
        concat!("struct Binary", "ExpressionNode"),
        concat!("struct Additive", "ExpressionNode"),
        concat!("struct Unary", "ExpressionNode"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("enum One", "Two", "Three", "Many"),
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

    assert!(THIS_ORACLE_SOURCE.contains("struct CombinedAdditiveChainEvidence"));
    assert!(THIS_ORACLE_SOURCE.contains("references: Vec<RecognizedReference"));
    assert!(THIS_ORACLE_SOURCE.contains("operand_count: usize"));
    assert!(THIS_ORACLE_SOURCE.contains("ORACLE DYNAMIC STORAGE"));
    assert!(THIS_ORACLE_SOURCE.contains("PRODUCTION STORAGE AUTHORITY"));
    assert!(THIS_ORACLE_SOURCE.contains("does **not** freeze future production representation"));
    assert!(THIS_ORACLE_SOURCE.contains("references.len() < operand_count"));
    assert!(THIS_ORACLE_SOURCE.contains("One | Two | Three | Many"));
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
