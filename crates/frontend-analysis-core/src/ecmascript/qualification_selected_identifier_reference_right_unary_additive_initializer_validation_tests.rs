//! Candidate-independent right-unary exactly-two `IdentifierReference`
//! additive token-boundary fact-preservation validation for Issue #777
//! (durable research: Issue #688 comment `5746900561`; accepted predecessor
//! authorities: Issue #746 / PR #747 leading `+`/`-` direct
//! `IdentifierReference` `UnaryExpression` fact-preservation Oracle, Issue
//! #750 / PR #751 escaped non-Reserved `IdentifierReference` unary
//! composition, Issue #752 / PR #753 candidate-independent ordered
//! two-`IdentifierReference` additive initializer theorem, Issue #241
//! candidate-independent escaped `IdentifierReference` authored-source/
//! decoded-name theorem, Issue #773/#774 and #775/#776 accepted left-unary
//! production precedents).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedIdentifierReferenceRightUnaryAdditiveInitializer ::=
//!     SelectedAcceptedIdentifierReference
//!     SelectedAdditiveTriviaBeforeBinary
//!     SelectedBinaryPlusMinus
//!     SelectedBinaryUnaryBoundary
//!     SelectedUnaryPlusMinus
//!     SelectedUnaryOperandTrivia
//!     SelectedAcceptedIdentifierReference
//!
//! SelectedBinaryPlusMinus ::= "+" | "-"
//! SelectedUnaryPlusMinus  ::= "+" | "-"
//!
//! SelectedAcceptedIdentifierReference ::=
//!     SelectedDirectIdentifierReference
//!   | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice. It
//! does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! The new load-bearing capability this Oracle adds beyond #746/#747 and
//! #752/#753 is `SelectedBinaryUnaryBoundary`: the punctuator boundary
//! between the binary additive operator and the right operand's own leading
//! unary operator. ECMAScript's lexical longest-input-element rule makes
//! `++` and `--` single `UpdateExpression`-adjacent punctuators, distinct
//! from two separate `+`/`-` code points. This Oracle independently proves:
//!
//! ```text
//! binary != unary:
//!     selected trivia between the two operators MAY be empty
//! binary == unary:
//!     selected trivia between the two operators MUST be non-empty
//! ```
//!
//! so that `a+-b` / `a-+b` are accepted with zero inter-operator trivia while
//! `a++b` / `a--b` are rejected from this theorem, and `a+ +b` / `a- -b` are
//! accepted only because non-empty selected trivia separates the two
//! punctuators. This is the first bounded family this project independently
//! proves must discriminate two adjacent same-family punctuators by trivia
//! cardinality rather than by code-point identity alone.
//!
//! `SelectedAdditiveTriviaBeforeBinary` and `SelectedUnaryOperandTrivia`
//! independently restate the complete already-accepted selected-slice trivia
//! contract established by Issue #742/#743 and restated again by Issue
//! #746/#747 and #752/#753 (the same code-point set the production
//! selected-trivia recognizer accepts): `TAB`, `VT`, `FF`, `BOM`, `LF`, `CR`,
//! `LINE SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen Unicode 17
//! `Space_Separator` property. Comments remain outside this contract and are
//! never accepted as trivia at any of the three trivia positions.
//!
//! `SelectedDirectIdentifierReference` and
//! `SelectedEscapedNonReservedIdentifierReference` independently restate the
//! same already-accepted Issue #237/#241 premises #746/#747 and #752/#753
//! already restate. Neither restatement imports another Oracle's code: this
//! leaf owns its own copy of every already-accepted primitive it consumes.
//!
//! This is a validation-only leaf: production supports no right-unary
//! additive expression family at the #777 baseline, so every positive
//! fixture below remains `UnsupportedCoverage` under current production and
//! exists only as independent Oracle evidence for a future, separately
//! authorized production decision. No completion successor file accompanies
//! this leaf: this Issue adds zero production capability, so the frozen
//! `193 / 10 / 183 / {}` completion partition cannot move, matching the
//! precedent set by #742/#743, #746/#747, and #752/#753.

use std::cmp::Ordering;

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 777;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_UNARY_IDENTIFIER_REFERENCE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_leading_plus_minus_direct_identifier_reference_unary_expression_initializer_validation_tests.rs"
);
const PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_identifier_reference_right_unary_additive_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "right-unary exactly-two IdentifierReference additive token-boundary ",
    "frontier only; later independently qualified owners may strengthen ",
    "classification for recursive right-unary, both-unary composition, ",
    "other unary operator families, heterogeneous numeric/literal operands, ",
    "3+ operand chains, comments, or production representation/placement"
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    SelectedAcceptedIncomplete,
    UnsupportedCoverage,
    StaticSemanticsRejected,
    SyntaxRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedLexicalBindingOrder {
    Before,
    Same,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedBindingScopeTarget {
    SameSourceSelectedLexicalBinding(ExpectedLexicalBindingOrder),
    NoSameSourceSelectedLexicalBinding,
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

// --- Direct IdentifierReference policy: independently restated from Issue
// #237/#746/#752, never imported from any of those Oracles. ---

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
// as Issue #746/#747 and #752/#753 already restate it. Reused unchanged at
// all three trivia positions this theorem owns (before the binary operator,
// between the binary and unary operators, and between the unary operator and
// the right operand): there is exactly one selected-trivia contract, not
// three different ones. ---

fn is_selected_right_unary_additive_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

fn trivia_run_end(candidate: &str) -> usize {
    let mut offset = 0_usize;
    for code_point in candidate.chars() {
        if !is_selected_right_unary_additive_trivia(code_point) {
            break;
        }
        offset += code_point.len_utf8();
    }
    offset
}

// --- The Issue #777 theorem itself. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperandSpellingKind {
    Direct,
    EscapedNonReserved,
}

/// One retained operand fact. A private test-only shape, not a claim about
/// future production storage.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedOperand<'a> {
    authored: &'a str,
    semantic_name: String,
    spelling: OperandSpellingKind,
}

