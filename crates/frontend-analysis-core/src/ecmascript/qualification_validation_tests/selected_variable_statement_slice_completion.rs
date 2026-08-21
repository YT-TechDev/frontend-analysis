//! Candidate-independent completion closure for the current selected ECMAScript
//! slice after the accepted bounded top-level `VariableStatement` extension
//! (Issue #312).
//!
//! This validation-only theorem aggregates immutable historical completion
//! authority, frozen first-envelope inventory, and already-accepted
//! candidate-independent var evidence. It deliberately does not call production
//! recognition, static semantics, qualification, Binding / Scope, or runtime
//! behavior to derive expected completion meaning.

use std::collections::BTreeSet;

use super::{
    EXPECTED_RULE_COUNTS,
    inventory::{
        RULE_UNITS, RuleUnit, RuleUnitKind, aggregate_gold_coverage_complete,
        aggregate_rule_expansion_complete,
    },
};

const HISTORICAL_FLAT_COMPLETION: &str = include_str!("selected_slice_completion.rs");
const BLOCK_COMPLETION: &str = include_str!("selected_one_level_block_slice_completion.rs");
const BLOCK_ORACLE: &str =
    include_str!("../qualification_selected_one_level_block_validation_tests.rs");
const BLOCK_TERMINAL_COMPOSITION_ORACLE: &str = include_str!(
    "../qualification_selected_one_level_block_terminal_composition_validation_tests.rs"
);
const VAR_ORACLE: &str =
    include_str!("../qualification_selected_top_level_variable_statement_validation_tests.rs");
const VAR_COMPOSITION_ORACLE: &str = include_str!(
    "../qualification_selected_top_level_variable_statement_composition_validation_tests.rs"
);
const SELECTED_AGGREGATE_SOURCE: &str = include_str!("../selected_qualification_integration.rs");
const QUALIFICATION_SOURCE: &str = include_str!("../qualification.rs");
const THIS_SOURCE: &str = include_str!("selected_variable_statement_slice_completion.rs");

const SELECTED_ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const SELECTED_ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const SELECTED_UNICODE_VERSION: &str = "17.0.0";
const SELECTED_SOURCE_AUTHORITY: &str = "Core UTF-8 SourceText";
const SELECTED_SOURCE_CONTEXT: &str = "Independent Source Unit";
const SELECTED_PARSE_GOAL: &str = "Script";
const SELECTED_IS_STRICT: bool = false;
const SELECTED_YIELD: bool = false;
const SELECTED_AWAIT: bool = false;
const CURRENT_SELECTED_TOP_LEVEL_ITEM_GRAMMAR: &str =
    "LexicalDeclaration | SelectedBlock | SelectedVariableStatement";
const CURRENT_SELECTED_BLOCK_BODY: &str = "LexicalDeclaration+";
const CURRENT_SELECTED_VARIABLE_STATEMENT_GRAMMAR: &str =
    "VariableStatement ::= var SelectedBindingIdentifier ;";

const HISTORICAL_REQUIRED_RULE_IDS: &[&str] = &[
    "EE-01-R01",
    "EE-01-R02",
    "EE-04-R08",
    "EE-15-R01",
    "EE-15-R02",
    "EE-15-R03",
    "EE-36-R01",
];

