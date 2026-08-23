use crate::{SourceAnchor, SourceId, SourceText};

use super::selected_lexical_slice::{
    SelectedLexicalSliceOutcome, SelectedVariableStatementScript, recognize_selected_lexical_slice,
};
use super::selected_static_semantics::{
    SelectedStaticSemanticsRejection, SelectedVariableStatementStaticSemanticsOutcome,
    evaluate_selected_variable_statement_static_semantics,
};
use super::selected_variable_statement_name_correspondence::{
    SelectedVariableStatementNameCorrespondenceAnalysis,
    SelectedVariableStatementNameCorrespondenceOutcome,
    SelectedVariableStatementNameCorrespondenceRegion,
    analyze_selected_variable_statement_name_correspondence,
};

fn source(text: &str) -> SourceText {
    SourceText::new(SourceId::new(316), text.to_owned())
}

fn recognized_variable(text: &str) -> (SourceText, SelectedVariableStatementScript) {
    let source = source(text);
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(script) => script,
        other => panic!("expected var-enabled recognition for {text:?}, got {other:?}"),
    };
    (source, script)
}

fn accepted_analysis<'script>(
    script: &'script SelectedVariableStatementScript,
) -> SelectedVariableStatementNameCorrespondenceAnalysis<'script> {
    let accepted = match evaluate_selected_variable_statement_static_semantics(script) {
        SelectedVariableStatementStaticSemanticsOutcome::Accepted(accepted) => accepted,
        other => panic!("expected var-enabled selected static acceptance, got {other:?}"),
    };

    match analyze_selected_variable_statement_name_correspondence(&accepted) {
        SelectedVariableStatementNameCorrespondenceOutcome::Complete(analysis) => analysis,
        other => panic!("expected complete selected var-name correspondence, got {other:?}"),
    }
}

fn range(anchor: &SourceAnchor) -> (usize, usize) {
    (anchor.range().start(), anchor.range().end())
}

fn one_relation<'analysis, 'script>(
    analysis: &'analysis SelectedVariableStatementNameCorrespondenceAnalysis<'script>,
) -> &'analysis super::selected_variable_statement_name_correspondence::SelectedVariableStatementNameCorrespondenceRelation<'script>
{
    let [relation] = analysis.relations() else {
        panic!("expected exactly one correspondence relation");
    };
    relation
}

#[test]
fn top_level_var_correspondence_is_whole_script_and_preserves_every_authored_contributor() {
    for (text, expected_contributors) in [
        ("var a; let x=a;", &[(4, 5)][..]),
        ("let x=a; var a;", &[(13, 14)][..]),
        ("var a; var a; let x=a;", &[(4, 5), (11, 12)][..]),
        ("var a=1; let x=a;", &[(4, 5)][..]),
        ("var a=1,a; let x=a;", &[(4, 5), (8, 9)][..]),
    ] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        let relation = one_relation(&analysis);
        assert_eq!(relation.semantic_name(), "a", "{text}");
        assert_eq!(relation.reference().fragment(), "a", "{text}");
        assert!(matches!(
            relation.current_region(),
            SelectedVariableStatementNameCorrespondenceRegion::TopLevel
        ));

        let contributors = relation
            .correspondence()
            .var_contributors()
            .expect("top-level var contributor relation");
        assert_eq!(contributors.len(), expected_contributors.len(), "{text}");
        for (actual, expected) in contributors.iter().zip(expected_contributors) {
            assert_eq!(range(actual), *expected, "{text}");
        }
    }
}

