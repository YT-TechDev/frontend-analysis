use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRange, SourceRangeError, SourceText};

use super::super::declaration::{
    CssDeclarationContractError, CssDeclarationOccurrence, CssDeclarationTermination,
};
use super::super::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerRunResult, CssTokenizerTermination,
};
use super::diagnostic::{CssParserDiagnostic, CssParserDiagnosticContractError};
use super::evidence::{
    CssParserDiscardEvidence, CssParserEvidenceContractError, CssParserRecoveryEvidence,
    CssParserRecoveryTermination, CssParserUnsupportedRegion,
};
use super::resource::{
    CssParserInvalidConfiguration, CssParserResourceContractError, CssParserResourceKind,
    CssParserResourceLimitEvidence, CssParserResourceUsage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserExecutionCompletion {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserCoverage {
    SupportedForSelectedQuestion,
    ContainsUnsupportedContexts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssParserTermination {
    EndOfTokenizerInput,
    UpstreamTokenizerIncomplete,
    ParserResourceLimit(CssParserResourceLimitEvidence),
}

/// The #138 crate-private parser-stage run result.
///
/// Owns the complete upstream [`CssTokenizerRunResult`] unchanged; tokenizer
/// diagnostics and resources are never translated or copied into parser
/// equivalents.
#[derive(Debug, Clone)]
pub(crate) struct CssParserRunResult {
    upstream_tokenizer_result: CssTokenizerRunResult,
    occurrences: Vec<CssDeclarationOccurrence>,
    parser_diagnostics: Vec<CssParserDiagnostic>,
    recovery_records: Vec<CssParserRecoveryEvidence>,
    unsupported_regions: Vec<CssParserUnsupportedRegion>,
    discard_records: Vec<CssParserDiscardEvidence>,
    terminal: SourceAnchor,
    execution_completion: CssParserExecutionCompletion,
    coverage: CssParserCoverage,
    termination: CssParserTermination,
    resources: CssParserResourceUsage,
}

impl CssParserRunResult {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        upstream_tokenizer_result: CssTokenizerRunResult,
        occurrences: Vec<CssDeclarationOccurrence>,
        parser_diagnostics: Vec<CssParserDiagnostic>,
        recovery_records: Vec<CssParserRecoveryEvidence>,
        unsupported_regions: Vec<CssParserUnsupportedRegion>,
        discard_records: Vec<CssParserDiscardEvidence>,
        terminal: SourceAnchor,
        execution_completion: CssParserExecutionCompletion,
        coverage: CssParserCoverage,
        termination: CssParserTermination,
        resources: CssParserResourceUsage,
    ) -> Result<Self, CssParserRunError> {
        validate_run(
            source_text,
            &upstream_tokenizer_result,
            &occurrences,
            &parser_diagnostics,
            &recovery_records,
            &unsupported_regions,
            &discard_records,
            &terminal,
            execution_completion,
            coverage,
            &termination,
            resources,
        )?;

        Ok(Self {
            upstream_tokenizer_result,
            occurrences,
            parser_diagnostics,
            recovery_records,
            unsupported_regions,
            discard_records,
            terminal,
            execution_completion,
            coverage,
            termination,
            resources,
        })
    }

    pub(crate) const fn upstream_tokenizer_result(&self) -> &CssTokenizerRunResult {
        &self.upstream_tokenizer_result
    }

    pub(crate) fn occurrences(&self) -> &[CssDeclarationOccurrence] {
        &self.occurrences
    }

    pub(crate) fn parser_diagnostics(&self) -> &[CssParserDiagnostic] {
        &self.parser_diagnostics
    }

    pub(crate) fn recovery_records(&self) -> &[CssParserRecoveryEvidence] {
        &self.recovery_records
    }

    pub(crate) fn unsupported_regions(&self) -> &[CssParserUnsupportedRegion] {
        &self.unsupported_regions
    }

    pub(crate) fn discard_records(&self) -> &[CssParserDiscardEvidence] {
        &self.discard_records
    }

    pub(crate) const fn terminal(&self) -> &SourceAnchor {
        &self.terminal
    }

    pub(crate) const fn execution_completion(&self) -> CssParserExecutionCompletion {
        self.execution_completion
    }

    pub(crate) const fn coverage(&self) -> CssParserCoverage {
        self.coverage
    }

    pub(crate) const fn termination(&self) -> &CssParserTermination {
        &self.termination
    }

    pub(crate) const fn resources(&self) -> CssParserResourceUsage {
        self.resources
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssParserRunError {
    InvalidConfiguration(CssParserInvalidConfiguration),
    InternalInvariantFailure(CssParserInvariantViolation),
}

impl From<CssParserInvalidConfiguration> for CssParserRunError {
    fn from(value: CssParserInvalidConfiguration) -> Self {
        Self::InvalidConfiguration(value)
    }
}

impl From<SourceRangeError> for CssParserRunError {
    fn from(error: SourceRangeError) -> Self {
        Self::InternalInvariantFailure(CssParserInvariantViolation::SourceRangeContractViolation {
            error,
        })
    }
}

impl From<CssDeclarationContractError> for CssParserRunError {
    fn from(error: CssDeclarationContractError) -> Self {
        Self::InternalInvariantFailure(CssParserInvariantViolation::DeclarationContractViolation {
            error,
        })
    }
}

impl From<CssParserResourceContractError> for CssParserRunError {
    fn from(error: CssParserResourceContractError) -> Self {
        Self::InternalInvariantFailure(CssParserInvariantViolation::ResourceContractViolation {
            error,
        })
    }
}

impl From<CssParserDiagnosticContractError> for CssParserRunError {
    fn from(error: CssParserDiagnosticContractError) -> Self {
        Self::InternalInvariantFailure(CssParserInvariantViolation::DiagnosticContractViolation {
            error,
        })
    }
}

impl From<CssParserEvidenceContractError> for CssParserRunError {
    fn from(error: CssParserEvidenceContractError) -> Self {
        Self::InternalInvariantFailure(CssParserInvariantViolation::EvidenceContractViolation {
            error,
        })
    }
}

impl fmt::Display for CssParserRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS parser run failure: {self:?}")
    }
}

impl Error for CssParserRunError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserRunEvidenceRole {
    Terminal,
    Occurrence { index: usize },
    Diagnostic { index: usize },
    Recovery { index: usize },
    Unsupported { index: usize },
    Discard { index: usize },
    ResourceLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssParserInvariantViolation {
    SourceIdentityMismatch {
        role: CssParserRunEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    SourceRangeContractViolation {
        error: SourceRangeError,
    },
    DeclarationContractViolation {
        error: CssDeclarationContractError,
    },
    ResourceContractViolation {
        error: CssParserResourceContractError,
    },
    DiagnosticContractViolation {
        error: CssParserDiagnosticContractError,
    },
    EvidenceContractViolation {
        error: CssParserEvidenceContractError,
    },
    UpstreamSourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    UpstreamProcessedPrefixMismatch,
    UpstreamUnprocessedRemainderMismatch,
    UpstreamFragmentMismatch,
    TerminalMustBeEmpty {
        start: usize,
        end: usize,
    },
    TerminalBeyondUpstreamTerminal {
        terminal: usize,
        upstream_terminal: usize,
    },
    EndOfTokenizerInputRequiresUpstreamComplete,
    EndOfTokenizerInputTerminalMismatch,
    UpstreamTokenizerIncompleteRequiresUpstreamIncomplete,
    UpstreamTokenizerIncompleteTerminalMismatch,
    ParserResourceLimitTerminalMismatch,
    ExecutionCompletionTerminationMismatch,
    CoverageUnsupportedRegionsMismatch,
    OccurrenceBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    OccurrenceOrderViolation {
        index: usize,
    },
    OccurrenceOverlapsUnsupportedRegion {
        index: usize,
        unsupported_index: usize,
    },
    OmittedAtEndOfInputRequiresUpstreamComplete {
        index: usize,
    },
    RecoveryEndOfInputRequiresUpstreamComplete {
        index: usize,
    },
    DiagnosticBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    DiagnosticOrderViolation {
        index: usize,
    },
    RecoveryBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    RecoveryOrderViolation {
        index: usize,
    },
    UnsupportedBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    UnsupportedOrderViolation {
        index: usize,
    },
    DiscardBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    DiscardOrderViolation {
        index: usize,
    },
    DiscardOverlapsOccurrence {
        index: usize,
        occurrence_index: usize,
    },
    DiscardOverlapsRecoveryRegion {
        index: usize,
        recovery_index: usize,
    },
    DiscardOverlapsUnsupportedRegion {
        index: usize,
        unsupported_index: usize,
    },
    ResourceUsageCountMismatch {
        kind: CssParserResourceKind,
        expected: usize,
        actual: usize,
    },
    ResourceAccountingOverflow {
        kind: CssParserResourceKind,
        current: usize,
        additional: usize,
    },
    /// A second speculative checkpoint was begun while one was already
    /// active (#139 `MAX_ACTIVE_SPECULATIVE_CHECKPOINT_DEPTH == 1`).
    CheckpointAlreadyActive,
    /// Checkpoint commit was requested with no active checkpoint.
    CheckpointCommitWithoutActive,
    /// Checkpoint rollback was requested with no active checkpoint.
    CheckpointRollbackWithoutActive,
    /// A component-frame opener conversion was attempted for an
    /// `ObservedKind` that is not a structural opener.
    ExpectedComponentOpener,
    /// The qualified-rule prelude scan reported a block opener, but the next
    /// lexical item observed was not the expected `LeftCurlyBracket`.
    ExpectedQualifiedRuleBlockOpener,
    /// A bounded declaration-value scan summary (first/window/counts) was
    /// inconsistent with the relationship its caller relied on.
    InconsistentValueScanSummary,
}

/// Computes `current + additional` for one [`CssParserResourceKind`] using
/// checked `usize` arithmetic, mirroring the tokenizer's committed-versus-
/// prospective resource-accounting contract.
pub(crate) fn checked_resource_add(
    kind: CssParserResourceKind,
    current: usize,
    additional: usize,
) -> Result<usize, CssParserRunError> {
    current
        .checked_add(additional)
        .ok_or(CssParserRunError::InternalInvariantFailure(
            CssParserInvariantViolation::ResourceAccountingOverflow {
                kind,
                current,
                additional,
            },
        ))
}

#[allow(clippy::too_many_arguments)]
fn validate_run(
    source_text: &SourceText,
    upstream: &CssTokenizerRunResult,
    occurrences: &[CssDeclarationOccurrence],
    parser_diagnostics: &[CssParserDiagnostic],
    recovery_records: &[CssParserRecoveryEvidence],
    unsupported_regions: &[CssParserUnsupportedRegion],
    discard_records: &[CssParserDiscardEvidence],
    terminal: &SourceAnchor,
    execution_completion: CssParserExecutionCompletion,
    coverage: CssParserCoverage,
    termination: &CssParserTermination,
    resources: CssParserResourceUsage,
) -> Result<(), CssParserRunError> {
    let expected_source = source_text.id();

    validate_upstream_boundary(source_text, upstream)?;
    require_source(
        expected_source,
        terminal,
        CssParserRunEvidenceRole::Terminal,
    )?;
    validate_lifecycle(upstream, terminal, execution_completion, termination)?;

    if let CssParserTermination::ParserResourceLimit(evidence) = termination {
        require_source(
            expected_source,
            evidence.location(),
            CssParserRunEvidenceRole::ResourceLimit,
        )?;
    }

    validate_coverage(coverage, unsupported_regions)?;

    let terminal_offset = terminal.range().start();
    validate_occurrences(
        expected_source,
        occurrences,
        unsupported_regions,
        terminal_offset,
    )?;
    validate_occurrence_lifecycle(upstream, occurrences)?;
    validate_diagnostics(expected_source, parser_diagnostics, terminal_offset)?;
    validate_recovery(expected_source, recovery_records, terminal_offset)?;
    validate_recovery_lifecycle(upstream, recovery_records)?;
    validate_unsupported(expected_source, unsupported_regions, terminal_offset)?;
    validate_discard(
        expected_source,
        discard_records,
        occurrences,
        recovery_records,
        unsupported_regions,
        terminal_offset,
    )?;

    validate_resource_counts(
        resources,
        occurrences.len(),
        parser_diagnostics.len(),
        recovery_records.len(),
        unsupported_regions.len(),
        discard_records.len(),
    )?;

    Ok(())
}

fn validate_upstream_boundary(
    source_text: &SourceText,
    upstream: &CssTokenizerRunResult,
) -> Result<(), CssParserRunError> {
    let expected = source_text.id();
    let actual = upstream.source_id();
    if actual != expected {
        return invariant(
            CssParserInvariantViolation::UpstreamSourceIdentityMismatch { expected, actual },
        );
    }

    let terminal_offset = upstream.terminal().range().start();
    let source_len = source_text.as_str().len();

    if upstream.processed_prefix().range().start() != 0
        || upstream.processed_prefix().range().end() != terminal_offset
    {
        return invariant(CssParserInvariantViolation::UpstreamProcessedPrefixMismatch);
    }
    if upstream.unprocessed_remainder().range().start() != terminal_offset
        || upstream.unprocessed_remainder().range().end() != source_len
    {
        return invariant(CssParserInvariantViolation::UpstreamUnprocessedRemainderMismatch);
    }

    let expected_prefix = source_text.anchor(0, terminal_offset)?;
    if expected_prefix.fragment() != upstream.processed_prefix().fragment() {
        return invariant(CssParserInvariantViolation::UpstreamFragmentMismatch);
    }
    let expected_remainder = source_text.anchor(terminal_offset, source_len)?;
    if expected_remainder.fragment() != upstream.unprocessed_remainder().fragment() {
        return invariant(CssParserInvariantViolation::UpstreamFragmentMismatch);
    }

    Ok(())
}

fn validate_lifecycle(
    upstream: &CssTokenizerRunResult,
    terminal: &SourceAnchor,
    execution_completion: CssParserExecutionCompletion,
    termination: &CssParserTermination,
) -> Result<(), CssParserRunError> {
    if !terminal.range().is_empty() {
        return invariant(CssParserInvariantViolation::TerminalMustBeEmpty {
            start: terminal.range().start(),
            end: terminal.range().end(),
        });
    }

    let upstream_terminal_offset = upstream.terminal().range().start();
    if terminal.range().start() > upstream_terminal_offset {
        return invariant(
            CssParserInvariantViolation::TerminalBeyondUpstreamTerminal {
                terminal: terminal.range().start(),
                upstream_terminal: upstream_terminal_offset,
            },
        );
    }

    match termination {
        CssParserTermination::EndOfTokenizerInput => {
            if !upstream_ended_at_true_eof(upstream) {
                return invariant(
                    CssParserInvariantViolation::EndOfTokenizerInputRequiresUpstreamComplete,
                );
            }
            if !same_anchor(terminal, upstream.terminal()) {
                return invariant(CssParserInvariantViolation::EndOfTokenizerInputTerminalMismatch);
            }
            if execution_completion != CssParserExecutionCompletion::Complete {
                return invariant(
                    CssParserInvariantViolation::ExecutionCompletionTerminationMismatch,
                );
            }
        }
        CssParserTermination::UpstreamTokenizerIncomplete => {
            if upstream.completion() != CssTokenizerCompletion::Incomplete {
                return invariant(
                    CssParserInvariantViolation::UpstreamTokenizerIncompleteRequiresUpstreamIncomplete,
                );
            }
            if !same_anchor(terminal, upstream.terminal()) {
                return invariant(
                    CssParserInvariantViolation::UpstreamTokenizerIncompleteTerminalMismatch,
                );
            }
            if execution_completion != CssParserExecutionCompletion::Incomplete {
                return invariant(
                    CssParserInvariantViolation::ExecutionCompletionTerminationMismatch,
                );
            }
        }
        CssParserTermination::ParserResourceLimit(evidence) => {
            if !same_anchor(terminal, evidence.location()) {
                return invariant(CssParserInvariantViolation::ParserResourceLimitTerminalMismatch);
            }
            if execution_completion != CssParserExecutionCompletion::Incomplete {
                return invariant(
                    CssParserInvariantViolation::ExecutionCompletionTerminationMismatch,
                );
            }
        }
    }

    Ok(())
}

/// Whether the upstream tokenizer completed at true `EndOfInput` (as
/// opposed to a resource-limited or otherwise incomplete terminal).
fn upstream_ended_at_true_eof(upstream: &CssTokenizerRunResult) -> bool {
    upstream.completion() == CssTokenizerCompletion::Complete
        && matches!(upstream.termination(), CssTokenizerTermination::EndOfInput)
}

fn validate_coverage(
    coverage: CssParserCoverage,
    unsupported_regions: &[CssParserUnsupportedRegion],
) -> Result<(), CssParserRunError> {
    let has_unsupported = !unsupported_regions.is_empty();
    match coverage {
        CssParserCoverage::SupportedForSelectedQuestion if has_unsupported => {
            invariant(CssParserInvariantViolation::CoverageUnsupportedRegionsMismatch)
        }
        CssParserCoverage::ContainsUnsupportedContexts if !has_unsupported => {
            invariant(CssParserInvariantViolation::CoverageUnsupportedRegionsMismatch)
        }
        _ => Ok(()),
    }
}

fn validate_occurrences(
    expected_source: SourceId,
    occurrences: &[CssDeclarationOccurrence],
    unsupported_regions: &[CssParserUnsupportedRegion],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_end: Option<usize> = None;
    for (index, occurrence) in occurrences.iter().enumerate() {
        let complete = occurrence.complete();
        require_source(
            expected_source,
            complete,
            CssParserRunEvidenceRole::Occurrence { index },
        )?;
        if complete.range().end() > terminal_offset {
            return invariant(CssParserInvariantViolation::OccurrenceBeyondTerminal {
                index,
                end: complete.range().end(),
                terminal: terminal_offset,
            });
        }
        if let Some(previous_end) = previous_end
            && complete.range().start() < previous_end
        {
            return invariant(CssParserInvariantViolation::OccurrenceOrderViolation { index });
        }
        previous_end = Some(complete.range().end());

        for (unsupported_index, region) in unsupported_regions.iter().enumerate() {
            if ranges_overlap(complete.range(), region.region().range()) {
                return invariant(
                    CssParserInvariantViolation::OccurrenceOverlapsUnsupportedRegion {
                        index,
                        unsupported_index,
                    },
                );
            }
        }
    }
    Ok(())
}

/// `OmittedAtEndOfInput` is valid declaration-termination evidence only when
/// the upstream tokenizer itself completed at true `EndOfInput`. An upstream
/// resource-limited or otherwise incomplete terminal must never be
/// relabeled as ordinary omitted-at-EOF declaration termination.
fn validate_occurrence_lifecycle(
    upstream: &CssTokenizerRunResult,
    occurrences: &[CssDeclarationOccurrence],
) -> Result<(), CssParserRunError> {
    if upstream_ended_at_true_eof(upstream) {
        return Ok(());
    }
    for (index, occurrence) in occurrences.iter().enumerate() {
        if matches!(
            occurrence.termination(),
            CssDeclarationTermination::OmittedAtEndOfInput { .. }
        ) {
            return invariant(
                CssParserInvariantViolation::OmittedAtEndOfInputRequiresUpstreamComplete { index },
            );
        }
    }
    Ok(())
}

fn validate_diagnostics(
    expected_source: SourceId,
    diagnostics: &[CssParserDiagnostic],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_key = None;
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        require_source(
            expected_source,
            diagnostic.location(),
            CssParserRunEvidenceRole::Diagnostic { index },
        )?;
        if diagnostic.location().range().end() > terminal_offset {
            return invariant(CssParserInvariantViolation::DiagnosticBeyondTerminal {
                index,
                end: diagnostic.location().range().end(),
                terminal: terminal_offset,
            });
        }
        let key = diagnostic.source_order_key();
        if previous_key.is_some_and(|previous| previous > key) {
            return invariant(CssParserInvariantViolation::DiagnosticOrderViolation { index });
        }
        previous_key = Some(key);
    }
    Ok(())
}

fn validate_recovery(
    expected_source: SourceId,
    recovery_records: &[CssParserRecoveryEvidence],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_key = None;
    for (index, record) in recovery_records.iter().enumerate() {
        require_source(
            expected_source,
            record.region(),
            CssParserRunEvidenceRole::Recovery { index },
        )?;
        if record.region().range().end() > terminal_offset {
            return invariant(CssParserInvariantViolation::RecoveryBeyondTerminal {
                index,
                end: record.region().range().end(),
                terminal: terminal_offset,
            });
        }
        let key = record.source_order_key();
        if previous_key.is_some_and(|previous| previous > key) {
            return invariant(CssParserInvariantViolation::RecoveryOrderViolation { index });
        }
        previous_key = Some(key);
    }
    Ok(())
}

/// `EndOfInput` recovery termination is valid internal evidence only when
/// the upstream tokenizer itself completed at true `EndOfInput`. The
/// standalone recovery constructor cannot prove tokenizer lifecycle, so this
/// is enforced here at the parser-run integration boundary, mirroring
/// [`validate_occurrence_lifecycle`]'s use of the same true-EOF predicate.
fn validate_recovery_lifecycle(
    upstream: &CssTokenizerRunResult,
    recovery_records: &[CssParserRecoveryEvidence],
) -> Result<(), CssParserRunError> {
    if upstream_ended_at_true_eof(upstream) {
        return Ok(());
    }
    for (index, record) in recovery_records.iter().enumerate() {
        if matches!(
            record.termination(),
            CssParserRecoveryTermination::EndOfInput { .. }
        ) {
            return invariant(
                CssParserInvariantViolation::RecoveryEndOfInputRequiresUpstreamComplete { index },
            );
        }
    }
    Ok(())
}

fn validate_unsupported(
    expected_source: SourceId,
    unsupported_regions: &[CssParserUnsupportedRegion],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_key = None;
    for (index, region) in unsupported_regions.iter().enumerate() {
        require_source(
            expected_source,
            region.region(),
            CssParserRunEvidenceRole::Unsupported { index },
        )?;
        if region.region().range().end() > terminal_offset {
            return invariant(CssParserInvariantViolation::UnsupportedBeyondTerminal {
                index,
                end: region.region().range().end(),
                terminal: terminal_offset,
            });
        }
        let key = region.source_order_key();
        if previous_key.is_some_and(|previous| previous > key) {
            return invariant(CssParserInvariantViolation::UnsupportedOrderViolation { index });
        }
        previous_key = Some(key);
    }
    Ok(())
}

/// Discard evidence is durable parser-owned evidence, but it is not
/// unsupported coverage and never assigns a second parser meaning to a
/// construct already claimed by an included occurrence, recovery record, or
/// unsupported region.
fn validate_discard(
    expected_source: SourceId,
    discard_records: &[CssParserDiscardEvidence],
    occurrences: &[CssDeclarationOccurrence],
    recovery_records: &[CssParserRecoveryEvidence],
    unsupported_regions: &[CssParserUnsupportedRegion],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_end: Option<usize> = None;
    for (index, record) in discard_records.iter().enumerate() {
        require_source(
            expected_source,
            record.region(),
            CssParserRunEvidenceRole::Discard { index },
        )?;
        if record.region().range().end() > terminal_offset {
            return invariant(CssParserInvariantViolation::DiscardBeyondTerminal {
                index,
                end: record.region().range().end(),
                terminal: terminal_offset,
            });
        }
        if let Some(previous_end) = previous_end
            && record.region().range().start() < previous_end
        {
            return invariant(CssParserInvariantViolation::DiscardOrderViolation { index });
        }
        previous_end = Some(record.region().range().end());

        for (occurrence_index, occurrence) in occurrences.iter().enumerate() {
            if ranges_overlap(record.region().range(), occurrence.complete().range()) {
                return invariant(CssParserInvariantViolation::DiscardOverlapsOccurrence {
                    index,
                    occurrence_index,
                });
            }
        }
        for (recovery_index, recovery) in recovery_records.iter().enumerate() {
            if ranges_overlap(record.region().range(), recovery.region().range()) {
                return invariant(CssParserInvariantViolation::DiscardOverlapsRecoveryRegion {
                    index,
                    recovery_index,
                });
            }
        }
        for (unsupported_index, region) in unsupported_regions.iter().enumerate() {
            if ranges_overlap(record.region().range(), region.region().range()) {
                return invariant(
                    CssParserInvariantViolation::DiscardOverlapsUnsupportedRegion {
                        index,
                        unsupported_index,
                    },
                );
            }
        }
    }
    Ok(())
}

fn validate_resource_counts(
    resources: CssParserResourceUsage,
    occurrence_count: usize,
    diagnostic_count: usize,
    recovery_count: usize,
    unsupported_count: usize,
    discard_count: usize,
) -> Result<(), CssParserRunError> {
    check_count(
        resources,
        CssParserResourceKind::DeclarationOccurrences,
        occurrence_count,
    )?;
    check_count(
        resources,
        CssParserResourceKind::ParserDiagnostics,
        diagnostic_count,
    )?;
    check_count(
        resources,
        CssParserResourceKind::RecoveryRecords,
        recovery_count,
    )?;
    check_count(
        resources,
        CssParserResourceKind::UnsupportedRegions,
        unsupported_count,
    )?;
    check_count(
        resources,
        CssParserResourceKind::DiscardRecords,
        discard_count,
    )?;
    Ok(())
}

fn check_count(
    resources: CssParserResourceUsage,
    kind: CssParserResourceKind,
    expected: usize,
) -> Result<(), CssParserRunError> {
    let actual = resources.value(kind);
    if actual != expected {
        return invariant(CssParserInvariantViolation::ResourceUsageCountMismatch {
            kind,
            expected,
            actual,
        });
    }
    Ok(())
}

fn ranges_overlap(left: SourceRange, right: SourceRange) -> bool {
    left.start() < right.end() && right.start() < left.end()
}

fn require_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: CssParserRunEvidenceRole,
) -> Result<(), CssParserRunError> {
    let actual = anchor.source_id();
    if actual != expected {
        return invariant(CssParserInvariantViolation::SourceIdentityMismatch {
            role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn invariant<T>(violation: CssParserInvariantViolation) -> Result<T, CssParserRunError> {
    Err(CssParserRunError::InternalInvariantFailure(violation))
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::declaration::{CssDeclarationContext, CssDeclarationTermination};
    use crate::css::parser::evidence::{
        CssParserDiscardKind, CssParserRecoveryKind, CssParserRecoveryTermination,
    };
    use crate::css::token::{CssLexicalItem, CssToken, CssTokenKind};
    use crate::css::tokenizer::diagnostic::CssTokenizerDiagnostic;
    use crate::css::tokenizer::resource::CssTokenizerResourceUsage;

    fn source(id: u64, text: &str) -> SourceText {
        SourceText::new(SourceId::new(id), text.to_owned())
    }

    /// A minimal but lexically-coherent `Complete` tokenizer run: one
    /// synthetic `Ident` lexical item spans the whole source so the
    /// tokenizer's own full-coverage contract is satisfied. It carries no
    /// semantic relationship to the source text; these tests only exercise
    /// `CssParserRunResult`'s own boundary/lifecycle validation against an
    /// upstream result that is itself contractually valid.
    fn complete_tokenizer_run(source: &SourceText) -> CssTokenizerRunResult {
        let len = source.as_str().len();
        let items: Vec<CssLexicalItem> = if len == 0 {
            Vec::new()
        } else {
            vec![CssLexicalItem::SemanticToken(
                CssToken::new(
                    source,
                    source.anchor(0, len).unwrap(),
                    CssTokenKind::Ident("x".to_owned()),
                )
                .unwrap(),
            )]
        };
        let item_count = items.len();
        CssTokenizerRunResult::new(
            source,
            None,
            items,
            Vec::<CssTokenizerDiagnostic>::new(),
            source.anchor(0, len).unwrap(),
            source.anchor(len, len).unwrap(),
            source.anchor(len, len).unwrap(),
            CssTokenizerCompletion::Complete,
            CssTokenizerTermination::EndOfInput,
            CssTokenizerResourceUsage::new(0, 1, item_count, 0, 0, 0),
        )
        .unwrap()
    }

    fn empty_resources() -> CssParserResourceUsage {
        CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0)
    }

    // contract-only: synthetic lifecycle construction to exercise
    // `CssParserRunResult`'s own invariants before #139 exists. Never a
    // source-driven parser recognition oracle.
    #[test]
    fn contract_only_clean_complete_run_is_distinguishable_from_unsupported() {
        let text = source(1, "a{color:red;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            empty_resources(),
        )
        .unwrap();

        assert!(result.occurrences().is_empty());
        assert!(result.unsupported_regions().is_empty());
        assert_eq!(
            result.execution_completion(),
            CssParserExecutionCompletion::Complete
        );
        assert_eq!(
            result.coverage(),
            CssParserCoverage::SupportedForSelectedQuestion
        );
    }

    #[test]
    fn contract_only_upstream_tokenizer_incomplete_cannot_become_parser_complete() {
        let text = source(2, "a{color:red;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::UpstreamTokenizerIncomplete,
            empty_resources(),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::UpstreamTokenizerIncompleteRequiresUpstreamIncomplete
            )
        );
    }

    #[test]
    fn contract_only_end_of_tokenizer_input_requires_upstream_complete() {
        let text = source(3, "a{");
        let upstream = CssTokenizerRunResult::new(
            &text,
            None,
            Vec::<CssLexicalItem>::new(),
            Vec::<CssTokenizerDiagnostic>::new(),
            text.anchor(0, 0).unwrap(),
            text.anchor(0, 2).unwrap(),
            text.anchor(0, 0).unwrap(),
            CssTokenizerCompletion::Incomplete,
            CssTokenizerTermination::ResourceLimit(
                crate::css::tokenizer::resource::CssTokenizerResourceLimitEvidence::new(
                    &text,
                    crate::css::tokenizer::resource::CssTokenizerResourceKind::AlgorithmSteps,
                    1,
                    2,
                    text.anchor(0, 0).unwrap(),
                )
                .unwrap(),
            ),
            CssTokenizerResourceUsage::new(0, 1, 0, 0, 0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(0, 0).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            empty_resources(),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::EndOfTokenizerInputRequiresUpstreamComplete
            )
        );
    }

    #[test]
    fn contract_only_coverage_must_agree_with_unsupported_region_presence() {
        let text = source(4, "@x{}a{color:red;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let unsupported = vec![
            CssParserUnsupportedRegion::new_top_level_at_rule(
                &text,
                text.anchor(0, 4).unwrap(),
                text.anchor(0, 2).unwrap(),
            )
            .unwrap(),
        ];

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            unsupported,
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 1, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::CoverageUnsupportedRegionsMismatch
            )
        );
    }

    #[test]
    fn contract_only_occurrence_extending_into_nested_remainder_is_rejected() {
        let text = source(5, "a{color:red;b{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let occurrence = CssDeclarationOccurrence::new(
            &text,
            text.anchor(2, 12).unwrap(),
            text.anchor(2, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 11).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(11, 12).unwrap(),
            },
            CssDeclarationContext::TopLevelQualifiedRuleLeadingDeclarationZone,
        )
        .unwrap();
        let unsupported = CssParserUnsupportedRegion::new_nested_content_remainder(
            &text,
            text.anchor(10, 15).unwrap(),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            vec![unsupported],
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 0, 0, 1, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::OccurrenceOverlapsUnsupportedRegion {
                    index: 0,
                    unsupported_index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_occurrence_overlapping_top_level_at_rule_is_rejected() {
        let text = source(9001, "@font-face{color:red;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        // Synthetic: a would-be declaration occurrence whose `complete`
        // range sits inside a `TopLevelAtRule` unsupported region. A real
        // producer never emits both for the same source region; this
        // proves the run-result contract rejects it regardless.
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            text.anchor(11, 21).unwrap(),
            text.anchor(11, 16).unwrap(),
            text.anchor(16, 17).unwrap(),
            text.anchor(17, 20).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(20, 21).unwrap(),
            },
            CssDeclarationContext::TopLevelQualifiedRuleLeadingDeclarationZone,
        )
        .unwrap();
        let unsupported = CssParserUnsupportedRegion::new_top_level_at_rule(
            &text,
            text.anchor(0, 22).unwrap(),
            text.anchor(0, 10).unwrap(),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            vec![unsupported],
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 0, 0, 1, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::OccurrenceOverlapsUnsupportedRegion {
                    index: 0,
                    unsupported_index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_omitted_at_end_of_input_occurrence_requires_upstream_true_eof() {
        let text = source(9002, "a{color:red");
        let len = text.as_str().len();

        // Upstream never confirms true `EndOfInput`: it is resource-limited
        // exactly at the source's end, which is a structurally valid but
        // lifecycle-incomplete tokenizer result.
        let resource_limit =
            crate::css::tokenizer::resource::CssTokenizerResourceLimitEvidence::new(
                &text,
                crate::css::tokenizer::resource::CssTokenizerResourceKind::AlgorithmSteps,
                1,
                2,
                text.anchor(len, len).unwrap(),
            )
            .unwrap();
        let upstream = CssTokenizerRunResult::new(
            &text,
            None,
            vec![CssLexicalItem::SemanticToken(
                CssToken::new(
                    &text,
                    text.anchor(0, len).unwrap(),
                    CssTokenKind::Ident("x".to_owned()),
                )
                .unwrap(),
            )],
            Vec::<CssTokenizerDiagnostic>::new(),
            text.anchor(0, len).unwrap(),
            text.anchor(len, len).unwrap(),
            text.anchor(len, len).unwrap(),
            CssTokenizerCompletion::Incomplete,
            CssTokenizerTermination::ResourceLimit(resource_limit),
            CssTokenizerResourceUsage::new(0, 1, 1, 0, 0, 0),
        )
        .unwrap();

        // The occurrence itself is internally self-consistent (its EOF
        // terminal really is an empty anchor at the true end of the raw
        // source), which is all `CssDeclarationOccurrence::new` can check
        // without tokenizer lifecycle input.
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            text.anchor(2, 11).unwrap(),
            text.anchor(2, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 11).unwrap(),
            None,
            CssDeclarationTermination::OmittedAtEndOfInput {
                terminal: text.anchor(11, 11).unwrap(),
            },
            CssDeclarationContext::TopLevelQualifiedRuleLeadingDeclarationZone,
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::UpstreamTokenizerIncomplete,
            CssParserResourceUsage::new(1, 0, 1, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::OmittedAtEndOfInputRequiresUpstreamComplete {
                    index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_recovery_end_of_input_accepted_with_upstream_complete_end_of_input() {
        let text = source(9010, "a{color red");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let recovery = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 11).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EndOfInput {
                terminal: text.anchor(11, 11).unwrap(),
            },
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![recovery],
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 1, 0, 0),
        )
        .unwrap();

        assert_eq!(result.recovery_records().len(), 1);
    }

    #[test]
    fn contract_only_recovery_end_of_input_rejected_with_upstream_resource_limited() {
        let text = source(9011, "a{color red");
        let len = text.as_str().len();

        // Upstream never confirms true `EndOfInput`: it is resource-limited
        // exactly at the source's end, which is a structurally valid but
        // lifecycle-incomplete tokenizer result. This also demonstrates that
        // a resource-limited upstream terminal coinciding with true source
        // end cannot masquerade as true-EOF recovery: the coupling checks
        // upstream completion/termination, not merely terminal offset.
        let resource_limit =
            crate::css::tokenizer::resource::CssTokenizerResourceLimitEvidence::new(
                &text,
                crate::css::tokenizer::resource::CssTokenizerResourceKind::AlgorithmSteps,
                1,
                2,
                text.anchor(len, len).unwrap(),
            )
            .unwrap();
        let upstream = CssTokenizerRunResult::new(
            &text,
            None,
            vec![CssLexicalItem::SemanticToken(
                CssToken::new(
                    &text,
                    text.anchor(0, len).unwrap(),
                    CssTokenKind::Ident("x".to_owned()),
                )
                .unwrap(),
            )],
            Vec::<CssTokenizerDiagnostic>::new(),
            text.anchor(0, len).unwrap(),
            text.anchor(len, len).unwrap(),
            text.anchor(len, len).unwrap(),
            CssTokenizerCompletion::Incomplete,
            CssTokenizerTermination::ResourceLimit(resource_limit),
            CssTokenizerResourceUsage::new(0, 1, 1, 0, 0, 0),
        )
        .unwrap();

        let recovery = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(2, 11).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::EndOfInput {
                terminal: text.anchor(11, 11).unwrap(),
            },
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![recovery],
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::UpstreamTokenizerIncomplete,
            CssParserResourceUsage::new(1, 0, 0, 0, 1, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::RecoveryEndOfInputRequiresUpstreamComplete {
                    index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_upstream_source_identity_mismatch_is_rejected() {
        let text = source(6, "a{}");
        let other = source(7, "a{}");
        let upstream = complete_tokenizer_run(&other);

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(3, 3).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            empty_resources(),
        );

        assert!(matches!(
            result,
            Err(CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::UpstreamSourceIdentityMismatch { .. }
            ))
        ));
    }

    #[test]
    fn contract_only_resource_limit_evidence_owns_termination_and_terminal() {
        let text = source(8, "a{color:red;color:blue;}");
        let upstream = complete_tokenizer_run(&text);
        let evidence = CssParserResourceLimitEvidence::new(
            &text,
            CssParserResourceKind::DeclarationOccurrences,
            1,
            2,
            text.anchor(12, 12).unwrap(),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(12, 12).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::ParserResourceLimit(evidence),
            empty_resources(),
        )
        .unwrap();

        assert_eq!(result.terminal().range().start(), 12);
    }

    #[test]
    fn contract_only_checked_resource_add_reports_typed_overflow() {
        let result = checked_resource_add(CssParserResourceKind::AlgorithmSteps, usize::MAX, 1);
        assert_eq!(
            result,
            Err(CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ResourceAccountingOverflow {
                    kind: CssParserResourceKind::AlgorithmSteps,
                    current: usize::MAX,
                    additional: 1,
                }
            ))
        );
    }

    #[test]
    fn contract_only_resource_usage_count_mismatch_is_rejected() {
        let text = source(9, "a{color:red;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 3, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ResourceUsageCountMismatch {
                    kind: CssParserResourceKind::DeclarationOccurrences,
                    expected: 0,
                    actual: 3,
                }
            )
        );
    }

    #[test]
    fn contract_only_debug_output_does_not_disclose_authored_source() {
        const SECRET: &str = "secret-parser-run-result-source";
        let text = source(10, SECRET);
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            empty_resources(),
        )
        .unwrap();

        assert!(!format!("{result:?}").contains(SECRET));
    }

    fn discard_evidence(
        text: &SourceText,
        region: (usize, usize),
        property_name: (usize, usize),
        colon: (usize, usize),
        decoded_property_name: &str,
    ) -> CssParserDiscardEvidence {
        CssParserDiscardEvidence::new(
            text,
            text.anchor(region.0, region.1).unwrap(),
            text.anchor(property_name.0, property_name.1).unwrap(),
            text.anchor(colon.0, colon.1).unwrap(),
            decoded_property_name,
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        )
        .unwrap()
    }

    #[test]
    fn contract_only_discard_record_coexists_with_supported_coverage() {
        let text = source(9003, "--foo:bar{color:red;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let discard = discard_evidence(&text, (0, 21), (0, 5), (5, 6), "--foo");

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![discard],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 1),
        )
        .unwrap();

        assert_eq!(result.discard_records().len(), 1);
        assert_eq!(
            result.coverage(),
            CssParserCoverage::SupportedForSelectedQuestion
        );
    }

    #[test]
    fn contract_only_discard_overlapping_occurrence_is_rejected() {
        let text = source(9004, "a{color:red;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let occurrence = CssDeclarationOccurrence::new(
            &text,
            text.anchor(2, 12).unwrap(),
            text.anchor(2, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 11).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(11, 12).unwrap(),
            },
            CssDeclarationContext::TopLevelQualifiedRuleLeadingDeclarationZone,
        )
        .unwrap();
        // Synthetic: a real producer never emits an occurrence and a discard
        // record for the same construct; this proves the run-result
        // contract rejects it regardless. The discard's colon reuses the
        // real ":" already present in "color:red" at the same offsets.
        let discard = discard_evidence(&text, (2, 12), (2, 7), (7, 8), "--fake");

        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![discard],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DiscardOverlapsOccurrence {
                    index: 0,
                    occurrence_index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_discard_overlapping_recovery_is_rejected() {
        let text = source(9005, "--p:q;rest");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let recovery = CssParserRecoveryEvidence::new(
            &text,
            text.anchor(0, 6).unwrap(),
            CssParserRecoveryKind::MalformedBlockItem,
            CssParserRecoveryTermination::AuthoredSemicolon {
                semicolon: text.anchor(5, 6).unwrap(),
            },
        )
        .unwrap();
        let discard = discard_evidence(&text, (0, 4), (0, 2), (3, 4), "--p");

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![recovery],
            Vec::new(),
            vec![discard],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 1, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DiscardOverlapsRecoveryRegion {
                    index: 0,
                    recovery_index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_discard_overlapping_unsupported_is_rejected() {
        let text = source(9006, "--p:q;rest");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();

        let unsupported = CssParserUnsupportedRegion::new_nested_content_remainder(
            &text,
            text.anchor(0, 4).unwrap(),
        )
        .unwrap();
        let discard = discard_evidence(&text, (0, 4), (0, 2), (3, 4), "--p");

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![unsupported],
            vec![discard],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 1, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DiscardOverlapsUnsupportedRegion {
                    index: 0,
                    unsupported_index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_discard_beyond_terminal_is_rejected() {
        let text = source(9007, "--p:q;rest");
        let upstream = complete_tokenizer_run(&text);
        let discard = discard_evidence(&text, (0, 6), (0, 2), (3, 4), "--p");
        let evidence = CssParserResourceLimitEvidence::new(
            &text,
            CssParserResourceKind::AlgorithmSteps,
            1,
            2,
            text.anchor(4, 4).unwrap(),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![discard],
            text.anchor(4, 4).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::ParserResourceLimit(evidence),
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DiscardBeyondTerminal {
                    index: 0,
                    end: 6,
                    terminal: 4,
                }
            )
        );
    }

    #[test]
    fn contract_only_discard_order_violation_is_rejected() {
        let text = source(9008, "--aa:1;--bb:2;");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let discard_a = discard_evidence(&text, (0, 7), (0, 4), (4, 5), "--aa");
        let discard_b = discard_evidence(&text, (7, 14), (7, 11), (11, 12), "--bb");

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![discard_b, discard_a],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DiscardOrderViolation { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_discard_resource_count_mismatch_is_rejected() {
        let text = source(9009, "--foo:bar{color:red;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let discard = discard_evidence(&text, (0, 21), (0, 5), (5, 6), "--foo");

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![discard],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ResourceUsageCountMismatch {
                    kind: CssParserResourceKind::DiscardRecords,
                    expected: 1,
                    actual: 2,
                }
            )
        );
    }
}
