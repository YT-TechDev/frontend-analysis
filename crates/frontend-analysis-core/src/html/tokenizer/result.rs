use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

use super::diagnostic::{
    HtmlTokenizerDiagnostic, HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticHandling,
    HtmlTokenizerDiagnosticSubject,
};
use super::resource::{
    HtmlTokenizerConfigurationFailure, HtmlTokenizerInvariantFailure, HtmlTokenizerLimits,
    HtmlTokenizerResource, HtmlTokenizerResourceLimit, HtmlTokenizerUsage,
};
use crate::html::token::{HtmlPreprocessingEvidence, HtmlTagKind, HtmlToken};

#[derive(Clone)]
pub(crate) struct HtmlTokenizerCoverage {
    processed_prefix: SourceAnchor,
    unprocessed_suffix: SourceAnchor,
}

impl HtmlTokenizerCoverage {
    pub(crate) fn new(
        source_text: &SourceText,
        processed_prefix: SourceAnchor,
        unprocessed_suffix: SourceAnchor,
    ) -> Result<Self, HtmlTokenizerRunContractError> {
        for (role, anchor) in [
            (HtmlRunEvidenceRole::ProcessedPrefix, &processed_prefix),
            (HtmlRunEvidenceRole::UnprocessedSuffix, &unprocessed_suffix),
        ] {
            require_source(source_text.id(), anchor, role)?;
        }
        if processed_prefix.range().start() != 0 {
            return Err(HtmlTokenizerRunContractError::CoverageMustStartAtZero);
        }
        if unprocessed_suffix.range().end() != source_text.as_str().len() {
            return Err(HtmlTokenizerRunContractError::CoverageMustEndAtSourceEnd);
        }
        if processed_prefix.range().end() != unprocessed_suffix.range().start() {
            return Err(HtmlTokenizerRunContractError::CoverageMustPartitionSource);
        }
        Ok(Self {
            processed_prefix,
            unprocessed_suffix,
        })
    }

    pub(crate) fn processed_prefix(&self) -> &SourceAnchor {
        &self.processed_prefix
    }

    pub(crate) fn unprocessed_suffix(&self) -> &SourceAnchor {
        &self.unprocessed_suffix
    }

    pub(crate) fn processed_end(&self) -> usize {
        self.processed_prefix.range().end()
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.unprocessed_suffix.range().is_empty()
    }
}

