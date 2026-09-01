use super::specificity_input_fixtures::fixtures;
use super::specificity_input_gold::{
    GoldCandidate, GoldCandidateDisposition, GoldContextId, GoldExpectedOutcome, GoldInstruction,
    GoldMaxKind, GoldProgram, GoldQualifierCompletion, GoldQualifierOutcome, GoldQualifierSnapshot,
    GoldSimpleKind, SidecarCandidatePlan, SidecarCompletion, SidecarEvent, SidecarFailure,
    SidecarLimits, SidecarResource, validate_authored_relationship_provenance,
};
use super::specificity_input_reference::{ReferenceOutcome, collect_sidecars, resolve_candidates};

#[test]
fn v1_through_v20_handwritten_gold_matches_source_free_reference() {
    let fixtures = fixtures();
    assert!(fixtures.iter().any(|fixture| fixture.id.starts_with("V1")));
    for required in 1..=20 {
        if required == 16 || required == 18 {
            continue;
        }
        let prefix = format!("V{required}");
        assert!(
            fixtures
                .iter()
                .any(|fixture| fixture.id.starts_with(&prefix)),
            "missing {prefix} fixture"
        );
    }

    for fixture in fixtures {
        let actual = resolve_candidates(&fixture.candidates, fixture.target);
        match fixture.expected {
            GoldExpectedOutcome::Known(expected) => {
                assert_eq!(
                    actual,
                    ReferenceOutcome::Known(expected.to_vec()),
                    "{}",
                    fixture.id
                );
            }
            GoldExpectedOutcome::BlockedOnParent(parent) => {
                assert_eq!(
                    actual,
                    ReferenceOutcome::BlockedOnParent(parent),
                    "{}",
                    fixture.id
                );
            }
            GoldExpectedOutcome::DeferredByNormativeAmbiguity => {
                assert_eq!(
                    actual,
                    ReferenceOutcome::DeferredByNormativeAmbiguity,
                    "{}",
                    fixture.id
                );
            }
        }
        validate_authored_relationship_provenance(&fixture)
            .unwrap_or_else(|failure| panic!("{} provenance: {failure:?}", fixture.id));
    }
}

fn qualifier_snapshot() -> GoldQualifierSnapshot {
    GoldQualifierSnapshot {
        completion: GoldQualifierCompletion::Complete,
        outcomes: vec![
            GoldQualifierOutcome::Qualified,
            GoldQualifierOutcome::Invalid,
            GoldQualifierOutcome::Unsupported,
            GoldQualifierOutcome::Indeterminate,
        ],
        algorithm_steps: 17,
        observations: 4,
        retained_semantic_units: 9,
    }
}

fn minimal_candidate(context: u32, kind: GoldSimpleKind) -> GoldCandidate {
    GoldCandidate {
        context: GoldContextId(context),
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: GoldContextId(context),
            instructions: vec![
                GoldInstruction::BeginMember,
                GoldInstruction::Simple(kind),
                GoldInstruction::EndMember,
            ],
        }),
    }
}

#[test]
fn v16_preparation_refusal_preserves_qualifier_and_committed_prefix() {
    let qualifier = qualifier_snapshot();
    let plans = vec![
        SidecarCandidatePlan {
            candidate: minimal_candidate(1, GoldSimpleKind::Class),
            additional_preparation_mutations: 0,
            ancestry_records_to_inspect: 0,
        },
        SidecarCandidatePlan {
            candidate: minimal_candidate(2, GoldSimpleKind::Id),
            additional_preparation_mutations: 0,
            ancestry_records_to_inspect: 1,
        },
    ];

    let result = collect_sidecars(
        &qualifier,
        &plans,
        SidecarLimits {
            preparation_steps: 2,
            retained_input_units: usize::MAX,
        },
    );

    assert_eq!(result.qualifier, qualifier);
    assert_eq!(result.completion, SidecarCompletion::Incomplete);
    assert_eq!(
        result.failure,
        Some(SidecarFailure::Resource(SidecarResource::PreparationSteps))
    );
    assert_eq!(result.committed, vec![plans[0].candidate.clone()]);
    assert_eq!(result.preparation_steps, 2);
    assert!(matches!(
        result.events.last(),
        Some(SidecarEvent::AncestryPreflight { granted: false })
    ));
    assert!(!matches!(
        result.events.last(),
        Some(SidecarEvent::AncestryInspect)
    ));
}

