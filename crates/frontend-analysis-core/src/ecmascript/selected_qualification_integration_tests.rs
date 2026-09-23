use crate::{SourceId, SourceText};

use super::qualification::{ProcessingStatus, QualificationVerdictKind, RejectionFamily};
use super::qualification_validation_tests::{gold_source, gold_subject_range};
use super::selected_qualification_integration::{
    SelectedQualificationAttempt, attempt_selected_qualification,
    selected_grammar_rejection_to_qualification,
};

fn attempt(text: &str) -> SelectedQualificationAttempt {
    let source = SourceText::new(SourceId::new(218), text.to_owned());
    attempt_selected_qualification(&source)
}

fn escaped_first_ascii_code_point(name: &str) -> String {
    assert!(name.is_ascii());
    let mut chars = name.chars();
    let first = chars
        .next()
        .expect("reserved-word control must be non-empty");
    format!(r"\u{:04X}{}", first as u32, chars.as_str())
}

#[test]
fn selected_positive_gold_remains_explicitly_incomplete_not_qualified() {
    for fixture_id in [
        "JS-GOLD-SCRIPT-VALID-001",
        "JS-GOLD-SCRIPT-MULTIBYTE-001",
        "JS-GOLD-LEXDECL-CONST-VALID-001",
        "JS-GOLD-LEXDECL-MULTIBIND-VALID-001",
        "JS-GOLD-LEXDECL-CONST-MULTIBIND-VALID-001",
        "JS-GOLD-LEXDECL-MULTIBIND-CANONICAL-DISTINCT-001",
        "JS-GOLD-LEXDECL-EE04-AWAIT-YIELD-001",
        "JS-GOLD-LEXDECL-EE04-FUTURE-RESERVED-001",
        "JS-GOLD-LEXDECL-EE04-EVAL-ARGUMENTS-001",
    ] {
        let text = gold_source(fixture_id).unwrap_or_else(|| panic!("{fixture_id} must exist"));
        assert!(matches!(
            attempt(text),
            SelectedQualificationAttempt::SelectedAcceptedIncomplete
        ));
    }
}

