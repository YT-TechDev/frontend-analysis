//! Candidate-independent completion proof for the current selected ECMAScript slice.
//!
//! This test-only matrix consumes the frozen first-envelope validation inventory
//! plus the #267 accepted grammar/context reachability contract. It never calls
//! production lexical, static-semantics, or aggregate qualification behavior to
//! derive expected completion meaning.

use std::collections::BTreeSet;

use super::focused::FOCUSED_RULE_EVIDENCE;
use super::inventory::{
    CONTAINERS, RULE_UNITS, RuleUnit, RuleUnitKind, aggregate_gold_coverage_complete,
    aggregate_rule_expansion_complete,
};

const THIS_SOURCE: &str = include_str!("selected_slice_completion.rs");
const RHS_ESCAPED_RESERVED_ORACLE_SOURCE: &str = include_str!(
    "../qualification_selected_escaped_reserved_identifier_initializer_validation_tests.rs"
);
const QUALIFICATION_SOURCE: &str = include_str!("../qualification.rs");
const SELECTED_AGGREGATE_SOURCE: &str =
    include_str!(concat!("../selected_", "qualification_integration.rs"));

const SELECTED_ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const SELECTED_ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const SELECTED_UNICODE_VERSION: &str = "17.0.0";
const SELECTED_SOURCE_AUTHORITY: &str = "Core UTF-8 SourceText";
const SELECTED_SOURCE_CONTEXT: &str = "Independent Source Unit";
const SELECTED_PARSE_GOAL: &str = "Script";
const SELECTED_TOP_LEVEL_GRAMMAR: &str = "LexicalDeclaration+";
const SELECTED_HAS_TOP_LEVEL_EXPRESSION_STATEMENT: bool = false;
const SELECTED_CAN_FORM_DIRECTIVE_PROLOGUE: bool = false;
const SELECTED_IS_STRICT: bool = false;
const FULL_FIRST_ENVELOPE_CAN_CONTAIN_STRICT_SCRIPT: bool = true;
const SELECTED_YIELD_PARAMETER: bool = false;
const SELECTED_AWAIT_PARAMETER: bool = false;
const SELECTED_INCLUDES_MANDATORY_LEGACY: bool = true;
const SELECTED_INCLUDES_NORMATIVE_OPTIONAL_SOURCE_STATIC: bool = false;

