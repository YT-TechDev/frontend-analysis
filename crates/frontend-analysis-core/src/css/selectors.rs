//! Public CSS `CoreV1` selector-analysis consumer facade (#857, ADR 0011).
//!
//! This is the single intentional public analysis boundary over the existing
//! crate-private CSS pipeline:
//!
//! ```text
//! &SourceText
//!   -> existing CSS tokenizer
//!   -> existing CSS parser
//!   -> existing Core source-evidence reconciliation
//!   -> existing CSS CoreV1 selector qualification
//!   -> owned CssSelectorReport
//! ```
//!
//! The facade applies the finite Phase 1 execution envelope approved in Issue
//! #857 internally; those values are execution policy, not CSS semantics, and
//! are not caller-configurable. Every report value is projected from evidence
//! already retained by the internal run result. No source text is searched,
//! rescanned, retokenized, reparsed, or otherwise reconstructed here.
//!
//! The tokenizer, parser, and selector implementation types remain
//! crate-private; this module mirrors only the meaning a consumer needs.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

use super::analysis::CssAnalysisError;
use super::parser::resource as parser_resource;
use super::parser::result as parser_result;
use super::selector::analysis::{CssSelectorAnalysisError, analyze_css_selectors};
use super::selector::context as selector_context;
use super::selector::profile as selector_profile;
use super::selector::resource as selector_resource;
use super::selector::result as selector_result;
use super::tokenizer::resource as tokenizer_resource;
use super::tokenizer::result as tokenizer_result;

#[cfg(test)]
mod tests;

// Issue #857 Phase 1 execution envelope (approved in #857 comment
// 5924774040). Execution policy only; not CSS semantics or public API.
const TOKENIZER_SOURCE_BYTES: usize = 32_768;
const TOKENIZER_ALGORITHM_STEPS: usize = 65_536;
const TOKENIZER_LEXICAL_ITEMS: usize = 8_192;
const TOKENIZER_DIAGNOSTICS: usize = 256;
const TOKENIZER_RETAINED_INTERPRETED_BYTES: usize = 98_304;
const TOKENIZER_TEMPORARY_BUFFER_BYTES: usize = 98_310;

const PARSER_ALGORITHM_STEPS: usize = 65_536;
const PARSER_PEAK_COMPONENT_DEPTH: usize = 32;
const PARSER_PEAK_CONTEXT_DEPTH: usize = 32;
const PARSER_DECLARATION_OCCURRENCES: usize = 512;
const PARSER_DIAGNOSTICS: usize = 129;
const PARSER_RECOVERY_RECORDS: usize = 128;
const PARSER_UNSUPPORTED_REGIONS: usize = 64;
const PARSER_DISCARD_RECORDS: usize = 32;
const PARSER_CONTEXT_RECORDS: usize = 2_048;

const SELECTOR_ALGORITHM_STEPS: usize = 16_384;
const SELECTOR_PEAK_SELECTOR_DEPTH: usize = 16;
const SELECTOR_OBSERVATIONS: usize = 1_024;

/// Runs the existing CSS tokenizer, parser, Core source-evidence
/// reconciliation, and `CoreV1` selector qualification over `source` under
/// the fixed Phase 1 execution envelope.
///
/// Ordinary analysis outcomes, including invalid, unsupported, indeterminate,
/// incomplete, and resource-refused results, are returned as `Ok`. `Err` is
/// reserved for a returned Core boundary or internal contract failure.
///
/// The returned report owns its source evidence and remains usable after the
/// caller drops `source`.
pub fn analyze_core_v1(source: &SourceText) -> Result<CssSelectorReport, CssSelectorCoreFailure> {
    let tokenizer_limits = tokenizer_resource::CssTokenizerLimits::new(
        TOKENIZER_SOURCE_BYTES,
        TOKENIZER_ALGORITHM_STEPS,
        TOKENIZER_LEXICAL_ITEMS,
        TOKENIZER_DIAGNOSTICS,
        TOKENIZER_RETAINED_INTERPRETED_BYTES,
        TOKENIZER_TEMPORARY_BUFFER_BYTES,
    )
    .map_err(|_| CssSelectorCoreFailure::execution_policy())?;
    let parser_limits = parser_resource::CssParserLimits::new(
        PARSER_ALGORITHM_STEPS,
        PARSER_PEAK_COMPONENT_DEPTH,
        PARSER_PEAK_CONTEXT_DEPTH,
        PARSER_DECLARATION_OCCURRENCES,
        PARSER_DIAGNOSTICS,
        PARSER_RECOVERY_RECORDS,
        PARSER_UNSUPPORTED_REGIONS,
        PARSER_DISCARD_RECORDS,
        PARSER_CONTEXT_RECORDS,
    )
    .map_err(|_| CssSelectorCoreFailure::execution_policy())?;
    let selector_limits = selector_resource::CssSelectorLimits::new(
        SELECTOR_ALGORITHM_STEPS,
        SELECTOR_PEAK_SELECTOR_DEPTH,
        SELECTOR_OBSERVATIONS,
    )
    .map_err(|_| CssSelectorCoreFailure::execution_policy())?;

    let run = analyze_css_selectors(source, tokenizer_limits, parser_limits, selector_limits)
        .map_err(CssSelectorCoreFailure::analysis)?;
    Ok(CssSelectorReport::project(&run))
}

