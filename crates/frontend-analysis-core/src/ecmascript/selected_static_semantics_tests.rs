use crate::{SourceId, SourceText};

use super::qualification::{ProcessingStatus, QualificationVerdictKind, RejectionFamily};
use super::qualification_validation_tests::{gold_source, gold_subject_range};
use super::selected_lexical_slice::{
    SelectedLexicalScript, SelectedLexicalSliceOutcome, recognize_selected_lexical_slice,
};
use super::selected_static_semantics::{
    SelectedStaticSemanticsOutcome, SelectedStaticSemanticsRejection,
    evaluate_selected_static_semantics, selected_rejection_to_qualification,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(218), text.to_owned())
}

fn recognized(text: &str) -> (SourceText, SelectedLexicalScript) {
    let source = source(text);
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(script) => script,
        other => panic!("expected selected-slice recognition for {text:?}, got {other:?}"),
    };
    (source, script)
}

fn accepted(text: &str) {
    let (_, script) = recognized(text);
    assert!(matches!(
        evaluate_selected_static_semantics(&script),
        SelectedStaticSemanticsOutcome::Accepted
    ));
}

#[test]
fn ee_15_r01_checks_every_selected_binding_in_source_order() {
    let text = gold_source("JS-GOLD-LEXDECL-CONST-LET-MISSING-INIT-001")
        .expect("combined R01/R03 gold must exist");
    let expected = gold_subject_range("JS-GOLD-LEXDECL-CONST-LET-MISSING-INIT-001")
        .expect("combined gold must retain primary subject");
    let (_, script) = recognized(text);

    match evaluate_selected_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::BindingNamedLet { binding },
        ) => {
            assert_eq!(binding.fragment(), "let");
            assert_eq!((binding.range().start(), binding.range().end()), expected);
        }
        other => panic!("expected R01 rejection, got {other:?}"),
    }

    let (_, script) = recognized("let x, let;");
    assert!(matches!(
        evaluate_selected_static_semantics(&script),
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::BindingNamedLet { .. }
        )
    ));
}

#[test]
fn ee_15_r02_is_declaration_local_and_retains_first_duplicate_occurrences() {
    let text = gold_source("JS-GOLD-LEXDECL-DUPBOUNDNAMES-001").expect("R02 gold must exist");
    let expected = gold_subject_range("JS-GOLD-LEXDECL-DUPBOUNDNAMES-001")
        .expect("R02 gold must retain primary subject");
    let (_, script) = recognized(text);

    match evaluate_selected_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateDeclarationBinding {
                first_binding,
                duplicate_binding,
            },
        ) => {
            assert_eq!(
                (first_binding.range().start(), first_binding.range().end()),
                (4, 5)
            );
            assert_eq!(
                (
                    duplicate_binding.range().start(),
                    duplicate_binding.range().end()
                ),
                expected
            );
        }
        other => panic!("expected R02 rejection, got {other:?}"),
    }
}

#[test]
fn ee_15_r03_uses_affected_binding_without_fabricating_missing_initializer_range() {
    let text = gold_source("JS-GOLD-LEXDECL-CONST-MISSING-INIT-001").expect("R03 gold must exist");
    let expected = gold_subject_range("JS-GOLD-LEXDECL-CONST-MISSING-INIT-001")
        .expect("R03 gold must retain binding subject");
    let (_, script) = recognized(text);

    match evaluate_selected_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::ConstBindingMissingInitializer { binding },
        ) => {
            assert_eq!(binding.fragment(), "x");
            assert_eq!((binding.range().start(), binding.range().end()), expected);
        }
        other => panic!("expected R03 rejection, got {other:?}"),
    }

    let (_, script) = recognized("const x=1, y;");
    match evaluate_selected_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::ConstBindingMissingInitializer { binding },
        ) => assert_eq!(binding.fragment(), "y"),
        other => panic!("expected missing initializer on y, got {other:?}"),
    }
}

#[test]
fn ee_36_runs_after_all_local_checks_and_flattens_binding_source_order() {
    let text = gold_source("JS-GOLD-SCRIPT-DUPLEXICAL-MULTIBIND-001")
        .expect("multibind Script duplicate gold must exist");
    let expected = gold_subject_range("JS-GOLD-SCRIPT-DUPLEXICAL-MULTIBIND-001")
        .expect("Script duplicate gold must retain primary subject");
    let (_, script) = recognized(text);

    match evaluate_selected_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateLexicalName {
                first_binding,
                duplicate_binding,
            },
        ) => {
            assert_eq!(first_binding.fragment(), "y");
            assert_eq!(
                (first_binding.range().start(), first_binding.range().end()),
                (7, 8)
            );
            assert_eq!(
                (
                    duplicate_binding.range().start(),
                    duplicate_binding.range().end()
                ),
                expected
            );
        }
        other => panic!("expected EE-36 rejection, got {other:?}"),
    }
}

