//! Retained selector-semantic handoff contracts (#405).
//!
//! This module owns only the linear, source-backed semantic program retained
//! after the existing `SelectorMachine` has authoritatively recognized the
//! selected `CoreV1` grammar. It is not a selector parser and owns no retained
//! AST/tree, tokenizer access, source search, or recovery policy.

use std::collections::BTreeSet;
use std::fmt;

use crate::SourceAnchor;

use super::super::parser::context::{
    CssParserContextId, CssParserContextKind, CssParserContextRecord, CssParserGroupRuleKind,
};

/// Program-local member identity. Numeric values have no compatibility or
/// persistence meaning and are not required to be contiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CssSelectorSemanticMemberId(usize);

impl CssSelectorSemanticMemberId {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> usize {
        self.0
    }
}

/// Program-local semantic-unit identity. Numeric values have no compatibility
/// or persistence meaning and are not required to be contiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CssSelectorSemanticUnitId(usize);

impl CssSelectorSemanticUnitId {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }

    pub(crate) const fn value(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorSemanticSimpleKind {
    Type,
    Universal,
    Id,
    Class,
    Attribute,
    IdentifierPseudoClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorSemanticFunctionKind {
    Is,
    Where,
    Not,
    Has,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorSemanticRelationshipTarget {
    ParentSelectorList(CssParserContextId),
    ScopeRoot(CssParserContextId),
    Zero,
}

#[derive(Clone)]
pub(crate) enum CssSelectorSemanticRelationshipOrigin {
    Authored(SourceAnchor),
    Derived,
}

impl fmt::Debug for CssSelectorSemanticRelationshipOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authored(anchor) => formatter
                .debug_struct("Authored")
                .field("source_id", &anchor.source_id())
                .field("range", &anchor.range())
                .finish(),
            Self::Derived => formatter.debug_tuple("Derived").finish(),
        }
    }
}

impl PartialEq for CssSelectorSemanticRelationshipOrigin {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Authored(left), Self::Authored(right)) => same_anchor(left, right),
            (Self::Derived, Self::Derived) => true,
            _ => false,
        }
    }
}

impl Eq for CssSelectorSemanticRelationshipOrigin {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorNestingPresenceDisposition {
    Contributing,
    NonContributingPresenceOnly,
}

/// One fact in the retained ordered linear program.
#[derive(Clone)]
pub(crate) enum CssSelectorSemanticFact {
    OpenMember {
        member: CssSelectorSemanticMemberId,
        range: SourceAnchor,
    },
    CloseMember {
        member: CssSelectorSemanticMemberId,
    },
    RejectedForgivingMember {
        member: CssSelectorSemanticMemberId,
        range: SourceAnchor,
    },
    Simple {
        unit: CssSelectorSemanticUnitId,
        kind: CssSelectorSemanticSimpleKind,
        range: SourceAnchor,
    },
    OpenFunction {
        unit: CssSelectorSemanticUnitId,
        kind: CssSelectorSemanticFunctionKind,
        range: SourceAnchor,
    },
    CloseFunction {
        unit: CssSelectorSemanticUnitId,
    },
    NestingPresence {
        member: CssSelectorSemanticMemberId,
        unit: CssSelectorSemanticUnitId,
        origin: CssSelectorSemanticRelationshipOrigin,
        disposition: CssSelectorNestingPresenceDisposition,
    },
    Relationship {
        target: CssSelectorSemanticRelationshipTarget,
        origin: CssSelectorSemanticRelationshipOrigin,
    },
}

impl PartialEq for CssSelectorSemanticFact {
    fn eq(&self, other: &Self) -> bool {
        use CssSelectorSemanticFact as Fact;
        match (self, other) {
            (
                Fact::OpenMember {
                    member: left_member,
                    range: left_range,
                },
                Fact::OpenMember {
                    member: right_member,
                    range: right_range,
                },
            ) => left_member == right_member && same_anchor(left_range, right_range),
            (
                Fact::CloseMember {
                    member: left_member,
                },
                Fact::CloseMember {
                    member: right_member,
                },
            ) => left_member == right_member,
            (
                Fact::RejectedForgivingMember {
                    member: left_member,
                    range: left_range,
                },
                Fact::RejectedForgivingMember {
                    member: right_member,
                    range: right_range,
                },
            ) => left_member == right_member && same_anchor(left_range, right_range),
            (
                Fact::Simple {
                    unit: left_unit,
                    kind: left_kind,
                    range: left_range,
                },
                Fact::Simple {
                    unit: right_unit,
                    kind: right_kind,
                    range: right_range,
                },
            ) => {
                left_unit == right_unit
                    && left_kind == right_kind
                    && same_anchor(left_range, right_range)
            }
            (
                Fact::OpenFunction {
                    unit: left_unit,
                    kind: left_kind,
                    range: left_range,
                },
                Fact::OpenFunction {
                    unit: right_unit,
                    kind: right_kind,
                    range: right_range,
                },
            ) => {
                left_unit == right_unit
                    && left_kind == right_kind
                    && same_anchor(left_range, right_range)
            }
            (
                Fact::CloseFunction { unit: left_unit },
                Fact::CloseFunction { unit: right_unit },
            ) => left_unit == right_unit,
            (
                Fact::NestingPresence {
                    member: left_member,
                    unit: left_unit,
                    origin: left_origin,
                    disposition: left_disposition,
                },
                Fact::NestingPresence {
                    member: right_member,
                    unit: right_unit,
                    origin: right_origin,
                    disposition: right_disposition,
                },
            ) => {
                left_member == right_member
                    && left_unit == right_unit
                    && left_origin == right_origin
                    && left_disposition == right_disposition
            }
            (
                Fact::Relationship {
                    target: left_target,
                    origin: left_origin,
                },
                Fact::Relationship {
                    target: right_target,
                    origin: right_origin,
                },
            ) => left_target == right_target && left_origin == right_origin,
            _ => false,
        }
    }
}

impl Eq for CssSelectorSemanticFact {}

/// Complete semantic program attached to exactly one parser-run-local context.
#[derive(Clone)]
pub(crate) struct CssSelectorSemanticProgram {
    owning_context: CssParserContextId,
    facts: Vec<CssSelectorSemanticFact>,
}

impl CssSelectorSemanticProgram {
    /// Seals an already-complete program staged by the authoritative selector
    /// machine. The supplied header is used only to revalidate already-owned
    /// source anchors; no source discovery occurs here.
    pub(crate) fn from_authoritative_staging(
        owning_context: CssParserContextId,
        context_header: &SourceAnchor,
        facts: Vec<CssSelectorSemanticFact>,
    ) -> Result<Self, CssSelectorHandoffInvariantViolation> {
        let program = Self {
            owning_context,
            facts,
        };
        program.validate_structure()?;
        program.validate_authored_provenance(context_header)?;
        Ok(program)
    }