#[test]
fn escaped_and_direct_spellings_share_exact_semantic_name_without_normalization() {
    for (text, expected_fragments) in [
        (r"var \u0061; let x=a;", &[r"\u0061"][..]),
        (r"var a; let x=\u0061;", &["a"][..]),
        (r"var a; var \u0061; let x=a;", &["a", r"\u0061"][..]),
        (r"var a=1,\u0061=2; let x=a;", &["a", r"\u0061"][..]),
    ] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        let relation = one_relation(&analysis);
        assert_eq!(relation.semantic_name(), "a", "{text}");
        let contributors = relation
            .correspondence()
            .var_contributors()
            .expect("same-name var contributors");
        assert_eq!(contributors.len(), expected_fragments.len(), "{text}");
        for (actual, expected) in contributors.iter().zip(expected_fragments) {
            assert_eq!(actual.fragment(), *expected, "{text}");
        }
    }

    let (_, script) = recognized_variable("var é; let x=e\u{301};");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "e\u{301}");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn block_region_falls_back_to_var_but_current_or_top_lexical_binding_wins() {
    let (_, script) = recognized_variable("var a; { let x=a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("Block must fall back to Script var contributor");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));

    for (text, expected_binding_range) in [
        ("var a; { let a=1; let x=a; }", (13, 14)),
        ("var a; { let x=a; let a=1; }", (22, 23)),
        ("var a; { let a=a; }", (13, 14)),
    ] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        let relation = one_relation(&analysis);
        let (binding, region) = relation
            .correspondence()
            .selected_lexical_binding()
            .expect("current Block lexical binding must win");
        assert_eq!(range(binding), expected_binding_range, "{text}");
        assert!(matches!(
            region,
            SelectedVariableStatementNameCorrespondenceRegion::Block(_)
        ));
        assert!(relation.correspondence().var_contributors().is_none());
    }

    let (_, script) = recognized_variable("let a=1; { let x=a; } var b;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("top-level lexical fallback");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn sibling_blocks_are_excluded_and_top_level_never_sees_block_local_lexical_bindings() {
    let (_, script) = recognized_variable("{ let a=1; } var a; { let x=a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("sibling lexical must be excluded before Script var fallback");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (17, 18));

    let (_, script) = recognized_variable("var a; { let a=1; } let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("top-level reference must not see Block-local lexical binding");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));
}

#[test]
fn no_selected_same_source_contributor_and_zero_relation_are_complete_source_claims_only() {
    let (_, script) = recognized_variable("let x=y; var a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "y");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    for text in ["var a; var a;", "var a; var a", "var a", "var a=1"] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }
}

#[test]
fn every_authored_var_contributor_survives_mixed_spelling_without_deduplication() {
    let (_, script) = recognized_variable(r"var a; var \u0061; var a; let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("all authored contributors must survive");
    assert_eq!(contributors.len(), 3);
    assert_eq!(contributors[0].fragment(), "a");
    assert_eq!(contributors[1].fragment(), r"\u0061");
    assert_eq!(contributors[2].fragment(), "a");
    assert_eq!(range(contributors[0]), (4, 5));
    assert_eq!(range(contributors[1]), (11, 17));
    assert_eq!(range(contributors[2]), (23, 24));
}

#[test]
fn static_rejection_prevents_correspondence_witness_construction_and_partial_relations() {
    for text in [
        "var a; let a;",
        "var a; { let a; let a; }",
        r"var \u0069f; let x=a;",
        "var a; { let x=a; let x=1; }",
    ] {
        let (_, script) = recognized_variable(text);
        assert!(matches!(
            evaluate_selected_variable_statement_static_semantics(&script),
            SelectedVariableStatementStaticSemanticsOutcome::Rejected(_)
        ));
    }

    let (_, script) = recognized_variable("var a; let a;");
    assert!(matches!(
        evaluate_selected_variable_statement_static_semantics(&script),
        SelectedVariableStatementStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. }
        )
    ));

    let (_, script) = recognized_variable("var a; { let a; let a; }");
    assert!(matches!(
        evaluate_selected_variable_statement_static_semantics(&script),
        SelectedVariableStatementStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateBlockLexicalName { .. }
        )
    ));

    let (_, script) = recognized_variable(r"var \u0069f; let x=a;");
    assert!(matches!(
        evaluate_selected_variable_statement_static_semantics(&script),
        SelectedVariableStatementStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::EscapedReservedWord { .. }
        )
    ));
}

#[test]
fn unsupported_and_incomplete_sources_never_reach_var_correspondence() {
    for text in [
        "var a=true; let x=a;",
        "var a; let x=(a);",
        "var a; { let x=a;",
    ] {
        let source = source(text);
        assert!(matches!(
            recognize_selected_lexical_slice(&source),
            SelectedLexicalSliceOutcome::UnsupportedCoverage
        ));
    }
}

#[test]
fn non_var_block_source_remains_owned_by_existing_block_enabled_partition() {
    let source = source("let a=1; { let x=a; } { let x=a; }");
    assert!(matches!(
        recognize_selected_lexical_slice(&source),
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(_)
    ));
}

#[test]
fn relation_keeps_containing_binding_reference_and_region_provenance() {
    let (_, script) = recognized_variable("var a; { let x=a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.containing_binding().fragment(), "x");
    assert_eq!(relation.reference().fragment(), "a");
    assert_eq!(range(relation.containing_binding()), (13, 14));
    assert_eq!(range(relation.reference()), (15, 16));
    match relation.current_region() {
        SelectedVariableStatementNameCorrespondenceRegion::Block(block) => {
            assert_eq!(block.fragment(), "{ let x=a; }");
            assert_eq!(range(block), (7, 19));
        }
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel => {
            panic!("reference must remain inside the selected Block region")
        }
    }
}

#[test]
fn capability_source_preserves_isolation_runtime_boundaries_and_complete_only_relations() {
    let production = include_str!("selected_variable_statement_name_correspondence.rs");

    for forbidden in [
        "selected_binding_scope",
        "selected_one_level_block_binding_scope",
        "first_var_by_name",
        "HashSet",
        "Before / Same / After,",
        "partial_relations",
    ] {
        assert!(
            !production.contains(forbidden),
            "selected var correspondence must preserve architecture boundary: found {forbidden}"
        );
    }

    assert!(production.contains("SameSourceSelectedVarNameContributors"));
    assert!(production.contains("NoSelectedSameSourceContributor"));
    assert!(production.contains("ResourceLimited"));
    assert!(production.contains("InternalFailure"));
    assert!(production.contains("authored `VariableDeclaration`"));
    assert!(production.contains("not runtime unresolvability"));
}

#[test]
fn declarators_of_one_statement_each_contribute_an_anchor_in_authored_order() {
    for (text, expected_contributors) in [
        ("var a,a; let x=a;", &[(4, 5), (6, 7)][..]),
        ("var a,b,a; let x=a;", &[(4, 5), (8, 9)][..]),
        ("var a,a,a; let x=a;", &[(4, 5), (6, 7), (8, 9)][..]),
        ("var a,b; let x=b;", &[(6, 7)][..]),
    ] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        let relation = one_relation(&analysis);
        let contributors = relation
            .correspondence()
            .var_contributors()
            .expect("within-statement var contributors");
        assert_eq!(contributors.len(), expected_contributors.len(), "{text}");
        for (actual, expected) in contributors.iter().zip(expected_contributors) {
            assert_eq!(range(actual), *expected, "{text}");
        }
    }
}

#[test]
fn escaped_and_direct_declarators_of_one_statement_keep_distinct_authored_anchors() {
    let (_, script) = recognized_variable(r"var a,\u0061; let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("mixed-spelling var contributors");
    assert_eq!(contributors.len(), 2);
    assert_eq!(contributors[0].fragment(), "a");
    assert_eq!(contributors[1].fragment(), r"\u0061");
    assert_eq!(range(contributors[0]), (4, 5));
    assert_eq!(range(contributors[1]), (6, 12));

    let (_, script) = recognized_variable(r"var é,e\u0301; let x=é;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "é");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("non-normalized var contributors");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 6));
}

#[test]
fn within_statement_and_cross_statement_contributors_compose_in_authored_order() {
    let (_, script) = recognized_variable("var a,b; var a; let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("composed var contributors");
    assert_eq!(contributors.len(), 2);
    assert_eq!(range(contributors[0]), (4, 5));
    assert_eq!(range(contributors[1]), (13, 14));

    let (_, script) = recognized_variable("var a; var b,a; let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("composed var contributors");
    assert_eq!(contributors.len(), 2);
    assert_eq!(range(contributors[0]), (4, 5));
    assert_eq!(range(contributors[1]), (13, 14));
}

#[test]
fn multi_declarator_statements_do_not_change_existing_relation_meanings() {
    let (_, script) = recognized_variable("var a,a; { let x=a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("Block reference falls back to Script var contributors");
    assert_eq!(contributors.len(), 2);
    assert_eq!(range(contributors[0]), (4, 5));
    assert_eq!(range(contributors[1]), (6, 7));

    let (_, script) = recognized_variable("var a,a; { let a=1; let x=a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("current Block lexical binding must still win");
    assert_eq!(range(binding), (15, 16));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert!(relation.correspondence().var_contributors().is_none());

    let (_, script) = recognized_variable("var a,b; let x=c;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    for text in ["var a,b;", "var a,b", "var a,b; var c,d"] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }
}

#[test]
fn direct_var_initializer_relations_preserve_whole_source_correspondence_meanings() {
    for (text, expected_binding, expected_reference, expected_contributors) in [
        ("var a; var x=a;", (11, 12), (13, 14), &[(4, 5)][..]),
        ("var x=a; var a;", (4, 5), (6, 7), &[(13, 14)][..]),
        ("var a=a;", (4, 5), (6, 7), &[(4, 5)][..]),
        (
            "var a; var a; var x=a;",
            (18, 19),
            (20, 21),
            &[(4, 5), (11, 12)][..],
        ),
        ("var a,a=a;", (6, 7), (8, 9), &[(4, 5), (6, 7)][..]),
        ("var x=a,a;", (4, 5), (6, 7), &[(8, 9)][..]),
    ] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        let relation = one_relation(&analysis);
        assert_eq!(
            range(relation.containing_binding()),
            expected_binding,
            "{text}"
        );
        assert_eq!(range(relation.reference()), expected_reference, "{text}");
        assert_eq!(relation.semantic_name(), "a", "{text}");
        assert!(matches!(
            relation.current_region(),
            SelectedVariableStatementNameCorrespondenceRegion::TopLevel
        ));
        let contributors = relation
            .correspondence()
            .var_contributors()
            .expect("direct var RHS must retain all authored same-name contributors");
        assert_eq!(contributors.len(), expected_contributors.len(), "{text}");
        for (actual, expected) in contributors.iter().zip(expected_contributors) {
            assert_eq!(range(actual), *expected, "{text}");
        }
    }
}

#[test]
fn direct_var_initializer_lexical_and_no_contributor_meanings_are_unchanged() {
    let (_, script) = recognized_variable("let a; var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(range(relation.containing_binding()), (11, 12));
    assert_eq!(range(relation.reference()), (13, 14));
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("top-level lexical binding must win");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));

    let (_, script) = recognized_variable("var x=z;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "z");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn direct_var_initializer_semantic_identity_uses_exact_direct_source_without_normalization() {
    let (_, script) = recognized_variable(r"var \u0061; var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    assert_eq!(relation.reference().fragment(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("escaped LHS semantic name must match direct RHS");
    assert_eq!(contributors.len(), 1);
    assert_eq!(contributors[0].fragment(), r"\u0061");

    let (_, script) = recognized_variable("var é; var x=e\u{301};");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "e\u{301}");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn direct_var_multi_reference_relations_remain_distinct_and_authored_ordered() {
    let (_, script) = recognized_variable("let a; var b; var x=a,y=b,z=q;");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 3);
    assert_eq!(
        relations
            .iter()
            .map(|relation| relation.semantic_name())
            .collect::<Vec<_>>(),
        ["a", "b", "q"]
    );
    assert_eq!(range(relations[0].containing_binding()), (18, 19));
    assert_eq!(range(relations[0].reference()), (20, 21));
    assert_eq!(range(relations[1].containing_binding()), (22, 23));
    assert_eq!(range(relations[1].reference()), (24, 25));
    assert_eq!(range(relations[2].containing_binding()), (26, 27));
    assert_eq!(range(relations[2].reference()), (28, 29));

    let (lexical, region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("first relation must target top-level lexical a");
    assert_eq!(range(lexical), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    let contributors = relations[1]
        .correspondence()
        .var_contributors()
        .expect("second relation must use authored var b contributor");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (11, 12));
    assert!(
        relations[2]
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn direct_var_static_rejection_stops_before_correspondence() {
    let (_, script) = recognized_variable("let a; var a; var x=a;");
    assert!(matches!(
        evaluate_selected_variable_statement_static_semantics(&script),
        SelectedVariableStatementStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. }
        )
    ));
}

#[test]
fn escaped_var_initializer_relations_use_decoded_identity_and_exact_authored_reference() {
    for (text, expected_binding, expected_reference, expected_contributors) in [
        (r"var a; var x=\u0061;", (11, 12), (13, 19), &[(4, 5)][..]),
        (r"var x=\u0061; var a;", (4, 5), (6, 12), &[(18, 19)][..]),
        (r"var a=\u0061;", (4, 5), (6, 12), &[(4, 5)][..]),
        (
            r"var a; var a; var x=\u0061;",
            (18, 19),
            (20, 26),
            &[(4, 5), (11, 12)][..],
        ),
        (r"var a,x=\u0061;", (6, 7), (8, 14), &[(4, 5)][..]),
    ] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        let relation = one_relation(&analysis);
        assert_eq!(
            range(relation.containing_binding()),
            expected_binding,
            "{text}"
        );
        assert_eq!(range(relation.reference()), expected_reference, "{text}");
        assert_eq!(relation.reference().fragment(), r"\u0061", "{text}");
        assert_eq!(relation.semantic_name(), "a", "{text}");
        let contributors = relation
            .correspondence()
            .var_contributors()
            .expect("escaped var RHS must preserve all authored same-name contributors");
        assert_eq!(contributors.len(), expected_contributors.len(), "{text}");
        for (actual, expected) in contributors.iter().zip(expected_contributors) {
            assert_eq!(range(actual), *expected, "{text}");
        }
    }
}

#[test]
fn escaped_var_initializer_lexical_priority_no_match_and_non_normalization_are_unchanged() {
    let (_, script) = recognized_variable(r"let a; var x=\u0061;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.reference().fragment(), r"\u0061");
    assert_eq!(relation.semantic_name(), "a");
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("top-level lexical binding must win for escaped var RHS");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));

    let (_, script) = recognized_variable(r"var x=\u007A;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.reference().fragment(), r"\u007A");
    assert_eq!(relation.semantic_name(), "z");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_variable(r"var \u0061; var x=\u0061;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("equal decoded identity must match escaped contributor");
    assert_eq!(contributors.len(), 1);
    assert_eq!(contributors[0].fragment(), r"\u0061");

    let (_, script) = recognized_variable("var é; var x=e\\u0301;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.reference().fragment(), r"e\u0301");
    assert_eq!(relation.semantic_name(), "e\u{301}");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn escaped_and_direct_var_initializer_relations_remain_distinct_and_authored_ordered() {
    let (_, script) = recognized_variable(r"let a; var b; var x=\u0061,y=b,z=\u0071;");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 3);
    assert_eq!(
        relations
            .iter()
            .map(|relation| relation.semantic_name())
            .collect::<Vec<_>>(),
        ["a", "b", "q"]
    );
    assert_eq!(range(relations[0].containing_binding()), (18, 19));
    assert_eq!(range(relations[0].reference()), (20, 26));
    assert_eq!(relations[0].reference().fragment(), r"\u0061");
    assert_eq!(range(relations[1].containing_binding()), (27, 28));
    assert_eq!(range(relations[1].reference()), (29, 30));
    assert_eq!(range(relations[2].containing_binding()), (31, 32));
    assert_eq!(range(relations[2].reference()), (33, 39));
    assert_eq!(relations[2].reference().fragment(), r"\u0071");

    let (lexical, region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("escaped relation must target top-level lexical a");
    assert_eq!(range(lexical), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    let contributors = relations[1]
        .correspondence()
        .var_contributors()
        .expect("direct relation must preserve authored var b contributor");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (11, 12));
    assert!(
        relations[2]
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn escaped_var_static_rejection_still_stops_before_correspondence() {
    let (_, script) = recognized_variable(r"let a; var a; var x=\u0061;");
    assert!(matches!(
        evaluate_selected_variable_statement_static_semantics(&script),
        SelectedVariableStatementStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. }
        )
    ));
}
