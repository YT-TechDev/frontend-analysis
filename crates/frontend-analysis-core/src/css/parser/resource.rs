//! Parser-owned resource accounting (#138).
//!
//! Deliberately smaller than the tokenizer's: no second `SourceBytes`
//! counter, and no `TemporaryBufferBytes` dimension until a real producer
//! need is demonstrated. Checkpoint attempts and speculative replay are
//! charged to `AlgorithmSteps` rather than a separate resource knob.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

/// The approved first-slice active speculative checkpoint depth invariant.
///
/// This is a fixed architectural invariant, not a configurable resource
/// limit: the future #139 producer may have at most one active speculative
/// checkpoint at a time.
pub(crate) const MAX_ACTIVE_SPECULATIVE_CHECKPOINT_DEPTH: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CssParserResourceKind {
    AlgorithmSteps,
    PeakComponentDepth,
    /// Achieved maximum number of simultaneously active authored parser
    /// contexts. The implicit stylesheet root is depth 0 and uncounted; the
    /// first top-level retained context is depth 1. Distinct from
    /// [`Self::PeakComponentDepth`], which tracks component-value nesting,
    /// not parser-context nesting (#166).
    PeakContextDepth,
    DeclarationOccurrences,
    ParserDiagnostics,
    RecoveryRecords,
    UnsupportedRegions,
    DiscardRecords,
    /// Committed/reserved durable context identities retained by the run,
    /// including partial contexts that must survive result construction
    /// (#166).
    ContextRecords,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssParserLimits {
    max_algorithm_steps: usize,
    max_peak_component_depth: usize,
    max_peak_context_depth: usize,
    max_declaration_occurrences: usize,
    max_parser_diagnostics: usize,
    max_recovery_records: usize,
    max_unsupported_regions: usize,
    max_discard_records: usize,
    max_context_records: usize,
}

impl CssParserLimits {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        max_algorithm_steps: usize,
        max_peak_component_depth: usize,
        max_peak_context_depth: usize,
        max_declaration_occurrences: usize,
        max_parser_diagnostics: usize,
        max_recovery_records: usize,
        max_unsupported_regions: usize,
        max_discard_records: usize,
        max_context_records: usize,
    ) -> Result<Self, CssParserInvalidConfiguration> {
        if max_algorithm_steps == 0 {
            return Err(CssParserInvalidConfiguration::ZeroAlgorithmSteps);
        }

        Ok(Self {
            max_algorithm_steps,
            max_peak_component_depth,
            max_peak_context_depth,
            max_declaration_occurrences,
            max_parser_diagnostics,
            max_recovery_records,
            max_unsupported_regions,
            max_discard_records,
            max_context_records,
        })
    }

    pub(crate) const fn limit(self, kind: CssParserResourceKind) -> usize {
        match kind {
            CssParserResourceKind::AlgorithmSteps => self.max_algorithm_steps,
            CssParserResourceKind::PeakComponentDepth => self.max_peak_component_depth,
            CssParserResourceKind::PeakContextDepth => self.max_peak_context_depth,
            CssParserResourceKind::DeclarationOccurrences => self.max_declaration_occurrences,
            CssParserResourceKind::ParserDiagnostics => self.max_parser_diagnostics,
            CssParserResourceKind::RecoveryRecords => self.max_recovery_records,
            CssParserResourceKind::UnsupportedRegions => self.max_unsupported_regions,
            CssParserResourceKind::DiscardRecords => self.max_discard_records,
            CssParserResourceKind::ContextRecords => self.max_context_records,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserInvalidConfiguration {
    ZeroAlgorithmSteps,
}

impl fmt::Display for CssParserInvalidConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid CSS parser configuration: {self:?}")
    }
}

impl Error for CssParserInvalidConfiguration {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssParserResourceUsage {
    algorithm_steps: usize,
    peak_component_depth: usize,
    peak_context_depth: usize,
    declaration_occurrences: usize,
    parser_diagnostics: usize,
    recovery_records: usize,
    unsupported_regions: usize,
    discard_records: usize,
    context_records: usize,
}

impl CssParserResourceUsage {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        algorithm_steps: usize,
        peak_component_depth: usize,
        peak_context_depth: usize,
        declaration_occurrences: usize,
        parser_diagnostics: usize,
        recovery_records: usize,
        unsupported_regions: usize,
        discard_records: usize,
        context_records: usize,
    ) -> Self {
        Self {
            algorithm_steps,
            peak_component_depth,
            peak_context_depth,
            declaration_occurrences,
            parser_diagnostics,
            recovery_records,
            unsupported_regions,
            discard_records,
            context_records,
        }
    }

    pub(crate) const fn value(self, kind: CssParserResourceKind) -> usize {
        match kind {
            CssParserResourceKind::AlgorithmSteps => self.algorithm_steps,
            CssParserResourceKind::PeakComponentDepth => self.peak_component_depth,
            CssParserResourceKind::PeakContextDepth => self.peak_context_depth,
            CssParserResourceKind::DeclarationOccurrences => self.declaration_occurrences,
            CssParserResourceKind::ParserDiagnostics => self.parser_diagnostics,
            CssParserResourceKind::RecoveryRecords => self.recovery_records,
            CssParserResourceKind::UnsupportedRegions => self.unsupported_regions,
            CssParserResourceKind::DiscardRecords => self.discard_records,
            CssParserResourceKind::ContextRecords => self.context_records,
        }
    }
}

#[derive(Clone)]
pub(crate) struct CssParserResourceLimitEvidence {
    kind: CssParserResourceKind,
    limit: usize,
    attempted: usize,
    location: SourceAnchor,
}

impl CssParserResourceLimitEvidence {
    pub(crate) fn new(
        source_text: &SourceText,
        kind: CssParserResourceKind,
        limit: usize,
        attempted: usize,
        location: SourceAnchor,
    ) -> Result<Self, CssParserResourceContractError> {
        let expected = source_text.id();
        let actual = location.source_id();
        if actual != expected {
            return Err(CssParserResourceContractError::SourceIdentityMismatch {
                expected,
                actual,
            });
        }
        if !location.range().is_empty() {
            return Err(CssParserResourceContractError::LocationMustBePoint {
                start: location.range().start(),
                end: location.range().end(),
            });
        }
        if attempted <= limit {
            return Err(CssParserResourceContractError::AttemptDidNotExceedLimit {
                limit,
                attempted,
            });
        }

        Ok(Self {
            kind,
            limit,
            attempted,
            location,
        })
    }

    pub(crate) const fn kind(&self) -> CssParserResourceKind {
        self.kind
    }

    pub(crate) const fn limit(&self) -> usize {
        self.limit
    }

    pub(crate) const fn attempted(&self) -> usize {
        self.attempted
    }

    pub(crate) const fn location(&self) -> &SourceAnchor {
        &self.location
    }
}

impl fmt::Debug for CssParserResourceLimitEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssParserResourceLimitEvidence")
            .field("kind", &self.kind)
            .field("limit", &self.limit)
            .field("attempted", &self.attempted)
            .field("source_id", &self.location.source_id())
            .field("location", &self.location.range())
            .finish()
    }
}

