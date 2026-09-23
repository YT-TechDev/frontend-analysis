use crate::{SourceAnchor, SourceId, SourceText};

use super::selected_lexical_slice::{
    SelectedLexicalSliceOutcome, SelectedOneLevelBlockScript, recognize_selected_lexical_slice,
};
use super::selected_one_level_block_binding_scope::{
    SelectedOneLevelBlockBindingScopeOutcome, SelectedOneLevelBlockBindingScopeRegion,
    SelectedOneLevelBlockBindingScopeTarget, analyze_selected_one_level_block_binding_scope,
};
use super::selected_static_semantics::{
    SelectedOneLevelBlockStaticSemanticsAccepted, SelectedOneLevelBlockStaticSemanticsOutcome,
    SelectedStaticSemanticsRejection, evaluate_selected_one_level_block_static_semantics,
};

const HIERARCHICAL_BINDING_SCOPE_SOURCE: &str =
    include_str!("selected_one_level_block_binding_scope.rs");

#[derive(Debug, Clone, PartialEq, Eq)]
struct AnchorSnapshot {
    start: usize,
    end: usize,
    fragment: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RegionSnapshot {
    TopLevel,
    Block(AnchorSnapshot),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TargetSnapshot {
    SelectedLexicalBinding {
        binding: AnchorSnapshot,
        region: RegionSnapshot,
    },
    NoSelectedLexicalBindingTargetInCoveredRegions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RelationSnapshot {
    containing_binding: AnchorSnapshot,
    current_region: RegionSnapshot,
    reference: AnchorSnapshot,
    semantic_name: String,
    target: TargetSnapshot,
}

fn anchor_snapshot(anchor: &SourceAnchor) -> AnchorSnapshot {
    AnchorSnapshot {
        start: anchor.range().start(),
        end: anchor.range().end(),
        fragment: anchor.fragment().to_owned(),
    }
}

fn region_snapshot(region: SelectedOneLevelBlockBindingScopeRegion<'_>) -> RegionSnapshot {
    match region {
        SelectedOneLevelBlockBindingScopeRegion::TopLevel => RegionSnapshot::TopLevel,
        SelectedOneLevelBlockBindingScopeRegion::Block(block) => {
            RegionSnapshot::Block(anchor_snapshot(block))
        }
    }
}

fn recognized(text: &str) -> SelectedOneLevelBlockScript {
    let source = SourceText::new(SourceId::new(304), text.to_owned());
    match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(script) => script,
        other => {
            panic!("expected one-level Block selected recognition for {text:?}, got {other:?}")
        }
    }
}

fn accepted(
    script: &SelectedOneLevelBlockScript,
) -> SelectedOneLevelBlockStaticSemanticsAccepted<'_> {
    match evaluate_selected_one_level_block_static_semantics(script) {
        SelectedOneLevelBlockStaticSemanticsOutcome::Accepted(accepted) => accepted,
        other => panic!("expected one-level Block static acceptance, got {other:?}"),
    }
}

fn relation_snapshots(text: &str) -> Vec<RelationSnapshot> {
    let script = recognized(text);
    let accepted = accepted(&script);
    let analysis = match analyze_selected_one_level_block_binding_scope(&accepted) {
        SelectedOneLevelBlockBindingScopeOutcome::Complete(analysis) => analysis,
        other => panic!("expected complete hierarchical Binding / Scope analysis, got {other:?}"),
    };

    analysis
        .relations()
        .iter()
        .map(|relation| {
            let target = match relation.target() {
                SelectedOneLevelBlockBindingScopeTarget::SelectedLexicalBinding {
                    binding,
                    region,
                } => TargetSnapshot::SelectedLexicalBinding {
                    binding: anchor_snapshot(binding),
                    region: region_snapshot(region),
                },
                SelectedOneLevelBlockBindingScopeTarget::NoSelectedLexicalBindingTargetInCoveredRegions => {
                    TargetSnapshot::NoSelectedLexicalBindingTargetInCoveredRegions
                }
            };

            RelationSnapshot {
                containing_binding: anchor_snapshot(relation.containing_binding()),
                current_region: region_snapshot(relation.current_region()),
                reference: anchor_snapshot(relation.reference()),
                semantic_name: relation.semantic_name().to_owned(),
                target,
            }
        })
        .collect()
}

fn anchor(start: usize, end: usize, fragment: &str) -> AnchorSnapshot {
    AnchorSnapshot {
        start,
        end,
        fragment: fragment.to_owned(),
    }
}

fn block(start: usize, end: usize, fragment: &str) -> RegionSnapshot {
    RegionSnapshot::Block(anchor(start, end, fragment))
}

fn target(start: usize, end: usize, fragment: &str, region: RegionSnapshot) -> TargetSnapshot {
    TargetSnapshot::SelectedLexicalBinding {
        binding: anchor(start, end, fragment),
        region,
    }
}

#[test]
fn accepted_witness_is_bound_to_the_exact_block_enabled_script() {
    let script = recognized("let a=1; { let x=a; }");
    let accepted = accepted(&script);
    assert!(std::ptr::eq(accepted.script(), &script));
}

#[test]
fn inner_shadowing_forward_shadowing_self_shadowing_and_outer_fallback_match_oracle() {
    let mixed_block = block(9, 30, "{ let a=2; let x=a; }");
    assert_eq!(
        relation_snapshots("let a=1; { let a=2; let x=a; } let y=a;"),
        vec![
            RelationSnapshot {
                containing_binding: anchor(24, 25, "x"),
                current_region: mixed_block.clone(),
                reference: anchor(26, 27, "a"),
                semantic_name: "a".to_owned(),
                target: target(15, 16, "a", mixed_block),
            },
            RelationSnapshot {
                containing_binding: anchor(35, 36, "y"),
                current_region: RegionSnapshot::TopLevel,
                reference: anchor(37, 38, "a"),
                semantic_name: "a".to_owned(),
                target: target(4, 5, "a", RegionSnapshot::TopLevel),
            },
        ]
    );

    let forward_block = block(9, 30, "{ let x=a; let a=1; }");
    assert_eq!(
        relation_snapshots("let a=0; { let x=a; let a=1; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(15, 16, "x"),
            current_region: forward_block.clone(),
            reference: anchor(17, 18, "a"),
            semantic_name: "a".to_owned(),
            target: target(24, 25, "a", forward_block),
        }]
    );

