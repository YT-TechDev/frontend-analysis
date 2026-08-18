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

fn qualification_outcome(text: &str) -> super::qualification::QualificationOutcome {
    match attempt(text) {
        SelectedQualificationAttempt::Outcome(outcome) => outcome,
        other => panic!("expected qualification outcome for {text:?}, got {other:?}"),
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
        r"const x=\u0066oo;",
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
    assert!(production.contains("SelectedAcceptedIncomplete"));
    assert!(production.contains("EvidenceSubject::authored(source, subject)"));
    assert!(production.contains("QualificationOutcome::syntax_rejected(subject)"));
}
