//! Candidate-independent ordered two-`IdentifierReference` additive
//! initializer fact-preservation validation for Issue #752 (durable
//! research: Issue #688 comment `5726598010`; accepted predecessor
//! authorities: Issue #746 / PR #747 leading `+`/`-` direct
//! `IdentifierReference` `UnaryExpression` single-fact-preservation Oracle,
//! Issue #750 / PR #751 accepted direct/escaped `IdentifierReference`
//! production, Issue #241 candidate-independent escaped `IdentifierReference`
//! authored-source/decoded-name theorem).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedTwoIdentifierReferenceAdditiveInitializer ::=
//!     SelectedAcceptedIdentifierReference
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
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice. It
//! does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! The load-bearing capability question is not merely whether source such as
//! `a + b` can be recognized. It is whether one selected initializer can
//! retain exactly two independently source-backed `IdentifierReference`
//! facts, in exact authored left-to-right order, each with its own exact
//! authored anchor and semantic name -- the first bounded initializer family
//! this project independently proves can carry more than one retained
//! reference fact.
//!
//! `SelectedAdditiveTrivia` independently restates the complete
//! already-accepted selected-slice trivia contract established by Issue
//! #742/#743 and restated again by Issue #746/#747 (the same code-point set
//! the production selected-trivia recognizer accepts): `TAB`, `VT`, `FF`,
//! `BOM`, `LF`, `CR`, `LINE SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen
//! Unicode 17 `Space_Separator` property. Comments remain outside this
//! contract and are never accepted as trivia.
//!
//! `SelectedDirectIdentifierReference` independently restates the
//! already-accepted Issue #237 direct escape-free `IdentifierName`
//! code-point shape (`is_id_start`/`is_id_continue` plus `$`/`_`) minus the
//! unconditionally reserved words, exactly as Issue #746/#747 restates it.
//! `SelectedEscapedNonReservedIdentifierReference` independently restates
//! the already-accepted Issue #241 `UnicodeEscapeSequence` decode/position/
//! decoded-reserved-word theorem. Neither restatement imports the other
//! Oracle's code: each candidate-independent leaf owns its own copy of the
//! already-accepted primitives it consumes, matching the convention already
//! set by Issue #746/#747 relative to Issue #237/#241.
//!
//! The only capability this Oracle adds beyond its predecessors is finding
//! where the first (left) operand ends when a second operand and operator
//! follow it in the same candidate -- every predecessor single-reference
//! Oracle could treat "the rest of the candidate" as the one operand.
//! `left_operand_run_end` performs exactly that bounded, single left-to-right
//! forward pass (consuming one direct code point or one syntactically
//! well-formed `UnicodeEscapeSequence` element at a time, judged for
//! identifier-position validity only afterward, at whole-operand-slice
//! granularity) and nothing else: it is not a general tokenizer, and it
//! never looks ahead, rescans, or reconstructs a boundary from a later
//! search.
//!
//! This is a validation-only leaf: production supports no additive
//! expression family at the #752 baseline, so every positive fixture below
//! remains `UnsupportedCoverage` under current production and exists only as
//! independent Oracle evidence for a future, separately authorized
//! production decision. No completion successor file accompanies this leaf:
//! this Issue adds zero production capability, so the frozen `193 / 10 / 183
//! / {}` completion partition cannot move, matching the precedent set by
//! #742/#743 and #746/#747 (which likewise added zero production capability
//! and shipped without one).

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 752;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_UNARY_IDENTIFIER_REFERENCE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_leading_plus_minus_direct_identifier_reference_unary_expression_initializer_validation_tests.rs"
);
const PREVIOUS_ESCAPED_IDENTIFIER_REFERENCE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "ordered two-IdentifierReference additive initializer frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "three-or-more operand chains, unary operands, non-IdentifierReference ",
    "operands, parenthesized/member/call operands, richer expression tails, ",
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

// --- Direct IdentifierReference policy: independently restated from Issue
// #237/#746, never imported from either Oracle. ---

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
// as Issue #746/#747 already restates it a second time. ---

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

