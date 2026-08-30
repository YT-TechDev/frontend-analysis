//! Candidate-independent semantic-handoff gold for #402.
//!
//! This module deliberately does not import selector production contracts. It
//! models only the semantic information theorem accepted by #402.

#![allow(dead_code)]

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
        unit: UnitId,
        range: AuthoredRange,
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
pub(super) enum GoldOutcome {
    Qualified,
    Invalid,
    Unsupported,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CompletionState {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldObservation {
    pub(super) context: ContextId,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldFixture {
    pub(super) id: &'static str,
    pub(super) source: &'static str,
    pub(super) program: GoldProgram,
    pub(super) authored: Vec<LiteralRangeExpectation>,
}

pub(super) fn authored(range: AuthoredRange) -> RelationshipOrigin {
    RelationshipOrigin::Authored(range)
}

pub(super) fn derived() -> RelationshipOrigin {
    RelationshipOrigin::Derived
}
