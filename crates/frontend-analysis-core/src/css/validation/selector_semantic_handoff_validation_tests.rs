use super::selector_semantic_handoff_gold::{
    AuthoredFactExpectation, AuthoredProgramProvenanceFailure, AuthoredRange, CompletionState,
    ContextId, FunctionKind, GoldFixture, GoldObservation, GoldOutcome, GoldProgram, GoldRun,
    IndeterminateReason, InvalidReason, LiteralRangeExpectation, LiteralRangeFailure, MemberId,
    NestingPresenceDisposition, RejectedNestingEffect, RejectedNestingPresenceExpectation,
    RejectedNestingPresenceFailure, RelationshipOrigin, RelationshipTarget, RunId, SelectorFact,
    SimpleKind, SourceId, UnitId, UnsupportedFeature, authored, derived,
    validate_program_authored_provenance, validate_rejected_nesting_presence, verify_literal_range,
};
use super::selector_semantic_handoff_reference::{
    BlockingOutcome, ConsumerBudget, ConsumerOutcome, ConsumerRunCompletion,
    DependencyResolutionError, DependencyStatus, RetentionBudget, Specificity, commit_observation,
    fold_observation, fold_program, resolve_retained_run,
};

fn range(start: usize, end: usize) -> AuthoredRange {
    AuthoredRange::new(start, end)
}

fn authored_fact(
    fact_index: usize,
    start: usize,
    end: usize,
    spelling: &'static str,
) -> AuthoredFactExpectation {
    AuthoredFactExpectation {
        fact_index,
        range: range(start, end),
        spelling,
    }
}

fn program(context: u32, facts: Vec<SelectorFact>) -> GoldProgram {
    GoldProgram {
        source: SourceId(1),
        run: RunId(1),
        profile: "CoreV1",
        context: ContextId(context),
        facts,
    }
}

fn qualified(program: GoldProgram) -> GoldObservation {
    GoldObservation {
        source: program.source,
        run: program.run,
        profile: program.profile,
        context: program.context,
        completion: CompletionState::Complete,
        outcome: GoldOutcome::Qualified,
        program: Some(program),
    }
}

fn nonqualified(
    context: u32,
    completion: CompletionState,
    outcome: GoldOutcome,
) -> GoldObservation {
    GoldObservation {
        source: SourceId(1),
        run: RunId(1),
        profile: "CoreV1",
        context: ContextId(context),
        completion,
        outcome,
        program: None,
    }
}

fn atom(unit: u32, kind: SimpleKind, start: usize, end: usize) -> SelectorFact {
    SelectorFact::Simple {
        unit: UnitId(unit),
        kind,
        range: range(start, end),
    }
}

fn open(member: u32, start: usize, end: usize) -> SelectorFact {
    SelectorFact::OpenMember {
        member: MemberId(member),
        range: range(start, end),
    }
}

fn close(member: u32) -> SelectorFact {
    SelectorFact::CloseMember {
        member: MemberId(member),
    }
}

fn specificity(a: u32, b: u32, c: u32) -> Specificity {
    Specificity { a, b, c }
}

fn complete_members(result: ConsumerOutcome) -> Vec<(MemberId, Specificity)> {
    match result {
        ConsumerOutcome::Complete(members) => members,
        other => panic!("expected complete specificity result, got {other:?}"),
    }
}

fn run(observations: Vec<GoldObservation>) -> GoldRun {
    GoldRun {
        source: SourceId(1),
        run: RunId(1),
        profile: "CoreV1",
        upstream: CompletionState::Complete,
        qualifier: CompletionState::Complete,
        observations,
    }
}

fn retained_class_and_id_parent(context: u32) -> GoldObservation {
    qualified(program(
        context,
        vec![
            open(1, 0, 2),
            atom(1, SimpleKind::Class, 0, 2),
            close(1),
            open(2, 4, 6),
            atom(2, SimpleKind::Id, 4, 6),
            close(2),
        ],
    ))
}

fn parent_relationship_child(context: u32, parent: u32) -> GoldObservation {
    qualified(program(
        context,
        vec![
            open(1, 0, 1),
            SelectorFact::NestingPresence {
                member: MemberId(1),
                unit: UnitId(1),
                origin: authored(range(0, 1)),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(parent)),
                origin: authored(range(0, 1)),
            },
            close(1),
        ],
    ))
}

fn minimal_independent_observation(context: u32) -> GoldObservation {
    qualified(program(context, vec![open(1, 0, 1), close(1)]))
}

fn rejected_forgiving_ampersand_fixture() -> GoldFixture {
    GoldFixture {
        id: "CSS-HANDOFF-REJECTED-AMPERSAND-001",
        source: ".a:is(:bad&, .b)",
        program: program(
            3,
            vec![
                open(1, 0, 16),
                atom(1, SimpleKind::Class, 0, 2),
                SelectorFact::OpenFunction {
                    unit: UnitId(2),
                    kind: FunctionKind::Is,
                    range: range(2, 6),
                },
                SelectorFact::RejectedForgivingMember {
                    member: MemberId(2),
                    range: range(6, 11),
                },
                SelectorFact::NestingPresence {
                    member: MemberId(2),
                    unit: UnitId(3),
                    origin: authored(range(10, 11)),
                    disposition: NestingPresenceDisposition::NonContributingPresenceOnly,
                },
                open(3, 13, 15),
                atom(4, SimpleKind::Class, 13, 15),
                close(3),
                SelectorFact::CloseFunction { unit: UnitId(2) },
                close(1),
            ],
        ),
        authored: vec![
            authored_fact(0, 0, 16, ".a:is(:bad&, .b)"),
            authored_fact(1, 0, 2, ".a"),
            authored_fact(2, 2, 6, ":is("),
            authored_fact(3, 6, 11, ":bad&"),
            authored_fact(4, 10, 11, "&"),
            authored_fact(5, 13, 15, ".b"),
            authored_fact(6, 13, 15, ".b"),
        ],
    }
}

fn rejected_ampersand_expectation() -> RejectedNestingPresenceExpectation {
    RejectedNestingPresenceExpectation {
        member: MemberId(2),
        rejected_range: range(6, 11),
        unit: UnitId(3),
        presence_range: range(10, 11),
    }
}

