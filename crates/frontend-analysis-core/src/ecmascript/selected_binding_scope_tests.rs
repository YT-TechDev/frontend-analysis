use crate::{SourceId, SourceText};

use super::selected_binding_scope::{
    SelectedBindingScopeOutcome, SelectedBindingScopeTarget, SelectedLexicalBindingOrder,
    analyze_selected_binding_scope,
};
use super::selected_lexical_slice::{
    SelectedIdentifierReferenceNameState, SelectedLexicalScript, SelectedLexicalSliceOutcome,
    recognize_selected_lexical_slice,
};
use super::selected_static_semantics::{
    SelectedStaticSemanticsAccepted, SelectedStaticSemanticsOutcome,
    SelectedStaticSemanticsRejection, evaluate_selected_static_semantics,
};

const BINDING_SCOPE_SOURCE: &str = include_str!("selected_binding_scope.rs");

#[derive(Debug, PartialEq, Eq)]
struct RelationSnapshot {
    containing_binding: (usize, usize, String),
    reference: (usize, usize, String),
    semantic_name: String,
    target: Option<(usize, usize, String, SelectedLexicalBindingOrder)>,
}

fn recognized(text: &str) -> SelectedLexicalScript {
    let source = SourceText::new(SourceId::new(274), text.to_owned());
    match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedSelectedSlice(script) => script,
        other => panic!("expected selected recognition for {text:?}, got {other:?}"),
    }
}

fn accepted(script: &SelectedLexicalScript) -> SelectedStaticSemanticsAccepted<'_> {
    match evaluate_selected_static_semantics(script) {
        SelectedStaticSemanticsOutcome::Accepted(accepted) => accepted,
        other => panic!("expected selected static acceptance, got {other:?}"),
    }
}

fn relation_snapshots(text: &str) -> Vec<RelationSnapshot> {
    let script = recognized(text);
    let accepted = accepted(&script);
    let analysis = match analyze_selected_binding_scope(&accepted) {
        SelectedBindingScopeOutcome::Complete(analysis) => analysis,
        other => panic!("expected complete Binding / Scope analysis, got {other:?}"),
    };

    analysis
        .relations()
        .iter()
        .map(|relation| {
            let containing_binding = relation.containing_binding();
            let reference = relation.reference();
            let target = match relation.target() {
                SelectedBindingScopeTarget::SameSourceSelectedLexicalBinding { binding, order } => {
                    Some((
                        binding.range().start(),
                        binding.range().end(),
                        binding.fragment().to_owned(),
                        order,
                    ))
                }
                SelectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding => None,
            };
            RelationSnapshot {
                containing_binding: (
                    containing_binding.range().start(),
                    containing_binding.range().end(),
                    containing_binding.fragment().to_owned(),
                ),
                reference: (
                    reference.range().start(),
                    reference.range().end(),
                    reference.fragment().to_owned(),
                ),
                semantic_name: relation.semantic_name().to_owned(),
                target,
            }
        })
        .collect()
}

#[test]
fn accepted_witness_is_bound_to_the_exact_selected_script() {
    let script = recognized("let x=1;");
    let accepted = accepted(&script);
    assert!(std::ptr::eq(accepted.script(), &script));
}

#[test]
fn direct_and_escaped_reference_facts_retain_only_consumer_required_identity() {
    let direct = recognized("let x=y;");
    let direct_fact = direct.declarations()[0].bindings()[0]
        .identifier_reference_initializer()
        .expect("direct reference fact must be retained");
    assert_eq!(direct_fact.reference().fragment(), "y");
    assert_eq!(
        (
            direct_fact.reference().range().start(),
            direct_fact.reference().range().end()
        ),
        (6, 7)
    );
    assert!(matches!(
        direct_fact.name_state(),
        SelectedIdentifierReferenceNameState::Direct
    ));
    assert_eq!(direct_fact.semantic_name(), "y");

    let escaped = recognized(r"let x=\u0061;");
    let escaped_fact = escaped.declarations()[0].bindings()[0]
        .identifier_reference_initializer()
        .expect("escaped reference fact must be retained");
    assert_eq!(escaped_fact.reference().fragment(), r"\u0061");
    assert_eq!(
        (
            escaped_fact.reference().range().start(),
            escaped_fact.reference().range().end()
        ),
        (6, 12)
    );
    match escaped_fact.name_state() {
        SelectedIdentifierReferenceNameState::Escaped { decoded } => assert_eq!(decoded, "a"),
        other => panic!("expected escaped reference name state, got {other:?}"),
    }
    assert_eq!(escaped_fact.semantic_name(), "a");
}