const CURRENT_REQUIRED_RULE_IDS: &[&str] = &[
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

// Explicit identity-by-identity current classification. This list is not
// derived as `193 - CURRENT_REQUIRED_RULE_IDS.len()`. The completion test first
// validates that every frozen identity belongs to exactly one literal set, then
// tallies the resulting dispositions.
const CURRENT_STRUCTURALLY_NON_TRIGGERING_RULE_IDS: &[&str] = &[
    // EE-02
    "EE-02-R01",
    // EE-03
    "EE-03-R01",
    // EE-04
    "EE-04-R01",
    "EE-04-R02",
    "EE-04-R03",
    "EE-04-R04",
    "EE-04-R05",
    "EE-04-R06",
    "EE-04-R07",
    "EE-04-I01",
    "EE-04-I02",
    // EE-05
    "EE-05-R01",
    "EE-05-R02",
    "EE-05-R03",
    "EE-05-R04",
    // EE-06
    "EE-06-R01",
    // EE-07
    "EE-07-R01",
    "EE-07-R02",
    "EE-07-R03",
    "EE-07-R04",
    "EE-07-R05",
    // EE-08
    "EE-08-R01",
    // EE-09
    "EE-09-R01",
    "EE-09-R02",
    // EE-10
    "EE-10-R01",
    "EE-10-R02",
    // EE-11
    "EE-11-R01",
    "EE-11-R02",
    // EE-12
    "EE-12-R01",
    "EE-12-R02",
    "EE-12-R03",
    "EE-12-R04",
    // EE-13
    "EE-13-R01",
    "EE-13-R02",
    "EE-13-R03",
    "EE-13-R04",
    // EE-14
    "EE-14-R02",
    // EE-16
    "EE-16-I01",
    "EE-16-I02",
    "EE-16-I03",
    // EE-17
    "EE-17-I01",
    // EE-18
    "EE-18-I01",
    // EE-19
    "EE-19-R01",
    "EE-19-I01",
    // EE-20
    "EE-20-R01",
    "EE-20-R02",
    "EE-20-R03",
    "EE-20-R04",
    "EE-20-R05",
    "EE-20-I01",
    // EE-21
    "EE-21-R01",
    // EE-22
    "EE-22-R01",
    // EE-23
    "EE-23-R01",
    "EE-23-I01",
    // EE-24
    "EE-24-R01",
    "EE-24-R02",
    // EE-25
    "EE-25-R01",
    // EE-26
    "EE-26-R01",
    "EE-26-R02",
    "EE-26-R03",
    // EE-27
    "EE-27-R01",
    "EE-27-R02",
    // EE-28
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
    // EE-29
    "EE-29-R01",
    "EE-29-R02",
    "EE-29-R03",
    "EE-29-R04",
    "EE-29-R05",
    // EE-30
    "EE-30-R01",
    "EE-30-R02",
    "EE-30-R03",
    "EE-30-R04",
    "EE-30-R05",
    // EE-31
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
    // EE-32
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
    // EE-33
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
    // EE-34
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
    // EE-35
    "EE-35-R01",
    "EE-35-R02",
    "EE-35-R03",
    "EE-35-R04",
    "EE-35-R05",
    "EE-35-R06",
    // EE-36
    "EE-36-R03",
    "EE-36-R04",
    "EE-36-R05",
    "EE-36-R06",
    "EE-36-R07",
    "EE-36-R08",
    // EE-37
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedGrammarPositivePartition {
    HistoricalFlat,
    BlockEnabledWithoutVar,
    VariableEnabled,
}

fn partition_selected_grammar_positive(
    block_count: usize,
    variable_statement_count: usize,
) -> SelectedGrammarPositivePartition {
    if variable_statement_count > 0 {
        SelectedGrammarPositivePartition::VariableEnabled
    } else if block_count > 0 {
        SelectedGrammarPositivePartition::BlockEnabledWithoutVar
    } else {
        SelectedGrammarPositivePartition::HistoricalFlat
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvidenceProvenance {
    HistoricalFrozen,
    PostInventoryBlock,
    PostInventoryVar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NonTriggerReason {
    ExplicitCurrentClassification,
    SelectedBlockHasNoLocalVarDeclaredNamesContributor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurrentSelectedDisposition {
    RequiredEvidence(EvidenceProvenance),
    StructurallyNonTriggering(NonTriggerReason),
}

fn classify_current_selected_rule(rule: &RuleUnit) -> Option<CurrentSelectedDisposition> {
    if HISTORICAL_REQUIRED_RULE_IDS.contains(&rule.id) {
        return Some(CurrentSelectedDisposition::RequiredEvidence(
            EvidenceProvenance::HistoricalFrozen,
        ));
    }

    match rule.id {
        "EE-14-R01" => Some(CurrentSelectedDisposition::RequiredEvidence(
            EvidenceProvenance::PostInventoryBlock,
        )),
        "EE-36-R02" => Some(CurrentSelectedDisposition::RequiredEvidence(
            EvidenceProvenance::PostInventoryVar,
        )),
        "EE-14-R02" => Some(CurrentSelectedDisposition::StructurallyNonTriggering(
            NonTriggerReason::SelectedBlockHasNoLocalVarDeclaredNamesContributor,
        )),
        id if CURRENT_STRUCTURALLY_NON_TRIGGERING_RULE_IDS.contains(&id) => Some(
            CurrentSelectedDisposition::StructurallyNonTriggering(
                NonTriggerReason::ExplicitCurrentClassification,
            ),
        ),
        _ => None,
    }
}

#[test]
fn fixed_envelope_and_immutable_authority_chain_are_present() {
    assert_eq!(SELECTED_ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        SELECTED_ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(SELECTED_UNICODE_VERSION, "17.0.0");
    assert_eq!(SELECTED_SOURCE_AUTHORITY, "Core UTF-8 SourceText");
    assert_eq!(SELECTED_SOURCE_CONTEXT, "Independent Source Unit");
    assert_eq!(SELECTED_PARSE_GOAL, "Script");
    assert!(!SELECTED_IS_STRICT);
    assert!(!SELECTED_YIELD);
    assert!(!SELECTED_AWAIT);

    assert!(HISTORICAL_FLAT_COMPLETION.contains("assert_eq!(required.len(), 7);"));
    assert!(HISTORICAL_FLAT_COMPLETION.contains("validation-only local selected slice"));
    assert!(BLOCK_COMPLETION.contains("assert_eq!(required.len(), 8);"));
    assert!(BLOCK_COMPLETION.contains("assert_eq!(structurally_non_triggering, 185);"));
    assert!(BLOCK_COMPLETION.contains("EE-14-R01"));
    assert!(BLOCK_COMPLETION.contains("EE-14-R02"));

    assert!(VAR_ORACLE.contains("VariableStatement ::= var SelectedBindingIdentifier ;"));
    assert!(VAR_ORACLE.contains("SELECTED_VARIABLE_TOP_LEVEL_ONLY: bool = true"));
    assert!(VAR_ORACLE.contains("SELECTED_VARIABLE_SINGLE_BINDING_ONLY: bool = true"));
    assert!(VAR_ORACLE.contains("SELECTED_VARIABLE_INITIALIZER: bool = false"));
    assert!(VAR_ORACLE.contains("SELECTED_VARIABLE_DESTRUCTURING: bool = false"));
    assert!(VAR_ORACLE.contains("SELECTED_VARIABLE_EOF_ASI: bool = false"));
    assert!(VAR_ORACLE.contains("SELECTED_BLOCK_VAR: bool = false"));

    assert!(VAR_COMPOSITION_ORACLE.contains("MULTI_COLLISION_EVENTS"));
    assert!(VAR_COMPOSITION_ORACLE.contains("REPEATED_VAR_EVENTS"));
    assert!(VAR_COMPOSITION_ORACLE.contains("INVERSE_ORDER_EVENTS"));
    assert!(VAR_COMPOSITION_ORACLE.contains("keyword-adjacent-malformed-var-remains-unsupported"));
    assert!(VAR_COMPOSITION_ORACLE.contains("trivia-separated-malformed-binding-uses-general-grammar-evidence"));
    assert!(VAR_COMPOSITION_ORACLE.contains("formed-escape-continues-maximal-identifier-name"));
    assert!(VAR_COMPOSITION_ORACLE.contains("var-eof-asi-remains-deferred"));
}

#[test]
fn current_selected_grammar_positive_source_space_is_exact_disjoint_union() {
    assert_eq!(
        CURRENT_SELECTED_TOP_LEVEL_ITEM_GRAMMAR,
        "LexicalDeclaration | SelectedBlock | SelectedVariableStatement"
    );
    assert_eq!(CURRENT_SELECTED_BLOCK_BODY, "LexicalDeclaration+");
    assert_eq!(
        CURRENT_SELECTED_VARIABLE_STATEMENT_GRAMMAR,
        "VariableStatement ::= var SelectedBindingIdentifier ;"
    );

    assert_eq!(
        partition_selected_grammar_positive(0, 0),
        SelectedGrammarPositivePartition::HistoricalFlat
    );
    for block_count in [1, 2, 3, usize::MAX] {
        assert_eq!(
            partition_selected_grammar_positive(block_count, 0),
            SelectedGrammarPositivePartition::BlockEnabledWithoutVar
        );
    }
    for (block_count, var_count) in [
        (0, 1),
        (1, 1),
        (3, 2),
        (usize::MAX, 1),
        (0, usize::MAX),
    ] {
        assert_eq!(
            partition_selected_grammar_positive(block_count, var_count),
            SelectedGrammarPositivePartition::VariableEnabled
        );
    }

    assert_ne!(
        SelectedGrammarPositivePartition::HistoricalFlat,
        SelectedGrammarPositivePartition::BlockEnabledWithoutVar
    );
    assert_ne!(
        SelectedGrammarPositivePartition::HistoricalFlat,
        SelectedGrammarPositivePartition::VariableEnabled
    );
    assert_ne!(
        SelectedGrammarPositivePartition::BlockEnabledWithoutVar,
        SelectedGrammarPositivePartition::VariableEnabled
    );
}

#[test]
fn frozen_rule_universe_closes_as_exact_nine_plus_one_hundred_eighty_four() {
    assert!(aggregate_rule_expansion_complete(RULE_UNITS));
    assert!(
        !aggregate_gold_coverage_complete(RULE_UNITS),
        "later selected evidence must not retroactively mark historical aggregate gold complete"
    );

    let independently_expected_total: usize = EXPECTED_RULE_COUNTS
        .iter()
        .map(|(_, active, inactive)| active + inactive)
        .sum();
    assert_eq!(independently_expected_total, 193);

    let expected_required: BTreeSet<_> = CURRENT_REQUIRED_RULE_IDS.iter().copied().collect();
    let expected_non_triggering: BTreeSet<_> = CURRENT_STRUCTURALLY_NON_TRIGGERING_RULE_IDS
        .iter()
        .copied()
        .collect();
    assert_eq!(expected_required.len(), CURRENT_REQUIRED_RULE_IDS.len());
    assert_eq!(
        expected_non_triggering.len(),
        CURRENT_STRUCTURALLY_NON_TRIGGERING_RULE_IDS.len(),
        "explicit non-triggering identities must be unique"
    );
    assert!(expected_required.is_disjoint(&expected_non_triggering));

    let mut seen = BTreeSet::new();
    let mut required = BTreeSet::new();
    let mut non_triggering = BTreeSet::new();
    let mut historical_required = BTreeSet::new();
    let mut post_inventory_block_required = BTreeSet::new();
    let mut post_inventory_var_required = BTreeSet::new();
    let mut active = 0usize;
    let mut inactive = 0usize;
    let mut sentinels = 0usize;

    for rule in RULE_UNITS {
        assert!(seen.insert(rule.id), "duplicate frozen rule unit {}", rule.id);

        match rule.kind {
            RuleUnitKind::NormativeRule => active += 1,
            RuleUnitKind::EnvelopeInactiveRule => inactive += 1,
            RuleUnitKind::ExpansionSentinel => sentinels += 1,
        }

        let disposition = classify_current_selected_rule(rule)
            .unwrap_or_else(|| panic!("unclassified frozen rule unit {}", rule.id));
        match disposition {
            CurrentSelectedDisposition::RequiredEvidence(provenance) => {
                assert_eq!(
                    rule.kind,
                    RuleUnitKind::NormativeRule,
                    "reachable rule {} must remain an active frozen rule",
                    rule.id
                );
                assert!(required.insert(rule.id));
                match provenance {
                    EvidenceProvenance::HistoricalFrozen => {
                        assert!(HISTORICAL_REQUIRED_RULE_IDS.contains(&rule.id));
                        assert!(historical_required.insert(rule.id));
                    }
                    EvidenceProvenance::PostInventoryBlock => {
                        assert_eq!(rule.id, "EE-14-R01");
                        assert!(post_inventory_block_required.insert(rule.id));
                    }
                    EvidenceProvenance::PostInventoryVar => {
                        assert_eq!(rule.id, "EE-36-R02");
                        assert!(post_inventory_var_required.insert(rule.id));
                    }
                }
            }
            CurrentSelectedDisposition::StructurallyNonTriggering(reason) => {
                assert!(non_triggering.insert(rule.id));
                match reason {
                    NonTriggerReason::ExplicitCurrentClassification => {
                        assert!(CURRENT_STRUCTURALLY_NON_TRIGGERING_RULE_IDS.contains(&rule.id));
                        assert_ne!(rule.id, "EE-14-R02");
                    }
                    NonTriggerReason::SelectedBlockHasNoLocalVarDeclaredNamesContributor => {
                        assert_eq!(rule.id, "EE-14-R02");
                    }
                }
            }
        }
    }

    let expected_historical: BTreeSet<_> = HISTORICAL_REQUIRED_RULE_IDS.iter().copied().collect();

    assert_eq!(seen.len(), independently_expected_total);
    assert_eq!(active, 183);
    assert_eq!(inactive, 10);
    assert_eq!(sentinels, 0);
    assert_eq!(required, expected_required);
    assert_eq!(non_triggering, expected_non_triggering);
    assert_eq!(required.len(), 9);
    assert_eq!(non_triggering.len(), 184);
    assert_eq!(required.len() + non_triggering.len(), independently_expected_total);
    assert_eq!(historical_required, expected_historical);
    assert_eq!(historical_required.len(), 7);
    assert_eq!(post_inventory_block_required, BTreeSet::from(["EE-14-R01"]));
    assert_eq!(post_inventory_var_required, BTreeSet::from(["EE-36-R02"]));
}

#[test]
fn ee36_r02_is_the_only_new_reachability_delta() {
    let historical: BTreeSet<_> = HISTORICAL_REQUIRED_RULE_IDS.iter().copied().collect();
    let current: BTreeSet<_> = CURRENT_REQUIRED_RULE_IDS.iter().copied().collect();
    let historical_plus_block: BTreeSet<_> = historical
        .iter()
        .copied()
        .chain(["EE-14-R01"])
        .collect();
    let delta: BTreeSet<_> = current
        .difference(&historical_plus_block)
        .copied()
        .collect();

    assert_eq!(delta, BTreeSet::from(["EE-36-R02"]));
    assert!(VAR_ORACLE.contains("EE-36-R02"));
    assert!(VAR_ORACLE.contains("ScriptLexical"));
    assert!(VAR_ORACLE.contains("BlockLexical"));
    assert!(VAR_ORACLE.contains("ScriptVar"));
    assert!(VAR_ORACLE.contains("canonical-equivalence-does-not-normalize"));
    assert!(VAR_ORACLE.contains("repeated-var-alone-is-not-lexical-var-collision"));
    assert!(VAR_COMPOSITION_ORACLE.contains(
        "multi_collision_primary_is_first_collision_completed_in_authored_order"
    ));
    assert!(VAR_COMPOSITION_ORACLE.contains(
        "repeated_var_preserves_first_var_provenance_until_lexical_collision_completes"
    ));
    assert!(VAR_COMPOSITION_ORACLE.contains(
        "inverse_order_control_falsifies_always_lexical_or_hash_iteration_primary_selection"
    ));
}

#[test]
fn ee14_r02_remains_the_explicit_tenth_rule_counterexample() {
    assert!(CURRENT_STRUCTURALLY_NON_TRIGGERING_RULE_IDS.contains(&"EE-14-R02"));
    assert!(BLOCK_ORACLE.contains("const SELECTED_BLOCK_BODY: &str = \"LexicalDeclaration+\";"));
    assert!(BLOCK_ORACLE.contains("const SELECTED_VAR_DECLARATION: bool = false;"));
    assert!(BLOCK_ORACLE.contains("EE-14-R02"));
    assert!(
        BLOCK_ORACLE.contains("selected Block contains no VarDeclaredNames contributor")
    );
    assert!(VAR_ORACLE.contains("SELECTED_BLOCK_VAR: bool = false"));

    for rule_id in [
        "EE-04-R01",
        "EE-04-I01",
        "EE-19-R01",
        "EE-20-R03",
        "EE-24-R02",
        "EE-26-R03",
        "EE-28-R10",
        "EE-33-R14",
        "EE-36-R03",
        "EE-36-R04",
        "EE-36-R05",
        "EE-36-R06",
        "EE-36-R07",
        "EE-36-R08",
    ] {
        assert!(
            CURRENT_STRUCTURALLY_NON_TRIGGERING_RULE_IDS.contains(&rule_id),
            "serious additional-reachability candidate {rule_id} must stay explicitly classified"
        );
    }
}

#[test]
fn var_fixture_authority_preserves_script_domains_ordering_and_no_normalization() {
    for marker in [
        "lexical-before-var-collision",
        "var-before-lexical-collision",
        "escaped-var-equals-direct-lexical",
        "escaped-var-before-direct-lexical",
        "canonical-equivalence-does-not-normalize",
        "block-local-lexical-does-not-enter-script-domain",
        "top-level-lexical-still-collides-across-block-item",
        "existing-script-duplicate-precedes-lexical-var-collision",
        "existing-declaration-duplicate-precedes-lexical-var-collision",
        "escaped-reserved-var-remains-ee04-owned",
    ] {
        assert!(VAR_ORACLE.contains(marker), "#306/#307 authority must retain {marker}");
    }

    for marker in [
        "MULTI_COLLISION_EVENTS",
        "REPEATED_VAR_EVENTS",
        "INVERSE_ORDER_EVENTS",
        "first_completed_collision",
        "first_var_by_name",
    ] {
        assert!(
            VAR_COMPOSITION_ORACLE.contains(marker),
            "#308/#309 authority must retain {marker}"
        );
    }
}

#[test]
fn terminal_and_transactional_closure_remain_owned_by_prior_oracles() {
    for marker in [
        "multiple-var-declarators",
        "numeric-var-initializer",
        "identifier-var-initializer",
        "var-destructuring",
        "for-var",
        "block-var",
        "var-eof-asi-deferred",
    ] {
        assert!(VAR_ORACLE.contains(marker), "#306/#307 authority must retain {marker}");
    }

    for marker in [
        "keyword-adjacent-malformed-var-remains-unsupported",
        "trivia-separated-malformed-binding-uses-general-grammar-evidence",
        "formed-escape-continues-maximal-identifier-name",
        "authored-var-semicolon-composes-with-existing-final-lexical-eof-asi",
    ] {
        assert!(
            VAR_COMPOSITION_ORACLE.contains(marker),
            "#308/#309 composition authority must retain {marker}"
        );
    }

    for marker in [
        "INCOMPLETE_BLOCK_FIXTURES",
        "INCOMPLETE_BLOCK_GRAMMAR_FIXTURES",
        "INCOMPLETE_BLOCK_COMPETITION_FIXTURES",
        "DEFERRED_EOF_AFTER_BLOCK",
        "NON_EOF_ASI_FIXTURES",
    ] {
        assert!(
            BLOCK_TERMINAL_COMPOSITION_ORACLE.contains(marker),
            "#297/#298 transaction authority must retain {marker}"
        );
    }

    assert!(VAR_COMPOSITION_ORACLE.contains(r#"source: r"var\u{};""#));
    assert!(VAR_COMPOSITION_ORACLE.contains(r#"source: r"var \u{};""#));
    assert!(VAR_COMPOSITION_ORACLE.contains(r#"source: r"var\u0061;""#));
    assert!(VAR_COMPOSITION_ORACLE.contains("source: \"var x\""));
    assert!(VAR_COMPOSITION_ORACLE.contains("source: \"var x; let y\""));
}

#[test]
fn selected_lifecycle_stays_incomplete_and_binding_scope_stays_out_of_scope() {
    assert!(SELECTED_AGGREGATE_SOURCE.contains("RecognizedVariableStatementSlice"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("SelectedVariableStatementStaticSemanticsOutcome"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("UnsupportedCoverage"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("DefinitiveGrammarRejectionEvidence"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("QualificationOutcome::resource_limited()"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("QualificationOutcome::internal_failure()"));
    assert!(
        !SELECTED_AGGREGATE_SOURCE.contains(concat!("QualificationOutcome::", "qualified(")),
        "selected aggregate production must not construct Qualified"
    );

    assert!(QUALIFICATION_SOURCE.contains("struct CompleteQualificationWitness"));
    assert!(
        QUALIFICATION_SOURCE.contains("No production constructor is exposed by this foundation.")
    );

    for forbidden in [
        concat!("Environment", "Record"),
        concat!("Resolve", "Binding"),
        concat!("InitializedAt", "Reference"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "completion proof must not grow runtime Binding / Scope semantics: {forbidden}"
        );
    }
}

#[test]
fn completion_oracle_remains_candidate_independent_and_does_not_widen_capability() {
    for forbidden in [
        concat!("recognize_selected_", "lexical_slice("),
        concat!("evaluate_selected_", "static_semantics("),
        concat!("evaluate_selected_one_level_block_static_", "semantics("),
        concat!("evaluate_selected_variable_statement_static_", "semantics("),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_", "binding_scope("),
        concat!("analyze_selected_one_level_block_binding_", "scope("),
        concat!("QualificationOutcome::", "qualified("),
        concat!("test_complete_qualification_", "witness()"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "#312 completion proof must not derive expected meaning from production: {forbidden}"
        );
    }

    assert!(VAR_ORACLE.contains("UnsupportedCoverage"));
    assert!(VAR_COMPOSITION_ORACLE.contains("UnsupportedCoverage"));
    assert!(BLOCK_TERMINAL_COMPOSITION_ORACLE.contains("UnsupportedCoverage"));
    assert!(
        BLOCK_ORACLE
            .contains("UnsupportedCoverage in the selected Block frontier != invalid ECMAScript")
    );
}
