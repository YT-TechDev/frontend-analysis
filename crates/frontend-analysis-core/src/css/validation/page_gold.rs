//! Candidate-independent CSS `@page`/page-margin evidence gold data model
//! (#170 validation phase).
//!
//! Structurally independent from production: fixtures are authored here from
//! the approved #170 architecture record, never from production
//! `CssParserContextRecord`, `CssParserPageMarginRuleKind`,
//! `CssPageDeclarationOccurrence`, `CssPageMarginDeclarationOccurrence`, a
//! producer, an external parser, CSSOM, or browser output. This module must
//! never import those production types, and production code must never
//! import this module.
//!
//! Kept a wholly separate layer from `context_gold.rs`/`context_fixtures.rs`
//! (the fixed 16-fixture #166 inventory), `group_context_fixtures.rs` (the
//! fixed 28-fixture #168 inventory), and `descriptor_gold.rs`/
//! `descriptor_fixtures.rs` (the #169 inventory): #170 introduces two new
//! context kinds (`PageRuleBlock`, root-owned, and `PageMarginRuleBlock`,
//! nested only inside a retained `PageRuleBlock`) whose own ordinal-space and
//! ancestry shape differ from all three existing inventories, so this model
//! does not reuse their machinery.

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

/// Independent finite sixteen-member page-margin-rule-kind vocabulary
/// (#170). Mirrors only the approved registry; not the production enum
/// itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PageGoldMarginKind {
    TopLeftCorner,
    TopLeft,
    TopCenter,
    TopRight,
    TopRightCorner,
    BottomLeftCorner,
    BottomLeft,
    BottomCenter,
    BottomRight,
    BottomRightCorner,
    LeftTop,
    LeftMiddle,
    LeftBottom,
    RightTop,
    RightMiddle,
    RightBottom,
}

impl PageGoldMarginKind {
    pub(super) const fn decoded_name(self) -> &'static str {
        match self {
            Self::TopLeftCorner => "top-left-corner",
            Self::TopLeft => "top-left",
            Self::TopCenter => "top-center",
            Self::TopRight => "top-right",
            Self::TopRightCorner => "top-right-corner",
            Self::BottomLeftCorner => "bottom-left-corner",
            Self::BottomLeft => "bottom-left",
            Self::BottomCenter => "bottom-center",
            Self::BottomRight => "bottom-right",
            Self::BottomRightCorner => "bottom-right-corner",
            Self::LeftTop => "left-top",
            Self::LeftMiddle => "left-middle",
            Self::LeftBottom => "left-bottom",
            Self::RightTop => "right-top",
            Self::RightMiddle => "right-middle",
            Self::RightBottom => "right-bottom",
        }
    }

    pub(super) const ALL: [Self; 16] = [
        Self::TopLeftCorner,
        Self::TopLeft,
        Self::TopCenter,
        Self::TopRight,
        Self::TopRightCorner,
        Self::BottomLeftCorner,
        Self::BottomLeft,
        Self::BottomCenter,
        Self::BottomRight,
        Self::BottomRightCorner,
        Self::LeftTop,
        Self::LeftMiddle,
        Self::LeftBottom,
        Self::RightTop,
        Self::RightMiddle,
        Self::RightBottom,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PageGoldTermination {
    AuthoredRightCurly(GoldRange),
    EndOfInput(GoldRange),
    UpstreamTokenizerIncomplete(GoldRange),
    ParserResourceLimit(GoldRange),
}

impl PageGoldTermination {
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
pub(super) struct PageGoldPriority {
    pub(super) complete: GoldRange,
    pub(super) bang: GoldRange,
    pub(super) important_ident: GoldRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PageGoldOccurrenceTermination {
    AuthoredSemicolon(GoldRange),
    OmittedBeforeRightCurly(GoldRange),
    OmittedAtEndOfInput(GoldRange),
}

/// One expected `Ident : value` syntax occurrence, shared shape between an
/// ordinary companion declaration, a `@font-face` companion descriptor, a
/// page declaration, and a page-margin declaration: all four share
/// `CssDeclarationSyntaxEvidence` in production, and result-level validation
/// enforces the semantic owner boundary between them, never this gold shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PageGoldOccurrence {
    pub(super) item_ordinal: usize,
    pub(super) complete: GoldRange,
    pub(super) name: GoldRange,
    pub(super) colon: GoldRange,
    pub(super) value: GoldRange,
    pub(super) priority: Option<PageGoldPriority>,
    pub(super) termination: PageGoldOccurrenceTermination,
}

/// One expected explicit unsupported nested at-rule direct item inside a
/// `PageRuleBlock` or `PageMarginRuleBlock` body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PageGoldNestedUnsupportedAtRule {
    pub(super) item_ordinal: usize,
    pub(super) complete: GoldRange,
    pub(super) at_keyword: GoldRange,
}

/// One independent `PageMarginRuleBlock` context record, nested only inside
/// a retained `PageRuleBlock`: `item_ordinal` is the ordinal this margin
/// child claims in its *parent Page context's* shared direct-item ordinal
/// space (declarations, margin children, and nested unsupported at-rules
/// together), never its own separate space. `PageMarginRuleBlock` never has
/// children, so its own body admits only declarations and nested unsupported
/// at-rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PageGoldMarginContext {
    pub(super) id: usize,
    pub(super) item_ordinal: usize,
    pub(super) kind: PageGoldMarginKind,
    pub(super) at_keyword: GoldRange,
    pub(super) header: GoldRange,
    pub(super) block_opener: GoldRange,
    pub(super) body: GoldRange,
    pub(super) termination: PageGoldTermination,
    pub(super) declarations: Vec<PageGoldOccurrence>,
    pub(super) nested_unsupported: Vec<PageGoldNestedUnsupportedAtRule>,
}