    pub(crate) const fn owning_context(&self) -> CssParserContextId {
        self.owning_context
    }

    pub(crate) fn facts(&self) -> &[CssSelectorSemanticFact] {
        &self.facts
    }

    pub(crate) const fn fact_count(&self) -> usize {
        self.facts.len()
    }

    pub(crate) fn validate_for_observation(
        &self,
        context: CssParserContextId,
        context_header: &SourceAnchor,
    ) -> Result<(), CssSelectorHandoffInvariantViolation> {
        if self.owning_context != context {
            return Err(CssSelectorHandoffInvariantViolation::OwningContextMismatch {
                expected: context,
                actual: self.owning_context,
            });
        }
        self.validate_structure()?;
        self.validate_authored_provenance(context_header)
    }

    fn validate_structure(&self) -> Result<(), CssSelectorHandoffInvariantViolation> {
        let mut member_stack = Vec::new();
        let mut function_stack = Vec::new();
        let mut seen_members = BTreeSet::new();
        let mut seen_units = BTreeSet::new();
        let mut pending_rejected = None;

        for fact in &self.facts {
            match fact {
                CssSelectorSemanticFact::OpenMember { member, .. } => {
                    pending_rejected = None;
                    if !seen_members.insert(*member) {
                        return Err(CssSelectorHandoffInvariantViolation::DuplicateMemberId {
                            member: *member,
                        });
                    }
                    member_stack.push(*member);
                }
                CssSelectorSemanticFact::CloseMember { member } => {
                    pending_rejected = None;
                    let actual = member_stack.pop().ok_or(
                        CssSelectorHandoffInvariantViolation::CloseMemberWithoutOpen {
                            member: *member,
                        },
                    )?;
                    if actual != *member {
                        return Err(CssSelectorHandoffInvariantViolation::MemberBalanceMismatch {
                            expected: actual,
                            actual: *member,
                        });
                    }
                }
                CssSelectorSemanticFact::RejectedForgivingMember { member, .. } => {
                    if !seen_members.insert(*member) {
                        return Err(CssSelectorHandoffInvariantViolation::DuplicateMemberId {
                            member: *member,
                        });
                    }
                    pending_rejected = Some(*member);
                }
                CssSelectorSemanticFact::Simple { unit, .. } => {
                    pending_rejected = None;
                    require_member(&member_stack)?;
                    insert_unit(&mut seen_units, *unit)?;
                }
                CssSelectorSemanticFact::OpenFunction { unit, .. } => {
                    pending_rejected = None;
                    require_member(&member_stack)?;
                    insert_unit(&mut seen_units, *unit)?;
                    function_stack.push(*unit);
                }
                CssSelectorSemanticFact::CloseFunction { unit } => {
                    pending_rejected = None;
                    let actual = function_stack.pop().ok_or(
                        CssSelectorHandoffInvariantViolation::CloseFunctionWithoutOpen {
                            unit: *unit,
                        },
                    )?;
                    if actual != *unit {
                        return Err(CssSelectorHandoffInvariantViolation::FunctionBalanceMismatch {
                            expected: actual,
                            actual: *unit,
                        });
                    }
                }
                CssSelectorSemanticFact::NestingPresence {
                    member,
                    unit,
                    disposition,
                    ..
                } => {
                    insert_unit(&mut seen_units, *unit)?;
                    match disposition {
                        CssSelectorNestingPresenceDisposition::Contributing => {
                            pending_rejected = None;
                            let current = require_member(&member_stack)?;
                            if current != *member {
                                return Err(
                                    CssSelectorHandoffInvariantViolation::NestingMemberMismatch {
                                        expected: current,
                                        actual: *member,
                                    },
                                );
                            }
                        }
                        CssSelectorNestingPresenceDisposition::NonContributingPresenceOnly => {
                            if pending_rejected != Some(*member) {
                                return Err(CssSelectorHandoffInvariantViolation::RejectedPresenceWithoutOwner {
                                    member: *member,
                                });
                            }
                        }
                    }
                }
                CssSelectorSemanticFact::Relationship { .. } => {
                    if let Some(member) = pending_rejected {
                        return Err(
                            CssSelectorHandoffInvariantViolation::RejectedMemberHasRelationship {
                                member,
                            },
                        );
                    }
                    require_member(&member_stack)?;
                }
            }
        }

        if let Some(member) = member_stack.pop() {
            return Err(CssSelectorHandoffInvariantViolation::UnclosedMember { member });
        }
        if let Some(unit) = function_stack.pop() {
            return Err(CssSelectorHandoffInvariantViolation::UnclosedFunction { unit });
        }
        Ok(())
    }

