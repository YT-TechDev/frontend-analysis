//! Candidate-independent ordered 2..N optional-leading-`+`/`-`
//! `IdentifierReference` additive-chain fact-preservation validation for
//! Issue #809 (durable research: Issue #688 comment `5789149241`; accepted
//! predecessor authorities: Issue #746 / PR #747 leading `+`/`-` direct
//! `IdentifierReference` `UnaryExpression` fact-preservation Oracle, Issue
//! #750 / PR #751 escaped non-Reserved `IdentifierReference` unary
//! composition, Issue #752 / PR #753 ordered exactly-two `IdentifierReference`
//! additive Oracle, Issue #777 / PR #778 candidate-independent right-unary
//! exactly-two additive token-boundary Oracle, Issue #795 / PR #796 ordered
//! exactly-three `IdentifierReference` additive Oracle, Issue #801 / PR #802
//! candidate-independent ordered plain-`IdentifierReference` additive-chain
//! Oracle, cardinality `2..N`). Production predecessors #803 / PR #805 and
//! #807 / PR #808 exist but are explicitly NOT expected-answer authority for
//! this Oracle.
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedOptionalPlusMinusIdentifierReferenceAdditiveChain ::=
//!     SelectedOptionalPlusMinusIdentifierReferenceOperand
//!     (
//!         SelectedAdditiveContinuation
//!         SelectedOptionalPlusMinusIdentifierReferenceOperand
//!     )+
//!
//! SelectedOptionalPlusMinusIdentifierReferenceOperand ::=
//!     SelectedAcceptedIdentifierReference
//!   | SelectedUnaryPlusMinus
//!     SelectedUnaryOperandTrivia
//!     SelectedAcceptedIdentifierReference
//!
//! SelectedUnaryPlusMinus ::= "+" | "-"
//!
//! SelectedAcceptedIdentifierReference ::=
//!     SelectedDirectIdentifierReference
//!   | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! Selected operand cardinality is `2..N`, not `1..N`: a single plain or
//! unary-wrapped `IdentifierReference` remains owned by already-accepted
//! single-reference/single-`UnaryExpression` authority. It does not call
//! production lexical, static-semantics, correspondence, Binding/Scope,
//! aggregate, or runtime evaluation code.
//!
//! ## Load-bearing novelty
//!
//! Issue #777 independently proved the punctuator boundary between one
//! binary additive operator and one following right-unary operator, but only
//! for an exactly-two candidate. Issue #801 independently proved a genuine
//! arbitrary-cardinality iterative additive continuation, but only for plain
//! operands. Neither predecessor proves their composition: that the accepted
//! #777 binary/unary punctuator boundary can be applied independently at
//! *every* continuation of an arbitrary finite #801-style chain, while every
//! operand -- including the first -- may independently carry its own
//! optional exactly-one leading `+`/`-` wrapper, and while one exact
//! source-backed `IdentifierReference` fact is preserved per operand in exact
//! authored order. This Oracle proves exactly that composed theorem, and
//! nothing more.
//!
//! `recognize_selected_optional_plus_minus_identifier_reference_additive_chain`
//! is a genuine `loop`, never an unrolled fixed sequence of named
//! `first`/`second`/`third`/`fourth` fields, and is deliberately exercised
//! beyond four and five operands (an eight-operand sentinel below) precisely
//! so an accidental fixed cap cannot silently satisfy the suite.
//!
//! `operand_interior_run_end` is the same bounded, single left-to-right
//! forward pass already established by Issue #752/#795/#801 (consuming one
//! direct code point or one syntactically well-formed `UnicodeEscapeSequence`
//! element at a time, stopping only at the first selected-trivia code point
//! or at `+`/`-`), invoked once per operand's identifier body. Because the
//! scan only stops at a trivia code point or an authored `+`/`-`, it is never
//! a general tokenizer: any other character (`.`, `(`, `=`, `,`, `?`, and so
//! on) is silently absorbed into the same run and is judged afterward, at
//! whole-operand-slice granularity, by `recognize_accepted_identifier_reference`.
//! This is never `.find`/`.rfind`, later substring search, source rescanning
//! after recognition, reparse, retokenization, operator-relative endpoint
//! reconstruction, or decoded-length endpoint inference.
//!
//! `recognize_optional_signed_operand` independently restates the shape of
//! `SelectedOptionalPlusMinusIdentifierReferenceOperand`: at most one leading
//! `+`/`-` code point, then `SelectedUnaryOperandTrivia`, then one accepted
//! `IdentifierReference` body. It is invoked once for the leading operand
//! (where there is no preceding binary operator, so no boundary rule
//! applies) and once per continuation operand (where the caller has already
//! consumed the preceding binary operator and the intervening
//! `SelectedAdditiveContinuation` trivia and applies the #777
//! `SelectedBinaryUnaryBoundary` rule against the leading sign this function
//! reports back, before the operand's own fact is retained). Because a bare
//! `+`/`-` immediately followed by another `+`/`-` never advances past zero
//! consumed identifier bytes, a recursive unary wrapper (`+-a`, `-+a`, and
//! interior equivalents) fails structurally without any dedicated recursive-
//! unary check.
//!
//! `SelectedAdditiveTrivia`-family functions independently restate the
//! complete already-accepted selected-slice trivia contract established by
//! Issue #742/#743 and restated again by Issue #746/#747, #752/#753,
//! #777/#778, and #801/#802 (the same code-point set the production
//! selected-trivia recognizer accepts): `TAB`, `VT`, `FF`, `BOM`, `LF`, `CR`,
//! `LINE SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen Unicode 17
//! `Space_Separator` property. Comments remain outside this contract and are
//! never accepted as trivia; a comment's own characters are simply absorbed
//! into an identifier run's trailing content by `operand_interior_run_end`
//! and rejected afterward, exactly like #801.
//!
//! `SelectedDirectIdentifierReference` independently restates the
//! already-accepted Issue #237 direct escape-free `IdentifierName`
//! code-point shape (`is_id_start`/`is_id_continue` plus `$`/`_`) minus the
//! unconditionally reserved words. `SelectedEscapedNonReservedIdentifierReference`
//! independently restates the already-accepted Issue #241
//! `UnicodeEscapeSequence` decode/position/decoded-reserved-word theorem.
//! Neither restatement imports another Oracle's code: this leaf owns its own
//! copy of the already-accepted primitives it consumes.
//!
//! ## What is retained, and what is not
//!
//! The Oracle's only retained representation is
//! `OptionalPlusMinusIdentifierReferenceAdditiveChainFacts`, an ordered
//! non-empty sequence (`Vec<RecognizedOperand>`, minimum length two) of
//! operand facts, each preserving its own exact authored
//! `SourceAnchor`/fragment, decoded semantic name, and
//! Direct/EscapedNonReserved spelling state, in exact authored left-to-right
//! order with duplicates never deduplicated. This `Vec` is a minimal private
//! test-only shape chosen only because the theorem itself is `2..N`:
//!
//! ```text
//! ORACLE DYNAMIC STORAGE (private Vec<RecognizedOperand>)
//! !=
//! PRODUCTION STORAGE AUTHORITY
//! ```
//!
//! It explicitly does **not** freeze future production representation as
//! `Vec<Fact>`, `first: Fact, rest: Vec<Fact>`, `One | Two | Three | Many`
//! (initializer or free-standing), a recursive occurrence representation, a
//! generic non-empty collection, a generic `Expression` node, or an AST/CST.
//! It likewise does not share or merge the initializer and free-standing
//! production carriers, and does not create a shared whole-chain production
//! transaction: those owners retain materially different rollback semantics
//! that this validation-only leaf does not resolve. No leading unary sign,
//! binary operator sign, operator `SourceAnchor`, trivia `SourceAnchor`, or
//! whole-expression `SourceAnchor` is retained; every authored `+`/`-` is
//! recognized only to prove the exact bounded continuation and boundary
//! family, never as retained evidence.
//!
//! This is a validation-only leaf: production supports no optional-leading-
//! `+`/`-` `IdentifierReference` additive family at the #809 baseline, so
//! every positive fixture below remains `UnsupportedCoverage` under current
//! production and exists only as independent Oracle evidence for a future,
//! separately authorized production decision. No completion successor file
//! accompanies this leaf: this Issue adds zero production capability, so the
//! frozen `193 / 10 / 183 / {}` completion partition cannot move, matching
//! the precedent set by #742/#743, #746/#747, #752/#753, #777/#778, and
//! #801/#802.

