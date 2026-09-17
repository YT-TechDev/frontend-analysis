//! Candidate-independent leading `+`/`-` decimal `UnaryExpression`
//! initializer validation for Issue #742 (durable research: Issue #688
//! comment `5709568879`; accepted operand lineage: Issue #735 / PR #736
//! exponent, Issue #727 / PR #728 fractional, decimal-integer premise).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedLeadingPlusMinusDecimalUnaryExpression ::=
//!     SelectedUnaryPlusMinus
//!     SelectedNumericOperandTrivia
//!     SelectedAcceptedPlainDecimalAtom
//!
//! SelectedUnaryPlusMinus ::= "+" | "-"
//!
//! SelectedAcceptedPlainDecimalAtom ::=
//!     SelectedPlainExponentDecimalLiteral
//!   | SelectedPlainFractionalDecimalLiteral
//!   | SelectedDecimalInteger
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice.
//! It does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! `SelectedAcceptedPlainDecimalAtom` (integer / fractional / exponent) is
//! treated as an already-settled premise: its grammar is independently
//! restated here (not imported from a predecessor oracle file, matching
//! the established candidate-independence convention), never re-derived or
//! re-litigated. The only new semantic question this file freezes is
//! operator ownership: exactly one authored leading `+` or `-`, only
//! already-accepted trivia before the operand, composed with exactly one
//! complete accepted atom, with no richer expression tail.
//!
//! `SelectedNumericOperandTrivia` is restated here only as the
//! ASCII-representable members of the existing selected-slice trivia
//! contract (`is_selected_trivia` in `selected_lexical_slice.rs`): space,
//! tab, line feed, carriage return, vertical tab, and form feed. It
//! deliberately does not restate the full Unicode `White_Space`
//! space-separator superset or the non-ASCII line terminators (BOM,
//! U+2028, U+2029) as a table, matching the same representative-only
//! scope predecessor oracles have always used for trivia composition
//! (e.g. the exponent/fractional oracles' boundary tests use only
//! `' ' | '\t' | '\n' | ','`). It is not a new trivia architecture; it is
//! a representative restatement of the already-accepted contract.
//!
//! No completion successor file accompanies this leaf: unlike the
//! fractional/exponent atom oracles (which each widened a production-
//! accepted initializer RHS family and therefore needed to prove the
//! frozen Early Error reachability partition was unaffected), this Issue
//! adds zero production capability. Production remains completely
//! unaware of leading unary composition (`UnsupportedCoverage` is
//! unchanged), so the accepted selected slice -- and therefore the
//! `193 / 10 / 183 / {}` completion partition -- cannot move. A successor
//! file would only restate that non-event; several precedent initializer
//! oracles that likewise did not touch production's accepted RHS
//! dispatch (`this`, boolean, null, escaped-identifier-reference) also
//! ship without one.
//!
//! `UnsupportedCoverage` expectations are scoped to this frontier. This
//! Issue does not authorize production recognizer changes, so every
//! positive fixture below remains `UnsupportedCoverage` under current
//! production and only exists as independent Oracle evidence for a future,
//! separately authorized production decision.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 742;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_FRACTIONAL_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_plain_fractional_decimal_literal_initializer_validation_tests.rs"
);
const PREVIOUS_EXPONENT_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_plain_exponent_decimal_literal_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_leading_plus_minus_decimal_unary_expression_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "leading plus/minus decimal UnaryExpression frontier only; ",
    "later independently qualified owners may strengthen classification ",
    "for nested/update unary, other unary operators, non-numeric operands, ",
    "numeric separator, BigInt, non-decimal, legacy-leading-zero, ",
    "richer expression, comment, or general ASI forms"
);
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    SelectedAcceptedIncomplete,
    UnsupportedCoverage,
    StaticSemanticsRejected,
    SyntaxRejected,
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

/// Independently restates only the already-accepted premise
/// `SelectedDecimalInteger` grammar.
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

/// Independently restates only the already-accepted premise
/// `SelectedPlainFractionalDecimalLiteral` grammar.
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

fn is_selected_plain_exponent_part(candidate: &str) -> bool {
    let digits = match candidate.as_bytes().first() {
        Some(b'+') | Some(b'-') => &candidate[1..],
        _ => candidate,
    };
    is_decimal_digits(digits)
}

