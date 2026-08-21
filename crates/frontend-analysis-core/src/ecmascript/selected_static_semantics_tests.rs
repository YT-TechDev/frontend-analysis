use crate::{SourceId, SourceText};

use super::qualification::{ProcessingStatus, QualificationVerdictKind, RejectionFamily};
use super::qualification_validation_tests::{gold_source, gold_subject_range};
use super::selected_lexical_slice::{
    SelectedLexicalScript, SelectedLexicalSliceOutcome, SelectedOneLevelBlockScript,
    SelectedVariableStatementScript, recognize_selected_lexical_slice,
};
use super::selected_static_semantics::{
    SelectedOneLevelBlockStaticSemanticsOutcome, SelectedStaticSemanticsOutcome,
    SelectedStaticSemanticsRejection, SelectedVariableStatementStaticSemanticsOutcome,
    evaluate_selected_one_level_block_static_semantics, evaluate_selected_static_semantics,
    evaluate_selected_variable_statement_static_semantics, selected_rejection_to_qualification,
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

fn recognized_block(text: &str) -> (SourceText, SelectedOneLevelBlockScript) {
    let source = source(text);
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(script) => script,
        other => panic!("expected one-level Block recognition for {text:?}, got {other:?}"),
    };
    (source, script)
}

fn recognized_variable(text: &str) -> (SourceText, SelectedVariableStatementScript) {
    let source = source(text);
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(script) => script,
        other => panic!("expected var-enabled recognition for {text:?}, got {other:?}"),
    };
    (source, script)
}

fn accepted(text: &str) {
    let (_, script) = recognized(text);
    assert!(matches!(
        evaluate_selected_static_semantics(&script),
        SelectedStaticSemanticsOutcome::Accepted(_)
    ));
}