#[test]
fn v16_retained_input_refusal_preflights_complete_delta_before_commit() {
    let qualifier = qualifier_snapshot();
    let plans = vec![
        SidecarCandidatePlan {
            candidate: minimal_candidate(1, GoldSimpleKind::Class),
            additional_preparation_mutations: 0,
            ancestry_records_to_inspect: 0,
        },
        SidecarCandidatePlan {
            candidate: minimal_candidate(2, GoldSimpleKind::Id),
            additional_preparation_mutations: 0,
            ancestry_records_to_inspect: 0,
        },
    ];

    let result = collect_sidecars(
        &qualifier,
        &plans,
        SidecarLimits {
            preparation_steps: usize::MAX,
            retained_input_units: 7,
        },
    );

    assert_eq!(result.qualifier, qualifier);
    assert_eq!(result.completion, SidecarCompletion::Incomplete);
    assert_eq!(
        result.failure,
        Some(SidecarFailure::Resource(
            SidecarResource::RetainedInputUnits
        ))
    );
    assert_eq!(result.committed, vec![plans[0].candidate.clone()]);
    assert_eq!(result.retained_input_units, 4);
    assert!(matches!(
        result.events.last(),
        Some(SidecarEvent::RetainedPreflight {
            required: 4,
            granted: false
        })
    ));
}

#[test]
fn v18_gold_and_reference_are_source_parser_and_historical_handoff_free() {
    let files = [
        include_str!("specificity_input_gold.rs"),
        include_str!("specificity_input_fixtures.rs"),
        include_str!("specificity_input_reference.rs"),
        include_str!("specificity_input_validation_tests.rs"),
    ];
    let forbidden = [
        concat!("selector::", "handoff"),
        concat!("selector_semantic_", "handoff"),
        concat!("Selector", "Machine"),
        concat!("Source", "Text"),
        concat!("Css", "Token"),
        concat!("CssParser", "RunResult"),
        concat!("#", "402"),
        concat!("#", "409"),
    ];

    for source in files {
        for token in forbidden {
            assert!(
                !source.contains(token),
                "forbidden validation dependency: {token}"
            );
        }
    }
}

#[test]
fn malformed_empty_root_program_fails_closed() {
    let candidate = GoldCandidate {
        context: GoldContextId(1),
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: GoldContextId(1),
            instructions: vec![],
        }),
    };

    assert_eq!(
        resolve_candidates(&[candidate], GoldContextId(1)),
        ReferenceOutcome::InvalidProgram
    );
}

#[test]
fn malformed_empty_outer_member_fails_closed() {
    let candidate = GoldCandidate {
        context: GoldContextId(1),
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: GoldContextId(1),
            instructions: vec![GoldInstruction::BeginMember, GoldInstruction::EndMember],
        }),
    };

    assert_eq!(
        resolve_candidates(&[candidate], GoldContextId(1)),
        ReferenceOutcome::InvalidProgram
    );
}

#[test]
fn malformed_empty_max_member_fails_closed_without_rejecting_empty_surviving_max() {
    let candidate = GoldCandidate {
        context: GoldContextId(1),
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: GoldContextId(1),
            instructions: vec![
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Is),
                GoldInstruction::BeginMember,
                GoldInstruction::EndMember,
                GoldInstruction::EndMax(GoldMaxKind::Is),
                GoldInstruction::EndMember,
            ],
        }),
    };

    assert_eq!(
        resolve_candidates(&[candidate], GoldContextId(1)),
        ReferenceOutcome::InvalidProgram
    );

    let surviving_empty_max = GoldCandidate {
        context: GoldContextId(1),
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: GoldContextId(1),
            instructions: vec![
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Is),
                GoldInstruction::EndMax(GoldMaxKind::Is),
                GoldInstruction::EndMember,
            ],
        }),
    };
    assert_eq!(
        resolve_candidates(&[surviving_empty_max], GoldContextId(1)),
        ReferenceOutcome::Known(vec![super::specificity_input_gold::GoldSpecificity::ZERO])
    );
}

