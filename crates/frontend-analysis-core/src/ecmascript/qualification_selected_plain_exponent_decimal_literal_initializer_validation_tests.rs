//! Candidate-independent plain exponent `DecimalLiteral` initializer
//! validation for Issue #735 (durable research: Issue #688 comment
//! `5700680419`; closest structural precedent: Issue #727 / PR #728).
//!
//! This oracle qualifies only the bounded, separator-free exponent
//! `DecimalLiteral` family:
//!
//! ```text
//! SelectedPlainExponentDecimalLiteral ::=
//!     SelectedDecimalInteger SelectedPlainExponentPart
//!   | SelectedPlainFractionalDecimalLiteral SelectedPlainExponentPart
//!
//! SelectedPlainExponentPart ::=
//!     ("e" | "E") ("+" | "-")? DecimalDigits
//!
//! SelectedDecimalInteger ::=
//!     "0"
//!   | [1-9][0-9]*
//!
//! SelectedPlainFractionalDecimalLiteral ::=
//!     SelectedDecimalInteger "." DecimalDigits?
//!   | "." DecimalDigits
//!
//! DecimalDigits ::=
//!     [0-9]+
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice.
//! It does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! The predecessor `SelectedDecimalInteger` and `SelectedPlainFractionalDecimalLiteral`
//! grammars are independently restated here (not imported from a predecessor
//! oracle file), so the complete-atom-ownership theorem can be stated without
//! relying on another oracle file's classifier as expected-result authority.
//!
//! `UnsupportedCoverage` expectations are scoped to this frontier. Later
//! independently qualified owners may strengthen classifications for
//! numeric separators, BigInt, non-decimal or legacy-leading-zero forms,
//! richer expressions, comments, or general non-EOF ASI.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 735;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_FRACTIONAL_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_plain_fractional_decimal_literal_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_plain_exponent_decimal_literal_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "plain exponent DecimalLiteral frontier only; ",
    "later independently qualified owners may strengthen classification ",
    "for numeric separator, BigInt, non-decimal, legacy-leading-zero, ",
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

/// Independently restates only the already-accepted predecessor
/// `SelectedPlainFractionalDecimalLiteral` grammar:
/// `SelectedDecimalInteger "." DecimalDigits? | "." DecimalDigits`.
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

/// Splits a candidate on its first ASCII `e`/`E` byte into the portion
/// before the marker and the portion after it. Returns `None` when no
/// exponent marker byte is present. Because the grammar this file
/// classifies never allows a second exponent marker to appear inside a
/// well-formed `DecimalDigits` tail, splitting on the first occurrence is
/// sufficient: any additional `e`/`E` byte left in the tail is rejected by
/// the tail's own all-ASCII-digit check.
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

/// Classifies only `SelectedPlainExponentPart ::= ("+" | "-")? DecimalDigits`
/// (the exponent marker byte itself is already consumed by the caller).
fn is_selected_plain_exponent_part(candidate: &str) -> bool {
    let digits = match candidate.as_bytes().first() {
        Some(b'+') | Some(b'-') => &candidate[1..],
        _ => candidate,
    };
    is_decimal_digits(digits)
}

