//! Focused tests for the public selected flat lexical binding reference
//! facade (#862).
//!
//! These prove the public projection only; the selected recognizer, static
//! semantics, and binding-scope suites remain the authority for their
//! behavior. Expected ranges come from the accepted `selected_binding_scope`
//! fixtures or independently hand-counted authored bytes, never from facade
//! output.

use super::*;
use crate::SourceId;

fn analyze(text: &str) -> EsBindingRefReport {
    let source = SourceText::new(SourceId::new(0), text.to_owned());
    analyze_selected_flat_lexical_binding_refs(&source)
        .expect("ordinary analysis must not be a Core failure")
}

fn range(anchor: &SourceAnchor) -> (usize, usize) {
    (anchor.range().start(), anchor.range().end())
}

fn relations(text: &str) -> Vec<EsBindingRefRelation> {
    match analyze(text).outcome() {
        EsBindingRefOutcome::Complete { relations } => relations.clone(),
        other => panic!("expected complete analysis for {text:?}, got {other:?}"),
    }
}

/// (containing, reference, name, target range + order)
type Snapshot = (
    (usize, usize),
    (usize, usize),
    &'static str,
    Option<((usize, usize), EsBindingRefOrder)>,
);

fn assert_relations(text: &str, expected: &[Snapshot]) {
    let actual = relations(text);
    assert_eq!(actual.len(), expected.len(), "relation count for {text:?}");
    for (relation, (containing, reference, name, target)) in actual.iter().zip(expected) {
        assert_eq!(
            range(relation.containing_binding()),
            *containing,
            "{text:?}"
        );
        assert_eq!(range(relation.reference()), *reference, "{text:?}");
        assert_eq!(relation.semantic_name(), *name, "{text:?}");
        match (relation.target(), target) {
            (
                EsBindingRefTarget::SameSourceSelectedLexicalBinding { binding, order },
                Some((expected_range, expected_order)),
            ) => {
                assert_eq!(range(binding), *expected_range, "{text:?}");
                assert_eq!(order, expected_order, "{text:?}");
            }
            (EsBindingRefTarget::NoSameSourceSelectedLexicalBinding, None) => {}
            (actual, expected) => {
                panic!("target mismatch for {text:?}: {actual:?} vs {expected:?}")
            }
        }
    }
}

use EsBindingRefOrder::{After, Before, Same};

#[test]
fn report_carries_source_identity() {
    let source = SourceText::new(SourceId::new(7), "let x=1;".to_owned());
    let report = analyze_selected_flat_lexical_binding_refs(&source).unwrap();
    assert_eq!(report.source_id(), SourceId::new(7));
}

#[test]
fn preceding_following_same_and_absent_targets() {
    assert_relations(
        "let a=1; let x=a;",
        &[((13, 14), (15, 16), "a", Some(((4, 5), Before)))],
    );
    assert_relations(
        "let x=y; let y=1;",
        &[((4, 5), (6, 7), "y", Some(((13, 14), After)))],
    );
    assert_relations("let x=x;", &[((4, 5), (6, 7), "x", Some(((4, 5), Same)))]);
    assert_relations("let x=y;", &[((4, 5), (6, 7), "y", None)]);
}

#[test]
fn several_declarations_preserve_producer_order() {
    assert_relations(
        "let a=1,b=2; let x=a,y=b;",
        &[
            ((17, 18), (19, 20), "a", Some(((4, 5), Before))),
            ((21, 22), (23, 24), "b", Some(((8, 9), Before))),
        ],
    );
}

#[test]
fn several_facts_in_one_initializer_keep_authored_order_without_deduplication() {
    // "let a=1; " = 0..9; x 13..14; "a+a" = 15..18.
    assert_relations(
        "let a=1; let x=a+a;",
        &[
            ((13, 14), (15, 16), "a", Some(((4, 5), Before))),
            ((13, 14), (17, 18), "a", Some(((4, 5), Before))),
        ],
    );
    // Authored order, not target declaration order: "let b=1,a=2; let x=a+b;"
    // b 4..5, a 8..9, x 17..18, a 19..20, b 21..22.
    assert_relations(
        "let b=1,a=2; let x=a+b;",
        &[
            ((17, 18), (19, 20), "a", Some(((8, 9), Before))),
            ((17, 18), (21, 22), "b", Some(((4, 5), Before))),
        ],
    );
}

