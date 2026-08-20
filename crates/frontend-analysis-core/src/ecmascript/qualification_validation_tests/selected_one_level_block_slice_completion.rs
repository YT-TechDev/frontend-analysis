//! Candidate-independent completion closure for the current selected ECMAScript
//! slice after the accepted one-level Block extension (Issue #299).
//!
//! This validation-only theorem aggregates frozen first-envelope inventory and
//! already-accepted candidate-independent authorities. It deliberately does not
//! call production lexical, static-semantics, aggregate qualification, Binding /
//! Scope, or runtime behavior to derive expected completion meaning.

use std::collections::BTreeSet;

use super::inventory::{
    RULE_UNITS, RuleUnit, RuleUnitKind, aggregate_gold_coverage_complete,
    aggregate_rule_expansion_complete,
};

const HISTORICAL_FLAT_COMPLETION: &str = include_str!("selected_slice_completion.rs");
const BLOCK_ORACLE: &str =
    include_str!("../qualification_selected_one_level_block_validation_tests.rs");
const BLOCK_COMPOSITION_ORACLE: &str =
    include_str!("../qualification_selected_one_level_block_composition_validation_tests.rs");
const TERMINAL_COMPOSITION_ORACLE: &str = include_str!(
    "../qualification_selected_one_level_block_terminal_composition_validation_tests.rs"
);
const SELECTED_AGGREGATE_SOURCE: &str = include_str!("../selected_qualification_integration.rs");
const QUALIFICATION_SOURCE: &str = include_str!("../qualification.rs");
const THIS_SOURCE: &str = include_str!("selected_one_level_block_slice_completion.rs");

const SELECTED_ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const SELECTED_ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const SELECTED_UNICODE_VERSION: &str = "17.0.0";
const SELECTED_SOURCE_AUTHORITY: &str = "Core UTF-8 SourceText";
const SELECTED_SOURCE_CONTEXT: &str = "Independent Source Unit";
const SELECTED_PARSE_GOAL: &str = "Script";
const CURRENT_SELECTED_TOP_LEVEL_ITEM_GRAMMAR: &str = "LexicalDeclaration | SelectedBlock";
const CURRENT_SELECTED_BLOCK_BODY: &str = "LexicalDeclaration+";

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
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedPositivePartition {
    HistoricalFlat,
    BlockEnabled,
}

