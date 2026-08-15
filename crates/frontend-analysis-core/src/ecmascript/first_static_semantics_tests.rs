use crate::{SourceId, SourceText};

use super::first_lexical_slice::{
    FirstLexicalSliceOutcome, SelectedLexicalScript, recognize_first_lexical_slice,
};
use super::first_static_semantics::{
    SelectedStaticSemanticsOutcome, SelectedStaticSemanticsRejection,
    evaluate_first_static_semantics, selected_rejection_to_qualification,
};
use super::qualification::{
    ProcessingStatus, QualificationVerdictKind, RejectionFamily,
};
use super::qualification_validation_tests::{gold_source, gold_subject_range};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(210), text.to_owned())
}

fn recognized(text: &str) -> (SourceText, SelectedLexicalScript) {
    let source = source(text);
    let script = match recognize_first_lexical_slice(&source) {
        FirstLexicalSliceOutcome::RecognizedSelectedSlice(script) => script,
        FirstLexicalSliceOutcome::UnsupportedCoverage => {
            panic!("expected selected-slice recognition, got unsupported coverage for {text:?}")
        }
        FirstLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence => {
            panic!("unexpected grammar rejection for selected static-semantics input {text:?}")
        }
        FirstLexicalSliceOutcome::ResourceLimited => {
            panic!("unexpected lexical resource limitation for {text:?}")
        }
        FirstLexicalSliceOutcome::InternalFailure => {
            panic!("unexpected lexical internal failure for {text:?}")
        }
    };
    (source, script)
}

fn accepted(text: &str) {
    let (_, script) = recognized(text);
    assert!(matches!(
        evaluate_first_static_semantics(&script),
        SelectedStaticSemanticsOutcome::Accepted
    ));
}

#[test]
fn ee_15_r01_rejects_binding_named_let_with_independent_gold_subject() {
    let text = gold_source("JS-GOLD-LEXDECL-LET-BINDING-001")
        .expect("EE-15-R01 independent gold must exist");
    let expected = gold_subject_range("JS-GOLD-LEXDECL-LET-BINDING-001")
        .expect("EE-15-R01 independent gold must retain a subject");
    let (_, script) = recognized(text);

    match evaluate_first_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::BindingNamedLet { binding },
        ) => {
            assert_eq!(binding.fragment(), "let");
            assert_eq!(binding.range().start(), expected.0);
            assert_eq!(binding.range().end(), expected.1);
        }
        other => panic!("expected binding-named-let rejection, got {other:?}"),
    }
}

#[test]
fn ee_36_r01_preserves_first_and_duplicate_occurrences_from_independent_gold() {
    let text = gold_source("JS-GOLD-SCRIPT-DUPLEXICAL-001")
        .expect("duplicate Script independent gold must exist");
    let expected = gold_subject_range("JS-GOLD-SCRIPT-DUPLEXICAL-001")
        .expect("duplicate Script independent gold must retain a subject");
    let (_, script) = recognized(text);

    match evaluate_first_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateLexicalName {
                first_binding,
                duplicate_binding,
            },
        ) => {
            assert_eq!(first_binding.fragment(), "x");
            assert_eq!(first_binding.range().start(), 4);
            assert_eq!(first_binding.range().end(), 5);
            assert_eq!(duplicate_binding.fragment(), "x");
            assert_eq!(duplicate_binding.range().start(), expected.0);
            assert_eq!(duplicate_binding.range().end(), expected.1);
        }
        other => panic!("expected duplicate lexical-name rejection, got {other:?}"),
    }
}

#[test]
fn canonically_equivalent_unescaped_spellings_remain_distinct() {
    accepted("let é; let e\u{301};");
}

#[test]
fn selected_non_strict_identifier_boundary_remains_non_triggering() {
    for text in [
        "let await=1;",
        "let yield=1;",
        "let static=1;",
        "let implements=1;",
        "let arguments=1;",
        "let eval=1;",
    ] {
        accepted(text);
    }
}

#[test]
fn declaration_local_rejection_precedes_script_duplicate_evidence_deterministically() {
    let (_, script) = recognized("let x; let x; let let;");

    match evaluate_first_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::BindingNamedLet { binding },
        ) => {
            assert_eq!(binding.fragment(), "let");
            assert_eq!(binding.range().start(), 18);
            assert_eq!(binding.range().end(), 21);
        }
        other => panic!("expected declaration-local rejection precedence, got {other:?}"),
    }
}

#[test]
fn duplicate_selection_is_first_repeat_in_source_order_and_repeatable() {
    fn duplicate_ranges(text: &str) -> ((usize, usize), (usize, usize)) {
        let (_, script) = recognized(text);
        match evaluate_first_static_semantics(&script) {
            SelectedStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateLexicalName {
                    first_binding,
                    duplicate_binding,
                },
            ) => (
                (first_binding.range().start(), first_binding.range().end()),
                (
                    duplicate_binding.range().start(),
                    duplicate_binding.range().end(),
                ),
            ),
            other => panic!("expected duplicate lexical-name rejection, got {other:?}"),
        }
    }

    let first = duplicate_ranges("let x; let y; let x; let x;");
    let second = duplicate_ranges("let x; let y; let x; let x;");
    assert_eq!(first, ((4, 5), (18, 19)));
    assert_eq!(first, second);
}

#[test]
fn proven_rejection_maps_to_199_static_semantics_rejected() {
    let (source, script) = recognized("let let;");
    let rejection = match evaluate_first_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
        other => panic!("expected selected rejection, got {other:?}"),
    };

    let outcome = selected_rejection_to_qualification(&source, &rejection);
    assert_eq!(outcome.processing(), ProcessingStatus::Complete);
    assert_eq!(
        outcome.verdict(),
        Some(QualificationVerdictKind::StaticSemanticsRejected)
    );

    let evidence = outcome
        .rejection_evidence()
        .expect("selected rejection must retain whole-source rejection evidence");
    assert_eq!(evidence.family(), RejectionFamily::StaticSemantics);
    let anchor = evidence
        .subject()
        .authored_anchor()
        .expect("selected rejection evidence must remain authored");
    assert_eq!(anchor.fragment(), "let");
    assert_eq!(anchor.range().start(), 4);
    assert_eq!(anchor.range().end(), 7);
}

#[test]
fn source_mismatch_fails_closed_without_a_verdict() {
    let (_, script) = recognized("let let;");
    let rejection = match evaluate_first_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
        other => panic!("expected selected rejection, got {other:?}"),
    };
    let unrelated = SourceText::new(SourceId::new(210), "let xxx;".to_owned());

    let outcome = selected_rejection_to_qualification(&unrelated, &rejection);
    assert_eq!(outcome.processing(), ProcessingStatus::InternalFailure);
    assert_eq!(outcome.verdict(), None);
    assert!(outcome.rejection_evidence().is_none());
}

#[test]
fn production_slice_preserves_architecture_boundaries_in_source() {
    let production = include_str!("first_static_semantics.rs");

    for forbidden in [
        concat!("recognize_", "first_lexical_slice"),
        concat!("source.", "as_str()"),
        concat!("source.", "anchor("),
        concat!("QualificationOutcome", "::qualified"),
        concat!("unicode_", "normalization"),
    ] {
        assert!(
            !production.contains(forbidden),
            "selected static semantics must preserve its architecture boundary: found {forbidden}"
        );
    }

    let lookup = production
        .find("first_by_name.get(name)")
        .expect("duplicate membership must be checked before allocation");
    let reserve = production
        .find("first_by_name.try_reserve(1)")
        .expect("new unique names must use fallible reservation");
    assert!(lookup < reserve);
}