// --- The Issue #752 theorem itself. ---

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
/// `SmallVec`, or any other specific collection type.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TwoIdentifierReferenceAdditiveFacts<'a> {
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
/// point -- the malformed escape never leaks an earlier valid-looking
/// prefix. Position validity of the collected run is judged afterward by
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

/// Central Issue #752 theorem. Whole-candidate, all-or-nothing: the left
/// operand fact is never returned unless the complete five-part bounded
/// family -- left operand, trivia, exactly one authored `+`/`-`, trivia,
/// right operand -- is recognized over the entire candidate, with nothing
/// left over. The operator is recognized only to prove the exact bounded
/// source family; it is never retained in the returned facts.
fn recognize_selected_two_identifier_reference_additive_initializer(
    candidate: &str,
) -> Option<TwoIdentifierReferenceAdditiveFacts<'_>> {
    let left_end = left_operand_run_end(candidate)?;
    let (left_slice, after_left) = candidate.split_at(left_end);
    let left = recognize_accepted_identifier_reference(left_slice)?;

    let after_left_trivia = &after_left[trivia_run_end(after_left)..];
    let mut after_left_trivia_chars = after_left_trivia.chars();
    match after_left_trivia_chars.next() {
        Some('+') | Some('-') => {}
        _ => return None,
    }
    let after_operator = &after_left_trivia[1..];

    let right_slice = &after_operator[trivia_run_end(after_operator)..];
    let right = recognize_accepted_identifier_reference(right_slice)?;

    Some(TwoIdentifierReferenceAdditiveFacts { left, right })
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 752);
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
    assert!(PREVIOUS_ESCAPED_IDENTIFIER_REFERENCE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 241"));
    assert!(
        FRONTIER_SCOPE_NOTE
            .contains("ordered two-IdentifierReference additive initializer frontier only")
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
        // Candidate independence for trivia recognition specifically: only
        // the stable normative Unicode `Space_Separator` property primitive
        // is reused, never the production selected-trivia recognizer.
        concat!("is_selected_", "trivia("),
        concat!("skip_selected_", "trivia("),
        concat!("use super::", "selected_lexical_slice::is_selected_trivia"),
        // No cross-Oracle import: this leaf restates rather than imports.
        concat!(
            "use super::qualification_selected_escaped_identifier_",
            "reference_initializer_validation_tests"
        ),
        concat!(
            "use super::qualification_selected_leading_plus_minus_direct_",
            "identifier_reference_unary_expression_initializer_validation_tests"
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

/// Required positive direct/direct matrix (acceptance criterion 6): both
/// operators over direct ASCII and direct multibyte operands, with
/// fixture-owned literal byte ranges for left operand, operator, and right
/// operand -- never derived by search, rescan, or reparse. Exactly two facts
/// are retained, in authored left-to-right order, each with its own exact
/// authored anchor (W1/W2/W3/W6/W7).
#[test]
fn positive_direct_direct_matrix_pins_two_ordered_facts_with_exact_provenance() {
    const POSITIVE_MATRIX: &[(&str, Range, Range, Range, &str, &str, &str)] = &[
        (
            "const x = a + b;",
            Range(10, 11),
            Range(12, 13),
            Range(14, 15),
            "a",
            "+",
            "b",
        ),
        (
            "const x = a - b;",
            Range(10, 11),
            Range(12, 13),
            Range(14, 15),
            "a",
            "-",
            "b",
        ),
        (
            "const x = foo + bar;",
            Range(10, 13),
            Range(14, 15),
            Range(16, 19),
            "foo",
            "+",
            "bar",
        ),
        (
            "const x = foo - bar;",
            Range(10, 13),
            Range(14, 15),
            Range(16, 19),
            "foo",
            "-",
            "bar",
        ),
        (
            "const x = \u{03C0} + \u{1D49C};",
            Range(10, 12),
            Range(13, 14),
            Range(15, 19),
            "\u{03C0}",
            "+",
            "\u{1D49C}",
        ),
    ];

    for (index, (text, left, operator, right, expected_left, expected_operator, expected_right)) in
        POSITIVE_MATRIX.iter().enumerate()
    {
        assert_eq!(slice(text, *left), *expected_left);
        assert_eq!(slice(text, *operator), *expected_operator);
        assert_eq!(slice(text, *right), *expected_right);
        // The left fact never includes the operator or intervening trivia.
        assert!(left.1 <= operator.0);
        assert!(operator.1 <= right.0);

        let whole = Range(left.0, right.1);
        let whole_text = slice(text, whole);
        let facts = recognize_selected_two_identifier_reference_additive_initializer(whole_text)
            .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(facts.left.authored, *expected_left);
        assert_eq!(facts.left.semantic_name, *expected_left);
        assert_eq!(facts.left.spelling, OperandSpellingKind::Direct);
        assert_eq!(facts.right.authored, *expected_right);
        assert_eq!(facts.right.semantic_name, *expected_right);
        assert_eq!(facts.right.spelling, OperandSpellingKind::Direct);

        assert_eq!(
            authored_anchor(752_000 + index as u64, text, *left),
            *expected_left
        );
        assert_eq!(
            authored_anchor(752_050 + index as u64, text, *right),
            *expected_right
        );
    }
}

/// Required direct/escaped cross-product (acceptance criteria 6): every
/// direct/direct, escaped/direct, direct/escaped, and escaped/escaped
/// combination decodes to its own semantic name while retaining its own
/// distinct exact authored spelling (W5).
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
        CrossProductFixture {
            text: "const x = a + b;",
            left: Range(10, 11),
            right: Range(14, 15),
            expected_left_authored: "a",
            expected_left_semantic: "a",
            expected_left_spelling: OperandSpellingKind::Direct,
            expected_right_authored: "b",
            expected_right_semantic: "b",
            expected_right_spelling: OperandSpellingKind::Direct,
        },
        CrossProductFixture {
            text: r"const x = \u{61} + b;",
            left: Range(10, 16),
            right: Range(19, 20),
            expected_left_authored: r"\u{61}",
            expected_left_semantic: "a",
            expected_left_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_right_authored: "b",
            expected_right_semantic: "b",
            expected_right_spelling: OperandSpellingKind::Direct,
        },
        CrossProductFixture {
            text: r"const x = a + \u{62};",
            left: Range(10, 11),
            right: Range(14, 20),
            expected_left_authored: "a",
            expected_left_semantic: "a",
            expected_left_spelling: OperandSpellingKind::Direct,
            expected_right_authored: r"\u{62}",
            expected_right_semantic: "b",
            expected_right_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        CrossProductFixture {
            text: r"const x = \u{61} - \u{62};",
            left: Range(10, 16),
            right: Range(19, 25),
            expected_left_authored: r"\u{61}",
            expected_left_semantic: "a",
            expected_left_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_right_authored: r"\u{62}",
            expected_right_semantic: "b",
            expected_right_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        CrossProductFixture {
            text: r"const x = f\u{6F}o + bar;",
            left: Range(10, 18),
            right: Range(21, 24),
            expected_left_authored: r"f\u{6F}o",
            expected_left_semantic: "foo",
            expected_left_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_right_authored: "bar",
            expected_right_semantic: "bar",
            expected_right_spelling: OperandSpellingKind::Direct,
        },
        CrossProductFixture {
            text: r"const x = foo - b\u{61}r;",
            left: Range(10, 13),
            right: Range(16, 24),
            expected_left_authored: "foo",
            expected_left_semantic: "foo",
            expected_left_spelling: OperandSpellingKind::Direct,
            expected_right_authored: r"b\u{61}r",
            expected_right_semantic: "bar",
            expected_right_spelling: OperandSpellingKind::EscapedNonReserved,
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
        let facts = recognize_selected_two_identifier_reference_additive_initializer(whole_text)
            .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(facts.left.authored, fixture.expected_left_authored);
        assert_eq!(facts.left.semantic_name, fixture.expected_left_semantic);
        assert_eq!(facts.left.spelling, fixture.expected_left_spelling);
        assert_eq!(facts.right.authored, fixture.expected_right_authored);
        assert_eq!(facts.right.semantic_name, fixture.expected_right_semantic);
        assert_eq!(facts.right.spelling, fixture.expected_right_spelling);

        // Authored spelling never becomes decoded identity (W5): an escaped
        // operand's authored text and semantic name differ exactly when it
        // is escaped.
        if fixture.expected_left_spelling == OperandSpellingKind::EscapedNonReserved {
            assert_ne!(facts.left.authored, facts.left.semantic_name);
        }
        if fixture.expected_right_spelling == OperandSpellingKind::EscapedNonReserved {
            assert_ne!(facts.right.authored, facts.right.semantic_name);
        }

        assert_eq!(
            authored_anchor(752_100 + index as u64, fixture.text, fixture.left),
            fixture.expected_left_authored
        );
        assert_eq!(
            authored_anchor(752_150 + index as u64, fixture.text, fixture.right),
            fixture.expected_right_authored
        );
    }
}

/// Ordering theorem (acceptance criteria 3, 7): authored left-to-right order
/// is load-bearing and is never reordered, and equal semantic names are
/// never deduplicated into one fact (W3/W4/W7).
#[test]
fn authored_order_is_preserved_and_equal_semantic_names_are_not_deduplicated() {
    let reordered_source = "const x = b + a;";
    let reordered_whole = slice(reordered_source, Range(10, 15));
    let reordered_facts =
        recognize_selected_two_identifier_reference_additive_initializer(reordered_whole)
            .expect("b + a must recognize the theorem");
    assert_eq!(reordered_facts.left.authored, "b");
    assert_eq!(reordered_facts.left.semantic_name, "b");
    assert_eq!(reordered_facts.right.authored, "a");
    assert_eq!(reordered_facts.right.semantic_name, "a");

    let duplicate_source = "const x = a + a;";
    let duplicate_whole = slice(duplicate_source, Range(10, 15));
    let duplicate_facts =
        recognize_selected_two_identifier_reference_additive_initializer(duplicate_whole)
            .expect("a + a must recognize the theorem");
    assert_eq!(
        duplicate_facts.left.semantic_name,
        duplicate_facts.right.semantic_name
    );
    // Two independently owned facts remain even though the semantic names
    // are equal: this is provable because both fields are populated from
    // their own independent operand slice, never merged into a single set.
    assert_eq!(duplicate_facts.left.authored, "a");
    assert_eq!(duplicate_facts.right.authored, "a");

    // Canonical-equivalence identity firewall (W5 adjacent): a precomposed
    // direct left operand and its canonically-equivalent combining-mark
    // decomposition right operand remain distinct authored spellings with
    // distinct byte lengths, never normalized toward each other.
    let normalization_source = "const x = \u{00E9} + e\u{0301};";
    let normalization_whole = slice(normalization_source, Range(10, 18));
    let normalization_facts =
        recognize_selected_two_identifier_reference_additive_initializer(normalization_whole)
            .expect("precomposed + decomposed must recognize the theorem");
    assert_eq!(normalization_facts.left.authored, "\u{00E9}");
    assert_eq!(normalization_facts.right.authored, "e\u{0301}");
    assert_ne!(
        normalization_facts.left.authored,
        normalization_facts.right.authored
    );
    assert_ne!(
        normalization_facts.left.authored.len(),
        normalization_facts.right.authored.len()
    );
}

/// Exact selected trivia around the binary operator (acceptance criterion
/// 9): the already-accepted selected trivia set composes on both sides of
/// the operator, a nearby non-selected code point remains outside, and
/// comments never compose as operand trivia (W6).
#[test]
fn selected_additive_trivia_matrix_around_binary_operator() {
    let fixtures: &[(&str, Range, &str)] = &[
        ("const x = a+b;", Range(10, 13), "a+b"),
        ("const x = a + b;", Range(10, 15), "a + b"),
        ("const x = a\t+\tb;", Range(10, 15), "a\t+\tb"),
        (
            "const x = a\u{00A0}+\u{00A0}b;",
            Range(10, 17),
            "a\u{00A0}+\u{00A0}b",
        ),
        (
            "const x = a\u{2028}-\u{2029}b;",
            Range(10, 19),
            "a\u{2028}-\u{2029}b",
        ),
    ];

    for (text, whole, expected_whole) in fixtures {
        let whole_text = slice(text, *whole);
        assert_eq!(whole_text, *expected_whole);
        let facts = recognize_selected_two_identifier_reference_additive_initializer(whole_text)
            .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));
        assert_eq!(facts.left.semantic_name, "a");
        assert_eq!(facts.right.semantic_name, "b");
    }

    // Comments never compose as operand trivia.
    for comment_control in ["a+/*c*/b", "a/*c*/+b", "a+//c\nb"] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(comment_control)
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
        recognize_selected_two_identifier_reference_additive_initializer("a+\u{200B}b").is_none()
    );
}

/// Cardinality firewall (acceptance criterion 10, W8): a single operand and
/// three-or-more-operand additive chains remain outside this exact bounded
/// theorem; a longer chain is never truncated to its first two operands.
#[test]
fn cardinality_firewall_keeps_single_and_longer_chains_outside() {
    for control in ["a", "a + b + c", "a - b - c", "a + b - c", "a - b + c"] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }
}