/// Independently restates only the already-accepted premise
/// `SelectedPlainExponentDecimalLiteral` grammar. The exponent-internal
/// `+`/`-` handled inside `is_selected_plain_exponent_part` is a different
/// grammar layer from the outer unary operator this file owns -- see
/// `two_layer_sign_witness_distinguishes_outer_unary_from_exponent_internal_sign`.
fn is_selected_plain_exponent_decimal_literal(candidate: &str) -> bool {
    let Some((mantissa, exponent_tail)) = split_at_first_exponent_marker(candidate) else {
        return false;
    };
    if !is_selected_plain_exponent_part(exponent_tail) {
        return false;
    }
    is_selected_decimal_integer(mantissa) || is_selected_plain_fractional_decimal_literal(mantissa)
}

/// `SelectedAcceptedPlainDecimalAtom ::= SelectedPlainExponentDecimalLiteral
/// | SelectedPlainFractionalDecimalLiteral | SelectedDecimalInteger`. Whole
/// atom, never a strict prefix: each alternative's own whole-string check
/// already forbids trailing leftover content.
fn is_selected_accepted_plain_decimal_atom(candidate: &str) -> bool {
    is_selected_plain_exponent_decimal_literal(candidate)
        || is_selected_plain_fractional_decimal_literal(candidate)
        || is_selected_decimal_integer(candidate)
}

/// Restates only the ASCII-representable members of the existing selected
/// trivia contract (`is_selected_trivia`): space, tab, LF, CR, VT, FF. See
/// the module doc comment for why this stops short of the full Unicode
/// contract.
fn is_selected_numeric_operand_trivia_byte(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0B | 0x0C)
}

