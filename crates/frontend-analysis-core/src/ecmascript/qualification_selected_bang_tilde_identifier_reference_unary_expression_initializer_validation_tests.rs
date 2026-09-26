//! Candidate-independent exactly-one `!`/`~` `IdentifierReference`
//! `UnaryExpression` fact-preservation validation for Issue #827 (durable
//! research: Issue #688 comment `5843902966`; accepted predecessor
//! authority: Issue #746 / PR #747 leading `+`/`-` direct `IdentifierReference`
//! `UnaryExpression` Oracle, Issue #241 escaped `IdentifierReference`
//! initializer Oracle, Issue #237 direct escape-free `IdentifierReference`
//! source/name boundary).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedBangTildeIdentifierReferenceUnaryExpression ::=
//!     SelectedBangTilde
//!     SelectedUnaryOperandTrivia
//!     SelectedAcceptedIdentifierReference
//!
//! SelectedBangTilde ::= "!" | "~"
//!
//! SelectedAcceptedIdentifierReference ::=
//!       SelectedDirectIdentifierReference
//!     | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice. It
//! does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! The load-bearing capability question is not merely whether source such as
//! `!a` / `~a` can be recognized. It is whether one exactly-one operator
//! wrapper can own syntax around an already source-backed `IdentifierReference`
//! fact -- Direct or EscapedNonReserved -- without destroying, widening,
//! reconstructing, or normalizing that fact, so that existing selected
//! Binding/Scope meaning still composes from the inner fact unchanged. `!`
//! and `~` are joined in one theorem because, for this source-fact
//! capability, they share the identical load-bearing shape (one
//! single-character operator, existing selected trivia, one inner
//! `IdentifierReference` fact); their distinct runtime algorithms
//! (`GetValue`/`ToBoolean` vs. `GetValue`/`ToNumeric`) are explicitly outside
//! this source-analysis theorem.
//!
//! `SelectedUnaryOperandTrivia` independently restates the complete
//! already-accepted selected-slice trivia contract established by
//! Issue #742/#743 and reused by Issue #746 (the same code-point set
//! `is_selected_trivia` in `selected_lexical_slice.rs` recognizes, restated
//! here rather than imported): `TAB`, `VT`, `FF`, `BOM`, `LF`, `CR`, `LINE
//! SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen Unicode 17
//! `Space_Separator` property. Comments remain outside this contract.
//!
//! `SelectedDirectIdentifierReference` independently restates only the
//! already-accepted Issue #237 direct escape-free `IdentifierName` code-point
//! shape. `SelectedEscapedNonReservedIdentifierReference` independently
//! restates only the fixed-form `\uXXXX` subset of the already-accepted
//! Issue #241 escaped `IdentifierReference` decode/position/ReservedWord
//! boundary that every representative witness in this Issue needs; it is not
//! a re-test of the full Issue #241 theorem (braced `\u{...}` composition
//! remains that Oracle's own concern, never rebuilt here).
//!
//! This is a validation-only leaf: production supports leading `!`/`~` over
//! no operand family at the #827 baseline, so every positive fixture below
//! remains `UnsupportedCoverage` under current production and exists only as
//! independent Oracle evidence for a future, separately authorized production
//! decision. No completion successor file accompanies this leaf: this Issue
//! adds zero production capability, so the frozen `193 / 10 / 183 / {}`
//! completion partition cannot move, matching the precedent set by #746
//! (which likewise added zero production capability and shipped without
//! one).