/// Unary operand firewall (acceptance criterion 11, W9): leading `+`/`-` on
/// either operand keeps the candidate outside this theorem, whatever
/// already-accepted unary-wrapper production exists elsewhere.
#[test]
fn unary_operand_firewall_keeps_signed_operands_outside() {
    for control in ["+a + b", "-a + b", "a + +b", "a + -b", "a - -b", "a - +b"] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }
}

/// Non-`IdentifierReference` operand firewall (acceptance criterion 12):
/// numeric, boolean, null, `this`, and string-literal operands remain
/// outside, whether on the left or right position.
#[test]
fn non_identifier_reference_operand_firewall_keeps_literals_outside() {
    for control in [
        "1 + b",
        "a + 1",
        "true + b",
        "a + false",
        "null + b",
        "a + null",
        "this + b",
        "a + this",
        "\"a\" + b",
        "a + \"b\"",
    ] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }
}

/// Grouping/member/call firewall: a parenthesized, member, or call primary
/// expression on either operand position remains outside this theorem
/// without any dedicated grouping/member/call recognition code.
#[test]
fn grouping_member_call_firewall_keeps_richer_primary_expressions_outside() {
    for control in [
        "(a) + b", "a + (b)", "a.b + c", "a + b.c", "a() + b", "a + b()",
    ] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }
}

