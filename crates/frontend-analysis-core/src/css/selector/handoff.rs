//! Retained selector-semantic handoff domain (#405, placed by #404).
//!
//! This module owns the retained handoff vocabulary, the structural invariants
//! of a retained program, and semantic relationship resolution over retained
//! parser ancestry. It is deliberately **not** a selector parser:
//! `super::producer::SelectorMachine` remains the sole selected `CoreV1`
//! grammar and recovery authority, and every fact retained here is staged while
//! that machine already owns the meaning of the evidence.
//!
//! The retained representation is linear, ordered, balanced and non-navigable.
//! No selector AST, child vector, or navigable node exists here. Member and
//! unit identifiers are program-local, opaque, deterministic, crate-private and
//! never serialized; gaps left by rolled-back speculative staging are valid and
//! numeric contiguity carries no semantic meaning.

use std::error::Error;
use std::fmt;

use crate::SourceAnchor;

use super::super::parser::context::{
    CssParserContextId, CssParserContextKind, CssParserContextRecord, CssParserGroupRuleKind,
};
use super::profile::CssSelectorFunctionalPseudoClass;

/// Program-local opaque member identity.
///
/// `Ord` exists only so structural validation can test set membership without
/// depending on allocation order. Numeric value carries no semantic meaning and
/// is never exposed outside this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CssSelectorSemanticMemberId(usize);

/// Program-local opaque semantic-unit identity.
///
/// The `Ord` note on [`CssSelectorSemanticMemberId`] applies identically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CssSelectorSemanticUnitId(usize);

impl CssSelectorSemanticMemberId {
    pub(super) const fn new(value: usize) -> Self {
        Self(value)
    }
}

impl CssSelectorSemanticUnitId {
    pub(super) const fn new(value: usize) -> Self {
        Self(value)
    }
}

/// Selected `CoreV1` simple-selector semantic kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorSemanticSimpleKind {
    Type,
    Universal,
    Id,
    Class,
    Attribute,
    IdentifierPseudoClass,
}

/// Structural semantic relationship target of one retained program.
///
/// This is deliberately distinct from [`super::context::CssSelectorGrammarContext`]:
/// grammar-entry selection gives `@scope` precedence over any qualified
/// ancestor, while the semantic relationship target is the nearest retained
/// structural boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorSemanticRelationshipTarget {
    ParentSelectorList(CssParserContextId),
    ScopeRoot(CssParserContextId),
    Zero,
}

/// Whether a relationship or nesting presence was authored or derived.
///
/// `Derived` never fabricates a [`SourceAnchor`].
#[derive(Debug, Clone)]
pub(crate) enum CssSelectorSemanticRelationshipOrigin {
    Authored(SourceAnchor),
    Derived,
}

/// Whether an authored nesting selector contributes to its member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorSemanticNestingDisposition {
    Contributing,
    NonContributingPresenceOnly,
}

/// One retained ordered semantic fact.
#[derive(Debug, Clone)]
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
        kind: CssSelectorFunctionalPseudoClass,
        range: SourceAnchor,
    },
    CloseFunction {
        unit: CssSelectorSemanticUnitId,
    },
    NestingPresence {
        member: CssSelectorSemanticMemberId,
        unit: CssSelectorSemanticUnitId,
        origin: CssSelectorSemanticRelationshipOrigin,
        disposition: CssSelectorSemanticNestingDisposition,
    },
    Relationship {
        target: CssSelectorSemanticRelationshipTarget,
        origin: CssSelectorSemanticRelationshipOrigin,
    },
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

impl PartialEq for CssSelectorSemanticFact {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::OpenMember {
                    member: left_member,
                    range: left_range,
                },
                Self::OpenMember {
                    member: right_member,
                    range: right_range,
                },
            )
            | (
                Self::RejectedForgivingMember {
                    member: left_member,
                    range: left_range,
                },
                Self::RejectedForgivingMember {
                    member: right_member,
                    range: right_range,
                },
            ) => left_member == right_member && same_anchor(left_range, right_range),
            (Self::CloseMember { member: left }, Self::CloseMember { member: right }) => {
                left == right
            }
            (
                Self::Simple {
                    unit: left_unit,
                    kind: left_kind,
                    range: left_range,
                },
                Self::Simple {
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
                Self::OpenFunction {
                    unit: left_unit,
                    kind: left_kind,
                    range: left_range,
                },
                Self::OpenFunction {
                    unit: right_unit,
                    kind: right_kind,
                    range: right_range,
                },
            ) => {
                left_unit == right_unit
                    && left_kind == right_kind
                    && same_anchor(left_range, right_range)
            }
            (Self::CloseFunction { unit: left }, Self::CloseFunction { unit: right }) => {
                left == right
            }
            (
                Self::NestingPresence {
                    member: left_member,
                    unit: left_unit,
                    origin: left_origin,
                    disposition: left_disposition,
                },
                Self::NestingPresence {
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
                Self::Relationship {
                    target: left_target,
                    origin: left_origin,
                },
                Self::Relationship {
                    target: right_target,
                    origin: right_origin,
                },
            ) => left_target == right_target && left_origin == right_origin,
            _ => false,
        }
    }
}