/// The Oracle's only retained representation: exactly two ordered operand
/// facts. This struct shape is a minimal private test representation, never
/// a claim that future production must use `Vec`, an array, a tuple, a
/// `SmallVec`, or any other specific collection type. No operator identity,
/// trivia identity, or whole-expression `SourceAnchor` is retained here:
/// those are Oracle-owned range evidence proven directly by fixtures/tests,
/// never part of the returned facts.
#[derive(Debug, Clone, PartialEq, Eq)]
struct IdentifierReferenceRightUnaryAdditiveFacts<'a> {
    left: RecognizedOperand<'a>,
    right: RecognizedOperand<'a>,
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

/// Finds where the left operand ends: one left-to-right forward pass
/// consuming either one direct code point or one syntactically well-formed
/// `UnicodeEscapeSequence` element, stopping at the first selected-trivia
/// code point or at `+`/`-`. A malformed escape aborts the whole candidate
/// immediately (`None`) rather than silently truncating the run at that
/// point. Position validity of the collected run is judged afterward by
/// `recognize_accepted_identifier_reference`, never here. Independently
/// restated from Issue #752's `left_operand_run_end`, never imported.
fn left_operand_run_end(candidate: &str) -> Option<usize> {
    let mut offset = 0_usize;
    while offset < candidate.len() {
        if candidate.as_bytes()[offset] == b'\\' {
            let formation = formed_escape_at(candidate, offset).ok()?;
            offset = formation.end;
            continue;
        }
        let code_point = candidate[offset..].chars().next()?;
        if is_selected_right_unary_additive_trivia(code_point)
            || code_point == '+'
            || code_point == '-'
        {
            break;
        }
        offset += code_point.len_utf8();
    }
    (offset > 0).then_some(offset)
}

/// Central Issue #777 theorem: `SelectedBinaryUnaryBoundary`. Whole-candidate,
/// all-or-nothing. The right operand's own leading unary operator is
/// recognized only to prove the exact bounded source family and the load-
/// bearing punctuator-cardinality boundary; it is never retained in the
/// returned facts, and its sign never becomes part of the right reference's
/// authored anchor (W5/W7).
///
/// The boundary rule is exactly: if the binary and unary operators are the
/// same code point, the trivia run between them must be non-empty (otherwise
/// the source is one `++`/`--` `UpdateExpression`-adjacent punctuator, not
/// two selected operators); if they differ, the trivia run between them may
/// be empty or non-empty (opposite-sign adjacency such as `+-`/`-+` is never
/// one combined punctuator under current ECMAScript lexical grammar).
fn recognize_selected_identifier_reference_right_unary_additive_initializer(
    candidate: &str,
) -> Option<IdentifierReferenceRightUnaryAdditiveFacts<'_>> {
    let left_end = left_operand_run_end(candidate)?;
    let (left_slice, after_left) = candidate.split_at(left_end);
    let left = recognize_accepted_identifier_reference(left_slice)?;

    let after_left_trivia = &after_left[trivia_run_end(after_left)..];
    let mut after_left_trivia_chars = after_left_trivia.chars();
    let binary = match after_left_trivia_chars.next() {
        Some(sign @ ('+' | '-')) => sign,
        _ => return None,
    };
    let after_operator = &after_left_trivia[1..];

    let boundary_trivia_end = trivia_run_end(after_operator);
    let boundary_trivia_is_empty = boundary_trivia_end == 0;
    let after_boundary = &after_operator[boundary_trivia_end..];

    let mut after_boundary_chars = after_boundary.chars();
    let unary = match after_boundary_chars.next() {
        Some(sign @ ('+' | '-')) => sign,
        _ => return None,
    };

    // SelectedBinaryUnaryBoundary: same-sign adjacency (`++`/`--`) requires
    // non-empty separating trivia; opposite-sign adjacency (`+-`/`-+`) never
    // does (W3/W4).
    if binary == unary && boundary_trivia_is_empty {
        return None;
    }

    let after_unary = &after_boundary[1..];
    let unary_operand_trivia_end = trivia_run_end(after_unary);
    let right_slice = &after_unary[unary_operand_trivia_end..];
    let right = recognize_accepted_identifier_reference(right_slice)?;

    Some(IdentifierReferenceRightUnaryAdditiveFacts { left, right })
}