use crate::{SourceId, SourceText};

use super::super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};
use super::inventory::RULE_UNITS;

const ISSUE_ID: u64 = 809;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("model.rs");
const PREVIOUS_RIGHT_UNARY_ADDITIVE_ORACLE_SOURCE: &str = include_str!(
    "../qualification_selected_identifier_reference_right_unary_additive_initializer_validation_tests.rs"
);
const PREVIOUS_PLAIN_ADDITIVE_CHAIN_ORACLE_SOURCE: &str =
    include_str!("selected_plain_identifier_reference_additive_chain_frontier.rs");
const THIS_ORACLE_SOURCE: &str =
    include_str!("selected_optional_plus_minus_identifier_reference_additive_chain_frontier.rs");
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "optional-leading-+/- IdentifierReference additive-chain 2..N frontier ",
    "only; later independently qualified owners may strengthen classification ",
    "for recursive unary, both-unary composition beyond exactly-one wrapper, ",
    "other unary operator families, heterogeneous non-IdentifierReference ",
    "operands, parenthesized/member/call/richer-expression operands, comments, ",
    "general ASI, or production representation/placement"
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
// #237/#752/#777/#801, never imported from any of those Oracles. ---

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

// --- Selected trivia: independently restated from Issue #742/#743, exactly
// as Issue #746/#747, #752/#753, #777/#778, and #801/#802 already restate
// it. Reused unchanged at every trivia position this theorem owns (before a
// binary operator, between a binary operator and a following unary operator,
// and between a unary operator and its IdentifierReference operand): there
// is exactly one selected-trivia contract, not several different ones. ---

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

