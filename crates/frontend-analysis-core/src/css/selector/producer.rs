//! Bounded production CSS selector qualification for `CoreV1` (#184/#405).
//!
//! Grammar meaning is derived only from the Core-validated parser result and
//! the retained tokenizer lexical items selected by `QualifiedRuleBlock`
//! headers. `SourceText` is borrowed to construct resource-limit points and
//! exact semantic anchors only from endpoints already owned by authoritative
//! recognition. Authored bytes are never searched, rescanned, retokenized, or
//! reconstructed here.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceRangeError, SourceText};

use super::super::parser::context::CssParserContextKind;
use super::super::parser::result::CssParserRunResult;
use super::super::token::{CssHashType, CssLexicalItem, CssToken, CssTokenKind};
use super::context::{
    CssSelectorContextContractError, CssSelectorGrammarContext, derive_selector_grammar_context,
};
use super::handoff::{
    CssSelectorHandoffInvariantViolation, CssSelectorNestingPresenceDisposition,
    CssSelectorRelationshipResolutionError, CssSelectorSemanticFact,
    CssSelectorSemanticFunctionKind, CssSelectorSemanticMemberId, CssSelectorSemanticProgram,
    CssSelectorSemanticRelationshipOrigin, CssSelectorSemanticRelationshipTarget,
    CssSelectorSemanticSimpleKind, CssSelectorSemanticUnitId, resolve_relationship_target,
};
use super::lexical::{CssSelectorLexicalWindowCursor, CssSelectorLexicalWindowError};
use super::profile::{
    CssSelectorFunctionalPseudoClass, CssSelectorGrammarProfile, core_v1_functional_pseudo_class,
    is_core_v1_identifier_pseudo_class,
};
use super::resource::{
    CssSelectorInvalidConfiguration, CssSelectorLimits, CssSelectorResourceContractError,
    CssSelectorResourceKind, CssSelectorResourceLimitEvidence, CssSelectorResourceUsage,
    checked_resource_add,
};
use super::result::{
    CssSelectorExecutionCompletion, CssSelectorIndeterminateReason, CssSelectorInvalidReason,
    CssSelectorInvariantViolation, CssSelectorQualificationObservation,
    CssSelectorQualificationOutcome, CssSelectorQualificationRunResult, CssSelectorRunError,
    CssSelectorTermination, CssSelectorUnsupportedFeature,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssSelectorProducerError {
    InvalidConfiguration(CssSelectorInvalidConfiguration),
    InternalInvariantFailure(CssSelectorProducerInvariantViolation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssSelectorProducerInvariantViolation {
    ContextContract(CssSelectorContextContractError),
    LexicalWindow(CssSelectorLexicalWindowError),
    HandoffContract(CssSelectorHandoffInvariantViolation),
    ResourceContract(CssSelectorResourceContractError),
    ResourcePointRange(SourceRangeError),
    SemanticRange(SourceRangeError),
    ResultContract(CssSelectorInvariantViolation),
    ActiveDepthUnderflow,
    MissingCurrentToken,
    MissingFunctionFrame,
    MissingRootFrame,
    MissingSemanticMember,
    MissingFunctionSemanticUnit,
    SemanticOpenFactMismatch,
    SemanticIdentityOverflow,
    UnexpectedTypeDispatch,
    UnexpectedFaultConversion,
    UpstreamSourceMismatch,
}

impl fmt::Display for CssSelectorProducerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS selector producer failure: {self:?}")
    }
}

impl Error for CssSelectorProducerError {}

impl From<CssSelectorRunError> for CssSelectorProducerError {
    fn from(error: CssSelectorRunError) -> Self {
        match error {
            CssSelectorRunError::InvalidConfiguration(error) => Self::InvalidConfiguration(error),
            CssSelectorRunError::InternalInvariantFailure(error) => Self::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::ResultContract(error),
            ),
        }
    }
}

/// Executes the #181/#182-approved selector stage over one Core-validated
/// parser result.
///
/// The normal Core entry path is `selector::analysis::analyze_css_selectors`,
/// which first calls the existing `css::analysis::analyze_css_source` boundary.
/// This lower-level function exists so conformance/resource tests can exercise
/// selector execution independently after constructing validated parser runs.
pub(crate) fn run(
    source: &SourceText,
    parser_result: CssParserRunResult,
    limits: CssSelectorLimits,
) -> Result<CssSelectorQualificationRunResult, CssSelectorProducerError> {
    if !source
        .retains_exact_anchor_source(parser_result.upstream_tokenizer_result().processed_prefix())
    {
        return Err(CssSelectorProducerError::InternalInvariantFailure(
            CssSelectorProducerInvariantViolation::UpstreamSourceMismatch,
        ));
    }

    let mut observations = Vec::new();
    let mut resources = ResourceTracker::new(source, limits);
    let mut lexical_cursor =
        CssSelectorLexicalWindowCursor::new(parser_result.upstream_tokenizer_result());

    for record in parser_result.context_records() {
        if !matches!(record.kind(), CssParserContextKind::QualifiedRuleBlock) {
            continue;
        }

        let grammar_context =
            derive_selector_grammar_context(parser_result.context_records(), record.id()).map_err(
                |error| {
                    CssSelectorProducerError::InternalInvariantFailure(
                        CssSelectorProducerInvariantViolation::ContextContract(error),
                    )
                },
            )?;
        let window = lexical_cursor
            .window_for(record.header())
            .map_err(|error| {
                CssSelectorProducerError::InternalInvariantFailure(
                    CssSelectorProducerInvariantViolation::LexicalWindow(error),
                )
            })?;

        let execution = qualify_candidate(
            record.header(),
            window.items(parser_result.upstream_tokenizer_result()),
            grammar_context,
            &mut resources,
        )?;

        let mut staged_facts = match execution {
            CandidateExecution::Qualified(facts) => Some(facts),
            CandidateExecution::Outcome(outcome) => {
                let retained_delta = resources.retained_delta(0)?;
                let observation_preflight = resources.preflight_observation(record.header())?;
                let ObservationPreflight::Allowed {
                    attempted: observation_attempted,
                } = observation_preflight
                else {
                    let ObservationPreflight::Refused(evidence) = observation_preflight else {
                        unreachable!()
                    };
                    return build_incomplete(
                        parser_result,
                        observations,
                        resources.usage(),
                        evidence,
                    );
                };
                let retained_preflight =
                    resources.preflight_retained_semantic(record.header(), retained_delta)?;
                let RetainedPreflight::Allowed {
                    attempted: retained_attempted,
                } = retained_preflight
                else {
                    let RetainedPreflight::Refused(evidence) = retained_preflight else {
                        unreachable!()
                    };
                    return build_incomplete(
                        parser_result,
                        observations,
                        resources.usage(),
                        evidence,
                    );
                };

                let observation = CssSelectorQualificationObservation::new(
                    &parser_result,
                    record.id(),
                    grammar_context,
                    outcome,
                    None,
                )?;
                observations.push(observation);
                resources.commit_persistent(observation_attempted, retained_attempted);
                continue;
            }
            CandidateExecution::ResourceLimit(evidence) => {
                return build_incomplete(parser_result, observations, resources.usage(), evidence);
            }
        };

        if let Some(facts) = staged_facts.as_mut()
            && facts
                .iter()
                .any(|fact| matches!(fact, CssSelectorSemanticFact::Relationship { .. }))
        {
            match resolve_staged_relationships(
                parser_result.context_records(),
                record.id(),
                record.header(),
                facts,
                &mut resources,
            )? {
                Some(evidence) => {
                    return build_incomplete(
                        parser_result,
                        observations,
                        resources.usage(),
                        evidence,
                    );
                }
                None => {}
            }
        }

        let fact_count = staged_facts.as_ref().map_or(0, Vec::len);
        let retained_delta = resources.retained_delta(fact_count)?;

        // Persistent refusal precedence is load-bearing: Observations is
        // preflighted before RetainedSemanticUnits and neither mutates usage.
        let observation_preflight = resources.preflight_observation(record.header())?;
        let ObservationPreflight::Allowed {
            attempted: observation_attempted,
        } = observation_preflight
        else {
            let ObservationPreflight::Refused(evidence) = observation_preflight else {
                unreachable!()
            };
            return build_incomplete(parser_result, observations, resources.usage(), evidence);
        };

        let retained_preflight =
            resources.preflight_retained_semantic(record.header(), retained_delta)?;
        let RetainedPreflight::Allowed {
            attempted: retained_attempted,
        } = retained_preflight
        else {
            let RetainedPreflight::Refused(evidence) = retained_preflight else {
                unreachable!()
            };
            return build_incomplete(parser_result, observations, resources.usage(), evidence);
        };

        // All normal resource refusals have succeeded before the fallible
        // attachment/invariant validation required by #405.
        let facts = staged_facts.take().ok_or_else(|| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingSemanticMember,
            )
        })?;
        let program = CssSelectorSemanticProgram::from_authoritative_staging(
            record.id(),
            record.header(),
            facts,
        )
        .map_err(|error| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::HandoffContract(error),
            )
        })?;
        let observation = CssSelectorQualificationObservation::new(
            &parser_result,
            record.id(),
            grammar_context,
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar,
            Some(program),
        )?;

        // Non-refusing durable commit region.
        observations.push(observation);
        resources.commit_persistent(observation_attempted, retained_attempted);
    }

    CssSelectorQualificationRunResult::new(
        parser_result,
        CssSelectorGrammarProfile::CoreV1,
        observations,
        CssSelectorExecutionCompletion::Complete,
        CssSelectorTermination::AllRetainedQualifiedContextsProcessed,
        resources.usage(),
    )
    .map_err(Into::into)
}