#[test]
fn semantic_units_keep_source_run_context_member_identity_and_order() {
    let gold = program(
        7,
        vec![
            open(1, 0, 1),
            atom(1, SimpleKind::Type, 0, 1),
            close(1),
            open(2, 3, 5),
            atom(2, SimpleKind::Id, 3, 5),
            close(2),
        ],
    );

    assert_eq!(gold.source, SourceId(1));
    assert_eq!(gold.run, RunId(1));
    assert_eq!(gold.profile, "CoreV1");
    assert_eq!(gold.context, ContextId(7));
    let result = fold_program(&gold, ConsumerBudget { limit: usize::MAX });
    assert_eq!(
        complete_members(result.outcome),
        vec![
            (MemberId(1), specificity(0, 0, 1)),
            (MemberId(2), specificity(1, 0, 0)),
        ]
    );
}

#[test]
fn authored_ranges_are_handwritten_and_literal_checked() {
    let fixture = GoldFixture {
        id: "CSS-HANDOFF-RANGE-IS-001",
        source: ".a:is(#b, c)",
        program: program(
            1,
            vec![
                open(1, 0, 12),
                atom(1, SimpleKind::Class, 0, 2),
                SelectorFact::OpenFunction {
                    unit: UnitId(2),
                    kind: FunctionKind::Is,
                    range: range(2, 6),
                },
                open(2, 6, 8),
                atom(3, SimpleKind::Id, 6, 8),
                close(2),
                open(3, 10, 11),
                atom(4, SimpleKind::Type, 10, 11),
                close(3),
                SelectorFact::CloseFunction { unit: UnitId(2) },
                close(1),
            ],
        ),
        authored: vec![
            authored_fact(0, 0, 12, ".a:is(#b, c)"),
            authored_fact(1, 0, 2, ".a"),
            authored_fact(2, 2, 6, ":is("),
            authored_fact(3, 6, 8, "#b"),
            authored_fact(4, 6, 8, "#b"),
            authored_fact(6, 10, 11, "c"),
            authored_fact(7, 10, 11, "c"),
        ],
    };

    assert_eq!(fixture.id, "CSS-HANDOFF-RANGE-IS-001");
    assert_eq!(fixture.program.source, SourceId(1));
    assert_eq!(fixture.program.context, ContextId(1));
    assert_eq!(
        validate_program_authored_provenance(&fixture.program, fixture.source, &fixture.authored),
        Ok(())
    );

    let mut corrupted = fixture.program.clone();
    let SelectorFact::OpenFunction {
        range: actual_range,
        ..
    } = &mut corrupted.facts[2]
    else {
        panic!("fact 2 is the authored function opener");
    };
    *actual_range = range(3, 6);
    assert_eq!(
        validate_program_authored_provenance(&corrupted, fixture.source, &fixture.authored),
        Err(AuthoredProgramProvenanceFailure::RangeMismatch {
            fact_index: 2,
            expected: range(2, 6),
            actual: range(3, 6),
        })
    );
}

#[test]
fn authored_ranges_use_utf8_byte_offsets_after_multibyte_scalars() {
    let fixtures = [
        GoldFixture {
            id: "CSS-HANDOFF-RANGE-UTF8-ID-001",
            source: "é#x",
            program: program(
                20,
                vec![
                    open(1, 0, 4),
                    atom(1, SimpleKind::Type, 0, 2),
                    atom(2, SimpleKind::Id, 2, 4),
                    close(1),
                ],
            ),
            authored: vec![
                authored_fact(0, 0, 4, "é#x"),
                authored_fact(1, 0, 2, "é"),
                authored_fact(2, 2, 4, "#x"),
            ],
        },
        GoldFixture {
            id: "CSS-HANDOFF-RANGE-UTF8-CLASS-002",
            source: "éあ.x",
            program: program(
                21,
                vec![
                    open(1, 0, 7),
                    atom(1, SimpleKind::Type, 0, 5),
                    atom(2, SimpleKind::Class, 5, 7),
                    close(1),
                ],
            ),
            authored: vec![
                authored_fact(0, 0, 7, "éあ.x"),
                authored_fact(1, 0, 5, "éあ"),
                authored_fact(2, 5, 7, ".x"),
            ],
        },
    ];

    for fixture in fixtures {
        assert_eq!(
            validate_program_authored_provenance(
                &fixture.program,
                fixture.source,
                &fixture.authored,
            ),
            Ok(())
        );
        assert!(matches!(
            fold_program(&fixture.program, ConsumerBudget { limit: usize::MAX }).outcome,
            ConsumerOutcome::Complete(_)
        ));
    }

    assert_eq!(
        verify_literal_range(
            "é#x",
            LiteralRangeExpectation {
                range: range(1, 3),
                spelling: "#x",
            },
        ),
        Err(LiteralRangeFailure::InvalidStartBoundary)
    );
}

#[test]
fn authored_and_derived_relationship_identity_are_disjoint() {
    let authored_origin = authored(range(10, 11));
    let derived_origin = derived();
    assert!(matches!(authored_origin, RelationshipOrigin::Authored(_)));
    assert_eq!(derived_origin, RelationshipOrigin::Derived);
}

#[test]
fn reference_fold_is_source_token_and_parser_free_by_signature() {
    let fold: fn(
        &GoldProgram,
        ConsumerBudget,
    ) -> super::selector_semantic_handoff_reference::ConsumerResult = fold_program;
    let resolve: fn(
        &GoldRun,
        ConsumerBudget,
    ) -> Result<
        super::selector_semantic_handoff_reference::ResolvedRun,
        DependencyResolutionError,
    > = resolve_retained_run;
    let gold = program(
        1,
        vec![open(1, 0, 1), atom(1, SimpleKind::Type, 0, 1), close(1)],
    );
    let result = fold(&gold, ConsumerBudget { limit: 3 });
    assert_eq!(result.steps, 3);
    assert_eq!(
        complete_members(result.outcome),
        vec![(MemberId(1), specificity(0, 0, 1))]
    );
    let resolved = resolve(&run(vec![qualified(gold)]), ConsumerBudget { limit: 9 })
        .expect("source-free retained resolution completes within its run budget");
    assert_eq!(resolved.completion(), ConsumerRunCompletion::Complete);
    assert_eq!(resolved.used(), 9);
}

#[test]
fn invalid_unsupported_and_indeterminate_remain_distinct() {
    for (outcome, expected) in [
        (
            GoldOutcome::Invalid(InvalidReason::SelectedGrammar),
            BlockingOutcome::Invalid,
        ),
        (
            GoldOutcome::Unsupported(UnsupportedFeature::PseudoElement),
            BlockingOutcome::Unsupported,
        ),
        (
            GoldOutcome::Indeterminate(IndeterminateReason::MissingNamespaceEnvironment),
            BlockingOutcome::Indeterminate,
        ),
    ] {
        let observation = nonqualified(1, CompletionState::Complete, outcome);
        assert_eq!(
            fold_observation(&observation, ConsumerBudget { limit: 1 }).outcome,
            ConsumerOutcome::Blocked(expected)
        );
    }
}