// --- The Issue #809 theorem itself. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperandSpellingKind {
    Direct,
    EscapedNonReserved,
}

/// One retained operand fact. A private test-only shape, not a claim about
/// future production storage. The leading unary sign, when one was present,
/// is never part of `authored`: it is consumed and discarded by
/// `recognize_optional_signed_operand` before this struct is built.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedOperand<'a> {
    authored: &'a str,
    semantic_name: String,
    spelling: OperandSpellingKind,
}

/// The Oracle's only retained representation: an ordered, minimum-length-two
/// sequence of operand facts. This `Vec` shape is a minimal private test
/// representation chosen only because the theorem itself is `2..N`; it is
/// never a claim that future production must use `Vec<Fact>`, `first: Fact,
/// rest: Vec<Fact>`, `One | Two | Three | Many`, a recursive occurrence
/// representation, a generic non-empty collection, a generic `Expression`
/// node, or an AST/CST.
#[derive(Debug, Clone, PartialEq, Eq)]
struct OptionalPlusMinusIdentifierReferenceAdditiveChainFacts<'a> {
    operands: Vec<RecognizedOperand<'a>>,
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

/// Finds where one operand's identifier body ends: one left-to-right forward
/// pass consuming either one direct code point or one syntactically
/// well-formed `UnicodeEscapeSequence` element, stopping at the first
/// selected-trivia code point or at `+`/`-`, or at the end of the candidate.
/// A malformed escape aborts the whole candidate immediately (`None`) rather
/// than silently truncating the run at that point -- the malformed escape
/// never leaks an earlier valid-looking prefix. Position validity of the
/// collected run is judged afterward by `recognize_accepted_identifier_reference`,
/// never here. Because the scan stops only at trivia or `+`/`-`, whichever
/// operand happens to be the chain's last one has its run naturally extend to
/// the end of the candidate -- absorbing any other trailing content (`.`,
/// `(`, `=`, `,`, `?`, and so on) into the same slice, which then fails
/// identifier validation rather than truncating to a shorter accepted
/// prefix. Independently restated from Issue #752/#795/#801's identically
/// named function, never imported.
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
/// when the operand was plain). The caller applies the `SelectedBinaryUnaryBoundary`
/// rule using the returned sign; this function itself knows nothing about any
/// preceding binary operator, which is exactly why it can be reused
/// unchanged for the chain's leading operand (which has no preceding binary
/// operator at all) and for every continuation operand.
///
/// A second leading sign immediately following the first (`+-`, `-+`, and
/// interior equivalents such as `a+-+b`) is rejected structurally rather
/// than by a dedicated recursive-unary check: after consuming the first sign
/// and its trivia, `operand_interior_run_end` immediately meets the second
/// sign as its very first code point and therefore returns `None` (zero
/// bytes consumed), which this function propagates.
fn recognize_optional_signed_operand(
    slice: &str,
) -> Option<(RecognizedOperand<'_>, &str, Option<char>)> {
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

/// Central Issue #809 theorem: a genuine left-to-right iterative recognizer,
/// not an unrolled fixed-cardinality theorem. Whole-candidate, all-or-nothing:
/// operand facts are only ever appended to a local `Vec` during the loop and
/// combined into the returned struct in a single final expression, so a
/// failure partway through the chain can never publish an earlier prepared
/// prefix as a partial result (every fallible step uses `?` and returns
/// `None` before that final expression is reached).
///
/// At every continuation this applies the accepted Issue #777
/// `SelectedBinaryUnaryBoundary` rule independently: when the continuation
/// operand carries its own leading sign (`leading_sign` is `Some`), same-sign
/// adjacency with the just-consumed binary operator requires the
/// intervening trivia run to be non-empty (otherwise the source is one
/// `++`/`--` `UpdateExpression`-adjacent punctuator, never two selected
/// operators); opposite-sign adjacency is accepted whether or not trivia
/// separates them. When the continuation operand is plain (`leading_sign` is
/// `None`), there is no unary sign to disambiguate against, so the
/// intervening trivia composes freely as ordinary `SelectedAdditiveTrivia`,
/// exactly as Issue #801 already established.
fn recognize_selected_optional_plus_minus_identifier_reference_additive_chain(
    candidate: &str,
) -> Option<OptionalPlusMinusIdentifierReferenceAdditiveChainFacts<'_>> {
    let mut operands: Vec<RecognizedOperand<'_>> = Vec::new();

    let (leading_operand, after_leading, _leading_operand_sign) =
        recognize_optional_signed_operand(candidate)?;
    operands.push(leading_operand);
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
            // trailing content that `operand_interior_run_end` stopped
            // short of absorbing: both remain unaccepted, never a partial
            // prefix.
            _ => return None,
        };
        let after_binary = &after_leading_trivia[1..];

        // SelectedBinaryUnaryBoundary: the trivia run between the binary
        // operator and the next operand's own optional leading sign.
        let boundary_trivia_end = trivia_run_end(after_binary);
        let boundary_trivia_is_empty = boundary_trivia_end == 0;
        let after_boundary_trivia = &after_binary[boundary_trivia_end..];

        let (operand, after_operand, leading_sign) =
            recognize_optional_signed_operand(after_boundary_trivia)?;

        if let Some(unary) = leading_sign
            && binary == unary
            && boundary_trivia_is_empty
        {
            return None;
        }

        operands.push(operand);
        remaining = after_operand;
    }

    (operands.len() >= 2)
        .then_some(OptionalPlusMinusIdentifierReferenceAdditiveChainFacts { operands })
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 809);
    assert_eq!(SELECTED_OPERAND_CARDINALITY, "2..N");
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_RIGHT_UNARY_ADDITIVE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 777"));
    assert!(PREVIOUS_RIGHT_UNARY_ADDITIVE_ORACLE_SOURCE.contains(
        "right-unary exactly-two IdentifierReference additive token-boundary frontier only"
    ));
    assert!(PREVIOUS_PLAIN_ADDITIVE_CHAIN_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 801"));
    assert!(
        PREVIOUS_PLAIN_ADDITIVE_CHAIN_ORACLE_SOURCE
            .contains("ordered 2..N plain-IdentifierReference additive-chain frontier only")
    );
    assert!(
        FRONTIER_SCOPE_NOTE
            .contains("optional-leading-+/- IdentifierReference additive-chain 2..N frontier only")
    );

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
            "recognize_selected_identifier_reference_right_unary_",
            "additive_initializer"
        ),
        concat!(
            "recognize_selected_plain_identifier_",
            "reference_additive_chain"
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
            "use super::qualification_selected_identifier_reference_right_unary_",
            "additive_initializer_validation_tests"
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
    // accepted oracle at the same `super::super::unicode` path (this leaf
    // sits one module level deeper, inside `qualification_validation_tests`).
    assert!(
        THIS_ORACLE_SOURCE.contains(
            "use super::super::unicode::{is_id_continue, is_id_start, is_space_separator}"
        )
    );
}