fn build_incomplete(
    parser_result: CssParserRunResult,
    observations: Vec<CssSelectorQualificationObservation>,
    usage: CssSelectorResourceUsage,
    evidence: CssSelectorResourceLimitEvidence,
) -> Result<CssSelectorQualificationRunResult, CssSelectorProducerError> {
    CssSelectorQualificationRunResult::new(
        parser_result,
        CssSelectorGrammarProfile::CoreV1,
        observations,
        CssSelectorExecutionCompletion::Incomplete,
        CssSelectorTermination::ResourceLimit(evidence),
        usage,
    )
    .map_err(Into::into)
}

#[derive(Debug)]
enum CandidateExecution {
    Qualified(Vec<CssSelectorSemanticFact>),
    Outcome(CssSelectorQualificationOutcome),
    ResourceLimit(CssSelectorResourceLimitEvidence),
}

fn qualify_candidate(
    header: &SourceAnchor,
    lexical_items: &[CssLexicalItem],
    grammar_context: CssSelectorGrammarContext,
    resources: &mut ResourceTracker<'_>,
) -> Result<CandidateExecution, CssSelectorProducerError> {
    let mut tokens = Vec::new();
    for item in lexical_items {
        if let Some(evidence) = resources.charge_algorithm_step(header, item.source())? {
            return Ok(CandidateExecution::ResourceLimit(evidence));
        }
        if let CssLexicalItem::SemanticToken(token) = item {
            tokens.push(token);
        }
    }

    SelectorMachine::new(header, tokens, grammar_context, resources)?.execute()
}

#[derive(Debug)]
enum RelationshipChargeStop {
    ResourceLimit(CssSelectorResourceLimitEvidence),
    Internal(CssSelectorProducerError),
}

fn resolve_staged_relationships(
    records: &[super::super::parser::context::CssParserContextRecord],
    owning_context: super::super::parser::context::CssParserContextId,
    header: &SourceAnchor,
    facts: &mut [CssSelectorSemanticFact],
    resources: &mut ResourceTracker<'_>,
) -> Result<Option<CssSelectorResourceLimitEvidence>, CssSelectorProducerError> {
    let target = match resolve_relationship_target(records, owning_context, |_parent_id| {
        match resources.charge_algorithm_step(header, header) {
            Ok(Some(evidence)) => Err(RelationshipChargeStop::ResourceLimit(evidence)),
            Ok(None) => Ok(()),
            Err(error) => Err(RelationshipChargeStop::Internal(error)),
        }
    }) {
        Ok(target) => target,
        Err(CssSelectorRelationshipResolutionError::Handoff(error)) => {
            return Err(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::HandoffContract(error),
            ));
        }
        Err(CssSelectorRelationshipResolutionError::Charge(
            RelationshipChargeStop::ResourceLimit(evidence),
        )) => return Ok(Some(evidence)),
        Err(CssSelectorRelationshipResolutionError::Charge(RelationshipChargeStop::Internal(
            error,
        ))) => return Err(error),
    };

    for fact in facts {
        if let CssSelectorSemanticFact::Relationship {
            target: staged_target,
            ..
        } = fact
        {
            *staged_target = target;
        }
    }
    Ok(None)
}

struct ResourceTracker<'a> {
    source: &'a SourceText,
    limits: CssSelectorLimits,
    algorithm_steps: usize,
    active_depth: usize,
    peak_depth: usize,
    observations: usize,
    retained_semantic_units: usize,
}

impl<'a> ResourceTracker<'a> {
    const fn new(source: &'a SourceText, limits: CssSelectorLimits) -> Self {
        Self {
            source,
            limits,
            algorithm_steps: 0,
            active_depth: 0,
            peak_depth: 0,
            observations: 0,
            retained_semantic_units: 0,
        }
    }

    const fn usage(&self) -> CssSelectorResourceUsage {
        CssSelectorResourceUsage::new(
            self.algorithm_steps,
            self.peak_depth,
            self.observations,
            self.retained_semantic_units,
        )
    }

    fn charge_algorithm_step(
        &mut self,
        header: &SourceAnchor,
        location: &SourceAnchor,
    ) -> Result<Option<CssSelectorResourceLimitEvidence>, CssSelectorProducerError> {
        let attempted = self.checked_add(self.algorithm_steps, 1)?;
        let limit = self.limits.limit(CssSelectorResourceKind::AlgorithmSteps);
        if attempted > limit {
            return Ok(Some(self.evidence(
                header,
                CssSelectorResourceKind::AlgorithmSteps,
                limit,
                attempted,
                location,
            )?));
        }
        self.algorithm_steps = attempted;
        Ok(None)
    }

    fn enter_depth(
        &mut self,
        header: &SourceAnchor,
        location: &SourceAnchor,
    ) -> Result<Option<CssSelectorResourceLimitEvidence>, CssSelectorProducerError> {
        let attempted = self.checked_add(self.active_depth, 1)?;
        let limit = self
            .limits
            .limit(CssSelectorResourceKind::PeakSelectorDepth);
        if attempted > limit {
            return Ok(Some(self.evidence(
                header,
                CssSelectorResourceKind::PeakSelectorDepth,
                limit,
                attempted,
                location,
            )?));
        }
        self.active_depth = attempted;
        self.peak_depth = self.peak_depth.max(attempted);
        Ok(None)
    }

    fn leave_depth(&mut self) -> Result<(), CssSelectorProducerError> {
        self.active_depth = self.active_depth.checked_sub(1).ok_or({
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::ActiveDepthUnderflow,
            )
        })?;
        Ok(())
    }

    fn retained_delta(&self, fact_count: usize) -> Result<usize, CssSelectorProducerError> {
        self.checked_add(1, fact_count)
    }

    fn preflight_observation(
        &self,
        header: &SourceAnchor,
    ) -> Result<ObservationPreflight, CssSelectorProducerError> {
        let attempted = self.checked_add(self.observations, 1)?;
        let limit = self.limits.limit(CssSelectorResourceKind::Observations);
        if attempted > limit {
            return Ok(ObservationPreflight::Refused(self.evidence(
                header,
                CssSelectorResourceKind::Observations,
                limit,
                attempted,
                header,
            )?));
        }
        Ok(ObservationPreflight::Allowed { attempted })
    }

    fn preflight_retained_semantic(
        &self,
        header: &SourceAnchor,
        delta: usize,
    ) -> Result<RetainedPreflight, CssSelectorProducerError> {
        let attempted = self.checked_add(self.retained_semantic_units, delta)?;
        let limit = self
            .limits
            .limit(CssSelectorResourceKind::RetainedSemanticUnits);
        if attempted > limit {
            return Ok(RetainedPreflight::Refused(self.evidence(
                header,
                CssSelectorResourceKind::RetainedSemanticUnits,
                limit,
                attempted,
                header,
            )?));
        }
        Ok(RetainedPreflight::Allowed { attempted })
    }

    fn commit_persistent(&mut self, observation_attempted: usize, retained_attempted: usize) {
        self.observations = observation_attempted;
        self.retained_semantic_units = retained_attempted;
    }

    fn checked_add(&self, current: usize, delta: usize) -> Result<usize, CssSelectorProducerError> {
        checked_resource_add(current, delta).map_err(|error| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::ResourceContract(error),
            )
        })
    }

    fn semantic_anchor(
        &self,
        start: usize,
        end: usize,
    ) -> Result<SourceAnchor, CssSelectorProducerError> {
        self.source.anchor(start, end).map_err(|error| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::SemanticRange(error),
            )
        })
    }

    fn evidence(
        &self,
        header: &SourceAnchor,
        kind: CssSelectorResourceKind,
        limit: usize,
        attempted: usize,
        location: &SourceAnchor,
    ) -> Result<CssSelectorResourceLimitEvidence, CssSelectorProducerError> {
        let offset = location.range().start();
        if offset < header.range().start() || offset > header.range().end() {
            return Err(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingCurrentToken,
            ));
        }
        let point = self.source.anchor(offset, offset).map_err(|error| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::ResourcePointRange(error),
            )
        })?;
        CssSelectorResourceLimitEvidence::new(self.source, kind, limit, attempted, point).map_err(
            |error| {
                CssSelectorProducerError::InternalInvariantFailure(
                    CssSelectorProducerInvariantViolation::ResourceContract(error),
                )
            },
        )
    }
}