    let self_block = block(9, 21, "{ let x=x; }");
    assert_eq!(
        relation_snapshots("let x=0; { let x=x; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(15, 16, "x"),
            current_region: self_block.clone(),
            reference: anchor(17, 18, "x"),
            semantic_name: "x".to_owned(),
            target: target(15, 16, "x", self_block),
        }]
    );

    let outer_block = block(9, 21, "{ let x=a; }");
    assert_eq!(
        relation_snapshots("let a=1; { let x=a; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(15, 16, "x"),
            current_region: outer_block,
            reference: anchor(17, 18, "a"),
            semantic_name: "a".to_owned(),
            target: target(4, 5, "a", RegionSnapshot::TopLevel),
        }]
    );
}

#[test]
fn sibling_and_child_regions_are_excluded_from_the_search_path() {
    let second_block = block(13, 25, "{ let x=a; }");
    assert_eq!(
        relation_snapshots("{ let a=1; } { let x=a; } let a=2;"),
        vec![RelationSnapshot {
            containing_binding: anchor(19, 20, "x"),
            current_region: second_block,
            reference: anchor(21, 22, "a"),
            semantic_name: "a".to_owned(),
            target: target(30, 31, "a", RegionSnapshot::TopLevel),
        }]
    );

    assert_eq!(
        relation_snapshots("let x=a; { let a=1; } let a=2;"),
        vec![RelationSnapshot {
            containing_binding: anchor(4, 5, "x"),
            current_region: RegionSnapshot::TopLevel,
            reference: anchor(6, 7, "a"),
            semantic_name: "a".to_owned(),
            target: target(26, 27, "a", RegionSnapshot::TopLevel),
        }]
    );
}

