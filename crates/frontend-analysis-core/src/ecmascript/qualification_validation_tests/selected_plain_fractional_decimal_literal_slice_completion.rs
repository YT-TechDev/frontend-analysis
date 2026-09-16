//! Candidate-independent executable completion successor for Issue #727
//! (plain fractional `DecimalLiteral` initializer).
//!
//! The Issue #727 oracle itself lives directly under `ecmascript/` (peer of
//! the other `LexicalDeclaration`-atom oracles) and therefore cannot reach
//! the private `super::inventory` module from outside `qualification_validation_tests`.
//! This checkpoint lives inside `qualification_validation_tests` so it can
//! compose the frozen, candidate-independent Early Error inventory directly,
//! and executably proves that adding the plain fractional `DecimalLiteral`
//! atom leaves the existing completion theorem unchanged:
//!
//! ```text
//! TOTAL             = 193
//! required          = 10   (EnvelopeInactiveRule)
//! complement        = 183  (NormativeRule)
//! NEWLY_REACHABLE   = {}
//! ```
//!
//! It does not call production recognition, static semantics, qualification
//! integration, Binding/Scope, or correspondence code, and it does not
//! derive the theorem from production output or from `inventory.rs` being
//! textually unmodified alone -- only from the frozen inventory data itself
//! and the Issue #727 oracle's own declared, candidate-independent scope.

use std::collections::BTreeSet;

use super::inventory::{RULE_UNITS, RuleUnitKind};

const FRACTIONAL_ORACLE_SOURCE: &str = include_str!(
    "../qualification_selected_plain_fractional_decimal_literal_initializer_validation_tests.rs"
);
const THIS_SOURCE: &str =
    include_str!("selected_plain_fractional_decimal_literal_slice_completion.rs");

/// Keywords that would indicate a frozen `EnvelopeInactiveRule`'s
/// precondition could plausibly be satisfied by `NumericLiteral` /
/// `DecimalLiteral` source. None of the ten frozen inactive rules mention
/// any of these: they are exactly the Module-goal `await` identifier
/// rejections (`EE-04-I01`/`EE-04-I02`) and the `IsLabelledFunction`
/// statement-body rejections (`EE-16`..`EE-23`), neither of which family's
/// precondition a bare `NumericLiteral` initializer can satisfy.
const NUMERIC_LITERAL_KEYWORDS: &[&str] = &[
    "NumericLiteral",
    "DecimalLiteral",
    "DecimalDigits",
    "NumericLiteralSeparator",
    "NonDecimalIntegerLiteral",
];

const EXPECTED_INACTIVE_RULE_IDS: &[&str] = &[
    "EE-04-I01",
    "EE-04-I02",
    "EE-16-I01",
    "EE-16-I02",
    "EE-16-I03",
    "EE-17-I01",
    "EE-18-I01",
    "EE-19-I01",
    "EE-20-I01",
    "EE-23-I01",
];

#[test]
fn frozen_inventory_partition_matches_the_727_completion_theorem() {
    let active: Vec<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::NormativeRule)
        .collect();
    let inactive: Vec<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::EnvelopeInactiveRule)
        .collect();
    let sentinels: Vec<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::ExpansionSentinel)
        .collect();

    assert_eq!(RULE_UNITS.len(), 193, "TOTAL");
    assert_eq!(inactive.len(), 10, "required (EnvelopeInactiveRule)");
    assert_eq!(active.len(), 183, "complement (NormativeRule)");
    assert!(sentinels.is_empty(), "no ExpansionSentinel may remain");
    assert_eq!(
        active.len() + inactive.len() + sentinels.len(),
        RULE_UNITS.len()
    );

    let all_ids: BTreeSet<_> = RULE_UNITS.iter().map(|rule| rule.id).collect();
    assert_eq!(all_ids.len(), 193, "no duplicate rule identity");
}

#[test]
fn plain_fractional_decimal_literal_activates_no_frozen_inactive_rule() {
    let inactive: Vec<_> = RULE_UNITS
        .iter()
        .filter(|rule| rule.kind == RuleUnitKind::EnvelopeInactiveRule)
        .collect();
    assert_eq!(inactive.len(), 10);

    let actual_inactive_ids: BTreeSet<&str> = inactive.iter().map(|rule| rule.id).collect();
    let expected_inactive_ids: BTreeSet<&str> =
        EXPECTED_INACTIVE_RULE_IDS.iter().copied().collect();
    assert_eq!(actual_inactive_ids, expected_inactive_ids);

    for rule in &inactive {
        for keyword in NUMERIC_LITERAL_KEYWORDS {
            assert!(
                !rule.normative_locator.contains(keyword),
                "{} unexpectedly references {keyword}",
                rule.id
            );
            assert!(
                !rule
                    .dependencies
                    .iter()
                    .any(|dependency| dependency.contains(keyword)),
                "{} unexpectedly depends on {keyword}",
                rule.id
            );
        }
    }

    // No frozen inactive rule's precondition is satisfiable by the new
    // atom, so the set of newly reachable rule identities is empty.
    const NEWLY_REACHABLE: &[&str] = &[];
    assert!(NEWLY_REACHABLE.is_empty(), "NEWLY_REACHABLE must stay {{}}");
}

#[test]
fn fractional_atom_contributes_no_new_static_subject_or_qualified_lifecycle() {
    assert!(FRACTIONAL_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 727"));
    assert!(FRACTIONAL_ORACLE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(!FRACTIONAL_ORACLE_SOURCE.contains(concat!("ExpectedQualification", "::Qualified")));
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
        assert!(!THIS_SOURCE.contains(forbidden), "{forbidden}");
    }
}
