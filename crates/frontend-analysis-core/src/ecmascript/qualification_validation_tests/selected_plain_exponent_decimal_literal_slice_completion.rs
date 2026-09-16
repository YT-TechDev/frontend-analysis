//! Candidate-independent executable completion successor for Issue #735
//! (plain exponent `DecimalLiteral` initializer), composing the accepted
//! Issue #727 required-rule theorem (plain fractional `DecimalLiteral`
//! initializer) with the new Issue #735 atom.
//!
//! The Issue #735 oracle itself lives directly under `ecmascript/` (peer of
//! the other `LexicalDeclaration`-atom oracles) and therefore cannot reach
//! the private `super::inventory` module from outside `qualification_validation_tests`.
//! This checkpoint lives inside `qualification_validation_tests` so it can
//! compose the frozen, candidate-independent Early Error inventory
//! directly, and executably proves that adding the plain exponent
//! `DecimalLiteral` atom leaves the completion theorem unchanged.
//!
//! `required` and `complement` are the *reachability* partition: which
//! frozen Early Error rule identities are currently reachable/required
//! under the accepted selected slice, versus everything else in the
//! inventory. This is a different dimension from `RuleUnitKind`, which
//! classifies each rule's *applicability* under the frozen envelope
//! (`NormativeRule` / `EnvelopeInactiveRule` / `ExpansionSentinel`).
//! `required` rules are always `NormativeRule`, but the `complement` is a
//! mix of both `NormativeRule` (173) and `EnvelopeInactiveRule` (10) --
//! `required != EnvelopeInactiveRule` and `complement != NormativeRule`,
//! despite the coincidental numeric match (10 and 183 respectively). This
//! successor does not conflate the two partitions.
//!
//! It does not call production recognition, static semantics, qualification
//! integration, Binding/Scope, or correspondence code, and it does not
//! derive the theorem from production output.

#![allow(clippy::assertions_on_constants)]

use std::collections::BTreeSet;

use super::inventory::{RULE_UNITS, RuleUnitKind};

const HISTORICAL_FRACTIONAL_COMPLETION: &str =
    include_str!("selected_plain_fractional_decimal_literal_slice_completion.rs");
const EXPONENT_ORACLE_SOURCE: &str = include_str!(
    "../qualification_selected_plain_exponent_decimal_literal_initializer_validation_tests.rs"
);
const THIS_SOURCE: &str =
    include_str!("selected_plain_exponent_decimal_literal_slice_completion.rs");

const SELECTED_PARSE_GOAL: &str = "Script";
const POSITIVE_LIFECYCLE: &str = "SelectedAcceptedIncomplete";

/// This successor reaches no new Early Error identity: the plain exponent
/// `DecimalLiteral` RHS is initializer-source widening only -- no new
/// `BindingIdentifier`, `BoundName`, `IdentifierReference`, escaped
/// `ReservedWord` C6 source position, Block/Script declared-name
/// contributor, static rule family, or Grammar owner is introduced.
const NEWLY_REACHABLE_RULE_IDS: &[&str] = &[];

