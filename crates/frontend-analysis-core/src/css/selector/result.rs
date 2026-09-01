//! Selector qualification semantic-stage result contracts (#182).
//!
//! The result owns the unchanged upstream parser result so run-local
//! `CssParserContextId` observations cannot outlive the structural evidence
//! they reference. This module defines representation and invariants only;
//! it contains no selector qualification algorithm.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId};

use super::super::parser::context::{CssParserContextId, CssParserContextKind};
use super::super::parser::result::CssParserRunResult;
use super::context::{
    CssSelectorContextContractError, CssSelectorGrammarContext, derive_selector_grammar_context,
};
use super::profile::CssSelectorGrammarProfile;
use super::resource::{
    CssSelectorInvalidConfiguration, CssSelectorResourceKind, CssSelectorResourceLimitEvidence,
    CssSelectorResourceUsage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorInvalidReason {
    EmptySelectorList,
    UnexpectedComma,
    UnexpectedCombinator,
    InvalidCompoundOrder,
    InvalidAttributeSelector,
    InvalidPseudoSyntax,
    InvalidNestingSelectorPlacement,
    InvalidFunctionalPseudoArgument,
    NestedHasNotAllowed,
    UnexpectedToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorUnsupportedFeature {
    IdentifierPseudoClass,
    FunctionalPseudoClass,
    PseudoElement,
    FunctionalPseudoElement,
    OtherSelectorFeature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorIndeterminateReason {
    MissingNamespaceEnvironment,
    UnavailableStructuralContext,
}

#[derive(Clone)]
pub(crate) enum CssSelectorQualificationOutcome {
    QualifiedBySelectedGrammar,
    InvalidForSelectedGrammar {
        reason: CssSelectorInvalidReason,
        subject: SourceAnchor,
    },
    UnsupportedBySelectedGrammarProfile {
        feature: CssSelectorUnsupportedFeature,
        subject: SourceAnchor,
    },
    Indeterminate {
        reason: CssSelectorIndeterminateReason,
        subject: Option<SourceAnchor>,
    },
}

impl fmt::Debug for CssSelectorQualificationOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QualifiedBySelectedGrammar => {
                formatter.debug_tuple("QualifiedBySelectedGrammar").finish()
            }
            Self::InvalidForSelectedGrammar { reason, subject } => formatter
                .debug_struct("InvalidForSelectedGrammar")
                .field("reason", reason)
                .field("source_id", &subject.source_id())
                .field("subject", &subject.range())
                .finish(),
            Self::UnsupportedBySelectedGrammarProfile { feature, subject } => formatter
                .debug_struct("UnsupportedBySelectedGrammarProfile")
                .field("feature", feature)
                .field("source_id", &subject.source_id())
                .field("subject", &subject.range())
                .finish(),
            Self::Indeterminate { reason, subject } => {
                let mut debug = formatter.debug_struct("Indeterminate");
                debug.field("reason", reason);
                if let Some(subject) = subject {
                    debug
                        .field("source_id", &subject.source_id())
                        .field("subject", &subject.range());
                }
                debug.finish()
            }
        }
    }
}

impl PartialEq for CssSelectorQualificationOutcome {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::QualifiedBySelectedGrammar, Self::QualifiedBySelectedGrammar) => true,
            (
                Self::InvalidForSelectedGrammar {
                    reason: left_reason,
                    subject: left_subject,
                },
                Self::InvalidForSelectedGrammar {
                    reason: right_reason,
                    subject: right_subject,
                },
            ) => left_reason == right_reason && same_anchor(left_subject, right_subject),
            (
                Self::UnsupportedBySelectedGrammarProfile {
                    feature: left_feature,
                    subject: left_subject,
                },
                Self::UnsupportedBySelectedGrammarProfile {
                    feature: right_feature,
                    subject: right_subject,
                },
            ) => left_feature == right_feature && same_anchor(left_subject, right_subject),
            (
                Self::Indeterminate {
                    reason: left_reason,
                    subject: left_subject,
                },
                Self::Indeterminate {
                    reason: right_reason,
                    subject: right_subject,
                },
            ) => {
                left_reason == right_reason
                    && match (left_subject, right_subject) {
                        (Some(left), Some(right)) => same_anchor(left, right),
                        (None, None) => true,
                        _ => false,
                    }
            }
            _ => false,
        }
    }
}

