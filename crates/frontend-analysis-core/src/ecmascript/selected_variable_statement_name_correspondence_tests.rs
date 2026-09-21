use crate::{SourceAnchor, SourceId, SourceText};

use super::selected_lexical_slice::{
    SelectedBlockReferenceUseEnabledScript, SelectedIdentifierReferenceExpressionStatementScript,
    SelectedLexicalSliceOutcome, SelectedOneLevelBlockScript, SelectedVariableStatementScript,
    recognize_selected_lexical_slice,
};
use super::selected_static_semantics::{
    SelectedBlockReferenceUseEnabledStaticSemanticsOutcome,
    SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome,
    SelectedOneLevelBlockStaticSemanticsOutcome, SelectedStaticSemanticsRejection,
    SelectedVariableStatementStaticSemanticsOutcome,
    evaluate_selected_block_reference_use_enabled_static_semantics,
    evaluate_selected_identifier_reference_expression_statement_static_semantics,
    evaluate_selected_one_level_block_static_semantics,
    evaluate_selected_variable_statement_static_semantics,
};
use super::selected_variable_statement_name_correspondence::{
    SelectedBlockUseSiteNameCorrespondenceAnalysis, SelectedBlockUseSiteNameCorrespondenceOutcome,
    SelectedTopLevelIdentifierReferenceUseSiteNameCorrespondenceAnalysis,
    SelectedTopLevelIdentifierReferenceUseSiteNameCorrespondenceOutcome,
    SelectedVariableStatementNameCorrespondenceAnalysis,
    SelectedVariableStatementNameCorrespondenceOutcome,
    SelectedVariableStatementNameCorrespondenceRegion,
    analyze_selected_block_reference_use_enabled_name_correspondence,
    analyze_selected_block_reference_use_enabled_top_level_identifier_reference_use_site_name_correspondence,
    analyze_selected_block_use_site_name_correspondence,
    analyze_selected_one_level_block_name_correspondence,
    analyze_selected_reference_use_enabled_name_correspondence,
    analyze_selected_top_level_identifier_reference_use_site_name_correspondence,
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
        "var a=null.foo; let x=a;",
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
fn leading_plus_minus_direct_identifier_reference_unary_expression_composes_unchanged_with_top_level_var_correspondence()
 {
    let (_, script) = recognized_variable("let a; var x=-a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(range(relation.containing_binding()), (11, 12));
    assert_eq!(range(relation.reference()), (14, 15));
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("existing top-level lexical target meaning");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));

    let (_, script) = recognized_variable("var a; var x=+a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(range(relation.containing_binding()), (11, 12));
    assert_eq!(range(relation.reference()), (14, 15));
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("existing same-source var contributor meaning");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));

    let (_, script) = recognized_variable("var x=-z;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "z");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_variable("var x=-a,y=+b;");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(range(relations[0].containing_binding()), (4, 5));
    assert_eq!(range(relations[0].reference()), (7, 8));
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(range(relations[1].containing_binding()), (9, 10));
    assert_eq!(range(relations[1].reference()), (12, 13));
    assert_eq!(relations[1].semantic_name(), "b");
}

#[test]
fn leading_plus_minus_identifier_reference_unary_expression_composes_unchanged_with_escaped_operand_and_top_level_var_correspondence()
 {
    let (_, script) = recognized_variable(r"let a; var x=-\u{61};");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(range(relation.containing_binding()), (11, 12));
    assert_eq!(range(relation.reference()), (14, 20));
    assert_eq!(relation.semantic_name(), "a");
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("existing top-level lexical target meaning");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));

    let (_, script) = recognized_variable(r"var a; var x=+\u{61};");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(range(relation.containing_binding()), (11, 12));
    assert_eq!(range(relation.reference()), (14, 20));
    assert_eq!(relation.semantic_name(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("existing same-source var contributor meaning");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));

    let (_, script) = recognized_variable(r"var x=-\u{7A};");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "z");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_variable(r"var x=-\u{61},y=+\u{62};");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(range(relations[0].containing_binding()), (4, 5));
    assert_eq!(range(relations[0].reference()), (7, 13));
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(range(relations[1].containing_binding()), (14, 15));
    assert_eq!(range(relations[1].reference()), (17, 23));
    assert_eq!(relations[1].semantic_name(), "b");
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

// --- One-level Block accepted-witness production regressions -------------
//
// These regressions directly exercise
// `analyze_selected_one_level_block_name_correspondence` against
// representative cases drawn from the candidate-independent #705/#706 oracle
// (`qualification_validation_tests::selected_one_level_block_name_correspondence_lifecycle_frontier`,
// left unmodified). The `SelectedOneLevelBlockStaticSemanticsAccepted`
// witness has no top-level `VariableStatement` item; every reference below
// is a top-level or Block-scoped `LexicalDeclaration`.

