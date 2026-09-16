//! Candidate-independent plain fractional `DecimalLiteral` initializer
//! validation for Issue #727 (durable research: Issue #688 comment
//! `5691915319`).
//!
//! This oracle qualifies only the bounded, separator-free, exponent-free
//! fractional `DecimalLiteral` family:
//!
//! ```text
//! SelectedPlainFractionalDecimalLiteral ::=
//!     SelectedDecimalInteger "." DecimalDigits?
//!   | "." DecimalDigits
//!
//! SelectedDecimalInteger ::=
//!     "0"
//!   | [1-9][0-9]*
//!
//! DecimalDigits ::=
//!     [0-9]+
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice.
//! It does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! `UnsupportedCoverage` expectations are scoped to this frontier. Later
//! independently qualified owners may strengthen classifications for
//! exponents, numeric separators, BigInt, non-decimal or legacy-leading-zero
//! forms, richer expressions, comments, or general non-EOF ASI.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 727;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_STRING_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escape_free_string_literal_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_plain_fractional_decimal_literal_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "plain fractional DecimalLiteral frontier only; ",
    "later independently qualified owners may strengthen classification ",
    "for exponent, numeric separator, BigInt, non-decimal, legacy-leading-zero, ",
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

/// Independently restates only the already-accepted predecessor
/// `SelectedDecimalInteger` grammar, so the overlapping-prefix theorem can be
/// stated without importing anything from a predecessor oracle file.
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

