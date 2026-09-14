//! Candidate-independent executable completion successor for the selected
//! ECMAScript qualification slice after composing the accepted one-level
//! Block-contained `var` 1..N declarator theorem (Issue #693 / PR #694
//! validation; #695 / PR #696 production) with the already-accepted
//! bounded top-level decimal-integer initializer leaf (Issue #326 / PR
//! #327), frozen for Issue #697 (validation only; production is explicitly
//! not authorized by this Issue).
//!
//! This checkpoint composes the immutable post-#693 completion authority
//! with the new Block-var decimal-initializer oracle. It deliberately does
//! not call production recognition, production static semantics,
//! qualification integration, Binding / Scope, source-name correspondence,
//! or runtime code to derive expected completion meaning. It also does not
//! rewrite the historical post-#693 checkpoint; that file remains frozen
//! evidence of the prior ten-rule state.
//!
//! This successor widens only the Block `var` declarator *source family*
//! (bare identifier -> identifier with an optional bounded decimal
//! initializer). `EE-14-R02` and `EE-36-R02` are both already reachable via
//! #689/#693. It reaches no new Early Error identity: the newly-reachable
//! set is empty.

#![allow(clippy::assertions_on_constants)]

use std::collections::BTreeSet;

use super::inventory::{RULE_UNITS, RuleUnitKind};

const HISTORICAL_POST_693_COMPLETION: &str =
    include_str!("selected_post_693_block_var_multi_declarator_slice_completion.rs");
const BLOCK_VAR_DECIMAL_INITIALIZER_ORACLE: &str =
    include_str!("selected_one_level_block_var_decimal_initializer_frontier.rs");
const THIS_SOURCE: &str =
    include_str!("selected_post_block_var_decimal_initializer_slice_completion.rs");

const SELECTED_PARSE_GOAL: &str = "Script";
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";

/// This successor reaches no new Early Error identity.
const NEWLY_REACHABLE_RULE_IDS: &[&str] = &[];

/// Both `EE-14-R02` and `EE-36-R02` were already reachable via the post-#693
/// checkpoint; this leaf only widens their Block `var` declarator *source
/// family* (bare identifier -> identifier with an optional bounded decimal
/// initializer). They must not appear in `NEWLY_REACHABLE_RULE_IDS`.
const WIDENED_INITIALIZER_COMPOSITION_ONLY_RULE_IDS: &[&str] = &["EE-14-R02", "EE-36-R02"];

const POST_693_REQUIRED_RULE_IDS: &[&str] = &[
    "EE-01-R01",
    "EE-01-R02",
    "EE-04-R08",
    "EE-14-R01",
    "EE-14-R02",
    "EE-15-R01",
    "EE-15-R02",
    "EE-15-R03",
    "EE-36-R01",
    "EE-36-R02",
];

/// Identical to `POST_693_REQUIRED_RULE_IDS`: this successor widens the
/// Block `var` declarator source family only and reaches no new rule
/// identity, so the required-rule set is unchanged.
const POST_697_REQUIRED_RULE_IDS: &[&str] = &[
    "EE-01-R01",
    "EE-01-R02",
    "EE-04-R08",
    "EE-14-R01",
    "EE-14-R02",
    "EE-15-R01",
    "EE-15-R02",
    "EE-15-R03",
    "EE-36-R01",
    "EE-36-R02",
];

// Independently listed literal complement: every frozen rule identity that
// is not in `POST_697_REQUIRED_RULE_IDS`. Because no rule identity newly
// became reachable, this is exactly the frozen post-#693 complement,
// restated independently rather than derived from the historical file.
const POST_697_COMPLEMENT_RULE_IDS: &[&str] = &[
    "EE-02-R01",
    "EE-03-R01",
    "EE-04-R01",
    "EE-04-R02",
    "EE-04-I01",
    "EE-04-R03",
    "EE-04-R04",
    "EE-04-R05",
    "EE-04-R06",
    "EE-04-R07",
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
    "EE-16-I01",
    "EE-16-I02",
    "EE-16-I03",
    "EE-17-I01",
    "EE-18-I01",
    "EE-19-I01",
    "EE-19-R01",
    "EE-20-I01",
    "EE-20-R01",
    "EE-20-R02",
    "EE-20-R03",
    "EE-20-R04",
    "EE-20-R05",
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
    assert_eq!(SELECTED_PARSE_GOAL, "Script");
    assert_eq!(POSITIVE_LIFECYCLE, "SelectedAcceptedIncomplete");

    assert!(HISTORICAL_POST_693_COMPLETION.contains("POST_693_REQUIRED_RULE_IDS"));
    assert!(
        HISTORICAL_POST_693_COMPLETION
            .contains("assert_eq!(POST_693_REQUIRED_RULE_IDS.len(), 10);")
    );
    assert!(
        HISTORICAL_POST_693_COMPLETION
            .contains("assert_eq!(POST_693_COMPLEMENT_RULE_IDS.len(), 183);")
    );
    assert!(BLOCK_VAR_DECIMAL_INITIALIZER_ORACLE.contains("SelectedBlockVarDeclaration ::="));
    assert!(
        BLOCK_VAR_DECIMAL_INITIALIZER_ORACLE
            .contains("SELECTED_DECIMAL_INTEGER_GRAMMAR: &str = \"0 | [1-9][0-9]*\"")
    );
    assert!(
        BLOCK_VAR_DECIMAL_INITIALIZER_ORACLE.contains("SELECTED_LIST_CARDINALITY: &str = \"1..N\"")
    );
}