#[test]
fn incomplete_upstream_or_qualifier_cannot_upgrade_to_complete() {
    let observation = qualified(program(
        1,
        vec![open(1, 0, 1), atom(1, SimpleKind::Type, 0, 1), close(1)],
    ));
    let mut upstream_incomplete = run(vec![observation.clone()]);
    upstream_incomplete.upstream = CompletionState::Incomplete;
    let mut qualifier_incomplete = run(vec![observation.clone()]);
    qualifier_incomplete.qualifier = CompletionState::Incomplete;
    for run in [upstream_incomplete, qualifier_incomplete] {
        let resolved = resolve_retained_run(&run, ConsumerBudget { limit: usize::MAX })
            .expect("authoritative incompleteness is a consumer outcome, not a graph error");
        assert_eq!(resolved.completion(), ConsumerRunCompletion::Incomplete);
        assert_eq!(resolved.used(), 0);
        assert!(resolved.result(ContextId(1)).is_none());
        assert_eq!(resolved.dependency(ContextId(1)), None);
    }
}

#[test]
fn semantic_program_commit_is_atomic() {
    let observation = qualified(program(
        2,
        vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
    ));
    let mut committed = Vec::new();
    let mut budget = RetentionBudget { limit: 2, used: 0 };
    let refusal = commit_observation(&mut committed, observation, &mut budget)
        .expect_err("one observation plus three facts must not partially fit a budget of two");
    assert!(committed.is_empty());
    assert_eq!(budget.used, 0);
    assert_eq!(refusal.required, 4);
    assert_eq!(refusal.remaining, 2);
}

#[test]
fn retention_refusal_preserves_previously_committed_prefix() {
    let first = qualified(program(1, vec![open(1, 0, 1), close(1)]));
    let second = qualified(program(
        2,
        vec![open(2, 0, 2), atom(2, SimpleKind::Class, 0, 2), close(2)],
    ));
    let mut committed = Vec::new();
    let mut budget = RetentionBudget { limit: 4, used: 0 };
    commit_observation(&mut committed, first.clone(), &mut budget).expect("first fits");
    assert!(commit_observation(&mut committed, second, &mut budget).is_err());
    assert_eq!(committed, vec![first]);
    assert_eq!(budget.used, 3);
}

#[test]
fn retained_observation_identity_is_never_zero_cost() {
    for outcome in [
        GoldOutcome::Invalid(InvalidReason::SelectedGrammar),
        GoldOutcome::Unsupported(UnsupportedFeature::PseudoElement),
        GoldOutcome::Indeterminate(IndeterminateReason::MissingNamespaceEnvironment),
    ] {
        let observation = nonqualified(1, CompletionState::Complete, outcome);
        let mut committed = Vec::new();
        let mut zero = RetentionBudget { limit: 0, used: 0 };
        let refusal = commit_observation(&mut committed, observation.clone(), &mut zero)
            .expect_err("a retained observation must consume at least one unit");
        assert_eq!(refusal.required, 1);
        assert_eq!(refusal.remaining, 0);
        assert!(committed.is_empty());
        assert_eq!(zero.used, 0);

        let mut one = RetentionBudget { limit: 1, used: 0 };
        commit_observation(&mut committed, observation, &mut one)
            .expect("one no-program observation fits one retained unit");
        assert_eq!(one.used, 1);
        assert_eq!(committed.len(), 1);
    }

    let zero_fact = qualified(program(9, Vec::new()));
    let mut committed = Vec::new();
    let mut zero = RetentionBudget { limit: 0, used: 0 };
    let refusal = commit_observation(&mut committed, zero_fact, &mut zero)
        .expect_err("a zero-fact qualified observation is still retained evidence");
    assert_eq!(refusal.required, 1);
    assert_eq!(refusal.remaining, 0);
    assert!(committed.is_empty());

    let first = nonqualified(
        1,
        CompletionState::Complete,
        GoldOutcome::Invalid(InvalidReason::SelectedGrammar),
    );
    let second = nonqualified(
        2,
        CompletionState::Complete,
        GoldOutcome::Invalid(InvalidReason::SelectedGrammar),
    );
    let mut committed = Vec::new();
    let mut one = RetentionBudget { limit: 1, used: 0 };
    commit_observation(&mut committed, first.clone(), &mut one).expect("first fits");
    let refusal = commit_observation(&mut committed, second, &mut one)
        .expect_err("the retained budget cannot reset for a second no-program observation");
    assert_eq!(refusal.required, 1);
    assert_eq!(refusal.remaining, 0);
    assert_eq!(one.used, 1);
    assert_eq!(committed, vec![first]);
}

#[test]
fn retention_and_consumer_resource_budgets_are_independent() {
    let observation = qualified(program(
        1,
        vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
    ));
    let mut committed = Vec::new();
    let mut retention = RetentionBudget { limit: 4, used: 0 };
    commit_observation(&mut committed, observation.clone(), &mut retention)
        .expect("retention fits");
    let before = retention;
    let result = fold_observation(&observation, ConsumerBudget { limit: 1 });
    assert_eq!(result.outcome, ConsumerOutcome::Incomplete);
    assert_eq!(retention, before);
}

#[test]
fn zero_budget_refuses_before_non_empty_dependency_graph_work() {
    let retained = run(vec![
        retained_class_and_id_parent(1),
        parent_relationship_child(2, 1),
    ]);
    let resolved = resolve_retained_run(&retained, ConsumerBudget { limit: 0 })
        .expect("budget refusal is consumer incompleteness, not a graph error");
    assert_eq!(resolved.completion(), ConsumerRunCompletion::Incomplete);
    assert_eq!(resolved.used(), 0);
    assert!(resolved.result(ContextId(1)).is_none());
    assert!(resolved.result(ContextId(2)).is_none());
}