enum ObservationPreflight {
    Allowed { attempted: usize },
    Refused(CssSelectorResourceLimitEvidence),
}

enum RetainedPreflight {
    Allowed { attempted: usize },
    Refused(CssSelectorResourceLimitEvidence),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryDelimiter {
    Parenthesis,
    SquareBracket,
}

#[derive(Debug)]
enum MachineFault {
    Invalid {
        reason: CssSelectorInvalidReason,
        subject: SourceAnchor,
        recovery_open: Option<RecoveryDelimiter>,
    },
    Unsupported {
        feature: CssSelectorUnsupportedFeature,
        subject: SourceAnchor,
    },
    Indeterminate {
        reason: CssSelectorIndeterminateReason,
        subject: SourceAnchor,
    },
    ResourceLimit(CssSelectorResourceLimitEvidence),
    Internal(CssSelectorProducerError),
}

#[derive(Debug)]
enum FaultResolution {
    Continue,
    Final(CssSelectorQualificationOutcome),
    ResourceLimit(CssSelectorResourceLimitEvidence),
}

#[derive(Debug, Clone, Default)]
struct MemberState {
    selector_has_any: bool,
    compound_has_any: bool,
    compound_has_nesting: bool,
    after_explicit_combinator: bool,
    pending_whitespace: bool,
    last_combinator: Option<SourceAnchor>,
}

#[derive(Debug, Clone)]
struct SemanticMemberState {
    member: CssSelectorSemanticMemberId,
    checkpoint: usize,
    open_fact_index: Option<usize>,
    semantic_start: Option<usize>,
    semantic_end: Option<usize>,
    authored_start: Option<usize>,
    authored_end: Option<usize>,
    nesting_presences: Vec<(CssSelectorSemanticUnitId, SourceAnchor)>,
}

impl SemanticMemberState {
    fn new(member: CssSelectorSemanticMemberId, checkpoint: usize) -> Self {
        Self {
            member,
            checkpoint,
            open_fact_index: None,
            semantic_start: None,
            semantic_end: None,
            authored_start: None,
            authored_end: None,
            nesting_presences: Vec::new(),
        }
    }

    fn note_authored(&mut self, anchor: &SourceAnchor) {
        let start = anchor.range().start();
        let end = anchor.range().end();
        self.authored_start = Some(self.authored_start.map_or(start, |current| current.min(start)));
        self.authored_end = Some(self.authored_end.map_or(end, |current| current.max(end)));
    }

    fn note_semantic_end(&mut self, end: usize) {
        self.semantic_end = Some(self.semantic_end.map_or(end, |current| current.max(end)));
    }
}

#[derive(Debug, Clone)]
struct ListFrame {
    function: Option<CssSelectorFunctionalPseudoClass>,
    function_subject: SourceAnchor,
    function_unit: Option<CssSelectorSemanticUnitId>,
    relative: bool,
    forgiving: bool,
    structural_relative: bool,
    after_comma: bool,
    last_comma: Option<SourceAnchor>,
    member: MemberState,
    semantic_member: SemanticMemberState,
}

impl ListFrame {
    fn root(
        header: &SourceAnchor,
        grammar_context: CssSelectorGrammarContext,
        semantic_member: SemanticMemberState,
    ) -> Self {
        let relative = matches!(
            grammar_context,
            CssSelectorGrammarContext::NestedRelativeSelectorList { .. }
                | CssSelectorGrammarContext::ScopedRelativeSelectorList { .. }
        );
        Self {
            function: None,
            function_subject: header.clone(),
            function_unit: None,
            relative,
            forgiving: false,
            structural_relative: relative,
            after_comma: false,
            last_comma: None,
            member: MemberState::default(),
            semantic_member,
        }
    }

    fn function(
        function: CssSelectorFunctionalPseudoClass,
        subject: SourceAnchor,
        function_unit: CssSelectorSemanticUnitId,
        semantic_member: SemanticMemberState,
    ) -> Self {
        Self {
            function: Some(function),
            function_subject: subject,
            function_unit: Some(function_unit),
            relative: matches!(function, CssSelectorFunctionalPseudoClass::Has),
            forgiving: matches!(
                function,
                CssSelectorFunctionalPseudoClass::Is | CssSelectorFunctionalPseudoClass::Where
            ),
            structural_relative: false,
            after_comma: false,
            last_comma: None,
            member: MemberState::default(),
            semantic_member,
        }
    }
}

#[derive(Debug, Default)]
struct SemanticBuilder {
    facts: Vec<CssSelectorSemanticFact>,
    next_member: usize,
    next_unit: usize,
}

impl SemanticBuilder {
    fn new() -> Self {
        Self {
            facts: Vec::new(),
            next_member: 1,
            next_unit: 1,
        }
    }

    fn allocate_member(
        &mut self,
    ) -> Result<CssSelectorSemanticMemberId, CssSelectorProducerError> {
        let value = self.next_member;
        self.next_member = self.next_member.checked_add(1).ok_or_else(|| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::SemanticIdentityOverflow,
            )
        })?;
        Ok(CssSelectorSemanticMemberId::new(value))
    }

    fn allocate_unit(&mut self) -> Result<CssSelectorSemanticUnitId, CssSelectorProducerError> {
        let value = self.next_unit;
        self.next_unit = self.next_unit.checked_add(1).ok_or_else(|| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::SemanticIdentityOverflow,
            )
        })?;
        Ok(CssSelectorSemanticUnitId::new(value))
    }
}

struct SelectorMachine<'tokens, 'source, 'tracker> {
    header: &'tokens SourceAnchor,
    tokens: Vec<&'tokens CssToken>,
    cursor: usize,
    frames: Vec<ListFrame>,
    semantic: SemanticBuilder,
    resources: &'tracker mut ResourceTracker<'source>,
}

impl<'tokens, 'source, 'tracker> SelectorMachine<'tokens, 'source, 'tracker> {
    fn new(
        header: &'tokens SourceAnchor,
        tokens: Vec<&'tokens CssToken>,
        grammar_context: CssSelectorGrammarContext,
        resources: &'tracker mut ResourceTracker<'source>,
    ) -> Result<Self, CssSelectorProducerError> {
        let mut semantic = SemanticBuilder::new();
        let root_member = semantic.allocate_member()?;
        let root_semantic = SemanticMemberState::new(root_member, semantic.facts.len());
        Ok(Self {
            header,
            tokens,
            cursor: 0,
            frames: vec![ListFrame::root(header, grammar_context, root_semantic)],
            semantic,
            resources,
        })
    }

    fn execute(mut self) -> Result<CandidateExecution, CssSelectorProducerError> {
        loop {
            if self.cursor >= self.tokens.len() {
                return self.finish_at_end();
            }

            let fault = match self.step() {
                Ok(()) => continue,
                Err(fault) => fault,
            };
            match self.resolve_fault(fault)? {
                FaultResolution::Continue => {}
                FaultResolution::Final(outcome) => {
                    return Ok(CandidateExecution::Outcome(outcome));
                }
                FaultResolution::ResourceLimit(evidence) => {
                    return Ok(CandidateExecution::ResourceLimit(evidence));
                }
            }
        }
    }

    fn step(&mut self) -> Result<(), MachineFault> {
        let kind = self.current_kind()?.clone();
        if !matches!(
            kind,
            CssTokenKind::Whitespace | CssTokenKind::Comma | CssTokenKind::RightParenthesis
        ) {
            let anchor = self.current_anchor()?.clone();
            self.note_authored_for_active_members(&anchor);
        }

        match kind {
            CssTokenKind::Whitespace => self.consume_whitespace(),
            CssTokenKind::Comma => self.consume_comma(),
            CssTokenKind::RightParenthesis => self.close_function_frame(),
            CssTokenKind::Delim('>') | CssTokenKind::Delim('+') | CssTokenKind::Delim('~') => {
                self.consume_explicit_combinator()
            }
            _ => {
                self.resolve_pending_whitespace()?;
                self.consume_simple_selector()
            }
        }
    }

    fn consume_whitespace(&mut self) -> Result<(), MachineFault> {
        self.consume_current()?;
        let frame = self.current_frame_mut()?;
        if frame.member.selector_has_any || frame.member.after_explicit_combinator {
            frame.member.pending_whitespace = true;
        }
        Ok(())
    }

