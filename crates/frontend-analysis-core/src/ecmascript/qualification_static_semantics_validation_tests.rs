//! Candidate-independent rule-local validation for Issue #208.
//!
//! This module records expected non-trigger meaning for the first selected
//! static-semantics slice without using a whole-source qualification verdict.
//! A selected rule not triggering is deliberately weaker than aggregate
//! `Qualified` and must remain independent of future production semantics.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};

#[allow(dead_code)]
#[path = "qualification_validation_tests/inventory.rs"]
mod inventory;

use inventory::{RULE_UNITS, RuleUnit, RuleUnitKind};

const EE_15_R01_GOLD_ID: &str = "JS-GOLD-LEXDECL-LET-BINDING-001";
const SCRIPT_DUPLICATE_GOLD_ID: &str = "JS-GOLD-SCRIPT-DUPLEXICAL-001";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuleNonTriggerFixture {
    id: &'static str,
    source: &'static str,
    rule_ids: &'static [&'static str],
}

const RULE_NON_TRIGGER_FIXTURES: &[RuleNonTriggerFixture] = &[
    RuleNonTriggerFixture {
        id: "JS-RULE-NONTRIGGER-CANONICAL-DISTINCT-001",
        source: "let é; let é;",
        rule_ids: &["EE-36-R01"],
    },
    RuleNonTriggerFixture {
        id: "JS-RULE-NONTRIGGER-AWAIT-SCRIPT-001",
        source: "let await=1;",
        rule_ids: &["EE-04-R04"],
    },
    RuleNonTriggerFixture {
        id: "JS-RULE-NONTRIGGER-YIELD-SCRIPT-001",
        source: "let yield=1;",
        rule_ids: &["EE-04-R02", "EE-04-R03"],
    },
    RuleNonTriggerFixture {
        id: "JS-RULE-NONTRIGGER-STATIC-NONSTRICT-001",
        source: "let static=1;",
        rule_ids: &["EE-04-R07"],
    },
    RuleNonTriggerFixture {
        id: "JS-RULE-NONTRIGGER-IMPLEMENTS-NONSTRICT-001",
        source: "let implements=1;",
        rule_ids: &["EE-04-R07"],
    },
    RuleNonTriggerFixture {
        id: "JS-RULE-NONTRIGGER-ARGUMENTS-NONSTRICT-001",
        source: "let arguments=1;",
        rule_ids: &["EE-04-R01"],
    },
    RuleNonTriggerFixture {
        id: "JS-RULE-NONTRIGGER-EVAL-NONSTRICT-001",
        source: "let eval=1;",
        rule_ids: &["EE-04-R01"],
    },
];

fn rule(rule_id: &str) -> &'static RuleUnit {
    RULE_UNITS
        .iter()
        .find(|rule| rule.id == rule_id)
        .unwrap_or_else(|| panic!("rule-local expectation maps to missing rule {rule_id}"))
}

#[test]
fn ee_15_r01_rejection_gold_is_exact_and_inventory_mapped() {
    assert_eq!(gold_source(EE_15_R01_GOLD_ID), Some("let let;"));
    assert_eq!(gold_subject_range(EE_15_R01_GOLD_ID), Some((4, 7)));

    let source = SourceText::new(SourceId::new(208), "let let;".to_owned());
    let binding = source
        .anchor(4, 7)
        .expect("the EE-15-R01 gold subject must select the binding occurrence");
    assert_eq!(binding.fragment(), "let");

    let rule = rule("EE-15-R01");
    assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
    assert_eq!(rule.gold_fixture_ids, &[EE_15_R01_GOLD_ID]);
}

#[test]
fn existing_script_duplicate_gold_remains_exact() {
    assert_eq!(
        gold_source(SCRIPT_DUPLICATE_GOLD_ID),
        Some("let x; let x;")
    );
    assert_eq!(
        gold_subject_range(SCRIPT_DUPLICATE_GOLD_ID),
        Some((11, 12))
    );

    let rule = rule("EE-36-R01");
    assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
    assert_eq!(rule.gold_fixture_ids, &[SCRIPT_DUPLICATE_GOLD_ID]);
}

#[test]
fn canonically_equivalent_unescaped_bindings_remain_distinct_code_point_sequences() {
    let fixture = RULE_NON_TRIGGER_FIXTURES
        .iter()
        .find(|fixture| fixture.id == "JS-RULE-NONTRIGGER-CANONICAL-DISTINCT-001")
        .expect("canonical-distinct rule-local fixture must exist");
    assert_eq!(fixture.rule_ids, &["EE-36-R01"]);

    let source = SourceText::new(SourceId::new(208), fixture.source.to_owned());
    let composed = source.anchor(4, 6).expect("é occupies UTF-8 bytes 4..6");
    let decomposed = source
        .anchor(12, 15)
        .expect("e plus combining acute occupies UTF-8 bytes 12..15");

    assert_eq!(composed.fragment(), "é");
    assert_eq!(decomposed.fragment(), "e\u{301}");
    assert_ne!(
        composed.fragment().chars().collect::<Vec<_>>(),
        decomposed.fragment().chars().collect::<Vec<_>>()
    );
}

#[test]
fn selected_non_strict_identifier_expectations_are_rule_local() {
    let expected_sources = [
        "let await=1;",
        "let yield=1;",
        "let static=1;",
        "let implements=1;",
        "let arguments=1;",
        "let eval=1;",
    ];

    let actual_sources: Vec<_> = RULE_NON_TRIGGER_FIXTURES
        .iter()
        .filter(|fixture| fixture.id != "JS-RULE-NONTRIGGER-CANONICAL-DISTINCT-001")
        .map(|fixture| fixture.source)
        .collect();
    assert_eq!(actual_sources, expected_sources);

    for fixture in RULE_NON_TRIGGER_FIXTURES {
        assert!(
            !fixture.rule_ids.is_empty(),
            "{} must map to at least one independently inventoried rule",
            fixture.id
        );
        for rule_id in fixture.rule_ids {
            assert_eq!(
                rule(rule_id).kind,
                RuleUnitKind::NormativeRule,
                "{} must map only to an active first-envelope rule",
                fixture.id
            );
        }
    }
}

#[test]
fn rule_local_validation_does_not_import_production_semantics_or_qualified_meaning() {
    let source = include_str!("qualification_static_semantics_validation_tests.rs");
    for forbidden in [
        "first_lexical_slice::",
        "qualification::",
        "ExpectedQualification::Qualified",
        "QualificationOutcome::qualified",
    ] {
        assert!(
            !source.contains(forbidden),
            "rule-local validation must remain candidate-independent: found {forbidden}"
        );
    }
}