impl Eq for CssSelectorSemanticFact {}

/// One complete retained semantic program owned by exactly one observation.
///
/// The minimum retained attachment identity fixed by #404 FA404-02 is exactly
/// one owning parser `ContextId`. That identity is parser-run-local attachment
/// evidence only; it is never a persistent, authored, serialized, or cross-run
/// identity, and numeric equality across distinct runs is never semantic
/// identity.
#[derive(Debug, Clone)]
pub(crate) struct CssSelectorSemanticProgram {
    owning_context: CssParserContextId,
    facts: Vec<CssSelectorSemanticFact>,
}

impl CssSelectorSemanticProgram {
    /// Creates a candidate program.
    ///
    /// Visibility is deliberately limited to the selector module so no caller
    /// outside the owning selector run can mint a detached program. Structural
    /// validity is enforced by
    /// [`super::result::CssSelectorQualificationObservation::new`] before
    /// durable commit and independently again by the run-result contract.
    pub(super) const fn new(
        owning_context: CssParserContextId,
        facts: Vec<CssSelectorSemanticFact>,
    ) -> Self {
        Self {
            owning_context,
            facts,
        }
    }

    pub(crate) const fn owning_context(&self) -> CssParserContextId {
        self.owning_context
    }

    pub(crate) fn facts(&self) -> &[CssSelectorSemanticFact] {
        &self.facts
    }

    /// Validates the linear/ordered/balanced retained-program invariants.
    pub(super) fn validate_structure(&self) -> Result<(), CssSelectorSemanticProgramError> {
        validate_program_structure(&self.facts)
    }
}

impl PartialEq for CssSelectorSemanticProgram {
    fn eq(&self, other: &Self) -> bool {
        self.owning_context == other.owning_context && self.facts == other.facts
    }
}

impl Eq for CssSelectorSemanticProgram {}

/// A retained program that violates the structural handoff invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorSemanticProgramError {
    MemberAlreadyOpen { fact_index: usize },
    DuplicateMemberIdentity { fact_index: usize },
    DuplicateUnitIdentity { fact_index: usize },
    UnbalancedCloseMember { fact_index: usize },
    FactOutsideOpenMember { fact_index: usize },
    NestingPresenceMemberMismatch { fact_index: usize },
    RejectedForgivingMemberOutsideFunction { fact_index: usize },
    RejectedForgivingMemberOutsideForgivingFunction { fact_index: usize },
    RejectedForgivingMemberInsideOpenMember { fact_index: usize },
    NestingPresenceMustBeAuthored { fact_index: usize },
    UnownedNonContributingPresence { fact_index: usize },
    NonContributingPresenceMemberMismatch { fact_index: usize },
    ContributingFactInRejectedMember { fact_index: usize },
    UnbalancedCloseFunction { fact_index: usize },
    CloseFunctionInsideOpenMember { fact_index: usize },
    UnbalancedProgramEnd,
}

impl fmt::Display for CssSelectorSemanticProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS selector semantic-program structure violation: {self:?}"
        )
    }
}

impl Error for CssSelectorSemanticProgramError {}

struct StructureFrame {
    function_unit: Option<CssSelectorSemanticUnitId>,
    /// The selected function this frame opened, or `None` for the root list.
    ///
    /// Retained so that structural validation can seal rejected forgiving
    /// members to selected forgiving functions without consulting authored
    /// bytes or any producer helper.
    function_kind: Option<CssSelectorFunctionalPseudoClass>,
    open_member: Option<CssSelectorSemanticMemberId>,
}

