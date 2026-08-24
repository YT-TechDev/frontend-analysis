//! Candidate-independent executable completion successor for the selected
//! ECMAScript qualification slice after the source/semantic production frontier
//! accepted through #343 (Issue #346).
//!
//! This checkpoint composes immutable historical selected-slice completion
//! authority with the accepted post-#313 validation successors. It deliberately
//! does not call production recognition, production static semantics,
//! qualification integration, Binding / Scope, source-name correspondence, or
//! runtime code to derive expected completion meaning.

use std::collections::BTreeSet;

use super::inventory::{RULE_UNITS, RuleUnitKind};

const HISTORICAL_VAR_COMPLETION: &str =
    include_str!("selected_variable_statement_slice_completion.rs");
const EOF_ASI_ORACLE: &str =
    include_str!("selected_variable_statement_eof_asi_frontier.rs");
const MULTI_DECLARATOR_ORACLE: &str =
    include_str!("selected_variable_statement_multi_declarator_frontier.rs");
const DECIMAL_INITIALIZER_ORACLE: &str =
    include_str!("selected_variable_statement_decimal_initializer_frontier.rs");
const DIRECT_IDENTIFIER_REFERENCE_ORACLE: &str = include_str!(
    "selected_variable_statement_direct_identifier_reference_initializer_frontier.rs"
);
const MULTI_REFERENCE_COMPOSITION_ORACLE: &str = include_str!(
    "selected_variable_statement_direct_identifier_reference_multi_reference_composition.rs"
);
const ESCAPED_IDENTIFIER_REFERENCE_ORACLE: &str = include_str!(
    "selected_variable_statement_escaped_identifier_reference_initializer_frontier.rs"
);
const EE04_INITIALIZER_ORACLE: &str =
    include_str!("selected_variable_statement_ee04_initializer_frontier.rs");
const RESEARCH_CHECKPOINT: &str = include_str!(
    "../../../../../docs/evidence/javascript/2026-08-post-343-selected-ecmascript-research-checkpoint.md"
);
const THIS_SOURCE: &str = include_str!("selected_post_343_slice_completion.rs");

const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const SOURCE_AUTHORITY: &str = "Core UTF-8 SourceText";
const SOURCE_CONTEXT: &str = "Independent Source Unit";
const PARSE_GOAL: &str = "Script";
const IS_STRICT: bool = false;
const YIELD_PARAMETER: bool = false;
const AWAIT_PARAMETER: bool = false;
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";

fn aggregate_qualified_available() -> bool {
    false
}

const SELECTED_TOP_LEVEL_ITEM_GRAMMAR: &str =
    "LexicalDeclaration | SelectedBlock | SelectedVariableStatement";
const SELECTED_BLOCK_BODY: &str = "LexicalDeclaration+";
const SELECTED_VAR_DECLARATION_LIST_CARDINALITY: &str = "1..N";
const SELECTED_DECIMAL_INTEGER_GRAMMAR: &str = "0 | [1-9][0-9]*";
const SELECTED_VAR_INITIALIZER_ROUTES: &[&str] = &[
    "Absent",
    "SelectedDecimalInteger",
    "SelectedDirectIdentifierReference",
    "SelectedEscapedNonReservedIdentifierReference",
    "SelectedEscapedReservedIdentifierName[C6 rejection route]",
];
const SELECTED_VAR_TERMINATORS: &[&str] = &["AuthoredSemicolon", "AutomaticAtEof"];

const BLOCK_LOCAL_VAR_DECLARED_NAMES_CONTRIBUTOR: bool = false;
const EE14_R02_NON_TRIGGER_REASON: &str =
    "SelectedBlock body is LexicalDeclaration+ and has no local VarDeclaredNames contributor";

const UNSUPPORTED_COVERAGE_BOUNDARY: &str =
    "UnsupportedCoverage is not evidence that source is invalid ECMAScript";
const QUALIFIED_BOUNDARY: &str =
    "SelectedAcceptedIncomplete is not aggregate Qualified";
const CORRESPONDENCE_RUNTIME_BOUNDARY: &str =
    "same-source correspondence is not ResolveBinding";
const VAR_CONTRIBUTOR_RUNTIME_BOUNDARY: &str =
    "same-source selected var contributor is authored provenance, not runtime binding identity";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositivePartition {
    HistoricalFlat,
    BlockEnabledWithoutVar,
    VariableEnabled,
}