/// Genuine iterative model, not an unrolled fixed-cardinality theorem
/// (acceptance criteria 2, 3, 4; W1): the recognizer is defined with a real
/// `loop`, never named `first`/`second`/`third`/`fourth` fields, and no
/// hardcoded four-or-five-operand cap exists anywhere in the source.
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
    for forbidden in [
        concat!("four", "th: RecognizedOperand"),
        concat!("fif", "th: RecognizedOperand"),
        concat!("struct", " Four"),
        concat!("struct", " Five"),
        concat!("Operand", "Four"),
        concat!("Operand", "Five"),
        concat!("enum ", "ExactlyFour"),
        concat!("operands.len()", " == 4"),
        concat!("operands.len()", " == 5"),
        concat!("operands.len()", " >= 4 &&"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }
    assert!(THIS_ORACLE_SOURCE.contains("operands.len() >= 2"));
}

/// Required cardinality witnesses (acceptance criteria 2, 6): 2, 3, 4, 5, and
/// a long 8-operand sentinel all recognize with exactly one fact per
/// authored operand, in exact authored order, mixing plain and
/// optionally-wrapped operands at various positions. The 8-operand sentinel
/// exists to falsify an accidental fixed cap, not as a stress benchmark.
#[test]
fn cardinality_witnesses_from_two_through_a_long_sentinel_all_recognize() {
    let expect = |source: &str, expected_names: &[&str]| {
        let facts =
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(source)
                .unwrap_or_else(|| panic!("{source:?} must recognize the theorem"));
        assert_eq!(facts.operands.len(), expected_names.len());
        for (operand, expected) in facts.operands.iter().zip(expected_names) {
            assert_eq!(&operand.semantic_name, expected);
        }
    };

    // 2.
    expect("a+b", &["a", "b"]);
    expect("+a+b", &["a", "b"]);
    expect("a+-b", &["a", "b"]);
    expect("+a+-b", &["a", "b"]);

    // 3.
    expect("a+b+c", &["a", "b", "c"]);
    expect("+a+b+c", &["a", "b", "c"]);
    expect("a+-b+c", &["a", "b", "c"]);
    expect("a+b+-c", &["a", "b", "c"]);
    expect("+a+-b+ +c", &["a", "b", "c"]);

    // 4.
    expect("+a+-b+c- -d", &["a", "b", "c", "d"]);
    expect("-a+ +b-c+ +d", &["a", "b", "c", "d"]);

    // 5+.
    expect("+a-b+ +c-d+-e", &["a", "b", "c", "d", "e"]);

    // 8-operand long sentinel: falsifies any accidental fixed cap.
    expect(
        "+a-b+ +c-d+-e+f- -g+h",
        &["a", "b", "c", "d", "e", "f", "g", "h"],
    );
}

