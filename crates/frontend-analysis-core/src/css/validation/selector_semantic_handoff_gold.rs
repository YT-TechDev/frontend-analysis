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
