use std::collections::BTreeMap;

use super::selector_semantic_handoff_gold::{
    AuthoredRange, CompletionState, ContextId, FunctionKind, GoldFixture, GoldObservation,
    GoldOutcome, GoldProgram, GoldRun, IndeterminateReason, InvalidReason,
    LiteralRangeExpectation, MemberId, NestingPresenceDisposition, RelationshipOrigin,
    RelationshipTarget, RunId, SelectorFact, SimpleKind, SourceId, UnitId, UnsupportedFeature,
    authored, derived,
};
use super::selector_semantic_handoff_reference::{
    BlockingOutcome, ConsumerBudget, ConsumerOutcome, DependencyStatus, RetentionBudget,
    Specificity, commit_observation, dependency_graph_is_acyclic, fold_observation, fold_program,
    fold_run,
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

fn empty_dependencies() -> BTreeMap<ContextId, DependencyStatus> {
    BTreeMap::new()
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
    let result = fold_program(
        &gold,
        &empty_dependencies(),
        ConsumerBudget { limit: usize::MAX },
    );
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
        assert!(expectation.range.start <= expectation.range.end);
        assert!(expectation.range.end <= fixture.source.len());
        assert!(fixture.source.is_char_boundary(expectation.range.start));
        assert!(fixture.source.is_char_boundary(expectation.range.end));
        assert_eq!(
            &fixture.source[expectation.range.start..expectation.range.end],
            expectation.spelling
        );
    }
}

#[test]
fn authored_and_derived_relationship_identity_are_disjoint() {
    let authored_origin = authored(range(10, 11));
    let derived_origin = derived();
    assert!(matches!(
        authored_origin,
        RelationshipOrigin::Authored(_)
    ));
    assert_eq!(derived_origin, RelationshipOrigin::Derived);
}

#[test]
fn reference_fold_is_source_token_and_parser_free_by_signature() {
    let fold: fn(
        &GoldProgram,
        &BTreeMap<ContextId, DependencyStatus>,
        ConsumerBudget,
    ) -> super::selector_semantic_handoff_reference::ConsumerResult = fold_program;
    let gold = program(
        1,
        vec![
            open(1, 0, 1),
            atom(1, SimpleKind::Type, 0, 1),
            close(1),
        ],
    );
    let result = fold(&gold, &empty_dependencies(), ConsumerBudget { limit: 3 });
    assert_eq!(
        complete_members(result.outcome),
        vec![(MemberId(1), specificity(0, 0, 1))]
    );
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
            outcome,
            program: None,
        };
        assert_eq!(
            fold_observation(
                &observation,
                &empty_dependencies(),
                ConsumerBudget { limit: 0 }
            )
            .outcome,
            ConsumerOutcome::Blocked(expected)
        );
    }
}

#[test]
fn incomplete_upstream_or_qualifier_cannot_upgrade_to_complete() {
    let observation = qualified(program(
        1,
        vec![
            open(1, 0, 1),
            atom(1, SimpleKind::Type, 0, 1),
            close(1),
        ],
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
            fold_run(
                &run,
                &empty_dependencies(),
                ConsumerBudget { limit: usize::MAX }
            )[0]
            .outcome,
            ConsumerOutcome::Incomplete
        );
    }
}

#[test]
fn semantic_program_commit_is_atomic() {
    let observation = qualified(program(
        2,
        vec![
            open(1, 0, 2),
            atom(1, SimpleKind::Class, 0, 2),
            close(1),
        ],
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
        vec![
            open(2, 0, 2),
            atom(2, SimpleKind::Class, 0, 2),
            close(2),
        ],
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
        vec![
            open(1, 0, 2),
            atom(1, SimpleKind::Class, 0, 2),
            close(1),
        ],
    ));
    let mut committed = Vec::new();
    let mut retention = RetentionBudget { limit: 3, used: 0 };
    commit_observation(&mut committed, observation.clone(), &mut retention)
        .expect("retention fits");
    let before = retention;
    let result = fold_observation(
        &observation,
        &empty_dependencies(),
        ConsumerBudget { limit: 1 },
    );
    assert_eq!(result.outcome, ConsumerOutcome::Incomplete);
    assert_eq!(retention, before);
}

#[test]
fn parent_dependencies_are_explicit_earlier_and_acyclic() {
    let edges = [
        (ContextId(3), ContextId(2)),
        (ContextId(2), ContextId(1)),
    ];
    assert!(edges.iter().all(|(child, parent)| child.0 > parent.0));
    assert!(dependency_graph_is_acyclic(&edges));
    assert!(!dependency_graph_is_acyclic(&[
        (ContextId(3), ContextId(2)),
        (ContextId(2), ContextId(1)),
        (ContextId(1), ContextId(3)),
    ]));
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
    let members = complete_members(
        fold_program(
            &gold,
            &empty_dependencies(),
            ConsumerBudget { limit: usize::MAX },
        )
        .outcome,
    );
    assert_eq!(members, vec![(MemberId(1), specificity(0, 0, 1))]);
}