/// Independently restates only the already-accepted premise that a selected
/// `IdentifierReference` composes with existing lexical Binding/Scope
/// meaning by comparing declaration-order positions -- the same conceptual
/// relation the production Binding/Scope analysis in
/// `selected_binding_scope.rs` derives, restated here from a fixture-owned
/// authored binding inventory rather than by calling production.
fn expected_binding_scope_target(
    binding_inventory: &[&str],
    containing_binding_name: &str,
    reference_semantic_name: &str,
) -> ExpectedBindingScopeTarget {
    let containing_position = binding_inventory
        .iter()
        .position(|name| *name == containing_binding_name)
        .expect("fixture-owned containing binding must be present in the inventory");

    match binding_inventory
        .iter()
        .position(|name| *name == reference_semantic_name)
    {
        None => ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
        Some(target_position) => {
            let order = match target_position.cmp(&containing_position) {
                Ordering::Less => ExpectedLexicalBindingOrder::Before,
                Ordering::Equal => ExpectedLexicalBindingOrder::Same,
                Ordering::Greater => ExpectedLexicalBindingOrder::After,
            };
            ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(order)
        }
    }
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 777);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_UNARY_IDENTIFIER_REFERENCE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 746"));
    assert!(
        PREVIOUS_UNARY_IDENTIFIER_REFERENCE_ORACLE_SOURCE.contains(
            "leading plus/minus direct IdentifierReference UnaryExpression frontier only"
        )
    );
    assert!(
        PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 752")
    );
    assert!(
        PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE
            .contains("ordered two-IdentifierReference additive initializer frontier only")
    );
    assert!(FRONTIER_SCOPE_NOTE.contains(
        "right-unary exactly-two IdentifierReference additive token-boundary frontier only"
    ));
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
        concat!(
            "consume_selected_leading_plus_minus_identifier_",
            "reference_left_additive_initializer"
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
        concat!("use super::", "selected_lexical_slice::is_selected_trivia"),
        // No cross-Oracle import: this leaf restates rather than imports.
        concat!(
            "use super::qualification_selected_leading_plus_minus_direct_",
            "identifier_reference_unary_expression_initializer_validation_tests"
        ),
        concat!(
            "use super::qualification_selected_two_identifier_reference_",
            "additive_initializer_validation_tests"
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
    // accepted oracles at the same `super::unicode` path.
    assert!(
        THIS_ORACLE_SOURCE
            .contains("use super::unicode::{is_id_continue, is_id_start, is_space_separator}")
    );
}

/// Required positive opposite-sign adjacency matrix (acceptance criterion 1):
/// `+-` and `-+` are accepted with zero inter-operator trivia, including a
/// multibyte direct operand pair, with fixture-owned literal byte ranges for
/// left operand, binary operator, unary operator, and right operand -- never
/// derived by search, rescan, or reparse (W3, W5, W7).
#[test]
fn positive_opposite_sign_zero_trivia_matrix_pins_two_ordered_facts_with_exact_provenance() {
    const POSITIVE_MATRIX: &[(&str, Range, Range, Range, Range, &str, &str)] = &[
        (
            "const x=a+-b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            "a",
            "b",
        ),
        (
            "const x=a-+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            "a",
            "b",
        ),
        (
            "const x=\u{03C0}+-\u{1D49C};",
            Range(8, 10),
            Range(10, 11),
            Range(11, 12),
            Range(12, 16),
            "\u{03C0}",
            "\u{1D49C}",
        ),
    ];

    for (index, (text, left, binary, unary, right, expected_left, expected_right)) in
        POSITIVE_MATRIX.iter().enumerate()
    {
        assert_eq!(slice(text, *left), *expected_left);
        assert!(matches!(slice(text, *binary), "+" | "-"));
        assert!(matches!(slice(text, *unary), "+" | "-"));
        assert_ne!(slice(text, *binary), slice(text, *unary));
        assert_eq!(slice(text, *right), *expected_right);
        // The left fact never includes the binary operator, and the right
        // fact never includes the unary sign (W5).
        assert_eq!(left.1, binary.0);
        assert_eq!(binary.1, unary.0);
        assert_eq!(unary.1, right.0);

        let whole = Range(left.0, right.1);
        let whole_text = slice(text, whole);
        let facts =
            recognize_selected_identifier_reference_right_unary_additive_initializer(whole_text)
                .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(facts.left.authored, *expected_left);
        assert_eq!(facts.left.semantic_name, *expected_left);
        assert_eq!(facts.left.spelling, OperandSpellingKind::Direct);
        assert_eq!(facts.right.authored, *expected_right);
        assert_eq!(facts.right.semantic_name, *expected_right);
        assert_eq!(facts.right.spelling, OperandSpellingKind::Direct);

        assert_eq!(
            authored_anchor(777_000 + index as u64, text, *left),
            *expected_left
        );
        assert_eq!(
            authored_anchor(777_050 + index as u64, text, *right),
            *expected_right
        );
    }
}

/// Load-bearing punctuator-boundary matrix (acceptance criteria 2, 3, 4;
/// core of Issue #777): same-sign adjacency is accepted only with non-empty
/// inter-operator trivia across the complete accepted trivia code-point set
/// (space, TAB, LF, CR, VT, FF, NBSP, BOM, LINE SEPARATOR, PARAGRAPH
/// SEPARATOR), while the corresponding zero-trivia same-sign adjacency
/// (`++`/`--`) is rejected because it is one `UpdateExpression`-adjacent
/// punctuator, never two selected operators (W1/W2/W4/W6).
#[test]
fn positive_same_sign_nonempty_trivia_matrix_and_zero_trivia_token_boundary_firewall() {
    const NONEMPTY_TRIVIA_MATRIX: &[(&str, Range, Range, Range, Range, Range)] = &[
        (
            "const x=a+ +b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            Range(12, 13),
        ),
        (
            "const x=a- -b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            Range(12, 13),
        ),
        (
            "const x=a+\t+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            Range(12, 13),
        ),
        (
            "const x=a+\n+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            Range(12, 13),
        ),
        (
            "const x=a+\r+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            Range(12, 13),
        ),
        (
            "const x=a+\u{000B}+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            Range(12, 13),
        ),
        (
            "const x=a+\u{000C}+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 11),
            Range(11, 12),
            Range(12, 13),
        ),
        (
            "const x=a+\u{00A0}+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 12),
            Range(12, 13),
            Range(13, 14),
        ),
        (
            "const x=a+\u{FEFF}+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 13),
            Range(13, 14),
            Range(14, 15),
        ),
        (
            "const x=a+\u{2028}+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 13),
            Range(13, 14),
            Range(14, 15),
        ),
        (
            "const x=a+\u{2029}+b;",
            Range(8, 9),
            Range(9, 10),
            Range(10, 13),
            Range(13, 14),
            Range(14, 15),
        ),
    ];

    for (index, (text, left, binary, boundary_trivia, unary, right)) in
        NONEMPTY_TRIVIA_MATRIX.iter().enumerate()
    {
        assert_eq!(slice(text, *left), "a");
        assert_eq!(slice(text, *binary), slice(text, *unary));
        assert!(boundary_trivia.1 > boundary_trivia.0);
        for code_point in slice(text, *boundary_trivia).chars() {
            assert!(is_selected_right_unary_additive_trivia(code_point));
        }
        assert_eq!(slice(text, *right), "b");

        let whole = Range(left.0, right.1);
        let whole_text = slice(text, whole);
        let facts =
            recognize_selected_identifier_reference_right_unary_additive_initializer(whole_text)
                .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));
        assert_eq!(facts.left.semantic_name, "a");
        assert_eq!(facts.right.semantic_name, "b");

        assert_eq!(
            authored_anchor(777_100 + index as u64, text, whole),
            whole_text
        );
    }

    // Zero-trivia same-sign adjacency is one UpdateExpression-adjacent
    // punctuator, not two selected operators, whatever the surrounding
    // additive/unary intent looks like (W1/W2/W4).
    for control in ["a++b", "a--b", "a+++b", "a---b"] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    // Trivia before the binary operator and after the unary operator also
    // compose freely with opposite-sign adjacency (SelectedAdditiveTrivia-
    // BeforeBinary / SelectedUnaryOperandTrivia), independently of the new
    // SelectedBinaryUnaryBoundary rule.
    for spaced in ["a + - b", "a - + b"] {
        let facts =
            recognize_selected_identifier_reference_right_unary_additive_initializer(spaced)
                .unwrap_or_else(|| panic!("{spaced:?} must recognize the theorem"));
        assert_eq!(facts.left.semantic_name, "a");
        assert_eq!(facts.right.semantic_name, "b");
    }
}

