//! Candidate-independent validation for the selected lexical initialization-order
//! and conditional initialization-state semantics accepted by Issue #276.
//!
//! This oracle owns expected authored ranges, binding-evaluation order, and the
//! explicit execution-premise envelope independently from any future production
//! initialization-order or TDZ implementation.

use std::collections::BTreeSet;

use crate::{SourceId, SourceText};

const THIS_SOURCE: &str = include_str!("selected_lexical_initialization_validation_tests.rs");

const CONDITIONAL_STATE_BOUNDARY: &str = "conditional initialization state under P1-P7 != source-only runtime verdict != runtime observation != guaranteed Script reachability != guaranteed GlobalDeclarationInstantiation success";
const NO_MATCH_BOUNDARY: &str = "NoSameSourceSelectedLexicalTarget != runtime unbound != ReferenceError != InitializedAtReference != UninitializedAtReference";
const QUALIFICATION_BOUNDARY: &str =
    "SelectedSliceComplete != FirstEnvelopeComplete != QualificationOutcome::Qualified";
const RAW_OFFSET_BOUNDARY: &str =
    "authored target range before reference range != TargetBindingBeforeContainingBinding";

const EXECUTION_PREMISES: &[(&str, &str)] = &[
    ("P1", "whole current selected lexical recognition completed"),
    (
        "P2",
        "selected static semantics Accepted for the exact retained script",
    ),
    (
        "P3",
        "Issue #267/#268 selected-slice completion authority remains valid",
    ),
    (
        "P4",
        "GlobalDeclarationInstantiation for this exact Script completed normally",
    ),
    (
        "P5",
        "evaluation is the standard ScriptEvaluation context established for that Script",
    ),
    (
        "P6",
        "the selected IdentifierReference evaluation point is reached",
    ),
    (
        "P7",
        "no syntax/context widening introduced an intervening lexical Environment Record",
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedAnchor {
    start: usize,
    end: usize,
    fragment: &'static str,
}

impl ExpectedAnchor {
    const fn new(start: usize, end: usize, fragment: &'static str) -> Self {
        Self {
            start,
            end,
            fragment,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedApplicability {
    ApplicableUnderExplicitExecutionPremises,
    NoSameSourceSelectedLexicalTarget,
    PrerequisiteStaticRejected { authority: &'static str },
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedTarget {
    SameSourceSelectedLexicalBinding(ExpectedAnchor),
    NoSameSourceSelectedLexicalBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedBindingOrder {
    TargetBindingBeforeContainingBinding,
    TargetIsContainingBinding,
    TargetBindingAfterContainingBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedConditionalInitializationState {
    InitializedAtReference,
    UninitializedAtReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExpectedReference {
    containing_binding: ExpectedAnchor,
    reference: ExpectedAnchor,
    semantic_name: &'static str,
    semantic_code_points: &'static [u32],
    target: ExpectedTarget,
    order: Option<ExpectedBindingOrder>,
    conditional_state: Option<ExpectedConditionalInitializationState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct InitializationFixture {
    id: &'static str,
    source: &'static str,
    applicability: ExpectedApplicability,
    reference: Option<ExpectedReference>,
}

const A_CODE_POINTS: &[u32] = &[0x61];
const X_CODE_POINTS: &[u32] = &[0x78];
const Y_CODE_POINTS: &[u32] = &[0x79];
const E_COMBINING_ACUTE_CODE_POINTS: &[u32] = &[0x65, 0x0301];

const FIXTURES: &[InitializationFixture] = &[
    InitializationFixture {
        id: "backward-separate-declarations",
        source: "let a=1; let x=a;",
        applicability: ExpectedApplicability::ApplicableUnderExplicitExecutionPremises,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(13, 14, "x"),
            reference: ExpectedAnchor::new(15, 16, "a"),
            semantic_name: "a",
            semantic_code_points: A_CODE_POINTS,
            target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(
                4, 5, "a",
            )),
            order: Some(ExpectedBindingOrder::TargetBindingBeforeContainingBinding),
            conditional_state: Some(ExpectedConditionalInitializationState::InitializedAtReference),
        }),
    },
    InitializationFixture {
        id: "forward-separate-declarations",
        source: "let x=y; let y=1;",
        applicability: ExpectedApplicability::ApplicableUnderExplicitExecutionPremises,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(4, 5, "x"),
            reference: ExpectedAnchor::new(6, 7, "y"),
            semantic_name: "y",
            semantic_code_points: Y_CODE_POINTS,
            target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(
                13, 14, "y",
            )),
            order: Some(ExpectedBindingOrder::TargetBindingAfterContainingBinding),
            conditional_state: Some(
                ExpectedConditionalInitializationState::UninitializedAtReference,
            ),
        }),
    },
    InitializationFixture {
        id: "self",
        source: "let x=x;",
        applicability: ExpectedApplicability::ApplicableUnderExplicitExecutionPremises,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(4, 5, "x"),
            reference: ExpectedAnchor::new(6, 7, "x"),
            semantic_name: "x",
            semantic_code_points: X_CODE_POINTS,
            target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(
                4, 5, "x",
            )),
            order: Some(ExpectedBindingOrder::TargetIsContainingBinding),
            conditional_state: Some(
                ExpectedConditionalInitializationState::UninitializedAtReference,
            ),
        }),
    },
    InitializationFixture {
        id: "backward-same-declaration",
        source: "let a=1, x=a;",
        applicability: ExpectedApplicability::ApplicableUnderExplicitExecutionPremises,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(9, 10, "x"),
            reference: ExpectedAnchor::new(11, 12, "a"),
            semantic_name: "a",
            semantic_code_points: A_CODE_POINTS,
            target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(
                4, 5, "a",
            )),
            order: Some(ExpectedBindingOrder::TargetBindingBeforeContainingBinding),
            conditional_state: Some(ExpectedConditionalInitializationState::InitializedAtReference),
        }),
    },
    InitializationFixture {
        id: "forward-same-declaration",
        source: "let x=y, y=1;",
        applicability: ExpectedApplicability::ApplicableUnderExplicitExecutionPremises,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(4, 5, "x"),
            reference: ExpectedAnchor::new(6, 7, "y"),
            semantic_name: "y",
            semantic_code_points: Y_CODE_POINTS,
            target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(
                9, 10, "y",
            )),
            order: Some(ExpectedBindingOrder::TargetBindingAfterContainingBinding),
            conditional_state: Some(
                ExpectedConditionalInitializationState::UninitializedAtReference,
            ),
        }),
    },
    InitializationFixture {
        id: "self-same-declaration",
        source: "let x=x, y=1;",
        applicability: ExpectedApplicability::ApplicableUnderExplicitExecutionPremises,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(4, 5, "x"),
            reference: ExpectedAnchor::new(6, 7, "x"),
            semantic_name: "x",
            semantic_code_points: X_CODE_POINTS,
            target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(
                4, 5, "x",
            )),
            order: Some(ExpectedBindingOrder::TargetIsContainingBinding),
            conditional_state: Some(
                ExpectedConditionalInitializationState::UninitializedAtReference,
            ),
        }),
    },
    InitializationFixture {
        id: "prior-let-without-initializer",
        source: "let a; let x=a;",
        applicability: ExpectedApplicability::ApplicableUnderExplicitExecutionPremises,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(11, 12, "x"),
            reference: ExpectedAnchor::new(13, 14, "a"),
            semantic_name: "a",
            semantic_code_points: A_CODE_POINTS,
            target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(
                4, 5, "a",
            )),
            order: Some(ExpectedBindingOrder::TargetBindingBeforeContainingBinding),
            conditional_state: Some(ExpectedConditionalInitializationState::InitializedAtReference),
        }),
    },
    InitializationFixture {
        id: "escaped-direct-equality",
        source: "let a=1; let x=\\u0061;",
        applicability: ExpectedApplicability::ApplicableUnderExplicitExecutionPremises,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(13, 14, "x"),
            reference: ExpectedAnchor::new(15, 21, "\\u0061"),
            semantic_name: "a",
            semantic_code_points: A_CODE_POINTS,
            target: ExpectedTarget::SameSourceSelectedLexicalBinding(ExpectedAnchor::new(
                4, 5, "a",
            )),
            order: Some(ExpectedBindingOrder::TargetBindingBeforeContainingBinding),
            conditional_state: Some(ExpectedConditionalInitializationState::InitializedAtReference),
        }),
    },
    InitializationFixture {
        id: "canonical-distinct",
        source: "let é=1; let x=e\\u0301;",
        applicability: ExpectedApplicability::NoSameSourceSelectedLexicalTarget,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(14, 15, "x"),
            reference: ExpectedAnchor::new(16, 23, "e\\u0301"),
            semantic_name: "e\u{301}",
            semantic_code_points: E_COMBINING_ACUTE_CODE_POINTS,
            target: ExpectedTarget::NoSameSourceSelectedLexicalBinding,
            order: None,
            conditional_state: None,
        }),
    },
    InitializationFixture {
        id: "no-same-source-target",
        source: "let x=y;",
        applicability: ExpectedApplicability::NoSameSourceSelectedLexicalTarget,
        reference: Some(ExpectedReference {
            containing_binding: ExpectedAnchor::new(4, 5, "x"),
            reference: ExpectedAnchor::new(6, 7, "y"),
            semantic_name: "y",
            semantic_code_points: Y_CODE_POINTS,
            target: ExpectedTarget::NoSameSourceSelectedLexicalBinding,
            order: None,
            conditional_state: None,
        }),
    },
    InitializationFixture {
        id: "duplicate-static-prerequisite",
        source: "let a=1; let a=2; let x=a;",
        applicability: ExpectedApplicability::PrerequisiteStaticRejected {
            authority: "EE-36-R01",
        },
        reference: None,
    },
    InitializationFixture {
        id: "escaped-reserved-static-prerequisite",
        source: "let x=\\u0069f;",
        applicability: ExpectedApplicability::PrerequisiteStaticRejected {
            authority: "EE-04-R08",
        },
        reference: None,
    },
    InitializationFixture {
        id: "unsupported-parenthesized-initializer",
        source: "let x=(y);",
        applicability: ExpectedApplicability::UnsupportedCoverage,
        reference: None,
    },
];

