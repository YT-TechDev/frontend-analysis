use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRange, SourceRangeError, SourceText};

use super::super::declaration::{
    CssDeclarationContractError, CssDeclarationOccurrence, CssDeclarationTermination,
};
use super::super::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerRunResult, CssTokenizerTermination,
};
use super::context::{
    CssParserContextContractError, CssParserContextRecord, CssParserContextTermination,
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
    /// Structurally committed/retained parser-context evidence (#166
    /// contract, #167 production): empty for every #166-only execution, and
    /// populated with real `QualifiedRuleBlock` records wherever #167
    /// retains at least one nested qualified-rule context.
    context_records: Vec<CssParserContextRecord>,
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
        context_records: Vec<CssParserContextRecord>,
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
            &context_records,
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
            context_records,
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

    /// Retained parser-context evidence, in deterministic source-allocation
    /// order (#166 contract, #167 production).
    pub(crate) fn context_records(&self) -> &[CssParserContextRecord] {
        &self.context_records
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

impl From<CssParserContextContractError> for CssParserRunError {
    fn from(error: CssParserContextContractError) -> Self {
        Self::InternalInvariantFailure(CssParserInvariantViolation::ContextContractViolation {
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
    Context { index: usize },
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
    ContextContractViolation {
        error: CssParserContextContractError,
    },
    /// A context record's `id()` did not equal its vector index (#166
    /// contiguous run-local identity).
    ContextIdIndexMismatch {
        index: usize,
    },
    /// A context record's parent `ContextId` does not refer to an
    /// earlier-retained record (forward parent or self-parent).
    ContextParentNotBefore {
        index: usize,
    },
    /// A child context's authored/partial extent is not contained in its
    /// parent's retained `body`.
    ContextChildOutsideParentBody {
        index: usize,
    },
    /// Two retained sibling contexts in the same ordinal scope (a real
    /// parent, or the implicit stylesheet root when both have no parent)
    /// claimed the same direct-item ordinal.
    ContextDuplicateSiblingItemOrdinal {
        index: usize,
    },
    /// Retained sibling-context item ordinals in the same scope did not
    /// strictly increase in retained (vector/id) order.
    ContextSiblingOrdinalOrderViolation {
        index: usize,
    },
    /// A retained sibling context's numeric ordinal increased relative to
    /// the previous sibling in its scope, but its source extent did not
    /// agree (reversed or overlapping raw source position).
    ContextSiblingSourceOrderViolation {
        index: usize,
    },
    ContextBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    /// An `EndOfInput` context termination requires the upstream tokenizer
    /// to have completed at true `EndOfInput` and the parser run itself to
    /// terminate at `EndOfTokenizerInput`.
    ContextEndOfInputRequiresUpstreamTrueEof {
        index: usize,
    },
    /// An `UpstreamTokenizerIncomplete` context termination requires the
    /// parser run's own termination to be `UpstreamTokenizerIncomplete`.
    ContextUpstreamIncompleteRequiresMatchingRunTermination {
        index: usize,
    },
    /// A `ParserResourceLimit` context termination requires the parser run's
    /// own termination to be `ParserResourceLimit`.
    ContextResourceLimitRequiresMatchingRunTermination {
        index: usize,
    },
    /// A partial context's terminal evidence does not agree with the parser
    /// run's own terminal, though both claim the same lifecycle boundary.
    ContextTerminalMismatchWithRunTerminal {
        index: usize,
    },
    /// Achieved `PeakContextDepth` usage did not equal the maximum ancestry
    /// depth represented by retained context records.
    ContextPeakDepthUsageMismatch {
        expected: usize,
        actual: usize,
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
    /// `commit_declaration` or `begin_checkpoint` ran with no active
    /// context on the stack; declarations only ever occur inside a
    /// qualified-rule context (#167).
    DeclarationOutsideActiveContext,
    /// An authored `}` or unsupported-remainder closure was reached with no
    /// active context to close (#167).
    NoActiveContextToClose,
    /// A reserved context slot was never finalized before result
    /// construction (#167): every reservation must be closed by an
    /// authored `}` or by run-stop cleanup.
    UnfinalizedContextRecord {
        index: usize,
    },
    /// `scan_qualified_rule_fallback` reached a custom-property-shaped
    /// candidate's block opener (#167 defensive guard): unreachable for any
    /// accepted #167 input, since `try_declaration` always recognizes an
    /// `is_custom` property regardless of value complexity.
    UnreachableNestedCustomPropertyFallback,
    /// A declaration's placement referenced a `CssParserContextId` with no
    /// corresponding retained context record.
    DeclarationPlacementUnknownContext {
        index: usize,
    },
    /// A declaration's complete source range is not contained in its
    /// placement context's retained `body`.
    DeclarationOutsidePlacementContextBody {
        index: usize,
    },
    /// A declaration and a child qualified-rule context retained by the
    /// same owning context claimed the same direct-item ordinal.
    DirectItemDuplicateOrdinal {
        context_id: usize,
    },
    /// Direct-item ordinals (declarations and child contexts combined) in
    /// one owning context did not strictly increase with source order.
    DirectItemOrderViolation {
        context_id: usize,
    },
    /// A declaration-run ordinal did not follow the approved run semantics
    /// (zero-based per context, non-decreasing, and never reused after a
    /// materialized child context has closed the previous run).
    DeclarationRunOrdinalViolation {
        index: usize,
    },
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
    context_records: &[CssParserContextRecord],
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

    let achieved_peak_context_depth = validate_contexts(
        expected_source,
        context_records,
        terminal_offset,
        terminal,
        upstream,
        termination,
    )?;

    validate_declaration_placement(occurrences, context_records)?;

    validate_resource_counts(
        resources,
        occurrences.len(),
        parser_diagnostics.len(),
        recovery_records.len(),
        unsupported_regions.len(),
        discard_records.len(),
        context_records.len(),
    )?;

    let actual_peak_context_depth = resources.value(CssParserResourceKind::PeakContextDepth);
    if actual_peak_context_depth != achieved_peak_context_depth {
        return invariant(CssParserInvariantViolation::ContextPeakDepthUsageMismatch {
            expected: achieved_peak_context_depth,
            actual: actual_peak_context_depth,
        });
    }

    Ok(())
}

/// Validates retained context evidence: run-local ID/index contiguity,
/// parent existence/ordering, source-containment within the parent's `body`,
/// sibling item-ordinal uniqueness/order/source-agreement (applied both to
/// real-parent siblings and to the implicit-root sibling set), the
/// context-beyond-terminal bound, and partial-termination/lifecycle coupling
/// against the parser run's own upstream/termination evidence. Returns the
/// achieved maximum ancestry depth represented by `context_records` (0 when
/// empty), which the caller reconciles against the committed
/// `PeakContextDepth` usage.
fn validate_contexts(
    expected_source: SourceId,
    context_records: &[CssParserContextRecord],
    terminal_offset: usize,
    terminal: &SourceAnchor,
    upstream: &CssTokenizerRunResult,
    termination: &CssParserTermination,
) -> Result<usize, CssParserRunError> {
    let upstream_true_eof = upstream_ended_at_true_eof(upstream);
    let mut depths: Vec<usize> = Vec::with_capacity(context_records.len());
    // Keyed by the ordinal scope: `Some(parent_index)` for a real parent,
    // `None` for the implicit stylesheet root. Every retained context
    // belongs to exactly one such scope and carries one ordinal within it.
    let mut last_sibling_ordinal: std::collections::BTreeMap<Option<usize>, usize> =
        std::collections::BTreeMap::new();
    let mut last_sibling_extent_end: std::collections::BTreeMap<Option<usize>, usize> =
        std::collections::BTreeMap::new();

    for (index, record) in context_records.iter().enumerate() {
        if record.id().index() != index {
            return invariant(CssParserInvariantViolation::ContextIdIndexMismatch { index });
        }
        require_source(
            expected_source,
            record.header(),
            CssParserRunEvidenceRole::Context { index },
        )?;

        let extent_start = record.extent_start();
        let extent_end = record.extent_end();
        if extent_end > terminal_offset {
            return invariant(CssParserInvariantViolation::ContextBeyondTerminal {
                index,
                end: extent_end,
                terminal: terminal_offset,
            });
        }

        let scope_key = record.parent().map(|parent_id| parent_id.index());

        let depth = if let Some(parent_id) = record.parent() {
            let parent_index = parent_id.index();
            if parent_index >= index {
                return invariant(CssParserInvariantViolation::ContextParentNotBefore { index });
            }
            let parent_record = &context_records[parent_index];
            let parent_body = parent_record.body().range();
            if record.header().source_id() != parent_record.body().source_id()
                || extent_start < parent_body.start()
                || extent_end > parent_body.end()
            {
                return invariant(CssParserInvariantViolation::ContextChildOutsideParentBody {
                    index,
                });
            }

            depths[parent_index] + 1
        } else {
            1
        };
        depths.push(depth);

        let ordinal = record.item_ordinal().value();
        if let Some(&previous_ordinal) = last_sibling_ordinal.get(&scope_key) {
            if ordinal == previous_ordinal {
                return invariant(
                    CssParserInvariantViolation::ContextDuplicateSiblingItemOrdinal { index },
                );
            }
            if ordinal < previous_ordinal {
                return invariant(
                    CssParserInvariantViolation::ContextSiblingOrdinalOrderViolation { index },
                );
            }
        }
        if let Some(&previous_extent_end) = last_sibling_extent_end.get(&scope_key)
            && extent_start < previous_extent_end
        {
            return invariant(
                CssParserInvariantViolation::ContextSiblingSourceOrderViolation { index },
            );
        }
        last_sibling_ordinal.insert(scope_key, ordinal);
        last_sibling_extent_end.insert(scope_key, extent_end);

        match record.termination() {
            CssParserContextTermination::AuthoredRightCurly { .. } => {}
            CssParserContextTermination::EndOfInput { .. } => {
                if !upstream_true_eof
                    || !matches!(termination, CssParserTermination::EndOfTokenizerInput)
                {
                    return invariant(
                        CssParserInvariantViolation::ContextEndOfInputRequiresUpstreamTrueEof {
                            index,
                        },
                    );
                }
            }
            CssParserContextTermination::UpstreamTokenizerIncomplete {
                terminal: context_terminal,
            } => {
                if !matches!(
                    termination,
                    CssParserTermination::UpstreamTokenizerIncomplete
                ) {
                    return invariant(
                        CssParserInvariantViolation::ContextUpstreamIncompleteRequiresMatchingRunTermination {
                            index,
                        },
                    );
                }
                if !same_anchor(context_terminal, terminal) {
                    return invariant(
                        CssParserInvariantViolation::ContextTerminalMismatchWithRunTerminal {
                            index,
                        },
                    );
                }
            }
            CssParserContextTermination::ParserResourceLimit {
                terminal: context_terminal,
            } => {
                if !matches!(termination, CssParserTermination::ParserResourceLimit(_)) {
                    return invariant(
                        CssParserInvariantViolation::ContextResourceLimitRequiresMatchingRunTermination {
                            index,
                        },
                    );
                }
                if !same_anchor(context_terminal, terminal) {
                    return invariant(
                        CssParserInvariantViolation::ContextTerminalMismatchWithRunTerminal {
                            index,
                        },
                    );
                }
            }
        }
    }

    Ok(depths.into_iter().max().unwrap_or(0))
}

/// One materialized direct block-content item's ordering data, keyed by its
/// owning context for [`validate_declaration_placement`]: either a
/// declaration (carrying its `occurrences` index and run ordinal) or a
/// retained child qualified-rule context.
enum DirectItem {
    Declaration { index: usize, run_ordinal: usize },
    ChildContext,
}

/// Reconciles declaration placement against retained context evidence
/// (#167): every declaration's [`CssParserContextId`] refers to an existing
/// retained context, its complete range lies within that context's `body`,
/// and -- per owning context -- the combined direct-item sequence
/// (declarations interleaved with child qualified-rule contexts, sharing
/// one ordinal space) has gapless strictly-increasing ordinals consistent
/// with source order, and declaration-run ordinals are zero-based,
/// non-decreasing, shared by consecutive declarations, and reset to the
/// next value by an intervening materialized child context.
fn validate_declaration_placement(
    occurrences: &[CssDeclarationOccurrence],
    context_records: &[CssParserContextRecord],
) -> Result<(), CssParserRunError> {
    let mut items_by_context: std::collections::BTreeMap<
        usize,
        Vec<(usize, usize, usize, DirectItem)>,
    > = std::collections::BTreeMap::new();

    for (index, occurrence) in occurrences.iter().enumerate() {
        let placement = occurrence.placement();
        let context_index = placement.context_id().index();
        let context = context_records.get(context_index).ok_or(
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DeclarationPlacementUnknownContext { index },
            ),
        )?;
        let complete = occurrence.complete().range();
        let body = context.body().range();
        if occurrence.complete().source_id() != context.body().source_id()
            || complete.start() < body.start()
            || complete.end() > body.end()
        {
            return invariant(
                CssParserInvariantViolation::DeclarationOutsidePlacementContextBody { index },
            );
        }

        items_by_context.entry(context_index).or_default().push((
            placement.item_ordinal().value(),
            complete.start(),
            complete.end(),
            DirectItem::Declaration {
                index,
                run_ordinal: placement.run_ordinal().value(),
            },
        ));
    }

    for record in context_records {
        if let Some(parent_id) = record.parent() {
            items_by_context
                .entry(parent_id.index())
                .or_default()
                .push((
                    record.item_ordinal().value(),
                    record.extent_start(),
                    record.extent_end(),
                    DirectItem::ChildContext,
                ));
        }
    }

    for (context_id, mut items) in items_by_context {
        items.sort_by_key(|(ordinal, ..)| *ordinal);

        let mut expected_next_ordinal = 0usize;
        let mut previous_ordinal: Option<usize> = None;
        let mut previous_end: Option<usize> = None;
        let mut current_run: Option<usize> = None;
        let mut expected_next_run = 0usize;

        for (ordinal, start, end, kind) in items {
            if ordinal != expected_next_ordinal {
                if previous_ordinal == Some(ordinal) {
                    return invariant(CssParserInvariantViolation::DirectItemDuplicateOrdinal {
                        context_id,
                    });
                }
                return invariant(CssParserInvariantViolation::DirectItemOrderViolation {
                    context_id,
                });
            }
            if let Some(previous_end) = previous_end
                && start < previous_end
            {
                return invariant(CssParserInvariantViolation::DirectItemOrderViolation {
                    context_id,
                });
            }
            previous_ordinal = Some(ordinal);
            previous_end = Some(end);
            expected_next_ordinal = ordinal + 1;

            match kind {
                DirectItem::Declaration { index, run_ordinal } => match current_run {
                    None => {
                        if run_ordinal != expected_next_run {
                            return invariant(
                                CssParserInvariantViolation::DeclarationRunOrdinalViolation {
                                    index,
                                },
                            );
                        }
                        current_run = Some(run_ordinal);
                        expected_next_run = run_ordinal + 1;
                    }
                    Some(open_run) => {
                        if run_ordinal != open_run {
                            return invariant(
                                CssParserInvariantViolation::DeclarationRunOrdinalViolation {
                                    index,
                                },
                            );
                        }
                    }
                },
                DirectItem::ChildContext => {
                    current_run = None;
                }
            }
        }
    }

    Ok(())
}

/// The single upstream/source-boundary invariant check: exact source
/// identity, `processed_prefix`/`unprocessed_remainder` boundary agreement
/// with the upstream terminal, and exact retained-fragment reconciliation
/// against `source_text`. Shared by [`super::producer::run`] (fail-fast,
/// before any parser semantics execute) and [`CssParserRunResult::new`]
/// (defense in depth at result construction) so the two never drift.
pub(super) fn validate_upstream_boundary(
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
    context_count: usize,
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
    check_count(
        resources,
        CssParserResourceKind::ContextRecords,
        context_count,
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
    use crate::css::declaration::{
        CssDeclarationPlacement, CssDeclarationRunOrdinal, CssDeclarationTermination,
    };
    use crate::css::parser::context::{CssParserContextId, CssParserDirectItemOrdinal};
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
        CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0, 0, 0)
    }

    fn test_placement() -> CssDeclarationPlacement {
        CssDeclarationPlacement::new(
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssDeclarationRunOrdinal::new(0),
        )
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 1, 0, 0),
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
            test_placement(),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 1, 0, 0),
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
            test_placement(),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 1, 0, 0),
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
            test_placement(),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::UpstreamTokenizerIncomplete,
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 0, 0, 0),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 1, 0, 0, 0),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::UpstreamTokenizerIncomplete,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 1, 0, 0, 0),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 3, 0, 0, 0, 0, 0),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0, 1, 0),
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
            test_placement(),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 0, 1, 0),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 1, 0, 1, 0),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 1, 1, 0),
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
            Vec::new(),
            text.anchor(4, 4).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::ParserResourceLimit(evidence),
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0, 1, 0),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0, 2, 0),
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 0, 0, 0, 0, 2, 0),
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

    // -------------------------------------------------------------------
    // #166 result-level context corruption matrix.
    // -------------------------------------------------------------------

    use crate::css::parser::context::CssParserContextKind;

    fn context_resources(
        context_records: usize,
        peak_context_depth: usize,
    ) -> CssParserResourceUsage {
        CssParserResourceUsage::new(1, 0, peak_context_depth, 0, 0, 0, 0, 0, context_records)
    }

    /// A single valid top-level `QualifiedRuleBlock` context over `"a{x}"`
    /// (`extent` `[0, 4)`), closed by an authored `}`, with the caller-
    /// supplied implicit-root-scoped item ordinal.
    fn single_top_level_context(
        text: &SourceText,
        id: usize,
        item_ordinal: usize,
    ) -> CssParserContextRecord {
        CssParserContextRecord::new(
            text,
            CssParserContextId::new(id),
            None,
            CssParserDirectItemOrdinal::new(item_ordinal),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 3).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(3, 4).unwrap(),
            },
        )
        .unwrap()
    }

    /// A valid two-level `"a{b{1}c{2}}"` fixture (outer `a{...}` containing
    /// two child block contexts `b{1}` and `c{2}`), with the children's
    /// parent-local item ordinals supplied by the caller so tests can exceed
    /// the well-formed `[0, 1]` ordering.
    fn two_children_under_one_parent(
        text: &SourceText,
        first_ordinal: usize,
        second_ordinal: usize,
    ) -> Vec<CssParserContextRecord> {
        let parent_id = CssParserContextId::new(0);
        let outer = CssParserContextRecord::new(
            text,
            parent_id,
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap();
        let child_b = CssParserContextRecord::new(
            text,
            CssParserContextId::new(1),
            Some(parent_id),
            CssParserDirectItemOrdinal::new(first_ordinal),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 5).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(5, 6).unwrap(),
            },
        )
        .unwrap();
        let child_c = CssParserContextRecord::new(
            text,
            CssParserContextId::new(2),
            Some(parent_id),
            CssParserDirectItemOrdinal::new(second_ordinal),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(9, 10).unwrap(),
            },
        )
        .unwrap();
        vec![outer, child_b, child_c]
    }

    #[test]
    fn contract_only_context_id_index_mismatch_is_rejected() {
        let text = source(20_001, "a{x}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        // Constructed with a run-local id of 1 at vector index 0.
        let context = single_top_level_context(&text, 1, 0);

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(1, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextIdIndexMismatch { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_context_forward_parent_is_rejected() {
        let text = source(20_002, "a{b{1}c{2}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let parent_id = CssParserContextId::new(0);
        // A first record (index 0) that already claims a parent cannot
        // refer to any earlier-retained record.
        let context = CssParserContextRecord::new(
            &text,
            parent_id,
            Some(parent_id),
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
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
            vec![context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(1, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextParentNotBefore { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_context_child_outside_parent_body_is_rejected() {
        let text = source(20_003, "a{b{1}c{2}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let parent_id = CssParserContextId::new(0);
        let outer = CssParserContextRecord::new(
            &text,
            parent_id,
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap();
        // This child's own local evidence is self-consistent, but its
        // authored close reuses the outer context's own closing `}` byte,
        // extending past the outer's retained `body` (which ends at 10).
        let escaping_child = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(1),
            Some(parent_id),
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
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
            vec![outer, escaping_child],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(2, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextChildOutsideParentBody { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_context_duplicate_sibling_item_ordinal_is_rejected() {
        let text = source(20_004, "a{b{1}c{2}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let records = two_children_under_one_parent(&text, 0, 0);

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            records,
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(3, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextDuplicateSiblingItemOrdinal { index: 2 }
            )
        );
    }

    #[test]
    fn contract_only_context_sibling_ordinal_order_violation_is_rejected() {
        let text = source(20_005, "a{b{1}c{2}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let records = two_children_under_one_parent(&text, 1, 0);

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            records,
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(3, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextSiblingOrdinalOrderViolation { index: 2 }
            )
        );
    }

    #[test]
    fn contract_only_root_scoped_sibling_duplicate_item_ordinal_is_rejected() {
        let text = source(20_012, "a{x}b{y}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let first = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 3).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(3, 4).unwrap(),
            },
        )
        .unwrap();
        // Same implicit-root scope as `first` (both `parent = None`), and
        // claims the same ordinal.
        let second = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(1),
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(4, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 7).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(7, 8).unwrap(),
            },
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
            vec![first, second],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(2, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextDuplicateSiblingItemOrdinal { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_root_scoped_sibling_ordinal_order_violation_is_rejected() {
        let text = source(20_013, "a{x}b{y}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let first = single_top_level_context(&text, 0, 1);
        let second = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(1),
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(4, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 7).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(7, 8).unwrap(),
            },
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
            vec![first, second],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(2, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextSiblingOrdinalOrderViolation { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_root_scoped_sibling_source_order_violation_is_rejected() {
        // `second`'s ordinal (1) increases relative to `first`'s (0), but
        // its source extent fully overlaps `first`'s instead of following
        // it: increasing ordinals must not paper over reversed/overlapping
        // retained source extents.
        let text = source(20_014, "a{x}b{y}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let first = single_top_level_context(&text, 0, 0);
        let second = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(1),
            None,
            CssParserDirectItemOrdinal::new(1),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 3).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(3, 4).unwrap(),
            },
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
            vec![first, second],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(2, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextSiblingSourceOrderViolation { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_root_scoped_siblings_with_ordinal_gaps_are_accepted() {
        let text = source(20_015, "a{x}b{y}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let first = single_top_level_context(&text, 0, 0);
        let second = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(1),
            None,
            // Ordinal 1 is presumed reserved for a future declaration item
            // between the two top-level rules; the gap is valid.
            CssParserDirectItemOrdinal::new(2),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(4, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 7).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(7, 8).unwrap(),
            },
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
            vec![first, second],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(2, 1),
        )
        .unwrap();

        assert_eq!(result.context_records().len(), 2);
    }

    #[test]
    fn contract_only_context_beyond_terminal_is_rejected() {
        let text = source(20_006, "a{x}");
        let context = single_top_level_context(&text, 0, 0);
        let resource_limit = CssParserResourceLimitEvidence::new(
            &text,
            CssParserResourceKind::AlgorithmSteps,
            1,
            2,
            text.anchor(3, 3).unwrap(),
        )
        .unwrap();
        let upstream = complete_tokenizer_run(&text);

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![context],
            text.anchor(3, 3).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::ParserResourceLimit(resource_limit),
            context_resources(1, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextBeyondTerminal {
                    index: 0,
                    end: 4,
                    terminal: 3,
                }
            )
        );
    }

    #[test]
    fn contract_only_context_records_usage_count_mismatch_is_rejected() {
        let text = source(20_007, "a{x}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let context = single_top_level_context(&text, 0, 0);

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(2, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ResourceUsageCountMismatch {
                    kind: CssParserResourceKind::ContextRecords,
                    expected: 1,
                    actual: 2,
                }
            )
        );
    }

    #[test]
    fn contract_only_peak_context_depth_usage_mismatch_is_rejected() {
        let text = source(20_008, "a{b{1}c{2}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let records = two_children_under_one_parent(&text, 0, 1);

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            records,
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            // Achieved depth is really 2 (outer=1, each child=2); this
            // claims 1.
            context_resources(3, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextPeakDepthUsageMismatch {
                    expected: 2,
                    actual: 1,
                }
            )
        );
    }

    #[test]
    fn contract_only_context_end_of_input_requires_upstream_true_eof() {
        let text = source(20_009, "a{color red");
        let len = text.as_str().len();
        // Locally valid `EndOfInput` context evidence (terminal sits at the
        // true retained source end), but the upstream tokenizer itself never
        // confirmed true `EndOfInput`.
        let context = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, len).unwrap(),
            CssParserContextTermination::EndOfInput {
                terminal: text.anchor(len, len).unwrap(),
            },
        )
        .unwrap();

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

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::UpstreamTokenizerIncomplete,
            context_resources(1, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ContextEndOfInputRequiresUpstreamTrueEof { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_nested_partial_ancestors_at_one_stop_boundary_are_accepted() {
        // Both an outer and an inner context remain active when the run
        // stops on upstream-tokenizer incompleteness; both honestly retain
        // `UpstreamTokenizerIncomplete` evidence at the exact same terminal,
        // with no fabricated authored closure for either.
        let text = source(20_010, "a{b{color:red");
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, len).unwrap(),
            CssParserContextTermination::UpstreamTokenizerIncomplete {
                terminal: text.anchor(len, len).unwrap(),
            },
        )
        .unwrap();
        let inner = CssParserContextRecord::new(
            &text,
            CssParserContextId::new(1),
            Some(CssParserContextId::new(0)),
            CssParserDirectItemOrdinal::new(0),
            CssParserContextKind::QualifiedRuleBlock,
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, len).unwrap(),
            CssParserContextTermination::UpstreamTokenizerIncomplete {
                terminal: text.anchor(len, len).unwrap(),
            },
        )
        .unwrap();

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

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer, inner],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::UpstreamTokenizerIncomplete,
            context_resources(2, 2),
        )
        .unwrap();

        assert_eq!(result.context_records().len(), 2);
    }

    #[test]
    fn contract_only_empty_context_table_with_zero_new_usage_is_accepted() {
        let text = source(20_011, "a{color:red;}");
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
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            context_resources(0, 0),
        )
        .unwrap();

        assert!(result.context_records().is_empty());
        assert_eq!(
            result
                .resources()
                .value(CssParserResourceKind::ContextRecords),
            0
        );
        assert_eq!(
            result
                .resources()
                .value(CssParserResourceKind::PeakContextDepth),
            0
        );
    }
}
