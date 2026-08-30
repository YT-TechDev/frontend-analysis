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
    pub(super) source: SourceId,
    pub(super) run: RunId,
    pub(super) profile: &'static str,
    pub(super) context: ContextId,
    pub(super) completion: CompletionState,
    pub(super) outcome: GoldOutcome,
    pub(super) program: Option<GoldProgram>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldRun {
    pub(super) source: SourceId,
    pub(super) run: RunId,
    pub(super) profile: &'static str,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AuthoredFactExpectation {
    pub(super) fact_index: usize,
    pub(super) range: AuthoredRange,
    pub(super) spelling: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AuthoredProgramProvenanceFailure {
    DuplicateExpectation {
        fact_index: usize,
    },
    MissingExpectation {
        fact_index: usize,
    },
    ExpectationForNonAuthoredFact {
        fact_index: usize,
    },
    RangeMismatch {
        fact_index: usize,
        expected: AuthoredRange,
        actual: AuthoredRange,
    },
    Literal {
        fact_index: usize,
        failure: LiteralRangeFailure,
    },
}

fn authored_range_for_fact(fact: &SelectorFact) -> Option<AuthoredRange> {
    match *fact {
        SelectorFact::OpenMember { range, .. }
        | SelectorFact::RejectedForgivingMember { range, .. }
        | SelectorFact::Simple { range, .. }
        | SelectorFact::OpenFunction { range, .. } => Some(range),
        SelectorFact::NestingPresence {
            origin: RelationshipOrigin::Authored(range),
            ..
        }
        | SelectorFact::Relationship {
            origin: RelationshipOrigin::Authored(range),
            ..
        } => Some(range),
        SelectorFact::CloseMember { .. }
        | SelectorFact::CloseFunction { .. }
        | SelectorFact::NestingPresence {
            origin: RelationshipOrigin::Derived,
            ..
        }
        | SelectorFact::Relationship {
            origin: RelationshipOrigin::Derived,
            ..
        } => None,
    }
}

pub(super) fn validate_program_authored_provenance(
    program: &GoldProgram,
    source: &str,
    expectations: &[AuthoredFactExpectation],
) -> Result<(), AuthoredProgramProvenanceFailure> {
    for (position, expectation) in expectations.iter().enumerate() {
        if expectations[..position]
            .iter()
            .any(|earlier| earlier.fact_index == expectation.fact_index)
        {
            return Err(AuthoredProgramProvenanceFailure::DuplicateExpectation {
                fact_index: expectation.fact_index,
            });
        }
    }

    for (fact_index, fact) in program.facts.iter().enumerate() {
        let Some(actual_range) = authored_range_for_fact(fact) else {
            continue;
        };
        let Some(expectation) = expectations
            .iter()
            .find(|expectation| expectation.fact_index == fact_index)
        else {
            return Err(AuthoredProgramProvenanceFailure::MissingExpectation { fact_index });
        };
        if actual_range != expectation.range {
            return Err(AuthoredProgramProvenanceFailure::RangeMismatch {
                fact_index,
                expected: expectation.range,
                actual: actual_range,
            });
        }
        verify_literal_range(
            source,
            LiteralRangeExpectation {
                range: actual_range,
                spelling: expectation.spelling,
            },
        )
        .map_err(|failure| AuthoredProgramProvenanceFailure::Literal {
            fact_index,
            failure,
        })?;
    }

    for expectation in expectations {
        if program
            .facts
            .get(expectation.fact_index)
            .and_then(authored_range_for_fact)
            .is_none()
        {
            return Err(
                AuthoredProgramProvenanceFailure::ExpectationForNonAuthoredFact {
                    fact_index: expectation.fact_index,
                },
            );
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldFixture {
    pub(super) id: &'static str,
    pub(super) source: &'static str,
    pub(super) program: GoldProgram,
    pub(super) authored: Vec<AuthoredFactExpectation>,
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
fn actual_semantic_fact_provenance_is_exhaustively_linked_to_utf8_source() {
    let program = GoldProgram {
        source: SourceId(1),
        run: RunId(1),
        profile: "CoreV1",
        context: ContextId(20),
        facts: vec![
            SelectorFact::OpenMember {
                member: MemberId(1),
                range: AuthoredRange::new(0, 4),
            },
            SelectorFact::Simple {
                unit: UnitId(1),
                kind: SimpleKind::Type,
                range: AuthoredRange::new(0, 2),
            },
            SelectorFact::Simple {
                unit: UnitId(2),
                kind: SimpleKind::Id,
                range: AuthoredRange::new(2, 4),
            },
            SelectorFact::CloseMember {
                member: MemberId(1),
            },
        ],
    };
    let expectations = [
        AuthoredFactExpectation {
            fact_index: 0,
            range: AuthoredRange::new(0, 4),
            spelling: "é#x",
        },
        AuthoredFactExpectation {
            fact_index: 1,
            range: AuthoredRange::new(0, 2),
            spelling: "é",
        },
        AuthoredFactExpectation {
            fact_index: 2,
            range: AuthoredRange::new(2, 4),
            spelling: "#x",
        },
    ];
    assert_eq!(
        validate_program_authored_provenance(&program, "é#x", &expectations),
        Ok(())
    );

    let mut corrupted = program.clone();
    let SelectorFact::Simple { range, .. } = &mut corrupted.facts[2] else {
        panic!("fact 2 is the authored ID fact");
    };
    *range = AuthoredRange::new(1, 3);
    assert_eq!(
        validate_program_authored_provenance(&corrupted, "é#x", &expectations),
        Err(AuthoredProgramProvenanceFailure::RangeMismatch {
            fact_index: 2,
            expected: AuthoredRange::new(2, 4),
            actual: AuthoredRange::new(1, 3),
        })
    );

    let mut scalar_expectations = expectations;
    scalar_expectations[2].range = AuthoredRange::new(1, 3);
    assert_eq!(
        validate_program_authored_provenance(&corrupted, "é#x", &scalar_expectations),
        Err(AuthoredProgramProvenanceFailure::Literal {
            fact_index: 2,
            failure: LiteralRangeFailure::InvalidStartBoundary,
        })
    );

    assert_eq!(
        validate_program_authored_provenance(&program, "é#x", &expectations[..2]),
        Err(AuthoredProgramProvenanceFailure::MissingExpectation { fact_index: 2 })
    );
}

#[test]
fn actual_semantic_fact_provenance_handles_multiple_multibyte_scalars() {
    let program = GoldProgram {
        source: SourceId(1),
        run: RunId(1),
        profile: "CoreV1",
        context: ContextId(21),
        facts: vec![
            SelectorFact::OpenMember {
                member: MemberId(1),
                range: AuthoredRange::new(0, 7),
            },
            SelectorFact::Simple {
                unit: UnitId(1),
                kind: SimpleKind::Type,
                range: AuthoredRange::new(0, 5),
            },
            SelectorFact::Simple {
                unit: UnitId(2),
                kind: SimpleKind::Class,
                range: AuthoredRange::new(5, 7),
            },
            SelectorFact::CloseMember {
                member: MemberId(1),
            },
        ],
    };
    let expectations = [
        AuthoredFactExpectation {
            fact_index: 0,
            range: AuthoredRange::new(0, 7),
            spelling: "éあ.x",
        },
        AuthoredFactExpectation {
            fact_index: 1,
            range: AuthoredRange::new(0, 5),
            spelling: "éあ",
        },
        AuthoredFactExpectation {
            fact_index: 2,
            range: AuthoredRange::new(5, 7),
            spelling: ".x",
        },
    ];
    assert_eq!(
        validate_program_authored_provenance(&program, "éあ.x", &expectations),
        Ok(())
    );
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
