use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRange, SourceRangeError, SourceText};

use super::super::declaration::{
    CssDeclarationContractError, CssDeclarationOccurrence, CssDeclarationTermination,
    CssDescriptorOccurrence, CssKeyframeDeclarationOccurrence, CssPageDeclarationOccurrence,
    CssPageMarginDeclarationOccurrence,
};
use super::super::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerRunResult, CssTokenizerTermination,
};
use super::context::{
    CssParserContextContractError, CssParserContextKind, CssParserContextRecord,
    CssParserContextTermination,
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
    /// Retained #169 descriptor occurrences: semantically distinct from
    /// `occurrences`, counted against the shared `DeclarationOccurrences`
    /// aggregate resource cap, never merged into that vector.
    descriptor_occurrences: Vec<CssDescriptorOccurrence>,
    /// Retained #170 page-declaration occurrences: semantically distinct
    /// from `occurrences`/`descriptor_occurrences`, counted against the same
    /// shared `DeclarationOccurrences` aggregate resource cap, never merged
    /// into either vector.
    page_occurrences: Vec<CssPageDeclarationOccurrence>,
    /// Retained #170 page-margin-declaration occurrences, counted against
    /// the same shared aggregate cap.
    page_margin_occurrences: Vec<CssPageMarginDeclarationOccurrence>,
    /// Retained #171 keyframe declarations; semantically distinct from all
    /// other declaration-shaped occurrence vectors while sharing the same
    /// aggregate `DeclarationOccurrences` resource cap.
    keyframe_occurrences: Vec<CssKeyframeDeclarationOccurrence>,
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
    /// Compatibility constructor for pre-#171 call sites whose result carries
    /// no keyframe occurrences.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        upstream_tokenizer_result: CssTokenizerRunResult,
        occurrences: Vec<CssDeclarationOccurrence>,
        descriptor_occurrences: Vec<CssDescriptorOccurrence>,
        page_occurrences: Vec<CssPageDeclarationOccurrence>,
        page_margin_occurrences: Vec<CssPageMarginDeclarationOccurrence>,
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
        Self::new_with_keyframes(
            source_text,
            upstream_tokenizer_result,
            occurrences,
            descriptor_occurrences,
            page_occurrences,
            page_margin_occurrences,
            Vec::new(),
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
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_with_keyframes(
        source_text: &SourceText,
        upstream_tokenizer_result: CssTokenizerRunResult,
        occurrences: Vec<CssDeclarationOccurrence>,
        descriptor_occurrences: Vec<CssDescriptorOccurrence>,
        page_occurrences: Vec<CssPageDeclarationOccurrence>,
        page_margin_occurrences: Vec<CssPageMarginDeclarationOccurrence>,
        keyframe_occurrences: Vec<CssKeyframeDeclarationOccurrence>,
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
            &descriptor_occurrences,
            &page_occurrences,
            &page_margin_occurrences,
            &keyframe_occurrences,
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
            descriptor_occurrences,
            page_occurrences,
            page_margin_occurrences,
            keyframe_occurrences,
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

    /// Retained #169 descriptor occurrences, in deterministic source order.
    pub(crate) fn descriptor_occurrences(&self) -> &[CssDescriptorOccurrence] {
        &self.descriptor_occurrences
    }

    /// Retained #170 page-declaration occurrences, in deterministic source
    /// order.
    pub(crate) fn page_occurrences(&self) -> &[CssPageDeclarationOccurrence] {
        &self.page_occurrences
    }

    /// Retained #170 page-margin-declaration occurrences, in deterministic
    /// source order.
    pub(crate) fn page_margin_occurrences(&self) -> &[CssPageMarginDeclarationOccurrence] {
        &self.page_margin_occurrences
    }

    /// Retained #171 keyframe declarations, in deterministic source order.
    pub(crate) fn keyframe_occurrences(&self) -> &[CssKeyframeDeclarationOccurrence] {
        &self.keyframe_occurrences
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
    DescriptorOccurrence { index: usize },
    PageOccurrence { index: usize },
    PageMarginOccurrence { index: usize },
    KeyframeOccurrence { index: usize },
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
    /// A `GroupRuleBlock` identity was attempted with no active parent
    /// context (#168): group contexts are never top-level in this Leaf.
    GroupContextCannotBeTopLevel,
    /// A `GroupRuleBlock` active-context frame reached record finalization
    /// without its at-keyword/decoded evidence retained alongside it.
    MissingGroupContextEvidence,
    /// A retained context's `nearest_qualified_ancestor` does not equal the
    /// value structurally derivable from the retained parent table (#168).
    NearestQualifiedAncestorMismatch {
        index: usize,
    },
    /// A retained context's `nearest_qualified_ancestor` refers to a context
    /// that either does not exist yet (not strictly earlier) or is not
    /// itself a `QualifiedRuleBlock`.
    NearestQualifiedAncestorInvalidReference {
        index: usize,
    },
    /// A nested unsupported at-rule's owning context does not exist.
    NestedAtRuleUnknownOwnerContext {
        index: usize,
    },
    /// A nested unsupported at-rule's complete range is not contained in its
    /// owning context's retained `body`.
    NestedAtRuleOutsideOwnerBody {
        index: usize,
    },
    /// A #169 descriptor occurrence's complete range extends beyond the
    /// parser run's own terminal.
    DescriptorOccurrenceBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    /// Retained #169 descriptor occurrences are not in strict, non-
    /// overlapping source order.
    DescriptorOccurrenceOrderViolation {
        index: usize,
    },
    /// A #169 descriptor occurrence's complete range overlaps a retained
    /// unsupported region.
    DescriptorOccurrenceOverlapsUnsupportedRegion {
        index: usize,
        unsupported_index: usize,
    },
    /// A #169 descriptor occurrence's `OmittedAtEndOfInput` termination
    /// requires the upstream tokenizer to have completed at true
    /// `EndOfInput`.
    DescriptorOmittedAtEndOfInputRequiresUpstreamComplete {
        index: usize,
    },
    /// A #169 descriptor occurrence's placement referenced a
    /// `CssParserContextId` with no corresponding retained context record.
    DescriptorPlacementUnknownContext {
        index: usize,
    },
    /// A #169 descriptor occurrence's complete source range is not contained
    /// in its placement context's retained `body`.
    DescriptorOutsidePlacementContextBody {
        index: usize,
    },
    /// A #169 descriptor occurrence's owning context is not a
    /// `DescriptorRuleBlock`.
    DescriptorOwnerMustBeDescriptorContext {
        index: usize,
    },
    /// An ordinary declaration's owning context is a `DescriptorRuleBlock`:
    /// `<declaration-list>` never produces ordinary `CssDeclarationOccurrence`
    /// evidence (#169).
    DeclarationOwnerMustNotBeDescriptorContext {
        index: usize,
    },
    /// A `DescriptorRuleBlock` active-context frame reached record
    /// finalization without its at-keyword/decoded evidence retained
    /// alongside it (#169).
    MissingDescriptorContextEvidence,
    /// A `DescriptorRuleBlock` identity was attempted while nested inside
    /// another active context, or a finalized `DescriptorRuleBlock` record
    /// retained a non-`None` parent: descriptor contexts are
    /// stylesheet-root-only in #169.
    DescriptorContextCannotBeNested,
    /// A new child context was attempted while the innermost active context
    /// was a `DescriptorRuleBlock`: no child context is ever entered from a
    /// descriptor context in #169.
    DescriptorContextCannotHaveChildren,
    /// A retained context's parent is a `DescriptorRuleBlock` context
    /// (#169): descriptor contexts are stylesheet-root-only and never have
    /// children.
    DescriptorContextHasChild {
        index: usize,
    },
    /// #169's descriptor-block malformed-item scan structurally cannot
    /// produce a nested-rule trigger (a top-level `{` is balanced through as
    /// ordinary malformed content, never a context boundary); reaching this
    /// branch is an internal invariant failure, not a panic.
    UnreachableDescriptorNestedRuleTrigger,
    /// A #170 page-declaration occurrence's complete range extends beyond
    /// the parser run's own terminal.
    PageOccurrenceBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    /// Retained #170 page-declaration occurrences are not in strict,
    /// non-overlapping source order.
    PageOccurrenceOrderViolation {
        index: usize,
    },
    /// A #170 page-declaration occurrence's complete range overlaps a
    /// retained unsupported region.
    PageOccurrenceOverlapsUnsupportedRegion {
        index: usize,
        unsupported_index: usize,
    },
    /// A #170 page-declaration occurrence's `OmittedAtEndOfInput`
    /// termination requires the upstream tokenizer to have completed at
    /// true `EndOfInput`.
    PageOmittedAtEndOfInputRequiresUpstreamComplete {
        index: usize,
    },
    /// A #170 page-declaration occurrence's placement referenced a
    /// `CssParserContextId` with no corresponding retained context record.
    PagePlacementUnknownContext {
        index: usize,
    },
    /// A #170 page-declaration occurrence's complete source range is not
    /// contained in its placement context's retained `body`.
    PageOutsidePlacementContextBody {
        index: usize,
    },
    /// A #170 page-declaration occurrence's owning context is not a
    /// `PageRuleBlock`.
    PageOwnerMustBePageContext {
        index: usize,
    },
    /// A #170 page-margin-declaration occurrence's complete range extends
    /// beyond the parser run's own terminal.
    PageMarginOccurrenceBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    /// Retained #170 page-margin-declaration occurrences are not in strict,
    /// non-overlapping source order.
    PageMarginOccurrenceOrderViolation {
        index: usize,
    },
    /// A #170 page-margin-declaration occurrence's complete range overlaps
    /// a retained unsupported region.
    PageMarginOccurrenceOverlapsUnsupportedRegion {
        index: usize,
        unsupported_index: usize,
    },
    /// A #170 page-margin-declaration occurrence's `OmittedAtEndOfInput`
    /// termination requires the upstream tokenizer to have completed at
    /// true `EndOfInput`.
    PageMarginOmittedAtEndOfInputRequiresUpstreamComplete {
        index: usize,
    },
    /// A #170 page-margin-declaration occurrence's placement referenced a
    /// `CssParserContextId` with no corresponding retained context record.
    PageMarginPlacementUnknownContext {
        index: usize,
    },
    /// A #170 page-margin-declaration occurrence's complete source range is
    /// not contained in its placement context's retained `body`.
    PageMarginOutsidePlacementContextBody {
        index: usize,
    },
    /// A #170 page-margin-declaration occurrence's owning context is not a
    /// `PageMarginRuleBlock`.
    PageMarginOwnerMustBePageMarginContext {
        index: usize,
    },
    /// An ordinary declaration's owning context is a `PageRuleBlock`:
    /// `@page` never produces ordinary `CssDeclarationOccurrence` evidence
    /// (#170, mirroring #169's `DeclarationOwnerMustNotBeDescriptorContext`).
    DeclarationOwnerMustNotBePageContext {
        index: usize,
    },
    /// An ordinary declaration's owning context is a `PageMarginRuleBlock`.
    DeclarationOwnerMustNotBePageMarginContext {
        index: usize,
    },
    /// A `PageRuleBlock` active-context frame reached record finalization
    /// without its at-keyword/decoded evidence retained alongside it
    /// (#170).
    MissingPageContextEvidence,
    /// A `PageMarginRuleBlock` active-context frame reached record
    /// finalization without its parent/at-keyword/decoded evidence retained
    /// alongside it (#170).
    MissingPageMarginContextEvidence,
    /// A `PageRuleBlock` identity was attempted while nested inside another
    /// active context, or a finalized `PageRuleBlock` record retained a
    /// non-`None` parent: `PageRuleBlock` is root-owned in #170.
    PageContextCannotBeNested,
    /// A `PageMarginRuleBlock` identity was attempted with no active parent
    /// context: page-margin contexts are never top-level in #170.
    PageMarginContextCannotBeTopLevel,
    /// A `PageMarginRuleBlock` identity was attempted with an active parent
    /// whose own kind is not `PageRuleBlock` (#170).
    PageMarginContextRequiresPageParent,
    /// A new child context was attempted while the innermost active context
    /// was a `PageMarginRuleBlock`: `PageMarginRuleBlock` never has children
    /// in #170.
    PageMarginContextCannotHaveChildren,
    /// A retained context's parent is a `PageRuleBlock` context, but the
    /// child's own kind is not `PageMarginRuleBlock` (#170): the only
    /// context family #170 ever enters from a `PageRuleBlock` is
    /// `PageMarginRuleBlock`.
    PageContextHasNonPageMarginChild {
        index: usize,
    },
    /// A retained context's parent is a `PageMarginRuleBlock` context
    /// (#170): `PageMarginRuleBlock` never has children.
    PageMarginContextHasChild {
        index: usize,
    },
    /// A retained `PageMarginRuleBlock` record's parent either does not
    /// exist or is not itself a `PageRuleBlock` (#170): proved
    /// independently from the retained parent table, mirroring
    /// [`Self::NearestQualifiedAncestorInvalidReference`]'s refusal to trust
    /// producer-provided ancestry without checking it here.
    PageMarginParentMustBePageContext {
        index: usize,
    },
    /// A retained `PageMarginRuleBlock` record carries non-`None`
    /// `page_selector_list` evidence (#170): only a `PageRuleBlock` may ever
    /// carry a selector-list envelope.
    PageMarginContextCarriesSelectorList {
        index: usize,
    },
    /// A #171 keyframe declaration occurrence extends beyond the parser
    /// terminal.
    KeyframeOccurrenceBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    /// Retained keyframe occurrences are not in strict non-overlapping source
    /// order.
    KeyframeOccurrenceOrderViolation {
        index: usize,
    },
    /// A keyframe occurrence overlaps explicit unsupported evidence.
    KeyframeOccurrenceOverlapsUnsupportedRegion {
        index: usize,
        unsupported_index: usize,
    },
    /// `OmittedAtEndOfInput` on a keyframe declaration requires upstream true
    /// EOF.
    KeyframeOmittedAtEndOfInputRequiresUpstreamComplete {
        index: usize,
    },
    KeyframePlacementUnknownContext {
        index: usize,
    },
    KeyframeOutsidePlacementContextBody {
        index: usize,
    },
    KeyframeOwnerMustBeKeyframeContext {
        index: usize,
    },
    DeclarationOwnerMustNotBeKeyframesContext {
        index: usize,
    },
    DeclarationOwnerMustNotBeKeyframeContext {
        index: usize,
    },
    MissingKeyframesContextEvidence,
    MissingKeyframeContextEvidence,
    KeyframeContextRequiresKeyframesParent,
    KeyframeContextCannotHaveChildren,
    KeyframesContextRequiresRootOrGroupOnlyAncestry,
    KeyframesContextHasInvalidParent {
        index: usize,
    },
    KeyframesContextHasInvalidChild {
        index: usize,
    },
    KeyframeParentMustBeKeyframesContext {
        index: usize,
    },
    KeyframeContextHasChild {
        index: usize,
    },
    KeyframesContextEvidenceMismatch {
        index: usize,
    },
    KeyframeContextEvidenceMismatch {
        index: usize,
    },
    UnqualifiedKeyframeBlockUnknownOwnerContext {
        index: usize,
    },
    UnqualifiedKeyframeBlockOutsideOwnerBody {
        index: usize,
    },
    UnqualifiedKeyframeBlockOwnerMustBeKeyframesContext {
        index: usize,
    },
    UnqualifiedKeyframeBlockOutsideKeyframes,
    KeyframeDeclarationOutsideKeyframeContext,
    UnreachableKeyframeNestedRuleTrigger,

    /// #170's page/page-margin-block malformed-item scan structurally
    /// cannot produce a nested-rule trigger, mirroring
    /// [`Self::UnreachableDescriptorNestedRuleTrigger`].
    UnreachablePageNestedRuleTrigger,
    /// Mirrors [`Self::UnreachablePageNestedRuleTrigger`] for a
    /// `PageMarginRuleBlock` body.
    UnreachablePageMarginNestedRuleTrigger,
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
    descriptor_occurrences: &[CssDescriptorOccurrence],
    page_occurrences: &[CssPageDeclarationOccurrence],
    page_margin_occurrences: &[CssPageMarginDeclarationOccurrence],
    keyframe_occurrences: &[CssKeyframeDeclarationOccurrence],
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
    validate_descriptor_occurrences(
        expected_source,
        descriptor_occurrences,
        unsupported_regions,
        terminal_offset,
    )?;
    validate_descriptor_occurrence_lifecycle(upstream, descriptor_occurrences)?;
    validate_page_occurrences(
        expected_source,
        page_occurrences,
        unsupported_regions,
        terminal_offset,
    )?;
    validate_page_occurrence_lifecycle(upstream, page_occurrences)?;
    validate_page_margin_occurrences(
        expected_source,
        page_margin_occurrences,
        unsupported_regions,
        terminal_offset,
    )?;
    validate_page_margin_occurrence_lifecycle(upstream, page_margin_occurrences)?;
    validate_keyframe_occurrences(
        expected_source,
        keyframe_occurrences,
        unsupported_regions,
        terminal_offset,
    )?;
    validate_keyframe_occurrence_lifecycle(upstream, keyframe_occurrences)?;
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

    validate_declaration_placement(
        occurrences,
        descriptor_occurrences,
        page_occurrences,
        page_margin_occurrences,
        keyframe_occurrences,
        unsupported_regions,
        context_records,
    )?;

    validate_resource_counts(
        resources,
        occurrences.len()
            + descriptor_occurrences.len()
            + page_occurrences.len()
            + page_margin_occurrences.len()
            + keyframe_occurrences.len(),
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

        validate_nearest_qualified_ancestor(context_records, index, record)?;
        validate_page_margin_parent(context_records, index, record)?;
        validate_keyframes_context_shape(context_records, index, record)?;

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

/// Validates one context record's `nearest_qualified_ancestor` (#141/#168)
/// against the retained parent table: first that a `Some` reference is
/// well-formed (strictly earlier, and itself a `QualifiedRuleBlock`), then
/// that the value equals the one structurally derivable from the parent's
/// own kind and nearest-ancestor evidence. Never trusts producer-provided
/// ancestry without proving it here.
fn validate_nearest_qualified_ancestor(
    context_records: &[CssParserContextRecord],
    index: usize,
    record: &CssParserContextRecord,
) -> Result<(), CssParserRunError> {
    if let Some(ancestor_id) = record.nearest_qualified_ancestor() {
        let ancestor_index = ancestor_id.index();
        let valid_reference = ancestor_index < index
            && matches!(
                context_records[ancestor_index].kind(),
                CssParserContextKind::QualifiedRuleBlock
            );
        if !valid_reference {
            return invariant(
                CssParserInvariantViolation::NearestQualifiedAncestorInvalidReference { index },
            );
        }
    }

    let expected = match record.parent() {
        None => None,
        Some(parent_id) => {
            let parent_record = &context_records[parent_id.index()];
            match parent_record.kind() {
                CssParserContextKind::QualifiedRuleBlock => Some(parent_id),
                CssParserContextKind::GroupRuleBlock(_) => {
                    parent_record.nearest_qualified_ancestor()
                }
                CssParserContextKind::DescriptorRuleBlock(_) => {
                    return invariant(CssParserInvariantViolation::DescriptorContextHasChild {
                        index,
                    });
                }
                CssParserContextKind::PageRuleBlock => {
                    // #170: the only context family ever entered from a
                    // `PageRuleBlock` is `PageMarginRuleBlock`, and Page
                    // semantic qualification is never inferred from
                    // group-rule ancestry, so the expected value is always
                    // `None` here regardless of the child's own kind.
                    if !matches!(record.kind(), CssParserContextKind::PageMarginRuleBlock(_)) {
                        return invariant(
                            CssParserInvariantViolation::PageContextHasNonPageMarginChild { index },
                        );
                    }
                    None
                }
                CssParserContextKind::PageMarginRuleBlock(_) => {
                    return invariant(CssParserInvariantViolation::PageMarginContextHasChild {
                        index,
                    });
                }
                CssParserContextKind::KeyframesRuleBlock => {
                    if !matches!(record.kind(), CssParserContextKind::KeyframeBlock) {
                        return invariant(
                            CssParserInvariantViolation::KeyframesContextHasInvalidChild { index },
                        );
                    }
                    None
                }
                CssParserContextKind::KeyframeBlock => {
                    return invariant(CssParserInvariantViolation::KeyframeContextHasChild {
                        index,
                    });
                }
            }
        }
    };
    if record.nearest_qualified_ancestor() != expected {
        return invariant(CssParserInvariantViolation::NearestQualifiedAncestorMismatch { index });
    }

    Ok(())
}

/// Validates one context record's Page-family parent shape (#170),
/// independent of [`validate_nearest_qualified_ancestor`]'s own concern: a
/// `PageMarginRuleBlock` record's parent must exist and must itself be a
/// `PageRuleBlock`, proved directly from the retained parent table rather
/// than inferred transitively through nearest-qualified-ancestor agreement
/// (which alone cannot distinguish a missing/wrong-kind parent whenever both
/// sides happen to already carry `None`). Also proves a `PageMarginRuleBlock`
/// record never carries `page_selector_list` evidence.
fn validate_page_margin_parent(
    context_records: &[CssParserContextRecord],
    index: usize,
    record: &CssParserContextRecord,
) -> Result<(), CssParserRunError> {
    if !matches!(record.kind(), CssParserContextKind::PageMarginRuleBlock(_)) {
        return Ok(());
    }
    if record.page_selector_list().is_some() {
        return invariant(
            CssParserInvariantViolation::PageMarginContextCarriesSelectorList { index },
        );
    }
    let valid_parent = record.parent().is_some_and(|parent_id| {
        matches!(
            context_records[parent_id.index()].kind(),
            CssParserContextKind::PageRuleBlock
        )
    });
    if !valid_parent {
        return invariant(CssParserInvariantViolation::PageMarginParentMustBePageContext { index });
    }
    Ok(())
}

/// Validates #171 context-family parent and evidence shape independently
/// from producer construction. `KeyframesRuleBlock` is root-owned or may be
/// parented only by a group-rule lineage with no qualified-rule ancestry;
/// `KeyframeBlock` must be a direct child of `KeyframesRuleBlock`.
fn validate_keyframes_context_shape(
    context_records: &[CssParserContextRecord],
    index: usize,
    record: &CssParserContextRecord,
) -> Result<(), CssParserRunError> {
    match record.kind() {
        CssParserContextKind::KeyframesRuleBlock => {
            if record.at_keyword().is_none()
                || record.keyframes_name().is_none()
                || record.keyframe_selector_list().is_some()
                || record.descriptor_property_name().is_some()
                || record.page_selector_list().is_some()
                || record.nearest_qualified_ancestor().is_some()
            {
                return invariant(
                    CssParserInvariantViolation::KeyframesContextEvidenceMismatch { index },
                );
            }
            if let Some(parent_id) = record.parent() {
                let parent = &context_records[parent_id.index()];
                if !matches!(parent.kind(), CssParserContextKind::GroupRuleBlock(_))
                    || parent.nearest_qualified_ancestor().is_some()
                {
                    return invariant(
                        CssParserInvariantViolation::KeyframesContextHasInvalidParent { index },
                    );
                }
            }
        }
        CssParserContextKind::KeyframeBlock => {
            if record.at_keyword().is_some()
                || record.keyframes_name().is_some()
                || record.keyframe_selector_list().is_none()
                || record.descriptor_property_name().is_some()
                || record.page_selector_list().is_some()
                || record.nearest_qualified_ancestor().is_some()
            {
                return invariant(
                    CssParserInvariantViolation::KeyframeContextEvidenceMismatch { index },
                );
            }
            let valid_parent = record.parent().is_some_and(|parent_id| {
                matches!(
                    context_records[parent_id.index()].kind(),
                    CssParserContextKind::KeyframesRuleBlock
                )
            });
            if !valid_parent {
                return invariant(
                    CssParserInvariantViolation::KeyframeParentMustBeKeyframesContext { index },
                );
            }
        }
        _ => {
            if record.keyframes_name().is_some() || record.keyframe_selector_list().is_some() {
                return invariant(
                    CssParserInvariantViolation::KeyframesContextEvidenceMismatch { index },
                );
            }
        }
    }
    Ok(())
}

/// One materialized direct block-content item's ordering data, keyed by its
/// owning context for [`validate_declaration_placement`]: a declaration
/// (carrying its `occurrences` index and run ordinal), a retained child
/// context (qualified-rule or supported group-rule, #168), a context-aware
/// nested unsupported at-rule (#168), or a #169 descriptor occurrence
/// (carrying its `descriptor_occurrences` index; no run ordinal, since
/// `<declaration-list>` admits no child rule whose interleaving requires the
/// style-rule declaration-run model).
enum DirectItem {
    Declaration {
        index: usize,
        run_ordinal: usize,
    },
    ChildContext,
    NestedUnsupportedAtRule,
    Descriptor {
        index: usize,
    },
    /// A #170 page-declaration occurrence, owned by a `PageRuleBlock`. Never
    /// participates in run validation (Amendment A): carries no run
    /// ordinal, mirroring [`Self::Descriptor`].
    PageDeclaration {
        index: usize,
    },
    /// A #170 page-margin-declaration occurrence, owned by a
    /// `PageMarginRuleBlock`. Never participates in run validation.
    PageMarginDeclaration {
        index: usize,
    },
    KeyframeDeclaration {
        index: usize,
    },
    UnqualifiedKeyframeBlock,
}

/// Reconciles declaration, descriptor-occurrence, and nested-unsupported-
/// at-rule placement against retained context evidence (#167/#168/#169):
/// every occurrence's [`CssParserContextId`] refers to an existing retained
/// context, its complete range lies within that context's `body`, its owning
/// context's kind matches the occurrence's semantic meaning (an ordinary
/// declaration can never be owned by a `DescriptorRuleBlock`, and a
/// descriptor occurrence can never be owned by a `QualifiedRuleBlock`/
/// `GroupRuleBlock`), and -- per owning context -- the combined direct-item
/// sequence (declarations, descriptor occurrences, retained child contexts,
/// and nested unsupported at-rules, sharing one ordinal space) has gapless
/// strictly-increasing ordinals consistent with source order, and
/// declaration-run ordinals are zero-based, non-decreasing, shared by
/// consecutive declarations, and reset to the next value by an intervening
/// materialized child context, nested unsupported at-rule, or descriptor
/// occurrence.
fn validate_declaration_placement(
    occurrences: &[CssDeclarationOccurrence],
    descriptor_occurrences: &[CssDescriptorOccurrence],
    page_occurrences: &[CssPageDeclarationOccurrence],
    page_margin_occurrences: &[CssPageMarginDeclarationOccurrence],
    keyframe_occurrences: &[CssKeyframeDeclarationOccurrence],
    unsupported_regions: &[CssParserUnsupportedRegion],
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
        if matches!(context.kind(), CssParserContextKind::DescriptorRuleBlock(_)) {
            return invariant(
                CssParserInvariantViolation::DeclarationOwnerMustNotBeDescriptorContext { index },
            );
        }
        if matches!(context.kind(), CssParserContextKind::PageRuleBlock) {
            return invariant(
                CssParserInvariantViolation::DeclarationOwnerMustNotBePageContext { index },
            );
        }
        if matches!(context.kind(), CssParserContextKind::PageMarginRuleBlock(_)) {
            return invariant(
                CssParserInvariantViolation::DeclarationOwnerMustNotBePageMarginContext { index },
            );
        }
        if matches!(context.kind(), CssParserContextKind::KeyframesRuleBlock) {
            return invariant(
                CssParserInvariantViolation::DeclarationOwnerMustNotBeKeyframesContext { index },
            );
        }
        if matches!(context.kind(), CssParserContextKind::KeyframeBlock) {
            return invariant(
                CssParserInvariantViolation::DeclarationOwnerMustNotBeKeyframeContext { index },
            );
        }
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

    for (index, occurrence) in descriptor_occurrences.iter().enumerate() {
        let placement = occurrence.placement();
        let context_index = placement.context_id().index();
        let context = context_records.get(context_index).ok_or(
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DescriptorPlacementUnknownContext { index },
            ),
        )?;
        if !matches!(context.kind(), CssParserContextKind::DescriptorRuleBlock(_)) {
            return invariant(
                CssParserInvariantViolation::DescriptorOwnerMustBeDescriptorContext { index },
            );
        }
        let complete = occurrence.complete().range();
        let body = context.body().range();
        if occurrence.complete().source_id() != context.body().source_id()
            || complete.start() < body.start()
            || complete.end() > body.end()
        {
            return invariant(
                CssParserInvariantViolation::DescriptorOutsidePlacementContextBody { index },
            );
        }

        items_by_context.entry(context_index).or_default().push((
            placement.item_ordinal().value(),
            complete.start(),
            complete.end(),
            DirectItem::Descriptor { index },
        ));
    }

    for (index, occurrence) in page_occurrences.iter().enumerate() {
        let placement = occurrence.placement();
        let context_index = placement.context_id().index();
        let context = context_records.get(context_index).ok_or(
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PagePlacementUnknownContext { index },
            ),
        )?;
        if !matches!(context.kind(), CssParserContextKind::PageRuleBlock) {
            return invariant(CssParserInvariantViolation::PageOwnerMustBePageContext { index });
        }
        let complete = occurrence.complete().range();
        let body = context.body().range();
        if occurrence.complete().source_id() != context.body().source_id()
            || complete.start() < body.start()
            || complete.end() > body.end()
        {
            return invariant(
                CssParserInvariantViolation::PageOutsidePlacementContextBody { index },
            );
        }

        items_by_context.entry(context_index).or_default().push((
            placement.item_ordinal().value(),
            complete.start(),
            complete.end(),
            DirectItem::PageDeclaration { index },
        ));
    }

    for (index, occurrence) in page_margin_occurrences.iter().enumerate() {
        let placement = occurrence.placement();
        let context_index = placement.context_id().index();
        let context = context_records.get(context_index).ok_or(
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginPlacementUnknownContext { index },
            ),
        )?;
        if !matches!(context.kind(), CssParserContextKind::PageMarginRuleBlock(_)) {
            return invariant(
                CssParserInvariantViolation::PageMarginOwnerMustBePageMarginContext { index },
            );
        }
        let complete = occurrence.complete().range();
        let body = context.body().range();
        if occurrence.complete().source_id() != context.body().source_id()
            || complete.start() < body.start()
            || complete.end() > body.end()
        {
            return invariant(
                CssParserInvariantViolation::PageMarginOutsidePlacementContextBody { index },
            );
        }

        items_by_context.entry(context_index).or_default().push((
            placement.item_ordinal().value(),
            complete.start(),
            complete.end(),
            DirectItem::PageMarginDeclaration { index },
        ));
    }

    for (index, occurrence) in keyframe_occurrences.iter().enumerate() {
        let placement = occurrence.placement();
        let context_index = placement.context_id().index();
        let context = context_records.get(context_index).ok_or(
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::KeyframePlacementUnknownContext { index },
            ),
        )?;
        if !matches!(context.kind(), CssParserContextKind::KeyframeBlock) {
            return invariant(
                CssParserInvariantViolation::KeyframeOwnerMustBeKeyframeContext { index },
            );
        }
        let complete = occurrence.complete().range();
        let body = context.body().range();
        if occurrence.complete().source_id() != context.body().source_id()
            || complete.start() < body.start()
            || complete.end() > body.end()
        {
            return invariant(
                CssParserInvariantViolation::KeyframeOutsidePlacementContextBody { index },
            );
        }
        items_by_context.entry(context_index).or_default().push((
            placement.item_ordinal().value(),
            complete.start(),
            complete.end(),
            DirectItem::KeyframeDeclaration { index },
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

    for (index, region) in unsupported_regions.iter().enumerate() {
        if let CssParserUnsupportedRegion::NestedAtRule {
            complete,
            context_id,
            item_ordinal,
            ..
        } = region
        {
            let context_index = context_id.index();
            let context = context_records.get(context_index).ok_or(
                CssParserRunError::InternalInvariantFailure(
                    CssParserInvariantViolation::NestedAtRuleUnknownOwnerContext { index },
                ),
            )?;
            let range = complete.range();
            let body = context.body().range();
            if complete.source_id() != context.body().source_id()
                || range.start() < body.start()
                || range.end() > body.end()
            {
                return invariant(CssParserInvariantViolation::NestedAtRuleOutsideOwnerBody {
                    index,
                });
            }
            items_by_context.entry(context_index).or_default().push((
                item_ordinal.value(),
                range.start(),
                range.end(),
                DirectItem::NestedUnsupportedAtRule,
            ));
        }
        if let CssParserUnsupportedRegion::UnqualifiedKeyframeBlock {
            complete,
            context_id,
            item_ordinal,
        } = region
        {
            let context_index = context_id.index();
            let context = context_records.get(context_index).ok_or(
                CssParserRunError::InternalInvariantFailure(
                    CssParserInvariantViolation::UnqualifiedKeyframeBlockUnknownOwnerContext {
                        index,
                    },
                ),
            )?;
            if !matches!(context.kind(), CssParserContextKind::KeyframesRuleBlock) {
                return invariant(
                    CssParserInvariantViolation::UnqualifiedKeyframeBlockOwnerMustBeKeyframesContext {
                        index,
                    },
                );
            }
            let range = complete.range();
            let body = context.body().range();
            if complete.source_id() != context.body().source_id()
                || range.start() < body.start()
                || range.end() > body.end()
            {
                return invariant(
                    CssParserInvariantViolation::UnqualifiedKeyframeBlockOutsideOwnerBody { index },
                );
            }
            items_by_context.entry(context_index).or_default().push((
                item_ordinal.value(),
                range.start(),
                range.end(),
                DirectItem::UnqualifiedKeyframeBlock,
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
                DirectItem::ChildContext
                | DirectItem::NestedUnsupportedAtRule
                | DirectItem::Descriptor { .. }
                | DirectItem::PageDeclaration { .. }
                | DirectItem::PageMarginDeclaration { .. }
                | DirectItem::KeyframeDeclaration { .. }
                | DirectItem::UnqualifiedKeyframeBlock => {
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

/// Mirrors [`validate_occurrences`] for #169 descriptor occurrences: exact
/// source identity, the terminal bound, strict non-overlapping source order,
/// and no overlap with a retained unsupported region. Descriptor and
/// ordinary occurrences are never cross-validated against each other here:
/// their owning context bodies are structurally disjoint (proved by
/// [`validate_contexts`]'s sibling-source-order check), so an overlap
/// between the two vectors is already unreachable once every other
/// invariant holds.
fn validate_descriptor_occurrences(
    expected_source: SourceId,
    descriptor_occurrences: &[CssDescriptorOccurrence],
    unsupported_regions: &[CssParserUnsupportedRegion],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_end: Option<usize> = None;
    for (index, occurrence) in descriptor_occurrences.iter().enumerate() {
        let complete = occurrence.complete();
        require_source(
            expected_source,
            complete,
            CssParserRunEvidenceRole::DescriptorOccurrence { index },
        )?;
        if complete.range().end() > terminal_offset {
            return invariant(
                CssParserInvariantViolation::DescriptorOccurrenceBeyondTerminal {
                    index,
                    end: complete.range().end(),
                    terminal: terminal_offset,
                },
            );
        }
        if let Some(previous_end) = previous_end
            && complete.range().start() < previous_end
        {
            return invariant(
                CssParserInvariantViolation::DescriptorOccurrenceOrderViolation { index },
            );
        }
        previous_end = Some(complete.range().end());

        for (unsupported_index, region) in unsupported_regions.iter().enumerate() {
            if ranges_overlap(complete.range(), region.region().range()) {
                return invariant(
                    CssParserInvariantViolation::DescriptorOccurrenceOverlapsUnsupportedRegion {
                        index,
                        unsupported_index,
                    },
                );
            }
        }
    }
    Ok(())
}

/// Mirrors [`validate_occurrence_lifecycle`] for #169 descriptor occurrences:
/// `OmittedAtEndOfInput` is valid termination evidence only when the
/// upstream tokenizer itself completed at true `EndOfInput`.
fn validate_descriptor_occurrence_lifecycle(
    upstream: &CssTokenizerRunResult,
    descriptor_occurrences: &[CssDescriptorOccurrence],
) -> Result<(), CssParserRunError> {
    if upstream_ended_at_true_eof(upstream) {
        return Ok(());
    }
    for (index, occurrence) in descriptor_occurrences.iter().enumerate() {
        if matches!(
            occurrence.termination(),
            CssDeclarationTermination::OmittedAtEndOfInput { .. }
        ) {
            return invariant(
                CssParserInvariantViolation::DescriptorOmittedAtEndOfInputRequiresUpstreamComplete {
                    index,
                },
            );
        }
    }
    Ok(())
}

/// Mirrors [`validate_descriptor_occurrences`] for #170 page-declaration
/// occurrences.
fn validate_page_occurrences(
    expected_source: SourceId,
    page_occurrences: &[CssPageDeclarationOccurrence],
    unsupported_regions: &[CssParserUnsupportedRegion],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_end: Option<usize> = None;
    for (index, occurrence) in page_occurrences.iter().enumerate() {
        let complete = occurrence.complete();
        require_source(
            expected_source,
            complete,
            CssParserRunEvidenceRole::PageOccurrence { index },
        )?;
        if complete.range().end() > terminal_offset {
            return invariant(CssParserInvariantViolation::PageOccurrenceBeyondTerminal {
                index,
                end: complete.range().end(),
                terminal: terminal_offset,
            });
        }
        if let Some(previous_end) = previous_end
            && complete.range().start() < previous_end
        {
            return invariant(CssParserInvariantViolation::PageOccurrenceOrderViolation { index });
        }
        previous_end = Some(complete.range().end());

        for (unsupported_index, region) in unsupported_regions.iter().enumerate() {
            if ranges_overlap(complete.range(), region.region().range()) {
                return invariant(
                    CssParserInvariantViolation::PageOccurrenceOverlapsUnsupportedRegion {
                        index,
                        unsupported_index,
                    },
                );
            }
        }
    }
    Ok(())
}

/// Mirrors [`validate_descriptor_occurrence_lifecycle`] for #170
/// page-declaration occurrences.
fn validate_page_occurrence_lifecycle(
    upstream: &CssTokenizerRunResult,
    page_occurrences: &[CssPageDeclarationOccurrence],
) -> Result<(), CssParserRunError> {
    if upstream_ended_at_true_eof(upstream) {
        return Ok(());
    }
    for (index, occurrence) in page_occurrences.iter().enumerate() {
        if matches!(
            occurrence.termination(),
            CssDeclarationTermination::OmittedAtEndOfInput { .. }
        ) {
            return invariant(
                CssParserInvariantViolation::PageOmittedAtEndOfInputRequiresUpstreamComplete {
                    index,
                },
            );
        }
    }
    Ok(())
}

/// Mirrors [`validate_descriptor_occurrences`] for #170
/// page-margin-declaration occurrences.
fn validate_page_margin_occurrences(
    expected_source: SourceId,
    page_margin_occurrences: &[CssPageMarginDeclarationOccurrence],
    unsupported_regions: &[CssParserUnsupportedRegion],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_end: Option<usize> = None;
    for (index, occurrence) in page_margin_occurrences.iter().enumerate() {
        let complete = occurrence.complete();
        require_source(
            expected_source,
            complete,
            CssParserRunEvidenceRole::PageMarginOccurrence { index },
        )?;
        if complete.range().end() > terminal_offset {
            return invariant(
                CssParserInvariantViolation::PageMarginOccurrenceBeyondTerminal {
                    index,
                    end: complete.range().end(),
                    terminal: terminal_offset,
                },
            );
        }
        if let Some(previous_end) = previous_end
            && complete.range().start() < previous_end
        {
            return invariant(
                CssParserInvariantViolation::PageMarginOccurrenceOrderViolation { index },
            );
        }
        previous_end = Some(complete.range().end());

        for (unsupported_index, region) in unsupported_regions.iter().enumerate() {
            if ranges_overlap(complete.range(), region.region().range()) {
                return invariant(
                    CssParserInvariantViolation::PageMarginOccurrenceOverlapsUnsupportedRegion {
                        index,
                        unsupported_index,
                    },
                );
            }
        }
    }
    Ok(())
}

/// Mirrors [`validate_descriptor_occurrence_lifecycle`] for #170
/// page-margin-declaration occurrences.
fn validate_page_margin_occurrence_lifecycle(
    upstream: &CssTokenizerRunResult,
    page_margin_occurrences: &[CssPageMarginDeclarationOccurrence],
) -> Result<(), CssParserRunError> {
    if upstream_ended_at_true_eof(upstream) {
        return Ok(());
    }
    for (index, occurrence) in page_margin_occurrences.iter().enumerate() {
        if matches!(
            occurrence.termination(),
            CssDeclarationTermination::OmittedAtEndOfInput { .. }
        ) {
            return invariant(
                CssParserInvariantViolation::PageMarginOmittedAtEndOfInputRequiresUpstreamComplete {
                    index,
                },
            );
        }
    }
    Ok(())
}

fn validate_keyframe_occurrences(
    expected_source: SourceId,
    keyframe_occurrences: &[CssKeyframeDeclarationOccurrence],
    unsupported_regions: &[CssParserUnsupportedRegion],
    terminal_offset: usize,
) -> Result<(), CssParserRunError> {
    let mut previous_end: Option<usize> = None;
    for (index, occurrence) in keyframe_occurrences.iter().enumerate() {
        let complete = occurrence.complete();
        require_source(
            expected_source,
            complete,
            CssParserRunEvidenceRole::KeyframeOccurrence { index },
        )?;
        if complete.range().end() > terminal_offset {
            return invariant(
                CssParserInvariantViolation::KeyframeOccurrenceBeyondTerminal {
                    index,
                    end: complete.range().end(),
                    terminal: terminal_offset,
                },
            );
        }
        if let Some(previous_end) = previous_end
            && complete.range().start() < previous_end
        {
            return invariant(
                CssParserInvariantViolation::KeyframeOccurrenceOrderViolation { index },
            );
        }
        previous_end = Some(complete.range().end());
        for (unsupported_index, region) in unsupported_regions.iter().enumerate() {
            if ranges_overlap(complete.range(), region.region().range()) {
                return invariant(
                    CssParserInvariantViolation::KeyframeOccurrenceOverlapsUnsupportedRegion {
                        index,
                        unsupported_index,
                    },
                );
            }
        }
    }
    Ok(())
}

fn validate_keyframe_occurrence_lifecycle(
    upstream: &CssTokenizerRunResult,
    keyframe_occurrences: &[CssKeyframeDeclarationOccurrence],
) -> Result<(), CssParserRunError> {
    if upstream_ended_at_true_eof(upstream) {
        return Ok(());
    }
    for (index, occurrence) in keyframe_occurrences.iter().enumerate() {
        if matches!(
            occurrence.termination(),
            CssDeclarationTermination::OmittedAtEndOfInput { .. }
        ) {
            return invariant(
                CssParserInvariantViolation::KeyframeOmittedAtEndOfInputRequiresUpstreamComplete {
                    index,
                },
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
        CssDescriptorPlacement, CssPageDeclarationPlacement, CssPageMarginDeclarationPlacement,
    };
    use crate::css::parser::context::{
        CssParserContextId, CssParserDescriptorRuleKind, CssParserDirectItemOrdinal,
        CssParserPageMarginRuleKind,
    };
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
            Vec::new(),
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
            Vec::new(),
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
            Vec::new(),
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
            Vec::new(),
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
            Vec::new(),
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
        CssParserContextRecord::new_qualified_rule_block(
            text,
            CssParserContextId::new(id),
            None,
            CssParserDirectItemOrdinal::new(item_ordinal),
            None,
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
        let outer = CssParserContextRecord::new_qualified_rule_block(
            text,
            parent_id,
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap();
        let child_b = CssParserContextRecord::new_qualified_rule_block(
            text,
            CssParserContextId::new(1),
            Some(parent_id),
            CssParserDirectItemOrdinal::new(first_ordinal),
            Some(parent_id),
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 5).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(5, 6).unwrap(),
            },
        )
        .unwrap();
        let child_c = CssParserContextRecord::new_qualified_rule_block(
            text,
            CssParserContextId::new(2),
            Some(parent_id),
            CssParserDirectItemOrdinal::new(second_ordinal),
            Some(parent_id),
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
        let context = CssParserContextRecord::new_qualified_rule_block(
            &text,
            parent_id,
            Some(parent_id),
            CssParserDirectItemOrdinal::new(0),
            None,
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
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            parent_id,
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
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
        let escaping_child = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            Some(parent_id),
            CssParserDirectItemOrdinal::new(0),
            Some(parent_id),
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
        let first = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
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
        let second = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
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
        let second = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
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
        let second = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            None,
            CssParserDirectItemOrdinal::new(1),
            None,
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
        let second = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            None,
            // Ordinal 1 is presumed reserved for a future declaration item
            // between the two top-level rules; the gap is valid.
            CssParserDirectItemOrdinal::new(2),
            None,
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
        let context = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
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
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, len).unwrap(),
            CssParserContextTermination::UpstreamTokenizerIncomplete {
                terminal: text.anchor(len, len).unwrap(),
            },
        )
        .unwrap();
        let inner = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            Some(CssParserContextId::new(0)),
            CssParserDirectItemOrdinal::new(0),
            Some(CssParserContextId::new(0)),
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

    // -------------------------------------------------------------------
    // #168 group-rule ancestry / nested-unsupported-at-rule corruption
    // matrix.
    // -------------------------------------------------------------------

    use crate::css::parser::context::CssParserGroupRuleKind;
    use crate::css::parser::evidence::CssParserUnsupportedRegion;

    fn group_resources(
        context_records: usize,
        peak_context_depth: usize,
    ) -> CssParserResourceUsage {
        CssParserResourceUsage::new(1, 0, peak_context_depth, 0, 0, 0, 0, 0, context_records)
    }

    fn unsupported_resources(
        context_records: usize,
        peak_context_depth: usize,
        unsupported: usize,
    ) -> CssParserResourceUsage {
        CssParserResourceUsage::new(
            1,
            0,
            peak_context_depth,
            0,
            0,
            0,
            unsupported,
            0,
            context_records,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn media_group_child(
        text: &SourceText,
        id: usize,
        parent: usize,
        item_ordinal: usize,
        nearest_qualified_ancestor: Option<usize>,
        at_keyword: (usize, usize),
        block_opener: (usize, usize),
        body: (usize, usize),
        right_curly: (usize, usize),
    ) -> CssParserContextRecord {
        CssParserContextRecord::new_group_rule_block(
            text,
            CssParserContextId::new(id),
            Some(CssParserContextId::new(parent)),
            CssParserDirectItemOrdinal::new(item_ordinal),
            nearest_qualified_ancestor.map(CssParserContextId::new),
            CssParserGroupRuleKind::Media,
            text.anchor(at_keyword.0, at_keyword.1).unwrap(),
            "media",
            text.anchor(at_keyword.0, at_keyword.1).unwrap(),
            text.anchor(block_opener.0, block_opener.1).unwrap(),
            text.anchor(body.0, body.1).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(right_curly.0, right_curly.1).unwrap(),
            },
        )
        .unwrap()
    }

    /// `outer` for `"a{@media{}}"`: header `[0,1)`, opener `[1,2)`, body
    /// `[2,10)`, authored close `[10,11)`.
    fn outer_for_single_media_fixture(
        text: &SourceText,
        nearest_qualified_ancestor: Option<usize>,
    ) -> CssParserContextRecord {
        CssParserContextRecord::new_qualified_rule_block(
            text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            nearest_qualified_ancestor.map(CssParserContextId::new),
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap()
    }

    #[test]
    fn contract_only_nearest_qualified_ancestor_wrong_value_is_rejected() {
        let text = source(30_001, "a{@media{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = outer_for_single_media_fixture(&text, None);
        // Wrong: the media group's real parent (`outer`, a `QualifiedRuleBlock`)
        // means the correct nearest qualified ancestor is `Some(0)`, not `None`.
        let media = media_group_child(&text, 1, 0, 0, None, (2, 8), (8, 9), (9, 9), (9, 10));

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer, media],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            group_resources(2, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::NearestQualifiedAncestorMismatch { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_nearest_qualified_ancestor_pointing_to_group_is_rejected() {
        let text = source(30_002, "a{@media{@media{}}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 18).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(18, 19).unwrap(),
            },
        )
        .unwrap();
        let outer_media =
            media_group_child(&text, 1, 0, 0, Some(0), (2, 8), (8, 9), (9, 17), (17, 18));
        // Wrong: claims its nearest qualified ancestor is context 1, which is
        // itself a `GroupRuleBlock`, never a `QualifiedRuleBlock`.
        let inner_media = media_group_child(
            &text,
            2,
            1,
            0,
            Some(1),
            (9, 15),
            (15, 16),
            (16, 16),
            (16, 17),
        );

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer, outer_media, inner_media],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            group_resources(3, 3),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::NearestQualifiedAncestorInvalidReference { index: 2 }
            )
        );
    }

    #[test]
    fn contract_only_nearest_qualified_ancestor_nonexistent_id_is_rejected() {
        let text = source(30_003, "a{@media{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = outer_for_single_media_fixture(&text, None);
        // References a `ContextId` beyond the retained table entirely.
        let media = media_group_child(&text, 1, 0, 0, Some(5), (2, 8), (8, 9), (9, 9), (9, 10));

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer, media],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            group_resources(2, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::NearestQualifiedAncestorInvalidReference { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_nearest_qualified_ancestor_forward_id_is_rejected() {
        let text = source(30_004, "a{@media{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        // References its own later sibling rather than any earlier-retained
        // record.
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            Some(CssParserContextId::new(1)),
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap();
        let media = media_group_child(&text, 1, 0, 0, Some(0), (2, 8), (8, 9), (9, 9), (9, 10));

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer, media],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            group_resources(2, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::NearestQualifiedAncestorInvalidReference { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_nested_at_rule_unknown_owner_context_is_rejected() {
        let text = source(30_005, "a{@x{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let unsupported = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(2, 6).unwrap(),
            text.anchor(2, 4).unwrap(),
            CssParserContextId::new(5),
            CssParserDirectItemOrdinal::new(0),
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
            vec![unsupported],
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            unsupported_resources(0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::NestedAtRuleUnknownOwnerContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_nested_at_rule_outside_owner_body_is_rejected() {
        let text = source(30_006, "a{}@x{}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 2).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(2, 3).unwrap(),
            },
        )
        .unwrap();
        // `outer`'s body is `[2, 2)`; this unsupported region sits entirely
        // outside it.
        let unsupported = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(3, 7).unwrap(),
            text.anchor(3, 5).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
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
            vec![unsupported],
            Vec::new(),
            vec![outer],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            unsupported_resources(1, 1, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::NestedAtRuleOutsideOwnerBody { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_nested_at_rule_duplicate_ordinal_with_declaration_is_rejected() {
        let text = source(30_007, "a{p:v;@x{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap();
        let occurrence = CssDeclarationOccurrence::new(
            &text,
            text.anchor(2, 6).unwrap(),
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 5).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(5, 6).unwrap(),
            },
            CssDeclarationPlacement::new(
                CssParserContextId::new(0),
                CssParserDirectItemOrdinal::new(0),
                CssDeclarationRunOrdinal::new(0),
            ),
        )
        .unwrap();
        // Duplicate: claims the same item ordinal (0) as the declaration.
        let unsupported = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 8).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
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
            vec![unsupported],
            Vec::new(),
            vec![outer],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 1, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DirectItemDuplicateOrdinal { context_id: 0 }
            )
        );
    }

    #[test]
    fn contract_only_nested_at_rule_duplicate_ordinal_with_child_context_is_rejected() {
        let text = source(30_008, "a{b{}@x{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 9).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(9, 10).unwrap(),
            },
        )
        .unwrap();
        let child = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            Some(CssParserContextId::new(0)),
            CssParserDirectItemOrdinal::new(0),
            Some(CssParserContextId::new(0)),
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 4).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(4, 5).unwrap(),
            },
        )
        .unwrap();
        // Duplicate: claims the same item ordinal (0) as the child context.
        let unsupported = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(5, 9).unwrap(),
            text.anchor(5, 7).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
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
            vec![unsupported],
            Vec::new(),
            vec![outer, child],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            unsupported_resources(2, 2, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DirectItemDuplicateOrdinal { context_id: 0 }
            )
        );
    }

    #[test]
    fn contract_only_nested_at_rule_source_ordinal_order_mismatch_is_rejected() {
        let text = source(30_009, "a{@y{}@x{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap();
        // Ordinal 0 is the *later* source span; ordinal 1 is the *earlier*
        // one, so numeric ordinal order disagrees with source order.
        let first_by_ordinal = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 8).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
        )
        .unwrap();
        let second_by_ordinal = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(2, 6).unwrap(),
            text.anchor(2, 4).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(1),
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
            vec![second_by_ordinal, first_by_ordinal],
            Vec::new(),
            vec![outer],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            unsupported_resources(1, 1, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DirectItemOrderViolation { context_id: 0 }
            )
        );
    }

    // -- #169 descriptor-context result-level corruption tests --------------

    fn descriptor_placement(context_index: usize, item_ordinal: usize) -> CssDescriptorPlacement {
        CssDescriptorPlacement::new(
            CssParserContextId::new(context_index),
            CssParserDirectItemOrdinal::new(item_ordinal),
        )
    }

    #[test]
    fn contract_only_descriptor_occurrence_owner_must_be_descriptor_context_qualified_is_rejected()
    {
        let text = source(30_101, "a{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 6).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(6, 7).unwrap(),
            },
        )
        .unwrap();
        let descriptor = CssDescriptorOccurrence::new(
            &text,
            text.anchor(2, 6).unwrap(),
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 5).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(5, 6).unwrap(),
            },
            descriptor_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            vec![descriptor],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DescriptorOwnerMustBeDescriptorContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_descriptor_occurrence_owner_must_be_descriptor_context_group_is_rejected() {
        let text = source(30_102, "a{@media{p:v;}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 14).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(14, 15).unwrap(),
            },
        )
        .unwrap();
        let media = CssParserContextRecord::new_group_rule_block(
            &text,
            CssParserContextId::new(1),
            Some(CssParserContextId::new(0)),
            CssParserDirectItemOrdinal::new(0),
            Some(CssParserContextId::new(0)),
            CssParserGroupRuleKind::Media,
            text.anchor(2, 8).unwrap(),
            "media",
            text.anchor(2, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            text.anchor(9, 13).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(13, 14).unwrap(),
            },
        )
        .unwrap();
        let descriptor = CssDescriptorOccurrence::new(
            &text,
            text.anchor(9, 13).unwrap(),
            text.anchor(9, 10).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 12).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(12, 13).unwrap(),
            },
            descriptor_placement(1, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            vec![descriptor],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer, media],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 2, 1, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DescriptorOwnerMustBeDescriptorContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_declaration_owner_must_not_be_descriptor_context_is_rejected() {
        let text = source(30_103, "@font-face{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let descriptor_context = CssParserContextRecord::new_descriptor_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserDescriptorRuleKind::FontFace,
            text.anchor(0, 10).unwrap(),
            "font-face",
            None,
            None,
            text.anchor(0, 10).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 15).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(15, 16).unwrap(),
            },
        )
        .unwrap();
        let ordinary = CssDeclarationOccurrence::new(
            &text,
            text.anchor(11, 15).unwrap(),
            text.anchor(11, 12).unwrap(),
            text.anchor(12, 13).unwrap(),
            text.anchor(13, 14).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(14, 15).unwrap(),
            },
            CssDeclarationPlacement::new(
                CssParserContextId::new(0),
                CssParserDirectItemOrdinal::new(0),
                CssDeclarationRunOrdinal::new(0),
            ),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![ordinary],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![descriptor_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DeclarationOwnerMustNotBeDescriptorContext {
                    index: 0
                }
            )
        );
    }

    #[test]
    fn contract_only_descriptor_placement_unknown_context_is_rejected() {
        let text = source(30_104, "@font-face{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let descriptor = CssDescriptorOccurrence::new(
            &text,
            text.anchor(11, 15).unwrap(),
            text.anchor(11, 12).unwrap(),
            text.anchor(12, 13).unwrap(),
            text.anchor(13, 14).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(14, 15).unwrap(),
            },
            descriptor_placement(5, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            vec![descriptor],
            Vec::new(),
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
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DescriptorPlacementUnknownContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_descriptor_outside_placement_context_body_is_rejected() {
        let text = source(30_105, "a:b;@font-face{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let descriptor_context = CssParserContextRecord::new_descriptor_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserDescriptorRuleKind::FontFace,
            text.anchor(4, 14).unwrap(),
            "font-face",
            None,
            None,
            text.anchor(4, 14).unwrap(),
            text.anchor(14, 15).unwrap(),
            text.anchor(15, 19).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(19, 20).unwrap(),
            },
        )
        .unwrap();
        // Structurally valid declaration-shaped evidence, but its source
        // span (0, 4) lies entirely outside the descriptor context's own
        // body (15, 19) -- an evidence-placement corruption, not a
        // recognizable production output shape.
        let descriptor = CssDescriptorOccurrence::new(
            &text,
            text.anchor(0, 4).unwrap(),
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 3).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(3, 4).unwrap(),
            },
            descriptor_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            vec![descriptor],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![descriptor_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DescriptorOutsidePlacementContextBody { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_descriptor_direct_item_duplicate_ordinal_is_rejected() {
        let text = source(30_106, "@font-face{p:v;q:w;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let descriptor_context = CssParserContextRecord::new_descriptor_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserDescriptorRuleKind::FontFace,
            text.anchor(0, 10).unwrap(),
            "font-face",
            None,
            None,
            text.anchor(0, 10).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 19).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(19, 20).unwrap(),
            },
        )
        .unwrap();
        let first = CssDescriptorOccurrence::new(
            &text,
            text.anchor(11, 15).unwrap(),
            text.anchor(11, 12).unwrap(),
            text.anchor(12, 13).unwrap(),
            text.anchor(13, 14).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(14, 15).unwrap(),
            },
            descriptor_placement(0, 0),
        )
        .unwrap();
        // Duplicate: claims the same item ordinal (0) as `first`.
        let second = CssDescriptorOccurrence::new(
            &text,
            text.anchor(15, 19).unwrap(),
            text.anchor(15, 16).unwrap(),
            text.anchor(16, 17).unwrap(),
            text.anchor(17, 18).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(18, 19).unwrap(),
            },
            descriptor_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            vec![first, second],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![descriptor_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 2, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DirectItemDuplicateOrdinal { context_id: 0 }
            )
        );
    }

    #[test]
    fn contract_only_descriptor_item_ordinal_collides_with_nested_unsupported_ordinal_is_rejected()
    {
        let text = source(30_107, "@font-face{@x{}p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let descriptor_context = CssParserContextRecord::new_descriptor_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserDescriptorRuleKind::FontFace,
            text.anchor(0, 10).unwrap(),
            "font-face",
            None,
            None,
            text.anchor(0, 10).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 19).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(19, 20).unwrap(),
            },
        )
        .unwrap();
        let nested_unsupported = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(11, 15).unwrap(),
            text.anchor(11, 13).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
        )
        .unwrap();
        // Collides: claims the same item ordinal (0) as `nested_unsupported`,
        // instead of the next unused ordinal (1).
        let descriptor = CssDescriptorOccurrence::new(
            &text,
            text.anchor(15, 19).unwrap(),
            text.anchor(15, 16).unwrap(),
            text.anchor(16, 17).unwrap(),
            text.anchor(17, 18).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(18, 19).unwrap(),
            },
            descriptor_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            vec![descriptor],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![nested_unsupported],
            Vec::new(),
            vec![descriptor_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 1, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DirectItemDuplicateOrdinal { context_id: 0 }
            )
        );
    }

    #[test]
    fn contract_only_descriptor_source_order_disagrees_with_item_ordinal_is_rejected() {
        let text = source(30_108, "@font-face{p:v;q:w;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let descriptor_context = CssParserContextRecord::new_descriptor_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserDescriptorRuleKind::FontFace,
            text.anchor(0, 10).unwrap(),
            "font-face",
            None,
            None,
            text.anchor(0, 10).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 19).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(19, 20).unwrap(),
            },
        )
        .unwrap();
        // The retained `descriptor_occurrences` vector itself stays in
        // strict source order (`p:v;` then `q:w;`), satisfying
        // `validate_descriptor_occurrences`'s own flat-array order check;
        // the corruption is purely in the item-ordinal values assigned to
        // each: the source-earlier `p:v;` claims ordinal 1, and the
        // source-later `q:w;` claims ordinal 0, so numeric ordinal order
        // disagrees with source order once `validate_declaration_placement`
        // reconciles the owning context's direct-item ordinal space.
        let earlier_source_later_ordinal = CssDescriptorOccurrence::new(
            &text,
            text.anchor(11, 15).unwrap(),
            text.anchor(11, 12).unwrap(),
            text.anchor(12, 13).unwrap(),
            text.anchor(13, 14).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(14, 15).unwrap(),
            },
            descriptor_placement(0, 1),
        )
        .unwrap();
        let later_source_earlier_ordinal = CssDescriptorOccurrence::new(
            &text,
            text.anchor(15, 19).unwrap(),
            text.anchor(15, 16).unwrap(),
            text.anchor(16, 17).unwrap(),
            text.anchor(17, 18).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(18, 19).unwrap(),
            },
            descriptor_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            vec![earlier_source_later_ordinal, later_source_earlier_ordinal],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![descriptor_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 2, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DirectItemOrderViolation { context_id: 0 }
            )
        );
    }

    #[test]
    fn contract_only_descriptor_occurrence_foreign_source_id_is_rejected() {
        let text = source(30_109, "@font-face{p:v;}");
        let other = source(30_110, "@font-face{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let descriptor = CssDescriptorOccurrence::new(
            &other,
            other.anchor(11, 15).unwrap(),
            other.anchor(11, 12).unwrap(),
            other.anchor(12, 13).unwrap(),
            other.anchor(13, 14).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: other.anchor(14, 15).unwrap(),
            },
            descriptor_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            vec![descriptor],
            Vec::new(),
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
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 0, 0, 0),
        );

        assert!(matches!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::SourceIdentityMismatch {
                    role: CssParserRunEvidenceRole::DescriptorOccurrence { index: 0 },
                    ..
                }
            )
        ));
    }

    // -- #170 page/page-margin corruption tests (Final Diff Audit finding) --

    fn page_placement(context_index: usize, item_ordinal: usize) -> CssPageDeclarationPlacement {
        CssPageDeclarationPlacement::new(
            CssParserContextId::new(context_index),
            CssParserDirectItemOrdinal::new(item_ordinal),
        )
    }

    fn page_margin_placement(
        context_index: usize,
        item_ordinal: usize,
    ) -> CssPageMarginDeclarationPlacement {
        CssPageMarginDeclarationPlacement::new(
            CssParserContextId::new(context_index),
            CssParserDirectItemOrdinal::new(item_ordinal),
        )
    }

    /// A genuinely `Incomplete` upstream tokenizer result whose terminal
    /// sits at `text`'s true end, mirroring
    /// `contract_only_omitted_at_end_of_input_occurrence_requires_upstream_true_eof`'s
    /// own helper: `upstream_ended_at_true_eof` requires both `Complete`
    /// completion and `EndOfInput` termination, so this deliberately
    /// satisfies neither.
    fn incomplete_upstream_at_end(text: &SourceText) -> CssTokenizerRunResult {
        let len = text.as_str().len();
        let resource_limit =
            crate::css::tokenizer::resource::CssTokenizerResourceLimitEvidence::new(
                text,
                crate::css::tokenizer::resource::CssTokenizerResourceKind::AlgorithmSteps,
                1,
                2,
                text.anchor(len, len).unwrap(),
            )
            .unwrap();
        CssTokenizerRunResult::new(
            text,
            None,
            vec![CssLexicalItem::SemanticToken(
                CssToken::new(
                    text,
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
        .unwrap()
    }

    #[test]
    fn contract_only_page_occurrence_beyond_terminal_is_rejected() {
        let text = source(40_001, "@page{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let occurrence = CssPageDeclarationOccurrence::new(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(9, 10).unwrap(),
            },
            page_placement(0, 0),
        )
        .unwrap();
        // The parser's own terminal (9) sits before the occurrence's end
        // (10): a resource-limited stop reached mid-occurrence.
        let evidence = CssParserResourceLimitEvidence::new(
            &text,
            CssParserResourceKind::AlgorithmSteps,
            1,
            2,
            text.anchor(9, 9).unwrap(),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(9, 9).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::ParserResourceLimit(evidence),
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageOccurrenceBeyondTerminal {
                    index: 0,
                    end: 10,
                    terminal: 9,
                }
            )
        );
    }

    #[test]
    fn contract_only_page_occurrence_order_violation_is_rejected() {
        let text = source(40_002, "@page{p:v;q:w;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let first = CssPageDeclarationOccurrence::new(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(9, 10).unwrap(),
            },
            page_placement(0, 0),
        )
        .unwrap();
        let second = CssPageDeclarationOccurrence::new(
            &text,
            text.anchor(10, 14).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 12).unwrap(),
            text.anchor(12, 13).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(13, 14).unwrap(),
            },
            page_placement(0, 1),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            // Retained out of source order: `second` (start 10) precedes
            // `first` (start 6) in the vector.
            vec![second, first],
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
            CssParserResourceUsage::new(1, 0, 0, 2, 0, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageOccurrenceOrderViolation { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_page_occurrence_overlaps_unsupported_region_is_rejected() {
        let text = source(40_003, "@page{@x:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        // Structurally valid declaration-shaped evidence, but its span
        // fully overlaps a retained unsupported region below -- an
        // evidence-placement corruption, not a recognizable production
        // output shape (production never commits both for the same span).
        let occurrence = CssPageDeclarationOccurrence::new(
            &text,
            text.anchor(6, 11).unwrap(),
            text.anchor(6, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            text.anchor(9, 10).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(10, 11).unwrap(),
            },
            page_placement(0, 0),
        )
        .unwrap();
        let unsupported = CssParserUnsupportedRegion::new_top_level_at_rule(
            &text,
            text.anchor(6, 11).unwrap(),
            text.anchor(6, 8).unwrap(),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
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
                CssParserInvariantViolation::PageOccurrenceOverlapsUnsupportedRegion {
                    index: 0,
                    unsupported_index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_page_omitted_at_end_of_input_requires_upstream_complete_is_rejected() {
        let text = source(40_004, "@page{p:v");
        let len = text.as_str().len();
        let upstream = incomplete_upstream_at_end(&text);
        // Internally self-consistent (its EOF terminal really is an empty
        // anchor at the true end of the raw source), which is all
        // `CssPageDeclarationOccurrence::new` can check without tokenizer
        // lifecycle input.
        let occurrence = CssPageDeclarationOccurrence::new(
            &text,
            text.anchor(6, 9).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::OmittedAtEndOfInput {
                terminal: text.anchor(9, 9).unwrap(),
            },
            page_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
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
                CssParserInvariantViolation::PageOmittedAtEndOfInputRequiresUpstreamComplete {
                    index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_occurrence_beyond_terminal_is_rejected() {
        let text = source(40_005, "@page{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let occurrence = CssPageMarginDeclarationOccurrence::new(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(9, 10).unwrap(),
            },
            page_margin_placement(0, 0),
        )
        .unwrap();
        let evidence = CssParserResourceLimitEvidence::new(
            &text,
            CssParserResourceKind::AlgorithmSteps,
            1,
            2,
            text.anchor(9, 9).unwrap(),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(9, 9).unwrap(),
            CssParserExecutionCompletion::Incomplete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::ParserResourceLimit(evidence),
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginOccurrenceBeyondTerminal {
                    index: 0,
                    end: 10,
                    terminal: 9,
                }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_occurrence_order_violation_is_rejected() {
        let text = source(40_006, "@page{p:v;q:w;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let first = CssPageMarginDeclarationOccurrence::new(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(9, 10).unwrap(),
            },
            page_margin_placement(0, 0),
        )
        .unwrap();
        let second = CssPageMarginDeclarationOccurrence::new(
            &text,
            text.anchor(10, 14).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 12).unwrap(),
            text.anchor(12, 13).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(13, 14).unwrap(),
            },
            page_margin_placement(0, 1),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![second, first],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 2, 0, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginOccurrenceOrderViolation { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_occurrence_overlaps_unsupported_region_is_rejected() {
        let text = source(40_007, "@page{@x:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let occurrence = CssPageMarginDeclarationOccurrence::new(
            &text,
            text.anchor(6, 11).unwrap(),
            text.anchor(6, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            text.anchor(9, 10).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(10, 11).unwrap(),
            },
            page_margin_placement(0, 0),
        )
        .unwrap();
        let unsupported = CssParserUnsupportedRegion::new_top_level_at_rule(
            &text,
            text.anchor(6, 11).unwrap(),
            text.anchor(6, 8).unwrap(),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
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
                CssParserInvariantViolation::PageMarginOccurrenceOverlapsUnsupportedRegion {
                    index: 0,
                    unsupported_index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_omitted_at_end_of_input_requires_upstream_complete_is_rejected() {
        let text = source(40_008, "@page{p:v");
        let len = text.as_str().len();
        let upstream = incomplete_upstream_at_end(&text);
        let occurrence = CssPageMarginDeclarationOccurrence::new(
            &text,
            text.anchor(6, 9).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::OmittedAtEndOfInput {
                terminal: text.anchor(9, 9).unwrap(),
            },
            page_margin_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
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
                CssParserInvariantViolation::PageMarginOmittedAtEndOfInputRequiresUpstreamComplete {
                    index: 0,
                }
            )
        );
    }

    #[test]
    fn contract_only_page_placement_unknown_context_is_rejected() {
        let text = source(40_009, "@page{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let occurrence = CssPageDeclarationOccurrence::new(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(9, 10).unwrap(),
            },
            // References a `ContextId` that has no retained record at all.
            page_placement(5, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![occurrence],
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
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PagePlacementUnknownContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_page_outside_placement_context_body_is_rejected() {
        let text = source(40_010, "a:b;@page{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let page_context = CssParserContextRecord::new_page_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            text.anchor(4, 9).unwrap(),
            "page",
            None,
            text.anchor(4, 9).unwrap(),
            text.anchor(9, 10).unwrap(),
            text.anchor(10, 14).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(14, 15).unwrap(),
            },
        )
        .unwrap();
        // Structurally valid declaration-shaped evidence, but its source
        // span (0, 4) lies entirely outside the page context's own body
        // (10, 14).
        let occurrence = CssPageDeclarationOccurrence::new(
            &text,
            text.anchor(0, 4).unwrap(),
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 3).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(3, 4).unwrap(),
            },
            page_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![page_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageOutsidePlacementContextBody { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_page_owner_must_be_page_context_is_rejected() {
        let text = source(40_011, "a{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 6).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(6, 7).unwrap(),
            },
        )
        .unwrap();
        // Page-typed occurrence, but its owning context is an ordinary
        // `QualifiedRuleBlock`, never a `PageRuleBlock`.
        let occurrence = CssPageDeclarationOccurrence::new(
            &text,
            text.anchor(2, 6).unwrap(),
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 5).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(5, 6).unwrap(),
            },
            page_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageOwnerMustBePageContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_placement_unknown_context_is_rejected() {
        let text = source(40_012, "@top-center{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let occurrence = CssPageMarginDeclarationOccurrence::new(
            &text,
            text.anchor(12, 16).unwrap(),
            text.anchor(12, 13).unwrap(),
            text.anchor(13, 14).unwrap(),
            text.anchor(14, 15).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(15, 16).unwrap(),
            },
            // References a `ContextId` that has no retained record at all.
            page_margin_placement(5, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 0, 1, 0, 0, 0, 0, 0),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginPlacementUnknownContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_outside_placement_context_body_is_rejected() {
        let text = source(40_013, "a:b;@page{@top-center{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let page_context = CssParserContextRecord::new_page_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            text.anchor(4, 9).unwrap(),
            "page",
            None,
            text.anchor(4, 9).unwrap(),
            text.anchor(9, 10).unwrap(),
            text.anchor(10, 23).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(23, 24).unwrap(),
            },
        )
        .unwrap();
        let margin_context = CssParserContextRecord::new_page_margin_rule_block(
            &text,
            CssParserContextId::new(1),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserPageMarginRuleKind::TopCenter,
            text.anchor(10, 21).unwrap(),
            "top-center",
            text.anchor(10, 21).unwrap(),
            text.anchor(21, 22).unwrap(),
            text.anchor(22, 22).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(22, 23).unwrap(),
            },
        )
        .unwrap();
        // Structurally valid declaration-shaped evidence, but its source
        // span (0, 4) lies entirely outside the margin context's own empty
        // body (22, 22).
        let occurrence = CssPageMarginDeclarationOccurrence::new(
            &text,
            text.anchor(0, 4).unwrap(),
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 3).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(3, 4).unwrap(),
            },
            page_margin_placement(1, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![page_context, margin_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 2, 1, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginOutsidePlacementContextBody { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_owner_must_be_page_margin_context_is_rejected() {
        let text = source(40_014, "@page{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let page_context = CssParserContextRecord::new_page_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            text.anchor(0, 5).unwrap(),
            "page",
            None,
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap();
        // Page-margin-typed occurrence, but its owning context is the
        // `PageRuleBlock` itself, never a `PageMarginRuleBlock`.
        let occurrence = CssPageMarginDeclarationOccurrence::new(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(9, 10).unwrap(),
            },
            page_margin_placement(0, 0),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![occurrence],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![page_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginOwnerMustBePageMarginContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_declaration_owner_must_not_be_page_context_is_rejected() {
        let text = source(40_015, "@page{p:v;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let page_context = CssParserContextRecord::new_page_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            text.anchor(0, 5).unwrap(),
            "page",
            None,
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 10).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(10, 11).unwrap(),
            },
        )
        .unwrap();
        let ordinary = CssDeclarationOccurrence::new(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 9).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(9, 10).unwrap(),
            },
            CssDeclarationPlacement::new(
                CssParserContextId::new(0),
                CssParserDirectItemOrdinal::new(0),
                CssDeclarationRunOrdinal::new(0),
            ),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![ordinary],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![page_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 1, 0, 0, 0, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DeclarationOwnerMustNotBePageContext { index: 0 }
            )
        );
    }

    #[test]
    fn contract_only_declaration_owner_must_not_be_page_margin_context_is_rejected() {
        let text = source(40_016, "@page{@top-center{p:v;}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let page_context = CssParserContextRecord::new_page_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            text.anchor(0, 5).unwrap(),
            "page",
            None,
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 23).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(23, 24).unwrap(),
            },
        )
        .unwrap();
        let margin_context = CssParserContextRecord::new_page_margin_rule_block(
            &text,
            CssParserContextId::new(1),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserPageMarginRuleKind::TopCenter,
            text.anchor(6, 17).unwrap(),
            "top-center",
            text.anchor(6, 17).unwrap(),
            text.anchor(17, 18).unwrap(),
            text.anchor(18, 22).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(22, 23).unwrap(),
            },
        )
        .unwrap();
        let ordinary = CssDeclarationOccurrence::new(
            &text,
            text.anchor(18, 22).unwrap(),
            text.anchor(18, 19).unwrap(),
            text.anchor(19, 20).unwrap(),
            text.anchor(20, 21).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(21, 22).unwrap(),
            },
            CssDeclarationPlacement::new(
                CssParserContextId::new(1),
                CssParserDirectItemOrdinal::new(0),
                CssDeclarationRunOrdinal::new(0),
            ),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![ordinary],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![page_context, margin_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 2, 1, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DeclarationOwnerMustNotBePageMarginContext {
                    index: 0
                }
            )
        );
    }

    #[test]
    fn contract_only_page_context_has_non_page_margin_child_is_rejected() {
        let text = source(40_017, "@page{a{p:v;}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let page_context = CssParserContextRecord::new_page_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            text.anchor(0, 5).unwrap(),
            "page",
            None,
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 13).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(13, 14).unwrap(),
            },
        )
        .unwrap();
        // Wrong: an ordinary `QualifiedRuleBlock` retained as a direct
        // child of a `PageRuleBlock`. The only context family #170 ever
        // enters from a `PageRuleBlock` is `PageMarginRuleBlock`.
        let child = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(1),
            Some(CssParserContextId::new(0)),
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 8).unwrap(),
            text.anchor(8, 12).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(12, 13).unwrap(),
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
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![page_context, child],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 2, 0, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageContextHasNonPageMarginChild { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_context_has_child_is_rejected() {
        let text = source(40_018, "@page{@top-center{a{p:v;}}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let page_context = CssParserContextRecord::new_page_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            text.anchor(0, 5).unwrap(),
            "page",
            None,
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 26).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(26, 27).unwrap(),
            },
        )
        .unwrap();
        let margin_context = CssParserContextRecord::new_page_margin_rule_block(
            &text,
            CssParserContextId::new(1),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserPageMarginRuleKind::TopCenter,
            text.anchor(6, 17).unwrap(),
            "top-center",
            text.anchor(6, 17).unwrap(),
            text.anchor(17, 18).unwrap(),
            text.anchor(18, 25).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(25, 26).unwrap(),
            },
        )
        .unwrap();
        // Wrong: `PageMarginRuleBlock` never has children in #170.
        let child = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(2),
            Some(CssParserContextId::new(1)),
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(18, 19).unwrap(),
            text.anchor(19, 20).unwrap(),
            text.anchor(20, 24).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(24, 25).unwrap(),
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
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![page_context, margin_context, child],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 3, 0, 0, 0, 0, 0, 3),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginContextHasChild { index: 2 }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_parent_must_be_page_context_is_rejected() {
        let text = source(40_019, "@media{@top-center{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        // A root `GroupRuleBlock` is itself producer-unreachable (#168
        // requires a real qualified-rule parent), but is directly
        // constructible here to model an otherwise-plausible corrupted
        // parent whose own `nearest_qualified_ancestor` happens to be
        // `None` too -- proving `validate_page_margin_parent` catches what
        // `validate_nearest_qualified_ancestor` alone cannot distinguish
        // from a genuinely root-owned margin.
        let fake_parent = CssParserContextRecord::new_group_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            CssParserGroupRuleKind::Media,
            text.anchor(0, 6).unwrap(),
            "media",
            text.anchor(0, 6).unwrap(),
            text.anchor(6, 7).unwrap(),
            text.anchor(7, 20).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(20, 21).unwrap(),
            },
        )
        .unwrap();
        let margin_context = CssParserContextRecord::new_page_margin_rule_block(
            &text,
            CssParserContextId::new(1),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserPageMarginRuleKind::TopCenter,
            text.anchor(7, 18).unwrap(),
            "top-center",
            text.anchor(7, 18).unwrap(),
            text.anchor(18, 19).unwrap(),
            text.anchor(19, 19).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(19, 20).unwrap(),
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
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![fake_parent, margin_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 2, 0, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginParentMustBePageContext { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_page_margin_context_carries_selector_list_is_rejected() {
        let text = source(40_020, "@page{@top-center{}}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let page_context = CssParserContextRecord::new_page_rule_block(
            &text,
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            text.anchor(0, 5).unwrap(),
            "page",
            None,
            text.anchor(0, 5).unwrap(),
            text.anchor(5, 6).unwrap(),
            text.anchor(6, 19).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(19, 20).unwrap(),
            },
        )
        .unwrap();
        let valid_margin_context = CssParserContextRecord::new_page_margin_rule_block(
            &text,
            CssParserContextId::new(1),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(0),
            CssParserPageMarginRuleKind::TopCenter,
            text.anchor(6, 17).unwrap(),
            "top-center",
            text.anchor(6, 17).unwrap(),
            text.anchor(17, 18).unwrap(),
            text.anchor(18, 18).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(18, 19).unwrap(),
            },
        )
        .unwrap();
        // No production constructor can build this combination directly
        // (see `CssParserContextRecord::new_test_only_page_margin_rule_block_with_selector_list`'s
        // own doc comment); it is corrupted here from an otherwise-valid
        // margin record.
        let margin_context =
            CssParserContextRecord::new_test_only_page_margin_rule_block_with_selector_list(
                valid_margin_context,
                text.anchor(0, 5).unwrap(),
            );

        let result = CssParserRunResult::new(
            &text,
            upstream,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![page_context, margin_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 2, 0, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::PageMarginContextCarriesSelectorList { index: 1 }
            )
        );
    }

    #[test]
    fn contract_only_aggregate_declaration_occurrences_usage_mismatch_is_rejected() {
        let text = source(30_111, "a{p:v;}@font-face{q:w;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 6).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(6, 7).unwrap(),
            },
        )
        .unwrap();
        let descriptor_context = CssParserContextRecord::new_descriptor_rule_block(
            &text,
            CssParserContextId::new(1),
            CssParserDirectItemOrdinal::new(1),
            CssParserDescriptorRuleKind::FontFace,
            text.anchor(7, 17).unwrap(),
            "font-face",
            None,
            None,
            text.anchor(7, 17).unwrap(),
            text.anchor(17, 18).unwrap(),
            text.anchor(18, 22).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(22, 23).unwrap(),
            },
        )
        .unwrap();
        let ordinary = CssDeclarationOccurrence::new(
            &text,
            text.anchor(2, 6).unwrap(),
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 5).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(5, 6).unwrap(),
            },
            CssDeclarationPlacement::new(
                CssParserContextId::new(0),
                CssParserDirectItemOrdinal::new(0),
                CssDeclarationRunOrdinal::new(0),
            ),
        )
        .unwrap();
        let descriptor = CssDescriptorOccurrence::new(
            &text,
            text.anchor(18, 22).unwrap(),
            text.anchor(18, 19).unwrap(),
            text.anchor(19, 20).unwrap(),
            text.anchor(20, 21).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(21, 22).unwrap(),
            },
            descriptor_placement(1, 0),
        )
        .unwrap();

        // Actual aggregate usage is 2 (1 ordinary + 1 descriptor), but the
        // declared `DeclarationOccurrences` resource usage claims 5.
        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![ordinary],
            vec![descriptor],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![outer, descriptor_context],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::SupportedForSelectedQuestion,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 5, 0, 0, 0, 0, 2),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::ResourceUsageCountMismatch {
                    kind: CssParserResourceKind::DeclarationOccurrences,
                    expected: 2,
                    actual: 5,
                }
            )
        );
    }

    #[test]
    fn contract_only_run_boundary_corruption_around_nested_at_rule_is_rejected() {
        let text = source(30_010, "a{p:v;@x{}q:w;}");
        let upstream = complete_tokenizer_run(&text);
        let len = text.as_str().len();
        let outer = CssParserContextRecord::new_qualified_rule_block(
            &text,
            CssParserContextId::new(0),
            None,
            CssParserDirectItemOrdinal::new(0),
            None,
            text.anchor(0, 1).unwrap(),
            text.anchor(1, 2).unwrap(),
            text.anchor(2, 14).unwrap(),
            CssParserContextTermination::AuthoredRightCurly {
                right_curly: text.anchor(14, 15).unwrap(),
            },
        )
        .unwrap();
        let first = CssDeclarationOccurrence::new(
            &text,
            text.anchor(2, 6).unwrap(),
            text.anchor(2, 3).unwrap(),
            text.anchor(3, 4).unwrap(),
            text.anchor(4, 5).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(5, 6).unwrap(),
            },
            CssDeclarationPlacement::new(
                CssParserContextId::new(0),
                CssParserDirectItemOrdinal::new(0),
                CssDeclarationRunOrdinal::new(0),
            ),
        )
        .unwrap();
        let unsupported = CssParserUnsupportedRegion::new_nested_at_rule(
            &text,
            text.anchor(6, 10).unwrap(),
            text.anchor(6, 8).unwrap(),
            CssParserContextId::new(0),
            CssParserDirectItemOrdinal::new(1),
        )
        .unwrap();
        // Wrong: the nested at-rule at item 1 must close run 0, so this
        // trailing declaration must open run 1, not reuse run 0.
        let second = CssDeclarationOccurrence::new(
            &text,
            text.anchor(10, 14).unwrap(),
            text.anchor(10, 11).unwrap(),
            text.anchor(11, 12).unwrap(),
            text.anchor(12, 13).unwrap(),
            None,
            CssDeclarationTermination::AuthoredSemicolon {
                semicolon: text.anchor(13, 14).unwrap(),
            },
            CssDeclarationPlacement::new(
                CssParserContextId::new(0),
                CssParserDirectItemOrdinal::new(2),
                CssDeclarationRunOrdinal::new(0),
            ),
        )
        .unwrap();

        let result = CssParserRunResult::new(
            &text,
            upstream,
            vec![first, second],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            vec![unsupported],
            Vec::new(),
            vec![outer],
            text.anchor(len, len).unwrap(),
            CssParserExecutionCompletion::Complete,
            CssParserCoverage::ContainsUnsupportedContexts,
            CssParserTermination::EndOfTokenizerInput,
            CssParserResourceUsage::new(1, 0, 1, 2, 0, 0, 1, 0, 1),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::DeclarationRunOrdinalViolation { index: 1 }
            )
        );
    }
}