#[test]
fn covered_path_no_target_is_distinct_from_same_source_binding_existence() {
    assert_eq!(
        relation_snapshots("{ let a=1; } { let x=a; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(19, 20, "x"),
            current_region: block(13, 25, "{ let x=a; }"),
            reference: anchor(21, 22, "a"),
            semantic_name: "a".to_owned(),
            target: TargetSnapshot::NoSelectedLexicalBindingTargetInCoveredRegions,
        }]
    );

    assert_eq!(
        relation_snapshots("{ let x=y; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(6, 7, "x"),
            current_region: block(0, 12, "{ let x=y; }"),
            reference: anchor(8, 9, "y"),
            semantic_name: "y".to_owned(),
            target: TargetSnapshot::NoSelectedLexicalBindingTargetInCoveredRegions,
        }]
    );
}

#[test]
fn block_enabled_positive_source_without_references_has_empty_analysis() {
    assert!(relation_snapshots("let a=1; { let a=2; }").is_empty());
}

#[test]
fn escaped_identity_and_non_normalization_match_the_candidate_independent_oracle() {
    assert_eq!(
        relation_snapshots(r"let a=1; { let x=\u0061; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(15, 16, "x"),
            current_region: block(9, 26, r"{ let x=\u0061; }"),
            reference: anchor(17, 23, r"\u0061"),
            semantic_name: "a".to_owned(),
            target: target(4, 5, "a", RegionSnapshot::TopLevel),
        }]
    );

    let escaped_inner = block(9, 35, r"{ let x=a; let \u{61}=1; }");
    assert_eq!(
        relation_snapshots(r"let a=0; { let x=a; let \u{61}=1; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(15, 16, "x"),
            current_region: escaped_inner.clone(),
            reference: anchor(17, 18, "a"),
            semantic_name: "a".to_owned(),
            target: target(24, 30, r"\u{61}", escaped_inner),
        }]
    );

    assert_eq!(
        relation_snapshots("let é=1; { let x=e\\u0301; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(16, 17, "x"),
            current_region: block(10, 28, r"{ let x=e\u0301; }"),
            reference: anchor(18, 25, r"e\u0301"),
            semantic_name: "e\u{301}".to_owned(),
            target: TargetSnapshot::NoSelectedLexicalBindingTargetInCoveredRegions,
        }]
    );
}

#[test]
fn multiple_relations_preserve_reference_traversal_order_across_regions() {
    assert_eq!(
        relation_snapshots("let a=1; let b=2; { let x=a; let y=z; let z=3; } let q=b;"),
        vec![
            RelationSnapshot {
                containing_binding: anchor(24, 25, "x"),
                current_region: block(18, 48, "{ let x=a; let y=z; let z=3; }"),
                reference: anchor(26, 27, "a"),
                semantic_name: "a".to_owned(),
                target: target(4, 5, "a", RegionSnapshot::TopLevel),
            },
            RelationSnapshot {
                containing_binding: anchor(33, 34, "y"),
                current_region: block(18, 48, "{ let x=a; let y=z; let z=3; }"),
                reference: anchor(35, 36, "z"),
                semantic_name: "z".to_owned(),
                target: target(42, 43, "z", block(18, 48, "{ let x=a; let y=z; let z=3; }")),
            },
            RelationSnapshot {
                containing_binding: anchor(53, 54, "q"),
                current_region: RegionSnapshot::TopLevel,
                reference: anchor(55, 56, "b"),
                semantic_name: "b".to_owned(),
                target: target(13, 14, "b", RegionSnapshot::TopLevel),
            },
        ]
    );
}

#[test]
fn leading_plus_minus_direct_identifier_reference_unary_expression_composes_unchanged_with_one_level_block_binding_scope()
 {
    let current_block = block(0, 22, "{ let a=1; let x=-a; }");
    assert_eq!(
        relation_snapshots("{ let a=1; let x=-a; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(15, 16, "x"),
            current_region: current_block.clone(),
            reference: anchor(18, 19, "a"),
            semantic_name: "a".to_owned(),
            target: target(6, 7, "a", current_block),
        }]
    );
}