const PREVIOUS_REQUIRED_RULE_IDS: &[&str] = &[
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

/// Independently stated (not aliased to `PREVIOUS_REQUIRED_RULE_IDS`):
/// identical in membership, because Issue #735 widens only the initializer
/// RHS source family and reaches no new rule identity.
const CURRENT_REQUIRED_RULE_IDS: &[&str] = &[
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
// is not in `CURRENT_REQUIRED_RULE_IDS`. Because no rule identity newly
// became reachable, this is exactly the frozen post-#727 complement,
// restated independently rather than derived from the historical file.
const CURRENT_COMPLEMENT_RULE_IDS: &[&str] = &[
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

    assert!(HISTORICAL_FRACTIONAL_COMPLETION.contains("CURRENT_REQUIRED_RULE_IDS"));
    assert!(HISTORICAL_FRACTIONAL_COMPLETION.contains("CURRENT_COMPLEMENT_RULE_IDS"));
    assert!(
        HISTORICAL_FRACTIONAL_COMPLETION
            .contains("assert_eq!(CURRENT_REQUIRED_RULE_IDS.len(), 10);")
    );
    assert!(
        HISTORICAL_FRACTIONAL_COMPLETION
            .contains("assert_eq!(CURRENT_COMPLEMENT_RULE_IDS.len(), 183);")
    );
    assert!(EXPONENT_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 735"));
    assert!(EXPONENT_ORACLE_SOURCE.contains("SelectedAcceptedIncomplete"));
}

#[test]
fn fractional_historical_checkpoint_is_preserved_not_rewritten() {
    // Frozen-evidence guard: the historical checkpoint must still state its
    // own ten-rule required/183-rule complement partition, unmodified.
    assert!(HISTORICAL_FRACTIONAL_COMPLETION.contains("NEWLY_REACHABLE_RULE_IDS: &[&str] = &[];"));
    assert!(HISTORICAL_FRACTIONAL_COMPLETION.contains("assert_eq!(required_active, 10);"));
    assert!(HISTORICAL_FRACTIONAL_COMPLETION.contains("assert_eq!(complement_active, 173);"));
    assert!(HISTORICAL_FRACTIONAL_COMPLETION.contains("assert_eq!(complement_inactive, 10);"));
}

#[test]
fn no_new_early_error_identity_becomes_reachable() {
    assert!(NEWLY_REACHABLE_RULE_IDS.is_empty());

    let previous: BTreeSet<_> = PREVIOUS_REQUIRED_RULE_IDS.iter().copied().collect();
    let current: BTreeSet<_> = CURRENT_REQUIRED_RULE_IDS.iter().copied().collect();

    assert_eq!(previous.len(), 10);
    assert_eq!(current.len(), 10);
    assert_eq!(
        previous, current,
        "the required-rule set is unchanged by Issue #735"
    );

    let newly_reachable: BTreeSet<_> = current.difference(&previous).copied().collect();
    assert!(newly_reachable.is_empty(), "NEWLY_REACHABLE must stay {{}}");
}

#[test]
fn literal_current_partition_covers_every_frozen_rule_identity_exactly_once() {
    assert_eq!(CURRENT_REQUIRED_RULE_IDS.len(), 10);
    assert_eq!(CURRENT_COMPLEMENT_RULE_IDS.len(), 183);

    let required: BTreeSet<_> = CURRENT_REQUIRED_RULE_IDS.iter().copied().collect();
    let complement: BTreeSet<_> = CURRENT_COMPLEMENT_RULE_IDS.iter().copied().collect();
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
}

/// Falsifies the wrong model that motivated the earlier remediation:
/// `required` is not `RuleUnitKind::EnvelopeInactiveRule`, and `complement`
/// is not `RuleUnitKind::NormativeRule`. `RuleUnitKind` classifies
/// applicability under the frozen envelope; `required`/`complement`
/// classify reachability under the currently accepted selected slice. They
/// are independent partitions that happen to share the totals 10 and 183.
#[test]
fn required_complement_partition_is_distinct_from_rule_unit_kind_partition() {
    let normative = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .count();
    let inactive = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::EnvelopeInactiveRule)
        .count();
    let sentinels = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::ExpansionSentinel)
        .count();
    assert_eq!(normative, 183, "frozen inventory NormativeRule count");
    assert_eq!(inactive, 10, "frozen inventory EnvelopeInactiveRule count");
    assert_eq!(sentinels, 0, "frozen inventory ExpansionSentinel count");
    assert_eq!(normative + inactive + sentinels, RULE_UNITS.len());

    // A required rule (NormativeRule) that is emphatically not an
    // EnvelopeInactiveRule sitting in the complement:
    assert!(CURRENT_REQUIRED_RULE_IDS.contains(&"EE-01-R01"));
    let ee01_r01 = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-01-R01")
        .expect("EE-01-R01 must remain frozen");
    assert_eq!(ee01_r01.kind, RuleUnitKind::NormativeRule);

    // A complement rule that is EnvelopeInactiveRule -- proving the
    // complement is not simply "everything NormativeRule":
    assert!(CURRENT_COMPLEMENT_RULE_IDS.contains(&"EE-16-I01"));
    assert!(!CURRENT_REQUIRED_RULE_IDS.contains(&"EE-16-I01"));
    let ee16_i01 = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-16-I01")
        .expect("EE-16-I01 must remain frozen");
    assert_eq!(ee16_i01.kind, RuleUnitKind::EnvelopeInactiveRule);

    // A complement rule that is NormativeRule -- proving the complement is
    // not simply "everything EnvelopeInactiveRule" either:
    assert!(CURRENT_COMPLEMENT_RULE_IDS.contains(&"EE-02-R01"));
    assert!(!CURRENT_REQUIRED_RULE_IDS.contains(&"EE-02-R01"));
    let ee02_r01 = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-02-R01")
        .expect("EE-02-R01 must remain frozen");
    assert_eq!(ee02_r01.kind, RuleUnitKind::NormativeRule);
}

#[test]
fn arithmetic_identities_hold() {
    assert_eq!(10 + 183, 193);
    assert_eq!(173 + 10, 183);
}

#[test]
fn completion_successor_is_candidate_independent() {
    for forbidden in [
        concat!("recognize_selected_lexical_", "slice("),
        concat!("evaluate_selected_static_", "semantics("),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_binding_", "scope("),
        concat!(
            "analyze_selected_variable_statement_name_",
            "correspondence("
        ),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "completion successor must not derive meaning from production: {forbidden}"
        );
    }

    assert!(THIS_SOURCE.contains("CURRENT_REQUIRED_RULE_IDS"));
    assert!(THIS_SOURCE.contains("CURRENT_COMPLEMENT_RULE_IDS"));
    assert!(THIS_SOURCE.contains("NEWLY_REACHABLE_RULE_IDS"));
}