#[test]
fn dependency_preparation_and_all_observations_share_one_aggregate_budget() {
    let retained = run(vec![
        minimal_independent_observation(1),
        minimal_independent_observation(2),
        minimal_independent_observation(3),
    ]);

    let complete = resolve_retained_run(&retained, ConsumerBudget { limit: usize::MAX })
        .expect("independent observations form a valid empty dependency graph");
    assert_eq!(complete.completion(), ConsumerRunCompletion::Complete);
    assert_eq!(complete.used(), 21);
    assert_eq!(
        complete.result(ContextId(1)).expect("first result").steps,
        3
    );
    assert_eq!(
        complete.result(ContextId(2)).expect("second result").steps,
        3
    );
    assert_eq!(
        complete.result(ContextId(3)).expect("third result").steps,
        3
    );

    let preparation_exhausted = resolve_retained_run(&retained, ConsumerBudget { limit: 8 })
        .expect("budgeted graph preparation may refuse before folding");
    assert_eq!(
        preparation_exhausted.completion(),
        ConsumerRunCompletion::Incomplete
    );
    assert_eq!(preparation_exhausted.used(), 8);
    assert!(
        preparation_exhausted.result(ContextId(1)).is_none(),
        "relationship discovery and graph preparation cannot run for free"
    );

    let exhausted = resolve_retained_run(&retained, ConsumerBudget { limit: 13 })
        .expect("aggregate exhaustion is consumer incompleteness");
    assert_eq!(exhausted.completion(), ConsumerRunCompletion::Incomplete);
    assert_eq!(exhausted.used(), 13);
    assert!(matches!(
        exhausted
            .result(ContextId(1))
            .expect("the completed prefix is retained")
            .outcome,
        ConsumerOutcome::Complete(_)
    ));
    assert_eq!(
        exhausted
            .result(ContextId(2))
            .expect("the first refused suffix observation is explicit")
            .outcome,
        ConsumerOutcome::Incomplete
    );
    assert_eq!(
        exhausted
            .result(ContextId(2))
            .expect("second result exists")
            .steps,
        0
    );
    assert!(exhausted.result(ContextId(3)).is_none());
    assert_eq!(
        exhausted.outcome(ContextId(3)),
        Some(ConsumerOutcome::Incomplete)
    );
}

#[test]
fn graph_parent_and_child_consume_the_same_remaining_budget() {
    let retained = run(vec![
        retained_class_and_id_parent(1),
        parent_relationship_child(2, 1),
    ]);

    let complete = resolve_retained_run(&retained, ConsumerBudget { limit: usize::MAX })
        .expect("the retained parent relationship is structurally valid");
    assert_eq!(complete.completion(), ConsumerRunCompletion::Complete);
    assert_eq!(complete.used(), 34);
    assert_eq!(
        complete.result(ContextId(1)).expect("parent result").steps,
        7
    );
    assert_eq!(
        complete.result(ContextId(2)).expect("child result").steps,
        6
    );

    let exhausted = resolve_retained_run(&retained, ConsumerBudget { limit: 32 })
        .expect("shared-budget exhaustion is consumer incompleteness");
    assert_eq!(exhausted.completion(), ConsumerRunCompletion::Incomplete);
    assert_eq!(exhausted.used(), 32);
    assert!(matches!(
        exhausted
            .result(ContextId(1))
            .expect("parent completed before exhaustion")
            .outcome,
        ConsumerOutcome::Complete(_)
    ));
    assert_eq!(
        exhausted
            .result(ContextId(2))
            .expect("child consumes only the remaining run budget")
            .outcome,
        ConsumerOutcome::Incomplete
    );
    assert_eq!(
        exhausted.result(ContextId(2)).expect("child result").steps,
        5
    );
}

#[test]
fn dependency_consumer_budget_does_not_mutate_retained_evidence_or_resources() {
    let parent = retained_class_and_id_parent(1);
    let child = parent_relationship_child(2, 1);
    let mut committed = Vec::new();
    let mut retention = RetentionBudget { limit: 12, used: 0 };
    commit_observation(&mut committed, parent, &mut retention).expect("parent retention fits");
    commit_observation(&mut committed, child, &mut retention).expect("child retention fits");
    let retained_before = committed.clone();
    let retention_before = retention;
    let retained_run = run(committed.clone());

    let zero = resolve_retained_run(&retained_run, ConsumerBudget { limit: 0 })
        .expect("zero consumer budget is an incomplete run");
    let complete = resolve_retained_run(&retained_run, ConsumerBudget { limit: usize::MAX })
        .expect("unbounded validation budget completes the retained run");

    assert_eq!(zero.completion(), ConsumerRunCompletion::Incomplete);
    assert_eq!(complete.completion(), ConsumerRunCompletion::Complete);
    assert_eq!(committed, retained_before);
    assert_eq!(retention, retention_before);
    assert_eq!(retained_run.upstream, CompletionState::Complete);
    assert_eq!(retained_run.qualifier, CompletionState::Complete);
}

#[test]
fn run_wide_consumer_accounting_and_result_prefix_are_deterministic() {
    let retained = run(vec![
        retained_class_and_id_parent(1),
        parent_relationship_child(2, 1),
    ]);
    let first = resolve_retained_run(&retained, ConsumerBudget { limit: 32 })
        .expect("first replay has a valid graph");
    let second = resolve_retained_run(&retained, ConsumerBudget { limit: 32 })
        .expect("second replay has a valid graph");
    assert_eq!(first, second);
    assert_eq!(first.completion(), ConsumerRunCompletion::Incomplete);
    assert_eq!(first.used(), 32);
    assert!(first.result(ContextId(1)).is_some());
    assert!(first.result(ContextId(2)).is_some());

    for limit in 0..=34 {
        let bounded = resolve_retained_run(&retained, ConsumerBudget { limit })
            .expect("the graph remains structurally valid at every consumer limit");
        assert!(bounded.used() <= limit);
        assert_eq!(
            bounded.completion(),
            if limit < 34 {
                ConsumerRunCompletion::Incomplete
            } else {
                ConsumerRunCompletion::Complete
            }
        );
    }
}

#[test]
fn retained_parent_members_are_folded_maximized_and_resolved_by_context() {
    let parent = retained_class_and_id_parent(1);
    let child = parent_relationship_child(2, 1);

    assert_eq!(
        fold_program(
            child
                .program
                .as_ref()
                .expect("qualified child has a program"),
            ConsumerBudget { limit: usize::MAX },
        )
        .outcome,
        ConsumerOutcome::Incomplete,
        "the public validation fold has no caller-supplied dependency-answer input"
    );

    let resolved = resolve_retained_run(
        &run(vec![parent, child]),
        ConsumerBudget { limit: usize::MAX },
    )
    .expect("the relationship points to an earlier retained context");
    assert_eq!(
        complete_members(
            resolved
                .result(ContextId(1))
                .expect("parent result exists")
                .outcome
                .clone()
        ),
        vec![
            (MemberId(1), specificity(0, 1, 0)),
            (MemberId(2), specificity(1, 0, 0)),
        ]
    );
    assert_eq!(
        resolved.dependency(ContextId(1)),
        Some(DependencyStatus::Resolved(specificity(1, 0, 0)))
    );
    assert_eq!(
        complete_members(
            resolved
                .result(ContextId(2))
                .expect("child result exists")
                .outcome
                .clone()
        ),
        vec![(MemberId(1), specificity(1, 0, 0))]
    );
}

