use crate::{SourceId, SourceText};

use super::first_qualification_integration::{
    FirstSelectedQualificationAttempt, attempt_first_selected_qualification,
};
use super::qualification::{ProcessingStatus, QualificationVerdictKind, RejectionFamily};
use super::qualification_validation_tests::{gold_source, gold_subject_range};

fn attempt(text: &str) -> FirstSelectedQualificationAttempt {
    let source = SourceText::new(SourceId::new(213), text.to_owned());
    attempt_first_selected_qualification(&source)
}

#[test]
fn independently_qualified_positive_gold_remains_selected_incomplete() {
    for fixture_id in ["JS-GOLD-SCRIPT-VALID-001", "JS-GOLD-SCRIPT-MULTIBYTE-001"] {
        let text = gold_source(fixture_id).unwrap_or_else(|| panic!("{fixture_id} must exist"));
        assert!(matches!(
            attempt(text),
            FirstSelectedQualificationAttempt::SelectedAcceptedIncomplete
        ));
    }
}

#[test]
fn selected_non_strict_and_canonical_distinct_cases_remain_selected_incomplete() {
    for text in [
        "let await=1;",
        "let yield=1;",
        "let static=1;",
        "let implements=1;",
        "let arguments=1;",
        "let eval=1;",
        "let é; let e\u{301};",
    ] {
        assert!(matches!(
            attempt(text),
            FirstSelectedQualificationAttempt::SelectedAcceptedIncomplete
        ));
    }
}

#[test]
fn let_binding_rejection_preserves_independent_gold_provenance() {
    let fixture_id = "JS-GOLD-LEXDECL-LET-BINDING-001";
    let text = gold_source(fixture_id).expect("let-binding rejection gold must exist");
    let expected = gold_subject_range(fixture_id).expect("let-binding gold must retain a subject");

    let outcome = match attempt(text) {
        FirstSelectedQualificationAttempt::Outcome(outcome) => outcome,
        other => panic!("expected qualification outcome, got {other:?}"),
    };

    assert_eq!(outcome.processing(), ProcessingStatus::Complete);
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::StaticSemanticsRejected)
    );
    let evidence = outcome
        .rejection_evidence()
        .expect("static rejection must retain evidence");
    assert_eq!(evidence.family(), RejectionFamily::StaticSemantics);
    let anchor = evidence
        .subject()
        .authored_anchor()
        .expect("selected rejection evidence must remain authored");
    assert_eq!(anchor.fragment(), "let");
    assert_eq!(anchor.range().start(), expected.0);
    assert_eq!(anchor.range().end(), expected.1);
}

#[test]
fn duplicate_rejection_preserves_independent_gold_primary_subject() {
    let fixture_id = "JS-GOLD-SCRIPT-DUPLEXICAL-001";
    let text = gold_source(fixture_id).expect("duplicate rejection gold must exist");
    let expected = gold_subject_range(fixture_id).expect("duplicate gold must retain a subject");

    let outcome = match attempt(text) {
        FirstSelectedQualificationAttempt::Outcome(outcome) => outcome,
        other => panic!("expected qualification outcome, got {other:?}"),
    };

    assert_eq!(outcome.processing(), ProcessingStatus::Complete);
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::StaticSemanticsRejected)
    );
    let anchor = outcome
        .rejection_evidence()
        .and_then(|evidence| evidence.subject().authored_anchor())
        .expect("duplicate rejection must retain authored primary evidence");
    assert_eq!(anchor.fragment(), "x");
    assert_eq!(anchor.range().start(), expected.0);
    assert_eq!(anchor.range().end(), expected.1);
}

#[test]
fn asi_gold_remains_unsupported_coverage_without_a_verdict() {
    let text = gold_source("JS-GOLD-ASI-NO-FABRICATED-RANGE-001")
        .expect("ASI staged-coverage gold must exist");
    assert!(matches!(
        attempt(text),
        FirstSelectedQualificationAttempt::UnsupportedCoverage
    ));
}

#[test]
fn integration_source_preserves_aggregate_architecture_boundaries() {
    let production = include_str!("first_qualification_integration.rs");

    for forbidden in [
        concat!("QualificationOutcome", "::qualified"),
        concat!("CompleteQualification", "Witness"),
        concat!("source.", "as_str()"),
        concat!("source.", "anchor("),
        concat!("unicode_", "normalization"),
    ] {
        assert!(
            !production.contains(forbidden),
            "selected aggregate integration must preserve its architecture boundary: found {forbidden}"
        );
    }

    assert_eq!(
        production
            .matches("recognize_first_lexical_slice(source)")
            .count(),
        1,
        "aggregate integration must invoke the selected recognizer exactly once"
    );
    assert_eq!(
        production
            .matches("evaluate_first_static_semantics(&script)")
            .count(),
        1,
        "recognized selected facts must enter the selected static-semantics owner exactly once"
    );
}