/// Required positive matrix with fixture-owned literal byte ranges for every
/// operand -- never derived by search, rescan, or reparse. Facts are
/// retained in authored left-to-right order, each with its own exact
/// authored anchor, whether plain or optionally wrapped, and the leading
/// unary sign is never part of a retained anchor (acceptance criteria 5, 14,
/// 15; W8/W9/W11).
#[test]
fn positive_matrix_pins_ordered_facts_with_exact_provenance() {
    // "const x = +a+-b+c- -d;"
    //            0123456789...
    // Byte offsets (verified against the literal below): `+`=10 `a`=11
    // `+`=12 `-`=13 `b`=14 `+`=15 `c`=16 `-`=17 ` `=18 `-`=19 `d`=20 `;`=21.
    // operand1 = "a" wrapped by leading "+" (opposite-sign boundary follows).
    // operand2 = "b" wrapped by leading "-" (opposite sign vs. the preceding
    // binary "+", so zero boundary trivia is accepted).
    // operand3 = "c" plain.
    // operand4 = "d" wrapped by leading "-" (same sign as the preceding
    // binary "-", so the single space boundary trivia is required).
    let text = "const x = +a+-b+c- -d;";
    let whole = Range(10, 21);
    let operand_ranges = [Range(11, 12), Range(14, 15), Range(16, 17), Range(20, 21)];
    let expected_names = ["a", "b", "c", "d"];

    assert_eq!(slice(text, whole), "+a+-b+c- -d");
    for (range, expected) in operand_ranges.iter().zip(expected_names) {
        assert_eq!(slice(text, *range), expected);
    }

    let whole_text = slice(text, whole);
    let facts =
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain(whole_text)
            .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

    assert_eq!(facts.operands.len(), 4);
    for (index, (operand_range, expected_name)) in
        operand_ranges.iter().zip(expected_names).enumerate()
    {
        assert_eq!(slice(text, *operand_range), expected_name);
        assert_eq!(facts.operands[index].authored, expected_name);
        assert_eq!(facts.operands[index].semantic_name, expected_name);
        assert_eq!(facts.operands[index].spelling, OperandSpellingKind::Direct);
        assert_eq!(
            authored_anchor(809_000 + index as u64, text, *operand_range),
            expected_name
        );
    }
}

/// Duplicate preservation theorem (acceptance criterion 16; W11): authored
/// left-to-right order is load-bearing and never reordered by semantic name;
/// equal semantic names are never deduplicated into fewer facts, whether the
/// duplicated operand is plain, opposite-sign wrapped, or same-sign wrapped
/// with trivia, exactly as the Issue's own `+a+-a+a- -a` example requires.
#[test]
fn authored_order_is_preserved_and_equal_semantic_names_are_never_deduplicated() {
    let reordered =
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain("c+a+b+d")
            .expect("c+a+b+d must recognize the theorem");
    let reordered_names: Vec<_> = reordered
        .operands
        .iter()
        .map(|operand| operand.semantic_name.as_str())
        .collect();
    assert_eq!(reordered_names, ["c", "a", "b", "d"]);

    let all_duplicate =
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain("+a+-a+a- -a")
            .expect("+a+-a+a- -a must recognize the theorem");
    assert_eq!(all_duplicate.operands.len(), 4);
    for operand in &all_duplicate.operands {
        assert_eq!(operand.semantic_name, "a");
        assert_eq!(operand.authored, "a");
    }

    // Built with `concat!` over individually escaped fragments so the
    // literal bytes `\`, `u`, `0`, `0`, `6`, `1` survive intact rather than
    // being collapsed into a decoded Unicode scalar by an intermediate write
    // layer, matching the remediation Issue #752/#753 already applied.
    const MIXED_SPELLING_DUPLICATE_SOURCE: &str =
        concat!("a+", "\\", "u0061", "+a+", "\\", "u0061");
    const ESCAPED_A: &str = concat!("\\", "u0061");
    let mixed_spelling_duplicate =
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain(
            MIXED_SPELLING_DUPLICATE_SOURCE,
        )
        .expect("mixed direct/escaped duplicate source must recognize the theorem");
    assert_eq!(mixed_spelling_duplicate.operands.len(), 4);
    let expected_authored = ["a", ESCAPED_A, "a", ESCAPED_A];
    let expected_spelling = [
        OperandSpellingKind::Direct,
        OperandSpellingKind::EscapedNonReserved,
        OperandSpellingKind::Direct,
        OperandSpellingKind::EscapedNonReserved,
    ];
    for (index, operand) in mixed_spelling_duplicate.operands.iter().enumerate() {
        assert_eq!(operand.authored, expected_authored[index]);
        assert_eq!(operand.spelling, expected_spelling[index]);
        assert_eq!(operand.semantic_name, "a");
    }
    assert_ne!(
        mixed_spelling_duplicate.operands[0].authored,
        mixed_spelling_duplicate.operands[1].authored
    );
}

