//! Bounded declaration-value qualification for selected post-freeze CSS
//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475).
//!
//! This module consumes only the already Core-validated parser result and its
//! retained tokenizer evidence. It does not search or decode raw source,
//! retokenize declaration fragments, mutate parser evidence, or claim cascade,
//! inheritance, computed-value, CSSOM, DOM, layout, or browser-runtime semantics.

use std::error::Error;
use std::fmt;
use std::ops::Range;

use crate::{SourceAnchor, SourceId};

use super::declaration::CssDeclarationPlacement;
use super::parser::result::{CssParserExecutionCompletion, CssParserRunResult};
use super::token::{CssLexicalItem, CssNumberSign, CssNumberType, CssNumericValue, CssTokenKind};
use super::tokenizer::result::CssTokenizerRunResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssDirectionValue {
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssDirectionUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssDirectionQualificationOutcome {
    Qualified(CssDirectionValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssDirectionUnsupportedReason),
}

/// One selected ordinary declaration's direction-value qualification.
///
/// `occurrence_index` is run-local and meaningful only through the structurally
/// owning [`CssValueQualificationRunResult`]. The observation deliberately does
/// not duplicate authored anchors: exact source evidence remains owned by the
/// corresponding upstream `CssDeclarationOccurrence`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssDirectionQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssDirectionQualificationOutcome,
}

impl CssDirectionQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssDirectionQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBoxSizingValue {
    ContentBox,
    BorderBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBoxSizingUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBoxSizingQualificationOutcome {
    Qualified(CssBoxSizingValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssBoxSizingUnsupportedReason),
}

/// One selected ordinary declaration's `box-sizing` value qualification.
///
/// As with direction observations, placement and `occurrence_index` remain
/// run-local references into the exact parser result structurally owned by the
/// enclosing [`CssValueQualificationRunResult`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssBoxSizingQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssBoxSizingQualificationOutcome,
}

impl CssBoxSizingQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssBoxSizingQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssIsolationValue {
    Auto,
    Isolate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssIsolationUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssIsolationQualificationOutcome {
    Qualified(CssIsolationValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssIsolationUnsupportedReason),
}

/// One selected ordinary declaration's `isolation` value qualification.
///
/// Placement and `occurrence_index` remain run-local references into the exact
/// parser result structurally owned by the enclosing
/// [`CssValueQualificationRunResult`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssIsolationQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssIsolationQualificationOutcome,
}

impl CssIsolationQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssIsolationQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBackfaceVisibilityValue {
    Visible,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBackfaceVisibilityUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBackfaceVisibilityQualificationOutcome {
    Qualified(CssBackfaceVisibilityValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssBackfaceVisibilityUnsupportedReason),
}

/// One selected ordinary declaration's bounded `backface-visibility`
/// qualification.
///
/// This profile qualifies only direct `visible | hidden` authored keyword
/// evidence. Transform matrices, 3D rendering context, backface geometry,
/// containing-block behavior, painting, and compositing remain outside this
/// slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssBackfaceVisibilityQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssBackfaceVisibilityQualificationOutcome,
}

impl CssBackfaceVisibilityQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssBackfaceVisibilityQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollSnapStopValue {
    Normal,
    Always,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollSnapStopUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollSnapStopQualificationOutcome {
    Qualified(CssScrollSnapStopValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssScrollSnapStopUnsupportedReason),
}

/// One selected ordinary declaration's bounded `scroll-snap-stop`
/// qualification.
///
/// This profile qualifies only direct `normal | always` authored keyword
/// evidence. Relative-scroll classification, snap trapping, snap-position
/// selection, scroll physics, and resnapping remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssScrollSnapStopQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssScrollSnapStopQualificationOutcome,
}

impl CssScrollSnapStopQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssScrollSnapStopQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssEmptyCellsValue {
    Show,
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssEmptyCellsUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssEmptyCellsQualificationOutcome {
    Qualified(CssEmptyCellsValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssEmptyCellsUnsupportedReason),
}

/// One selected ordinary declaration's bounded `empty-cells` qualification.
///
/// This profile qualifies only direct `show | hide` authored keyword evidence.
/// Empty-cell determination, table layout, border/background suppression,
/// baseline alignment, painting, and used-value behavior remain outside this
/// slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssEmptyCellsQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssEmptyCellsQualificationOutcome,
}

impl CssEmptyCellsQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssEmptyCellsQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationStyleValue {
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationStyleUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationStyleQualificationOutcome {
    Qualified(CssTextDecorationStyleValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextDecorationStyleUnsupportedReason),
}

/// One selected ordinary declaration's bounded `text-decoration-style`
/// qualification.
///
/// This profile qualifies only direct `solid | double | dotted | dashed | wavy`
/// authored keyword evidence. Decoration painting, dash/wave geometry,
/// thickness/font-metric interaction, pseudo applicability, and rendering
/// remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextDecorationStyleQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextDecorationStyleQualificationOutcome,
}

impl CssTextDecorationStyleQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextDecorationStyleQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTableLayoutValue {
    Auto,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTableLayoutUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTableLayoutQualificationOutcome {
    Qualified(CssTableLayoutValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTableLayoutUnsupportedReason),
}

/// One selected ordinary declaration's bounded `table-layout` qualification.
///
/// This profile qualifies only direct `auto | fixed` authored keyword evidence.
/// Table layout algorithms, intrinsic/column sizing, width distribution,
/// table-wrapper/grid applicability, and used-value behavior remain outside
/// this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTableLayoutQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTableLayoutQualificationOutcome,
}

impl CssTableLayoutQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTableLayoutQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderCollapseValue {
    Separate,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderCollapseUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderCollapseQualificationOutcome {
    Qualified(CssBorderCollapseValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssBorderCollapseUnsupportedReason),
}

/// One selected ordinary declaration's bounded `border-collapse` qualification.
///
/// This profile qualifies only direct `separate | collapse` authored keyword
/// evidence. Collapsed-border conflict resolution, border painting, table
/// layout/sizing, sticky-border behavior, and used-value processing remain
/// outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssBorderCollapseQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssBorderCollapseQualificationOutcome,
}

impl CssBorderCollapseQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssBorderCollapseQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBoxDecorationBreakValue {
    Slice,
    Clone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBoxDecorationBreakUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBoxDecorationBreakQualificationOutcome {
    Qualified(CssBoxDecorationBreakValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssBoxDecorationBreakUnsupportedReason),
}

/// One selected ordinary declaration's bounded `box-decoration-break`
/// qualification.
///
/// This profile qualifies only direct `slice | clone` authored keyword
/// evidence. Fragment construction, fragmentation algorithms, border/background
/// painting, mask/clip interaction, and used-value processing remain outside
/// this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssBoxDecorationBreakQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssBoxDecorationBreakQualificationOutcome,
}

impl CssBoxDecorationBreakQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssBoxDecorationBreakQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontKerningValue {
    Auto,
    Normal,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontKerningUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontKerningQualificationOutcome {
    Qualified(CssFontKerningValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontKerningUnsupportedReason),
}

/// One selected ordinary declaration's bounded `font-kerning` qualification.
///
/// This profile qualifies only direct `auto | normal | none` authored keyword
/// evidence. Glyph shaping, OpenType `kern`/`vkrn` processing, font-table
/// inspection, font selection/fallback, text layout, and used-value processing
/// remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontKerningQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontKerningQualificationOutcome,
}

impl CssFontKerningQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontKerningQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontSynthesisWeightValue {
    Auto,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontSynthesisWeightUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontSynthesisWeightQualificationOutcome {
    Qualified(CssFontSynthesisWeightValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontSynthesisWeightUnsupportedReason),
}

/// One selected ordinary declaration's bounded `font-synthesis-weight`
/// qualification.
///
/// This profile qualifies only direct `auto | none` authored keyword evidence.
/// Font selection, glyph synthesis, synthetic emboldening, font-table
/// inspection, shaping, rendering, and used-value processing remain outside
/// this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontSynthesisWeightQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontSynthesisWeightQualificationOutcome,
}

impl CssFontSynthesisWeightQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontSynthesisWeightQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantPositionValue {
    Normal,
    Sub,
    Super,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantPositionUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantPositionQualificationOutcome {
    Qualified(CssFontVariantPositionValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontVariantPositionUnsupportedReason),
}

/// One selected ordinary declaration's bounded `font-variant-position`
/// qualification.
///
/// This profile qualifies only direct `normal | sub | super` authored keyword
/// evidence. Glyph shaping and substitution, OpenType `subs`/`sups` processing,
/// synthetic sub/sup sizing or positioning, font metric overrides, baseline and
/// line-box layout, and used-value processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontVariantPositionQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontVariantPositionQualificationOutcome,
}

impl CssFontVariantPositionQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontVariantPositionQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOrderValue {
    DirectIntegerLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOrderUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOrderQualificationOutcome {
    Qualified(CssOrderValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOrderUnsupportedReason),
}

/// One selected ordinary declaration's bounded `order` qualification.
///
/// This outcome only qualifies direct authored integer literals. Exact sign,
/// digits, and source provenance remain in the structurally owning tokenizer
/// and parser evidence rather than being copied into this observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOrderQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOrderQualificationOutcome,
}

impl CssOrderQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOrderQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColumnCountValue {
    Auto,
    DirectIntegerLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColumnCountUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColumnCountQualificationOutcome {
    Qualified(CssColumnCountValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssColumnCountUnsupportedReason),
}

/// One selected ordinary declaration's bounded `column-count` qualification.
///
/// This profile qualifies only direct authored `auto` and direct authored
/// integer literals proven inside `[1,∞]`. Exact sign, digits, and source
/// provenance remain in the structurally owning tokenizer and parser evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssColumnCountQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssColumnCountQualificationOutcome,
}

impl CssColumnCountQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssColumnCountQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFlexGrowValue {
    DirectNumberLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFlexGrowUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFlexGrowQualificationOutcome {
    Qualified(CssFlexGrowValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFlexGrowUnsupportedReason),
}

/// One selected ordinary declaration's bounded `flex-grow` qualification.
///
/// This profile qualifies only direct authored number literals proven inside
/// `[0,∞]`. Exact source provenance remains in the structurally owning tokenizer
/// and parser evidence; no machine-number conversion or exponent evaluation is
/// performed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFlexGrowQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFlexGrowQualificationOutcome,
}

impl CssFlexGrowQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFlexGrowQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFlexShrinkValue {
    DirectNumberLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFlexShrinkUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFlexShrinkQualificationOutcome {
    Qualified(CssFlexShrinkValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFlexShrinkUnsupportedReason),
}

/// One selected ordinary declaration's bounded `flex-shrink` qualification.
///
/// This profile qualifies only direct authored number literals proven inside
/// `[0,∞]`. Exact source provenance remains in the structurally owning tokenizer
/// and parser evidence; no machine-number conversion or exponent evaluation is
/// performed here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFlexShrinkQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFlexShrinkQualificationOutcome,
}

impl CssFlexShrinkQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFlexShrinkQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOpacityValue {
    DirectNumberLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOpacityUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOpacityQualificationOutcome {
    Qualified(CssOpacityValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOpacityUnsupportedReason),
}

/// One selected ordinary declaration's bounded `opacity` qualification.
///
/// This profile qualifies direct authored Number and Percentage tokens only.
/// Values outside `[0,1]` remain qualified because CSS Color preserves them as
/// specified values and clamps only later in computed-value processing, which
/// this slice intentionally does not perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOpacityQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOpacityQualificationOutcome,
}

impl CssOpacityQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOpacityQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeImageThresholdValue {
    DirectNumberLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeImageThresholdUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeImageThresholdQualificationOutcome {
    Qualified(CssShapeImageThresholdValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssShapeImageThresholdUnsupportedReason),
}

/// One selected ordinary declaration's bounded `shape-image-threshold`
/// qualification.
///
/// This profile qualifies direct authored Number and Percentage tokens only.
/// Out-of-range authored values remain qualified; CSS Shapes clamps the
/// specified threshold only in downstream computed-value processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssShapeImageThresholdQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssShapeImageThresholdQualificationOutcome,
}

impl CssShapeImageThresholdQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssShapeImageThresholdQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeMarginValue {
    DirectLengthLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeMarginUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeMarginQualificationOutcome {
    Qualified(CssShapeMarginValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssShapeMarginUnsupportedReason),
}

/// One selected ordinary declaration's bounded `shape-margin` qualification.
///
/// This profile qualifies direct `<length-percentage [0,∞]>` evidence only.
/// It performs no percentage resolution, unit conversion, function evaluation,
/// shape construction, or float-layout processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssShapeMarginQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssShapeMarginQualificationOutcome,
}

impl CssShapeMarginQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssShapeMarginQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLineHeightValue {
    Normal,
    DirectNumberLiteral,
    DirectLengthLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLineHeightUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLineHeightQualificationOutcome {
    Qualified(CssLineHeightValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssLineHeightUnsupportedReason),
}

/// One selected ordinary declaration's bounded `line-height` qualification.
///
/// This profile composes direct `normal`, `<number [0,∞]>`, and
/// `<length-percentage [0,∞]>` evidence only. Ambiguous unitless zero belongs
/// to the Number branch. No calculation evaluation, percentage resolution,
/// font-metric processing, or line-box layout is performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssLineHeightQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssLineHeightQualificationOutcome,
}

impl CssLineHeightQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssLineHeightQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWordSpacingValue {
    Normal,
    DirectLengthLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWordSpacingUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWordSpacingQualificationOutcome {
    Qualified(CssWordSpacingValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssWordSpacingUnsupportedReason),
}

/// One selected ordinary declaration's bounded `word-spacing` qualification.
///
/// This profile composes direct `normal` and unrestricted signed
/// `<length-percentage>` evidence. It performs no machine numeric ordering,
/// percentage resolution, font-metric processing, shaping, or text layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssWordSpacingQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssWordSpacingQualificationOutcome,
}

impl CssWordSpacingQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssWordSpacingQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextUnderlineOffsetValue {
    Auto,
    DirectLengthLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextUnderlineOffsetUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextUnderlineOffsetQualificationOutcome {
    Qualified(CssTextUnderlineOffsetValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextUnderlineOffsetUnsupportedReason),
}

/// One selected ordinary declaration's bounded `text-underline-offset`
/// qualification.
///
/// This profile composes direct `auto` and unrestricted signed
/// `<length-percentage>` evidence. It performs no machine numeric ordering,
/// percentage resolution, inheritance processing, font-metric processing,
/// underline placement, pixel snapping, or painting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextUnderlineOffsetQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextUnderlineOffsetQualificationOutcome,
}

impl CssTextUnderlineOffsetQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextUnderlineOffsetQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollMarginTopValue {
    DirectLengthLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollMarginTopUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollMarginTopQualificationOutcome {
    Qualified(CssScrollMarginTopValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssScrollMarginTopUnsupportedReason),
}

/// One selected ordinary declaration's bounded `scroll-margin-top` qualification.
///
/// This profile qualifies direct unrestricted signed `<length>` evidence only.
/// It deliberately rejects Percentage tokens and performs no machine numeric
/// ordering, unit conversion, calculation evaluation, or scroll-snap geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssScrollMarginTopQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssScrollMarginTopQualificationOutcome,
}

impl CssScrollMarginTopQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssScrollMarginTopQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderTopWidthValue {
    Thin,
    Medium,
    Thick,
    DirectLengthLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderTopWidthUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderTopWidthQualificationOutcome {
    Qualified(CssBorderTopWidthValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssBorderTopWidthUnsupportedReason),
}

/// One selected ordinary declaration's bounded `border-top-width` qualification.
///
/// This profile composes the three direct `<line-width>` keywords with the
/// accepted direct `<length [0,∞]>` boundary. It performs no unit conversion,
/// numeric-function evaluation, shorthand expansion, or computed border width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssBorderTopWidthQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssBorderTopWidthQualificationOutcome,
}

