//! Crate-private semantic contract for the first bounded ECMAScript qualification.
//!
//! This module intentionally does not implement lexical analysis, parsing, static
//! semantics, or aggregate qualification. It fixes only the semantic boundaries
//! that later implementation leaves must preserve.

use crate::{SourceAnchor, SourceText};

pub(crate) const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
pub(crate) const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
pub(crate) const UNICODE_VERSION: &str = "17.0.0";
pub(crate) const SOURCE_AUTHORITY: &str = "Core UTF-8 SourceText";
pub(crate) const SOURCE_CONTEXT: &str = "Independent Source Unit";
pub(crate) const PARSE_GOAL: &str = "Script";

/// The one qualification envelope supported by the first implementation slice.
///
/// The marker has no caller-configurable fields by design. Its identity remains
/// recoverable through the accessors below without creating a generic profile
/// configuration surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct QualificationEnvelope {
    _private: (),
}

pub(crate) const FIRST_QUALIFICATION_ENVELOPE: QualificationEnvelope =
    QualificationEnvelope { _private: () };

impl QualificationEnvelope {
    pub(crate) const fn ecma_262_edition(self) -> &'static str {
        ECMA_262_EDITION
    }

    pub(crate) const fn ecma_262_snapshot(self) -> &'static str {
        ECMA_262_SNAPSHOT
    }

    pub(crate) const fn unicode_version(self) -> &'static str {
        UNICODE_VERSION
    }

    pub(crate) const fn source_authority(self) -> &'static str {
        SOURCE_AUTHORITY
    }

    pub(crate) const fn source_context(self) -> &'static str {
        SOURCE_CONTEXT
    }

    pub(crate) const fn parse_goal(self) -> &'static str {
        PARSE_GOAL
    }

    pub(crate) const fn includes_mandatory_legacy(self) -> bool {
        true
    }

    pub(crate) const fn includes_normative_optional_source_static(self) -> bool {
        false
    }
}

/// Whether the qualification operation completed far enough to justify a
/// whole-source standard verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessingStatus {
    Complete,
    ResourceLimited,
    InternalFailure,
}

/// Whole-source semantic meaning after sufficient normal processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QualificationVerdictKind {
    Qualified,
    SyntaxRejected,
    StaticSemanticsRejected,
    ProfileBoundaryRejected,
}

/// Whether rejection evidence points at authored source or only at a derived
/// semantic fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvidenceOrigin {
    Authored,
    Derived,
}

/// A deterministic refusal to attach an authored anchor from another source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EvidenceSourceMismatch;

/// Source ownership for one piece of qualification evidence.
#[derive(Debug, Clone)]
pub(crate) enum EvidenceSubject {
    Authored(SourceAnchor),
    Derived,
}

impl EvidenceSubject {
    pub(crate) fn authored(
        source: &SourceText,
        anchor: SourceAnchor,
    ) -> Result<Self, EvidenceSourceMismatch> {
        if source.retains_exact_anchor_source(&anchor) {
            Ok(Self::Authored(anchor))
        } else {
            Err(EvidenceSourceMismatch)
        }
    }

    pub(crate) const fn derived() -> Self {
        Self::Derived
    }

    pub(crate) fn origin(&self) -> EvidenceOrigin {
        match self {
            Self::Authored(_) => EvidenceOrigin::Authored,
            Self::Derived => EvidenceOrigin::Derived,
        }
    }

    pub(crate) fn authored_anchor(&self) -> Option<&SourceAnchor> {
        match self {
            Self::Authored(anchor) => Some(anchor),
            Self::Derived => None,
        }
    }

    fn equivalent_meaning(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Authored(left), Self::Authored(right)) => {
                left.range() == right.range() && left.retains_exact_source(right)
            }
            (Self::Derived, Self::Derived) => true,
            _ => false,
        }
    }
}

/// Coarse semantic ownership for source rejection evidence.
///
/// This intentionally stops short of a durable rule-ID or diagnostic taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RejectionFamily {
    Grammar,
    StaticSemantics,
    ProfileBoundary,
}