use std::cmp::Ordering;

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 827;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_UNARY_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_leading_plus_minus_direct_identifier_reference_unary_expression_initializer_validation_tests.rs"
);
const PREVIOUS_ESCAPED_IDENTIFIER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_bang_tilde_identifier_reference_unary_expression_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "exactly-one leading bang/tilde IdentifierReference UnaryExpression frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "recursive/mixed unary, other unary operators, non-IdentifierReference ",
    "operands, parenthesized operands, richer expression tails, comments, or ",
    "top-level/Block var placement"
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentifierReferenceProvenance {
    Direct,
    EscapedNonReserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodeFailure {
    MalformedEscape,
    InvalidStart,
    InvalidPart,
    DecodedReserved,
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
/// an `IdentifierReference`. Never a `BindingIdentifier` policy, an escaped
/// form, or a broader `PrimaryExpression`.
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

/// Decodes exactly one fixed-form `\uXXXX` `UnicodeEscapeSequence` starting
/// at `start`, returning its code point and the byte offset immediately
/// after it. The braced `\u{...}` form remains Issue #241's own concern and
/// is never rebuilt here: every representative witness this Issue needs uses
/// only the fixed form.
fn decode_fixed_escape_at(bytes: &[u8], start: usize) -> Result<(u32, usize), DecodeFailure> {
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return Err(DecodeFailure::MalformedEscape);
    }
    let digits_start = start + 2;
    let digits = bytes
        .get(digits_start..digits_start + 4)
        .ok_or(DecodeFailure::MalformedEscape)?;
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return Err(DecodeFailure::MalformedEscape);
    }
    let mut value = 0_u32;
    for byte in digits {
        value = value * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }
    Ok((value, digits_start + 4))
}

fn is_selected_identifier_start_code_point(code_point: u32) -> bool {
    char::from_u32(code_point).is_some_and(is_direct_identifier_start)
}

fn is_selected_identifier_part_code_point(code_point: u32) -> bool {
    char::from_u32(code_point).is_some_and(is_direct_identifier_part)
}