impl CssBorderTopWidthQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssBorderTopWidthQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPerspectiveValue {
    None,
    DirectLengthLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPerspectiveUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPerspectiveQualificationOutcome {
    Qualified(CssPerspectiveValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssPerspectiveUnsupportedReason),
}

/// One selected ordinary declaration's bounded `perspective` qualification.
///
/// This profile qualifies `none`, direct unitless zero, and direct retained
/// CSS length Dimensions proven inside `[0,∞]`. It performs no unit conversion,
/// numeric-function evaluation, or computed perspective processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssPerspectiveQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssPerspectiveQualificationOutcome,
}

impl CssPerspectiveQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssPerspectiveQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollSnapAlignKeyword {
    None,
    Start,
    End,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollSnapAlignValue {
    Single(CssScrollSnapAlignKeyword),
    Pair {
        first: CssScrollSnapAlignKeyword,
        second: CssScrollSnapAlignKeyword,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollSnapAlignUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScrollSnapAlignQualificationOutcome {
    Qualified(CssScrollSnapAlignValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssScrollSnapAlignUnsupportedReason),
}

/// One selected ordinary declaration's bounded `scroll-snap-align`
/// qualification.
///
/// Authored one- and two-keyword forms remain distinct here. This observation
/// does not perform the property's computed-value pair completion or CSSOM
/// serialization canonicalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssScrollSnapAlignQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssScrollSnapAlignQualificationOutcome,
}

impl CssScrollSnapAlignQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssScrollSnapAlignQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssZIndexValue {
    Auto,
    DirectIntegerLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssZIndexUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssZIndexQualificationOutcome {
    Qualified(CssZIndexValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssZIndexUnsupportedReason),
}

/// One selected ordinary declaration's bounded `z-index` qualification.
///
/// This profile only qualifies direct authored `auto` and direct authored
/// integer literals. Exact integer spelling and all source provenance remain in
/// the structurally owning tokenizer and parser evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssZIndexQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssZIndexQualificationOutcome,
}

impl CssZIndexQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssZIndexQualificationOutcome {
        self.outcome
    }
}

/// Run-owned result for the currently selected bounded CSS value capabilities.
///
/// The exact Core-validated parser result is owned once here. Property-specific
/// observations stay typed and do not establish a generic property registry or
/// value-grammar language. This capability introduces no independent resource
/// or termination state; overall completion is exactly the upstream parser
/// completion.
#[derive(Debug, Clone)]
pub(crate) struct CssValueQualificationRunResult {
    upstream_parser_result: CssParserRunResult,
    direction_observations: Vec<CssDirectionQualificationObservation>,
    box_sizing_observations: Vec<CssBoxSizingQualificationObservation>,
    isolation_observations: Vec<CssIsolationQualificationObservation>,
    backface_visibility_observations: Vec<CssBackfaceVisibilityQualificationObservation>,
    order_observations: Vec<CssOrderQualificationObservation>,
    column_count_observations: Vec<CssColumnCountQualificationObservation>,
    flex_grow_observations: Vec<CssFlexGrowQualificationObservation>,
    flex_shrink_observations: Vec<CssFlexShrinkQualificationObservation>,
    opacity_observations: Vec<CssOpacityQualificationObservation>,
    shape_image_threshold_observations: Vec<CssShapeImageThresholdQualificationObservation>,
    shape_margin_observations: Vec<CssShapeMarginQualificationObservation>,
    line_height_observations: Vec<CssLineHeightQualificationObservation>,
    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,
    text_underline_offset_observations: Vec<CssTextUnderlineOffsetQualificationObservation>,
    scroll_margin_top_observations: Vec<CssScrollMarginTopQualificationObservation>,
    border_top_width_observations: Vec<CssBorderTopWidthQualificationObservation>,
    perspective_observations: Vec<CssPerspectiveQualificationObservation>,
    scroll_snap_align_observations: Vec<CssScrollSnapAlignQualificationObservation>,
    scroll_snap_stop_observations: Vec<CssScrollSnapStopQualificationObservation>,
    empty_cells_observations: Vec<CssEmptyCellsQualificationObservation>,
    text_decoration_style_observations: Vec<CssTextDecorationStyleQualificationObservation>,
    table_layout_observations: Vec<CssTableLayoutQualificationObservation>,
    border_collapse_observations: Vec<CssBorderCollapseQualificationObservation>,
    box_decoration_break_observations: Vec<CssBoxDecorationBreakQualificationObservation>,
    font_kerning_observations: Vec<CssFontKerningQualificationObservation>,
    font_synthesis_weight_observations: Vec<CssFontSynthesisWeightQualificationObservation>,
    font_variant_position_observations: Vec<CssFontVariantPositionQualificationObservation>,
    z_index_observations: Vec<CssZIndexQualificationObservation>,
}

impl CssValueQualificationRunResult {
    pub(crate) const fn upstream_parser_result(&self) -> &CssParserRunResult {
        &self.upstream_parser_result
    }

    pub(crate) fn direction_observations(&self) -> &[CssDirectionQualificationObservation] {
        &self.direction_observations
    }

    pub(crate) fn box_sizing_observations(&self) -> &[CssBoxSizingQualificationObservation] {
        &self.box_sizing_observations
    }

    pub(crate) fn isolation_observations(&self) -> &[CssIsolationQualificationObservation] {
        &self.isolation_observations
    }

    pub(crate) fn backface_visibility_observations(
        &self,
    ) -> &[CssBackfaceVisibilityQualificationObservation] {
        &self.backface_visibility_observations
    }

    pub(crate) fn order_observations(&self) -> &[CssOrderQualificationObservation] {
        &self.order_observations
    }

    pub(crate) fn column_count_observations(&self) -> &[CssColumnCountQualificationObservation] {
        &self.column_count_observations
    }

    pub(crate) fn flex_grow_observations(&self) -> &[CssFlexGrowQualificationObservation] {
        &self.flex_grow_observations
    }

    pub(crate) fn flex_shrink_observations(&self) -> &[CssFlexShrinkQualificationObservation] {
        &self.flex_shrink_observations
    }

    pub(crate) fn opacity_observations(&self) -> &[CssOpacityQualificationObservation] {
        &self.opacity_observations
    }

    pub(crate) fn shape_image_threshold_observations(
        &self,
    ) -> &[CssShapeImageThresholdQualificationObservation] {
        &self.shape_image_threshold_observations
    }

    pub(crate) fn shape_margin_observations(&self) -> &[CssShapeMarginQualificationObservation] {
        &self.shape_margin_observations
    }

    pub(crate) fn line_height_observations(&self) -> &[CssLineHeightQualificationObservation] {
        &self.line_height_observations
    }

    pub(crate) fn word_spacing_observations(&self) -> &[CssWordSpacingQualificationObservation] {
        &self.word_spacing_observations
    }

    pub(crate) fn text_underline_offset_observations(
        &self,
    ) -> &[CssTextUnderlineOffsetQualificationObservation] {
        &self.text_underline_offset_observations
    }

    pub(crate) fn scroll_margin_top_observations(
        &self,
    ) -> &[CssScrollMarginTopQualificationObservation] {
        &self.scroll_margin_top_observations
    }

    pub(crate) fn border_top_width_observations(
        &self,
    ) -> &[CssBorderTopWidthQualificationObservation] {
        &self.border_top_width_observations
    }

    pub(crate) fn perspective_observations(&self) -> &[CssPerspectiveQualificationObservation] {
        &self.perspective_observations
    }

    pub(crate) fn scroll_snap_align_observations(
        &self,
    ) -> &[CssScrollSnapAlignQualificationObservation] {
        &self.scroll_snap_align_observations
    }