#[test]
fn eof_asi_positive_sources_remain_selected_accepted_incomplete() {
    for text in [
        "let x = 1",
        "const x = 1",
        "let x",
        "let x, y",
        "const x = 1, y = 2",
        "let x; const y = 1",
        r"let \u0061",
        "let x = 1 \t\n\r\u{2028}\u{2029}\u{00A0}\u{1680}\u{2000}\u{202F}\u{205F}\u{3000}\u{FEFF}",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn identifier_reference_initializers_remain_selected_accepted_incomplete() {
    for text in [
        gold_source("JS-GOLD-LEXDECL-CONST-IDENTIFIER-INIT-001").expect("identifier init gold"),
        "const x = foo",
        "let x = foo;",
        "let x = foo",
        "let x = foo, y = bar;",
        "let x = 1, y = foo;",
        "let x = foo, y = 1;",
        "const x = foo, y = bar;",
        "let x = 1; const y = foo",
        "let x = foo; const y = bar",
        "const π = 𝒜;",
        r"const \u0078 = foo;",
        "const x = $;",
        "const x = _;",
        "const x = a0;",
        "const x = a\u{0301};",
        "const x = a\u{200C}b;",
        "const x = a\u{200D}b;",
        "const x = 𝒜;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }

    for name in [
        "let",
        "static",
        "implements",
        "interface",
        "package",
        "private",
        "protected",
        "public",
        "yield",
        "await",
        "eval",
        "arguments",
    ] {
        let text = format!("const x = {name};");
        assert!(
            matches!(
                attempt(&text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn escaped_identifier_reference_initializers_remain_selected_accepted_incomplete() {
    for text in [
        r"const x = \u0066oo;",
        r"const x = f\u006Fo;",
        r"let x = \u{66}oo, y = bar;",
        r"const x = \u{00000066}oo",
        r"const x = \u0066\u006F\u006F;",
        r"const x = \u{1D49C};",
        r"const x = \u0024;",
        r"const x = \u005F;",
        r"const x = a\u0030;",
        r"const x = a\u0301;",
        r"const x = a\u200Cb;",
        r"const x = a\u200Db;",
        r"const \u0078 = \u0066oo;",
        r"let x = \u0066oo, y = bar;",
        r"let x = foo, y = \u0062ar;",
        r"let x = 1, y = \u0062ar;",
        r"let x = \u0066oo, y = 1;",
        r"const x = \u0066oo, y = \u0062ar;",
        r"const x = \u00E9; const y = e\u0301;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }

    for rhs in [
        r"\u006Cet",
        r"\u0073tatic",
        r"\u0069mplements",
        r"\u0069nterface",
        r"\u0070ackage",
        r"\u0070rivate",
        r"\u0070rotected",
        r"\u0070ublic",
        r"\u0079ield",
        r"a\u0077ait",
        r"\u0065val",
        r"\u0061rguments",
    ] {
        let text = format!("const x = {rhs};");
        assert!(
            matches!(
                attempt(&text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }
}

/// Issue #791 (per #688 comment 5762579228): the newly selected
/// one-reference / one-plain-decimal heterogeneous additive initializer
/// reaches the same existing `SelectedAcceptedIncomplete` lifecycle as any
/// other production-accepted, not-yet-Oracle-qualified source -- never
/// `UnsupportedCoverage` and never `Qualified`. #789/#790 remain the
/// candidate-independent Oracle for the underlying theorem; this leaf adds
/// no new qualification branch.
#[test]
fn one_reference_one_plain_decimal_additive_initializer_remains_selected_accepted_incomplete() {
    for text in [
        "const x = a + 1;",
        "const x = 1 + a;",
        "let x = a - 1.0;",
        "let x = .5 - a;",
        "var x = a + 1e-2;",
        "var x = 1e-2 + a;",
        "{ var x = a - 1e2; }",
        "{ var x = 1e2 - a; }",
        r"const x = a + 1;",
        r"const x = 1 + a;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }
}

/// Issue #797 (per #688 comment 5771773635): the newly selected
/// exactly-three ordered `IdentifierReference` additive initializer reaches
/// the same existing `SelectedAcceptedIncomplete` lifecycle as any other
/// production-accepted, not-yet-Oracle-qualified source -- never
/// `UnsupportedCoverage` and never `Qualified` -- for all three joint
/// owners (`LexicalDeclaration`, top-level `var`, and Block `var`). #795/PR
/// #796 remain the candidate-independent Oracle for the underlying theorem;
/// this leaf adds no new qualification branch.
#[test]
fn three_identifier_reference_additive_initializer_remains_selected_accepted_incomplete() {
    for text in [
        "const x = a + b + c;",
        "let x = a - b - c;",
        "let x = a + b - c;",
        "let x = a - b + c;",
        "var x = a + b + c;",
        "{ var x = a + b + c; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }
}

/// Issue #803 (per #688 comment 5779735385): the newly selected unbounded
/// 2..N ordered `IdentifierReference` additive-chain initializer reaches the
/// same existing `SelectedAcceptedIncomplete` lifecycle as any other
/// production-accepted, not-yet-Oracle-qualified source -- never
/// `UnsupportedCoverage` and never `Qualified` -- for all three joint owners
/// (`LexicalDeclaration`, top-level `var`, and Block `var`), and for both
/// the fourth-operand transition and further 5+ growth. #801/PR #802 remain
/// the candidate-independent Oracle for the underlying theorem; this leaf
/// adds no new qualification branch.
#[test]
fn many_identifier_reference_additive_initializer_remains_selected_accepted_incomplete() {
    for text in [
        "const x = a + b + c + d;",
        "let x = a - b + c - d;",
        "const x = a + b + c + d + e;",
        "var x = a + b + c + d;",
        "{ var x = a + b + c + d; }",
        "var x = a + b + c + d + e;",
        "{ var x = a + b + c + d + e; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text}"
        );
    }
}

#[test]
fn escaped_identifier_reference_invalid_and_tail_families_remain_unsupported() {
    for text in [
        r"const x = \u{};",
        r"const x = \u0;",
        r"const x = \u{61",
        r"const x = \u{110000};",
        r"const x = \u{G};",
        r"const x = \u00G0;",
        r"const x = \u{61;",
        r"const x = \u0x;",
        r"const x = \u;",
        r"const x = \u{",
        r"const x = \u0030;",
        r"const x = a\u002D;",
        r"const x = \uD800;",
        r"const x = \u{D800};",
        r"const x = \u200C;",
        r"const x = \u200D;",
        r"const x = \u0066oo.bar;",
        r"const x = \u0066oo();",
        r"const x = \u0066oo = bar;",
        r"const x = \u0066oo ? bar : baz;",
        r"const x = \u0066oo/*comment*/;",
        r"const x = \u0066oo unexpected;",
        r"const x = \u0066oo;;",
        "const x = \\u0066oo\nconst y = bar;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text}"
        );
    }
}

fn qualification_outcome(text: &str) -> super::qualification::QualificationOutcome {
    match attempt(text) {
        SelectedQualificationAttempt::Outcome(outcome) => outcome,
        other => panic!("expected qualification outcome for {text:?}, got {other:?}"),
    }
}

#[test]
fn escaped_reserved_identifier_initializers_reach_existing_static_rejection() {
    for name in [
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
    ] {
        let rhs = escaped_first_ascii_code_point(name);
        let text = format!("const x = {rhs};");
        let outcome = qualification_outcome(&text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored static subject");
        assert_eq!(anchor.fragment(), rhs, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            (10, 10 + rhs.len()),
            "{text}"
        );
    }

    for rhs in [
        r"\u0074his",
        r"\u006Eull",
        r"\u0074rue",
        r"\u0066alse",
        r"\u0069f",
        r"\u0063lass",
        r"\u0069mport",
        r"\u0065xport",
        r"n\u0075ll",
        r"tr\u0075e",
        r"thi\u0073",
    ] {
        let text = format!("const x = {rhs};");
        let outcome = qualification_outcome(&text);
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored static subject");
        assert_eq!(anchor.fragment(), rhs, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            (10, 10 + rhs.len()),
            "{text}"
        );
    }
}

#[test]
fn escaped_reserved_initializer_static_precedence_survives_aggregate_handoff() {
    for (text, expected_fragment, expected_range) in [
        (r"let let = \u006Eull;", "let", (4, 7)),
        (r"let x = \u006Eull, x = foo;", r"\u006Eull", (8, 17)),
        (r"const x = \u006Eull, y;", r"\u006Eull", (10, 19)),
        (r"let x = foo; let x = \u006Eull;", r"\u006Eull", (21, 30)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored static subject");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn eof_asi_static_composition_preserves_existing_authored_subjects() {
    for (text, expected_fragment, expected_range) in [
        ("const x", "x", (6, 7)),
        ("let let", "let", (4, 7)),
        ("let x, x", "x", (7, 8)),
        ("let x; let x", "x", (11, 12)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored static subject");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn identifier_reference_static_composition_preserves_existing_authored_subjects() {
    for (text, expected_fragment, expected_range) in [
        ("let let = foo;", "let", (4, 7)),
        ("let x = foo, x = bar;", "x", (13, 14)),
        ("let x = foo; let x = bar;", "x", (17, 18)),
        (r"let \u006Cet = let;", r"\u006Cet", (4, 12)),
        (r"let \u0030 = foo;", r"\u0030", (4, 10)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored static subject");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn escaped_identifier_reference_static_reachability_preserves_existing_authored_subjects() {
    for (text, expected_fragment, expected_range) in [
        (r"let \u0030 = \u0066oo;", r"\u0030", (4, 10)),
        (r"let \u0069f = \u0066oo;", r"\u0069f", (4, 11)),
        (r"let \u006Cet = \u006Cet;", r"\u006Cet", (4, 12)),
        (r"let x = \u0066oo, x = \u0062ar;", "x", (18, 19)),
        (r"const x = \u0066oo, y;", "y", (20, 21)),
        (r"let x = \u0066oo; let x = \u0062ar;", "x", (22, 23)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored static subject");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn selected_r01_r02_r03_and_ee36_rejections_preserve_static_family() {
    for text in [
        "const let;",
        "let x, x;",
        "const x;",
        "let x, y; let y;",
        "let x, x; const y;",
        "const x; let y, y;",
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete);
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected)
        );
        assert_eq!(
            outcome
                .rejection_evidence()
                .expect("rejection evidence")
                .family(),
            RejectionFamily::StaticSemantics
        );
    }
}

#[test]
fn candidate_independent_primary_ranges_survive_aggregate_handoff() {
    for (fixture_id, expected_fragment) in [
        ("JS-GOLD-LEXDECL-DUPBOUNDNAMES-001", "x"),
        ("JS-GOLD-LEXDECL-CONST-MISSING-INIT-001", "x"),
        ("JS-GOLD-SCRIPT-DUPLEXICAL-MULTIBIND-001", "y"),
        ("JS-GOLD-LEXDECL-CONST-LET-MISSING-INIT-001", "let"),
        ("JS-GOLD-LEXDECL-CONST-DUP-MISSING-INIT-001", "x"),
    ] {
        let text = gold_source(fixture_id).unwrap_or_else(|| panic!("{fixture_id} source"));
        let expected =
            gold_subject_range(fixture_id).unwrap_or_else(|| panic!("{fixture_id} range"));
        let outcome = qualification_outcome(text);
        let anchor = outcome
            .rejection_evidence()
            .and_then(|evidence| evidence.subject().authored_anchor())
            .expect("authored rejection evidence");
        assert_eq!(anchor.fragment(), expected_fragment);
        assert_eq!((anchor.range().start(), anchor.range().end()), expected);
    }
}

#[test]
fn unsupported_rhs_and_broader_grammar_remain_unsupported_without_source_verdict() {
    for text in [
        gold_source("JS-GOLD-LEXDECL-CONST-MALFORMED-INIT-001").expect("malformed init gold"),
        "const x=if;",
        "const x=foo.bar;",
        "const x=foo();",
        "const x=(foo);",
        "const x=foo = bar;",
        "const x=foo ? bar : baz;",
        "const x=foo/*comment*/;",
        "const x=foo unexpected;",
        "const x=/a/;",
        "let [x]=y;",
        "'use strict'; let x=1;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn eof_asi_keeps_non_eof_incomplete_and_deferred_neighbors_unsupported() {
    for text in [
        "let x\nconst y = 1",
        "let x =",
        "let x,",
        "let x y",
        "let x = foo\nconst y = bar;",
        "let x/*comment*/",
        "#!node\nlet x = 1",
        r"let \u",
        r"let \u{",
        r"let \u0",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn eof_asi_preserves_existing_terminal_grammar_evidence() {
    for (text, expected_fragment, expected_range) in [
        (r"let \u{}", r"\u{}", (4, 8)),
        (r"let a\u{}", r"\u{}", (5, 9)),
        (r"let \u{61", r"\u{61", (4, 9)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::SyntaxRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("grammar evidence");
        assert_eq!(evidence.family(), RejectionFamily::Grammar, "{text}");
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored grammar subject");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn identifier_reference_coverage_makes_existing_later_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = foo; let \u{};", r"\u{}", (19, 23)),
        (r"const x = foo; let a\u{};", r"\u{}", (20, 24)),
        (r"const x = foo; let \u{61", r"\u{61", (19, 24)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::SyntaxRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("grammar evidence");
        assert_eq!(evidence.family(), RejectionFamily::Grammar, "{text}");
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored grammar subject");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn escaped_identifier_reference_coverage_makes_existing_later_grammar_evidence_reachable() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = \u0066oo; let \u{};", r"\u{}", (24, 28)),
        (r"const x = \u0066oo; let a\u{};", r"\u{}", (25, 29)),
        (r"const x = \u0066oo; let \u{61", r"\u{61", (24, 29)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::SyntaxRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("grammar evidence");
        assert_eq!(evidence.family(), RejectionFamily::Grammar, "{text}");
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored grammar subject");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn escaped_bindingidentifier_candidate_conforms_to_oracle_handoff() {
    for fixture_id in [
        "JS-GOLD-LEXDECL-ESCAPED-CANONICAL-DISTINCT-001",
        "JS-GOLD-LEXDECL-ESCAPED-CONTEXTUAL-NAMES-001",
        "JS-GOLD-LEXDECL-LONG-BRACED-ESCAPE-001",
    ] {
        let text = gold_source(fixture_id).unwrap_or_else(|| panic!("{fixture_id} source"));
        assert!(matches!(
            attempt(text),
            SelectedQualificationAttempt::SelectedAcceptedIncomplete
        ));
    }

    for fixture_id in [
        "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
        "JS-GOLD-IDENTIFIER-ESCAPED-PART-HYPHEN-001",
        "JS-GOLD-IDENTIFIER-ESCAPED-START-SURROGATE-FIXED-001",
        "JS-GOLD-IDENTIFIER-ESCAPED-START-SURROGATE-BRACED-001",
        "JS-GOLD-LEXDECL-ESCAPED-DUPBOUNDNAMES-001",
        "JS-GOLD-LEXDECL-DOLLAR-ESCAPED-DUPBOUNDNAMES-001",
        "JS-GOLD-LEXDECL-UNDERSCORE-ESCAPED-DUPBOUNDNAMES-001",
        "JS-GOLD-LEXDECL-SUPPLEMENTARY-ESCAPED-DUPBOUNDNAMES-001",
        "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
        "JS-GOLD-LEXDECL-ESCAPED-LET-BINDING-001",
    ] {
        let text = gold_source(fixture_id).unwrap_or_else(|| panic!("{fixture_id} source"));
        let expected =
            gold_subject_range(fixture_id).unwrap_or_else(|| panic!("{fixture_id} range"));
        let outcome = qualification_outcome(text);
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected)
        );
        let anchor = outcome
            .rejection_evidence()
            .and_then(|evidence| evidence.subject().authored_anchor())
            .expect("authored primary evidence");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected,
            "{fixture_id}"
        );
    }
}

#[test]
fn bounded_malformed_escaped_binding_gold_now_produces_grammar_rejection() {
    for (fixture_id, expected_fragment) in [
        (
            "JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-EMPTY-BRACED-001",
            r"\u{}",
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-SHORT-FIXED-001",
            r"\u0",
        ),
        (
            "JS-GOLD-IDENTIFIER-ESCAPE-MALFORMED-UNCLOSED-BRACED-001",
            r"\u{61",
        ),
        ("JS-GOLD-IDENTIFIER-ESCAPE-NONCODEPOINT-001", r"\u{110000}"),
    ] {
        let text = gold_source(fixture_id).unwrap_or_else(|| panic!("{fixture_id} source"));
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete);
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::SyntaxRejected)
        );
        let evidence = outcome.rejection_evidence().expect("grammar evidence");
        assert_eq!(evidence.family(), RejectionFamily::Grammar);
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored grammar evidence");
        assert_eq!(anchor.fragment(), expected_fragment, "{fixture_id}");
    }
}

#[test]
fn bounded_part_and_keyword_grammar_ranges_survive_aggregate_handoff() {
    for (text, expected_fragment, expected_range) in [
        (r"let a\u{};", r"\u{}", (5, 9)),
        (r"let a\u0;", r"\u0", (5, 8)),
        (r"let a\u{61", r"\u{61", (5, 10)),
        (r"let a\u{110000};", r"\u{110000}", (5, 15)),
        (r"let\u{};", r"\u{}", (3, 7)),
        (r"let\u{110000};", r"\u{110000}", (3, 13)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::SyntaxRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("grammar evidence");
        assert_eq!(evidence.family(), RejectionFamily::Grammar);
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("authored grammar evidence");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn grammar_primary_discards_tentative_static_evidence_and_is_terminal() {
    for text in [
        r"let \u0030\u{};",
        r"let a\u002D\u{};",
        r"let a\u00001\u{};",
        r"let \u{}\u0030;",
        r"let \u0069f; let \u{};",
        r"let let; let \u{};",
        r"const x; let \u{};",
        r"let x, x; let \u{};",
        r"let x; let x; let \u{};",
        r"let \u{} = foo;",
        r"let a\u{} = foo;",
        r"let \u{}, x;",
        r"let \u{}; var x;",
        r"const x = foo; let \u{};",
        r"const x = \u006Eull; let \u{};",
        r"var x=\u0069f; let \u{};",
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::SyntaxRejected),
            "{text}"
        );
        assert_eq!(
            outcome
                .rejection_evidence()
                .expect("grammar evidence")
                .family(),
            RejectionFamily::Grammar,
            "{text}"
        );
    }
}

#[test]
fn unowned_and_deferred_grammar_boundaries_remain_unsupported() {
    for text in [
        r"let \u0030; foo();",
        r"let\u002D\u{};",
        r"let\u0;",
        r"let\u{61",
        r"const\u{};",
        r"let \u{G};",
        r"let \u00G0;",
        r"let \u{61;",
        r"let \u0x;",
        r"let \u;",
        r"let \u{",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text}"
        );
    }
}

#[test]
fn grammar_handoff_rejects_anchor_from_another_source() {
    let source = SourceText::new(SourceId::new(231), r"let \u{};".to_owned());
    let other = SourceText::new(SourceId::new(232), r"let \u{};".to_owned());
    let anchor = other.anchor(4, 8).expect("other source anchor");
    let outcome = selected_grammar_rejection_to_qualification(&source, anchor);
    assert_eq!(outcome.processing(), ProcessingStatus::InternalFailure);
    assert_eq!(outcome.verdict(), None);
    assert!(outcome.rejection_evidence().is_none());
}

#[test]
fn aggregate_integration_source_preserves_incomplete_and_single_pass_boundaries() {
    let production = include_str!("selected_qualification_integration.rs");

    for forbidden in [
        concat!("QualificationOutcome", "::qualified"),
        concat!("CompleteQualification", "Witness"),
        concat!("source.", "as_str()"),
        concat!("source.", "anchor("),
        concat!("unicode_", "normalization"),
        concat!("qualification_validation", "_tests"),
    ] {
        assert!(
            !production.contains(forbidden),
            "selected aggregate integration must preserve architecture boundary: found {forbidden}"
        );
    }

    assert_eq!(
        production
            .matches("recognize_selected_lexical_slice(source)")
            .count(),
        1
    );
    assert_eq!(
        production
            .matches("evaluate_selected_static_semantics(&script)")
            .count(),
        1
    );
    assert_eq!(
        production
            .matches("evaluate_selected_one_level_block_static_semantics(&script)")
            .count(),
        1
    );
    assert_eq!(
        production
            .matches("evaluate_selected_variable_statement_static_semantics(&script)")
            .count(),
        1
    );
    assert!(production.contains("SelectedAcceptedIncomplete"));
    assert!(production.contains("EvidenceSubject::authored(source, subject)"));
    assert!(production.contains("QualificationOutcome::syntax_rejected(subject)"));
}

#[test]
fn one_level_block_accepted_sources_remain_selected_accepted_incomplete() {
    for text in [
        "{ let a=1; }",
        "let a=1; { let a=2; }",
        "{ let a=1; } { let a=2; }",
        r"let a=1; { let x=\u0061; }",
        r"let é=1; { let x=e\u0301; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn one_level_block_static_rejections_preserve_region_owned_authored_subjects() {
    for (text, expected_range) in [
        ("{ let a=1; let a=2; }", (15, 16)),
        ("let a=1; { let a=2; } let a=3;", (26, 27)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("Block/static subject must remain authored");
        assert_eq!(anchor.fragment(), "a", "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn one_level_block_unsupported_neighbors_remain_unsupported_coverage() {
    for text in [
        "{}",
        "{ let a=1 }",
        "{ { let a=1; } }",
        "{ let a=1; /*c*/ let x=a; }",
        "{ function f(){} }",
        "{ 1; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn top_level_variable_statement_positive_sources_remain_selected_accepted_incomplete() {
    for text in [
        "var x;",
        "var x",
        "var x   \t\n",
        "var let;",
        r"var \u006Cet;",
        r"var \u006Cet",
        "let x; var y;",
        "var x; var x;",
        "{ let x; } var x;",
        "let é; var e\u{301};",
        "var x; let y",
        "var a=0;",
        "var a = 1",
        "var a=foo;",
        "var x = y;",
        "var x=foo",
        "var π=𝒜;",
        "var x=$;",
        "var x=_;",
        "var x=a0;",
        "var x=let;",
        "var x=yield;",
        "var x=await;",
        "var x=eval;",
        "var x=arguments;",
        r"var x=\u0066oo;",
        r"var x=f\u006Fo;",
        r"var x=\u0066oo",
        r"var x=\u006Cet;",
        r"var x=\u0079ield;",
        r"var x=a\u0077ait;",
        r"var x=\u0065val;",
        r"var x=\u0061rguments;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn top_level_variable_statement_static_rejections_preserve_authored_primary_subject() {
    for (text, expected_fragment, expected_range) in [
        (r"var \u0069f;", r"\u0069f", (4, 11)),
        (r"var \u0069f", r"\u0069f", (4, 11)),
        (r"var \u0069f=1;", r"\u0069f", (4, 11)),
        (r"var x=\u0069f;", r"\u0069f", (6, 13)),
        (r"var x=n\u0075ll;", r"n\u0075ll", (6, 15)),
        (r"var x=\u0069f", r"\u0069f", (6, 13)),
        ("let x; var x;", "x", (11, 12)),
        ("let x; var x", "x", (11, 12)),
        ("var x; let x;", "x", (11, 12)),
        ("var y; var x; let x; let y;", "x", (18, 19)),
        ("var x; var x; let x;", "x", (18, 19)),
        ("let y; var y; var x; let x;", "y", (11, 12)),
        (r"let x,x; var \u0069f;", "x", (6, 7)),
        (r"let a; var a=1, \u0069f=2;", r"\u0069f", (16, 23)),
        (r"let a; var a=foo, \u0069f=bar;", r"\u0069f", (18, 25)),
        ("let b; var a=foo,b=bar;", "b", (17, 18)),
        (r"let a; var a; var x=\u0061;", "a", (11, 12)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("var/static primary must remain authored");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn top_level_variable_statement_grammar_and_deferred_boundaries_remain_distinct() {
    let outcome = qualification_outcome(r"var \u{};");
    assert_eq!(outcome.processing(), ProcessingStatus::Complete);
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::SyntaxRejected)
    );
    let evidence = outcome.rejection_evidence().expect("grammar evidence");
    assert_eq!(evidence.family(), RejectionFamily::Grammar);
    let anchor = evidence
        .subject()
        .authored_anchor()
        .expect("var grammar subject must remain authored");
    assert_eq!(anchor.fragment(), r"\u{}");
    assert_eq!((anchor.range().start(), anchor.range().end()), (4, 8));

    for text in [
        r"var\u{};",
        "var x\nvar y;",
        r"var x=\u0030;",
        r"var x=a\u002Db;",
        r"var x=\u0066oo.bar;",
        r"var x=\u0066oo();",
        "var x=foo.bar;",
        "var x=foo();",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }

    let outcome = qualification_outcome(r"var x=\u0066oo; let \u{};");
    assert_eq!(outcome.processing(), ProcessingStatus::Complete);
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::SyntaxRejected)
    );
    let evidence = outcome
        .rejection_evidence()
        .expect("later grammar evidence");
    assert_eq!(evidence.family(), RejectionFamily::Grammar);
    let anchor = evidence
        .subject()
        .authored_anchor()
        .expect("authored later grammar subject");
    assert_eq!(anchor.fragment(), r"\u{}");
    assert_eq!((anchor.range().start(), anchor.range().end()), (20, 24));

    let outcome = qualification_outcome(r"var x=\u0069f; let \u{};");
    assert_eq!(outcome.processing(), ProcessingStatus::Complete);
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::SyntaxRejected)
    );
    let evidence = outcome
        .rejection_evidence()
        .expect("later grammar evidence must discard tentative C6 static evidence");
    assert_eq!(evidence.family(), RejectionFamily::Grammar);
    let anchor = evidence
        .subject()
        .authored_anchor()
        .expect("authored later grammar subject");
    assert_eq!(anchor.fragment(), r"\u{}");
    assert_eq!((anchor.range().start(), anchor.range().end()), (19, 23));
}

#[test]
fn multi_declarator_variable_statements_remain_selected_accepted_incomplete() {
    for text in [
        "var a,b;",
        "var a,b",
        "var a,b,c;",
        "var a,a;",
        "var a, let;",
        "var a,\n b",
        "var a\n, b;",
        r"var a,\u0061;",
        r"var é,e\u0301;",
        "let x; var a,b;",
        "{ let x; } var a,b;",
        "var a,b; var c; let d",
        "var a=1,b;",
        "var a,b=2;",
        "var a=1,b=2;",
        "var a=1,b,c=2;",
        "var a=1,b=2,c=3",
        "var a=1\n, b=2;",
        "var a=1,\n b=2",
        r"var a=1,\u0061=2;",
        "var x=foo,y;",
        "var x,y=foo;",
        "var x=foo,y=bar;",
        "var x=1,y=foo;",
        "var x=foo,y=2,z;",
        "var x=foo,y=bar,z=baz",
        "let a; var b; var x=a,y=b,z=q;",
        r"var x=\u0066oo,y;",
        r"var x,y=\u0066oo;",
        r"var x=\u0066oo,y=bar;",
        r"var x=foo,y=\u0062ar;",
        r"var x=1,y=\u0062ar;",
        r"var x=\u0066oo,y=2,z;",
        r"var x=\u0066oo,y=\u0062ar,z=baz",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn multi_declarator_static_rejections_preserve_the_authored_primary_subject() {
    for (text, expected_fragment, expected_range) in [
        (r"var a, \u0069f;", r"\u0069f", (7, 14)),
        (r"let b; var a,b,\u0069f;", r"\u0069f", (15, 22)),
        (r"var a=1,b=\u0069f,c;", r"\u0069f", (10, 17)),
        ("let b; var a,b;", "b", (13, 14)),
        ("var b,a; let a,b;", "a", (13, 14)),
        ("var a,a; let a;", "a", (13, 14)),
        ("let b; var a=1,b=2;", "b", (15, 16)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("static evidence");
        assert_eq!(
            evidence.family(),
            RejectionFamily::StaticSemantics,
            "{text}"
        );
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("var/static primary must remain authored");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn multi_declarator_grammar_rejection_stays_distinct_from_deferred_coverage() {
    // "{ var a,b; }" is deliberately not listed below: Issue #695 makes
    // multi-declarator Block `var` statements a selected accepted form.
    for (text, expected_range) in [
        (r"var a,\u{};", (6, 10)),
        (r"var a,b,\u{};", (8, 12)),
        (r"var a,\u{}=1;", (6, 10)),
        (r"var a=1,b,\u{}=2;", (10, 14)),
        (r"var x=foo,y=bar,\u{}=baz;", (16, 20)),
    ] {
        let outcome = qualification_outcome(text);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete, "{text}");
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::SyntaxRejected),
            "{text}"
        );
        let evidence = outcome.rejection_evidence().expect("grammar evidence");
        assert_eq!(evidence.family(), RejectionFamily::Grammar, "{text}");
        let anchor = evidence
            .subject()
            .authored_anchor()
            .expect("var grammar subject must remain authored");
        assert_eq!(anchor.fragment(), r"\u{}", "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }

    // "var a=this;" is deliberately not listed below: Issue #723 makes a
    // direct-authored `PrimaryExpression : this` initializer a selected
    // accepted form (see the "Issue #723" test section below). "var
    // a="x";" is also deliberately not listed below: Issue #725 makes a
    // direct-authored, escape-free `StringLiteral` initializer a selected
    // accepted form (see the "Issue #725" test section below). "var
    // a=1,b=1.0;" and "var a=1.0;" are also deliberately not listed below:
    // Issue #732 makes a direct-authored plain fractional `DecimalLiteral`
    // initializer a selected accepted form (see the "Issue #732" test
    // section below). "var a=1e2;" is also deliberately not listed below:
    // Issue #740 makes a direct-authored, separator-free exponent
    // `DecimalLiteral` initializer a selected accepted form (see the
    // "Issue #740" test section below). "var a=+1;" and "var a=-1;" are
    // also deliberately not listed below: Issue #744 makes a
    // direct-authored leading `+`/`-` decimal `UnaryExpression` initializer
    // a selected accepted form (see the "Issue #744" test section below).
    for text in [
        "var a,",
        "var a,b,",
        "var a=1,",
        "var a=1,b=",
        "var a=01;",
        "var a=1_0;",
        "var a=1n;",
        r"var a=\u0030;",
        r"var a=a\u002Db;",
        "var a=foo.bar;",
        "var a,{b}=c;",
        "var a, if;",
        "var a=1, /*comment*/ b;",
        "for (var a,b;;) {}",
        "var a=1\nvar c;",
        "var x=foo,y=bar,z=",
        r"var x=\u0066oo,y=\u0062ar,z=",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

// --- Issue #691: one-level Block-contained bare `var` production leaf -----
//
// These focused production tests seal the candidate against the #688/#689
// theorem independently validated (without calling production) by PR #690's
// `qualification_selected_one_level_block_bare_var_validation_tests.rs`. They
// exercise the real production entry point (`attempt_selected_qualification`)
// rather than duplicating that oracle's fixture set.

fn assert_static_semantics_rejected(
    text: &str,
    expected_fragment: &str,
    expected_range: (usize, usize),
) {
    let outcome = qualification_outcome(text);
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::StaticSemanticsRejected),
        "{text:?}"
    );
    let evidence = outcome.rejection_evidence().expect("static evidence");
    assert_eq!(
        evidence.family(),
        RejectionFamily::StaticSemantics,
        "{text:?}"
    );
    let anchor = evidence
        .subject()
        .authored_anchor()
        .expect("authored static subject");
    assert_eq!(anchor.fragment(), expected_fragment, "{text:?}");
    assert_eq!(
        (anchor.range().start(), anchor.range().end()),
        expected_range,
        "{text:?}"
    );
}

#[test]
fn bare_block_var_positive_leaf_is_selected_accepted_incomplete() {
    assert!(matches!(
        attempt("{ var x; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
}

#[test]
fn bare_block_var_lexical_collision_reaches_ee14_r02_in_both_source_orders() {
    assert_static_semantics_rejected("{ let x; var x; }", "x", (13, 14));
    assert_static_semantics_rejected("{ var x; let x; }", "x", (13, 14));
}

#[test]
fn bare_block_var_lexical_distinct_names_remain_accepted_in_both_source_orders() {
    for text in ["{ let x; var y; }", "{ var x; let y; }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn bare_block_var_propagates_into_existing_script_ee36_r02_in_both_placements() {
    assert_static_semantics_rejected("let x; { var x; }", "x", (13, 14));
    assert_static_semantics_rejected("{ var x; } let x;", "x", (15, 16));
}

#[test]
fn bare_block_var_script_propagation_accepts_distinct_names() {
    assert!(matches!(
        attempt("let x; { var y; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
}

#[test]
fn outer_var_inner_lexical_asymmetry_is_not_falsely_symmetric() {
    // `var x; { let x; }` must remain accepted: a top-level var contributor
    // never enters an inner Block's own VarDeclaredNames, unlike the
    // symmetric-looking `let x; { var x; }`, which does reach EE-36-R02 (see
    // `bare_block_var_propagates_into_existing_script_ee36_r02_in_both_placements`).
    assert!(matches!(
        attempt("var x; { let x; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
}

#[test]
fn semantic_name_equality_not_authored_spelling_drives_block_collision() {
    // Authored spellings differ ("a" vs a six-character Unicode escape); only
    // semantic-name equality must drive the EE-14-R02 collision.
    let escape = escaped_first_ascii_code_point("a");
    let text = format!("{{ let a; var {escape}; }}");
    assert_static_semantics_rejected(&text, &escape, (13, 13 + escape.len()));
}

#[test]
fn repeated_bare_block_var_names_alone_remain_accepted() {
    assert!(matches!(
        attempt("{ var x; var x; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
}

#[test]
fn block_ee14_r02_outranks_script_ee36_r02_when_both_are_true() {
    assert_static_semantics_rejected("let x; { let x; var x; }", "x", (20, 21));
}

#[test]
fn block_ee14_r01_outranks_block_ee14_r02_in_the_same_block() {
    assert_static_semantics_rejected("{ let x; let x; var x; }", "x", (13, 14));
}

#[test]
fn multi_block_first_qualifying_block_supplies_ee14_r02_primary_evidence() {
    assert_static_semantics_rejected("{ let x; var x; } { let y; var y; }", "x", (13, 14));
}

#[test]
fn ee14_r01_tier_outranks_earlier_block_ee14_r02_by_cross_block_source_position() {
    // The earlier Block only reaches Tier 2b (EE-14-R02); the later Block
    // reaches Tier 2a (EE-14-R01). Tier priority must still decide, so the
    // later Block's EE-14-R01 wins even though the earlier Block's EE-14-R02
    // would otherwise be the first-in-source collision.
    assert_static_semantics_rejected("{ let x; var x; } { let y; let y; }", "y", (31, 32));
}

#[test]
fn bare_block_var_unsupported_boundaries_remain_unsupported_coverage() {
    // "{var x,y;}" is deliberately not listed here: Issue #695 makes
    // multi-declarator Block `var` statements a selected accepted form (see
    // the "Issue #695" test section below). "{var x=x;}" is likewise not
    // listed here: Issue #710 makes a direct-authored, escape-free
    // `IdentifierReference` initializer a selected accepted form (see the
    // "Issue #710" test section below). "{var x}" is likewise not listed
    // here: Issue #717 makes bounded close-brace ASI a selected accepted
    // termination route (see the "Issue #717" test section below).
    for text in [
        "{var x; /*c*/}",        // comment trivia
        "{ { var x; } }",        // deeper Block nesting
        "for (var x;;) {}",      // for(var...)
        "{var [x]=y;}",          // BindingPattern
        "{ var x; } export {};", // Module-only widening
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

// --- Issue #695: one-level Block `var` widened to 1..N declarators --------
//
// These focused production tests seal the candidate against the #693/#694
// theorem independently validated (without calling production) by
// `qualification_validation_tests/selected_one_level_block_var_multi_declarator_frontier.rs`
// and `selected_post_693_block_var_multi_declarator_slice_completion.rs`.
// They exercise the real production entry point
// (`attempt_selected_qualification`) with independently authored expected
// values rather than importing or deriving from that oracle.

fn assert_grammar_rejected(text: &str, expected_fragment: &str, expected_range: (usize, usize)) {
    let outcome = qualification_outcome(text);
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::SyntaxRejected),
        "{text:?}"
    );
    let evidence = outcome.rejection_evidence().expect("grammar evidence");
    assert_eq!(evidence.family(), RejectionFamily::Grammar, "{text:?}");
    let anchor = evidence
        .subject()
        .authored_anchor()
        .expect("authored grammar subject");
    assert_eq!(anchor.fragment(), expected_fragment, "{text:?}");
    assert_eq!(
        (anchor.range().start(), anchor.range().end()),
        expected_range,
        "{text:?}"
    );
}

#[test]
fn block_var_multi_declarator_positive_cardinality_is_selected_accepted_incomplete() {
    // Defeats an accidental 2-item cardinality cap.
    for text in ["{ var x, y; }", "{ var x, y, z; }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_multi_declarator_repeated_contributors_remain_accepted() {
    // Defeats a wrong "deduplicated contributor set" mental model.
    for text in ["{ var x, x; }", "{ var a, \\u0061; }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_multi_declarator_ee14_r02_reaches_first_interior_and_final_positions() {
    // Defeats both a "first declarator only" and a "last declarator only"
    // wrong oracle: each fixture collides at a *different* declarator
    // position within the same-shaped VariableDeclarationList.
    assert_static_semantics_rejected("{ let x; var x, y; }", "x", (13, 14));
    assert_static_semantics_rejected("{ let y; var x, y; }", "y", (16, 17));
    assert_static_semantics_rejected("{ let y; var x, y, z; }", "y", (16, 17));
    assert_static_semantics_rejected("{ let z; var x, y, z; }", "z", (19, 20));
}

#[test]
fn block_var_multi_declarator_ee14_r02_lexical_after_list_is_offset_discriminating() {
    // The primary evidence is the later-in-source occurrence that completes
    // the collision, not a hard-coded numeric offset: these two fixtures use
    // different offsets for the completing lexical binding (17 vs 20).
    assert_static_semantics_rejected("{ var x, y; let y; }", "y", (16, 17));
    assert_static_semantics_rejected("{ var x, y, z; let y; }", "y", (19, 20));
}

#[test]
fn block_var_multi_declarator_ee36_r02_propagates_every_declarator_position() {
    // The colliding contributor is the *second* declarator of the Block's
    // own VariableDeclarationList, defeating a "first declarator only"
    // Script-propagation model.
    assert_static_semantics_rejected("let y; { var x, y; }", "y", (16, 17));
    assert_static_semantics_rejected("{ var x, y; } let y;", "y", (18, 19));
}

#[test]
fn block_var_multi_declarator_top_level_var_block_lexical_asymmetry_remains_accepted() {
    assert!(matches!(
        attempt("var y; { let y; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
}

#[test]
fn block_var_multi_declarator_escaped_collisions_use_semantic_equality_at_any_position() {
    assert_static_semantics_rejected("{ let a; var \\u0061, x; }", "\\u0061", (13, 19));
    assert_static_semantics_rejected("{ let a; var x, \\u0061; }", "\\u0061", (16, 22));
}

#[test]
fn block_var_multi_declarator_no_unicode_normalization_remains_accepted() {
    assert!(matches!(
        attempt("{ var é, e\u{0301}; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
}

#[test]
fn block_var_multi_declarator_same_tier_sibling_blocks_select_first_qualifying_block() {
    // Both Blocks independently qualify for Tier 2b at their own second
    // declarator; project evidence-order policy selects the first Block in
    // authored source order.
    assert_static_semantics_rejected("{ let a; var x, a; }\n{ let b; var y, b; }", "a", (16, 17));
}

#[test]
fn block_var_multi_declarator_tier_outranks_cross_block_source_position() {
    // The earlier Block only reaches Tier 2b (EE-14-R02); the later Block
    // reaches Tier 2a (EE-14-R01). Tier priority must still decide, so the
    // later Block's EE-14-R01 wins.
    assert_static_semantics_rejected("{ let a; var x, a; } { let b; let b; }", "b", (34, 35));
}

#[test]
fn block_var_multi_declarator_malformed_later_binding_is_definitively_syntax_rejected() {
    // The earlier valid "x" declarator must not commit; the later malformed
    // escape must remain a definitive grammar rejection, never downgraded to
    // UnsupportedCoverage merely for appearing late in the declarator list.
    assert_grammar_rejected(r"{ var x, \u{}; }", r"\u{}", (9, 13));
}

#[test]
fn block_var_multi_declarator_incomplete_and_firewalled_lists_remain_unsupported_coverage() {
    for text in [
        "{ var x, ; }",          // incomplete list: missing later declarator
        "{ var x, y, ; }",       // incomplete list: missing later declarator
        "{ var x, /* c */ y; }", // comment-trivia firewall, before declarator
        "{ var x, y /* c */; }", // comment-trivia firewall, after declarator
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

// --- Issue #699: one-level Block `var` widened to optional selected -------
// decimal-integer initializers per declarator
//
// These focused production tests seal the candidate against the #697/#698
// theorem independently validated (without calling production) by
// `qualification_validation_tests/selected_one_level_block_var_decimal_initializer_frontier.rs`
// and `selected_post_block_var_decimal_initializer_slice_completion.rs`. They
// exercise the real production entry point (`attempt_selected_qualification`)
// with independently authored expected values rather than importing or
// deriving from that oracle.

#[test]
fn block_var_decimal_initializer_positive_cardinality_and_shapes_are_selected_accepted_incomplete()
{
    for text in [
        "{ var a=0; }",
        "{ var a=12345; }",
        "{ var a=1,b; }",
        "{ var a,b=2; }",
        "{ var a=1,b=2; }",
        "{ var a=1,b,c=2; }",
        "{ var a,b=2,c; }",
        "{ var a=1,b=2,c=3; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_decimal_initializer_repeated_and_escaped_contributors_remain_accepted() {
    // Defeats a wrong "deduplicated contributor set" mental model even when
    // one occurrence carries a decimal initializer.
    for text in ["{ var a=1,a; }", r"{ var a=1,\u0061=2; }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_decimal_initializer_ee14_r02_reaches_first_interior_and_final_positions() {
    // Defeats both a "first declarator only" and a "last declarator only"
    // wrong oracle, and confirms the decimal initializer never becomes
    // collision evidence: the primary anchor is always the colliding
    // BindingIdentifier, never the RHS digits.
    assert_static_semantics_rejected("{ let a; var a=1,b; }", "a", (13, 14));
    assert_static_semantics_rejected("{ let b; var a=1,b=2; }", "b", (17, 18));
    assert_static_semantics_rejected("{ let b; var a,b=2,c; }", "b", (15, 16));
    assert_static_semantics_rejected("{ let c; var a=1,b=2,c=3; }", "c", (21, 22));
}

#[test]
fn block_var_decimal_initializer_ee14_r02_lexical_after_list_is_offset_discriminating() {
    assert_static_semantics_rejected("{ var a=1,b=2; let b; }", "b", (19, 20));
}

#[test]
fn block_var_decimal_initializer_ee36_r02_propagates_non_first_declarator() {
    // The colliding contributor is the *second* declarator of the Block's
    // own VariableDeclarationList, defeating a "first declarator only"
    // Script-propagation model, in both source-order placements.
    assert_static_semantics_rejected("let b; { var a=1,b=2; }", "b", (17, 18));
    assert_static_semantics_rejected("{ var a=1,b=2; } let b;", "b", (21, 22));
}

#[test]
fn block_var_decimal_initializer_script_block_asymmetry_remains_accepted() {
    assert!(matches!(
        attempt("var b; { let b; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
}

#[test]
fn block_var_decimal_initializer_same_tier_sibling_blocks_select_first_qualifying_block() {
    // Both Blocks independently qualify for Tier 2b at their own second,
    // initialized declarator; project evidence-order policy selects the
    // first Block in authored source order.
    assert_static_semantics_rejected(
        "{ let a; var x=1,a=2; } { let b; var y=3,b=4; }",
        "a",
        (17, 18),
    );
}

#[test]
fn block_var_decimal_initializer_tier_outranks_cross_block_source_position() {
    // The earlier Block only reaches Tier 2b (EE-14-R02) via an initialized
    // declarator; the later Block reaches Tier 2a (EE-14-R01). Tier priority
    // must still decide, so the later Block's EE-14-R01 wins.
    assert_static_semantics_rejected("{ let a; var x=1,a=2; } { let b; let b; }", "b", (37, 38));
}

#[test]
fn block_var_decimal_initializer_tier1_escaped_reserved_word_outranks_tier2b_block_collision() {
    // Mandatory race: the same statement carries both a Tier-2b Block
    // lexical/var collision on "a" and a later Tier-1 binding-local
    // EE-04-R08 (escaped ReservedWord BindingIdentifier). Tier 1 must win
    // even though the Tier-2b collision completes earlier in source order.
    assert_static_semantics_rejected(r"{ let a; var a=1, \u0069f=2; }", r"\u0069f", (18, 25));
}

#[test]
fn block_var_decimal_initializer_no_unicode_normalization_remains_accepted() {
    assert!(matches!(
        attempt("{ var é=1, e\u{0301}=2; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
}

#[test]
fn block_var_decimal_initializer_numeric_neighbors_remain_unsupported_coverage() {
    // "{ var a=1.0; }" is deliberately not listed here: Issue #732 makes a
    // direct-authored plain fractional `DecimalLiteral` initializer a
    // selected accepted form (see the "Issue #732" test section below).
    // "{ var a=1e2; }" is also deliberately not listed here: Issue #740
    // makes a direct-authored, separator-free exponent `DecimalLiteral`
    // initializer a selected accepted form (see the "Issue #740" test
    // section below). "{ var a=+1; }" and "{ var a=-1; }" are also
    // deliberately not listed here: Issue #744 makes a direct-authored
    // leading `+`/`-` decimal `UnaryExpression` initializer a selected
    // accepted form (see the "Issue #744" test section below).
    for text in ["{ var a=01; }", "{ var a=1_0; }", "{ var a=1n; }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

// The former `block_var_other_initializer_families_remain_unsupported_coverage`
// sentinel is removed here: its only remaining case, `{ var a="x"; }`, is
// no longer `UnsupportedCoverage`. Issue #725 makes a direct-authored,
// escape-free `StringLiteral` initializer a selected accepted form (see the
// "Issue #725" test section below). Every other family this sentinel used
// to guard (IdentifierReference, escaped IdentifierReference,
// escaped-ReservedWord C6, BooleanLiteral, NullLiteral, and `this`) was
// already migrated to its own accepted test section by the corresponding
// earlier Issue.

#[test]
fn block_var_decimal_initializer_comment_neighbors_remain_unsupported_coverage() {
    for text in [
        "{ var a=1, /* c */ b; }",
        "{ var a=/* c */1; }",
        "{ var a=1 /* c */; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_decimal_initializer_transactional_failure_commits_no_prefix() {
    // A valid initialized declarator prefix (e.g. "a=1") must never escape as
    // a committed Block var contributor when a later declarator fails: the
    // whole statement is transactional. "{ var a=1,b=1.0; }" is deliberately
    // not listed here: Issue #732 makes a direct-authored plain fractional
    // `DecimalLiteral` initializer a selected accepted form.
    for text in ["{ var a=1, ; }", "{ var a=1,b= ; }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_decimal_initializer_malformed_later_binding_is_definitively_syntax_rejected() {
    // The earlier valid "a=1,b" declarator prefix must not commit; the later
    // malformed escape must remain a definitive grammar rejection, never
    // downgraded to UnsupportedCoverage merely because an earlier declarator
    // carried a selected decimal initializer.
    assert_grammar_rejected(r"{ var a=1,b,\u{}=2; }", r"\u{}", (12, 16));
}

// --- Issue #710: one-level Block `var` widened to a direct-authored, ------
// escape-free `IdentifierReference` initializer
//
// These focused production tests exercise the real production entry point
// (`attempt_selected_qualification`) with independently authored expected
// values, sealing the candidate against the accepted #688/#710 theorem.

#[test]
fn block_var_direct_identifier_reference_initializer_is_selected_accepted_incomplete() {
    for text in [
        "{ var x=a; }",
        "{ var x=x; }",
        "{ var x=a,y=b; }",
        "{ var x=1,y=a,z; }",
        "{ var x=a,y=2,z=b; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_direct_identifier_reference_initializer_does_not_become_bound_name_evidence() {
    // W7 (kills "the RHS semantic name accidentally becomes a
    // VarDeclaredName/static-error contributor"): the Block-var RHS
    // reference name must never participate in BoundNames/EE-14/EE-36
    // collision detection; only the LHS `BindingIdentifier` does.
    assert!(matches!(
        attempt("{ let a; var x=a; }"),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));
    assert_static_semantics_rejected("{ let x; var x=a; }", "x", (13, 14));
}

// --- Issue #713: one-level Block `var` widened to a selected escaped ------
// non-ReservedWord `IdentifierReference` initializer
//
// These focused production tests exercise the real production entry point
// (`attempt_selected_qualification`) with independently authored expected
// values, sealing the candidate against the accepted #712/#713 theorem.
// `escaped_first_ascii_code_point` (defined above) builds the escaped RHS
// spelling programmatically so this section never hand-types a literal
// ECMAScript `\uXXXX` escape.

#[test]
fn block_var_escaped_identifier_reference_initializer_is_selected_accepted_incomplete() {
    for text in [
        format!("{{ var x={}; }}", escaped_first_ascii_code_point("a")),
        format!("{{ var x={},y=b; }}", escaped_first_ascii_code_point("foo")),
        format!(
            "{{ var x=1,y={},z; }}",
            escaped_first_ascii_code_point("bar")
        ),
        format!(
            "{{ var x={},y=2,z={}; }}",
            escaped_first_ascii_code_point("a"),
            escaped_first_ascii_code_point("b"),
        ),
    ] {
        assert!(
            matches!(
                attempt(&text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_escaped_identifier_reference_initializer_does_not_become_bound_name_evidence() {
    // Escaped counterpart of the #710 W7 seal: an escaped Block-var RHS
    // reference must never participate in BoundNames/EE-14/EE-36 collision
    // detection; only the LHS `BindingIdentifier` does.
    let text = format!(
        "{{ let a; var x={}; }}",
        escaped_first_ascii_code_point("a")
    );
    assert!(matches!(
        attempt(&text),
        SelectedQualificationAttempt::SelectedAcceptedIncomplete
    ));

    let text = format!(
        "{{ let x; var x={}; }}",
        escaped_first_ascii_code_point("a")
    );
    assert_static_semantics_rejected(&text, "x", (13, 14));
}

// --- Issue #715: one-level Block `var` widened to a selected escaped -----
// ReservedWord `IdentifierName` initializer source position (existing
// `EE-04-R08`)
//
// These focused production tests exercise the real production entry point
// (`attempt_selected_qualification`) with independently authored expected
// values, sealing the candidate against the accepted #688-comment-5683929558
// theorem. They reuse the existing #340/#341/#342/#343 top-level C6 oracle
// authority and the accepted Block-var Tier-1/transactionality lineage
// rather than cloning a new oracle.

#[test]
fn block_var_escaped_reserved_identifier_name_initializer_reaches_ee04_r08() {
    // W5 successor: an escaped spelling that the shared recognizer
    // classifies as a ReservedWord now reaches complete selected
    // recognition and the existing EE-04-R08 static rejection, at the
    // production entry point, rather than remaining `UnsupportedCoverage`.
    let text = format!("{{ var x={}; }}", escaped_first_ascii_code_point("if"));
    assert_static_semantics_rejected(&text, r"\u0069f", (8, 15));
}

#[test]
fn block_var_escaped_reserved_identifier_name_initializer_no_prefix_commit() {
    // Richer-expression / no-prefix-commit firewall: a member-expression
    // suffix after the escaped ReservedWord `IdentifierName` keeps the
    // whole statement outside this leaf's accepted profile.
    let text = format!("{{ var x={}.foo; }}", escaped_first_ascii_code_point("if"));
    assert!(
        matches!(
            attempt(&text),
            SelectedQualificationAttempt::UnsupportedCoverage
        ),
        "{text:?}"
    );
}

#[test]
fn block_var_escaped_reserved_identifier_name_initializer_transactional_rollback() {
    // Whole-source transactionality: a later declarator that prevents
    // complete selected recognition must not let the tentative C6 fact
    // from an earlier declarator escape as a committed candidate.
    let text = format!("{{ var x={},y=; }}", escaped_first_ascii_code_point("if"));
    assert!(
        matches!(
            attempt(&text),
            SelectedQualificationAttempt::UnsupportedCoverage
        ),
        "{text:?}"
    );
}

#[test]
fn block_var_escaped_reserved_identifier_name_initializer_correspondence_suppression() {
    // Correspondence suppression: a completely recognized source that
    // reaches the escaped-ReservedWord initializer rejects during static
    // semantics before any static-acceptance witness exists, so
    // correspondence never produces a committed relation from it.
    let text = format!(
        "{{ var a=foo,x={}; }}",
        escaped_first_ascii_code_point("if")
    );
    assert_static_semantics_rejected(&text, r"\u0069f", (14, 21));
}

// --- Issue #717: one-level Block `var` widened to admit bounded --------
// close-brace ASI as an additional statement terminator
//
// These focused production tests exercise the real production entry point
// (`attempt_selected_qualification`) with independently authored expected
// values, sealing the new terminator route against the accepted
// #688-comment-5685046994 theorem and the existing #318/#320 EOF-only ASI
// terminator-provenance precedent. No separate candidate-independent oracle
// predecessor is used for this leaf.

#[test]
fn block_var_close_brace_asi_positive_sources_remain_selected_accepted_incomplete() {
    for text in [
        "{ var x }",
        "{ var x, y }",
        "{ var a=1 }",
        "{ var a=1,b=2 }",
        "{ var x=a }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_close_brace_asi_reaches_same_ee14_r02_as_authored_semicolon() {
    // `{ let x; var x }` must reach the same Block EE-14-R02 collision, at
    // the same authored subject position, as the authored-semicolon form.
    assert_static_semantics_rejected("{ let x; var x; }", "x", (13, 14));
    assert_static_semantics_rejected("{ let x; var x }", "x", (13, 14));
}

#[test]
fn block_var_close_brace_asi_reaches_same_ee36_r02_as_authored_semicolon() {
    // `let x; { var x }` must reach the same Script EE-36-R02 propagation,
    // at the same authored subject position, as the authored-semicolon
    // form.
    assert_static_semantics_rejected("let x; { var x; }", "x", (13, 14));
    assert_static_semantics_rejected("let x; { var x }", "x", (13, 14));
}

#[test]
fn block_var_close_brace_asi_reaches_same_ee04_r08_as_authored_semicolon() {
    // Close-brace ASI must not convert the classification-only escaped
    // ReservedWord C6 route into an accepted IdentifierReference: it must
    // still reach the same EE-04-R08 rejection at the same authored subject
    // position as the authored-semicolon form.
    let semicolon_text = format!("{{ var x={}; }}", escaped_first_ascii_code_point("if"));
    assert_static_semantics_rejected(&semicolon_text, r"\u0069f", (8, 15));

    let close_brace_text = format!("{{ var x={} }}", escaped_first_ascii_code_point("if"));
    assert_static_semantics_rejected(&close_brace_text, r"\u0069f", (8, 15));
}

#[test]
fn block_var_close_brace_asi_eof_is_not_equivalent_to_close_brace() {
    // EOF must never be silently treated as this leaf's bounded close-brace
    // ASI route.
    assert!(matches!(
        attempt("{ var x"),
        SelectedQualificationAttempt::UnsupportedCoverage
    ));
}

#[test]
fn block_var_close_brace_asi_does_not_repair_incomplete_declarator_or_initializer() {
    for text in ["{ var x, }", "{ var x= }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_close_brace_asi_does_not_commit_tentative_c6_evidence_on_later_failure() {
    let text = format!("{{ var x={},y= }}", escaped_first_ascii_code_point("if"));
    assert!(
        matches!(
            attempt(&text),
            SelectedQualificationAttempt::UnsupportedCoverage
        ),
        "{text:?}"
    );
}

#[test]
fn block_var_close_brace_asi_does_not_widen_to_line_terminator_asi() {
    // General LineTerminator-triggered ASI before another statement remains
    // outside this leaf; only the bounded "next significant token == }"
    // route is recognized.
    for text in ["{\n  var x\n  let y;\n}", "{\n  var x\n  var y;\n}"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_close_brace_asi_does_not_widen_to_comments_or_initializer_family() {
    // "{ var a=true }" is deliberately not listed here: Issue #719 makes a
    // direct-authored `BooleanLiteral` initializer compose with close-brace
    // ASI as a selected accepted form (see the "Issue #719" test section
    // below). "{ var a=null }" is also deliberately not listed here: Issue
    // #721 makes a direct-authored `NullLiteral` initializer compose with
    // close-brace ASI as a selected accepted form (see the "Issue #721" test
    // section below). "{ var a=this }" is also deliberately not listed
    // here: Issue #723 makes a direct-authored `PrimaryExpression : this`
    // initializer compose with close-brace ASI as a selected accepted form
    // (see the "Issue #723" test section below). `{ var a="x" }` is also
    // deliberately not listed here: Issue #725 makes a direct-authored,
    // escape-free `StringLiteral` initializer compose with close-brace ASI
    // as a selected accepted form (see the "Issue #725" test section below).
    for text in ["{ var x /* c */ }", "{ var x=1 /* c */ }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

// --- Issue #719: direct BooleanLiteral initializers in both selected ------
// `var` placements
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5690396598 / #719 theorem: existing top-level and Block
// `var` terminator, transactionality, static-tier, and lifecycle authority
// must compose unchanged with the newly admitted direct `BooleanLiteral`
// initializer reused from #245/#247/#248.

#[test]
fn top_level_var_direct_boolean_literal_positive_lifecycle_is_selected_accepted_incomplete() {
    for text in [
        "var x=true;",
        "var x=false;",
        "var x=true",
        "var x=false",
        "var a=true,b=false;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_direct_boolean_literal_positive_lifecycle_is_selected_accepted_incomplete() {
    for text in [
        "{ var x=true; }",
        "{ var x=false; }",
        "{ var x=true }",
        "{ var x=false }",
        "{ var a=true,b=false; }",
        "{ var a=true,b=false }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_boolean_backed_declarator_reaches_existing_ee14_r02() {
    assert_static_semantics_rejected("{ let x; var x=true; }", "x", (13, 14));
}

#[test]
fn block_var_boolean_backed_declarator_reaches_existing_ee36_r02() {
    assert_static_semantics_rejected("let x; { var x=false; }", "x", (13, 14));
}

#[test]
fn top_level_var_escaped_boolean_like_reserved_spelling_preserves_c6_semantics() {
    // `\u0074rue` decodes to the ReservedWord `true`, so it must remain
    // the existing escaped-ReservedWord C6 route (EE-04-R08), never the direct
    // BooleanLiteral route added by this leaf.
    assert_static_semantics_rejected(r"var x=\u0074rue;", r"\u0074rue", (6, 15));
}

#[test]
fn block_var_escaped_boolean_like_reserved_spelling_preserves_c6_semantics() {
    assert_static_semantics_rejected(r"{ var x=\u0074rue; }", r"\u0074rue", (8, 17));
}

// --- Issue #721: direct NullLiteral initializers in both selected --------
// `var` placements
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5690791939 / #721 theorem: existing top-level and Block
// `var` terminator, transactionality, static-tier, and lifecycle authority
// must compose unchanged with the newly admitted direct `NullLiteral`
// initializer reused from #249/#250/#251/#252. The escaped-semantic-`null`
// C6 theorem is already sealed above (see
// `top_level_variable_statement_static_rejections_preserve_authored_primary_subject`,
// r"var x=n\u0075ll;") and is intentionally not duplicated here.

#[test]
fn top_level_var_direct_null_literal_positive_lifecycle_is_selected_accepted_incomplete() {
    for text in ["var x=null;", "var x=null", "var a=null,b=null;"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_direct_null_literal_positive_lifecycle_is_selected_accepted_incomplete() {
    for text in [
        "{ var x=null; }",
        "{ var x=null }",
        "{ var a=null,b=null; }",
        "{ var a=null,b=null }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_null_backed_declarator_reaches_existing_ee14_r02() {
    assert_static_semantics_rejected("{ let x; var x=null; }", "x", (13, 14));
}

#[test]
fn block_var_null_backed_declarator_reaches_existing_ee36_r02() {
    assert_static_semantics_rejected("let x; { var x=null; }", "x", (13, 14));
}

// --- Issue #723: direct `PrimaryExpression : this` initializers in --------
// both selected `var` placements
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5691238988 / #723 theorem: existing top-level and Block
// `var` terminator, transactionality, static-tier, and lifecycle authority
// must compose unchanged with the newly admitted direct `PrimaryExpression
// : this` initializer reused from #253/#254/#255/#256. Escaped semantic
// `this` remains the existing escaped-ReservedWord `EE-04-R08` C6 route,
// never this new direct route.

#[test]
fn top_level_var_direct_this_positive_lifecycle_is_selected_accepted_incomplete() {
    for text in ["var x=this;", "var x=this", "var a=this,b=this;"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_direct_this_positive_lifecycle_is_selected_accepted_incomplete() {
    for text in [
        "{ var x=this; }",
        "{ var x=this }",
        "{ var a=this,b=this; }",
        "{ var a=this,b=this }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_this_backed_declarator_reaches_existing_ee14_r02() {
    assert_static_semantics_rejected("{ let x; var x=this; }", "x", (13, 14));
}

#[test]
fn block_var_this_backed_declarator_reaches_existing_ee36_r02() {
    assert_static_semantics_rejected("let x; { var x=this; }", "x", (13, 14));
}

#[test]
fn top_level_var_escaped_this_like_reserved_spelling_preserves_c6_semantics() {
    // `\u0074his` decodes to the ReservedWord `this`, so it must remain the
    // existing escaped-ReservedWord C6 route (EE-04-R08), never the new
    // direct `this` route added by this leaf.
    assert_static_semantics_rejected(r"var x=\u0074his;", r"\u0074his", (6, 15));
}

#[test]
fn block_var_escaped_this_like_reserved_spelling_preserves_c6_semantics() {
    assert_static_semantics_rejected(r"{ var x=\u0074his; }", r"\u0074his", (8, 17));
}

// --- Issue #725: direct, escape-free `StringLiteral` initializers in ------
// both selected `var` placements
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5691565519 / #725 theorem: existing top-level and Block
// `var` terminator, transactionality, static-tier, and lifecycle authority
// must compose unchanged with the newly admitted direct, escape-free
// `StringLiteral` initializer reused unchanged from #257/#258/#259/#260.
// Unlike the preceding BooleanLiteral/NullLiteral/`this` keyword leaves, a
// quoted StringLiteral has no maximal-IdentifierName continuation boundary
// and no escaped-reserved-spelling C6 interaction, so no analogous test is
// required here.

#[test]
fn top_level_var_direct_string_literal_positive_lifecycle_is_selected_accepted_incomplete() {
    for text in [
        "var x=\"\";",
        "var x='';",
        "var x=\"abc\";",
        "var x=\"abc\"",
        "var a=\"x\",b='y';",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_direct_string_literal_positive_lifecycle_is_selected_accepted_incomplete() {
    for text in [
        "{ var x=\"\"; }",
        "{ var x=''; }",
        "{ var x=\"abc\"; }",
        "{ var x=\"abc\" }",
        "{ var a=\"x\",b='y'; }",
        "{ var a=\"x\",b='y' }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_string_literal_backed_declarator_reaches_existing_ee14_r02() {
    assert_static_semantics_rejected("{ let x; var x=\"s\"; }", "x", (13, 14));
}

#[test]
fn block_var_string_literal_backed_declarator_reaches_existing_ee36_r02() {
    assert_static_semantics_rejected("let x; { var x=\"s\"; }", "x", (13, 14));
}

// --- Issue #732: both selected `var` placements widened to a direct- ------
// authored plain fractional `DecimalLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// #688-comment-5698108598 joint-composition theorem: existing top-level and
// Block `var` static-tier and lifecycle authority must compose unchanged
// with the newly admitted direct-authored plain fractional `DecimalLiteral`
// initializer reused unchanged from #727/#728/#730/#731.

#[test]
fn top_level_var_direct_plain_fractional_decimal_literal_positive_lifecycle_is_selected_accepted_incomplete()
 {
    for text in [
        "var x=0.;",
        "var x=1.;",
        "var x=1.0;",
        "var x=12.34;",
        "var x=.0;",
        "var x=.5;",
        "var x=1.0",
        "var a=1.0,b=.5;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_direct_plain_fractional_decimal_literal_positive_lifecycle_is_selected_accepted_incomplete()
 {
    for text in [
        "{ var x=0.; }",
        "{ var x=1.; }",
        "{ var x=1.0; }",
        "{ var x=12.34; }",
        "{ var x=.0; }",
        "{ var x=.5; }",
        "{ var x=1.0 }",
        "{ var a=1.0,b=.5; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_plain_fractional_decimal_literal_backed_declarator_reaches_existing_ee14_r02() {
    assert_static_semantics_rejected("{ let x; var x=1.0; }", "x", (13, 14));
}

#[test]
fn block_var_plain_fractional_decimal_literal_backed_declarator_reaches_existing_ee36_r02() {
    assert_static_semantics_rejected("let x; { var x=.5; }", "x", (13, 14));
}

#[test]
fn top_level_var_plain_fractional_decimal_literal_backed_declarator_reaches_existing_ee36_r02() {
    assert_static_semantics_rejected("let x; var x=1.0;", "x", (11, 12));
}

#[test]
fn fractional_var_prefix_preserves_later_existing_grammar_evidence_in_both_owners() {
    // Issue #732: a valid newly accepted fractional initializer prefix must
    // not downgrade or suppress an already-owned later malformed
    // `BindingIdentifier` Grammar rejection, in either selected `var` owner.
    assert_grammar_rejected(r"var a=1.0,b,\u{}=2;", r"\u{}", (12, 16));

    assert_grammar_rejected(r"{ var a=1.0,b,\u{}=2; }", r"\u{}", (14, 18));
}

// --- Issue #740: both selected `var` placements widened to a direct- ------
// authored, separator-free exponent `DecimalLiteral` initializer
//
// These focused production tests seal the candidate against the accepted
// joint-composition theorem: existing top-level and Block `var` static-tier
// and lifecycle authority must compose unchanged with the newly admitted
// direct-authored, separator-free exponent `DecimalLiteral` initializer
// reused unchanged from #735/#736/#738/#739.

#[test]
fn top_level_var_direct_plain_exponent_decimal_literal_positive_lifecycle_is_selected_accepted_incomplete()
 {
    for text in [
        "var x=1e2;",
        "var x=1E2;",
        "var x=1e+2;",
        "var x=1e-2;",
        "var x=1.e2;",
        "var x=1.0e2;",
        "var x=.5E+2;",
        "var x=12.34E-56;",
        "var x=1e2",
        "var a=1e2,b=.5e2;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_direct_plain_exponent_decimal_literal_positive_lifecycle_is_selected_accepted_incomplete()
 {
    for text in [
        "{ var x=1e2; }",
        "{ var x=1E2; }",
        "{ var x=1e+2; }",
        "{ var x=1e-2; }",
        "{ var x=1.e2; }",
        "{ var x=1.0e2; }",
        "{ var x=.5E+2; }",
        "{ var x=12.34E-56; }",
        "{ var x=1e2 }",
        "{ var a=1e2,b=.5e2; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_plain_exponent_decimal_literal_backed_declarator_reaches_existing_ee14_r02() {
    assert_static_semantics_rejected("{ let x; var x=1e2; }", "x", (13, 14));
}

#[test]
fn block_var_plain_exponent_decimal_literal_backed_declarator_reaches_existing_ee36_r02() {
    assert_static_semantics_rejected("let x; { var x=.5e2; }", "x", (13, 14));
}

#[test]
fn top_level_var_plain_exponent_decimal_literal_backed_declarator_reaches_existing_ee36_r02() {
    assert_static_semantics_rejected("let x; var x=1e2;", "x", (11, 12));
}

#[test]
fn exponent_var_prefix_preserves_later_existing_grammar_evidence_in_both_owners() {
    // Issue #740: a valid newly accepted exponent initializer prefix must
    // not downgrade or suppress an already-owned later malformed
    // `BindingIdentifier` Grammar rejection, in either selected `var` owner.
    assert_grammar_rejected(r"var a=1e2,b,\u{}=2;", r"\u{}", (12, 16));

    assert_grammar_rejected(r"{ var a=1e2,b,\u{}=2; }", r"\u{}", (14, 18));
}

// --- Issue #744: all three selected initializer owners widened to a ------
// bounded, placement-neutral leading `+`/`-` decimal `UnaryExpression`
//
// These focused production tests seal the candidate against the accepted
// joint-composition theorem: existing static-tier, collision, and lifecycle
// authority must compose unchanged with the newly admitted leading `+`/`-`
// decimal `UnaryExpression` initializer, reused unchanged from #742/#743.

#[test]
fn top_level_var_direct_leading_plus_minus_decimal_unary_expression_positive_lifecycle_is_selected_accepted_incomplete()
 {
    for text in [
        "var x=-1;",
        "var x=+1;",
        "var x=-1e-2;",
        "var x=+1e+2;",
        "var x=-1",
        "var a=-1,b=+.5e2;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_direct_leading_plus_minus_decimal_unary_expression_positive_lifecycle_is_selected_accepted_incomplete()
 {
    for text in [
        "{ var x=-1; }",
        "{ var x=+1; }",
        "{ var x=-1e-2; }",
        "{ var x=+1e+2; }",
        "{ var x=-1 }",
        "{ var a=-1,b=+.5e2; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_leading_plus_minus_decimal_unary_expression_backed_declarator_reaches_existing_ee14_r02()
 {
    assert_static_semantics_rejected("{ let x; var x=-1; }", "x", (13, 14));
}

#[test]
fn block_var_leading_plus_minus_decimal_unary_expression_backed_declarator_reaches_existing_ee36_r02()
 {
    assert_static_semantics_rejected("let x; { var x=-1; }", "x", (13, 14));
}

#[test]
fn top_level_var_leading_plus_minus_decimal_unary_expression_backed_declarator_reaches_existing_ee36_r02()
 {
    assert_static_semantics_rejected("let x; var x=-1;", "x", (11, 12));
}

#[test]
fn leading_plus_minus_decimal_unary_expression_var_prefix_preserves_later_existing_grammar_evidence_in_both_owners()
 {
    // Issue #744: a valid newly accepted leading `+`/`-` decimal unary
    // initializer prefix must not downgrade or suppress an already-owned
    // later malformed `BindingIdentifier` Grammar rejection, in either
    // selected `var` owner.
    assert_grammar_rejected(r"var a=-1,b,\u{}=2;", r"\u{}", (11, 15));

    assert_grammar_rejected(r"{ var a=-1,b,\u{}=2; }", r"\u{}", (13, 17));
}

// --- Issue #758: top-level free-standing `IdentifierReference`
// `ExpressionStatement` use-site leaf. ---

#[test]
fn top_level_identifier_reference_expression_statement_use_site_remains_selected_accepted_incomplete()
 {
    for text in [
        "a;",
        "let;",
        "varfoo;",
        r"\u0061;",
        r"\u{61};",
        r"f\u006Fo;",
        "let a;\na;",
        "a;\nlet a;",
        "var a;\na;",
        "var a;\nvar a;\na;",
        "let a;\nlet x = a;\na;",
        "{ var a; }\na;",
        "{ let a; }\na;",
        "let a;\na;\na;",
        "let b;\nlet a;\na;\nb;",
        // Issue #768: EOF-only automatic termination reaches the same
        // qualification outcome as the authored-semicolon forms above.
        "a",
        "let",
        "varfoo",
        r"a",
        "let a;\na",
        "let b;\nlet a;\na;\nb",
        // Issue #771: production composes the already-accepted leading
        // `+`/`-` `IdentifierReference` `UnaryExpression` into this
        // free-standing body leaf, but this Oracle's own bounded theorem
        // (Issue #756) never qualifies a unary-wrapped operand, so these
        // reach the same `SelectedAcceptedIncomplete` outcome as any other
        // production-accepted, not-yet-Oracle-qualified source -- never
        // `UnsupportedCoverage` and never `Qualified`.
        "+a;",
        "-a;",
        "+\\u0061;",
        "-f\\u006Fo;",
        "+a",
        "-a",
        // Issue #773: production further composes that same leading `+`/`-`
        // `IdentifierReference` `UnaryExpression` with an optional exactly-
        // one binary `+`/`-` plain `IdentifierReference` continuation, but
        // this Oracle's own bounded theorem never qualifies a unary-wrapped
        // left operand, so these also reach `SelectedAcceptedIncomplete`.
        "+a+b;",
        "+a-b;",
        "-a+b;",
        "-a-b;",
        "+\\u0061+b;",
        "-f\\u006Fo+bar;",
        "+a+b",
        "-a-b",
        // Issue #787 widens production recognition with the both-unary
        // right operand while preserving the existing qualification
        // lifecycle; these complete selected sources therefore remain
        // `SelectedAcceptedIncomplete`.
        "+a+-b;",
        "+a-+b;",
        "-a+-b;",
        "-a-+b;",
        "+a+ +b;",
        "-a- -b;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn top_level_identifier_reference_expression_statement_use_site_dispatch_reaches_accepted_incomplete_while_declaration_grammar_stays_owned()
 {
    // `let;` / `varfoo;` must reach `SelectedAcceptedIncomplete` through the
    // new use-site leaf, while `let a;` / `var a;` remain owned by the
    // existing `LexicalDeclaration` / `VariableStatement` grammar (proved
    // directly at the carrier/item level by
    // `dispatch_selects_use_site_before_raw_top_level_dispatch` in
    // `selected_lexical_slice_tests.rs`; this seals the same discriminator
    // at the qualification entrypoint).
    for text in ["let;", "varfoo;", "let a;", "var a;"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn top_level_identifier_reference_expression_statement_use_site_static_rejections_preserve_authored_primary_subject()
 {
    for (text, expected_fragment, expected_range) in [
        ("let a;\nlet a;\na;", "a", (11, 12)),
        ("let a;\nvar a;\na;", "a", (11, 12)),
        ("var a;\nlet a;\na;", "a", (11, 12)),
        ("let a;\n{ var a; }\na;", "a", (13, 14)),
    ] {
        assert_static_semantics_rejected(text, expected_fragment, expected_range);
    }
}

#[test]
fn top_level_identifier_reference_expression_statement_use_site_asi_general_expression_and_nested_boundaries_remain_unsupported()
 {
    for text in [
        // ASI boundaries (acceptance criterion 4 / issue section 47). Bare
        // `a` moved to selected-positive TopLevel EOF-automatic-termination
        // coverage by Issue #768 (see the "Issue #768" section below) and is
        // no longer listed here; general `LineTerminator` ASI remains
        // explicitly out of scope, so `a\nlet b;` stays unsupported.
        "a\nlet b;",
        // General-expression firewall (acceptance criterion 31 / W15).
        // `a+b;` moved to selected-positive coverage by Issue #766 (see the
        // "Issue #766" section below) and `+a;` / `-a;` moved to
        // selected-positive (`SelectedAcceptedIncomplete`) coverage by Issue
        // #771 (see the "Issue #771" section above); neither is listed here
        // any longer.
        "(a);",
        "a.b;",
        "a[b];",
        "a();",
        "a=b;",
        // Richer-expression firewall composed with the unary body form
        // (Issue #771): a locally recognized `+a`/`-a` prefix must not
        // authorize a richer neighbor. `+a+b;` / `-a-b;` moved to
        // selected-positive coverage by Issue #773 (see the "Issue #773"
        // section below); a third-or-later additive operand remains outside
        // this leaf.
        "+a+b+c;",
        "-a-b-c;",
        "+a.b;",
        "-a[b];",
        "+a();",
        "+a=b;",
        // Issue #787: a right-unary-wrapped second operand still admits no
        // third-or-later additive operand.
        "+a+-b+c;",
        "-a-+b-c;",
        // Other unary operators remain outside this leaf (Issue #771).
        "!a;",
        "~a;",
        "typeof a;",
        "++a;",
        "+-a;",
        // Parenthesized firewall (Issue #771).
        "+(a);",
        "-(a);",
        // Deeper-nesting firewall (acceptance criterion 15). `{ a; }` itself
        // is now accepted as a Block-contained use-site by Issue #762 (see
        // the "Issue #762" section below); only recursion beyond one level
        // remains outside selected coverage.
        "{ { a; } }",
        // Escaped-reserved / malformed boundaries (issue section 50).
        r"\u0069f;",
        r"\u{};",
        r"\u0030;",
        r"a\u002Db;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn top_level_identifier_reference_expression_statement_use_site_whole_source_transactionality() {
    for text in ["let a;\na;\n???", "let a;\na;\nlet x = ;"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

// --- Issue #762: one-level Block-contained free-standing
// `IdentifierReference` `ExpressionStatement` use-site leaf. ---

#[test]
fn block_identifier_reference_expression_statement_use_site_remains_selected_accepted_incomplete() {
    for text in [
        "{ a; }",
        r"{ \u0061; }",
        r"{ \u{61}; }",
        r"{ f\u006Fo; }",
        "{ let; }",
        "{ varfoo; }",
        "{ let a; }",
        "{ var a; }",
        "{ let a; a; }",
        "let a;\n{ a; }",
        "let a;\n{ a; let a; }",
        "{ let a; }\n{ a; }",
        "{ a; }\nlet a;",
        "var a;\n{ a; }",
        "{ var a; a; }",
        "{ a; }\n{ var a; }",
        "{ a; a; }",
        "{ a; }\n{ a; }",
        "a;\n{ a; }\na;",
        // Issue #768: before-close automatic termination reaches the same
        // qualification outcome as the authored-semicolon forms above.
        "{ a }",
        r"{ a }",
        "{ let }",
        "{ varfoo }",
        "let a;\n{ a }",
        "{ a }\nlet a;",
        "a;\n{ a }\na;",
        // Issue #771: the same composed leading `+`/`-` unary body form
        // reaches `SelectedAcceptedIncomplete` for the Block placement too.
        "{ +a; }",
        "{ -a; }",
        "{ +\\u0061; }",
        "{ -f\\u006Fo; }",
        "{ +a }",
        "{ -a }",
        // Issue #773: the same composed leading `+`/`-` unary-additive body
        // form reaches `SelectedAcceptedIncomplete` for the Block placement
        // too.
        "{ +a+b; }",
        "{ -a-b; }",
        "{ +\\u0061-b }",
        "{ -f\\u006Fo+\\u0062 }",
        // Issue #787: the same right-unary-wrapped second operand reaches
        // `SelectedAcceptedIncomplete` for the Block placement too.
        "{ +a+-b; }",
        "{ -a-+b; }",
        "{ +a+ +b }",
        "{ -a- -b }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_identifier_reference_expression_statement_use_site_dispatch_reaches_accepted_incomplete_while_declaration_grammar_stays_owned()
 {
    // `{ let; }` / `{ varfoo; }` must reach `SelectedAcceptedIncomplete`
    // through the new Block use-site leaf, while `{ let a; }` / `{ var a; }`
    // remain owned by the existing lexical-declaration / Block-var grammar
    // (proved directly at the carrier/item level by
    // `block_dispatch_selects_use_site_before_raw_block_dispatch` in
    // `selected_lexical_slice_tests.rs`; this seals the same discriminator
    // at the qualification entrypoint).
    for text in ["{ let; }", "{ varfoo; }", "{ let a; }", "{ var a; }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_identifier_reference_expression_statement_use_site_static_rejections_preserve_authored_primary_subject()
 {
    for (text, expected_fragment, expected_range) in [
        ("let a;\n{ var a; a; }", "a", (13, 14)),
        ("{ let a; var a; a; }", "a", (13, 14)),
        ("{ let a; let a; a; }", "a", (13, 14)),
        ("let a;\nlet a;\n{ a; }", "a", (11, 12)),
    ] {
        assert_static_semantics_rejected(text, expected_fragment, expected_range);
    }
}

#[test]
fn block_identifier_reference_expression_statement_use_site_asi_general_expression_and_nesting_boundaries_remain_unsupported()
 {
    for text in [
        // ASI boundaries (issue section "ASI boundary"). `{ a }` moved to
        // selected-positive Block automatic-before-close-brace termination
        // coverage by Issue #768 (see the "Issue #768" section below) and is
        // no longer listed here.
        // General-expression firewall (issue section "General-expression
        // firewall" / W15). `{ a+b; }` moved to selected-positive coverage
        // by Issue #766 (see the "Issue #766" section below) and `{ +a; }` /
        // `{ -a; }` moved to selected-positive (`SelectedAcceptedIncomplete`)
        // coverage by Issue #771 (see the "Issue #771" section above);
        // neither is listed here any longer.
        "{ (a); }",
        "{ a.b; }",
        "{ a[b]; }",
        "{ a(); }",
        "{ a=b; }",
        "{ a ? b : c; }",
        "{ a && b; }",
        "{ a, b; }",
        "{ new a; }",
        // Richer-expression firewall composed with the unary body form
        // (Issue #771). `{ +a+b; }` / `{ -a-b; }` moved to selected-positive
        // coverage by Issue #773 (see the "Issue #773" section below); a
        // third-or-later additive operand remains outside this leaf.
        "{ +a+b+c; }",
        "{ -a-b-c; }",
        "{ +a.b; }",
        "{ -a[b]; }",
        // Issue #787: a right-unary-wrapped second operand still admits no
        // third-or-later additive operand, for the Block placement too.
        "{ +a+-b+c; }",
        "{ -a-+b-c; }",
        // Other unary operators and parenthesized forms remain outside this
        // leaf (Issue #771).
        "{ !a; }",
        "{ ~a; }",
        "{ +(a); }",
        "{ -(a); }",
        // Empty-Block and deeper-nesting firewalls (issue section "Empty
        // Block / recursive Block boundaries" / W26/W27).
        "{}",
        "{ { a; } }",
        // Escaped-reserved / malformed boundaries.
        r"{ \u0069f; }",
        r"{ \u{}; }",
        r"{ \u0030; }",
        r"{ a\u002Db; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_identifier_reference_expression_statement_use_site_whole_source_transactionality() {
    for text in [
        "{ a; ??? }",
        "{ a; let x = ; }",
        "{ a; }\n???",
        "{ a; }\nlet x = ;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn block_var_close_brace_asi_regression_remains_selected_accepted_incomplete_after_block_parser_refactor()
 {
    // Because the Block parser architecture changes for Issue #762, this
    // seals that existing Block-var `AutomaticBeforeBlockClose` support
    // (Issue #717) is unaffected: this source has no Block-contained
    // use-site, so it must remain owned by the historical `SelectedBlock`
    // route, not the new fifth carrier.
    for text in ["{ var a }", "{ var a, b }", "{ let a; var b }"] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

// --- Issue #766: exactly-two IdentifierReference additive free-standing
// `ExpressionStatement` use-sites. `a+b;` / `{ a+b; }` move from the old
// `UnsupportedCoverage` firewall lists into selected-positive coverage;
// every neighboring firewall source remains unsupported. ---

#[test]
fn two_operand_identifier_reference_expression_statement_use_site_remains_selected_accepted_incomplete()
 {
    for text in [
        "a+b;",
        "a-b;",
        "a+-b;",
        "a-+b;",
        "a+ +b;",
        "a- -b;",
        "a + b;",
        "\\u0061+b;",
        "a+\\u0062;",
        "\\u0061-\\u0062;",
        "f\\u006Fo-b\\u0061r;",
        "{ a+b; }",
        "{ \\u0061-b; }",
        "a;\nb+c;\nd;",
        "let a;\nvar b;\na+b;",
        "let b;\n{ let a;\na+b; }",
        "a+b;\nlet a;\nlet b;",
        "let b;\n{ a+b;\nlet a; }",
        "a+a;",
        "{ a+a; }",
        "a+b;\nc-d;",
        "a+b;\n{ c-d; }\ne+f;",
        "let x=a+b;\nc+d;",
        // Issue #768: EOF-only / before-close automatic termination reaches
        // the same qualification outcome as the authored-semicolon forms
        // above.
        "a+b",
        "a-b",
        "{ a+b }",
        "let a;\nvar b;\na+b",
        "let b;\n{ a+b }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

// --- Issue #799 (per #688 comment 5775218176): composes the
// candidate-independent exactly-three ordered `IdentifierReference` additive
// theorem accepted by #795/PR #796 into free-standing production. `a+b+c;`
// and its neighbors move from the old `UnsupportedCoverage` firewall list
// into selected-positive coverage; every newly selected complete source
// reaches exactly the existing `SelectedAcceptedIncomplete` lifecycle, never
// a new qualification branch or evidence family. ---

#[test]
fn three_operand_identifier_reference_expression_statement_use_site_remains_selected_accepted_incomplete()
 {
    for text in [
        // All four operator-pair combinations, TopLevel authored-semicolon.
        "a+b+c;",
        "a-b-c;",
        "a+b-c;",
        "a-b+c;",
        // Direct / escaped provenance, all three positions.
        "\\u0061+b+c;",
        "a+\\u0062+c;",
        "a+b+\\u0063;",
        // One-level Block, authored semicolon.
        "{ a+b+c; }",
        "{ a-b-c; }",
        // Automatic termination: TopLevel EOF, Block before-close.
        "a+b+c",
        "{ a+b+c }",
        // Duplicate operands.
        "a+a+a;",
        // Mixed one/two/three-operand item-order composition.
        "a;\nb+c;\nd+e+f;\ng;",
        // Correspondence composition (per-operand independent resolution).
        "let c;\nlet a;\nvar b;\na+b+c;",
        "let c;\nvar b;\n{ let a;\na+b+c; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn two_operand_cardinality_operand_and_richer_expression_firewalls_remain_unsupported() {
    for text in [
        // Cardinality firewall: exactly three plain operands (`a+b+c;`,
        // `a-b-c;`, `a+b-c;`, `a-b+c;`) migrated to selected-positive
        // coverage by Issue #799 (see
        // `three_operand_identifier_reference_expression_statement_use_site_remains_selected_accepted_incomplete`);
        // four-or-more plain operands (`a+b+c+d;`) migrated to
        // selected-positive coverage by Issue #807 (see
        // `many_operand_identifier_reference_expression_statement_use_site_remains_selected_accepted_incomplete`);
        // there is no remaining plain-cardinality firewall.
        // Operand firewall (W25). A leading `+`/`-` *left* operand (`+a+b;`,
        // `-a+b;`) moved to selected-positive coverage by Issue #773 (see
        // the "Issue #773" section below); other unary operators/recursion
        // on the left operand remain outside this leaf.
        "++a+b;",
        "--a+b;",
        "!a+b;",
        "a++b;",
        "true+b;",
        "a+null;",
        "this+b;",
        "\"a\"+b;",
        "(a)+b;",
        "a+(b);",
        "a.b+c;",
        "a+b.c;",
        "a()+b;",
        "a+b();",
        // Precedence / richer-expression firewall.
        "a*b;",
        "a+b*c;",
        "a*b+c;",
        "a**b+c;",
        "a+b**c;",
        "a=b+c;",
        "a+=b;",
        "a?b:c;",
        "a||b;",
        "a&&b;",
        "a??b;",
        "a,b;",
        // Issue #781: recognizing the right-unary body does not widen
        // general LineTerminator-triggered ASI.
        "a+-b\nlet c;",
        // Escaped-ReservedWord boundary (W26).
        "\\u0069f+b;",
        "a+\\u0069f;",
        // ASI / recursive-Block boundary (W27/W28). `a+b` and `{ a+b }`
        // moved to selected-positive EOF/before-close automatic-termination
        // coverage by Issue #768 (see the "Issue #768" section below) and
        // are no longer listed here; recursive Block topology remains
        // unwidened.
        "{ { a+b; } }",
        "{ { a+-b; } }",
        // Selected trivia remains comment-free at the binary/right-unary
        // punctuator boundary.
        "a+/*comment*/+b;",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn two_operand_use_site_whole_source_transactionality() {
    for text in [
        "a+b;\n???",
        "a+b;\nlet x = ;",
        "{ a+b; ??? }",
        "{ a+b; }\n???",
        "a+-b;\n???",
        "a+-b;\nlet x = ;",
        "{ a+-b; ??? }",
        "{ a+-b; }\n???",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn two_operand_use_site_known_static_rejection_gates_qualification() {
    assert_static_semantics_rejected("let a;\n{ var a; b+-c; }", "a", (13, 14));
}

/// Issue #797 (per #688 comment 5771773635) section 22/43: a source
/// containing a valid `Three` initializer still reaches an existing static
/// rejection unchanged when another existing obligation fails -- no new
/// rejection type and no changed rejection ordering. Reuses the smallest
/// existing static conflict already proven above (a Block `var` duplicating
/// an enclosing top-level lexical name), with the free-standing use-site
/// replaced by a complete three-operand initializer.
#[test]
fn three_identifier_reference_additive_initializer_known_static_rejection_gates_qualification() {
    assert_static_semantics_rejected("let a;\n{ var a; const x=a+b+c; }", "a", (13, 14));
}

/// Issue #803 (per #688 comment 5779735385) section 22/43: a source
/// containing a valid `Many` initializer still reaches an existing static
/// rejection unchanged when another existing obligation fails -- no new
/// rejection type and no changed rejection ordering. Reuses the smallest
/// existing static conflict already proven above (a Block `var` duplicating
/// an enclosing top-level lexical name), with the free-standing use-site
/// replaced by a complete four-operand initializer.
#[test]
fn many_identifier_reference_additive_initializer_known_static_rejection_gates_qualification() {
    assert_static_semantics_rejected("let a;\n{ var a; const x=a+b+c+d; }", "a", (13, 14));
}

/// Issue #799 (per #688 comment 5775218176) section 42: a source containing
/// a valid free-standing `Three` use-site still reaches an existing static
/// rejection unchanged when another existing obligation fails -- no new
/// rejection type and no changed rejection ordering. Reuses the smallest
/// existing static conflict already proven above (a Block `var` duplicating
/// an enclosing top-level lexical name), with the initializer replaced by a
/// complete three-operand free-standing use-site.
#[test]
fn three_operand_use_site_known_static_rejection_gates_qualification() {
    assert_static_semantics_rejected("let a;\n{ var a; b+c+d; }", "a", (13, 14));
}

// --- Issue #807 (per #688 comment 5780964757): composes the
// candidate-independent ordered 2..N `IdentifierReference` additive-chain
// theorem accepted by #801/PR #802 into free-standing production. `a+b+c+d;`
// and its neighbors move from the firewall list above into selected-positive
// coverage; every newly selected complete source reaches exactly the
// existing `SelectedAcceptedIncomplete` lifecycle, never a new qualification
// branch or evidence family. ---

#[test]
fn many_operand_identifier_reference_expression_statement_use_site_remains_selected_accepted_incomplete()
 {
    for text in [
        // Four, five, and eight operands, mixed operators, TopLevel
        // authored-semicolon.
        "a+b+c+d;",
        "a-b+c-d;",
        "a+b+c+d+e;",
        "a+b-c+d-e+f+g+h;",
        // Direct / escaped provenance: fourth and later operand.
        "a+b+c+\\u0064;",
        "a+b+c+d+\\u0065+f+g+h;",
        // One-level Block, authored semicolon.
        "{ a+b+c+d; }",
        "{ a-b+c-d+e; }",
        // Automatic termination: TopLevel EOF, Block before-close.
        "a+b+c+d",
        "{ a+b+c+d }",
        // Duplicate operands.
        "a+a+a+a;",
        // Mixed one/two/three/many-operand item-order composition.
        "a;\nb+c;\nd+e+f;\ng+h+i+j;\nk;",
        // Correspondence composition (per-operand independent resolution).
        "let d;\nlet a;\nvar c;\nlet b;\na+b+c+d;",
        "let d;\nvar c;\nlet e;\n{ let a;\nlet b;\na+b+c+d+e; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

/// Issue #807 (per #688 comment 5780964757) section 42: a source containing
/// a valid free-standing `Many` use-site still reaches an existing static
/// rejection unchanged when another existing obligation fails -- no new
/// rejection type and no changed rejection ordering. Reuses the smallest
/// existing static conflict already proven above (a Block `var` duplicating
/// an enclosing top-level lexical name), with the free-standing use-site
/// replaced by a complete four-operand free-standing use-site.
#[test]
fn many_operand_use_site_known_static_rejection_gates_qualification() {
    assert_static_semantics_rejected("let a;\n{ var a; b+c+d+e; }", "a", (13, 14));
}

// --- Issue #793 (per #688 comment 5764090454): composes the
// candidate-independent one-reference / one-plain-decimal heterogeneous
// additive theorem accepted by #789/#790 into free-standing production,
// reusing the existing `SelectedFreeStandingIdentifierReferenceUseSite::One`
// carrier. `a+1;` and `1+b;` move from the firewall list above into
// selected-positive coverage; qualification lifecycle for every newly
// selected complete source remains exactly `SelectedAcceptedIncomplete`. ---

#[test]
fn one_reference_one_plain_decimal_additive_use_site_remains_selected_accepted_incomplete() {
    for text in [
        // Reference-left, both operators, all three plain Decimal atom
        // families, TopLevel authored-semicolon.
        "a + 1;",
        "a - 1.0;",
        "a + .5;",
        "a - 1e-2;",
        // Decimal-left, both operators, all three plain Decimal atom
        // families, TopLevel authored-semicolon.
        "1 + a;",
        "1.0 - a;",
        ".5 + a;",
        "1e-2 - a;",
        // Direct / escaped provenance, both orientations.
        "\\u0061 + 1;",
        "1 + \\u0061;",
        "f\\u006Fo - 1e2;",
        ".5 + b\\u0061r;",
        // One-level Block, authored semicolon.
        "{ a + 1; }",
        "{ 1 + a; }",
        // Automatic termination: TopLevel EOF, Block before-close.
        "a + 1",
        "1 + a",
        "{ a + 1 }",
        "{ 1 + a }",
        // Selected-trivia LineTerminator continuation.
        "a\n+ 1;",
        "1\n- a;",
        // Correspondence sealing sources (Block-contained lexical + use-site).
        "let a;\na + 1;",
        "let a;\n1 + a;",
        "let a;\n{ 1 + a; }",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::SelectedAcceptedIncomplete
            ),
            "{text:?}"
        );
    }
}

#[test]
fn one_reference_one_plain_decimal_additive_use_site_firewalls_remain_unsupported() {
    for text in [
        // Right-unary firewall (reference-left): a consumed right-unary
        // wrapper forecloses the Decimal fallback unconditionally.
        "a+-1;",
        "a-+1;",
        "a+ +1;",
        "a- -1;",
        // Leading-unary branch hard-zero.
        "+a + 1;",
        "-a + 1;",
        // Decimal-left unary firewall.
        "1 + +a;",
        "1 + -a;",
        "+1 + a;",
        "-1 + a;",
        // Body-level incomplete continuation.
        "a + ;",
        "1 + ;",
        "a + true;",
        "1 + true;",
        // Cardinality / richer-tail rollback.
        "a + 1 + b;",
        "1 + a + 2;",
        // Zero-reference firewall.
        "1;",
        "1 + 2;",
        // Richer-expression / grouping / member / call.
        "(a) + 1;",
        "1 + (a);",
        "a.b + 1;",
        "1 + a.b;",
        "a() + 1;",
        "1 + a();",
        "a + 1 * b;",
        "a = 1 + b;",
        // Other operand families.
        "a + true;",
        "true + a;",
        "a + null;",
        "null + a;",
        "a + this;",
        "this + a;",
        "a + \"x\";",
        "\"x\" + a;",
        // Numeric frontier.
        "a + 1_0;",
        "1_0 + a;",
        "a + 0x10;",
        "0x10 + a;",
        "a + 1n;",
        "1n + a;",
        // Comments / general ASI / recursive Block.
        "a/*c*/+1;",
        "1+/*c*/a;",
        "a + 1\nlet b;",
        "{ { a + 1; } }",
        // Escaped ReservedWord / malformed.
        "\\u0069f + 1;",
        "1 + \\u0069f;",
        "1 + \\u{};",
    ] {
        assert!(
            matches!(
                attempt(text),
                SelectedQualificationAttempt::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}