fn positive_partition(block_count: usize, variable_statement_count: usize) -> PositivePartition {
    if variable_statement_count > 0 {
        PositivePartition::VariableEnabled
    } else if block_count > 0 {
        PositivePartition::BlockEnabledWithoutVar
    } else {
        PositivePartition::HistoricalFlat
    }
}

#[derive(Debug, Clone, Copy)]
struct GrammarDelta {
    id: &'static str,
    authority: &'static str,
    authority_issue_marker: &'static str,
    semantic_marker: &'static str,
    newly_reachable_outside_required: bool,
}

const POST_313_GRAMMAR_DELTAS: &[GrammarDelta] = &[
    GrammarDelta {
        id: "eof-only-asi",
        authority: EOF_ASI_ORACLE,
        authority_issue_marker: "Issue #318",
        semantic_marker: "AutomaticAtEof",
        newly_reachable_outside_required: false,
    },
    GrammarDelta {
        id: "multi-declarator",
        authority: MULTI_DECLARATOR_ORACLE,
        authority_issue_marker: "Issue #322",
        semantic_marker: "1..N",
        newly_reachable_outside_required: false,
    },
    GrammarDelta {
        id: "decimal-initializer",
        authority: DECIMAL_INITIALIZER_ORACLE,
        authority_issue_marker: "Issue #326",
        semantic_marker: "0 | [1-9][0-9]*",
        newly_reachable_outside_required: false,
    },
    GrammarDelta {
        id: "direct-identifier-reference-initializer",
        authority: DIRECT_IDENTIFIER_REFERENCE_ORACLE,
        authority_issue_marker: "Issue #330",
        semantic_marker: "direct-only",
        newly_reachable_outside_required: false,
    },
    GrammarDelta {
        id: "escaped-non-reserved-identifier-reference-initializer",
        authority: ESCAPED_IDENTIFIER_REFERENCE_ORACLE,
        authority_issue_marker: "Issue #336",
        semantic_marker: "frontier-scoped UnsupportedCoverage",
        newly_reachable_outside_required: false,
    },
    GrammarDelta {
        id: "escaped-reserved-ee04-initializer-source-position",
        authority: EE04_INITIALIZER_ORACLE,
        authority_issue_marker: "Issue #340",
        semantic_marker: "C6_RESERVED",
        newly_reachable_outside_required: false,
    },
];

const POST_343_REQUIRED_RULE_IDS: &[&str] = &[
    "EE-01-R01",
    "EE-01-R02",
    "EE-04-R08",
    "EE-14-R01",
    "EE-15-R01",
    "EE-15-R02",
    "EE-15-R03",
    "EE-36-R01",
    "EE-36-R02",
];