    pub(crate) fn scroll_snap_stop_observations(
        &self,
    ) -> &[CssScrollSnapStopQualificationObservation] {
        &self.scroll_snap_stop_observations
    }

    pub(crate) fn empty_cells_observations(&self) -> &[CssEmptyCellsQualificationObservation] {
        &self.empty_cells_observations
    }

    pub(crate) fn text_decoration_style_observations(
        &self,
    ) -> &[CssTextDecorationStyleQualificationObservation] {
        &self.text_decoration_style_observations
    }

    pub(crate) fn table_layout_observations(&self) -> &[CssTableLayoutQualificationObservation] {
        &self.table_layout_observations
    }

    pub(crate) fn border_collapse_observations(
        &self,
    ) -> &[CssBorderCollapseQualificationObservation] {
        &self.border_collapse_observations
    }

    pub(crate) fn box_decoration_break_observations(
        &self,
    ) -> &[CssBoxDecorationBreakQualificationObservation] {
        &self.box_decoration_break_observations
    }

    pub(crate) fn font_kerning_observations(&self) -> &[CssFontKerningQualificationObservation] {
        &self.font_kerning_observations
    }

    pub(crate) fn font_synthesis_weight_observations(
        &self,
    ) -> &[CssFontSynthesisWeightQualificationObservation] {
        &self.font_synthesis_weight_observations
    }

    pub(crate) fn font_variant_position_observations(
        &self,
    ) -> &[CssFontVariantPositionQualificationObservation] {
        &self.font_variant_position_observations
    }

    pub(crate) fn z_index_observations(&self) -> &[CssZIndexQualificationObservation] {
        &self.z_index_observations
    }