    fn consume_comma(&mut self) -> Result<(), MachineFault> {
        let comma = self.current_anchor()?.clone();
        let classification = self.member_end_classification()?;
        let forgiving = self.current_frame()?.forgiving;
        let frame_index = self.frames.len() - 1;
        let mut semantic_already_reset = false;

        match classification {
            MemberEnd::Qualified => self.finalize_qualified_member(frame_index)?,
            MemberEnd::Empty if forgiving => {}
            MemberEnd::Empty => {
                return Err(MachineFault::Invalid {
                    reason: CssSelectorInvalidReason::UnexpectedComma,
                    subject: comma,
                    recovery_open: None,
                });
            }
            MemberEnd::TrailingCombinator { .. } if forgiving => {
                self.rollback_member_for_rejection(frame_index)
                    .map_err(MachineFault::Internal)?;
                self.finalize_rejected_member(frame_index)
                    .map_err(MachineFault::Internal)?;
                semantic_already_reset = true;
            }
            MemberEnd::TrailingCombinator { subject } => {
                return Err(MachineFault::Invalid {
                    reason: CssSelectorInvalidReason::UnexpectedCombinator,
                    subject,
                    recovery_open: None,
                });
            }
        }

        self.consume_current()?;
        let frame = self.current_frame_mut()?;
        frame.member = MemberState::default();
        frame.after_comma = true;
        frame.last_comma = Some(comma);
        if !semantic_already_reset {
            self.reset_semantic_member(frame_index)
                .map_err(MachineFault::Internal)?;
        }
        Ok(())
    }