/// Direct/escaped cross-product (acceptance criterion 7): every Direct/
/// Direct, Escaped/Direct, Direct/Escaped, and Escaped/Escaped combination
/// decodes to its own semantic name while retaining its own distinct exact
/// authored spelling, using both the braced and classic four-hex-digit
/// `UnicodeEscapeSequence` forms (W5).
///
/// The classic four-hex-digit fixtures are built with `concat!` over
/// individually escaped fragments so the literal bytes `\`, `u`, `0`, `0`,
/// `6`, `1` survive intact rather than being collapsed into a decoded
/// Unicode scalar by an intermediate write layer, matching the established
/// repository-safe construction technique.
#[test]
fn direct_escaped_cross_product_pins_authored_vs_decoded_identity_per_operand() {
    struct CrossProductFixture {
        text: &'static str,
        left: Range,
        right: Range,
        expected_left_authored: &'static str,
        expected_left_semantic: &'static str,
        expected_left_spelling: OperandSpellingKind,
        expected_right_authored: &'static str,
        expected_right_semantic: &'static str,
        expected_right_spelling: OperandSpellingKind,
    }

    let fixtures = [
        // Escaped/Direct, classic form: `a+-b`.
        CrossProductFixture {
            text: concat!("const x=", "\\", "u0061", "+-b;"),
            left: Range(8, 14),
            right: Range(16, 17),
            expected_left_authored: concat!("\\", "u0061"),
            expected_left_semantic: "a",
            expected_left_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_right_authored: "b",
            expected_right_semantic: "b",
            expected_right_spelling: OperandSpellingKind::Direct,
        },
        // Direct/Escaped, classic form: `a-+b`.
        CrossProductFixture {
            text: concat!("const x=a-+", "\\", "u0062", ";"),
            left: Range(8, 9),
            right: Range(11, 17),
            expected_left_authored: "a",
            expected_left_semantic: "a",
            expected_left_spelling: OperandSpellingKind::Direct,
            expected_right_authored: concat!("\\", "u0062"),
            expected_right_semantic: "b",
            expected_right_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        // Escaped/Escaped, classic form, with non-empty inter-operator
        // trivia: `a+ -b`.
        CrossProductFixture {
            text: concat!("const x=", "\\", "u0061", "+ -", "\\", "u0062", ";"),
            left: Range(8, 14),
            right: Range(17, 23),
            expected_left_authored: concat!("\\", "u0061"),
            expected_left_semantic: "a",
            expected_left_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_right_authored: concat!("\\", "u0062"),
            expected_right_semantic: "b",
            expected_right_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        // Mixed embedded escapes on both sides, classic form:
        // `foo- +bar` decodes to `foo` / `bar`.
        CrossProductFixture {
            text: concat!("const x=f", "\\", "u006F", "o- +b", "\\", "u0061", "r;"),
            left: Range(8, 16),
            right: Range(19, 27),
            expected_left_authored: concat!("f", "\\", "u006F", "o"),
            expected_left_semantic: "foo",
            expected_left_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_right_authored: concat!("b", "\\", "u0061", "r"),
            expected_right_semantic: "bar",
            expected_right_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        // Direct/Direct control, braced form left, direct right.
        CrossProductFixture {
            text: r"const x=\u{61}+-b;",
            left: Range(8, 14),
            right: Range(16, 17),
            expected_left_authored: r"\u{61}",
            expected_left_semantic: "a",
            expected_left_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_right_authored: "b",
            expected_right_semantic: "b",
            expected_right_spelling: OperandSpellingKind::Direct,
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        assert_eq!(
            slice(fixture.text, fixture.left),
            fixture.expected_left_authored
        );
        assert_eq!(
            slice(fixture.text, fixture.right),
            fixture.expected_right_authored
        );

        let whole = Range(fixture.left.0, fixture.right.1);
        let whole_text = slice(fixture.text, whole);
        let facts =
            recognize_selected_identifier_reference_right_unary_additive_initializer(whole_text)
                .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(facts.left.authored, fixture.expected_left_authored);
        assert_eq!(facts.left.semantic_name, fixture.expected_left_semantic);
        assert_eq!(facts.left.spelling, fixture.expected_left_spelling);
        assert_eq!(facts.right.authored, fixture.expected_right_authored);
        assert_eq!(facts.right.semantic_name, fixture.expected_right_semantic);
        assert_eq!(facts.right.spelling, fixture.expected_right_spelling);

        if fixture.expected_left_spelling == OperandSpellingKind::EscapedNonReserved {
            assert_ne!(facts.left.authored, facts.left.semantic_name);
        }
        if fixture.expected_right_spelling == OperandSpellingKind::EscapedNonReserved {
            assert_ne!(facts.right.authored, facts.right.semantic_name);
        }

        assert_eq!(
            authored_anchor(777_200 + index as u64, fixture.text, fixture.left),
            fixture.expected_left_authored
        );
        assert_eq!(
            authored_anchor(777_250 + index as u64, fixture.text, fixture.right),
            fixture.expected_right_authored
        );
    }
}

/// Duplicate occurrence and authored-order theorem (acceptance criteria 5,
/// 8): two independently owned facts remain even when their semantic names
/// are equal, and authored left-to-right order is never reordered to
/// operator order, binding-target order, or semantic-name order (W7).
#[test]
fn duplicate_occurrence_and_authored_order_are_never_collapsed_or_reordered() {
    let duplicate_source = "const x=a+-a;";
    let duplicate_whole = slice(duplicate_source, Range(8, 12));
    let duplicate_facts =
        recognize_selected_identifier_reference_right_unary_additive_initializer(duplicate_whole)
            .expect("a+-a must recognize the theorem");
    assert_eq!(
        duplicate_facts.left.semantic_name,
        duplicate_facts.right.semantic_name
    );
    // Two independently owned facts remain even though the semantic names
    // are equal: both fields are populated from their own independent
    // operand slice, never merged into a single set.
    assert_eq!(duplicate_facts.left.authored, "a");
    assert_eq!(duplicate_facts.right.authored, "a");

    let reordered_source = "const x=b+-a;";
    let reordered_whole = slice(reordered_source, Range(8, 12));
    let reordered_facts =
        recognize_selected_identifier_reference_right_unary_additive_initializer(reordered_whole)
            .expect("b+-a must recognize the theorem");
    assert_eq!(reordered_facts.left.authored, "b");
    assert_eq!(reordered_facts.right.authored, "a");
}

/// Comments never compose as trivia at any of the three trivia positions,
/// and the adversarial ZERO WIDTH SPACE boundary (immediately outside the
/// frozen `Space_Separator` range) is never accepted merely because it
/// visually resembles spacing (W6, item 13).
#[test]
fn comments_and_zero_width_space_remain_outside_selected_trivia_at_every_position() {
    for comment_control in ["a+/*c*/+b", "a/*c*/+-b", "a+-/*c*/b", "a+//c\n+b"] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(
                comment_control
            )
            .is_none(),
            "{comment_control:?}"
        );
    }

    assert!(!is_selected_right_unary_additive_trivia('\u{200B}'));
    assert!(is_selected_right_unary_additive_trivia('\u{00A0}'));
    assert!(
        recognize_selected_identifier_reference_right_unary_additive_initializer("a+\u{200B}+b")
            .is_none()
    );

    // The corresponding non-comment, non-ZWSP forms remain accepted.
    assert!(
        recognize_selected_identifier_reference_right_unary_additive_initializer("a+ +b").is_some()
    );
    assert!(
        recognize_selected_identifier_reference_right_unary_additive_initializer("a+-b").is_some()
    );
}

/// Recursive right-unary, both-unary, and other-unary-operator-family
/// firewalls (acceptance criteria 9, 11, 12; W8/W10/W11): exactly one
/// right-side leading `+`/`-` wrapper is owned by this theorem, left-unary
/// wrapping the whole candidate is never automatically composed with the new
/// right-unary theorem, and no other `UnaryExpression` operator family
/// leaks in.
#[test]
fn recursive_right_unary_both_unary_and_other_unary_firewalls_remain_outside() {
    // Recursive right-unary: more than one leading sign on the right side.
    for control in ["a+-+b", "a-+-b", "a+--b", "a-++b"] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    // Both-unary: a leading sign on the LEFT operand as well remains
    // outside, whatever already-accepted left-unary production exists
    // elsewhere (#773/#774, #775/#776).
    for control in ["+a+-b", "-a-+b", "+a+ +b", "-a- -b"] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    // Other unary operator families never leak into this bounded `+`/`-`
    // theorem.
    for control in ["a+!b", "a+~b", "a+typeof b", "a+void b", "a+delete b"] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    // The bounded family itself remains independently valid; only the
    // recursive/both/other-unary widenings are unowned.
    assert!(
        recognize_selected_identifier_reference_right_unary_additive_initializer("a+-b").is_some()
    );
}

/// Heterogeneous operand and cardinality/richer-expression firewalls
/// (acceptance criteria 13, 14; W20): non-`IdentifierReference` operands and
/// 3+-operand or richer-tail continuations remain outside this exact bounded
/// theorem; no valid bounded prefix authorizes a richer source.
#[test]
fn heterogeneous_operand_and_cardinality_richer_expression_firewalls_remain_outside() {
    for control in ["a+-1", "1+-b", "a+true", "a+null", "a+this", "a+\"b\""] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    for control in [
        "a+-b+c", "a-+b-c", "a+-b*c", "a+-b.c", "a+-b()", "a+-(b)", "a=a+-b", "a?b:c",
    ] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    assert!(
        recognize_selected_identifier_reference_right_unary_additive_initializer("a+-b").is_some()
    );
}

/// Escaped `ReservedWord` and malformed/position-invalid escape firewalls
/// (acceptance criterion 17, W14/W15): neither operand may be a decoded
/// unconditionally reserved word, whatever position within the operand the
/// escape occupies, and a malformed or position-invalid escape on the right
/// operand never leaks a successful left-prefix evidence (whole-candidate
/// transactionality).
#[test]
fn escaped_reserved_word_and_malformed_escape_firewalls_never_leak_partial_evidence() {
    // Escaped ReservedWord: `if` decodes to `if`, an unconditionally
    // reserved word, on either operand position.
    let escaped_reserved_left: &str = concat!("\\", "u0069", "f+-b");
    let escaped_reserved_right: &str = concat!("a+-", "\\", "u0069", "f");
    for control in [escaped_reserved_left, escaped_reserved_right] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0069", "f")),
        Err(DecodeFailure::DecodedReserved)
    );

    // Malformed / position-invalid escape on the right operand must not
    // permit a valid left-prefix fact to leak through: the whole candidate
    // is rejected, never a partial left-only success.
    for control in [
        r"a+-\u{}",
        r"a+-\u{30}",
        r"a+-b\u{2D}",
        r"a+-\uD800",
        r"a+-\u{110000}",
        r"\u{30}+-b",
        r"a\u{2D}+-b",
    ] {
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(control)
                .is_none(),
            "{control:?}"
        );
    }

    assert_eq!(
        decode_selected_escaped_identifier(r"\u{}"),
        Err(DecodeFailure::MalformedEscape)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{30}"),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"b\u{2D}"),
        Err(DecodeFailure::InvalidPart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"\uD800"),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{110000}"),
        Err(DecodeFailure::NonCodePoint)
    );

    // The corresponding fully-valid candidate remains independently
    // accepted; only the reserved/malformed/invalid-position controls above
    // are rejected.
    assert!(
        recognize_selected_identifier_reference_right_unary_additive_initializer("a+-b").is_some()
    );
}

