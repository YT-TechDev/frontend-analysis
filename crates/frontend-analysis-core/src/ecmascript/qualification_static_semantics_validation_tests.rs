//! Candidate-independent rule-local validation for Issue #208.
//!
//! This module records expected non-trigger meaning for the first selected
//! static-semantics slice without using a whole-source qualification verdict.
//! A selected rule not triggering is deliberately weaker than aggregate
//! `Qualified` and must remain independent of future production semantics.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::{gold_source, gold_subject_range};

const EE_15_R01_GOLD_ID: &str = "JS-GOLD-LEXDECL-LET-BINDING-001";
const SCRIPT_DUPLICATE_GOLD_ID: &str = "JS-GOLD-SCRIPT-DUPLEXICAL-001";
const INVENTORY_SOURCE: &str = include_str!("qualification_validation_tests/inventory.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuleNonTriggerFixture {
    id: &'static str,
    source: &'static str,
    rule_ids: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RulePrecedenceFixture {
    id: &'static str,
    source: &'static str,
    primary_rule_id: &'static str,
    primary_subject: (usize, usize),
    supporting_subjects: &'static [(usize, usize)],
    co_trigger_rule_ids: &'static [&'static str],
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

const RULE_PRECEDENCE_FIXTURES: &[RulePrecedenceFixture] = &[
    RulePrecedenceFixture {
        id: "JS-RULE-PRECEDENCE-EARLIER-R03-BEFORE-LATER-R01-001",
        source: "const x; let let;",
        primary_rule_id: "EE-15-R03",
        primary_subject: (6, 7),
        supporting_subjects: &[(13, 16)],
        co_trigger_rule_ids: &["EE-15-R01"],
    },
    RulePrecedenceFixture {
        id: "JS-RULE-PRECEDENCE-R01-BEFORE-R02-001",
        source: "let let, let;",
        primary_rule_id: "EE-15-R01",
        primary_subject: (4, 7),
        supporting_subjects: &[(9, 12)],
        co_trigger_rule_ids: &["EE-15-R02", "EE-36-R01"],
    },
    RulePrecedenceFixture {
        id: "JS-RULE-PRECEDENCE-LOCAL-R01-BEFORE-SCRIPT-DUPLICATE-001",
        source: "let x; let x; let let;",
        primary_rule_id: "EE-15-R01",
        primary_subject: (18, 21),
        supporting_subjects: &[(4, 5), (11, 12)],
        co_trigger_rule_ids: &["EE-36-R01"],
    },
];

fn active_rule_block(rule_id: &str) -> &'static str {
    let marker = format!("active_rule(\n        \"{rule_id}\",");
    let start = INVENTORY_SOURCE
        .find(&marker)
        .unwrap_or_else(|| panic!("rule-local expectation maps to missing active rule {rule_id}"));
    let rest = &INVENTORY_SOURCE[start..];
    let end = rest
        .find("\n    ),")
        .unwrap_or_else(|| panic!("active rule {rule_id} must retain a complete inventory block"))
        + "\n    ),".len();
    &rest[..end]
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

    let rule = active_rule_block("EE-15-R01");
    assert!(
        rule.contains("\"JS-GOLD-LEXDECL-LET-BINDING-001\""),
        "EE-15-R01 must preserve its dedicated rejecting gold mapping"
    );
}

#[test]
fn existing_script_duplicate_gold_remains_exact() {
    assert_eq!(gold_source(SCRIPT_DUPLICATE_GOLD_ID), Some("let x; let x;"));
    assert_eq!(gold_subject_range(SCRIPT_DUPLICATE_GOLD_ID), Some((11, 12)));

    let rule = active_rule_block("EE-36-R01");
    assert!(
        rule.contains("\"JS-GOLD-SCRIPT-DUPLEXICAL-001\""),
        "EE-36-R01 must preserve the existing duplicate Script gold mapping"
    );
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
    assert_eq!(actual_sources.as_slice(), expected_sources.as_slice());

    for fixture in RULE_NON_TRIGGER_FIXTURES {
        assert!(
            !fixture.rule_ids.is_empty(),
            "{} must map to at least one independently inventoried rule",
            fixture.id
        );
        for rule_id in fixture.rule_ids {
            let _ = active_rule_block(rule_id);
        }
    }
}

#[test]
fn second_lexical_slice_precedence_witnesses_reject_family_global_traversal() {
    assert_eq!(RULE_PRECEDENCE_FIXTURES.len(), 3);

    for fixture in RULE_PRECEDENCE_FIXTURES {
        let _ = active_rule_block(fixture.primary_rule_id);
        assert!(!fixture.co_trigger_rule_ids.is_empty());
        for rule_id in fixture.co_trigger_rule_ids {
            assert_ne!(*rule_id, fixture.primary_rule_id);
            let _ = active_rule_block(rule_id);
        }

        let source = SourceText::new(SourceId::new(216), fixture.source.to_owned());
        let primary = source
            .anchor(fixture.primary_subject.0, fixture.primary_subject.1)
            .unwrap_or_else(|_| {
                panic!("{} primary subject must be an authored range", fixture.id)
            });
        assert!(!primary.fragment().is_empty());

        for (start, end) in fixture.supporting_subjects {
            let supporting = source.anchor(*start, *end).unwrap_or_else(|_| {
                panic!("{} supporting subject must be authored", fixture.id)
            });
            assert!(!supporting.fragment().is_empty());
        }
    }

    let earlier_r03 = &RULE_PRECEDENCE_FIXTURES[0];
    assert_eq!(earlier_r03.source, "const x; let let;");
    assert_eq!(earlier_r03.primary_rule_id, "EE-15-R03");
    assert_eq!(earlier_r03.primary_subject, (6, 7));
    assert_eq!(earlier_r03.supporting_subjects, &[(13, 16)]);
    assert_eq!(earlier_r03.co_trigger_rule_ids, &["EE-15-R01"]);

    let r01_before_r02 = &RULE_PRECEDENCE_FIXTURES[1];
    assert_eq!(r01_before_r02.source, "let let, let;");
    assert_eq!(r01_before_r02.primary_rule_id, "EE-15-R01");
    assert_eq!(r01_before_r02.primary_subject, (4, 7));
    assert_eq!(r01_before_r02.supporting_subjects, &[(9, 12)]);
    assert_eq!(
        r01_before_r02.co_trigger_rule_ids,
        &["EE-15-R02", "EE-36-R01"]
    );

    let local_before_script = &RULE_PRECEDENCE_FIXTURES[2];
    assert_eq!(local_before_script.source, "let x; let x; let let;");
    assert_eq!(local_before_script.primary_rule_id, "EE-15-R01");
    assert_eq!(local_before_script.primary_subject, (18, 21));
    assert_eq!(local_before_script.supporting_subjects, &[(4, 5), (11, 12)]);
    assert_eq!(local_before_script.co_trigger_rule_ids, &["EE-36-R01"]);
}

#[test]
fn rule_local_validation_does_not_import_production_semantics_or_qualified_meaning() {
    let source = include_str!("qualification_static_semantics_validation_tests.rs");
    for forbidden in [
        concat!("first_", "lexical_slice::"),
        concat!("qualification", "::"),
        concat!("ExpectedQualification", "::Qualified"),
        concat!("QualificationOutcome", "::qualified"),
        concat!("mod ", "inventory;"),
    ] {
        assert!(
            !source.contains(forbidden),
            "rule-local validation must remain candidate-independent: found {forbidden}"
        );
    }
}