#[test]
fn leading_plus_minus_identifier_reference_unary_expression_composes_unchanged_with_escaped_operand_and_one_level_block_binding_scope()
 {
    let current_block = block(0, 27, r"{ let a=1; let x=-\u{61}; }");
    assert_eq!(
        relation_snapshots(r"{ let a=1; let x=-\u{61}; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(15, 16, "x"),
            current_region: current_block.clone(),
            reference: anchor(18, 24, r"\u{61}"),
            semantic_name: "a".to_owned(),
            target: target(6, 7, "a", current_block),
        }]
    );
}

#[test]
fn static_rejection_remains_an_upstream_prerequisite_boundary() {
    let duplicate = recognized("{ let x=a; let a=1; let a=2; }");
    assert!(matches!(
        evaluate_selected_one_level_block_static_semantics(&duplicate),
        SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateBlockLexicalName { .. }
        )
    ));
}

// Issue #754: the two-`IdentifierReference` additive initializer widens
// this consumer's input to 0..2 facts per binding, consumed in exact
// authored reference order. `#752`/PR #753 independently proves the
// underlying bounded theorem; these tests seal only that this distinct
// one-level Block consumer composes it correctly.

#[test]
fn two_identifier_reference_additive_initializer_composes_two_relations_in_authored_order() {
    let current_block = block(14, 28, "{ let x=a+b; }");
    assert_eq!(
        relation_snapshots("let a; let b; { let x=a+b; }"),
        vec![
            RelationSnapshot {
                containing_binding: anchor(20, 21, "x"),
                current_region: current_block.clone(),
                reference: anchor(22, 23, "a"),
                semantic_name: "a".to_owned(),
                target: target(4, 5, "a", RegionSnapshot::TopLevel),
            },
            RelationSnapshot {
                containing_binding: anchor(20, 21, "x"),
                current_region: current_block,
                reference: anchor(24, 25, "b"),
                semantic_name: "b".to_owned(),
                target: target(11, 12, "b", RegionSnapshot::TopLevel),
            },
        ]
    );
}

/// Issue #779: the right-unary exactly-two `IdentifierReference` additive
/// initializer reaches this distinct Block consumer through the unchanged
/// `Two { first, second }` carrier, producing exactly the same two ordered
/// relations as the plain `a+b` spelling above, with the right unary sign
/// excluded from the second reference's anchor.
#[test]
fn right_unary_additive_initializer_composes_the_same_two_relations_as_plain_additive() {
    let current_block = block(14, 29, "{ let x=a+-b; }");
    assert_eq!(
        relation_snapshots("let a; let b; { let x=a+-b; }"),
        vec![
            RelationSnapshot {
                containing_binding: anchor(20, 21, "x"),
                current_region: current_block.clone(),
                reference: anchor(22, 23, "a"),
                semantic_name: "a".to_owned(),
                target: target(4, 5, "a", RegionSnapshot::TopLevel),
            },
            RelationSnapshot {
                containing_binding: anchor(20, 21, "x"),
                current_region: current_block,
                // The `-` at 24 is recognition-time evidence only.
                reference: anchor(25, 26, "b"),
                semantic_name: "b".to_owned(),
                target: target(11, 12, "b", RegionSnapshot::TopLevel),
            },
        ]
    );
}

/// Issue #785: the both-unary exactly-two `IdentifierReference` additive
/// initializer reaches this distinct Block consumer through the unchanged
/// `Two { first, second }` carrier, producing exactly the same two ordered
/// relations as the plain `a+b` spelling above, with both the left unary
/// sign and the right unary sign excluded from their respective reference
/// anchors.
#[test]
fn both_unary_additive_initializer_composes_the_same_two_relations_as_plain_additive() {
    let current_block = block(14, 30, "{ let x=+a+-b; }");
    assert_eq!(
        relation_snapshots("let a; let b; { let x=+a+-b; }"),
        vec![
            RelationSnapshot {
                containing_binding: anchor(20, 21, "x"),
                current_region: current_block.clone(),
                // The leading `+` at 22 is recognition-time evidence only.
                reference: anchor(23, 24, "a"),
                semantic_name: "a".to_owned(),
                target: target(4, 5, "a", RegionSnapshot::TopLevel),
            },
            RelationSnapshot {
                containing_binding: anchor(20, 21, "x"),
                current_region: current_block,
                // The `-` at 25 is recognition-time evidence only.
                reference: anchor(26, 27, "b"),
                semantic_name: "b".to_owned(),
                target: target(11, 12, "b", RegionSnapshot::TopLevel),
            },
        ]
    );
}