/// Direct/escaped provenance at arbitrary positions, including wrapped
/// positions (acceptance criterion 13): first, interior, and final operands
/// each preserve their own authored spelling distinct from their decoded
/// semantic name, exactly matching the Issue's representative
/// `+a+b+c+d` / `a+-b+c+d` / `a+b+ +c+d` / `a+b+c-d` set.
#[test]
fn escaped_operands_at_arbitrary_wrapped_and_plain_positions_preserve_authored_vs_decoded_identity()
{
    struct PositionFixture {
        source: &'static str,
        escaped_index: usize,
        expected_authored: &'static str,
    }

    let fixtures = [
        // Wrapped escaped first operand: `+a+b+c+d`.
        PositionFixture {
            source: concat!("+", "\\", "u0061", "+b+c+d"),
            escaped_index: 0,
            expected_authored: concat!("\\", "u0061"),
        },
        // Opposite-sign wrapped escaped second operand: `a+-b+c+d`.
        PositionFixture {
            source: concat!("a+-", "\\", "u0062", "+c+d"),
            escaped_index: 1,
            expected_authored: concat!("\\", "u0062"),
        },
        // Same-sign wrapped (non-empty trivia) escaped third operand:
        // `a+b+ +c+d`.
        PositionFixture {
            source: concat!("a+b+ +", "\\", "u0063", "+d"),
            escaped_index: 2,
            expected_authored: concat!("\\", "u0063"),
        },
        // Wrapped escaped final operand: `a+b+c-d`.
        PositionFixture {
            source: concat!("a+b+c-", "\\", "u0064"),
            escaped_index: 3,
            expected_authored: concat!("\\", "u0064"),
        },
    ];

    let expected_names = ["a", "b", "c", "d"];
    for fixture in fixtures {
        let facts = recognize_selected_optional_plus_minus_identifier_reference_additive_chain(
            fixture.source,
        )
        .unwrap_or_else(|| panic!("{:?} must recognize the theorem", fixture.source));
        assert_eq!(facts.operands.len(), 4);
        for (index, operand) in facts.operands.iter().enumerate() {
            assert_eq!(operand.semantic_name, expected_names[index]);
            if index == fixture.escaped_index {
                assert_eq!(operand.authored, fixture.expected_authored);
                assert_eq!(operand.spelling, OperandSpellingKind::EscapedNonReserved);
                assert_ne!(operand.authored, operand.semantic_name);
            } else {
                assert_eq!(operand.authored, expected_names[index]);
                assert_eq!(operand.spelling, OperandSpellingKind::Direct);
            }
        }
    }
}

/// Load-bearing binary/unary punctuator-boundary matrix (acceptance criteria
/// 7, 8, 9, 10; core of Issue #809): the accepted Issue #777 boundary rule
/// applies independently at the first, an interior, the final, and a
/// long-chain continuation, never merely at the first occurrence.
/// Opposite-sign adjacency is always accepted whether or not trivia
/// separates the two operators; same-sign adjacency is accepted only with
/// non-empty separating trivia and rejected with none, because zero-trivia
/// same-sign adjacency is one `++`/`--` `UpdateExpression`-adjacent
/// punctuator, never two selected operators (W5, W6, W7).
#[test]
fn binary_unary_punctuator_boundary_matrix_applies_independently_at_every_continuation() {
    // Opposite-sign adjacency, zero trivia, at the first and only boundary.
    for accepted in ["a+-b", "a-+b"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(accepted)
                .is_some(),
            "{accepted:?}"
        );
    }

    // Same-sign adjacency, non-empty trivia, at the first and only boundary.
    for accepted in ["a+ +b", "a- -b", "a+\t+b", "a+\n+b"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(accepted)
                .is_some(),
            "{accepted:?}"
        );
    }

    // Same-sign adjacency, zero trivia: rejected at the first boundary.
    for rejected in ["a++b", "a--b"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(rejected)
                .is_none(),
            "{rejected:?}"
        );
    }

    // The boundary rule at an INTERIOR continuation, with a preceding and
    // following plain operand on each side.
    for accepted in ["a+b+-c+d", "a+b-+c+d", "a+b+ +c+d", "a+b- -c+d"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(accepted)
                .is_some(),
            "{accepted:?}"
        );
    }
    for rejected in ["a+b++c+d", "a+b--c+d"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(rejected)
                .is_none(),
            "{rejected:?}"
        );
    }

    // The boundary rule at the FINAL continuation.
    for accepted in ["a+b+c+-d", "a+b+c-+d", "a+b+c+ +d", "a+b+c- -d"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(accepted)
                .is_some(),
            "{accepted:?}"
        );
    }
    for rejected in ["a+b+c++d", "a+b+c--d"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(rejected)
                .is_none(),
            "{rejected:?}"
        );
    }

    // A long-chain candidate applying the rule at every one of its several
    // boundaries at once: opposite-sign zero-trivia, same-sign with-trivia,
    // and plain continuations all appear together, exactly matching the
    // Issue's own long sentinel.
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain(
            "+a-b+ +c-d+-e+f- -g+h"
        )
        .is_some()
    );

    // The corresponding long-chain zero-trivia same-sign violation at an
    // interior boundary (`+ +` tightened to `++`) is rejected.
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain(
            "+a-b++c-d+-e+f- -g+h"
        )
        .is_none()
    );
}