#[test]
fn accepted_precedence_matrix_is_declaration_source_order_then_local_rule_order() {
    let cases = [
        ("const x; let let;", "R03", (6, 7)),
        ("let let, let;", "R01", (4, 7)),
        ("let x; let x; let let;", "R01", (18, 21)),
        ("let x, x; const y;", "R02", (7, 8)),
        ("const x; let y, y;", "R03", (6, 7)),
        ("const x = 1, x;", "R02", (13, 14)),
    ];

    for (text, expected_rule, expected_range) in cases {
        let (_, script) = recognized(text);
        let rejection = match evaluate_selected_static_semantics(&script) {
            SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
            other => panic!("expected rejection for {text:?}, got {other:?}"),
        };

        let (actual_rule, actual_range) = match rejection {
            SelectedStaticSemanticsRejection::BindingNamedLet { binding } => {
                ("R01", (binding.range().start(), binding.range().end()))
            }
            SelectedStaticSemanticsRejection::DuplicateDeclarationBinding {
                duplicate_binding,
                ..
            } => (
                "R02",
                (
                    duplicate_binding.range().start(),
                    duplicate_binding.range().end(),
                ),
            ),
            SelectedStaticSemanticsRejection::ConstBindingMissingInitializer { binding } => {
                ("R03", (binding.range().start(), binding.range().end()))
            }
            SelectedStaticSemanticsRejection::DuplicateLexicalName {
                duplicate_binding, ..
            } => (
                "EE36",
                (
                    duplicate_binding.range().start(),
                    duplicate_binding.range().end(),
                ),
            ),
        };

        assert_eq!(
            actual_rule, expected_rule,
            "wrong primary rule for {text:?}"
        );
        assert_eq!(
            actual_range, expected_range,
            "wrong primary range for {text:?}"
        );
    }
}

#[test]
fn exact_non_normalized_name_identity_and_selected_ee04_closure_remain_non_triggering() {
    for text in [
        "let é, e\u{301};",
        "let await, yield;",
        "let static, implements;",
        "const arguments=1, eval=2;",
    ] {
        accepted(text);
    }
}

#[test]
fn selected_rejections_map_to_static_semantics_rejected_with_authored_primary_evidence() {
    for (text, expected_fragment) in [
        ("const let;", "let"),
        ("let x, x;", "x"),
        ("const x;", "x"),
        ("let x, y; let y;", "y"),
    ] {
        let (source, script) = recognized(text);
        let rejection = match evaluate_selected_static_semantics(&script) {
            SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
            other => panic!("expected selected rejection for {text:?}, got {other:?}"),
        };

        let outcome = selected_rejection_to_qualification(&source, &rejection);
        assert_eq!(outcome.processing(), ProcessingStatus::Complete);
        assert_eq!(
            outcome.verdict(),
            Some(QualificationVerdictKind::StaticSemanticsRejected)
        );
        let evidence = outcome.rejection_evidence().expect("rejection evidence");
        assert_eq!(evidence.family(), RejectionFamily::StaticSemantics);
        assert_eq!(
            evidence
                .subject()
                .authored_anchor()
                .expect("authored primary evidence")
                .fragment(),
            expected_fragment
        );
    }
}

#[test]
fn source_mismatch_fails_closed_without_a_verdict() {
    let (_, script) = recognized("const x;");
    let rejection = match evaluate_selected_static_semantics(&script) {
        SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
        other => panic!("expected selected rejection, got {other:?}"),
    };
    let unrelated = SourceText::new(SourceId::new(218), "const y;".to_owned());

    let outcome = selected_rejection_to_qualification(&unrelated, &rejection);
    assert_eq!(outcome.processing(), ProcessingStatus::InternalFailure);
    assert_eq!(outcome.verdict(), None);
    assert!(outcome.rejection_evidence().is_none());
}

#[test]
fn production_static_semantics_preserves_architecture_boundaries_in_source() {
    let production = include_str!("selected_static_semantics.rs");

    for forbidden in [
        concat!("recognize_", "selected_lexical_slice"),
        concat!("source.", "as_str()"),
        concat!("source.", "anchor("),
        concat!("QualificationOutcome", "::qualified"),
        concat!("unicode_", "normalization"),
        concat!("qualification_validation", "_tests"),
    ] {
        assert!(
            !production.contains(forbidden),
            "selected static semantics must preserve its architecture boundary: found {forbidden}"
        );
    }

    assert_eq!(production.matches("HashMap<&str").count(), 2);
    assert_eq!(production.matches("try_reserve(1)").count(), 2);
    assert!(production.contains("DuplicateDeclarationBinding"));
    assert!(production.contains("DuplicateLexicalName"));
}