/// `SelectedEscapedNonReservedIdentifierReference`: independently restates
/// only the fixed-form `\uXXXX` subset of the already-accepted Issue #241
/// escaped `IdentifierReference` decode/position/ReservedWord boundary.
/// Returns the decoded semantic name, never the authored spelling.
fn decode_selected_escaped_identifier(spelling: &str) -> Result<String, DecodeFailure> {
    let bytes = spelling.as_bytes();
    let mut offset = 0_usize;
    let mut element_index = 0_usize;
    let mut code_points: Vec<u32> = Vec::new();

    while offset < bytes.len() {
        let (code_point, end) = if bytes[offset] == b'\\' {
            decode_fixed_escape_at(bytes, offset)?
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

/// `SelectedAcceptedIdentifierReference`: Direct first, EscapedNonReserved
/// otherwise. Returns the decoded semantic name and provenance -- never the
/// operator/trivia-prefixed spelling.
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

fn is_selected_accepted_identifier_reference(operand: &str) -> bool {
    classify_selected_accepted_identifier_reference(operand).is_some()
}

/// Independently restates the complete already-accepted
/// `SelectedUnaryOperandTrivia` theorem established by Issue #742/#743 and
/// reused by Issue #746: the frozen `is_selected_trivia` code-point set plus
/// the frozen Unicode 17 `Space_Separator` property. Not a call into the
/// production selected-trivia recognizer.
fn is_selected_unary_operand_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

/// Central Issue #827 theorem: exactly one leading authored `!`/`~`,
/// followed by zero or more already-accepted trivia code points, followed by
/// exactly one complete Direct or EscapedNonReserved `IdentifierReference`
/// occupying the rest of the candidate. Whole-string: no richer tail, no
/// nested/mixed operator, no non-identifier operand survives. `!=`/`!==`
/// (W10) and recursive/mixed unary (W9) are excluded structurally: `chars()`
/// consumes exactly one leading operator character, so a second `!`/`~` or a
/// leading `=` immediately fails the operand classification below rather
/// than being specially detected.
///
/// The trivia scan advances a UTF-8 byte offset by each accepted code
/// point's own `len_utf8()` as it is examined -- a single forward pass that
/// is this oracle's owned recognition, never a later search, rescan, or
/// reparse over already-classified source.
fn is_selected_bang_tilde_identifier_reference_unary_expression(candidate: &str) -> bool {
    let mut chars = candidate.chars();
    match chars.next() {
        Some('!') | Some('~') => {}
        _ => return false,
    }
    let after_operator = &candidate[1..];
    let mut operand_start = 0usize;
    for code_point in after_operator.chars() {
        if !is_selected_unary_operand_trivia(code_point) {
            break;
        }
        operand_start += code_point.len_utf8();
    }
    let operand = &after_operator[operand_start..];
    is_selected_accepted_identifier_reference(operand)
}

/// Independently restates only the already-accepted premise that a
/// selected `IdentifierReference` composes with existing lexical Binding/
/// Scope meaning by comparing declaration-order positions -- the same
/// conceptual relation the production Binding/Scope analysis in
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
    assert_eq!(ISSUE_ID, 827);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_UNARY_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 746"));
    assert!(
        PREVIOUS_UNARY_ORACLE_SOURCE.contains(
            "leading plus/minus direct IdentifierReference UnaryExpression frontier only"
        )
    );
    assert!(PREVIOUS_ESCAPED_IDENTIFIER_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 241"));
    assert!(FRONTIER_SCOPE_NOTE.contains(
        "exactly-one leading bang/tilde IdentifierReference UnaryExpression frontier only"
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
        concat!("recognize_selected_", "lexical_slice"),
        concat!(
            "consume_selected_leading_plus_minus_decimal_",
            "unary_expression"
        ),
        concat!(
            "consume_selected_leading_plus_minus_direct_",
            "identifier_reference_unary_expression"
        ),
        concat!("consume_selected_", "identifier_reference"),
        concat!("analyze_selected_binding_", "scope("),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_selected_block_var_", "statement"),
        concat!("parse_unary_", "expression"),
        // Candidate independence for trivia recognition specifically: the
        // production selected-trivia *recognizer* is never imported or
        // called, only the stable normative Unicode `Space_Separator`
        // property primitive (`is_space_separator`) is reused.
        concat!("is_selected_", "trivia("),
        concat!("skip_selected_", "trivia("),
        concat!("use super::", "selected_lexical_slice::is_selected_trivia"),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // `is_id_start` / `is_id_continue` / `is_space_separator` are stable,
    // frozen normative Unicode 17 primitives this oracle is independently
    // entitled to consult (also already used by predecessor accepted
    // oracles at the same `super::unicode` path); none of them is the
    // production selected-trivia recognizer or a production expression
    // parser.
    assert!(
        THIS_ORACLE_SOURCE
            .contains("use super::unicode::{is_id_continue, is_id_start, is_space_separator}")
    );
}

/// Required positive matrix: both operators over Direct ASCII, Direct
/// multibyte, and EscapedNonReserved `IdentifierReference` operands, with
/// fixture-owned literal byte ranges for operator, inner reference, and the
/// complete unary occurrence -- never derived by search, rescan, or reparse.
/// The inner reference range never includes the operator byte (W6/W7); the
/// decoded semantic name and Direct/EscapedNonReserved provenance are
/// preserved exactly (W3).
#[test]
fn exact_positive_matrix_pins_fixture_owned_operator_and_reference_ranges() {
    struct PositiveRow {
        source: &'static str,
        operator: Range,
        reference: Range,
        whole: Range,
        expected_operator: &'static str,
        expected_reference: &'static str,
        expected_name: &'static str,
        expected_provenance: IdentifierReferenceProvenance,
    }

    let rows = [
        PositiveRow {
            source: "const x = !a;",
            operator: Range(10, 11),
            reference: Range(11, 12),
            whole: Range(10, 12),
            expected_operator: "!",
            expected_reference: "a",
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::Direct,
        },
        PositiveRow {
            source: "const x = ~a;",
            operator: Range(10, 11),
            reference: Range(11, 12),
            whole: Range(10, 12),
            expected_operator: "~",
            expected_reference: "a",
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::Direct,
        },
        PositiveRow {
            source: "const x = !foo;",
            operator: Range(10, 11),
            reference: Range(11, 14),
            whole: Range(10, 14),
            expected_operator: "!",
            expected_reference: "foo",
            expected_name: "foo",
            expected_provenance: IdentifierReferenceProvenance::Direct,
        },
        PositiveRow {
            source: "const x = ~\u{03C0};",
            operator: Range(10, 11),
            reference: Range(11, 13),
            whole: Range(10, 13),
            expected_operator: "~",
            expected_reference: "\u{03C0}",
            expected_name: "\u{03C0}",
            expected_provenance: IdentifierReferenceProvenance::Direct,
        },
        PositiveRow {
            source: "const x = ~\\u0061;",
            operator: Range(10, 11),
            reference: Range(11, 17),
            whole: Range(10, 17),
            expected_operator: "~",
            expected_reference: "\\u0061",
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
        },
        PositiveRow {
            source: "const x = !f\\u006Fo;",
            operator: Range(10, 11),
            reference: Range(11, 19),
            whole: Range(10, 19),
            expected_operator: "!",
            expected_reference: "f\\u006Fo",
            expected_name: "foo",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
        },
    ];

    for (index, row) in rows.iter().enumerate() {
        assert_eq!(slice(row.source, row.operator), row.expected_operator);
        assert_eq!(slice(row.source, row.reference), row.expected_reference);
        // The inner reference range never includes the operator byte.
        assert_eq!(row.reference.0, row.operator.1);
        assert_eq!(row.whole.0, row.operator.0);
        assert_eq!(row.whole.1, row.reference.1);

        let whole_text = slice(row.source, row.whole);
        assert!(
            is_selected_bang_tilde_identifier_reference_unary_expression(whole_text),
            "{whole_text:?}"
        );

        let reference_text = slice(row.source, row.reference);
        let (name, provenance) = classify_selected_accepted_identifier_reference(reference_text)
            .unwrap_or_else(|| {
                panic!("{reference_text:?} must be an accepted IdentifierReference")
            });
        assert_eq!(name, row.expected_name);
        assert_eq!(provenance, row.expected_provenance);

        assert_eq!(
            authored_anchor(827_000 + index as u64, row.source, row.whole),
            whole_text
        );
        // The inner reference anchor is exactly the (possibly escaped)
        // spelling, never the operator-prefixed spelling (W6), and the
        // decoded semantic name never includes the operator or trivia (W7).
        assert_eq!(
            authored_anchor(827_050 + index as u64, row.source, row.reference),
            row.expected_reference
        );
    }

    let expected = FrontierOutcome::SelectedAcceptedIncomplete;
    assert!(matches!(
        expected,
        FrontierOutcome::SelectedAcceptedIncomplete
    ));
}

/// First validation carrier from Issue #827: `let a; const x = !a;`. This is
/// a validation-harness choice, not a claim that the theorem is
/// `LexicalDeclaration`-specific.
#[test]
fn first_validation_carrier_form_is_recognized() {
    let source = "let a;\nconst x = !a;";
    assert!(
        is_selected_bang_tilde_identifier_reference_unary_expression(slice(source, Range(17, 19)))
    );
}

/// Freezes the complete already-accepted selected trivia policy between
/// operator and operand: ASCII TAB/LF, and the frozen Unicode 17
/// `Space_Separator` property beyond ASCII space (NBSP) all compose;
/// comments never do (W1 adjacent: this pressure is unrelated to runtime
/// evaluation support).
#[test]
fn selected_trivia_matrix_between_operator_and_operand() {
    let fixtures: &[(&str, Range, Range, Range, &str)] = &[
        (
            "const x = ! a;",
            Range(10, 11),
            Range(12, 13),
            Range(10, 13),
            "! a",
        ),
        (
            "const x = ~\ta;",
            Range(10, 11),
            Range(12, 13),
            Range(10, 13),
            "~\ta",
        ),
        (
            "const x = !\na;",
            Range(10, 11),
            Range(12, 13),
            Range(10, 13),
            "!\na",
        ),
        (
            "const x = ~\u{00A0}a;",
            Range(10, 11),
            Range(13, 14),
            Range(10, 14),
            "~\u{00A0}a",
        ),
    ];

    for (index, (text, operator, reference, whole, expected_whole)) in fixtures.iter().enumerate() {
        let whole_text = slice(text, *whole);
        assert_eq!(whole_text, *expected_whole);
        assert!(matches!(slice(text, *operator), "!" | "~"), "{operator:?}");
        assert_eq!(slice(text, *reference), "a");
        assert!(
            is_selected_bang_tilde_identifier_reference_unary_expression(whole_text),
            "{whole_text:?}"
        );
        assert_eq!(
            authored_anchor(827_100 + index as u64, text, *whole),
            whole_text
        );
    }

    // Comments never compose as operand trivia.
    for comment_control in ["!/*c*/a", "~/*c*/a", "!//c\na"] {
        assert!(
            !is_selected_bang_tilde_identifier_reference_unary_expression(comment_control),
            "{comment_control:?}"
        );
    }
}

/// Identifier policy composition (representative categories only, not a
/// re-test of Issue #237/#241): ordinary direct, multibyte direct, `$`/`_`,
/// the `Yield=false`/`Await=false` special forms, the `eval`/`arguments`
/// binding-contrast names, and ordinary/mixed EscapedNonReserved spellings
/// all compose as valid operands with both operators (W2: `!`/`~` share
/// identical operand policy); unconditionally reserved direct spellings,
/// malformed escapes, and decoded-reserved escaped spellings remain outside
/// (W6 adjacent: operator identity plays no role in this policy).
#[test]
fn identifier_reference_policy_composition_matrix() {
    const ACCEPTED_DIRECT_OPERANDS: &[&str] = &[
        "a",
        "foo",
        "\u{03C0}",
        "$",
        "_",
        "yield",
        "await",
        "eval",
        "arguments",
    ];
    for operand in ACCEPTED_DIRECT_OPERANDS {
        assert_eq!(
            classify_selected_accepted_identifier_reference(operand),
            Some(((*operand).to_owned(), IdentifierReferenceProvenance::Direct)),
            "{operand:?}"
        );
        for prefixed in [format!("!{operand}"), format!("~{operand}")] {
            assert!(
                is_selected_bang_tilde_identifier_reference_unary_expression(&prefixed),
                "{prefixed:?}"
            );
        }
    }

    const ACCEPTED_ESCAPED_OPERANDS: &[(&str, &str)] =
        &[("\\u0061", "a"), ("f\\u006Fo", "foo"), ("\\u0024", "$")];
    for (rhs, decoded) in ACCEPTED_ESCAPED_OPERANDS {
        assert_eq!(
            classify_selected_accepted_identifier_reference(rhs),
            Some((
                (*decoded).to_owned(),
                IdentifierReferenceProvenance::EscapedNonReserved
            )),
            "{rhs:?}"
        );
        for prefixed in [format!("!{rhs}"), format!("~{rhs}")] {
            assert!(
                is_selected_bang_tilde_identifier_reference_unary_expression(&prefixed),
                "{prefixed:?}"
            );
        }
    }

    // Unconditionally reserved direct spellings remain outside
    // `IdentifierReference` (and therefore outside this unary theorem).
    for reserved in ["if", "this", "true", "false", "null", "typeof", "delete"] {
        assert_eq!(
            classify_selected_accepted_identifier_reference(reserved),
            None
        );
        for prefixed in [format!("!{reserved}"), format!("~{reserved}")] {
            assert!(
                !is_selected_bang_tilde_identifier_reference_unary_expression(&prefixed),
                "{prefixed:?}"
            );
        }
    }

    // Malformed escapes remain outside (short/invalid hex digits).
    for malformed in [r"\u006", r"\u0G61"] {
        assert_eq!(
            decode_selected_escaped_identifier(malformed),
            Err(DecodeFailure::MalformedEscape),
            "{malformed:?}"
        );
        assert_eq!(
            classify_selected_accepted_identifier_reference(malformed),
            None
        );
        assert!(
            !is_selected_bang_tilde_identifier_reference_unary_expression(&format!("!{malformed}"))
        );
    }

    // A decoded-reserved escaped spelling remains outside (`if`
    // decodes to the reserved word "if").
    assert_eq!(
        decode_selected_escaped_identifier("\\u0069f"),
        Err(DecodeFailure::DecodedReserved)
    );
    assert_eq!(
        classify_selected_accepted_identifier_reference("\\u0069f"),
        None
    );
    assert!(!is_selected_bang_tilde_identifier_reference_unary_expression("!\\u0069f"));

    // Invalid decoded start/part positions remain outside (`0` decodes
    // to a leading digit; `a-` decodes to a trailing hyphen).
    assert_eq!(
        decode_selected_escaped_identifier("\\u0030"),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier("a\\u002D"),
        Err(DecodeFailure::InvalidPart)
    );

    assert_eq!(UNCONDITIONALLY_RESERVED_WORDS.len(), 36);
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"yield"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"await"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"eval"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"arguments"));
}

/// Exactly-one firewall (W9): recursive/mixed unary combinations, the
/// leading `+`/`-` sibling operators, and keyword unary operators (W4) all
/// remain outside this bounded theorem.
#[test]
fn exactly_one_unary_firewall_remains_unowned() {
    const EXACTLY_ONE_FIREWALL_CONTROLS: &[&str] = &[
        "!!a", "~~a", "!~a", "~!a", "+a", "-a", "typeof a", "void a", "delete a",
    ];
    for control in EXACTLY_ONE_FIREWALL_CONTROLS {
        assert!(
            !is_selected_bang_tilde_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Punctuator firewall (W10): a `!=`/`!==` relational/equality punctuator
/// prefix must never be reinterpreted as a unary `!` followed by an operand
/// route. No special-case detection is required: the character immediately
/// after the consumed `!` is `=`, which is neither accepted trivia nor a
/// valid `IdentifierReference` start, so operand classification fails on its
/// own (W11 adjacent).
#[test]
fn punctuator_firewall_prevents_relational_equality_reinterpretation() {
    for control in ["!=a", "!==a"] {
        assert!(
            !is_selected_bang_tilde_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    // The corresponding valid unary prefix remains accepted.
    assert!(is_selected_bang_tilde_identifier_reference_unary_expression("!a"));
}

/// Operand firewall (W5): Decimal/literal/boolean/null/`this`/string/
/// parenthesized operands remain outside this theorem, whatever operator
/// prefixes them.
#[test]
fn operand_firewall_excludes_non_identifier_atoms() {
    const NON_IDENTIFIER_OPERAND_CONTROLS: &[&str] = &[
        "!1", "~1", "!1e2", "~true", "!null", "~this", "!\"x\"", "!(a)",
    ];
    for control in NON_IDENTIFIER_OPERAND_CONTROLS {
        assert!(
            !is_selected_bang_tilde_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Richer-expression tail firewall (W11): a complete unary-reference prefix
/// never authorizes member, call, binary, assignment, or conditional
/// continuation; only the richer tail is unowned, and the bare prefix itself
/// remains independently valid.
#[test]
fn richer_expression_tail_controls_preserve_whole_source_transactionality() {
    const RICHER_EXPRESSION_CONTROLS: &[&str] =
        &["!a.b", "~a()", "!a + b", "~a * b", "!a = b", "!a ? b : c"];
    for control in RICHER_EXPRESSION_CONTROLS {
        assert!(
            !is_selected_bang_tilde_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    assert!(is_selected_bang_tilde_identifier_reference_unary_expression("!a"));
    assert!(is_selected_bang_tilde_identifier_reference_unary_expression("~a"));

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Exact fact-preservation theorem: for representative `Before`, `Same`,
/// `After`, and no-target sources, the inner `IdentifierReference` fact
/// (semantic name and authored anchor) is frozen independently of the
/// operator/trivia wrapper, and the existing selected lexical Binding/Scope
/// relation composes from that inner fact unchanged (W12/W13).
#[test]
fn lexical_binding_scope_before_same_after_and_no_target_relations_are_independently_frozen() {
    struct BindingScopeFixture {
        id: &'static str,
        source: &'static str,
        binding_inventory: &'static [&'static str],
        containing_binding_name: &'static str,
        containing_binding_range: Range,
        operator_range: Range,
        reference_range: Range,
        target_binding_range: Option<Range>,
        expected: ExpectedBindingScopeTarget,
    }

    let fixtures = [
        BindingScopeFixture {
            id: "BINDSCOPE-BEFORE-001",
            source: "let a;\nconst x = !a;",
            binding_inventory: &["a", "x"],
            containing_binding_name: "x",
            containing_binding_range: Range(13, 14),
            operator_range: Range(17, 18),
            reference_range: Range(18, 19),
            target_binding_range: Some(Range(4, 5)),
            expected: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Before,
            ),
        },
        BindingScopeFixture {
            id: "BINDSCOPE-SAME-001",
            source: "let a = !a;",
            binding_inventory: &["a"],
            containing_binding_name: "a",
            containing_binding_range: Range(4, 5),
            operator_range: Range(8, 9),
            reference_range: Range(9, 10),
            target_binding_range: Some(Range(4, 5)),
            expected: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Same,
            ),
        },
        BindingScopeFixture {
            id: "BINDSCOPE-AFTER-001",
            source: "const x = ~a;\nlet a;",
            binding_inventory: &["x", "a"],
            containing_binding_name: "x",
            containing_binding_range: Range(6, 7),
            operator_range: Range(10, 11),
            reference_range: Range(11, 12),
            target_binding_range: Some(Range(18, 19)),
            expected: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::After,
            ),
        },
        BindingScopeFixture {
            id: "BINDSCOPE-NOTARGET-001",
            source: "const x = !z;",
            binding_inventory: &["x"],
            containing_binding_name: "x",
            containing_binding_range: Range(6, 7),
            operator_range: Range(10, 11),
            reference_range: Range(11, 12),
            target_binding_range: None,
            expected: ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        let whole = Range(fixture.operator_range.0, fixture.reference_range.1);
        let whole_text = slice(fixture.source, whole);
        assert!(
            is_selected_bang_tilde_identifier_reference_unary_expression(whole_text),
            "{}: {whole_text:?}",
            fixture.id
        );

        assert_eq!(
            slice(fixture.source, fixture.containing_binding_range),
            fixture.containing_binding_name,
            "{}",
            fixture.id
        );
        let reference_semantic_name = slice(fixture.source, fixture.reference_range);
        assert!(
            is_selected_accepted_identifier_reference(reference_semantic_name),
            "{}",
            fixture.id
        );
        // The inner reference anchor never includes the operator (W6/W7).
        assert_eq!(fixture.reference_range.0, fixture.operator_range.1);

        let computed = expected_binding_scope_target(
            fixture.binding_inventory,
            fixture.containing_binding_name,
            reference_semantic_name,
        );
        assert_eq!(computed, fixture.expected, "{}", fixture.id);

        match (fixture.target_binding_range, computed) {
            (
                Some(target_range),
                ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(_),
            ) => {
                assert_eq!(
                    slice(fixture.source, target_range),
                    reference_semantic_name,
                    "{}",
                    fixture.id
                );
                assert_eq!(
                    authored_anchor(827_200 + index as u64, fixture.source, target_range),
                    reference_semantic_name,
                    "{}",
                    fixture.id
                );
            }
            (None, ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding) => {}
            _ => panic!("{}: target range and expected target disagree", fixture.id),
        }

        assert_eq!(
            authored_anchor(
                827_250 + index as u64,
                fixture.source,
                fixture.containing_binding_range
            ),
            fixture.containing_binding_name,
            "{}",
            fixture.id
        );
        assert_eq!(
            authored_anchor(
                827_300 + index as u64,
                fixture.source,
                fixture.reference_range
            ),
            reference_semantic_name,
            "{}",
            fixture.id
        );
    }

    // `Same` is proven with a clean, already-valid selected source (a
    // self-referencing lexical initializer) rather than an invented
    // fixture: `let a = !a;` needs no grammar/static widening because a
    // direct `IdentifierReference` RHS is already an accepted source shape.
    assert_eq!(
        expected_binding_scope_target(&["a"], "a", "a"),
        ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
            ExpectedLexicalBindingOrder::Same
        )
    );

    // No-target composes as absence of a same-source selected lexical
    // binding, never as a runtime unresolved-identifier exception claim
    // (W13 adjacent): this oracle asserts no such runtime vocabulary
    // appears.
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("Reference", "Error")));
}

/// Static-semantics composition (W13 adjacent): the unary operator and the
/// inner RHS semantic name never become declaration names or static
/// rejection keys. Existing duplicate/collision and missing-initializer
/// authority remains driven only by authored `BindingIdentifier` bindings.
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
            source: "let let = !a;",
            subject: Range(4, 7),
            subject_fragment: "let",
            control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
        },
        StaticCompositionFixture {
            source: "let x = !a, x = foo;",
            subject: Range(12, 13),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        },
        StaticCompositionFixture {
            source: "const x = !a, y;",
            subject: Range(14, 15),
            subject_fragment: "y",
            control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        },
        StaticCompositionFixture {
            source: "let x = !a; let x = foo;",
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

    // Initializer presence: `const x = !a;` alone has an initializer
    // present, so it never triggers the missing-const-initializer rejection
    // that the bare `y` sibling above does.
    let with_initializer = "const x = !a;";
    assert!(
        is_selected_bang_tilde_identifier_reference_unary_expression(slice(
            with_initializer,
            Range(10, 12)
        ))
    );

    // The RHS reference name never becomes an additional `BoundName`: `a`
    // is declared exactly once (`let a;`) even though it also appears as
    // the unary operand's reference in `x`'s initializer.
    let rhs_not_a_new_bound_name = "let a;\nconst x = !a;";
    let binding_inventory = ["a", "x"];
    assert_eq!(binding_inventory.len(), 2);
    assert_eq!(
        expected_binding_scope_target(&binding_inventory, "x", "a"),
        ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
            ExpectedLexicalBindingOrder::Before
        )
    );
    assert!(
        is_selected_bang_tilde_identifier_reference_unary_expression(slice(
            rhs_not_a_new_bound_name,
            Range(17, 19)
        ))
    );

    let expected = FrontierOutcome::StaticSemanticsRejected;
    assert!(matches!(expected, FrontierOutcome::StaticSemanticsRejected));
}

/// Whole-source transactionality (W12 adjacent): a valid earlier
/// unary-reference initializer never escapes as a committed selected
/// success when a later binding in the same source is incomplete or
/// triggers an already-owned malformed `BindingIdentifier` Grammar failure.
#[test]
fn whole_source_transactionality_prevents_partial_prefix_commitment() {
    struct FailedTransactionFixture {
        source: &'static str,
        earlier_unary_rhs: Range,
        outcome: FrontierOutcome,
        grammar_subject: Option<Range>,
    }

    let fixtures = [
        FailedTransactionFixture {
            source: "let a = !a, b = ;",
            earlier_unary_rhs: Range(8, 10),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "const x=!a, y=",
            earlier_unary_rhs: Range(8, 10),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "let a; const x=!a, y=;",
            earlier_unary_rhs: Range(15, 17),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: r"let a = !a, \u{}=2;",
            earlier_unary_rhs: Range(8, 10),
            outcome: FrontierOutcome::SyntaxRejected,
            grammar_subject: Some(Range(12, 16)),
        },
    ];

    for fixture in &fixtures {
        let earlier = slice(fixture.source, fixture.earlier_unary_rhs);
        assert!(
            is_selected_bang_tilde_identifier_reference_unary_expression(earlier),
            "{earlier:?}"
        );
        assert_ne!(fixture.outcome, FrontierOutcome::SelectedAcceptedIncomplete);

        if let Some(subject) = fixture.grammar_subject {
            assert_eq!(fixture.outcome, FrontierOutcome::SyntaxRejected);
            assert_eq!(slice(fixture.source, subject), r"\u{}");
        }
    }

    // Composing with an already-owned malformed BindingIdentifier Grammar
    // case does not retroactively invalidate the earlier accepted unary
    // occurrence itself -- the failure is a whole-source outcome, not
    // evidence that the earlier occurrence was mis-owned.
    assert!(is_selected_bang_tilde_identifier_reference_unary_expression("!a"));
}

/// Freezes the presence-only, unqualified, validation-only handoff: this
/// leaf composes to `SelectedAcceptedIncomplete`, `UnsupportedCoverage`,
/// `StaticSemanticsRejected`, and `SyntaxRejected` remain distinct, and no
/// retained production representation, runtime, or general-parser
/// vocabulary is introduced (W1/W6/W7/W8 adjacent).
#[test]
fn handoff_remains_presence_only_unqualified_and_validation_only() {
    assert!(THIS_ORACLE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(THIS_ORACLE_SOURCE.contains("UnsupportedCoverage"));
    assert!(THIS_ORACLE_SOURCE.contains("StaticSemanticsRejected"));
    assert!(THIS_ORACLE_SOURCE.contains("SyntaxRejected"));
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);

    for forbidden in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("enum Selected", "Literal"),
        concat!("enum SelectedPrimary", "Expression"),
        concat!("enum SelectedExpression", "Kind"),
        concat!("enum Selected", "UnaryExpressionKind"),
        concat!("struct Selected", "UnaryExpressionNode"),
        concat!("enum Operator", "Kind"),
        concat!("enum Bang", "Tilde"),
        concat!("struct Signed", "Flag"),
        concat!("struct Operator", "SourceAnchor"),
        concat!("struct Unary", "SourceAnchor"),
        concat!("struct Boolean", "Value"),
        concat!("struct Number", "Value"),
        concat!("struct Reference", "Record"),
        concat!("struct Environment", "Record"),
        concat!("enum Initializer", "Kind"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("Get", "Value("),
        concat!("Resolve", "Binding("),
        concat!("To", "Boolean("),
        concat!("To", "Numeric("),
        concat!("To", "Number("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