#[test]
fn nonqualified_parent_observations_are_bound_to_enclosing_run_identity() {
    let child = parent_relationship_child(3, 2);
    for (source, run_id, profile) in [
        (SourceId(9), RunId(1), "CoreV1"),
        (SourceId(1), RunId(9), "CoreV1"),
        (SourceId(1), RunId(1), "OtherProfile"),
    ] {
        let foreign_parent = GoldObservation {
            source,
            run: run_id,
            profile,
            context: ContextId(2),
            completion: CompletionState::Complete,
            outcome: GoldOutcome::Invalid(InvalidReason::SelectedGrammar),
            program: None,
        };
        assert_eq!(
            resolve_retained_run(
                &run(vec![foreign_parent, child.clone()]),
                ConsumerBudget { limit: usize::MAX },
            ),
            Err(DependencyResolutionError::ObservationIdentityMismatch {
                context: ContextId(2),
                expected_source: SourceId(1),
                actual_source: source,
                expected_run: RunId(1),
                actual_run: run_id,
                expected_profile: "CoreV1",
                actual_profile: profile,
            })
        );
    }

    let local_parent = nonqualified(
        2,
        CompletionState::Complete,
        GoldOutcome::Invalid(InvalidReason::SelectedGrammar),
    );
    let resolved = resolve_retained_run(
        &run(vec![local_parent, child]),
        ConsumerBudget { limit: usize::MAX },
    )
    .expect("same-domain non-qualified parent remains a resolvable structural dependency");
    assert_eq!(
        resolved.dependency(ContextId(2)),
        Some(DependencyStatus::Invalid)
    );
    assert_eq!(
        resolved
            .result(ContextId(3))
            .expect("child result exists")
            .outcome,
        ConsumerOutcome::Blocked(BlockingOutcome::Invalid)
    );
}

#[test]
fn relationship_facts_reject_missing_future_self_and_cyclic_dependencies() {
    let missing = run(vec![parent_relationship_child(2, 99)]);
    assert_eq!(
        resolve_retained_run(&missing, ConsumerBudget { limit: usize::MAX }),
        Err(DependencyResolutionError::MissingContext {
            child: ContextId(2),
            parent: ContextId(99),
        })
    );

    let future = run(vec![
        parent_relationship_child(1, 2),
        retained_class_and_id_parent(2),
    ]);
    assert_eq!(
        resolve_retained_run(&future, ConsumerBudget { limit: usize::MAX }),
        Err(DependencyResolutionError::FutureContext {
            child: ContextId(1),
            parent: ContextId(2),
        })
    );

    let itself = run(vec![parent_relationship_child(1, 1)]);
    assert_eq!(
        resolve_retained_run(&itself, ConsumerBudget { limit: usize::MAX }),
        Err(DependencyResolutionError::SelfDependency(ContextId(1)))
    );

    let cycle = run(vec![
        parent_relationship_child(1, 2),
        parent_relationship_child(2, 1),
    ]);
    assert_eq!(
        resolve_retained_run(&cycle, ConsumerBudget { limit: usize::MAX }),
        Err(DependencyResolutionError::Cycle)
    );
}

#[test]
fn two_authored_nesting_occurrences_contribute_twice() {
    let child = qualified(program(
        2,
        vec![
            open(1, 0, 5),
            SelectorFact::NestingPresence {
                member: MemberId(1),
                unit: UnitId(1),
                origin: authored(range(0, 1)),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(1)),
                origin: authored(range(0, 1)),
            },
            SelectorFact::NestingPresence {
                member: MemberId(1),
                unit: UnitId(2),
                origin: authored(range(4, 5)),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(1)),
                origin: authored(range(4, 5)),
            },
            close(1),
        ],
    ));
    let facts = &child.program.as_ref().expect("child program exists").facts;
    assert_eq!(
        facts
            .iter()
            .filter(|fact| matches!(fact, SelectorFact::NestingPresence { .. }))
            .count(),
        2
    );
    assert_eq!(
        validate_program_authored_provenance(
            child.program.as_ref().expect("child program exists"),
            "& + &",
            &[
                authored_fact(0, 0, 5, "& + &"),
                authored_fact(1, 0, 1, "&"),
                authored_fact(2, 0, 1, "&"),
                authored_fact(3, 4, 5, "&"),
                authored_fact(4, 4, 5, "&"),
            ],
        ),
        Ok(())
    );

    let resolved = resolve_retained_run(
        &run(vec![retained_class_and_id_parent(1), child]),
        ConsumerBudget { limit: usize::MAX },
    )
    .expect("both relationships resolve through the retained parent");
    let value = complete_members(
        resolved
            .result(ContextId(2))
            .expect("child result exists")
            .outcome
            .clone(),
    )[0]
    .1;
    assert_eq!(value, specificity(2, 0, 0));

    let collapsed = resolve_retained_run(
        &run(vec![
            retained_class_and_id_parent(1),
            parent_relationship_child(2, 1),
        ]),
        ConsumerBudget { limit: usize::MAX },
    )
    .expect("one retained occurrence is structurally valid");
    assert_ne!(
        complete_members(
            collapsed
                .result(ContextId(2))
                .expect("collapsed child result exists")
                .outcome
                .clone()
        )[0]
        .1,
        specificity(2, 0, 0),
        "collapsing two authored ampersands cannot satisfy the gold result"
    );
}

#[test]
fn where_ampersand_preserves_presence_but_replaces_specificity_with_zero() {
    let child = qualified(program(
        2,
        vec![
            open(1, 0, 9),
            SelectorFact::OpenFunction {
                unit: UnitId(1),
                kind: FunctionKind::Where,
                range: range(0, 7),
            },
            open(2, 7, 8),
            SelectorFact::NestingPresence {
                member: MemberId(2),
                unit: UnitId(2),
                origin: authored(range(7, 8)),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(1)),
                origin: authored(range(7, 8)),
            },
            close(2),
            SelectorFact::CloseFunction { unit: UnitId(1) },
            close(1),
        ],
    ));
    assert!(
        child
            .program
            .as_ref()
            .expect("child program exists")
            .facts
            .iter()
            .any(|fact| matches!(fact, SelectorFact::NestingPresence { .. }))
    );
    assert_eq!(
        validate_program_authored_provenance(
            child.program.as_ref().expect("child program exists"),
            ":where(&)",
            &[
                authored_fact(0, 0, 9, ":where(&)"),
                authored_fact(1, 0, 7, ":where("),
                authored_fact(2, 7, 8, "&"),
                authored_fact(3, 7, 8, "&"),
                authored_fact(4, 7, 8, "&"),
            ],
        ),
        Ok(())
    );
    let resolved = resolve_retained_run(
        &run(vec![retained_class_and_id_parent(1), child]),
        ConsumerBudget { limit: usize::MAX },
    )
    .expect(":where relationship resolves structurally");
    assert_eq!(
        complete_members(
            resolved
                .result(ContextId(2))
                .expect("child result exists")
                .outcome
                .clone()
        ),
        vec![(MemberId(1), Specificity::ZERO)]
    );
}