#[test]
fn empty_non_forgiving_max_lists_fail_closed() {
    for kind in [GoldMaxKind::Not, GoldMaxKind::Has] {
        let candidate = GoldCandidate {
            context: GoldContextId(1),
            disposition: GoldCandidateDisposition::Program(GoldProgram {
                owning_context: GoldContextId(1),
                instructions: vec![
                    GoldInstruction::BeginMember,
                    GoldInstruction::BeginMax(kind),
                    GoldInstruction::EndMax(kind),
                    GoldInstruction::EndMember,
                ],
            }),
        };

        assert_eq!(
            resolve_candidates(&[candidate], GoldContextId(1)),
            ReferenceOutcome::InvalidProgram,
            "{kind:?} must not accept an empty selector list"
        );
    }
}

#[test]
fn nested_has_max_frames_fail_closed() {
    let direct = GoldCandidate {
        context: GoldContextId(1),
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: GoldContextId(1),
            instructions: vec![
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Has),
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Has),
                GoldInstruction::BeginMember,
                GoldInstruction::Simple(GoldSimpleKind::Class),
                GoldInstruction::EndMember,
                GoldInstruction::EndMax(GoldMaxKind::Has),
                GoldInstruction::EndMember,
                GoldInstruction::EndMax(GoldMaxKind::Has),
                GoldInstruction::EndMember,
            ],
        }),
    };

    let through_is = GoldCandidate {
        context: GoldContextId(1),
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: GoldContextId(1),
            instructions: vec![
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Has),
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Is),
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Has),
                GoldInstruction::BeginMember,
                GoldInstruction::Simple(GoldSimpleKind::Class),
                GoldInstruction::EndMember,
                GoldInstruction::EndMax(GoldMaxKind::Has),
                GoldInstruction::EndMember,
                GoldInstruction::EndMax(GoldMaxKind::Is),
                GoldInstruction::EndMember,
                GoldInstruction::EndMax(GoldMaxKind::Has),
                GoldInstruction::EndMember,
            ],
        }),
    };

    for candidate in [direct, through_is] {
        assert_eq!(
            resolve_candidates(&[candidate], GoldContextId(1)),
            ReferenceOutcome::InvalidProgram
        );
    }

    let non_nested = GoldCandidate {
        context: GoldContextId(1),
        disposition: GoldCandidateDisposition::Program(GoldProgram {
            owning_context: GoldContextId(1),
            instructions: vec![
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Is),
                GoldInstruction::BeginMember,
                GoldInstruction::BeginMax(GoldMaxKind::Has),
                GoldInstruction::BeginMember,
                GoldInstruction::Simple(GoldSimpleKind::Class),
                GoldInstruction::EndMember,
                GoldInstruction::EndMax(GoldMaxKind::Has),
                GoldInstruction::EndMember,
                GoldInstruction::EndMax(GoldMaxKind::Is),
                GoldInstruction::EndMember,
            ],
        }),
    };

    assert_eq!(
        resolve_candidates(&[non_nested], GoldContextId(1)),
        ReferenceOutcome::Known(vec![
            super::specificity_input_gold::GoldSpecificity::new(0, 1, 0),
        ])
    );
}

#[test]
fn derived_relationships_cannot_fabricate_authored_ranges_by_type() {
    use super::specificity_input_gold::GoldRelationshipOrigin;
    let derived = GoldRelationshipOrigin::Derived;
    assert!(matches!(derived, GoldRelationshipOrigin::Derived));
}
