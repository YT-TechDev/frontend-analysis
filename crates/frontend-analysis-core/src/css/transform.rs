//! Public CSS selected direct-authored `transform` qualification consumer
//! facade (#914).
//!
//! This is a capability-specific public boundary over the existing
//! crate-private CSS pipeline:
//!
//! ```text
//! &SourceText
//!   -> existing CSS tokenizer
//!   -> existing CSS parser
//!   -> existing Core source-evidence reconciliation
//!   -> existing authored-value qualification run
//!   -> existing selected `transform` observations
//!   -> owned CssTransformReport
//! ```
//!
//! The report answers one question: for every ordinary declaration retained
//! by the current parser and recognized by the current value dispatcher as the
//! property `transform`, what is the selected direct-authored transform value
//! outcome, and where are its authored declaration, property, value, optional
//! priority, and owning structural context evidence?
//!
//! It is an observation of retained ordinary declarations, **not a
//! source-wide transform detector**. A report with zero observations means
//! only that this run produced no selected transform observations among the
//! ordinary declarations it retained. It never establishes that the source
//! contains no authored `transform` text: unsupported regions, recovery and
//! discard regions, non-ordinary declaration categories (descriptors,
//! `@page`, page margins, keyframes), and incomplete or resource-limited
//! analysis can all hold such text without producing an observation. Consult
//! [`CssTransformReport::tokenizer`] and [`CssTransformReport::parser`] for
//! completion, coverage, and source-locatable partial-coverage evidence.
//!
//! The report claims nothing about selector validity or matching, cascade
//! winners, declaration applicability, computed or used values, CSSOM state,
//! layout, rendering, animation, transform geometry, or browser behavior.
//! Selector qualification is not invoked by this capability.
//!
//! The facade applies a fixed private execution policy (Issue #914); those
//! values are execution policy, not CSS semantics, and are not
//! caller-configurable. Every report value is projected from evidence already
//! retained by the internal run. No source text is searched, rescanned,
//! retokenized, reparsed, or otherwise reconstructed here, and no transform
//! argument payload is exposed.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

use super::analysis::{CssAnalysisError, analyze_css_source};
use super::parser::resource as parser_resource;
use super::selectors::{CssParserStage, CssTokenizerStage};
use super::tokenizer::resource as tokenizer_resource;
use super::value_qualification as value;

#[cfg(test)]
mod tests;

// Issue #914 execution envelope. Execution policy only; not CSS semantics or
// public API. Deliberately selector-free: no selector budget exists here.
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

/// Runs the existing CSS tokenizer, parser, Core source-evidence
/// reconciliation, and authored-value qualification over `source` under the
/// fixed execution policy, and projects the selected `transform`
/// observations.
///
/// Ordinary analysis outcomes, including invalid, unsupported, incomplete,
/// and resource-refused results, are returned as `Ok`. `Err` is reserved for a
/// returned Core boundary or internal contract failure.
///
/// The returned report owns its source evidence and remains usable after the
/// caller drops `source`.
pub fn analyze_authored_transforms(
    source: &SourceText,
) -> Result<CssTransformReport, CssTransformCoreFailure> {
    let tokenizer_limits = tokenizer_resource::CssTokenizerLimits::new(
        TOKENIZER_SOURCE_BYTES,
        TOKENIZER_ALGORITHM_STEPS,
        TOKENIZER_LEXICAL_ITEMS,
        TOKENIZER_DIAGNOSTICS,
        TOKENIZER_RETAINED_INTERPRETED_BYTES,
        TOKENIZER_TEMPORARY_BUFFER_BYTES,
    )
    .map_err(|_| CssTransformCoreFailure::execution_policy())?;
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
    .map_err(|_| CssTransformCoreFailure::execution_policy())?;

    analyze_with_limits(source, tokenizer_limits, parser_limits)
}

/// The single execution path. Split from the public entry only so tests can
/// force upstream resource refusal through the existing private limit seams.
fn analyze_with_limits(
    source: &SourceText,
    tokenizer_limits: tokenizer_resource::CssTokenizerLimits,
    parser_limits: parser_resource::CssParserLimits,
) -> Result<CssTransformReport, CssTransformCoreFailure> {
    let parser_result = analyze_css_source(source, tokenizer_limits, parser_limits)
        .map_err(CssTransformCoreFailure::analysis)?;
    let run = value::run(parser_result).map_err(CssTransformCoreFailure::value_qualification)?;
    CssTransformReport::project(&run)
}