impl fmt::Debug for HtmlTokenizerCoverage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTokenizerCoverage")
            .field("source_id", &self.processed_prefix.source_id())
            .field("processed_prefix", &self.processed_prefix.range())
            .field("unprocessed_suffix", &self.unprocessed_suffix.range())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerCapabilityAvailability {
    Deferred,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlCharacterReferenceContext {
    Data,
    AttributeValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerMode {
    Rcdata,
    RawText,
    ScriptData,
    Plaintext,
    Noscript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerCapability {
    CharacterReference {
        context: HtmlCharacterReferenceContext,
    },
    MarkupDeclaration,
    ProcessingInstruction,
    BogusCommentRecovery,
    ContextDependentTokenizerMode {
        mode: HtmlTokenizerMode,
    },
    TreeConstructionControlledState,
    ForeignContentControl,
}

#[derive(Clone)]
pub(crate) enum HtmlTokenizerUnsupportedTrigger {
    Input(SourceAnchor),
    EmittedToken {
        token_index: usize,
        boundary: SourceAnchor,
    },
}

impl HtmlTokenizerUnsupportedTrigger {
    fn anchor(&self) -> &SourceAnchor {
        match self {
            Self::Input(anchor)
            | Self::EmittedToken {
                boundary: anchor, ..
            } => anchor,
        }
    }
}

impl fmt::Debug for HtmlTokenizerUnsupportedTrigger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(anchor) => formatter
                .debug_struct("Input")
                .field("source_id", &anchor.source_id())
                .field("range", &anchor.range())
                .finish(),
            Self::EmittedToken {
                token_index,
                boundary,
            } => formatter
                .debug_struct("EmittedToken")
                .field("token_index", token_index)
                .field("source_id", &boundary.source_id())
                .field("range", &boundary.range())
                .finish(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct HtmlTokenizerUnsupportedCapability {
    capability: HtmlTokenizerCapability,
    availability: HtmlTokenizerCapabilityAvailability,
    trigger: HtmlTokenizerUnsupportedTrigger,
}

impl HtmlTokenizerUnsupportedCapability {
    pub(crate) fn new(
        source_text: &SourceText,
        capability: HtmlTokenizerCapability,
        availability: HtmlTokenizerCapabilityAvailability,
        trigger: HtmlTokenizerUnsupportedTrigger,
    ) -> Result<Self, HtmlTokenizerRunContractError> {
        require_source(
            source_text.id(),
            trigger.anchor(),
            HtmlRunEvidenceRole::UnsupportedTrigger,
        )?;
        Ok(Self {
            capability,
            availability,
            trigger,
        })
    }

    pub(crate) fn capability(&self) -> HtmlTokenizerCapability {
        self.capability
    }

    pub(crate) fn availability(&self) -> HtmlTokenizerCapabilityAvailability {
        self.availability
    }

    pub(crate) fn trigger(&self) -> &HtmlTokenizerUnsupportedTrigger {
        &self.trigger
    }
}

impl fmt::Debug for HtmlTokenizerUnsupportedCapability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTokenizerUnsupportedCapability")
            .field("capability", &self.capability)
            .field("availability", &self.availability)
            .field("trigger", &self.trigger)
            .finish()
    }
}

#[derive(Clone)]
pub(crate) enum HtmlTokenizerIncompleteCause {
    UnsupportedCapability(HtmlTokenizerUnsupportedCapability),
    ResourceLimit(HtmlTokenizerResourceLimit),
    InvalidConfiguration(HtmlTokenizerConfigurationFailure),
    InternalInvariantFailure(HtmlTokenizerInvariantFailure),
}

impl fmt::Debug for HtmlTokenizerIncompleteCause {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedCapability(value) => formatter
                .debug_tuple("UnsupportedCapability")
                .field(value)
                .finish(),
            Self::ResourceLimit(value) => {
                formatter.debug_tuple("ResourceLimit").field(value).finish()
            }
            Self::InvalidConfiguration(value) => formatter
                .debug_tuple("InvalidConfiguration")
                .field(value)
                .finish(),
            Self::InternalInvariantFailure(value) => formatter
                .debug_tuple("InternalInvariantFailure")
                .field(value)
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum HtmlTokenizerCompletion {
    Complete,
    Incomplete(HtmlTokenizerIncompleteCause),
}

#[derive(Clone)]
pub(crate) struct HtmlTokenizerRunResult {
    tokens: Vec<HtmlToken>,
    preprocessing: HtmlPreprocessingEvidence,
    diagnostics: Vec<HtmlTokenizerDiagnostic>,
    coverage: HtmlTokenizerCoverage,
    completion: HtmlTokenizerCompletion,
    limits: HtmlTokenizerLimits,
    usage: HtmlTokenizerUsage,
}

impl HtmlTokenizerRunResult {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        tokens: Vec<HtmlToken>,
        preprocessing: HtmlPreprocessingEvidence,
        diagnostics: Vec<HtmlTokenizerDiagnostic>,
        coverage: HtmlTokenizerCoverage,
        completion: HtmlTokenizerCompletion,
        limits: HtmlTokenizerLimits,
        usage: HtmlTokenizerUsage,
    ) -> Result<Self, HtmlTokenizerRunContractError> {
        validate_configuration_cause(limits, &completion)?;
        validate_coverage_identity(source_text, &coverage)?;
        validate_preprocessing(source_text, &preprocessing, &coverage)?;
        validate_tokens(source_text, &tokens, &coverage, &completion)?;
        validate_diagnostics(source_text, &tokens, &diagnostics, &coverage, &completion)?;
        validate_usage(
            source_text,
            &tokens,
            &diagnostics,
            limits,
            usage,
            &completion,
        )?;
        validate_completion(
            source_text,
            &tokens,
            &diagnostics,
            &coverage,
            limits,
            usage,
            &completion,
        )?;

        Ok(Self {
            tokens,
            preprocessing,
            diagnostics,
            coverage,
            completion,
            limits,
            usage,
        })
    }

    pub(crate) fn tokens(&self) -> &[HtmlToken] {
        &self.tokens
    }

    pub(crate) fn preprocessing(&self) -> &HtmlPreprocessingEvidence {
        &self.preprocessing
    }

    pub(crate) fn diagnostics(&self) -> &[HtmlTokenizerDiagnostic] {
        &self.diagnostics
    }

    pub(crate) fn coverage(&self) -> &HtmlTokenizerCoverage {
        &self.coverage
    }

    pub(crate) fn completion(&self) -> &HtmlTokenizerCompletion {
        &self.completion
    }

    pub(crate) fn limits(&self) -> HtmlTokenizerLimits {
        self.limits
    }

    pub(crate) fn usage(&self) -> HtmlTokenizerUsage {
        self.usage
    }

    pub(crate) fn is_clean_complete(&self) -> bool {
        matches!(&self.completion, HtmlTokenizerCompletion::Complete) && self.diagnostics.is_empty()
    }

    pub(crate) fn is_complete_with_diagnostics(&self) -> bool {
        matches!(&self.completion, HtmlTokenizerCompletion::Complete)
            && !self.diagnostics.is_empty()
    }

    pub(crate) fn is_incomplete(&self) -> bool {
        matches!(&self.completion, HtmlTokenizerCompletion::Incomplete(_))
    }
}

impl fmt::Debug for HtmlTokenizerRunResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTokenizerRunResult")
            .field("token_count", &self.tokens.len())
            .field("preprocessing", &self.preprocessing)
            .field("diagnostics", &self.diagnostics)
            .field("coverage", &self.coverage)
            .field("completion", &self.completion)
            .field("limits", &self.limits)
            .field("usage", &self.usage)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlRunEvidenceRole {
    ProcessedPrefix,
    UnprocessedSuffix,
    PreprocessingBom,
    Token,
    Diagnostic,
    DiagnosticAbandonedRegion,
    UnsupportedTrigger,
    ResourceLimitLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerRunContractError {
    SourceIdentityMismatch {
        role: HtmlRunEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    CoverageMustStartAtZero,
    CoverageMustEndAtSourceEnd,
    CoverageMustPartitionSource,
    PreprocessingOutsideProcessedPrefix,
    CompleteCoverageRequired,
    IncompleteResultMustNotContainEof,
    CompleteResultMustContainExactlyOneEof,
    EofMustBeLast,
    TokenOrderOrOverlap,
    TokenOutsideProcessedPrefix {
        token_index: usize,
    },
    DiagnosticOrder,
    DiagnosticOutsideProcessedPrefix {
        diagnostic_index: usize,
    },
    StoppedDiagnosticRequiresIncompleteResult {
        diagnostic_index: usize,
    },
    StoppedDiagnosticMustBeLast {
        diagnostic_index: usize,
    },
    InvalidDiagnosticTokenReference {
        diagnostic_index: usize,
        token_index: usize,
    },
    DiagnosticMustNotReferenceEof {
        diagnostic_index: usize,
        token_index: usize,
    },
    EmissionConditionedSubjectMustBeEmittedToken {
        diagnostic_index: usize,
        code: HtmlTokenizerDiagnosticCode,
    },
    EmissionConditionedTokenMustBeEndTag {
        diagnostic_index: usize,
        token_index: usize,
        code: HtmlTokenizerDiagnosticCode,
    },
    EmissionConditionedEvidenceMissing {
        diagnostic_index: usize,
        token_index: usize,
        code: HtmlTokenizerDiagnosticCode,
    },
    EmissionConditionedDiagnosticOutsideReferencedTag {
        diagnostic_index: usize,
        token_index: usize,
        code: HtmlTokenizerDiagnosticCode,
    },
    UnsupportedInputMustStartAtCoverageBoundary,
    InvalidUnsupportedTokenReference {
        token_index: usize,
    },
    UnsupportedTriggerMustReferenceNonEofToken {
        token_index: usize,
    },
    UnsupportedTokenBoundaryOutsideProcessedPrefix,
    UnsupportedTokenBoundaryMustFollowReferencedToken,
    ConfigurationFailureMismatch,
    InvalidConfigurationMustPrecedeProcessing,
    ResourceLimitPolicyMismatch,
    ResourceLimitAttemptMismatch,
    ResourceLimitLocationAfterProcessedPrefix,
    UsageSourceBytesMismatch {
        expected: usize,
        actual: usize,
    },
    UsageTokenCountMismatch {
        expected: usize,
        actual: usize,
    },
    UsageDiagnosticCountMismatch {
        expected: usize,
        actual: usize,
    },
    UsagePeakAttributesTooSmall {
        minimum: usize,
        actual: usize,
    },
    UsageRetainedInterpretedBytesTooSmall {
        minimum: usize,
        actual: usize,
    },
    UsageExceedsLimit {
        resource: HtmlTokenizerResource,
        usage: usize,
        limit: usize,
    },
    CounterOverflow,
}

impl fmt::Display for HtmlTokenizerRunContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HTML tokenizer run contract violation: {self:?}")
    }
}

impl Error for HtmlTokenizerRunContractError {}

fn validate_configuration_cause(
    limits: HtmlTokenizerLimits,
    completion: &HtmlTokenizerCompletion,
) -> Result<(), HtmlTokenizerRunContractError> {
    match (limits.configuration_failure(), completion) {
        (
            Some(expected),
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::InvalidConfiguration(actual),
            ),
        ) if expected == *actual => Ok(()),
        (Some(_), _) => Err(HtmlTokenizerRunContractError::ConfigurationFailureMismatch),
        (
            None,
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::InvalidConfiguration(_),
            ),
        ) => Err(HtmlTokenizerRunContractError::ConfigurationFailureMismatch),
        (None, _) => Ok(()),
    }
}