impl PartialEq for CssParserResourceLimitEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.limit == other.limit
            && self.attempted == other.attempted
            && same_anchor(&self.location, &other.location)
    }
}

impl Eq for CssParserResourceLimitEvidence {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserResourceContractError {
    SourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    LocationMustBePoint {
        start: usize,
        end: usize,
    },
    AttemptDidNotExceedLimit {
        limit: usize,
        attempted: usize,
    },
}

impl fmt::Display for CssParserResourceContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS parser resource contract violation: {self:?}"
        )
    }
}

impl Error for CssParserResourceContractError {}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(id: u64, text: &str) -> SourceText {
        SourceText::new(SourceId::new(id), text.to_owned())
    }

    #[test]
    fn zero_algorithm_steps_is_invalid_configuration() {
        let result = CssParserLimits::new(0, 10, 10, 10, 10, 10, 10, 10, 10);
        assert_eq!(
            result,
            Err(CssParserInvalidConfiguration::ZeroAlgorithmSteps)
        );
    }

    #[test]
    fn zero_capacity_limits_remain_valid_for_non_algorithm_kinds() {
        let limits = CssParserLimits::new(1, 0, 0, 0, 0, 0, 0, 0, 0).unwrap();
        assert_eq!(limits.limit(CssParserResourceKind::AlgorithmSteps), 1);
        assert_eq!(limits.limit(CssParserResourceKind::PeakComponentDepth), 0);
        assert_eq!(limits.limit(CssParserResourceKind::PeakContextDepth), 0);
        assert_eq!(
            limits.limit(CssParserResourceKind::DeclarationOccurrences),
            0
        );
        assert_eq!(limits.limit(CssParserResourceKind::DiscardRecords), 0);
        assert_eq!(limits.limit(CssParserResourceKind::ContextRecords), 0);
    }

    #[test]
    fn discard_records_limit_and_usage_are_independently_addressable() {
        let limits = CssParserLimits::new(1, 0, 0, 0, 0, 0, 0, 3, 0).unwrap();
        assert_eq!(limits.limit(CssParserResourceKind::DiscardRecords), 3);

        let usage = CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0, 2, 0);
        assert_eq!(usage.value(CssParserResourceKind::DiscardRecords), 2);
        // Unrelated dimensions are unaffected by a non-zero discard count.
        assert_eq!(usage.value(CssParserResourceKind::RecoveryRecords), 0);
        assert_eq!(usage.value(CssParserResourceKind::UnsupportedRegions), 0);
        assert_eq!(usage.value(CssParserResourceKind::PeakContextDepth), 0);
        assert_eq!(usage.value(CssParserResourceKind::ContextRecords), 0);
    }

    /// #166: `PeakContextDepth` and `ContextRecords` are addressable exactly
    /// like every other resource dimension, independent of unrelated
    /// dimensions, before any production producer allocates either.
    #[test]
    fn peak_context_depth_limit_and_usage_are_independently_addressable() {
        let limits = CssParserLimits::new(1, 0, 4, 0, 0, 0, 0, 0, 0).unwrap();
        assert_eq!(limits.limit(CssParserResourceKind::PeakContextDepth), 4);

        let usage = CssParserResourceUsage::new(1, 0, 3, 0, 0, 0, 0, 0, 0);
        assert_eq!(usage.value(CssParserResourceKind::PeakContextDepth), 3);
        assert_eq!(usage.value(CssParserResourceKind::ContextRecords), 0);
        assert_eq!(usage.value(CssParserResourceKind::PeakComponentDepth), 0);
    }

    #[test]
    fn context_records_limit_and_usage_are_independently_addressable() {
        let limits = CssParserLimits::new(1, 0, 0, 0, 0, 0, 0, 0, 5).unwrap();
        assert_eq!(limits.limit(CssParserResourceKind::ContextRecords), 5);

        let usage = CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0, 0, 4);
        assert_eq!(usage.value(CssParserResourceKind::ContextRecords), 4);
        assert_eq!(usage.value(CssParserResourceKind::PeakContextDepth), 0);
        assert_eq!(usage.value(CssParserResourceKind::DiscardRecords), 0);
    }

    /// #166: zero is a valid limit/usage for both new dimensions when no
    /// context capability is executing (the current #166 producer state).
    #[test]
    fn zero_is_valid_for_both_new_context_resource_dimensions() {
        let limits = CssParserLimits::new(1, 0, 0, 0, 0, 0, 0, 0, 0).unwrap();
        assert_eq!(limits.limit(CssParserResourceKind::PeakContextDepth), 0);
        assert_eq!(limits.limit(CssParserResourceKind::ContextRecords), 0);

        let usage = CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(usage.value(CssParserResourceKind::PeakContextDepth), 0);
        assert_eq!(usage.value(CssParserResourceKind::ContextRecords), 0);
    }

    #[test]
    fn resource_limit_evidence_requires_point_location() {
        let text = source(1, "a{color:red;}");
        let result = CssParserResourceLimitEvidence::new(
            &text,
            CssParserResourceKind::AlgorithmSteps,
            1,
            2,
            text.anchor(0, 1).unwrap(),
        );
        assert!(matches!(
            result,
            Err(CssParserResourceContractError::LocationMustBePoint { .. })
        ));
    }

    #[test]
    fn resource_limit_evidence_requires_attempted_exceeds_limit() {
        let text = source(2, "a{color:red;}");
        let result = CssParserResourceLimitEvidence::new(
            &text,
            CssParserResourceKind::DeclarationOccurrences,
            5,
            5,
            text.anchor(0, 0).unwrap(),
        );
        assert_eq!(
            result,
            Err(CssParserResourceContractError::AttemptDidNotExceedLimit {
                limit: 5,
                attempted: 5,
            })
        );
    }

    #[test]
    fn active_speculative_checkpoint_depth_invariant_is_exactly_one() {
        assert_eq!(MAX_ACTIVE_SPECULATIVE_CHECKPOINT_DEPTH, 1);
    }

    #[test]
    fn debug_output_does_not_disclose_authored_source() {
        const SECRET: &str = "secret-parser-resource-location";
        let text = source(3, SECRET);
        let error = CssParserResourceLimitEvidence::new(
            &text,
            CssParserResourceKind::AlgorithmSteps,
            1,
            1,
            text.anchor(0, 0).unwrap(),
        )
        .unwrap_err();
        assert!(!format!("{error:?}").contains(SECRET));
        assert!(!error.to_string().contains(SECRET));
    }
}