#[test]
fn one_level_block_same_block_var_contributor_is_seen_by_block_lexical_reference() {
    // P1 / oracle F1 (kills W1, W2, W3): a same-Block Block-var contributor
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
    // P5 / oracle F8 (kills W10): a same-name selected var contributor
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
    // P6 / oracle F7 (kills W11): the existing Block-origin top-level
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
    // P7 / oracle F9 (kills W6): two declarators of one Block `var` statement
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
    // P8 / oracle F10 (kills W7): the Block-var contributor anchor keeps its
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
    // P9 / oracle F11 (kills W8): a composed Block-var contributor matches
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
    // P10 / oracle F12 (kills W9): a decimal-initialized Block var
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
    // P11 / oracle F1/F1Q (kills W14): comparing a representative
    // `OneLevelBlock` fixture against the same shape prefixed with an unrelated
    // top-level `var q;` (which moves recognition to the distinct
    // `VariableStatement` witness) must agree on the correspondence projection
    // for the unrelated queried name `a`, after accounting for the exact prefix
    // byte-length shift. This proves the two accepted-witness entrypoints share
    // one semantic theorem without collapsing witness identity.
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

// --- Issue #710: one-level Block `var` widened to a direct-authored, ------
// escape-free `IdentifierReference` initializer, with a source-name
// correspondence relation owned by the exact containing Block-var
// declarator.
//
// These focused production tests seal the candidate against the accepted
// #688/#710 theorem with independently authored expected values rather than
// values derived from production output. They reuse this module's existing
// helpers and every existing correspondence meaning; no fourth
// correspondence meaning or parallel type hierarchy is introduced.

#[test]
fn one_level_block_var_direct_identifier_reference_relation_retains_exact_binding_reference_and_region()
 {
    // V1: direct RHS selected, exact containing binding = x, exact
    // reference = a, semantic name = "a", current region = Block, exactly
    // one correspondence relation.
    let (_, script) = recognized_one_level_block("{ var x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(range(relation.containing_binding()), (6, 7));
    assert_eq!(range(relation.reference()), (8, 9));
    assert_eq!(relation.semantic_name(), "a");
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
}

#[test]
fn one_level_block_var_and_lexical_relations_follow_mixed_authored_item_order() {
    // V3 (load-bearing): once lexical declarations and Block `var`
    // declarators can both own RHS `IdentifierReference` queries, relation
    // emission must follow exact authored Block-item order, never a
    // grouping by declaration kind. Kills both "lexical relations first,
    // then var relations" and "var relations first, then lexical relations"
    // wrong models.
    let (_, script) = recognized_one_level_block("{ let p=q; var x=a; let r=s; var y=b; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 4);

    let semantic_names: Vec<_> = relations.iter().map(|r| r.semantic_name()).collect();
    assert_eq!(semantic_names, ["q", "a", "s", "b"]);

    let containing_bindings: Vec<_> = relations
        .iter()
        .map(|r| range(r.containing_binding()))
        .collect();
    assert_eq!(containing_bindings, [(6, 7), (15, 16), (24, 25), (33, 34)]);

    let reference_ranges: Vec<_> = relations.iter().map(|r| range(r.reference())).collect();
    assert_eq!(reference_ranges, [(8, 9), (17, 18), (26, 27), (35, 36)]);
}

#[test]
fn leading_plus_minus_direct_identifier_reference_unary_expression_composes_unchanged_with_block_var_correspondence()
 {
    // Current-Block lexical target.
    let (_, script) = recognized_one_level_block("{ let a; var x=-a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("current-Block lexical target meaning");
    assert_eq!(range(binding), (6, 7));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));

    // Top-level lexical fallback.
    let (_, script) = recognized_one_level_block("let a; { var x=+a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("top-level lexical fallback meaning");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));

    // Existing same-source var contributor meaning.
    let (_, script) = recognized_variable("var a; { var x=-a; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("existing same-source var contributor meaning");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));

    // No contributor.
    let (_, script) = recognized_one_level_block("{ var x=-z; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "z");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    // Mixed authored Block relation order (unary-wrapped var RHS composed
    // with lexical declarations): relation emission follows exact authored
    // Block-item order, never a grouping by declaration kind.
    let (_, script) = recognized_one_level_block("{ let p=q; var x=-a; let r=s; var y=+b; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 4);
    let semantic_names: Vec<_> = relations.iter().map(|r| r.semantic_name()).collect();
    assert_eq!(semantic_names, ["q", "a", "s", "b"]);
    let containing_bindings: Vec<_> = relations
        .iter()
        .map(|r| range(r.containing_binding()))
        .collect();
    assert_eq!(containing_bindings, [(6, 7), (15, 16), (25, 26), (34, 35)]);
    let reference_ranges: Vec<_> = relations.iter().map(|r| range(r.reference())).collect();
    assert_eq!(reference_ranges, [(8, 9), (18, 19), (27, 28), (37, 38)]);
}

#[test]
fn leading_plus_minus_identifier_reference_unary_expression_composes_unchanged_with_escaped_operand_and_block_var_correspondence()
 {
    // Current-Block lexical target.
    let (_, script) = recognized_one_level_block(r"{ let a; var x=-\u{61}; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("current-Block lexical target meaning");
    assert_eq!(range(binding), (6, 7));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));

    // Top-level lexical fallback.
    let (_, script) = recognized_one_level_block(r"let a; { var x=+\u{61}; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("top-level lexical fallback meaning");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));

    // Existing same-source var contributor meaning.
    let (_, script) = recognized_variable(r"var a; { var x=-\u{61}; }");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("existing same-source var contributor meaning");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));

    // No contributor.
    let (_, script) = recognized_one_level_block(r"{ var x=-\u{7A}; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "z");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    // Mixed authored Block relation order (escaped unary-wrapped var RHS
    // composed with lexical declarations): relation emission follows exact
    // authored Block-item order, never a grouping by declaration kind.
    let (_, script) =
        recognized_one_level_block(r"{ let p=q; var x=-\u{61}; let r=s; var y=+\u{62}; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 4);
    let semantic_names: Vec<_> = relations.iter().map(|r| r.semantic_name()).collect();
    assert_eq!(semantic_names, ["q", "a", "s", "b"]);
    let containing_bindings: Vec<_> = relations
        .iter()
        .map(|r| range(r.containing_binding()))
        .collect();
    assert_eq!(containing_bindings, [(6, 7), (15, 16), (30, 31), (39, 40)]);
    let reference_ranges: Vec<_> = relations.iter().map(|r| range(r.reference())).collect();
    assert_eq!(reference_ranges, [(8, 9), (18, 24), (32, 33), (42, 48)]);
}

#[test]
fn one_level_block_var_direct_identifier_reference_reuses_existing_correspondence_precedence() {
    // V6: a Block-var RHS relation reuses this module's single existing
    // correspondence semantic owner and precedence rather than a new
    // meaning. Current-Block lexical precedence still wins first...
    let (_, script) = recognized_one_level_block("{ let a; var x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("current-Block lexical binding must win over the widened var domain");
    assert_eq!(range(binding), (6, 7));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert!(relation.correspondence().var_contributors().is_none());

    // ...and, absent a current-Block or top-level lexical binding, the
    // existing all-selected-var contributor domain (#701/#703) still
    // applies, even when the sole contributor is the Block-var statement's
    // own sibling declarator.
    let (_, script) = recognized_one_level_block("{ var a; var x=a; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("existing all-selected-var contributor domain must still apply");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (6, 7));
}

#[test]
fn one_level_block_var_direct_identifier_reference_q_prefix_correspondence_projection_matches_variable_statement_witness()
 {
    // W9 (kills a witness-route asymmetry model): the same Block-var direct
    // RHS relation must be produced by both accepted-witness routes.
    // Comparing a representative `OneLevelBlock` fixture against the same
    // shape prefixed with an unrelated top-level `var q;` (which moves
    // recognition to the distinct `VariableStatement` witness) must agree on
    // the correspondence projection after accounting for the exact prefix
    // byte-length shift.
    const PREFIX_LEN: usize = "var q;\n".len();

    let (_, one_level_block_script) = recognized_one_level_block("{ var x=a; }");
    let one_level_block_analysis = accepted_one_level_block_analysis(&one_level_block_script);
    let one_level_block_relation = one_relation(&one_level_block_analysis);

    let (_, variable_statement_script) = recognized_variable("var q;\n{ var x=a; }");
    let variable_statement_analysis = accepted_analysis(&variable_statement_script);
    let variable_statement_relation = one_relation(&variable_statement_analysis);

    assert_eq!(one_level_block_relation.semantic_name(), "a");
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

    let base_binding = range(one_level_block_relation.containing_binding());
    let prefixed_binding = range(variable_statement_relation.containing_binding());
    assert_eq!(base_binding.0 + PREFIX_LEN, prefixed_binding.0);
    assert_eq!(base_binding.1 + PREFIX_LEN, prefixed_binding.1);

    let base_reference = range(one_level_block_relation.reference());
    let prefixed_reference = range(variable_statement_relation.reference());
    assert_eq!(base_reference.0 + PREFIX_LEN, prefixed_reference.0);
    assert_eq!(base_reference.1 + PREFIX_LEN, prefixed_reference.1);

    // The accepted-witness identity is explicitly NOT invariant across the
    // prefix: the unrelated `var q;` moves recognition from
    // `RecognizedOneLevelBlockSlice` to `RecognizedVariableStatementSlice`.
    assert!(matches!(
        recognize_selected_lexical_slice(&source("{ var x=a; }")),
        SelectedLexicalSliceOutcome::RecognizedOneLevelBlockSlice(_)
    ));
    assert!(matches!(
        recognize_selected_lexical_slice(&source("var q;\n{ var x=a; }")),
        SelectedLexicalSliceOutcome::RecognizedVariableStatementSlice(_)
    ));
}

// --- Issue #713: one-level Block `var` widened to a selected escaped ------
// non-ReservedWord `IdentifierReference` initializer, with a source-name
// correspondence relation reusing the exact same #710 relation owner and
// every existing correspondence meaning/precedence.
//
// These focused production tests seal the candidate against the accepted
// #712/#713 theorem with independently authored expected values rather
// than values derived from production output. No fourth correspondence
// meaning or parallel type hierarchy is introduced.

#[test]
fn one_level_block_var_escaped_identifier_reference_relation_retains_exact_binding_reference_and_region()
 {
    // Escaped counterpart of the #710 direct-RHS identity seal: exact
    // containing binding = x, exact reference = \u0061, decoded semantic
    // name = "a", current region = Block, exactly one correspondence
    // relation.
    let (_, script) = recognized_one_level_block(r"{ var x=\u0061; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(range(relation.containing_binding()), (6, 7));
    assert_eq!(range(relation.reference()), (8, 14));
    assert_eq!(relation.semantic_name(), "a");
    assert!(matches!(
        relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
}

#[test]
fn one_level_block_var_escaped_identifier_reference_reuses_existing_correspondence_precedence() {
    // W9 (kills a widened-domain-bypasses-lexical-precedence model): an
    // escaped Block-var RHS relation reuses this module's single existing
    // correspondence semantic owner and precedence rather than a new
    // meaning. Current-Block lexical precedence still wins first...
    let (_, script) = recognized_one_level_block(r"{ let a; var x=\u0061; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("current-Block lexical binding must win over the widened var domain");
    assert_eq!(range(binding), (6, 7));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert!(relation.correspondence().var_contributors().is_none());

    // ...and, absent a current-Block or top-level lexical binding, the
    // existing all-selected-var contributor domain (#701/#703) still
    // applies to an escaped RHS exactly as it does for a direct one.
    let (_, script) = recognized_one_level_block(r"{ var a; var x=\u0061; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("existing all-selected-var contributor domain must still apply");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (6, 7));
}

#[test]
fn one_level_block_var_escaped_identifier_reference_reaches_top_level_lexical_fallback() {
    // Precedence step 2 (kills a "widened escaped RHS domain skips the
    // top-level lexical fallback and falls straight through to the var
    // contributor domain, or to NoSelectedSameSourceContributor" model): an
    // escaped Block-var RHS with no current-Block lexical binding of the
    // same decoded name must still reach the existing Block-origin
    // top-level lexical fallback, exactly as the direct RHS case does
    // (`one_level_block_top_level_lexical_fallback_is_reachable_from_block_reference`
    // above), before any var-contributor or no-contributor branch is
    // considered.
    let (_, script) = recognized_one_level_block("let a;\n{ var x=\\u0061; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");

    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("top-level lexical fallback must remain reachable for an escaped RHS");
    assert_eq!(range(binding), (4, 5));
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    assert!(relation.correspondence().var_contributors().is_none());
}

#[test]
fn one_level_block_var_and_lexical_relations_follow_mixed_authored_item_order_with_escaped_rhs() {
    // V3/W12 (load-bearing): an escaped Block-var RHS of variable authored
    // byte length must not disturb exact authored `block.items()` relation
    // order. Kills both "lexical relations first, then var relations" and
    // "var relations first, then lexical relations" wrong models even when
    // one var declarator's authored RHS is longer than its decoded name.
    let (_, script) = recognized_one_level_block(r"{ let p=q; var x=\u0061; let r=s; var y=b; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 4);

    let semantic_names: Vec<_> = relations.iter().map(|r| r.semantic_name()).collect();
    assert_eq!(semantic_names, ["q", "a", "s", "b"]);

    let containing_bindings: Vec<_> = relations
        .iter()
        .map(|r| range(r.containing_binding()))
        .collect();
    assert_eq!(containing_bindings, [(6, 7), (15, 16), (29, 30), (38, 39)]);

    let reference_ranges: Vec<_> = relations.iter().map(|r| range(r.reference())).collect();
    assert_eq!(reference_ranges, [(8, 9), (17, 23), (31, 32), (40, 41)]);
}

#[test]
fn one_level_block_var_escaped_identifier_reference_q_prefix_correspondence_projection_matches_variable_statement_witness()
 {
    // W14 (kills a witness-route asymmetry model): the same Block-var
    // escaped RHS relation must be produced by both accepted-witness
    // routes, exactly as already proven for the direct RHS case.
    const PREFIX_LEN: usize = "var q;\n".len();

    let (_, one_level_block_script) = recognized_one_level_block(r"{ var x=\u0061; }");
    let one_level_block_analysis = accepted_one_level_block_analysis(&one_level_block_script);
    let one_level_block_relation = one_relation(&one_level_block_analysis);

    let (_, variable_statement_script) = recognized_variable("var q;\n{ var x=\\u0061; }");
    let variable_statement_analysis = accepted_analysis(&variable_statement_script);
    let variable_statement_relation = one_relation(&variable_statement_analysis);

    assert_eq!(one_level_block_relation.semantic_name(), "a");
    assert_eq!(
        one_level_block_relation.semantic_name(),
        variable_statement_relation.semantic_name()
    );

    let base_binding = range(one_level_block_relation.containing_binding());
    let prefixed_binding = range(variable_statement_relation.containing_binding());
    assert_eq!(base_binding.0 + PREFIX_LEN, prefixed_binding.0);
    assert_eq!(base_binding.1 + PREFIX_LEN, prefixed_binding.1);

    let base_reference = range(one_level_block_relation.reference());
    let prefixed_reference = range(variable_statement_relation.reference());
    assert_eq!(base_reference.0 + PREFIX_LEN, prefixed_reference.0);
    assert_eq!(base_reference.1 + PREFIX_LEN, prefixed_reference.1);
}

#[test]
fn one_level_block_var_escaped_identifier_reference_composition_matches_differently_spelled_contributor_without_unicode_normalization()
 {
    // W10/W8: a Block-var escaped RHS matches a same-source contributor by
    // decoded semantic name even when the contributor's authored spelling
    // differs (a direct composed "\u{e9}" contributor matched by an escaped
    // "\u00E9" RHS reference), and Unicode normalization never creates a
    // false match: a decoded decomposed sequence ("e" + combining acute)
    // remains code-point-distinct from the composed contributor and finds
    // no same-source contributor.
    let (_, script) =
        recognized_variable("var \u{e9}; { var x=\\u00E9; } { var y=\\u0065\\u0301; }");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);

    assert_eq!(range(relations[0].containing_binding()), (14, 15));
    assert_eq!(range(relations[0].reference()), (16, 22));
    assert_eq!(relations[0].semantic_name(), "\u{e9}");
    let contributors = relations[0]
        .correspondence()
        .var_contributors()
        .expect("escaped composed RHS must match the differently-spelled direct contributor");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 6));
    assert_eq!(contributors[0].fragment(), "\u{e9}");

    assert_eq!(range(relations[1].containing_binding()), (32, 33));
    assert_eq!(range(relations[1].reference()), (34, 46));
    assert_eq!(relations[1].semantic_name().chars().count(), 2);
    assert_ne!(relations[1].semantic_name(), "\u{e9}");
    assert!(
        relations[1]
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn block_var_close_brace_asi_terminator_provenance_does_not_change_correspondence_meaning() {
    // Issue #717: for otherwise-identical selected Block-var source, an
    // authored semicolon and bounded close-brace ASI must produce the exact
    // same correspondence relation; only statement terminator provenance
    // differs, and that is terminator-blind to correspondence.
    let (_, semicolon_script) = recognized_one_level_block("let a; { var x=a; }");
    let semicolon_analysis = accepted_one_level_block_analysis(&semicolon_script);
    let semicolon_relation = one_relation(&semicolon_analysis);

    let (_, close_brace_script) = recognized_one_level_block("let a; { var x=a }");
    let close_brace_analysis = accepted_one_level_block_analysis(&close_brace_script);
    let close_brace_relation = one_relation(&close_brace_analysis);

    assert!(matches!(
        semicolon_relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert!(matches!(
        close_brace_relation.current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));

    assert_eq!(
        range(semicolon_relation.containing_binding()),
        range(close_brace_relation.containing_binding())
    );
    assert_eq!(
        range(semicolon_relation.reference()),
        range(close_brace_relation.reference())
    );
    assert_eq!(
        semicolon_relation.semantic_name(),
        close_brace_relation.semantic_name()
    );

    // The top-level lexical "a" outranks the Block var contributor "x=a";
    // both terminator forms must resolve to the exact same
    // `VisibleSelectedLexicalBinding` fact, never `SameSourceSelectedVarNameContributors`.
    let (semicolon_binding, semicolon_region) = semicolon_relation
        .correspondence()
        .selected_lexical_binding()
        .expect("authored-semicolon top-level lexical fallback");
    let (close_brace_binding, close_brace_region) = close_brace_relation
        .correspondence()
        .selected_lexical_binding()
        .expect("close-brace-ASI top-level lexical fallback");
    assert_eq!(range(semicolon_binding), range(close_brace_binding));
    assert!(matches!(
        semicolon_region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    assert!(matches!(
        close_brace_region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn boolean_backed_var_declarator_contributes_no_correspondence_relation_while_siblings_are_preserved()
 {
    // Issue #719: a direct BooleanLiteral initializer retains no
    // `identifier_reference_initializer` fact, so it must never fabricate a
    // correspondence relation. A sibling IdentifierReference-backed
    // declarator in the same list must keep its existing relation meaning.
    let (_, script) = recognized_variable("var a=true,b=x;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_one_level_block("{ var a=false,b=x; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn null_backed_var_declarator_contributes_no_correspondence_relation_while_siblings_are_preserved()
{
    // Issue #721: a direct NullLiteral initializer retains no
    // `identifier_reference_initializer` fact, so it must never fabricate a
    // correspondence relation. A sibling IdentifierReference-backed
    // declarator in the same list must keep its existing relation meaning.
    let (_, script) = recognized_variable("var a=null,b=x;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_one_level_block("{ var a=null,b=x; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn this_backed_var_declarator_contributes_no_correspondence_relation_while_siblings_are_preserved()
{
    // Issue #723: a direct `PrimaryExpression : this` initializer retains no
    // `identifier_reference_initializer` fact, so it must never fabricate a
    // correspondence relation. A sibling IdentifierReference-backed
    // declarator in the same list must keep its existing relation meaning.
    let (_, script) = recognized_variable("var a=this,b=x;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_one_level_block("{ var a=this,b=x; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn this_only_var_declarator_list_produces_zero_correspondence_relations() {
    // Issue #723: a direct `PrimaryExpression : this`-only declarator list
    // contains no IdentifierReference contributor at all, so the
    // correspondence analysis must yield zero relations, not merely a
    // `NoSelectedSameSourceContributor` relation.
    for text in ["var a=this;", "var a=this,b=this;"] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }

    for text in ["{ var a=this; }", "{ var a=this,b=this; }"] {
        let (_, script) = recognized_one_level_block(text);
        let analysis = accepted_one_level_block_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }
}

#[test]
fn string_literal_backed_var_declarator_contributes_no_correspondence_relation_while_siblings_are_preserved()
 {
    // Issue #725: a direct, escape-free `StringLiteral` initializer retains
    // no `identifier_reference_initializer` fact, so it must never fabricate
    // a correspondence relation. A sibling IdentifierReference-backed
    // declarator in the same list must keep its existing relation meaning.
    let (_, script) = recognized_variable("var a=\"abc\",b=x;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_one_level_block("{ var a='abc',b=x; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn string_literal_only_var_declarator_list_produces_zero_correspondence_relations() {
    // Issue #725: a direct, escape-free `StringLiteral`-only declarator list
    // contains no IdentifierReference contributor at all, so the
    // correspondence analysis must yield zero relations, not merely a
    // `NoSelectedSameSourceContributor` relation.
    for text in ["var a=\"abc\";", "var a=\"abc\",b='def';"] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }

    for text in ["{ var a=\"abc\"; }", "{ var a=\"abc\",b='def'; }"] {
        let (_, script) = recognized_one_level_block(text);
        let analysis = accepted_one_level_block_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }
}

#[test]
fn null_only_var_declarator_list_produces_zero_correspondence_relations() {
    // Issue #721: a direct NullLiteral-only declarator list contains no
    // IdentifierReference contributor at all, so the correspondence
    // analysis must yield zero relations, not merely a
    // `NoSelectedSameSourceContributor` relation.
    for text in ["var a=null;", "var a=null,b=null;"] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }

    for text in ["{ var a=null; }", "{ var a=null,b=null; }"] {
        let (_, script) = recognized_one_level_block(text);
        let analysis = accepted_one_level_block_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }
}

#[test]
fn plain_fractional_decimal_literal_backed_var_declarator_contributes_no_correspondence_relation_while_siblings_are_preserved()
 {
    // Issue #732: a direct-authored plain fractional `DecimalLiteral`
    // initializer retains no `identifier_reference_initializer` fact, so it
    // must never fabricate a correspondence relation. A sibling
    // IdentifierReference-backed declarator in the same list must keep its
    // existing relation meaning.
    let (_, script) = recognized_variable("var a=1.0,b=x;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_one_level_block("{ var a=.5,b=x; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn plain_fractional_decimal_literal_only_var_declarator_list_produces_zero_correspondence_relations()
 {
    // Issue #732: a direct-authored plain fractional `DecimalLiteral`-only
    // declarator list contains no IdentifierReference contributor at all, so
    // the correspondence analysis must yield zero relations, not merely a
    // `NoSelectedSameSourceContributor` relation.
    for text in ["var a=1.0;", "var a=1.0,b=.5;"] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }

    for text in ["{ var a=1.0; }", "{ var a=1.0,b=.5; }"] {
        let (_, script) = recognized_one_level_block(text);
        let analysis = accepted_one_level_block_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }
}

#[test]
fn plain_fractional_decimal_literal_backed_var_lhs_still_contributes_as_existing_same_source_contributor()
 {
    // Issue #732 same-source-contributor invariant: the RHS being a
    // fractional `DecimalLiteral` must not suppress or duplicate the
    // declarator's own LHS contribution when that same name is later
    // referenced (`var a=1.0; let x=a;`).
    let (_, script) = recognized_variable("var a=1.0; let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    assert_eq!(relation.reference().fragment(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("top-level var contributor relation");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));
}

#[test]
fn plain_exponent_decimal_literal_backed_var_declarator_contributes_no_correspondence_relation_while_siblings_are_preserved()
 {
    // Issue #740: a direct-authored, separator-free exponent
    // `DecimalLiteral` initializer retains no `identifier_reference_initializer`
    // fact, so it must never fabricate a correspondence relation. A sibling
    // IdentifierReference-backed declarator in the same list must keep its
    // existing relation meaning.
    let (_, script) = recognized_variable("var a=1e2,b=x;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_one_level_block("{ var a=.5e2,b=x; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn plain_exponent_decimal_literal_only_var_declarator_list_produces_zero_correspondence_relations()
{
    // Issue #740: a direct-authored, separator-free exponent
    // `DecimalLiteral`-only declarator list contains no IdentifierReference
    // contributor at all, so the correspondence analysis must yield zero
    // relations, not merely a `NoSelectedSameSourceContributor` relation.
    for text in ["var a=1e2;", "var a=1e2,b=.5e2;"] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }

    for text in ["{ var a=1e2; }", "{ var a=1e2,b=.5e2; }"] {
        let (_, script) = recognized_one_level_block(text);
        let analysis = accepted_one_level_block_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }
}

#[test]
fn plain_exponent_decimal_literal_backed_var_lhs_still_contributes_as_existing_same_source_contributor()
 {
    // Issue #740 same-source-contributor invariant: the RHS being an
    // exponent `DecimalLiteral` must not suppress or duplicate the
    // declarator's own LHS contribution when that same name is later
    // referenced (`var a=1e2; let x=a;`).
    let (_, script) = recognized_variable("var a=1e2; let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    assert_eq!(relation.reference().fragment(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("top-level var contributor relation");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));
}

#[test]
fn leading_plus_minus_decimal_unary_expression_backed_var_declarator_contributes_no_correspondence_relation_while_siblings_are_preserved()
 {
    // Issue #744: a direct-authored leading `+`/`-` decimal
    // `UnaryExpression` initializer retains no `identifier_reference_initializer`
    // fact, so it must never fabricate a correspondence relation. A sibling
    // IdentifierReference-backed declarator in the same list must keep its
    // existing relation meaning.
    let (_, script) = recognized_variable("var a=-1,b=x;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );

    let (_, script) = recognized_one_level_block("{ var a=-1e-2,b=x; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "x");
    assert_eq!(relation.reference().fragment(), "x");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn leading_plus_minus_decimal_unary_expression_only_var_declarator_list_produces_zero_correspondence_relations()
 {
    // Issue #744: a direct-authored leading `+`/`-` decimal
    // `UnaryExpression`-only declarator list contains no IdentifierReference
    // contributor at all, so the correspondence analysis must yield zero
    // relations, not merely a `NoSelectedSameSourceContributor` relation.
    for text in ["var a=-1;", "var a=-1,b=+1e2;"] {
        let (_, script) = recognized_variable(text);
        let analysis = accepted_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }

    for text in ["{ var a=-1; }", "{ var a=-1,b=+1e2; }"] {
        let (_, script) = recognized_one_level_block(text);
        let analysis = accepted_one_level_block_analysis(&script);
        assert!(analysis.relations().is_empty(), "{text}");
    }
}

#[test]
fn leading_plus_minus_decimal_unary_expression_backed_var_lhs_still_contributes_as_existing_same_source_contributor()
 {
    // Issue #744 same-source-contributor invariant: the RHS being a leading
    // `+`/`-` decimal `UnaryExpression` must not suppress or duplicate the
    // declarator's own LHS contribution when that same name is later
    // referenced (`var a=-1; let x=a;`).
    let (_, script) = recognized_variable("var a=-1; let x=a;");
    let analysis = accepted_analysis(&script);
    let relation = one_relation(&analysis);
    assert_eq!(relation.semantic_name(), "a");
    assert_eq!(relation.reference().fragment(), "a");
    let contributors = relation
        .correspondence()
        .var_contributors()
        .expect("top-level var contributor relation");
    assert_eq!(contributors.len(), 1);
    assert_eq!(range(contributors[0]), (4, 5));
}

// Issue #754: the two-`IdentifierReference` additive initializer widens
// this consumer's input to 0..2 facts per binding/declarator, consumed in
// exact authored reference order without reordering or deduplication.
// `#752`/PR #753 independently proves the underlying bounded theorem; these
// tests seal only that this consumer composes it correctly across
// declarator lists, `var` placement, and Block `var` placement.

#[test]
fn two_identifier_reference_additive_initializer_preserves_1_to_n_declarator_ordering_for_var() {
    // Issue #754 1..N declarator composition: `x=a+b, y=c, z=d+e` composes
    // as `x.first, x.second, y.one, z.first, z.second`, never flattened out
    // of declarator order.
    let (_, script) =
        recognized_variable("let a; let b; let c; let d; let e; var x=a+b, y=c, z=d+e;");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(
        relations
            .iter()
            .map(|relation| relation.semantic_name())
            .collect::<Vec<_>>(),
        ["a", "b", "c", "d", "e"]
    );
    for relation in relations {
        let (_, region) = relation
            .correspondence()
            .selected_lexical_binding()
            .unwrap_or_else(|| {
                panic!(
                    "expected top-level lexical target for {}",
                    relation.semantic_name()
                )
            });
        assert!(matches!(
            region,
            SelectedVariableStatementNameCorrespondenceRegion::TopLevel
        ));
    }
}

#[test]
fn two_identifier_reference_additive_initializer_does_not_deduplicate_equal_semantic_names_for_var()
{
    let (_, script) = recognized_variable("let a; var x = a + a;");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "a");
    assert_ne!(
        range(relations[0].reference()),
        range(relations[1].reference())
    );
}

#[test]
fn two_identifier_reference_additive_initializer_composes_for_lexical_declaration_in_a_var_enabled_script()
 {
    let (_, script) = recognized_variable("let a; let b; const x = a + b; var unused;");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(
        relations
            .iter()
            .map(|relation| relation.semantic_name())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
}

#[test]
fn two_identifier_reference_additive_initializer_composes_for_one_level_block_var() {
    let (_, script) = recognized_one_level_block("let a; let b; { var x = a + b; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "b");
    assert!(matches!(
        relations[0].current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert!(matches!(
        relations[1].current_region(),
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    let (_, first_target_region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("first fact must resolve to the top-level lexical target a");
    assert!(matches!(
        first_target_region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

/// Issue #779: the right-unary exactly-two `IdentifierReference` additive
/// initializer reaches this consumer through the unchanged `Two { first,
/// second }` carrier. Both inner facts resolve independently under the
/// existing correspondence precedence, for top-level `var` and Block `var`
/// alike; no correspondence meaning or precedence rule changes.
#[test]
fn right_unary_additive_initializer_resolves_both_inner_facts_independently() {
    let (_, script) = recognized_variable("let a; var b; var x=a+-b;");
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "b");
    // The right unary sign is excluded from the second reference anchor:
    // `a` at 20, `+` at 21, `-` at 22, `b` at 23.
    assert_eq!(range(relations[0].reference()), (20, 21));
    assert_eq!(range(relations[1].reference()), (23, 24));

    // Block `var`, with each inner fact resolving to a different region
    // under the existing precedence.
    let (_, script) = recognized_one_level_block("let b; { let a; var x=a-+b; }");
    let analysis = accepted_one_level_block_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "b");
    let (_, first_target_region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("first fact must resolve to the Block-local lexical target a");
    assert!(matches!(
        first_target_region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    let (_, second_target_region) = relations[1]
        .correspondence()
        .selected_lexical_binding()
        .expect("second fact must resolve to the top-level lexical target b");
    assert!(matches!(
        second_target_region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn two_identifier_reference_additive_initializer_direct_escaped_combination_for_var() {
    // Fixture text is built with `concat!` over an individually escaped
    // fragment (matching PR #753's own technique) so the literal
    // backslash-u-hex bytes survive intact.
    let escaped_operand = concat!("\\", "u0061");
    let text = format!("let a; var x = {escaped_operand} + b;");
    let (_, script) = recognized_variable(&text);
    let analysis = accepted_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].reference().fragment(), escaped_operand);
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].reference().fragment(), "b");
    assert_eq!(relations[1].semantic_name(), "b");
    assert!(
        relations[0]
            .correspondence()
            .selected_lexical_binding()
            .is_some(),
        "escaped first operand must still resolve to the top-level lexical target a"
    );
    assert!(
        relations[1]
            .correspondence()
            .is_no_selected_same_source_contributor(),
        "second operand b has no same-source contributor"
    );
}

// --- Issue #758: top-level free-standing `IdentifierReference`
// `ExpressionStatement` use-site correspondence. ---

fn recognized_reference_use(
    text: &str,
) -> (
    SourceText,
    SelectedIdentifierReferenceExpressionStatementScript,
) {
    let source = source(text);
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedIdentifierReferenceExpressionStatementSlice(
            script,
        ) => script,
        other => panic!("expected reference-use-enabled recognition for {text:?}, got {other:?}"),
    };
    (source, script)
}

fn accepted_reference_use_initializer_analysis<'script>(
    script: &'script SelectedIdentifierReferenceExpressionStatementScript,
) -> SelectedVariableStatementNameCorrespondenceAnalysis<'script> {
    let accepted =
        match evaluate_selected_identifier_reference_expression_statement_static_semantics(script) {
            SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Accepted(
                accepted,
            ) => accepted,
            other => {
                panic!("expected reference-use-enabled selected static acceptance, got {other:?}")
            }
        };

    match analyze_selected_reference_use_enabled_name_correspondence(&accepted) {
        SelectedVariableStatementNameCorrespondenceOutcome::Complete(analysis) => analysis,
        other => panic!("expected complete initializer correspondence, got {other:?}"),
    }
}

fn accepted_use_site_analysis<'script>(
    script: &'script SelectedIdentifierReferenceExpressionStatementScript,
) -> SelectedTopLevelIdentifierReferenceUseSiteNameCorrespondenceAnalysis<'script> {
    let accepted =
        match evaluate_selected_identifier_reference_expression_statement_static_semantics(script) {
            SelectedIdentifierReferenceExpressionStatementStaticSemanticsOutcome::Accepted(
                accepted,
            ) => accepted,
            other => {
                panic!("expected reference-use-enabled selected static acceptance, got {other:?}")
            }
        };

    match analyze_selected_top_level_identifier_reference_use_site_name_correspondence(&accepted) {
        SelectedTopLevelIdentifierReferenceUseSiteNameCorrespondenceOutcome::Complete(analysis) => {
            analysis
        }
        other => panic!("expected complete use-site correspondence, got {other:?}"),
    }
}

#[test]
fn use_site_no_selected_same_source_contributor_for_a_lone_reference() {
    let (_, script) = recognized_reference_use("a;");
    let analysis = accepted_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one use-site relation");
    };
    assert_eq!(relation.semantic_name(), "a");
    assert_eq!(relation.reference().fragment(), "a");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn use_site_backward_lexical_correspondence() {
    let (_, script) = recognized_reference_use("let a;\na;");
    let analysis = accepted_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one use-site relation");
    };
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("must resolve to the top-level lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn use_site_forward_lexical_correspondence() {
    // Issue #758 section 31: the free-standing use-site may correspond to a
    // selected same-source top-level lexical declaration authored later.
    // This is same-source static provenance, not TDZ/execution-order
    // resolution.
    let (_, script) = recognized_reference_use("a;\nlet a;");
    let analysis = accepted_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one use-site relation");
    };
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("must resolve to the later-authored top-level lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn use_site_var_contributor_semantics_preserve_authored_order_and_no_deduplication() {
    let (_, script) = recognized_reference_use("var a;\na;");
    let analysis = accepted_use_site_analysis(&script);
    let contributors = analysis.relations()[0]
        .correspondence()
        .var_contributors()
        .expect("one var contributor");
    assert_eq!(contributors.len(), 1);

    let (_, script) = recognized_reference_use("var a;\nvar a;\na;");
    let analysis = accepted_use_site_analysis(&script);
    let contributors = analysis.relations()[0]
        .correspondence()
        .var_contributors()
        .expect("two var contributors");
    assert_eq!(contributors.len(), 2);
    assert!(contributors[0].range().start() < contributors[1].range().start());

    let (_, script) = recognized_reference_use("{ var a; }\na;");
    let analysis = accepted_use_site_analysis(&script);
    let contributors = analysis.relations()[0]
        .correspondence()
        .var_contributors()
        .expect("Block var contributor composes into Script same-source provenance");
    assert_eq!(contributors.len(), 1);
}

#[test]
fn use_site_top_level_block_composition_reuses_block_lexical_visibility() {
    let (_, script) = recognized_reference_use("{ let a; }\na;");
    let analysis = accepted_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one use-site relation");
    };
    // A Block-local lexical binding never enters the top-level lexical
    // domain (issue section 29): the free-standing top-level `a;` must not
    // resolve to it.
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn use_site_duplicate_occurrences_are_preserved_not_deduplicated() {
    let (_, script) = recognized_reference_use("let a;\na;\na;");
    let analysis = accepted_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "a");
    assert!(relations[0].reference().range().start() < relations[1].reference().range().start());
    for relation in relations {
        assert!(
            relation
                .correspondence()
                .selected_lexical_binding()
                .is_some()
        );
    }
}

#[test]
fn use_site_authored_occurrence_order_is_preserved_not_target_declaration_order() {
    let (_, script) = recognized_reference_use("let b;\nlet a;\na;\nb;");
    let analysis = accepted_use_site_analysis(&script);
    let names: Vec<&str> = analysis
        .relations()
        .iter()
        .map(|relation| relation.semantic_name())
        .collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn initializer_and_use_site_relation_surfaces_remain_separate() {
    let (_, script) = recognized_reference_use("let a;\nlet x = a;\na;");

    let initializer_analysis = accepted_reference_use_initializer_analysis(&script);
    let initializer_relations = initializer_analysis.relations();
    assert_eq!(initializer_relations.len(), 1);
    assert_eq!(initializer_relations[0].semantic_name(), "a");
    assert_eq!(
        initializer_relations[0].containing_binding().fragment(),
        "x"
    );

    let use_site_analysis = accepted_use_site_analysis(&script);
    let use_site_relations = use_site_analysis.relations();
    assert_eq!(use_site_relations.len(), 1);
    assert_eq!(use_site_relations[0].semantic_name(), "a");

    // Distinct authored occurrences: the initializer RHS `a` and the
    // free-standing `a;` never collapse into one relation stream.
    assert_ne!(
        (
            initializer_relations[0].reference().range().start(),
            initializer_relations[0].reference().range().end(),
        ),
        (
            use_site_relations[0].reference().range().start(),
            use_site_relations[0].reference().range().end(),
        ),
    );
}

// --- Issue #762: Block-contained free-standing `IdentifierReference`
// `ExpressionStatement` use-site correspondence (matching the accepted
// #760/#761 Oracle). ---

fn recognized_block_reference_use(
    text: &str,
) -> (SourceText, SelectedBlockReferenceUseEnabledScript) {
    let source = source(text);
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(script) => script,
        other => {
            panic!("expected Block-reference-use-enabled recognition for {text:?}, got {other:?}")
        }
    };
    (source, script)
}

fn accepted_block_reference_use_initializer_analysis<'script>(
    script: &'script SelectedBlockReferenceUseEnabledScript,
) -> SelectedVariableStatementNameCorrespondenceAnalysis<'script> {
    let accepted = match evaluate_selected_block_reference_use_enabled_static_semantics(script) {
        SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Accepted(accepted) => accepted,
        other => {
            panic!("expected Block-reference-use-enabled selected static acceptance, got {other:?}")
        }
    };

    match analyze_selected_block_reference_use_enabled_name_correspondence(&accepted) {
        SelectedVariableStatementNameCorrespondenceOutcome::Complete(analysis) => analysis,
        other => panic!("expected complete initializer correspondence, got {other:?}"),
    }
}

fn accepted_block_reference_use_top_level_use_site_analysis<'script>(
    script: &'script SelectedBlockReferenceUseEnabledScript,
) -> SelectedTopLevelIdentifierReferenceUseSiteNameCorrespondenceAnalysis<'script> {
    let accepted = match evaluate_selected_block_reference_use_enabled_static_semantics(script) {
        SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Accepted(accepted) => accepted,
        other => {
            panic!("expected Block-reference-use-enabled selected static acceptance, got {other:?}")
        }
    };

    match analyze_selected_block_reference_use_enabled_top_level_identifier_reference_use_site_name_correspondence(&accepted) {
        SelectedTopLevelIdentifierReferenceUseSiteNameCorrespondenceOutcome::Complete(analysis) => analysis,
        other => panic!("expected complete TopLevel use-site correspondence, got {other:?}"),
    }
}

fn accepted_block_use_site_analysis<'script>(
    script: &'script SelectedBlockReferenceUseEnabledScript,
) -> SelectedBlockUseSiteNameCorrespondenceAnalysis<'script> {
    let accepted = match evaluate_selected_block_reference_use_enabled_static_semantics(script) {
        SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Accepted(accepted) => accepted,
        other => {
            panic!("expected Block-reference-use-enabled selected static acceptance, got {other:?}")
        }
    };

    match analyze_selected_block_use_site_name_correspondence(&accepted) {
        SelectedBlockUseSiteNameCorrespondenceOutcome::Complete(analysis) => analysis,
        other => panic!("expected complete Block use-site correspondence, got {other:?}"),
    }
}

#[test]
fn block_use_site_no_selected_same_source_contributor_for_a_lone_reference() {
    let (_, script) = recognized_block_reference_use("{ a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one Block use-site relation");
    };
    assert_eq!(relation.semantic_name(), "a");
    assert_eq!(relation.reference().fragment(), "a");
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn block_use_site_current_block_lexical_precedence() {
    let (_, script) = recognized_block_reference_use("{ let a; a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one Block use-site relation");
    };
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("must resolve to the same-Block lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
}

#[test]
fn block_use_site_top_level_lexical_fallback() {
    let (_, script) = recognized_block_reference_use("let a;\n{ a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one Block use-site relation");
    };
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("must fall back to the TopLevel lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn block_use_site_same_block_lexical_shadows_top_level_lexical() {
    let (_, script) = recognized_block_reference_use("let a;\n{ let a; a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one Block use-site relation");
    };
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("must resolve to the same-Block lexical binding a, not the TopLevel one");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
}

#[test]
fn block_use_site_forward_same_block_and_top_level_lexical_correspondence() {
    // Same-source forward correspondence, not TDZ/execution-order
    // resolution (issue section "New Block-only use-site relation").
    let (_, script) = recognized_block_reference_use("{ a; let a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one Block use-site relation");
    };
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("must resolve to the later-authored same-Block lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));

    let (_, script) = recognized_block_reference_use("{ a; }\nlet a;");
    let analysis = accepted_block_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one Block use-site relation");
    };
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("must resolve to the later-authored TopLevel lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn block_use_site_sibling_block_lexical_never_leaks() {
    let (_, script) = recognized_block_reference_use("{ let a; }\n{ a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one Block use-site relation");
    };
    assert!(
        relation
            .correspondence()
            .is_no_selected_same_source_contributor(),
        "a sibling Block's own lexical binding must never leak into this Block's use-site"
    );
}

#[test]
fn block_use_site_var_contributor_eligibility_top_level_same_block_and_sibling_block() {
    // Top-level var contributor.
    let (_, script) = recognized_block_reference_use("var a;\n{ a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let contributors = analysis.relations()[0]
        .correspondence()
        .var_contributors()
        .expect("top-level var contributor");
    assert_eq!(contributors.len(), 1);

    // Same-Block var contributor.
    let (_, script) = recognized_block_reference_use("{ var a; a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let contributors = analysis.relations()[0]
        .correspondence()
        .var_contributors()
        .expect("same-Block var contributor");
    assert_eq!(contributors.len(), 1);

    // Sibling-Block var contributor, eligible under Script-wide provenance.
    let (_, script) = recognized_block_reference_use("{ var a; }\n{ a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let contributors = analysis.relations()[0]
        .correspondence()
        .var_contributors()
        .expect("sibling-Block var contributor remains Script-wide eligible");
    assert_eq!(contributors.len(), 1);
}

#[test]
fn block_use_site_lexical_precedence_over_same_name_var_contributor() {
    let (_, script) = recognized_block_reference_use("{ var a; let x; }\n{ let a; a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let [relation] = analysis.relations() else {
        panic!("expected exactly one Block use-site relation");
    };
    let (binding, region) = relation
        .correspondence()
        .selected_lexical_binding()
        .expect("current-Block lexical must outrank a Script-wide var contributor");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
}

#[test]
fn block_use_site_var_contributors_preserve_duplicates_and_authored_order() {
    let (_, script) = recognized_block_reference_use("var a;\nvar a;\n{ a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let contributors = analysis.relations()[0]
        .correspondence()
        .var_contributors()
        .expect("two var contributors");
    assert_eq!(contributors.len(), 2);
    assert!(contributors[0].range().start() < contributors[1].range().start());
}

#[test]
fn block_use_site_duplicate_occurrences_preserved_one_for_one() {
    let (_, script) = recognized_block_reference_use("{ a; a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "a");
    assert!(relations[0].reference().range().start() < relations[1].reference().range().start());
    // Both must resolve to the same containing Block anchor.
    assert_eq!(
        range(relations[0].containing_block()),
        range(relations[1].containing_block())
    );
}

#[test]
fn block_use_site_distinct_blocks_retain_distinct_ownership() {
    let (_, script) = recognized_block_reference_use("{ a; }\n{ a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_ne!(
        range(relations[0].containing_block()),
        range(relations[1].containing_block()),
        "distinct Blocks must retain distinct containing-Block ownership"
    );
}

#[test]
fn block_use_site_authored_occurrence_order_is_preserved_across_blocks() {
    let (_, script) = recognized_block_reference_use("{ b; }\n{ a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let names: Vec<&str> = analysis
        .relations()
        .iter()
        .map(|relation| relation.semantic_name())
        .collect();
    assert_eq!(names, ["b", "a"]);
}

#[test]
fn block_use_site_known_static_rejection_suppresses_relation_construction() {
    let source = source("let a;\n{ var a; a; }");
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(script) => script,
        other => panic!("expected Block-reference-use-enabled recognition, got {other:?}"),
    };
    assert!(matches!(
        evaluate_selected_block_reference_use_enabled_static_semantics(&script),
        SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. }
        )
    ));
    // No relation surface is reachable without an accepted witness: the
    // three correspondence entrypoints all require
    // `SelectedBlockReferenceUseEnabledStaticSemanticsAccepted`, which
    // static rejection never produces.
}

#[test]
fn three_correspondence_surfaces_remain_separate_for_block_reference_use_enabled_script() {
    let (_, script) = recognized_block_reference_use("let a;\nlet x = a;\na;\n{ a; }");

    let initializer_analysis = accepted_block_reference_use_initializer_analysis(&script);
    let initializer_relations = initializer_analysis.relations();
    assert_eq!(initializer_relations.len(), 1);
    assert_eq!(initializer_relations[0].semantic_name(), "a");
    assert_eq!(
        initializer_relations[0].containing_binding().fragment(),
        "x"
    );

    let top_level_use_site_analysis =
        accepted_block_reference_use_top_level_use_site_analysis(&script);
    let top_level_use_site_relations = top_level_use_site_analysis.relations();
    assert_eq!(top_level_use_site_relations.len(), 1);
    assert_eq!(top_level_use_site_relations[0].semantic_name(), "a");

    let block_use_site_analysis = accepted_block_use_site_analysis(&script);
    let block_use_site_relations = block_use_site_analysis.relations();
    assert_eq!(block_use_site_relations.len(), 1);
    assert_eq!(block_use_site_relations[0].semantic_name(), "a");

    // All three authored `a` occurrences (initializer RHS, TopLevel
    // use-site, Block use-site) remain distinct authored ranges across the
    // three separate result surfaces -- never one combined relation stream.
    let all_ranges = [
        range(initializer_relations[0].reference()),
        range(top_level_use_site_relations[0].reference()),
        range(block_use_site_relations[0].reference()),
    ];
    assert_ne!(all_ranges[0], all_ranges[1]);
    assert_ne!(all_ranges[0], all_ranges[2]);
    assert_ne!(all_ranges[1], all_ranges[2]);
}

#[test]
fn block_use_site_escaped_direct_and_escaped_provenance_are_exact() {
    for (text, expected_fragment, expected_name) in [
        ("{ a; }", "a", "a"),
        (r"{ \u0061; }", r"\u0061", "a"),
        (r"{ f\u006Fo; }", r"f\u006Fo", "foo"),
    ] {
        let (_, script) = recognized_block_reference_use(text);
        let analysis = accepted_block_use_site_analysis(&script);
        let [relation] = analysis.relations() else {
            panic!("expected exactly one Block use-site relation for {text}");
        };
        assert_eq!(relation.reference().fragment(), expected_fragment, "{text}");
        assert_eq!(relation.semantic_name(), expected_name, "{text}");
    }
}

// --- Issue #766: exactly-two IdentifierReference additive free-standing
// `ExpressionStatement` use-site correspondence. Each retained fact of a
// `SelectedFreeStandingIdentifierReferenceUseSite::Two` occurrence is an
// independent correspondence query, emitting one relation per fact in exact
// authored item order, then exact authored operand order within each item
// (per #688 comment 5739718987). ---

#[test]
fn two_operand_use_site_per_operand_correspondence_top_level() {
    // `let a; var b; a+b;`: `a` resolves to the lexical binding, `b` to the
    // Script-wide var contributor -- two independent per-operand queries,
    // never one shared result for the whole additive use-site.
    let (_, script) = recognized_reference_use("let a;\nvar b;\na+b;");
    let analysis = accepted_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    let (binding, region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("left operand must resolve to the lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    assert_eq!(relations[1].semantic_name(), "b");
    let contributors = relations[1]
        .correspondence()
        .var_contributors()
        .expect("right operand must resolve to the Script-wide var contributor b");
    assert_eq!(contributors.len(), 1);
}

#[test]
fn two_operand_use_site_per_operand_correspondence_block() {
    // `let b; { let a; a+b; }`: `a` resolves to the current-Block lexical
    // binding, `b` falls back to the TopLevel lexical binding.
    let (_, script) = recognized_block_reference_use("let b;\n{ let a;\na+b; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    let (binding, region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("left operand must resolve to the current-Block lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert_eq!(relations[1].semantic_name(), "b");
    let (binding, region) = relations[1]
        .correspondence()
        .selected_lexical_binding()
        .expect("right operand must fall back to the TopLevel lexical binding b");
    assert_eq!(binding.fragment(), "b");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn two_operand_use_site_forward_correspondence_top_level() {
    // Same-source forward correspondence, not TDZ/execution-order
    // resolution.
    let (_, script) = recognized_reference_use("a+b;\nlet a;\nlet b;");
    let analysis = accepted_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    for (relation, expected_name) in relations.iter().zip(["a", "b"]) {
        let (binding, region) = relation
            .correspondence()
            .selected_lexical_binding()
            .unwrap_or_else(|| {
                panic!("must resolve to the later-authored binding {expected_name}")
            });
        assert_eq!(binding.fragment(), expected_name);
        assert!(matches!(
            region,
            SelectedVariableStatementNameCorrespondenceRegion::TopLevel
        ));
    }
}

#[test]
fn two_operand_use_site_forward_correspondence_block() {
    let (_, script) = recognized_block_reference_use("let b;\n{ a+b;\nlet a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);

    assert_eq!(relations[0].semantic_name(), "a");
    let (binding, region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("must resolve to the later-authored same-Block lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));

    assert_eq!(relations[1].semantic_name(), "b");
    let (binding, region) = relations[1]
        .correspondence()
        .selected_lexical_binding()
        .expect("must resolve to the TopLevel lexical binding b");
    assert_eq!(binding.fragment(), "b");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn two_operand_use_site_duplicate_operands_top_level() {
    let (_, script) = recognized_reference_use("a+a;");
    let analysis = accepted_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(
        relations.len(),
        2,
        "duplicate operands must not deduplicate"
    );
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "a");
    assert!(relations[0].reference().range().start() < relations[1].reference().range().start());
}

#[test]
fn two_operand_use_site_duplicate_operands_block() {
    let (_, script) = recognized_block_reference_use("{ a+a; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(
        relations.len(),
        2,
        "duplicate operands must not deduplicate"
    );
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "a");
    assert!(relations[0].reference().range().start() < relations[1].reference().range().start());
    assert_eq!(
        range(relations[0].containing_block()),
        range(relations[1].containing_block())
    );
}

#[test]
fn two_operand_use_site_multi_item_ordering_top_level() {
    let (_, script) = recognized_reference_use("a+b;\nc-d;");
    let analysis = accepted_use_site_analysis(&script);
    let names: Vec<&str> = analysis
        .relations()
        .iter()
        .map(|relation| relation.semantic_name())
        .collect();
    assert_eq!(names, ["a", "b", "c", "d"]);
}

#[test]
fn two_operand_use_site_multi_item_ordering_block() {
    let (_, script) = recognized_block_reference_use("{\n    a+b;\n    c-d;\n}");
    let analysis = accepted_block_use_site_analysis(&script);
    let names: Vec<&str> = analysis
        .relations()
        .iter()
        .map(|relation| relation.semantic_name())
        .collect();
    assert_eq!(names, ["a", "b", "c", "d"]);
}

#[test]
fn two_operand_use_site_cross_surface_separation() {
    let (_, script) = recognized_block_reference_use("a+b;\n{ c-d; }\ne+f;");

    let top_level_analysis = accepted_block_reference_use_top_level_use_site_analysis(&script);
    let top_level_names: Vec<&str> = top_level_analysis
        .relations()
        .iter()
        .map(|relation| relation.semantic_name())
        .collect();
    assert_eq!(top_level_names, ["a", "b", "e", "f"]);

    let block_analysis = accepted_block_use_site_analysis(&script);
    let block_names: Vec<&str> = block_analysis
        .relations()
        .iter()
        .map(|relation| relation.semantic_name())
        .collect();
    assert_eq!(block_names, ["c", "d"]);
}

#[test]
fn two_operand_use_site_initializer_and_free_standing_surfaces_remain_separate() {
    let (_, script) = recognized_reference_use("let x=a+b;\nc+d;");

    let initializer_analysis = accepted_reference_use_initializer_analysis(&script);
    let initializer_names: Vec<&str> = initializer_analysis
        .relations()
        .iter()
        .map(|relation| relation.semantic_name())
        .collect();
    assert_eq!(initializer_names, ["a", "b"]);
    for relation in initializer_analysis.relations() {
        assert_eq!(relation.containing_binding().fragment(), "x");
    }

    let use_site_analysis = accepted_use_site_analysis(&script);
    let use_site_names: Vec<&str> = use_site_analysis
        .relations()
        .iter()
        .map(|relation| relation.semantic_name())
        .collect();
    assert_eq!(use_site_names, ["c", "d"]);
}

#[test]
fn two_operand_use_site_known_static_rejection_suppresses_relation_construction() {
    let source = source("let a;\n{ var a; b+c; }");
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(script) => script,
        other => panic!("expected Block-reference-use-enabled recognition, got {other:?}"),
    };
    assert!(matches!(
        evaluate_selected_block_reference_use_enabled_static_semantics(&script),
        SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. }
        )
    ));
    // No relation surface is reachable without an accepted witness: the two
    // operands of the locally valid `b+c;` use-site never publish a
    // relation once the existing lexical/var static rejection wins.
}

// --- Issue #768: correspondence remains terminator-blind for the new
// automatic (`AutomaticAtEof` / `AutomaticBeforeBlockClose`) free-standing
// use-site termination provenance -- relation meaning depends only on the
// existing `One`/`Two` body, never on how the use-site was terminated. ---

#[test]
fn top_level_use_site_correspondence_is_terminator_blind_for_one_body() {
    let (_, authored_script) = recognized_reference_use("let a;\na;");
    let (_, automatic_script) = recognized_reference_use("let a;\na");

    let authored_analysis = accepted_use_site_analysis(&authored_script);
    let automatic_analysis = accepted_use_site_analysis(&automatic_script);
    let authored_relation = &authored_analysis.relations()[0];
    let automatic_relation = &automatic_analysis.relations()[0];

    for relation in [authored_relation, automatic_relation] {
        let (binding, region) = relation
            .correspondence()
            .selected_lexical_binding()
            .expect("must resolve to the top-level lexical binding a");
        assert_eq!(binding.fragment(), "a");
        assert!(matches!(
            region,
            SelectedVariableStatementNameCorrespondenceRegion::TopLevel
        ));
    }
}

#[test]
fn top_level_use_site_correspondence_is_terminator_blind_for_two_body() {
    let (_, authored_script) = recognized_reference_use("let a;\nvar b;\na+b;");
    let (_, automatic_script) = recognized_reference_use("let a;\nvar b;\na+b");

    let authored_relations = accepted_use_site_analysis(&authored_script);
    let automatic_relations = accepted_use_site_analysis(&automatic_script);

    for relations in [
        authored_relations.relations(),
        automatic_relations.relations(),
    ] {
        let [a, b] = relations else {
            panic!("expected exactly two use-site relations");
        };
        let (binding, region) = a
            .correspondence()
            .selected_lexical_binding()
            .expect("a must resolve to the visible top-level lexical binding");
        assert_eq!(binding.fragment(), "a");
        assert!(matches!(
            region,
            SelectedVariableStatementNameCorrespondenceRegion::TopLevel
        ));
        assert_eq!(
            b.correspondence()
                .var_contributors()
                .expect("b must resolve to the same-source var contributors")
                .len(),
            1
        );
    }
}

#[test]
fn block_use_site_correspondence_is_terminator_blind() {
    let (_, authored_script) = recognized_block_reference_use("let b;\n{ let a;\na+b; }");
    let (_, automatic_script) = recognized_block_reference_use("let b;\n{ let a;\na+b }");

    let authored_relations = accepted_block_use_site_analysis(&authored_script);
    let automatic_relations = accepted_block_use_site_analysis(&automatic_script);

    for relations in [
        authored_relations.relations(),
        automatic_relations.relations(),
    ] {
        let [a, b] = relations else {
            panic!("expected exactly two Block use-site relations");
        };
        let (a_binding, a_region) = a
            .correspondence()
            .selected_lexical_binding()
            .expect("a must resolve to the current-Block lexical binding");
        assert_eq!(a_binding.fragment(), "a");
        assert!(matches!(
            a_region,
            SelectedVariableStatementNameCorrespondenceRegion::Block(_)
        ));

        let (b_binding, b_region) = b
            .correspondence()
            .selected_lexical_binding()
            .expect("b must resolve to the TopLevel lexical binding");
        assert_eq!(b_binding.fragment(), "b");
        assert!(matches!(
            b_region,
            SelectedVariableStatementNameCorrespondenceRegion::TopLevel
        ));
    }
}

#[test]
fn block_use_site_known_static_rejection_suppresses_relation_construction_for_automatic_termination()
 {
    // The same static-rejection gate applies whether the locally valid
    // `b+c` use-site is authored-semicolon-terminated or
    // before-close-automatic-terminated.
    let source = source("let a;\n{ var a; b+c }");
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(script) => script,
        other => panic!("expected Block-reference-use-enabled recognition, got {other:?}"),
    };
    assert!(matches!(
        evaluate_selected_block_reference_use_enabled_static_semantics(&script),
        SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. }
        )
    ));
}

// --- Issue #773: left-unary exactly-two `IdentifierReference` additive
// free-standing use-site correspondence. Section 13 of the frozen theorem
// requires zero correspondence semantic change: the two retained facts of a
// `+a+b;`-shaped `Two` occurrence are queried exactly as the existing plain
// `a+b;` `Two` occurrence's facts already are, per operand, in exact
// authored order -- the leading unary sign is construction syntax only and
// never reaches this correspondence layer. ---

#[test]
fn left_unary_use_site_per_operand_correspondence_top_level() {
    // `let a; var b; +a+b;`: identical correspondence meaning to the
    // existing `let a; var b; a+b;` per-operand theorem -- only the first
    // operand's own leading `+`/`-` construction syntax differs.
    let (_, script) = recognized_reference_use("let a;\nvar b;\n+a+b;");
    let analysis = accepted_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    let (binding, region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("left operand must resolve to the lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
    assert_eq!(relations[1].semantic_name(), "b");
    let contributors = relations[1]
        .correspondence()
        .var_contributors()
        .expect("right operand must resolve to the Script-wide var contributor b");
    assert_eq!(contributors.len(), 1);
}

#[test]
fn left_unary_use_site_per_operand_correspondence_block() {
    // `let b; { let a; -a-b; }`: current-Block lexical precedence for `a`,
    // TopLevel lexical fallback for `b`, exactly as the existing plain
    // two-operand theorem.
    let (_, script) = recognized_block_reference_use("let b;\n{ let a;\n-a-b; }");
    let analysis = accepted_block_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    let (binding, region) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("left operand must resolve to the current-Block lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::Block(_)
    ));
    assert_eq!(relations[1].semantic_name(), "b");
    let (binding, region) = relations[1]
        .correspondence()
        .selected_lexical_binding()
        .expect("right operand must fall back to the TopLevel lexical binding b");
    assert_eq!(binding.fragment(), "b");
    assert!(matches!(
        region,
        SelectedVariableStatementNameCorrespondenceRegion::TopLevel
    ));
}

#[test]
fn left_unary_use_site_no_selected_same_source_contributor_for_unmatched_right_operand() {
    // `let a; +a+z;`: `a` resolves the visible lexical binding (through the
    // unary-wrapped left operand); `z` has no same-source contributor at
    // all, exactly as the existing plain-additive theorem.
    let (_, script) = recognized_reference_use("let a;\n+a+z;");
    let analysis = accepted_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name(), "a");
    let (binding, _) = relations[0]
        .correspondence()
        .selected_lexical_binding()
        .expect("left operand must resolve to the visible lexical binding a");
    assert_eq!(binding.fragment(), "a");
    assert_eq!(relations[1].semantic_name(), "z");
    assert!(
        relations[1]
            .correspondence()
            .is_no_selected_same_source_contributor()
    );
}

#[test]
fn left_unary_use_site_duplicate_operands_are_preserved_distinctly() {
    let (_, script) = recognized_reference_use("+a+a;");
    let analysis = accepted_use_site_analysis(&script);
    let relations = analysis.relations();
    assert_eq!(
        relations.len(),
        2,
        "duplicate operands must not deduplicate"
    );
    assert_eq!(relations[0].semantic_name(), "a");
    assert_eq!(relations[1].semantic_name(), "a");
    assert!(relations[0].reference().range().start() < relations[1].reference().range().start());
}

#[test]
fn left_unary_use_site_correspondence_is_terminator_blind() {
    let (_, authored_script) = recognized_reference_use("let a;\nvar b;\n+a+b;");
    let (_, automatic_script) = recognized_reference_use("let a;\nvar b;\n+a+b");

    let authored_relations = accepted_use_site_analysis(&authored_script);
    let automatic_relations = accepted_use_site_analysis(&automatic_script);

    for relations in [
        authored_relations.relations(),
        automatic_relations.relations(),
    ] {
        let [a, b] = relations else {
            panic!("expected exactly two use-site relations");
        };
        let (binding, region) = a
            .correspondence()
            .selected_lexical_binding()
            .expect("a must resolve to the visible top-level lexical binding");
        assert_eq!(binding.fragment(), "a");
        assert!(matches!(
            region,
            SelectedVariableStatementNameCorrespondenceRegion::TopLevel
        ));
        assert_eq!(
            b.correspondence()
                .var_contributors()
                .expect("b must resolve to the same-source var contributors")
                .len(),
            1
        );
    }
}

#[test]
fn left_unary_use_site_known_static_rejection_suppresses_relation_construction() {
    let source = source("let a;\n{ var a; -b-c; }");
    let script = match recognize_selected_lexical_slice(&source) {
        SelectedLexicalSliceOutcome::RecognizedBlockReferenceUseEnabledSlice(script) => script,
        other => panic!("expected Block-reference-use-enabled recognition, got {other:?}"),
    };
    assert!(matches!(
        evaluate_selected_block_reference_use_enabled_static_semantics(&script),
        SelectedBlockReferenceUseEnabledStaticSemanticsOutcome::Rejected(
            SelectedStaticSemanticsRejection::LexicalVarNameCollision { .. }
        )
    ));
    // No relation surface is reachable without an accepted witness: the
    // locally valid `-b-c;` use-site never publishes a relation once the
    // existing lexical/var static rejection wins.
}

// --- Issue #781: the plain-left right-unary additive spelling retains the
// same ordered `Two` facts consumed by the existing correspondence layer. ---

#[test]
fn right_unary_use_site_reuses_per_operand_correspondence_in_authored_order() {
    let (_, script) = recognized_reference_use("let a;\nvar b;\na+-b;");
    let analysis = accepted_use_site_analysis(&script);
    let [a, b] = analysis.relations() else {
        panic!("expected exactly two use-site relations");
    };
    assert_eq!(a.semantic_name(), "a");
    assert_eq!(b.semantic_name(), "b");
    assert!(a.reference().range().start() < b.reference().range().start());
    assert!(a.correspondence().selected_lexical_binding().is_some());
    assert_eq!(
        b.correspondence()
            .var_contributors()
            .expect("b must resolve to the same-source var contributor")
            .len(),
        1
    );
}