/// Binding/Scope composition (acceptance criterion 18): the right unary
/// wrapper does not change per-occurrence Binding/Scope meaning, and both
/// retained reference facts independently compose existing selected lexical
/// Binding/Scope meaning -- representative `Before`, `Same`, `After`, and
/// `NoSameSourceSelectedLexicalBinding` relations are covered for both the
/// left and right occurrence positions.
#[test]
fn binding_scope_composition_preserves_inner_facts_independently_for_both_occurrences() {
    struct BindingScopeFixture {
        id: &'static str,
        source: &'static str,
        binding_inventory: &'static [&'static str],
        containing_binding_name: &'static str,
        whole: Range,
        left_range: Range,
        right_range: Range,
        expected_left_target: ExpectedBindingScopeTarget,
        expected_right_target: ExpectedBindingScopeTarget,
    }

    let fixtures = [
        // Both `a` and `b` declared before `x`'s initializer: both facts
        // relate `Before` the containing binding.
        BindingScopeFixture {
            id: "BINDSCOPE-BOTH-BEFORE-001",
            source: "let a; let b; const x=a+-b;",
            binding_inventory: &["a", "b", "x"],
            containing_binding_name: "x",
            whole: Range(22, 26),
            left_range: Range(22, 23),
            right_range: Range(25, 26),
            expected_left_target: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Before,
            ),
            expected_right_target: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Before,
            ),
        },
        // Left targets an earlier binding; right has no same-source
        // selected lexical binding at all.
        BindingScopeFixture {
            id: "BINDSCOPE-LEFT-BEFORE-RIGHT-NOTARGET-001",
            source: "let a; const x=a+-z;",
            binding_inventory: &["a", "x"],
            containing_binding_name: "x",
            whole: Range(15, 19),
            left_range: Range(15, 16),
            right_range: Range(18, 19),
            expected_left_target: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Before,
            ),
            expected_right_target: ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
        },
        // Self-referencing initializer: the left occurrence's target is the
        // containing binding itself (`Same`); the right occurrence has no
        // same-source target.
        BindingScopeFixture {
            id: "BINDSCOPE-LEFT-SAME-001",
            source: "let a=a+-b;",
            binding_inventory: &["a"],
            containing_binding_name: "a",
            whole: Range(6, 10),
            left_range: Range(6, 7),
            right_range: Range(9, 10),
            expected_left_target: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Same,
            ),
            expected_right_target: ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
        },
        // Left targets an earlier binding; right targets a binding declared
        // after `x`'s initializer.
        BindingScopeFixture {
            id: "BINDSCOPE-LEFT-BEFORE-RIGHT-AFTER-001",
            source: "let a; const x=a+-b; let b;",
            binding_inventory: &["a", "x", "b"],
            containing_binding_name: "x",
            whole: Range(15, 19),
            left_range: Range(15, 16),
            right_range: Range(18, 19),
            expected_left_target: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Before,
            ),
            expected_right_target: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::After,
            ),
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        let whole_text = slice(fixture.source, fixture.whole);
        let facts =
            recognize_selected_identifier_reference_right_unary_additive_initializer(whole_text)
                .unwrap_or_else(|| {
                    panic!("{}: {whole_text:?} must recognize the theorem", fixture.id)
                });

        let left_semantic = slice(fixture.source, fixture.left_range);
        let right_semantic = slice(fixture.source, fixture.right_range);
        assert_eq!(facts.left.semantic_name, left_semantic, "{}", fixture.id);
        assert_eq!(facts.right.semantic_name, right_semantic, "{}", fixture.id);
        // The right fact's authored anchor never includes the unary sign.
        assert!(!facts.right.authored.starts_with('+') && !facts.right.authored.starts_with('-'));

        let computed_left = expected_binding_scope_target(
            fixture.binding_inventory,
            fixture.containing_binding_name,
            left_semantic,
        );
        let computed_right = expected_binding_scope_target(
            fixture.binding_inventory,
            fixture.containing_binding_name,
            right_semantic,
        );
        assert_eq!(
            computed_left, fixture.expected_left_target,
            "{}",
            fixture.id
        );
        assert_eq!(
            computed_right, fixture.expected_right_target,
            "{}",
            fixture.id
        );

        assert_eq!(
            authored_anchor(777_300 + index as u64, fixture.source, fixture.left_range),
            left_semantic,
            "{}",
            fixture.id
        );
        assert_eq!(
            authored_anchor(777_350 + index as u64, fixture.source, fixture.right_range),
            right_semantic,
            "{}",
            fixture.id
        );
    }

    // No runtime unresolved-identifier vocabulary appears (W14 adjacent):
    // no-target composes as absence of a same-source selected lexical
    // binding, never as a runtime exception claim.
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("Reference", "Error")));
}