    fn validate_authored_provenance(
        &self,
        context_header: &SourceAnchor,
    ) -> Result<(), CssSelectorHandoffInvariantViolation> {
        for fact in &self.facts {
            if let Some(anchor) = authored_anchor(fact) {
                validate_authored_anchor(context_header, anchor)?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for CssSelectorSemanticProgram {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssSelectorSemanticProgram")
            .field("owning_context", &self.owning_context)
            .field("fact_count", &self.facts.len())
            .finish()
    }
}

impl PartialEq for CssSelectorSemanticProgram {
    fn eq(&self, other: &Self) -> bool {
        self.owning_context == other.owning_context && self.facts == other.facts
    }
}

impl Eq for CssSelectorSemanticProgram {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssSelectorHandoffInvariantViolation {
    OwningContextMismatch {
        expected: CssParserContextId,
        actual: CssParserContextId,
    },
    DuplicateMemberId {
        member: CssSelectorSemanticMemberId,
    },
    DuplicateUnitId {
        unit: CssSelectorSemanticUnitId,
    },
    FactOutsideMember,
    CloseMemberWithoutOpen {
        member: CssSelectorSemanticMemberId,
    },
    MemberBalanceMismatch {
        expected: CssSelectorSemanticMemberId,
        actual: CssSelectorSemanticMemberId,
    },
    UnclosedMember {
        member: CssSelectorSemanticMemberId,
    },
    CloseFunctionWithoutOpen {
        unit: CssSelectorSemanticUnitId,
    },
    FunctionBalanceMismatch {
        expected: CssSelectorSemanticUnitId,
        actual: CssSelectorSemanticUnitId,
    },
    UnclosedFunction {
        unit: CssSelectorSemanticUnitId,
    },
    NestingMemberMismatch {
        expected: CssSelectorSemanticMemberId,
        actual: CssSelectorSemanticMemberId,
    },
    RejectedPresenceWithoutOwner {
        member: CssSelectorSemanticMemberId,
    },
    RejectedMemberHasRelationship {
        member: CssSelectorSemanticMemberId,
    },
    AuthoredAnchorSourceMismatch,
    AuthoredAnchorOutsideContextHeader,
    EmptyAuthoredAnchor,
    RelationshipContextNotFound {
        context: CssParserContextId,
    },
    UnexpectedRelationshipParentKind {
        context: CssParserContextId,
    },
}

#[derive(Debug)]
pub(crate) enum CssSelectorRelationshipResolutionError<E> {
    Handoff(CssSelectorHandoffInvariantViolation),
    Charge(E),
}

/// Resolves one selector observation's structural relationship target from
/// retained parser parents. Each parent is charged by the caller before this
/// function inspects that parent's record.
pub(crate) fn resolve_relationship_target<E>(
    records: &[CssParserContextRecord],
    owning_context: CssParserContextId,
    mut charge_before_parent_inspection: impl FnMut(CssParserContextId) -> Result<(), E>,
) -> Result<CssSelectorSemanticRelationshipTarget, CssSelectorRelationshipResolutionError<E>> {
    let current = records
        .get(owning_context.index())
        .filter(|record| record.id() == owning_context)
        .ok_or_else(|| {
            CssSelectorRelationshipResolutionError::Handoff(
                CssSelectorHandoffInvariantViolation::RelationshipContextNotFound {
                    context: owning_context,
                },
            )
        })?;

    let mut parent = current.parent();
    while let Some(parent_id) = parent {
        charge_before_parent_inspection(parent_id)
            .map_err(CssSelectorRelationshipResolutionError::Charge)?;

        // The charge above is deliberately before this lookup/inspection.
        let record = records
            .get(parent_id.index())
            .filter(|record| record.id() == parent_id)
            .ok_or_else(|| {
                CssSelectorRelationshipResolutionError::Handoff(
                    CssSelectorHandoffInvariantViolation::RelationshipContextNotFound {
                        context: parent_id,
                    },
                )
            })?;

        match record.kind() {
            CssParserContextKind::QualifiedRuleBlock => {
                return Ok(CssSelectorSemanticRelationshipTarget::ParentSelectorList(
                    parent_id,
                ));
            }
            CssParserContextKind::GroupRuleBlock(CssParserGroupRuleKind::Scope) => {
                return Ok(CssSelectorSemanticRelationshipTarget::ScopeRoot(parent_id));
            }
            CssParserContextKind::GroupRuleBlock(_) => {
                parent = record.parent();
            }
            _ => {
                return Err(CssSelectorRelationshipResolutionError::Handoff(
                    CssSelectorHandoffInvariantViolation::UnexpectedRelationshipParentKind {
                        context: parent_id,
                    },
                ));
            }
        }
    }

    Ok(CssSelectorSemanticRelationshipTarget::Zero)
}

fn require_member(
    member_stack: &[CssSelectorSemanticMemberId],
) -> Result<CssSelectorSemanticMemberId, CssSelectorHandoffInvariantViolation> {
    member_stack
        .last()
        .copied()
        .ok_or(CssSelectorHandoffInvariantViolation::FactOutsideMember)
}

fn insert_unit(
    seen_units: &mut BTreeSet<CssSelectorSemanticUnitId>,
    unit: CssSelectorSemanticUnitId,
) -> Result<(), CssSelectorHandoffInvariantViolation> {
    if !seen_units.insert(unit) {
        return Err(CssSelectorHandoffInvariantViolation::DuplicateUnitId { unit });
    }
    Ok(())
}

fn authored_anchor(fact: &CssSelectorSemanticFact) -> Option<&SourceAnchor> {
    match fact {
        CssSelectorSemanticFact::OpenMember { range, .. }
        | CssSelectorSemanticFact::RejectedForgivingMember { range, .. }
        | CssSelectorSemanticFact::Simple { range, .. }
        | CssSelectorSemanticFact::OpenFunction { range, .. } => Some(range),
        CssSelectorSemanticFact::NestingPresence {
            origin: CssSelectorSemanticRelationshipOrigin::Authored(anchor),
            ..
        }
        | CssSelectorSemanticFact::Relationship {
            origin: CssSelectorSemanticRelationshipOrigin::Authored(anchor),
            ..
        } => Some(anchor),
        CssSelectorSemanticFact::CloseMember { .. }
        | CssSelectorSemanticFact::CloseFunction { .. }
        | CssSelectorSemanticFact::NestingPresence {
            origin: CssSelectorSemanticRelationshipOrigin::Derived,
            ..
        }
        | CssSelectorSemanticFact::Relationship {
            origin: CssSelectorSemanticRelationshipOrigin::Derived,
            ..
        } => None,
    }
}

fn validate_authored_anchor(
    context_header: &SourceAnchor,
    anchor: &SourceAnchor,
) -> Result<(), CssSelectorHandoffInvariantViolation> {
    if !context_header.retains_exact_source(anchor) {
        return Err(CssSelectorHandoffInvariantViolation::AuthoredAnchorSourceMismatch);
    }
    if anchor.range().is_empty() {
        return Err(CssSelectorHandoffInvariantViolation::EmptyAuthoredAnchor);
    }
    if anchor.range().start() < context_header.range().start()
        || anchor.range().end() > context_header.range().end()
    {
        return Err(CssSelectorHandoffInvariantViolation::AuthoredAnchorOutsideContextHeader);
    }
    Ok(())
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.retains_exact_source(right) && left.range() == right.range()
}