/// The owned result of one [`analyze_authored_transforms`] invocation.
///
/// Stage completion is reported per stage and is exactly the upstream
/// tokenizer/parser meaning; the value-qualification step adds no completion
/// or termination state of its own. A `Complete` parser stage means the parser
/// consumed its tokenizer input, not that the whole source was analyzed or is
/// well-formed, and zero [`observations`](Self::observations) never establish
/// whole-source transform absence (see the module documentation).
///
/// The stage types are the existing tokenizer/parser stage vocabulary of
/// [`crate::css::selectors`]. Their resource-refusal evidence can only name
/// tokenizer or parser dimensions here: this capability runs no selector
/// stage and so never reports a selector-owned dimension.
#[derive(Debug, Clone)]
pub struct CssTransformReport {
    source_id: SourceId,
    profile: CssTransformProfile,
    tokenizer: CssTokenizerStage,
    parser: CssParserStage,
    observations: Vec<CssTransformObservation>,
}

impl CssTransformReport {
    fn project(
        run: &value::CssValueQualificationRunResult,
    ) -> Result<Self, CssTransformCoreFailure> {
        let parser = run.upstream_parser_result();
        let tokenizer = parser.upstream_tokenizer_result();
        let occurrences = parser.occurrences();
        let contexts = parser.context_records();

        let mut observations = Vec::with_capacity(run.transform_observations().len());
        for observation in run.transform_observations() {
            let occurrence = occurrences
                .get(observation.occurrence_index())
                .ok_or_else(|| {
                    CssTransformCoreFailure::projection(Projection::OccurrenceMissing)
                })?;
            let placement = observation.placement();
            if occurrence.placement() != placement {
                return Err(CssTransformCoreFailure::projection(
                    Projection::PlacementMismatch,
                ));
            }
            let context = contexts
                .get(placement.context_id().index())
                .filter(|record| record.id() == placement.context_id())
                .ok_or_else(|| CssTransformCoreFailure::projection(Projection::ContextMissing))?;

            observations.push(CssTransformObservation {
                declaration: occurrence.complete().clone(),
                property: occurrence.property_name().clone(),
                value: occurrence.value().clone(),
                priority: occurrence
                    .priority()
                    .map(|priority| priority.complete().clone()),
                context: context.header().clone(),
                outcome: CssTransformOutcome::project(observation.outcome()),
            });
        }

        Ok(Self {
            source_id: tokenizer.source_id(),
            profile: CssTransformProfile::SelectedDirectAuthoredTransform,
            tokenizer: CssTokenizerStage::project(tokenizer),
            parser: CssParserStage::project(parser),
            observations,
        })
    }

    /// The identity of the analyzed source, exactly as supplied by the caller.
    pub fn source_id(&self) -> SourceId {
        self.source_id
    }

    /// The fixed capability meaning of this report.
    pub fn profile(&self) -> CssTransformProfile {
        self.profile
    }

    /// Tokenizer-stage completion and termination.
    pub fn tokenizer(&self) -> &CssTokenizerStage {
        &self.tokenizer
    }

    /// Parser-stage completion, coverage, termination, and source-locatable
    /// recovery, unsupported, and discard evidence.
    pub fn parser(&self) -> &CssParserStage {
        &self.parser
    }

    /// One observation per retained ordinary declaration recognized as the
    /// property `transform`, in retained source order. Duplicates are kept;
    /// nothing is deduplicated, cascaded, or filtered by selector.
    ///
    /// An empty slice does not establish that the source contains no authored
    /// transform.
    pub fn observations(&self) -> &[CssTransformObservation] {
        &self.observations
    }
}

/// The fixed meaning of a [`CssTransformReport`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssTransformProfile {
    /// Selected direct-authored `transform` qualification over retained
    /// ordinary declarations. It is not a claim of complete `transform`
    /// grammar support, complete standards or user-agent validity, or
    /// browser applicability.
    SelectedDirectAuthoredTransform,
}

/// One selected `transform` observation for a retained ordinary declaration.
///
/// All anchors are retained authored evidence: they carry the authored
/// spelling (for example an escaped or case-varied property name) and are
/// never normalized or reconstructed. Declaration identity within a report is
/// the declaration anchor together with report order.
#[derive(Debug, Clone)]
pub struct CssTransformObservation {
    declaration: SourceAnchor,
    property: SourceAnchor,
    value: SourceAnchor,
    priority: Option<SourceAnchor>,
    context: SourceAnchor,
    outcome: CssTransformOutcome,
}

impl CssTransformObservation {
    /// The retained complete authored declaration, including its authored
    /// terminator where the parser retained one.
    pub fn declaration(&self) -> &SourceAnchor {
        &self.declaration
    }

    /// The authored property-name spelling. Its selection as `transform` is
    /// owned by Core and may differ from this spelling (case or escapes).
    pub fn property(&self) -> &SourceAnchor {
        &self.property
    }

    /// The retained authored value extent. For an `Invalid` or `Unsupported`
    /// outcome this whole value is the subject evidence; no culprit-only
    /// range exists. It may be empty.
    pub fn value(&self) -> &SourceAnchor {
        &self.value
    }

