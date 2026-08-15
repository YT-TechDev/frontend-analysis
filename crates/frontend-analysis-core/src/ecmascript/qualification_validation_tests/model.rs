//! Candidate-independent validation vocabulary for Issue #197.
//!
//! This module intentionally does not import any future ECMAScript production
//! parser, syntax, diagnostic, or qualification-result type. It describes
//! validation evidence only.

pub(crate) const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
pub(crate) const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
pub(crate) const UNICODE_VERSION: &str = "17.0.0";
pub(crate) const TEST262_REVISION: &str = "3655e7464de3d52643ecddd4b5f9f4f3e7f62398";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ValidationDimension {
    Grammar,
    EarlyErrorRule,
    ValidityDependency,
    RegExpPattern,
    ProfileContext,
    UnicodeData,
    SourceProvenance,
    LifecycleVerdict,
    Determinism,
    Test262Challenge,
}

pub(crate) const REQUIRED_DIMENSIONS: [ValidationDimension; 10] = [
    ValidationDimension::Grammar,
    ValidationDimension::EarlyErrorRule,
    ValidationDimension::ValidityDependency,
    ValidationDimension::RegExpPattern,
    ValidationDimension::ProfileContext,
    ValidationDimension::UnicodeData,
    ValidationDimension::SourceProvenance,
    ValidationDimension::LifecycleVerdict,
    ValidationDimension::Determinism,
    ValidationDimension::Test262Challenge,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpectedQualification {
    Qualified,
    SyntaxRejected,
    StaticSemanticsRejected,
    ProfileBoundaryRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExpectedProcessing {
    Complete,
    ResourceLimited,
    InternalFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImplementationCoverage {
    Implemented,
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GoldRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl GoldRange {
    pub(crate) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(crate) fn is_valid_for(self, source: &str) -> bool {
        self.start <= self.end
            && self.end <= source.len()
            && source.is_char_boundary(self.start)
            && source.is_char_boundary(self.end)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyntheticKind {
    AutomaticSemicolonInsertion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SyntheticExpectation {
    pub(crate) kind: SyntheticKind,
    /// Synthetic/derived material must not receive fabricated authored ranges.
    pub(crate) authored_range: Option<GoldRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestContext {
    ScriptIndependentSource,
    Module,
    DirectEval,
    FunctionConstructor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RequestApplicability {
    Supported,
    UnsupportedGoal,
    UnsupportedSourceContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GoldFixture {
    pub(crate) id: &'static str,
    pub(crate) source: &'static str,
    pub(crate) request_context: RequestContext,
    pub(crate) applicability: RequestApplicability,
    /// A source-standard verdict exists only when processing completes far enough
    /// to justify one. Request inapplicability and resource/internal
    /// incompleteness must not fabricate a whole-source verdict.
    pub(crate) qualification: Option<ExpectedQualification>,
    pub(crate) processing: ExpectedProcessing,
    pub(crate) implementation_coverage: ImplementationCoverage,
    pub(crate) subject: Option<GoldRange>,
    pub(crate) synthetic: &'static [SyntheticExpectation],
    pub(crate) dimensions: &'static [ValidationDimension],
}