#[test]
fn scope_relationship_adds_zero_specificity_and_ignores_scope_prelude() {
    let gold = program(
        2,
        vec![
            open(1, 14, 17),
            SelectorFact::Relationship {
                target: RelationshipTarget::ScopeRoot(ContextId(1)),
                origin: RelationshipOrigin::Derived,
            },
            atom(1, SimpleKind::Type, 14, 17),
            close(1),
        ],
    );
    let members =
        complete_members(fold_program(&gold, ConsumerBudget { limit: usize::MAX }).outcome);
    assert_eq!(members, vec![(MemberId(1), specificity(0, 0, 1))]);
}

#[test]
fn scope_and_nesting_order_resolve_same_relative_grammar_to_distinct_targets() {
    let retained_parent = qualified(program(
        1,
        vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
    ));

    // @scope { .a { & .b {} } }: the qualified style parent is nearest.
    let scope_outside_nested_style = qualified(program(
        2,
        vec![
            open(1, 0, 4),
            SelectorFact::NestingPresence {
                member: MemberId(1),
                unit: UnitId(1),
                origin: authored(range(0, 1)),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(1)),
                origin: authored(range(0, 1)),
            },
            atom(2, SimpleKind::Class, 2, 4),
            close(1),
        ],
    ));

    // .a { @scope { & .b {} } }: the intervening scope boundary is nearest.
    let scope_inside_ancestor_style = qualified(program(
        2,
        vec![
            open(1, 0, 4),
            SelectorFact::NestingPresence {
                member: MemberId(1),
                unit: UnitId(1),
                origin: authored(range(0, 1)),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ScopeRoot(ContextId(50)),
                origin: authored(range(0, 1)),
            },
            atom(2, SimpleKind::Class, 2, 4),
            close(1),
        ],
    ));

    for child in [&scope_outside_nested_style, &scope_inside_ancestor_style] {
        assert_eq!(
            validate_program_authored_provenance(
                child.program.as_ref().expect("child program exists"),
                "& .b",
                &[
                    authored_fact(0, 0, 4, "& .b"),
                    authored_fact(1, 0, 1, "&"),
                    authored_fact(2, 0, 1, "&"),
                    authored_fact(3, 2, 4, ".b"),
                ],
            ),
            Ok(())
        );
    }

    let parent_target = resolve_retained_run(
        &run(vec![retained_parent.clone(), scope_outside_nested_style]),
        ConsumerBudget { limit: usize::MAX },
    )
    .expect("earlier parent dependency resolves from retained evidence");
    let scope_target = resolve_retained_run(
        &run(vec![retained_parent, scope_inside_ancestor_style]),
        ConsumerBudget { limit: usize::MAX },
    )
    .expect("scope-root relationships require no parent specificity answer");

    assert_eq!(
        complete_members(
            parent_target
                .result(ContextId(2))
                .expect("child result exists")
                .outcome
                .clone()
        )[0]
        .1,
        specificity(0, 2, 0)
    );
    assert_eq!(
        complete_members(
            scope_target
                .result(ContextId(2))
                .expect("child result exists")
                .outcome
                .clone()
        )[0]
        .1,
        specificity(0, 1, 0)
    );
}

#[test]
fn identical_spelling_in_distinct_contexts_remains_distinguishable() {
    let left = program(
        10,
        vec![open(1, 0, 5), atom(1, SimpleKind::Class, 0, 5), close(1)],
    );
    let right = program(
        11,
        vec![
            open(1, 20, 25),
            atom(1, SimpleKind::Class, 20, 25),
            close(1),
        ],
    );
    assert_ne!(left.context, right.context);
    assert_ne!(left.facts, right.facts);
    let left_result = fold_program(&left, ConsumerBudget { limit: usize::MAX });
    let right_result = fold_program(&right, ConsumerBudget { limit: usize::MAX });
    assert_eq!(left_result.outcome, right_result.outcome);
}

fn function_fixture(kind: FunctionKind) -> GoldFixture {
    let (id, source, function_end, function_spelling, id_start, id_end, type_start, type_end) =
        match kind {
            FunctionKind::Is => (
                "CSS-HANDOFF-FUNCTION-IS-001",
                ".a:is(#b, c)",
                6,
                ":is(",
                6,
                8,
                10,
                11,
            ),
            FunctionKind::Not => (
                "CSS-HANDOFF-FUNCTION-NOT-001",
                ".a:not(#b, c)",
                7,
                ":not(",
                7,
                9,
                11,
                12,
            ),
            FunctionKind::Has => (
                "CSS-HANDOFF-FUNCTION-HAS-001",
                ".a:has(> #b, + c)",
                7,
                ":has(",
                9,
                11,
                15,
                16,
            ),
            FunctionKind::Where => (
                "CSS-HANDOFF-FUNCTION-WHERE-001",
                ".a:where(#b, c)",
                9,
                ":where(",
                9,
                11,
                13,
                14,
            ),
        };
    let source_end = source.len();
    GoldFixture {
        id,
        source,
        program: program(
            1,
            vec![
                open(1, 0, source_end),
                atom(1, SimpleKind::Class, 0, 2),
                SelectorFact::OpenFunction {
                    unit: UnitId(2),
                    kind,
                    range: range(2, function_end),
                },
                open(2, id_start, id_end),
                atom(3, SimpleKind::Id, id_start, id_end),
                close(2),
                open(3, type_start, type_end),
                atom(4, SimpleKind::Type, type_start, type_end),
                close(3),
                SelectorFact::CloseFunction { unit: UnitId(2) },
                close(1),
            ],
        ),
        authored: vec![
            authored_fact(0, 0, source_end, source),
            authored_fact(1, 0, 2, ".a"),
            authored_fact(2, 2, function_end, function_spelling),
            authored_fact(3, id_start, id_end, "#b"),
            authored_fact(4, id_start, id_end, "#b"),
            authored_fact(6, type_start, type_end, "c"),
            authored_fact(7, type_start, type_end, "c"),
        ],
    }
}