fn partition_selected_positive_by_block_count(block_count: usize) -> SelectedPositivePartition {
    if block_count == 0 {
        SelectedPositivePartition::HistoricalFlat
    } else {
        SelectedPositivePartition::BlockEnabled
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvidenceProvenance {
    HistoricalFrozen,
    PostInventoryBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NonTriggerReason {
    HistoricalDispositionPreserved,
    SelectedBlockHasNoVarDeclaredNamesContributor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurrentSelectedDisposition {
    RequiredEvidence(EvidenceProvenance),
    StructurallyNonTriggering(NonTriggerReason),
}

fn classify_current_selected_rule(rule: &RuleUnit) -> CurrentSelectedDisposition {
    if HISTORICAL_REQUIRED_RULE_IDS.contains(&rule.id) {
        return CurrentSelectedDisposition::RequiredEvidence(EvidenceProvenance::HistoricalFrozen);
    }

    match rule.id {
        "EE-14-R01" => {
            CurrentSelectedDisposition::RequiredEvidence(EvidenceProvenance::PostInventoryBlock)
        }
        "EE-14-R02" => CurrentSelectedDisposition::StructurallyNonTriggering(
            NonTriggerReason::SelectedBlockHasNoVarDeclaredNamesContributor,
        ),
        _ => CurrentSelectedDisposition::StructurallyNonTriggering(
            NonTriggerReason::HistoricalDispositionPreserved,
        ),
    }
}

#[test]
fn fixed_envelope_and_accepted_authority_chain_are_present() {
    assert_eq!(SELECTED_ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        SELECTED_ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(SELECTED_UNICODE_VERSION, "17.0.0");
    assert_eq!(SELECTED_SOURCE_AUTHORITY, "Core UTF-8 SourceText");
    assert_eq!(SELECTED_SOURCE_CONTEXT, "Independent Source Unit");
    assert_eq!(SELECTED_PARSE_GOAL, "Script");

    assert!(HISTORICAL_FLAT_COMPLETION.contains("Candidate-independent completion proof"));
    assert!(HISTORICAL_FLAT_COMPLETION.contains(
        "const SELECTED_TOP_LEVEL_GRAMMAR: &str = \"LexicalDeclaration+\";"
    ));
    assert!(HISTORICAL_FLAT_COMPLETION.contains("assert_eq!(required.len(), 7);"));
    assert!(HISTORICAL_FLAT_COMPLETION.contains("(\"EE-14\", \"Block\")"));

    assert!(BLOCK_ORACLE.contains("const SELECTED_BLOCK_BODY: &str = \"LexicalDeclaration+\";"));
    assert!(BLOCK_ORACLE.contains("const BLOCK_RULES"));
    assert!(BLOCK_ORACLE.contains("EE-14-R01"));
    assert!(BLOCK_ORACLE.contains("fixture_id: \"block-local-duplicate\""));
    assert!(BLOCK_ORACLE.contains("EE-14-R02"));
    assert!(BLOCK_ORACLE.contains("selected Block contains no VarDeclaredNames contributor"));

    assert!(BLOCK_COMPOSITION_ORACLE.contains("GRAMMAR_VS_BLOCK"));
    assert!(BLOCK_COMPOSITION_ORACLE.contains("STATIC_TIER_FIXTURES"));
    assert!(BLOCK_COMPOSITION_ORACLE.contains("REGION_DOMAIN_FIXTURES"));
    assert!(BLOCK_COMPOSITION_ORACLE.contains("SHORT_FIXED_DELIMITER_FIXTURES"));

    assert!(TERMINAL_COMPOSITION_ORACLE.contains("KEYWORD_ADJACENT_FIXTURES"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("EOF_COMPOSITION_FIXTURES"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("INCOMPLETE_BLOCK_FIXTURES"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("INCOMPLETE_BLOCK_GRAMMAR_FIXTURES"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("DEFERRED_EOF_AFTER_BLOCK"));
}

#[test]
fn current_selected_positive_source_space_is_exact_disjoint_union() {
    assert_eq!(
        CURRENT_SELECTED_TOP_LEVEL_ITEM_GRAMMAR,
        "LexicalDeclaration | SelectedBlock"
    );
    assert_eq!(CURRENT_SELECTED_BLOCK_BODY, "LexicalDeclaration+");
    assert!(HISTORICAL_FLAT_COMPLETION.contains("LexicalDeclaration+"));
    assert!(BLOCK_ORACLE.contains("SELECTED_BLOCK_BODY"));
    assert!(BLOCK_ORACLE.contains("LexicalDeclaration+"));

    assert_eq!(
        partition_selected_positive_by_block_count(0),
        SelectedPositivePartition::HistoricalFlat
    );
    for block_count in [1, 2, 3, usize::MAX] {
        assert_eq!(
            partition_selected_positive_by_block_count(block_count),
            SelectedPositivePartition::BlockEnabled
        );
    }

    assert_ne!(
        SelectedPositivePartition::HistoricalFlat,
        SelectedPositivePartition::BlockEnabled
    );
}

#[test]
fn frozen_rule_universe_closes_as_exact_eight_plus_one_hundred_eighty_five() {
    assert!(aggregate_rule_expansion_complete(RULE_UNITS));
    assert!(
        !aggregate_gold_coverage_complete(RULE_UNITS),
        "later selected evidence must not retroactively mark historical aggregate gold complete"
    );

    let mut seen = BTreeSet::new();
    let mut required = BTreeSet::new();
    let mut historical_required = BTreeSet::new();
    let mut post_inventory_required = BTreeSet::new();
    let mut structurally_non_triggering = 0usize;
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

        match classify_current_selected_rule(rule) {
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
                        assert!(post_inventory_required.insert(rule.id));
                    }
                }
            }
            CurrentSelectedDisposition::StructurallyNonTriggering(reason) => {
                structurally_non_triggering += 1;
                match reason {
                    NonTriggerReason::HistoricalDispositionPreserved => {
                        assert!(!HISTORICAL_REQUIRED_RULE_IDS.contains(&rule.id));
                        assert_ne!(rule.id, "EE-14-R01");
                        assert_ne!(rule.id, "EE-14-R02");
                    }
                    NonTriggerReason::SelectedBlockHasNoVarDeclaredNamesContributor => {
                        assert_eq!(rule.id, "EE-14-R02");
                    }
                }
            }
        }
    }

    let expected_current: BTreeSet<_> = CURRENT_REQUIRED_RULE_IDS.iter().copied().collect();
    let expected_historical: BTreeSet<_> = HISTORICAL_REQUIRED_RULE_IDS.iter().copied().collect();

    assert_eq!(seen.len(), 193);
    assert_eq!(active, 183);
    assert_eq!(inactive, 10);
    assert_eq!(sentinels, 0);
    assert_eq!(required, expected_current);
    assert_eq!(required.len(), 8);
    assert_eq!(historical_required, expected_historical);
    assert_eq!(historical_required.len(), 7);
    assert_eq!(post_inventory_required, BTreeSet::from(["EE-14-R01"]));
    assert_eq!(structurally_non_triggering, 185);
    assert_eq!(required.len() + structurally_non_triggering, 193);
}

#[test]
fn ee14_delta_preserves_frozen_identity_without_rewriting_historical_gold() {
    let r01 = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-14-R01")
        .expect("EE-14-R01 must remain in the frozen inventory");
    let r02 = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-14-R02")
        .expect("EE-14-R02 must remain in the frozen inventory");

    assert_eq!(r01.kind, RuleUnitKind::NormativeRule);
    assert_eq!(r02.kind, RuleUnitKind::NormativeRule);
    assert!(
        r01.gold_fixture_ids.is_empty(),
        "post-inventory Block evidence must not be written into frozen inventory gold"
    );
    assert!(
        r02.gold_fixture_ids.is_empty(),
        "structurally non-triggering Block rule must not gain invented gold"
    );

    assert!(HISTORICAL_FLAT_COMPLETION.contains("(\"EE-14\", \"Block\")"));
    assert!(BLOCK_ORACLE.contains("EE-14-R01"));
    assert!(BLOCK_ORACLE.contains("block-local-duplicate"));
    assert!(BLOCK_ORACLE.contains("EE-14-R02"));
    assert!(BLOCK_ORACLE.contains("selected Block contains no VarDeclaredNames contributor"));
}

#[test]
fn composition_and_terminal_closure_remain_owned_by_prior_oracles() {
    for marker in [
        "GRAMMAR_VS_BLOCK",
        "STATIC_TIER_FIXTURES",
        "REGION_DOMAIN_FIXTURES",
        "DELIMITER_FIXTURES",
        "SHORT_FIXED_DELIMITER_FIXTURES",
    ] {
        assert!(
            BLOCK_COMPOSITION_ORACLE.contains(marker),
            "#295/#296 authority must retain {marker}"
        );
    }

    for marker in [
        "KEYWORD_ADJACENT_FIXTURES",
        "EOF_COMPOSITION_FIXTURES",
        "NON_EOF_ASI_FIXTURES",
        "INCOMPLETE_BLOCK_FIXTURES",
        "INCOMPLETE_BLOCK_GRAMMAR_FIXTURES",
        "INCOMPLETE_BLOCK_COMPETITION_FIXTURES",
        "DEFERRED_EOF_AFTER_BLOCK",
    ] {
        assert!(
            TERMINAL_COMPOSITION_ORACLE.contains(marker),
            "#297/#298 authority must retain {marker}"
        );
    }

    assert!(BLOCK_COMPOSITION_ORACLE.contains("RequiresLookahead(\";\")"));
    assert!(BLOCK_COMPOSITION_ORACLE.contains("RequiresEof"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("RequiresSemicolonLookahead"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("RequiresEof"));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("automatic_semicolon_authored_range: None"));
}

#[test]
fn selected_lifecycle_stays_incomplete_and_resource_failures_stay_distinct() {
    assert!(SELECTED_AGGREGATE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("UnsupportedCoverage"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("DefinitiveGrammarRejectionEvidence"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("QualificationOutcome::resource_limited()"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("QualificationOutcome::internal_failure()"));
    assert!(SELECTED_AGGREGATE_SOURCE.contains(
        "does not construct aggregate `QualificationOutcome::Qualified`"
    ));

    assert!(QUALIFICATION_SOURCE.contains("struct CompleteQualificationWitness"));
    assert!(QUALIFICATION_SOURCE.contains("#[cfg(test)]"));
    assert!(QUALIFICATION_SOURCE.contains("test_complete_qualification_witness"));
}

#[test]
fn completion_oracle_remains_candidate_independent_and_does_not_widen_capability() {
    for forbidden in [
        concat!("recognize_selected_", "lexical_slice("),
        concat!("evaluate_selected_", "static_semantics("),
        concat!("evaluate_selected_one_level_block_", "static_semantics("),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_", "binding_scope("),
        concat!("QualificationOutcome::", "qualified("),
        concat!("test_complete_qualification_", "witness()"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "#299 completion proof must not derive meaning from production: {forbidden}"
        );
    }

    for boundary in [
        "NestedBlock",
        "NonEofAsiBeforeClose",
        "EmptyBlock",
        "CommentTrivia",
        "VarDeclaration",
        "FunctionDeclaration",
        "ExpressionStatement",
    ] {
        assert!(
            BLOCK_ORACLE.contains(boundary),
            "Block frontier must retain unsupported boundary {boundary}"
        );
    }

    assert!(BLOCK_ORACLE.contains(
        "UnsupportedCoverage in the selected Block frontier != invalid ECMAScript"
    ));
    assert!(TERMINAL_COMPOSITION_ORACLE.contains("UnsupportedCoverage"));
}