/// Static-semantics composition (acceptance criterion 19): the binary and
/// unary operator spellings and the two RHS reference names never become
/// declaration names or static rejection keys; existing duplicate/
/// collision and missing-initializer authority remains driven only by
/// authored `BindingIdentifier` bindings, reusing existing accepted gold
/// authority where the same fixture family already exists.
#[test]
fn static_semantics_composition_remains_driven_by_authored_bindings() {
    struct StaticCompositionFixture {
        source: &'static str,
        subject: Range,
        subject_fragment: &'static str,
        control_gold_id: &'static str,
    }

    const FIXTURES: &[StaticCompositionFixture] = &[
        StaticCompositionFixture {
            source: "let let=a+-b;",
            subject: Range(4, 7),
            subject_fragment: "let",
            control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
        },
        StaticCompositionFixture {
            source: "let x=a+-b, x=foo;",
            subject: Range(12, 13),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        },
        StaticCompositionFixture {
            source: "const x=a+-b, y;",
            subject: Range(14, 15),
            subject_fragment: "y",
            control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        },
        StaticCompositionFixture {
            source: "let x=a+-b; let x=foo;",
            subject: Range(16, 17),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        },
    ];

    for fixture in FIXTURES {
        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment
        );
        assert!(
            gold_source(fixture.control_gold_id).is_some(),
            "{}",
            fixture.control_gold_id
        );
    }

    // The RHS reference names never become an additional `BoundName`: `a`
    // and `b` are each declared exactly once even though they also appear
    // as this theorem's reference facts inside `x`'s initializer.
    let rhs_not_new_bound_names = "let a; let b; const x=a+-b;";
    let binding_inventory = ["a", "b", "x"];
    assert_eq!(binding_inventory.len(), 3);
    assert_eq!(
        expected_binding_scope_target(&binding_inventory, "x", "a"),
        ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
            ExpectedLexicalBindingOrder::Before
        )
    );
    assert_eq!(
        expected_binding_scope_target(&binding_inventory, "x", "b"),
        ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
            ExpectedLexicalBindingOrder::Before
        )
    );
    assert!(
        recognize_selected_identifier_reference_right_unary_additive_initializer(slice(
            rhs_not_new_bound_names,
            Range(22, 26)
        ))
        .is_some()
    );

    let expected = FrontierOutcome::StaticSemanticsRejected;
    assert!(matches!(expected, FrontierOutcome::StaticSemanticsRejected));
}