#[test]
fn source_only_fold_covers_basic_and_selected_function_specificity() {
    let basic = GoldFixture {
        id: "CSS-HANDOFF-BASIC-ATOMS-001",
        source: "*.a[b]:",
        program: program(
            1,
            vec![
                open(1, 0, 7),
                atom(1, SimpleKind::Universal, 0, 1),
                atom(2, SimpleKind::Class, 1, 3),
                atom(3, SimpleKind::Attribute, 3, 6),
                atom(4, SimpleKind::IdentifierPseudoClass, 6, 7),
                close(1),
            ],
        ),
        authored: vec![
            authored_fact(0, 0, 7, "*.a[b]:"),
            authored_fact(1, 0, 1, "*"),
            authored_fact(2, 1, 3, ".a"),
            authored_fact(3, 3, 6, "[b]"),
            authored_fact(4, 6, 7, ":"),
        ],
    };
    assert_eq!(
        validate_program_authored_provenance(&basic.program, basic.source, &basic.authored),
        Ok(())
    );
    assert_eq!(
        complete_members(fold_program(&basic.program, ConsumerBudget { limit: usize::MAX }).outcome)
            [0]
            .1,
        specificity(0, 3, 0)
    );

    for kind in [FunctionKind::Is, FunctionKind::Not, FunctionKind::Has] {
        let fixture = function_fixture(kind);
        assert_eq!(
            validate_program_authored_provenance(
                &fixture.program,
                fixture.source,
                &fixture.authored,
            ),
            Ok(())
        );
        assert_eq!(
            complete_members(
                fold_program(&fixture.program, ConsumerBudget { limit: usize::MAX }).outcome
            )[0]
                .1,
            specificity(1, 1, 0)
        );
    }
    let where_fixture = function_fixture(FunctionKind::Where);
    assert_eq!(
        validate_program_authored_provenance(
            &where_fixture.program,
            where_fixture.source,
            &where_fixture.authored,
        ),
        Ok(())
    );
    assert_eq!(
        complete_members(
            fold_program(
                &where_fixture.program,
                ConsumerBudget { limit: usize::MAX },
            )
            .outcome
        )[0]
            .1,
        specificity(0, 1, 0)
    );
}

#[test]
fn selector_list_output_stays_per_member_not_match_effective() {
    let gold = program(
        1,
        vec![
            open(1, 0, 1),
            atom(1, SimpleKind::Type, 0, 1),
            close(1),
            open(2, 3, 5),
            atom(2, SimpleKind::Id, 3, 5),
            close(2),
        ],
    );
    let members =
        complete_members(fold_program(&gold, ConsumerBudget { limit: usize::MAX }).outcome);
    assert_eq!(members.len(), 2);
    assert_eq!(members[0].1, specificity(0, 0, 1));
    assert_eq!(members[1].1, specificity(1, 0, 0));
}

#[test]
fn unsupported_and_namespace_indeterminate_are_not_promoted() {
    for (outcome, expected) in [
        (
            GoldOutcome::Unsupported(UnsupportedFeature::FunctionalPseudoClass),
            BlockingOutcome::Unsupported,
        ),
        (
            GoldOutcome::Indeterminate(IndeterminateReason::MissingNamespaceEnvironment),
            BlockingOutcome::Indeterminate,
        ),
    ] {
        let observation = nonqualified(4, CompletionState::Complete, outcome);
        assert_eq!(
            fold_observation(&observation, ConsumerBudget { limit: usize::MAX }).outcome,
            ConsumerOutcome::Blocked(expected)
        );
    }
}

#[test]
fn parent_failure_category_is_preserved_through_structural_dependency() {
    let child = qualified(program(
        3,
        vec![
            open(1, 0, 2),
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(2)),
                origin: RelationshipOrigin::Derived,
            },
            atom(1, SimpleKind::Class, 0, 2),
            close(1),
        ],
    ));
    let mut incomplete_parent = qualified(program(
        2,
        vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
    ));
    incomplete_parent.completion = CompletionState::Incomplete;
    for (parent, dependency, expected) in [
        (
            nonqualified(
                2,
                CompletionState::Complete,
                GoldOutcome::Invalid(InvalidReason::SelectedGrammar),
            ),
            DependencyStatus::Invalid,
            ConsumerOutcome::Blocked(BlockingOutcome::Invalid),
        ),
        (
            nonqualified(
                2,
                CompletionState::Complete,
                GoldOutcome::Unsupported(UnsupportedFeature::PseudoElement),
            ),
            DependencyStatus::Unsupported,
            ConsumerOutcome::Blocked(BlockingOutcome::Unsupported),
        ),
        (
            nonqualified(
                2,
                CompletionState::Complete,
                GoldOutcome::Indeterminate(IndeterminateReason::MissingNamespaceEnvironment),
            ),
            DependencyStatus::Indeterminate,
            ConsumerOutcome::Blocked(BlockingOutcome::Indeterminate),
        ),
        (
            incomplete_parent,
            DependencyStatus::Incomplete,
            ConsumerOutcome::Incomplete,
        ),
    ] {
        let resolved = resolve_retained_run(
            &run(vec![parent, child.clone()]),
            ConsumerBudget { limit: usize::MAX },
        )
        .expect("the relationship targets an earlier retained parent");
        assert_eq!(resolved.dependency(ContextId(2)), Some(dependency));
        if dependency == DependencyStatus::Incomplete {
            assert_eq!(resolved.completion(), ConsumerRunCompletion::Incomplete);
            assert!(resolved.result(ContextId(3)).is_none());
            assert_eq!(
                resolved.outcome(ContextId(3)),
                Some(ConsumerOutcome::Incomplete)
            );
        } else {
            assert_eq!(resolved.completion(), ConsumerRunCompletion::Complete);
            assert_eq!(
                resolved
                    .result(ContextId(3))
                    .expect("child result exists")
                    .outcome
                    .clone(),
                expected
            );
        }
    }
}

#[test]
fn parent_consumer_exhaustion_propagates_incomplete_through_the_resolver() {
    let resolved = resolve_retained_run(
        &run(vec![
            retained_class_and_id_parent(2),
            parent_relationship_child(3, 2),
        ]),
        ConsumerBudget { limit: 24 },
    )
    .expect("the dependency graph is structurally valid");

    assert_eq!(resolved.completion(), ConsumerRunCompletion::Incomplete);
    assert_eq!(resolved.used(), 24);
    assert_eq!(
        resolved.dependency(ContextId(2)),
        Some(DependencyStatus::Incomplete)
    );
    assert_eq!(
        resolved
            .result(ContextId(2))
            .expect("parent records its exhausted fold")
            .outcome,
        ConsumerOutcome::Incomplete
    );
    assert_eq!(
        resolved
            .result(ContextId(2))
            .expect("parent result exists")
            .steps,
        6
    );
    assert!(
        resolved.result(ContextId(3)).is_none(),
        "the child cannot receive a precomputed or fresh-budget parent answer"
    );
    assert_eq!(
        resolved.outcome(ContextId(3)),
        Some(ConsumerOutcome::Incomplete)
    );
}