const SELECTED_REACHES_REGEXP_PATTERN: bool = false;
const FULL_FIRST_ENVELOPE_REQUIRES_REGEXP_PATTERN: bool = true;
const SELECTED_REQUIRES_PROFILE_BOUNDARY_PRODUCER: bool = false;
const FULL_FIRST_ENVELOPE_PROFILE_COMPLETION_PROVEN: bool = false;
const SELECTED_GRAMMAR_REJECTION_AUTHORITY: &str = "#227/#229/#231";
const SELECTED_SLICE_COMPLETION_SCOPE: &str = "validation-only local selected slice";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NonTriggerReason {
    OwningSyntaxAbsent(&'static str),
    StrictModeImpossibleInSelectedGrammar,
    YieldParameterFalse,
    AwaitParameterFalse,
    ScriptGoalExcludesModuleRule,
    SelectedLiteralAlternativeCannotTriggerRule(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedSliceDisposition {
    RequiredEvidence { fixture_id: &'static str },
    StructurallyNonTriggering(NonTriggerReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RequiredEvidence {
    rule_id: &'static str,
    container_id: &'static str,
    fixture_id: &'static str,
}

const REQUIRED_EVIDENCE: &[RequiredEvidence] = &[
    RequiredEvidence {
        rule_id: "EE-01-R01",
        container_id: "EE-01",
        fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
    },
    RequiredEvidence {
        rule_id: "EE-01-R02",
        container_id: "EE-01",
        fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-PART-HYPHEN-001",
    },
    RequiredEvidence {
        rule_id: "EE-04-R08",
        container_id: "EE-04",
        fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
    },
    RequiredEvidence {
        rule_id: "EE-15-R01",
        container_id: "EE-15",
        fixture_id: "JS-GOLD-LEXDECL-CONST-LET-MISSING-INIT-001",
    },
    RequiredEvidence {
        rule_id: "EE-15-R02",
        container_id: "EE-15",
        fixture_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
    },
    RequiredEvidence {
        rule_id: "EE-15-R03",
        container_id: "EE-15",
        fixture_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
    },
    RequiredEvidence {
        rule_id: "EE-36-R01",
        container_id: "EE-36",
        fixture_id: "JS-GOLD-SCRIPT-DUPLEXICAL-MULTIBIND-001",
    },
];

const OWNING_SYNTAX_ABSENT_CONTAINERS: &[(&str, &str)] = &[
    ("EE-05", "Object Initializer"),
    ("EE-06", "RegularExpressionLiteral"),
    ("EE-07", "TemplateLiteral"),
    ("EE-08", "Grouping / parenthesized expression"),
    ("EE-09", "Left-Hand-Side Expression family"),
    ("EE-10", "UpdateExpression"),
    ("EE-11", "delete UnaryExpression"),
    ("EE-12", "AssignmentExpression"),
    ("EE-13", "DestructuringAssignmentTarget"),
    ("EE-14", "Block"),
    ("EE-16", "IfStatement"),
    ("EE-17", "DoWhileStatement"),
    ("EE-18", "WhileStatement"),
    ("EE-19", "ForStatement"),
    ("EE-20", "ForInOfStatement"),
    ("EE-21", "ContinueStatement"),
    ("EE-22", "BreakStatement"),
    ("EE-23", "WithStatement"),
    ("EE-24", "SwitchStatement / CaseBlock"),
    ("EE-25", "LabelledStatement"),
    ("EE-26", "TryStatement / Catch"),
    ("EE-27", "FormalParameters"),
    ("EE-28", "FunctionDefinition"),
    ("EE-29", "ArrowFunction"),
    ("EE-30", "MethodDefinition"),
    ("EE-31", "GeneratorFunction"),
    ("EE-32", "AsyncGeneratorFunction"),
    ("EE-33", "ClassDefinition"),
    ("EE-34", "AsyncFunction"),
    ("EE-35", "AsyncArrowFunction"),
];

fn required_evidence(rule_id: &str) -> Option<&'static RequiredEvidence> {
    REQUIRED_EVIDENCE
        .iter()
        .find(|evidence| evidence.rule_id == rule_id)
}

fn absent_owner(container_id: &str) -> Option<&'static str> {
    OWNING_SYNTAX_ABSENT_CONTAINERS
        .iter()
        .find_map(|(id, owner)| (*id == container_id).then_some(*owner))
}

fn classify(rule: &RuleUnit) -> SelectedSliceDisposition {
    if let Some(evidence) = required_evidence(rule.id) {
        assert_eq!(rule.container_id, evidence.container_id);
        return SelectedSliceDisposition::RequiredEvidence {
            fixture_id: evidence.fixture_id,
        };
    }

    match (rule.container_id, rule.id) {
        ("EE-01", unexpected) => panic!("unclassified selected EE-01 rule {unexpected}"),
        ("EE-02", "EE-02-R01") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::SelectedLiteralAlternativeCannotTriggerRule(
                "selected decimal integer excludes the strict legacy numeric alternatives",
            ),
        ),
        ("EE-02", unexpected) => panic!("unclassified selected EE-02 rule {unexpected}"),
        ("EE-03", "EE-03-R01") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::SelectedLiteralAlternativeCannotTriggerRule(
                "selected StringLiteral is escape-free and excludes legacy escape alternatives",
            ),
        ),
        ("EE-03", unexpected) => panic!("unclassified selected EE-03 rule {unexpected}"),
        ("EE-04", "EE-04-R01" | "EE-04-R02" | "EE-04-R07") => {
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::StrictModeImpossibleInSelectedGrammar,
            )
        }
        ("EE-04", "EE-04-R03" | "EE-04-R05") => {
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::YieldParameterFalse,
            )
        }
        ("EE-04", "EE-04-R04" | "EE-04-R06") => {
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::AwaitParameterFalse,
            )
        }
        ("EE-04", "EE-04-I01" | "EE-04-I02") => {
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::ScriptGoalExcludesModuleRule,
            )
        }
        ("EE-04", unexpected) => panic!("unclassified selected EE-04 rule {unexpected}"),
        ("EE-15", unexpected) => panic!("unclassified selected EE-15 rule {unexpected}"),
        ("EE-36", "EE-36-R02") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::OwningSyntaxAbsent("VarDeclaredNames contributor"),
        ),
        ("EE-36", "EE-36-R03") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::OwningSyntaxAbsent("super syntax"),
        ),
        ("EE-36", "EE-36-R04") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::OwningSyntaxAbsent("NewTarget syntax"),
        ),
        ("EE-36", "EE-36-R05") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::OwningSyntaxAbsent("labelled syntax"),
        ),
        ("EE-36", "EE-36-R06") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::OwningSyntaxAbsent("break syntax"),
        ),
        ("EE-36", "EE-36-R07") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::OwningSyntaxAbsent("continue syntax"),
        ),
        ("EE-36", "EE-36-R08") => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::OwningSyntaxAbsent("PrivateIdentifier syntax"),
        ),
        ("EE-36", unexpected) => panic!("unclassified selected EE-36 rule {unexpected}"),
        ("EE-37", _) => SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::OwningSyntaxAbsent(
                "RegularExpressionLiteral bridge and RegExp Pattern",
            ),
        ),
        (container_id, _) => {
            let owner = absent_owner(container_id)
                .unwrap_or_else(|| panic!("unclassified selected container {container_id}"));
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent(owner),
            )
        }
    }
}