const POST_343_COMPLEMENT_RULE_IDS: &[&str] = &[
    "EE-02-R01",
    "EE-03-R01",
    "EE-04-R01",
    "EE-04-R02",
    "EE-04-R03",
    "EE-04-R04",
    "EE-04-R05",
    "EE-04-R06",
    "EE-04-R07",
    "EE-04-I01",
    "EE-04-I02",
    "EE-05-R01",
    "EE-05-R02",
    "EE-05-R03",
    "EE-05-R04",
    "EE-06-R01",
    "EE-07-R01",
    "EE-07-R02",
    "EE-07-R03",
    "EE-07-R04",
    "EE-07-R05",
    "EE-08-R01",
    "EE-09-R01",
    "EE-09-R02",
    "EE-10-R01",
    "EE-10-R02",
    "EE-11-R01",
    "EE-11-R02",
    "EE-12-R01",
    "EE-12-R02",
    "EE-12-R03",
    "EE-12-R04",
    "EE-13-R01",
    "EE-13-R02",
    "EE-13-R03",
    "EE-13-R04",
    "EE-14-R02",
    "EE-16-I01",
    "EE-16-I02",
    "EE-16-I03",
    "EE-17-I01",
    "EE-18-I01",
    "EE-19-R01",
    "EE-19-I01",
    "EE-20-R01",
    "EE-20-R02",
    "EE-20-R03",
    "EE-20-R04",
    "EE-20-R05",
    "EE-20-I01",
    "EE-21-R01",
    "EE-22-R01",
    "EE-23-R01",
    "EE-23-I01",
    "EE-24-R01",
    "EE-24-R02",
    "EE-25-R01",
    "EE-26-R01",
    "EE-26-R02",
    "EE-26-R03",
    "EE-27-R01",
    "EE-27-R02",
    "EE-28-R01",
    "EE-28-R02",
    "EE-28-R03",
    "EE-28-R04",
    "EE-28-R05",
    "EE-28-R06",
    "EE-28-R07",
    "EE-28-R08",
    "EE-28-R09",
    "EE-28-R10",
    "EE-28-R11",
    "EE-28-R12",
    "EE-28-R13",
    "EE-29-R01",
    "EE-29-R02",
    "EE-29-R03",
    "EE-29-R04",
    "EE-29-R05",
    "EE-30-R01",
    "EE-30-R02",
    "EE-30-R03",
    "EE-30-R04",
    "EE-30-R05",
    "EE-31-R01",
    "EE-31-R02",
    "EE-31-R03",
    "EE-31-R04",
    "EE-31-R05",
    "EE-31-R06",
    "EE-31-R07",
    "EE-31-R08",
    "EE-31-R09",
    "EE-31-R10",
    "EE-31-R11",
    "EE-31-R12",
    "EE-31-R13",
    "EE-32-R01",
    "EE-32-R02",
    "EE-32-R03",
    "EE-32-R04",
    "EE-32-R05",
    "EE-32-R06",
    "EE-32-R07",
    "EE-32-R08",
    "EE-32-R09",
    "EE-32-R10",
    "EE-32-R11",
    "EE-32-R12",
    "EE-32-R13",
    "EE-32-R14",
    "EE-32-R15",
    "EE-33-R01",
    "EE-33-R02",
    "EE-33-R03",
    "EE-33-R04",
    "EE-33-R05",
    "EE-33-R06",
    "EE-33-R07",
    "EE-33-R08",
    "EE-33-R09",
    "EE-33-R10",
    "EE-33-R11",
    "EE-33-R12",
    "EE-33-R13",
    "EE-33-R14",
    "EE-33-R15",
    "EE-33-R16",
    "EE-33-R17",
    "EE-33-R18",
    "EE-33-R19",
    "EE-33-R20",
    "EE-34-R01",
    "EE-34-R02",
    "EE-34-R03",
    "EE-34-R04",
    "EE-34-R05",
    "EE-34-R06",
    "EE-34-R07",
    "EE-34-R08",
    "EE-34-R09",
    "EE-34-R10",
    "EE-34-R11",
    "EE-34-R12",
    "EE-34-R13",
    "EE-35-R01",
    "EE-35-R02",
    "EE-35-R03",
    "EE-35-R04",
    "EE-35-R05",
    "EE-35-R06",
    "EE-36-R03",
    "EE-36-R04",
    "EE-36-R05",
    "EE-36-R06",
    "EE-36-R07",
    "EE-36-R08",
    "EE-37-R01",
    "EE-37-R02",
    "EE-37-R03",
    "EE-37-R04",
    "EE-37-R05",
    "EE-37-R06",
    "EE-37-R07",
    "EE-37-R08",
    "EE-37-R09",
    "EE-37-R10",
    "EE-37-R11",
    "EE-37-R12",
    "EE-37-R13",
    "EE-37-R14",
    "EE-37-R15",
    "EE-37-R16",
    "EE-37-R17",
    "EE-37-R18",
    "EE-37-R19",
    "EE-37-R20",
    "EE-37-R21",
    "EE-37-R22",
    "EE-37-R23",
    "EE-37-R24",
    "EE-37-R25",
    "EE-37-R26",
];