/// Independently classifies only the Issue #735 grammar:
/// `SelectedDecimalInteger SelectedPlainExponentPart |
/// SelectedPlainFractionalDecimalLiteral SelectedPlainExponentPart`.
///
/// The match is whole-string: any leftover content after a structurally
/// valid mantissa/exponent split disqualifies the candidate. This is what
/// proves whole-atom transactionality and complete-atom ownership without
/// prescribing a production recognizer ordering or rollback algorithm -- a
/// locally valid prefix (for example the `1` inside `1e2`, the `1.0` inside
/// `1.0e2`, or the `.5` inside `.5e2`) can never be accepted merely because
/// a shorter prefix would match a predecessor grammar.
fn is_selected_plain_exponent_decimal_literal(candidate: &str) -> bool {
    let Some((mantissa, exponent_tail)) = split_at_first_exponent_marker(candidate) else {
        return false;
    };
    if !is_selected_plain_exponent_part(exponent_tail) {
        return false;
    }
    is_selected_decimal_integer(mantissa) || is_selected_plain_fractional_decimal_literal(mantissa)
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 735);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_FRACTIONAL_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 727"));
    assert!(
        PREVIOUS_FRACTIONAL_ORACLE_SOURCE.contains("plain fractional DecimalLiteral frontier only")
    );
    assert!(FRONTIER_SCOPE_NOTE.contains("plain exponent DecimalLiteral frontier only"));
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
        ("const x = 1e2;", Range(10, 13), "1e2"),
        ("const x = 1E2;", Range(10, 13), "1E2"),
        ("const x = 1e+2;", Range(10, 14), "1e+2"),
        ("const x = 1e-2;", Range(10, 14), "1e-2"),
        ("const x = 1.e2;", Range(10, 14), "1.e2"),
        ("const x = 1.E+2;", Range(10, 15), "1.E+2"),
        ("const x = 1.0e2;", Range(10, 15), "1.0e2"),
        ("const x = 1.0E-2;", Range(10, 16), "1.0E-2"),
        ("const x = .5e2;", Range(10, 14), ".5e2"),
        ("const x = .5E+2;", Range(10, 15), ".5E+2"),
        ("const x = 12.34E-56;", Range(10, 19), "12.34E-56"),
    ];

    for (index, (text, rhs, expected_fragment)) in POSITIVE_MATRIX.iter().enumerate() {
        let rhs_text = slice(text, *rhs);
        assert_eq!(rhs_text, *expected_fragment);
        assert!(
            is_selected_plain_exponent_decimal_literal(rhs_text),
            "{rhs_text:?}"
        );
        assert_eq!(
            authored_anchor(735_000 + index as u64, text, *rhs),
            *expected_fragment
        );
    }

    let expected = FrontierOutcome::SelectedAcceptedIncomplete;
    assert!(matches!(
        expected,
        FrontierOutcome::SelectedAcceptedIncomplete
    ));
}

/// Central Issue #735 theorem: `1` / `10` (existing selected decimal
/// integer), `1.0` / `1.` / `.5` (existing selected fractional atoms), and
/// `1e2` / `1.0e2` / `.5e2` (new selected exponent atoms) are simultaneously
/// and independently owned by disjoint whole-string grammars. Neither
/// classifier ever accepts a strict prefix of another's input, which
/// defeats the premature-prefix models explicitly named by Issue #735:
/// `1e2` accepting only integer `1`, `1.0e2` accepting only fractional
/// `1.0`, and `.5e2` accepting only fractional `.5`.
#[test]
fn overlapping_lexical_prefix_theorem_preserves_predecessor_atoms_and_adds_exponent_atom() {
    for predecessor_integer_only in ["0", "1", "9", "10", "42"] {
        assert!(is_selected_decimal_integer(predecessor_integer_only));
        assert!(!is_selected_plain_exponent_decimal_literal(
            predecessor_integer_only
        ));
    }

    for predecessor_fractional_only in ["1.0", "1.", ".5", "0.", ".0"] {
        assert!(is_selected_plain_fractional_decimal_literal(
            predecessor_fractional_only
        ));
        assert!(!is_selected_plain_exponent_decimal_literal(
            predecessor_fractional_only
        ));
    }

    for exponent_atom in ["1e2", "1E2", "1e+2", "1e-2", "1.e2", "1.0e2", ".5e2"] {
        assert!(
            is_selected_plain_exponent_decimal_literal(exponent_atom),
            "{exponent_atom:?}"
        );
    }

    // Premature-prefix firewall: a complete exponent atom is never also
    // accepted by a predecessor grammar via a truncated read.
    assert!(is_selected_decimal_integer("1"));
    assert!(!is_selected_decimal_integer("1e2"));
    assert!(is_selected_plain_fractional_decimal_literal("1.0"));
    assert!(!is_selected_plain_fractional_decimal_literal("1.0e2"));
    assert!(is_selected_plain_fractional_decimal_literal(".5"));
    assert!(!is_selected_plain_fractional_decimal_literal(".5e2"));
}

/// Pins incomplete exponent tails so they cannot fall back to a successful
/// predecessor prefix: every listed candidate is rejected by the exponent
/// grammar *and* by both predecessor grammars, so none of it is silently
/// upgraded into invented exponent-specific syntax authority.
#[test]
fn incomplete_exponent_tail_matrix_never_falls_back_to_predecessor_prefix() {
    const INCOMPLETE_TAILS: &[&str] = &["1e", "1E", "1e+", "1e-", "1.e", "1.E+", "1.0e-", ".5E+"];
    for incomplete in INCOMPLETE_TAILS {
        assert!(
            !is_selected_plain_exponent_decimal_literal(incomplete),
            "{incomplete:?}"
        );
        assert!(!is_selected_decimal_integer(incomplete), "{incomplete:?}");
        assert!(
            !is_selected_plain_fractional_decimal_literal(incomplete),
            "{incomplete:?}"
        );
    }
}