fn validate_preprocessing(
    source_text: &SourceText,
    preprocessing: &HtmlPreprocessingEvidence,
    coverage: &HtmlTokenizerCoverage,
) -> Result<(), HtmlTokenizerRunContractError> {
    if let Some(bom) = preprocessing.skipped_leading_bom() {
        require_source(source_text.id(), bom, HtmlRunEvidenceRole::PreprocessingBom)?;
        if bom.range().end() > coverage.processed_end() {
            return Err(HtmlTokenizerRunContractError::PreprocessingOutsideProcessedPrefix);
        }
    }
    Ok(())
}

fn validate_coverage_identity(
    source_text: &SourceText,
    coverage: &HtmlTokenizerCoverage,
) -> Result<(), HtmlTokenizerRunContractError> {
    for (role, anchor) in [
        (
            HtmlRunEvidenceRole::ProcessedPrefix,
            coverage.processed_prefix(),
        ),
        (
            HtmlRunEvidenceRole::UnprocessedSuffix,
            coverage.unprocessed_suffix(),
        ),
    ] {
        require_source(source_text.id(), anchor, role)?;
    }
    if coverage.processed_prefix().range().start() != 0 {
        return Err(HtmlTokenizerRunContractError::CoverageMustStartAtZero);
    }
    if coverage.unprocessed_suffix().range().end() != source_text.as_str().len() {
        return Err(HtmlTokenizerRunContractError::CoverageMustEndAtSourceEnd);
    }
    if coverage.processed_end() != coverage.unprocessed_suffix().range().start() {
        return Err(HtmlTokenizerRunContractError::CoverageMustPartitionSource);
    }
    Ok(())
}