#[test]
fn two_identifier_reference_additive_initializer_does_not_deduplicate_equal_semantic_names() {
    let relations = relation_snapshots("let a; { let x=a+a; }");
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name, "a");
    assert_eq!(relations[1].semantic_name, "a");
    assert_ne!(relations[0].reference.start, relations[1].reference.start);
}

// Issue #797 (per #688 comment 5771773635): the exactly-three
// `IdentifierReference` additive initializer widens this distinct Block
// consumer's input to 0..3 facts per binding, consumed in exact authored
// reference order. This consumer's own logic is unchanged; #795/PR #796
// independently proves the underlying bounded three-fact theorem. This test
// mixes current-Block and TopLevel targets across the three facts, proving
// each independently resolves through existing region rules.

#[test]
fn three_identifier_reference_additive_initializer_composes_three_relations_in_authored_order() {
    let current_block = block(7, 41, "{ let a; let b; const x = a+b+c; }");
    assert_eq!(
        relation_snapshots("let c; { let a; let b; const x = a+b+c; }"),
        vec![
            RelationSnapshot {
                containing_binding: anchor(29, 30, "x"),
                current_region: current_block.clone(),
                reference: anchor(33, 34, "a"),
                semantic_name: "a".to_owned(),
                target: target(13, 14, "a", current_block.clone()),
            },
            RelationSnapshot {
                containing_binding: anchor(29, 30, "x"),
                current_region: current_block.clone(),
                reference: anchor(35, 36, "b"),
                semantic_name: "b".to_owned(),
                target: target(20, 21, "b", current_block.clone()),
            },
            RelationSnapshot {
                containing_binding: anchor(29, 30, "x"),
                current_region: current_block,
                reference: anchor(37, 38, "c"),
                semantic_name: "c".to_owned(),
                target: target(4, 5, "c", RegionSnapshot::TopLevel),
            },
        ]
    );
}

#[test]
fn three_identifier_reference_additive_initializer_does_not_deduplicate_equal_semantic_names() {
    let relations = relation_snapshots("let a; { let x=a+a+a; }");
    assert_eq!(relations.len(), 3);
    for relation in &relations {
        assert_eq!(relation.semantic_name, "a");
    }
    assert_ne!(relations[0].reference.start, relations[1].reference.start);
    assert_ne!(relations[1].reference.start, relations[2].reference.start);
    assert_ne!(relations[0].reference.start, relations[2].reference.start);
}

// Issue #803 (per #688 comment 5779735385): the unbounded 2..N
// `IdentifierReference` additive-chain initializer widens this distinct
// Block consumer's input to 0..N facts per binding, consumed in exact
// authored reference order. This consumer's own logic is unchanged;
// #801/PR #802 independently proves the underlying candidate-independent
// N-fact theorem. This test mixes current-Block and TopLevel targets across
// five facts (with one operand left deliberately undeclared), proving each
// independently resolves through existing region rules.

#[test]
fn many_identifier_reference_additive_initializer_composes_five_relations_in_authored_order() {
    let current_block = block(14, 52, "{ let a; let b; const x = a+b+c+d+e; }");
    assert_eq!(
        relation_snapshots("let c; let d; { let a; let b; const x = a+b+c+d+e; }"),
        vec![
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block.clone(),
                reference: anchor(40, 41, "a"),
                semantic_name: "a".to_owned(),
                target: target(20, 21, "a", current_block.clone()),
            },
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block.clone(),
                reference: anchor(42, 43, "b"),
                semantic_name: "b".to_owned(),
                target: target(27, 28, "b", current_block.clone()),
            },
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block.clone(),
                reference: anchor(44, 45, "c"),
                semantic_name: "c".to_owned(),
                target: target(4, 5, "c", RegionSnapshot::TopLevel),
            },
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block.clone(),
                reference: anchor(46, 47, "d"),
                semantic_name: "d".to_owned(),
                target: target(11, 12, "d", RegionSnapshot::TopLevel),
            },
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block,
                reference: anchor(48, 49, "e"),
                semantic_name: "e".to_owned(),
                target: TargetSnapshot::NoSelectedLexicalBindingTargetInCoveredRegions,
            },
        ]
    );
}