#[test]
fn non_reference_atoms_do_not_retain_ordinary_reference_facts() {
    for text in [
        "let x=1;",
        "let x=true;",
        "let x=null;",
        "let x=this;",
        "let x='a';",
    ] {
        let script = recognized(text);
        assert!(
            script.declarations()[0].bindings()[0]
                .identifier_reference_initializer()
                .is_none(),
            "unexpected ordinary reference fact for {text:?}"
        );
    }
}

#[test]
fn backward_forward_self_and_no_match_match_the_candidate_independent_oracles() {
    assert_eq!(
        relation_snapshots("let a=1; let x=a;"),
        vec![RelationSnapshot {
            containing_binding: (13, 14, "x".to_owned()),
            reference: (15, 16, "a".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
        }]
    );

    assert_eq!(
        relation_snapshots("let x=y; let y=1;"),
        vec![RelationSnapshot {
            containing_binding: (4, 5, "x".to_owned()),
            reference: (6, 7, "y".to_owned()),
            semantic_name: "y".to_owned(),
            target: Some((13, 14, "y".to_owned(), SelectedLexicalBindingOrder::After,)),
        }]
    );

    let self_relation = relation_snapshots("let x=x;");
    assert_eq!(
        self_relation,
        vec![RelationSnapshot {
            containing_binding: (4, 5, "x".to_owned()),
            reference: (6, 7, "x".to_owned()),
            semantic_name: "x".to_owned(),
            target: Some((4, 5, "x".to_owned(), SelectedLexicalBindingOrder::Same)),
        }]
    );
    let self_target = self_relation[0]
        .target
        .as_ref()
        .expect("self relation must have a same-source target");
    assert!(self_target.0 < self_relation[0].reference.0);
    assert_eq!(self_target.3, SelectedLexicalBindingOrder::Same);

    assert_eq!(
        relation_snapshots("let x=y;"),
        vec![RelationSnapshot {
            containing_binding: (4, 5, "x".to_owned()),
            reference: (6, 7, "y".to_owned()),
            semantic_name: "y".to_owned(),
            target: None,
        }]
    );
}

#[test]
fn binding_list_order_and_prior_let_without_initializer_match_the_oracle() {
    assert_eq!(
        relation_snapshots("let a=1, x=a;"),
        vec![RelationSnapshot {
            containing_binding: (9, 10, "x".to_owned()),
            reference: (11, 12, "a".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
        }]
    );

    assert_eq!(
        relation_snapshots("let x=y, y=1;"),
        vec![RelationSnapshot {
            containing_binding: (4, 5, "x".to_owned()),
            reference: (6, 7, "y".to_owned()),
            semantic_name: "y".to_owned(),
            target: Some((9, 10, "y".to_owned(), SelectedLexicalBindingOrder::After,)),
        }]
    );

    assert_eq!(
        relation_snapshots("let x=x, y=1;"),
        vec![RelationSnapshot {
            containing_binding: (4, 5, "x".to_owned()),
            reference: (6, 7, "x".to_owned()),
            semantic_name: "x".to_owned(),
            target: Some((4, 5, "x".to_owned(), SelectedLexicalBindingOrder::Same)),
        }]
    );

    assert_eq!(
        relation_snapshots("let a; let x=a;"),
        vec![RelationSnapshot {
            containing_binding: (11, 12, "x".to_owned()),
            reference: (13, 14, "a".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
        }]
    );
}

#[test]
fn escaped_direct_equality_and_non_normalization_match_the_oracles() {
    assert_eq!(
        relation_snapshots(r"let a=1; let x=\u0061;"),
        vec![RelationSnapshot {
            containing_binding: (13, 14, "x".to_owned()),
            reference: (15, 21, r"\u0061".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
        }]
    );

    assert_eq!(
        relation_snapshots("let é=1; let x=e\\u0301;"),
        vec![RelationSnapshot {
            containing_binding: (14, 15, "x".to_owned()),
            reference: (16, 23, r"e\u0301".to_owned()),
            semantic_name: "e\u{301}".to_owned(),
            target: None,
        }]
    );
}

#[test]
fn multiple_relations_preserve_reference_source_order() {
    assert_eq!(
        relation_snapshots("let a=1,b=2; let x=a,y=b;"),
        vec![
            RelationSnapshot {
                containing_binding: (17, 18, "x".to_owned()),
                reference: (19, 20, "a".to_owned()),
                semantic_name: "a".to_owned(),
                target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
            RelationSnapshot {
                containing_binding: (21, 22, "y".to_owned()),
                reference: (23, 24, "b".to_owned()),
                semantic_name: "b".to_owned(),
                target: Some((8, 9, "b".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
        ]
    );
}

#[test]
fn production_order_is_not_derived_from_source_offsets_or_pointer_identity() {
    assert!(!BINDING_SCOPE_SOURCE.contains(".range().start()"));
    assert!(!BINDING_SCOPE_SOURCE.contains("std::ptr"));
    assert!(BINDING_SCOPE_SOURCE.contains("binding_position"));
    assert!(BINDING_SCOPE_SOURCE.contains("containing_position"));
}

#[test]
fn production_does_not_store_plain_conditional_runtime_state() {
    let forbidden = [
        concat!("Initialized", "AtReference"),
        concat!("Uninitialized", "AtReference"),
        concat!("ScriptDefinitelyThrows", "ReferenceError"),
    ];

    for term in forbidden {
        assert!(
            !BINDING_SCOPE_SOURCE.contains(term),
            "source-only Binding / Scope result must not store premise-dependent runtime state: {term}"
        );
    }
}

#[test]
fn selected_source_without_identifier_reference_has_empty_analysis() {
    assert!(relation_snapshots("let x=1;").is_empty());
}

#[test]
fn static_rejected_sources_cannot_produce_an_accepted_analysis_witness() {
    let duplicate = recognized("let a=1; let a=2; let x=a;");
    assert!(matches!(
        evaluate_selected_static_semantics(&duplicate),
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateLexicalName { .. }
        )
    ));

    let reserved = recognized(r"let x=\u0069f;");
    assert!(
        reserved.declarations()[0].bindings()[0]
            .identifier_reference_initializer()
            .is_none()
    );
    assert!(
        reserved.declarations()[0].bindings()[0]
            .escaped_reserved_initializer_identifier()
            .is_some()
    );
    assert!(matches!(
        evaluate_selected_static_semantics(&reserved),
        SelectedStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::EscapedReservedWordInitializer { .. }
        )
    ));
}

#[test]
fn leading_plus_minus_direct_identifier_reference_unary_expression_composes_unchanged_with_binding_scope()
 {
    assert_eq!(
        relation_snapshots("let a; const x=-a;"),
        vec![RelationSnapshot {
            containing_binding: (13, 14, "x".to_owned()),
            reference: (16, 17, "a".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
        }]
    );

    assert_eq!(
        relation_snapshots("let a=-a;"),
        vec![RelationSnapshot {
            containing_binding: (4, 5, "a".to_owned()),
            reference: (7, 8, "a".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Same)),
        }]
    );

    assert_eq!(
        relation_snapshots("const x=+a; let a;"),
        vec![RelationSnapshot {
            containing_binding: (6, 7, "x".to_owned()),
            reference: (9, 10, "a".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((16, 17, "a".to_owned(), SelectedLexicalBindingOrder::After,)),
        }]
    );

    assert_eq!(
        relation_snapshots("const x=-z;"),
        vec![RelationSnapshot {
            containing_binding: (6, 7, "x".to_owned()),
            reference: (9, 10, "z".to_owned()),
            semantic_name: "z".to_owned(),
            target: None,
        }]
    );
}

#[test]
fn leading_plus_minus_identifier_reference_unary_expression_composes_unchanged_with_escaped_operand_and_binding_scope()
 {
    assert_eq!(
        relation_snapshots(r"let foo; const x=-\u{66}oo;"),
        vec![RelationSnapshot {
            containing_binding: (15, 16, "x".to_owned()),
            reference: (18, 26, r"\u{66}oo".to_owned()),
            semantic_name: "foo".to_owned(),
            target: Some((4, 7, "foo".to_owned(), SelectedLexicalBindingOrder::Before,)),
        }]
    );

    assert_eq!(
        relation_snapshots(r"let foo=-\u{66}oo;"),
        vec![RelationSnapshot {
            containing_binding: (4, 7, "foo".to_owned()),
            reference: (9, 17, r"\u{66}oo".to_owned()),
            semantic_name: "foo".to_owned(),
            target: Some((4, 7, "foo".to_owned(), SelectedLexicalBindingOrder::Same)),
        }]
    );

    assert_eq!(
        relation_snapshots(r"const x=+f\u{6F}o; let foo;"),
        vec![RelationSnapshot {
            containing_binding: (6, 7, "x".to_owned()),
            reference: (9, 17, r"f\u{6F}o".to_owned()),
            semantic_name: "foo".to_owned(),
            target: Some((23, 26, "foo".to_owned(), SelectedLexicalBindingOrder::After,)),
        }]
    );

    assert_eq!(
        relation_snapshots(r"const x=-\u{7A};"),
        vec![RelationSnapshot {
            containing_binding: (6, 7, "x".to_owned()),
            reference: (9, 15, r"\u{7A}".to_owned()),
            semantic_name: "z".to_owned(),
            target: None,
        }]
    );
}

#[test]
fn unsupported_parenthesized_initializer_never_enters_binding_scope_analysis() {
    let source = SourceText::new(SourceId::new(274), "let x=(y);".to_owned());
    assert!(matches!(
        recognize_selected_lexical_slice(&source),
        SelectedLexicalSliceOutcome::UnsupportedCoverage
    ));
}

// Issue #754: the two-`IdentifierReference` additive initializer widens this
// consumer's input to 0..2 facts per binding, consumed in exact authored
// reference order. `#752`/PR #753 independently proves the underlying
// bounded theorem; these tests seal only that this consumer composes it
// correctly (P16: never reordered by target declaration position).

#[test]
fn two_identifier_reference_additive_initializer_composes_two_relations_in_authored_order() {
    assert_eq!(
        relation_snapshots("let a; let b; const x = a + b;"),
        vec![
            RelationSnapshot {
                containing_binding: (20, 21, "x".to_owned()),
                reference: (24, 25, "a".to_owned()),
                semantic_name: "a".to_owned(),
                target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
            RelationSnapshot {
                containing_binding: (20, 21, "x".to_owned()),
                reference: (28, 29, "b".to_owned()),
                semantic_name: "b".to_owned(),
                target: Some((11, 12, "b".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
        ]
    );
}

/// Issue #779: the right-unary exactly-two `IdentifierReference` additive
/// initializer reaches this consumer through the unchanged `Two { first,
/// second }` carrier, producing exactly the same two ordered relations as
/// the plain `a + b` spelling above -- with the right unary sign excluded
/// from the second reference's anchor. No new relation type, relation
/// meaning, or ordering rule is introduced by the new syntax.
#[test]
fn right_unary_additive_initializer_composes_the_same_two_relations_as_plain_additive() {
    assert_eq!(
        relation_snapshots("let a; let b; const x = a+-b;"),
        vec![
            RelationSnapshot {
                containing_binding: (20, 21, "x".to_owned()),
                reference: (24, 25, "a".to_owned()),
                semantic_name: "a".to_owned(),
                target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
            RelationSnapshot {
                containing_binding: (20, 21, "x".to_owned()),
                // The `-` at 26 is recognition-time evidence only: the
                // second reference anchor starts at `b`, at 27.
                reference: (27, 28, "b".to_owned()),
                semantic_name: "b".to_owned(),
                target: Some((11, 12, "b".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
        ]
    );
}

/// Issue #785: the both-unary exactly-two `IdentifierReference` additive
/// initializer reaches this consumer through the unchanged `Two { first,
/// second }` carrier, producing exactly the same two ordered relations as
/// the plain `a + b` spelling above -- with both the left unary sign and the
/// right unary sign excluded from their respective reference anchors. No new
/// relation type, relation meaning, or ordering rule is introduced by the
/// new syntax.
#[test]
fn both_unary_additive_initializer_composes_the_same_two_relations_as_plain_additive() {
    assert_eq!(
        relation_snapshots("let a; let b; const x = +a+-b;"),
        vec![
            RelationSnapshot {
                containing_binding: (20, 21, "x".to_owned()),
                // The leading `+` at 24 is recognition-time evidence only:
                // the first reference anchor starts at `a`, at 25.
                reference: (25, 26, "a".to_owned()),
                semantic_name: "a".to_owned(),
                target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
            RelationSnapshot {
                containing_binding: (20, 21, "x".to_owned()),
                // The `-` at 27 is recognition-time evidence only: the
                // second reference anchor starts at `b`, at 28.
                reference: (28, 29, "b".to_owned()),
                semantic_name: "b".to_owned(),
                target: Some((11, 12, "b".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
        ]
    );
}

#[test]
fn two_identifier_reference_additive_initializer_order_is_authored_not_target_declaration_order() {
    // `b` is declared before `a`, but the reference order is `a` then `b`
    // (authored left-to-right operand order); target declaration order
    // would emit `b` first. Both targets independently resolve to `Before`,
    // so only authored order distinguishes the correct output.
    let relations = relation_snapshots("let b; let a; const x = a + b;");
    let semantic_names: Vec<_> = relations
        .iter()
        .map(|relation| relation.semantic_name.clone())
        .collect();
    assert_eq!(semantic_names, vec!["a".to_owned(), "b".to_owned()]);
    assert!(
        relations
            .iter()
            .all(|relation| relation.target.as_ref().map(|target| target.3)
                == Some(SelectedLexicalBindingOrder::Before))
    );
}

#[test]
fn two_identifier_reference_additive_initializer_first_and_second_target_combinations() {
    // first target / second no-target.
    assert_eq!(
        relation_snapshots("let a; const x = a + b;")
            .iter()
            .map(|relation| (relation.semantic_name.clone(), relation.target.is_some()))
            .collect::<Vec<_>>(),
        vec![("a".to_owned(), true), ("b".to_owned(), false)]
    );

    // first no-target / second target.
    assert_eq!(
        relation_snapshots("let b; const x = a + b;")
            .iter()
            .map(|relation| (relation.semantic_name.clone(), relation.target.is_some()))
            .collect::<Vec<_>>(),
        vec![("a".to_owned(), false), ("b".to_owned(), true)]
    );

    // both no-target.
    assert_eq!(
        relation_snapshots("const x = a + b;")
            .iter()
            .map(|relation| (relation.semantic_name.clone(), relation.target.is_some()))
            .collect::<Vec<_>>(),
        vec![("a".to_owned(), false), ("b".to_owned(), false)]
    );
}

#[test]
fn two_identifier_reference_additive_initializer_does_not_deduplicate_equal_semantic_names() {
    // `a + a`: two independently owned relations, not one.
    let relations = relation_snapshots("let a; const x = a + a;");
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name, "a");
    assert_eq!(relations[1].semantic_name, "a");
    // Equal fragment text and semantic name, but two distinct authored
    // source occurrences (different byte offsets): never collapsed to one.
    assert_eq!(relations[0].reference.2, relations[1].reference.2);
    assert_ne!(relations[0].reference.0, relations[1].reference.0);
    assert_eq!(
        relations.iter().map(|r| r.reference.0).collect::<Vec<_>>(),
        vec![17, 21]
    );
}

#[test]
fn two_identifier_reference_additive_initializer_direct_escaped_combinations_compose() {
    // Fixture text and expected authored spelling are built with `concat!`
    // over individually escaped fragments (matching PR #753's own
    // technique) so the literal backslash-u-hex bytes survive intact rather
    // than being collapsed by an intermediate write/review layer.
    let escaped_operand = concat!("\\", "u0061");
    let text = format!("let a; const x = {escaped_operand} + b;");
    let expected_escaped_fragment = escaped_operand.to_owned();

    assert_eq!(
        relation_snapshots(&text),
        vec![
            RelationSnapshot {
                containing_binding: (13, 14, "x".to_owned()),
                reference: (17, 23, expected_escaped_fragment),
                semantic_name: "a".to_owned(),
                target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
            },
            RelationSnapshot {
                containing_binding: (13, 14, "x".to_owned()),
                reference: (26, 27, "b".to_owned()),
                semantic_name: "b".to_owned(),
                target: None,
            },
        ]
    );
}

// Issue #791 (per #688 comment 5762579228): the one-reference /
// one-plain-decimal heterogeneous additive initializer reaches this
// consumer through the unchanged `One(reference)` carrier -- exactly one
// relation, in either orientation, with the Decimal operand and the binary
// operator never entering this consumer's input at all. #789/#790
// independently proves the underlying candidate-independent theorem; this
// test seals only that this consumer composes the new producer correctly.

#[test]
fn one_reference_one_plain_decimal_additive_initializer_composes_exactly_one_relation() {
    assert_eq!(
        relation_snapshots("let a; const x=a+1;"),
        vec![RelationSnapshot {
            containing_binding: (13, 14, "x".to_owned()),
            reference: (15, 16, "a".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
        }]
    );

    assert_eq!(
        relation_snapshots("let a; const x=1+a;"),
        vec![RelationSnapshot {
            containing_binding: (13, 14, "x".to_owned()),
            reference: (17, 18, "a".to_owned()),
            semantic_name: "a".to_owned(),
            target: Some((4, 5, "a".to_owned(), SelectedLexicalBindingOrder::Before,)),
        }]
    );
}