/// UpdateExpression / longest-input-element firewall (acceptance criterion
/// 10; W5): authored `++`/`--` is never split into a binary sign and a
/// following unary sign, at the first, an interior, a final, or a leading
/// continuation boundary.
#[test]
fn update_expression_and_longest_input_element_firewall_remains_outside() {
    for control in [
        "a++b+c", "a--b+c", "a+b++c", "a+b--c", "+a++b+c", "+a+b--c", "a+++b+c", "a---b+c",
        "a+b+++c", "a+b---c",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
}

/// Exactly-one unary wrapper / recursive-unary firewall (acceptance
/// criterion 11; W12): a second leading sign immediately following the
/// first is rejected at the leading operand and at any continuation operand.
#[test]
fn recursive_unary_wrapper_firewall_remains_outside() {
    for control in ["+-a+b", "-+a+b", "a+-+b+c", "a-+-b+c", "a+b+-+c", "a+b-+-c"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain("a+-b+c")
            .is_some()
    );
}

/// Other unary operator families remain outside (acceptance criterion 12;
/// W15): `!`, `~`, `typeof`, `void`, `delete` never leak into this bounded
/// `+`/`-` theorem at any operand position.
#[test]
fn other_unary_operator_families_remain_outside() {
    for control in [
        "!a+b+c",
        "~a+b+c",
        "a+!b+c",
        "a+b+~c",
        "typeof a+b",
        "void a+b",
        "delete a+b",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
}

/// Heterogeneous operand firewall (acceptance criterion 19; W14): numeric,
/// boolean, `null`, `this`, and string-literal operands remain outside at
/// any position, plain or in the position a unary sign could otherwise wrap.
#[test]
fn heterogeneous_operand_firewall_remains_outside() {
    for control in [
        "a+1+b",
        "a+b+1",
        "1+a+b",
        "a+true+b",
        "a+null+b",
        "this+a+b",
        "\"a\"+b+c",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
}

/// Parenthesized / precedence / member / call / assignment / conditional /
/// comma firewall (acceptance criterion 20; W16): none of these compose with
/// this exact bounded theorem, and no dedicated recognition code for any of
/// them exists in this Oracle.
#[test]
fn parenthesized_precedence_member_call_assignment_richer_expression_firewall_remains_outside() {
    for control in [
        "(a)+b+c", "a+(b)+c", "a*b+c", "a+b*c+d", "a.b+c", "a+b.c+d", "a()+b+c", "a+b()+c",
        "a=b+c", "a+b=c", "a+b?c:d", "a+b,c",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain("a+b+c")
            .is_some()
    );
}

/// Comments and U+200B remain outside selected trivia at every trivia
/// position this theorem owns; an intra-expression `LineTerminator` that is
/// already selected trivia composes without implying general ASI
/// (acceptance criterion 21; W17).
#[test]
fn comments_and_zero_width_space_remain_outside_selected_trivia_at_every_position() {
    for comment_control in [
        "a/*x*/+b+c",
        "a+/*x*/b+c",
        "a+b/*x*/+c",
        "a+b+/*x*/c",
        "+a/*x*/+b+c",
        "a+-/*x*/b+c",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(
                comment_control
            )
            .is_none(),
            "{comment_control:?}"
        );
    }

    assert!(!is_selected_additive_trivia('\u{200B}'));
    assert!(is_selected_additive_trivia('\u{00A0}'));
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain(
            "a+\u{200B}+b+c"
        )
        .is_none()
    );

    // The corresponding already-accepted LineTerminator trivia composes
    // without implying any general ASI widening.
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain("a+\n+b+c")
            .is_some()
    );
}

/// Escaped `ReservedWord` firewall at first, interior, and final positions,
/// both plain and unary-wrapped (acceptance criterion 17; W13): a decoded
/// unconditionally reserved word never becomes a selected operand anywhere.
#[test]
fn escaped_reserved_word_firewall_first_interior_final_plain_and_wrapped_positions() {
    for control in [
        concat!("\\", "u0069f+b+c"),
        concat!("+", "\\", "u0069f+b+c"),
        concat!("a+", "\\", "u0069f+c"),
        concat!("a+-", "\\", "u0069f+c"),
        concat!("a+b+", "\\", "u0069f"),
        concat!("a+b-", "\\", "u0069f"),
    ] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0069", "f")),
        Err(DecodeFailure::DecodedReserved)
    );
}

/// Malformed escape firewall at first, interior, and final positions
/// (acceptance criterion 18; W1/W21): a malformed escape never leaks a
/// truncated prefix or a valid sibling fact -- the whole candidate is
/// rejected.
#[test]
fn malformed_escape_firewall_first_interior_final_positions_never_leaks_a_sibling_fact() {
    for control in [
        r"\u{}+b+c",
        r"+\u{}+b+c",
        r"a+\u{}+c",
        r"a+-\u{}+c",
        r"a+b+\u{}",
        r"a+b-\u{}",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{}"),
        Err(DecodeFailure::MalformedEscape)
    );
}