#[test]
fn post_693_historical_checkpoint_is_preserved_not_rewritten() {
    // This is a frozen-evidence guard: the historical ten-rule checkpoint
    // must continue to state ten required rules and must not have been
    // silently altered in place of adding this successor.
    assert!(HISTORICAL_POST_693_COMPLETION.contains("NEWLY_REACHABLE_RULE_IDS: &[&str] = &[];"));
    assert!(HISTORICAL_POST_693_COMPLETION.contains(
        "WIDENED_CONTRIBUTOR_CARDINALITY_ONLY_RULE_IDS: &[&str] = &[\"EE-14-R02\", \"EE-36-R02\"];"
    ));
    assert!(HISTORICAL_POST_693_COMPLETION.contains("assert_eq!(required_active, 10);"));
    assert!(HISTORICAL_POST_693_COMPLETION.contains("assert_eq!(complement_active, 173);"));
    assert!(HISTORICAL_POST_693_COMPLETION.contains("assert_eq!(complement_inactive, 10);"));
}

#[test]
fn no_new_early_error_identity_becomes_reachable() {
    assert!(NEWLY_REACHABLE_RULE_IDS.is_empty());
    assert_eq!(
        WIDENED_INITIALIZER_COMPOSITION_ONLY_RULE_IDS,
        ["EE-14-R02", "EE-36-R02"]
    );

    let previous: BTreeSet<_> = POST_693_REQUIRED_RULE_IDS.iter().copied().collect();
    let current: BTreeSet<_> = POST_697_REQUIRED_RULE_IDS.iter().copied().collect();

    assert_eq!(previous.len(), 10);
    assert_eq!(current.len(), 10);
    assert_eq!(
        previous, current,
        "the required-rule set is unchanged by this successor"
    );

    let newly_reachable: BTreeSet<_> = current.difference(&previous).copied().collect();
    assert!(newly_reachable.is_empty());

    for widened in WIDENED_INITIALIZER_COMPOSITION_ONLY_RULE_IDS {
        assert!(
            previous.contains(widened) && current.contains(widened),
            "{widened} must remain required before and after this successor"
        );
    }
}

#[test]
fn literal_post_697_partition_covers_every_frozen_rule_identity_exactly_once() {
    assert_eq!(POST_697_REQUIRED_RULE_IDS.len(), 10);
    assert_eq!(POST_697_COMPLEMENT_RULE_IDS.len(), 183);

    let required: BTreeSet<_> = POST_697_REQUIRED_RULE_IDS.iter().copied().collect();
    let complement: BTreeSet<_> = POST_697_COMPLEMENT_RULE_IDS.iter().copied().collect();
    assert_eq!(required.len(), 10);
    assert_eq!(complement.len(), 183);
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

    assert_eq!(required_active, 10);
    assert_eq!(complement_active, 173);
    assert_eq!(complement_inactive, 10);
    assert_eq!(sentinels, 0);
    assert_eq!(
        required_active + complement_active + complement_inactive,
        193
    );

    for rule_id in ["EE-14-R02", "EE-36-R02"] {
        let rule = RULE_UNITS
            .iter()
            .find(|rule| rule.id == rule_id)
            .unwrap_or_else(|| panic!("{rule_id} must remain frozen"));
        assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
        assert!(required.contains(rule_id));
        assert!(!complement.contains(rule_id));
    }
}

#[test]
fn arithmetic_identities_hold() {
    assert_eq!(10 + 183, 193);
    assert_eq!(173 + 10, 183);
}

#[test]
fn completion_successor_remains_candidate_independent_and_does_not_widen_capability() {
    for forbidden in [
        concat!("recognize_selected_", "lexical_slice("),
        concat!("evaluate_selected_", "static_semantics("),
        concat!("evaluate_selected_one_level_block_", "static_semantics("),
        concat!("evaluate_selected_variable_statement_", "static_semantics("),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_", "binding_scope("),
        concat!("analyze_selected_one_level_block_", "binding_scope("),
        concat!(
            "analyze_selected_variable_statement_name_",
            "correspondence("
        ),
        concat!("QualificationOutcome::", "qualified("),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "#697 completion successor must not derive meaning from production: {forbidden}"
        );
    }

    assert!(THIS_SOURCE.contains("POST_697_REQUIRED_RULE_IDS"));
    assert!(THIS_SOURCE.contains("POST_697_COMPLEMENT_RULE_IDS"));
    assert!(THIS_SOURCE.contains("NEWLY_REACHABLE_RULE_IDS"));
}
