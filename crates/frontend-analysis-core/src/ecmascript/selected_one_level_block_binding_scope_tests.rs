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
        other => panic!("expected one-level Block selected recognition for {text:?}, got {other:?}"),
    }
}

fn accepted(script: &SelectedOneLevelBlockScript) -> SelectedOneLevelBlockStaticSemanticsAccepted<'_> {
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

fn target(
    start: usize,
    end: usize,
    fragment: &str,
    region: RegionSnapshot,
) -> TargetSnapshot {
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
                target: target(
                    42,
                    43,
                    "z",
                    block(18, 48, "{ let x=a; let y=z; let z=3; }"),
                ),
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
fn static_rejection_remains_an_upstream_prerequisite_boundary() {
    let duplicate = recognized("{ let x=a; let a=1; let a=2; }");
    assert!(matches!(
        evaluate_selected_one_level_block_static_semantics(&duplicate),
        SelectedOneLevelBlockStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateBlockLexicalName { .. }
        )
    ));
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