// Issue #811 (per #688 comment 5791317234): composing the accepted
// optional-leading-`+`/`-` `IdentifierReference` additive-chain theorem
// proven by #809/PR #810 widens this distinct Block consumer's input facts
// to independently plain-or-unary-wrapped operands, with no change to this
// consumer's own region-resolution logic. This test mixes current-Block and
// TopLevel targets across five optional-unary facts (with one operand left
// deliberately undeclared), proving each independently resolves through
// existing region rules and that the wrapper signs never enter the
// retained reference anchors.

#[test]
fn optional_unary_many_identifier_reference_additive_initializer_composes_five_relations_in_authored_order()
 {
    let current_block = block(14, 56, "{ let a; let b; const x = +a+-b+c- -d+e; }");
    assert_eq!(
        relation_snapshots("let c; let d; { let a; let b; const x = +a+-b+c- -d+e; }"),
        vec![
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block.clone(),
                reference: anchor(41, 42, "a"),
                semantic_name: "a".to_owned(),
                target: target(20, 21, "a", current_block.clone()),
            },
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block.clone(),
                reference: anchor(44, 45, "b"),
                semantic_name: "b".to_owned(),
                target: target(27, 28, "b", current_block.clone()),
            },
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block.clone(),
                reference: anchor(46, 47, "c"),
                semantic_name: "c".to_owned(),
                target: target(4, 5, "c", RegionSnapshot::TopLevel),
            },
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block.clone(),
                reference: anchor(50, 51, "d"),
                semantic_name: "d".to_owned(),
                target: target(11, 12, "d", RegionSnapshot::TopLevel),
            },
            RelationSnapshot {
                containing_binding: anchor(36, 37, "x"),
                current_region: current_block,
                reference: anchor(52, 53, "e"),
                semantic_name: "e".to_owned(),
                target: TargetSnapshot::NoSelectedLexicalBindingTargetInCoveredRegions,
            },
        ]
    );
}

// Issue #791 (per #688 comment 5762579228): the one-reference /
// one-plain-decimal heterogeneous additive initializer reaches this
// distinct Block consumer through the unchanged `One(reference)` carrier --
// exactly one relation, in either orientation, with unchanged region
// semantics.

#[test]
fn one_reference_one_plain_decimal_additive_initializer_composes_exactly_one_relation() {
    let current_block = block(7, 21, "{ let x=a+1; }");
    assert_eq!(
        relation_snapshots("let a; { let x=a+1; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(13, 14, "x"),
            current_region: current_block,
            reference: anchor(15, 16, "a"),
            semantic_name: "a".to_owned(),
            target: target(4, 5, "a", RegionSnapshot::TopLevel),
        }]
    );

    let current_block = block(7, 21, "{ let x=1+a; }");
    assert_eq!(
        relation_snapshots("let a; { let x=1+a; }"),
        vec![RelationSnapshot {
            containing_binding: anchor(13, 14, "x"),
            current_region: current_block,
            reference: anchor(17, 18, "a"),
            semantic_name: "a".to_owned(),
            target: target(4, 5, "a", RegionSnapshot::TopLevel),
        }]
    );
}

#[test]
fn production_remains_private_source_level_and_does_not_reuse_flat_no_target_or_order_types() {
    for forbidden in [
        "SelectedLexicalBindingOrder",
        "NoSameSourceSelectedLexicalBinding",
        "RegionId",
        "ScopeId",
        "std::ptr",
        ".range().start()",
    ] {
        assert!(
            !HIERARCHICAL_BINDING_SCOPE_SOURCE.contains(forbidden),
            "hierarchical production crossed its accepted private source-level boundary: {forbidden}"
        );
    }
}