/// The owned result of one [`analyze_core_v1`] invocation.
///
/// Stage completion is reported per stage: a `Complete` selector stage means
/// every qualified-rule context retained by the parser was processed, not that
/// the whole source was analyzed. Consumers must consult the tokenizer and
/// parser stages before claiming whole-source coverage.
#[derive(Debug, Clone)]
pub struct CssSelectorReport {
    source_id: SourceId,
    profile: CssSelectorProfile,
    tokenizer: CssTokenizerStage,
    parser: CssParserStage,
    selector: CssSelectorStage,
    observations: Vec<CssSelectorObservation>,
}

impl CssSelectorReport {
    fn project(run: &selector_result::CssSelectorQualificationRunResult) -> Self {
        let parser = run.upstream_parser_result();
        let tokenizer = parser.upstream_tokenizer_result();
        Self {
            source_id: tokenizer.source_id(),
            profile: CssSelectorProfile::project(run.profile()),
            tokenizer: CssTokenizerStage::project(tokenizer),
            parser: CssParserStage::project(parser),
            selector: CssSelectorStage::project(run),
            observations: run
                .observations()
                .iter()
                .map(CssSelectorObservation::project)
                .collect(),
        }
    }

    /// The identity of the analyzed source.
    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// The selected selector grammar profile.
    pub fn profile(&self) -> CssSelectorProfile {
        self.profile
    }

    /// Tokenizer-stage completion and termination.
    pub fn tokenizer(&self) -> &CssTokenizerStage {
        &self.tokenizer
    }

    /// Parser-stage completion, coverage, and termination.
    pub fn parser(&self) -> &CssParserStage {
        &self.parser
    }

    /// Selector-qualification-stage completion and termination.
    pub fn selector(&self) -> &CssSelectorStage {
        &self.selector
    }

    /// One observation per processed retained qualified-rule context, in
    /// retained parser-context order.
    pub fn observations(&self) -> &[CssSelectorObservation] {
        &self.observations
    }
}

/// The selector grammar profile used for qualification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssSelectorProfile {
    /// The bounded browser-independent `CoreV1` profile. It is not a claim of
    /// complete standards or user-agent validity.
    CoreV1,
}

impl CssSelectorProfile {
    fn project(profile: selector_profile::CssSelectorGrammarProfile) -> Self {
        match profile {
            selector_profile::CssSelectorGrammarProfile::CoreV1 => Self::CoreV1,
        }
    }
}

/// Whether one analysis stage processed all of its own input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssStageCompletion {
    Complete,
    Incomplete,
}

/// Tokenizer-stage state.
#[derive(Debug, Clone)]
pub struct CssTokenizerStage {
    completion: CssStageCompletion,
    termination: CssTokenizerTermination,
    diagnostics: usize,
}

impl CssTokenizerStage {
    fn project(result: &tokenizer_result::CssTokenizerRunResult) -> Self {
        Self {
            completion: match result.completion() {
                tokenizer_result::CssTokenizerCompletion::Complete => CssStageCompletion::Complete,
                tokenizer_result::CssTokenizerCompletion::Incomplete => {
                    CssStageCompletion::Incomplete
                }
            },
            termination: match result.termination() {
                tokenizer_result::CssTokenizerTermination::EndOfInput => {
                    CssTokenizerTermination::EndOfInput
                }
                tokenizer_result::CssTokenizerTermination::ResourceLimit(evidence) => {
                    CssTokenizerTermination::ResourceLimit(CssResourceRefusal::tokenizer(evidence))
                }
            },
            diagnostics: result.diagnostics().len(),
        }
    }