fn fixture(id: &str) -> &'static InitializationFixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing initialization fixture {id}"))
}

fn expected_reference(id: &str) -> ExpectedReference {
    fixture(id)
        .reference
        .unwrap_or_else(|| panic!("fixture {id} has no expected reference"))
}

fn decoded_expected_name(code_points: &[u32]) -> String {
    let mut decoded = String::new();
    for code_point in code_points {
        let scalar = char::from_u32(*code_point)
            .unwrap_or_else(|| panic!("fixture contains non-scalar code point U+{code_point:04X}"));
        decoded.push(scalar);
    }
    decoded
}

fn validate_anchor(source: &SourceText, expected: ExpectedAnchor) {
    let anchor = source
        .anchor(expected.start, expected.end)
        .unwrap_or_else(|error| panic!("invalid fixture anchor {expected:?}: {error}"));
    assert_eq!(anchor.fragment(), expected.fragment);
}

#[test]
fn execution_premises_are_explicit_complete_and_unique() {
    let mut ids = BTreeSet::new();
    let mut descriptions = BTreeSet::new();

    for (id, description) in EXECUTION_PREMISES {
        assert!(ids.insert(*id), "duplicate execution premise id {id}");
        assert!(
            descriptions.insert(*description),
            "duplicate execution premise description {description}"
        );
    }

    assert_eq!(ids.len(), 7);
    assert_eq!(descriptions.len(), 7);
    assert!(
        EXECUTION_PREMISES[3]
            .1
            .contains("GlobalDeclarationInstantiation")
    );
    assert!(EXECUTION_PREMISES[4].1.contains("ScriptEvaluation"));
    assert!(
        EXECUTION_PREMISES[5]
            .1
            .contains("evaluation point is reached")
    );
    assert!(
        EXECUTION_PREMISES[6]
            .1
            .contains("intervening lexical Environment Record")
    );
}