    fn close_function_frame(&mut self) -> Result<(), MachineFault> {
        if self.frames.len() == 1 {
            return Err(MachineFault::Invalid {
                reason: CssSelectorInvalidReason::UnexpectedToken,
                subject: self.current_anchor()?.clone(),
                recovery_open: None,
            });
        }

        let right = self.current_anchor()?.clone();
        let classification = self.member_end_classification()?;
        let frame_index = self.frames.len() - 1;
        let frame_snapshot = self.current_frame()?.clone();

        match classification {
            MemberEnd::Qualified => self.finalize_qualified_member(frame_index)?,
            MemberEnd::Empty if frame_snapshot.forgiving => {}
            MemberEnd::Empty => {
                let subject = if frame_snapshot.after_comma {
                    frame_snapshot.last_comma.unwrap_or(right)
                } else {
                    frame_snapshot.function_subject
                };
                return Err(MachineFault::Invalid {
                    reason: if frame_snapshot.after_comma {
                        CssSelectorInvalidReason::UnexpectedComma
                    } else {
                        CssSelectorInvalidReason::InvalidFunctionalPseudoArgument
                    },
                    subject,
                    recovery_open: None,
                });
            }
            MemberEnd::TrailingCombinator { .. } if frame_snapshot.forgiving => {
                self.rollback_member_for_rejection(frame_index)
                    .map_err(MachineFault::Internal)?;
                self.finalize_rejected_member(frame_index)
                    .map_err(MachineFault::Internal)?;
            }
            MemberEnd::TrailingCombinator { subject } => {
                return Err(MachineFault::Invalid {
                    reason: CssSelectorInvalidReason::UnexpectedCombinator,
                    subject,
                    recovery_open: None,
                });
            }
        }

        let function_unit = self.frames[frame_index].function_unit.ok_or_else(|| {
            MachineFault::Internal(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingFunctionSemanticUnit,
            ))
        })?;
        self.consume_current()?;
        self.frames.pop();
        self.resources
            .leave_depth()
            .map_err(MachineFault::Internal)?;
        self.semantic
            .facts
            .push(CssSelectorSemanticFact::CloseFunction {
                unit: function_unit,
            });
        let parent_index = self.frames.len() - 1;
        self.frames[parent_index]
            .semantic_member
            .note_semantic_end(right.range().end());
        self.frames[parent_index]
            .semantic_member
            .note_authored(&right);
        Ok(())
    }

    fn consume_explicit_combinator(&mut self) -> Result<(), MachineFault> {
        let anchor = self.current_anchor()?.clone();
        let frame = self.current_frame()?;
        let leading_relative = !frame.member.selector_has_any
            && !frame.member.after_explicit_combinator
            && frame.relative;
        let after_compound = frame.member.compound_has_any;

        if !leading_relative && !after_compound {
            return Err(MachineFault::Invalid {
                reason: CssSelectorInvalidReason::UnexpectedCombinator,
                subject: anchor,
                recovery_open: None,
            });
        }

        self.consume_current()?;
        let frame = self.current_frame_mut()?;
        frame.member.pending_whitespace = false;
        frame.member.compound_has_any = false;
        frame.member.compound_has_nesting = false;
        frame.member.after_explicit_combinator = true;
        frame.member.last_combinator = Some(anchor);
        Ok(())
    }

    fn resolve_pending_whitespace(&mut self) -> Result<(), MachineFault> {
        let frame = self.current_frame_mut()?;
        if !frame.member.pending_whitespace {
            return Ok(());
        }
        if frame.member.compound_has_any && !frame.member.after_explicit_combinator {
            frame.member.compound_has_any = false;
            frame.member.compound_has_nesting = false;
        }
        frame.member.pending_whitespace = false;
        Ok(())
    }

    fn consume_simple_selector(&mut self) -> Result<(), MachineFault> {
        let kind = self.current_kind()?.clone();
        match kind {
            CssTokenKind::Ident(_) | CssTokenKind::Delim('*') | CssTokenKind::Delim('|') => {
                self.consume_type_or_universal()
            }
            CssTokenKind::Delim('.') => self.consume_class_selector(),
            CssTokenKind::Hash { .. } => self.consume_id_selector(),
            CssTokenKind::LeftSquareBracket => self.consume_attribute_selector(),
            CssTokenKind::Colon => self.consume_pseudo_selector(),
            CssTokenKind::Delim('&') => self.consume_nesting_selector(),
            _ => Err(MachineFault::Invalid {
                reason: CssSelectorInvalidReason::UnexpectedToken,
                subject: self.current_anchor()?.clone(),
                recovery_open: None,
            }),
        }
    }

    fn consume_type_or_universal(&mut self) -> Result<(), MachineFault> {
        if self.current_frame()?.member.compound_has_any {
            let frame = self.current_frame()?;
            return Err(MachineFault::Invalid {
                reason: if frame.member.compound_has_nesting {
                    CssSelectorInvalidReason::InvalidNestingSelectorPlacement
                } else {
                    CssSelectorInvalidReason::InvalidCompoundOrder
                },
                subject: self.current_anchor()?.clone(),
                recovery_open: None,
            });
        }

        match self.current_kind()?.clone() {
            CssTokenKind::Ident(_) => {
                if self.peek_delim(1, '|') && !self.peek_delim(2, '=') {
                    return Err(MachineFault::Indeterminate {
                        reason: CssSelectorIndeterminateReason::MissingNamespaceEnvironment,
                        subject: self.current_anchor()?.clone(),
                    });
                }
                let anchor = self.current_anchor()?.clone();
                self.consume_current()?;
                self.stage_simple(CssSelectorSemanticSimpleKind::Type, anchor)?;
                self.mark_simple(false)?;
                Ok(())
            }
            CssTokenKind::Delim('*') => {
                let first = self.current_anchor()?.clone();
                if self.peek_delim(1, '|') {
                    self.consume_current()?;
                    self.consume_current()?;
                    let final_anchor = self.current_anchor()?.clone();
                    let semantic_kind = match self.current_kind_opt() {
                        Some(CssTokenKind::Ident(_)) => CssSelectorSemanticSimpleKind::Type,
                        Some(CssTokenKind::Delim('*')) => {
                            CssSelectorSemanticSimpleKind::Universal
                        }
                        _ => {
                            return Err(MachineFault::Invalid {
                                reason: CssSelectorInvalidReason::UnexpectedToken,
                                subject: first,
                                recovery_open: None,
                            });
                        }
                    };
                    self.consume_current()?;
                    let range = self.semantic_anchor(
                        first.range().start(),
                        final_anchor.range().end(),
                    )?;
                    self.stage_simple(semantic_kind, range)?;
                    self.mark_simple(false)?;
                    Ok(())
                } else {
                    self.consume_current()?;
                    self.stage_simple(CssSelectorSemanticSimpleKind::Universal, first)?;
                    self.mark_simple(false)?;
                    Ok(())
                }
            }
            CssTokenKind::Delim('|') => {
                let first = self.current_anchor()?.clone();
                self.consume_current()?;
                let final_anchor = self.current_anchor()?.clone();
                let semantic_kind = match self.current_kind_opt() {
                    Some(CssTokenKind::Ident(_)) => CssSelectorSemanticSimpleKind::Type,
                    Some(CssTokenKind::Delim('*')) => CssSelectorSemanticSimpleKind::Universal,
                    _ => {
                        return Err(MachineFault::Invalid {
                            reason: CssSelectorInvalidReason::UnexpectedToken,
                            subject: first,
                            recovery_open: None,
                        });
                    }
                };
                self.consume_current()?;
                let range =
                    self.semantic_anchor(first.range().start(), final_anchor.range().end())?;
                self.stage_simple(semantic_kind, range)?;
                self.mark_simple(false)?;
                Ok(())
            }
            _ => Err(MachineFault::Internal(
                CssSelectorProducerError::InternalInvariantFailure(
                    CssSelectorProducerInvariantViolation::UnexpectedTypeDispatch,
                ),
            )),
        }
    }

    fn consume_class_selector(&mut self) -> Result<(), MachineFault> {
        let dot = self.current_anchor()?.clone();
        self.consume_current()?;
        if !matches!(self.current_kind_opt(), Some(CssTokenKind::Ident(_))) {
            return Err(MachineFault::Invalid {
                reason: CssSelectorInvalidReason::UnexpectedToken,
                subject: dot,
                recovery_open: None,
            });
        }
        let ident = self.current_anchor()?.clone();
        self.consume_current()?;
        let range = self.semantic_anchor(dot.range().start(), ident.range().end())?;
        self.stage_simple(CssSelectorSemanticSimpleKind::Class, range)?;
        self.mark_simple(false)
    }

    fn consume_id_selector(&mut self) -> Result<(), MachineFault> {
        let token = self.current_token()?;
        if !matches!(
            token.kind(),
            CssTokenKind::Hash {
                hash_type: CssHashType::Id,
                ..
            }
        ) {
            return Err(MachineFault::Invalid {
                reason: CssSelectorInvalidReason::UnexpectedToken,
                subject: token.source().clone(),
                recovery_open: None,
            });
        }
        let anchor = token.source().clone();
        self.consume_current()?;
        self.stage_simple(CssSelectorSemanticSimpleKind::Id, anchor)?;
        self.mark_simple(false)
    }

    fn consume_nesting_selector(&mut self) -> Result<(), MachineFault> {
        let anchor = self.current_anchor()?.clone();
        self.consume_current()?;
        self.stage_nesting(anchor)?;
        self.mark_simple(true)
    }

    fn consume_attribute_selector(&mut self) -> Result<(), MachineFault> {
        let opener = self.current_anchor()?.clone();
        self.consume_current()?;
        if let Some(evidence) = self
            .resources
            .enter_depth(self.header, &opener)
            .map_err(MachineFault::Internal)?
        {
            return Err(MachineFault::ResourceLimit(evidence));
        }

        let result = self.consume_attribute_body(opener.clone());
        match &result {
            Ok(()) => self
                .resources
                .leave_depth()
                .map_err(MachineFault::Internal)?,
            Err(MachineFault::Invalid { .. })
            | Err(MachineFault::Unsupported { .. })
            | Err(MachineFault::Indeterminate { .. }) => self
                .resources
                .leave_depth()
                .map_err(MachineFault::Internal)?,
            Err(MachineFault::ResourceLimit(_)) | Err(MachineFault::Internal(_)) => {}
        }
        result?;
        let closer = self.previous_anchor()?;
        let range = self.semantic_anchor(opener.range().start(), closer.range().end())?;
        self.stage_simple(CssSelectorSemanticSimpleKind::Attribute, range)?;
        self.mark_simple(false)
    }

    fn consume_attribute_body(&mut self, opener: SourceAnchor) -> Result<(), MachineFault> {
        self.consume_optional_whitespace()?;

        match self.current_kind_opt() {
            Some(CssTokenKind::Ident(_)) => {
                if self.peek_delim(1, '|') && !self.peek_delim(2, '=') {
                    return Err(MachineFault::Indeterminate {
                        reason: CssSelectorIndeterminateReason::MissingNamespaceEnvironment,
                        subject: self.current_anchor()?.clone(),
                    });
                }
                self.consume_current()?;
            }
            Some(CssTokenKind::Delim('*')) if self.peek_delim(1, '|') => {
                self.consume_current()?;
                self.consume_current()?;
                if !matches!(self.current_kind_opt(), Some(CssTokenKind::Ident(_))) {
                    return Err(self.invalid_attribute(opener, true));
                }
                self.consume_current()?;
            }
            Some(CssTokenKind::Delim('|')) => {
                self.consume_current()?;
                if !matches!(self.current_kind_opt(), Some(CssTokenKind::Ident(_))) {
                    return Err(self.invalid_attribute(opener, true));
                }
                self.consume_current()?;
            }
            _ => return Err(self.invalid_attribute(opener, true)),
        }

        self.consume_optional_whitespace()?;
        if matches!(
            self.current_kind_opt(),
            Some(CssTokenKind::RightSquareBracket)
        ) {
            self.consume_current()?;
            return Ok(());
        }

        if !self.consume_attribute_matcher()? {
            return Err(self.invalid_attribute(opener, true));
        }
        self.consume_optional_whitespace()?;
        if !matches!(
            self.current_kind_opt(),
            Some(CssTokenKind::Ident(_)) | Some(CssTokenKind::String(_))
        ) {
            return Err(self.invalid_attribute(opener, true));
        }
        self.consume_current()?;
        self.consume_optional_whitespace()?;

        if let Some(CssTokenKind::Ident(modifier)) = self.current_kind_opt()
            && (modifier.eq_ignore_ascii_case("i") || modifier.eq_ignore_ascii_case("s"))
        {
            self.consume_current()?;
            self.consume_optional_whitespace()?;
        }

        if !matches!(
            self.current_kind_opt(),
            Some(CssTokenKind::RightSquareBracket)
        ) {
            return Err(self.invalid_attribute(opener, true));
        }
        self.consume_current()?;
        Ok(())
    }

    fn consume_attribute_matcher(&mut self) -> Result<bool, MachineFault> {
        if self.peek_delim(0, '=') {
            self.consume_current()?;
            return Ok(true);
        }
        if matches!(
            self.current_kind_opt(),
            Some(CssTokenKind::Delim('~' | '|' | '^' | '$' | '*'))
        ) && self.peek_delim(1, '=')
        {
            self.consume_current()?;
            self.consume_current()?;
            return Ok(true);
        }
        Ok(false)
    }

    fn invalid_attribute(&self, opener: SourceAnchor, recovery_open: bool) -> MachineFault {
        MachineFault::Invalid {
            reason: CssSelectorInvalidReason::InvalidAttributeSelector,
            subject: self.current_anchor_opt().cloned().unwrap_or(opener),
            recovery_open: recovery_open.then_some(RecoveryDelimiter::SquareBracket),
        }
    }

    fn consume_pseudo_selector(&mut self) -> Result<(), MachineFault> {
        let colon = self.current_anchor()?.clone();
        self.consume_current()?;

        if matches!(self.current_kind_opt(), Some(CssTokenKind::Colon)) {
            self.consume_current()?;
            return match self.current_kind_opt() {
                Some(CssTokenKind::Ident(_)) => Err(MachineFault::Unsupported {
                    feature: CssSelectorUnsupportedFeature::PseudoElement,
                    subject: self.current_anchor()?.clone(),
                }),
                Some(CssTokenKind::Function(_)) => Err(MachineFault::Unsupported {
                    feature: CssSelectorUnsupportedFeature::FunctionalPseudoElement,
                    subject: self.current_anchor()?.clone(),
                }),
                _ => Err(MachineFault::Invalid {
                    reason: CssSelectorInvalidReason::InvalidPseudoSyntax,
                    subject: colon,
                    recovery_open: None,
                }),
            };
        }

        match self.current_kind_opt() {
            Some(CssTokenKind::Ident(name)) => {
                let subject = self.current_anchor()?.clone();
                if !is_core_v1_identifier_pseudo_class(name) {
                    return Err(MachineFault::Unsupported {
                        feature: CssSelectorUnsupportedFeature::IdentifierPseudoClass,
                        subject,
                    });
                }
                self.consume_current()?;
                let range = self.semantic_anchor(colon.range().start(), subject.range().end())?;
                self.stage_simple(CssSelectorSemanticSimpleKind::IdentifierPseudoClass, range)?;
                self.mark_simple(false)
            }
            Some(CssTokenKind::Function(name)) => {
                let subject = self.current_anchor()?.clone();
                let Some(function) = core_v1_functional_pseudo_class(name) else {
                    return Err(MachineFault::Unsupported {
                        feature: CssSelectorUnsupportedFeature::FunctionalPseudoClass,
                        subject,
                    });
                };
                if matches!(function, CssSelectorFunctionalPseudoClass::Has)
                    && self.frames.iter().any(|frame| {
                        matches!(frame.function, Some(CssSelectorFunctionalPseudoClass::Has))
                    })
                {
                    return Err(MachineFault::Invalid {
                        reason: CssSelectorInvalidReason::NestedHasNotAllowed,
                        subject,
                        recovery_open: None,
                    });
                }

                self.consume_current()?;
                let function_range =
                    self.semantic_anchor(colon.range().start(), subject.range().end())?;
                let function_unit = self.stage_open_function(function, function_range)?;
                self.mark_simple(false)?;
                if let Some(evidence) = self
                    .resources
                    .charge_algorithm_step(self.header, &subject)
                    .map_err(MachineFault::Internal)?
                {
                    return Err(MachineFault::ResourceLimit(evidence));
                }
                if let Some(evidence) = self
                    .resources
                    .enter_depth(self.header, &subject)
                    .map_err(MachineFault::Internal)?
                {
                    return Err(MachineFault::ResourceLimit(evidence));
                }
                let member = self
                    .semantic
                    .allocate_member()
                    .map_err(MachineFault::Internal)?;
                let semantic_member =
                    SemanticMemberState::new(member, self.semantic.facts.len());
                self.frames.push(ListFrame::function(
                    function,
                    subject,
                    function_unit,
                    semantic_member,
                ));
                Ok(())
            }
            _ => Err(MachineFault::Invalid {
                reason: CssSelectorInvalidReason::InvalidPseudoSyntax,
                subject: colon,
                recovery_open: None,
            }),
        }
    }

    fn consume_optional_whitespace(&mut self) -> Result<(), MachineFault> {
        while matches!(self.current_kind_opt(), Some(CssTokenKind::Whitespace)) {
            self.consume_current()?;
        }
        Ok(())
    }

    fn mark_simple(&mut self, nesting: bool) -> Result<(), MachineFault> {
        let frame = self.current_frame_mut()?;
        frame.member.selector_has_any = true;
        frame.member.compound_has_any = true;
        frame.member.compound_has_nesting |= nesting;
        frame.member.after_explicit_combinator = false;
        frame.member.pending_whitespace = false;
        frame.after_comma = false;
        Ok(())
    }

    fn member_end_classification(&self) -> Result<MemberEnd, MachineFault> {
        let member = &self.current_frame()?.member;
        if member.after_explicit_combinator {
            return Ok(MemberEnd::TrailingCombinator {
                subject: member
                    .last_combinator
                    .clone()
                    .unwrap_or_else(|| self.header.clone()),
            });
        }
        Ok(if member.selector_has_any {
            MemberEnd::Qualified
        } else {
            MemberEnd::Empty
        })
    }

    fn finish_at_end(mut self) -> Result<CandidateExecution, CssSelectorProducerError> {
        if self.frames.len() > 1 {
            let frame = self.frames.last().ok_or({
                CssSelectorProducerError::InternalInvariantFailure(
                    CssSelectorProducerInvariantViolation::MissingFunctionFrame,
                )
            })?;
            return Ok(CandidateExecution::Outcome(
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::InvalidFunctionalPseudoArgument,
                    subject: frame.function_subject.clone(),
                },
            ));
        }

        let frame = self.frames.last().ok_or({
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingRootFrame,
            )
        })?;
        let classification = self.member_end_classification_for(frame);
        match classification {
            MemberEnd::Qualified => {
                self.finalize_qualified_member(0)
                    .map_err(machine_fault_to_error)?;
                Ok(CandidateExecution::Qualified(self.semantic.facts))
            }
            MemberEnd::TrailingCombinator { subject } => Ok(CandidateExecution::Outcome(
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::UnexpectedCombinator,
                    subject,
                },
            )),
            MemberEnd::Empty if frame.after_comma => Ok(CandidateExecution::Outcome(
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::UnexpectedComma,
                    subject: frame
                        .last_comma
                        .clone()
                        .unwrap_or_else(|| frame.function_subject.clone()),
                },
            )),
            MemberEnd::Empty => Ok(CandidateExecution::Outcome(
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::EmptySelectorList,
                    subject: frame.function_subject.clone(),
                },
            )),
        }
    }

    fn member_end_classification_for(&self, frame: &ListFrame) -> MemberEnd {
        if frame.member.after_explicit_combinator {
            return MemberEnd::TrailingCombinator {
                subject: frame
                    .member
                    .last_combinator
                    .clone()
                    .unwrap_or_else(|| frame.function_subject.clone()),
            };
        }
        if frame.member.selector_has_any {
            MemberEnd::Qualified
        } else {
            MemberEnd::Empty
        }
    }

    fn resolve_fault(
        &mut self,
        fault: MachineFault,
    ) -> Result<FaultResolution, CssSelectorProducerError> {
        match fault {
            MachineFault::ResourceLimit(evidence) => Ok(FaultResolution::ResourceLimit(evidence)),
            MachineFault::Internal(error) => Err(error),
            MachineFault::Unsupported { feature, subject } => Ok(FaultResolution::Final(
                CssSelectorQualificationOutcome::UnsupportedBySelectedGrammarProfile {
                    feature,
                    subject,
                },
            )),
            MachineFault::Indeterminate { reason, subject } => Ok(FaultResolution::Final(
                CssSelectorQualificationOutcome::Indeterminate {
                    reason,
                    subject: Some(subject),
                },
            )),
            MachineFault::Invalid {
                reason,
                subject,
                recovery_open,
            } => {
                self.note_authored_for_active_members(&subject);
                let Some(forgiving_index) = self.frames.iter().rposition(|frame| frame.forgiving)
                else {
                    return Ok(FaultResolution::Final(
                        CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                            reason,
                            subject,
                        },
                    ));
                };

                self.rollback_member_for_rejection(forgiving_index)?;
                let popped_functions = self.frames.len() - 1 - forgiving_index;
                for _ in 0..popped_functions {
                    self.resources.leave_depth()?;
                }
                self.frames.truncate(forgiving_index + 1);
                self.current_frame_mut()
                    .map_err(machine_fault_to_error)?
                    .member = MemberState::default();

                match self.recover_forgiving_member(popped_functions, recovery_open)? {
                    Some(evidence) => Ok(FaultResolution::ResourceLimit(evidence)),
                    None => Ok(FaultResolution::Continue),
                }
            }
        }
    }

    fn recover_forgiving_member(
        &mut self,
        popped_functions: usize,
        recovery_open: Option<RecoveryDelimiter>,
    ) -> Result<Option<CssSelectorResourceLimitEvidence>, CssSelectorProducerError> {
        let mut delimiters = vec![RecoveryDelimiter::Parenthesis; popped_functions];
        if let Some(delimiter) = recovery_open {
            delimiters.push(delimiter);
        }

        while self.cursor < self.tokens.len() {
            let kind = self.current_kind().map_err(machine_fault_to_error)?.clone();
            if delimiters.is_empty() {
                if matches!(kind, CssTokenKind::Comma) {
                    let comma = self
                        .current_anchor()
                        .map_err(machine_fault_to_error)?
                        .clone();
                    let frame_index = self.frames.len() - 1;
                    self.finalize_rejected_member(frame_index)?;
                    if let Some(evidence) = self.consume_for_recovery(false)? {
                        return Ok(Some(evidence));
                    }
                    let frame = self.current_frame_mut().map_err(machine_fault_to_error)?;
                    frame.member = MemberState::default();
                    frame.after_comma = true;
                    frame.last_comma = Some(comma);
                    return Ok(None);
                }
                if matches!(kind, CssTokenKind::RightParenthesis) {
                    let frame_index = self.frames.len() - 1;
                    self.finalize_rejected_member(frame_index)?;
                    self.current_frame_mut()
                        .map_err(machine_fault_to_error)?
                        .member = MemberState::default();
                    return Ok(None);
                }
            }

            match kind {
                CssTokenKind::Function(_) | CssTokenKind::LeftParenthesis => {
                    delimiters.push(RecoveryDelimiter::Parenthesis)
                }
                CssTokenKind::LeftSquareBracket => {
                    delimiters.push(RecoveryDelimiter::SquareBracket)
                }
                CssTokenKind::RightParenthesis => {
                    if matches!(delimiters.last(), Some(RecoveryDelimiter::Parenthesis)) {
                        delimiters.pop();
                    }
                }
                CssTokenKind::RightSquareBracket => {
                    if matches!(delimiters.last(), Some(RecoveryDelimiter::SquareBracket)) {
                        delimiters.pop();
                    }
                }
                _ => {}
            }
            if let Some(evidence) = self.consume_for_recovery(true)? {
                return Ok(Some(evidence));
            }
        }

        Ok(None)
    }

    fn consume_for_recovery(
        &mut self,
        retain_as_member_content: bool,
    ) -> Result<Option<CssSelectorResourceLimitEvidence>, CssSelectorProducerError> {
        let anchor = self
            .current_anchor()
            .map_err(machine_fault_to_error)?
            .clone();
        let kind = self
            .current_kind()
            .map_err(machine_fault_to_error)?
            .clone();
        if let Some(evidence) = self.resources.charge_algorithm_step(self.header, &anchor)? {
            return Ok(Some(evidence));
        }
        if retain_as_member_content && !matches!(kind, CssTokenKind::Whitespace) {
            self.note_authored_for_active_members(&anchor);
            if matches!(kind, CssTokenKind::Delim('&')) {
                let unit = self.semantic.allocate_unit()?;
                self.record_nesting_presence_for_active_members(unit, anchor.clone());
            }
        }
        self.cursor += 1;
        Ok(None)
    }

    fn consume_current(&mut self) -> Result<(), MachineFault> {
        let anchor = self.current_anchor()?.clone();
        if let Some(evidence) = self
            .resources
            .charge_algorithm_step(self.header, &anchor)
            .map_err(MachineFault::Internal)?
        {
            return Err(MachineFault::ResourceLimit(evidence));
        }
        self.cursor += 1;
        Ok(())
    }

    fn semantic_anchor(&self, start: usize, end: usize) -> Result<SourceAnchor, MachineFault> {
        self.resources
            .semantic_anchor(start, end)
            .map_err(MachineFault::Internal)
    }

    fn ensure_current_member_open(&mut self, anchor: &SourceAnchor) -> Result<(), MachineFault> {
        let frame_index = self.frames.len() - 1;
        let needs_open = self.frames[frame_index]
            .semantic_member
            .open_fact_index
            .is_none();
        if needs_open {
            let member = self.frames[frame_index].semantic_member.member;
            let fact_index = self.semantic.facts.len();
            self.semantic
                .facts
                .push(CssSelectorSemanticFact::OpenMember {
                    member,
                    range: anchor.clone(),
                });
            let semantic_member = &mut self.frames[frame_index].semantic_member;
            semantic_member.open_fact_index = Some(fact_index);
            semantic_member.semantic_start = Some(anchor.range().start());
            semantic_member.semantic_end = Some(anchor.range().end());
        } else {
            self.frames[frame_index]
                .semantic_member
                .note_semantic_end(anchor.range().end());
        }
        Ok(())
    }

    fn stage_simple(
        &mut self,
        kind: CssSelectorSemanticSimpleKind,
        range: SourceAnchor,
    ) -> Result<(), MachineFault> {
        self.ensure_current_member_open(&range)?;
        let unit = self
            .semantic
            .allocate_unit()
            .map_err(MachineFault::Internal)?;
        self.semantic
            .facts
            .push(CssSelectorSemanticFact::Simple { unit, kind, range });
        Ok(())
    }

    fn stage_open_function(
        &mut self,
        function: CssSelectorFunctionalPseudoClass,
        range: SourceAnchor,
    ) -> Result<CssSelectorSemanticUnitId, MachineFault> {
        self.ensure_current_member_open(&range)?;
        let unit = self
            .semantic
            .allocate_unit()
            .map_err(MachineFault::Internal)?;
        self.semantic
            .facts
            .push(CssSelectorSemanticFact::OpenFunction {
                unit,
                kind: semantic_function_kind(function),
                range,
            });
        Ok(unit)
    }

    fn stage_nesting(&mut self, range: SourceAnchor) -> Result<(), MachineFault> {
        self.ensure_current_member_open(&range)?;
        let unit = self
            .semantic
            .allocate_unit()
            .map_err(MachineFault::Internal)?;
        let member = self.current_frame()?.semantic_member.member;
        self.record_nesting_presence_for_active_members(unit, range.clone());
        self.semantic
            .facts
            .push(CssSelectorSemanticFact::NestingPresence {
                member,
                unit,
                origin: CssSelectorSemanticRelationshipOrigin::Authored(range.clone()),
                disposition: CssSelectorNestingPresenceDisposition::Contributing,
            });
        self.semantic
            .facts
            .push(CssSelectorSemanticFact::Relationship {
                target: CssSelectorSemanticRelationshipTarget::Zero,
                origin: CssSelectorSemanticRelationshipOrigin::Authored(range),
            });
        Ok(())
    }

    fn record_nesting_presence_for_active_members(
        &mut self,
        unit: CssSelectorSemanticUnitId,
        range: SourceAnchor,
    ) {
        for frame in &mut self.frames {
            frame
                .semantic_member
                .nesting_presences
                .push((unit, range.clone()));
        }
    }

    fn note_authored_for_active_members(&mut self, anchor: &SourceAnchor) {
        for frame in &mut self.frames {
            frame.semantic_member.note_authored(anchor);
        }
    }

    fn finalize_qualified_member(&mut self, frame_index: usize) -> Result<(), MachineFault> {
        let semantic_member = &self.frames[frame_index].semantic_member;
        let member = semantic_member.member;
        let open_fact_index = semantic_member.open_fact_index.ok_or_else(|| {
            MachineFault::Internal(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingSemanticMember,
            ))
        })?;
        let start = semantic_member.semantic_start.ok_or_else(|| {
            MachineFault::Internal(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingSemanticMember,
            ))
        })?;
        let end = semantic_member.semantic_end.ok_or_else(|| {
            MachineFault::Internal(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingSemanticMember,
            ))
        })?;
        let requires_implied_relationship = self.frames[frame_index].structural_relative
            && semantic_member.nesting_presences.is_empty();
        let range = self.semantic_anchor(start, end)?;

        match self.semantic.facts.get_mut(open_fact_index) {
            Some(CssSelectorSemanticFact::OpenMember {
                member: actual,
                range: actual_range,
            }) if *actual == member => {
                *actual_range = range;
            }
            _ => {
                return Err(MachineFault::Internal(
                    CssSelectorProducerError::InternalInvariantFailure(
                        CssSelectorProducerInvariantViolation::SemanticOpenFactMismatch,
                    ),
                ));
            }
        }

        if requires_implied_relationship {
            self.semantic.facts.insert(
                open_fact_index + 1,
                CssSelectorSemanticFact::Relationship {
                    target: CssSelectorSemanticRelationshipTarget::Zero,
                    origin: CssSelectorSemanticRelationshipOrigin::Derived,
                },
            );
        }
        self.semantic
            .facts
            .push(CssSelectorSemanticFact::CloseMember { member });
        Ok(())
    }

    fn rollback_member_for_rejection(
        &mut self,
        frame_index: usize,
    ) -> Result<(), CssSelectorProducerError> {
        let checkpoint = self
            .frames
            .get(frame_index)
            .ok_or_else(|| {
                CssSelectorProducerError::InternalInvariantFailure(
                    CssSelectorProducerInvariantViolation::MissingFunctionFrame,
                )
            })?
            .semantic_member
            .checkpoint;
        self.semantic.facts.truncate(checkpoint);
        let semantic_member = &mut self.frames[frame_index].semantic_member;
        semantic_member.open_fact_index = None;
        semantic_member.semantic_start = None;
        semantic_member.semantic_end = None;
        Ok(())
    }

    fn finalize_rejected_member(
        &mut self,
        frame_index: usize,
    ) -> Result<(), CssSelectorProducerError> {
        let state = self
            .frames
            .get(frame_index)
            .ok_or_else(|| {
                CssSelectorProducerError::InternalInvariantFailure(
                    CssSelectorProducerInvariantViolation::MissingFunctionFrame,
                )
            })?
            .semantic_member
            .clone();

        if let (Some(start), Some(end)) = (state.authored_start, state.authored_end)
            && start < end
        {
            let range = self.resources.semantic_anchor(start, end)?;
            self.semantic
                .facts
                .push(CssSelectorSemanticFact::RejectedForgivingMember {
                    member: state.member,
                    range,
                });
            for (unit, authored) in state.nesting_presences {
                self.semantic
                    .facts
                    .push(CssSelectorSemanticFact::NestingPresence {
                        member: state.member,
                        unit,
                        origin: CssSelectorSemanticRelationshipOrigin::Authored(authored),
                        disposition:
                            CssSelectorNestingPresenceDisposition::NonContributingPresenceOnly,
                    });
            }
        }
        self.reset_semantic_member(frame_index)
    }

    fn reset_semantic_member(
        &mut self,
        frame_index: usize,
    ) -> Result<(), CssSelectorProducerError> {
        let member = self.semantic.allocate_member()?;
        let checkpoint = self.semantic.facts.len();
        let frame = self.frames.get_mut(frame_index).ok_or_else(|| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingFunctionFrame,
            )
        })?;
        frame.semantic_member = SemanticMemberState::new(member, checkpoint);
        Ok(())
    }

    fn current_frame(&self) -> Result<&ListFrame, MachineFault> {
        self.frames.last().ok_or({
            MachineFault::Internal(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingFunctionFrame,
            ))
        })
    }

    fn current_frame_mut(&mut self) -> Result<&mut ListFrame, MachineFault> {
        self.frames.last_mut().ok_or({
            MachineFault::Internal(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingFunctionFrame,
            ))
        })
    }

    fn current_token(&self) -> Result<&CssToken, MachineFault> {
        self.tokens.get(self.cursor).copied().ok_or({
            MachineFault::Internal(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingCurrentToken,
            ))
        })
    }

    fn current_anchor(&self) -> Result<&SourceAnchor, MachineFault> {
        Ok(self.current_token()?.source())
    }

    fn previous_anchor(&self) -> Result<&SourceAnchor, MachineFault> {
        self.cursor
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index).copied())
            .map(CssToken::source)
            .ok_or({
                MachineFault::Internal(CssSelectorProducerError::InternalInvariantFailure(
                    CssSelectorProducerInvariantViolation::MissingCurrentToken,
                ))
            })
    }

    fn current_anchor_opt(&self) -> Option<&SourceAnchor> {
        self.tokens.get(self.cursor).map(|token| token.source())
    }

    fn current_kind(&self) -> Result<&CssTokenKind, MachineFault> {
        Ok(self.current_token()?.kind())
    }

    fn current_kind_opt(&self) -> Option<&CssTokenKind> {
        self.tokens.get(self.cursor).map(|token| token.kind())
    }

    fn peek_delim(&self, offset: usize, delimiter: char) -> bool {
        matches!(
            self.tokens.get(self.cursor + offset).map(|token| token.kind()),
            Some(CssTokenKind::Delim(actual)) if *actual == delimiter
        )
    }
}