fn validate_tokens(
    source_text: &SourceText,
    tokens: &[HtmlToken],
    coverage: &HtmlTokenizerCoverage,
    completion: &HtmlTokenizerCompletion,
) -> Result<(), HtmlTokenizerRunContractError> {
    let mut previous_end = 0;
    let mut eof_indices = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        let anchor = token_anchor(token);
        require_source(source_text.id(), anchor, HtmlRunEvidenceRole::Token)?;
        if anchor.range().start() < previous_end {
            return Err(HtmlTokenizerRunContractError::TokenOrderOrOverlap);
        }
        if !matches!(token, HtmlToken::EndOfFile(_))
            && anchor.range().end() > coverage.processed_end()
        {
            return Err(HtmlTokenizerRunContractError::TokenOutsideProcessedPrefix {
                token_index: index,
            });
        }
        if matches!(token, HtmlToken::EndOfFile(_)) {
            eof_indices.push(index);
        }
        previous_end = anchor.range().end();
    }

    match completion {
        HtmlTokenizerCompletion::Complete => {
            if eof_indices.len() != 1 {
                return Err(HtmlTokenizerRunContractError::CompleteResultMustContainExactlyOneEof);
            }
            if eof_indices[0] + 1 != tokens.len() {
                return Err(HtmlTokenizerRunContractError::EofMustBeLast);
            }
        }
        HtmlTokenizerCompletion::Incomplete(_) => {
            if !eof_indices.is_empty() {
                return Err(HtmlTokenizerRunContractError::IncompleteResultMustNotContainEof);
            }
        }
    }
    Ok(())
}

