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
        r"const x = \u0066oo + 1;",
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
        "const x=foo + 1;",
        "const x=(foo);",
        "const x=foo = bar;",
        "const x=foo ? bar : baz;",
        "const x=foo/*comment*/;",
        "const x=foo unexpected;",
        "const x=/a/;",
        "var x=1;",
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
        r"let\u0030;",
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
        assert_eq!(evidence.family(), RejectionFamily::StaticSemantics, "{text}");
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
        "{ var a=1; }",
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