#[test]
fn parenthesized_reference_retains_only_the_inner_anchor() {
    assert_relations("let x=(y);", &[((4, 5), (7, 8), "y", None)]);
}

#[test]
fn escaped_reference_keeps_authored_spelling_and_decoded_semantic_name() {
    let found = relations(r"let a=1; let x=\u0061;");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].reference().fragment(), r"\u0061");
    assert_eq!(range(found[0].reference()), (15, 21));
    assert_eq!(found[0].semantic_name(), "a");
    match found[0].target() {
        EsBindingRefTarget::SameSourceSelectedLexicalBinding { binding, order } => {
            assert_eq!(range(binding), (4, 5));
            assert_eq!(binding.fragment(), "a");
            assert_eq!(*order, Before);
        }
        other => panic!("expected target, got {other:?}"),
    }

    // No normalization: `é` (precomposed) is not `e` + U+0301.
    let found = relations("let é=1; let x=e\\u0301;");
    assert_eq!(found[0].semantic_name(), "e\u{301}");
    assert_eq!(range(found[0].reference()), (16, 23));
    assert!(matches!(
        found[0].target(),
        EsBindingRefTarget::NoSameSourceSelectedLexicalBinding
    ));
}

#[test]
fn crlf_coordinates_come_from_retained_anchors() {
    // "let a=1;" 0..8, CRLF 8..10, "let " 10..14, x 14..15, "=" 15, a 16..17.
    let found = relations("let a=1;\r\nlet x=a;");
    assert_eq!(range(found[0].reference()), (16, 17));
    let start = found[0].reference().start_coordinate();
    assert_eq!((start.line_index(), start.byte_column()), (1, 6));
}

#[test]
fn bom_is_preserved_in_byte_offsets() {
    // BOM = 0..3; "let " 3..7; a 7..8; "=1; let " 8..16; x 16..17; a 18..19.
    assert_relations(
        "\u{feff}let a=1; let x=a;",
        &[((16, 17), (18, 19), "a", Some(((7, 8), Before)))],
    );
}

#[test]
fn complete_zero_relations_is_distinct_from_unsupported_coverage() {
    assert!(relations("let x=1;").is_empty());
    // Call-headed additive continuations are free-standing use-site carriers
    // (#853), outside the flat lexical initializer scope.
    for text in ["x;", "var x=1;", "{ let x=1; }", "let a=1; f()+a;"] {
        assert!(
            matches!(
                analyze(text).outcome(),
                EsBindingRefOutcome::UnsupportedCoverage
            ),
            "{text:?}"
        );
    }
}

#[test]
fn definitive_grammar_rejection_is_distinct_from_static_rejection() {
    let report = analyze(r"let \u{};");
    match report.outcome() {
        EsBindingRefOutcome::SelectedGrammarRejection { subject } => {
            assert_eq!(subject.fragment(), r"\u{}");
            assert_eq!(range(subject), (4, 8));
        }
        other => panic!("expected grammar rejection, got {other:?}"),
    }
}

fn static_rejection(text: &str) -> EsBindingRefStaticRejection {
    match analyze(text).outcome() {
        EsBindingRefOutcome::SelectedStaticRejection(rejection) => rejection.clone(),
        other => panic!("expected static rejection for {text:?}, got {other:?}"),
    }
}

