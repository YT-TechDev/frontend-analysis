//! Candidate-independent semantic-handoff gold for #402.
//!
//! This module deliberately does not import selector production contracts. It
//! models only the semantic information theorem accepted by #402.

#![allow(dead_code)]

pub(super) const CSSWG_REVISION: &str = "8fe035fe18fe98a62becff96df0e55dc3a5c1033";
pub(super) const SELECTORS_4_BLOB: &str = "3b81851cdaf8ea6eec5f63e6867822de0bad9410";
pub(super) const CSS_NESTING_1_BLOB: &str = "41db452e107401cab5b8394b85213f007287a14e";
pub(super) const CSS_SYNTAX_3_BLOB: &str = "62ece32e4f48299395f23db4a37336b25d21fe1e";
pub(super) const CSS_NAMESPACES_3_BLOB: &str = "9442ce15f4af6b7240f86eb44a4edd2ab116d958";
pub(super) const CSS_CASCADE_6_BLOB: &str = "8cd75053a1babf221f724781334180a842bf1d7b";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SourceId(pub(super) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RunId(pub(super) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ContextId(pub(super) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MemberId(pub(super) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct UnitId(pub(super) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AuthoredRange {
    pub(super) start: usize,
    pub(super) end: usize,
}

impl AuthoredRange {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RelationshipOrigin {
    Authored(AuthoredRange),
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SimpleKind {
    Type,
    Universal,
    Id,
    Class,
    Attribute,
    IdentifierPseudoClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FunctionKind {
    Is,
    Where,
    Not,
    Has,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RelationshipTarget {
    ParentSelectorList(ContextId),
    ScopeRoot(ContextId),
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NestingPresenceDisposition {
    Contributing,
    NonContributingPresenceOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SelectorFact {
    OpenMember {
        member: MemberId,
        range: AuthoredRange,
    },
    CloseMember {
        member: MemberId,
    },
    RejectedForgivingMember {
        member: MemberId,
        range: AuthoredRange,
    },
    Simple {
        unit: UnitId,
        kind: SimpleKind,
        range: AuthoredRange,
    },
    OpenFunction {
        unit: UnitId,
        kind: FunctionKind,
        range: AuthoredRange,
    },
    CloseFunction {
        unit: UnitId,
    },
    NestingPresence {
        member: MemberId,
        unit: UnitId,
        origin: RelationshipOrigin,
        disposition: NestingPresenceDisposition,
    },
    Relationship {
        target: RelationshipTarget,
        origin: RelationshipOrigin,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldProgram {
    pub(super) source: SourceId,
    pub(super) run: RunId,
    pub(super) profile: &'static str,
    pub(super) context: ContextId,
    pub(super) facts: Vec<SelectorFact>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InvalidReason {
    SelectedGrammar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UnsupportedFeature {
    FunctionalPseudoClass,
    PseudoElement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IndeterminateReason {
    MissingNamespaceEnvironment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldOutcome {
    Qualified,
    Invalid(InvalidReason),
    Unsupported(UnsupportedFeature),
    Indeterminate(IndeterminateReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CompletionState {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldObservation {
    pub(super) context: ContextId,
    pub(super) completion: CompletionState,
    pub(super) outcome: GoldOutcome,
    pub(super) program: Option<GoldProgram>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldRun {
    pub(super) upstream: CompletionState,
    pub(super) qualifier: CompletionState,
    pub(super) observations: Vec<GoldObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LiteralRangeExpectation {
    pub(super) range: AuthoredRange,
    pub(super) spelling: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LiteralRangeFailure {
    Reversed,
    OutOfBounds,
    InvalidStartBoundary,
    InvalidEndBoundary,
    SpellingMismatch,
}

pub(super) fn verify_literal_range(
    source: &str,
    expectation: LiteralRangeExpectation,
) -> Result<(), LiteralRangeFailure> {
    let range = expectation.range;
    if range.start > range.end {
        return Err(LiteralRangeFailure::Reversed);
    }
    if range.end > source.len() {
        return Err(LiteralRangeFailure::OutOfBounds);
    }
    if !source.is_char_boundary(range.start) {
        return Err(LiteralRangeFailure::InvalidStartBoundary);
    }
    if !source.is_char_boundary(range.end) {
        return Err(LiteralRangeFailure::InvalidEndBoundary);
    }
    if &source[range.start..range.end] != expectation.spelling {
        return Err(LiteralRangeFailure::SpellingMismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldFixture {
    pub(super) id: &'static str,
    pub(super) source: &'static str,
    pub(super) program: GoldProgram,
    pub(super) authored: Vec<LiteralRangeExpectation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RejectedNestingPresenceExpectation {
    pub(super) member: MemberId,
    pub(super) rejected_range: AuthoredRange,
    pub(super) unit: UnitId,
    pub(super) presence_range: AuthoredRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RejectedNestingEffect {
    SuppressesImpliedNesting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RejectedNestingPresenceEvidence {
    pub(super) member: MemberId,
    pub(super) unit: UnitId,
    pub(super) authored_range: AuthoredRange,
    pub(super) disposition: NestingPresenceDisposition,
    pub(super) effect: RejectedNestingEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RejectedNestingPresenceFailure {
    RejectedMemberMissing,
    RejectedMemberAmbiguous,
    RejectedMemberRangeMismatch {
        expected: AuthoredRange,
        actual: AuthoredRange,
    },
    PresenceMissing,
    PresenceAmbiguous,
    PresenceMemberMismatch {
        expected: MemberId,
        actual: MemberId,
    },
    PresenceUnitMismatch {
        expected: UnitId,
        actual: UnitId,
    },
    PresenceMustBeAuthored,
    PresenceRangeMismatch {
        expected: AuthoredRange,
        actual: AuthoredRange,
    },
    PresenceMustBeNonContributing,
    ContributingRelationshipInRejectedMember,
}

pub(super) fn validate_rejected_nesting_presence(
    program: &GoldProgram,
    expectation: RejectedNestingPresenceExpectation,
) -> Result<RejectedNestingPresenceEvidence, RejectedNestingPresenceFailure> {
    let rejected_members = program
        .facts
        .iter()
        .enumerate()
        .filter_map(|(position, fact)| match fact {
            SelectorFact::RejectedForgivingMember { member, range }
                if *member == expectation.member =>
            {
                Some((position, *range))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let [(rejected_position, rejected_range)] = rejected_members.as_slice() else {
        return Err(if rejected_members.is_empty() {
            RejectedNestingPresenceFailure::RejectedMemberMissing
        } else {
            RejectedNestingPresenceFailure::RejectedMemberAmbiguous
        });
    };
    if *rejected_range != expectation.rejected_range {
        return Err(
            RejectedNestingPresenceFailure::RejectedMemberRangeMismatch {
                expected: expectation.rejected_range,
                actual: *rejected_range,
            },
        );
    }

    let rejected_facts = program.facts[rejected_position + 1..]
        .iter()
        .take_while(|fact| {
            !matches!(
                fact,
                SelectorFact::RejectedForgivingMember { .. }
                    | SelectorFact::OpenMember { .. }
                    | SelectorFact::CloseFunction { .. }
            )
        })
        .collect::<Vec<_>>();
    if rejected_facts
        .iter()
        .any(|fact| matches!(fact, SelectorFact::Relationship { .. }))
    {
        return Err(RejectedNestingPresenceFailure::ContributingRelationshipInRejectedMember);
    }

    let presences = rejected_facts
        .iter()
        .filter_map(|fact| match fact {
            SelectorFact::NestingPresence {
                member,
                unit,
                origin,
                disposition,
            } => Some((*member, *unit, *origin, *disposition)),
            _ => None,
        })
        .collect::<Vec<_>>();
    let [(member, unit, origin, disposition)] = presences.as_slice() else {
        return Err(if presences.is_empty() {
            RejectedNestingPresenceFailure::PresenceMissing
        } else {
            RejectedNestingPresenceFailure::PresenceAmbiguous
        });
    };
    if *member != expectation.member {
        return Err(RejectedNestingPresenceFailure::PresenceMemberMismatch {
            expected: expectation.member,
            actual: *member,
        });
    }
    if *unit != expectation.unit {
        return Err(RejectedNestingPresenceFailure::PresenceUnitMismatch {
            expected: expectation.unit,
            actual: *unit,
        });
    }
    let RelationshipOrigin::Authored(authored_range) = origin else {
        return Err(RejectedNestingPresenceFailure::PresenceMustBeAuthored);
    };
    if *authored_range != expectation.presence_range {
        return Err(RejectedNestingPresenceFailure::PresenceRangeMismatch {
            expected: expectation.presence_range,
            actual: *authored_range,
        });
    }
    if *disposition != NestingPresenceDisposition::NonContributingPresenceOnly {
        return Err(RejectedNestingPresenceFailure::PresenceMustBeNonContributing);
    }

    Ok(RejectedNestingPresenceEvidence {
        member: *member,
        unit: *unit,
        authored_range: *authored_range,
        disposition: *disposition,
        effect: RejectedNestingEffect::SuppressesImpliedNesting,
    })
}

pub(super) fn authored(range: AuthoredRange) -> RelationshipOrigin {
    RelationshipOrigin::Authored(range)
}

pub(super) fn derived() -> RelationshipOrigin {
    RelationshipOrigin::Derived
}

#[test]
fn csswg_authority_is_frozen_to_immutable_git_objects() {
    for id in [
        CSSWG_REVISION,
        SELECTORS_4_BLOB,
        CSS_NESTING_1_BLOB,
        CSS_SYNTAX_3_BLOB,
        CSS_NAMESPACES_3_BLOB,
        CSS_CASCADE_6_BLOB,
    ] {
        assert_eq!(id.len(), 40);
        assert!(id.bytes().all(|byte| byte.is_ascii_hexdigit()));
    }
}