/// Operator boundary firewall (acceptance criterion 13, W10): `UpdateExpression`
/// and assignment-operator tokens are never mistaken for the selected binary
/// `+`/`-`, and precedence/richer-expression tails never compose by
/// valid-prefix truncation (acceptance criterion 14, W11).
#[test]
fn operator_boundary_and_richer_expression_firewalls_remain_unowned() {
    for control in ["a++b", "a--b", "a += b", "a -= b"] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    for control in [
        "a + b * c",
        "a * b + c",
        "a ** b + c",
        "a + b ** c",
        "a = b + c",
        "a ? b : c",
        "a || b",
        "a && b",
        "a ?? b",
    ] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    // The bounded family itself remains independently valid; only the
    // richer/precedence tails and mis-tokenized operators are unowned.
    assert!(recognize_selected_two_identifier_reference_additive_initializer("a + b").is_some());
}

/// Escaped `ReservedWord` firewall (acceptance criterion 15): a decoded
/// unconditionally reserved word never becomes a selected operand, whether
/// on the left or right position, and whatever position within the operand
/// the escape occupies.
#[test]
fn escaped_reserved_word_firewall_keeps_decoded_reserved_operands_outside() {
    for control in [r"\u{69}f + a", r"a + \u{69}f"] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    assert_eq!(
        decode_selected_escaped_identifier(r"\u{69}f"),
        Err(DecodeFailure::DecodedReserved)
    );
}

