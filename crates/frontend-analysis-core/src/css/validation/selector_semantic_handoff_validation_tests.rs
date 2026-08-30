use super::selector_semantic_handoff_gold::{
    AuthoredRange, CompletionState, ContextId, FunctionKind, GoldFixture, GoldObservation,
    GoldOutcome, GoldProgram, GoldRun, IndeterminateReason, InvalidReason, LiteralRangeExpectation,
    LiteralRangeFailure, MemberId, NestingPresenceDisposition, RelationshipOrigin,
    RelationshipTarget, RunId, SelectorFact, SimpleKind, SourceId, UnitId, UnsupportedFeature,
    authored, derived, verify_literal_range,
};
use super::selector_semantic_handoff_reference::{
    BlockingOutcome, ConsumerBudget, ConsumerOutcome, DependencyResolutionError, DependencyStatus,
    RetentionBudget, Specificity, commit_observation, fold_observation, fold_program,
    resolve_retained_run,
};

fn range(start: usize, end: usize) -> AuthoredRange {
    AuthoredRange::new(start, end)
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
        context: program.context,
        completion: CompletionState::Complete,
        outcome: GoldOutcome::Qualified,
        program: Some(program),
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
                unit: UnitId(1),
                range: range(0, 1),
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
            LiteralRangeExpectation {
                range: range(0, 12),
                spelling: ".a:is(#b, c)",
            },
            LiteralRangeExpectation {
                range: range(0, 2),
                spelling: ".a",
            },
            LiteralRangeExpectation {
                range: range(2, 6),
                spelling: ":is(",
            },
            LiteralRangeExpectation {
                range: range(6, 8),
                spelling: "#b",
            },
            LiteralRangeExpectation {
                range: range(10, 11),
                spelling: "c",
            },
        ],
    };

    assert_eq!(fixture.id, "CSS-HANDOFF-RANGE-IS-001");
    assert_eq!(fixture.program.source, SourceId(1));
    assert_eq!(fixture.program.context, ContextId(1));
    for expectation in fixture.authored {
        assert_eq!(verify_literal_range(fixture.source, expectation), Ok(()));
    }
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
            authored: vec![LiteralRangeExpectation {
                range: range(2, 4),
                spelling: "#x",
            }],
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
            authored: vec![LiteralRangeExpectation {
                range: range(5, 7),
                spelling: ".x",
            }],
        },
    ];

    for fixture in fixtures {
        for expectation in fixture.authored {
            assert_eq!(verify_literal_range(fixture.source, expectation), Ok(()));
        }
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
    assert_eq!(
        complete_members(result.outcome),
        vec![(MemberId(1), specificity(0, 0, 1))]
    );
    assert!(resolve(&run(vec![qualified(gold)]), ConsumerBudget { limit: 3 }).is_ok());
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
        let observation = GoldObservation {
            context: ContextId(1),
            completion: CompletionState::Complete,
            outcome,
            program: None,
        };
        assert_eq!(
            fold_observation(&observation, ConsumerBudget { limit: 0 }).outcome,
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
    for run in [
        GoldRun {
            upstream: CompletionState::Incomplete,
            qualifier: CompletionState::Complete,
            observations: vec![observation.clone()],
        },
        GoldRun {
            upstream: CompletionState::Complete,
            qualifier: CompletionState::Incomplete,
            observations: vec![observation.clone()],
        },
    ] {
        assert_eq!(
            resolve_retained_run(&run, ConsumerBudget { limit: usize::MAX })
                .expect("an incomplete run still has structurally valid retained evidence")
                .result(ContextId(1))
                .expect("context result exists")
                .outcome
                .clone(),
            ConsumerOutcome::Incomplete
        );
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
        .expect_err("three facts must not partially fit a budget of two");
    assert!(committed.is_empty());
    assert_eq!(budget.used, 0);
    assert_eq!(refusal.required, 3);
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
    assert_eq!(budget.used, 2);
}

#[test]
fn retention_and_consumer_resource_budgets_are_independent() {
    let observation = qualified(program(
        1,
        vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
    ));
    let mut committed = Vec::new();
    let mut retention = RetentionBudget { limit: 3, used: 0 };
    commit_observation(&mut committed, observation.clone(), &mut retention)
        .expect("retention fits");
    let before = retention;
    let result = fold_observation(&observation, ConsumerBudget { limit: 1 });
    assert_eq!(result.outcome, ConsumerOutcome::Incomplete);
    assert_eq!(retention, before);
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
                unit: UnitId(1),
                range: range(0, 1),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(1)),
                origin: authored(range(0, 1)),
            },
            SelectorFact::NestingPresence {
                unit: UnitId(2),
                range: range(4, 5),
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
    for expectation in [
        LiteralRangeExpectation {
            range: range(0, 1),
            spelling: "&",
        },
        LiteralRangeExpectation {
            range: range(4, 5),
            spelling: "&",
        },
    ] {
        assert_eq!(verify_literal_range("& + &", expectation), Ok(()));
    }

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
                unit: UnitId(2),
                range: range(7, 8),
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
    for expectation in [
        LiteralRangeExpectation {
            range: range(0, 7),
            spelling: ":where(",
        },
        LiteralRangeExpectation {
            range: range(7, 8),
            spelling: "&",
        },
    ] {
        assert_eq!(verify_literal_range(":where(&)", expectation), Ok(()));
    }
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
                unit: UnitId(1),
                range: range(0, 1),
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
                unit: UnitId(1),
                range: range(0, 1),
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

fn function_program(kind: FunctionKind) -> GoldProgram {
    program(
        1,
        vec![
            open(1, 0, 12),
            atom(1, SimpleKind::Class, 0, 2),
            SelectorFact::OpenFunction {
                unit: UnitId(2),
                kind,
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
    )
}

#[test]
fn source_only_fold_covers_basic_and_selected_function_specificity() {
    let basic = program(
        1,
        vec![
            open(1, 0, 7),
            atom(1, SimpleKind::Universal, 0, 1),
            atom(2, SimpleKind::Class, 1, 3),
            atom(3, SimpleKind::Attribute, 3, 6),
            atom(4, SimpleKind::IdentifierPseudoClass, 6, 7),
            close(1),
        ],
    );
    assert_eq!(
        complete_members(fold_program(&basic, ConsumerBudget { limit: usize::MAX }).outcome)[0].1,
        specificity(0, 3, 0)
    );

    for kind in [FunctionKind::Is, FunctionKind::Not, FunctionKind::Has] {
        assert_eq!(
            complete_members(
                fold_program(
                    &function_program(kind),
                    ConsumerBudget { limit: usize::MAX }
                )
                .outcome
            )[0]
            .1,
            specificity(1, 1, 0)
        );
    }
    assert_eq!(
        complete_members(
            fold_program(
                &function_program(FunctionKind::Where),
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
        let observation = GoldObservation {
            context: ContextId(4),
            completion: CompletionState::Complete,
            outcome,
            program: None,
        };
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
    let incomplete_parent = GoldObservation {
        context: ContextId(2),
        completion: CompletionState::Incomplete,
        outcome: GoldOutcome::Qualified,
        program: Some(program(
            2,
            vec![open(1, 0, 2), atom(1, SimpleKind::Class, 0, 2), close(1)],
        )),
    };
    for (parent, dependency, expected) in [
        (
            GoldObservation {
                context: ContextId(2),
                completion: CompletionState::Complete,
                outcome: GoldOutcome::Invalid(InvalidReason::SelectedGrammar),
                program: None,
            },
            DependencyStatus::Invalid,
            ConsumerOutcome::Blocked(BlockingOutcome::Invalid),
        ),
        (
            GoldObservation {
                context: ContextId(2),
                completion: CompletionState::Complete,
                outcome: GoldOutcome::Unsupported(UnsupportedFeature::PseudoElement),
                program: None,
            },
            DependencyStatus::Unsupported,
            ConsumerOutcome::Blocked(BlockingOutcome::Unsupported),
        ),
        (
            GoldObservation {
                context: ContextId(2),
                completion: CompletionState::Complete,
                outcome: GoldOutcome::Indeterminate(
                    IndeterminateReason::MissingNamespaceEnvironment,
                ),
                program: None,
            },
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

#[test]
fn parent_consumer_exhaustion_propagates_incomplete_through_the_resolver() {
    let resolved = resolve_retained_run(
        &run(vec![
            retained_class_and_id_parent(2),
            parent_relationship_child(3, 2),
        ]),
        ConsumerBudget { limit: 5 },
    )
    .expect("the dependency graph is structurally valid");

    assert_eq!(
        resolved.dependency(ContextId(2)),
        Some(DependencyStatus::Incomplete)
    );
    assert_eq!(
        resolved
            .result(ContextId(3))
            .expect("child result exists")
            .outcome
            .clone(),
        ConsumerOutcome::Incomplete
    );
}

#[test]
fn invalid_forgiving_ampersand_can_suppress_implied_nesting_without_contribution() {
    let gold = program(
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
                unit: UnitId(3),
                range: range(10, 11),
                disposition: NestingPresenceDisposition::NonContributingPresenceOnly,
            },
            open(3, 13, 15),
            atom(4, SimpleKind::Class, 13, 15),
            close(3),
            SelectorFact::CloseFunction { unit: UnitId(2) },
            close(1),
        ],
    );
    let first = fold_program(&gold, ConsumerBudget { limit: usize::MAX });
    let second = fold_program(&gold, ConsumerBudget { limit: usize::MAX });
    assert_eq!(first, second);
    assert_eq!(complete_members(first.outcome)[0].1, specificity(0, 2, 0));
}