#[derive(Debug)]
enum MemberEnd {
    Qualified,
    Empty,
    TrailingCombinator { subject: SourceAnchor },
}

fn semantic_function_kind(
    function: CssSelectorFunctionalPseudoClass,
) -> CssSelectorSemanticFunctionKind {
    match function {
        CssSelectorFunctionalPseudoClass::Is => CssSelectorSemanticFunctionKind::Is,
        CssSelectorFunctionalPseudoClass::Where => CssSelectorSemanticFunctionKind::Where,
        CssSelectorFunctionalPseudoClass::Not => CssSelectorSemanticFunctionKind::Not,
        CssSelectorFunctionalPseudoClass::Has => CssSelectorSemanticFunctionKind::Has,
    }
}

fn machine_fault_to_error(fault: MachineFault) -> CssSelectorProducerError {
    match fault {
        MachineFault::Internal(error) => error,
        _ => CssSelectorProducerError::InternalInvariantFailure(
            CssSelectorProducerInvariantViolation::UnexpectedFaultConversion,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::analysis::analyze_css_source;
    use crate::css::parser::resource::CssParserLimits;
    use crate::css::tokenizer::resource::CssTokenizerLimits;
    use crate::{SourceId, SourceText};

    fn tokenizer_limits() -> CssTokenizerLimits {
        CssTokenizerLimits::new(4096, 100_000, 8192, 1024, 8192, 8192).unwrap()
    }

    fn parser_limits() -> CssParserLimits {
        CssParserLimits::new(100_000, 256, 256, 8192, 1024, 1024, 1024, 1024, 8192).unwrap()
    }

    fn selector_limits() -> CssSelectorLimits {
        CssSelectorLimits::new(100_000, 64, 8192, 100_000).unwrap()
    }

    fn qualify(source: &SourceText) -> CssSelectorQualificationRunResult {
        let parser = analyze_css_source(source, tokenizer_limits(), parser_limits()).unwrap();
        run(source, parser, selector_limits()).unwrap()
    }

    fn first_outcome(source: &str) -> CssSelectorQualificationOutcome {
        let source = SourceText::new(SourceId::new(1), source.to_owned());
        qualify(&source).observations()[0].outcome().clone()
    }

    #[test]
    fn basic_complex_and_namespace_free_selectors_qualify() {
        for source in [
            "a > .b + #c ~ [x=\"y\" i]{}",
            "*|a{}",
            "|a{}",
            ".a .b{}",
            "a/**/.b{}",
        ] {
            let source_text = SourceText::new(SourceId::new(10), source.to_owned());
            let result = qualify(&source_text);
            assert!(matches!(
                result.observations()[0].outcome(),
                CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
            ));
            assert!(result.observations()[0].semantic_program().is_some());
        }
    }

    #[test]
    fn named_namespace_is_indeterminate_without_environment() {
        assert!(matches!(
            first_outcome("svg|a{}"),
            CssSelectorQualificationOutcome::Indeterminate {
                reason: CssSelectorIndeterminateReason::MissingNamespaceEnvironment,
                ..
            }
        ));
    }

    #[test]
    fn nesting_type_order_and_relative_leading_combinator_are_distinct() {
        let source = SourceText::new(SourceId::new(2), ".a{&Bar{} Bar&{} > .b{}}".to_owned());
        let result = qualify(&source);
        assert_eq!(result.observations().len(), 4);
        assert!(matches!(
            result.observations()[1].outcome(),
            CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                reason: CssSelectorInvalidReason::InvalidNestingSelectorPlacement,
                ..
            }
        ));
        assert!(matches!(
            result.observations()[2].outcome(),
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
        ));
        assert!(matches!(
            result.observations()[3].outcome(),
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
        ));
    }

    #[test]
    fn trailing_outer_comma_is_invalid_while_forgiving_function_may_drop_empty_member() {
        assert!(matches!(
            first_outcome("a,{}"),
            CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                reason: CssSelectorInvalidReason::UnexpectedComma,
                ..
            }
        ));
        assert!(matches!(
            first_outcome(":is(.a,){}"),
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
        ));
    }

    #[test]
    fn selected_function_profile_preserves_forgiving_and_unforgiving_behavior() {
        for source in [":is(.a, > > .b, .c){}", ":where(.a,,.b){}", ":has(> .a){}"] {
            assert!(matches!(
                first_outcome(source),
                CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
            ));
        }
        assert!(matches!(
            first_outcome(":not(.a, > > .b){}"),
            CssSelectorQualificationOutcome::InvalidForSelectedGrammar { .. }
        ));
        assert!(matches!(
            first_outcome(":has(:has(.a)){}"),
            CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                reason: CssSelectorInvalidReason::NestedHasNotAllowed,
                ..
            }
        ));
    }

    #[test]
    fn unsupported_inside_forgiving_function_is_not_silently_dropped() {
        assert!(matches!(
            first_outcome(":is(.a,:future-pseudo,.b){}"),
            CssSelectorQualificationOutcome::UnsupportedBySelectedGrammarProfile {
                feature: CssSelectorUnsupportedFeature::IdentifierPseudoClass,
                ..
            }
        ));
    }

    #[test]
    fn lower_level_run_rejects_same_source_id_with_different_retained_bytes() {
        let source = SourceText::new(SourceId::new(4), "a{}".to_owned());
        let different = SourceText::new(SourceId::new(4), "b{}".to_owned());
        let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();

        assert!(matches!(
            run(&different, parser, selector_limits()),
            Err(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::UpstreamSourceMismatch
            ))
        ));
    }

    #[test]
    fn observation_resource_refusal_preserves_committed_prefix() {
        let source = SourceText::new(SourceId::new(3), "a{}b{}".to_owned());
        let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();
        let limits = CssSelectorLimits::new(100_000, 64, 1, 100_000).unwrap();
        let result = run(&source, parser, limits).unwrap();
        assert_eq!(result.observations().len(), 1);
        assert_eq!(
            result.execution_completion(),
            CssSelectorExecutionCompletion::Incomplete
        );
        assert!(matches!(
            result.termination(),
            CssSelectorTermination::ResourceLimit(evidence)
                if evidence.kind() == CssSelectorResourceKind::Observations
        ));
    }

    #[test]
    fn retained_semantic_refusal_preserves_committed_prefix_and_usage() {
        let source = SourceText::new(SourceId::new(5), "a{}b{}".to_owned());
        let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();
        let limits = CssSelectorLimits::new(100_000, 64, 8, 4).unwrap();
        let result = run(&source, parser, limits).unwrap();
        assert_eq!(result.observations().len(), 1);
        assert_eq!(
            result.resources().value(CssSelectorResourceKind::RetainedSemanticUnits),
            4
        );
        assert!(matches!(
            result.termination(),
            CssSelectorTermination::ResourceLimit(evidence)
                if evidence.kind() == CssSelectorResourceKind::RetainedSemanticUnits
        ));
    }

    #[test]
    fn observations_refusal_precedes_retained_semantic_refusal() {
        let source = SourceText::new(SourceId::new(6), "a{}".to_owned());
        let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();
        let limits = CssSelectorLimits::new(100_000, 64, 0, 0).unwrap();
        let result = run(&source, parser, limits).unwrap();
        assert!(result.observations().is_empty());
        assert!(matches!(
            result.termination(),
            CssSelectorTermination::ResourceLimit(evidence)
                if evidence.kind() == CssSelectorResourceKind::Observations
        ));
        assert_eq!(
            result.resources().value(CssSelectorResourceKind::RetainedSemanticUnits),
            0
        );
    }

    #[test]
    fn retained_semantic_overflow_is_internal_and_non_mutating() {
        let source = SourceText::new(SourceId::new(7), "a{}".to_owned());
        let limits = CssSelectorLimits::new(100_000, 64, usize::MAX, usize::MAX).unwrap();
        let mut tracker = ResourceTracker::new(&source, limits);
        tracker.retained_semantic_units = usize::MAX;
        assert!(matches!(
            tracker.preflight_retained_semantic(source.anchor(0, 1).unwrap().borrow(), 1),
            Err(CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::ResourceContract(
                    CssSelectorResourceContractError::AccountingOverflow { .. }
                )
            ))
        ));
        assert_eq!(tracker.retained_semantic_units, usize::MAX);
    }
}