/// Whole-source transactionality (acceptance criterion 20): a valid earlier
/// right-unary candidate never publishes final relation evidence when a
/// later source component in the same `LexicalDeclaration+` slice fails,
/// whether the failure is a later declarator's empty initializer in the same
/// statement, a later statement's empty initializer, or a later declarator's
/// malformed `BindingIdentifier`.
#[test]
fn whole_source_transactionality_never_commits_partial_evidence_when_later_source_fails() {
    struct FailedTransactionFixture {
        source: &'static str,
        earlier_whole: Range,
        outcome: FrontierOutcome,
        grammar_subject: Option<Range>,
    }

    let fixtures = [
        FailedTransactionFixture {
            source: "const x=a+-b, y=;",
            earlier_whole: Range(8, 12),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "const x=a+-b;\nlet y = ;",
            earlier_whole: Range(8, 12),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: concat!("const x=a+-b, ", "\\", "u{}", "=1;"),
            earlier_whole: Range(8, 12),
            outcome: FrontierOutcome::SyntaxRejected,
            grammar_subject: Some(Range(14, 18)),
        },
    ];

    for fixture in &fixtures {
        let earlier = slice(fixture.source, fixture.earlier_whole);
        assert!(
            recognize_selected_identifier_reference_right_unary_additive_initializer(earlier)
                .is_some(),
            "{earlier:?}"
        );
        assert_ne!(fixture.outcome, FrontierOutcome::SelectedAcceptedIncomplete);

        if let Some(subject) = fixture.grammar_subject {
            assert_eq!(fixture.outcome, FrontierOutcome::SyntaxRejected);
            assert_eq!(slice(fixture.source, subject), concat!("\\", "u{}"));
        }
    }

    // Composing with an already-owned malformed BindingIdentifier Grammar
    // case, or a later empty initializer, does not retroactively invalidate
    // the earlier accepted right-unary occurrence itself: the failure is a
    // whole-source outcome, never evidence that the earlier occurrence was
    // mis-owned.
    assert!(
        recognize_selected_identifier_reference_right_unary_additive_initializer("a+-b").is_some()
    );
}

