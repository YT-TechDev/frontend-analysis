//! Selector-semantic resource contracts (#182, extended by #405).
//!
//! These counters are deliberately independent from parser resources. The
//! selector producer must never consume parser counters as a hidden execution
//! budget. #405 adds `RetainedSemanticUnits`, the only selector-owned
//! persistent dimension bounding retained semantic-handoff evidence; the
//! existing three dimensions keep their exact #182/#184 meanings.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CssSelectorResourceKind {
    AlgorithmSteps,
    PeakSelectorDepth,
    Observations,
    RetainedSemanticUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssSelectorLimits {
    max_algorithm_steps: usize,
    max_peak_selector_depth: usize,
    max_observations: usize,
    max_retained_semantic_units: usize,
}

impl CssSelectorLimits {
    /// Configures every selector execution limit explicitly.
    ///
    /// `max_retained_semantic_units` is required rather than defaulted: #405
    /// prohibits a silently unbounded retention budget, so every selector
    /// execution call site must state the retained-semantic limit it accepts.
    pub(crate) fn new(
        max_algorithm_steps: usize,
        max_peak_selector_depth: usize,
        max_observations: usize,
        max_retained_semantic_units: usize,
    ) -> Result<Self, CssSelectorInvalidConfiguration> {
        if max_algorithm_steps == 0 {
            return Err(CssSelectorInvalidConfiguration::ZeroAlgorithmSteps);
        }
        Ok(Self {
            max_algorithm_steps,
            max_peak_selector_depth,
            max_observations,
            max_retained_semantic_units,
        })
    }

    pub(crate) const fn limit(self, kind: CssSelectorResourceKind) -> usize {
        match kind {
            CssSelectorResourceKind::AlgorithmSteps => self.max_algorithm_steps,
            CssSelectorResourceKind::PeakSelectorDepth => self.max_peak_selector_depth,
            CssSelectorResourceKind::Observations => self.max_observations,
            CssSelectorResourceKind::RetainedSemanticUnits => self.max_retained_semantic_units,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorInvalidConfiguration {
    ZeroAlgorithmSteps,
}

impl fmt::Display for CssSelectorInvalidConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid CSS selector configuration: {self:?}")
    }
}

impl Error for CssSelectorInvalidConfiguration {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssSelectorResourceUsage {
    algorithm_steps: usize,
    peak_selector_depth: usize,
    observations: usize,
    retained_semantic_units: usize,
}

impl CssSelectorResourceUsage {
    pub(crate) const fn new(
        algorithm_steps: usize,
        peak_selector_depth: usize,
        observations: usize,
        retained_semantic_units: usize,
    ) -> Self {
        Self {
            algorithm_steps,
            peak_selector_depth,
            observations,
            retained_semantic_units,
        }
    }

    pub(crate) const fn value(self, kind: CssSelectorResourceKind) -> usize {
        match kind {
            CssSelectorResourceKind::AlgorithmSteps => self.algorithm_steps,
            CssSelectorResourceKind::PeakSelectorDepth => self.peak_selector_depth,
            CssSelectorResourceKind::Observations => self.observations,
            CssSelectorResourceKind::RetainedSemanticUnits => self.retained_semantic_units,
        }
    }
}

#[derive(Clone)]
pub(crate) struct CssSelectorResourceLimitEvidence {
    kind: CssSelectorResourceKind,
    limit: usize,
    attempted: usize,
    location: SourceAnchor,
}

impl CssSelectorResourceLimitEvidence {
    pub(crate) fn new(
        source_text: &SourceText,
        kind: CssSelectorResourceKind,
        limit: usize,
        attempted: usize,
        location: SourceAnchor,
    ) -> Result<Self, CssSelectorResourceContractError> {
        let expected = source_text.id();
        let actual = location.source_id();
        if actual != expected {
            return Err(CssSelectorResourceContractError::SourceIdentityMismatch {
                expected,
                actual,
            });
        }
        if !source_text.retains_exact_anchor_source(&location) {
            return Err(CssSelectorResourceContractError::SourceContentMismatch {
                source_id: expected,
            });
        }
        if !location.range().is_empty() {
            return Err(CssSelectorResourceContractError::LocationMustBePoint {
                start: location.range().start(),
                end: location.range().end(),
            });
        }
        if attempted <= limit {
            return Err(CssSelectorResourceContractError::AttemptDidNotExceedLimit {
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

    pub(crate) const fn kind(&self) -> CssSelectorResourceKind {
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

impl fmt::Debug for CssSelectorResourceLimitEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssSelectorResourceLimitEvidence")
            .field("kind", &self.kind)
            .field("limit", &self.limit)
            .field("attempted", &self.attempted)
            .field("source_id", &self.location.source_id())
            .field("location", &self.location.range())
            .finish()
    }
}

impl PartialEq for CssSelectorResourceLimitEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.limit == other.limit
            && self.attempted == other.attempted
            && self.location.retains_exact_source(&other.location)
            && self.location.range() == other.location.range()
    }
}

impl Eq for CssSelectorResourceLimitEvidence {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorResourceContractError {
    SourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    SourceContentMismatch {
        source_id: SourceId,
    },
    LocationMustBePoint {
        start: usize,
        end: usize,
    },
    AttemptDidNotExceedLimit {
        limit: usize,
        attempted: usize,
    },
    AccountingOverflow {
        current: usize,
        delta: usize,
    },
}

impl fmt::Display for CssSelectorResourceContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS selector resource contract violation: {self:?}"
        )
    }
}

impl Error for CssSelectorResourceContractError {}

/// Checked accounting primitive for selector resource commitment.
///
/// Overflow is an internal resource-contract failure, never a semantic
/// `ResourceLimit` refusal.
pub(crate) fn checked_resource_add(
    current: usize,
    delta: usize,
) -> Result<usize, CssSelectorResourceContractError> {
    current
        .checked_add(delta)
        .ok_or(CssSelectorResourceContractError::AccountingOverflow { current, delta })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceId, SourceText};