#[test]
fn invalid_forgiving_ampersand_can_suppress_implied_nesting_without_contribution() {
    let fixture = rejected_forgiving_ampersand_fixture();
    assert_eq!(
        validate_program_authored_provenance(&fixture.program, fixture.source, &fixture.authored),
        Ok(())
    );
    let evidence =
        validate_rejected_nesting_presence(&fixture.program, rejected_ampersand_expectation())
            .expect("the rejected member retains exact authored non-contributing presence");
    assert_eq!(evidence.member, MemberId(2));
    assert_eq!(evidence.unit, UnitId(3));
    assert_eq!(evidence.authored_range, range(10, 11));
    assert_eq!(
        evidence.disposition,
        NestingPresenceDisposition::NonContributingPresenceOnly
    );
    assert_eq!(
        evidence.effect,
        RejectedNestingEffect::SuppressesImpliedNesting
    );

    let first = fold_program(&fixture.program, ConsumerBudget { limit: usize::MAX });
    let second = fold_program(&fixture.program, ConsumerBudget { limit: usize::MAX });
    assert_eq!(first, second);
    assert_eq!(complete_members(first.outcome)[0].1, specificity(0, 2, 0));
}

#[test]
fn rejected_forgiving_ampersand_presence_cannot_be_deleted() {
    let mut fixture = rejected_forgiving_ampersand_fixture();
    fixture
        .program
        .facts
        .retain(|fact| !matches!(fact, SelectorFact::NestingPresence { .. }));
    assert_eq!(
        validate_rejected_nesting_presence(&fixture.program, rejected_ampersand_expectation(),),
        Err(RejectedNestingPresenceFailure::PresenceMissing)
    );
}

#[test]
fn rejected_forgiving_ampersand_presence_cannot_contribute_or_gain_a_relationship() {
    let mut contributing = rejected_forgiving_ampersand_fixture();
    let presence = contributing
        .program
        .facts
        .iter_mut()
        .find(|fact| matches!(fact, SelectorFact::NestingPresence { .. }))
        .expect("fixture has rejected-member presence");
    let SelectorFact::NestingPresence { disposition, .. } = presence else {
        unreachable!("selected fact is nesting presence");
    };
    *disposition = NestingPresenceDisposition::Contributing;
    assert_eq!(
        validate_rejected_nesting_presence(&contributing.program, rejected_ampersand_expectation(),),
        Err(RejectedNestingPresenceFailure::PresenceMustBeNonContributing)
    );

    let mut relationship = rejected_forgiving_ampersand_fixture();
    let presence_position = relationship
        .program
        .facts
        .iter()
        .position(|fact| matches!(fact, SelectorFact::NestingPresence { .. }))
        .expect("fixture has rejected-member presence");
    relationship.program.facts.insert(
        presence_position + 1,
        SelectorFact::Relationship {
            target: RelationshipTarget::ParentSelectorList(ContextId(1)),
            origin: authored(range(10, 11)),
        },
    );
    assert_eq!(
        validate_rejected_nesting_presence(&relationship.program, rejected_ampersand_expectation(),),
        Err(RejectedNestingPresenceFailure::ContributingRelationshipInRejectedMember)
    );
}

#[test]
fn rejected_forgiving_ampersand_presence_cannot_move_to_another_member() {
    let mut wrong_presence_member = rejected_forgiving_ampersand_fixture();
    let presence = wrong_presence_member
        .program
        .facts
        .iter_mut()
        .find(|fact| matches!(fact, SelectorFact::NestingPresence { .. }))
        .expect("fixture has rejected-member presence");
    let SelectorFact::NestingPresence { member, .. } = presence else {
        unreachable!("selected fact is nesting presence");
    };
    *member = MemberId(3);
    assert_eq!(
        validate_rejected_nesting_presence(
            &wrong_presence_member.program,
            rejected_ampersand_expectation(),
        ),
        Err(RejectedNestingPresenceFailure::PresenceMemberMismatch {
            expected: MemberId(2),
            actual: MemberId(3),
        })
    );

    let mut lost_rejected_identity = rejected_forgiving_ampersand_fixture();
    let rejected = lost_rejected_identity
        .program
        .facts
        .iter_mut()
        .find(|fact| matches!(fact, SelectorFact::RejectedForgivingMember { .. }))
        .expect("fixture has rejected member");
    let SelectorFact::RejectedForgivingMember { member, .. } = rejected else {
        unreachable!("selected fact is rejected member");
    };
    *member = MemberId(9);
    assert_eq!(
        validate_rejected_nesting_presence(
            &lost_rejected_identity.program,
            rejected_ampersand_expectation(),
        ),
        Err(RejectedNestingPresenceFailure::RejectedMemberMissing)
    );
}

#[test]
fn rejected_forgiving_ampersand_presence_requires_exact_authored_provenance() {
    let mut wrong_range = rejected_forgiving_ampersand_fixture();
    let presence = wrong_range
        .program
        .facts
        .iter_mut()
        .find(|fact| matches!(fact, SelectorFact::NestingPresence { .. }))
        .expect("fixture has rejected-member presence");
    let SelectorFact::NestingPresence { origin, .. } = presence else {
        unreachable!("selected fact is nesting presence");
    };
    *origin = authored(range(9, 10));
    assert_eq!(
        validate_rejected_nesting_presence(&wrong_range.program, rejected_ampersand_expectation(),),
        Err(RejectedNestingPresenceFailure::PresenceRangeMismatch {
            expected: range(10, 11),
            actual: range(9, 10),
        })
    );

    let mut derived_origin = rejected_forgiving_ampersand_fixture();
    let presence = derived_origin
        .program
        .facts
        .iter_mut()
        .find(|fact| matches!(fact, SelectorFact::NestingPresence { .. }))
        .expect("fixture has rejected-member presence");
    let SelectorFact::NestingPresence { origin, .. } = presence else {
        unreachable!("selected fact is nesting presence");
    };
    *origin = derived();
    assert_eq!(
        validate_rejected_nesting_presence(
            &derived_origin.program,
            rejected_ampersand_expectation(),
        ),
        Err(RejectedNestingPresenceFailure::PresenceMustBeAuthored)
    );
}