/// Resource / failure semantics (W16): the project's `ResourceLimited`/
/// `InternalFailure` lifecycle states remain distinct from
/// `UnsupportedCoverage`, matching the minimal symbolic model already
/// accepted by the predecessor candidate-independent `IdentifierReference`
/// oracles (#241, #746/#747, #752/#753) -- this leaf adds no further
/// resource machinery beyond that already-accepted minimum.
#[test]
fn resource_and_internal_failure_lifecycle_states_remain_distinct_from_unsupported_coverage() {
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
    assert!(THIS_ORACLE_SOURCE.contains("ResourceLimited"));
    assert!(THIS_ORACLE_SOURCE.contains("InternalFailure"));
    assert_ne!(PROCESSING_FAILURES[0], "UnsupportedCoverage");
    assert_ne!(PROCESSING_FAILURES[1], "UnsupportedCoverage");
}

/// Freezes the presence-only, unqualified, validation-only handoff: no
/// generic tokenizer, no generic `Expression`/`UnaryExpression`/
/// `AdditiveExpression` representation, no operator enum, no operator or
/// whole-expression `SourceAnchor`, and no runtime/static vocabulary is
/// introduced (acceptance criteria 21, 22; W17/W18/W20/W21/W22 adjacent).
#[test]
fn handoff_remains_validation_only_with_no_operator_or_runtime_representation() {
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
        concat!("enum Token", "Kind"),
        concat!("struct Token", "Stream"),
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
        concat!("ExpectedQualification", "::Qualified"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // No operator or trivia value is ever retained in the returned facts:
    // the facts type carries exactly the two operand fields and nothing
    // else.
    assert!(THIS_ORACLE_SOURCE.contains("struct IdentifierReferenceRightUnaryAdditiveFacts"));
    assert!(THIS_ORACLE_SOURCE.contains("left: RecognizedOperand"));
    assert!(THIS_ORACLE_SOURCE.contains("right: RecognizedOperand"));

    assert!(THIS_ORACLE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(THIS_ORACLE_SOURCE.contains("UnsupportedCoverage"));
    assert!(THIS_ORACLE_SOURCE.contains("StaticSemanticsRejected"));
    assert!(THIS_ORACLE_SOURCE.contains("SyntaxRejected"));

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
