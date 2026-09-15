use crate::{SourceAnchor, SourceId, SourceText};

use super::selected_lexical_slice::{
    SelectedLexicalSliceOutcome, SelectedOneLevelBlockScript, SelectedVariableStatementScript,
    recognize_selected_lexical_slice,
};
use super::selected_static_semantics::{
    SelectedOneLevelBlockStaticSemanticsOutcome, SelectedStaticSemanticsRejection,
    SelectedVariableStatementStaticSemanticsOutcome,
    evaluate_selected_one_level_block_static_semantics,
    evaluate_selected_variable_statement_static_semantics,
};
use super::selected_variable_statement_name_correspondence::{
    SelectedVariableStatementNameCorrespondenceAnalysis,
    SelectedVariableStatementNameCorrespondenceOutcome,
    SelectedVariableStatementNameCorrespondenceRegion,
    analyze_selected_one_level_block_name_correspondence,
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

fn recognized_one_level_block(text: &str) -> (SourceText, SelectedOneLevelBlockScript) {
    let source = source(text);
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(script) => script,
        other => panic!("expected one-level Block recognition for {text:?}, got {other:?}"),
    };
    (source, script)
}

fn accepted_one_level_block_analysis<'script>(
    script: &'script SelectedOneLevelBlockScript,
) -> SelectedVariableStatementNameCorrespondenceAnalysis<'script> {
    let accepted = match evaluate_selected_one_level_block_static_semantics(script) {
        SelectedOneLevelBlockStaticSemanticsOutcome::Accepted(accepted) => accepted,
        other => panic!("expected one-level Block selected static acceptance, got {other:?}"),
    };

    match analyze_selected_one_level_block_name_correspondence(&accepted) {
        SelectedVariableStatementNameCorrespondenceOutcome::Complete(analysis) => analysis,
        other => panic!("expected complete one-level Block var-name correspondence, got {other:?}"),
    }
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

// Issue #703: `SameSourceSelectedVarNameContributors` widens its eligible
// contributor domain from selected top-level `VariableStatement` declarators
// only to every selected authored `var` declarator in the Script, including
// selected one-level Block-contained `var` declarators. The fixtures below
// exercise that widened domain through the real production entry point
// (`analyze_selected_variable_statement_name_correspondence`), matching the
// candidate-independent oracle frozen by #701/PR #702
// (`selected_all_var_contributor_correspondence_frontier.rs`, F1-F12/G/K1/K2/
// SR1). A few oracle sources have no top-level `var` statement and are
// therefore not `RecognizedVariableStatementSlice` on their own (this
// capability consumes only the var-enabled witness); those are extended with
// one harmless, distinctly-named top-level `var` declaration so the same
// Block/lexical shape reaches this production module, exactly as the
// existing `block_region_falls_back_to_var_but_current_or_top_lexical_binding_wins`
// test already does for its own top-level-lexical-fallback fixture.