/// Independently classifies only the Issue #727 grammar:
/// `SelectedDecimalInteger "." DecimalDigits? | "." DecimalDigits`.
///
/// The match is whole-string: any leftover content after a structurally
/// valid integer/fraction split disqualifies the candidate. This is what
/// proves whole-atom transactionality without prescribing a production
/// recognizer ordering or rollback algorithm — a locally valid prefix (for
/// example the `1.0` inside `1.0.foo`) can never be accepted merely because
/// a shorter prefix would match.
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

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 727);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_STRING_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 257"));
    assert!(PREVIOUS_STRING_ORACLE_SOURCE.contains("escape-free StringLiteral frontier only"));
    assert!(FRONTIER_SCOPE_NOTE.contains("plain fractional DecimalLiteral frontier only"));
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
        concat!("f64::", "from_str"),
        concat!("parse::", "<f64>"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn exact_positive_matrix_pins_fixture_owned_rhs_ranges() {
    const POSITIVE_MATRIX: &[(&str, Range, &str)] = &[
        ("const x = 0.;", Range(10, 12), "0."),
        ("const x = 1.;", Range(10, 12), "1."),
        ("const x = 1.0;", Range(10, 13), "1.0"),
        ("const x = 12.34;", Range(10, 15), "12.34"),
        ("const x = .0;", Range(10, 12), ".0"),
        ("const x = .5;", Range(10, 12), ".5"),
        (
            "const x = 123456789.987654321;",
            Range(10, 29),
            "123456789.987654321",
        ),
        ("const x = 0.0;", Range(10, 13), "0.0"),
        ("const x = 999.;", Range(10, 14), "999."),
        ("const x = .0001;", Range(10, 15), ".0001"),
    ];

    for (index, (text, rhs, expected_fragment)) in POSITIVE_MATRIX.iter().enumerate() {
        let rhs_text = slice(text, *rhs);
        assert_eq!(rhs_text, *expected_fragment);
        assert!(
            is_selected_plain_fractional_decimal_literal(rhs_text),
            "{rhs_text:?}"
        );
        assert_eq!(
            authored_anchor(727_000 + index as u64, text, *rhs),
            *expected_fragment
        );
    }

    let expected = FrontierOutcome::SelectedAcceptedIncomplete;
    assert!(matches!(
        expected,
        FrontierOutcome::SelectedAcceptedIncomplete
    ));
}

/// Central Issue #727 theorem: `1` (existing selected decimal integer) and
/// `1.0` / `1.` / `.5` (new selected fractional atoms) are simultaneously
/// and independently owned by disjoint whole-string grammars. Neither
/// classifier ever accepts a strict prefix of the other's input, which
/// defeats the "accept `1.0` by consuming only `1`" wrong model without
/// requiring either classifier to know about the other.
#[test]
fn overlapping_lexical_prefix_theorem_preserves_predecessor_integer_and_adds_longer_atom() {
    for predecessor_only in ["0", "1", "9", "10", "42"] {
        assert!(is_selected_decimal_integer(predecessor_only));
        assert!(!is_selected_plain_fractional_decimal_literal(
            predecessor_only
        ));
    }

    for new_atom_only in ["1.0", "1.", ".5", "0.", ".0"] {
        assert!(!is_selected_decimal_integer(new_atom_only));
        assert!(is_selected_plain_fractional_decimal_literal(new_atom_only));
    }

    // W3: "." alone is neither an integer nor a fractional literal.
    assert!(!is_selected_decimal_integer("."));
    assert!(!is_selected_plain_fractional_decimal_literal("."));

    // W4 / W5: a missing integer prefix or a missing post-dot digit does not
    // disqualify the *other* alternative of the same production.
    assert!(is_selected_plain_fractional_decimal_literal(".5"));
    assert!(is_selected_plain_fractional_decimal_literal("1."));
}

#[test]
fn numeric_neighbor_and_dot_prefix_boundary_matrix_remain_unowned_here() {
    const NUMERIC_NEIGHBORS: &[&str] = &[
        "1e2", "1.0e2", ".5e2", "1_0", "1.0_0", ".5_0", "1n", "0x10", "0X10", "0b10", "0B10",
        "0o10", "0O10", "01", "+1.0", "-1.0",
    ];
    for neighbor in NUMERIC_NEIGHBORS {
        assert!(
            !is_selected_plain_fractional_decimal_literal(neighbor),
            "{neighbor:?}"
        );
    }

    const DOT_BOUNDARY_CONTROLS: &[&str] =
        &[".", "..", "1..foo", "1.0.foo", ".5.foo", "1.0()", ".5()"];
    for control in DOT_BOUNDARY_CONTROLS {
        assert!(
            !is_selected_plain_fractional_decimal_literal(control),
            "{control:?}"
        );
    }

    // A complete local atom (`1.`) does not authorize a richer tail: the
    // whole occupied source `1..foo` is rejected as a unit, not partially
    // accepted as `1.` followed by unowned trailing source.
    assert!(is_selected_plain_fractional_decimal_literal("1."));
    assert!(!is_selected_plain_fractional_decimal_literal("1..foo"));

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
    assert!(FRONTIER_SCOPE_NOTE.contains("later independently qualified owners"));
}

#[test]
fn richer_expression_tail_and_comment_controls_preserve_whole_source_transactionality() {
    const RICHER_EXPRESSION_CONTROLS: &[&str] = &[
        "1.0.foo",
        "1.0()",
        "1.0 + x",
        "1.0 = x",
        "1.0 ? x : y",
        ".5 + x",
        "1.0/*comment*/",
        "1.0 unexpected",
    ];
    for control in RICHER_EXPRESSION_CONTROLS {
        assert!(
            !is_selected_plain_fractional_decimal_literal(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

#[test]
fn selected_boundaries_and_existing_sibling_initializers_compose() {
    let fixtures = [
        ("let x = 1.0, y;", Range(8, 11), 11, Some(b',')),
        ("let x, y = .5;", Range(11, 13), 13, Some(b';')),
        ("let x = 1, y = 2.5;", Range(15, 18), 18, Some(b';')),
        ("let x = true, y = 3.0;", Range(18, 21), 21, Some(b';')),
        ("let x = null, y = .25;", Range(18, 21), 21, Some(b';')),
        ("let x = this, y = 4.0;", Range(18, 21), 21, Some(b';')),
        ("let x = \"a\", y = .25;", Range(17, 20), 20, Some(b';')),
        ("let x = foo, y = 2.0;", Range(17, 20), 20, Some(b';')),
        ("const x = 1.0", Range(10, 13), 13, None),
        ("const x = .5   \n", Range(10, 12), 16, None),
    ];

    for (index, (text, rhs, boundary, expected_byte)) in fixtures.iter().enumerate() {
        let rhs_text = slice(text, *rhs);
        assert!(
            is_selected_plain_fractional_decimal_literal(rhs_text),
            "{rhs_text:?}"
        );
        assert_eq!(
            authored_anchor(727_100 + index as u64, text, *rhs),
            rhs_text
        );

        let Range(_, end) = *rhs;
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
    // preserved: the first fixture's uninitialized `y` and the third
    // fixture's leading decimal-integer sibling are not fractional RHS
    // occurrences, and neither is mistaken for one.
    assert!(!is_selected_plain_fractional_decimal_literal("y"));
    assert!(!is_selected_plain_fractional_decimal_literal("1"));
    assert!(!is_selected_plain_fractional_decimal_literal("foo"));
}

// Built via `concat!` with an explicit Rust backslash escape, rather than a
// raw string literal, so the fixture carries exactly one authored reverse
// solidus (matching authored ECMAScript `0` / `if` source).
const ESCAPED_START_DIGIT_SOURCE: &str = concat!("let ", "\\", "u0030", " = 1.5;");
const ESCAPED_START_DIGIT_SUBJECT: &str = concat!("\\", "u0030");
const ESCAPED_RESERVED_WORD_SOURCE: &str = concat!("let ", "\\", "u0069f", " = 1.5;");
const ESCAPED_RESERVED_WORD_SUBJECT: &str = concat!("\\", "u0069f");

#[test]
fn existing_static_subjects_and_families_remain_reachable() {
    let fixtures = [
        (
            ESCAPED_START_DIGIT_SOURCE,
            Range(13, 16),
            Range(4, 10),
            ESCAPED_START_DIGIT_SUBJECT,
            "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
        ),
        (
            ESCAPED_RESERVED_WORD_SOURCE,
            Range(14, 17),
            Range(4, 11),
            ESCAPED_RESERVED_WORD_SUBJECT,
            "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
        ),
        (
            "let let = 1.5;",
            Range(10, 13),
            Range(4, 7),
            "let",
            "JS-GOLD-LEXDECL-LET-BINDING-001",
        ),
        (
            "let x = 1.5, x = foo;",
            Range(8, 11),
            Range(13, 14),
            "x",
            "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        ),
        (
            "const x = 1.5, y;",
            Range(10, 13),
            Range(15, 16),
            "y",
            "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        ),
        (
            "let x = 1.5; let x = foo;",
            Range(8, 11),
            Range(17, 18),
            "x",
            "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        ),
    ];

    for (index, (text, rhs, subject, fragment, gold)) in fixtures.iter().enumerate() {
        assert!(is_selected_plain_fractional_decimal_literal(slice(
            text, *rhs
        )));
        assert_eq!(slice(text, *subject), *fragment);
        assert!(gold_source(gold).is_some(), "{gold}");
        assert_eq!(
            authored_anchor(727_200 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::StaticSemanticsRejected;
    assert!(matches!(expected, FrontierOutcome::StaticSemanticsRejected));
}

#[test]
fn existing_grammar_subjects_and_syntax_family_remain_reachable() {
    let fixtures = [
        (r"const x = 1.5; let \u{};", Range(19, 23), r"\u{}"),
        (r"const x = 1.5; let a\u{};", Range(20, 24), r"\u{}"),
        (r"const x = 1.5; let \u{61", Range(19, 24), r"\u{61"),
    ];

    for (index, (text, subject, fragment)) in fixtures.iter().enumerate() {
        assert_eq!(slice(text, Range(10, 13)), "1.5");
        assert!(is_selected_plain_fractional_decimal_literal("1.5"));
        assert_eq!(slice(text, *subject), *fragment);
        assert_eq!(
            authored_anchor(727_300 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::SyntaxRejected;
    assert!(matches!(expected, FrontierOutcome::SyntaxRejected));
}

/// Whole declaration / whole source recognition remains transactional: a
/// valid earlier fractional initializer must never escape as a committed
/// selected success when a later binding is incomplete, carries an unowned
/// numeric neighbor, or triggers already-owned Grammar evidence.
#[test]
fn whole_declaration_transactionality_prevents_partial_prefix_commitment() {
    struct FailedTransactionFixture {
        source: &'static str,
        earlier_fractional_rhs: Range,
        outcome: FrontierOutcome,
        grammar_subject: Option<Range>,
    }

    let fixtures = [
        FailedTransactionFixture {
            source: "let a = 1.0, b = ;",
            earlier_fractional_rhs: Range(8, 11),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "let a = .5, b = 1e2;",
            earlier_fractional_rhs: Range(8, 10),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "let a = 1.0, b =",
            earlier_fractional_rhs: Range(8, 11),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: r"let a = 1.0, \u{}=2;",
            earlier_fractional_rhs: Range(8, 11),
            outcome: FrontierOutcome::SyntaxRejected,
            grammar_subject: Some(Range(13, 17)),
        },
    ];

    for fixture in &fixtures {
        let earlier = slice(fixture.source, fixture.earlier_fractional_rhs);
        assert!(
            is_selected_plain_fractional_decimal_literal(earlier),
            "{earlier:?}"
        );
        assert_ne!(fixture.outcome, FrontierOutcome::SelectedAcceptedIncomplete);

        if let Some(subject) = fixture.grammar_subject {
            assert_eq!(fixture.outcome, FrontierOutcome::SyntaxRejected);
            assert_eq!(slice(fixture.source, subject), r"\u{}");
        }
    }
}

#[test]
fn correspondence_and_binding_scope_remain_silent_for_fractional_rhs() {
    // A plain fractional DecimalLiteral has no IdentifierReference: the
    // classifier never accepts identifier- or binding-shaped source, so it
    // never becomes a source-name query, contributor, or Binding/Scope
    // relation. Existing sibling IdentifierReference behavior (owned by
    // Issue #237's independent authority) is unaffected and unreferenced.
    for identifier_shaped in ["x", "y", "foo", "ref", "let", "this"] {
        assert!(!is_selected_plain_fractional_decimal_literal(
            identifier_shaped
        ));
    }
    assert!(is_selected_plain_fractional_decimal_literal("1.0"));
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
        concat!("struct Numeric", "Value"),
        concat!("struct DecimalLiteral", "Fact"),
        concat!("f64", "Value"),
        concat!("RHS", "SourceAnchor"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