fn validate_diagnostics(
    source_text: &SourceText,
    tokens: &[HtmlToken],
    diagnostics: &[HtmlTokenizerDiagnostic],
    coverage: &HtmlTokenizerCoverage,
    completion: &HtmlTokenizerCompletion,
) -> Result<(), HtmlTokenizerRunContractError> {
    let mut previous_start = 0;
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        let location = diagnostic.location();
        require_source(source_text.id(), location, HtmlRunEvidenceRole::Diagnostic)?;
        if index > 0 && location.range().start() < previous_start {
            return Err(HtmlTokenizerRunContractError::DiagnosticOrder);
        }
        if location.range().end() > coverage.processed_end() {
            return Err(
                HtmlTokenizerRunContractError::DiagnosticOutsideProcessedPrefix {
                    diagnostic_index: index,
                },
            );
        }
        previous_start = location.range().start();

        if matches!(
            diagnostic.handling(),
            HtmlTokenizerDiagnosticHandling::Stopped
        ) {
            if matches!(completion, HtmlTokenizerCompletion::Complete) {
                return Err(
                    HtmlTokenizerRunContractError::StoppedDiagnosticRequiresIncompleteResult {
                        diagnostic_index: index,
                    },
                );
            }
            if index + 1 != diagnostics.len() {
                return Err(HtmlTokenizerRunContractError::StoppedDiagnosticMustBeLast {
                    diagnostic_index: index,
                });
            }
        }

        if diagnostic.code().is_emission_conditioned()
            && !matches!(
                diagnostic.subject(),
                HtmlTokenizerDiagnosticSubject::EmittedToken { .. }
            )
        {
            return Err(
                HtmlTokenizerRunContractError::EmissionConditionedSubjectMustBeEmittedToken {
                    diagnostic_index: index,
                    code: diagnostic.code(),
                },
            );
        }

        match diagnostic.subject() {
            HtmlTokenizerDiagnosticSubject::InputLocation => {}
            HtmlTokenizerDiagnosticSubject::EmittedToken { token_index } => {
                let Some(token) = tokens.get(*token_index) else {
                    return Err(
                        HtmlTokenizerRunContractError::InvalidDiagnosticTokenReference {
                            diagnostic_index: index,
                            token_index: *token_index,
                        },
                    );
                };
                if matches!(token, HtmlToken::EndOfFile(_)) {
                    return Err(
                        HtmlTokenizerRunContractError::DiagnosticMustNotReferenceEof {
                            diagnostic_index: index,
                            token_index: *token_index,
                        },
                    );
                }
                if diagnostic.code().is_emission_conditioned() {
                    validate_emission_conditioned_evidence(
                        index,
                        diagnostic.code(),
                        *token_index,
                        token,
                        location,
                    )?;
                }
            }
            HtmlTokenizerDiagnosticSubject::AbandonedInput { region } => {
                require_source(
                    source_text.id(),
                    region,
                    HtmlRunEvidenceRole::DiagnosticAbandonedRegion,
                )?;
                if region.range().end() > coverage.processed_end() {
                    return Err(
                        HtmlTokenizerRunContractError::DiagnosticOutsideProcessedPrefix {
                            diagnostic_index: index,
                        },
                    );
                }
            }
        }
    }
    Ok(())
}

