use crate::{SourceAnchor, SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HtmlTokenizerLimits {
    max_source_bytes: usize,
    max_transition_steps: usize,
    max_emitted_tokens: usize,
    max_diagnostics: usize,
    max_attributes_per_tag: usize,
    max_retained_interpreted_bytes: usize,
    max_temporary_buffer_bytes: usize,
}

impl HtmlTokenizerLimits {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        max_source_bytes: usize,
        max_transition_steps: usize,
        max_emitted_tokens: usize,
        max_diagnostics: usize,
        max_attributes_per_tag: usize,
        max_retained_interpreted_bytes: usize,
        max_temporary_buffer_bytes: usize,
    ) -> Self {
        Self {
            max_source_bytes,
            max_transition_steps,
            max_emitted_tokens,
            max_diagnostics,
            max_attributes_per_tag,
            max_retained_interpreted_bytes,
            max_temporary_buffer_bytes,
        }
    }

    pub(crate) const fn max_source_bytes(self) -> usize {
        self.max_source_bytes
    }

    pub(crate) const fn max_transition_steps(self) -> usize {
        self.max_transition_steps
    }

    pub(crate) const fn max_emitted_tokens(self) -> usize {
        self.max_emitted_tokens
    }

    pub(crate) const fn max_diagnostics(self) -> usize {
        self.max_diagnostics
    }

    pub(crate) const fn max_attributes_per_tag(self) -> usize {
        self.max_attributes_per_tag
    }

    pub(crate) const fn max_retained_interpreted_bytes(self) -> usize {
        self.max_retained_interpreted_bytes
    }

    pub(crate) const fn max_temporary_buffer_bytes(self) -> usize {
        self.max_temporary_buffer_bytes
    }

    pub(crate) const fn configuration_failure(self) -> Option<HtmlTokenizerConfigurationFailure> {
        if self.max_transition_steps == 0 {
            Some(HtmlTokenizerConfigurationFailure::ZeroTransitionStepLimit)
        } else if self.max_emitted_tokens == 0 {
            Some(HtmlTokenizerConfigurationFailure::ZeroEmittedTokenLimit)
        } else {
            None
        }
    }

    pub(crate) const fn limit_for(self, resource: HtmlTokenizerResource) -> usize {
        match resource {
            HtmlTokenizerResource::SourceBytes => self.max_source_bytes,
            HtmlTokenizerResource::TransitionSteps => self.max_transition_steps,
            HtmlTokenizerResource::EmittedTokens => self.max_emitted_tokens,
            HtmlTokenizerResource::Diagnostics => self.max_diagnostics,
            HtmlTokenizerResource::AttributesPerTag => self.max_attributes_per_tag,
            HtmlTokenizerResource::RetainedInterpretedBytes => self.max_retained_interpreted_bytes,
            HtmlTokenizerResource::TemporaryBufferBytes => self.max_temporary_buffer_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HtmlTokenizerUsage {
    source_bytes: usize,
    transition_steps: usize,
    emitted_tokens: usize,
    diagnostics: usize,
    peak_attributes_per_tag: usize,
    retained_interpreted_bytes: usize,
    peak_temporary_buffer_bytes: usize,
}

impl HtmlTokenizerUsage {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        source_bytes: usize,
        transition_steps: usize,
        emitted_tokens: usize,
        diagnostics: usize,
        peak_attributes_per_tag: usize,
        retained_interpreted_bytes: usize,
        peak_temporary_buffer_bytes: usize,
    ) -> Self {
        Self {
            source_bytes,
            transition_steps,
            emitted_tokens,
            diagnostics,
            peak_attributes_per_tag,
            retained_interpreted_bytes,
            peak_temporary_buffer_bytes,
        }
    }

    pub(crate) const fn source_bytes(self) -> usize {
        self.source_bytes
    }

    pub(crate) const fn transition_steps(self) -> usize {
        self.transition_steps
    }

    pub(crate) const fn emitted_tokens(self) -> usize {
        self.emitted_tokens
    }

    pub(crate) const fn diagnostics(self) -> usize {
        self.diagnostics
    }

    pub(crate) const fn peak_attributes_per_tag(self) -> usize {
        self.peak_attributes_per_tag
    }

    pub(crate) const fn retained_interpreted_bytes(self) -> usize {
        self.retained_interpreted_bytes
    }

    pub(crate) const fn peak_temporary_buffer_bytes(self) -> usize {
        self.peak_temporary_buffer_bytes
    }

    pub(crate) const fn value_for(self, resource: HtmlTokenizerResource) -> usize {
        match resource {
            HtmlTokenizerResource::SourceBytes => self.source_bytes,
            HtmlTokenizerResource::TransitionSteps => self.transition_steps,
            HtmlTokenizerResource::EmittedTokens => self.emitted_tokens,
            HtmlTokenizerResource::Diagnostics => self.diagnostics,
            HtmlTokenizerResource::AttributesPerTag => self.peak_attributes_per_tag,
            HtmlTokenizerResource::RetainedInterpretedBytes => self.retained_interpreted_bytes,
            HtmlTokenizerResource::TemporaryBufferBytes => self.peak_temporary_buffer_bytes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerResource {
    SourceBytes,
    TransitionSteps,
    EmittedTokens,
    Diagnostics,
    AttributesPerTag,
    RetainedInterpretedBytes,
    TemporaryBufferBytes,
}

impl HtmlTokenizerResource {
    pub(crate) const ALL: [Self; 7] = [
        Self::SourceBytes,
        Self::TransitionSteps,
        Self::EmittedTokens,
        Self::Diagnostics,
        Self::AttributesPerTag,
        Self::RetainedInterpretedBytes,
        Self::TemporaryBufferBytes,
    ];
}

#[derive(Clone)]
pub(crate) struct HtmlTokenizerResourceLimit {
    resource: HtmlTokenizerResource,
    limit: usize,
    attempted: usize,
    at: SourceAnchor,
}

impl HtmlTokenizerResourceLimit {
    pub(crate) fn new(
        source_text: &SourceText,
        resource: HtmlTokenizerResource,
        limit: usize,
        attempted: usize,
        at: SourceAnchor,
    ) -> Result<Self, HtmlTokenizerResourceContractError> {
        if at.source_id() != source_text.id() {
            return Err(HtmlTokenizerResourceContractError::SourceIdentityMismatch {
                expected: source_text.id(),
                actual: at.source_id(),
            });
        }
        if attempted <= limit {
            return Err(
                HtmlTokenizerResourceContractError::AttemptedMustExceedLimit { limit, attempted },
            );
        }
        Ok(Self {
            resource,
            limit,
            attempted,
            at,
        })
    }

    pub(crate) fn resource(&self) -> HtmlTokenizerResource {
        self.resource
    }

    pub(crate) fn limit(&self) -> usize {
        self.limit
    }

    pub(crate) fn attempted(&self) -> usize {
        self.attempted
    }

    pub(crate) fn at(&self) -> &SourceAnchor {
        &self.at
    }
}

impl std::fmt::Debug for HtmlTokenizerResourceLimit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HtmlTokenizerResourceLimit")
            .field("resource", &self.resource)
            .field("limit", &self.limit)
            .field("attempted", &self.attempted)
            .field("source_id", &self.at.source_id())
            .field("range", &self.at.range())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerConfigurationFailure {
    ZeroTransitionStepLimit,
    ZeroEmittedTokenLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerInvariantFailure {
    CursorState,
    ReconsumeState,
    BuilderState,
    EmissionOrder,
    SourceEvidenceConstruction,
    CounterOverflow,
    RunAssembly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerResourceContractError {
    SourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    AttemptedMustExceedLimit {
        limit: usize,
        attempted: usize,
    },
}

impl std::fmt::Display for HtmlTokenizerResourceContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "HTML tokenizer resource contract violation: {self:?}"
        )
    }
}

impl std::error::Error for HtmlTokenizerResourceContractError {}