#[test]
fn selected_context_is_pinned_independently_from_production_behavior() {
    assert_eq!(SELECTED_ECMA_262_EDITION, "ECMA-262, 17th edition, 2026");
    assert_eq!(
        SELECTED_ECMA_262_SNAPSHOT,
        "d89c03f2db8a597bc915b363a6518d0cc8acdbc0"
    );
    assert_eq!(SELECTED_UNICODE_VERSION, "17.0.0");
    assert_eq!(SELECTED_SOURCE_AUTHORITY, "Core UTF-8 SourceText");
    assert_eq!(SELECTED_SOURCE_CONTEXT, "Independent Source Unit");
    assert_eq!(SELECTED_PARSE_GOAL, "Script");
    assert_eq!(SELECTED_TOP_LEVEL_GRAMMAR, "LexicalDeclaration+");
    assert!(!SELECTED_HAS_TOP_LEVEL_EXPRESSION_STATEMENT);
    assert!(!SELECTED_CAN_FORM_DIRECTIVE_PROLOGUE);
    assert!(!SELECTED_IS_STRICT);
    assert!(FULL_FIRST_ENVELOPE_CAN_CONTAIN_STRICT_SCRIPT);
    assert!(!SELECTED_YIELD_PARAMETER);
    assert!(!SELECTED_AWAIT_PARAMETER);
    assert!(SELECTED_INCLUDES_MANDATORY_LEGACY);
    assert!(!SELECTED_INCLUDES_NORMATIVE_OPTIONAL_SOURCE_STATIC);
}

#[test]
fn all_frozen_rule_units_receive_exactly_one_selected_slice_disposition() {
    let mut seen = BTreeSet::new();
    let mut required = BTreeSet::new();
    let mut active = 0;
    let mut inactive = 0;
    let mut sentinels = 0;

    for rule in RULE_UNITS {
        assert!(
            seen.insert(rule.id),
            "duplicate frozen rule unit {}",
            rule.id
        );

        match rule.kind {
            RuleUnitKind::NormativeRule => active += 1,
            RuleUnitKind::EnvelopeInactiveRule => inactive += 1,
            RuleUnitKind::ExpansionSentinel => sentinels += 1,
        }

        match classify(rule) {
            SelectedSliceDisposition::RequiredEvidence { fixture_id } => {
                assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
                assert!(!fixture_id.is_empty());
                assert!(required.insert(rule.id));
            }
            SelectedSliceDisposition::StructurallyNonTriggering(reason) => {
                assert!(
                    !matches!(reason, NonTriggerReason::OwningSyntaxAbsent("")),
                    "non-trigger reason for {} must remain reviewable",
                    rule.id
                );
            }
        }
    }

    assert_eq!(seen.len(), 193);
    assert_eq!(active, 183);
    assert_eq!(inactive, 10);
    assert_eq!(sentinels, 0);

    let expected_required: BTreeSet<_> = REQUIRED_EVIDENCE
        .iter()
        .map(|evidence| evidence.rule_id)
        .collect();
    assert_eq!(required, expected_required);
    assert_eq!(required.len(), 7);
}