/// Enforces the retained linear/ordered/balanced program invariants.
///
/// The walk is a single forward pass over the ordered facts. It never builds a
/// navigable structure and never interprets authored bytes.
fn validate_program_structure(
    facts: &[CssSelectorSemanticFact],
) -> Result<(), CssSelectorSemanticProgramError> {
    let mut frames = vec![StructureFrame {
        function_unit: None,
        function_kind: None,
        open_member: None,
    }];
    let mut members: Vec<CssSelectorSemanticMemberId> = Vec::new();
    let mut units: Vec<CssSelectorSemanticUnitId> = Vec::new();
    let mut rejected: Option<CssSelectorSemanticMemberId> = None;

    for (fact_index, fact) in facts.iter().enumerate() {
        // A nesting presence records an authored nesting-selector occurrence,
        // so a derived origin is malformed retained evidence under either
        // disposition. This is checked before any other dispatch so neither
        // the contributing path nor the rejected-member path can admit one.
        if matches!(
            fact,
            CssSelectorSemanticFact::NestingPresence {
                origin: CssSelectorSemanticRelationshipOrigin::Derived,
                ..
            }
        ) {
            return Err(
                CssSelectorSemanticProgramError::NestingPresenceMustBeAuthored { fact_index },
            );
        }

        if let Some(rejected_member) = rejected {
            match fact {
                CssSelectorSemanticFact::NestingPresence {
                    member,
                    unit,
                    disposition: CssSelectorSemanticNestingDisposition::NonContributingPresenceOnly,
                    ..
                } => {
                    if *member != rejected_member {
                        return Err(
                            CssSelectorSemanticProgramError::NonContributingPresenceMemberMismatch {
                                fact_index,
                            },
                        );
                    }
                    claim_unit(&mut units, *unit, fact_index)?;
                    continue;
                }
                CssSelectorSemanticFact::OpenMember { .. }
                | CssSelectorSemanticFact::RejectedForgivingMember { .. }
                | CssSelectorSemanticFact::CloseFunction { .. } => rejected = None,
                _ => {
                    return Err(
                        CssSelectorSemanticProgramError::ContributingFactInRejectedMember {
                            fact_index,
                        },
                    );
                }
            }
        } else if matches!(
            fact,
            CssSelectorSemanticFact::NestingPresence {
                disposition: CssSelectorSemanticNestingDisposition::NonContributingPresenceOnly,
                ..
            }
        ) {
            return Err(
                CssSelectorSemanticProgramError::UnownedNonContributingPresence { fact_index },
            );
        }

        let depth = frames.len();
        let frame = frames
            .last()
            .ok_or(CssSelectorSemanticProgramError::UnbalancedProgramEnd)?;
        let open_member = frame.open_member;
        let function_unit = frame.function_unit;
        let function_kind = frame.function_kind;
        match fact {
            CssSelectorSemanticFact::OpenMember { member, .. } => {
                if open_member.is_some() {
                    return Err(CssSelectorSemanticProgramError::MemberAlreadyOpen { fact_index });
                }
                claim_member(&mut members, *member, fact_index)?;
                set_open_member(&mut frames, Some(*member))?;
            }
            CssSelectorSemanticFact::CloseMember { member } => {
                if open_member != Some(*member) {
                    return Err(CssSelectorSemanticProgramError::UnbalancedCloseMember {
                        fact_index,
                    });
                }
                set_open_member(&mut frames, None)?;
            }
            CssSelectorSemanticFact::RejectedForgivingMember { member, .. } => {
                if depth < 2 {
                    return Err(
                        CssSelectorSemanticProgramError::RejectedForgivingMemberOutsideFunction {
                            fact_index,
                        },
                    );
                }
                // Only `:is()` and `:where()` are forgiving under the selected
                // CoreV1 semantics, so a rejected member is impossible inside
                // `:not()` or `:has()`.
                if !matches!(
                    function_kind,
                    Some(
                        CssSelectorFunctionalPseudoClass::Is
                            | CssSelectorFunctionalPseudoClass::Where
                    )
                ) {
                    return Err(
                        CssSelectorSemanticProgramError::RejectedForgivingMemberOutsideForgivingFunction {
                            fact_index,
                        },
                    );
                }
                if open_member.is_some() {
                    return Err(
                        CssSelectorSemanticProgramError::RejectedForgivingMemberInsideOpenMember {
                            fact_index,
                        },
                    );
                }
                claim_member(&mut members, *member, fact_index)?;
                rejected = Some(*member);
            }
            CssSelectorSemanticFact::Simple { unit, .. } => {
                if open_member.is_none() {
                    return Err(CssSelectorSemanticProgramError::FactOutsideOpenMember {
                        fact_index,
                    });
                }
                claim_unit(&mut units, *unit, fact_index)?;
            }
            CssSelectorSemanticFact::OpenFunction { unit, kind, .. } => {
                if open_member.is_none() {
                    return Err(CssSelectorSemanticProgramError::FactOutsideOpenMember {
                        fact_index,
                    });
                }
                claim_unit(&mut units, *unit, fact_index)?;
                frames.push(StructureFrame {
                    function_unit: Some(*unit),
                    function_kind: Some(*kind),
                    open_member: None,
                });
            }
            CssSelectorSemanticFact::CloseFunction { unit } => {
                if depth < 2 || function_unit != Some(*unit) {
                    return Err(CssSelectorSemanticProgramError::UnbalancedCloseFunction {
                        fact_index,
                    });
                }
                if open_member.is_some() {
                    return Err(
                        CssSelectorSemanticProgramError::CloseFunctionInsideOpenMember {
                            fact_index,
                        },
                    );
                }
                frames.pop();
            }
            CssSelectorSemanticFact::NestingPresence { member, unit, .. } => {
                if open_member != Some(*member) {
                    return Err(
                        CssSelectorSemanticProgramError::NestingPresenceMemberMismatch {
                            fact_index,
                        },
                    );
                }
                claim_unit(&mut units, *unit, fact_index)?;
            }
            CssSelectorSemanticFact::Relationship { .. } => {
                if open_member.is_none() {
                    return Err(CssSelectorSemanticProgramError::FactOutsideOpenMember {
                        fact_index,
                    });
                }
            }
        }
    }

    match frames.as_slice() {
        [only] if only.open_member.is_none() && rejected.is_none() => Ok(()),
        _ => Err(CssSelectorSemanticProgramError::UnbalancedProgramEnd),
    }
}