    #[test]
    fn only_algorithm_steps_must_be_non_zero() {
        assert_eq!(
            CssSelectorLimits::new(0, 1, 1, 1),
            Err(CssSelectorInvalidConfiguration::ZeroAlgorithmSteps)
        );
        let limits = CssSelectorLimits::new(1, 0, 0, 0).unwrap();
        assert_eq!(limits.limit(CssSelectorResourceKind::PeakSelectorDepth), 0);
        assert_eq!(limits.limit(CssSelectorResourceKind::Observations), 0);
        assert_eq!(
            limits.limit(CssSelectorResourceKind::RetainedSemanticUnits),
            0
        );
    }

    #[test]
    fn retained_semantic_limit_is_explicit_and_independently_configured() {
        let limits = CssSelectorLimits::new(9, 8, 7, 6).unwrap();
        assert_eq!(limits.limit(CssSelectorResourceKind::AlgorithmSteps), 9);
        assert_eq!(limits.limit(CssSelectorResourceKind::PeakSelectorDepth), 8);
        assert_eq!(limits.limit(CssSelectorResourceKind::Observations), 7);
        assert_eq!(
            limits.limit(CssSelectorResourceKind::RetainedSemanticUnits),
            6
        );
    }

    #[test]
    fn resource_dimensions_are_independent() {
        let usage = CssSelectorResourceUsage::new(7, 3, 2, 5);
        assert_eq!(usage.value(CssSelectorResourceKind::AlgorithmSteps), 7);
        assert_eq!(usage.value(CssSelectorResourceKind::PeakSelectorDepth), 3);
        assert_eq!(usage.value(CssSelectorResourceKind::Observations), 2);
        assert_eq!(
            usage.value(CssSelectorResourceKind::RetainedSemanticUnits),
            5
        );
    }

    #[test]
    fn resource_limit_requires_a_point_and_real_refusal() {
        let source = SourceText::new(SourceId::new(20), "a{}".to_owned());
        let point = source.anchor(1, 1).unwrap();
        let evidence = CssSelectorResourceLimitEvidence::new(
            &source,
            CssSelectorResourceKind::AlgorithmSteps,
            5,
            6,
            point,
        )
        .unwrap();
        assert_eq!(evidence.limit(), 5);
        assert_eq!(evidence.attempted(), 6);
    }

    #[test]
    fn resource_limit_rejects_same_source_id_with_different_retained_bytes() {
        let source = SourceText::new(SourceId::new(21), "a{}".to_owned());
        let different = SourceText::new(SourceId::new(21), "b{}".to_owned());

        assert!(matches!(
            CssSelectorResourceLimitEvidence::new(
                &source,
                CssSelectorResourceKind::AlgorithmSteps,
                1,
                2,
                different.anchor(1, 1).unwrap(),
            ),
            Err(CssSelectorResourceContractError::SourceContentMismatch { .. })
        ));
    }

    #[test]
    fn checked_accounting_rejects_usize_overflow() {
        assert_eq!(
            checked_resource_add(usize::MAX, 1),
            Err(CssSelectorResourceContractError::AccountingOverflow {
                current: usize::MAX,
                delta: 1,
            })
        );
    }
}