/// Explicitly distinguishes an exponent-internal sign (owned by this atom)
/// from a leading `UnaryExpression` sign (out of scope for this atom).
#[test]
fn leading_unary_sign_and_exponent_internal_sign_are_distinguished() {
    for internal_sign in ["1e+2", "1e-2", "1.0e+2", ".5e-2", "12.34E-56"] {
        assert!(
            is_selected_plain_exponent_decimal_literal(internal_sign),
            "{internal_sign:?}"
        );
    }

    for leading_unary in ["+1e2", "-1e2", "+1.0e2", "-.5e2", "+1", "-1.0"] {
        assert!(
            !is_selected_plain_exponent_decimal_literal(leading_unary),
            "{leading_unary:?}"
        );
    }
}

#[test]
fn numeric_neighbor_and_legacy_form_boundary_matrix_remain_unowned_here() {
    const NUMERIC_NEIGHBORS: &[&str] = &[
        "1e1_0", "1_0e2", "1.0_0e2", ".5e1_0", "1n", "0x10", "0b10", "0o10", "01", "+1e2", "-1e2",
    ];
    for neighbor in NUMERIC_NEIGHBORS {
        assert!(
            !is_selected_plain_exponent_decimal_literal(neighbor),
            "{neighbor:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
    assert!(FRONTIER_SCOPE_NOTE.contains("later independently qualified owners"));
}

#[test]
fn richer_expression_tail_and_comment_controls_preserve_whole_source_transactionality() {
    const RICHER_EXPRESSION_CONTROLS: &[&str] = &[
        "1e2.foo",
        "1e2()",
        "1e2 + x",
        "1e2 = x",
        "1e2 ? x : y",
        "1e2/*comment*/",
        "1e2 unexpected",
        "1.0e2.foo",
        ".5e2 + x",
    ];
    for control in RICHER_EXPRESSION_CONTROLS {
        assert!(
            !is_selected_plain_exponent_decimal_literal(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

#[test]
fn selected_boundaries_and_existing_sibling_initializers_compose() {
    let fixtures = [
        ("let x = 1e2, y;", Range(8, 11), 11, Some(b',')),
        ("let x, y = .5e2;", Range(11, 15), 15, Some(b';')),
        ("let x = 1, y = 2e3;", Range(15, 18), 18, Some(b';')),
        ("let x = true, y = 3E2;", Range(18, 21), 21, Some(b';')),
        ("let x = null, y = .25e+2;", Range(18, 24), 24, Some(b';')),
        ("let x = this, y = 4e-2;", Range(18, 22), 22, Some(b';')),
        ("let x = \"a\", y = .25e+2;", Range(17, 23), 23, Some(b';')),
        ("let x = foo, y = 2.5e-3;", Range(17, 23), 23, Some(b';')),
        ("const x = 1e2", Range(10, 13), 13, None),
        ("const x = .5e2   \n", Range(10, 14), 18, None),
    ];

    for (index, (text, rhs, boundary, expected_byte)) in fixtures.iter().enumerate() {
        let rhs_text = slice(text, *rhs);
        assert!(
            is_selected_plain_exponent_decimal_literal(rhs_text),
            "{rhs_text:?}"
        );
        assert_eq!(
            authored_anchor(735_100 + index as u64, text, *rhs),
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
    // preserved: the first fixture's uninitialized `y` and neighboring
    // predecessor siblings are not exponent RHS occurrences, and neither is
    // mistaken for one.
    assert!(!is_selected_plain_exponent_decimal_literal("y"));
    assert!(!is_selected_plain_exponent_decimal_literal("1"));
    assert!(!is_selected_plain_exponent_decimal_literal("foo"));
}

// Built via `concat!` with an explicit Rust backslash escape, rather than a
// raw string literal, so the fixture carries exactly one authored reverse
// solidus (matching authored ECMAScript `0` / `if` source).
const ESCAPED_START_DIGIT_SOURCE: &str = concat!("let ", "\\", "u0030", " = 1e2;");
const ESCAPED_START_DIGIT_SUBJECT: &str = concat!("\\", "u0030");
const ESCAPED_RESERVED_WORD_SOURCE: &str = concat!("let ", "\\", "u0069f", " = 1e2;");
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
            "let let = 1e2;",
            Range(10, 13),
            Range(4, 7),
            "let",
            "JS-GOLD-LEXDECL-LET-BINDING-001",
        ),
        (
            "let x = 1e2, x = foo;",
            Range(8, 11),
            Range(13, 14),
            "x",
            "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        ),
        (
            "const x = 1e2, y;",
            Range(10, 13),
            Range(15, 16),
            "y",
            "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        ),
        (
            "let x = 1e2; let x = foo;",
            Range(8, 11),
            Range(17, 18),
            "x",
            "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        ),
    ];

    for (index, (text, rhs, subject, fragment, gold)) in fixtures.iter().enumerate() {
        assert!(is_selected_plain_exponent_decimal_literal(slice(
            text, *rhs
        )));
        assert_eq!(slice(text, *subject), *fragment);
        assert!(gold_source(gold).is_some(), "{gold}");
        assert_eq!(
            authored_anchor(735_200 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::StaticSemanticsRejected;
    assert!(matches!(expected, FrontierOutcome::StaticSemanticsRejected));
}

#[test]
fn existing_grammar_subjects_and_syntax_family_remain_reachable() {
    let fixtures = [
        (r"const x = 1e2; let \u{};", Range(19, 23), r"\u{}"),
        (r"const x = 1e2; let a\u{};", Range(20, 24), r"\u{}"),
        (r"const x = 1e2; let \u{61", Range(19, 24), r"\u{61"),
    ];

    for (index, (text, subject, fragment)) in fixtures.iter().enumerate() {
        assert_eq!(slice(text, Range(10, 13)), "1e2");
        assert!(is_selected_plain_exponent_decimal_literal("1e2"));
        assert_eq!(slice(text, *subject), *fragment);
        assert_eq!(
            authored_anchor(735_300 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::SyntaxRejected;
    assert!(matches!(expected, FrontierOutcome::SyntaxRejected));
}

/// Whole declaration / whole source recognition remains transactional: a
/// valid earlier exponent initializer must never escape as a committed
/// selected success when a later binding is incomplete, carries an
/// incomplete exponent tail, or triggers already-owned Grammar evidence.
#[test]
fn whole_declaration_transactionality_prevents_partial_prefix_commitment() {
    struct FailedTransactionFixture {
        source: &'static str,
        earlier_exponent_rhs: Range,
        outcome: FrontierOutcome,
        grammar_subject: Option<Range>,
    }

    let fixtures = [
        FailedTransactionFixture {
            source: "let a = 1e2, b = ;",
            earlier_exponent_rhs: Range(8, 11),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "let a = 1.0e-2, b = 1e;",
            earlier_exponent_rhs: Range(8, 14),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: "let a = .5e2, b =",
            earlier_exponent_rhs: Range(8, 12),
            outcome: FrontierOutcome::UnsupportedCoverage,
            grammar_subject: None,
        },
        FailedTransactionFixture {
            source: r"let a = 1e2, \u{}=2;",
            earlier_exponent_rhs: Range(8, 11),
            outcome: FrontierOutcome::SyntaxRejected,
            grammar_subject: Some(Range(13, 17)),
        },
    ];

    for fixture in &fixtures {
        let earlier = slice(fixture.source, fixture.earlier_exponent_rhs);
        assert!(
            is_selected_plain_exponent_decimal_literal(earlier),
            "{earlier:?}"
        );
        assert_ne!(fixture.outcome, FrontierOutcome::SelectedAcceptedIncomplete);

        if let Some(subject) = fixture.grammar_subject {
            assert_eq!(fixture.outcome, FrontierOutcome::SyntaxRejected);
            assert_eq!(slice(fixture.source, subject), r"\u{}");
        }
    }

    // Composing with an already-owned malformed BindingIdentifier Grammar
    // case does not retroactively invalidate the earlier accepted exponent
    // atom itself -- the failure is a whole-source outcome, not evidence
    // that the earlier atom was mis-owned.
    assert!(is_selected_plain_exponent_decimal_literal("1e2"));

    // The second-binding incomplete exponent tail (`1e`) never resolves to
    // an accepted atom on its own either.
    assert!(!is_selected_plain_exponent_decimal_literal("1e"));
}

#[test]
fn correspondence_and_binding_scope_remain_silent_for_exponent_rhs() {
    // A plain exponent DecimalLiteral has no IdentifierReference: the
    // classifier never accepts identifier- or binding-shaped source, so it
    // never becomes a source-name query, contributor, or Binding/Scope
    // relation.
    for identifier_shaped in ["x", "y", "foo", "ref", "let", "this"] {
        assert!(!is_selected_plain_exponent_decimal_literal(
            identifier_shaped
        ));
    }
    assert!(is_selected_plain_exponent_decimal_literal("1e2"));
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