fn set_open_member(
    frames: &mut [StructureFrame],
    member: Option<CssSelectorSemanticMemberId>,
) -> Result<(), CssSelectorSemanticProgramError> {
    frames
        .last_mut()
        .ok_or(CssSelectorSemanticProgramError::UnbalancedProgramEnd)?
        .open_member = member;
    Ok(())
}

fn claim_member(
    members: &mut Vec<CssSelectorSemanticMemberId>,
    member: CssSelectorSemanticMemberId,
    fact_index: usize,
) -> Result<(), CssSelectorSemanticProgramError> {
    match members.binary_search(&member) {
        Ok(_) => Err(CssSelectorSemanticProgramError::DuplicateMemberIdentity { fact_index }),
        Err(position) => {
            members.insert(position, member);
            Ok(())
        }
    }
}

fn claim_unit(
    units: &mut Vec<CssSelectorSemanticUnitId>,
    unit: CssSelectorSemanticUnitId,
    fact_index: usize,
) -> Result<(), CssSelectorSemanticProgramError> {
    match units.binary_search(&unit) {
        Ok(_) => Err(CssSelectorSemanticProgramError::DuplicateUnitIdentity { fact_index }),
        Err(position) => {
            units.insert(position, unit);
            Ok(())
        }
    }
}

/// A retained ancestry structure that cannot support relationship resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorSemanticRelationshipError {
    ParentNotFound {
        context: CssParserContextId,
        parent: CssParserContextId,
    },
}

impl fmt::Display for CssSelectorSemanticRelationshipError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS selector semantic relationship violation: {self:?}"
        )
    }
}

impl Error for CssSelectorSemanticRelationshipError {}

/// Result of the caller-owned selector `AlgorithmSteps` precharge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CssSelectorRelationshipCharge {
    Granted,
    Refused,
}

/// Outcome of one relationship resolution walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CssSelectorRelationshipResolution {
    Target(CssSelectorSemanticRelationshipTarget),
    Refused,
}