#[test]
fn selected_slice_required_rule_set_is_exact_and_independently_evidence_backed() {
    let expected_rule_ids: BTreeSet<_> = [
        "EE-01-R01",
        "EE-01-R02",
        "EE-04-R08",
        "EE-15-R01",
        "EE-15-R02",
        "EE-15-R03",
        "EE-36-R01",
    ]
    .into_iter()
    .collect();
    let configured_rule_ids: BTreeSet<_> = REQUIRED_EVIDENCE
        .iter()
        .map(|evidence| evidence.rule_id)
        .collect();
    assert_eq!(configured_rule_ids, expected_rule_ids);

    for expected in REQUIRED_EVIDENCE {
        let rule = RULE_UNITS
            .iter()
            .find(|rule| rule.id == expected.rule_id)
            .unwrap_or_else(|| panic!("missing required rule {}", expected.rule_id));
        assert_eq!(rule.kind, RuleUnitKind::NormativeRule);
        assert_eq!(rule.container_id, expected.container_id);
        assert!(
            rule.gold_fixture_ids.contains(&expected.fixture_id),
            "{} must retain independent inventory fixture {}",
            expected.rule_id,
            expected.fixture_id
        );
        assert!(
            FOCUSED_RULE_EVIDENCE.iter().any(|evidence| {
                evidence.primary_rule_id == expected.rule_id
                    && evidence.fixture_id == expected.fixture_id
            }),
            "{} must retain focused candidate-independent evidence {}",
            expected.rule_id,
            expected.fixture_id
        );
    }
}

#[test]
fn escaped_reserved_initializer_oracle_extends_the_same_ee_04_r08_identity() {
    assert_eq!(
        RULE_UNITS
            .iter()
            .filter(|rule| rule.id == "EE-04-R08")
            .count(),
        1
    );
    assert!(
        RHS_ESCAPED_RESERVED_ORACLE_SOURCE
            .contains("Candidate-independent RHS escaped-ReservedWord validation for Issue #263")
    );
    assert!(RHS_ESCAPED_RESERVED_ORACLE_SOURCE.contains("const RULE_ID: &str = \"EE-04-R08\";"));
    assert!(RHS_ESCAPED_RESERVED_ORACLE_SOURCE.contains("StaticSemanticsRejected"));
}

#[test]
fn strict_numeric_and_string_non_triggers_are_not_inferred_from_production_omission() {
    let ee02 = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-02-R01")
        .expect("EE-02-R01 must remain frozen");
    assert!(matches!(
        classify(ee02),
        SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::SelectedLiteralAlternativeCannotTriggerRule(_)
        )
    ));

    let ee03 = RULE_UNITS
        .iter()
        .find(|rule| rule.id == "EE-03-R01")
        .expect("EE-03-R01 must remain frozen");
    assert!(matches!(
        classify(ee03),
        SelectedSliceDisposition::StructurallyNonTriggering(
            NonTriggerReason::SelectedLiteralAlternativeCannotTriggerRule(_)
        )
    ));

    assert!(!SELECTED_IS_STRICT);
    assert!(!SELECTED_CAN_FORM_DIRECTIVE_PROLOGUE);
    assert!(FULL_FIRST_ENVELOPE_CAN_CONTAIN_STRICT_SCRIPT);
}

#[test]
fn ee_04_selected_context_classification_is_explicit() {
    let expected = [
        (
            "EE-04-R01",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::StrictModeImpossibleInSelectedGrammar,
            ),
        ),
        (
            "EE-04-R02",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::StrictModeImpossibleInSelectedGrammar,
            ),
        ),
        (
            "EE-04-R03",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::YieldParameterFalse,
            ),
        ),
        (
            "EE-04-R04",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::AwaitParameterFalse,
            ),
        ),
        (
            "EE-04-R05",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::YieldParameterFalse,
            ),
        ),
        (
            "EE-04-R06",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::AwaitParameterFalse,
            ),
        ),
        (
            "EE-04-R07",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::StrictModeImpossibleInSelectedGrammar,
            ),
        ),
        (
            "EE-04-R08",
            SelectedSliceDisposition::RequiredEvidence {
                fixture_id: "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
            },
        ),
        (
            "EE-04-I01",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::ScriptGoalExcludesModuleRule,
            ),
        ),
        (
            "EE-04-I02",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::ScriptGoalExcludesModuleRule,
            ),
        ),
    ];

    for (rule_id, expected_disposition) in expected {
        let rule = RULE_UNITS
            .iter()
            .find(|rule| rule.id == rule_id)
            .unwrap_or_else(|| panic!("missing frozen rule {rule_id}"));
        assert_eq!(classify(rule), expected_disposition);
    }
}