impl Eq for CssSelectorQualificationOutcome {}

#[derive(Debug, Clone)]
pub(crate) struct CssSelectorQualificationObservation {
    context_id: CssParserContextId,
    context_header: SourceAnchor,
    grammar_context: CssSelectorGrammarContext,
    outcome: CssSelectorQualificationOutcome,
}

impl CssSelectorQualificationObservation {
    pub(crate) fn new(
        parser_result: &CssParserRunResult,
        context_id: CssParserContextId,
        grammar_context: CssSelectorGrammarContext,
        outcome: CssSelectorQualificationOutcome,
    ) -> Result<Self, CssSelectorRunError> {
        let records = parser_result.context_records();
        let record = records
            .get(context_id.index())
            .filter(|record| record.id() == context_id)
            .ok_or_else(|| {
                invariant(CssSelectorInvariantViolation::ObservationContextNotFound {
                    context: context_id,
                })
            })?;
        if !matches!(record.kind(), CssParserContextKind::QualifiedRuleBlock) {
            return Err(invariant(
                CssSelectorInvariantViolation::ObservationContextNotQualified {
                    context: context_id,
                },
            ));
        }

        let expected_context =
            derive_selector_grammar_context(records, context_id).map_err(|error| {
                invariant(CssSelectorInvariantViolation::ContextContractViolation { error })
            })?;
        if grammar_context != expected_context {
            return Err(invariant(
                CssSelectorInvariantViolation::ObservationGrammarContextMismatch {
                    context: context_id,
                    expected: expected_context,
                    actual: grammar_context,
                },
            ));
        }

        validate_outcome_subject(context_id, record.header(), &outcome)?;

        Ok(Self {
            context_id,
            context_header: record.header().clone(),
            grammar_context,
            outcome,
        })
    }

    pub(crate) const fn context_id(&self) -> CssParserContextId {
        self.context_id
    }

    pub(crate) const fn grammar_context(&self) -> CssSelectorGrammarContext {
        self.grammar_context
    }

    pub(crate) const fn outcome(&self) -> &CssSelectorQualificationOutcome {
        &self.outcome
    }
}

impl PartialEq for CssSelectorQualificationObservation {
    fn eq(&self, other: &Self) -> bool {
        self.context_id == other.context_id
            && same_anchor(&self.context_header, &other.context_header)
            && self.grammar_context == other.grammar_context
            && self.outcome == other.outcome
    }
}

impl Eq for CssSelectorQualificationObservation {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorExecutionCompletion {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssSelectorTermination {
    AllRetainedQualifiedContextsProcessed,
    ResourceLimit(CssSelectorResourceLimitEvidence),
}

#[derive(Debug, Clone)]
pub(crate) struct CssSelectorQualificationRunResult {
    upstream_parser_result: CssParserRunResult,
    profile: CssSelectorGrammarProfile,
    observations: Vec<CssSelectorQualificationObservation>,
    execution_completion: CssSelectorExecutionCompletion,
    termination: CssSelectorTermination,
    resources: CssSelectorResourceUsage,
}

impl CssSelectorQualificationRunResult {
    pub(crate) fn new(
        upstream_parser_result: CssParserRunResult,
        profile: CssSelectorGrammarProfile,
        observations: Vec<CssSelectorQualificationObservation>,
        execution_completion: CssSelectorExecutionCompletion,
        termination: CssSelectorTermination,
        resources: CssSelectorResourceUsage,
    ) -> Result<Self, CssSelectorRunError> {
        validate_run_result(
            &upstream_parser_result,
            &observations,
            execution_completion,
            &termination,
            resources,
        )?;

        Ok(Self {
            upstream_parser_result,
            profile,
            observations,
            execution_completion,
            termination,
            resources,
        })
    }