fn accepted_variable(text: &str) {
    let (_, script) = recognized_variable(text);
    assert!(matches!(
        evaluate_selected_variable_statement_static_semantics(&script),
        SelectedVariableStatementStaticSemanticsOutcome::Accepted
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
            SelectedStaticSemanticsRejection::DuplicateBlockLexicalName { .. } => {
                panic!("flat fixture must not acquire Block duplicate evidence")
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
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. } => {
                panic!("flat fixture must not acquire var collision evidence")
            }
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierStart { escape } => {
                ("EE01-R01", (escape.range().start(), escape.range().end()))
            }
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierPart { escape } => {
                ("EE01-R02", (escape.range().start(), escape.range().end()))
            }
            SelectedStaticSemanticsRejection::EscapedReservedWord { binding } => {
                ("EE04-R08", (binding.range().start(), binding.range().end()))
            }
            SelectedStaticSemanticsRejection::EscapedReservedWordInitializer { identifier } => (
                "EE04-R08",
                (identifier.range().start(), identifier.range().end()),
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
fn escaped_ee01_and_ee04_preserve_exact_primary_authored_evidence() {
    let cases = [
        (r"let \u0030;", "EE01-R01", (4, 10)),
        (r"let a\u002D;", "EE01-R02", (5, 11)),
        (r"let \uD800;", "EE01-R01", (4, 10)),
        (r"let \u{D800};", "EE01-R01", (4, 12)),
        (r"let \u0069f;", "EE04-R08", (4, 11)),
        (r"let \u006Cet;", "R01", (4, 12)),
    ];

    for (text, expected_rule, expected_range) in cases {
        let (_, script) = recognized(text);
        let rejection = match evaluate_selected_static_semantics(&script) {
            SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
            other => panic!("expected rejection for {text:?}, got {other:?}"),
        };
        let (rule, range) = match rejection {
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierStart { escape } => {
                ("EE01-R01", (escape.range().start(), escape.range().end()))
            }
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierPart { escape } => {
                ("EE01-R02", (escape.range().start(), escape.range().end()))
            }
            SelectedStaticSemanticsRejection::EscapedReservedWord { binding } => {
                ("EE04-R08", (binding.range().start(), binding.range().end()))
            }
            SelectedStaticSemanticsRejection::BindingNamedLet { binding } => {
                ("R01", (binding.range().start(), binding.range().end()))
            }
            other => panic!("unexpected rejection for {text:?}: {other:?}"),
        };
        assert_eq!(rule, expected_rule, "{text:?}");
        assert_eq!(range, expected_range, "{text:?}");
    }
}

#[test]
fn escaped_reserved_initializer_ee04_preserves_exact_rhs_subject_and_precedence() {
    for (text, expected_fragment, expected_range) in [
        (r"const x = \u0074his;", r"\u0074his", (10, 19)),
        (r"const x = \u006Eull;", r"\u006Eull", (10, 19)),
        (r"const x = \u0074rue;", r"\u0074rue", (10, 19)),
        (r"const x = \u0066alse;", r"\u0066alse", (10, 20)),
        (r"const x = n\u0075ll;", r"n\u0075ll", (10, 19)),
        (r"const x = tr\u0075e;", r"tr\u0075e", (10, 19)),
        (r"const x = thi\u0073;", r"thi\u0073", (10, 19)),
    ] {
        let (_, script) = recognized(text);
        match evaluate_selected_static_semantics(&script) {
            SelectedStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::EscapedReservedWordInitializer { identifier },
            ) => {
                assert_eq!(identifier.fragment(), expected_fragment, "{text}");
                assert_eq!(
                    (identifier.range().start(), identifier.range().end()),
                    expected_range,
                    "{text}"
                );
            }
            other => panic!("expected RHS EE-04-R08 rejection for {text:?}, got {other:?}"),
        }
    }

    for (text, expected_rule, expected_fragment, expected_range) in [
        (r"let let = \u006Eull;", "R01", "let", (4, 7)),
        (
            r"let x = \u006Eull, x = foo;",
            "EE04-R08-RHS",
            r"\u006Eull",
            (8, 17),
        ),
        (
            r"const x = \u006Eull, y;",
            "EE04-R08-RHS",
            r"\u006Eull",
            (10, 19),
        ),
        (
            r"let x = foo; let x = \u006Eull;",
            "EE04-R08-RHS",
            r"\u006Eull",
            (21, 30),
        ),
    ] {
        let (_, script) = recognized(text);
        let rejection = match evaluate_selected_static_semantics(&script) {
            SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
            other => panic!("expected precedence rejection for {text:?}, got {other:?}"),
        };
        let (actual_rule, anchor) = match rejection {
            SelectedStaticSemanticsRejection::BindingNamedLet { binding } => ("R01", binding),
            SelectedStaticSemanticsRejection::EscapedReservedWordInitializer { identifier } => {
                ("EE04-R08-RHS", identifier)
            }
            other => panic!("unexpected precedence rejection for {text:?}: {other:?}"),
        };
        assert_eq!(actual_rule, expected_rule, "{text}");
        assert_eq!(anchor.fragment(), expected_fragment, "{text}");
        assert_eq!(
            (anchor.range().start(), anchor.range().end()),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn escaped_reserved_initializer_non_triggers_remain_accepted() {
    for rhs in [
        r"\u0079ield",
        r"a\u0077ait",
        r"\u006Cet",
        r"\u0073tatic",
        r"\u0069mplements",
        r"\u0069nterface",
        r"\u0070ackage",
        r"\u0070rivate",
        r"\u0070rotected",
        r"\u0070ublic",
        r"\u0065val",
        r"\u0061rguments",
    ] {
        accepted(&format!("const x = {rhs};"));
    }
}

#[test]
fn escaped_semantic_name_equality_is_exact_and_not_normalized() {
    for text in [
        r"let \u0061, a;",
        r"let $, \u0024;",
        r"let _, \u005F;",
        "let \\u{1D49C}, 𝒜;",
    ] {
        let (_, script) = recognized(text);
        assert!(matches!(
            evaluate_selected_static_semantics(&script),
            SelectedStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::DuplicateDeclarationBinding { .. }
            )
        ));
    }

    accepted(r"let \u00E9, e\u0301;");
    for text in [
        r"let \u0061wait, \u0079ield, \u0073tatic, \u0069mplements, \u0061rguments, \u0065val;",
        r"let a\u200C, b\u200D;",
        r"let \u{00000061};",
    ] {
        accepted(text);
    }
}

#[test]
fn escaped_binding_precedence_matches_all_twelve_candidate_independent_witnesses() {
    let cases = [
        (r"let let, \u0030;", "R01", (4, 7)),
        (r"let \u0030, let;", "EE01-R01", (4, 10)),
        (r"let \u0069f, let;", "EE04-R08", (4, 11)),
        (r"let let, \u0069f;", "R01", (4, 7)),
        (r"let x, x, \u0030;", "EE01-R01", (10, 16)),
        (r"let x, x, \u0069f;", "EE04-R08", (10, 17)),
        (r"const x, \u0030;", "EE01-R01", (9, 15)),
        (r"const x, \u0069f;", "EE04-R08", (9, 16)),
        (r"let x, x; let \u0030;", "R02", (7, 8)),
        (r"const x; let \u0030;", "R03", (6, 7)),
        (r"let x; let x; let \u0030;", "EE01-R01", (18, 24)),
        (r"let x; let x; let \u0069f;", "EE04-R08", (18, 25)),
    ];

    for (text, expected_rule, expected_range) in cases {
        let (_, script) = recognized(text);
        let rejection = match evaluate_selected_static_semantics(&script) {
            SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
            other => panic!("expected rejection for {text:?}, got {other:?}"),
        };
        let (rule, range) = match rejection {
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierStart { escape } => {
                ("EE01-R01", (escape.range().start(), escape.range().end()))
            }
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierPart { escape } => {
                ("EE01-R02", (escape.range().start(), escape.range().end()))
            }
            SelectedStaticSemanticsRejection::EscapedReservedWord { binding } => {
                ("EE04-R08", (binding.range().start(), binding.range().end()))
            }
            SelectedStaticSemanticsRejection::EscapedReservedWordInitializer { identifier } => (
                "EE04-R08",
                (identifier.range().start(), identifier.range().end()),
            ),
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
            SelectedStaticSemanticsRejection::DuplicateBlockLexicalName { .. } => {
                panic!("flat fixture must not acquire Block duplicate evidence")
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
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. } => {
                panic!("flat fixture must not acquire var collision evidence")
            }
        };
        assert_eq!(rule, expected_rule, "{text:?}");
        assert_eq!(range, expected_range, "{text:?}");
    }
}

#[test]
fn same_binding_invalid_escape_primary_is_first_authored_occurrence() {
    for (text, expected_rule, expected_range) in [
        (r"let \u0030\u002D;", "EE01-R01", (4, 10)),
        (r"let a\u002D\u002E;", "EE01-R02", (5, 11)),
        (r"let \uD800\uDC00;", "EE01-R01", (4, 10)),
    ] {
        let (_, script) = recognized(text);
        let rejection = match evaluate_selected_static_semantics(&script) {
            SelectedStaticSemanticsOutcome::Rejected(rejection) => rejection,
            other => panic!("expected EE-01 rejection for {text:?}, got {other:?}"),
        };
        let (rule, range) = match rejection {
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierStart { escape } => {
                ("EE01-R01", (escape.range().start(), escape.range().end()))
            }
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierPart { escape } => {
                ("EE01-R02", (escape.range().start(), escape.range().end()))
            }
            other => panic!("unexpected rejection for {text:?}: {other:?}"),
        };
        assert_eq!(rule, expected_rule);
        assert_eq!(range, expected_range);
    }
}

#[test]
fn selected_rejections_map_to_static_semantics_rejected_with_authored_primary_evidence() {
    for (text, expected_fragment) in [
        ("const let;", "let"),
        ("let x, x;", "x"),
        ("const x;", "x"),
        ("let x, y; let y;", "y"),
        (r"const x = \u006Eull;", r"\u006Eull"),
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
fn one_level_block_static_semantics_preserve_region_duplicate_domains() {
    let (_, script) = recognized_block("let a=1; { let a=2; }");
    let accepted = match evaluate_selected_one_level_block_static_semantics(&script) {
        SelectedOneLevelBlockStaticSemanticsOutcome::Accepted(accepted) => accepted,
        other => panic!("legal outer/inner shadowing must be accepted, got {other:?}"),
    };
    assert_eq!(accepted.script().items().len(), 2);

    let (_, script) = recognized_block("{ let a=1; } { let a=2; }");
    assert!(matches!(
        evaluate_selected_one_level_block_static_semantics(&script),
        SelectedOneLevelBlockStaticSemanticsOutcome::Accepted(_)
    ));

    let (_, script) = recognized_block("{ let a=1; let a=2; }");
    match evaluate_selected_one_level_block_static_semantics(&script) {
        SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateBlockLexicalName {
                first_binding,
                duplicate_binding,
            },
        ) => {
            assert_eq!(
                (first_binding.range().start(), first_binding.range().end()),
                (6, 7)
            );
            assert_eq!(
                (
                    duplicate_binding.range().start(),
                    duplicate_binding.range().end()
                ),
                (15, 16)
            );
        }
        other => panic!("expected EE-14 Block duplicate rejection, got {other:?}"),
    }

    let (_, script) = recognized_block("let a=1; { let a=2; } let a=3;");
    match evaluate_selected_one_level_block_static_semantics(&script) {
        SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateLexicalName {
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
                (26, 27)
            );
        }
        other => panic!("expected top-level EE-36 duplicate rejection, got {other:?}"),
    }
}

#[test]
fn one_level_block_static_evidence_selection_is_local_then_block_then_script() {
    let (_, script) = recognized_block("{ let a=1; let a=2; } const x;");
    match evaluate_selected_one_level_block_static_semantics(&script) {
        SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::ConstBindingMissingInitializer { binding },
        ) => assert_eq!((binding.range().start(), binding.range().end()), (28, 29)),
        other => panic!("declaration-local R03 must precede Block duplicate evidence: {other:?}"),
    }

    let (_, script) = recognized_block("{ let a=1; let a=2; } let z=1; let z=2;");
    match evaluate_selected_one_level_block_static_semantics(&script) {
        SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateBlockLexicalName {
                duplicate_binding, ..
            },
        ) => assert_eq!(
            (
                duplicate_binding.range().start(),
                duplicate_binding.range().end()
            ),
            (15, 16)
        ),
        other => panic!("Block EE-14 duplicate must precede top-level EE-36 duplicate: {other:?}"),
    }
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

    assert_eq!(production.matches("first_by_name: HashMap<").count(), 2);
    assert_eq!(production.matches("try_reserve(1)").count(), 4);
    assert!(production.contains("DuplicateDeclarationBinding"));
    assert!(production.contains("DuplicateBlockLexicalName"));
    assert!(production.contains("DuplicateLexicalName"));
    assert!(production.contains("LexicalVarNameCollision"));
    assert!(production.contains("SelectedOneLevelBlockStaticSemanticsAccepted"));
    assert!(production.contains("EscapedReservedWordInitializer"));
}

#[test]
fn one_level_block_preserves_existing_malformed_binding_and_rhs_lifecycles() {
    let malformed_binding_source = source(r"{ let \u{}; }");
    match recognize_selected_lexical_slice(&malformed_binding_source) {
        SelectedLexicalSliceOutcome::DefinitiveGrammarRejectionEvidence { subject } => {
            assert_eq!(subject.fragment(), r"\u{}");
            assert_eq!((subject.range().start(), subject.range().end()), (6, 10));
        }
        other => panic!("unexpected Block grammar lifecycle: {other:?}"),
    }

    let (_, script) = recognized_block(r"{ let \u0030; }");
    match evaluate_selected_one_level_block_static_semantics(&script) {
        SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::InvalidEscapedIdentifierStart { escape },
        ) => {
            assert_eq!(escape.fragment(), r"\u0030");
            assert_eq!((escape.range().start(), escape.range().end()), (6, 12));
        }
        other => panic!("unexpected Block static lifecycle: {other:?}"),
    }

    let malformed_rhs_source = source(r"{ const x = \u{}; }");
    assert!(matches!(
        recognize_selected_lexical_slice(&malformed_rhs_source),
        SelectedLexicalSliceOutcome::UnsupportedCoverage
    ));
}

#[test]
fn variable_binding_context_keeps_let_valid_and_reserved_escape_rejected() {
    accepted_variable("var let;");
    accepted_variable(r"var \u006Cet;");

    let (_, script) = recognized_variable(r"var \u0069f;");
    match evaluate_selected_variable_statement_static_semantics(&script) {
        SelectedVariableStatementStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::EscapedReservedWord { binding },
        ) => {
            assert_eq!(binding.fragment(), r"\u0069f");
            assert_eq!((binding.range().start(), binding.range().end()), (4, 11));
        }
        other => panic!("expected existing escaped ReservedWord rejection, got {other:?}"),
    }
}

#[test]
fn ee_36_r02_preserves_exact_provenance_and_completing_primary() {
    for (text, lexical_range, var_range, primary_range) in [
        ("let x; var x;", (4, 5), (11, 12), (11, 12)),
        ("var x; let x;", (11, 12), (4, 5), (11, 12)),
        (r"let a; var \u0061;", (4, 5), (11, 17), (11, 17)),
        (r"var \u0061; let a;", (16, 17), (4, 10), (16, 17)),
    ] {
        let (_, script) = recognized_variable(text);
        match evaluate_selected_variable_statement_static_semantics(&script) {
            SelectedVariableStatementStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                    lexical_binding,
                    var_binding,
                    primary_binding,
                },
            ) => {
                assert_eq!(
                    (lexical_binding.range().start(), lexical_binding.range().end()),
                    lexical_range,
                    "{text}"
                );
                assert_eq!(
                    (var_binding.range().start(), var_binding.range().end()),
                    var_range,
                    "{text}"
                );
                assert_eq!(
                    (primary_binding.range().start(), primary_binding.range().end()),
                    primary_range,
                    "{text}"
                );
            }
            other => panic!("expected lexical/var collision for {text:?}, got {other:?}"),
        }
    }
}