#[derive(Debug, Clone)]
pub(crate) struct RejectionEvidence {
    family: RejectionFamily,
    subject: EvidenceSubject,
}

impl RejectionEvidence {
    fn new(family: RejectionFamily, subject: EvidenceSubject) -> Self {
        Self { family, subject }
    }

    pub(crate) fn family(&self) -> RejectionFamily {
        self.family
    }

    pub(crate) fn subject(&self) -> &EvidenceSubject {
        &self.subject
    }

    fn equivalent_meaning(&self, other: &Self) -> bool {
        self.family == other.family && self.subject.equivalent_meaning(&other.subject)
    }
}

/// Proof token required to construct a whole-source `Qualified` verdict.
///
/// No production constructor is exposed by this foundation. A later aggregate
/// owner must add the only reviewed construction path after satisfying the full
/// first-envelope prerequisite set. Parser success alone therefore cannot
/// manufacture a complete qualification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CompleteQualificationWitness {
    _private: (),
}

#[cfg(test)]
pub(crate) const fn test_complete_qualification_witness() -> CompleteQualificationWitness {
    CompleteQualificationWitness { _private: () }
}

/// Crate-private whole-source qualification contract.
///
/// Processing lifecycle and source-standard verdict are deliberately stored as
/// separate concepts. Incomplete processing never fabricates a verdict.
#[derive(Debug, Clone)]
pub(crate) struct QualificationOutcome {
    processing: ProcessingStatus,
    verdict: Option<QualificationVerdictKind>,
    rejection_evidence: Option<RejectionEvidence>,
}

impl QualificationOutcome {
    pub(crate) fn resource_limited() -> Self {
        Self {
            processing: ProcessingStatus::ResourceLimited,
            verdict: None,
            rejection_evidence: None,
        }
    }

    pub(crate) fn internal_failure() -> Self {
        Self {
            processing: ProcessingStatus::InternalFailure,
            verdict: None,
            rejection_evidence: None,
        }
    }

    pub(crate) fn syntax_rejected(subject: EvidenceSubject) -> Self {
        Self::rejected(
            QualificationVerdictKind::SyntaxRejected,
            RejectionFamily::Grammar,
            subject,
        )
    }

    pub(crate) fn static_semantics_rejected(subject: EvidenceSubject) -> Self {
        Self::rejected(
            QualificationVerdictKind::StaticSemanticsRejected,
            RejectionFamily::StaticSemantics,
            subject,
        )
    }

    pub(crate) fn profile_boundary_rejected(subject: EvidenceSubject) -> Self {
        Self::rejected(
            QualificationVerdictKind::ProfileBoundaryRejected,
            RejectionFamily::ProfileBoundary,
            subject,
        )
    }

    pub(crate) fn qualified(_witness: CompleteQualificationWitness) -> Self {
        Self {
            processing: ProcessingStatus::Complete,
            verdict: Some(QualificationVerdictKind::Qualified),
            rejection_evidence: None,
        }
    }

    fn rejected(
        verdict: QualificationVerdictKind,
        family: RejectionFamily,
        subject: EvidenceSubject,
    ) -> Self {
        Self {
            processing: ProcessingStatus::Complete,
            verdict: Some(verdict),
            rejection_evidence: Some(RejectionEvidence::new(family, subject)),
        }
    }

    pub(crate) const fn envelope(&self) -> QualificationEnvelope {
        FIRST_QUALIFICATION_ENVELOPE
    }

    pub(crate) fn processing(&self) -> ProcessingStatus {
        self.processing
    }

    pub(crate) fn verdict(&self) -> Option<QualificationVerdictKind> {
        self.verdict
    }

    pub(crate) fn rejection_evidence(&self) -> Option<&RejectionEvidence> {
        self.rejection_evidence.as_ref()
    }

    pub(crate) fn equivalent_meaning(&self, other: &Self) -> bool {
        if self.processing != other.processing || self.verdict != other.verdict {
            return false;
        }

        match (&self.rejection_evidence, &other.rejection_evidence) {
            (Some(left), Some(right)) => left.equivalent_meaning(right),
            (None, None) => true,
            _ => false,
        }
    }
}