/// Resolves the nearest retained structural relationship boundary.
///
/// The walk follows retained parser parent `ContextId` links only. It never
/// scans source, never scans tokenizer evidence, and never consumes parser or
/// tokenizer resource counters.
///
/// `charge_parent_inspection` is the selector-owned `AlgorithmSteps` precharge
/// required by #404 FA404-01: it is invoked for a known parent `ContextId`
/// **before** that parent record is looked up or inspected, so a refusal always
/// happens before the refused record is read.
pub(super) fn resolve_relationship_target<E, C>(
    records: &[CssParserContextRecord],
    record: &CssParserContextRecord,
    mut charge_parent_inspection: C,
) -> Result<CssSelectorRelationshipResolution, E>
where
    E: From<CssSelectorSemanticRelationshipError>,
    C: FnMut() -> Result<CssSelectorRelationshipCharge, E>,
{
    let context = record.id();
    let mut parent = record.parent();
    while let Some(parent_id) = parent {
        if matches!(
            charge_parent_inspection()?,
            CssSelectorRelationshipCharge::Refused
        ) {
            return Ok(CssSelectorRelationshipResolution::Refused);
        }
        let parent_record = records
            .get(parent_id.index())
            .filter(|candidate| candidate.id() == parent_id)
            .ok_or_else(|| {
                E::from(CssSelectorSemanticRelationshipError::ParentNotFound {
                    context,
                    parent: parent_id,
                })
            })?;
        match parent_record.kind() {
            CssParserContextKind::QualifiedRuleBlock => {
                return Ok(CssSelectorRelationshipResolution::Target(
                    CssSelectorSemanticRelationshipTarget::ParentSelectorList(parent_id),
                ));
            }
            CssParserContextKind::GroupRuleBlock(CssParserGroupRuleKind::Scope) => {
                return Ok(CssSelectorRelationshipResolution::Target(
                    CssSelectorSemanticRelationshipTarget::ScopeRoot(parent_id),
                ));
            }
            _ => parent = parent_record.parent(),
        }
    }

    Ok(CssSelectorRelationshipResolution::Target(
        CssSelectorSemanticRelationshipTarget::Zero,
    ))
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.retains_exact_source(right) && left.range() == right.range()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::context::{CssParserContextTermination, CssParserDirectItemOrdinal};
    use crate::css::parser::producer::run as run_parser;
    use crate::css::parser::resource::CssParserLimits;
    use crate::css::tokenizer::producer::run as run_tokenizer;
    use crate::css::tokenizer::resource::CssTokenizerLimits;
    use crate::{SourceId, SourceText};

    fn member(value: usize) -> CssSelectorSemanticMemberId {
        CssSelectorSemanticMemberId::new(value)
    }

    fn unit(value: usize) -> CssSelectorSemanticUnitId {
        CssSelectorSemanticUnitId::new(value)
    }

    fn source() -> SourceText {
        SourceText::new(SourceId::new(900), ".a:is(:b&, .c)".to_owned())
    }

    fn anchor(source: &SourceText, start: usize, end: usize) -> SourceAnchor {
        source.anchor(start, end).unwrap()
    }

    fn open(
        source: &SourceText,
        value: usize,
        start: usize,
        end: usize,
    ) -> CssSelectorSemanticFact {
        CssSelectorSemanticFact::OpenMember {
            member: member(value),
            range: anchor(source, start, end),
        }
    }

    fn close(value: usize) -> CssSelectorSemanticFact {
        CssSelectorSemanticFact::CloseMember {
            member: member(value),
        }
    }

    #[test]
    fn balanced_linear_program_is_structurally_valid() {
        let source = source();
        let program = CssSelectorSemanticProgram::new(
            CssParserContextId::new(0),
            vec![
                open(&source, 1, 0, 14),
                CssSelectorSemanticFact::Simple {
                    unit: unit(1),
                    kind: CssSelectorSemanticSimpleKind::Class,
                    range: anchor(&source, 0, 2),
                },
                CssSelectorSemanticFact::OpenFunction {
                    unit: unit(2),
                    kind: CssSelectorFunctionalPseudoClass::Is,
                    range: anchor(&source, 2, 6),
                },
                CssSelectorSemanticFact::RejectedForgivingMember {
                    member: member(2),
                    range: anchor(&source, 6, 9),
                },
                CssSelectorSemanticFact::NestingPresence {
                    member: member(2),
                    unit: unit(3),
                    origin: CssSelectorSemanticRelationshipOrigin::Authored(anchor(&source, 8, 9)),
                    disposition: CssSelectorSemanticNestingDisposition::NonContributingPresenceOnly,
                },
                open(&source, 3, 11, 13),
                CssSelectorSemanticFact::Simple {
                    unit: unit(4),
                    kind: CssSelectorSemanticSimpleKind::Class,
                    range: anchor(&source, 11, 13),
                },
                close(3),
                CssSelectorSemanticFact::CloseFunction { unit: unit(2) },
                close(1),
            ],
        );

        assert_eq!(program.validate_structure(), Ok(()));
        assert_eq!(program.owning_context(), CssParserContextId::new(0));
        assert_eq!(program.facts().len(), 10);
    }

    #[test]
    fn structural_validation_rejects_unbalanced_and_unowned_evidence() {
        let source = source();

        assert_eq!(
            validate_program_structure(&[open(&source, 1, 0, 2)]),
            Err(CssSelectorSemanticProgramError::UnbalancedProgramEnd)
        );
        assert_eq!(
            validate_program_structure(&[open(&source, 1, 0, 2), close(2)]),
            Err(CssSelectorSemanticProgramError::UnbalancedCloseMember { fact_index: 1 })
        );
        assert_eq!(
            validate_program_structure(&[
                open(&source, 1, 0, 2),
                close(1),
                open(&source, 1, 0, 2),
                close(1),
            ]),
            Err(CssSelectorSemanticProgramError::DuplicateMemberIdentity { fact_index: 2 })
        );
        assert_eq!(
            validate_program_structure(&[CssSelectorSemanticFact::Relationship {
                target: CssSelectorSemanticRelationshipTarget::Zero,
                origin: CssSelectorSemanticRelationshipOrigin::Derived,
            }]),
            Err(CssSelectorSemanticProgramError::FactOutsideOpenMember { fact_index: 0 })
        );
        assert_eq!(
            validate_program_structure(&[CssSelectorSemanticFact::NestingPresence {
                member: member(9),
                unit: unit(9),
                origin: CssSelectorSemanticRelationshipOrigin::Authored(anchor(&source, 8, 9)),
                disposition: CssSelectorSemanticNestingDisposition::NonContributingPresenceOnly,
            }]),
            Err(CssSelectorSemanticProgramError::UnownedNonContributingPresence { fact_index: 0 })
        );
    }

    #[test]
    fn rejected_member_region_retains_no_contributing_evidence() {
        let source = source();
        let rejected = CssSelectorSemanticFact::RejectedForgivingMember {
            member: member(2),
            range: anchor(&source, 6, 9),
        };

        assert_eq!(
            validate_program_structure(&[
                open(&source, 1, 0, 14),
                CssSelectorSemanticFact::OpenFunction {
                    unit: unit(2),
                    kind: CssSelectorFunctionalPseudoClass::Is,
                    range: anchor(&source, 2, 6),
                },
                rejected.clone(),
                CssSelectorSemanticFact::Relationship {
                    target: CssSelectorSemanticRelationshipTarget::Zero,
                    origin: CssSelectorSemanticRelationshipOrigin::Derived,
                },
                CssSelectorSemanticFact::CloseFunction { unit: unit(2) },
                close(1),
            ]),
            Err(
                CssSelectorSemanticProgramError::ContributingFactInRejectedMember { fact_index: 3 }
            )
        );

        assert_eq!(
            validate_program_structure(&[rejected]),
            Err(
                CssSelectorSemanticProgramError::RejectedForgivingMemberOutsideFunction {
                    fact_index: 0
                }
            )
        );
    }

    fn function_program(
        source: &SourceText,
        kind: CssSelectorFunctionalPseudoClass,
        inner: Vec<CssSelectorSemanticFact>,
    ) -> Vec<CssSelectorSemanticFact> {
        let mut facts = vec![
            open(source, 1, 0, 14),
            CssSelectorSemanticFact::OpenFunction {
                unit: unit(2),
                kind,
                range: anchor(source, 2, 6),
            },
        ];
        facts.extend(inner);
        facts.push(CssSelectorSemanticFact::CloseFunction { unit: unit(2) });
        facts.push(close(1));
        facts
    }

    fn rejected_member(source: &SourceText) -> Vec<CssSelectorSemanticFact> {
        vec![
            CssSelectorSemanticFact::RejectedForgivingMember {
                member: member(2),
                range: anchor(source, 6, 9),
            },
            CssSelectorSemanticFact::NestingPresence {
                member: member(2),
                unit: unit(3),
                origin: CssSelectorSemanticRelationshipOrigin::Authored(anchor(source, 8, 9)),
                disposition: CssSelectorSemanticNestingDisposition::NonContributingPresenceOnly,
            },
        ]
    }

    #[test]
    fn rejected_forgiving_members_are_sealed_to_selected_forgiving_functions() {
        let source = source();

        // Positive controls: the two selected forgiving functions.
        for kind in [
            CssSelectorFunctionalPseudoClass::Is,
            CssSelectorFunctionalPseudoClass::Where,
        ] {
            assert_eq!(
                validate_program_structure(&function_program(
                    &source,
                    kind,
                    rejected_member(&source)
                )),
                Ok(()),
                "{kind:?} is forgiving and may retain a rejected member"
            );
        }

        // Negative falsifiers: the unforgiving selected functions never
        // reject a member, so this retained shape is impossible.
        for kind in [
            CssSelectorFunctionalPseudoClass::Not,
            CssSelectorFunctionalPseudoClass::Has,
        ] {
            assert_eq!(
                validate_program_structure(&function_program(
                    &source,
                    kind,
                    rejected_member(&source)
                )),
                Err(
                    CssSelectorSemanticProgramError::RejectedForgivingMemberOutsideForgivingFunction {
                        fact_index: 2,
                    }
                ),
                "{kind:?} is not forgiving and must fail closed"
            );
        }
    }

    #[test]
    fn every_retained_nesting_presence_must_carry_authored_evidence() {
        let source = source();

        // A derived contributing presence is malformed retained evidence.
        assert_eq!(
            validate_program_structure(&[
                open(&source, 1, 0, 2),
                CssSelectorSemanticFact::NestingPresence {
                    member: member(1),
                    unit: unit(1),
                    origin: CssSelectorSemanticRelationshipOrigin::Derived,
                    disposition: CssSelectorSemanticNestingDisposition::Contributing,
                },
                close(1),
            ]),
            Err(CssSelectorSemanticProgramError::NestingPresenceMustBeAuthored { fact_index: 1 })
        );

        // So is a derived presence owned by a rejected forgiving member.
        assert_eq!(
            validate_program_structure(&function_program(
                &source,
                CssSelectorFunctionalPseudoClass::Is,
                vec![
                    CssSelectorSemanticFact::RejectedForgivingMember {
                        member: member(2),
                        range: anchor(&source, 6, 9),
                    },
                    CssSelectorSemanticFact::NestingPresence {
                        member: member(2),
                        unit: unit(3),
                        origin: CssSelectorSemanticRelationshipOrigin::Derived,
                        disposition:
                            CssSelectorSemanticNestingDisposition::NonContributingPresenceOnly,
                    },
                ],
            )),
            Err(CssSelectorSemanticProgramError::NestingPresenceMustBeAuthored { fact_index: 3 })
        );

        // Positive control: the same shapes with authored evidence stay valid.
        assert_eq!(
            validate_program_structure(&[
                open(&source, 1, 0, 2),
                CssSelectorSemanticFact::NestingPresence {
                    member: member(1),
                    unit: unit(1),
                    origin: CssSelectorSemanticRelationshipOrigin::Authored(anchor(&source, 0, 1)),
                    disposition: CssSelectorSemanticNestingDisposition::Contributing,
                },
                close(1),
            ]),
            Ok(())
        );
    }

    fn parse(source: &SourceText) -> Vec<CssParserContextRecord> {
        let tokenizer = run_tokenizer(
            source,
            CssTokenizerLimits::new(4096, 50_000, 4096, 256, 4096, 4096).unwrap(),
        )
        .unwrap();
        run_parser(
            source,
            tokenizer,
            CssParserLimits::new(50_000, 128, 128, 4096, 256, 256, 256, 256, 4096).unwrap(),
        )
        .unwrap()
        .context_records()
        .to_vec()
    }

    #[derive(Debug, PartialEq, Eq)]
    enum TestError {
        Relationship(CssSelectorSemanticRelationshipError),
    }

    impl From<CssSelectorSemanticRelationshipError> for TestError {
        fn from(error: CssSelectorSemanticRelationshipError) -> Self {
            Self::Relationship(error)
        }
    }

    fn resolve(
        records: &[CssParserContextRecord],
        record: &CssParserContextRecord,
    ) -> (CssSelectorRelationshipResolution, usize) {
        let mut charges = 0usize;
        let resolution = resolve_relationship_target::<TestError, _>(records, record, || {
            charges += 1;
            Ok(CssSelectorRelationshipCharge::Granted)
        })
        .unwrap();
        (resolution, charges)
    }

    #[test]
    fn grammar_scope_precedence_does_not_decide_the_semantic_relationship_target() {
        // Both `& .b` rules select scoped-relative grammar, because grammar
        // selection gives the innermost retained `@scope` ancestor precedence.
        // Their nearest retained structural boundary differs. The outer
        // qualified rule exists because this baseline's parser retains group
        // contexts only for nested at-rules.
        let outer_scope = SourceText::new(SourceId::new(901), ".z{@scope{.a{& .b{}}}}".to_owned());
        let records = parse(&outer_scope);
        let (resolution, charges) = resolve(&records, &records[3]);
        assert_eq!(
            resolution,
            CssSelectorRelationshipResolution::Target(
                CssSelectorSemanticRelationshipTarget::ParentSelectorList(records[2].id())
            )
        );
        assert_eq!(charges, 1);

        let inner_scope = SourceText::new(SourceId::new(902), ".z{.a{@scope{& .b{}}}}".to_owned());
        let records = parse(&inner_scope);
        let (resolution, charges) = resolve(&records, &records[3]);
        assert_eq!(
            resolution,
            CssSelectorRelationshipResolution::Target(
                CssSelectorSemanticRelationshipTarget::ScopeRoot(records[2].id())
            )
        );
        assert_eq!(charges, 1);
    }

    #[test]
    fn interposed_group_contexts_charge_one_step_per_inspected_parent() {
        let source = SourceText::new(
            SourceId::new(903),
            ".z{@layer l{@supports (a:b){@media all{.a{.b{}}}}}}".to_owned(),
        );
        let records = parse(&source);
        assert_eq!(records.len(), 6);

        // `.b` reaches its direct qualified parent immediately.
        let (resolution, charges) = resolve(&records, &records[5]);
        assert_eq!(
            resolution,
            CssSelectorRelationshipResolution::Target(
                CssSelectorSemanticRelationshipTarget::ParentSelectorList(records[4].id())
            )
        );
        assert_eq!(charges, 1);

        // `.a` walks Media -> Supports -> Layer -> `.z`: exactly one charge per
        // inspected retained parent record.
        let (resolution, charges) = resolve(&records, &records[4]);
        assert_eq!(
            resolution,
            CssSelectorRelationshipResolution::Target(
                CssSelectorSemanticRelationshipTarget::ParentSelectorList(records[0].id())
            )
        );
        assert_eq!(charges, 4);
    }

    #[test]
    fn refused_precharge_stops_before_the_refused_parent_is_inspected() {
        let source = SourceText::new(SourceId::new(904), "a{}".to_owned());
        let records = parse(&source);
        // The parent link is deliberately dangling: any resolution that read
        // the refused record would fail with ParentNotFound instead.
        let dangling = CssParserContextRecord::new_qualified_rule_block(
            &source,
            CssParserContextId::new(1),
            Some(CssParserContextId::new(9)),
            CssParserDirectItemOrdinal::new(0),
            None,
            source.anchor(0, 1).unwrap(),
            source.anchor(1, 2).unwrap(),
            source.anchor(2, 2).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: source.anchor(2, 3).unwrap(),
            },
        )
        .unwrap();

        assert_eq!(
            resolve_relationship_target::<TestError, _>(&records, &dangling, || Ok(
                CssSelectorRelationshipCharge::Refused
            )),
            Ok(CssSelectorRelationshipResolution::Refused)
        );
        assert_eq!(
            resolve_relationship_target::<TestError, _>(&records, &dangling, || Ok(
                CssSelectorRelationshipCharge::Granted
            )),
            Err(TestError::Relationship(
                CssSelectorSemanticRelationshipError::ParentNotFound {
                    context: CssParserContextId::new(1),
                    parent: CssParserContextId::new(9),
                }
            ))
        );
    }

    #[test]
    fn absent_semantic_boundary_resolves_to_zero_without_any_charge() {
        let source = SourceText::new(SourceId::new(905), "a{}".to_owned());
        let records = parse(&source);
        let (resolution, charges) = resolve(&records, &records[0]);

        assert_eq!(
            resolution,
            CssSelectorRelationshipResolution::Target(CssSelectorSemanticRelationshipTarget::Zero)
        );
        assert_eq!(charges, 0);
    }
}
