//! Candidate-independent CSS descriptor-context evidence gold data model
//! (#169 validation phase).
//!
//! Structurally independent from production: fixtures are authored here from
//! the approved #169 architecture record (proposal comment `5236345421`),
//! never from production `CssParserContextRecord`, `CssParserDescriptorRuleKind`,
//! `CssDescriptorOccurrence`, a producer, an external parser, CSSOM, or
//! browser output. This module must never import those production types, and
//! production code must never import this module.
//!
//! Kept a wholly separate layer from `context_gold.rs`/`context_fixtures.rs`
//! (the fixed 16-fixture #166 inventory) and `group_context_fixtures.rs` (the
//! fixed 28-fixture #168 inventory): #169 descriptor contexts are always
//! stylesheet-root-only with no child contexts and no declaration-run
//! ordinal, so this model does not need -- and deliberately does not reuse --
//! the general recursive parent/ancestor machinery those fixed inventories
//! validate.

use super::gold::GoldRange;
use crate::{SourceId, SourceText};

fn anchor_ok(source: &SourceText, range: GoldRange) -> bool {
    source.anchor(range.start, range.end).is_ok()
}

fn fragment_is(source: &SourceText, range: GoldRange, expected: &str) -> bool {
    source
        .anchor(range.start, range.end)
        .is_ok_and(|anchor| anchor.fragment() == expected)
}

fn fragment_starts_with(source: &SourceText, range: GoldRange, expected: &str) -> bool {
    source
        .anchor(range.start, range.end)
        .is_ok_and(|anchor| anchor.fragment().starts_with(expected))
}

fn ranges_overlap(left: GoldRange, right: GoldRange) -> bool {
    left.start < right.end && right.start < left.end
}

/// Independent finite descriptor-rule-kind vocabulary (#169). Mirrors only
/// the approved two-member registry; not the production enum itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DescriptorGoldRuleKind {
    FontFace,
    Property,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DescriptorGoldTermination {
    AuthoredRightCurly(GoldRange),
    EndOfInput(GoldRange),
    UpstreamTokenizerIncomplete(GoldRange),
    ParserResourceLimit(GoldRange),
}

impl DescriptorGoldTermination {
    const fn evidence_range(self) -> GoldRange {
        match self {
            Self::AuthoredRightCurly(range)
            | Self::EndOfInput(range)
            | Self::UpstreamTokenizerIncomplete(range)
            | Self::ParserResourceLimit(range) => range,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DescriptorGoldPriority {
    pub(super) complete: GoldRange,
    pub(super) bang: GoldRange,
    pub(super) important_ident: GoldRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DescriptorGoldOccurrenceTermination {
    AuthoredSemicolon(GoldRange),
    OmittedBeforeRightCurly(GoldRange),
    OmittedAtEndOfInput(GoldRange),
}

/// One expected `Ident : value` syntax occurrence, shared shape between an
/// ordinary companion declaration ([`DescriptorGoldQualifiedCompanion`]) and
/// a #169 descriptor occurrence ([`DescriptorGoldContext`]): the two are
/// still validated as semantically distinct claims by
/// [`validate_descriptor_context`]/[`validate_qualified_companion`] and by
/// `descriptor_candidate.rs`'s production comparison, never interchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DescriptorGoldOccurrence {
    pub(super) item_ordinal: usize,
    pub(super) complete: GoldRange,
    pub(super) name: GoldRange,
    pub(super) colon: GoldRange,
    pub(super) value: GoldRange,
    pub(super) priority: Option<DescriptorGoldPriority>,
    pub(super) termination: DescriptorGoldOccurrenceTermination,
}

/// One expected explicit unsupported nested at-rule direct item inside a
/// descriptor context's body (#169: `<declaration-list>` automatically
/// excludes at-rules, so this is the only nested-at-rule outcome possible
/// there -- never a child `GroupRuleBlock`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DescriptorGoldNestedUnsupportedAtRule {
    pub(super) item_ordinal: usize,
    pub(super) complete: GoldRange,
    pub(super) at_keyword: GoldRange,
}

/// One independent `DescriptorRuleBlock` context record: stylesheet-root-only
/// (#169), so it carries a `root_item_ordinal` instead of a
/// parent/nearest-ancestor pair, and its direct-item ordinal space is shared
/// only between `occurrences` and `nested_unsupported` (never a child
/// context, since #169 never enters one from a descriptor context).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DescriptorGoldContext {
    pub(super) id: usize,
    pub(super) root_item_ordinal: usize,
    pub(super) kind: DescriptorGoldRuleKind,
    pub(super) at_keyword: GoldRange,
    /// The exact authored custom-property-name anchor; `Some` only for
    /// `Property`, always `None` for `FontFace` (#169).
    pub(super) property_name: Option<GoldRange>,
    pub(super) header: GoldRange,
    pub(super) block_opener: GoldRange,
    pub(super) body: GoldRange,
    pub(super) termination: DescriptorGoldTermination,
    pub(super) occurrences: Vec<DescriptorGoldOccurrence>,
    pub(super) nested_unsupported: Vec<DescriptorGoldNestedUnsupportedAtRule>,
}

impl DescriptorGoldContext {
    fn extent_end(&self) -> usize {
        self.termination.evidence_range().end
    }
}

/// A minimal independently-authored ordinary `QualifiedRuleBlock` companion
/// context, used only to prove #169 categories 5/23 (identical spelling
/// yields semantically distinct occurrence types; custom-property brace
/// values outside a descriptor context remain ordinary declaration
/// behavior). Not a general qualified-rule gold model -- that is the fixed
/// #166/#167 16-fixture inventory's own responsibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DescriptorGoldQualifiedCompanion {
    pub(super) id: usize,
    pub(super) root_item_ordinal: usize,
    pub(super) header: GoldRange,
    pub(super) block_opener: GoldRange,
    pub(super) body: GoldRange,
    pub(super) termination: DescriptorGoldTermination,
    pub(super) declarations: Vec<DescriptorGoldOccurrence>,
}