#[test]
fn same_relative_grammar_can_resolve_to_parent_or_scope_target() {
    let nested = program(
        3,
        vec![
            open(1, 0, 4),
            SelectorFact::NestingPresence {
                unit: UnitId(1),
                range: range(0, 1),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(ContextId(2)),
                origin: authored(range(0, 1)),
            },
            atom(2, SimpleKind::Class, 2, 4),
            close(1),
        ],
    );
    let scoped = program(
        3,
        vec![
            open(1, 0, 4),
            SelectorFact::NestingPresence {
                unit: UnitId(1),
                range: range(0, 1),
                disposition: NestingPresenceDisposition::Contributing,
            },
            SelectorFact::Relationship {
                target: RelationshipTarget::ScopeRoot(ContextId(2)),
                origin: authored(range(0, 1)),
            },
            atom(2, SimpleKind::Class, 2, 4),
            close(1),
        ],
    );
    let mut dependencies = BTreeMap::new();
    dependencies.insert(
        ContextId(2),
        DependencyStatus::Resolved(specificity(0, 1, 0)),
    );
    let nested_value = complete_members(
        fold_program(
            &nested,
            &dependencies,
            ConsumerBudget { limit: usize::MAX },
        )
        .outcome,
    );
    let scoped_value = complete_members(
        fold_program(
            &scoped,
            &dependencies,
            ConsumerBudget { limit: usize::MAX },
        )
        .outcome,
    );
    assert_eq!(nested_value[0].1, specificity(0, 2, 0));
    assert_eq!(scoped_value[0].1, specificity(0, 1, 0));
}

#[test]
fn identical_spelling_in_distinct_contexts_remains_distinguishable() {
    let left = program(
        10,
        vec![
            open(1, 0, 5),
            atom(1, SimpleKind::Class, 0, 5),
            close(1),
        ],
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
    let left_result = fold_program(
        &left,
        &empty_dependencies(),
        ConsumerBudget { limit: usize::MAX },
    );
    let right_result = fold_program(
        &right,
        &empty_dependencies(),
        ConsumerBudget { limit: usize::MAX },
    );
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
        complete_members(
            fold_program(
                &basic,
                &empty_dependencies(),
                ConsumerBudget { limit: usize::MAX },
            )
            .outcome
        )[0]
            .1,
        specificity(0, 3, 0)
    );

    for kind in [FunctionKind::Is, FunctionKind::Not, FunctionKind::Has] {
        assert_eq!(
            complete_members(
                fold_program(
                    &function_program(kind),
                    &empty_dependencies(),
                    ConsumerBudget { limit: usize::MAX },
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
                &empty_dependencies(),
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
    let members = complete_members(
        fold_program(
            &gold,
            &empty_dependencies(),
            ConsumerBudget { limit: usize::MAX },
        )
        .outcome,
    );
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
            outcome,
            program: None,
        };
        assert_eq!(
            fold_observation(
                &observation,
                &empty_dependencies(),
                ConsumerBudget { limit: usize::MAX }
            )
            .outcome,
            ConsumerOutcome::Blocked(expected)
        );
    }
}

#[test]
fn parent_failure_category_is_preserved_through_structural_dependency() {
    let gold = program(
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
    );
    for (dependency, expected) in [
        (
            DependencyStatus::Invalid,
            ConsumerOutcome::Blocked(BlockingOutcome::Invalid),
        ),
        (
            DependencyStatus::Unsupported,
            ConsumerOutcome::Blocked(BlockingOutcome::Unsupported),
        ),
        (
            DependencyStatus::Indeterminate,
            ConsumerOutcome::Blocked(BlockingOutcome::Indeterminate),
        ),
        (DependencyStatus::Incomplete, ConsumerOutcome::Incomplete),
    ] {
        let mut dependencies = BTreeMap::new();
        dependencies.insert(ContextId(2), dependency);
        assert_eq!(
            fold_program(
                &gold,
                &dependencies,
                ConsumerBudget { limit: usize::MAX }
            )
            .outcome,
            expected
        );
    }
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
    let mut dependencies = BTreeMap::new();
    dependencies.insert(
        ContextId(2),
        DependencyStatus::Resolved(specificity(1, 0, 0)),
    );
    let first = fold_program(
        &gold,
        &dependencies,
        ConsumerBudget { limit: usize::MAX },
    );
    let second = fold_program(
        &gold,
        &dependencies,
        ConsumerBudget { limit: usize::MAX },
    );
    assert_eq!(first, second);
    assert_eq!(
        complete_members(first.outcome)[0].1,
        specificity(0, 2, 0)
    );
}