    pub(crate) const fn upstream_parser_result(&self) -> &CssParserRunResult {
        &self.upstream_parser_result
    }

    pub(crate) const fn profile(&self) -> CssSelectorGrammarProfile {
        self.profile
    }

    pub(crate) fn observations(&self) -> &[CssSelectorQualificationObservation] {
        &self.observations
    }

    pub(crate) const fn execution_completion(&self) -> CssSelectorExecutionCompletion {
        self.execution_completion
    }

    pub(crate) const fn termination(&self) -> &CssSelectorTermination {
        &self.termination
    }

    pub(crate) const fn resources(&self) -> CssSelectorResourceUsage {
        self.resources
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssSelectorRunError {
    InvalidConfiguration(CssSelectorInvalidConfiguration),
    InternalInvariantFailure(CssSelectorInvariantViolation),
}

impl From<CssSelectorInvalidConfiguration> for CssSelectorRunError {
    fn from(error: CssSelectorInvalidConfiguration) -> Self {
        Self::InvalidConfiguration(error)
    }
}

impl fmt::Display for CssSelectorRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS selector run failure: {self:?}")
    }
}

impl Error for CssSelectorRunError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssSelectorInvariantViolation {
    ContextContractViolation {
        error: CssSelectorContextContractError,
    },
    ObservationContextNotFound {
        context: CssParserContextId,
    },
    ObservationContextNotQualified {
        context: CssParserContextId,
    },
    ObservationGrammarContextMismatch {
        context: CssParserContextId,
        expected: CssSelectorGrammarContext,
        actual: CssSelectorGrammarContext,
    },
    ObservationSubjectSourceIdentityMismatch {
        context: CssParserContextId,
        expected: SourceId,
        actual: SourceId,
    },
    ObservationSubjectSourceContentMismatch {
        context: CssParserContextId,
        source_id: SourceId,
    },
    ObservationSubjectOutsideHeader {
        context: CssParserContextId,
    },
    ObservationContextOrderMismatch {
        observation_index: usize,
        expected: CssParserContextId,
        actual: CssParserContextId,
    },
    ObservationProvenanceMismatch {
        context: CssParserContextId,
    },
    ResourceObservationCountMismatch {
        expected: usize,
        actual: usize,
    },
    CompletionTerminationMismatch,
    CompleteRunMissingObservation {
        expected: usize,
        actual: usize,
    },
    ResourceLimitedRunHasNoRemainingContext,
    ResourceLimitSourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    ResourceLimitSourceContentMismatch {
        source_id: SourceId,
    },
    ResourceLimitOutsideNextContextHeader {
        next_context: CssParserContextId,
    },
}

fn validate_run_result(
    parser_result: &CssParserRunResult,
    observations: &[CssSelectorQualificationObservation],
    execution_completion: CssSelectorExecutionCompletion,
    termination: &CssSelectorTermination,
    resources: CssSelectorResourceUsage,
) -> Result<(), CssSelectorRunError> {
    let qualified_contexts: Vec<_> = parser_result
        .context_records()
        .iter()
        .filter(|record| matches!(record.kind(), CssParserContextKind::QualifiedRuleBlock))
        .collect();

    for (index, observation) in observations.iter().enumerate() {
        let expected_record = qualified_contexts.get(index).ok_or_else(|| {
            invariant(
                CssSelectorInvariantViolation::CompleteRunMissingObservation {
                    expected: qualified_contexts.len(),
                    actual: observations.len(),
                },
            )
        })?;
        let expected = expected_record.id();
        if observation.context_id() != expected {
            return Err(invariant(
                CssSelectorInvariantViolation::ObservationContextOrderMismatch {
                    observation_index: index,
                    expected,
                    actual: observation.context_id(),
                },
            ));
        }
        if !same_anchor(&observation.context_header, expected_record.header()) {
            return Err(invariant(
                CssSelectorInvariantViolation::ObservationProvenanceMismatch { context: expected },
            ));
        }
        let expected_context =
            derive_selector_grammar_context(parser_result.context_records(), expected).map_err(
                |error| {
                    invariant(CssSelectorInvariantViolation::ContextContractViolation { error })
                },
            )?;
        if observation.grammar_context() != expected_context {
            return Err(invariant(
                CssSelectorInvariantViolation::ObservationGrammarContextMismatch {
                    context: expected,
                    expected: expected_context,
                    actual: observation.grammar_context(),
                },
            ));
        }
        validate_outcome_subject(expected, expected_record.header(), observation.outcome())?;
    }

    let actual_observations = resources.value(CssSelectorResourceKind::Observations);
    if actual_observations != observations.len() {
        return Err(invariant(
            CssSelectorInvariantViolation::ResourceObservationCountMismatch {
                expected: observations.len(),
                actual: actual_observations,
            },
        ));
    }

    match (execution_completion, termination) {
        (
            CssSelectorExecutionCompletion::Complete,
            CssSelectorTermination::AllRetainedQualifiedContextsProcessed,
        ) => {
            if observations.len() != qualified_contexts.len() {
                return Err(invariant(
                    CssSelectorInvariantViolation::CompleteRunMissingObservation {
                        expected: qualified_contexts.len(),
                        actual: observations.len(),
                    },
                ));
            }
        }
        (
            CssSelectorExecutionCompletion::Incomplete,
            CssSelectorTermination::ResourceLimit(evidence),
        ) => {
            let Some(next_context) = qualified_contexts.get(observations.len()) else {
                return Err(invariant(
                    CssSelectorInvariantViolation::ResourceLimitedRunHasNoRemainingContext,
                ));
            };
            let expected_source = parser_result.upstream_tokenizer_result().source_id();
            let actual_source = evidence.location().source_id();
            if actual_source != expected_source {
                return Err(invariant(
                    CssSelectorInvariantViolation::ResourceLimitSourceIdentityMismatch {
                        expected: expected_source,
                        actual: actual_source,
                    },
                ));
            }
            if !next_context
                .header()
                .retains_exact_source(evidence.location())
            {
                return Err(invariant(
                    CssSelectorInvariantViolation::ResourceLimitSourceContentMismatch {
                        source_id: expected_source,
                    },
                ));
            }
            let location = evidence.location().range().start();
            if location < next_context.header().range().start()
                || location > next_context.header().range().end()
            {
                return Err(invariant(
                    CssSelectorInvariantViolation::ResourceLimitOutsideNextContextHeader {
                        next_context: next_context.id(),
                    },
                ));
            }
        }
        _ => {
            return Err(invariant(
                CssSelectorInvariantViolation::CompletionTerminationMismatch,
            ));
        }
    }

    Ok(())
}

fn validate_outcome_subject(
    context_id: CssParserContextId,
    header: &SourceAnchor,
    outcome: &CssSelectorQualificationOutcome,
) -> Result<(), CssSelectorRunError> {
    match outcome {
        CssSelectorQualificationOutcome::QualifiedBySelectedGrammar => Ok(()),
        CssSelectorQualificationOutcome::InvalidForSelectedGrammar { subject, .. }
        | CssSelectorQualificationOutcome::UnsupportedBySelectedGrammarProfile {
            subject, ..
        } => validate_subject(context_id, header, subject),
        CssSelectorQualificationOutcome::Indeterminate { subject, .. } => {
            if let Some(subject) = subject {
                validate_subject(context_id, header, subject)?;
            }
            Ok(())
        }
    }
}

fn validate_subject(
    context_id: CssParserContextId,
    header: &SourceAnchor,
    subject: &SourceAnchor,
) -> Result<(), CssSelectorRunError> {
    let expected = header.source_id();
    let actual = subject.source_id();
    if actual != expected {
        return Err(invariant(
            CssSelectorInvariantViolation::ObservationSubjectSourceIdentityMismatch {
                context: context_id,
                expected,
                actual,
            },
        ));
    }
    if !header.retains_exact_source(subject) {
        return Err(invariant(
            CssSelectorInvariantViolation::ObservationSubjectSourceContentMismatch {
                context: context_id,
                source_id: expected,
            },
        ));
    }
    if subject.range().start() < header.range().start()
        || subject.range().end() > header.range().end()
    {
        return Err(invariant(
            CssSelectorInvariantViolation::ObservationSubjectOutsideHeader {
                context: context_id,
            },
        ));
    }
    Ok(())
}

fn invariant(violation: CssSelectorInvariantViolation) -> CssSelectorRunError {
    CssSelectorRunError::InternalInvariantFailure(violation)
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.retains_exact_source(right) && left.range() == right.range()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::parser::producer::run as run_parser;
    use crate::css::parser::resource::CssParserLimits;
    use crate::css::tokenizer::producer::run as run_tokenizer;
    use crate::css::tokenizer::resource::CssTokenizerLimits;
    use crate::{SourceId, SourceText};

    fn tokenizer_limits() -> CssTokenizerLimits {
        CssTokenizerLimits::new(4096, 50_000, 4096, 256, 4096, 4096).unwrap()
    }

    fn parser_limits() -> CssParserLimits {
        CssParserLimits::new(50_000, 128, 128, 4096, 256, 256, 256, 256, 4096).unwrap()
    }

    fn parse(source: &SourceText) -> CssParserRunResult {
        let tokenizer = run_tokenizer(source, tokenizer_limits()).unwrap();
        run_parser(source, tokenizer, parser_limits()).unwrap()
    }

    #[test]
    fn observation_is_bound_to_existing_qualified_context_and_derived_mode() {
        let source = SourceText::new(SourceId::new(30), "a{p:v;}".to_owned());
        let parser = parse(&source);
        let observation = CssSelectorQualificationObservation::new(
            &parser,
            CssParserContextId::new(0),
            CssSelectorGrammarContext::NormalSelectorList,
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar,
        )
        .unwrap();

        assert_eq!(observation.context_id(), CssParserContextId::new(0));
        assert_eq!(
            observation.grammar_context(),
            CssSelectorGrammarContext::NormalSelectorList
        );
    }

    #[test]
    fn authored_failure_subject_must_be_inside_the_context_header() {
        let source = SourceText::new(SourceId::new(31), "a{p:v;}".to_owned());
        let parser = parse(&source);
        let body_subject = source.anchor(2, 3).unwrap();

        assert!(matches!(
            CssSelectorQualificationObservation::new(
                &parser,
                CssParserContextId::new(0),
                CssSelectorGrammarContext::NormalSelectorList,
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::UnexpectedToken,
                    subject: body_subject,
                },
            ),
            Err(CssSelectorRunError::InternalInvariantFailure(
                CssSelectorInvariantViolation::ObservationSubjectOutsideHeader { .. }
            ))
        ));
    }