/// Malformed / position-invalid escape firewall (acceptance criterion 16,
/// W12): a malformed or position-invalid escape on either operand never
/// leaks a truncated prefix or a valid sibling fact -- the whole candidate
/// is rejected.
#[test]
fn malformed_and_position_invalid_escape_firewall_never_leaks_a_sibling_fact() {
    for control in [
        r"\u{30} + a",
        r"a + \u{30}",
        r"a\u{2D}b + c",
        r"a + b\u{2D}c",
        r"\uD800 + a",
        r"a + \u{110000}",
        r"\u{} + a",
    ] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    // The malformed/invalid-position operand's own independent decode
    // failure is the exact one this firewall exercises, confirming no
    // graceful truncation ever occurs before the whole candidate is judged.
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{30}"),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"a\u{2D}"),
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
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{}"),
        Err(DecodeFailure::MalformedEscape)
    );
}

/// Transactionality (acceptance criterion 18): the whole five-part bounded
/// candidate must fully recognize before any fact is returned; a trailing
/// operator, a missing left operand, a malformed operand on either side, or
/// an unexpected trailing tail never yields a partial result (W7 adjacent:
/// no fact is ever published ahead of the complete candidate).
#[test]
fn transactionality_never_commits_partial_evidence() {
    for control in [
        "a +",
        "+ b",
        r"a + \u{}",
        r"\u{} + b",
        "a + b unexpected",
        "a + b +",
    ] {
        assert!(
            recognize_selected_two_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    // The complete bounded candidate remains independently valid; only the
    // incomplete/overextended forms above are rejected.
    assert!(recognize_selected_two_identifier_reference_additive_initializer("a + b").is_some());
}

/// Resource / failure semantics (acceptance criterion 17, W13): the
/// project's `ResourceLimited`/`InternalFailure` lifecycle states remain
/// distinct from `UnsupportedCoverage`, matching the minimal symbolic model
/// already accepted by the predecessor candidate-independent
/// `IdentifierReference` oracles (#241, #746/#747) -- this leaf adds no
/// further resource machinery beyond that already-accepted minimum.
#[test]
fn resource_and_internal_failure_lifecycle_states_remain_distinct_from_unsupported_coverage() {
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
    assert!(THIS_ORACLE_SOURCE.contains("ResourceLimited"));
    assert!(THIS_ORACLE_SOURCE.contains("InternalFailure"));
    assert_ne!(PROCESSING_FAILURES[0], "UnsupportedCoverage");
    assert_ne!(PROCESSING_FAILURES[1], "UnsupportedCoverage");
}

/// Freezes the presence-only, unqualified, validation-only handoff: no
/// operator enum, operator/whole-expression `SourceAnchor`, binary-expression
/// node, AST/CST, token tape, or runtime/static vocabulary is introduced
/// (acceptance criteria 19, 20; W14/W15/W16 adjacent).
#[test]
fn handoff_remains_validation_only_with_no_operator_or_runtime_representation() {
    for forbidden in [
        concat!("enum Selected", "AdditiveOperatorKind"),
        concat!("enum Operator", "Kind"),
        concat!("enum Plus", "Minus"),
        concat!("struct Operator", "SourceAnchor"),
        concat!("struct WholeExpression", "SourceAnchor"),
        concat!("struct Binary", "ExpressionNode"),
        concat!("struct Additive", "ExpressionNode"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
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

    // No operator value is ever retained in the returned facts: the facts
    // type carries exactly the two operand fields and nothing else.
    assert!(THIS_ORACLE_SOURCE.contains("struct TwoIdentifierReferenceAdditiveFacts"));
    assert!(THIS_ORACLE_SOURCE.contains("left: RecognizedOperand"));
    assert!(THIS_ORACLE_SOURCE.contains("right: RecognizedOperand"));

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
