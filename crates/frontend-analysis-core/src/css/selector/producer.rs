//! Bounded production CSS selector qualification for `CoreV1` (#184).
//!
//! Grammar meaning is derived only from the Core-validated parser result and
//! the retained tokenizer lexical items selected by `QualifiedRuleBlock`
//! headers. `SourceText` is borrowed solely to construct exact zero-length
//! resource-limit point evidence; authored bytes are never searched, rescanned,
//! retokenized, or reconstructed here.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceRangeError, SourceText};

use super::super::parser::context::CssParserContextKind;
use super::super::parser::result::CssParserRunResult;
use super::super::token::{CssHashType, CssLexicalItem, CssToken, CssTokenKind};
use super::context::{
    CssSelectorContextContractError, CssSelectorGrammarContext, derive_selector_grammar_context,
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
    ResourceContract(CssSelectorResourceContractError),
    ResourcePointRange(SourceRangeError),
    ResultContract(CssSelectorInvariantViolation),
    ActiveDepthUnderflow,
    MissingCurrentToken,
    MissingFunctionFrame,
    MissingRootFrame,
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

        let outcome = match execution {
            CandidateExecution::Outcome(outcome) => outcome,
            CandidateExecution::ResourceLimit(evidence) => {
                return build_incomplete(parser_result, observations, resources.usage(), evidence);
            }
        };

        match resources.preflight_observation(record.header())? {
            ObservationPreflight::Allowed { attempted } => {
                let observation = CssSelectorQualificationObservation::new(
                    &parser_result,
                    record.id(),
                    grammar_context,
                    outcome,
                )?;
                observations.push(observation);
                resources.commit_observation(attempted);
            }
            ObservationPreflight::Refused(evidence) => {
                return build_incomplete(parser_result, observations, resources.usage(), evidence);
            }
        }
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

    SelectorMachine::new(header, tokens, grammar_context, resources).execute()
}