#[test]
fn fixture_ids_and_sources_are_stable_and_unique() {
    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();

    for fixture in FIXTURES {
        assert!(
            ids.insert(fixture.id),
            "duplicate fixture id {}",
            fixture.id
        );
        assert!(
            sources.insert(fixture.source),
            "duplicate fixture source {}",
            fixture.source
        );
    }

    assert_eq!(ids.len(), 13);
    assert_eq!(sources.len(), 13);
}

#[test]
fn all_expected_authored_ranges_are_valid_utf8_source_anchors() {
    for (index, fixture) in FIXTURES.iter().enumerate() {
        let source = SourceText::new(SourceId::new(index as u64 + 1), fixture.source.to_owned());

        if let Some(reference) = fixture.reference {
            validate_anchor(&source, reference.containing_binding);
            validate_anchor(&source, reference.reference);
            assert_eq!(
                decoded_expected_name(reference.semantic_code_points),
                reference.semantic_name
            );

            if let ExpectedTarget::SameSourceSelectedLexicalBinding(target) = reference.target {
                validate_anchor(&source, target);
            }
        }
    }

    let canonical = fixture("canonical-distinct");
    let source = SourceText::new(SourceId::new(100), canonical.source.to_owned());
    validate_anchor(&source, ExpectedAnchor::new(4, 6, "é"));
}