#[test]
fn ee_36_r02_uses_authored_completion_order_and_first_var_provenance() {
    for (text, lexical_range, var_range, primary_range) in [
        (
            "var y; var x; let x; let y;",
            (18, 19),
            (11, 12),
            (18, 19),
        ),
        ("var x; var x; let x;", (18, 19), (4, 5), (18, 19)),
        (
            "let y; var y; var x; let x;",
            (4, 5),
            (11, 12),
            (11, 12),
        ),
    ] {
        let (_, script) = recognized_variable(text);
        match evaluate_selected_variable_statement_static_semantics(&script) {
            SelectedVariableStatementStaticSemanticsOutcome::Rejected(
                SelectedStaticSemanticsRejection::LexicalVarNameCollision {
                    lexical_binding,
                    var_binding,
                    primary_binding,
                },
            ) => {
                assert_eq!(
                    (lexical_binding.range().start(), lexical_binding.range().end()),
                    lexical_range,
                    "{text}"
                );
                assert_eq!(
                    (var_binding.range().start(), var_binding.range().end()),
                    var_range,
                    "{text}"
                );
                assert_eq!(
                    (primary_binding.range().start(), primary_binding.range().end()),
                    primary_range,
                    "{text}"
                );
            }
            other => panic!("wrong multi-collision result for {text:?}: {other:?}"),
        }
    }
}