impl DescriptorGoldQualifiedCompanion {
    fn extent_end(&self) -> usize {
        self.termination.evidence_range().end
    }
}

/// One expected root-level (stylesheet-root direct child) item, sharing one
/// root-ordinal space (#169 architecture item 10): a supported descriptor
/// context, an ordinary qualified-rule companion context, or a structurally
/// consumed unsupported top-level at-rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DescriptorGoldRoot {
    Descriptor(DescriptorGoldContext),
    Qualified(DescriptorGoldQualifiedCompanion),
    UnsupportedAtRule {
        root_item_ordinal: usize,
        complete: GoldRange,
        at_keyword: GoldRange,
    },
}

impl DescriptorGoldRoot {
    const fn root_item_ordinal(&self) -> usize {
        match self {
            Self::Descriptor(context) => context.root_item_ordinal,
            Self::Qualified(companion) => companion.root_item_ordinal,
            Self::UnsupportedAtRule {
                root_item_ordinal, ..
            } => *root_item_ordinal,
        }
    }

    fn span(&self) -> (usize, usize) {
        match self {
            Self::Descriptor(context) => (context.header.start, context.extent_end()),
            Self::Qualified(companion) => (companion.header.start, companion.extent_end()),
            Self::UnsupportedAtRule { complete, .. } => (complete.start, complete.end),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DescriptorGoldResourceKind {
    AlgorithmSteps,
    PeakContextDepth,
    ContextRecords,
    DeclarationOccurrences,
    UnsupportedRegions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DescriptorGoldResourceExpectation {
    pub(super) kind: DescriptorGoldResourceKind,
    pub(super) limit: usize,
    pub(super) attempted: usize,
}

/// The #169 architecture category (proposal comment `5236345421`, section
/// 15) this fixture proves. One fixture may honestly prove more than one
/// category; every category 1-24 required by the validation-phase mandate
/// must be reachable from at least one fixture's `categories` list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum DescriptorGoldCategory {
    FontFaceMultipleDescriptors = 1,
    PropertyMultipleDescriptors = 2,
    CaseInsensitiveAtKeyword = 3,
    EscapedCustomPropertyName = 4,
    SameSpellingDistinctOccurrenceTypes = 5,
    DuplicateSpellingDistinctContexts = 6,
    PropertyColorDoesNotQualify = 7,
    PropertyReservedDoubleHyphenDoesNotQualify = 8,
    PropertyMultiNameRemainsUnsupported = 9,
    FontFaceNonEmptyPreludeRemainsUnsupported = 10,
    UnknownDescriptorBearingAtRuleUnsupported = 11,
    MalformedDescriptorItemRecovers = 12,
    QualifiedRuleShapedFragmentNotChildContext = 13,
    NestedAtKeywordSingleUnsupportedItem = 14,
    TrueEofWhileDescriptorActive = 15,
    UpstreamIncompleteWhileDescriptorActive = 16,
    GenericParserResourceStopAfterEntry = 17,
    PeakContextDepthRefusalAtEntry = 18,
    ContextRecordsRefusalAtEntry = 19,
    DeclarationOccurrencesAggregateLimit = 20,
    UnsupportedRegionsRefusalInsideDescriptor = 21,
    DeterministicRepeatAndCrossSourceId = 22,
    CustomPropertyBraceValueOutsideDescriptorUnaffected = 23,
    QualifiedGroupBehaviorRemainsGreen = 24,
}

pub(super) const ALL_CATEGORIES: [DescriptorGoldCategory; 24] = [
    DescriptorGoldCategory::FontFaceMultipleDescriptors,
    DescriptorGoldCategory::PropertyMultipleDescriptors,
    DescriptorGoldCategory::CaseInsensitiveAtKeyword,
    DescriptorGoldCategory::EscapedCustomPropertyName,
    DescriptorGoldCategory::SameSpellingDistinctOccurrenceTypes,
    DescriptorGoldCategory::DuplicateSpellingDistinctContexts,
    DescriptorGoldCategory::PropertyColorDoesNotQualify,
    DescriptorGoldCategory::PropertyReservedDoubleHyphenDoesNotQualify,
    DescriptorGoldCategory::PropertyMultiNameRemainsUnsupported,
    DescriptorGoldCategory::FontFaceNonEmptyPreludeRemainsUnsupported,
    DescriptorGoldCategory::UnknownDescriptorBearingAtRuleUnsupported,
    DescriptorGoldCategory::MalformedDescriptorItemRecovers,
    DescriptorGoldCategory::QualifiedRuleShapedFragmentNotChildContext,
    DescriptorGoldCategory::NestedAtKeywordSingleUnsupportedItem,
    DescriptorGoldCategory::TrueEofWhileDescriptorActive,
    DescriptorGoldCategory::UpstreamIncompleteWhileDescriptorActive,
    DescriptorGoldCategory::GenericParserResourceStopAfterEntry,
    DescriptorGoldCategory::PeakContextDepthRefusalAtEntry,
    DescriptorGoldCategory::ContextRecordsRefusalAtEntry,
    DescriptorGoldCategory::DeclarationOccurrencesAggregateLimit,
    DescriptorGoldCategory::UnsupportedRegionsRefusalInsideDescriptor,
    DescriptorGoldCategory::DeterministicRepeatAndCrossSourceId,
    DescriptorGoldCategory::CustomPropertyBraceValueOutsideDescriptorUnaffected,
    DescriptorGoldCategory::QualifiedGroupBehaviorRemainsGreen,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DescriptorGoldFixture {
    pub(super) id: &'static str,
    pub(super) categories: &'static [DescriptorGoldCategory],
    pub(super) source_id: u64,
    pub(super) source: &'static str,
    pub(super) byte_len: usize,
    pub(super) roots: Vec<DescriptorGoldRoot>,
    pub(super) resource_expectation: Option<DescriptorGoldResourceExpectation>,
    /// Expected `InvalidBlockItem` diagnostic locations (the only diagnostic
    /// code a malformed descriptor-block item can produce; shared, pre-#169
    /// vocabulary, never a descriptor-specific taxonomy per architecture
    /// item 14).
    pub(super) diagnostics: Vec<GoldRange>,
    pub(super) recovery: Vec<DescriptorGoldRecovery>,
}

/// Mirrors the shared, pre-#169 `MalformedBlockItem` recovery shape
/// (`parser_gold.rs`'s `ParserGoldRecovery`), authored independently here so
/// this module never imports that flat-parser gold layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DescriptorGoldRecovery {
    pub(super) region: GoldRange,
    pub(super) termination: DescriptorGoldRecoveryTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DescriptorGoldRecoveryTermination {
    AuthoredSemicolon(GoldRange),
    EnclosingBlockEnd(GoldRange),
    EndOfInput(GoldRange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DescriptorGoldValidationError {
    ByteLengthMismatch,
    EmptyCategories,
    InvalidRange,
    OpenerNotExact,
    AuthoredCloserNotExact,
    HeaderOpenerBoundaryMismatch,
    BodyOpenerBoundaryMismatch,
    TerminationBoundaryMismatch,
    TerminationMustBeEmpty,
    EndOfInputNotAtSourceEnd,
    IdIndexMismatch,
    AtKeywordShapeInvalid,
    PropertyNamePresenceMismatch,
    PropertyNameShapeInvalid,
    PropertyNameNotCustomPropertyShaped,
    RootOrdinalOrderViolation,
    RootSourceOrderViolation,
    OccurrenceOutsideBody,
    OccurrenceSyntaxInvalid,
    DirectItemDuplicateOrdinal,
    DirectItemOrderViolation,
    UnsupportedAtRuleShapeInvalid,
    ResourceExpectationAttemptDidNotExceedLimit,
    DiagnosticOrder,
    RecoveryOrder,
}

pub(super) fn validate_fixture(
    fixture: &DescriptorGoldFixture,
) -> Result<(), DescriptorGoldValidationError> {
    if fixture.source.len() != fixture.byte_len {
        return Err(DescriptorGoldValidationError::ByteLengthMismatch);
    }
    if fixture.categories.is_empty() {
        return Err(DescriptorGoldValidationError::EmptyCategories);
    }
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());

    let mut context_records = 0usize;
    let mut any_context = false;
    let mut unsupported_regions = 0usize;
    let mut declaration_occurrences = 0usize;

    // Every retained context's own zero-based index into the flat
    // `context_records()` vector, in the order production would append them
    // (root order == the order retained contexts commit).
    let mut next_context_id = 0usize;

    for root in &fixture.roots {
        match root {
            DescriptorGoldRoot::Descriptor(context) => {
                if context.id != next_context_id {
                    return Err(DescriptorGoldValidationError::IdIndexMismatch);
                }
                next_context_id += 1;
                context_records += 1;
                any_context = true;
                validate_descriptor_context(&source, context)?;
                unsupported_regions += context.nested_unsupported.len();
                declaration_occurrences += context.occurrences.len();
            }
            DescriptorGoldRoot::Qualified(companion) => {
                if companion.id != next_context_id {
                    return Err(DescriptorGoldValidationError::IdIndexMismatch);
                }
                next_context_id += 1;
                context_records += 1;
                any_context = true;
                validate_qualified_companion(&source, companion)?;
                declaration_occurrences += companion.declarations.len();
            }
            DescriptorGoldRoot::UnsupportedAtRule {
                complete,
                at_keyword,
                ..
            } => {
                unsupported_regions += 1;
                if !anchor_ok(&source, *complete) || complete.is_empty() {
                    return Err(DescriptorGoldValidationError::UnsupportedAtRuleShapeInvalid);
                }
                if !anchor_ok(&source, *at_keyword) || at_keyword.is_empty() {
                    return Err(DescriptorGoldValidationError::UnsupportedAtRuleShapeInvalid);
                }
                if at_keyword.start != complete.start || !complete.contains(*at_keyword) {
                    return Err(DescriptorGoldValidationError::UnsupportedAtRuleShapeInvalid);
                }
                if !fragment_starts_with(&source, *at_keyword, "@") {
                    return Err(DescriptorGoldValidationError::UnsupportedAtRuleShapeInvalid);
                }
            }
        }
    }

    validate_root_ordinal_space(fixture)?;

    if let Some(expectation) = fixture.resource_expectation
        && expectation.attempted <= expectation.limit
    {
        return Err(DescriptorGoldValidationError::ResourceExpectationAttemptDidNotExceedLimit);
    }

    let mut previous: Option<(usize, usize)> = None;
    for diagnostic in &fixture.diagnostics {
        if !anchor_ok(&source, *diagnostic) {
            return Err(DescriptorGoldValidationError::InvalidRange);
        }
        let key = (diagnostic.start, diagnostic.end);
        if previous.is_some_and(|previous| previous > key) {
            return Err(DescriptorGoldValidationError::DiagnosticOrder);
        }
        previous = Some(key);
    }

    let mut previous: Option<(usize, usize)> = None;
    for recovery in &fixture.recovery {
        if !anchor_ok(&source, recovery.region) || recovery.region.is_empty() {
            return Err(DescriptorGoldValidationError::InvalidRange);
        }
        match recovery.termination {
            DescriptorGoldRecoveryTermination::AuthoredSemicolon(semicolon) => {
                if !anchor_ok(&source, semicolon) || semicolon.end != recovery.region.end {
                    return Err(DescriptorGoldValidationError::InvalidRange);
                }
                if !fragment_is(&source, semicolon, ";") {
                    return Err(DescriptorGoldValidationError::InvalidRange);
                }
            }
            DescriptorGoldRecoveryTermination::EnclosingBlockEnd(right_curly) => {
                if !anchor_ok(&source, right_curly) || right_curly.start != recovery.region.end {
                    return Err(DescriptorGoldValidationError::InvalidRange);
                }
                if !fragment_is(&source, right_curly, "}") {
                    return Err(DescriptorGoldValidationError::InvalidRange);
                }
            }
            DescriptorGoldRecoveryTermination::EndOfInput(terminal) => {
                if !anchor_ok(&source, terminal) || !terminal.is_empty() {
                    return Err(DescriptorGoldValidationError::InvalidRange);
                }
                if terminal.start != source.as_str().len() || terminal.start != recovery.region.end
                {
                    return Err(DescriptorGoldValidationError::InvalidRange);
                }
            }
        }
        let key = (recovery.region.start, recovery.region.end);
        if previous.is_some_and(|previous| previous > key) {
            return Err(DescriptorGoldValidationError::RecoveryOrder);
        }
        previous = Some(key);
    }

    let _ = context_records;
    let _ = any_context;
    let _ = unsupported_regions;
    let _ = declaration_occurrences;

    Ok(())
}

/// Root-level direct-item ordinal space: descriptor contexts, qualified
/// companions, and unsupported top-level at-rules share one gapless,
/// strictly-increasing, source-order-consistent ordinal space (#169
/// architecture item 10), mirroring `context_gold.rs`'s sibling reconciliation
/// but flattened to a single scope since #169 descriptor contexts are always
/// stylesheet-root direct children.
fn validate_root_ordinal_space(
    fixture: &DescriptorGoldFixture,
) -> Result<(), DescriptorGoldValidationError> {
    let mut items: Vec<(usize, usize, usize)> = fixture
        .roots
        .iter()
        .map(|root| {
            let (start, end) = root.span();
            (root.root_item_ordinal(), start, end)
        })
        .collect();
    items.sort_by_key(|(ordinal, ..)| *ordinal);

    let mut expected_next = 0usize;
    let mut previous_end: Option<usize> = None;
    for (ordinal, start, end) in items {
        if ordinal != expected_next {
            return Err(DescriptorGoldValidationError::RootOrdinalOrderViolation);
        }
        if let Some(previous_end) = previous_end
            && start < previous_end
        {
            return Err(DescriptorGoldValidationError::RootSourceOrderViolation);
        }
        previous_end = Some(end);
        expected_next = ordinal + 1;
    }
    Ok(())
}

fn validate_descriptor_context(
    source: &SourceText,
    context: &DescriptorGoldContext,
) -> Result<(), DescriptorGoldValidationError> {
    validate_shared_shape(
        source,
        context.header,
        context.block_opener,
        context.body,
        context.termination,
    )?;

    if !anchor_ok(source, context.at_keyword) || context.at_keyword.is_empty() {
        return Err(DescriptorGoldValidationError::AtKeywordShapeInvalid);
    }
    if context.at_keyword.start != context.header.start
        || !context.header.contains(context.at_keyword)
    {
        return Err(DescriptorGoldValidationError::AtKeywordShapeInvalid);
    }
    if !fragment_starts_with(source, context.at_keyword, "@") {
        return Err(DescriptorGoldValidationError::AtKeywordShapeInvalid);
    }

    match (context.kind, context.property_name) {
        (DescriptorGoldRuleKind::FontFace, None) => {}
        (DescriptorGoldRuleKind::Property, Some(property_name)) => {
            if !anchor_ok(source, property_name) || property_name.is_empty() {
                return Err(DescriptorGoldValidationError::PropertyNameShapeInvalid);
            }
            if property_name.start < context.at_keyword.end
                || !context.header.contains(property_name)
            {
                return Err(DescriptorGoldValidationError::PropertyNameShapeInvalid);
            }
            if !fragment_starts_with(source, property_name, "--") {
                return Err(DescriptorGoldValidationError::PropertyNameNotCustomPropertyShaped);
            }
            if fragment_is(source, property_name, "--") {
                return Err(DescriptorGoldValidationError::PropertyNameNotCustomPropertyShaped);
            }
        }
        _ => return Err(DescriptorGoldValidationError::PropertyNamePresenceMismatch),
    }

    let mut items: Vec<(usize, usize, usize, bool)> = Vec::new();
    for occurrence in &context.occurrences {
        validate_occurrence_syntax(source, occurrence, context.body)?;
        items.push((
            occurrence.item_ordinal,
            occurrence.complete.start,
            occurrence.complete.end,
            true,
        ));
        for unsupported in &context.nested_unsupported {
            if ranges_overlap(occurrence.complete, unsupported.complete) {
                return Err(DescriptorGoldValidationError::OccurrenceOutsideBody);
            }
        }
    }
    for unsupported in &context.nested_unsupported {
        if !anchor_ok(source, unsupported.complete) || unsupported.complete.is_empty() {
            return Err(DescriptorGoldValidationError::UnsupportedAtRuleShapeInvalid);
        }
        if !anchor_ok(source, unsupported.at_keyword) || unsupported.at_keyword.is_empty() {
            return Err(DescriptorGoldValidationError::UnsupportedAtRuleShapeInvalid);
        }
        if unsupported.at_keyword.start != unsupported.complete.start
            || !unsupported.complete.contains(unsupported.at_keyword)
        {
            return Err(DescriptorGoldValidationError::UnsupportedAtRuleShapeInvalid);
        }
        if !fragment_starts_with(source, unsupported.at_keyword, "@") {
            return Err(DescriptorGoldValidationError::UnsupportedAtRuleShapeInvalid);
        }
        if unsupported.complete.start < context.body.start
            || unsupported.complete.end > context.body.end
        {
            return Err(DescriptorGoldValidationError::OccurrenceOutsideBody);
        }
        items.push((
            unsupported.item_ordinal,
            unsupported.complete.start,
            unsupported.complete.end,
            false,
        ));
    }

    items.sort_by_key(|(ordinal, ..)| *ordinal);
    let mut expected_next = 0usize;
    let mut previous_ordinal: Option<usize> = None;
    let mut previous_end: Option<usize> = None;
    for (ordinal, start, end, _) in items {
        if ordinal != expected_next {
            if previous_ordinal == Some(ordinal) {
                return Err(DescriptorGoldValidationError::DirectItemDuplicateOrdinal);
            }
            return Err(DescriptorGoldValidationError::DirectItemOrderViolation);
        }
        if let Some(previous_end) = previous_end
            && start < previous_end
        {
            return Err(DescriptorGoldValidationError::DirectItemOrderViolation);
        }
        previous_ordinal = Some(ordinal);
        previous_end = Some(end);
        expected_next = ordinal + 1;
    }

    Ok(())
}

fn validate_qualified_companion(
    source: &SourceText,
    companion: &DescriptorGoldQualifiedCompanion,
) -> Result<(), DescriptorGoldValidationError> {
    validate_shared_shape(
        source,
        companion.header,
        companion.block_opener,
        companion.body,
        companion.termination,
    )?;
    let mut previous_end: Option<usize> = None;
    for (index, declaration) in companion.declarations.iter().enumerate() {
        validate_occurrence_syntax(source, declaration, companion.body)?;
        if declaration.item_ordinal != index {
            return Err(DescriptorGoldValidationError::DirectItemOrderViolation);
        }
        if let Some(previous_end) = previous_end
            && declaration.complete.start < previous_end
        {
            return Err(DescriptorGoldValidationError::DirectItemOrderViolation);
        }
        previous_end = Some(declaration.complete.end);
    }
    Ok(())
}

fn validate_shared_shape(
    source: &SourceText,
    header: GoldRange,
    block_opener: GoldRange,
    body: GoldRange,
    termination: DescriptorGoldTermination,
) -> Result<(), DescriptorGoldValidationError> {
    if !anchor_ok(source, header) {
        return Err(DescriptorGoldValidationError::InvalidRange);
    }
    if !anchor_ok(source, block_opener) || block_opener.is_empty() {
        return Err(DescriptorGoldValidationError::InvalidRange);
    }
    if !fragment_is(source, block_opener, "{") {
        return Err(DescriptorGoldValidationError::OpenerNotExact);
    }
    if header.end != block_opener.start {
        return Err(DescriptorGoldValidationError::HeaderOpenerBoundaryMismatch);
    }
    if !anchor_ok(source, body) {
        return Err(DescriptorGoldValidationError::InvalidRange);
    }
    if body.start != block_opener.end {
        return Err(DescriptorGoldValidationError::BodyOpenerBoundaryMismatch);
    }

    match termination {
        DescriptorGoldTermination::AuthoredRightCurly(right_curly) => {
            if !anchor_ok(source, right_curly) || right_curly.is_empty() {
                return Err(DescriptorGoldValidationError::InvalidRange);
            }
            if !fragment_is(source, right_curly, "}") {
                return Err(DescriptorGoldValidationError::AuthoredCloserNotExact);
            }
            if right_curly.start != body.end {
                return Err(DescriptorGoldValidationError::TerminationBoundaryMismatch);
            }
        }
        DescriptorGoldTermination::EndOfInput(terminal) => {
            if !anchor_ok(source, terminal) || !terminal.is_empty() {
                return Err(DescriptorGoldValidationError::TerminationMustBeEmpty);
            }
            if terminal.start != body.end {
                return Err(DescriptorGoldValidationError::TerminationBoundaryMismatch);
            }
            if terminal.start != source.as_str().len() {
                return Err(DescriptorGoldValidationError::EndOfInputNotAtSourceEnd);
            }
        }
        DescriptorGoldTermination::UpstreamTokenizerIncomplete(terminal)
        | DescriptorGoldTermination::ParserResourceLimit(terminal) => {
            if !anchor_ok(source, terminal) || !terminal.is_empty() {
                return Err(DescriptorGoldValidationError::TerminationMustBeEmpty);
            }
            if terminal.start != body.end {
                return Err(DescriptorGoldValidationError::TerminationBoundaryMismatch);
            }
        }
    }

    Ok(())
}

fn validate_occurrence_syntax(
    source: &SourceText,
    occurrence: &DescriptorGoldOccurrence,
    body: GoldRange,
) -> Result<(), DescriptorGoldValidationError> {
    let complete = occurrence.complete;
    let name = occurrence.name;
    let colon = occurrence.colon;
    let value = occurrence.value;

    if !anchor_ok(source, complete) || complete.is_empty() {
        return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if complete.start < body.start || complete.end > body.end {
        return Err(DescriptorGoldValidationError::OccurrenceOutsideBody);
    }
    if !anchor_ok(source, name) || name.is_empty() {
        return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if complete.start != name.start || !complete.contains(name) {
        return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if !anchor_ok(source, colon) || colon.is_empty() {
        return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if !complete.contains(colon) || name.end > colon.start {
        return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if !fragment_is(source, colon, ":") {
        return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if !anchor_ok(source, value) {
        return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if value.is_empty() {
        if value.start != colon.end {
            return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
        }
    } else if colon.end > value.start || !complete.contains(value) {
        return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
    }

    if let Some(priority) = occurrence.priority {
        if !anchor_ok(source, priority.complete)
            || !anchor_ok(source, priority.bang)
            || !anchor_ok(source, priority.important_ident)
            || priority.complete.is_empty()
            || priority.bang.is_empty()
            || priority.important_ident.is_empty()
        {
            return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
        }
        if priority.complete.start != priority.bang.start
            || priority.complete.end != priority.important_ident.end
            || priority.bang.end > priority.important_ident.start
            || !priority.complete.contains(priority.important_ident)
            || !complete.contains(priority.complete)
        {
            return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
        }
        if !fragment_is(source, priority.bang, "!") {
            return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
        }
    }

    match occurrence.termination {
        DescriptorGoldOccurrenceTermination::AuthoredSemicolon(semicolon) => {
            if !anchor_ok(source, semicolon) || semicolon.is_empty() {
                return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if complete.end != semicolon.end {
                return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if !fragment_is(source, semicolon, ";") {
                return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
            }
        }
        DescriptorGoldOccurrenceTermination::OmittedBeforeRightCurly(right_curly) => {
            if !anchor_ok(source, right_curly) || right_curly.is_empty() {
                return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if right_curly.start < complete.end {
                return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if !fragment_is(source, right_curly, "}") {
                return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
            }
        }
        DescriptorGoldOccurrenceTermination::OmittedAtEndOfInput(eof) => {
            if !eof.is_empty() {
                return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if eof.start != source.as_str().len() {
                return Err(DescriptorGoldValidationError::OccurrenceSyntaxInvalid);
            }
        }
    }

    Ok(())
}