impl PageGoldMarginContext {
    fn extent_end(&self) -> usize {
        self.termination.evidence_range().end
    }
}

/// One independent `PageRuleBlock` context record: always stylesheet-root
/// (#170), so it carries a `root_item_ordinal` instead of a
/// parent/nearest-ancestor pair. `declarations`, `margins`, and
/// `nested_unsupported` together share one direct-item ordinal space,
/// mirroring production's `DirectItem` ledger (Amendment A: no run
/// semantic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PageGoldContext {
    pub(super) id: usize,
    pub(super) root_item_ordinal: usize,
    pub(super) at_keyword: GoldRange,
    /// `None` for a selector-less `@page`; `Some` with the exact authored
    /// `<page-selector-list>` envelope otherwise.
    pub(super) page_selector_list: Option<GoldRange>,
    pub(super) header: GoldRange,
    pub(super) block_opener: GoldRange,
    pub(super) body: GoldRange,
    pub(super) termination: PageGoldTermination,
    pub(super) declarations: Vec<PageGoldOccurrence>,
    pub(super) margins: Vec<PageGoldMarginContext>,
    pub(super) nested_unsupported: Vec<PageGoldNestedUnsupportedAtRule>,
}

impl PageGoldContext {
    fn extent_end(&self) -> usize {
        self.termination.evidence_range().end
    }
}

/// A minimal independently-authored ordinary `QualifiedRuleBlock` companion,
/// used only to prove category 38 (the `DeclarationOccurrences` aggregate
/// cap sums all four occurrence categories). Not a general qualified-rule
/// gold model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PageGoldQualifiedCompanion {
    pub(super) id: usize,
    pub(super) root_item_ordinal: usize,
    pub(super) header: GoldRange,
    pub(super) block_opener: GoldRange,
    pub(super) body: GoldRange,
    pub(super) termination: PageGoldTermination,
    pub(super) declarations: Vec<PageGoldOccurrence>,
    /// Used only to prove category 23 (`@page` nested in a non-root
    /// supported context remains unsupported): a companion normally carries
    /// none.
    pub(super) nested_unsupported: Vec<PageGoldNestedUnsupportedAtRule>,
}

impl PageGoldQualifiedCompanion {
    fn extent_end(&self) -> usize {
        self.termination.evidence_range().end
    }
}

/// A minimal independently-authored `@font-face` descriptor companion, used
/// only to prove category 38 alongside [`PageGoldQualifiedCompanion`]. Not a
/// general descriptor gold model -- `descriptor_gold.rs` owns that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PageGoldDescriptorCompanion {
    pub(super) id: usize,
    pub(super) root_item_ordinal: usize,
    pub(super) at_keyword: GoldRange,
    pub(super) header: GoldRange,
    pub(super) block_opener: GoldRange,
    pub(super) body: GoldRange,
    pub(super) termination: PageGoldTermination,
    pub(super) occurrences: Vec<PageGoldOccurrence>,
}