/// Validates that an emission-conditioned diagnostic's `EmittedToken` subject
/// resolves to the corresponding end-tag token: one that both carries the
/// structured evidence its code requires and whose complete authored range
/// contains the diagnostic's own location. Containment prevents a diagnostic
/// from correctly referencing a *different* qualifying end tag elsewhere in
/// the run. Uses only structured #110 token evidence, never source text or
/// string parsing.
fn validate_emission_conditioned_evidence(
    diagnostic_index: usize,
    code: HtmlTokenizerDiagnosticCode,
    token_index: usize,
    token: &HtmlToken,
    location: &SourceAnchor,
) -> Result<(), HtmlTokenizerRunContractError> {
    let HtmlToken::Tag(tag) = token else {
        return Err(
            HtmlTokenizerRunContractError::EmissionConditionedTokenMustBeEndTag {
                diagnostic_index,
                token_index,
                code,
            },
        );
    };
    if tag.kind() != HtmlTagKind::End {
        return Err(
            HtmlTokenizerRunContractError::EmissionConditionedTokenMustBeEndTag {
                diagnostic_index,
                token_index,
                code,
            },
        );
    }
    let evidence_present = match code {
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes => !tag.attributes().is_empty(),
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus => {
            tag.self_closing_solidus().is_some()
        }
        _ => false,
    };
    if !evidence_present {
        return Err(
            HtmlTokenizerRunContractError::EmissionConditionedEvidenceMissing {
                diagnostic_index,
                token_index,
                code,
            },
        );
    }
    let complete = tag.complete();
    if complete.source_id() != location.source_id()
        || location.range().start() < complete.range().start()
        || location.range().end() > complete.range().end()
    {
        return Err(
            HtmlTokenizerRunContractError::EmissionConditionedDiagnosticOutsideReferencedTag {
                diagnostic_index,
                token_index,
                code,
            },
        );
    }
    Ok(())
}