#[test]
fn backward_forward_and_self_use_binding_evaluation_order_not_raw_offsets() {
    let backward = expected_reference("backward-separate-declarations");
    assert_eq!(
        backward.order,
        Some(ExpectedBindingOrder::TargetBindingBeforeContainingBinding)
    );
    assert_eq!(
        backward.conditional_state,
        Some(ExpectedConditionalInitializationState::InitializedAtReference)
    );

    let forward = expected_reference("forward-separate-declarations");
    assert_eq!(
        forward.order,
        Some(ExpectedBindingOrder::TargetBindingAfterContainingBinding)
    );
    assert_eq!(
        forward.conditional_state,
        Some(ExpectedConditionalInitializationState::UninitializedAtReference)
    );

    let self_reference = expected_reference("self");
    let ExpectedTarget::SameSourceSelectedLexicalBinding(self_target) = self_reference.target
    else {
        panic!("self fixture must have a same-source target");
    };
    assert!(self_target.start < self_reference.reference.start);
    assert_eq!(
        self_reference.order,
        Some(ExpectedBindingOrder::TargetIsContainingBinding)
    );
    assert_eq!(
        self_reference.conditional_state,
        Some(ExpectedConditionalInitializationState::UninitializedAtReference)
    );
    assert!(RAW_OFFSET_BOUNDARY.contains("!= TargetBindingBeforeContainingBinding"));
}

#[test]
fn binding_list_order_and_prior_uninitialized_let_are_pinned() {
    let backward = expected_reference("backward-same-declaration");
    assert_eq!(
        backward.order,
        Some(ExpectedBindingOrder::TargetBindingBeforeContainingBinding)
    );
    assert_eq!(
        backward.conditional_state,
        Some(ExpectedConditionalInitializationState::InitializedAtReference)
    );

    let forward = expected_reference("forward-same-declaration");
    assert_eq!(
        forward.order,
        Some(ExpectedBindingOrder::TargetBindingAfterContainingBinding)
    );
    assert_eq!(
        forward.conditional_state,
        Some(ExpectedConditionalInitializationState::UninitializedAtReference)
    );

    let self_reference = expected_reference("self-same-declaration");
    assert_eq!(
        self_reference.order,
        Some(ExpectedBindingOrder::TargetIsContainingBinding)
    );

    let prior_let = expected_reference("prior-let-without-initializer");
    assert_eq!(
        prior_let.containing_binding,
        ExpectedAnchor::new(11, 12, "x")
    );
    assert_eq!(prior_let.reference, ExpectedAnchor::new(13, 14, "a"));
    assert_eq!(
        prior_let.conditional_state,
        Some(ExpectedConditionalInitializationState::InitializedAtReference)
    );
}

#[test]
fn escaped_identity_and_no_normalization_are_independently_pinned() {
    let escaped = expected_reference("escaped-direct-equality");
    assert_eq!(escaped.reference, ExpectedAnchor::new(15, 21, "\\u0061"));
    assert_eq!(escaped.semantic_name, "a");
    assert_eq!(escaped.semantic_code_points, &[0x61]);
    assert_eq!(
        escaped.order,
        Some(ExpectedBindingOrder::TargetBindingBeforeContainingBinding)
    );

    let canonical = expected_reference("canonical-distinct");
    assert_eq!(
        canonical.containing_binding,
        ExpectedAnchor::new(14, 15, "x")
    );
    assert_eq!(canonical.reference, ExpectedAnchor::new(16, 23, "e\\u0301"));
    assert_eq!(canonical.semantic_name, "e\u{301}");
    assert!(matches!(
        canonical.target,
        ExpectedTarget::NoSameSourceSelectedLexicalBinding
    ));
    assert!(canonical.order.is_none());
    assert!(canonical.conditional_state.is_none());
}

