//! Source-free reference consumer and sidecar lifecycle model.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use super::specificity_input_gold::{
    GoldCandidate, GoldCandidateDisposition, GoldContextId, GoldInstruction, GoldMaxKind,
    GoldQualifierSnapshot, GoldRelationshipOrigin, GoldRelationshipTarget, GoldSimpleKind,
    GoldSpecificity, SidecarCandidatePlan, SidecarCollection, SidecarCompletion, SidecarEvent,
    SidecarFailure, SidecarLimits, SidecarResource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ReferenceOutcome {
    Known(Vec<GoldSpecificity>),
    BlockedOnParent(GoldContextId),
    DeferredByNormativeAmbiguity,
    InvalidProgram,
    ArithmeticOverflow,
    Cycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerKind {
    Root,
    Max(GoldMaxKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Container {
    kind: ContainerKind,
    current: Option<GoldSpecificity>,
    current_has_input: bool,
    current_has_derived_relationship: bool,
    completed: Vec<GoldSpecificity>,
}

impl Container {
    fn root() -> Self {
        Self {
            kind: ContainerKind::Root,
            current: None,
            current_has_input: false,
            current_has_derived_relationship: false,
            completed: Vec::new(),
        }
    }

    fn max(kind: GoldMaxKind) -> Self {
        Self {
            kind: ContainerKind::Max(kind),
            current: None,
            current_has_input: false,
            current_has_derived_relationship: false,
            completed: Vec::new(),
        }
    }
}

fn simple_specificity(kind: GoldSimpleKind) -> GoldSpecificity {
    match kind {
        GoldSimpleKind::Id => GoldSpecificity::new(1, 0, 0),
        GoldSimpleKind::Class
        | GoldSimpleKind::Attribute
        | GoldSimpleKind::IdentifierPseudoClass => GoldSpecificity::new(0, 1, 0),
        GoldSimpleKind::Type => GoldSpecificity::new(0, 0, 1),
        GoldSimpleKind::Universal => GoldSpecificity::ZERO,
    }
}

fn checked_add(
    left: GoldSpecificity,
    right: GoldSpecificity,
) -> Result<GoldSpecificity, ReferenceOutcome> {
    Ok(GoldSpecificity {
        a: left
            .a
            .checked_add(right.a)
            .ok_or(ReferenceOutcome::ArithmeticOverflow)?,
        b: left
            .b
            .checked_add(right.b)
            .ok_or(ReferenceOutcome::ArithmeticOverflow)?,
        c: left
            .c
            .checked_add(right.c)
            .ok_or(ReferenceOutcome::ArithmeticOverflow)?,
    })
}

fn add_to_current(
    containers: &mut [Container],
    contribution: GoldSpecificity,
) -> Result<(), ReferenceOutcome> {
    let Some(container) = containers.last_mut() else {
        return Err(ReferenceOutcome::InvalidProgram);
    };
    let Some(current) = container.current.as_mut() else {
        return Err(ReferenceOutcome::InvalidProgram);
    };
    *current = checked_add(*current, contribution)?;
    container.current_has_input = true;
    Ok(())
}

fn mark_derived_root_relationship(containers: &mut [Container]) -> Result<(), ReferenceOutcome> {
    if containers.len() != 1 {
        return Err(ReferenceOutcome::InvalidProgram);
    }
    let Some(root) = containers.last_mut() else {
        return Err(ReferenceOutcome::InvalidProgram);
    };
    if root.kind != ContainerKind::Root
        || root.current.is_none()
        || root.current_has_derived_relationship
    {
        return Err(ReferenceOutcome::InvalidProgram);
    }
    root.current_has_derived_relationship = true;
    Ok(())
}

#[derive(Debug, Clone)]
struct Resolver {
    programs: BTreeMap<GoldContextId, super::specificity_input_gold::GoldProgram>,
    deferred: BTreeSet<GoldContextId>,
    cache: BTreeMap<GoldContextId, Vec<GoldSpecificity>>,
    visiting: BTreeSet<GoldContextId>,
}

impl Resolver {
    fn new(candidates: &[GoldCandidate]) -> Result<Self, ReferenceOutcome> {
        let mut programs = BTreeMap::new();
        let mut deferred = BTreeSet::new();
        for candidate in candidates {
            match &candidate.disposition {
                GoldCandidateDisposition::Program(program) => {
                    if program.owning_context != candidate.context
                        || programs
                            .insert(candidate.context, program.clone())
                            .is_some()
                        || deferred.contains(&candidate.context)
                    {
                        return Err(ReferenceOutcome::InvalidProgram);
                    }
                }
                GoldCandidateDisposition::DeferredByNormativeAmbiguity => {
                    if programs.contains_key(&candidate.context)
                        || !deferred.insert(candidate.context)
                    {
                        return Err(ReferenceOutcome::InvalidProgram);
                    }
                }
            }
        }
        Ok(Self {
            programs,
            deferred,
            cache: BTreeMap::new(),
            visiting: BTreeSet::new(),
        })
    }

    fn resolve_context(&mut self, context: GoldContextId) -> ReferenceOutcome {
        if self.deferred.contains(&context) {
            return ReferenceOutcome::DeferredByNormativeAmbiguity;
        }
        if let Some(cached) = self.cache.get(&context) {
            return ReferenceOutcome::Known(cached.clone());
        }
        let Some(program) = self.programs.get(&context).cloned() else {
            return ReferenceOutcome::BlockedOnParent(context);
        };
        if !self.visiting.insert(context) {
            return ReferenceOutcome::Cycle;
        }
        let outcome = self.fold_program(&program);
        self.visiting.remove(&context);
        if let ReferenceOutcome::Known(values) = &outcome {
            self.cache.insert(context, values.clone());
        }
        outcome
    }

    fn parent_specificity(
        &mut self,
        context: GoldContextId,
    ) -> Result<GoldSpecificity, ReferenceOutcome> {
        match self.resolve_context(context) {
            ReferenceOutcome::Known(values) => values
                .into_iter()
                .max()
                .ok_or(ReferenceOutcome::InvalidProgram),
            other => Err(other),
        }
    }

    fn fold_program(
        &mut self,
        program: &super::specificity_input_gold::GoldProgram,
    ) -> ReferenceOutcome {
        let mut containers = vec![Container::root()];

        for instruction in &program.instructions {
            let result = match *instruction {
                GoldInstruction::BeginMember => {
                    let Some(container) = containers.last_mut() else {
                        return ReferenceOutcome::InvalidProgram;
                    };
                    if container.current.is_some() {
                        Err(ReferenceOutcome::InvalidProgram)
                    } else {
                        container.current = Some(GoldSpecificity::ZERO);
                        container.current_has_input = false;
                        container.current_has_derived_relationship = false;
                        Ok(())
                    }
                }
                GoldInstruction::EndMember => {
                    let Some(container) = containers.last_mut() else {
                        return ReferenceOutcome::InvalidProgram;
                    };
                    if !container.current_has_input {
                        return ReferenceOutcome::InvalidProgram;
                    }
                    let Some(value) = container.current.take() else {
                        return ReferenceOutcome::InvalidProgram;
                    };
                    container.current_has_input = false;
                    container.current_has_derived_relationship = false;
                    container.completed.push(value);
                    Ok(())
                }
                GoldInstruction::Simple(kind) => {
                    add_to_current(&mut containers, simple_specificity(kind))
                }
                GoldInstruction::BeginMax(kind) => {
                    let nested_has = kind == GoldMaxKind::Has
                        && containers.iter().any(|container| {
                            container.kind == ContainerKind::Max(GoldMaxKind::Has)
                        });
                    if nested_has
                        || containers
                            .last()
                            .and_then(|container| container.current)
                            .is_none()
                    {
                        Err(ReferenceOutcome::InvalidProgram)
                    } else {
                        containers.push(Container::max(kind));
                        Ok(())
                    }
                }
                GoldInstruction::EndMax(kind) => {
                    if containers.len() <= 1 {
                        Err(ReferenceOutcome::InvalidProgram)
                    } else {
                        let Some(container) = containers.pop() else {
                            return ReferenceOutcome::InvalidProgram;
                        };
                        if container.kind != ContainerKind::Max(kind)
                            || container.current.is_some()
                            || (container.completed.is_empty() && kind != GoldMaxKind::Is)
                        {
                            Err(ReferenceOutcome::InvalidProgram)
                        } else {
                            let maximum = container
                                .completed
                                .into_iter()
                                .max()
                                .unwrap_or(GoldSpecificity::ZERO);
                            add_to_current(&mut containers, maximum)
                        }
                    }
                }
                GoldInstruction::WhereZero => {
                    add_to_current(&mut containers, GoldSpecificity::ZERO)
                }
                GoldInstruction::Relationship { target, origin } => {
                    if origin == GoldRelationshipOrigin::Derived
                        && let Err(outcome) = mark_derived_root_relationship(&mut containers)
                    {
                        return outcome;
                    }
                    match target {
                        GoldRelationshipTarget::Zero => {
                            add_to_current(&mut containers, GoldSpecificity::ZERO)
                        }
                        GoldRelationshipTarget::ParentSelectorList(parent) => {
                            if parent >= program.owning_context {
                                return ReferenceOutcome::InvalidProgram;
                            }
                            match self.parent_specificity(parent) {
                                Ok(value) => add_to_current(&mut containers, value),
                                Err(outcome) => return outcome,
                            }
                        }
                    }
                }
            };

            if let Err(outcome) = result {
                return outcome;
            }
        }

        if containers.len() != 1 || containers[0].current.is_some() {
            return ReferenceOutcome::InvalidProgram;
        }
        let Some(root) = containers.pop() else {
            return ReferenceOutcome::InvalidProgram;
        };
        if root.completed.is_empty() {
            return ReferenceOutcome::InvalidProgram;
        }
        ReferenceOutcome::Known(root.completed)
    }
}

pub(super) fn resolve_candidates(
    candidates: &[GoldCandidate],
    target: GoldContextId,
) -> ReferenceOutcome {
    let mut resolver = match Resolver::new(candidates) {
        Ok(resolver) => resolver,
        Err(outcome) => return outcome,
    };
    resolver.resolve_context(target)
}

fn retained_units(candidate: &GoldCandidate) -> Option<usize> {
    1usize.checked_add(match &candidate.disposition {
        GoldCandidateDisposition::Program(program) => program.instructions.len(),
        GoldCandidateDisposition::DeferredByNormativeAmbiguity => 0,
    })
}

pub(super) fn collect_sidecars(
    qualifier: &GoldQualifierSnapshot,
    plans: &[SidecarCandidatePlan],
    limits: SidecarLimits,
) -> SidecarCollection {
    let mut collection = SidecarCollection {
        qualifier: qualifier.clone(),
        completion: SidecarCompletion::Complete,
        committed: Vec::new(),
        preparation_steps: 0,
        retained_input_units: 0,
        failure: None,
        events: Vec::new(),
    };

    for plan in plans {
        let identity_granted = collection.preparation_steps < limits.preparation_steps;
        collection.events.push(SidecarEvent::PreparationPreflight {
            granted: identity_granted,
        });
        if !identity_granted {
            collection.completion = SidecarCompletion::Incomplete;
            collection.failure = Some(SidecarFailure::Resource(SidecarResource::PreparationSteps));
            return collection;
        }
        collection.preparation_steps += 1;
        collection
            .events
            .push(SidecarEvent::CandidateIdentityEstablished {
                context: plan.candidate.context,
            });

        for _ in 0..plan.additional_preparation_mutations {
            let granted = collection.preparation_steps < limits.preparation_steps;
            collection
                .events
                .push(SidecarEvent::PreparationPreflight { granted });
            if !granted {
                collection.completion = SidecarCompletion::Incomplete;
                collection.failure =
                    Some(SidecarFailure::Resource(SidecarResource::PreparationSteps));
                return collection;
            }
            collection.preparation_steps += 1;
            collection.events.push(SidecarEvent::PreparationMutation);
        }

        for _ in 0..plan.ancestry_records_to_inspect {
            let granted = collection.preparation_steps < limits.preparation_steps;
            collection
                .events
                .push(SidecarEvent::AncestryPreflight { granted });
            if !granted {
                collection.completion = SidecarCompletion::Incomplete;
                collection.failure =
                    Some(SidecarFailure::Resource(SidecarResource::PreparationSteps));
                return collection;
            }
            collection.preparation_steps += 1;
            collection.events.push(SidecarEvent::AncestryInspect);
        }

        let Some(required) = retained_units(&plan.candidate) else {
            collection.completion = SidecarCompletion::Incomplete;
            collection.failure = Some(SidecarFailure::ArithmeticOverflow);
            return collection;
        };
        let Some(next_retained) = collection.retained_input_units.checked_add(required) else {
            collection.completion = SidecarCompletion::Incomplete;
            collection.failure = Some(SidecarFailure::ArithmeticOverflow);
            return collection;
        };
        let granted = next_retained <= limits.retained_input_units;
        collection
            .events
            .push(SidecarEvent::RetainedPreflight { required, granted });
        if !granted {
            collection.completion = SidecarCompletion::Incomplete;
            collection.failure = Some(SidecarFailure::Resource(
                SidecarResource::RetainedInputUnits,
            ));
            return collection;
        }

        collection.retained_input_units = next_retained;
        collection.committed.push(plan.candidate.clone());
        collection.events.push(SidecarEvent::Commit {
            context: plan.candidate.context,
        });
    }

    collection
}