impl PageGoldDescriptorCompanion {
    fn extent_end(&self) -> usize {
        self.termination.evidence_range().end
    }
}

/// One expected root-level (stylesheet-root direct child) item, sharing one
/// root-ordinal space: a supported `PageRuleBlock`, an ordinary
/// qualified-rule companion, an `@font-face` descriptor companion, or a
/// structurally consumed unsupported top-level at-rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PageGoldRoot {
    Page(PageGoldContext),
    Qualified(PageGoldQualifiedCompanion),
    Descriptor(PageGoldDescriptorCompanion),
    UnsupportedAtRule {
        root_item_ordinal: usize,
        complete: GoldRange,
        at_keyword: GoldRange,
    },
}

impl PageGoldRoot {
    const fn root_item_ordinal(&self) -> usize {
        match self {
            Self::Page(context) => context.root_item_ordinal,
            Self::Qualified(companion) => companion.root_item_ordinal,
            Self::Descriptor(companion) => companion.root_item_ordinal,
            Self::UnsupportedAtRule {
                root_item_ordinal, ..
            } => *root_item_ordinal,
        }
    }

    fn span(&self) -> (usize, usize) {
        match self {
            Self::Page(context) => (context.header.start, context.extent_end()),
            Self::Qualified(companion) => (companion.header.start, companion.extent_end()),
            Self::Descriptor(companion) => (companion.header.start, companion.extent_end()),
            Self::UnsupportedAtRule { complete, .. } => (complete.start, complete.end),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PageGoldResourceKind {
    AlgorithmSteps,
    PeakContextDepth,
    ContextRecords,
    DeclarationOccurrences,
    UnsupportedRegions,
    RecoveryRecords,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PageGoldResourceExpectation {
    pub(super) kind: PageGoldResourceKind,
    pub(super) limit: usize,
    pub(super) attempted: usize,
}

/// The #170 architecture validation category (section 14 of the focused
/// implementation/evidence plan) this fixture proves. One fixture may
/// honestly prove more than one category; every category 1-40 must be
/// reachable from at least one fixture's `categories` list or a documented
/// cross-cutting proof elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum PageGoldCategory {
    SelectorLessPage = 1,
    NamedPage = 2,
    PseudoOnlyPage = 3,
    NamedAndPseudoPage = 4,
    RepeatedPseudo = 5,
    CommaSeparatedSelectorList = 6,
    EscapedSelectorComponent = 7,
    CommentAdjacentPseudoQualifies = 8,
    RealWhitespaceBeforePseudoDisqualifies = 9,
    LeadingCommaDisqualifies = 10,
    TrailingCommaDisqualifies = 11,
    EmptyMemberDisqualifies = 12,
    UnknownPseudoDisqualifies = 13,
    AutoStructurallyValid = 14,
    OneMarginChild = 15,
    AllSixteenMarginKinds = 16,
    PageDeclarationMarginPageDeclarationOrdering = 17,
    MultipleMarginChildren = 18,
    SamePropertySpellingPageVsMargin = 19,
    DuplicateSpellingDistinctContextIds = 20,
    NonEmptyMarginPreludeUnsupported = 21,
    TopLevelMarginRuleUnsupported = 22,
    PageNestedInNonRootSupportedContextUnsupported = 23,
    GroupRuleNameInsidePageUnsupported = 24,
    QualifiedRuleShapedMalformedPageContent = 25,
    QualifiedRuleShapedMalformedMarginContent = 26,
    NestedAtRuleInMargin = 27,
    ValidParentMalformedChildFollowingValidItemRecovery = 28,
    AuthoredClosePage = 29,
    TrueEofPage = 30,
    TrueEofInsideMarginPreservesBothContexts = 31,
    UpstreamIncompleteWhilePageActive = 32,
    UpstreamIncompleteWhileMarginActive = 33,
    ParserResourceStopAfterPageEntry = 34,
    ParserResourceStopAfterMarginEntry = 35,
    PeakContextDepthRefusal = 36,
    ContextRecordsRefusal = 37,
    AggregateDeclarationOccurrencesAcrossAllFour = 38,
    UnsupportedRegionsRefusalNoOrdinalLeak = 39,
    DeterministicRepeatAndCrossSourceId = 40,
}

pub(super) const ALL_CATEGORIES: [PageGoldCategory; 40] = [
    PageGoldCategory::SelectorLessPage,
    PageGoldCategory::NamedPage,
    PageGoldCategory::PseudoOnlyPage,
    PageGoldCategory::NamedAndPseudoPage,
    PageGoldCategory::RepeatedPseudo,
    PageGoldCategory::CommaSeparatedSelectorList,
    PageGoldCategory::EscapedSelectorComponent,
    PageGoldCategory::CommentAdjacentPseudoQualifies,
    PageGoldCategory::RealWhitespaceBeforePseudoDisqualifies,
    PageGoldCategory::LeadingCommaDisqualifies,
    PageGoldCategory::TrailingCommaDisqualifies,
    PageGoldCategory::EmptyMemberDisqualifies,
    PageGoldCategory::UnknownPseudoDisqualifies,
    PageGoldCategory::AutoStructurallyValid,
    PageGoldCategory::OneMarginChild,
    PageGoldCategory::AllSixteenMarginKinds,
    PageGoldCategory::PageDeclarationMarginPageDeclarationOrdering,
    PageGoldCategory::MultipleMarginChildren,
    PageGoldCategory::SamePropertySpellingPageVsMargin,
    PageGoldCategory::DuplicateSpellingDistinctContextIds,
    PageGoldCategory::NonEmptyMarginPreludeUnsupported,
    PageGoldCategory::TopLevelMarginRuleUnsupported,
    PageGoldCategory::PageNestedInNonRootSupportedContextUnsupported,
    PageGoldCategory::GroupRuleNameInsidePageUnsupported,
    PageGoldCategory::QualifiedRuleShapedMalformedPageContent,
    PageGoldCategory::QualifiedRuleShapedMalformedMarginContent,
    PageGoldCategory::NestedAtRuleInMargin,
    PageGoldCategory::ValidParentMalformedChildFollowingValidItemRecovery,
    PageGoldCategory::AuthoredClosePage,
    PageGoldCategory::TrueEofPage,
    PageGoldCategory::TrueEofInsideMarginPreservesBothContexts,
    PageGoldCategory::UpstreamIncompleteWhilePageActive,
    PageGoldCategory::UpstreamIncompleteWhileMarginActive,
    PageGoldCategory::ParserResourceStopAfterPageEntry,
    PageGoldCategory::ParserResourceStopAfterMarginEntry,
    PageGoldCategory::PeakContextDepthRefusal,
    PageGoldCategory::ContextRecordsRefusal,
    PageGoldCategory::AggregateDeclarationOccurrencesAcrossAllFour,
    PageGoldCategory::UnsupportedRegionsRefusalNoOrdinalLeak,
    PageGoldCategory::DeterministicRepeatAndCrossSourceId,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PageGoldFixture {
    pub(super) id: &'static str,
    pub(super) categories: &'static [PageGoldCategory],
    pub(super) source_id: u64,
    pub(super) source: &'static str,
    pub(super) byte_len: usize,
    pub(super) roots: Vec<PageGoldRoot>,
    pub(super) resource_expectation: Option<PageGoldResourceExpectation>,
    /// Expected `InvalidBlockItem` diagnostic locations (the only diagnostic
    /// code a malformed page/page-margin-block item can produce; shared,
    /// pre-#170 vocabulary, never a Page-specific taxonomy).
    pub(super) diagnostics: Vec<GoldRange>,
    pub(super) recovery: Vec<PageGoldRecovery>,
}

/// Mirrors the shared, pre-#170 `MalformedBlockItem` recovery shape,
/// authored independently here so this module never imports the flat-parser
/// or descriptor gold layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PageGoldRecovery {
    pub(super) region: GoldRange,
    pub(super) termination: PageGoldRecoveryTermination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PageGoldRecoveryTermination {
    AuthoredSemicolon(GoldRange),
    EnclosingBlockEnd(GoldRange),
    EndOfInput(GoldRange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PageGoldValidationError {
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
    SelectorListPresenceInvalid,
    SelectorListShapeInvalid,
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

pub(super) fn validate_fixture(fixture: &PageGoldFixture) -> Result<(), PageGoldValidationError> {
    if fixture.source.len() != fixture.byte_len {
        return Err(PageGoldValidationError::ByteLengthMismatch);
    }
    if fixture.categories.is_empty() {
        return Err(PageGoldValidationError::EmptyCategories);
    }
    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());

    let mut next_context_id = 0usize;

    for root in &fixture.roots {
        match root {
            PageGoldRoot::Page(context) => {
                if context.id != next_context_id {
                    return Err(PageGoldValidationError::IdIndexMismatch);
                }
                next_context_id += 1;
                validate_page_context(&source, context, &mut next_context_id)?;
            }
            PageGoldRoot::Qualified(companion) => {
                if companion.id != next_context_id {
                    return Err(PageGoldValidationError::IdIndexMismatch);
                }
                next_context_id += 1;
                validate_qualified_companion(&source, companion)?;
            }
            PageGoldRoot::Descriptor(companion) => {
                if companion.id != next_context_id {
                    return Err(PageGoldValidationError::IdIndexMismatch);
                }
                next_context_id += 1;
                validate_descriptor_companion(&source, companion)?;
            }
            PageGoldRoot::UnsupportedAtRule {
                complete,
                at_keyword,
                ..
            } => {
                if !anchor_ok(&source, *complete) || complete.is_empty() {
                    return Err(PageGoldValidationError::UnsupportedAtRuleShapeInvalid);
                }
                if !anchor_ok(&source, *at_keyword) || at_keyword.is_empty() {
                    return Err(PageGoldValidationError::UnsupportedAtRuleShapeInvalid);
                }
                if at_keyword.start != complete.start || !complete.contains(*at_keyword) {
                    return Err(PageGoldValidationError::UnsupportedAtRuleShapeInvalid);
                }
                if !fragment_starts_with(&source, *at_keyword, "@") {
                    return Err(PageGoldValidationError::UnsupportedAtRuleShapeInvalid);
                }
            }
        }
    }

    validate_root_ordinal_space(fixture)?;

    if let Some(expectation) = fixture.resource_expectation
        && expectation.attempted <= expectation.limit
    {
        return Err(PageGoldValidationError::ResourceExpectationAttemptDidNotExceedLimit);
    }

    let mut previous: Option<(usize, usize)> = None;
    for diagnostic in &fixture.diagnostics {
        if !anchor_ok(&source, *diagnostic) {
            return Err(PageGoldValidationError::InvalidRange);
        }
        let key = (diagnostic.start, diagnostic.end);
        if previous.is_some_and(|previous| previous > key) {
            return Err(PageGoldValidationError::DiagnosticOrder);
        }
        previous = Some(key);
    }

    let mut previous: Option<(usize, usize)> = None;
    for recovery in &fixture.recovery {
        if !anchor_ok(&source, recovery.region) || recovery.region.is_empty() {
            return Err(PageGoldValidationError::InvalidRange);
        }
        match recovery.termination {
            PageGoldRecoveryTermination::AuthoredSemicolon(semicolon) => {
                if !anchor_ok(&source, semicolon) || semicolon.end != recovery.region.end {
                    return Err(PageGoldValidationError::InvalidRange);
                }
                if !fragment_is(&source, semicolon, ";") {
                    return Err(PageGoldValidationError::InvalidRange);
                }
            }
            PageGoldRecoveryTermination::EnclosingBlockEnd(right_curly) => {
                if !anchor_ok(&source, right_curly) || right_curly.start != recovery.region.end {
                    return Err(PageGoldValidationError::InvalidRange);
                }
                if !fragment_is(&source, right_curly, "}") {
                    return Err(PageGoldValidationError::InvalidRange);
                }
            }
            PageGoldRecoveryTermination::EndOfInput(terminal) => {
                if !anchor_ok(&source, terminal) || !terminal.is_empty() {
                    return Err(PageGoldValidationError::InvalidRange);
                }
                if terminal.start != source.as_str().len() || terminal.start != recovery.region.end
                {
                    return Err(PageGoldValidationError::InvalidRange);
                }
            }
        }
        let key = (recovery.region.start, recovery.region.end);
        if previous.is_some_and(|previous| previous > key) {
            return Err(PageGoldValidationError::RecoveryOrder);
        }
        previous = Some(key);
    }

    Ok(())
}

/// Root-level direct-item ordinal space: Page contexts, qualified/descriptor
/// companions, and unsupported top-level at-rules share one gapless,
/// strictly-increasing, source-order-consistent ordinal space, mirroring
/// `descriptor_gold.rs`'s own root reconciliation.
fn validate_root_ordinal_space(fixture: &PageGoldFixture) -> Result<(), PageGoldValidationError> {
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
            return Err(PageGoldValidationError::RootOrdinalOrderViolation);
        }
        if let Some(previous_end) = previous_end
            && start < previous_end
        {
            return Err(PageGoldValidationError::RootSourceOrderViolation);
        }
        previous_end = Some(end);
        expected_next = ordinal + 1;
    }
    Ok(())
}

fn validate_page_context(
    source: &SourceText,
    context: &PageGoldContext,
    next_context_id: &mut usize,
) -> Result<(), PageGoldValidationError> {
    validate_shared_shape(
        source,
        context.header,
        context.block_opener,
        context.body,
        context.termination,
    )?;

    if !anchor_ok(source, context.at_keyword) || context.at_keyword.is_empty() {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }
    if context.at_keyword.start != context.header.start
        || !context.header.contains(context.at_keyword)
    {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }
    if !fragment_starts_with(source, context.at_keyword, "@") {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }

    if let Some(selector_list) = context.page_selector_list {
        if !anchor_ok(source, selector_list) || selector_list.is_empty() {
            return Err(PageGoldValidationError::SelectorListShapeInvalid);
        }
        if selector_list.start < context.at_keyword.end || !context.header.contains(selector_list) {
            return Err(PageGoldValidationError::SelectorListShapeInvalid);
        }
    }

    for declaration in &context.declarations {
        validate_occurrence_syntax(source, declaration, context.body)?;
    }
    for unsupported in &context.nested_unsupported {
        validate_unsupported_shape(source, unsupported, context.body)?;
    }
    for margin in &context.margins {
        if margin.id != *next_context_id {
            return Err(PageGoldValidationError::IdIndexMismatch);
        }
        *next_context_id += 1;
        validate_margin_context(source, margin, context.body)?;
    }

    let mut items: Vec<(usize, usize, usize)> = Vec::new();
    for declaration in &context.declarations {
        items.push((
            declaration.item_ordinal,
            declaration.complete.start,
            declaration.complete.end,
        ));
    }
    for unsupported in &context.nested_unsupported {
        items.push((
            unsupported.item_ordinal,
            unsupported.complete.start,
            unsupported.complete.end,
        ));
    }
    for margin in &context.margins {
        items.push((
            margin.item_ordinal,
            margin.header.start,
            margin.extent_end(),
        ));
    }
    validate_direct_item_ordinals(&items)
}

fn validate_margin_context(
    source: &SourceText,
    margin: &PageGoldMarginContext,
    page_body: GoldRange,
) -> Result<(), PageGoldValidationError> {
    validate_shared_shape(
        source,
        margin.header,
        margin.block_opener,
        margin.body,
        margin.termination,
    )?;
    if margin.header.start < page_body.start || margin.extent_end() > page_body.end {
        return Err(PageGoldValidationError::OccurrenceOutsideBody);
    }

    if !anchor_ok(source, margin.at_keyword) || margin.at_keyword.is_empty() {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }
    if margin.at_keyword.start != margin.header.start || !margin.header.contains(margin.at_keyword)
    {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }
    if !fragment_is(
        source,
        margin.at_keyword,
        &format!("@{}", margin.kind.decoded_name()),
    ) {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }

    for declaration in &margin.declarations {
        validate_occurrence_syntax(source, declaration, margin.body)?;
    }
    for unsupported in &margin.nested_unsupported {
        validate_unsupported_shape(source, unsupported, margin.body)?;
    }

    let mut items: Vec<(usize, usize, usize)> = Vec::new();
    for declaration in &margin.declarations {
        items.push((
            declaration.item_ordinal,
            declaration.complete.start,
            declaration.complete.end,
        ));
    }
    for unsupported in &margin.nested_unsupported {
        items.push((
            unsupported.item_ordinal,
            unsupported.complete.start,
            unsupported.complete.end,
        ));
    }
    validate_direct_item_ordinals(&items)
}

fn validate_direct_item_ordinals(
    items: &[(usize, usize, usize)],
) -> Result<(), PageGoldValidationError> {
    let mut sorted = items.to_vec();
    sorted.sort_by_key(|(ordinal, ..)| *ordinal);
    let mut expected_next = 0usize;
    let mut previous_ordinal: Option<usize> = None;
    let mut previous_end: Option<usize> = None;
    for (ordinal, start, end) in sorted {
        if ordinal != expected_next {
            if previous_ordinal == Some(ordinal) {
                return Err(PageGoldValidationError::DirectItemDuplicateOrdinal);
            }
            return Err(PageGoldValidationError::DirectItemOrderViolation);
        }
        if let Some(previous_end) = previous_end
            && start < previous_end
        {
            return Err(PageGoldValidationError::DirectItemOrderViolation);
        }
        previous_ordinal = Some(ordinal);
        previous_end = Some(end);
        expected_next = ordinal + 1;
    }
    Ok(())
}

fn validate_unsupported_shape(
    source: &SourceText,
    unsupported: &PageGoldNestedUnsupportedAtRule,
    body: GoldRange,
) -> Result<(), PageGoldValidationError> {
    if !anchor_ok(source, unsupported.complete) || unsupported.complete.is_empty() {
        return Err(PageGoldValidationError::UnsupportedAtRuleShapeInvalid);
    }
    if !anchor_ok(source, unsupported.at_keyword) || unsupported.at_keyword.is_empty() {
        return Err(PageGoldValidationError::UnsupportedAtRuleShapeInvalid);
    }
    if unsupported.at_keyword.start != unsupported.complete.start
        || !unsupported.complete.contains(unsupported.at_keyword)
    {
        return Err(PageGoldValidationError::UnsupportedAtRuleShapeInvalid);
    }
    if !fragment_starts_with(source, unsupported.at_keyword, "@") {
        return Err(PageGoldValidationError::UnsupportedAtRuleShapeInvalid);
    }
    if unsupported.complete.start < body.start || unsupported.complete.end > body.end {
        return Err(PageGoldValidationError::OccurrenceOutsideBody);
    }
    Ok(())
}

fn validate_qualified_companion(
    source: &SourceText,
    companion: &PageGoldQualifiedCompanion,
) -> Result<(), PageGoldValidationError> {
    validate_shared_shape(
        source,
        companion.header,
        companion.block_opener,
        companion.body,
        companion.termination,
    )?;
    for declaration in &companion.declarations {
        validate_occurrence_syntax(source, declaration, companion.body)?;
    }
    for unsupported in &companion.nested_unsupported {
        validate_unsupported_shape(source, unsupported, companion.body)?;
    }

    let mut items: Vec<(usize, usize, usize)> = Vec::new();
    for declaration in &companion.declarations {
        items.push((
            declaration.item_ordinal,
            declaration.complete.start,
            declaration.complete.end,
        ));
    }
    for unsupported in &companion.nested_unsupported {
        items.push((
            unsupported.item_ordinal,
            unsupported.complete.start,
            unsupported.complete.end,
        ));
    }
    validate_direct_item_ordinals(&items)
}

fn validate_descriptor_companion(
    source: &SourceText,
    companion: &PageGoldDescriptorCompanion,
) -> Result<(), PageGoldValidationError> {
    validate_shared_shape(
        source,
        companion.header,
        companion.block_opener,
        companion.body,
        companion.termination,
    )?;
    if !anchor_ok(source, companion.at_keyword) || companion.at_keyword.is_empty() {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }
    if companion.at_keyword.start != companion.header.start
        || !companion.header.contains(companion.at_keyword)
    {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }
    if !fragment_is(source, companion.at_keyword, "@font-face") {
        return Err(PageGoldValidationError::AtKeywordShapeInvalid);
    }
    let mut previous_end: Option<usize> = None;
    for (index, occurrence) in companion.occurrences.iter().enumerate() {
        validate_occurrence_syntax(source, occurrence, companion.body)?;
        if occurrence.item_ordinal != index {
            return Err(PageGoldValidationError::DirectItemOrderViolation);
        }
        if let Some(previous_end) = previous_end
            && occurrence.complete.start < previous_end
        {
            return Err(PageGoldValidationError::DirectItemOrderViolation);
        }
        previous_end = Some(occurrence.complete.end);
    }
    Ok(())
}

fn validate_shared_shape(
    source: &SourceText,
    header: GoldRange,
    block_opener: GoldRange,
    body: GoldRange,
    termination: PageGoldTermination,
) -> Result<(), PageGoldValidationError> {
    if !anchor_ok(source, header) {
        return Err(PageGoldValidationError::InvalidRange);
    }
    if !anchor_ok(source, block_opener) || block_opener.is_empty() {
        return Err(PageGoldValidationError::InvalidRange);
    }
    if !fragment_is(source, block_opener, "{") {
        return Err(PageGoldValidationError::OpenerNotExact);
    }
    if header.end != block_opener.start {
        return Err(PageGoldValidationError::HeaderOpenerBoundaryMismatch);
    }
    if !anchor_ok(source, body) {
        return Err(PageGoldValidationError::InvalidRange);
    }
    if body.start != block_opener.end {
        return Err(PageGoldValidationError::BodyOpenerBoundaryMismatch);
    }

    match termination {
        PageGoldTermination::AuthoredRightCurly(right_curly) => {
            if !anchor_ok(source, right_curly) || right_curly.is_empty() {
                return Err(PageGoldValidationError::InvalidRange);
            }
            if !fragment_is(source, right_curly, "}") {
                return Err(PageGoldValidationError::AuthoredCloserNotExact);
            }
            if right_curly.start != body.end {
                return Err(PageGoldValidationError::TerminationBoundaryMismatch);
            }
        }
        PageGoldTermination::EndOfInput(terminal) => {
            if !anchor_ok(source, terminal) || !terminal.is_empty() {
                return Err(PageGoldValidationError::TerminationMustBeEmpty);
            }
            if terminal.start != body.end {
                return Err(PageGoldValidationError::TerminationBoundaryMismatch);
            }
            if terminal.start != source.as_str().len() {
                return Err(PageGoldValidationError::EndOfInputNotAtSourceEnd);
            }
        }
        PageGoldTermination::UpstreamTokenizerIncomplete(terminal)
        | PageGoldTermination::ParserResourceLimit(terminal) => {
            if !anchor_ok(source, terminal) || !terminal.is_empty() {
                return Err(PageGoldValidationError::TerminationMustBeEmpty);
            }
            if terminal.start != body.end {
                return Err(PageGoldValidationError::TerminationBoundaryMismatch);
            }
        }
    }

    Ok(())
}

fn validate_occurrence_syntax(
    source: &SourceText,
    occurrence: &PageGoldOccurrence,
    body: GoldRange,
) -> Result<(), PageGoldValidationError> {
    let complete = occurrence.complete;
    let name = occurrence.name;
    let colon = occurrence.colon;
    let value = occurrence.value;

    if !anchor_ok(source, complete) || complete.is_empty() {
        return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if complete.start < body.start || complete.end > body.end {
        return Err(PageGoldValidationError::OccurrenceOutsideBody);
    }
    if !anchor_ok(source, name) || name.is_empty() {
        return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if complete.start != name.start || !complete.contains(name) {
        return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if !anchor_ok(source, colon) || colon.is_empty() {
        return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if !complete.contains(colon) || name.end > colon.start {
        return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if !fragment_is(source, colon, ":") {
        return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if !anchor_ok(source, value) {
        return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
    }
    if value.is_empty() {
        if value.start != colon.end {
            return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
        }
    } else if colon.end > value.start || !complete.contains(value) {
        return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
    }

    if let Some(priority) = occurrence.priority {
        if !anchor_ok(source, priority.complete)
            || !anchor_ok(source, priority.bang)
            || !anchor_ok(source, priority.important_ident)
            || priority.complete.is_empty()
            || priority.bang.is_empty()
            || priority.important_ident.is_empty()
        {
            return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
        }
        if priority.complete.start != priority.bang.start
            || priority.complete.end != priority.important_ident.end
            || priority.bang.end > priority.important_ident.start
            || !priority.complete.contains(priority.important_ident)
            || !complete.contains(priority.complete)
        {
            return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
        }
        if !fragment_is(source, priority.bang, "!") {
            return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
        }
    }

    match occurrence.termination {
        PageGoldOccurrenceTermination::AuthoredSemicolon(semicolon) => {
            if !anchor_ok(source, semicolon) || semicolon.is_empty() {
                return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if complete.end != semicolon.end {
                return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if !fragment_is(source, semicolon, ";") {
                return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
            }
        }
        PageGoldOccurrenceTermination::OmittedBeforeRightCurly(right_curly) => {
            if !anchor_ok(source, right_curly) || right_curly.is_empty() {
                return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if right_curly.start < complete.end {
                return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if !fragment_is(source, right_curly, "}") {
                return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
            }
        }
        PageGoldOccurrenceTermination::OmittedAtEndOfInput(eof) => {
            if !eof.is_empty() {
                return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
            }
            if eof.start != source.as_str().len() {
                return Err(PageGoldValidationError::OccurrenceSyntaxInvalid);
            }
        }
    }

    Ok(())
}