#[test]
fn fixed_envelope_and_checkpoint_lineage_are_exact() {
    assert_eq!(ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(UNICODE_VERSION, "17.0.0");
    assert_eq!(SOURCE_AUTHORITY, "Core UTF-8 SourceText");
    assert_eq!(SOURCE_CONTEXT, "Independent Source Unit");
    assert_eq!(PARSE_GOAL, "Script");
    assert!(!IS_STRICT);
    assert!(!YIELD_PARAMETER);
    assert!(!AWAIT_PARAMETER);
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert!(!aggregate_qualified_available());

    assert!(HISTORICAL_VAR_COMPLETION.contains("Issue #312"));
    assert!(
        HISTORICAL_VAR_COMPLETION
            .contains("VariableStatement ::= var SelectedBindingIdentifier ;")
    );
    assert!(
        RESEARCH_CHECKPOINT
            .contains("# Post-#343 Selected ECMAScript Research Checkpoint")
    );
    assert!(
        RESEARCH_CHECKPOINT.contains("CURRENT SELECTED SEMANTIC CLOSURE: PASS")
    );
    assert!(RESEARCH_CHECKPOINT.contains("CURRENT AGGREGATE EXECUTABLE COMPLETION AUTHORITY: STALE / NEEDS SUCCESSOR"));
}

#[test]
fn post_343_selected_source_envelope_is_checkpoint_specific() {
    assert_eq!(
        SELECTED_TOP_LEVEL_ITEM_GRAMMAR,
        "LexicalDeclaration | SelectedBlock | SelectedVariableStatement"
    );
    assert_eq!(SELECTED_BLOCK_BODY, "LexicalDeclaration+");
    assert_eq!(SELECTED_VAR_DECLARATION_LIST_CARDINALITY, "1..N");
    assert_eq!(SELECTED_DECIMAL_INTEGER_GRAMMAR, "0 | [1-9][0-9]*");
    assert_eq!(SELECTED_VAR_INITIALIZER_ROUTES.len(), 5);
    assert_eq!(
        SELECTED_VAR_INITIALIZER_ROUTES,
        [
            "Absent",
            "SelectedDecimalInteger",
            "SelectedDirectIdentifierReference",
            "SelectedEscapedNonReservedIdentifierReference",
            "SelectedEscapedReservedIdentifierName[C6 rejection route]",
        ]
    );
    assert_eq!(
        SELECTED_VAR_TERMINATORS,
        ["AuthoredSemicolon", "AutomaticAtEof"]
    );

    assert_eq!(positive_partition(0, 0), PositivePartition::HistoricalFlat);
    assert_eq!(
        positive_partition(1, 0),
        PositivePartition::BlockEnabledWithoutVar
    );
    assert_eq!(
        positive_partition(0, 1),
        PositivePartition::VariableEnabled
    );
    assert_eq!(
        positive_partition(2, 3),
        PositivePartition::VariableEnabled
    );

    assert!(!BLOCK_LOCAL_VAR_DECLARED_NAMES_CONTRIBUTOR);
    assert!(EE14_R02_NON_TRIGGER_REASON.contains("no local VarDeclaredNames contributor"));
}

#[test]
fn literal_post_343_partition_covers_every_frozen_rule_identity_exactly_once() {
    assert_eq!(POST_343_REQUIRED_RULE_IDS.len(), 9);
    assert_eq!(POST_343_COMPLEMENT_RULE_IDS.len(), 184);

    let required: BTreeSet<_> = POST_343_REQUIRED_RULE_IDS.iter().copied().collect();
    let complement: BTreeSet<_> = POST_343_COMPLEMENT_RULE_IDS.iter().copied().collect();
    assert_eq!(required.len(), 9);
    assert_eq!(complement.len(), 184);
    assert!(required.is_disjoint(&complement));

    let expected_all: BTreeSet<_> = required.union(&complement).copied().collect();
    let actual_all: BTreeSet<_> = RULE_UNITS.iter().map(|rule| rule.id).collect();
    assert_eq!(expected_all.len(), 193);
    assert_eq!(actual_all.len(), 193);
    assert_eq!(expected_all, actual_all);

    let mut required_active = 0usize;
    let mut complement_active = 0usize;
    let mut complement_inactive = 0usize;
    let mut sentinels = 0usize;

    for rule in RULE_UNITS {
        if required.contains(rule.id) {
            assert_eq!(
                rule.kind,
                RuleUnitKind::NormativeRule,
                "{} required rule must remain active",
                rule.id
            );
            required_active += 1;
            continue;
        }

        assert!(
            complement.contains(rule.id),
            "{} must belong to the literal complement",
            rule.id
        );
        match rule.kind {
            RuleUnitKind::NormativeRule => complement_active += 1,
            RuleUnitKind::EnvelopeInactiveRule => complement_inactive += 1,
            RuleUnitKind::ExpansionSentinel => sentinels += 1,
        }
    }

    assert_eq!(required_active, 9);
    assert_eq!(complement_active, 174);
    assert_eq!(complement_inactive, 10);
    assert_eq!(sentinels, 0);
    assert_eq!(required_active + complement_active + complement_inactive, 193);

    assert!(complement.contains("EE-14-R02"));
    let ee14_r02 = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-14-R02")
        .expect("EE-14-R02 must remain frozen");
    assert_eq!(ee14_r02.kind, RuleUnitKind::NormativeRule);
    assert!(!BLOCK_LOCAL_VAR_DECLARED_NAMES_CONTRIBUTOR);
}

#[test]
fn six_post_313_grammar_deltas_are_bounded_successors() {
    assert_eq!(POST_313_GRAMMAR_DELTAS.len(), 6);

    let ids: BTreeSet<_> = POST_313_GRAMMAR_DELTAS
        .iter()
        .map(|delta| delta.id)
        .collect();
    assert_eq!(ids.len(), 6);

    for delta in POST_313_GRAMMAR_DELTAS {
        assert!(
            delta.authority.contains(delta.authority_issue_marker),
            "{} must retain its candidate-independent Issue identity",
            delta.id
        );
        assert!(
            delta.authority.contains(delta.semantic_marker),
            "{} must retain its load-bearing semantic marker",
            delta.id
        );
        assert!(
            delta.authority.contains("CURRENT_REQUIRED_RULE_IDS"),
            "{} must preserve the nine-rule successor set",
            delta.id
        );
        assert!(
            !delta.newly_reachable_outside_required,
            "{} must not introduce a tenth current rule identity",
            delta.id
        );
    }

    assert!(EOF_ASI_ORACLE.contains("AutomaticAtEof"));
    assert!(MULTI_DECLARATOR_ORACLE.contains("1..N"));
    assert!(DECIMAL_INITIALIZER_ORACLE.contains("EE-02-R01"));
    assert!(DIRECT_IDENTIFIER_REFERENCE_ORACLE.contains("direct-only"));
    assert!(
        ESCAPED_IDENTIFIER_REFERENCE_ORACLE
            .contains("frontier-scoped UnsupportedCoverage")
    );
    assert!(EE04_INITIALIZER_ORACLE.contains("EE-04-R08"));
    assert!(EE04_INITIALIZER_ORACLE.contains("C6_RESERVED"));
}

#[test]
fn multi_reference_is_a_semantic_cardinality_supplement_not_a_seventh_grammar_delta() {
    assert!(MULTI_REFERENCE_COMPOSITION_ORACLE.contains("Issue #332"));
    assert!(MULTI_REFERENCE_COMPOSITION_ORACLE.contains("cardinality"));
    assert_eq!(POST_313_GRAMMAR_DELTAS.len(), 6);
}

#[test]
fn completion_checkpoint_preserves_lifecycle_and_runtime_firewalls() {
    assert!(UNSUPPORTED_COVERAGE_BOUNDARY.contains("not evidence"));
    assert!(QUALIFIED_BOUNDARY.contains("not aggregate Qualified"));
    assert!(CORRESPONDENCE_RUNTIME_BOUNDARY.contains("not ResolveBinding"));
    assert!(VAR_CONTRIBUTOR_RUNTIME_BOUNDARY.contains("not runtime binding identity"));
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");
    assert!(!aggregate_qualified_available());
}

#[test]
fn successor_source_is_candidate_independent_and_does_not_call_production_owners() {
    for forbidden in [
        concat!("recognize_selected_lexical_", "slice("),
        concat!("evaluate_selected_static_", "semantics("),
        concat!("evaluate_selected_one_level_block_static_", "semantics("),
        concat!("evaluate_selected_variable_statement_static_", "semantics("),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_binding_", "scope("),
        concat!("analyze_selected_one_level_block_binding_", "scope("),
        concat!("analyze_selected_variable_statement_name_", "correspondence("),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "completion successor must not call production owner {forbidden}"
        );
    }

    assert!(THIS_SOURCE.contains("POST_343_REQUIRED_RULE_IDS"));
    assert!(THIS_SOURCE.contains("POST_343_COMPLEMENT_RULE_IDS"));
    assert!(THIS_SOURCE.contains("POST_313_GRAMMAR_DELTAS"));
}