    pub fn completion(&self) -> CssStageCompletion {
        self.completion
    }

    pub fn termination(&self) -> &CssTokenizerTermination {
        &self.termination
    }

    /// Number of retained tokenizer diagnostics.
    pub fn diagnostics(&self) -> usize {
        self.diagnostics
    }
}

/// Why the tokenizer stage stopped.
#[derive(Debug, Clone)]
pub enum CssTokenizerTermination {
    EndOfInput,
    ResourceLimit(CssResourceRefusal),
}

/// Parser-stage state.
#[derive(Debug, Clone)]
pub struct CssParserStage {
    completion: CssStageCompletion,
    coverage: CssParserCoverage,
    termination: CssParserTermination,
    diagnostics: usize,
    recovery_records: usize,
    unsupported_regions: usize,
    discard_records: usize,
}

impl CssParserStage {
    fn project(result: &parser_result::CssParserRunResult) -> Self {
        Self {
            completion: match result.execution_completion() {
                parser_result::CssParserExecutionCompletion::Complete => {
                    CssStageCompletion::Complete
                }
                parser_result::CssParserExecutionCompletion::Incomplete => {
                    CssStageCompletion::Incomplete
                }
            },
            coverage: match result.coverage() {
                parser_result::CssParserCoverage::SupportedForSelectedQuestion => {
                    CssParserCoverage::SupportedForSelectedQuestion
                }
                parser_result::CssParserCoverage::ContainsUnsupportedContexts => {
                    CssParserCoverage::ContainsUnsupportedContexts
                }
            },
            termination: match result.termination() {
                parser_result::CssParserTermination::EndOfTokenizerInput => {
                    CssParserTermination::EndOfTokenizerInput
                }
                parser_result::CssParserTermination::UpstreamTokenizerIncomplete => {
                    CssParserTermination::UpstreamTokenizerIncomplete
                }
                parser_result::CssParserTermination::ParserResourceLimit(evidence) => {
                    CssParserTermination::ResourceLimit(CssResourceRefusal::parser(evidence))
                }
            },
            diagnostics: result.parser_diagnostics().len(),
            recovery_records: result.recovery_records().len(),
            unsupported_regions: result.unsupported_regions().len(),
            discard_records: result.discard_records().len(),
        }
    }

    pub fn completion(&self) -> CssStageCompletion {
        self.completion
    }

    pub fn coverage(&self) -> CssParserCoverage {
        self.coverage
    }

    pub fn termination(&self) -> &CssParserTermination {
        &self.termination
    }

    /// Number of retained parser diagnostics.
    pub fn diagnostics(&self) -> usize {
        self.diagnostics
    }

    /// Number of retained parser recovery records.
    pub fn recovery_records(&self) -> usize {
        self.recovery_records
    }

    /// Number of retained unsupported regions.
    pub fn unsupported_regions(&self) -> usize {
        self.unsupported_regions
    }

    /// Number of retained discard records.
    pub fn discard_records(&self) -> usize {
        self.discard_records
    }
}

/// Parser coverage of the selected structural question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssParserCoverage {
    SupportedForSelectedQuestion,
    ContainsUnsupportedContexts,
}

/// Why the parser stage stopped.
#[derive(Debug, Clone)]
pub enum CssParserTermination {
    EndOfTokenizerInput,
    UpstreamTokenizerIncomplete,
    ResourceLimit(CssResourceRefusal),
}

/// Selector-qualification-stage state.
#[derive(Debug, Clone)]
pub struct CssSelectorStage {
    completion: CssStageCompletion,
    termination: CssSelectorTermination,
}

impl CssSelectorStage {
    fn project(run: &selector_result::CssSelectorQualificationRunResult) -> Self {
        Self {
            completion: match run.execution_completion() {
                selector_result::CssSelectorExecutionCompletion::Complete => {
                    CssStageCompletion::Complete
                }
                selector_result::CssSelectorExecutionCompletion::Incomplete => {
                    CssStageCompletion::Incomplete
                }
            },
            termination: match run.termination() {
                selector_result::CssSelectorTermination::AllRetainedQualifiedContextsProcessed => {
                    CssSelectorTermination::AllRetainedQualifiedContextsProcessed
                }
                selector_result::CssSelectorTermination::ResourceLimit(evidence) => {
                    CssSelectorTermination::ResourceLimit(CssResourceRefusal::selector(evidence))
                }
            },
        }
    }