/// Whole-candidate transactionality and no-prefix-truncation (acceptance
/// criteria 22, 23; W1/W21): an incomplete or richer-tail chain never
/// publishes a prepared prefix, and a longer, otherwise-valid-looking
/// candidate with a malformed or richer final segment never truncates to a
/// shorter accepted result. Because the return type is `Option`, a rejected
/// candidate structurally cannot yield a partial `Vec` of fewer facts.
#[test]
fn whole_candidate_transactionality_never_commits_a_partial_or_truncated_result() {
    for control in [
        "+a+-b+c+",
        concat!("+a+-b+c+", "\\", "u0069f"),
        r"+a+-b+\u{}+d",
        "+a+-b+c.d",
        "+a+-b+c()",
        "+a+-b+c=d",
        "+a+-b+c,d",
        "+a+-b+c?d:e",
    ] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }

    // The complete bounded prefix itself remains independently valid, and a
    // rejected longer candidate never truncates to it.
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain("+a+-b+c")
            .is_some()
    );
}

/// Cardinality firewall (acceptance criterion 1; W1): a single plain or
/// unary-wrapped `IdentifierReference` remains outside this exact `2..N`
/// theorem, owned instead by already-accepted single-reference/
/// single-`UnaryExpression` authority.
#[test]
fn single_reference_cardinality_remains_outside_the_two_to_n_theorem() {
    for control in ["a", "foo", "+a", "-a", r"a"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain("a+b").is_some()
    );
}

/// Outer trivia firewall: leading and trailing statement/declaration-level
/// trivia does not belong to the naked theorem and is never silently
/// absorbed, whatever internal trivia the same candidate otherwise accepts.
#[test]
fn outer_leading_and_trailing_trivia_remain_outside_the_naked_theorem() {
    assert!(
        recognize_selected_optional_plus_minus_identifier_reference_additive_chain("a+b+c")
            .is_some()
    );
    for control in [" a+b+c", "a+b+c ", " a+b+c ", "\ta+b+c", "a+b+c\n"] {
        assert!(
            recognize_selected_optional_plus_minus_identifier_reference_additive_chain(control)
                .is_none(),
            "{control:?}"
        );
    }
}

/// Resource / failure semantics (W1 adjacent): the project's
/// `ResourceLimited`/`InternalFailure` lifecycle states remain distinct from
/// `UnsupportedCoverage`, matching the minimal symbolic model already
/// accepted by the predecessor candidate-independent `IdentifierReference`
/// oracles (#241, #746/#747, #752/#753, #777/#778, #795/#796, #801/#802) --
/// this leaf adds no further resource machinery beyond that already-accepted
/// minimum.
#[test]
fn resource_and_internal_failure_lifecycle_states_remain_distinct_from_unsupported_coverage() {
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
    assert!(THIS_ORACLE_SOURCE.contains("ResourceLimited"));
    assert!(THIS_ORACLE_SOURCE.contains("InternalFailure"));
    assert_ne!(PROCESSING_FAILURES[0], "UnsupportedCoverage");
    assert_ne!(PROCESSING_FAILURES[1], "UnsupportedCoverage");
}

/// Freezes the presence-only, unqualified, validation-only handoff: no
/// operator enum, operator/whole-expression/trivia `SourceAnchor`, no
/// unary-sign-carrying anchor, no binary-expression node, no AST/CST, no
/// token tape, no runtime/static vocabulary, and no frozen production
/// collection-representation choice is introduced (acceptance criteria 26,
/// 27, 28, 30; W8/W9/W23/W24).
#[test]
fn handoff_remains_validation_only_with_no_operator_runtime_or_production_representation_freeze() {
    for forbidden in [
        concat!("enum Selected", "AdditiveOperatorKind"),
        concat!("enum Operator", "Kind"),
        concat!("enum Plus", "Minus"),
        concat!("struct Operator", "SourceAnchor"),
        concat!("struct WholeExpression", "SourceAnchor"),
        concat!("struct Unary", "SourceAnchor"),
        concat!("struct Trivia", "SourceAnchor"),
        concat!("struct Binary", "ExpressionNode"),
        concat!("struct Additive", "ExpressionNode"),
        concat!("struct Unary", "ExpressionNode"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("enum One", "Two", "Three", "Many"),
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

    assert!(
        THIS_ORACLE_SOURCE
            .contains("struct OptionalPlusMinusIdentifierReferenceAdditiveChainFacts")
    );
    assert!(THIS_ORACLE_SOURCE.contains("operands: Vec<RecognizedOperand"));
    assert!(THIS_ORACLE_SOURCE.contains("ORACLE DYNAMIC STORAGE"));
    assert!(THIS_ORACLE_SOURCE.contains("PRODUCTION STORAGE AUTHORITY"));
    assert!(THIS_ORACLE_SOURCE.contains("does **not** freeze future production representation"));
    assert!(THIS_ORACLE_SOURCE.contains("One | Two | Three | Many"));
    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}

/// Completion hard-zero (acceptance criteria 27, 28, 29; W25): this
/// validation-only leaf adds zero production capability, so it must not
/// alter the frozen `193 / 10 / 183` rule-unit partition. The authoritative
/// count assertions already live in `qualification_validation_tests::mod`
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