    /// The whole authored `!important` priority anchor, when retained. It is
    /// separate from, and never part of, [`value`](Self::value).
    pub fn priority(&self) -> Option<&SourceAnchor> {
        self.priority.as_ref()
    }

    /// The retained authored header of the directly owning structural
    /// context. This proves structural location only: it is not selector
    /// qualification, matching, or applicability.
    pub fn context(&self) -> &SourceAnchor {
        &self.context
    }

    pub fn outcome(&self) -> CssTransformOutcome {
        self.outcome
    }
}

/// The selected direct-authored `transform` outcome for one declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssTransformOutcome {
    /// Qualified by the selected direct-authored profile only. The private
    /// qualified value is not exposed. Not a claim of complete `transform`
    /// grammar validity.
    Qualified,
    /// Invalid for the selected value grammar.
    InvalidForSelectedValueGrammar,
    /// Outside the selected profile; not a claim of invalid CSS.
    UnsupportedBySelectedValueProfile(CssTransformUnsupportedReason),
}

impl CssTransformOutcome {
    fn project(outcome: &value::CssTransformQualificationOutcome) -> Self {
        use value::CssTransformQualificationOutcome as Internal;
        match outcome {
            Internal::Qualified(_) => Self::Qualified,
            Internal::InvalidForSelectedValueGrammar => Self::InvalidForSelectedValueGrammar,
            Internal::UnsupportedBySelectedValueProfile(reason) => {
                Self::UnsupportedBySelectedValueProfile(CssTransformUnsupportedReason::project(
                    *reason,
                ))
            }
        }
    }
}

/// Why a `transform` value is outside the selected profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssTransformUnsupportedReason {
    /// The whole value is a CSS-wide keyword.
    CssWideKeyword,
    /// The value contains a deferred substitution function.
    DeferredSubstitutionFunction,
    /// The whole value is a whole-value-only function.
    WholeValueFunction,
    /// The value contains a transform function outside the selected set.
    UnselectedTransformFunction,
    /// A selected transform function has a function-valued argument.
    FunctionValuedTransformArgument,
}

impl CssTransformUnsupportedReason {
    fn project(reason: value::CssTransformUnsupportedReason) -> Self {
        use value::CssTransformUnsupportedReason as Internal;
        match reason {
            Internal::CssWideKeyword => Self::CssWideKeyword,
            Internal::DeferredSubstitutionFunction => Self::DeferredSubstitutionFunction,
            Internal::WholeValueFunction => Self::WholeValueFunction,
            Internal::UnselectedTransformFunction => Self::UnselectedTransformFunction,
            Internal::FunctionValuedTransformArgument => Self::FunctionValuedTransformArgument,
        }
    }
}

/// A returned Core boundary or internal contract failure.
///
/// This is never an ordinary analysis outcome: invalid, unsupported,
/// incomplete, and resource-refused results are reported through
/// [`CssTransformReport`]. The underlying internal failure is kept private;
/// `Display` names only the failing Core boundary and never includes authored
/// source content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssTransformCoreFailure {
    repr: FailureRepr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FailureRepr {
    ExecutionPolicy,
    Analysis(CssAnalysisError),
    ValueQualification(value::CssValueQualificationError),
    Projection(Projection),
}

/// Required owned evidence that was missing or inconsistent while projecting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Projection {
    OccurrenceMissing,
    PlacementMismatch,
    ContextMissing,
}

impl CssTransformCoreFailure {
    fn execution_policy() -> Self {
        Self {
            repr: FailureRepr::ExecutionPolicy,
        }
    }

    fn analysis(error: CssAnalysisError) -> Self {
        Self {
            repr: FailureRepr::Analysis(error),
        }
    }

    fn value_qualification(error: value::CssValueQualificationError) -> Self {
        Self {
            repr: FailureRepr::ValueQualification(error),
        }
    }

    fn projection(violation: Projection) -> Self {
        Self {
            repr: FailureRepr::Projection(violation),
        }
    }

    fn boundary(&self) -> &'static str {
        match &self.repr {
            FailureRepr::ExecutionPolicy => "execution policy",
            FailureRepr::Analysis(CssAnalysisError::TokenizerRun(_)) => "tokenizer",
            FailureRepr::Analysis(CssAnalysisError::ParserRun(_)) => "parser",
            FailureRepr::Analysis(_) => "source-evidence reconciliation",
            FailureRepr::ValueQualification(_) => "authored-value qualification",
            FailureRepr::Projection(_) => "transform report projection",
        }
    }
}

impl fmt::Display for CssTransformCoreFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS transform analysis returned a Core failure at the {} boundary",
            self.boundary()
        )
    }
}

impl Error for CssTransformCoreFailure {}
