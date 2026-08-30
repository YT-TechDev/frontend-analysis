//! Source-free reference fold for #402 semantic-handoff validation.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::selector_semantic_handoff_gold::{
    CompletionState, ContextId, FunctionKind, GoldObservation, GoldOutcome, GoldProgram, GoldRun,
    MemberId, RelationshipTarget, SelectorFact, SimpleKind, UnitId,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ConsumerResult {
    pub(super) outcome: ConsumerOutcome,
    pub(super) steps: usize,
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

fn max_specificity(values: &[(MemberId, Specificity)]) -> Specificity {
    values
        .iter()
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(Specificity::ZERO)
}

fn relationship_specificity(
    target: RelationshipTarget,
    dependencies: &BTreeMap<ContextId, DependencyStatus>,
) -> Result<Specificity, ConsumerOutcome> {
    match target {
        RelationshipTarget::ScopeRoot(_) | RelationshipTarget::Zero => Ok(Specificity::ZERO),
        RelationshipTarget::ParentSelectorList(context) => match dependencies.get(&context) {
            Some(DependencyStatus::Resolved(value)) => Ok(*value),
            Some(DependencyStatus::Invalid) => Err(ConsumerOutcome::Blocked(BlockingOutcome::Invalid)),
            Some(DependencyStatus::Unsupported) => {
                Err(ConsumerOutcome::Blocked(BlockingOutcome::Unsupported))
            }
            Some(DependencyStatus::Indeterminate) => {
                Err(ConsumerOutcome::Blocked(BlockingOutcome::Indeterminate))
            }
            Some(DependencyStatus::Incomplete) | None => Err(ConsumerOutcome::Incomplete),
        },
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

pub(super) fn fold_program(
    program: &GoldProgram,
    dependencies: &BTreeMap<ContextId, DependencyStatus>,
    budget: ConsumerBudget,
) -> ConsumerResult {
    let mut containers = vec![Container::root()];
    let mut steps = 0usize;

    for fact in &program.facts {
        if steps == budget.limit {
            return ConsumerResult {
                outcome: ConsumerOutcome::Incomplete,
                steps,
            };
        }
        steps += 1;

        let result = match *fact {
            SelectorFact::OpenMember { member, .. } => {
                let Some(container) = containers.last_mut() else {
                    return ConsumerResult {
                        outcome: ConsumerOutcome::Incomplete,
                        steps,
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
                        steps,
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
            SelectorFact::Simple { kind, .. } => add_to_current(&mut containers, simple_specificity(kind)),
            SelectorFact::OpenFunction { unit, kind, .. } => {
                if containers.last().and_then(|container| container.current).is_none() {
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
                            steps,
                        };
                    };
                    match function.kind {
                        ContainerKind::Function(kind, open_unit)
                            if open_unit == unit && function.current.is_none() =>
                        {
                            let contribution = match kind {
                                FunctionKind::Where => Specificity::ZERO,
                                FunctionKind::Is | FunctionKind::Not | FunctionKind::Has => {
                                    max_specificity(&function.completed)
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
                match relationship_specificity(target, dependencies) {
                    Ok(value) => add_to_current(&mut containers, value),
                    Err(outcome) => Err(outcome),
                }
            }
        };

        if let Err(outcome) = result {
            return ConsumerResult { outcome, steps };
        }
    }

    if containers.len() != 1 || containers[0].current.is_some() {
        return ConsumerResult {
            outcome: ConsumerOutcome::Incomplete,
            steps,
        };
    }

    ConsumerResult {
        outcome: ConsumerOutcome::Complete(containers.pop().expect("root exists").completed),
        steps,
    }
}

pub(super) fn fold_observation(
    observation: &GoldObservation,
    dependencies: &BTreeMap<ContextId, DependencyStatus>,
    budget: ConsumerBudget,
) -> ConsumerResult {
    match observation.outcome {
        GoldOutcome::Qualified => match observation.program.as_ref() {
            Some(program) => fold_program(program, dependencies, budget),
            None => ConsumerResult {
                outcome: ConsumerOutcome::Incomplete,
                steps: 0,
            },
        },
        GoldOutcome::Invalid => ConsumerResult {
            outcome: ConsumerOutcome::Blocked(BlockingOutcome::Invalid),
            steps: 0,
        },
        GoldOutcome::Unsupported => ConsumerResult {
            outcome: ConsumerOutcome::Blocked(BlockingOutcome::Unsupported),
            steps: 0,
        },
        GoldOutcome::Indeterminate => ConsumerResult {
            outcome: ConsumerOutcome::Blocked(BlockingOutcome::Indeterminate),
            steps: 0,
        },
    }
}

pub(super) fn fold_run(
    run: &GoldRun,
    dependencies: &BTreeMap<ContextId, DependencyStatus>,
    budget: ConsumerBudget,
) -> Vec<ConsumerResult> {
    if run.upstream == CompletionState::Incomplete || run.qualifier == CompletionState::Incomplete {
        return run
            .observations
            .iter()
            .map(|_| ConsumerResult {
                outcome: ConsumerOutcome::Incomplete,
                steps: 0,
            })
            .collect();
    }

    run.observations
        .iter()
        .map(|observation| fold_observation(observation, dependencies, budget))
        .collect()
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

pub(super) fn dependency_graph_is_acyclic(edges: &[(ContextId, ContextId)]) -> bool {
    fn visit(
        node: ContextId,
        adjacency: &BTreeMap<ContextId, Vec<ContextId>>,
        visiting: &mut BTreeSet<ContextId>,
        visited: &mut BTreeSet<ContextId>,
    ) -> bool {
        if visited.contains(&node) {
            return true;
        }
        if !visiting.insert(node) {
            return false;
        }
        if let Some(parents) = adjacency.get(&node) {
            for parent in parents {
                if !visit(*parent, adjacency, visiting, visited) {
                    return false;
                }
            }
        }
        visiting.remove(&node);
        visited.insert(node);
        true
    }

    let mut adjacency = BTreeMap::<ContextId, Vec<ContextId>>::new();
    for (child, parent) in edges {
        adjacency.entry(*child).or_default().push(*parent);
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    adjacency
        .keys()
        .copied()
        .all(|node| visit(node, &adjacency, &mut visiting, &mut visited))
}