    #[test]
    fn complete_result_owns_unchanged_parser_result_and_one_observation_per_qualified_context() {
        let source = SourceText::new(SourceId::new(32), "a{p:v;}".to_owned());
        let parser = parse(&source);
        let observation = CssSelectorQualificationObservation::new(
            &parser,
            CssParserContextId::new(0),
            CssSelectorGrammarContext::NormalSelectorList,
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar,
        )
        .unwrap();
        let result = CssSelectorQualificationRunResult::new(
            parser,
            CssSelectorGrammarProfile::CoreV1,
            vec![observation],
            CssSelectorExecutionCompletion::Complete,
            CssSelectorTermination::AllRetainedQualifiedContextsProcessed,
            CssSelectorResourceUsage::new(1, 1, 1),
        )
        .unwrap();

        assert_eq!(result.profile(), CssSelectorGrammarProfile::CoreV1);
        assert_eq!(result.observations().len(), 1);
        assert_eq!(result.upstream_parser_result().context_records().len(), 1);
    }

    #[test]
    fn observation_rejects_same_source_id_with_different_retained_bytes() {
        let source = SourceText::new(SourceId::new(34), "a{p:v;}".to_owned());
        let different = SourceText::new(SourceId::new(34), "b{p:v;}".to_owned());
        let parser = parse(&source);

        assert!(matches!(
            CssSelectorQualificationObservation::new(
                &parser,
                CssParserContextId::new(0),
                CssSelectorGrammarContext::NormalSelectorList,
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::UnexpectedToken,
                    subject: different.anchor(0, 1).unwrap(),
                },
            ),
            Err(CssSelectorRunError::InternalInvariantFailure(
                CssSelectorInvariantViolation::ObservationSubjectSourceContentMismatch { .. }
            ))
        ));
    }

