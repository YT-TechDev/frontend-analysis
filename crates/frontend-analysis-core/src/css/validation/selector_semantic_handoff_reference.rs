//! Source-free reference fold for #402 semantic-handoff validation.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::selector_semantic_handoff_gold::{
    AuthoredRange, CompletionState, ContextId, FunctionKind, GoldObservation, GoldOutcome,
    GoldProgram, GoldRun, MemberId, RelationshipOrigin, RelationshipTarget, RunId, SelectorFact,
    SimpleKind, SourceId, UnitId,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Specificity {
    pub(super) a: u32,
    pub(super) b: u32,
    pub(super) c: u32,
}

impl Specificity {
    pub(super) const ZERO: Self = Self { a: 0, b: 0, c: 0 };

    fn add(self, other: Self) -> Self {
        Self {
            a: self.a + other.a,
            b: self.b + other.b,
            c: self.c + other.c,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DependencyStatus {
    Resolved(Specificity),
    Invalid,
    Unsupported,
    Indeterminate,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DependencyResolutionError {
    DuplicateContext(ContextId),
    QualifiedWithoutProgram(ContextId),
    ProgramContextMismatch {
        observation: ContextId,
        program: ContextId,
    },
    ProgramIdentityMismatch {
        context: ContextId,
        expected_source: SourceId,
        actual_source: SourceId,
        expected_run: RunId,
        actual_run: RunId,
        expected_profile: &'static str,
        actual_profile: &'static str,
    },
    ProgramForNonQualifiedContext(ContextId),
    MissingContext {
        child: ContextId,
        parent: ContextId,
    },
    FutureContext {
        child: ContextId,
        parent: ContextId,
    },
    SelfDependency(ContextId),
    Cycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlockingOutcome {
    Invalid,
    Unsupported,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConsumerOutcome {
    Complete(Vec<(MemberId, Specificity)>),
    Blocked(BlockingOutcome),
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ConsumerBudget {
    pub(super) limit: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConsumerBudgetState {
    // Validation-only work units prove bounded ownership; they do not select
    // future production accounting granularity or representation.
    limit: usize,
    used: usize,
}

impl ConsumerBudgetState {
    fn new(budget: ConsumerBudget) -> Self {
        Self {
            limit: budget.limit,
            used: 0,
        }
    }

    fn charge(&mut self) -> bool {
        if self.used >= self.limit {
            return false;
        }
        self.used += 1;
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ConsumerResult {
    pub(super) outcome: ConsumerOutcome,
    pub(super) steps: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConsumerRunCompletion {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedRun {
    completion: ConsumerRunCompletion,
    known_contexts: Option<BTreeMap<ContextId, usize>>,
    // A refused run retains only its deterministic evaluated prefix. An
    // absent suffix was never granted consumer work and cannot be Complete.
    results: Vec<(ContextId, ConsumerResult)>,
    dependencies: BTreeMap<ContextId, DependencyStatus>,
    used: usize,
}

impl ResolvedRun {
    pub(super) fn result(&self, context: ContextId) -> Option<&ConsumerResult> {
        self.results
            .iter()
            .find_map(|(candidate, result)| (*candidate == context).then_some(result))
    }

    pub(super) fn dependency(&self, context: ContextId) -> Option<DependencyStatus> {
        self.dependencies.get(&context).copied()
    }

    pub(super) fn completion(&self) -> ConsumerRunCompletion {
        self.completion
    }

    pub(super) fn used(&self) -> usize {
        self.used
    }

    pub(super) fn outcome(&self, context: ContextId) -> Option<ConsumerOutcome> {
        if let Some(result) = self.result(context) {
            return Some(result.outcome.clone());
        }
        (self.completion == ConsumerRunCompletion::Incomplete
            && self
                .known_contexts
                .as_ref()
                .is_some_and(|contexts| contexts.contains_key(&context)))
        .then_some(ConsumerOutcome::Incomplete)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerKind {
    Root,
    Function(FunctionKind, UnitId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Container {
    kind: ContainerKind,
    current: Option<(MemberId, Specificity)>,
    completed: Vec<(MemberId, Specificity)>,
}

impl Container {
    fn root() -> Self {
        Self {
            kind: ContainerKind::Root,
            current: None,
            completed: Vec::new(),
        }
    }

    fn function(kind: FunctionKind, unit: UnitId) -> Self {
        Self {
            kind: ContainerKind::Function(kind, unit),
            current: None,
            completed: Vec::new(),
        }
    }
}

fn simple_specificity(kind: SimpleKind) -> Specificity {
    match kind {
        SimpleKind::Id => Specificity { a: 1, b: 0, c: 0 },
        SimpleKind::Class | SimpleKind::Attribute | SimpleKind::IdentifierPseudoClass => {
            Specificity { a: 0, b: 1, c: 0 }
        }
        SimpleKind::Type => Specificity { a: 0, b: 0, c: 1 },
        SimpleKind::Universal => Specificity::ZERO,
    }
}

fn max_specificity(
    values: &[(MemberId, Specificity)],
    budget: &mut ConsumerBudgetState,
) -> Result<Specificity, ConsumerOutcome> {
    let mut maximum = Specificity::ZERO;
    for (_, value) in values {
        if !budget.charge() {
            return Err(ConsumerOutcome::Incomplete);
        }
        maximum = maximum.max(*value);
    }
    Ok(maximum)
}

fn relationship_specificity(
    target: RelationshipTarget,
    dependencies: &BTreeMap<ContextId, DependencyStatus>,
    budget: &mut ConsumerBudgetState,
) -> Result<Specificity, ConsumerOutcome> {
    match target {
        RelationshipTarget::ScopeRoot(_) | RelationshipTarget::Zero => Ok(Specificity::ZERO),
        RelationshipTarget::ParentSelectorList(context) => {
            if !budget.charge() {
                return Err(ConsumerOutcome::Incomplete);
            }
            match dependencies.get(&context) {
                Some(DependencyStatus::Resolved(value)) => Ok(*value),
                Some(DependencyStatus::Invalid) => {
                    Err(ConsumerOutcome::Blocked(BlockingOutcome::Invalid))
                }
                Some(DependencyStatus::Unsupported) => {
                    Err(ConsumerOutcome::Blocked(BlockingOutcome::Unsupported))
                }
                Some(DependencyStatus::Indeterminate) => {
                    Err(ConsumerOutcome::Blocked(BlockingOutcome::Indeterminate))
                }
                Some(DependencyStatus::Incomplete) | None => Err(ConsumerOutcome::Incomplete),
            }
        }
    }
}

fn add_to_current(containers: &mut [Container], value: Specificity) -> Result<(), ConsumerOutcome> {
    let Some(container) = containers.last_mut() else {
        return Err(ConsumerOutcome::Incomplete);
    };
    let Some((_, current)) = container.current.as_mut() else {
        return Err(ConsumerOutcome::Incomplete);
    };
    *current = current.add(value);
    Ok(())
}

fn fold_program_with_dependencies(
    program: &GoldProgram,
    dependencies: &BTreeMap<ContextId, DependencyStatus>,
    budget: &mut ConsumerBudgetState,
) -> ConsumerResult {
    let mut containers = vec![Container::root()];
    let started_at = budget.used;

    for fact in &program.facts {
        if !budget.charge() {
            return ConsumerResult {
                outcome: ConsumerOutcome::Incomplete,
                steps: budget.used - started_at,
            };
        }

        let result = match *fact {
            SelectorFact::OpenMember { member, .. } => {
                let Some(container) = containers.last_mut() else {
                    return ConsumerResult {
                        outcome: ConsumerOutcome::Incomplete,
                        steps: budget.used - started_at,
                    };
                };
                if container.current.is_some() {
                    Err(ConsumerOutcome::Incomplete)
                } else {
                    container.current = Some((member, Specificity::ZERO));
                    Ok(())
                }
            }
            SelectorFact::CloseMember { member } => {
                let Some(container) = containers.last_mut() else {
                    return ConsumerResult {
                        outcome: ConsumerOutcome::Incomplete,
                        steps: budget.used - started_at,
                    };
                };
                match container.current.take() {
                    Some((current_member, value)) if current_member == member => {
                        container.completed.push((current_member, value));
                        Ok(())
                    }
                    other => {
                        container.current = other;
                        Err(ConsumerOutcome::Incomplete)
                    }
                }
            }
            SelectorFact::RejectedForgivingMember { .. } => {
                if containers.len() > 1
                    && containers
                        .last()
                        .is_some_and(|container| container.current.is_none())
                {
                    Ok(())
                } else {
                    Err(ConsumerOutcome::Incomplete)
                }
            }
            SelectorFact::Simple { kind, .. } => {
                add_to_current(&mut containers, simple_specificity(kind))
            }
            SelectorFact::OpenFunction { unit, kind, .. } => {
                if containers
                    .last()
                    .and_then(|container| container.current)
                    .is_none()
                {
                    Err(ConsumerOutcome::Incomplete)
                } else {
                    containers.push(Container::function(kind, unit));
                    Ok(())
                }
            }
            SelectorFact::CloseFunction { unit } => {
                if containers.len() <= 1 {
                    Err(ConsumerOutcome::Incomplete)
                } else {
                    let Some(function) = containers.pop() else {
                        return ConsumerResult {
                            outcome: ConsumerOutcome::Incomplete,
                            steps: budget.used - started_at,
                        };
                    };
                    match function.kind {
                        ContainerKind::Function(kind, open_unit)
                            if open_unit == unit && function.current.is_none() =>
                        {
                            let contribution = match kind {
                                FunctionKind::Where => Specificity::ZERO,
                                FunctionKind::Is | FunctionKind::Not | FunctionKind::Has => {
                                    match max_specificity(&function.completed, budget) {
                                        Ok(value) => value,
                                        Err(outcome) => {
                                            return ConsumerResult {
                                                outcome,
                                                steps: budget.used - started_at,
                                            };
                                        }
                                    }
                                }
                            };
                            add_to_current(&mut containers, contribution)
                        }
                        _ => Err(ConsumerOutcome::Incomplete),
                    }
                }
            }
            SelectorFact::NestingPresence { .. } => Ok(()),
            SelectorFact::Relationship { target, .. } => {
                match relationship_specificity(target, dependencies, budget) {
                    Ok(value) => add_to_current(&mut containers, value),
                    Err(outcome) => Err(outcome),
                }
            }
        };

        if let Err(outcome) = result {
            return ConsumerResult {
                outcome,
                steps: budget.used - started_at,
            };
        }
    }

    if containers.len() != 1 || containers[0].current.is_some() {
        return ConsumerResult {
            outcome: ConsumerOutcome::Incomplete,
            steps: budget.used - started_at,
        };
    }

    ConsumerResult {
        outcome: ConsumerOutcome::Complete(containers.pop().expect("root exists").completed),
        steps: budget.used - started_at,
    }
}

pub(super) fn fold_program(program: &GoldProgram, budget: ConsumerBudget) -> ConsumerResult {
    let mut budget = ConsumerBudgetState::new(budget);
    fold_program_with_dependencies(program, &BTreeMap::new(), &mut budget)
}

pub(super) fn fold_observation(
    observation: &GoldObservation,
    budget: ConsumerBudget,
) -> ConsumerResult {
    let mut budget = ConsumerBudgetState::new(budget);
    fold_observation_with_dependencies(observation, &BTreeMap::new(), &mut budget)
}

fn fold_observation_with_dependencies(
    observation: &GoldObservation,
    dependencies: &BTreeMap<ContextId, DependencyStatus>,
    budget: &mut ConsumerBudgetState,
) -> ConsumerResult {
    let started_at = budget.used;
    if !budget.charge() {
        return ConsumerResult {
            outcome: ConsumerOutcome::Incomplete,
            steps: 0,
        };
    }
    if observation.completion == CompletionState::Incomplete {
        return ConsumerResult {
            outcome: ConsumerOutcome::Incomplete,
            steps: budget.used - started_at,
        };
    }

    let outcome = match observation.outcome {
        GoldOutcome::Qualified => match observation.program.as_ref() {
            Some(program) => fold_program_with_dependencies(program, dependencies, budget).outcome,
            None => ConsumerOutcome::Incomplete,
        },
        GoldOutcome::Invalid(_) => ConsumerOutcome::Blocked(BlockingOutcome::Invalid),
        GoldOutcome::Unsupported(_) => ConsumerOutcome::Blocked(BlockingOutcome::Unsupported),
        GoldOutcome::Indeterminate(_) => ConsumerOutcome::Blocked(BlockingOutcome::Indeterminate),
    };
    ConsumerResult {
        outcome,
        steps: budget.used - started_at,
    }
}

pub(super) fn resolve_retained_run(
    run: &GoldRun,
    budget: ConsumerBudget,
) -> Result<ResolvedRun, DependencyResolutionError> {
    let mut budget = ConsumerBudgetState::new(budget);
    let mut dependencies = BTreeMap::new();
    let mut results = Vec::new();

    if run.upstream == CompletionState::Incomplete || run.qualifier == CompletionState::Incomplete {
        return Ok(incomplete_resolved_run(
            &budget,
            None,
            results,
            dependencies,
        ));
    }

    let positions = match observation_positions(&run.observations, &mut budget) {
        Ok(positions) => positions,
        Err(PreparationFailure::BudgetExhausted) => {
            return Ok(incomplete_resolved_run(
                &budget,
                None,
                results,
                dependencies,
            ));
        }
        Err(PreparationFailure::Dependency(error)) => return Err(error),
    };
    let edges = match relationship_edges(&run.observations, &mut budget) {
        Ok(edges) => edges,
        Err(PreparationFailure::BudgetExhausted) => {
            return Ok(incomplete_resolved_run(
                &budget,
                Some(positions),
                results,
                dependencies,
            ));
        }
        Err(PreparationFailure::Dependency(error)) => return Err(error),
    };
    match validate_relationship_edges(&positions, &edges, &mut budget) {
        Ok(()) => {}
        Err(PreparationFailure::BudgetExhausted) => {
            return Ok(incomplete_resolved_run(
                &budget,
                Some(positions),
                results,
                dependencies,
            ));
        }
        Err(PreparationFailure::Dependency(error)) => return Err(error),
    }

    for observation in &run.observations {
        let result = fold_observation_with_dependencies(observation, &dependencies, &mut budget);
        let result_incomplete = result.outcome == ConsumerOutcome::Incomplete;
        let dependency = match dependency_from_result(&result, &mut budget) {
            Ok(dependency) => dependency,
            Err(ConsumerOutcome::Incomplete) => DependencyStatus::Incomplete,
            Err(_) => unreachable!("dependency aggregation only refuses for consumer budget"),
        };
        let dependency_incomplete = dependency == DependencyStatus::Incomplete;
        dependencies.insert(observation.context, dependency);
        results.push((observation.context, result));
        if result_incomplete || dependency_incomplete {
            return Ok(incomplete_resolved_run(
                &budget,
                Some(positions),
                results,
                dependencies,
            ));
        }
    }

    Ok(ResolvedRun {
        completion: ConsumerRunCompletion::Complete,
        known_contexts: Some(positions),
        results,
        dependencies,
        used: budget.used,
    })
}

fn incomplete_resolved_run(
    budget: &ConsumerBudgetState,
    known_contexts: Option<BTreeMap<ContextId, usize>>,
    results: Vec<(ContextId, ConsumerResult)>,
    dependencies: BTreeMap<ContextId, DependencyStatus>,
) -> ResolvedRun {
    ResolvedRun {
        completion: ConsumerRunCompletion::Incomplete,
        known_contexts,
        results,
        dependencies,
        used: budget.used,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PreparationFailure {
    BudgetExhausted,
    Dependency(DependencyResolutionError),
}

fn observation_positions(
    observations: &[GoldObservation],
    budget: &mut ConsumerBudgetState,
) -> Result<BTreeMap<ContextId, usize>, PreparationFailure> {
    let mut positions = BTreeMap::new();
    // The first qualified retained program establishes the validation run's
    // source/run/profile identity domain. ContextId remains run-local only.
    let mut identity_domain: Option<(SourceId, RunId, &'static str)> = None;
    for (position, observation) in observations.iter().enumerate() {
        if !budget.charge() {
            return Err(PreparationFailure::BudgetExhausted);
        }
        if positions.insert(observation.context, position).is_some() {
            return Err(PreparationFailure::Dependency(
                DependencyResolutionError::DuplicateContext(observation.context),
            ));
        }

        match (observation.outcome, observation.program.as_ref()) {
            (GoldOutcome::Qualified, Some(program)) if program.context == observation.context => {
                match identity_domain {
                    None => {
                        identity_domain = Some((program.source, program.run, program.profile));
                    }
                    Some((expected_source, expected_run, expected_profile))
                        if program.source != expected_source
                            || program.run != expected_run
                            || program.profile != expected_profile =>
                    {
                        return Err(PreparationFailure::Dependency(
                            DependencyResolutionError::ProgramIdentityMismatch {
                                context: observation.context,
                                expected_source,
                                actual_source: program.source,
                                expected_run,
                                actual_run: program.run,
                                expected_profile,
                                actual_profile: program.profile,
                            },
                        ));
                    }
                    Some(_) => {}
                }
            }
            (GoldOutcome::Qualified, Some(program)) => {
                return Err(PreparationFailure::Dependency(
                    DependencyResolutionError::ProgramContextMismatch {
                        observation: observation.context,
                        program: program.context,
                    },
                ));
            }
            (GoldOutcome::Qualified, None) => {
                return Err(PreparationFailure::Dependency(
                    DependencyResolutionError::QualifiedWithoutProgram(observation.context),
                ));
            }
            (_, Some(_)) => {
                return Err(PreparationFailure::Dependency(
                    DependencyResolutionError::ProgramForNonQualifiedContext(observation.context),
                ));
            }
            (_, None) => {}
        }
    }
    Ok(positions)
}

fn relationship_edges(
    observations: &[GoldObservation],
    budget: &mut ConsumerBudgetState,
) -> Result<Vec<(ContextId, ContextId)>, PreparationFailure> {
    let mut edges = Vec::new();
    for observation in observations {
        let Some(program) = observation.program.as_ref() else {
            continue;
        };
        for fact in &program.facts {
            if !budget.charge() {
                return Err(PreparationFailure::BudgetExhausted);
            }
            if let SelectorFact::Relationship {
                target: RelationshipTarget::ParentSelectorList(parent),
                ..
            } = fact
            {
                edges.push((observation.context, *parent));
            }
        }
    }
    Ok(edges)
}

fn validate_relationship_edges(
    positions: &BTreeMap<ContextId, usize>,
    edges: &[(ContextId, ContextId)],
    budget: &mut ConsumerBudgetState,
) -> Result<(), PreparationFailure> {
    for (child, parent) in edges {
        if !budget.charge() {
            return Err(PreparationFailure::BudgetExhausted);
        }
        if child == parent {
            return Err(PreparationFailure::Dependency(
                DependencyResolutionError::SelfDependency(*child),
            ));
        }
        if !positions.contains_key(parent) {
            return Err(PreparationFailure::Dependency(
                DependencyResolutionError::MissingContext {
                    child: *child,
                    parent: *parent,
                },
            ));
        }
    }

    if !dependency_graph_is_acyclic(edges, budget)? {
        return Err(PreparationFailure::Dependency(
            DependencyResolutionError::Cycle,
        ));
    }

    for (child, parent) in edges {
        if !budget.charge() {
            return Err(PreparationFailure::BudgetExhausted);
        }
        if positions[parent] >= positions[child] {
            return Err(PreparationFailure::Dependency(
                DependencyResolutionError::FutureContext {
                    child: *child,
                    parent: *parent,
                },
            ));
        }
    }
    Ok(())
}

fn dependency_from_result(
    result: &ConsumerResult,
    budget: &mut ConsumerBudgetState,
) -> Result<DependencyStatus, ConsumerOutcome> {
    match &result.outcome {
        ConsumerOutcome::Complete(members) => {
            max_specificity(members, budget).map(DependencyStatus::Resolved)
        }
        ConsumerOutcome::Blocked(BlockingOutcome::Invalid) => Ok(DependencyStatus::Invalid),
        ConsumerOutcome::Blocked(BlockingOutcome::Unsupported) => Ok(DependencyStatus::Unsupported),
        ConsumerOutcome::Blocked(BlockingOutcome::Indeterminate) => {
            Ok(DependencyStatus::Indeterminate)
        }
        ConsumerOutcome::Incomplete => Ok(DependencyStatus::Incomplete),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RetentionBudget {
    pub(super) limit: usize,
    pub(super) used: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RetentionRefusal {
    pub(super) required: usize,
    pub(super) remaining: usize,
}

pub(super) fn commit_observation(
    committed: &mut Vec<GoldObservation>,
    observation: GoldObservation,
    budget: &mut RetentionBudget,
) -> Result<(), RetentionRefusal> {
    let required = observation
        .program
        .as_ref()
        .map_or(0usize, |program| program.facts.len());
    let remaining = budget.limit.saturating_sub(budget.used);
    if required > remaining {
        return Err(RetentionRefusal {
            required,
            remaining,
        });
    }

    budget.used += required;
    committed.push(observation);
    Ok(())
}

fn dependency_graph_is_acyclic(
    edges: &[(ContextId, ContextId)],
    budget: &mut ConsumerBudgetState,
) -> Result<bool, PreparationFailure> {
    fn visit(
        node: ContextId,
        adjacency: &BTreeMap<ContextId, Vec<ContextId>>,
        visiting: &mut BTreeSet<ContextId>,
        visited: &mut BTreeSet<ContextId>,
        budget: &mut ConsumerBudgetState,
    ) -> Result<bool, PreparationFailure> {
        if !budget.charge() {
            return Err(PreparationFailure::BudgetExhausted);
        }
        if visited.contains(&node) {
            return Ok(true);
        }
        if !visiting.insert(node) {
            return Ok(false);
        }
        if let Some(parents) = adjacency.get(&node) {
            for parent in parents {
                if !budget.charge() {
                    return Err(PreparationFailure::BudgetExhausted);
                }
                if !visit(*parent, adjacency, visiting, visited, budget)? {
                    return Ok(false);
                }
            }
        }
        visiting.remove(&node);
        visited.insert(node);
        Ok(true)
    }

    let mut adjacency = BTreeMap::<ContextId, Vec<ContextId>>::new();
    for (child, parent) in edges {
        if !budget.charge() {
            return Err(PreparationFailure::BudgetExhausted);
        }
        adjacency.entry(*child).or_default().push(*parent);
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for node in adjacency.keys().copied() {
        if !visit(node, &adjacency, &mut visiting, &mut visited, budget)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[test]
fn retained_parent_dependency_rejects_cross_source_run_or_profile_identity() {
    fn program_with_identity(
        source: u32,
        run: u32,
        profile: &'static str,
        context: u32,
        facts: Vec<SelectorFact>,
    ) -> GoldProgram {
        GoldProgram {
            source: SourceId(source),
            run: RunId(run),
            profile,
            context: ContextId(context),
            facts,
        }
    }

    fn qualified_program(program: GoldProgram) -> GoldObservation {
        GoldObservation {
            context: program.context,
            completion: CompletionState::Complete,
            outcome: GoldOutcome::Qualified,
            program: Some(program),
        }
    }

    fn parent() -> GoldObservation {
        qualified_program(program_with_identity(
            1,
            1,
            "CoreV1",
            1,
            vec![
                SelectorFact::OpenMember {
                    member: MemberId(1),
                    range: AuthoredRange::new(0, 1),
                },
                SelectorFact::CloseMember {
                    member: MemberId(1),
                },
            ],
        ))
    }

    fn child(source: u32, run: u32, profile: &'static str) -> GoldObservation {
        qualified_program(program_with_identity(
            source,
            run,
            profile,
            2,
            vec![
                SelectorFact::OpenMember {
                    member: MemberId(1),
                    range: AuthoredRange::new(0, 1),
                },
                SelectorFact::Relationship {
                    target: RelationshipTarget::ParentSelectorList(ContextId(1)),
                    origin: RelationshipOrigin::Derived,
                },
                SelectorFact::CloseMember {
                    member: MemberId(1),
                },
            ],
        ))
    }

    fn retained(parent: GoldObservation, child: GoldObservation) -> GoldRun {
        GoldRun {
            upstream: CompletionState::Complete,
            qualifier: CompletionState::Complete,
            observations: vec![parent, child],
        }
    }

    let valid = resolve_retained_run(
        &retained(parent(), child(1, 1, "CoreV1")),
        ConsumerBudget { limit: usize::MAX },
    )
    .expect("one retained source/run/profile identity domain is valid");
    assert_eq!(valid.completion(), ConsumerRunCompletion::Complete);

    assert_eq!(
        resolve_retained_run(
            &retained(parent(), child(9, 1, "CoreV1")),
            ConsumerBudget { limit: usize::MAX },
        ),
        Err(DependencyResolutionError::ProgramIdentityMismatch {
            context: ContextId(2),
            expected_source: SourceId(1),
            actual_source: SourceId(9),
            expected_run: RunId(1),
            actual_run: RunId(1),
            expected_profile: "CoreV1",
            actual_profile: "CoreV1",
        })
    );

    assert_eq!(
        resolve_retained_run(
            &retained(parent(), child(1, 9, "CoreV1")),
            ConsumerBudget { limit: usize::MAX },
        ),
        Err(DependencyResolutionError::ProgramIdentityMismatch {
            context: ContextId(2),
            expected_source: SourceId(1),
            actual_source: SourceId(1),
            expected_run: RunId(1),
            actual_run: RunId(9),
            expected_profile: "CoreV1",
            actual_profile: "CoreV1",
        })
    );

    assert_eq!(
        resolve_retained_run(
            &retained(parent(), child(1, 1, "OtherProfile")),
            ConsumerBudget { limit: usize::MAX },
        ),
        Err(DependencyResolutionError::ProgramIdentityMismatch {
            context: ContextId(2),
            expected_source: SourceId(1),
            actual_source: SourceId(1),
            expected_run: RunId(1),
            actual_run: RunId(1),
            expected_profile: "CoreV1",
            actual_profile: "OtherProfile",
        })
    );
}