    pub(crate) const fn execution_completion(&self) -> CssParserExecutionCompletion {
        self.upstream_parser_result.execution_completion()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssValueQualificationError {
    InternalInvariantFailure(CssValueQualificationInvariantViolation),
}

impl fmt::Display for CssValueQualificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS value qualification failure: {self:?}")
    }
}

impl Error for CssValueQualificationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssValueQualificationInvariantViolation {
    EvidenceSourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    EvidenceSourceContentMismatch {
        source_id: SourceId,
    },
    NonMonotonicEvidence {
        previous_start: usize,
        actual_start: usize,
    },
    LexicalItemSourceIdentityMismatch {
        index: usize,
        expected: SourceId,
        actual: SourceId,
    },
    EvidenceCutsLexicalItem {
        index: usize,
        item_start: usize,
        item_end: usize,
        evidence_start: usize,
        evidence_end: usize,
    },
    PropertyNameNotSingleIdentifier {
        occurrence_index: usize,
    },
}

/// Qualifies the selected ordinary declaration values in one already
/// Core-validated parser run.
///
/// The function consumes the parser result once to keep every produced
/// run-local observation structurally attached to the exact upstream
/// source/lifecycle evidence. Authored-invalid selected values are normal
/// semantic outcomes; only contradictions in already-retained evidence become
/// Rust errors.
pub(crate) fn run(
    parser_result: CssParserRunResult,
) -> Result<CssValueQualificationRunResult, CssValueQualificationError> {
    let (
        direction_observations,
        box_sizing_observations,
        isolation_observations,
        backface_visibility_observations,
        order_observations,
        column_count_observations,
        flex_grow_observations,
        flex_shrink_observations,
        opacity_observations,
        shape_image_threshold_observations,
        shape_margin_observations,
        line_height_observations,
        word_spacing_observations,
        text_underline_offset_observations,
        scroll_margin_top_observations,
        border_top_width_observations,
        perspective_observations,
        scroll_snap_align_observations,
        scroll_snap_stop_observations,
        empty_cells_observations,
        text_decoration_style_observations,
        table_layout_observations,
        border_collapse_observations,
        box_decoration_break_observations,
        font_kerning_observations,
        font_synthesis_weight_observations,
        font_variant_position_observations,
        z_index_observations,
    ) = {
        let tokenizer_result = parser_result.upstream_tokenizer_result();
        let mut cursor = LexicalWindowCursor::new(tokenizer_result);
        let mut direction_observations = Vec::new();
        let mut box_sizing_observations = Vec::new();
        let mut isolation_observations = Vec::new();
        let mut backface_visibility_observations = Vec::new();
        let mut order_observations = Vec::new();
        let mut column_count_observations = Vec::new();
        let mut flex_grow_observations = Vec::new();
        let mut flex_shrink_observations = Vec::new();
        let mut opacity_observations = Vec::new();
        let mut shape_image_threshold_observations = Vec::new();
        let mut shape_margin_observations = Vec::new();
        let mut line_height_observations = Vec::new();
        let mut word_spacing_observations = Vec::new();
        let mut text_underline_offset_observations = Vec::new();
        let mut scroll_margin_top_observations = Vec::new();
        let mut border_top_width_observations = Vec::new();
        let mut perspective_observations = Vec::new();
        let mut scroll_snap_align_observations = Vec::new();
        let mut scroll_snap_stop_observations = Vec::new();
        let mut empty_cells_observations = Vec::new();
        let mut text_decoration_style_observations = Vec::new();
        let mut table_layout_observations = Vec::new();
        let mut border_collapse_observations = Vec::new();
        let mut box_decoration_break_observations = Vec::new();
        let mut font_kerning_observations = Vec::new();
        let mut font_synthesis_weight_observations = Vec::new();
        let mut font_variant_position_observations = Vec::new();
        let mut z_index_observations = Vec::new();

        for (occurrence_index, occurrence) in parser_result.occurrences().iter().enumerate() {
            let property_range = cursor.window_for(occurrence.property_name())?;
            let property_items = &tokenizer_result.lexical_items()[property_range];
            let property_name = single_property_identifier(property_items, occurrence_index)?;

            if property_name.eq_ignore_ascii_case("direction") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                direction_observations.push(CssDirectionQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_direction_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("box-sizing") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                box_sizing_observations.push(CssBoxSizingQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_box_sizing_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("isolation") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                isolation_observations.push(CssIsolationQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_isolation_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("backface-visibility") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                backface_visibility_observations.push(
                    CssBackfaceVisibilityQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_backface_visibility_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("order") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                order_observations.push(CssOrderQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_order_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("column-count") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                column_count_observations.push(CssColumnCountQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_column_count_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("flex-grow") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                flex_grow_observations.push(CssFlexGrowQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_flex_grow_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("flex-shrink") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                flex_shrink_observations.push(CssFlexShrinkQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_flex_shrink_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("opacity") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                opacity_observations.push(CssOpacityQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_opacity_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("shape-image-threshold") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                shape_image_threshold_observations.push(
                    CssShapeImageThresholdQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_shape_image_threshold_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("shape-margin") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                shape_margin_observations.push(CssShapeMarginQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_shape_margin_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("line-height") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                line_height_observations.push(CssLineHeightQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_line_height_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("word-spacing") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                word_spacing_observations.push(CssWordSpacingQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_word_spacing_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-underline-offset") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_underline_offset_observations.push(
                    CssTextUnderlineOffsetQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_text_underline_offset_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("scroll-margin-top") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                scroll_margin_top_observations.push(CssScrollMarginTopQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_scroll_margin_top_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("border-top-width") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                border_top_width_observations.push(CssBorderTopWidthQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_border_top_width_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("perspective") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                perspective_observations.push(CssPerspectiveQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_perspective_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("z-index") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                z_index_observations.push(CssZIndexQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_z_index_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("scroll-snap-align") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                scroll_snap_align_observations.push(CssScrollSnapAlignQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_scroll_snap_align_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("scroll-snap-stop") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                scroll_snap_stop_observations.push(CssScrollSnapStopQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_scroll_snap_stop_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("empty-cells") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                empty_cells_observations.push(CssEmptyCellsQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_empty_cells_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-decoration-style") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_decoration_style_observations.push(
                    CssTextDecorationStyleQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_text_decoration_style_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("table-layout") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                table_layout_observations.push(CssTableLayoutQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_table_layout_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("border-collapse") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                border_collapse_observations.push(CssBorderCollapseQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_border_collapse_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("box-decoration-break") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                box_decoration_break_observations.push(
                    CssBoxDecorationBreakQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_box_decoration_break_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-kerning") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_kerning_observations.push(CssFontKerningQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_font_kerning_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-synthesis-weight") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_synthesis_weight_observations.push(
                    CssFontSynthesisWeightQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_font_synthesis_weight_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-variant-position") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_variant_position_observations.push(
                    CssFontVariantPositionQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_font_variant_position_value(value_items),
                    },
                );
            }
        }

        (
            direction_observations,
            box_sizing_observations,
            isolation_observations,
            backface_visibility_observations,
            order_observations,
            column_count_observations,
            flex_grow_observations,
            flex_shrink_observations,
            opacity_observations,
            shape_image_threshold_observations,
            shape_margin_observations,
            line_height_observations,
            word_spacing_observations,
            text_underline_offset_observations,
            scroll_margin_top_observations,
            border_top_width_observations,
            perspective_observations,
            scroll_snap_align_observations,
            scroll_snap_stop_observations,
            empty_cells_observations,
            text_decoration_style_observations,
            table_layout_observations,
            border_collapse_observations,
            box_decoration_break_observations,
            font_kerning_observations,
            font_synthesis_weight_observations,
            font_variant_position_observations,
            z_index_observations,
        )
    };

    Ok(CssValueQualificationRunResult {
        upstream_parser_result: parser_result,
        direction_observations,
        box_sizing_observations,
        isolation_observations,
        backface_visibility_observations,
        order_observations,
        column_count_observations,
        flex_grow_observations,
        flex_shrink_observations,
        opacity_observations,
        shape_image_threshold_observations,
        shape_margin_observations,
        line_height_observations,
        word_spacing_observations,
        text_underline_offset_observations,
        scroll_margin_top_observations,
        border_top_width_observations,
        perspective_observations,
        scroll_snap_align_observations,
        scroll_snap_stop_observations,
        empty_cells_observations,
        text_decoration_style_observations,
        table_layout_observations,
        border_collapse_observations,
        box_decoration_break_observations,
        font_kerning_observations,
        font_synthesis_weight_observations,
        font_variant_position_observations,
        z_index_observations,
    })
}

fn single_property_identifier(
    items: &[CssLexicalItem],
    occurrence_index: usize,
) -> Result<&str, CssValueQualificationError> {
    let mut identifier = None;

    for item in items {
        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
        if matches!(token.kind(), CssTokenKind::Whitespace) {
            continue;
        }

        match (identifier, token.kind()) {
            (None, CssTokenKind::Ident(value)) => identifier = Some(value.as_str()),
            _ => return Err(property_name_violation(occurrence_index)),
        }
    }

    identifier.ok_or_else(|| property_name_violation(occurrence_index))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssSingleKeywordValue<'a> {
    Identifier(&'a str),
    Invalid,
    UnsupportedFunction,
}

fn classify_single_keyword_value(items: &[CssLexicalItem]) -> CssSingleKeywordValue<'_> {
    let mut semantic_count = 0usize;
    let mut only_identifier = None;

    for item in items {
        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
        if matches!(token.kind(), CssTokenKind::Whitespace) {
            continue;
        }

        semantic_count += 1;
        match token.kind() {
            CssTokenKind::Ident(value) if semantic_count == 1 => {
                only_identifier = Some(value.as_str());
            }
            _ => {
                only_identifier = None;
            }
        }
    }

    // Arbitrary substitution functions make parse-time grammar validity
    // deferred wherever they occur. Generic `<whole-value>` functions cross
    // this slice's boundary only when that function occupies the entire value.
    if contains_deferred_substitution_function(items) || is_entire_whole_value_function(items) {
        return CssSingleKeywordValue::UnsupportedFunction;
    }

    if semantic_count != 1 {
        return CssSingleKeywordValue::Invalid;
    }

    only_identifier
        .map(CssSingleKeywordValue::Identifier)
        .unwrap_or(CssSingleKeywordValue::Invalid)
}

fn qualify_direction_value(items: &[CssLexicalItem]) -> CssDirectionQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssDirectionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssDirectionUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssDirectionQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier) if identifier.eq_ignore_ascii_case("ltr") => {
            CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Ltr)
        }
        CssSingleKeywordValue::Identifier(identifier) if identifier.eq_ignore_ascii_case("rtl") => {
            CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Rtl)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssDirectionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssDirectionUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssDirectionQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_box_sizing_value(items: &[CssLexicalItem]) -> CssBoxSizingQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssBoxSizingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBoxSizingUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssBoxSizingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("content-box") =>
        {
            CssBoxSizingQualificationOutcome::Qualified(CssBoxSizingValue::ContentBox)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("border-box") =>
        {
            CssBoxSizingQualificationOutcome::Qualified(CssBoxSizingValue::BorderBox)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssBoxSizingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBoxSizingUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssBoxSizingQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_isolation_value(items: &[CssLexicalItem]) -> CssIsolationQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssIsolationQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssIsolationUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssIsolationQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssIsolationQualificationOutcome::Qualified(CssIsolationValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("isolate") =>
        {
            CssIsolationQualificationOutcome::Qualified(CssIsolationValue::Isolate)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssIsolationQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssIsolationUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssIsolationQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_backface_visibility_value(
    items: &[CssLexicalItem],
) -> CssBackfaceVisibilityQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssBackfaceVisibilityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBackfaceVisibilityUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssBackfaceVisibilityQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("visible") =>
        {
            CssBackfaceVisibilityQualificationOutcome::Qualified(
                CssBackfaceVisibilityValue::Visible,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("hidden") =>
        {
            CssBackfaceVisibilityQualificationOutcome::Qualified(CssBackfaceVisibilityValue::Hidden)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssBackfaceVisibilityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBackfaceVisibilityUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssBackfaceVisibilityQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_scroll_snap_stop_value(
    items: &[CssLexicalItem],
) -> CssScrollSnapStopQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssScrollSnapStopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollSnapStopUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssScrollSnapStopQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssScrollSnapStopQualificationOutcome::Qualified(CssScrollSnapStopValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("always") =>
        {
            CssScrollSnapStopQualificationOutcome::Qualified(CssScrollSnapStopValue::Always)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssScrollSnapStopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollSnapStopUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssScrollSnapStopQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_empty_cells_value(items: &[CssLexicalItem]) -> CssEmptyCellsQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssEmptyCellsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssEmptyCellsUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssEmptyCellsQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("show") =>
        {
            CssEmptyCellsQualificationOutcome::Qualified(CssEmptyCellsValue::Show)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("hide") =>
        {
            CssEmptyCellsQualificationOutcome::Qualified(CssEmptyCellsValue::Hide)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssEmptyCellsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssEmptyCellsUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssEmptyCellsQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_text_decoration_style_value(
    items: &[CssLexicalItem],
) -> CssTextDecorationStyleQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssTextDecorationStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextDecorationStyleUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssTextDecorationStyleQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("solid") =>
        {
            CssTextDecorationStyleQualificationOutcome::Qualified(
                CssTextDecorationStyleValue::Solid,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("double") =>
        {
            CssTextDecorationStyleQualificationOutcome::Qualified(
                CssTextDecorationStyleValue::Double,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("dotted") =>
        {
            CssTextDecorationStyleQualificationOutcome::Qualified(
                CssTextDecorationStyleValue::Dotted,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("dashed") =>
        {
            CssTextDecorationStyleQualificationOutcome::Qualified(
                CssTextDecorationStyleValue::Dashed,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("wavy") =>
        {
            CssTextDecorationStyleQualificationOutcome::Qualified(CssTextDecorationStyleValue::Wavy)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssTextDecorationStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextDecorationStyleUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssTextDecorationStyleQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_table_layout_value(items: &[CssLexicalItem]) -> CssTableLayoutQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssTableLayoutQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTableLayoutUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssTableLayoutQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssTableLayoutQualificationOutcome::Qualified(CssTableLayoutValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("fixed") =>
        {
            CssTableLayoutQualificationOutcome::Qualified(CssTableLayoutValue::Fixed)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssTableLayoutQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTableLayoutUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssTableLayoutQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_border_collapse_value(
    items: &[CssLexicalItem],
) -> CssBorderCollapseQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssBorderCollapseQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderCollapseUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssBorderCollapseQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("separate") =>
        {
            CssBorderCollapseQualificationOutcome::Qualified(CssBorderCollapseValue::Separate)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("collapse") =>
        {
            CssBorderCollapseQualificationOutcome::Qualified(CssBorderCollapseValue::Collapse)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssBorderCollapseQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderCollapseUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssBorderCollapseQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_box_decoration_break_value(
    items: &[CssLexicalItem],
) -> CssBoxDecorationBreakQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssBoxDecorationBreakQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBoxDecorationBreakUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssBoxDecorationBreakQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("slice") =>
        {
            CssBoxDecorationBreakQualificationOutcome::Qualified(CssBoxDecorationBreakValue::Slice)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("clone") =>
        {
            CssBoxDecorationBreakQualificationOutcome::Qualified(CssBoxDecorationBreakValue::Clone)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssBoxDecorationBreakQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBoxDecorationBreakUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssBoxDecorationBreakQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_font_kerning_value(items: &[CssLexicalItem]) -> CssFontKerningQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssFontKerningQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontKerningUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssFontKerningQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssFontKerningQualificationOutcome::Qualified(CssFontKerningValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssFontKerningQualificationOutcome::Qualified(CssFontKerningValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssFontKerningQualificationOutcome::Qualified(CssFontKerningValue::None)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssFontKerningQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontKerningUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssFontKerningQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_font_synthesis_weight_value(
    items: &[CssLexicalItem],
) -> CssFontSynthesisWeightQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssFontSynthesisWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisWeightUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssFontSynthesisWeightQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssFontSynthesisWeightQualificationOutcome::Qualified(CssFontSynthesisWeightValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssFontSynthesisWeightQualificationOutcome::Qualified(CssFontSynthesisWeightValue::None)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssFontSynthesisWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisWeightUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssFontSynthesisWeightQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_font_variant_position_value(
    items: &[CssLexicalItem],
) -> CssFontVariantPositionQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssFontVariantPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantPositionUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssFontVariantPositionQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssFontVariantPositionQualificationOutcome::Qualified(
                CssFontVariantPositionValue::Normal,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if identifier.eq_ignore_ascii_case("sub") => {
            CssFontVariantPositionQualificationOutcome::Qualified(CssFontVariantPositionValue::Sub)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("super") =>
        {
            CssFontVariantPositionQualificationOutcome::Qualified(
                CssFontVariantPositionValue::Super,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssFontVariantPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantPositionUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssFontVariantPositionQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_order_value(items: &[CssLexicalItem]) -> CssOrderQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOrderUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOrderUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOrderUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssOrderQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssOrderQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number {
            number_type: CssNumberType::Integer,
            ..
        } => CssOrderQualificationOutcome::Qualified(CssOrderValue::DirectIntegerLiteral),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOrderUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssOrderQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_column_count_value(items: &[CssLexicalItem]) -> CssColumnCountQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssColumnCountQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssColumnCountUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssColumnCountQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssColumnCountUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssColumnCountQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssColumnCountUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssColumnCountQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssColumnCountQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto") => {
            CssColumnCountQualificationOutcome::Qualified(CssColumnCountValue::Auto)
        }
        CssTokenKind::Number {
            value,
            number_type: CssNumberType::Integer,
        } if is_positive_direct_integer(value) => {
            CssColumnCountQualificationOutcome::Qualified(CssColumnCountValue::DirectIntegerLiteral)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssColumnCountQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColumnCountUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssColumnCountQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn is_positive_direct_integer(value: &CssNumericValue) -> bool {
    if matches!(value.sign(), Some(CssNumberSign::Minus)) {
        return false;
    }

    value
        .decimal()
        .integer_digits()
        .bytes()
        .any(|digit| digit != b'0')
}

fn qualify_flex_grow_value(items: &[CssLexicalItem]) -> CssFlexGrowQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssFlexGrowQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFlexGrowUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssFlexGrowQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFlexGrowUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssFlexGrowQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFlexGrowUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssFlexGrowQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssFlexGrowQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { value, .. } if is_non_negative_direct_number(value) => {
            CssFlexGrowQualificationOutcome::Qualified(CssFlexGrowValue::DirectNumberLiteral)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssFlexGrowQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexGrowUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssFlexGrowQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn is_non_negative_direct_number(value: &CssNumericValue) -> bool {
    if !matches!(value.sign(), Some(CssNumberSign::Minus)) {
        return true;
    }

    let decimal = value.decimal();
    decimal.integer_digits().bytes().all(|digit| digit == b'0')
        && decimal.fraction_digits().bytes().all(|digit| digit == b'0')
}

fn qualify_flex_shrink_value(items: &[CssLexicalItem]) -> CssFlexShrinkQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssFlexShrinkQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFlexShrinkUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssFlexShrinkQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFlexShrinkUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssFlexShrinkQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFlexShrinkUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssFlexShrinkQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssFlexShrinkQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { value, .. } if is_non_negative_direct_number(value) => {
            CssFlexShrinkQualificationOutcome::Qualified(CssFlexShrinkValue::DirectNumberLiteral)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssFlexShrinkQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexShrinkUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssFlexShrinkQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_opacity_value(items: &[CssLexicalItem]) -> CssOpacityQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOpacityUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOpacityUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOpacityUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { .. } => {
            CssOpacityQualificationOutcome::Qualified(CssOpacityValue::DirectNumberLiteral)
        }
        CssTokenKind::Percentage { .. } => {
            CssOpacityQualificationOutcome::Qualified(CssOpacityValue::DirectPercentageLiteral)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssOpacityQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_shape_image_threshold_value(
    items: &[CssLexicalItem],
) -> CssShapeImageThresholdQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssShapeImageThresholdQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssShapeImageThresholdUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssShapeImageThresholdQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssShapeImageThresholdUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssShapeImageThresholdQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssShapeImageThresholdUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssShapeImageThresholdQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssShapeImageThresholdQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { .. } => CssShapeImageThresholdQualificationOutcome::Qualified(
            CssShapeImageThresholdValue::DirectNumberLiteral,
        ),
        CssTokenKind::Percentage { .. } => CssShapeImageThresholdQualificationOutcome::Qualified(
            CssShapeImageThresholdValue::DirectPercentageLiteral,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssShapeImageThresholdQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeImageThresholdUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssShapeImageThresholdQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_shape_margin_value(items: &[CssLexicalItem]) -> CssShapeMarginQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssShapeMarginQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssShapeMarginUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssShapeMarginQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssShapeMarginUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssShapeMarginQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssShapeMarginUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssShapeMarginQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssShapeMarginQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssShapeMarginQualificationOutcome::Qualified(CssShapeMarginValue::DirectLengthLiteral)
        }
        CssTokenKind::Dimension { value, unit, .. }
            if is_css_length_unit(unit) && is_non_negative_direct_number(value) =>
        {
            CssShapeMarginQualificationOutcome::Qualified(CssShapeMarginValue::DirectLengthLiteral)
        }
        CssTokenKind::Percentage { value } if is_non_negative_direct_number(value) => {
            CssShapeMarginQualificationOutcome::Qualified(
                CssShapeMarginValue::DirectPercentageLiteral,
            )
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssShapeMarginQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeMarginUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssShapeMarginQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_line_height_value(items: &[CssLexicalItem]) -> CssLineHeightQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssLineHeightQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssLineHeightUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssLineHeightQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssLineHeightUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssLineHeightQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssLineHeightUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssLineHeightQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssLineHeightQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("normal") => {
            CssLineHeightQualificationOutcome::Qualified(CssLineHeightValue::Normal)
        }
        CssTokenKind::Number { value, .. } if is_non_negative_direct_number(value) => {
            CssLineHeightQualificationOutcome::Qualified(CssLineHeightValue::DirectNumberLiteral)
        }
        CssTokenKind::Dimension { value, unit, .. }
            if is_css_length_unit(unit) && is_non_negative_direct_number(value) =>
        {
            CssLineHeightQualificationOutcome::Qualified(CssLineHeightValue::DirectLengthLiteral)
        }
        CssTokenKind::Percentage { value } if is_non_negative_direct_number(value) => {
            CssLineHeightQualificationOutcome::Qualified(
                CssLineHeightValue::DirectPercentageLiteral,
            )
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssLineHeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineHeightUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssLineHeightQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_word_spacing_value(items: &[CssLexicalItem]) -> CssWordSpacingQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssWordSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssWordSpacingUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssWordSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssWordSpacingUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssWordSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssWordSpacingUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssWordSpacingQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssWordSpacingQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("normal") => {
            CssWordSpacingQualificationOutcome::Qualified(CssWordSpacingValue::Normal)
        }
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssWordSpacingQualificationOutcome::Qualified(CssWordSpacingValue::DirectLengthLiteral)
        }
        CssTokenKind::Dimension { unit, .. } if is_css_length_unit(unit) => {
            CssWordSpacingQualificationOutcome::Qualified(CssWordSpacingValue::DirectLengthLiteral)
        }
        CssTokenKind::Percentage { .. } => CssWordSpacingQualificationOutcome::Qualified(
            CssWordSpacingValue::DirectPercentageLiteral,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssWordSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssWordSpacingUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssWordSpacingQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_text_underline_offset_value(
    items: &[CssLexicalItem],
) -> CssTextUnderlineOffsetQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTextUnderlineOffsetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextUnderlineOffsetUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTextUnderlineOffsetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextUnderlineOffsetUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssTextUnderlineOffsetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextUnderlineOffsetUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssTextUnderlineOffsetQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssTextUnderlineOffsetQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto") => {
            CssTextUnderlineOffsetQualificationOutcome::Qualified(CssTextUnderlineOffsetValue::Auto)
        }
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssTextUnderlineOffsetQualificationOutcome::Qualified(
                CssTextUnderlineOffsetValue::DirectLengthLiteral,
            )
        }
        CssTokenKind::Dimension { unit, .. } if is_css_length_unit(unit) => {
            CssTextUnderlineOffsetQualificationOutcome::Qualified(
                CssTextUnderlineOffsetValue::DirectLengthLiteral,
            )
        }
        CssTokenKind::Percentage { .. } => CssTextUnderlineOffsetQualificationOutcome::Qualified(
            CssTextUnderlineOffsetValue::DirectPercentageLiteral,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssTextUnderlineOffsetQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextUnderlineOffsetUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssTextUnderlineOffsetQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_scroll_margin_top_value(
    items: &[CssLexicalItem],
) -> CssScrollMarginTopQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssScrollMarginTopQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssScrollMarginTopUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssScrollMarginTopQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssScrollMarginTopUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssScrollMarginTopQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssScrollMarginTopUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssScrollMarginTopQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssScrollMarginTopQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssScrollMarginTopQualificationOutcome::Qualified(
                CssScrollMarginTopValue::DirectLengthLiteral,
            )
        }
        CssTokenKind::Dimension { unit, .. } if is_css_length_unit(unit) => {
            CssScrollMarginTopQualificationOutcome::Qualified(
                CssScrollMarginTopValue::DirectLengthLiteral,
            )
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssScrollMarginTopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollMarginTopUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssScrollMarginTopQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_border_top_width_value(
    items: &[CssLexicalItem],
) -> CssBorderTopWidthQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssBorderTopWidthQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssBorderTopWidthUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssBorderTopWidthQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssBorderTopWidthUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssBorderTopWidthQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssBorderTopWidthUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssBorderTopWidthQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssBorderTopWidthQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("thin") => {
            CssBorderTopWidthQualificationOutcome::Qualified(CssBorderTopWidthValue::Thin)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("medium") => {
            CssBorderTopWidthQualificationOutcome::Qualified(CssBorderTopWidthValue::Medium)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("thick") => {
            CssBorderTopWidthQualificationOutcome::Qualified(CssBorderTopWidthValue::Thick)
        }
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssBorderTopWidthQualificationOutcome::Qualified(
                CssBorderTopWidthValue::DirectLengthLiteral,
            )
        }
        CssTokenKind::Dimension { value, unit, .. }
            if is_css_length_unit(unit) && is_non_negative_direct_number(value) =>
        {
            CssBorderTopWidthQualificationOutcome::Qualified(
                CssBorderTopWidthValue::DirectLengthLiteral,
            )
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssBorderTopWidthQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderTopWidthUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssBorderTopWidthQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_perspective_value(items: &[CssLexicalItem]) -> CssPerspectiveQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssPerspectiveQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssPerspectiveUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssPerspectiveQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssPerspectiveUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssPerspectiveQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssPerspectiveUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssPerspectiveQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssPerspectiveQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("none") => {
            CssPerspectiveQualificationOutcome::Qualified(CssPerspectiveValue::None)
        }
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssPerspectiveQualificationOutcome::Qualified(CssPerspectiveValue::DirectLengthLiteral)
        }
        CssTokenKind::Dimension { value, unit, .. }
            if is_css_length_unit(unit) && is_non_negative_direct_number(value) =>
        {
            CssPerspectiveQualificationOutcome::Qualified(CssPerspectiveValue::DirectLengthLiteral)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssPerspectiveQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPerspectiveUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssPerspectiveQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn is_direct_zero_numeric_value(value: &CssNumericValue) -> bool {
    let decimal = value.decimal();
    decimal.integer_digits().bytes().all(|digit| digit == b'0')
        && decimal.fraction_digits().bytes().all(|digit| digit == b'0')
}

fn is_css_length_unit(unit: &str) -> bool {
    [
        "cm", "mm", "q", "in", "pt", "pc", "px", "em", "rem", "ex", "rex", "cap", "rcap", "ch",
        "rch", "ic", "ric", "lh", "rlh", "vw", "vh", "vi", "vb", "vmin", "vmax", "svw", "svh",
        "svi", "svb", "svmin", "svmax", "lvw", "lvh", "lvi", "lvb", "lvmin", "lvmax", "dvw", "dvh",
        "dvi", "dvb", "dvmin", "dvmax", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax",
    ]
    .iter()
    .any(|length_unit| unit.eq_ignore_ascii_case(length_unit))
}

fn qualify_z_index_value(items: &[CssLexicalItem]) -> CssZIndexQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssZIndexQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssZIndexUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssZIndexQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssZIndexUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssZIndexQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssZIndexUnsupportedReason::FunctionValue,
        );
    }

    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssZIndexQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssZIndexQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto") => {
            CssZIndexQualificationOutcome::Qualified(CssZIndexValue::Auto)
        }
        CssTokenKind::Number {
            number_type: CssNumberType::Integer,
            ..
        } => CssZIndexQualificationOutcome::Qualified(CssZIndexValue::DirectIntegerLiteral),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssZIndexQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssZIndexUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssZIndexQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_scroll_snap_align_value(
    items: &[CssLexicalItem],
) -> CssScrollSnapAlignQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssScrollSnapAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssScrollSnapAlignUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssScrollSnapAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssScrollSnapAlignUnsupportedReason::WholeValueFunction,
        );
    }

    let tokens: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some(token)
            }
            _ => None,
        })
        .collect();

    match tokens.as_slice() {
        [token] => match token.kind() {
            CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
                CssScrollSnapAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssScrollSnapAlignUnsupportedReason::CssWideKeyword,
                )
            }
            CssTokenKind::Ident(identifier) => scroll_snap_align_keyword(identifier)
                .map(|keyword| {
                    CssScrollSnapAlignQualificationOutcome::Qualified(
                        CssScrollSnapAlignValue::Single(keyword),
                    )
                })
                .unwrap_or(CssScrollSnapAlignQualificationOutcome::InvalidForSelectedValueGrammar),
            _ => CssScrollSnapAlignQualificationOutcome::InvalidForSelectedValueGrammar,
        },
        [first, second] => match (first.kind(), second.kind()) {
            (CssTokenKind::Ident(first), CssTokenKind::Ident(second)) => {
                match (
                    scroll_snap_align_keyword(first),
                    scroll_snap_align_keyword(second),
                ) {
                    (Some(first), Some(second)) => {
                        CssScrollSnapAlignQualificationOutcome::Qualified(
                            CssScrollSnapAlignValue::Pair { first, second },
                        )
                    }
                    _ => CssScrollSnapAlignQualificationOutcome::InvalidForSelectedValueGrammar,
                }
            }
            _ => CssScrollSnapAlignQualificationOutcome::InvalidForSelectedValueGrammar,
        },
        _ => CssScrollSnapAlignQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn scroll_snap_align_keyword(identifier: &str) -> Option<CssScrollSnapAlignKeyword> {
    if identifier.eq_ignore_ascii_case("none") {
        return Some(CssScrollSnapAlignKeyword::None);
    }
    if identifier.eq_ignore_ascii_case("start") {
        return Some(CssScrollSnapAlignKeyword::Start);
    }
    if identifier.eq_ignore_ascii_case("end") {
        return Some(CssScrollSnapAlignKeyword::End);
    }
    if identifier.eq_ignore_ascii_case("center") {
        return Some(CssScrollSnapAlignKeyword::Center);
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssValueBlockCloser {
    Parenthesis,
    SquareBracket,
    CurlyBracket,
}

fn entire_function_name(items: &[CssLexicalItem]) -> Option<&str> {
    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let first = tokens.next()?;
    let CssTokenKind::Function(name) = first.kind() else {
        return None;
    };

    let mut block_stack = vec![CssValueBlockCloser::Parenthesis];
    for token in tokens {
        if block_stack.is_empty() {
            return None;
        }

        match token.kind() {
            CssTokenKind::Function(_) | CssTokenKind::LeftParenthesis => {
                block_stack.push(CssValueBlockCloser::Parenthesis);
            }
            CssTokenKind::LeftSquareBracket => {
                block_stack.push(CssValueBlockCloser::SquareBracket);
            }
            CssTokenKind::LeftCurlyBracket => {
                block_stack.push(CssValueBlockCloser::CurlyBracket);
            }
            CssTokenKind::RightParenthesis
                if block_stack.last() == Some(&CssValueBlockCloser::Parenthesis) =>
            {
                block_stack.pop();
            }
            CssTokenKind::RightSquareBracket
                if block_stack.last() == Some(&CssValueBlockCloser::SquareBracket) =>
            {
                block_stack.pop();
            }
            CssTokenKind::RightCurlyBracket
                if block_stack.last() == Some(&CssValueBlockCloser::CurlyBracket) =>
            {
                block_stack.pop();
            }
            _ => {}
        }
    }

    Some(name)
}

fn is_entire_whole_value_function(items: &[CssLexicalItem]) -> bool {
    entire_function_name(items).is_some_and(is_whole_value_function)
}

fn contains_deferred_substitution_function(items: &[CssLexicalItem]) -> bool {
    items.iter().any(|item| {
        let CssLexicalItem::SemanticToken(token) = item else {
            return false;
        };
        let CssTokenKind::Function(name) = token.kind() else {
            return false;
        };
        is_deferred_substitution_function(name)
    })
}

fn is_deferred_substitution_function(name: &str) -> bool {
    name.starts_with("--")
        || [
            "var",
            "env",
            "attr",
            "if",
            "inherit",
            "ident",
            "random-item",
        ]
        .iter()
        .any(|function| name.eq_ignore_ascii_case(function))
}

fn is_whole_value_function(name: &str) -> bool {
    ["first-valid", "cycle", "interpolate"]
        .iter()
        .any(|function| name.eq_ignore_ascii_case(function))
}

fn is_css_wide_keyword(identifier: &str) -> bool {
    [
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
        "revert-rule",
    ]
    .iter()
    .any(|keyword| identifier.eq_ignore_ascii_case(keyword))
}

fn property_name_violation(occurrence_index: usize) -> CssValueQualificationError {
    CssValueQualificationError::InternalInvariantFailure(
        CssValueQualificationInvariantViolation::PropertyNameNotSingleIdentifier {
            occurrence_index,
        },
    )
}

struct LexicalWindowCursor<'a> {
    tokenizer_result: &'a CssTokenizerRunResult,
    next_index: usize,
    previous_start: Option<usize>,
}

impl<'a> LexicalWindowCursor<'a> {
    const fn new(tokenizer_result: &'a CssTokenizerRunResult) -> Self {
        Self {
            tokenizer_result,
            next_index: 0,
            previous_start: None,
        }
    }

    fn window_for(
        &mut self,
        evidence: &SourceAnchor,
    ) -> Result<Range<usize>, CssValueQualificationError> {
        let expected = self.tokenizer_result.source_id();
        let actual = evidence.source_id();
        if expected != actual {
            return Err(invariant(
                CssValueQualificationInvariantViolation::EvidenceSourceIdentityMismatch {
                    expected,
                    actual,
                },
            ));
        }
        if !self
            .tokenizer_result
            .processed_prefix()
            .retains_exact_source(evidence)
        {
            return Err(invariant(
                CssValueQualificationInvariantViolation::EvidenceSourceContentMismatch {
                    source_id: expected,
                },
            ));
        }

        if let Some(previous_start) = self.previous_start
            && evidence.range().start() < previous_start
        {
            return Err(invariant(
                CssValueQualificationInvariantViolation::NonMonotonicEvidence {
                    previous_start,
                    actual_start: evidence.range().start(),
                },
            ));
        }
        self.previous_start = Some(evidence.range().start());

        let items = self.tokenizer_result.lexical_items();
        while self.next_index < items.len() {
            let item = &items[self.next_index];
            validate_item_source(item, self.next_index, expected)?;
            let range = item.source().range();
            if range.end() <= evidence.range().start() {
                self.next_index += 1;
                continue;
            }
            if range.start() < evidence.range().start() {
                return Err(cut_violation(self.next_index, item, evidence));
            }
            break;
        }

        let start = self.next_index;
        let mut end = start;
        while end < items.len() {
            let item = &items[end];
            validate_item_source(item, end, expected)?;
            let range = item.source().range();
            if range.start() >= evidence.range().end() {
                break;
            }
            if range.end() > evidence.range().end() {
                return Err(cut_violation(end, item, evidence));
            }
            end += 1;
        }

        self.next_index = end;
        Ok(start..end)
    }
}

fn validate_item_source(
    item: &CssLexicalItem,
    index: usize,
    expected: SourceId,
) -> Result<(), CssValueQualificationError> {
    let actual = item.source().source_id();
    if actual != expected {
        return Err(invariant(
            CssValueQualificationInvariantViolation::LexicalItemSourceIdentityMismatch {
                index,
                expected,
                actual,
            },
        ));
    }
    Ok(())
}

fn cut_violation(
    index: usize,
    item: &CssLexicalItem,
    evidence: &SourceAnchor,
) -> CssValueQualificationError {
    invariant(
        CssValueQualificationInvariantViolation::EvidenceCutsLexicalItem {
            index,
            item_start: item.source().range().start(),
            item_end: item.source().range().end(),
            evidence_start: evidence.range().start(),
            evidence_end: evidence.range().end(),
        },
    )
}

fn invariant(violation: CssValueQualificationInvariantViolation) -> CssValueQualificationError {
    CssValueQualificationError::InternalInvariantFailure(violation)
}