#[test]
fn applicability_controls_whether_execution_state_is_available() {
    for fixture in FIXTURES {
        match fixture.applicability {
            ExpectedApplicability::ApplicableUnderExplicitExecutionPremises => {
                let reference = fixture
                    .reference
                    .unwrap_or_else(|| panic!("{} must have a reference", fixture.id));
                assert!(matches!(
                    reference.target,
                    ExpectedTarget::SameSourceSelectedLexicalBinding(_)
                ));
                assert!(reference.order.is_some());
                assert!(reference.conditional_state.is_some());
            }
            ExpectedApplicability::NoSameSourceSelectedLexicalTarget => {
                let reference = fixture
                    .reference
                    .unwrap_or_else(|| panic!("{} must have a reference", fixture.id));
                assert!(matches!(
                    reference.target,
                    ExpectedTarget::NoSameSourceSelectedLexicalBinding
                ));
                assert!(reference.order.is_none());
                assert!(reference.conditional_state.is_none());
            }
            ExpectedApplicability::PrerequisiteStaticRejected { authority } => {
                assert!(matches!(authority, "EE-36-R01" | "EE-04-R08"));
                assert!(fixture.reference.is_none());
            }
            ExpectedApplicability::UnsupportedCoverage => {
                assert!(fixture.reference.is_none());
            }
        }
    }
}

#[test]
fn no_same_source_target_remains_runtime_undetermined() {
    let no_match = fixture("no-same-source-target");
    assert_eq!(
        no_match.applicability,
        ExpectedApplicability::NoSameSourceSelectedLexicalTarget
    );
    let reference = expected_reference("no-same-source-target");
    assert!(matches!(
        reference.target,
        ExpectedTarget::NoSameSourceSelectedLexicalBinding
    ));
    assert!(reference.order.is_none());
    assert!(reference.conditional_state.is_none());
    assert!(NO_MATCH_BOUNDARY.contains("runtime unbound"));
    assert!(NO_MATCH_BOUNDARY.contains("ReferenceError"));
}

#[test]
fn static_rejection_and_unsupported_grammar_do_not_gain_execution_state() {
    assert_eq!(
        fixture("duplicate-static-prerequisite").applicability,
        ExpectedApplicability::PrerequisiteStaticRejected {
            authority: "EE-36-R01"
        }
    );
    assert_eq!(
        fixture("escaped-reserved-static-prerequisite").applicability,
        ExpectedApplicability::PrerequisiteStaticRejected {
            authority: "EE-04-R08"
        }
    );
    assert_eq!(
        fixture("unsupported-parenthesized-initializer").applicability,
        ExpectedApplicability::UnsupportedCoverage
    );
}

#[test]
fn conditional_state_and_global_qualification_boundaries_remain_explicit() {
    assert!(CONDITIONAL_STATE_BOUNDARY.contains("under P1-P7"));
    assert!(CONDITIONAL_STATE_BOUNDARY.contains("source-only runtime verdict"));
    assert!(CONDITIONAL_STATE_BOUNDARY.contains("GlobalDeclarationInstantiation"));
    assert!(QUALIFICATION_BOUNDARY.contains("FirstEnvelopeComplete"));
    assert!(QUALIFICATION_BOUNDARY.contains("QualificationOutcome::Qualified"));

    let forbidden_unconditional_outcome = concat!("ScriptDefinitelyThrows", "ReferenceError");
    assert!(!THIS_SOURCE.contains(forbidden_unconditional_outcome));
}

#[test]
fn validation_oracle_has_no_production_semantic_dependency() {
    let forbidden_fragments = [
        concat!("use super::selected_", "binding_scope"),
        concat!("use super::selected_lexical_", "initialization"),
        concat!("analyze_selected_", "binding_scope"),
        concat!("evaluate_selected_", "static_semantics"),
        concat!("recognize_selected_", "lexical_slice"),
    ];

    for forbidden in forbidden_fragments {
        assert!(
            !THIS_SOURCE.contains(forbidden),
            "validation oracle must not depend on production semantic candidate: {forbidden}"
        );
    }

    assert!(THIS_SOURCE.contains("use crate::{SourceId, SourceText};"));
}