struct ResourceTracker<'a> {
    source: &'a SourceText,
    limits: CssSelectorLimits,
    algorithm_steps: usize,
    active_depth: usize,
    peak_depth: usize,
    observations: usize,
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
        }
    }

    const fn usage(&self) -> CssSelectorResourceUsage {
        CssSelectorResourceUsage::new(self.algorithm_steps, self.peak_depth, self.observations)
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

    fn commit_observation(&mut self, attempted: usize) {
        self.observations = attempted;
    }

    fn checked_add(&self, current: usize, delta: usize) -> Result<usize, CssSelectorProducerError> {
        checked_resource_add(current, delta).map_err(|error| {
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::ResourceContract(error),
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
struct ListFrame {
    function: Option<CssSelectorFunctionalPseudoClass>,
    function_subject: SourceAnchor,
    relative: bool,
    forgiving: bool,
    after_comma: bool,
    last_comma: Option<SourceAnchor>,
    member: MemberState,
}

impl ListFrame {
    fn root(header: &SourceAnchor, grammar_context: CssSelectorGrammarContext) -> Self {
        let relative = matches!(
            grammar_context,
            CssSelectorGrammarContext::NestedRelativeSelectorList { .. }
                | CssSelectorGrammarContext::ScopedRelativeSelectorList { .. }
        );
        Self {
            function: None,
            function_subject: header.clone(),
            relative,
            forgiving: false,
            after_comma: false,
            last_comma: None,
            member: MemberState::default(),
        }
    }

    fn function(function: CssSelectorFunctionalPseudoClass, subject: SourceAnchor) -> Self {
        Self {
            function: Some(function),
            function_subject: subject,
            relative: matches!(function, CssSelectorFunctionalPseudoClass::Has),
            forgiving: matches!(
                function,
                CssSelectorFunctionalPseudoClass::Is | CssSelectorFunctionalPseudoClass::Where
            ),
            after_comma: false,
            last_comma: None,
            member: MemberState::default(),
        }
    }
}

struct SelectorMachine<'tokens, 'source, 'tracker> {
    header: &'tokens SourceAnchor,
    tokens: Vec<&'tokens CssToken>,
    cursor: usize,
    frames: Vec<ListFrame>,
    resources: &'tracker mut ResourceTracker<'source>,
}

impl<'tokens, 'source, 'tracker> SelectorMachine<'tokens, 'source, 'tracker> {
    fn new(
        header: &'tokens SourceAnchor,
        tokens: Vec<&'tokens CssToken>,
        grammar_context: CssSelectorGrammarContext,
        resources: &'tracker mut ResourceTracker<'source>,
    ) -> Self {
        Self {
            header,
            tokens,
            cursor: 0,
            frames: vec![ListFrame::root(header, grammar_context)],
            resources,
        }
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
        match self.current_kind()? {
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

        match classification {
            MemberEnd::Qualified => {}
            MemberEnd::Empty if forgiving => {}
            MemberEnd::Empty => {
                return Err(MachineFault::Invalid {
                    reason: CssSelectorInvalidReason::UnexpectedComma,
                    subject: comma,
                    recovery_open: None,
                });
            }
            MemberEnd::TrailingCombinator { subject } if forgiving => {
                let _ = subject;
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
        let frame = self.current_frame()?.clone();

        match classification {
            MemberEnd::Qualified => {}
            MemberEnd::Empty if frame.forgiving => {}
            MemberEnd::Empty => {
                let subject = if frame.after_comma {
                    frame.last_comma.unwrap_or(right)
                } else {
                    frame.function_subject
                };
                return Err(MachineFault::Invalid {
                    reason: if frame.after_comma {
                        CssSelectorInvalidReason::UnexpectedComma
                    } else {
                        CssSelectorInvalidReason::InvalidFunctionalPseudoArgument
                    },
                    subject,
                    recovery_open: None,
                });
            }
            MemberEnd::TrailingCombinator { .. } if frame.forgiving => {}
            MemberEnd::TrailingCombinator { subject } => {
                return Err(MachineFault::Invalid {
                    reason: CssSelectorInvalidReason::UnexpectedCombinator,
                    subject,
                    recovery_open: None,
                });
            }
        }

        self.consume_current()?;
        self.frames.pop();
        self.resources
            .leave_depth()
            .map_err(MachineFault::Internal)?;
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
                self.consume_current()?;
                self.mark_simple(false)?;
                Ok(())
            }
            CssTokenKind::Delim('*') => {
                if self.peek_delim(1, '|') {
                    let star = self.current_anchor()?.clone();
                    self.consume_current()?;
                    self.consume_current()?;
                    match self.current_kind_opt() {
                        Some(CssTokenKind::Ident(_)) | Some(CssTokenKind::Delim('*')) => {
                            self.consume_current()?;
                            self.mark_simple(false)?;
                            Ok(())
                        }
                        _ => Err(MachineFault::Invalid {
                            reason: CssSelectorInvalidReason::UnexpectedToken,
                            subject: star,
                            recovery_open: None,
                        }),
                    }
                } else {
                    self.consume_current()?;
                    self.mark_simple(false)?;
                    Ok(())
                }
            }
            CssTokenKind::Delim('|') => {
                let pipe = self.current_anchor()?.clone();
                self.consume_current()?;
                match self.current_kind_opt() {
                    Some(CssTokenKind::Ident(_)) | Some(CssTokenKind::Delim('*')) => {
                        self.consume_current()?;
                        self.mark_simple(false)?;
                        Ok(())
                    }
                    _ => Err(MachineFault::Invalid {
                        reason: CssSelectorInvalidReason::UnexpectedToken,
                        subject: pipe,
                        recovery_open: None,
                    }),
                }
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
        self.consume_current()?;
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
        self.consume_current()?;
        self.mark_simple(false)
    }

    fn consume_nesting_selector(&mut self) -> Result<(), MachineFault> {
        self.consume_current()?;
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
                self.frames.push(ListFrame::function(function, subject));
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

        let frame = self.frames.pop().ok_or({
            CssSelectorProducerError::InternalInvariantFailure(
                CssSelectorProducerInvariantViolation::MissingRootFrame,
            )
        })?;
        let outcome = match self.member_end_classification_for(&frame) {
            MemberEnd::Qualified => CssSelectorQualificationOutcome::QualifiedBySelectedGrammar,
            MemberEnd::TrailingCombinator { subject } => {
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::UnexpectedCombinator,
                    subject,
                }
            }
            MemberEnd::Empty if frame.after_comma => {
                CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::UnexpectedComma,
                    subject: frame.last_comma.unwrap_or(frame.function_subject),
                }
            }
            MemberEnd::Empty => CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                reason: CssSelectorInvalidReason::EmptySelectorList,
                subject: frame.function_subject,
            },
        };
        Ok(CandidateExecution::Outcome(outcome))
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
                let Some(forgiving_index) = self.frames.iter().rposition(|frame| frame.forgiving)
                else {
                    return Ok(FaultResolution::Final(
                        CssSelectorQualificationOutcome::InvalidForSelectedGrammar {
                            reason,
                            subject,
                        },
                    ));
                };

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
                    if let Some(evidence) = self.consume_for_recovery()? {
                        return Ok(Some(evidence));
                    }
                    let frame = self.current_frame_mut().map_err(machine_fault_to_error)?;
                    frame.member = MemberState::default();
                    frame.after_comma = true;
                    frame.last_comma = Some(comma);
                    return Ok(None);
                }
                if matches!(kind, CssTokenKind::RightParenthesis) {
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
            if let Some(evidence) = self.consume_for_recovery()? {
                return Ok(Some(evidence));
            }
        }

        Ok(None)
    }

    fn consume_for_recovery(
        &mut self,
    ) -> Result<Option<CssSelectorResourceLimitEvidence>, CssSelectorProducerError> {
        let anchor = self
            .current_anchor()
            .map_err(machine_fault_to_error)?
            .clone();
        if let Some(evidence) = self.resources.charge_algorithm_step(self.header, &anchor)? {
            return Ok(Some(evidence));
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
        CssSelectorLimits::new(100_000, 64, 8192).unwrap()
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
            assert!(matches!(
                first_outcome(source),
                CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
            ));
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
    fn resource_refusal_preserves_observation_prefix() {
        let source = SourceText::new(SourceId::new(3), "a{}b{}".to_owned());
        let parser = analyze_css_source(&source, tokenizer_limits(), parser_limits()).unwrap();
        let limits = CssSelectorLimits::new(100_000, 64, 1).unwrap();
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
}