fn validate_usage(
    source_text: &SourceText,
    tokens: &[HtmlToken],
    diagnostics: &[HtmlTokenizerDiagnostic],
    limits: HtmlTokenizerLimits,
    usage: HtmlTokenizerUsage,
    completion: &HtmlTokenizerCompletion,
) -> Result<(), HtmlTokenizerRunContractError> {
    let source_len = source_text.as_str().len();
    if usage.source_bytes() != source_len {
        return Err(HtmlTokenizerRunContractError::UsageSourceBytesMismatch {
            expected: source_len,
            actual: usage.source_bytes(),
        });
    }
    if usage.emitted_tokens() != tokens.len() {
        return Err(HtmlTokenizerRunContractError::UsageTokenCountMismatch {
            expected: tokens.len(),
            actual: usage.emitted_tokens(),
        });
    }
    if usage.diagnostics() != diagnostics.len() {
        return Err(
            HtmlTokenizerRunContractError::UsageDiagnosticCountMismatch {
                expected: diagnostics.len(),
                actual: usage.diagnostics(),
            },
        );
    }

    let minimum_attributes = tokens
        .iter()
        .filter_map(|token| match token {
            HtmlToken::Tag(tag) => Some(tag.attributes().len()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    if usage.peak_attributes_per_tag() < minimum_attributes {
        return Err(HtmlTokenizerRunContractError::UsagePeakAttributesTooSmall {
            minimum: minimum_attributes,
            actual: usage.peak_attributes_per_tag(),
        });
    }

    let minimum_interpreted = tokens.iter().try_fold(0usize, |total, token| {
        total
            .checked_add(token_interpreted_bytes(token)?)
            .ok_or(HtmlTokenizerRunContractError::CounterOverflow)
    })?;
    if usage.retained_interpreted_bytes() < minimum_interpreted {
        return Err(
            HtmlTokenizerRunContractError::UsageRetainedInterpretedBytesTooSmall {
                minimum: minimum_interpreted,
                actual: usage.retained_interpreted_bytes(),
            },
        );
    }

    let skipped_resource = match completion {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(limit)) => {
            Some(limit.resource())
        }
        HtmlTokenizerCompletion::Incomplete(
            HtmlTokenizerIncompleteCause::InvalidConfiguration(_),
        ) => Some(HtmlTokenizerResource::SourceBytes),
        _ => None,
    };

    for resource in HtmlTokenizerResource::ALL {
        if skipped_resource == Some(resource) {
            continue;
        }
        let value = usage.value_for(resource);
        let limit = limits.limit_for(resource);
        if value > limit {
            return Err(HtmlTokenizerRunContractError::UsageExceedsLimit {
                resource,
                usage: value,
                limit,
            });
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_completion(
    source_text: &SourceText,
    tokens: &[HtmlToken],
    diagnostics: &[HtmlTokenizerDiagnostic],
    coverage: &HtmlTokenizerCoverage,
    limits: HtmlTokenizerLimits,
    usage: HtmlTokenizerUsage,
    completion: &HtmlTokenizerCompletion,
) -> Result<(), HtmlTokenizerRunContractError> {
    match completion {
        HtmlTokenizerCompletion::Complete => {
            if !coverage.is_complete() {
                return Err(HtmlTokenizerRunContractError::CompleteCoverageRequired);
            }
        }
        HtmlTokenizerCompletion::Incomplete(cause) => match cause {
            HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported) => {
                validate_unsupported(source_text, tokens, coverage, unsupported)?;
            }
            HtmlTokenizerIncompleteCause::ResourceLimit(limit) => {
                validate_resource_limit(source_text, coverage, limits, usage, limit)?;
            }
            HtmlTokenizerIncompleteCause::InvalidConfiguration(_) => {
                if coverage.processed_end() != 0
                    || !tokens.is_empty()
                    || !diagnostics.is_empty()
                    || usage.transition_steps() != 0
                    || usage.emitted_tokens() != 0
                    || usage.diagnostics() != 0
                    || usage.peak_attributes_per_tag() != 0
                    || usage.retained_interpreted_bytes() != 0
                    || usage.peak_temporary_buffer_bytes() != 0
                {
                    return Err(
                        HtmlTokenizerRunContractError::InvalidConfigurationMustPrecedeProcessing,
                    );
                }
            }
            HtmlTokenizerIncompleteCause::InternalInvariantFailure(_) => {}
        },
    }
    Ok(())
}

fn validate_unsupported(
    source_text: &SourceText,
    tokens: &[HtmlToken],
    coverage: &HtmlTokenizerCoverage,
    unsupported: &HtmlTokenizerUnsupportedCapability,
) -> Result<(), HtmlTokenizerRunContractError> {
    require_source(
        source_text.id(),
        unsupported.trigger().anchor(),
        HtmlRunEvidenceRole::UnsupportedTrigger,
    )?;
    match unsupported.trigger() {
        HtmlTokenizerUnsupportedTrigger::Input(anchor) => {
            if anchor.range().start() != coverage.processed_end() {
                return Err(
                    HtmlTokenizerRunContractError::UnsupportedInputMustStartAtCoverageBoundary,
                );
            }
        }
        HtmlTokenizerUnsupportedTrigger::EmittedToken {
            token_index,
            boundary,
        } => {
            let Some(token) = tokens.get(*token_index) else {
                return Err(
                    HtmlTokenizerRunContractError::InvalidUnsupportedTokenReference {
                        token_index: *token_index,
                    },
                );
            };
            if matches!(token, HtmlToken::EndOfFile(_)) {
                return Err(
                    HtmlTokenizerRunContractError::UnsupportedTriggerMustReferenceNonEofToken {
                        token_index: *token_index,
                    },
                );
            }
            if boundary.range().end() > coverage.processed_end() {
                return Err(
                    HtmlTokenizerRunContractError::UnsupportedTokenBoundaryOutsideProcessedPrefix,
                );
            }
            if !boundary.range().is_empty()
                || boundary.range().start() != token_anchor(token).range().end()
            {
                return Err(
                    HtmlTokenizerRunContractError::UnsupportedTokenBoundaryMustFollowReferencedToken,
                );
            }
        }
    }
    Ok(())
}

fn validate_resource_limit(
    source_text: &SourceText,
    coverage: &HtmlTokenizerCoverage,
    limits: HtmlTokenizerLimits,
    usage: HtmlTokenizerUsage,
    resource_limit: &HtmlTokenizerResourceLimit,
) -> Result<(), HtmlTokenizerRunContractError> {
    require_source(
        source_text.id(),
        resource_limit.at(),
        HtmlRunEvidenceRole::ResourceLimitLocation,
    )?;
    if resource_limit.limit() != limits.limit_for(resource_limit.resource()) {
        return Err(HtmlTokenizerRunContractError::ResourceLimitPolicyMismatch);
    }
    if resource_limit.attempted() <= resource_limit.limit() {
        return Err(HtmlTokenizerRunContractError::ResourceLimitAttemptMismatch);
    }
    if resource_limit.at().range().start() > coverage.processed_end() {
        return Err(HtmlTokenizerRunContractError::ResourceLimitLocationAfterProcessedPrefix);
    }

    if resource_limit.resource() == HtmlTokenizerResource::SourceBytes {
        if resource_limit.attempted() != source_text.as_str().len()
            || coverage.processed_end() != 0
            || resource_limit.at().range().start() != 0
            || resource_limit.at().range().end() != 0
        {
            return Err(HtmlTokenizerRunContractError::ResourceLimitAttemptMismatch);
        }
    } else {
        let committed = usage.value_for(resource_limit.resource());
        // validate_usage() deliberately skips the resource that caused this
        // termination, so committed usage for that resource is only checked
        // against its configured limit here.
        if committed > resource_limit.limit() {
            return Err(HtmlTokenizerRunContractError::UsageExceedsLimit {
                resource: resource_limit.resource(),
                usage: committed,
                limit: resource_limit.limit(),
            });
        }
        // Diagnostics may commit multiple units atomically (one logical
        // operation can require several diagnostics to become valid
        // together), so it cannot share the exact-single-increment
        // invariant that holds for the other structurally one-unit-at-a-time
        // resources.
        let exact_increment = matches!(
            resource_limit.resource(),
            HtmlTokenizerResource::TransitionSteps
                | HtmlTokenizerResource::EmittedTokens
                | HtmlTokenizerResource::AttributesPerTag
        );
        if resource_limit.attempted() <= committed
            || (exact_increment
                && resource_limit.attempted()
                    != committed
                        .checked_add(1)
                        .ok_or(HtmlTokenizerRunContractError::CounterOverflow)?)
        {
            return Err(HtmlTokenizerRunContractError::ResourceLimitAttemptMismatch);
        }
    }
    Ok(())
}

fn token_anchor(token: &HtmlToken) -> &SourceAnchor {
    match token {
        HtmlToken::Character(token) => token.source(),
        HtmlToken::Tag(token) => token.complete(),
        HtmlToken::EndOfFile(token) => token.source(),
    }
}

fn token_interpreted_bytes(token: &HtmlToken) -> Result<usize, HtmlTokenizerRunContractError> {
    match token {
        HtmlToken::Character(token) => Ok(token.interpreted().len()),
        HtmlToken::Tag(token) => token.attributes().iter().try_fold(
            token.name().interpreted().len(),
            |total, attribute| {
                total
                    .checked_add(attribute.name().interpreted().len())
                    .and_then(|value| value.checked_add(attribute.interpreted_value().len()))
                    .ok_or(HtmlTokenizerRunContractError::CounterOverflow)
            },
        ),
        HtmlToken::EndOfFile(_) => Ok(0),
    }
}

fn require_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: HtmlRunEvidenceRole,
) -> Result<(), HtmlTokenizerRunContractError> {
    if anchor.source_id() != expected {
        return Err(HtmlTokenizerRunContractError::SourceIdentityMismatch {
            role,
            expected,
            actual: anchor.source_id(),
        });
    }
    Ok(())
}