#[test]
fn every_reachable_flat_static_rejection_keeps_its_retained_evidence() {
    match static_rejection(r"let \u0030;") {
        EsBindingRefStaticRejection::InvalidEscapedIdentifierStart { escape } => {
            assert_eq!(range(&escape), (4, 10));
            assert_eq!(escape.fragment(), r"\u0030");
        }
        other => panic!("{other:?}"),
    }
    match static_rejection(r"let a\u002D;") {
        EsBindingRefStaticRejection::InvalidEscapedIdentifierPart { escape } => {
            assert_eq!(range(&escape), (5, 11));
        }
        other => panic!("{other:?}"),
    }
    match static_rejection(r"let \u0069f=1;") {
        EsBindingRefStaticRejection::EscapedReservedWordBinding { binding } => {
            assert_eq!(range(&binding), (4, 11));
        }
        other => panic!("{other:?}"),
    }
    match static_rejection(r"let x=\u0069f;") {
        EsBindingRefStaticRejection::EscapedReservedWordInitializer { identifier } => {
            assert_eq!(range(&identifier), (6, 13));
        }
        other => panic!("{other:?}"),
    }
    match static_rejection("let let=1;") {
        EsBindingRefStaticRejection::BindingNamedLet { binding } => {
            assert_eq!(range(&binding), (4, 7));
        }
        other => panic!("{other:?}"),
    }
    match static_rejection("let a=1,a=2;") {
        EsBindingRefStaticRejection::DuplicateDeclarationBinding {
            first_binding,
            duplicate_binding,
        } => {
            assert_eq!(range(&first_binding), (4, 5));
            assert_eq!(range(&duplicate_binding), (8, 9));
        }
        other => panic!("{other:?}"),
    }
    match static_rejection("const x;") {
        EsBindingRefStaticRejection::ConstBindingMissingInitializer { binding } => {
            assert_eq!(range(&binding), (6, 7));
        }
        other => panic!("{other:?}"),
    }
    match static_rejection("let a=1; let a=2;") {
        EsBindingRefStaticRejection::DuplicateLexicalName {
            first_binding,
            duplicate_binding,
        } => {
            assert_eq!(range(&first_binding), (4, 5));
            assert_eq!(range(&duplicate_binding), (13, 14));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn unreachable_static_rejection_variants_fail_closed() {
    let source = SourceText::new(SourceId::new(0), "let a=1;".to_owned());
    let anchor = source.anchor(4, 5).unwrap();
    assert!(
        project_static_rejection(
            SelectedStaticSemanticsRejection::DuplicateBlockLexicalName {
                first_binding: anchor.clone(),
                duplicate_binding: anchor.clone(),
            }
        )
        .is_none()
    );
    assert!(
        project_static_rejection(SelectedStaticSemanticsRejection::LexicalVarNameCollision {
            lexical_binding: anchor.clone(),
            var_binding: anchor.clone(),
            primary_binding: anchor,
        })
        .is_none()
    );
}

#[test]
fn report_anchors_outlive_the_caller_source_text() {
    let report = {
        let source = SourceText::new(SourceId::new(0), "let a=1; let x=a;".to_owned());
        analyze_selected_flat_lexical_binding_refs(&source).unwrap()
    };
    let EsBindingRefOutcome::Complete { relations } = report.outcome() else {
        panic!("expected complete analysis");
    };
    let relation = &relations[0];
    assert_eq!(relation.containing_binding().fragment(), "x");
    assert_eq!(relation.reference().fragment(), "a");
    let EsBindingRefTarget::SameSourceSelectedLexicalBinding { binding, .. } = relation.target()
    else {
        panic!("expected target");
    };
    assert_eq!(binding.fragment(), "a");
}

#[test]
fn core_failure_display_names_no_source_content() {
    assert_eq!(
        EsBindingRefCoreFailure::InternalFailure.to_string(),
        "ECMAScript binding reference analysis internal failure"
    );
}

#[test]
fn empty_source_follows_existing_recognizer_semantics() {
    // The facade adds no special interpretation: the existing selected
    // recognizer does not recognize an empty source as the flat carrier.
    assert!(matches!(
        analyze("").outcome(),
        EsBindingRefOutcome::UnsupportedCoverage
    ));
}