#[test]
fn same_block_var_contributor_is_visible_to_block_lexical_top_level_lexical_and_top_level_var_references()
 {
    // F1 (kills W1, W3): a Block-only `var` contributor is seen by a Block
    // lexical reference in the very same Block. `var q;` is appended only to
    // reach `RecognizedVariableStatementSlice`.
    let (_, script) = recognized_variable("var q; { var a; let x=a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("same-Block var contributor must be visible to the Block lexical reference");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (13, 14));

    // F3 (kills W2): the same Block-only contributor is seen by an existing
    // top-level lexical reference.
    let (_, script) = recognized_variable("var q; { var a; } let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("Block var contributor must be visible to a top-level lexical reference");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (13, 14));

    // F4 (kills W2): the same Block-only contributor is seen by an existing
    // top-level var reference; no new query-input type is introduced.
    let (_, script) = recognized_variable("{ var a; } var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("Block var contributor must be visible to a top-level var reference");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (6, 7));
}

#[test]
fn sibling_block_var_contributor_is_script_wide_provenance_not_block_local_lookup() {
    // F2 (kills W4, W5): the contributor and the reference live in
    // different, sibling Blocks.
    let (_, script) = recognized_variable("var q; { var a; } { let x=a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("sibling-Block var contributor must remain Script-wide provenance");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (13, 14));
}

#[test]
fn later_block_var_contributor_is_whole_source_provenance_not_temporally_filtered() {
    // F5 (kills W6): the reference is authored strictly before its only
    // contributor, which is authored later inside a Block.
    let (_, script) = recognized_variable("var x=a; { var a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("a later-authored Block var contributor must still be eligible");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (15, 16));
    assert!(relation.reference().range().start() < contributors[0].range().start());
}

#[test]
fn mixed_top_level_and_block_contributor_order_follows_exact_authored_occurrence() {
    // F6 (kills W7, W8): top-level contributor authored first.
    let (_, script) = recognized_variable("var a; { var a; } var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("mixed top-level/Block contributors");
    assert_eq!(contributors.len(), 2);
    assert_eq!(range(contributors[0]), (4, 5));
    assert_eq!(range(contributors[1]), (13, 14));

    // F6I: the inverse authored arrangement swaps which anchor leads,
    // defeating any region-grouped collection strategy.
    let (_, script) = recognized_variable("{ var a; } var a; var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("inverse mixed top-level/Block contributors");
    assert_eq!(contributors.len(), 2);
    assert_eq!(range(contributors[0]), (6, 7));
    assert_eq!(range(contributors[1]), (15, 16));
}

#[test]
fn repeated_block_var_contributors_and_cross_placement_multiplicity_are_never_deduplicated() {
    // F7 (kills W9): two declarators of one Block `var` statement remain two
    // distinct authored occurrences of the same semantic name.
    let (_, script) = recognized_variable("{ var a,a; } var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("repeated same-statement Block var contributors");
    assert_eq!(contributors.len(), 2);
    assert_eq!(range(contributors[0]), (6, 7));
    assert_eq!(range(contributors[1]), (8, 9));

    // F8: four occurrences across top-level and Block placements remain four
    // occurrences, in exact authored order, with no set semantics.
    let (_, script) = recognized_variable("var a; { var a,a; } var a; var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("cross-placement repeated contributors");
    assert_eq!(contributors.len(), 4);
    assert_eq!(range(contributors[0]), (4, 5));
    assert_eq!(range(contributors[1]), (13, 14));
    assert_eq!(range(contributors[2]), (15, 16));
    assert_eq!(range(contributors[3]), (24, 25));
}

#[test]
fn block_var_cardinality_first_interior_final_declarators_all_independently_contribute() {
    // G (kills W10 first-only, W11 final-only): three declarators of one
    // Block `var` statement each independently resolve as the sole
    // contributor for their own top-level var reference.
    let (_, script) = recognized_variable("{ var a,b,c; } var p=a; var q=b; var r=c;");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 3);

    let a_contributors = relations[0]
        .correspondence()
        .var_contributors()
        .expect("first Block var declarator contributes");
    assert_eq!(a_contributors.len(), 1);
    assert_eq!(range(a_contributors[0]), (6, 7));

    let b_contributors = relations[1]
        .correspondence()
        .var_contributors()
        .expect("interior Block var declarator contributes");
    assert_eq!(b_contributors.len(), 1);
    assert_eq!(range(b_contributors[0]), (8, 9));

    let c_contributors = relations[2]
        .correspondence()
        .var_contributors()
        .expect("final Block var declarator contributes");
    assert_eq!(c_contributors.len(), 1);
    assert_eq!(range(c_contributors[0]), (10, 11));
}

#[test]
fn decimal_initialized_block_var_contributes_via_lhs_only() {
    // K1 (kills W12, W13): a decimal-initialized Block var contributes
    // through its LHS `BindingIdentifier`; the decimal RHS anchor `(8, 9)`
    // never appears as a contributor or reference anchor.
    let (_, script) = recognized_variable("{ var a=1; } var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("decimal-initialized Block var contributes via LHS");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (6, 7));
    assert_ne!(range(contributors[0]), (8, 9));

    // K2: a bare declarator sharing a Block `var` statement with a
    // decimal-initialized sibling contributes identically through its own
    // LHS.
    let (_, script) = recognized_variable("{ var a=1,b; } var x=b;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("bare declarator sharing a statement with a decimal-initialized sibling");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (10, 11));
}

#[test]
fn escaped_block_var_lhs_uses_semantic_equality_with_exact_authored_anchor() {
    // F9 (kills W14): the Block-var contributor anchor keeps its exact
    // authored escaped spelling; the top-level var reference's semantic name
    // is the decoded "a".
    let (_, script) = recognized_variable(r"{ var \u{61}; } var x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("escaped Block-var LHS must match by decoded semantic name");
    assert_eq!(contributors.len(), 1);
    assert_eq!(contributors[0].fragment(), r"\u{61}");
    assert_ne!(contributors[0].fragment(), "a");
    assert_eq!(range(contributors[0]), (6, 12));
}

#[test]
fn block_var_contributor_domain_has_no_unicode_normalization() {
    // F10 (kills W15): a composed Block-var contributor matches only a
    // composed reference of the identical code point sequence; a
    // code-point-distinct decomposed reference of the same visual name
    // matches nothing.
    let (_, script) = recognized_variable("{ var \u{e9}; } var x=\u{e9}; var y=e\u{301};");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);

    let contributors = relations[0]
        .correspondence()
        .var_contributors()
        .expect("composed reference must match composed Block-var contributor");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (6, 8));
    assert_eq!(contributors[0].fragment(), "\u{e9}");

    assert_eq!(relations[1].semantic_name().chars().count(), 2);
    assert!(
        relations[1]
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn lexical_precedence_outranks_widened_block_var_contributors() {
    // F11A (kills W16): a current-region Block lexical binding wins even
    // though a same-named `var` contributor exists elsewhere in the Script.
    let (_, script) = recognized_variable("var a; { let a; let y=a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("current-region Block lexical binding must win over widened var domain");
    assert_eq!(range(binding), (13, 14));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert!(relation.correspondence().var_contributors().is_none());

    // The existing Block-origin top-level lexical fallback (F11B) is
    // unchanged by the widened var domain: it is already covered by
    // `block_region_falls_back_to_var_but_current_or_top_lexical_binding_wins`
    // above (`"let a=1; { let x=a; } var b;"` resolves to the top-level
    // lexical `a`, never a var contributor).
}

#[test]
fn widened_block_var_domain_never_fabricates_a_false_positive_no_contributor_match() {
    // F12: an unrelated reference name absent from both lexical bindings and
    // every selected var contributor (top-level and Block) must still reach
    // `NoSelectedSameSourceContributor`. `var z;` is a distinctly-named
    // top-level var so the source both reaches `RecognizedVariableStatementSlice`
    // and adds an unrelated contributor the widened domain must not match.
    let (_, script) = recognized_variable("var z; { var a; } let y=q;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "q");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn static_rejection_still_gates_correspondence_when_a_block_var_contributor_is_present() {
    // SR1-equivalent (kills W17): the whole source is statically rejected
    // (Script duplicate top-level lexical `a`) before any correspondence
    // relation for `var x=a;` could be committed, even though a same-named
    // Block var contributor is also authored in the source.
    let (_, script) = recognized_variable("let a=1; let a=2; { var a; } var x=a;");
    assert!(matches!(
        evaluate_selected_variable_statement_static_semantics(&script),
        SelectedVariableStatementStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::DuplicateLexicalName { .. }
        )
    ));
}

// --- One-level Block accepted-witness entrypoint (Issue #705/#706) --------
//
// These regressions directly exercise
// `analyze_selected_one_level_block_name_correspondence` against
// representative cases drawn from the candidate-independent #706 oracle
// (`qualification_validation_tests::selected_one_level_block_name_correspondence_lifecycle_frontier`,
// left unmodified). The `SelectedOneLevelBlockStaticSemanticsAccepted`
// witness has no top-level `VariableStatement` item; every reference below
// is a top-level or Block-scoped `LexicalDeclaration`.

#[test]
fn one_level_block_same_block_var_contributor_is_seen_by_block_lexical_reference() {
    // P1 / oracle F1 (kills W1, W3, W4): a same-Block Block-var contributor
    // is visible to an existing Block lexical reference under the
    // `OneLevelBlock` accepted witness, without requiring any top-level
    // `VariableStatement` to exist.
    let (_, script) = recognized_one_level_block("{ var a; let x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("same-Block var contributor must be visible to the Block lexical reference");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (6, 7));
}

#[test]
fn one_level_block_zero_reference_source_is_a_completed_empty_correspondence() {
    // P2 / oracle F2 (kills W13): an accepted `OneLevelBlock` witness with no
    // reference inputs still produces `Complete([])`, never absence.
    let (_, script) = recognized_one_level_block("{ var a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    assert!(analysis.relations().is_empty());
}

#[test]
fn one_level_block_unrelated_undeclared_reference_is_no_contributor() {
    // P3 / oracle F3: an unrelated, undeclared reference inside the Block is
    // `NoSelectedSameSourceContributor`, not a static failure.
    let (_, script) = recognized_one_level_block("{ let x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn one_level_block_var_contributor_is_visible_to_top_level_lexical_reference() {
    // P4 / oracle F4 (kills W4, W5): a Block-var contributor is seen by an
    // existing top-level lexical reference, and the whole source still
    // reaches the `OneLevelBlock` witness because no top-level
    // `VariableStatement` is present.
    let (_, script) = recognized_one_level_block("{ var a; }\nlet x=a;");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("Block var contributor must be visible to a top-level lexical reference");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (6, 7));
}

#[test]
fn one_level_block_current_region_lexical_precedence_outranks_sibling_block_var_contributor() {
    // P5 / oracle F8 (kills W8, W10): a same-name selected var contributor
    // genuinely exists in a sibling Block; current-region (Block) lexical
    // precedence must still win before the all-selected-var contributor
    // stage.
    let (_, script) = recognized_one_level_block("{ var a; } { let a=1; let x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("current-region Block lexical binding must win over a sibling var contributor");
    assert_eq!(range(binding), (17, 18));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert!(relation.correspondence().var_contributors().is_none());
}

#[test]
fn one_level_block_top_level_lexical_fallback_is_reachable_from_block_reference() {
    // P6 / oracle F7 (kills W9, W11): the existing Block-origin top-level
    // lexical fallback remains reachable from the `OneLevelBlock` witness; no
    // var contributor may override it.
    let (_, script) = recognized_one_level_block("let a=1;\n{ let x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("top-level lexical fallback must remain reachable");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn one_level_block_repeated_var_contributors_preserve_multiplicity_and_order() {
    // P7 / oracle F9 (kills W5): two declarators of one Block `var` statement
    // remain two distinct authored occurrences, in exact authored order,
    // never deduplicated.
    let (_, script) = recognized_one_level_block("{ var a,a; let x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("repeated Block var contributors");
    assert_eq!(contributors.len(), 2);
    assert_eq!(range(contributors[0]), (6, 7));
    assert_eq!(range(contributors[1]), (8, 9));
}

#[test]
fn one_level_block_escaped_var_lhs_uses_semantic_equality_with_exact_authored_anchor() {
    // P8 / oracle F10 (kills W6): the Block-var contributor anchor keeps its
    // exact authored escaped spelling; the reference's semantic name is the
    // decoded "a".
    let (_, script) = recognized_one_level_block(r"{ var \u{61}; let x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("escaped Block-var LHS must match by decoded semantic name");
    assert_eq!(contributors.len(), 1);
    assert_eq!(contributors[0].fragment(), r"\u{61}");
    assert_ne!(contributors[0].fragment(), "a");
    assert_eq!(range(contributors[0]), (6, 12));
}

#[test]
fn one_level_block_var_contributor_domain_has_no_unicode_normalization() {
    // P9 / oracle F11 (kills W7): a composed Block-var contributor matches
    // only a composed reference; a code-point-distinct decomposed reference
    // of the same visual name matches nothing.
    let (_, script) = recognized_one_level_block("{ var \u{e9}; let x=\u{e9}; let y=e\u{301}; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);

    let contributors = relations[0]
        .correspondence()
        .var_contributors()
        .expect("composed reference must match composed Block-var contributor");
    assert_eq!(contributors.len(), 1);
    assert_eq!(contributors[0].fragment(), "\u{e9}");

    assert_eq!(relations[1].semantic_name().chars().count(), 2);
    assert!(
        relations[1]
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn one_level_block_decimal_initialized_var_contributes_via_lhs_only() {
    // P10 / oracle F12 (kills W10-decimal): a decimal-initialized Block var
    // contributes through its LHS `BindingIdentifier`; the decimal RHS is
    // never contributor or reference evidence.
    let (_, script) = recognized_one_level_block("{ var a=1; let x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("decimal-initialized Block var contributes via LHS");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (6, 7));
    assert_ne!(range(contributors[0]), (8, 9));
}

#[test]
fn one_level_block_q_prefix_correspondence_projection_matches_variable_statement_witness() {
    // P11 (kills W12): comparing a representative `OneLevelBlock` fixture
    // against the same shape prefixed with an unrelated top-level `var q;`
    // (which moves recognition to the distinct `VariableStatement` witness)
    // must agree on the correspondence projection for the unrelated queried
    // name `a`, after accounting for the exact prefix byte-length shift.
    // This proves the two accepted-witness entrypoints share one semantic
    // theorem without collapsing witness identity.
    const PREFIX_LEN: usize = "var q;\n".len();

    let (_, one_level_block_script) = recognized_one_level_block("{ var a; let x=a; }");
    let one_level_block_analysis = accepted_one_level_block_analysis(&one_level_block_script);
    let one_level_block_relation = one_relation(&one_level_block_analysis);

    let (_, variable_statement_script) = recognized_variable("var q;\n{ var a; let x=a; }");
    let variable_statement_analysis = accepted_analysis(&variable_statement_script);
    let variable_statement_relation = one_relation(&variable_statement_analysis);

    assert_eq!(
        one_level_block_relation.semantic_name(),
        variable_statement_relation.semantic_name()
    );
    assert!(matches!(
        one_level_block_relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert!(matches!(
        variable_statement_relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));

    let one_level_block_contributors = one_level_block_relation
        .correspondence()
        .var_contributors()
        .expect("OneLevelBlock witness must produce a var contributor relation");
    let variable_statement_contributors = variable_statement_relation
        .correspondence()
        .var_contributors()
        .expect("VariableStatement witness must produce a var contributor relation");

    assert_eq!(
        one_level_block_contributors.len(),
        variable_statement_contributors.len()
    );
    for (base, prefixed) in one_level_block_contributors
        .iter()
        .zip(variable_statement_contributors.iter())
    {
        assert_eq!(base.fragment(), prefixed.fragment());
        assert_eq!(base.range().start() + PREFIX_LEN, prefixed.range().start());
        assert_eq!(base.range().end() + PREFIX_LEN, prefixed.range().end());
    }

    // The accepted-witness identity is explicitly NOT invariant across the
    // prefix: the unrelated `var q;` moves recognition from
    // `RecognizedOneLevelBlockSlice` to `RecognizedVariableStatementSlice`.
    // Only the correspondence projection for the unrelated queried name `a`
    // is invariant.
    assert!(matches!(
        recognize_selected_lexical_slice(&source("{ var a; let x=a; }")),
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(_)
    ));
    assert!(matches!(
        recognize_selected_lexical_slice(&source("var q;\n{ var a; let x=a; }")),
        SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(_)
    ));
}