#[test]
fn variable_static_semantics_excludes_block_names_and_preserves_non_normalization() {
    for text in [
        "let x; var y;",
        "var x; var x;",
        "{ let x; } var x;",
        "let é; var e\u{301};",
    ] {
        accepted_variable(text);
    }

    let (_, script) = recognized_variable("let x; { let y; } var x;");
    assert!(matches!(
        evaluate_selected_variable_statement_static_semantics(&script),
        SelectedVariableStatementStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. }
        )
    ));
}

#[test]
fn variable_static_semantics_preserves_tier_one_through_three_precedence() {
    for (text, expected) in [
        ("let x,x; var x;", "local"),
        ("let x; let x; var x;", "script"),
        (r"let x,x; var \u0069f;", "local"),
    ] {
        let (_, script) = recognized_variable(text);
        let rejection = match evaluate_selected_variable_statement_static_semantics(&script) {
            SelectedVariableStatementStaticSemanticsOutcome::Rejected(rejection) => rejection,
            other => panic!("expected precedence rejection for {text:?}, got {other:?}"),
        };
        match (expected, rejection) {
            (
                "local",
                SelectedStaticSemanticsRejection::DuplicateDeclarationBinding { .. },
            ) => {}
            ("script", SelectedStaticSemanticsRejection::DuplicateLexicalName { .. }) => {}
            (_, other) => panic!("wrong primary rejection for {text:?}: {other:?}"),
        }
    }
}