    pub fn completion(&self) -> CssStageCompletion {
        self.completion
    }

    pub fn termination(&self) -> &CssSelectorTermination {
        &self.termination
    }
}

/// Why the selector-qualification stage stopped.
#[derive(Debug, Clone)]
pub enum CssSelectorTermination {
    AllRetainedQualifiedContextsProcessed,
    ResourceLimit(CssResourceRefusal),
}

/// The analysis stage that owns a resource dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssAnalysisStage {
    Tokenizer,
    Parser,
    Selector,
}

/// A Core resource dimension, grouped by owning stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssResourceKind {
    Tokenizer(CssTokenizerResourceKind),
    Parser(CssParserResourceKind),
    Selector(CssSelectorResourceKind),
}

impl CssResourceKind {
    /// The stage that owns this dimension.
    pub fn stage(self) -> CssAnalysisStage {
        match self {
            Self::Tokenizer(_) => CssAnalysisStage::Tokenizer,
            Self::Parser(_) => CssAnalysisStage::Parser,
            Self::Selector(_) => CssAnalysisStage::Selector,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssTokenizerResourceKind {
    SourceBytes,
    AlgorithmSteps,
    LexicalItems,
    Diagnostics,
    RetainedInterpretedBytes,
    TemporaryBufferBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssParserResourceKind {
    AlgorithmSteps,
    PeakComponentDepth,
    PeakContextDepth,
    DeclarationOccurrences,
    ParserDiagnostics,
    RecoveryRecords,
    UnsupportedRegions,
    DiscardRecords,
    ContextRecords,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssSelectorResourceKind {
    AlgorithmSteps,
    PeakSelectorDepth,
    Observations,
}

/// Retained Core resource-refusal evidence: the refused dimension, its
/// configured limit, the attempted value that exceeded it, and the retained
/// source point where the stage stopped.
#[derive(Debug, Clone)]
pub struct CssResourceRefusal {
    kind: CssResourceKind,
    limit: usize,
    attempted: usize,
    location: SourceAnchor,
}

impl CssResourceRefusal {
    fn tokenizer(evidence: &tokenizer_resource::CssTokenizerResourceLimitEvidence) -> Self {
        use tokenizer_resource::CssTokenizerResourceKind as Internal;
        let kind = match evidence.kind() {
            Internal::SourceBytes => CssTokenizerResourceKind::SourceBytes,
            Internal::AlgorithmSteps => CssTokenizerResourceKind::AlgorithmSteps,
            Internal::LexicalItems => CssTokenizerResourceKind::LexicalItems,
            Internal::Diagnostics => CssTokenizerResourceKind::Diagnostics,
            Internal::RetainedInterpretedBytes => {
                CssTokenizerResourceKind::RetainedInterpretedBytes
            }
            Internal::TemporaryBufferBytes => CssTokenizerResourceKind::TemporaryBufferBytes,
        };
        Self {
            kind: CssResourceKind::Tokenizer(kind),
            limit: evidence.limit(),
            attempted: evidence.attempted(),
            location: evidence.location().clone(),
        }
    }

    fn parser(evidence: &parser_resource::CssParserResourceLimitEvidence) -> Self {
        use parser_resource::CssParserResourceKind as Internal;
        let kind = match evidence.kind() {
            Internal::AlgorithmSteps => CssParserResourceKind::AlgorithmSteps,
            Internal::PeakComponentDepth => CssParserResourceKind::PeakComponentDepth,
            Internal::PeakContextDepth => CssParserResourceKind::PeakContextDepth,
            Internal::DeclarationOccurrences => CssParserResourceKind::DeclarationOccurrences,
            Internal::ParserDiagnostics => CssParserResourceKind::ParserDiagnostics,
            Internal::RecoveryRecords => CssParserResourceKind::RecoveryRecords,
            Internal::UnsupportedRegions => CssParserResourceKind::UnsupportedRegions,
            Internal::DiscardRecords => CssParserResourceKind::DiscardRecords,
            Internal::ContextRecords => CssParserResourceKind::ContextRecords,
        };
        Self {
            kind: CssResourceKind::Parser(kind),
            limit: evidence.limit(),
            attempted: evidence.attempted(),
            location: evidence.location().clone(),
        }
    }

    fn selector(evidence: &selector_resource::CssSelectorResourceLimitEvidence) -> Self {
        use selector_resource::CssSelectorResourceKind as Internal;
        let kind = match evidence.kind() {
            Internal::AlgorithmSteps => CssSelectorResourceKind::AlgorithmSteps,
            Internal::PeakSelectorDepth => CssSelectorResourceKind::PeakSelectorDepth,
            Internal::Observations => CssSelectorResourceKind::Observations,
        };
        Self {
            kind: CssResourceKind::Selector(kind),
            limit: evidence.limit(),
            attempted: evidence.attempted(),
            location: evidence.location().clone(),
        }
    }

    pub fn kind(&self) -> CssResourceKind {
        self.kind
    }

    /// The owning stage of the refused dimension.
    pub fn stage(&self) -> CssAnalysisStage {
        self.kind.stage()
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn attempted(&self) -> usize {
        self.attempted
    }

    /// The retained empty source point at which the stage stopped.
    pub fn location(&self) -> &SourceAnchor {
        &self.location
    }
}

/// One `CoreV1` qualification observation for a retained qualified-rule
/// context.
#[derive(Debug, Clone)]
pub struct CssSelectorObservation {
    context: SourceAnchor,
    grammar_context: CssSelectorGrammarContext,
    outcome: CssSelectorOutcome,
}

impl CssSelectorObservation {
    fn project(observation: &selector_result::CssSelectorQualificationObservation) -> Self {
        Self {
            context: observation.context_header().clone(),
            grammar_context: CssSelectorGrammarContext::project(observation.grammar_context()),
            outcome: CssSelectorOutcome::project(observation.outcome()),
        }
    }

    /// The retained authored context header (the selector prelude) that was
    /// qualified.
    pub fn context(&self) -> &SourceAnchor {
        &self.context
    }

    pub fn grammar_context(&self) -> CssSelectorGrammarContext {
        self.grammar_context
    }

    pub fn outcome(&self) -> &CssSelectorOutcome {
        &self.outcome
    }
}

/// The selector grammar entry point derived from retained context ancestry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssSelectorGrammarContext {
    NormalSelectorList,
    NestedRelativeSelectorList,
    ScopedRelativeSelectorList,
}

impl CssSelectorGrammarContext {
    fn project(context: selector_context::CssSelectorGrammarContext) -> Self {
        match context {
            selector_context::CssSelectorGrammarContext::NormalSelectorList => {
                Self::NormalSelectorList
            }
            selector_context::CssSelectorGrammarContext::NestedRelativeSelectorList { .. } => {
                Self::NestedRelativeSelectorList
            }
            selector_context::CssSelectorGrammarContext::ScopedRelativeSelectorList { .. } => {
                Self::ScopedRelativeSelectorList
            }
        }
    }
}

/// The `CoreV1` qualification outcome for one retained context.
#[derive(Debug, Clone)]
pub enum CssSelectorOutcome {
    /// Qualified by the bounded `CoreV1` profile only.
    QualifiedBySelectedGrammar,
    InvalidForSelectedGrammar {
        reason: CssSelectorInvalidReason,
        subject: SourceAnchor,
    },
    /// Outside `CoreV1` coverage; not a claim of invalid CSS.
    UnsupportedBySelectedGrammarProfile {
        feature: CssSelectorUnsupportedFeature,
        subject: SourceAnchor,
    },
    Indeterminate {
        reason: CssSelectorIndeterminateReason,
        subject: Option<SourceAnchor>,
    },
}

impl CssSelectorOutcome {
    fn project(outcome: &selector_result::CssSelectorQualificationOutcome) -> Self {
        use selector_result::CssSelectorQualificationOutcome as Internal;
        match outcome {
            Internal::QualifiedBySelectedGrammar => Self::QualifiedBySelectedGrammar,
            Internal::InvalidForSelectedGrammar { reason, subject } => {
                Self::InvalidForSelectedGrammar {
                    reason: CssSelectorInvalidReason::project(*reason),
                    subject: subject.clone(),
                }
            }
            Internal::UnsupportedBySelectedGrammarProfile { feature, subject } => {
                Self::UnsupportedBySelectedGrammarProfile {
                    feature: CssSelectorUnsupportedFeature::project(*feature),
                    subject: subject.clone(),
                }
            }
            Internal::Indeterminate { reason, subject } => Self::Indeterminate {
                reason: CssSelectorIndeterminateReason::project(*reason),
                subject: subject.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssSelectorInvalidReason {
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

impl CssSelectorInvalidReason {
    fn project(reason: selector_result::CssSelectorInvalidReason) -> Self {
        use selector_result::CssSelectorInvalidReason as Internal;
        match reason {
            Internal::EmptySelectorList => Self::EmptySelectorList,
            Internal::UnexpectedComma => Self::UnexpectedComma,
            Internal::UnexpectedCombinator => Self::UnexpectedCombinator,
            Internal::InvalidCompoundOrder => Self::InvalidCompoundOrder,
            Internal::InvalidAttributeSelector => Self::InvalidAttributeSelector,
            Internal::InvalidPseudoSyntax => Self::InvalidPseudoSyntax,
            Internal::InvalidNestingSelectorPlacement => Self::InvalidNestingSelectorPlacement,
            Internal::InvalidFunctionalPseudoArgument => Self::InvalidFunctionalPseudoArgument,
            Internal::NestedHasNotAllowed => Self::NestedHasNotAllowed,
            Internal::UnexpectedToken => Self::UnexpectedToken,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssSelectorUnsupportedFeature {
    IdentifierPseudoClass,
    FunctionalPseudoClass,
    PseudoElement,
    FunctionalPseudoElement,
    OtherSelectorFeature,
}

impl CssSelectorUnsupportedFeature {
    fn project(feature: selector_result::CssSelectorUnsupportedFeature) -> Self {
        use selector_result::CssSelectorUnsupportedFeature as Internal;
        match feature {
            Internal::IdentifierPseudoClass => Self::IdentifierPseudoClass,
            Internal::FunctionalPseudoClass => Self::FunctionalPseudoClass,
            Internal::PseudoElement => Self::PseudoElement,
            Internal::FunctionalPseudoElement => Self::FunctionalPseudoElement,
            Internal::OtherSelectorFeature => Self::OtherSelectorFeature,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssSelectorIndeterminateReason {
    MissingNamespaceEnvironment,
    UnavailableStructuralContext,
}

impl CssSelectorIndeterminateReason {
    fn project(reason: selector_result::CssSelectorIndeterminateReason) -> Self {
        use selector_result::CssSelectorIndeterminateReason as Internal;
        match reason {
            Internal::MissingNamespaceEnvironment => Self::MissingNamespaceEnvironment,
            Internal::UnavailableStructuralContext => Self::UnavailableStructuralContext,
        }
    }
}

/// A returned Core boundary or internal contract failure.
///
/// This is never an ordinary analysis outcome: invalid, unsupported,
/// indeterminate, incomplete, and resource-refused results are reported
/// through [`CssSelectorReport`]. The underlying internal failure is kept
/// private; `Display` names only the failing Core boundary and never includes
/// authored source content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssSelectorCoreFailure {
    repr: FailureRepr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FailureRepr {
    ExecutionPolicy,
    Analysis(CssSelectorAnalysisError),
}

impl CssSelectorCoreFailure {
    fn execution_policy() -> Self {
        Self {
            repr: FailureRepr::ExecutionPolicy,
        }
    }

    fn analysis(error: CssSelectorAnalysisError) -> Self {
        Self {
            repr: FailureRepr::Analysis(error),
        }
    }

    fn boundary(&self) -> &'static str {
        match &self.repr {
            FailureRepr::ExecutionPolicy => "execution policy",
            FailureRepr::Analysis(CssSelectorAnalysisError::Structural(
                CssAnalysisError::TokenizerRun(_),
            )) => "tokenizer",
            FailureRepr::Analysis(CssSelectorAnalysisError::Structural(
                CssAnalysisError::ParserRun(_),
            )) => "parser",
            FailureRepr::Analysis(CssSelectorAnalysisError::Structural(_)) => {
                "source-evidence reconciliation"
            }
            FailureRepr::Analysis(CssSelectorAnalysisError::Selector(_)) => {
                "selector qualification"
            }
        }
    }
}

impl fmt::Display for CssSelectorCoreFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS CoreV1 selector analysis returned a Core failure at the {} boundary",
            self.boundary()
        )
    }
}

impl Error for CssSelectorCoreFailure {}