    #[test]
    fn run_result_rejects_qualified_observation_from_different_source_run() {
        let source = SourceText::new(SourceId::new(36), "a{}".to_owned());
        let different = SourceText::new(SourceId::new(36), "b{}".to_owned());
        let source_parser = parse(&source);
        let different_parser = parse(&different);
        let observation = CssSelectorQualificationObservation::new(
            &source_parser,
            CssParserContextId::new(0),
            CssSelectorGrammarContext::NormalSelectorList,
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar,
        )
        .unwrap();

        assert!(matches!(
            CssSelectorQualificationRunResult::new(
                different_parser,
                CssSelectorGrammarProfile::CoreV1,
                vec![observation],
                CssSelectorExecutionCompletion::Complete,
                CssSelectorTermination::AllRetainedQualifiedContextsProcessed,
                CssSelectorResourceUsage::new(1, 1, 1),
            ),
            Err(CssSelectorRunError::InternalInvariantFailure(
                CssSelectorInvariantViolation::ObservationProvenanceMismatch { .. }
            ))
        ));
    }

    #[test]
    fn resource_limited_result_rejects_same_id_different_source_run() {
        let source = SourceText::new(SourceId::new(35), "a{}b{}".to_owned());
        let different = SourceText::new(SourceId::new(35), "x{}y{}".to_owned());
        let parser = parse(&source);
        let first = CssSelectorQualificationObservation::new(
            &parser,
            CssParserContextId::new(0),
            CssSelectorGrammarContext::NormalSelectorList,
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar,
        )
        .unwrap();
        let refusal = CssSelectorResourceLimitEvidence::new(
            &different,
            CssSelectorResourceKind::AlgorithmSteps,
            1,
            2,
            different.anchor(3, 3).unwrap(),
        )
        .unwrap();

        assert!(matches!(
            CssSelectorQualificationRunResult::new(
                parser,
                CssSelectorGrammarProfile::CoreV1,
                vec![first],
                CssSelectorExecutionCompletion::Incomplete,
                CssSelectorTermination::ResourceLimit(refusal),
                CssSelectorResourceUsage::new(1, 1, 1),
            ),
            Err(CssSelectorRunError::InternalInvariantFailure(
                CssSelectorInvariantViolation::ResourceLimitSourceContentMismatch { .. }
            ))
        ));
    }

    #[test]
    fn resource_limited_result_preserves_prior_observation_prefix() {
        let source = SourceText::new(SourceId::new(33), "a{}b{}".to_owned());
        let parser = parse(&source);
        let first = CssSelectorQualificationObservation::new(
            &parser,
            CssParserContextId::new(0),
            CssSelectorGrammarContext::NormalSelectorList,
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar,
        )
        .unwrap();
        let refusal_location = source.anchor(3, 3).unwrap();
        let refusal = CssSelectorResourceLimitEvidence::new(
            &source,
            CssSelectorResourceKind::AlgorithmSteps,
            1,
            2,
            refusal_location,
        )
        .unwrap();

        let result = CssSelectorQualificationRunResult::new(
            parser,
            CssSelectorGrammarProfile::CoreV1,
            vec![first],
            CssSelectorExecutionCompletion::Incomplete,
            CssSelectorTermination::ResourceLimit(refusal),
            CssSelectorResourceUsage::new(1, 1, 1),
        )
        .unwrap();

        assert_eq!(result.observations().len(), 1);
        assert_eq!(
            result.execution_completion(),
            CssSelectorExecutionCompletion::Incomplete
        );
    }
}
