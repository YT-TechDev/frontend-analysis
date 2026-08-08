use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerResourceKind {
    SourceBytes,
    AlgorithmSteps,
    LexicalItems,
    Diagnostics,
    RetainedInterpretedBytes,
    TemporaryBufferBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTokenizerLimits {
    max_source_bytes: usize,
    max_algorithm_steps: usize,
    max_lexical_items: usize,
    max_diagnostics: usize,
    max_retained_interpreted_bytes: usize,
    max_temporary_buffer_bytes: usize,
}

impl CssTokenizerLimits {
    pub(crate) fn new(
        max_source_bytes: usize,
        max_algorithm_steps: usize,
        max_lexical_items: usize,
        max_diagnostics: usize,
        max_retained_interpreted_bytes: usize,
        max_temporary_buffer_bytes: usize,
    ) -> Result<Self, CssTokenizerInvalidConfiguration> {
        if max_algorithm_steps == 0 {
            return Err(CssTokenizerInvalidConfiguration::ZeroAlgorithmSteps);
        }

        Ok(Self {
            max_source_bytes,
            max_algorithm_steps,
            max_lexical_items,
            max_diagnostics,
            max_retained_interpreted_bytes,
            max_temporary_buffer_bytes,
        })
    }

    pub(crate) const fn limit(self, kind: CssTokenizerResourceKind) -> usize {
        match kind {
            CssTokenizerResourceKind::SourceBytes => self.max_source_bytes,
            CssTokenizerResourceKind::AlgorithmSteps => self.max_algorithm_steps,
            CssTokenizerResourceKind::LexicalItems => self.max_lexical_items,
            CssTokenizerResourceKind::Diagnostics => self.max_diagnostics,
            CssTokenizerResourceKind::RetainedInterpretedBytes => {
                self.max_retained_interpreted_bytes
            }
            CssTokenizerResourceKind::TemporaryBufferBytes => self.max_temporary_buffer_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerInvalidConfiguration {
    ZeroAlgorithmSteps,
}

impl fmt::Display for CssTokenizerInvalidConfiguration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid CSS tokenizer configuration: {self:?}")
    }
}

impl Error for CssTokenizerInvalidConfiguration {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTokenizerResourceUsage {
    source_bytes: usize,
    algorithm_steps: usize,
    lexical_items: usize,
    diagnostics: usize,
    retained_interpreted_bytes: usize,
    peak_temporary_buffer_bytes: usize,
}

impl CssTokenizerResourceUsage {
    pub(crate) const fn new(
        source_bytes: usize,
        algorithm_steps: usize,
        lexical_items: usize,
        diagnostics: usize,
        retained_interpreted_bytes: usize,
        peak_temporary_buffer_bytes: usize,
    ) -> Self {
        Self {
            source_bytes,
            algorithm_steps,
            lexical_items,
            diagnostics,
            retained_interpreted_bytes,
            peak_temporary_buffer_bytes,
        }
    }

    pub(crate) const fn value(self, kind: CssTokenizerResourceKind) -> usize {
        match kind {
            CssTokenizerResourceKind::SourceBytes => self.source_bytes,
            CssTokenizerResourceKind::AlgorithmSteps => self.algorithm_steps,
            CssTokenizerResourceKind::LexicalItems => self.lexical_items,
            CssTokenizerResourceKind::Diagnostics => self.diagnostics,
            CssTokenizerResourceKind::RetainedInterpretedBytes => self.retained_interpreted_bytes,
            CssTokenizerResourceKind::TemporaryBufferBytes => self.peak_temporary_buffer_bytes,
        }
    }
}

#[derive(Clone)]
pub(crate) struct CssTokenizerResourceLimitEvidence {
    kind: CssTokenizerResourceKind,
    limit: usize,
    attempted: usize,
    location: SourceAnchor,
}

impl CssTokenizerResourceLimitEvidence {
    pub(crate) fn new(
        source_text: &SourceText,
        kind: CssTokenizerResourceKind,
        limit: usize,
        attempted: usize,
        location: SourceAnchor,
    ) -> Result<Self, CssTokenizerResourceContractError> {
        let expected = source_text.id();
        let actual = location.source_id();
        if actual != expected {
            return Err(CssTokenizerResourceContractError::SourceIdentityMismatch {
                expected,
                actual,
            });
        }
        if !location.range().is_empty() {
            return Err(CssTokenizerResourceContractError::LocationMustBePoint {
                start: location.range().start(),
                end: location.range().end(),
            });
        }
        if attempted <= limit {
            return Err(CssTokenizerResourceContractError::AttemptDidNotExceedLimit {
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

    pub(crate) const fn kind(&self) -> CssTokenizerResourceKind {
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

impl fmt::Debug for CssTokenizerResourceLimitEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssTokenizerResourceLimitEvidence")
            .field("kind", &self.kind)
            .field("limit", &self.limit)
            .field("attempted", &self.attempted)
            .field("source_id", &self.location.source_id())
            .field("location", &self.location.range())
            .finish()
    }
}

impl PartialEq for CssTokenizerResourceLimitEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.limit == other.limit
            && self.attempted == other.attempted
            && same_anchor(&self.location, &other.location)
    }
}

impl Eq for CssTokenizerResourceLimitEvidence {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerResourceContractError {
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

impl fmt::Display for CssTokenizerResourceContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS tokenizer resource contract violation: {self:?}")
    }
}

impl Error for CssTokenizerResourceContractError {}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}