/// Central Issue #742 theorem: exactly one leading authored `+`/`-`,
/// followed by zero or more already-accepted trivia bytes, followed by
/// exactly one complete accepted plain decimal atom occupying the rest of
/// the candidate. Whole-string: no richer tail, no nested operator, no
/// non-numeric or unowned-numeric operand survives.
fn is_selected_leading_plus_minus_decimal_unary_expression(candidate: &str) -> bool {
    let mut bytes = candidate.bytes();
    let Some(operator) = bytes.next() else {
        return false;
    };
    if !matches!(operator, b'+' | b'-') {
        return false;
    }
    let after_operator = &candidate[1..];
    let trivia_len = after_operator
        .bytes()
        .take_while(|byte| is_selected_numeric_operand_trivia_byte(*byte))
        .count();
    let operand = &after_operator[trivia_len..];
    is_selected_accepted_plain_decimal_atom(operand)
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 742);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_FRACTIONAL_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 727"));
    assert!(
        PREVIOUS_FRACTIONAL_ORACLE_SOURCE.contains("plain fractional DecimalLiteral frontier only")
    );
    assert!(PREVIOUS_EXPONENT_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 735"));
    assert!(
        PREVIOUS_EXPONENT_ORACLE_SOURCE.contains("plain exponent DecimalLiteral frontier only")
    );
    assert!(
        FRONTIER_SCOPE_NOTE.contains("leading plus/minus decimal UnaryExpression frontier only")
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
        concat!("recognize_selected_", "lexical_slice"),
        concat!("consume_selected_", "decimal_integer"),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_selected_block_var_", "statement"),
        concat!("parse_unary_", "expression"),
        concat!("f64::", "from_str"),
        concat!("parse::", "<f64>"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }
}

/// Required positive matrix: both operators over every accepted decimal
/// operand family, with fixture-owned literal byte ranges for operator,
/// operand, and the complete unary occurrence -- never derived by search,
/// rescan, or reparse.
#[test]
fn exact_positive_matrix_pins_fixture_owned_operator_and_operand_ranges() {
    const POSITIVE_MATRIX: &[(&str, Range, Range, Range, &str, &str)] = &[
        (
            "const x = +1;",
            Range(10, 11),
            Range(11, 12),
            Range(10, 12),
            "+",
            "1",
        ),
        (
            "const x = -1;",
            Range(10, 11),
            Range(11, 12),
            Range(10, 12),
            "-",
            "1",
        ),
        (
            "const x = +1.;",
            Range(10, 11),
            Range(11, 13),
            Range(10, 13),
            "+",
            "1.",
        ),
        (
            "const x = -1.0;",
            Range(10, 11),
            Range(11, 14),
            Range(10, 14),
            "-",
            "1.0",
        ),
        (
            "const x = +.5;",
            Range(10, 11),
            Range(11, 13),
            Range(10, 13),
            "+",
            ".5",
        ),
        (
            "const x = -.5;",
            Range(10, 11),
            Range(11, 13),
            Range(10, 13),
            "-",
            ".5",
        ),
        (
            "const x = +1e2;",
            Range(10, 11),
            Range(11, 14),
            Range(10, 14),
            "+",
            "1e2",
        ),
        (
            "const x = -1e2;",
            Range(10, 11),
            Range(11, 14),
            Range(10, 14),
            "-",
            "1e2",
        ),
        (
            "const x = +1e+2;",
            Range(10, 11),
            Range(11, 15),
            Range(10, 15),
            "+",
            "1e+2",
        ),
        (
            "const x = -1e-2;",
            Range(10, 11),
            Range(11, 15),
            Range(10, 15),
            "-",
            "1e-2",
        ),
        (
            "const x = +1.0e2;",
            Range(10, 11),
            Range(11, 16),
            Range(10, 16),
            "+",
            "1.0e2",
        ),
        (
            "const x = -.5E+2;",
            Range(10, 11),
            Range(11, 16),
            Range(10, 16),
            "-",
            ".5E+2",
        ),
    ];

    for (index, (text, operator, operand, whole, expected_operator, expected_operand)) in
        POSITIVE_MATRIX.iter().enumerate()
    {
        assert_eq!(slice(text, *operator), *expected_operator);
        assert_eq!(slice(text, *operand), *expected_operand);
        let whole_text = slice(text, *whole);
        assert!(
            is_selected_leading_plus_minus_decimal_unary_expression(whole_text),
            "{whole_text:?}"
        );
        assert!(is_selected_accepted_plain_decimal_atom(slice(
            text, *operand
        )));
        assert_eq!(
            authored_anchor(742_000 + index as u64, text, *whole),
            whole_text
        );
    }

    let expected = FrontierOutcome::SelectedAcceptedIncomplete;
    assert!(matches!(
        expected,
        FrontierOutcome::SelectedAcceptedIncomplete
    ));
}

/// First validation carrier from Issue #742: `let x = -1;` /
/// `const x = +1e2;`. This is a validation-harness choice, not a claim
/// that the theorem is `LexicalDeclaration`-specific.
#[test]
fn first_validation_carrier_forms_are_recognized() {
    assert!(is_selected_leading_plus_minus_decimal_unary_expression(
        slice("let x = -1;", Range(8, 10))
    ));
    assert!(is_selected_leading_plus_minus_decimal_unary_expression(
        slice("const x = +1e2;", Range(10, 14))
    ));
}

/// Critical two-layer sign witness (`-1e-2`): the outer `-` is the selected
/// unary operator, and the inner `-` is the exponent-internal sign owned
/// entirely by the accepted exponent atom `1e-2`. Also preserves the
/// unsigned predecessor `1e-2` (an atom on its own, never itself a unary
/// occurrence) and exercises the plus analogue `+1e+2`.
#[test]
fn two_layer_sign_witness_distinguishes_outer_unary_from_exponent_internal_sign() {
    let source = "const x = -1e-2;";
    let whole = Range(10, 15);
    let operand = Range(11, 15);
    assert_eq!(slice(source, whole), "-1e-2");
    assert_eq!(slice(source, operand), "1e-2");
    assert!(is_selected_leading_plus_minus_decimal_unary_expression(
        "-1e-2"
    ));
    // The operand is exactly the complete exponent atom `1e-2`, including
    // its own internal sign -- not a truncated `1e` with a stray `-2`.
    assert!(is_selected_plain_exponent_decimal_literal("1e-2"));
    assert_eq!(authored_anchor(742_400, source, whole), "-1e-2");

    // Unsigned predecessor: an accepted atom by itself, never a unary
    // occurrence (no leading operator byte).
    assert!(is_selected_plain_exponent_decimal_literal("1e-2"));
    assert!(!is_selected_leading_plus_minus_decimal_unary_expression(
        "1e-2"
    ));

    // Plus analogue.
    assert!(is_selected_leading_plus_minus_decimal_unary_expression(
        "+1e+2"
    ));
    assert!(is_selected_plain_exponent_decimal_literal("1e+2"));

    // W4 falsification: the exponent-internal sign is never mistaken for
    // (or merged with) the outer unary sign -- `--2` is not `-` applied to
    // exponent-shorthand `-2`; it fails both the outer-operator layer and
    // the operand-atom layer.
    assert!(!is_selected_leading_plus_minus_decimal_unary_expression(
        "--2"
    ));
}

/// Freezes the already-accepted selected trivia policy between operator
/// and operand: representative space/tab/LF forms compose, and comments
/// never do (W9).
#[test]
fn selected_trivia_matrix_between_operator_and_operand() {
    let fixtures: &[(&str, Range, Range, Range, &str)] = &[
        (
            "const x = + 1;",
            Range(10, 11),
            Range(12, 13),
            Range(10, 13),
            "+ 1",
        ),
        (
            "const x = -\t1.0;",
            Range(10, 11),
            Range(12, 15),
            Range(10, 15),
            "-\t1.0",
        ),
        (
            "const x = +\n1e2;",
            Range(10, 11),
            Range(12, 15),
            Range(10, 15),
            "+\n1e2",
        ),
        (
            "const x = -\n.5;",
            Range(10, 11),
            Range(12, 14),
            Range(10, 14),
            "-\n.5",
        ),
    ];

    for (index, (text, operator, operand, whole, expected_whole)) in fixtures.iter().enumerate() {
        let whole_text = slice(text, *whole);
        assert_eq!(whole_text, *expected_whole);
        assert!(matches!(slice(text, *operator), "+" | "-"), "{operator:?}");
        assert!(
            is_selected_leading_plus_minus_decimal_unary_expression(whole_text),
            "{whole_text:?}"
        );
        assert!(is_selected_accepted_plain_decimal_atom(slice(
            text, *operand
        )));
        assert_eq!(
            authored_anchor(742_100 + index as u64, text, *whole),
            whole_text
        );
    }

    // W9: comments are never accepted as operand trivia, whatever the
    // atom that would otherwise follow.
    for comment_control in ["+/*c*/1", "-/*c*/1.0", "+//c\n1"] {
        assert!(
            !is_selected_leading_plus_minus_decimal_unary_expression(comment_control),
            "{comment_control:?}"
        );
    }
}

/// Operator / nested-unary firewall (W5): keeps `++`, `--`, mixed
/// double-operator forms, spaced double-operator forms, and other unary
/// operators (`!`, `~`) outside this bounded theorem.
#[test]
fn operator_and_nested_unary_firewall_remain_unowned() {
    const NESTED_AND_OTHER_UNARY_CONTROLS: &[&str] = &[
        "++1", "--1", "+-1", "-+1", "+ +1", "- -1", "+ -1", "- +1", "!1", "~1",
    ];
    for control in NESTED_AND_OTHER_UNARY_CONTROLS {
        assert!(
            !is_selected_leading_plus_minus_decimal_unary_expression(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Operand firewall (W6): IdentifierReference, parenthesized, String,
/// Boolean, Null, and `this` operands remain outside this theorem.
#[test]
fn operand_firewall_remains_unowned() {
    const NON_NUMERIC_OPERAND_CONTROLS: &[&str] = &[
        "-foo", "+foo", "-(1)", "+(1)", "-\"x\"", "+\"x\"", "+true", "-false", "+null", "-null",
        "+this", "-this",
    ];
    for control in NON_NUMERIC_OPERAND_CONTROLS {
        assert!(
            !is_selected_leading_plus_minus_decimal_unary_expression(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Numeric operand firewall (W7): separator, non-decimal, BigInt, and
/// legacy-leading-zero operands remain outside this theorem.
#[test]
fn numeric_operand_firewall_remains_unowned() {
    const UNOWNED_NUMERIC_OPERAND_CONTROLS: &[&str] = &[
        "-1_0", "+1.0_0", "-1e1_0", "-0x10", "+0b10", "-0o10", "+1n", "-01",
    ];
    for control in UNOWNED_NUMERIC_OPERAND_CONTROLS {
        assert!(
            !is_selected_leading_plus_minus_decimal_unary_expression(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
    assert!(FRONTIER_SCOPE_NOTE.contains("later independently qualified owners"));
}

/// Richer-expression tail and comment firewall (W8/W9): a complete
/// selected unary prefix never authorizes binary, assignment, conditional,
/// call, member, exponentiation, or comment continuation. `-1 ** 2` is not
/// treated as a cue to build exponentiation semantics; it is simply a
/// richer tail this bounded theorem does not own.
#[test]
fn richer_expression_tail_and_comment_controls_preserve_whole_source_transactionality() {
    const RICHER_EXPRESSION_CONTROLS: &[&str] = &[
        "-1 + x",
        "+1 * x",
        "-1 = x",
        "-1 ? x : y",
        "-1()",
        "-1.foo",
        "-1 ** 2",
        "-1/*comment*/",
        "-1 unexpected",
    ];
    for control in RICHER_EXPRESSION_CONTROLS {
        assert!(
            !is_selected_leading_plus_minus_decimal_unary_expression(control),
            "{control:?}"
        );
    }

    // The complete unary prefix itself remains independently valid; only
    // the richer tail is unowned.
    assert!(is_selected_leading_plus_minus_decimal_unary_expression(
        "-1"
    ));

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

#[test]
fn selected_boundaries_and_existing_sibling_initializers_compose() {
    let fixtures = [
        ("let x = -1, y;", Range(8, 10), 10, Some(b',')),
        ("let x, y = +.5;", Range(11, 14), 14, Some(b';')),
        ("let x = 1, y = -2e3;", Range(15, 19), 19, Some(b';')),
        ("let x = true, y = +3E2;", Range(18, 22), 22, Some(b';')),
        ("let x = null, y = -.25e+2;", Range(18, 25), 25, Some(b';')),
        ("let x = this, y = +4e-2;", Range(18, 23), 23, Some(b';')),
        ("let x = \"a\", y = -.25e+2;", Range(17, 24), 24, Some(b';')),
        ("let x = foo, y = +2.5e-3;", Range(17, 24), 24, Some(b';')),
        ("const x = -1e2", Range(10, 14), 14, None),
        ("const x = +.5e2   \n", Range(10, 15), 19, None),
    ];

    for (index, (text, whole, boundary, expected_byte)) in fixtures.iter().enumerate() {
        let whole_text = slice(text, *whole);
        assert!(
            is_selected_leading_plus_minus_decimal_unary_expression(whole_text),
            "{whole_text:?}"
        );
        assert_eq!(
            authored_anchor(742_500 + index as u64, text, *whole),
            whole_text
        );

        let Range(_, end) = *whole;
        let trailing = text.get(end..*boundary).expect("fixture boundary");
        assert!(
            trailing
                .chars()
                .all(|code_point| matches!(code_point, ' ' | '\t' | '\n' | ',')),
            "{trailing:?}"
        );

        match expected_byte {
            Some(byte) => assert_eq!(text.as_bytes().get(*boundary), Some(byte)),
            None => assert_eq!(*boundary, text.len()),
        }
    }

    // Initializer presence remains per binding, and authored order is
    // preserved: an uninitialized sibling and neighboring predecessor
    // atom/identifier siblings are not unary occurrences.
    assert!(!is_selected_leading_plus_minus_decimal_unary_expression(
        "y"
    ));
    assert!(!is_selected_leading_plus_minus_decimal_unary_expression(
        "1"
    ));
    assert!(!is_selected_leading_plus_minus_decimal_unary_expression(
        "foo"
    ));
}

#[test]
fn existing_static_subjects_and_families_remain_reachable() {
    // Built via `concat!` with an explicit Rust backslash escape, rather
    // than a raw string literal, so the fixture carries exactly one
    // authored reverse solidus (matching authored ECMAScript `0` / `if`
    // source).
    let escaped_start_digit_source: &str = concat!("let ", "\\", "u0030", " = -1e2;");
    let escaped_reserved_word_source: &str = concat!("let ", "\\", "u0069f", " = -1e2;");

    let fixtures = [
        (
            escaped_start_digit_source,
            Range(13, 17),
            Range(4, 10),
            concat!("\\", "u0030"),
            "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
        ),
        (
            escaped_reserved_word_source,
            Range(14, 18),
            Range(4, 11),
            concat!("\\", "u0069f"),
            "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
        ),
        (
            "let let = -1e2;",
            Range(10, 14),
            Range(4, 7),
            "let",
            "JS-GOLD-LEXDECL-LET-BINDING-001",
        ),
        (
            "let x = -1e2, x = foo;",
            Range(8, 12),
            Range(14, 15),
            "x",
            "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        ),
        (
            "const x = -1e2, y;",
            Range(10, 14),
            Range(16, 17),
            "y",
            "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        ),
        (
            "let x = -1e2; let x = foo;",
            Range(8, 12),
            Range(18, 19),
            "x",
            "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        ),
    ];

    for (index, (text, whole, subject, fragment, gold)) in fixtures.iter().enumerate() {
        assert!(is_selected_leading_plus_minus_decimal_unary_expression(
            slice(text, *whole)
        ));
        assert_eq!(slice(text, *subject), *fragment);
        assert!(gold_source(gold).is_some(), "{gold}");
        assert_eq!(
            authored_anchor(742_200 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::StaticSemanticsRejected;
    assert!(matches!(expected, FrontierOutcome::StaticSemanticsRejected));
}

#[test]
fn existing_grammar_subjects_and_syntax_family_remain_reachable() {
    let fixtures = [
        (r"const x = -1e2; let \u{};", Range(20, 24), r"\u{}"),
        (r"const x = -1e2; let a\u{};", Range(21, 25), r"\u{}"),
        (r"const x = -1e2; let \u{61", Range(20, 25), r"\u{61"),
    ];

    for (index, (text, subject, fragment)) in fixtures.iter().enumerate() {
        assert_eq!(slice(text, Range(10, 14)), "-1e2");
        assert!(is_selected_leading_plus_minus_decimal_unary_expression(
            "-1e2"
        ));
        assert_eq!(slice(text, *subject), *fragment);
        assert_eq!(
            authored_anchor(742_300 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::SyntaxRejected;
    assert!(matches!(expected, FrontierOutcome::SyntaxRejected));
}

/// Whole declaration / whole source recognition remains transactional: a
/// valid earlier unary initializer never escapes as a committed selected
/// success when a later binding is incomplete, carries an incomplete
/// exponent tail, or triggers already-owned Grammar evidence.
#[test]
fn whole_declaration_transactionality_prevents_partial_prefix_commitment() {
    struct FailedTransactionFixture {
        source: &'static str,
        earlier_unary_rhs: Range,
        outcome: FrontierOutcome,
        grammar_subject: Option<Range>,
    }

    let fixtures = [
        FailedTransactionFixture {
            source: "let a = -1, b = ;",
            earlier_unary_rhs: Range(8, 10),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "let a = +1.0, b = 1e;",
            earlier_unary_rhs: Range(8, 12),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "let a = -1e-2, b =",
            earlier_unary_rhs: Range(8, 13),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: r"let a = -1, \u{}=2;",
            earlier_unary_rhs: Range(8, 10),
            outcome: FrontierOutcome::SyntaxRejected,
            grammar_subject: Some(Range(12, 16)),
        },
    ];

    for fixture in &fixtures {
        let earlier = slice(fixture.source, fixture.earlier_unary_rhs);
        assert!(
            is_selected_leading_plus_minus_decimal_unary_expression(earlier),
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
    assert!(is_selected_leading_plus_minus_decimal_unary_expression(
        "-1"
    ));

    // The second binding's incomplete exponent tail (`1e`) never resolves
    // to an accepted atom, and therefore never to an accepted unary
    // occurrence, on its own either.
    assert!(!is_selected_plain_exponent_decimal_literal("1e"));
    assert!(!is_selected_leading_plus_minus_decimal_unary_expression(
        "+1e"
    ));
}

#[test]
fn correspondence_and_binding_scope_remain_silent_for_unary_rhs() {
    // The selected operand contains no IdentifierReference: the classifier
    // never accepts identifier- or binding-shaped source as the operand,
    // so it never becomes a source-name query, contributor, or
    // Binding/Scope relation.
    for identifier_shaped in ["x", "y", "foo", "ref", "let", "this"] {
        assert!(!is_selected_leading_plus_minus_decimal_unary_expression(
            identifier_shaped
        ));
        let signed = format!("-{identifier_shaped}");
        assert!(
            !is_selected_leading_plus_minus_decimal_unary_expression(&signed),
            "{signed:?}"
        );
    }
    assert!(is_selected_leading_plus_minus_decimal_unary_expression(
        "-1"
    ));
}

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
        concat!("enum Plus", "Minus"),
        concat!("struct Signed", "Flag"),
        concat!("struct Operator", "SourceAnchor"),
        concat!("struct Operand", "SourceAnchor"),
        concat!("struct Unary", "SourceAnchor"),
        concat!("struct Numeric", "Value"),
        concat!("struct DecimalLiteral", "Fact"),
        concat!("f64", "Value"),
        concat!("struct Number", "Value"),
        concat!("RHS", "SourceAnchor"),
        concat!("Exponent", "SourceAnchor"),
        concat!("struct Mantissa", "Value"),
        concat!("struct Exponent", "Value"),
        concat!("enum Exponent", "SignDomain"),
        concat!("struct Digit", "Array"),
        concat!("enum Initializer", "Kind"),
        concat!("enum Numeric", "Literal"),
        concat!("struct Literal", "Node"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