#[test]
fn ee_36_selected_context_classification_is_explicit() {
    let expected = [
        (
            "EE-36-R01",
            SelectedSliceDisposition::RequiredEvidence {
                fixture_id: "JS-GOLD-SCRIPT-DUPLEXICAL-MULTIBIND-001",
            },
        ),
        (
            "EE-36-R02",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent("VarDeclaredNames contributor"),
            ),
        ),
        (
            "EE-36-R03",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent("super syntax"),
            ),
        ),
        (
            "EE-36-R04",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent("NewTarget syntax"),
            ),
        ),
        (
            "EE-36-R05",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent("labelled syntax"),
            ),
        ),
        (
            "EE-36-R06",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent("break syntax"),
            ),
        ),
        (
            "EE-36-R07",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent("continue syntax"),
            ),
        ),
        (
            "EE-36-R08",
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent("PrivateIdentifier syntax"),
            ),
        ),
    ];

    for (rule_id, expected_disposition) in expected {
        let rule = RULE_UNITS
            .iter()
            .find(|rule| rule.id == rule_id)
            .unwrap_or_else(|| panic!("missing frozen rule {rule_id}"));
        assert_eq!(classify(rule), expected_disposition);
    }
}

#[test]
fn absent_container_table_is_complete_and_reviewable() {
    assert_eq!(OWNING_SYNTAX_ABSENT_CONTAINERS.len(), 30);

    let mut accounted: BTreeSet<_> = OWNING_SYNTAX_ABSENT_CONTAINERS
        .iter()
        .map(|(id, owner)| {
            assert!(!owner.is_empty());
            *id
        })
        .collect();
    for special in [
        "EE-01", "EE-02", "EE-03", "EE-04", "EE-15", "EE-36", "EE-37",
    ] {
        assert!(accounted.insert(special));
    }

    let frozen: BTreeSet<_> = CONTAINERS.iter().map(|container| container.id).collect();
    assert_eq!(accounted, frozen);
}

#[test]
fn regexp_and_profile_non_reachability_are_slice_local_only() {
    assert!(!SELECTED_REACHES_REGEXP_PATTERN);
    assert!(FULL_FIRST_ENVELOPE_REQUIRES_REGEXP_PATTERN);
    assert!(!SELECTED_REQUIRES_PROFILE_BOUNDARY_PRODUCER);
    assert!(!FULL_FIRST_ENVELOPE_PROFILE_COMPLETION_PROVEN);

    for rule in RULE_UNITS
        .iter()
        .filter(|rule| rule.container_id == "EE-37")
    {
        assert!(matches!(
            classify(rule),
            SelectedSliceDisposition::StructurallyNonTriggering(
                NonTriggerReason::OwningSyntaxAbsent(_)
            )
        ));
    }
}

#[test]
fn grammar_rejection_closure_remains_separate_from_early_error_rule_closure() {
    assert_eq!(SELECTED_GRAMMAR_REJECTION_AUTHORITY, "#227/#229/#231");
    assert!(RULE_UNITS.iter().all(|rule| rule.id.starts_with("EE-")));
    assert_eq!(REQUIRED_EVIDENCE.len(), 7);
}

#[test]
fn global_first_envelope_remains_fail_closed_when_selected_slice_is_complete() {
    assert_eq!(
        SELECTED_SLICE_COMPLETION_SCOPE,
        "validation-only local selected slice"
    );
    assert!(aggregate_rule_expansion_complete(RULE_UNITS));
    assert!(!aggregate_gold_coverage_complete(RULE_UNITS));

    assert!(QUALIFICATION_SOURCE.contains("struct CompleteQualificationWitness"));
    assert!(QUALIFICATION_SOURCE.contains("test_complete_qualification_witness"));
    assert!(!QUALIFICATION_SOURCE.contains("fn complete_qualification_witness("));
    assert!(SELECTED_AGGREGATE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(!SELECTED_AGGREGATE_SOURCE.contains("QualificationOutcome::qualified"));
}

#[test]
fn completion_oracle_does_not_import_production_owners() {
    for forbidden in [
        concat!("selected_", "lexical_slice"),
        concat!("selected_", "static_semantics"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("attempt_selected_", "qualification"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "selected-slice completion authority must remain candidate-independent: {forbidden}"
        );
    }
}
