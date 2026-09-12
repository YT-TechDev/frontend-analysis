//! Bounded declaration-value qualification for selected post-freeze CSS
//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483/#485/#487/#489/#491/#493/#495/#497/#499/#501/#503/#505/#508/#510/#512/#514/#516/#518/#520/#522/#524/#526/#528/#530/#532/#534/#536/#538/#541/#546/#549/#551/#553/#555/#559/#561/#596/#598/#600/#602/#604/#606/#608/#610).
//!
//! This module consumes only the already Core-validated parser result and its
//! retained tokenizer evidence. It does not search or decode raw source,
//! retokenize declaration fragments, mutate parser evidence, or claim cascade,
//! inheritance, computed-value, CSSOM, DOM, layout, or browser-runtime semantics.

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::ops::Range;

use crate::{SourceAnchor, SourceId};

use super::declaration::CssDeclarationPlacement;
use super::parser::result::{CssParserExecutionCompletion, CssParserRunResult};
use super::token::{
    CssDecimalExponent, CssExponentSign, CssLexicalItem, CssNumberSign, CssNumberType,
    CssNumericValue, CssToken, CssTokenKind,
};
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
pub(crate) enum CssListStylePositionValue {
    Inside,
    Outside,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssListStylePositionUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssListStylePositionQualificationOutcome {
    Qualified(CssListStylePositionValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssListStylePositionUnsupportedReason),
}

/// One selected ordinary declaration's `list-style-position` value
/// qualification.
///
/// As with direction/box-sizing observations, placement and
/// `occurrence_index` remain run-local references into the exact parser
/// result structurally owned by the enclosing
/// [`CssValueQualificationRunResult`]. This authored keyword identity is
/// distinct from list-item applicability, `::marker` generation, marker
/// attachment/geometry, and any other downstream layout semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssListStylePositionQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssListStylePositionQualificationOutcome,
}

impl CssListStylePositionQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssListStylePositionQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStrokeLinecapValue {
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStrokeLinecapUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStrokeLinecapQualificationOutcome {
    Qualified(CssStrokeLinecapValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssStrokeLinecapUnsupportedReason),
}

/// One selected ordinary declaration's `stroke-linecap` value
/// qualification.
///
/// As with direction/box-sizing/list-style-position observations,
/// placement and `occurrence_index` remain run-local references into the
/// exact parser result structurally owned by the enclosing
/// [`CssValueQualificationRunResult`]. This authored keyword identity is
/// distinct from SVG presentation-attribute semantics, cap geometry
/// construction, dash/marker interaction, and any other downstream
/// painting or rendering semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssStrokeLinecapQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssStrokeLinecapQualificationOutcome,
}

impl CssStrokeLinecapQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssStrokeLinecapQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStrokeOpacityValue {
    DirectNumberLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStrokeOpacityUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStrokeOpacityQualificationOutcome {
    Qualified(CssStrokeOpacityValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssStrokeOpacityUnsupportedReason),
}

/// One selected ordinary declaration's bounded `stroke-opacity`
/// qualification.
///
/// This profile reuses the accepted direct authored `<opacity-value>`
/// Number/Percentage boundary already used by `opacity`/`fill-opacity`.
/// Out-of-range authored values remain qualified; clamping,
/// percentage-to-number conversion, SVG painting/applicability, CSSOM, and
/// computed/used values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssStrokeOpacityQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssStrokeOpacityQualificationOutcome,
}

impl CssStrokeOpacityQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssStrokeOpacityQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStopOpacityValue {
    DirectNumberLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStopOpacityUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssStopOpacityQualificationOutcome {
    Qualified(CssStopOpacityValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssStopOpacityUnsupportedReason),
}

/// One selected ordinary declaration's bounded `stop-opacity`
/// qualification.
///
/// This profile reuses the accepted direct authored `<opacity-value>`
/// Number/Percentage boundary already used by `opacity`/`fill-opacity`/
/// `stroke-opacity`. Out-of-range authored values remain qualified;
/// clamping, percentage-to-number conversion, SVG gradient/stop
/// applicability, CSSOM, and computed/used values remain outside this
/// slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssStopOpacityQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssStopOpacityQualificationOutcome,
}

impl CssStopOpacityQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssStopOpacityQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFloodOpacityValue {
    DirectNumberLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFloodOpacityUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFloodOpacityQualificationOutcome {
    Qualified(CssFloodOpacityValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFloodOpacityUnsupportedReason),
}

/// One selected ordinary declaration's bounded `flood-opacity`
/// qualification.
///
/// This profile reuses the accepted direct authored `<opacity-value>`
/// Number/Percentage boundary already used by `opacity`/`fill-opacity`/
/// `stroke-opacity`/`stop-opacity`. Out-of-range authored values remain
/// qualified; clamping, percentage-to-number conversion, filter primitive
/// (`feFlood`/`feDropShadow`) applicability, CSSOM, and computed/used
/// values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFloodOpacityQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFloodOpacityQualificationOutcome,
}

impl CssFloodOpacityQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFloodOpacityQualificationOutcome {
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
pub(crate) enum CssFontSynthesisSmallCapsValue {
    Auto,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontSynthesisSmallCapsUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontSynthesisSmallCapsQualificationOutcome {
    Qualified(CssFontSynthesisSmallCapsValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontSynthesisSmallCapsUnsupportedReason),
}

/// One selected ordinary declaration's bounded `font-synthesis-small-caps`
/// qualification.
///
/// This profile qualifies only direct `auto | none` authored keyword evidence.
/// Font selection, glyph synthesis, casing transformation, OpenType feature
/// execution, shaping, rendering, and used-value processing remain outside this
/// slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontSynthesisSmallCapsQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontSynthesisSmallCapsQualificationOutcome,
}

impl CssFontSynthesisSmallCapsQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontSynthesisSmallCapsQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontSynthesisPositionValue {
    Auto,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontSynthesisPositionUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontSynthesisPositionQualificationOutcome {
    Qualified(CssFontSynthesisPositionValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontSynthesisPositionUnsupportedReason),
}

/// One selected ordinary declaration's bounded `font-synthesis-position`
/// qualification.
///
/// This profile qualifies only direct `auto | none` authored keyword evidence.
/// Simulated sub/sup glyph synthesis, OpenType `subs`/`sups` execution,
/// font fallback, synthetic sizing or positioning, baseline/line-box layout,
/// and used-value processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontSynthesisPositionQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontSynthesisPositionQualificationOutcome,
}

impl CssFontSynthesisPositionQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontSynthesisPositionQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantEmojiValue {
    Normal,
    Text,
    Emoji,
    Unicode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantEmojiUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantEmojiQualificationOutcome {
    Qualified(CssFontVariantEmojiValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontVariantEmojiUnsupportedReason),
}

/// One selected ordinary declaration's bounded `font-variant-emoji`
/// qualification.
///
/// This profile qualifies only direct `normal | text | emoji | unicode`
/// authored keyword evidence. Unicode emoji classification, variation
/// selector or ZWJ processing, font fallback, glyph shaping/rendering,
/// and used-value processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontVariantEmojiQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontVariantEmojiQualificationOutcome,
}

impl CssFontVariantEmojiQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontVariantEmojiQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantCapsValue {
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantCapsUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantCapsQualificationOutcome {
    Qualified(CssFontVariantCapsValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontVariantCapsUnsupportedReason),
}

/// One selected ordinary declaration's bounded `font-variant-caps`
/// qualification.
///
/// This profile qualifies only direct
/// `normal | small-caps | all-small-caps | petite-caps | all-petite-caps |
/// unicase | titling-caps` authored keyword evidence. OpenType feature
/// selection, small/petite-caps synthesis, case conversion, font fallback,
/// glyph shaping/rendering, and used-value processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontVariantCapsQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontVariantCapsQualificationOutcome,
}

impl CssFontVariantCapsQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontVariantCapsQualificationOutcome {
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
pub(crate) enum CssFontWeightValue {
    Normal,
    Bold,
    Bolder,
    Lighter,
    DirectNumberLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontWeightUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontWeightQualificationOutcome {
    Qualified(CssFontWeightValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontWeightUnsupportedReason),
}

/// One selected ordinary declaration's bounded `font-weight` qualification.
///
/// This profile qualifies the direct authored keyword branches and direct
/// Number tokens proven exactly inside `[1,1000]`. Exact authored numeric
/// spelling remains in the structurally owning tokenizer evidence. Relative
/// weight resolution, font matching, variation axes, synthesis, calculations,
/// percentages, descriptors, and computed/used values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontWeightQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontWeightQualificationOutcome,
}

impl CssFontWeightQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontWeightQualificationOutcome {
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
pub(crate) enum CssFillOpacityValue {
    DirectNumberLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFillOpacityUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFillOpacityQualificationOutcome {
    Qualified(CssFillOpacityValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFillOpacityUnsupportedReason),
}

/// One selected ordinary declaration's bounded `fill-opacity` qualification.
///
/// This profile reuses the accepted direct authored `<opacity-value>`
/// Number/Percentage boundary. Out-of-range authored values remain qualified;
/// clamping, SVG painting/applicability, CSSOM, animation, and computed/used
/// values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFillOpacityQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFillOpacityQualificationOutcome,
}

impl CssFillOpacityQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFillOpacityQualificationOutcome {
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
pub(crate) enum CssLineBreakValue {
    Auto,
    Loose,
    Normal,
    Strict,
    Anywhere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLineBreakUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLineBreakQualificationOutcome {
    Qualified(CssLineBreakValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssLineBreakUnsupportedReason),
}

/// One selected ordinary declaration's bounded `line-break` qualification.
///
/// This profile qualifies only direct
/// `auto | loose | normal | strict | anywhere` authored keyword evidence.
/// Unicode line-breaking classes, UAX #14 processing, writing-system/language
/// tailoring, CJK punctuation behavior, soft-wrap generation, shaping, line
/// layout, and intrinsic sizing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssLineBreakQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssLineBreakQualificationOutcome,
}

impl CssLineBreakQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssLineBreakQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPrintColorAdjustValue {
    Economy,
    Exact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPrintColorAdjustUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPrintColorAdjustQualificationOutcome {
    Qualified(CssPrintColorAdjustValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssPrintColorAdjustUnsupportedReason),
}

/// One selected ordinary declaration's bounded `print-color-adjust`
/// qualification.
///
/// This profile qualifies only direct `economy | exact` authored keyword
/// evidence. Printer/device behavior, ink-economy execution, actual color
/// rewriting, user preferences, viewport propagation, and printing/rendering
/// behavior remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssPrintColorAdjustQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssPrintColorAdjustQualificationOutcome,
}

impl CssPrintColorAdjustQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssPrintColorAdjustQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverflowWrapValue {
    Normal,
    BreakWord,
    Anywhere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverflowWrapUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverflowWrapQualificationOutcome {
    Qualified(CssOverflowWrapValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOverflowWrapUnsupportedReason),
}

/// One selected ordinary declaration's bounded `overflow-wrap` qualification.
///
/// This profile qualifies only direct `normal | break-word | anywhere` authored
/// keyword evidence. The legacy `word-wrap` alias, line-breaking execution,
/// soft-wrap generation, intrinsic sizing, shaping, and line layout remain
/// outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOverflowWrapQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOverflowWrapQualificationOutcome,
}

impl CssOverflowWrapQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOverflowWrapQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssUnicodeBidiValue {
    Normal,
    Embed,
    Isolate,
    BidiOverride,
    IsolateOverride,
    Plaintext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssUnicodeBidiUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssUnicodeBidiQualificationOutcome {
    Qualified(CssUnicodeBidiValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssUnicodeBidiUnsupportedReason),
}

/// One selected ordinary declaration's bounded `unicode-bidi` qualification.
///
/// This profile qualifies only direct
/// `normal | embed | isolate | bidi-override | isolate-override | plaintext`
/// authored keyword evidence. Unicode Bidirectional Algorithm execution,
/// embedding-level resolution, isolate/override processing, base-direction
/// inference, inline reordering, ruby interaction, and layout remain outside
/// this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssUnicodeBidiQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssUnicodeBidiQualificationOutcome,
}

impl CssUnicodeBidiQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssUnicodeBidiQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMaskTypeValue {
    Luminance,
    Alpha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMaskTypeUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMaskTypeQualificationOutcome {
    Qualified(CssMaskTypeValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssMaskTypeUnsupportedReason),
}

/// One selected ordinary declaration's bounded `mask-type` qualification.
///
/// This profile qualifies only direct `luminance | alpha` authored keyword
/// evidence. SVG `<mask>` applicability, element identity, mask rendering and
/// compositing, luminance calculation, alpha extraction, `mask-mode`
/// interaction, and computed/used-value processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssMaskTypeQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssMaskTypeQualificationOutcome,
}

impl CssMaskTypeQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssMaskTypeQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColorInterpolationFiltersValue {
    Auto,
    Srgb,
    LinearRgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColorInterpolationFiltersUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColorInterpolationFiltersQualificationOutcome {
    Qualified(CssColorInterpolationFiltersValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssColorInterpolationFiltersUnsupportedReason),
}

/// One selected ordinary declaration's bounded `color-interpolation-filters`
/// qualification.
///
/// This profile qualifies only direct `auto | sRGB | linearRGB` authored
/// keyword evidence. Filter-primitive applicability, SVG element identity,
/// filter graph execution, sRGB/linear-light conversion, gamma processing,
/// pixel interpolation, wide-gamut handling, and computed/used-value
/// processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssColorInterpolationFiltersQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssColorInterpolationFiltersQualificationOutcome,
}

impl CssColorInterpolationFiltersQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssColorInterpolationFiltersQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeRenderingValue {
    Auto,
    OptimizeSpeed,
    CrispEdges,
    GeometricPrecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeRenderingUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssShapeRenderingQualificationOutcome {
    Qualified(CssShapeRenderingValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssShapeRenderingUnsupportedReason),
}

/// One selected ordinary declaration's bounded `shape-rendering` qualification.
///
/// This profile qualifies only direct
/// `auto | optimizeSpeed | crispEdges | geometricPrecision` authored keyword
/// evidence. SVG shape applicability, presentation-attribute semantics,
/// rendering-hint execution, anti-aliasing, edge snapping, geometric-precision
/// algorithms, painting, and computed/used-value processing remain outside this
/// slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssShapeRenderingQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssShapeRenderingQualificationOutcome,
}

impl CssShapeRenderingQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssShapeRenderingQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextRenderingValue {
    Auto,
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextRenderingUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextRenderingQualificationOutcome {
    Qualified(CssTextRenderingValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextRenderingUnsupportedReason),
}

/// One selected ordinary declaration's bounded `text-rendering` qualification.
///
/// This profile qualifies only direct
/// `auto | optimizeSpeed | optimizeLegibility | geometricPrecision` authored
/// keyword evidence. Text shaping, kerning and ligature execution, font
/// selection, glyph rasterization, anti-aliasing, rendering-hint execution, SVG
/// text applicability, presentation-attribute semantics, painting, and
/// computed/used-value processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextRenderingQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextRenderingQualificationOutcome,
}

impl CssTextRenderingQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextRenderingQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextAnchorValue {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextAnchorUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextAnchorQualificationOutcome {
    Qualified(CssTextAnchorValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextAnchorUnsupportedReason),
}

/// One selected ordinary declaration's bounded `text-anchor` qualification.
///
/// This profile qualifies only direct `start | middle | end` authored keyword
/// evidence. SVG text-content applicability and identity, presentation-attribute
/// semantics, direction/writing-mode resolution, bidi processing, text-chunk
/// construction, glyph shaping, anchoring-position calculation, textPath
/// behavior, layout, painting, and computed/used-value processing remain outside
/// this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextAnchorQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextAnchorQualificationOutcome,
}

impl CssTextAnchorQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextAnchorQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssForcedColorAdjustValue {
    Auto,
    None,
    PreserveParentColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssForcedColorAdjustUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssForcedColorAdjustQualificationOutcome {
    Qualified(CssForcedColorAdjustValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssForcedColorAdjustUnsupportedReason),
}

/// One selected ordinary declaration's bounded `forced-color-adjust`
/// qualification.
///
/// This profile qualifies only direct `auto | none | preserve-parent-color`
/// authored keyword evidence. Forced-colors mode execution, OS accessibility
/// state, color replacement/system-color resolution, parent-color lookup,
/// inheritance/propagation execution, rendering, and computed/used-value
/// processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssForcedColorAdjustQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssForcedColorAdjustQualificationOutcome,
}

impl CssForcedColorAdjustQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssForcedColorAdjustQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextAlignLastValue {
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
    MatchParent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextAlignLastUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextAlignLastQualificationOutcome {
    Qualified(CssTextAlignLastValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextAlignLastUnsupportedReason),
}

/// One selected ordinary declaration's bounded `text-align-last` qualification.
///
/// This profile qualifies only direct
/// `auto | start | end | left | right | center | justify | match-parent`
/// authored keyword evidence. Last-line construction/alignment execution,
/// direction/writing-mode resolution, `match-parent` computed-value
/// transformation, bidi processing, justification, layout, rendering, and
/// computed/used-value processing remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextAlignLastQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextAlignLastQualificationOutcome,
}

impl CssTextAlignLastQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextAlignLastQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMathStyleValue {
    Normal,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMathStyleUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMathStyleQualificationOutcome {
    Qualified(CssMathStyleValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssMathStyleUnsupportedReason),
}

/// One selected ordinary declaration's bounded `math-style` qualification.
///
/// This profile qualifies only the direct authored `normal | compact`
/// keyword grammar. Math layout, inherited descendant behavior, MathML
/// attribute mapping, and computed/used values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssMathStyleQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssMathStyleQualificationOutcome,
}

impl CssMathStyleQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssMathStyleQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMathShiftValue {
    Normal,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMathShiftUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssMathShiftQualificationOutcome {
    Qualified(CssMathShiftValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssMathShiftUnsupportedReason),
}

/// One selected ordinary declaration's bounded `math-shift` qualification.
///
/// This profile qualifies only the direct authored `normal | compact`
/// keyword grammar. Script placement, cramped layout, MathML user-agent
/// stylesheet behavior, and computed/used values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssMathShiftQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssMathShiftQualificationOutcome,
}

impl CssMathShiftQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssMathShiftQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyAlignValue {
    Start,
    Center,
    SpaceBetween,
    SpaceAround,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyAlignUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyAlignQualificationOutcome {
    Qualified(CssRubyAlignValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssRubyAlignUnsupportedReason),
}

/// One selected ordinary declaration's bounded `ruby-align` qualification.
///
/// This profile qualifies only the direct authored
/// `start | center | space-between | space-around` keyword grammar.
/// Ruby box layout, content distribution/justification, language-dependent
/// behavior, bopomofo placement, and computed/used values remain outside
/// this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssRubyAlignQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssRubyAlignQualificationOutcome,
}

impl CssRubyAlignQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssRubyAlignQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyMergeValue {
    Separate,
    Merge,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyMergeUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyMergeQualificationOutcome {
    Qualified(CssRubyMergeValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssRubyMergeUnsupportedReason),
}

/// One selected ordinary declaration's bounded `ruby-merge` qualification.
///
/// This profile qualifies only the direct authored
/// `separate | merge | auto` keyword grammar. Ruby layout, annotation
/// pairing/merging execution, applicability, and computed/used values remain
/// outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssRubyMergeQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssRubyMergeQualificationOutcome,
}

impl CssRubyMergeQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssRubyMergeQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyPositionComponent {
    Alternate,
    Over,
    Under,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssRubyPositionComponents {
    authored: [CssRubyPositionComponent; 2],
    count: usize,
}

impl CssRubyPositionComponents {
    pub(crate) fn authored_components(&self) -> &[CssRubyPositionComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyPositionValue {
    InterCharacter,
    Components(CssRubyPositionComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyPositionUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyPositionQualificationOutcome {
    Qualified(CssRubyPositionValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssRubyPositionUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored
/// `ruby-position` qualification.
///
/// The composite branch preserves exact authored order for the
/// property-specific `[ alternate || [ over | under ] ]` grammar.
/// `alternate` occupies one singleton slot while `over | under` share
/// one mutually-exclusive SIDE slot. `inter-character` remains a
/// standalone authored identity. This slice does not execute alternating
/// annotation-level placement, inter-character layout, writing-mode
/// resolution, CSSOM canonicalization, or computed/used-value semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssRubyPositionQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssRubyPositionQualificationOutcome,
}

impl CssRubyPositionQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssRubyPositionQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyOverhangValue {
    Auto,
    Spaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyOverhangUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRubyOverhangQualificationOutcome {
    Qualified(CssRubyOverhangValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssRubyOverhangUnsupportedReason),
}

/// One selected ordinary declaration's bounded `ruby-overhang` qualification.
///
/// This profile qualifies the direct authored `auto | spaces` keyword grammar
/// and maps the legacy `none` value alias to `Spaces`. Ruby layout, annotation
/// container construction, overhang measurement, applicability, CSSOM
/// serialization, and computed/used values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssRubyOverhangQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssRubyOverhangQualificationOutcome,
}

impl CssRubyOverhangQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssRubyOverhangQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssClipRuleValue {
    Nonzero,
    Evenodd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssClipRuleUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssClipRuleQualificationOutcome {
    Qualified(CssClipRuleValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssClipRuleUnsupportedReason),
}

/// One selected ordinary declaration's bounded `clip-rule` qualification.
///
/// This profile qualifies only the direct authored `nonzero | evenodd`
/// keyword grammar. Clipping-path construction, winding execution, SVG
/// applicability, masking, painting, hit-testing, interpolation, and
/// computed/used values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssClipRuleQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssClipRuleQualificationOutcome,
}

impl CssClipRuleQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssClipRuleQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFillRuleValue {
    Nonzero,
    Evenodd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFillRuleUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFillRuleQualificationOutcome {
    Qualified(CssFillRuleValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFillRuleUnsupportedReason),
}

/// One selected ordinary declaration's bounded `fill-rule` qualification.
///
/// This profile qualifies only the direct authored `nonzero | evenodd`
/// keyword grammar. Fill-area construction, winding execution, path/text
/// geometry, SVG applicability, painting, hit-testing, interpolation, and
/// computed/used values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFillRuleQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFillRuleQualificationOutcome,
}

impl CssFillRuleQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFillRuleQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColumnFillValue {
    Auto,
    Balance,
    BalanceAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColumnFillUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColumnFillQualificationOutcome {
    Qualified(CssColumnFillValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssColumnFillUnsupportedReason),
}

/// One selected ordinary declaration's bounded `column-fill` qualification.
///
/// This profile qualifies only the direct authored
/// `auto | balance | balance-all` keyword grammar. Multi-column construction,
/// balancing execution, fragmentation, applicability, and computed/used values
/// remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssColumnFillQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssColumnFillQualificationOutcome,
}

impl CssColumnFillQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssColumnFillQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationSkipInkValue {
    Auto,
    None,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationSkipInkUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationSkipInkQualificationOutcome {
    Qualified(CssTextDecorationSkipInkValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextDecorationSkipInkUnsupportedReason),
}

/// One selected ordinary declaration's bounded
/// `text-decoration-skip-ink` qualification.
///
/// This profile qualifies only the direct authored `auto | none | all`
/// keyword grammar. Decoration-line construction, ink intersection geometry,
/// painting, applicability, and computed/used values remain outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextDecorationSkipInkQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextDecorationSkipInkQualificationOutcome,
}

impl CssTextDecorationSkipInkQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextDecorationSkipInkQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantLigaturesComponent {
    CommonLigatures,
    NoCommonLigatures,
    DiscretionaryLigatures,
    NoDiscretionaryLigatures,
    HistoricalLigatures,
    NoHistoricalLigatures,
    Contextual,
    NoContextual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontVariantLigaturesComponents {
    authored: [CssFontVariantLigaturesComponent; 4],
    count: usize,
}

impl CssFontVariantLigaturesComponents {
    pub(crate) fn authored_components(&self) -> &[CssFontVariantLigaturesComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantLigaturesValue {
    Normal,
    None,
    Components(CssFontVariantLigaturesComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantLigaturesUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantLigaturesQualificationOutcome {
    Qualified(CssFontVariantLigaturesValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontVariantLigaturesUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored
/// `font-variant-ligatures` qualification.
///
/// Composite values retain authored component order even though the grammar is
/// order-insensitive. Slot uniqueness is validated during qualification. This
/// slice does not expand keywords to OpenType feature tags, normalize feature
/// state, shape glyphs, or claim computed/used-value or font-feature precedence
/// semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontVariantLigaturesQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontVariantLigaturesQualificationOutcome,
}

impl CssFontVariantLigaturesQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontVariantLigaturesQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantNumericComponent {
    LiningNums,
    OldstyleNums,
    ProportionalNums,
    TabularNums,
    DiagonalFractions,
    StackedFractions,
    Ordinal,
    SlashedZero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontVariantNumericComponents {
    authored: [CssFontVariantNumericComponent; 5],
    count: usize,
}

impl CssFontVariantNumericComponents {
    pub(crate) fn authored_components(&self) -> &[CssFontVariantNumericComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantNumericValue {
    Normal,
    Components(CssFontVariantNumericComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantNumericUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssFontVariantNumericQualificationOutcome {
    Qualified(CssFontVariantNumericValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssFontVariantNumericUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored
/// `font-variant-numeric` qualification.
///
/// The composite branch preserves the exact authored component order even
/// though CSS `||` matching is order-insensitive. Slot uniqueness is validated
/// during qualification. This slice does not expand keywords to OpenType
/// feature tags, normalize feature state, shape glyphs, or claim computed/used
/// value or font-feature precedence semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssFontVariantNumericQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssFontVariantNumericQualificationOutcome,
}

impl CssFontVariantNumericQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssFontVariantNumericQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationLineComponent {
    Underline,
    Overline,
    LineThrough,
    Blink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextDecorationLineComponents {
    authored: [CssTextDecorationLineComponent; 4],
    count: usize,
}

impl CssTextDecorationLineComponents {
    pub(crate) fn authored_components(&self) -> &[CssTextDecorationLineComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationLineValue {
    None,
    SpellingError,
    GrammarError,
    Components(CssTextDecorationLineComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationLineUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextDecorationLineQualificationOutcome {
    Qualified(CssTextDecorationLineValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextDecorationLineUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored
/// `text-decoration-line` qualification.
///
/// Composite values preserve exact authored component order even though CSS
/// `||` matching is order-insensitive. Slot uniqueness is validated during
/// qualification. This slice does not render decorations, blink, detect
/// spelling/grammar errors, propagate decorations, canonicalize CSSOM order,
/// or claim computed/used-value semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextDecorationLineQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextDecorationLineQualificationOutcome,
}

impl CssTextDecorationLineQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextDecorationLineQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextTransformComponent {
    Capitalize,
    Uppercase,
    Lowercase,
    FullWidth,
    FullSizeKana,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextTransformComponents {
    authored: [CssTextTransformComponent; 3],
    count: usize,
}

impl CssTextTransformComponents {
    pub(crate) fn authored_components(&self) -> &[CssTextTransformComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextTransformValue {
    None,
    MathAuto,
    Components(CssTextTransformComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextTransformUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextTransformQualificationOutcome {
    Qualified(CssTextTransformValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextTransformUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored
/// `text-transform` qualification.
///
/// Composite values preserve exact authored component order even though CSS
/// `||` matching is order-insensitive. The CASE slot is mutually exclusive,
/// while `full-width` and `full-size-kana` each claim their own singleton
/// slot. `none` and `math-auto` remain standalone authored identities. This
/// slice does not transform text, perform language-sensitive casing, map
/// width/kana/math characters, canonicalize CSSOM order, or claim
/// computed/used-value semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextTransformQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextTransformQualificationOutcome,
}

impl CssTextTransformQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextTransformQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextEmphasisPositionComponent {
    Over,
    Under,
    Right,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextEmphasisPositionComponents {
    authored: [CssTextEmphasisPositionComponent; 2],
    count: usize,
}

impl CssTextEmphasisPositionComponents {
    pub(crate) fn authored_components(&self) -> &[CssTextEmphasisPositionComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextEmphasisPositionValue {
    Components(CssTextEmphasisPositionComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextEmphasisPositionUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextEmphasisPositionQualificationOutcome {
    Qualified(CssTextEmphasisPositionValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextEmphasisPositionUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored
/// `text-emphasis-position` qualification.
///
/// The selected normative grammar is
/// `[ over | under ] && [ right | left ]?`: one vertical component is
/// required, the side component is optional, and the two groups may occur
/// in either authored order. Exact authored sequence is retained. This
/// slice does not synthesize the default `right`, canonicalize CSSOM
/// serialization, implement language-dependent placement, or claim
/// computed/used-value or rendering semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextEmphasisPositionQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextEmphasisPositionQualificationOutcome,
}

impl CssTextEmphasisPositionQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextEmphasisPositionQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextUnderlinePositionComponent {
    FromFont,
    Under,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextUnderlinePositionComponents {
    authored: [CssTextUnderlinePositionComponent; 2],
    count: usize,
}

impl CssTextUnderlinePositionComponents {
    pub(crate) fn authored_components(&self) -> &[CssTextUnderlinePositionComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextUnderlinePositionValue {
    Auto,
    Components(CssTextUnderlinePositionComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextUnderlinePositionUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextUnderlinePositionQualificationOutcome {
    Qualified(CssTextUnderlinePositionValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextUnderlinePositionUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored
/// `text-underline-position` qualification.
///
/// The selected normative grammar is
/// `auto | [ from-font | under ] || [ left | right ]`: `auto` is an
/// exclusive singleton branch, while the component branch accepts one or
/// two authored direct identifiers with at most one occupying Slot A
/// (`from-font`/`under`) and at most one occupying Slot B
/// (`left`/`right`). Exact authored sequence is retained, including a
/// side-only `left`/`right` singleton, which is never synthesized with an
/// implied `auto`. This slice does not resolve font metrics, writing-mode
/// behavior, layout, painting, or claim computed/used-value or rendering
/// semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextUnderlinePositionQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextUnderlinePositionQualificationOutcome,
}

impl CssTextUnderlinePositionQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTextUnderlinePositionQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorXValue {
    Contain,
    None,
    Auto,
    Chain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorXUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorXQualificationOutcome {
    Qualified(CssOverscrollBehaviorXValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOverscrollBehaviorXUnsupportedReason),
}

/// One selected ordinary declaration's bounded `overscroll-behavior-x`
/// qualification.
///
/// This profile qualifies only the direct authored
/// `contain | none | auto | chain` keyword grammar. Scroll-container
/// applicability, boundary actions, scrolling, and computed/used values remain
/// outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOverscrollBehaviorXQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOverscrollBehaviorXQualificationOutcome,
}

impl CssOverscrollBehaviorXQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOverscrollBehaviorXQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorYValue {
    Contain,
    None,
    Auto,
    Chain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorYUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorYQualificationOutcome {
    Qualified(CssOverscrollBehaviorYValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOverscrollBehaviorYUnsupportedReason),
}

/// One selected ordinary declaration's bounded `overscroll-behavior-y`
/// qualification.
///
/// This profile qualifies only the direct authored
/// `contain | none | auto | chain` keyword grammar. Scroll-container
/// applicability, boundary actions, scrolling, and computed/used values remain
/// outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOverscrollBehaviorYQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOverscrollBehaviorYQualificationOutcome,
}

impl CssOverscrollBehaviorYQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOverscrollBehaviorYQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorInlineValue {
    Contain,
    None,
    Auto,
    Chain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorInlineUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorInlineQualificationOutcome {
    Qualified(CssOverscrollBehaviorInlineValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOverscrollBehaviorInlineUnsupportedReason),
}

/// One selected ordinary declaration's bounded `overscroll-behavior-inline`
/// qualification.
///
/// This profile qualifies only the direct authored
/// `contain | none | auto | chain` keyword grammar. Scroll-container
/// applicability, boundary actions, scrolling, and computed/used values remain
/// outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOverscrollBehaviorInlineQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOverscrollBehaviorInlineQualificationOutcome,
}

impl CssOverscrollBehaviorInlineQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOverscrollBehaviorInlineQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorBlockValue {
    Contain,
    None,
    Auto,
    Chain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorBlockUnsupportedReason {
    CssWideKeyword,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorBlockQualificationOutcome {
    Qualified(CssOverscrollBehaviorBlockValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOverscrollBehaviorBlockUnsupportedReason),
}

/// One selected ordinary declaration's bounded `overscroll-behavior-block`
/// qualification.
///
/// This profile qualifies only the direct authored
/// `contain | none | auto | chain` keyword grammar. Scroll-container
/// applicability, boundary actions, scrolling, and computed/used values remain
/// outside this slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOverscrollBehaviorBlockQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOverscrollBehaviorBlockQualificationOutcome,
}

impl CssOverscrollBehaviorBlockQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOverscrollBehaviorBlockQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorKeyword {
    Contain,
    None,
    Auto,
    Chain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorValue {
    Single(CssOverscrollBehaviorKeyword),
    Pair {
        first: CssOverscrollBehaviorKeyword,
        second: CssOverscrollBehaviorKeyword,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOverscrollBehaviorQualificationOutcome {
    Qualified(CssOverscrollBehaviorValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOverscrollBehaviorUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored
/// `overscroll-behavior` shorthand qualification.
///
/// Authored one- and two-keyword forms remain distinct here. This observation
/// performs no shorthand expansion, x/y mapping, one-value defaulting,
/// computed-value processing, or CSSOM serialization collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOverscrollBehaviorQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOverscrollBehaviorQualificationOutcome,
}

impl CssOverscrollBehaviorQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOverscrollBehaviorQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssContainComponent {
    Size,
    InlineSize,
    Layout,
    Style,
    Paint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssContainComponents {
    authored: [CssContainComponent; 4],
    count: usize,
}

impl CssContainComponents {
    pub(crate) fn authored_components(&self) -> &[CssContainComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssContainValue {
    None,
    Strict,
    Content,
    Components(CssContainComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssContainUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssContainQualificationOutcome {
    Qualified(CssContainValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssContainUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored `contain`
/// qualification.
///
/// Composite values preserve the author's component order even though the
/// grammar is order-insensitive. Slot uniqueness is validated during
/// qualification; this observation performs no canonical serialization,
/// `strict`/`content` expansion, computed-value processing, or containment
/// execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssContainQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssContainQualificationOutcome,
}

impl CssContainQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssContainQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPaintOrderComponent {
    Fill,
    Stroke,
    Markers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssPaintOrderComponents {
    authored: [CssPaintOrderComponent; 3],
    count: usize,
}

impl CssPaintOrderComponents {
    pub(crate) fn authored_components(&self) -> &[CssPaintOrderComponent] {
        &self.authored[..self.count]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPaintOrderValue {
    Normal,
    Components(CssPaintOrderComponents),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPaintOrderUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPaintOrderQualificationOutcome {
    Qualified(CssPaintOrderValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssPaintOrderUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored `paint-order`
/// qualification.
///
/// The `Components` branch preserves the author's exact component order and
/// cardinality. Slot uniqueness is validated during qualification; this
/// observation performs no omitted-operation synthesis, effective/rendering
/// paint-order completion, or CSSOM shortest-serialization collapse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssPaintOrderQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssPaintOrderQualificationOutcome,
}

impl CssPaintOrderQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssPaintOrderQualificationOutcome {
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
pub(crate) enum CssPageValue {
    Auto,
    CustomIdent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPageUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssPageQualificationOutcome {
    Qualified(CssPageValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssPageUnsupportedReason),
}

/// Run-local locator for the exact tokenizer item selected during authoritative
/// `page` custom-ident recognition. The index is evidence placement, not the
/// custom identifier's semantic identity; that identity remains the tokenizer-
/// owned decoded `Ident` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssPageCustomIdentEvidenceRef {
    lexical_item_index: usize,
}

impl CssPageCustomIdentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One selected ordinary declaration's bounded `page` qualification.
///
/// Open-ended custom-ident payload and exact authored source remain owned by
/// the tokenizer/parser chain. A qualified custom-ident records only the
/// recognition-time run-local relation to that exact retained Ident token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssPageQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssPageQualificationOutcome,
    custom_ident_evidence: Option<CssPageCustomIdentEvidenceRef>,
}

impl CssPageQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssPageQualificationOutcome {
        self.outcome
    }

    pub(crate) const fn custom_ident_evidence(&self) -> Option<CssPageCustomIdentEvidenceRef> {
        self.custom_ident_evidence
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderSpacingValue {
    Single,
    Pair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderSpacingUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssBorderSpacingQualificationOutcome {
    Qualified(CssBorderSpacingValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssBorderSpacingUnsupportedReason),
}

/// One selected ordinary declaration's bounded `border-spacing` qualification.
///
/// This profile partitions the already-retained declaration value window into
/// one or two top-level components using a recognition-time block-depth
/// balanced walk, then qualifies each direct component against the already
/// accepted direct `<length [0,∞]>` boundary. Function-headed components are
/// classified by placement and identity only; no Function is evaluated or
/// type-checked, and no unit conversion or table-layout geometry is derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssBorderSpacingQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssBorderSpacingQualificationOutcome,
}

impl CssBorderSpacingQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssBorderSpacingQualificationOutcome {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAspectRatioRatioValue {
    Single,
    Pair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAspectRatioValue {
    Auto,
    Ratio(CssAspectRatioRatioValue),
    AutoAndRatio(CssAspectRatioRatioValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAspectRatioUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAspectRatioQualificationOutcome {
    Qualified(CssAspectRatioValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssAspectRatioUnsupportedReason),
}

/// One selected ordinary declaration's bounded `aspect-ratio` qualification.
///
/// This profile partitions the already-retained declaration value window into
/// ordered top-level components using a recognition-time block-depth balanced
/// walk, then identifies the optional direct `auto` operand and one optional
/// contiguous `<ratio>` operand in either authored order. Direct ratio
/// components reuse the already accepted direct `<number [0,∞]>` boundary;
/// Function-headed components are classified by placement and identity only,
/// never evaluated. No ratio arithmetic, normalization, or computed-value
/// semantics are derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssAspectRatioQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssAspectRatioQualificationOutcome,
}

impl CssAspectRatioQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssAspectRatioQualificationOutcome {
        self.outcome
    }
}

/// One authored `offset-rotate` structural composition of at most one
/// `auto | reverse` keyword operand and at most one direct `<angle>`
/// operand, in either authored order. This is membership/shape evidence
/// only: no numeric angle value, unit, canonical degrees, or authored
/// component order is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOffsetRotateValue {
    Auto,
    Reverse,
    DirectAngle,
    AutoAndDirectAngle,
    ReverseAndDirectAngle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOffsetRotateUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssOffsetRotateQualificationOutcome {
    Qualified(CssOffsetRotateValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssOffsetRotateUnsupportedReason),
}

/// One selected ordinary declaration's bounded `offset-rotate`
/// qualification against `[ auto | reverse ] || <angle>`.
///
/// This profile partitions the already-retained declaration value window
/// into ordered top-level components using a recognition-time block-depth
/// balanced walk, then identifies the optional direct `auto`/`reverse`
/// keyword operand and one optional direct `<angle>` operand in either
/// authored order. A direct `<angle>` component is exactly one retained
/// `Dimension` token whose decoded unit is ASCII-case-insensitively `deg`,
/// `grad`, `rad`, or `turn`; a unitless Number (including zero) is not a
/// direct `<angle>` literal. Function-headed components are classified by
/// placement and identity only, never evaluated. No angle unit conversion,
/// canonicalization, or computed-value semantics are derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssOffsetRotateQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssOffsetRotateQualificationOutcome,
}

impl CssOffsetRotateQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssOffsetRotateQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationPlayStateValue {
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationPlayStateUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssAnimationPlayStateQualificationOutcome {
    Qualified(Vec<CssAnimationPlayStateValue>),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssAnimationPlayStateUnsupportedReason),
}

/// One selected ordinary declaration's bounded `animation-play-state`
/// qualification.
///
/// This profile recognizes only the authored
/// `<single-animation-play-state>#` grammar. Ordered list values are
/// retained as property-specific `Running | Paused` items. Exact source
/// evidence, separators, trivia, and occurrence identity remain owned by
/// the upstream tokenizer/parser result; no animation runtime or
/// coordinating-list semantics are derived here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssAnimationPlayStateQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssAnimationPlayStateQualificationOutcome,
}

impl CssAnimationPlayStateQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssAnimationPlayStateQualificationOutcome {
        &self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationIterationCountValue {
    Infinite,
    DirectNumberLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationIterationCountUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssAnimationIterationCountQualificationOutcome {
    Qualified(Vec<CssAnimationIterationCountValue>),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssAnimationIterationCountUnsupportedReason),
}

/// One selected ordinary declaration's bounded `animation-iteration-count`
/// qualification.
///
/// This profile recognizes only the authored
/// `<single-animation-iteration-count>#` grammar, where
/// `<single-animation-iteration-count> = infinite | <number [0,∞]>`. Ordered
/// list values are retained as property-specific `Infinite |
/// DirectNumberLiteral` items. Function-backed numeric items (e.g. `calc()`)
/// remain conservatively `UnsupportedBySelectedValueProfile(FunctionValue)`
/// because this slice does not evaluate numeric Functions; a decisive direct
/// Invalid item always takes precedence over a residual Function item
/// elsewhere in the same list. Exact source evidence, separators, trivia, and
/// occurrence identity remain owned by the upstream tokenizer/parser result;
/// no coordinated animation-list or runtime semantics are derived here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssAnimationIterationCountQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssAnimationIterationCountQualificationOutcome,
}

impl CssAnimationIterationCountQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssAnimationIterationCountQualificationOutcome {
        &self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationDelayValue {
    DirectTimeLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationDelayUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssAnimationDelayQualificationOutcome {
    Qualified(Vec<CssAnimationDelayValue>),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssAnimationDelayUnsupportedReason),
}

/// One selected ordinary declaration's bounded `animation-delay`
/// qualification.
///
/// This profile recognizes only the authored `<time>#` grammar, where a
/// direct item is exactly one non-trivia `Dimension` token whose decoded
/// unit is ASCII-case-insensitively `s` or `ms`. Unlike accepted `<length>`
/// leaves, a unitless Number (including zero) is not a direct `<time>`
/// literal and is therefore `InvalidForSelectedValueGrammar`; `animation-delay`
/// places no non-negative range restriction, so negative direct time
/// literals qualify unchanged. Function-backed numeric items (e.g. `calc()`)
/// remain conservatively `UnsupportedBySelectedValueProfile(FunctionValue)`
/// because this slice does not evaluate numeric Functions; a decisive direct
/// Invalid item always takes precedence over a residual Function item
/// elsewhere in the same list. Exact source evidence, separators, trivia, and
/// occurrence identity remain owned by the upstream tokenizer/parser result;
/// no coordinated animation-list or runtime semantics are derived here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssAnimationDelayQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssAnimationDelayQualificationOutcome,
}

impl CssAnimationDelayQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssAnimationDelayQualificationOutcome {
        &self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransitionDurationValue {
    DirectTimeLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransitionDurationUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTransitionDurationQualificationOutcome {
    Qualified(Vec<CssTransitionDurationValue>),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTransitionDurationUnsupportedReason),
}

/// One selected ordinary declaration's bounded `transition-duration`
/// qualification.
///
/// This profile recognizes only the authored `<time [0s,∞]>#` grammar,
/// composing the accepted top-level comma-list theorem, the accepted direct
/// `<time>` Dimension theorem from `animation-delay` (#575), and the
/// existing exact `is_non_negative_direct_number` authored-range theorem. A
/// direct item qualifies only when it is exactly one non-trivia `Dimension`
/// token whose decoded unit is ASCII-case-insensitively `s` or `ms` and whose
/// retained numeric evidence is non-negative; unlike `animation-delay`, a
/// direct negative non-zero time literal is therefore
/// `InvalidForSelectedValueGrammar`. A calculation such as `calc(-1s)` is not
/// range-checked here and remains conservatively
/// `UnsupportedBySelectedValueProfile(FunctionValue)`; a decisive direct
/// Invalid item elsewhere in the same list always outranks a residual
/// Function item. Exact source evidence, separators, trivia, and occurrence
/// identity remain owned by the upstream tokenizer/parser result; no
/// coordinated transition-list or runtime semantics are derived here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssTransitionDurationQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTransitionDurationQualificationOutcome,
}

impl CssTransitionDurationQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssTransitionDurationQualificationOutcome {
        &self.outcome
    }
}

/// One authored `<single-transition-property>` list item kind: either the
/// predefined `all` keyword or an open-ended `<custom-ident>`. This carries
/// no payload; a qualified `CustomIdent` item's decoded semantic identity
/// remains tokenizer-owned and is related only through the observation's
/// parallel, index-aligned evidence-reference list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransitionPropertyItemValue {
    All,
    CustomIdent,
}

/// Run-local locator for the exact tokenizer item selected during authoritative
/// `transition-property` custom-ident recognition. The index is evidence
/// placement, not the custom identifier's semantic identity; that identity
/// remains the tokenizer-owned decoded `Ident` value, exactly as for the
/// accepted `page` theorem (`CssPageCustomIdentEvidenceRef`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransitionPropertyCustomIdentEvidenceRef {
    lexical_item_index: usize,
}

impl CssTransitionPropertyCustomIdentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTransitionPropertyValue {
    None,
    Items(Vec<CssTransitionPropertyItemValue>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransitionPropertyUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTransitionPropertyQualificationOutcome {
    Qualified(CssTransitionPropertyValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTransitionPropertyUnsupportedReason),
}

/// One selected ordinary declaration's bounded `transition-property`
/// qualification.
///
/// This profile recognizes only the authored
/// `none | [ all | <custom-ident> ]#` grammar, composing the accepted
/// top-level comma-list theorem (#571/#573/#575/#577) with the accepted
/// open-ended custom-ident evidence-reference ownership theorem proven by
/// `page` (#578 / #418 comment 5580383097). Sole `none` is the dedicated
/// whole-value branch; it is not a valid list item. Inside the list, `all`
/// is matched ASCII-case-insensitively as a predefined keyword; every other
/// direct Ident item is an open-ended `<custom-ident>` whose decoded
/// semantic identity remains tokenizer-owned. When `outcome()` is
/// `Qualified(CssTransitionPropertyValue::Items(kinds))`,
/// `custom_ident_evidence()` is index-aligned with `kinds`: `Some` exactly
/// where the item is `CustomIdent` and `None` where it is `All`, so no
/// custom-ident evidence reference is ever fabricated for an `All` item.
/// `none`, `default`, and CSS-wide keywords are excluded from list-item
/// position. Unlike `<time>`/`<number>` grammars, this item grammar has no
/// ordinary Function-backed branch, so any Function or other multi-token
/// item is directly `InvalidForSelectedValueGrammar`; there is intentionally
/// no residual `FunctionValue` Unsupported item class. Authored order and
/// duplicate items are preserved exactly; no deduplication, canonicalization,
/// or property-registry lookup is performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssTransitionPropertyQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTransitionPropertyQualificationOutcome,
    custom_ident_evidence: Vec<Option<CssTransitionPropertyCustomIdentEvidenceRef>>,
}

impl CssTransitionPropertyQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssTransitionPropertyQualificationOutcome {
        &self.outcome
    }

    pub(crate) fn custom_ident_evidence(
        &self,
    ) -> &[Option<CssTransitionPropertyCustomIdentEvidenceRef>] {
        &self.custom_ident_evidence
    }
}

/// Run-local locator for the exact tokenizer item selected during authoritative
/// `hyphenate-character` direct `<string>` recognition. The index is evidence
/// placement, not the String's semantic identity; that identity remains the
/// tokenizer-owned decoded `CssTokenKind::String(String)` value, exactly as
/// `CssPageCustomIdentEvidenceRef` and
/// `CssTransitionPropertyCustomIdentEvidenceRef` locate their Ident tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssHyphenateCharacterStringEvidenceRef {
    lexical_item_index: usize,
}

impl CssHyphenateCharacterStringEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssHyphenateCharacterValue {
    Auto,
    DirectStringLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssHyphenateCharacterUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssHyphenateCharacterQualificationOutcome {
    Qualified(CssHyphenateCharacterValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssHyphenateCharacterUnsupportedReason),
}

/// One selected ordinary declaration's bounded `hyphenate-character`
/// qualification.
///
/// This profile recognizes only the authored `auto | <string>` grammar: the
/// first direct authored CSS `<string>` qualification leaf (#580 / #418
/// comment 5581206679). A direct Ident `auto` (ASCII-case-insensitive)
/// qualifies as `Auto` with no String evidence. Exactly one non-trivia
/// retained `CssTokenKind::String(_)` qualifies as `DirectStringLiteral`; its
/// decoded payload remains tokenizer-owned, and this observation retains only
/// a recognition-time evidence reference to that exact token, mirroring the
/// accepted `page` / `transition-property` custom-ident evidence-reference
/// ownership pattern. A quoted String never inherits keyword semantics from
/// its decoded contents (`"auto"`, `"initial"`, `"none"` all remain String),
/// and a retained `CssTokenKind::BadString` is not `<string>` for this
/// profile. This grammar has no ordinary Function-backed branch, so any
/// Function value is directly `InvalidForSelectedValueGrammar`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssHyphenateCharacterQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssHyphenateCharacterQualificationOutcome,
    string_evidence: Option<CssHyphenateCharacterStringEvidenceRef>,
}

impl CssHyphenateCharacterQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssHyphenateCharacterQualificationOutcome {
        self.outcome
    }

    pub(crate) const fn string_evidence(&self) -> Option<CssHyphenateCharacterStringEvidenceRef> {
        self.string_evidence
    }
}

/// Run-local locator for the exact tokenizer item selected during authoritative
/// `animation-name` `<keyframes-name>` recognition. Unlike
/// `CssTransitionPropertyCustomIdentEvidenceRef` and
/// `CssHyphenateCharacterStringEvidenceRef`, this locator may point at either
/// an accepted direct `Ident` token or an accepted non-empty direct `String`
/// token: `<keyframes-name> = <custom-ident> | <string>`. The index is
/// evidence placement, not the interpreted identity itself; the interpreted
/// `<keyframes-name>` text remains the tokenizer-owned decoded `Ident`/`String`
/// value at that exact retained position, resolved through
/// `animation_name_keyframes_name_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssAnimationNameKeyframesNameEvidenceRef {
    lexical_item_index: usize,
}

impl CssAnimationNameKeyframesNameEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationNameItemValue {
    None,
    KeyframesName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationNameUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssAnimationNameQualificationOutcome {
    Qualified(Vec<CssAnimationNameItemValue>),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssAnimationNameUnsupportedReason),
}

/// One selected ordinary declaration's bounded `animation-name` qualification.
///
/// This profile recognizes only the authored
/// `[ none | <keyframes-name> ]#` grammar, where
/// `<keyframes-name> = <custom-ident> | <string>` (#582 / #418 comment
/// 5581930292), composing the accepted top-level comma-list theorem
/// (#571/#573/#575/#577) with the accepted open-ended evidence-reference
/// ownership theorem proven by `page` / `transition-property` /
/// `hyphenate-character`. Unlike `transition-property`, `none` is not a
/// dedicated whole-value branch here: it is matched ASCII-case-insensitively
/// as a repeated list-item sentinel, so it may appear any number of times in
/// any position. A quoted String never inherits identifier keyword semantics
/// from its decoded contents, so `"none"` is always a `KeyframesName` item,
/// distinct from the unquoted `none` sentinel; an empty direct String is
/// `InvalidForSelectedValueGrammar`, unlike the accepted `hyphenate-character`
/// empty-String allowance. Every other direct Ident item other than `none`,
/// `default`, and the CSS-wide keywords is an open-ended `<custom-ident>`
/// `KeyframesName` item whose decoded semantic identity remains
/// tokenizer-owned and fully case-sensitive. When `outcome()` is
/// `Qualified(items)`, `keyframes_name_evidence()` is index-aligned with
/// `items`: `Some` exactly where the item is `KeyframesName` (whether
/// Ident-backed or String-backed) and `None` where it is the `None` sentinel,
/// so no keyframes-name evidence reference is ever fabricated for a sentinel
/// item. This item grammar has no ordinary Function-backed branch, so any
/// Function or other multi-token item is directly
/// `InvalidForSelectedValueGrammar`; there is intentionally no residual
/// `FunctionValue` Unsupported item class. Authored order and duplicate items
/// are preserved exactly; no deduplication, case normalization, Unicode
/// normalization, or `@keyframes` lookup is performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssAnimationNameQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssAnimationNameQualificationOutcome,
    keyframes_name_evidence: Vec<Option<CssAnimationNameKeyframesNameEvidenceRef>>,
}

impl CssAnimationNameQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssAnimationNameQualificationOutcome {
        &self.outcome
    }

    pub(crate) fn keyframes_name_evidence(
        &self,
    ) -> &[Option<CssAnimationNameKeyframesNameEvidenceRef>] {
        &self.keyframes_name_evidence
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `anchor-name` `<dashed-ident>` recognition, reusing the
/// `page` / `transition-property` / `hyphenate-character` / `animation-name`
/// evidence-reference ownership pattern. The index is evidence placement,
/// not the interpreted identifier itself; the decoded `<dashed-ident>` text
/// remains the tokenizer-owned decoded `Ident` value at that exact retained
/// position, resolved through `anchor_name_dashed_ident_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssAnchorNameDashedIdentEvidenceRef {
    lexical_item_index: usize,
}

impl CssAnchorNameDashedIdentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One authored `anchor-name` value: either the dedicated whole-value `none`
/// sentinel or an ordered, possibly-duplicated list of qualified
/// `<dashed-ident>` items. Unlike `CssAnimationNameItemValue`, this grammar
/// has exactly one list-item kind, so no separate item-value enum is
/// introduced: each `Names` entry is directly the evidence reference to its
/// qualified `<dashed-ident>` item, and the decoded identifier payload
/// itself remains tokenizer-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssAnchorNameValue {
    None,
    Names(Vec<CssAnchorNameDashedIdentEvidenceRef>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnchorNameUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssAnchorNameQualificationOutcome {
    Qualified(CssAnchorNameValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssAnchorNameUnsupportedReason),
}

/// One selected ordinary declaration's bounded `anchor-name` qualification.
///
/// This profile recognizes only the authored `none | <dashed-ident>#`
/// grammar (#584 / #418 comment 5583004626), composing the accepted
/// top-level comma-list theorem (#571/#573/#575/#577) with the accepted
/// open-ended evidence-reference ownership theorem proven by `page` /
/// `transition-property` / `hyphenate-character` / `animation-name`. Sole
/// `none` is a dedicated whole-value branch, matched ASCII-case-
/// insensitively; unlike `animation-name`'s `none`, it is **not** a valid
/// list item -- `none` in any list position, alone or alongside qualified
/// `<dashed-ident>` items, is `InvalidForSelectedValueGrammar`. Every
/// direct-Ident list item qualifies iff its tokenizer-decoded identifier
/// starts with `--`; this test is the identifier's *interpreted* decoded
/// identity, never its raw authored spelling, so an escape-authored item
/// that decodes to a leading `--` qualifies and an unescaped `--none`
/// qualifies as an ordinary `<dashed-ident>` distinct from the `none`
/// sentinel. The dashed-ident identity remains fully case-sensitive; no
/// case-folding, deduplication, or reordering is performed. This item
/// grammar has no ordinary Function-backed branch, so any Function or other
/// multi-token item is directly `InvalidForSelectedValueGrammar`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssAnchorNameQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssAnchorNameQualificationOutcome,
}

impl CssAnchorNameQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssAnchorNameQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `container-name` `<custom-ident>` recognition, reusing the
/// `page` / `transition-property` / `hyphenate-character` / `animation-name`
/// / `anchor-name` evidence-reference ownership pattern. The index is
/// evidence placement, not the interpreted identifier itself; the decoded
/// `<custom-ident>` text remains the tokenizer-owned decoded `Ident` value
/// at that exact retained position, resolved through
/// `container_name_custom_ident_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssContainerNameCustomIdentEvidenceRef {
    lexical_item_index: usize,
}

impl CssContainerNameCustomIdentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One authored `container-name` value: either the dedicated whole-value
/// `none` sentinel or an ordered, possibly-duplicated one-or-more list of
/// qualified `<custom-ident>` items. Unlike `CssAnimationNameItemValue`,
/// `none` is never a repeated-item sentinel here -- it occupies only the
/// dedicated whole-value branch, never a `Names` entry -- and this grammar
/// has exactly one list-item kind, so each `Names` entry is directly the
/// evidence reference to its qualified `<custom-ident>` item; the decoded
/// identifier payload itself remains tokenizer-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssContainerNameValue {
    None,
    Names(Vec<CssContainerNameCustomIdentEvidenceRef>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssContainerNameUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssContainerNameQualificationOutcome {
    Qualified(CssContainerNameValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssContainerNameUnsupportedReason),
}

/// One selected ordinary declaration's bounded `container-name`
/// qualification against `none | <custom-ident>+` (#588 / #418 comment
/// 5584554364), composing the accepted delimiter-free top-level-component
/// theorem (`border-spacing` / `offset-rotate`) with the accepted
/// open-ended evidence-reference ownership theorem proven by `page` /
/// `transition-property` / `hyphenate-character` / `animation-name` /
/// `anchor-name`. Sole `none` is a dedicated whole-value branch, matched
/// ASCII-case-insensitively; unlike `animation-name`'s `none`, it is
/// **not** a valid list item -- `none` in any list position, alone or
/// alongside qualified `<custom-ident>` items, is
/// `InvalidForSelectedValueGrammar`. Every top-level, delimiter-free
/// (never comma-separated) component must be exactly one direct `Ident`
/// token whose tokenizer-decoded identifier is not `none`, `and`, `not`,
/// or `or` (ASCII-case-insensitively), not the generic `<custom-ident>`
/// reserved identifier `default`, and not a CSS-wide keyword. The
/// identity remains fully case-sensitive; no case-folding,
/// Unicode-normalization, deduplication, or reordering is performed. This
/// item grammar has no ordinary Function-backed branch, so any Function or
/// other multi-token component is directly `InvalidForSelectedValueGrammar`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssContainerNameQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssContainerNameQualificationOutcome,
}

impl CssContainerNameQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssContainerNameQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `color-scheme` `<custom-ident>` recognition, reusing the
/// `page` / `transition-property` / `hyphenate-character` /
/// `animation-name` / `anchor-name` / `container-name` evidence-reference
/// ownership pattern. The index is evidence placement, not the interpreted
/// identifier itself; the decoded `<custom-ident>` text remains the
/// tokenizer-owned decoded `Ident` value at that exact retained position,
/// resolved through `color_scheme_custom_ident_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssColorSchemeCustomIdentEvidenceRef {
    lexical_item_index: usize,
}

impl CssColorSchemeCustomIdentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One ordered `color-scheme` repeated scheme-group item: the two
/// predefined keywords, matched ASCII-case-insensitively, or an open-ended
/// tokenizer-owned `<custom-ident>` evidence reference. Unlike
/// `container-name`, `none` is deliberately not excluded here -- it is an
/// ordinary qualified `CustomIdent` item, since this property does not
/// reserve it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColorSchemeItemValue {
    Light,
    Dark,
    CustomIdent(CssColorSchemeCustomIdentEvidenceRef),
}

/// One authored `color-scheme` value: either the dedicated standalone
/// `normal` branch or the composite `[ light | dark | <custom-ident> ]+ &&
/// only?` branch. `normal` never combines with the composite branch, and
/// `only` is an orthogonal modifier of the *entire* repeated `items` group
/// -- it is never itself a repeated item and is never interior to the
/// group. The authored outer placement of `only` (leading vs. trailing) is
/// not retained; both placements are equally `Qualified`, and no CSSOM
/// canonical-serialization order is implied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssColorSchemeValue {
    Normal,
    Schemes {
        items: Vec<CssColorSchemeItemValue>,
        only: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssColorSchemeUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssColorSchemeQualificationOutcome {
    Qualified(CssColorSchemeValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssColorSchemeUnsupportedReason),
}

/// One selected ordinary declaration's bounded `color-scheme`
/// qualification against `normal | [ light | dark | <custom-ident> ]+ &&
/// only?` (#590 / #418 comment 5585104754), composing the accepted
/// delimiter-free `+` repetition theorem (`container-name`) with the
/// accepted property-local `&&` composition theorem
/// (`text-emphasis-position`) and the accepted open-ended
/// evidence-reference ownership theorem proven by `page` /
/// `transition-property` / `hyphenate-character` / `animation-name` /
/// `anchor-name` / `container-name`. A sole retained direct `normal`
/// Ident, ASCII-case-insensitively, qualifies the dedicated standalone
/// branch and never combines with the composite branch. Otherwise every
/// top-level, delimiter-free (never comma-separated) component must be
/// exactly one direct `Ident` token that is either the predefined keyword
/// `light`/`dark`, the structural modifier `only`, or an
/// otherwise-unreserved `<custom-ident>` -- `normal`, `default`, and
/// CSS-wide keywords never qualify as a component in this position. At
/// most one `only` component may appear, and only at the very start or
/// very end of the component sequence; the remaining (non-`only`)
/// components form the mandatory, order- and duplicate-preserving `items`
/// sequence, which must be non-empty. Identity remains fully
/// case-sensitive for `<custom-ident>` items; no case-folding,
/// Unicode-normalization, deduplication, or reordering is performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssColorSchemeQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssColorSchemeQualificationOutcome,
}

impl CssColorSchemeQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssColorSchemeQualificationOutcome {
        &self.outcome
    }
}

/// One authored `image-resolution` structural composition selected against
/// `[ from-image || <resolution> ] && snap?` (#596 / #418 comment
/// 5601753463). This is shape-only membership evidence, analogous to
/// `offset-rotate`: no resolution numeric magnitude, unit String, canonical
/// `dppx` value, or authored component order is retained. `from-image
/// 300dpi` and `300dpi from-image` are therefore the same
/// `FromImageAndResolution` shape, and leading vs. trailing `snap`
/// placement collapses into the same `*AndSnap` shape once grammar
/// validity is established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssImageResolutionValue {
    FromImage,
    DirectResolution,
    FromImageAndResolution,
    FromImageAndSnap,
    DirectResolutionAndSnap,
    FromImageAndResolutionAndSnap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssImageResolutionUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssImageResolutionQualificationOutcome {
    Qualified(CssImageResolutionValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssImageResolutionUnsupportedReason),
}

/// One selected ordinary declaration's bounded `image-resolution`
/// qualification against `[ from-image || <resolution> ] && snap?`.
///
/// This profile partitions the already-retained declaration value window
/// into ordered top-level components using a recognition-time block-depth
/// balanced walk (mirroring `offset-rotate` / `color-scheme`), then treats
/// `snap` as an orthogonal structural modifier of the *entire* `[
/// from-image || <resolution> ]` group -- reusing the accepted
/// `color-scheme` `only` boundary theorem -- so `snap` may appear only
/// immediately before or immediately after that group, never interior to
/// it. A direct `<resolution>` component is exactly one retained
/// `Dimension` token whose decoded unit is ASCII-case-insensitively `dpi`,
/// `dpcm`, `dppx`, or `x` and whose value is not negative-non-zero (signed
/// zero remains qualified). No unit conversion, canonical `dppx`
/// materialization, or computed-value semantics are derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssImageResolutionQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssImageResolutionQualificationOutcome,
}

impl CssImageResolutionQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssImageResolutionQualificationOutcome {
        self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `counter-increment` `<counter-name>` recognition, reusing
/// the `page` / `transition-property` / `hyphenate-character` /
/// `animation-name` / `anchor-name` / `container-name` / `color-scheme`
/// evidence-reference ownership pattern. The index is evidence placement,
/// not the interpreted identifier itself; the decoded `<counter-name>` text
/// remains the tokenizer-owned decoded `Ident` value at that exact retained
/// position, resolved through `counter_increment_name_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterIncrementNameEvidenceRef {
    lexical_item_index: usize,
}

impl CssCounterIncrementNameEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `counter-increment` direct explicit `<integer>`
/// recognition. The index is evidence placement, not a copied numeric
/// value; the exact retained `Number`-token structure (sign/zero spelling,
/// magnitude) remains tokenizer-owned and is never converted to a machine
/// integer for qualification, resolved through
/// `counter_increment_integer_token`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterIncrementIntegerEvidenceRef {
    lexical_item_index: usize,
}

impl CssCounterIncrementIntegerEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `counter-increment` repeated item:
/// `DirectCounterItem := DirectCounterNameEvidence DirectIntegerEvidence?`
/// (#592 / #418 comment 5593320578). Authored omission of the integer is
/// load-bearing and distinct from an explicit `1` -- this leaf never
/// synthesizes the property's interpreted default increment of `1` for an
/// omitted item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterIncrementItem {
    name: CssCounterIncrementNameEvidenceRef,
    explicit_integer: Option<CssCounterIncrementIntegerEvidenceRef>,
}

impl CssCounterIncrementItem {
    pub(crate) const fn name(&self) -> CssCounterIncrementNameEvidenceRef {
        self.name
    }

    pub(crate) const fn explicit_integer(&self) -> Option<CssCounterIncrementIntegerEvidenceRef> {
        self.explicit_integer
    }
}

/// One authored `counter-increment` value under the narrowed
/// direct-authored structured-repetition profile
/// `QualifiedDirectCounterIncrement := none | DirectCounterItem+` (#592 /
/// #418 comment 5593320578): either the dedicated whole-value `none`
/// sentinel or an ordered, possibly duplicated, one-or-more list of
/// qualified `DirectCounterItem`s. This is not a complete normative
/// `counter-increment` grammar: it qualifies only direct-literal items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssCounterIncrementValue {
    None,
    Items(Vec<CssCounterIncrementItem>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCounterIncrementUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValuedIntegerSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssCounterIncrementQualificationOutcome {
    Qualified(CssCounterIncrementValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssCounterIncrementUnsupportedReason),
}

/// One selected ordinary declaration's bounded `counter-increment`
/// qualification against the narrowed direct-authored profile `none |
/// DirectCounterItem+` where `DirectCounterItem := <counter-name>
/// <integer>?` (#592 / #418 comment 5593320578), composing the accepted
/// delimiter-free top-level-component partitioning theorem
/// (`container-name` / `color-scheme`) with a new structured-tuple
/// sequential grouping pass over the classified components.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `container-name`/`color-scheme`. A sole retained
/// direct `none` Ident, ASCII-case-insensitively, qualifies the dedicated
/// whole-value branch and is never a repeated-item sentinel. Otherwise
/// every top-level, delimiter-free (never comma-separated) component is
/// classified, then components are walked left-to-right: each item
/// requires one direct `<counter-name>` `Ident` (not `none`, `default`, or
/// a CSS-wide keyword), optionally followed by one direct `Integer`-typed
/// `Number` component. A Function occupying that optional-integer position
/// is a provisional feasible-integer-slot ambiguity -- resolved to
/// `UnsupportedBySelectedValueProfile(FunctionValuedIntegerSlot)` only if
/// no later component decisively invalidates the declaration regardless of
/// what the Function computes to. A Function can never satisfy the
/// required name position; a recognized generic whole-value-only function
/// name (e.g. `first-valid`) occupying a non-whole-value position is
/// always decisively `InvalidForSelectedValueGrammar`, never softened to
/// Unsupported. This leaf performs no numeric Function evaluation, no
/// machine-integer conversion, and no runtime counter semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssCounterIncrementQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssCounterIncrementQualificationOutcome,
}

impl CssCounterIncrementQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssCounterIncrementQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `counter-reset` `<counter-name>` recognition, reusing the
/// `counter-increment` evidence-reference ownership pattern (#594 / #418
/// comment 5595916114). The index is evidence placement, not the
/// interpreted identifier itself. For a `Direct` item name it points at the
/// top-level authored `Ident`; for a `Reversed` item name it points at the
/// exact tokenizer-owned INNER `Ident` retained inside the grammar-native
/// `reversed(...)` Function component, never at the Function opener,
/// parenthesis, or trivia. The decoded `<counter-name>` text is resolved
/// through `counter_reset_name_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterResetNameEvidenceRef {
    lexical_item_index: usize,
}

impl CssCounterResetNameEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `counter-reset` item name: either the ordinary
/// top-level `<counter-name>` `Direct` branch, or the grammar-native
/// `reversed(<counter-name>)` Function branch carrying the exact
/// tokenizer-owned inner `Ident` evidence (#594 / #418 comment
/// 5595916114). Both variants share the same evidence-ref shape because
/// each always resolves to a direct retained `Ident` token; only the
/// selected branch differs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCounterResetName {
    Direct(CssCounterResetNameEvidenceRef),
    Reversed(CssCounterResetNameEvidenceRef),
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `counter-reset` direct explicit `<integer>` recognition,
/// reusing the `counter-increment` pattern unchanged. The exact retained
/// `Number`-token structure (sign/zero spelling, magnitude) remains
/// tokenizer-owned and is never converted to a machine integer for
/// qualification, resolved through `counter_reset_integer_token`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterResetIntegerEvidenceRef {
    lexical_item_index: usize,
}

impl CssCounterResetIntegerEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One authored `counter-reset` repeated item:
/// `CounterResetItem := CssCounterResetName <integer>?` (#594 / #418
/// comment 5595916114). Authored omission of the integer is load-bearing
/// and distinct from an explicit `0` for a `Direct` name, and distinct
/// from any synthesized reversed-counter starting value for a `Reversed`
/// name -- this leaf never synthesizes either interpreted default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterResetItem {
    name: CssCounterResetName,
    explicit_integer: Option<CssCounterResetIntegerEvidenceRef>,
}

impl CssCounterResetItem {
    pub(crate) const fn name(&self) -> CssCounterResetName {
        self.name
    }

    pub(crate) const fn explicit_integer(&self) -> Option<CssCounterResetIntegerEvidenceRef> {
        self.explicit_integer
    }
}

/// One authored `counter-reset` value under the narrowed direct-authored
/// structured-repetition profile `QualifiedDirectCounterReset := none |
/// CounterResetItem+` (#594 / #418 comment 5595916114): either the
/// dedicated whole-value `none` sentinel or an ordered, possibly
/// duplicated, one-or-more list of qualified `CounterResetItem`s. This is
/// not a complete normative `counter-reset` grammar: it qualifies only
/// direct-literal `Direct`/`Reversed` items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssCounterResetValue {
    None,
    Items(Vec<CssCounterResetItem>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCounterResetUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValuedIntegerSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssCounterResetQualificationOutcome {
    Qualified(CssCounterResetValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssCounterResetUnsupportedReason),
}

/// One selected ordinary declaration's bounded `counter-reset`
/// qualification against the narrowed direct-authored profile `none |
/// CounterResetItem+` where `CounterResetItem := CssCounterResetName
/// <integer>?` and `CssCounterResetName := <counter-name> |
/// reversed(<counter-name>)` (#594 / #418 comment 5595916114), composing
/// the accepted `counter-increment` structured-repetition theorem with one
/// new grammar-native Function branch recognized at the required item-name
/// position.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `counter-increment`, over the entire flat token
/// stream -- this already covers `reversed(var(--x))` and similar nested
/// deferred forms without any reversed()-specific handling, since the
/// existing any-occurrence preflight scans regardless of nesting depth. A
/// sole retained direct `none` Ident, ASCII-case-insensitively, qualifies
/// the dedicated whole-value branch and is never a repeated-item sentinel.
/// Otherwise every top-level, delimiter-free (never comma-separated)
/// component is classified, then components are walked left-to-right
/// exactly as `counter-increment` does: each item requires either one
/// direct `<counter-name>` `Ident` or one direct grammar-native
/// `reversed(<counter-name>)` Function, optionally followed by one direct
/// `Integer`-typed `Number` component. A `reversed` Function whose decoded
/// name matches ASCII-case-insensitively but whose direct inner grammar is
/// not exactly one valid `<counter-name>` `Ident` is never itself a valid
/// item name; it is classified the same generic, unevaluated way as any
/// other unrecognized Function, so it is decisively invalid at a required
/// name position and a provisional feasible-integer-slot ambiguity at an
/// optional-integer position, identically to `counter-increment`. This
/// leaf performs no numeric Function evaluation, no machine-integer
/// conversion, and no runtime counter semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssCounterResetQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssCounterResetQualificationOutcome,
}

impl CssCounterResetQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssCounterResetQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `counter-set` `<counter-name>` recognition, reusing the
/// `counter-increment` evidence-reference ownership pattern (#600). The
/// index is evidence placement, not the interpreted identifier itself; the
/// decoded `<counter-name>` text remains the tokenizer-owned decoded `Ident`
/// value at that exact retained position, resolved through
/// `counter_set_name_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterSetNameEvidenceRef {
    lexical_item_index: usize,
}

impl CssCounterSetNameEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `counter-set` direct explicit `<integer>` recognition. The
/// index is evidence placement, not a copied numeric value; the exact
/// retained `Number`-token structure (sign/zero spelling, magnitude) remains
/// tokenizer-owned and is never converted to a machine integer for
/// qualification, resolved through `counter_set_integer_token`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterSetIntegerEvidenceRef {
    lexical_item_index: usize,
}

impl CssCounterSetIntegerEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `counter-set` repeated item: `DirectCounterItem :=
/// DirectCounterNameEvidence DirectIntegerEvidence?` (#600), reusing the
/// `counter-increment` item shape unchanged. Authored omission of the
/// integer is load-bearing and distinct from an explicit `0` -- this leaf
/// never synthesizes the property's interpreted default reset-to-`0` for an
/// omitted item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCounterSetItem {
    name: CssCounterSetNameEvidenceRef,
    explicit_integer: Option<CssCounterSetIntegerEvidenceRef>,
}

impl CssCounterSetItem {
    pub(crate) const fn name(&self) -> CssCounterSetNameEvidenceRef {
        self.name
    }

    pub(crate) const fn explicit_integer(&self) -> Option<CssCounterSetIntegerEvidenceRef> {
        self.explicit_integer
    }
}

/// One authored `counter-set` value under the narrowed direct-authored
/// structured-repetition profile `QualifiedDirectCounterSet := none |
/// DirectCounterItem+` (#600): either the dedicated whole-value `none`
/// sentinel or an ordered, possibly duplicated, one-or-more list of
/// qualified `DirectCounterItem`s. This is not a complete normative
/// `counter-set` grammar: it qualifies only direct-literal items, and it
/// deliberately does not carry a `reversed(<counter-name>)` branch --
/// `counter-set` has no such branch, unlike `counter-reset`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssCounterSetValue {
    None,
    Items(Vec<CssCounterSetItem>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCounterSetUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValuedIntegerSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssCounterSetQualificationOutcome {
    Qualified(CssCounterSetValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssCounterSetUnsupportedReason),
}

/// One selected ordinary declaration's bounded `counter-set` qualification
/// against the narrowed direct-authored profile `none | DirectCounterItem+`
/// where `DirectCounterItem := <counter-name> <integer>?` (#600), composing
/// the accepted `counter-increment` structured-repetition theorem
/// (#592/#593) unchanged. `counter-increment`, not `counter-reset`, is the
/// syntactic/mechanical transfer source, since `counter-set` has no
/// `reversed(<counter-name>)` branch.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `counter-increment`. A sole retained direct `none`
/// Ident, ASCII-case-insensitively, qualifies the dedicated whole-value
/// branch and is never a repeated-item sentinel. Otherwise every top-level,
/// delimiter-free (never comma-separated) component is classified, then
/// components are walked left-to-right: each item requires one direct
/// `<counter-name>` `Ident` (not `none`, `default`, or a CSS-wide keyword),
/// optionally followed by one direct `Integer`-typed `Number` component. An
/// unresolved Function occupying that optional-integer position is a
/// provisional feasible-integer-slot ambiguity -- resolved to
/// `UnsupportedBySelectedValueProfile(FunctionValuedIntegerSlot)` only if no
/// later component decisively invalidates the declaration regardless of what
/// the Function computes to. A Function can never satisfy the required name
/// position; a recognized generic whole-value-only function name (e.g.
/// `first-valid`) occupying a non-whole-value position is always decisively
/// `InvalidForSelectedValueGrammar`, never softened to Unsupported -- and so
/// is a `reversed(...)` Function at any position, since it is a known CSS
/// Lists grammar-native Function belonging to `counter-reset`'s grammar,
/// never merely an unresolved Function that might compute to an integer.
/// This leaf performs no numeric Function evaluation, no machine-integer
/// conversion, and no runtime counter semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssCounterSetQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssCounterSetQualificationOutcome,
}

impl CssCounterSetQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssCounterSetQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `will-change` `<custom-ident>` recognition, reusing the
/// `page` / `transition-property` / `hyphenate-character` / `animation-name`
/// / `anchor-name` / `container-name` / `color-scheme` evidence-reference
/// ownership pattern. The index is evidence placement, not the interpreted
/// identifier itself; the decoded `<custom-ident>` text remains the
/// tokenizer-owned decoded `Ident` value at that exact retained position,
/// resolved through `will_change_custom_ident_value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssWillChangeCustomIdentEvidenceRef {
    lexical_item_index: usize,
}

impl CssWillChangeCustomIdentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One ordered `will-change` `<animateable-feature>` list item: the two
/// predefined keywords, matched ASCII-case-insensitively, or an open-ended
/// tokenizer-owned `<custom-ident>` evidence reference (#598 / css-will-change-1
/// `<animateable-feature> = scroll-position | contents | <custom-ident>`).
/// This leaf never tests whether a `CustomIdent` names an existing built-in
/// CSS property, an alias, or a shorthand -- an unrecognized property-shaped
/// name such as `Not-A-Property` and a custom-property-shaped name such as
/// `--var` are both ordinary valid `CustomIdent` items, indistinguishable in
/// this grammar from `transform`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWillChangeItemValue {
    ScrollPosition,
    Contents,
    CustomIdent(CssWillChangeCustomIdentEvidenceRef),
}

/// One authored `will-change` value: either the dedicated whole-value `auto`
/// branch or the non-empty comma-list `<animateable-feature>#` branch (#598
/// / css-will-change-1 `will-change: auto | <animateable-feature>#`). Unlike
/// `animation-name`'s `none`, `auto` never combines with the list branch --
/// `auto, transform` and `transform, auto` are both
/// `InvalidForSelectedValueGrammar`, never a partial acceptance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssWillChangeValue {
    Auto,
    Features(Vec<CssWillChangeItemValue>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWillChangeUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssWillChangeQualificationOutcome {
    Qualified(CssWillChangeValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssWillChangeUnsupportedReason),
}

/// One selected ordinary declaration's bounded `will-change` qualification
/// against `auto | <animateable-feature>#` where `<animateable-feature> =
/// scroll-position | contents | <custom-ident>` (#598), composing the
/// accepted top-level comma-list theorem (#571/#573/#575/#577) with the
/// accepted open-ended evidence-reference ownership theorem proven by
/// `page` / `transition-property` / `hyphenate-character` / `animation-name`
/// / `anchor-name` / `container-name` / `color-scheme`.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `anchor-name`/`container-name`. A sole retained
/// direct `auto` Ident, ASCII-case-insensitively, qualifies the dedicated
/// whole-value branch and never reaches list segmentation; unlike
/// `animation-name`'s `none`, `auto` is deliberately never a repeated-item
/// sentinel here. A sole CSS-wide keyword preserves the existing whole-value
/// Unsupported boundary. Otherwise every top-level depth-zero-comma-delimited
/// item is classified independently: exactly one direct Ident decoding
/// (ASCII-case-insensitively) to `scroll-position` or `contents` qualifies
/// the corresponding dedicated feature; any other single direct Ident
/// qualifies as an open-ended `<custom-ident>` item unless its decoded
/// identity is `will-change`, `none`, `all`, `auto`, `default`, or a
/// CSS-wide keyword -- the property-local exclusions layered on top of the
/// normal `<custom-ident>` exclusions. Any decisive `Invalid` item anywhere
/// in the list makes the whole declaration `InvalidForSelectedValueGrammar`,
/// preserving exact authored order and duplicate items -- including
/// case-differing duplicates such as `transform, TRANSFORM` -- in the
/// resulting item vector when every item qualifies. `<custom-ident>` identity
/// is never canonicalized, lowercased, deduplicated, or checked against a
/// built-in property registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssWillChangeQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssWillChangeQualificationOutcome,
}

impl CssWillChangeQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssWillChangeQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `scale` direct component recognition (#602). The index is
/// evidence placement, not a copied numeric value; the exact retained
/// `Number`/`Percentage` token structure (sign/zero spelling, magnitude,
/// decimal/exponent shape) remains tokenizer-owned and is never converted
/// to a machine number for qualification, resolved through
/// `scale_component_token`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssScaleComponentEvidenceRef {
    lexical_item_index: usize,
}

impl CssScaleComponentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `scale` component's tokenizer-owned kind (#602). A
/// direct `<number>` and a direct `<percentage>` remain distinct authored
/// branches even though CSS Transforms interprets them as equivalent scale
/// factors downstream; this leaf never normalizes a Percentage token into
/// an interpreted fraction or into a Number token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScaleComponentKind {
    Number,
    Percentage,
}

/// One direct authored `scale` component: `ScaleComponent := DirectNumber |
/// DirectPercentage` (#602), preserving authored kind and exact
/// tokenizer-owned evidence without any interpreted scale-factor
/// conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssScaleComponent {
    kind: CssScaleComponentKind,
    evidence_ref: CssScaleComponentEvidenceRef,
}

impl CssScaleComponent {
    pub(crate) const fn kind(&self) -> CssScaleComponentKind {
        self.kind
    }

    pub(crate) const fn evidence_ref(&self) -> CssScaleComponentEvidenceRef {
        self.evidence_ref
    }
}

/// One authored `scale` value under the narrowed direct-authored profile
/// `QualifiedDirectScale := None | Components(ScaleComponent{1,3})` (#602):
/// either the dedicated whole-value `none` sentinel or an ordered,
/// authored-cardinality-preserving list of one to three qualified direct
/// components. Authored omission of a trailing Y or Z component is never
/// synthesized here -- `scale: 2`, `scale: 2 2`, and `scale: 2 2 1` remain
/// three structurally distinct component vectors, never a materialized
/// three-axis object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssScaleValue {
    None,
    Components(Vec<CssScaleComponent>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssScaleUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssScaleQualificationOutcome {
    Qualified(CssScaleValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssScaleUnsupportedReason),
}

/// One selected ordinary declaration's bounded `scale` qualification
/// against the pin-bounded direct-authored profile `none | [ <number> |
/// <percentage> ]{1,3}` (#602 / css-transforms-2 `scale`), reusing the
/// accepted `border-spacing`/`aspect-ratio` block-depth-aware top-level
/// component partition and residual-Function-profile theorem together with
/// the accepted `opacity` direct Number-vs-Percentage evidence distinction.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for the other selected leaves. A sole retained direct
/// `none` Ident, ASCII-case-insensitively, qualifies the dedicated
/// whole-value branch and never reaches component partitioning; `none` is
/// never a numeric component or an omission sentinel, so it is decisively
/// invalid combined with any other component. A sole CSS-wide keyword
/// preserves the existing whole-value Unsupported boundary. Otherwise the
/// value is partitioned into one to three ordered top-level components
/// using depth-zero Whitespace/Comment trivia as separators -- never a
/// `Comma`, since this grammar is whitespace-separated repetition, not a
/// comma list. Each component qualifies only when it is exactly one direct
/// `Number` or `Percentage` token; a Function-headed component is
/// classified by placement and identity only -- a recognized generic
/// whole-value-only Function embedded in a component position is
/// decisively invalid, while any other Function is a provisional residual
/// ambiguity resolved to `UnsupportedBySelectedValueProfile(FunctionValue)`
/// only if no other component decisively invalidates the declaration
/// regardless of what the Function computes to. More than three components
/// or zero components is decisive `InvalidForSelectedValueGrammar`
/// regardless of Function content. This leaf performs no CSS math
/// evaluation, no machine-number conversion, and no downstream transform
/// semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssScaleQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssScaleQualificationOutcome,
}

impl CssScaleQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssScaleQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `rotate` direct component recognition (#604). The index is
/// evidence placement, not a copied numeric value; the exact retained
/// `Number`/`Dimension` token structure (sign/zero spelling, magnitude,
/// decimal/exponent shape, unit spelling) remains tokenizer-owned and is
/// never converted to a machine number or normalized angle for
/// qualification, resolved through `rotate_evidence_token`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssRotateEvidenceRef {
    lexical_item_index: usize,
}

impl CssRotateEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One authored `rotate` axis keyword (#604): tokenizer-decoded `x`, `y`, or
/// `z`, matched using existing ASCII-case-insensitive CSS keyword
/// comparison. Never converted into vector evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRotateAxisKeyword {
    X,
    Y,
    Z,
}

/// One authored `rotate` axis operand: either a keyword axis or an exact
/// three-`<number>` vector axis (#604). A vector axis retains exact
/// tokenizer-owned evidence for each of its three components in authored
/// order; it is never normalized, never rejected for being the zero
/// vector, and never canonicalized into (or from) a keyword axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRotateAxis {
    Keyword(CssRotateAxisKeyword),
    Vector([CssRotateEvidenceRef; 3]),
}

/// One authored `rotate` non-`none` value: an optional axis operand
/// (absent for the angle-only authored form) plus a direct `<angle>`
/// evidence reference (#604). Authored axis omission is never synthesized
/// as `Z` or as a zero vector -- `30deg`, `z 30deg`, and `0 0 1 30deg`
/// remain three distinct authored structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssRotateRotation {
    axis: Option<CssRotateAxis>,
    angle: CssRotateEvidenceRef,
}

impl CssRotateRotation {
    pub(crate) const fn axis(&self) -> Option<CssRotateAxis> {
        self.axis
    }

    pub(crate) const fn angle(&self) -> CssRotateEvidenceRef {
        self.angle
    }
}

/// One authored `rotate` value under the pin-bounded finite-shape
/// axis-angle profile `none | <angle> | [ x | y | z | <number>{3} ] &&
/// <angle>` (#604 / css-transforms-2 `rotate`): either the dedicated
/// whole-value `none` sentinel or a qualified `Rotation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRotateValue {
    None,
    Rotation(CssRotateRotation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRotateUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssRotateQualificationOutcome {
    Qualified(CssRotateValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssRotateUnsupportedReason),
}

/// One selected ordinary declaration's bounded `rotate` qualification
/// against the pin-bounded finite-shape axis-angle profile `none |
/// <angle> | [ x | y | z | <number>{3} ] && <angle>` (#604 /
/// css-transforms-2 `rotate`).
///
/// After existing block-depth-aware top-level component recognition,
/// direct authored shapes are finite: one component (`<angle>`), two
/// components (a keyword axis and an angle, in either order), or four
/// components (an exact three-`<number>` vector axis and an angle, in
/// either order, with the vector's three `<number>` positions never
/// accepting the angle interleaved between them) -- plus the dedicated
/// whole-value `none` branch. Any other component count is decisive
/// `InvalidForSelectedValueGrammar` regardless of Function content. A
/// residual Function occupying a structurally feasible `<number>` or
/// `<angle>` position resolves to
/// `UnsupportedBySelectedValueProfile(FunctionValue)` only once every
/// other component in the shape is confirmed structurally feasible; a
/// structurally impossible Function assignment is decisive Invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssRotateQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssRotateQualificationOutcome,
}

impl CssRotateQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssRotateQualificationOutcome {
        self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `translate` direct component recognition (#606). The index
/// is evidence placement, not a copied numeric value; the exact retained
/// `Number`/`Dimension`/`Percentage` token structure (sign/zero spelling,
/// magnitude, decimal/exponent shape, unit spelling) remains tokenizer-owned
/// and is never converted to a machine number for qualification, resolved
/// through `translate_component_token`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTranslateComponentEvidenceRef {
    lexical_item_index: usize,
}

impl CssTranslateComponentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `translate` component's tokenizer-owned kind (#606).
/// A direct exact-zero `Number`/`Dimension` `Length` and a direct
/// `Percentage` remain distinct authored branches; this leaf never
/// normalizes `0`, `0px`, or `0%` into a shared interpreted displacement,
/// and never converts a `Percentage` token into a `Length` token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTranslateComponentKind {
    Length,
    Percentage,
}

/// One direct authored `translate` component: `TranslateComponent :=
/// DirectLength | DirectPercentage` (#606), preserving authored kind and
/// exact tokenizer-owned evidence without any interpreted displacement
/// conversion. A `Length` evidence reference may resolve to either an
/// exact-zero `Number` or a recognized CSS length `Dimension`; that
/// distinction remains recoverable from the retained token itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTranslateComponent {
    kind: CssTranslateComponentKind,
    evidence_ref: CssTranslateComponentEvidenceRef,
}

impl CssTranslateComponent {
    pub(crate) const fn kind(&self) -> CssTranslateComponentKind {
        self.kind
    }

    pub(crate) const fn evidence_ref(&self) -> CssTranslateComponentEvidenceRef {
        self.evidence_ref
    }
}

/// One authored `translate` value under the pin-bounded heterogeneous
/// positional-component profile `QualifiedDirectTranslate := None |
/// Components(TranslateComponent{1,3})` (#606): either the dedicated
/// whole-value `none` sentinel or an ordered, authored-cardinality-
/// preserving list of one to three qualified direct components. Authored
/// omission of a trailing Y or Z component is never synthesized here --
/// `translate: 10px`, `translate: 10px 0`, and `translate: 10px 0 0` remain
/// three structurally distinct component vectors, never a materialized
/// three-axis object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTranslateValue {
    None,
    Components(Vec<CssTranslateComponent>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTranslateUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTranslateQualificationOutcome {
    Qualified(CssTranslateValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTranslateUnsupportedReason),
}

/// One selected ordinary declaration's bounded `translate` qualification
/// against the pin-bounded heterogeneous positional profile `none |
/// <length-percentage> [ <length-percentage> <length>? ]?` (#606 /
/// css-transforms-2 `translate`), reusing the accepted `scale`
/// block-depth-aware top-level component partition and residual-Function-
/// profile theorem together with the accepted `word-spacing`/
/// `scroll-margin-top` direct exact-zero-`Number`-as-`Length` and CSS
/// length-`Dimension` recognition.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for the other selected leaves. A sole retained direct
/// `none` Ident, ASCII-case-insensitively, qualifies the dedicated
/// whole-value branch and never reaches component partitioning; `none` is
/// never a translation component, so it is decisively invalid combined with
/// any other component. A sole CSS-wide keyword preserves the existing
/// whole-value Unsupported boundary. Otherwise the value is partitioned
/// into one to three ordered top-level components using depth-zero
/// Whitespace/Comment trivia as separators -- never a `Comma`, since this
/// grammar is whitespace-separated repetition, not a comma list. Each
/// component qualifies only when it is exactly one direct exact-zero
/// `Number`, a `Dimension` with a recognized CSS length unit, or a
/// `Percentage` token; a Function-headed component is classified by
/// placement and identity only. The new semantic pressure over the
/// accepted `scale` shape is positional heterogeneity: slots one and two
/// accept either `Length` or `Percentage`, but the third slot accepts only
/// `Length` -- a direct `Percentage` (including `0%`) in the third slot is
/// decisive `InvalidForSelectedValueGrammar` regardless of any residual
/// Function found elsewhere, since structural/direct decisive invalidity
/// always outranks a provisional Function ambiguity. More than three
/// components or zero components is likewise decisive
/// `InvalidForSelectedValueGrammar`. Only once every component is confirmed
/// structurally feasible does an unresolved residual Function resolve to
/// `UnsupportedBySelectedValueProfile(FunctionValue)`. This leaf performs no
/// CSS math evaluation, no machine-number conversion, no percentage-basis
/// resolution, and no downstream transform semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssTranslateQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTranslateQualificationOutcome,
}

impl CssTranslateQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssTranslateQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `transform-origin` direct Length/Percentage component
/// recognition (#608). A direct keyword component carries no evidence
/// reference -- its full authored identity is already captured by the
/// decoded `CssTransformOriginKeyword` itself -- while a `Length` or
/// `Percentage` component retains this run-local index, resolved through
/// `transform_origin_component_token` without any machine-number
/// conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformOriginComponentEvidenceRef {
    lexical_item_index: usize,
}

impl CssTransformOriginComponentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `transform-origin` position keyword (#608),
/// tokenizer-decoded ASCII-case-insensitively. `Center` is retained as its
/// own identity -- never collapsed into either axis role -- since a single
/// authored `center` may satisfy the horizontal role, the vertical role, or
/// (as a sole one-component value) neither role exclusively.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformOriginKeyword {
    Left,
    Center,
    Right,
    Top,
    Bottom,
}

/// One direct authored `transform-origin` component under the pin-bounded
/// role-sensitive finite position theorem (#608): a direct keyword, a
/// direct `<length>` (an exact-zero `Number` or a `Dimension` with a
/// recognized CSS length unit), or a direct `Percentage`. Authored kind and
/// exact tokenizer-owned evidence are preserved without interpreted
/// coordinate conversion; a `Length` and a `Percentage` remain distinct
/// authored branches even where both would resolve to the same offset
/// downstream (`0` vs `0%`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformOriginComponent {
    Keyword(CssTransformOriginKeyword),
    Length(CssTransformOriginComponentEvidenceRef),
    Percentage(CssTransformOriginComponentEvidenceRef),
}

/// One authored `transform-origin` value under the pin-bounded role-
/// sensitive finite position profile (#608): an ordered, authored-
/// cardinality-preserving list of one to three qualified direct
/// components. Authored omission of a downstream-assumed second `center`
/// or third `0px` is never synthesized here, and authored keyword source
/// order (`top left` vs `left top`) is never canonicalized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTransformOriginValue {
    Components(Vec<CssTransformOriginComponent>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformOriginUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTransformOriginQualificationOutcome {
    Qualified(CssTransformOriginValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTransformOriginUnsupportedReason),
}

/// One selected ordinary declaration's bounded `transform-origin`
/// qualification against the pin-bounded property-local grammar (#608 /
/// css-transforms-1 `transform-origin`):
///
/// ```text
/// [ left | center | right | top | bottom | <length-percentage> ]
/// |
/// [ left | center | right | <length-percentage> ]
/// [ top | center | bottom | <length-percentage> ] <length>?
/// |
/// [ [ center | left | right ] && [ center | top | bottom ] ] <length>?
/// ```
///
/// This property-local grammar is deliberately not routed through a
/// generic `<position>` matcher: the CSS Values explanatory shortcut
/// treating `transform-origin` as `<position> <length>?` is insufficient
/// here, since e.g. `top 20px` is valid `<position>` syntax but decisively
/// invalid `transform-origin` syntax.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for the other selected leaves; `transform-origin` has
/// no dedicated whole-value keyword. Otherwise the value is partitioned
/// into one to three ordered top-level components using depth-zero
/// Whitespace/Comment trivia as separators. Once partitioned, qualification
/// dispatches purely on component count: exactly one component (`H | V | C
/// | LP`), exactly two components (the role-sensitive ordered/`&&` finite
/// theorem below), or exactly three components (a valid two-component
/// position plus a third, `<length>`-only, Z component). Any other
/// cardinality -- zero, or four or more -- is decisive
/// `InvalidForSelectedValueGrammar` regardless of any residual Function
/// present.
///
/// The two-component theorem is role-sensitive and asymmetric: a component
/// pair qualifies iff it satisfies the ordered branch (first component is
/// `left | right | center | <length-percentage>`, second is `top | bottom |
/// center | <length-percentage>`) or, when both components are direct
/// keywords, the reversed keyword-only `&&` branch (first keyword satisfies
/// the vertical-or-center role, second the horizontal-or-center role). A
/// bare numeric component can only occupy the ordered branch's own slot
/// order -- `left 20px` qualifies but `top 20px` does not, and `20px top`
/// qualifies but `20px left` does not -- since a `<length-percentage>` can
/// never stand in for a literal keyword role in the reversed `&&` branch.
/// The third, Z, component of a three-component value must be a direct
/// `<length>`: a direct `Percentage` there (including `0%`) is decisive
/// `InvalidForSelectedValueGrammar` regardless of any residual Function
/// found in the first two components, since structural/direct decisive
/// invalidity always outranks a provisional Function ambiguity.
///
/// A residual Function is `UnsupportedBySelectedValueProfile(FunctionValue)`
/// only when assigning it to the ordered branch's own `<length-percentage>`
/// slot (never a keyword role) could complete a structurally valid finite
/// shape; a Function that could never occupy its position under any such
/// assignment (e.g. `top calc(20px)`, `calc(20px) left`) is decisive
/// `InvalidForSelectedValueGrammar` instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssTransformOriginQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTransformOriginQualificationOutcome,
}

impl CssTransformOriginQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssTransformOriginQualificationOutcome {
        &self.outcome
    }
}

/// One direct authored `transform-box` keyword identity (#610), tokenizer-
/// decoded ASCII-case-insensitively. These five authored identities remain
/// pairwise distinct even where CSS Transforms defines a context-dependent
/// downstream used-value mapping between two of them (e.g. an SVG element
/// without an associated CSS layout box uses authored `content-box` as
/// `fill-box`): that used-value equivalence is never imported here, and this
/// leaf never rewrites one authored identity into another.
// The pinned five-keyword `transform-box` grammar is itself exactly the set
// `content-box | border-box | fill-box | stroke-box | view-box`; each variant
// name mirrors its authoritative CSSWG keyword spelling rather than an
// incidental naming choice, so the shared `Box` postfix is kept intact.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformBoxValue {
    ContentBox,
    BorderBox,
    FillBox,
    StrokeBox,
    ViewBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformBoxUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformBoxQualificationOutcome {
    Qualified(CssTransformBoxValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTransformBoxUnsupportedReason),
}

/// One selected ordinary declaration's `transform-box` value qualification.
///
/// As with the other selected leaves, placement and `occurrence_index` remain
/// run-local references into the exact parser result structurally owned by
/// the enclosing [`CssValueQualificationRunResult`]. This qualifies only the
/// pin-bounded exact five-keyword authored grammar; it proves no transform
/// reference-box selection, SVG/CSS layout-box mapping, or other downstream
/// semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformBoxQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTransformBoxQualificationOutcome,
}

impl CssTransformBoxQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTransformBoxQualificationOutcome {
        self.outcome
    }
}

/// One direct authored `transform-style` keyword identity (#612), tokenizer-
/// decoded ASCII-case-insensitively. This proves only authored grammar
/// membership and exact authored keyword identity -- never the downstream
/// used value, which CSS Transforms 2 defines as forced to `flat` whenever a
/// grouping property is present regardless of the authored keyword. That
/// grouping-property-forced used-value mapping is never imported here, and
/// this leaf never rewrites `Preserve3d` into `Flat` or vice versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformStyleValue {
    Flat,
    Preserve3d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformStyleUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformStyleQualificationOutcome {
    Qualified(CssTransformStyleValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTransformStyleUnsupportedReason),
}

/// One selected ordinary declaration's `transform-style` value qualification.
///
/// As with the other selected leaves, placement and `occurrence_index` remain
/// run-local references into the exact parser result structurally owned by
/// the enclosing [`CssValueQualificationRunResult`]. This qualifies only the
/// pin-bounded exact two-keyword authored grammar; it proves no grouping-
/// property evaluation, 3D rendering context, flattening, or other
/// downstream used-value semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformStyleQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTransformStyleQualificationOutcome,
}

impl CssTransformStyleQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssTransformStyleQualificationOutcome {
        self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `text-indent` direct Length/Percentage component
/// recognition (#615), resolved through `text_indent_component_token`
/// without any machine-number conversion. A `Hanging`/`EachLine` component
/// carries no evidence reference -- its full authored identity is already
/// captured by the decoded keyword variant itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTextIndentComponentEvidenceRef {
    lexical_item_index: usize,
}

impl CssTextIndentComponentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `text-indent` component under the pin-bounded
/// required-typed-anchor-plus-unique-optional-keyword unordered-composition
/// theorem (#615 / css-text-3, css-text-4 `text-indent`:
/// `[ <length-percentage> ] && hanging? && each-line?`): a direct `<length>`
/// (an exact-zero `Number` or a `Dimension` with a recognized CSS length
/// unit), a direct `Percentage`, or one of the two direct optional
/// keywords. A `Length` and a `Percentage` remain distinct authored
/// branches -- and `0`, `0px`, and `0%` each retain distinct run-local
/// evidence -- even where all three would resolve to the same offset
/// downstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextIndentComponent {
    Length(CssTextIndentComponentEvidenceRef),
    Percentage(CssTextIndentComponentEvidenceRef),
    Hanging,
    EachLine,
}

/// One authored `text-indent` value under the pin-bounded finite `&&`
/// composition profile (#615): an ordered, authored-cardinality-preserving
/// list of one to three qualified direct components. The `&&` combinator
/// authorizes any authored order of the required `<length-percentage>`
/// anchor and the two unique optional keywords, but never justifies
/// canonicalizing that order -- authored component order is preserved
/// exactly, never rewritten into a `<length-percentage> hanging each-line`
/// or any other fixed shape, and never confused with CSSOM serialization
/// order, computed representation, or layout effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTextIndentValue {
    Components(Vec<CssTextIndentComponent>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTextIndentUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTextIndentQualificationOutcome {
    Qualified(CssTextIndentValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTextIndentUnsupportedReason),
}

/// One selected ordinary declaration's bounded `text-indent` qualification
/// against the pin-bounded property-local grammar (#615 / css-text-3,
/// css-text-4 `text-indent`):
///
/// ```text
/// [ <length-percentage> ] && hanging? && each-line?
/// ```
///
/// A direct value qualifies iff its ordered top-level components contain
/// exactly one direct `<length-percentage>` anchor, zero or one `hanging`,
/// and zero or one `each-line`, in any authored order -- direct authored
/// cardinality is exactly one to three. An ordinary residual Function may
/// only satisfy the single `<length-percentage>` role: it is
/// `UnsupportedBySelectedValueProfile(FunctionValue)` only while a valid
/// finite role assignment remains possible (e.g. `hanging calc(...)`), and
/// decisively `InvalidForSelectedValueGrammar` whenever no assignment can
/// complete the grammar (e.g. `10px calc(...)`, `calc(...) calc(...)`,
/// `hanging hanging calc(...)`) -- structural/direct decisive invalidity,
/// including any duplicate anchor or duplicate keyword, always outranks a
/// provisional Function ambiguity.
///
/// This qualifies only authored grammar membership and authored component
/// identity/order; it proves no first-formatted-line selection, forced/soft
/// line-break behavior, block layout or indentation geometry, percentage
/// basis resolution, inheritance, computed/used values, or CSSOM
/// serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssTextIndentQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTextIndentQualificationOutcome,
}

impl CssTextIndentQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssTextIndentQualificationOutcome {
        &self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLetterSpacingValue {
    Normal,
    DirectLengthLiteral,
    DirectPercentageLiteral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLetterSpacingUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    FunctionValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLetterSpacingQualificationOutcome {
    Qualified(CssLetterSpacingValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssLetterSpacingUnsupportedReason),
}

/// One selected ordinary declaration's bounded `letter-spacing` qualification
/// (#619), reusing the accepted direct `word-spacing` profile: `normal` and
/// unrestricted signed `<length-percentage>` evidence.
///
/// This qualifies only authored grammar membership; it performs no
/// percentage resolution, glyph advance adjustment, bidi distribution, or
/// shaping/layout/rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssLetterSpacingQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssLetterSpacingQualificationOutcome,
}

impl CssLetterSpacingQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssLetterSpacingQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCaretAnimationValue {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCaretAnimationUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCaretAnimationQualificationOutcome {
    Qualified(CssCaretAnimationValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssCaretAnimationUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored `caret-animation`
/// qualification (#636): exactly one direct decoded Ident `auto` or `manual`.
///
/// This proves only authored keyword identity. `Auto` records that the UA
/// decides whether/how to animate the caret; `Manual` records only that
/// UA-driven caret animation is disabled -- CSS animations affecting the
/// caret remain unaffected and are not represented here. This observation
/// performs no UA/platform blink-or-fade timing, CSS animation execution,
/// applicability filtering, or inheritance/cascade/computed-value semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCaretAnimationQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssCaretAnimationQualificationOutcome,
}

impl CssCaretAnimationQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssCaretAnimationQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCaretShapeValue {
    Auto,
    Bar,
    Block,
    Underscore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCaretShapeUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssCaretShapeQualificationOutcome {
    Qualified(CssCaretShapeValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssCaretShapeUnsupportedReason),
}

/// One selected ordinary declaration's bounded authored `caret-shape`
/// qualification (#638): exactly one direct decoded Ident `auto`, `bar`,
/// `block`, or `underscore`.
///
/// This proves only authored keyword identity. `Auto` records that the UA
/// selects the effective caret shape; `Bar`, `Block`, and `Underscore` record
/// only the authored identity, even when downstream UA/runtime behavior
/// (including IME composition) renders a different shape. This observation
/// performs no UA/platform effective-shape selection, IME composition-state
/// integration, caret rendering/geometry, applicability filtering, or
/// inheritance/cascade/computed-value semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssCaretShapeQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssCaretShapeQualificationOutcome,
}

impl CssCaretShapeQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> CssCaretShapeQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationFillModeValue {
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssAnimationFillModeUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssAnimationFillModeQualificationOutcome {
    Qualified(Vec<CssAnimationFillModeValue>),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssAnimationFillModeUnsupportedReason),
}

/// One selected ordinary declaration's bounded `animation-fill-mode`
/// qualification (#641): `<single-animation-fill-mode>#`, where each item is
/// exactly one direct decoded Ident `none | forwards | backwards | both`.
///
/// This proves only authored comma-list sequence identity, in exact authored
/// order and cardinality. CSS `animation-fill-mode` does not accept `auto`
/// (that keyword belongs only to the Web Animations `FillMode` enum), so
/// `auto` remains `InvalidForSelectedValueGrammar` here. This observation
/// performs no runtime fill-effect application, `AnimationEffect`/
/// `AnimationTrigger` semantics, playback/currentTime state, sibling
/// animation-list synchronization, shorthand expansion, or
/// inheritance/cascade/computed-value semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssAnimationFillModeQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssAnimationFillModeQualificationOutcome,
}

impl CssAnimationFillModeQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssAnimationFillModeQualificationOutcome {
        &self.outcome
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `transform` `matrix()` direct `<number>` argument
/// recognition (#418). The index is evidence placement, not an interpreted
/// numeric magnitude: it always points at the exact tokenizer-owned direct
/// `Number` token retained inside one `matrix()` argument slot, never at
/// the Function opener, an argument-separating `Comma`, a parenthesis, or
/// trivia. The exact retained `Number`-token structure (sign spelling,
/// integer/fraction digits, exponent spelling) remains tokenizer-owned and
/// is never converted to a machine float for qualification, resolved
/// through `transform_matrix_argument_token`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformMatrixArgumentEvidenceRef {
    lexical_item_index: usize,
}

impl CssTransformMatrixArgumentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One qualified authored `matrix()` transform component: exactly six
/// ordered direct authored `<number>` arguments (#418), each carrying the
/// exact tokenizer-owned evidence retained at its own semantic argument
/// slot. The arity is carried structurally by a fixed-size array, so a
/// qualified component can never represent a short, long, or partially
/// synthesized argument vector. This leaf constructs no matrix, performs
/// no matrix multiplication, and resolves no computed transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformMatrixFunction {
    arguments: [CssTransformMatrixArgumentEvidenceRef; 6],
}

impl CssTransformMatrixFunction {
    pub(crate) const fn arguments(&self) -> &[CssTransformMatrixArgumentEvidenceRef; 6] {
        &self.arguments
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `transform` `scale()` direct `<number>`/`<percentage>`
/// argument recognition (#645). Mirrors
/// `CssTransformMatrixArgumentEvidenceRef`: the index is evidence
/// placement, not an interpreted numeric magnitude, and always points at
/// the exact tokenizer-owned direct `Number` or `Percentage` token
/// retained inside one `scale()` argument slot, resolved through
/// `transform_scale_argument_token`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformScaleArgumentEvidenceRef {
    lexical_item_index: usize,
}

impl CssTransformScaleArgumentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `scale()` transform-function argument's
/// tokenizer-owned kind (#645). A direct `<number>` and a direct
/// `<percentage>` remain distinct authored branches; this leaf never
/// normalizes a Percentage token into an interpreted fraction or into a
/// Number token. This is a distinct type from the longhand `scale`
/// property's `CssScaleComponentKind`: longhand property component
/// placement and `transform` function argument placement remain distinct
/// semantic roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformScaleArgumentKind {
    Number,
    Percentage,
}

/// One direct authored `scale()` transform-function argument:
/// `ScaleArgument := DirectNumber | DirectPercentage` (#645), preserving
/// authored kind and exact tokenizer-owned evidence without any
/// interpreted scale-factor conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformScaleArgument {
    kind: CssTransformScaleArgumentKind,
    evidence_ref: CssTransformScaleArgumentEvidenceRef,
}

impl CssTransformScaleArgument {
    pub(crate) const fn kind(&self) -> CssTransformScaleArgumentKind {
        self.kind
    }

    pub(crate) const fn evidence_ref(&self) -> CssTransformScaleArgumentEvidenceRef {
        self.evidence_ref
    }
}

/// One qualified `scale()` transform component's ordered argument list
/// (#645), carrying authored one-vs-two cardinality structurally so a
/// qualified component can never represent zero, three, or a synthesized
/// omitted argument. `scale(2)` is `One`; `scale(2, 3)` is `Two`; there is
/// no third shape to construct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformScaleArguments {
    One(CssTransformScaleArgument),
    Two(CssTransformScaleArgument, CssTransformScaleArgument),
}

impl CssTransformScaleArguments {
    /// The always-present first authored argument.
    pub(crate) const fn first(&self) -> CssTransformScaleArgument {
        match self {
            Self::One(first) | Self::Two(first, _) => *first,
        }
    }

    /// The second authored argument, or `None` when only one was authored.
    /// Never synthesized: an authored one-argument `scale()` has no second
    /// evidence to report.
    pub(crate) const fn second(&self) -> Option<CssTransformScaleArgument> {
        match self {
            Self::One(_) => None,
            Self::Two(_, second) => Some(*second),
        }
    }
}

/// One qualified authored `scale()` transform component: one or two
/// ordered direct authored `<number>`/`<percentage>` arguments (#645),
/// each carrying the exact tokenizer-owned evidence retained at its own
/// semantic argument slot. This leaf constructs no scale factor, performs
/// no matrix construction, and resolves no computed transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformScaleFunction {
    arguments: CssTransformScaleArguments,
}

impl CssTransformScaleFunction {
    pub(crate) const fn arguments(&self) -> &CssTransformScaleArguments {
        &self.arguments
    }
}

/// Run-local locator for the exact tokenizer item selected during
/// authoritative `transform` `translate3d()` direct `<length>`/
/// `<length-percentage>` argument recognition (#647). Mirrors
/// `CssTransformScaleArgumentEvidenceRef`: the index is evidence placement,
/// not an interpreted numeric magnitude, and always points at the exact
/// tokenizer-owned direct `Number`, `Dimension`, or `Percentage` token
/// retained inside one `translate3d()` argument slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformTranslate3dArgumentEvidenceRef {
    lexical_item_index: usize,
}

impl CssTransformTranslate3dArgumentEvidenceRef {
    pub(crate) const fn lexical_item_index(&self) -> usize {
        self.lexical_item_index
    }
}

/// One direct authored `translate3d()` X/Y argument's tokenizer-owned kind
/// (#647): `XyArgument := DirectLength | DirectPercentage`. A `Percentage`
/// is never collapsed into a `Length`, matching the accepted `scale`
/// Number/Percentage distinction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformTranslate3dXyArgumentKind {
    Length,
    Percentage,
}

/// One direct authored `translate3d()` X or Y argument, preserving authored
/// kind and exact tokenizer-owned evidence without any interpreted-length
/// conversion (#647). `DirectLength` here is either a `Dimension` with a
/// recognized CSS length unit or a direct exact-zero `Number`, reusing the
/// accepted `translate` (#606) zero-as-length theorem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformTranslate3dXyArgument {
    kind: CssTransformTranslate3dXyArgumentKind,
    evidence_ref: CssTransformTranslate3dArgumentEvidenceRef,
}

impl CssTransformTranslate3dXyArgument {
    pub(crate) const fn kind(&self) -> CssTransformTranslate3dXyArgumentKind {
        self.kind
    }

    pub(crate) const fn evidence_ref(&self) -> CssTransformTranslate3dArgumentEvidenceRef {
        self.evidence_ref
    }
}

/// One qualified authored `translate3d()` transform component: exactly
/// three ordered, position-sensitive direct authored arguments (#647) --
/// `X`/`Y` may each be a direct `<length>` or `<percentage>`, while `Z` is
/// restricted to a direct `<length>` only. `Z`'s field type carries no
/// `Percentage` variant at all, so a qualified component can never
/// represent a `Percentage` in the Z position; this leaf constructs no
/// translation, resolves no percentage basis, and performs no unit
/// conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssTransformTranslate3dFunction {
    x: CssTransformTranslate3dXyArgument,
    y: CssTransformTranslate3dXyArgument,
    z: CssTransformTranslate3dArgumentEvidenceRef,
}

impl CssTransformTranslate3dFunction {
    pub(crate) const fn x(&self) -> CssTransformTranslate3dXyArgument {
        self.x
    }

    pub(crate) const fn y(&self) -> CssTransformTranslate3dXyArgument {
        self.y
    }

    /// The Z argument's evidence, always a direct `<length>`: this type has
    /// no `Percentage` variant to hold, so a Z `Percentage` can never reach
    /// this field.
    pub(crate) const fn z(&self) -> CssTransformTranslate3dArgumentEvidenceRef {
        self.z
    }
}

/// One qualified selected `transform` component under the profile
/// `SelectedTransformFunction := Matrix | Scale | Translate3d` (#418 /
/// #645 / #647), preserving exact authored order between the three
/// selected function kinds. This is a closed property-local alternation,
/// not a generic CSS function AST: it exists only to retain heterogeneous
/// authored order for the selected `transform` branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformFunction {
    Matrix(CssTransformMatrixFunction),
    Scale(CssTransformScaleFunction),
    Translate3d(CssTransformTranslate3dFunction),
}

/// One authored `transform` value under the direct-authored profile
/// `QualifiedDirectTransform := none | [ matrix(<number>#{6}) |
/// scale([<number> | <percentage>]#{1,2}) | translate3d(<length-percentage>,
/// <length-percentage>, <length>) ]+` (#418 / #645 / #647): either the
/// dedicated whole-value `none` sentinel or an ordered, possibly
/// repeated, one-or-more list of qualified `matrix()`/`scale()`/
/// `translate3d()` components in exact authored order. This is not a
/// complete normative `transform` grammar: it qualifies only the
/// `matrix()`, `scale()`, and `translate3d()` branches of
/// `<transform-function>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTransformValue {
    None,
    Functions(Vec<CssTransformFunction>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTransformUnsupportedReason {
    CssWideKeyword,
    DeferredSubstitutionFunction,
    WholeValueFunction,
    UnselectedTransformFunction,
    FunctionValuedTransformArgument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTransformQualificationOutcome {
    Qualified(CssTransformValue),
    InvalidForSelectedValueGrammar,
    UnsupportedBySelectedValueProfile(CssTransformUnsupportedReason),
}

/// One selected ordinary declaration's bounded `transform` qualification
/// against the direct-authored profile `QualifiedDirectTransform := none |
/// [ matrix(<number>#{6}) | scale([<number> | <percentage>]#{1,2}) |
/// translate3d(<length-percentage>, <length-percentage>, <length>) ]+`
/// (#418 / #645 / #647), reusing this repository's depth-scoped
/// grammar-native multi-argument function qualification for a third
/// selected branch.
///
/// Normative `transform` is `none | <transform-list>` where
/// `<transform-list> = <transform-function>+`, `matrix() =
/// matrix(<number>#{6})`, `scale() = scale([<number> |
/// <percentage>]#{1,2})`, and `translate3d() =
/// translate3d(<length-percentage>, <length-percentage>, <length>)`. This
/// observation deliberately selects only the `matrix()`, `scale()`, and
/// `translate3d()` branches: every other `<transform-function>` stays
/// outside selected-profile coverage rather than being decided here,
/// because deciding it would require the full `<transform-function>`
/// dispatch and the length/angle semantics this leaf does not own.
///
/// The shared semantic responsibility is argument qualification scoped to
/// a grammar frame whose delimiter depth is relative to each selected
/// function's own body: the body is partitioned on `Comma` tokens at that
/// relative depth zero only, so a comma retained inside a nested Function
/// or block never becomes an argument separator, the resulting ordered
/// slots are preserved (an authored-empty position is retained as a slot
/// and rejected, never collapsed), and exactly six slots each holding
/// exactly one direct retained `Number` token qualify a `matrix()`, one or
/// two slots each holding exactly one direct retained `Number` or
/// `Percentage` token qualify a `scale()`, and exactly three
/// position-sensitive slots -- `Length | Percentage`, `Length |
/// Percentage`, then `Length` only -- qualify a `translate3d()`.
///
/// This observation performs no machine-float normalization, constructs no
/// matrix, multiplies no matrices, resolves no computed value, applies no
/// `calc()` semantics, and adds no browser layout or rendering behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssTransformQualificationObservation {
    occurrence_index: usize,
    placement: CssDeclarationPlacement,
    outcome: CssTransformQualificationOutcome,
}

impl CssTransformQualificationObservation {
    pub(crate) const fn occurrence_index(&self) -> usize {
        self.occurrence_index
    }

    pub(crate) const fn placement(&self) -> CssDeclarationPlacement {
        self.placement
    }

    pub(crate) const fn outcome(&self) -> &CssTransformQualificationOutcome {
        &self.outcome
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
    fill_opacity_observations: Vec<CssFillOpacityQualificationObservation>,
    shape_image_threshold_observations: Vec<CssShapeImageThresholdQualificationObservation>,
    shape_margin_observations: Vec<CssShapeMarginQualificationObservation>,
    line_height_observations: Vec<CssLineHeightQualificationObservation>,
    line_break_observations: Vec<CssLineBreakQualificationObservation>,
    print_color_adjust_observations: Vec<CssPrintColorAdjustQualificationObservation>,
    overflow_wrap_observations: Vec<CssOverflowWrapQualificationObservation>,
    unicode_bidi_observations: Vec<CssUnicodeBidiQualificationObservation>,
    mask_type_observations: Vec<CssMaskTypeQualificationObservation>,
    color_interpolation_filters_observations:
        Vec<CssColorInterpolationFiltersQualificationObservation>,
    shape_rendering_observations: Vec<CssShapeRenderingQualificationObservation>,
    text_rendering_observations: Vec<CssTextRenderingQualificationObservation>,
    text_anchor_observations: Vec<CssTextAnchorQualificationObservation>,
    forced_color_adjust_observations: Vec<CssForcedColorAdjustQualificationObservation>,
    text_align_last_observations: Vec<CssTextAlignLastQualificationObservation>,
    math_style_observations: Vec<CssMathStyleQualificationObservation>,
    math_shift_observations: Vec<CssMathShiftQualificationObservation>,
    ruby_align_observations: Vec<CssRubyAlignQualificationObservation>,
    ruby_merge_observations: Vec<CssRubyMergeQualificationObservation>,
    ruby_position_observations: Vec<CssRubyPositionQualificationObservation>,
    ruby_overhang_observations: Vec<CssRubyOverhangQualificationObservation>,
    clip_rule_observations: Vec<CssClipRuleQualificationObservation>,
    fill_rule_observations: Vec<CssFillRuleQualificationObservation>,
    column_fill_observations: Vec<CssColumnFillQualificationObservation>,
    text_decoration_skip_ink_observations: Vec<CssTextDecorationSkipInkQualificationObservation>,
    overscroll_behavior_observations: Vec<CssOverscrollBehaviorQualificationObservation>,
    contain_observations: Vec<CssContainQualificationObservation>,
    font_variant_ligatures_observations: Vec<CssFontVariantLigaturesQualificationObservation>,
    font_variant_numeric_observations: Vec<CssFontVariantNumericQualificationObservation>,
    text_decoration_line_observations: Vec<CssTextDecorationLineQualificationObservation>,
    text_transform_observations: Vec<CssTextTransformQualificationObservation>,
    text_emphasis_position_observations: Vec<CssTextEmphasisPositionQualificationObservation>,
    overscroll_behavior_x_observations: Vec<CssOverscrollBehaviorXQualificationObservation>,
    overscroll_behavior_y_observations: Vec<CssOverscrollBehaviorYQualificationObservation>,
    overscroll_behavior_inline_observations:
        Vec<CssOverscrollBehaviorInlineQualificationObservation>,
    overscroll_behavior_block_observations: Vec<CssOverscrollBehaviorBlockQualificationObservation>,
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
    font_synthesis_small_caps_observations: Vec<CssFontSynthesisSmallCapsQualificationObservation>,
    font_synthesis_position_observations: Vec<CssFontSynthesisPositionQualificationObservation>,
    font_variant_emoji_observations: Vec<CssFontVariantEmojiQualificationObservation>,
    font_variant_caps_observations: Vec<CssFontVariantCapsQualificationObservation>,
    font_variant_position_observations: Vec<CssFontVariantPositionQualificationObservation>,
    font_weight_observations: Vec<CssFontWeightQualificationObservation>,
    page_observations: Vec<CssPageQualificationObservation>,
    border_spacing_observations: Vec<CssBorderSpacingQualificationObservation>,
    z_index_observations: Vec<CssZIndexQualificationObservation>,
    aspect_ratio_observations: Vec<CssAspectRatioQualificationObservation>,
    animation_play_state_observations: Vec<CssAnimationPlayStateQualificationObservation>,
    animation_iteration_count_observations: Vec<CssAnimationIterationCountQualificationObservation>,
    animation_delay_observations: Vec<CssAnimationDelayQualificationObservation>,
    transition_duration_observations: Vec<CssTransitionDurationQualificationObservation>,
    transition_property_observations: Vec<CssTransitionPropertyQualificationObservation>,
    hyphenate_character_observations: Vec<CssHyphenateCharacterQualificationObservation>,
    animation_name_observations: Vec<CssAnimationNameQualificationObservation>,
    anchor_name_observations: Vec<CssAnchorNameQualificationObservation>,
    offset_rotate_observations: Vec<CssOffsetRotateQualificationObservation>,
    container_name_observations: Vec<CssContainerNameQualificationObservation>,
    color_scheme_observations: Vec<CssColorSchemeQualificationObservation>,
    counter_increment_observations: Vec<CssCounterIncrementQualificationObservation>,
    counter_reset_observations: Vec<CssCounterResetQualificationObservation>,
    counter_set_observations: Vec<CssCounterSetQualificationObservation>,
    image_resolution_observations: Vec<CssImageResolutionQualificationObservation>,
    will_change_observations: Vec<CssWillChangeQualificationObservation>,
    scale_observations: Vec<CssScaleQualificationObservation>,
    rotate_observations: Vec<CssRotateQualificationObservation>,
    translate_observations: Vec<CssTranslateQualificationObservation>,
    transform_origin_observations: Vec<CssTransformOriginQualificationObservation>,
    transform_box_observations: Vec<CssTransformBoxQualificationObservation>,
    transform_style_observations: Vec<CssTransformStyleQualificationObservation>,
    text_indent_observations: Vec<CssTextIndentQualificationObservation>,
    letter_spacing_observations: Vec<CssLetterSpacingQualificationObservation>,
    text_underline_position_observations: Vec<CssTextUnderlinePositionQualificationObservation>,
    list_style_position_observations: Vec<CssListStylePositionQualificationObservation>,
    stroke_linecap_observations: Vec<CssStrokeLinecapQualificationObservation>,
    stroke_opacity_observations: Vec<CssStrokeOpacityQualificationObservation>,
    stop_opacity_observations: Vec<CssStopOpacityQualificationObservation>,
    flood_opacity_observations: Vec<CssFloodOpacityQualificationObservation>,
    paint_order_observations: Vec<CssPaintOrderQualificationObservation>,
    caret_animation_observations: Vec<CssCaretAnimationQualificationObservation>,
    caret_shape_observations: Vec<CssCaretShapeQualificationObservation>,
    animation_fill_mode_observations: Vec<CssAnimationFillModeQualificationObservation>,
    transform_observations: Vec<CssTransformQualificationObservation>,
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

    pub(crate) fn fill_opacity_observations(&self) -> &[CssFillOpacityQualificationObservation] {
        &self.fill_opacity_observations
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

    pub(crate) fn line_break_observations(&self) -> &[CssLineBreakQualificationObservation] {
        &self.line_break_observations
    }

    pub(crate) fn print_color_adjust_observations(
        &self,
    ) -> &[CssPrintColorAdjustQualificationObservation] {
        &self.print_color_adjust_observations
    }

    pub(crate) fn overflow_wrap_observations(&self) -> &[CssOverflowWrapQualificationObservation] {
        &self.overflow_wrap_observations
    }

    pub(crate) fn unicode_bidi_observations(&self) -> &[CssUnicodeBidiQualificationObservation] {
        &self.unicode_bidi_observations
    }

    pub(crate) fn mask_type_observations(&self) -> &[CssMaskTypeQualificationObservation] {
        &self.mask_type_observations
    }

    pub(crate) fn color_interpolation_filters_observations(
        &self,
    ) -> &[CssColorInterpolationFiltersQualificationObservation] {
        &self.color_interpolation_filters_observations
    }

    pub(crate) fn shape_rendering_observations(
        &self,
    ) -> &[CssShapeRenderingQualificationObservation] {
        &self.shape_rendering_observations
    }

    pub(crate) fn text_rendering_observations(
        &self,
    ) -> &[CssTextRenderingQualificationObservation] {
        &self.text_rendering_observations
    }

    pub(crate) fn text_anchor_observations(&self) -> &[CssTextAnchorQualificationObservation] {
        &self.text_anchor_observations
    }

    pub(crate) fn forced_color_adjust_observations(
        &self,
    ) -> &[CssForcedColorAdjustQualificationObservation] {
        &self.forced_color_adjust_observations
    }

    pub(crate) fn text_align_last_observations(
        &self,
    ) -> &[CssTextAlignLastQualificationObservation] {
        &self.text_align_last_observations
    }

    pub(crate) fn math_style_observations(&self) -> &[CssMathStyleQualificationObservation] {
        &self.math_style_observations
    }

    pub(crate) fn math_shift_observations(&self) -> &[CssMathShiftQualificationObservation] {
        &self.math_shift_observations
    }

    pub(crate) fn ruby_align_observations(&self) -> &[CssRubyAlignQualificationObservation] {
        &self.ruby_align_observations
    }

    pub(crate) fn ruby_merge_observations(&self) -> &[CssRubyMergeQualificationObservation] {
        &self.ruby_merge_observations
    }

    pub(crate) fn ruby_position_observations(&self) -> &[CssRubyPositionQualificationObservation] {
        &self.ruby_position_observations
    }

    pub(crate) fn ruby_overhang_observations(&self) -> &[CssRubyOverhangQualificationObservation] {
        &self.ruby_overhang_observations
    }

    pub(crate) fn clip_rule_observations(&self) -> &[CssClipRuleQualificationObservation] {
        &self.clip_rule_observations
    }

    pub(crate) fn fill_rule_observations(&self) -> &[CssFillRuleQualificationObservation] {
        &self.fill_rule_observations
    }

    pub(crate) fn column_fill_observations(&self) -> &[CssColumnFillQualificationObservation] {
        &self.column_fill_observations
    }

    pub(crate) fn text_decoration_skip_ink_observations(
        &self,
    ) -> &[CssTextDecorationSkipInkQualificationObservation] {
        &self.text_decoration_skip_ink_observations
    }

    pub(crate) fn overscroll_behavior_observations(
        &self,
    ) -> &[CssOverscrollBehaviorQualificationObservation] {
        &self.overscroll_behavior_observations
    }

    pub(crate) fn contain_observations(&self) -> &[CssContainQualificationObservation] {
        &self.contain_observations
    }

    pub(crate) fn font_variant_ligatures_observations(
        &self,
    ) -> &[CssFontVariantLigaturesQualificationObservation] {
        &self.font_variant_ligatures_observations
    }

    pub(crate) fn font_variant_numeric_observations(
        &self,
    ) -> &[CssFontVariantNumericQualificationObservation] {
        &self.font_variant_numeric_observations
    }

    pub(crate) fn text_decoration_line_observations(
        &self,
    ) -> &[CssTextDecorationLineQualificationObservation] {
        &self.text_decoration_line_observations
    }

    pub(crate) fn text_transform_observations(
        &self,
    ) -> &[CssTextTransformQualificationObservation] {
        &self.text_transform_observations
    }

    pub(crate) fn text_emphasis_position_observations(
        &self,
    ) -> &[CssTextEmphasisPositionQualificationObservation] {
        &self.text_emphasis_position_observations
    }

    pub(crate) fn overscroll_behavior_x_observations(
        &self,
    ) -> &[CssOverscrollBehaviorXQualificationObservation] {
        &self.overscroll_behavior_x_observations
    }

    pub(crate) fn overscroll_behavior_y_observations(
        &self,
    ) -> &[CssOverscrollBehaviorYQualificationObservation] {
        &self.overscroll_behavior_y_observations
    }

    pub(crate) fn overscroll_behavior_inline_observations(
        &self,
    ) -> &[CssOverscrollBehaviorInlineQualificationObservation] {
        &self.overscroll_behavior_inline_observations
    }

    pub(crate) fn overscroll_behavior_block_observations(
        &self,
    ) -> &[CssOverscrollBehaviorBlockQualificationObservation] {
        &self.overscroll_behavior_block_observations
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

    pub(crate) fn font_synthesis_small_caps_observations(
        &self,
    ) -> &[CssFontSynthesisSmallCapsQualificationObservation] {
        &self.font_synthesis_small_caps_observations
    }

    pub(crate) fn font_synthesis_position_observations(
        &self,
    ) -> &[CssFontSynthesisPositionQualificationObservation] {
        &self.font_synthesis_position_observations
    }

    pub(crate) fn font_variant_emoji_observations(
        &self,
    ) -> &[CssFontVariantEmojiQualificationObservation] {
        &self.font_variant_emoji_observations
    }

    pub(crate) fn font_variant_caps_observations(
        &self,
    ) -> &[CssFontVariantCapsQualificationObservation] {
        &self.font_variant_caps_observations
    }

    pub(crate) fn font_variant_position_observations(
        &self,
    ) -> &[CssFontVariantPositionQualificationObservation] {
        &self.font_variant_position_observations
    }

    pub(crate) fn font_weight_observations(&self) -> &[CssFontWeightQualificationObservation] {
        &self.font_weight_observations
    }

    pub(crate) fn page_observations(&self) -> &[CssPageQualificationObservation] {
        &self.page_observations
    }

    pub(crate) fn page_custom_ident_value<'a>(
        &'a self,
        observation: &CssPageQualificationObservation,
    ) -> Option<&'a str> {
        if observation.outcome()
            != CssPageQualificationOutcome::Qualified(CssPageValue::CustomIdent)
        {
            return None;
        }
        let evidence = observation.custom_ident_evidence()?;
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    pub(crate) fn border_spacing_observations(
        &self,
    ) -> &[CssBorderSpacingQualificationObservation] {
        &self.border_spacing_observations
    }

    pub(crate) fn z_index_observations(&self) -> &[CssZIndexQualificationObservation] {
        &self.z_index_observations
    }

    pub(crate) fn aspect_ratio_observations(&self) -> &[CssAspectRatioQualificationObservation] {
        &self.aspect_ratio_observations
    }

    pub(crate) fn animation_play_state_observations(
        &self,
    ) -> &[CssAnimationPlayStateQualificationObservation] {
        &self.animation_play_state_observations
    }

    pub(crate) fn animation_iteration_count_observations(
        &self,
    ) -> &[CssAnimationIterationCountQualificationObservation] {
        &self.animation_iteration_count_observations
    }

    pub(crate) fn animation_delay_observations(
        &self,
    ) -> &[CssAnimationDelayQualificationObservation] {
        &self.animation_delay_observations
    }

    pub(crate) fn transition_duration_observations(
        &self,
    ) -> &[CssTransitionDurationQualificationObservation] {
        &self.transition_duration_observations
    }

    pub(crate) fn transition_property_observations(
        &self,
    ) -> &[CssTransitionPropertyQualificationObservation] {
        &self.transition_property_observations
    }

    /// Resolves one qualified `transition-property` custom-ident item's
    /// tokenizer-owned decoded identity through its run-local evidence
    /// reference, mirroring `page_custom_ident_value` without copying the
    /// identifier payload into a second owner.
    pub(crate) fn transition_property_custom_ident_value(
        &self,
        evidence: CssTransitionPropertyCustomIdentEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    pub(crate) fn hyphenate_character_observations(
        &self,
    ) -> &[CssHyphenateCharacterQualificationObservation] {
        &self.hyphenate_character_observations
    }

    /// Resolves one qualified `hyphenate-character` direct String's
    /// tokenizer-owned decoded identity through its run-local evidence
    /// reference, mirroring `page_custom_ident_value` and
    /// `transition_property_custom_ident_value` without copying the
    /// open-ended String payload into a second owner.
    pub(crate) fn hyphenate_character_string_value<'a>(
        &'a self,
        observation: &CssHyphenateCharacterQualificationObservation,
    ) -> Option<&'a str> {
        if observation.outcome()
            != CssHyphenateCharacterQualificationOutcome::Qualified(
                CssHyphenateCharacterValue::DirectStringLiteral,
            )
        {
            return None;
        }
        let evidence = observation.string_evidence()?;
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::String(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    pub(crate) fn animation_name_observations(
        &self,
    ) -> &[CssAnimationNameQualificationObservation] {
        &self.animation_name_observations
    }

    /// Resolves one qualified `animation-name` `KeyframesName` item's
    /// tokenizer-owned decoded interpreted identity through its run-local
    /// evidence reference, mirroring `page_custom_ident_value`,
    /// `transition_property_custom_ident_value`, and
    /// `hyphenate_character_string_value` without copying the payload into a
    /// second owner. Unlike those single-kind resolvers, the retained token
    /// at the evidence position may be either a direct `Ident` or a direct
    /// non-empty `String`, since `<keyframes-name> = <custom-ident> |
    /// <string>`; both resolve to the same interpreted `&str` shape here, and
    /// their authored lexical kind remains separately inspectable from the
    /// exact retained token at `lexical_item_index()`.
    pub(crate) fn animation_name_keyframes_name_value(
        &self,
        evidence: CssAnimationNameKeyframesNameEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        match token.kind() {
            CssTokenKind::Ident(value) => Some(value.as_str()),
            CssTokenKind::String(value) => Some(value.as_str()),
            _ => None,
        }
    }

    pub(crate) fn anchor_name_observations(&self) -> &[CssAnchorNameQualificationObservation] {
        &self.anchor_name_observations
    }

    /// Resolves one qualified `anchor-name` `<dashed-ident>` item's
    /// tokenizer-owned decoded interpreted identity through its run-local
    /// evidence reference, mirroring `page_custom_ident_value`,
    /// `transition_property_custom_ident_value`, and
    /// `animation_name_keyframes_name_value` without copying the payload into
    /// a second owner. The retained token at the evidence position is always
    /// a direct `Ident`, since this item grammar has no `<string>` branch.
    pub(crate) fn anchor_name_dashed_ident_value(
        &self,
        evidence: CssAnchorNameDashedIdentEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    pub(crate) const fn execution_completion(&self) -> CssParserExecutionCompletion {
        self.upstream_parser_result.execution_completion()
    }

    pub(crate) fn offset_rotate_observations(&self) -> &[CssOffsetRotateQualificationObservation] {
        &self.offset_rotate_observations
    }

    pub(crate) fn container_name_observations(
        &self,
    ) -> &[CssContainerNameQualificationObservation] {
        &self.container_name_observations
    }

    /// Resolves one qualified `container-name` `<custom-ident>` item's
    /// tokenizer-owned decoded interpreted identity through its run-local
    /// evidence reference, mirroring `anchor_name_dashed_ident_value`
    /// without copying the payload into a second owner. The retained token
    /// at the evidence position is always a direct `Ident`, since this item
    /// grammar has no `<string>` branch.
    pub(crate) fn container_name_custom_ident_value(
        &self,
        evidence: CssContainerNameCustomIdentEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    pub(crate) fn color_scheme_observations(&self) -> &[CssColorSchemeQualificationObservation] {
        &self.color_scheme_observations
    }

    /// Resolves one qualified `color-scheme` `<custom-ident>` item's
    /// tokenizer-owned decoded interpreted identity through its run-local
    /// evidence reference, mirroring `container_name_custom_ident_value`
    /// without copying the payload into a second owner. The retained token
    /// at the evidence position is always a direct `Ident`, since this
    /// item grammar has no `<string>` branch.
    pub(crate) fn color_scheme_custom_ident_value(
        &self,
        evidence: CssColorSchemeCustomIdentEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    pub(crate) fn counter_increment_observations(
        &self,
    ) -> &[CssCounterIncrementQualificationObservation] {
        &self.counter_increment_observations
    }

    /// Resolves one qualified `counter-increment` item's tokenizer-owned
    /// decoded `<counter-name>` identity through its run-local evidence
    /// reference, mirroring `container_name_custom_ident_value` /
    /// `color_scheme_custom_ident_value` without copying the payload into a
    /// second owner. The retained token at the evidence position is always
    /// a direct `Ident`, since this item grammar has no `<string>` branch.
    pub(crate) fn counter_increment_name_value(
        &self,
        evidence: CssCounterIncrementNameEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    /// Resolves one qualified `counter-increment` item's explicit direct
    /// `<integer>` evidence to its exact retained tokenizer token kind,
    /// preserving sign/zero spelling and magnitude without machine-integer
    /// conversion. The retained token at the evidence position is always a
    /// direct `Number` with `CssNumberType::Integer`.
    pub(crate) fn counter_increment_integer_token(
        &self,
        evidence: CssCounterIncrementIntegerEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    pub(crate) fn counter_reset_observations(&self) -> &[CssCounterResetQualificationObservation] {
        &self.counter_reset_observations
    }

    /// Resolves one qualified `counter-reset` item's tokenizer-owned
    /// decoded `<counter-name>` identity through its run-local evidence
    /// reference, mirroring `counter_increment_name_value`. For a
    /// `Direct` name the evidence points at the top-level authored
    /// `Ident`; for a `Reversed` name it points at the exact tokenizer-
    /// owned INNER `Ident` retained inside `reversed(...)`. Both resolve
    /// through the same retained-token lookup since each evidence ref
    /// always targets a direct `Ident` token.
    pub(crate) fn counter_reset_name_value(
        &self,
        evidence: CssCounterResetNameEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    /// Resolves one qualified `counter-reset` item's explicit direct
    /// `<integer>` evidence to its exact retained tokenizer token kind,
    /// preserving sign/zero spelling and magnitude without machine-integer
    /// conversion. The retained token at the evidence position is always a
    /// direct `Number` with `CssNumberType::Integer`.
    pub(crate) fn counter_reset_integer_token(
        &self,
        evidence: CssCounterResetIntegerEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    pub(crate) fn counter_set_observations(&self) -> &[CssCounterSetQualificationObservation] {
        &self.counter_set_observations
    }

    /// Resolves one qualified `counter-set` item's tokenizer-owned decoded
    /// `<counter-name>` identity through its run-local evidence reference,
    /// mirroring `counter_increment_name_value`. The retained token at the
    /// evidence position is always a direct `Ident`, since this item
    /// grammar has no `<string>` branch.
    pub(crate) fn counter_set_name_value(
        &self,
        evidence: CssCounterSetNameEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    /// Resolves one qualified `counter-set` item's explicit direct
    /// `<integer>` evidence to its exact retained tokenizer token kind,
    /// preserving sign/zero spelling and magnitude without machine-integer
    /// conversion. The retained token at the evidence position is always a
    /// direct `Number` with `CssNumberType::Integer`.
    pub(crate) fn counter_set_integer_token(
        &self,
        evidence: CssCounterSetIntegerEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    pub(crate) fn image_resolution_observations(
        &self,
    ) -> &[CssImageResolutionQualificationObservation] {
        &self.image_resolution_observations
    }

    pub(crate) fn will_change_observations(&self) -> &[CssWillChangeQualificationObservation] {
        &self.will_change_observations
    }

    /// Resolves one qualified `will-change` `CustomIdent` item's
    /// tokenizer-owned decoded interpreted identity through its run-local
    /// evidence reference, mirroring `container_name_custom_ident_value` /
    /// `color_scheme_custom_ident_value` without copying the payload into a
    /// second owner. The retained token at the evidence position is always
    /// a direct `Ident`, since this item grammar has no `<string>` branch.
    pub(crate) fn will_change_custom_ident_value(
        &self,
        evidence: CssWillChangeCustomIdentEvidenceRef,
    ) -> Option<&str> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            return None;
        };
        Some(value.as_str())
    }

    pub(crate) fn scale_observations(&self) -> &[CssScaleQualificationObservation] {
        &self.scale_observations
    }

    /// Resolves one qualified `scale` component's run-local evidence
    /// reference to its exact retained tokenizer token kind, preserving
    /// sign/zero spelling, magnitude, and decimal/exponent shape without
    /// machine-number conversion. The retained token at the evidence
    /// position is always a direct `Number` or `Percentage`, matching the
    /// component's `CssScaleComponentKind`.
    pub(crate) fn scale_component_token(
        &self,
        evidence: CssScaleComponentEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    pub(crate) fn rotate_observations(&self) -> &[CssRotateQualificationObservation] {
        &self.rotate_observations
    }

    /// Resolves one qualified `rotate` evidence reference -- an axis
    /// vector component or the angle -- to its exact retained tokenizer
    /// token kind, preserving sign/zero spelling, magnitude, decimal/
    /// exponent shape, and unit spelling without machine-number
    /// conversion or angle normalization. The retained token at the
    /// evidence position is always a direct `Number` or `Dimension`,
    /// matching the evidence's structural position (vector component vs.
    /// angle).
    pub(crate) fn rotate_evidence_token(
        &self,
        evidence: CssRotateEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    pub(crate) fn translate_observations(&self) -> &[CssTranslateQualificationObservation] {
        &self.translate_observations
    }

    /// Resolves one qualified `translate` component's run-local evidence
    /// reference to its exact retained tokenizer token kind, preserving
    /// sign/zero spelling, magnitude, decimal/exponent shape, and unit
    /// spelling without machine-number conversion. The retained token at
    /// the evidence position is always a direct `Number`, `Dimension`, or
    /// `Percentage`, matching the component's `CssTranslateComponentKind`.
    pub(crate) fn translate_component_token(
        &self,
        evidence: CssTranslateComponentEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    pub(crate) fn transform_origin_observations(
        &self,
    ) -> &[CssTransformOriginQualificationObservation] {
        &self.transform_origin_observations
    }

    /// Resolves one qualified `transform-origin` `Length`/`Percentage`
    /// component's run-local evidence reference to its exact retained
    /// tokenizer token kind, preserving sign/zero spelling, magnitude,
    /// decimal/exponent shape, and unit spelling without machine-number
    /// conversion. A `Keyword` component carries no evidence reference --
    /// its authored identity is already fully captured by the decoded
    /// keyword itself.
    pub(crate) fn transform_origin_component_token(
        &self,
        evidence: CssTransformOriginComponentEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    pub(crate) fn transform_box_observations(&self) -> &[CssTransformBoxQualificationObservation] {
        &self.transform_box_observations
    }

    pub(crate) fn transform_style_observations(
        &self,
    ) -> &[CssTransformStyleQualificationObservation] {
        &self.transform_style_observations
    }

    pub(crate) fn text_indent_observations(&self) -> &[CssTextIndentQualificationObservation] {
        &self.text_indent_observations
    }

    pub(crate) fn letter_spacing_observations(
        &self,
    ) -> &[CssLetterSpacingQualificationObservation] {
        &self.letter_spacing_observations
    }

    pub(crate) fn text_underline_position_observations(
        &self,
    ) -> &[CssTextUnderlinePositionQualificationObservation] {
        &self.text_underline_position_observations
    }

    pub(crate) fn list_style_position_observations(
        &self,
    ) -> &[CssListStylePositionQualificationObservation] {
        &self.list_style_position_observations
    }

    pub(crate) fn stroke_linecap_observations(
        &self,
    ) -> &[CssStrokeLinecapQualificationObservation] {
        &self.stroke_linecap_observations
    }

    pub(crate) fn stroke_opacity_observations(
        &self,
    ) -> &[CssStrokeOpacityQualificationObservation] {
        &self.stroke_opacity_observations
    }

    pub(crate) fn stop_opacity_observations(&self) -> &[CssStopOpacityQualificationObservation] {
        &self.stop_opacity_observations
    }

    pub(crate) fn flood_opacity_observations(&self) -> &[CssFloodOpacityQualificationObservation] {
        &self.flood_opacity_observations
    }

    pub(crate) fn paint_order_observations(&self) -> &[CssPaintOrderQualificationObservation] {
        &self.paint_order_observations
    }

    pub(crate) fn caret_animation_observations(
        &self,
    ) -> &[CssCaretAnimationQualificationObservation] {
        &self.caret_animation_observations
    }

    pub(crate) fn caret_shape_observations(&self) -> &[CssCaretShapeQualificationObservation] {
        &self.caret_shape_observations
    }

    pub(crate) fn animation_fill_mode_observations(
        &self,
    ) -> &[CssAnimationFillModeQualificationObservation] {
        &self.animation_fill_mode_observations
    }

    pub(crate) fn transform_observations(&self) -> &[CssTransformQualificationObservation] {
        &self.transform_observations
    }

    /// Resolves one qualified `transform` `matrix()` argument's run-local
    /// evidence reference to its exact retained tokenizer token kind,
    /// preserving authored sign spelling, integer/fraction digits, and
    /// exponent spelling without any machine-float conversion. The
    /// retained token at the evidence position is always a direct
    /// `Number`, so `1`, `1.0`, `1e0`, `+1`, and `-0` remain distinct
    /// authored evidence here.
    pub(crate) fn transform_matrix_argument_token(
        &self,
        evidence: CssTransformMatrixArgumentEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    /// Resolves one qualified `transform` `scale()` argument's run-local
    /// evidence reference to its exact retained tokenizer token kind,
    /// preserving authored sign spelling, integer/fraction digits, and
    /// exponent spelling without any machine-number conversion. The
    /// retained token at the evidence position is always a direct
    /// `Number` or `Percentage`, matching the argument's
    /// `CssTransformScaleArgumentKind`.
    pub(crate) fn transform_scale_argument_token(
        &self,
        evidence: CssTransformScaleArgumentEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    /// Resolves one qualified `transform` `translate3d()` argument's
    /// run-local evidence reference to its exact retained tokenizer token
    /// kind, preserving authored sign spelling, integer/fraction digits,
    /// exponent spelling, and unit identity without any machine-number
    /// conversion or percentage/unit normalization. The retained token at
    /// an X/Y evidence position is a direct exact-zero `Number`, a
    /// recognized-length `Dimension`, or a `Percentage`; the retained token
    /// at the Z evidence position is always a direct exact-zero `Number` or
    /// a recognized-length `Dimension`, matching
    /// `CssTransformTranslate3dFunction::z`'s `Percentage`-free type.
    pub(crate) fn transform_translate3d_argument_token(
        &self,
        evidence: CssTransformTranslate3dArgumentEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
    }

    /// Resolves one qualified `text-indent` `Length`/`Percentage`
    /// component's run-local evidence reference to its exact retained
    /// tokenizer token kind, preserving sign/zero spelling, magnitude,
    /// decimal/exponent shape, and unit spelling without machine-number
    /// conversion. A `Hanging`/`EachLine` component carries no evidence
    /// reference -- its authored identity is already fully captured by the
    /// decoded keyword variant itself.
    pub(crate) fn text_indent_component_token(
        &self,
        evidence: CssTextIndentComponentEvidenceRef,
    ) -> Option<&CssTokenKind> {
        let item = self
            .upstream_parser_result
            .upstream_tokenizer_result()
            .lexical_items()
            .get(evidence.lexical_item_index())?;
        let CssLexicalItem::SemanticToken(token) = item else {
            return None;
        };
        Some(token.kind())
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
        fill_opacity_observations,
        shape_image_threshold_observations,
        shape_margin_observations,
        line_height_observations,
        line_break_observations,
        print_color_adjust_observations,
        overflow_wrap_observations,
        unicode_bidi_observations,
        mask_type_observations,
        color_interpolation_filters_observations,
        shape_rendering_observations,
        text_rendering_observations,
        text_anchor_observations,
        forced_color_adjust_observations,
        text_align_last_observations,
        math_style_observations,
        math_shift_observations,
        ruby_align_observations,
        ruby_merge_observations,
        ruby_position_observations,
        ruby_overhang_observations,
        clip_rule_observations,
        fill_rule_observations,
        column_fill_observations,
        text_decoration_skip_ink_observations,
        overscroll_behavior_observations,
        contain_observations,
        font_variant_ligatures_observations,
        font_variant_numeric_observations,
        text_decoration_line_observations,
        text_transform_observations,
        text_emphasis_position_observations,
        overscroll_behavior_x_observations,
        overscroll_behavior_y_observations,
        overscroll_behavior_inline_observations,
        overscroll_behavior_block_observations,
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
        font_synthesis_small_caps_observations,
        font_synthesis_position_observations,
        font_variant_emoji_observations,
        font_variant_caps_observations,
        font_variant_position_observations,
        font_weight_observations,
        page_observations,
        border_spacing_observations,
        z_index_observations,
        aspect_ratio_observations,
        animation_play_state_observations,
        animation_iteration_count_observations,
        animation_delay_observations,
        transition_duration_observations,
        transition_property_observations,
        hyphenate_character_observations,
        animation_name_observations,
        anchor_name_observations,
        offset_rotate_observations,
        container_name_observations,
        color_scheme_observations,
        counter_increment_observations,
        counter_reset_observations,
        counter_set_observations,
        image_resolution_observations,
        will_change_observations,
        scale_observations,
        rotate_observations,
        translate_observations,
        transform_origin_observations,
        transform_box_observations,
        transform_style_observations,
        text_indent_observations,
        letter_spacing_observations,
        text_underline_position_observations,
        list_style_position_observations,
        stroke_linecap_observations,
        stroke_opacity_observations,
        stop_opacity_observations,
        flood_opacity_observations,
        paint_order_observations,
        caret_animation_observations,
        caret_shape_observations,
        animation_fill_mode_observations,
        transform_observations,
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
        let mut fill_opacity_observations = Vec::new();
        let mut shape_image_threshold_observations = Vec::new();
        let mut shape_margin_observations = Vec::new();
        let mut line_height_observations = Vec::new();
        let mut line_break_observations = Vec::new();
        let mut print_color_adjust_observations = Vec::new();
        let mut overflow_wrap_observations = Vec::new();
        let mut unicode_bidi_observations = Vec::new();
        let mut mask_type_observations = Vec::new();
        let mut color_interpolation_filters_observations = Vec::new();
        let mut shape_rendering_observations = Vec::new();
        let mut text_rendering_observations = Vec::new();
        let mut text_anchor_observations = Vec::new();
        let mut forced_color_adjust_observations = Vec::new();
        let mut text_align_last_observations = Vec::new();
        let mut math_style_observations = Vec::new();
        let mut math_shift_observations = Vec::new();
        let mut ruby_align_observations = Vec::new();
        let mut ruby_merge_observations = Vec::new();
        let mut ruby_position_observations = Vec::new();
        let mut ruby_overhang_observations = Vec::new();
        let mut clip_rule_observations = Vec::new();
        let mut fill_rule_observations = Vec::new();
        let mut column_fill_observations = Vec::new();
        let mut text_decoration_skip_ink_observations = Vec::new();
        let mut overscroll_behavior_observations = Vec::new();
        let mut contain_observations = Vec::new();
        let mut font_variant_ligatures_observations = Vec::new();
        let mut font_variant_numeric_observations = Vec::new();
        let mut text_decoration_line_observations = Vec::new();
        let mut text_transform_observations = Vec::new();
        let mut text_emphasis_position_observations = Vec::new();
        let mut overscroll_behavior_x_observations = Vec::new();
        let mut overscroll_behavior_y_observations = Vec::new();
        let mut overscroll_behavior_inline_observations = Vec::new();
        let mut overscroll_behavior_block_observations = Vec::new();
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
        let mut font_synthesis_small_caps_observations = Vec::new();
        let mut font_synthesis_position_observations = Vec::new();
        let mut font_variant_emoji_observations = Vec::new();
        let mut font_variant_caps_observations = Vec::new();
        let mut font_variant_position_observations = Vec::new();
        let mut font_weight_observations = Vec::new();
        let mut page_observations = Vec::new();
        let mut border_spacing_observations = Vec::new();
        let mut z_index_observations = Vec::new();
        let mut aspect_ratio_observations = Vec::new();
        let mut animation_play_state_observations = Vec::new();
        let mut animation_iteration_count_observations = Vec::new();
        let mut animation_delay_observations = Vec::new();
        let mut transition_duration_observations = Vec::new();
        let mut transition_property_observations = Vec::new();
        let mut hyphenate_character_observations = Vec::new();
        let mut animation_name_observations = Vec::new();
        let mut anchor_name_observations = Vec::new();
        let mut offset_rotate_observations = Vec::new();
        let mut container_name_observations = Vec::new();
        let mut color_scheme_observations = Vec::new();
        let mut counter_increment_observations = Vec::new();
        let mut counter_reset_observations = Vec::new();
        let mut counter_set_observations = Vec::new();
        let mut image_resolution_observations = Vec::new();
        let mut will_change_observations = Vec::new();
        let mut scale_observations = Vec::new();
        let mut rotate_observations = Vec::new();
        let mut translate_observations = Vec::new();
        let mut transform_origin_observations = Vec::new();
        let mut transform_box_observations = Vec::new();
        let mut transform_style_observations = Vec::new();
        let mut text_indent_observations = Vec::new();
        let mut letter_spacing_observations = Vec::new();
        let mut text_underline_position_observations = Vec::new();
        let mut list_style_position_observations = Vec::new();
        let mut stroke_linecap_observations = Vec::new();
        let mut stroke_opacity_observations = Vec::new();
        let mut stop_opacity_observations = Vec::new();
        let mut flood_opacity_observations = Vec::new();
        let mut paint_order_observations = Vec::new();
        let mut caret_animation_observations = Vec::new();
        let mut caret_shape_observations = Vec::new();
        let mut animation_fill_mode_observations = Vec::new();
        let mut transform_observations = Vec::new();

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

            if property_name.eq_ignore_ascii_case("fill-opacity") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                fill_opacity_observations.push(CssFillOpacityQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_fill_opacity_value(value_items),
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

            if property_name.eq_ignore_ascii_case("line-break") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                line_break_observations.push(CssLineBreakQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_line_break_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("print-color-adjust") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                print_color_adjust_observations.push(CssPrintColorAdjustQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_print_color_adjust_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("overflow-wrap") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overflow_wrap_observations.push(CssOverflowWrapQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_overflow_wrap_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("unicode-bidi") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                unicode_bidi_observations.push(CssUnicodeBidiQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_unicode_bidi_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("mask-type") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                mask_type_observations.push(CssMaskTypeQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_mask_type_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("color-interpolation-filters") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                color_interpolation_filters_observations.push(
                    CssColorInterpolationFiltersQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_color_interpolation_filters_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("shape-rendering") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                shape_rendering_observations.push(CssShapeRenderingQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_shape_rendering_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-rendering") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_rendering_observations.push(CssTextRenderingQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_text_rendering_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-anchor") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_anchor_observations.push(CssTextAnchorQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_text_anchor_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("forced-color-adjust") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                forced_color_adjust_observations.push(
                    CssForcedColorAdjustQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_forced_color_adjust_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-align-last") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_align_last_observations.push(CssTextAlignLastQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_text_align_last_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("math-style") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                math_style_observations.push(CssMathStyleQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_math_style_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("math-shift") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                math_shift_observations.push(CssMathShiftQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_math_shift_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("ruby-align") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                ruby_align_observations.push(CssRubyAlignQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_ruby_align_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("ruby-merge") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                ruby_merge_observations.push(CssRubyMergeQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_ruby_merge_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("ruby-position") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                ruby_position_observations.push(CssRubyPositionQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_ruby_position_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("ruby-overhang") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                ruby_overhang_observations.push(CssRubyOverhangQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_ruby_overhang_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("clip-rule") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                clip_rule_observations.push(CssClipRuleQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_clip_rule_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("fill-rule") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                fill_rule_observations.push(CssFillRuleQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_fill_rule_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("column-fill") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                column_fill_observations.push(CssColumnFillQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_column_fill_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-decoration-skip-ink") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_decoration_skip_ink_observations.push(
                    CssTextDecorationSkipInkQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_text_decoration_skip_ink_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("contain") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                contain_observations.push(CssContainQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_contain_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("overscroll-behavior") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overscroll_behavior_observations.push(
                    CssOverscrollBehaviorQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_overscroll_behavior_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-variant-ligatures") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_variant_ligatures_observations.push(
                    CssFontVariantLigaturesQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_font_variant_ligatures_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-variant-numeric") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_variant_numeric_observations.push(
                    CssFontVariantNumericQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_font_variant_numeric_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-decoration-line") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_decoration_line_observations.push(
                    CssTextDecorationLineQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_text_decoration_line_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-transform") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_transform_observations.push(CssTextTransformQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_text_transform_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-emphasis-position") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_emphasis_position_observations.push(
                    CssTextEmphasisPositionQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_text_emphasis_position_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("overscroll-behavior-x") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overscroll_behavior_x_observations.push(
                    CssOverscrollBehaviorXQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_overscroll_behavior_x_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("overscroll-behavior-y") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overscroll_behavior_y_observations.push(
                    CssOverscrollBehaviorYQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_overscroll_behavior_y_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("overscroll-behavior-inline") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overscroll_behavior_inline_observations.push(
                    CssOverscrollBehaviorInlineQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_overscroll_behavior_inline_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("overscroll-behavior-block") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overscroll_behavior_block_observations.push(
                    CssOverscrollBehaviorBlockQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_overscroll_behavior_block_value(value_items),
                    },
                );
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

            if property_name.eq_ignore_ascii_case("page") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let (outcome, custom_ident_evidence) =
                    qualify_page_value(value_items, lexical_item_start);
                page_observations.push(CssPageQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                    custom_ident_evidence,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("border-spacing") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                border_spacing_observations.push(CssBorderSpacingQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_border_spacing_value(value_items),
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

            if property_name.eq_ignore_ascii_case("aspect-ratio") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                aspect_ratio_observations.push(CssAspectRatioQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_aspect_ratio_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("animation-play-state") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                animation_play_state_observations.push(
                    CssAnimationPlayStateQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_animation_play_state_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("animation-iteration-count") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                animation_iteration_count_observations.push(
                    CssAnimationIterationCountQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_animation_iteration_count_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("animation-delay") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                animation_delay_observations.push(CssAnimationDelayQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_animation_delay_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("transition-duration") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                transition_duration_observations.push(
                    CssTransitionDurationQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_transition_duration_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("transition-property") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let (outcome, custom_ident_evidence) =
                    qualify_transition_property_value(value_items, lexical_item_start);
                transition_property_observations.push(
                    CssTransitionPropertyQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome,
                        custom_ident_evidence,
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("hyphenate-character") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let (outcome, string_evidence) =
                    qualify_hyphenate_character_value(value_items, lexical_item_start);
                hyphenate_character_observations.push(
                    CssHyphenateCharacterQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome,
                        string_evidence,
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("animation-name") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let (outcome, keyframes_name_evidence) =
                    qualify_animation_name_value(value_items, lexical_item_start);
                animation_name_observations.push(CssAnimationNameQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                    keyframes_name_evidence,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("anchor-name") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_anchor_name_value(value_items, lexical_item_start);
                anchor_name_observations.push(CssAnchorNameQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("offset-rotate") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                offset_rotate_observations.push(CssOffsetRotateQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_offset_rotate_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("container-name") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_container_name_value(value_items, lexical_item_start);
                container_name_observations.push(CssContainerNameQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("color-scheme") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_color_scheme_value(value_items, lexical_item_start);
                color_scheme_observations.push(CssColorSchemeQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("counter-increment") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_counter_increment_value(value_items, lexical_item_start);
                counter_increment_observations.push(CssCounterIncrementQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("counter-reset") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_counter_reset_value(value_items, lexical_item_start);
                counter_reset_observations.push(CssCounterResetQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("counter-set") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_counter_set_value(value_items, lexical_item_start);
                counter_set_observations.push(CssCounterSetQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("image-resolution") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                image_resolution_observations.push(CssImageResolutionQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_image_resolution_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("will-change") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_will_change_value(value_items, lexical_item_start);
                will_change_observations.push(CssWillChangeQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
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

            if property_name.eq_ignore_ascii_case("font-synthesis-small-caps") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_synthesis_small_caps_observations.push(
                    CssFontSynthesisSmallCapsQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_font_synthesis_small_caps_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-synthesis-position") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_synthesis_position_observations.push(
                    CssFontSynthesisPositionQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_font_synthesis_position_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-variant-emoji") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_variant_emoji_observations.push(CssFontVariantEmojiQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_font_variant_emoji_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-variant-caps") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_variant_caps_observations.push(CssFontVariantCapsQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_font_variant_caps_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("font-weight") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                font_weight_observations.push(CssFontWeightQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_font_weight_value(value_items),
                });
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
                continue;
            }

            if property_name.eq_ignore_ascii_case("scale") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_scale_value(value_items, lexical_item_start);
                scale_observations.push(CssScaleQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("rotate") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_rotate_value(value_items, lexical_item_start);
                rotate_observations.push(CssRotateQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("translate") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_translate_value(value_items, lexical_item_start);
                translate_observations.push(CssTranslateQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("transform-origin") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_transform_origin_value(value_items, lexical_item_start);
                transform_origin_observations.push(CssTransformOriginQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("transform-box") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                transform_box_observations.push(CssTransformBoxQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_transform_box_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("transform-style") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                transform_style_observations.push(CssTransformStyleQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_transform_style_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-indent") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                let outcome = qualify_text_indent_value(value_items, lexical_item_start);
                text_indent_observations.push(CssTextIndentQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome,
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("letter-spacing") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                letter_spacing_observations.push(CssLetterSpacingQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_letter_spacing_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("text-underline-position") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                text_underline_position_observations.push(
                    CssTextUnderlinePositionQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_text_underline_position_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("list-style-position") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                list_style_position_observations.push(
                    CssListStylePositionQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_list_style_position_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("stroke-linecap") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                stroke_linecap_observations.push(CssStrokeLinecapQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_stroke_linecap_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("stroke-opacity") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                stroke_opacity_observations.push(CssStrokeOpacityQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_stroke_opacity_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("stop-opacity") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                stop_opacity_observations.push(CssStopOpacityQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_stop_opacity_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("flood-opacity") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                flood_opacity_observations.push(CssFloodOpacityQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_flood_opacity_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("paint-order") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                paint_order_observations.push(CssPaintOrderQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_paint_order_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("caret-animation") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                caret_animation_observations.push(CssCaretAnimationQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_caret_animation_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("caret-shape") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                caret_shape_observations.push(CssCaretShapeQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_caret_shape_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("animation-fill-mode") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                animation_fill_mode_observations.push(
                    CssAnimationFillModeQualificationObservation {
                        occurrence_index,
                        placement: occurrence.placement(),
                        outcome: qualify_animation_fill_mode_value(value_items),
                    },
                );
                continue;
            }

            if property_name.eq_ignore_ascii_case("transform") {
                let value_range = cursor.window_for(occurrence.value())?;
                let lexical_item_start = value_range.start;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                transform_observations.push(CssTransformQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_transform_value(value_items, lexical_item_start),
                });
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
            fill_opacity_observations,
            shape_image_threshold_observations,
            shape_margin_observations,
            line_height_observations,
            line_break_observations,
            print_color_adjust_observations,
            overflow_wrap_observations,
            unicode_bidi_observations,
            mask_type_observations,
            color_interpolation_filters_observations,
            shape_rendering_observations,
            text_rendering_observations,
            text_anchor_observations,
            forced_color_adjust_observations,
            text_align_last_observations,
            math_style_observations,
            math_shift_observations,
            ruby_align_observations,
            ruby_merge_observations,
            ruby_position_observations,
            ruby_overhang_observations,
            clip_rule_observations,
            fill_rule_observations,
            column_fill_observations,
            text_decoration_skip_ink_observations,
            overscroll_behavior_observations,
            contain_observations,
            font_variant_ligatures_observations,
            font_variant_numeric_observations,
            text_decoration_line_observations,
            text_transform_observations,
            text_emphasis_position_observations,
            overscroll_behavior_x_observations,
            overscroll_behavior_y_observations,
            overscroll_behavior_inline_observations,
            overscroll_behavior_block_observations,
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
            font_synthesis_small_caps_observations,
            font_synthesis_position_observations,
            font_variant_emoji_observations,
            font_variant_caps_observations,
            font_variant_position_observations,
            font_weight_observations,
            page_observations,
            border_spacing_observations,
            z_index_observations,
            aspect_ratio_observations,
            animation_play_state_observations,
            animation_iteration_count_observations,
            animation_delay_observations,
            transition_duration_observations,
            transition_property_observations,
            hyphenate_character_observations,
            animation_name_observations,
            anchor_name_observations,
            offset_rotate_observations,
            container_name_observations,
            color_scheme_observations,
            counter_increment_observations,
            counter_reset_observations,
            counter_set_observations,
            image_resolution_observations,
            will_change_observations,
            scale_observations,
            rotate_observations,
            translate_observations,
            transform_origin_observations,
            transform_box_observations,
            transform_style_observations,
            text_indent_observations,
            letter_spacing_observations,
            text_underline_position_observations,
            list_style_position_observations,
            stroke_linecap_observations,
            stroke_opacity_observations,
            stop_opacity_observations,
            flood_opacity_observations,
            paint_order_observations,
            caret_animation_observations,
            caret_shape_observations,
            animation_fill_mode_observations,
            transform_observations,
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
        fill_opacity_observations,
        shape_image_threshold_observations,
        shape_margin_observations,
        line_height_observations,
        line_break_observations,
        print_color_adjust_observations,
        overflow_wrap_observations,
        unicode_bidi_observations,
        mask_type_observations,
        color_interpolation_filters_observations,
        shape_rendering_observations,
        text_rendering_observations,
        text_anchor_observations,
        forced_color_adjust_observations,
        text_align_last_observations,
        math_style_observations,
        math_shift_observations,
        ruby_align_observations,
        ruby_merge_observations,
        ruby_position_observations,
        ruby_overhang_observations,
        clip_rule_observations,
        fill_rule_observations,
        column_fill_observations,
        text_decoration_skip_ink_observations,
        overscroll_behavior_observations,
        contain_observations,
        font_variant_ligatures_observations,
        font_variant_numeric_observations,
        text_decoration_line_observations,
        text_transform_observations,
        text_emphasis_position_observations,
        overscroll_behavior_x_observations,
        overscroll_behavior_y_observations,
        overscroll_behavior_inline_observations,
        overscroll_behavior_block_observations,
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
        font_synthesis_small_caps_observations,
        font_synthesis_position_observations,
        font_variant_emoji_observations,
        font_variant_caps_observations,
        font_variant_position_observations,
        font_weight_observations,
        page_observations,
        border_spacing_observations,
        z_index_observations,
        aspect_ratio_observations,
        animation_play_state_observations,
        animation_iteration_count_observations,
        animation_delay_observations,
        transition_duration_observations,
        transition_property_observations,
        hyphenate_character_observations,
        animation_name_observations,
        anchor_name_observations,
        offset_rotate_observations,
        container_name_observations,
        color_scheme_observations,
        counter_increment_observations,
        counter_reset_observations,
        counter_set_observations,
        image_resolution_observations,
        will_change_observations,
        scale_observations,
        rotate_observations,
        translate_observations,
        transform_origin_observations,
        transform_box_observations,
        transform_style_observations,
        text_indent_observations,
        letter_spacing_observations,
        text_underline_position_observations,
        list_style_position_observations,
        stroke_linecap_observations,
        stroke_opacity_observations,
        stop_opacity_observations,
        flood_opacity_observations,
        paint_order_observations,
        caret_animation_observations,
        caret_shape_observations,
        animation_fill_mode_observations,
        transform_observations,
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

fn qualify_list_style_position_value(
    items: &[CssLexicalItem],
) -> CssListStylePositionQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssListStylePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssListStylePositionUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssListStylePositionQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("inside") =>
        {
            CssListStylePositionQualificationOutcome::Qualified(CssListStylePositionValue::Inside)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("outside") =>
        {
            CssListStylePositionQualificationOutcome::Qualified(CssListStylePositionValue::Outside)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssListStylePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssListStylePositionUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssListStylePositionQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_stroke_linecap_value(items: &[CssLexicalItem]) -> CssStrokeLinecapQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssStrokeLinecapQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeLinecapUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssStrokeLinecapQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("butt") =>
        {
            CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Butt)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("round") =>
        {
            CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Round)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("square") =>
        {
            CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Square)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssStrokeLinecapQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeLinecapUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssStrokeLinecapQualificationOutcome::InvalidForSelectedValueGrammar
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

fn qualify_font_synthesis_small_caps_value(
    items: &[CssLexicalItem],
) -> CssFontSynthesisSmallCapsQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssFontSynthesisSmallCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisSmallCapsUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssFontSynthesisSmallCapsQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssFontSynthesisSmallCapsQualificationOutcome::Qualified(
                CssFontSynthesisSmallCapsValue::Auto,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssFontSynthesisSmallCapsQualificationOutcome::Qualified(
                CssFontSynthesisSmallCapsValue::None,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssFontSynthesisSmallCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisSmallCapsUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssFontSynthesisSmallCapsQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_font_synthesis_position_value(
    items: &[CssLexicalItem],
) -> CssFontSynthesisPositionQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssFontSynthesisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisPositionUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssFontSynthesisPositionQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssFontSynthesisPositionQualificationOutcome::Qualified(
                CssFontSynthesisPositionValue::Auto,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssFontSynthesisPositionQualificationOutcome::Qualified(
                CssFontSynthesisPositionValue::None,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssFontSynthesisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisPositionUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssFontSynthesisPositionQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_font_variant_emoji_value(
    items: &[CssLexicalItem],
) -> CssFontVariantEmojiQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssFontVariantEmojiQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantEmojiUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssFontVariantEmojiQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssFontVariantEmojiQualificationOutcome::Qualified(CssFontVariantEmojiValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("text") =>
        {
            CssFontVariantEmojiQualificationOutcome::Qualified(CssFontVariantEmojiValue::Text)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("emoji") =>
        {
            CssFontVariantEmojiQualificationOutcome::Qualified(CssFontVariantEmojiValue::Emoji)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("unicode") =>
        {
            CssFontVariantEmojiQualificationOutcome::Qualified(CssFontVariantEmojiValue::Unicode)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssFontVariantEmojiQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantEmojiUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssFontVariantEmojiQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_font_variant_caps_value(
    items: &[CssLexicalItem],
) -> CssFontVariantCapsQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssFontVariantCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantCapsUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssFontVariantCapsQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("small-caps") =>
        {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::SmallCaps)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("all-small-caps") =>
        {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::AllSmallCaps)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("petite-caps") =>
        {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::PetiteCaps)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("all-petite-caps") =>
        {
            CssFontVariantCapsQualificationOutcome::Qualified(
                CssFontVariantCapsValue::AllPetiteCaps,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("unicase") =>
        {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::Unicase)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("titling-caps") =>
        {
            CssFontVariantCapsQualificationOutcome::Qualified(CssFontVariantCapsValue::TitlingCaps)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssFontVariantCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantCapsUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssFontVariantCapsQualificationOutcome::InvalidForSelectedValueGrammar
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

fn qualify_font_weight_value(items: &[CssLexicalItem]) -> CssFontWeightQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssFontWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFontWeightUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssFontWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFontWeightUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssFontWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFontWeightUnsupportedReason::FunctionValue,
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
        return CssFontWeightQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssFontWeightQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("normal") => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::Normal)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("bold") => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::Bold)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("bolder") => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::Bolder)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("lighter") => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::Lighter)
        }
        CssTokenKind::Number { value, .. } if is_direct_font_weight_number(value) => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::DirectNumberLiteral)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssFontWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontWeightUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssFontWeightQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

/// Decides the closed direct-literal range `[1,1000]` from retained decimal
/// structure only. The authored numeric evidence is never rewritten or
/// converted to a host floating-point or bounded exponent value.
fn is_direct_font_weight_number(value: &CssNumericValue) -> bool {
    if matches!(value.sign(), Some(CssNumberSign::Minus)) {
        return false;
    }

    let decimal = value.decimal();
    let integer_digits = decimal.integer_digits().as_bytes();
    let fraction_digits = decimal.fraction_digits().as_bytes();
    let total_digits = integer_digits.len() + fraction_digits.len();

    let first_nonzero = integer_digits
        .iter()
        .chain(fraction_digits)
        .position(|digit| *digit != b'0');
    let Some(first_nonzero) = first_nonzero else {
        return false;
    };

    let trailing_zeros = fraction_digits
        .iter()
        .rev()
        .chain(integer_digits.iter().rev())
        .take_while(|digit| **digit == b'0')
        .count();
    let core_len = total_digits - first_nonzero - trailing_zeros;

    let lower = compare_font_weight_decimal_order(
        decimal.exponent(),
        core_len,
        trailing_zeros,
        fraction_digits.len(),
        1,
    );
    if lower == Ordering::Less {
        return false;
    }

    match compare_font_weight_decimal_order(
        decimal.exponent(),
        core_len,
        trailing_zeros,
        fraction_digits.len(),
        4,
    ) {
        Ordering::Less => true,
        Ordering::Greater => false,
        Ordering::Equal => {
            core_len == 1
                && integer_digits
                    .iter()
                    .chain(fraction_digits)
                    .nth(first_nonzero)
                    == Some(&b'1')
        }
    }
}

fn compare_font_weight_decimal_order(
    exponent: Option<&CssDecimalExponent>,
    core_len: usize,
    trailing_zeros: usize,
    fraction_len: usize,
    target_order: u128,
) -> Ordering {
    let left = core_len as u128 + trailing_zeros as u128;
    let right = fraction_len as u128 + target_order;

    let Some(exponent) = exponent else {
        return left.cmp(&right);
    };
    let exponent_digits = exponent.digits().trim_start_matches('0');
    if exponent_digits.is_empty() {
        return left.cmp(&right);
    }

    match exponent.sign() {
        Some(CssExponentSign::Minus) => {
            if left <= right {
                return Ordering::Less;
            }
            compare_decimal_digits_to_u128(exponent_digits, left - right).reverse()
        }
        None | Some(CssExponentSign::Plus) => {
            if left >= right {
                return Ordering::Greater;
            }
            compare_decimal_digits_to_u128(exponent_digits, right - left)
        }
    }
}

/// Compares a normalized non-empty ASCII decimal digit string with one small
/// non-negative metadata count without parsing the digit string as an integer.
fn compare_decimal_digits_to_u128(digits: &str, value: u128) -> Ordering {
    let rendered = value.to_string();
    match digits.len().cmp(&rendered.len()) {
        Ordering::Equal => digits.as_bytes().cmp(rendered.as_bytes()),
        ordering => ordering,
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

fn qualify_fill_opacity_value(items: &[CssLexicalItem]) -> CssFillOpacityQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssFillOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFillOpacityUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssFillOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFillOpacityUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssFillOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFillOpacityUnsupportedReason::FunctionValue,
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
        return CssFillOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssFillOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { .. } => {
            CssFillOpacityQualificationOutcome::Qualified(CssFillOpacityValue::DirectNumberLiteral)
        }
        CssTokenKind::Percentage { .. } => CssFillOpacityQualificationOutcome::Qualified(
            CssFillOpacityValue::DirectPercentageLiteral,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssFillOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssFillOpacityQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_stroke_opacity_value(items: &[CssLexicalItem]) -> CssStrokeOpacityQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssStrokeOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssStrokeOpacityUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssStrokeOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssStrokeOpacityUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssStrokeOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssStrokeOpacityUnsupportedReason::FunctionValue,
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
        return CssStrokeOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssStrokeOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { .. } => CssStrokeOpacityQualificationOutcome::Qualified(
            CssStrokeOpacityValue::DirectNumberLiteral,
        ),
        CssTokenKind::Percentage { .. } => CssStrokeOpacityQualificationOutcome::Qualified(
            CssStrokeOpacityValue::DirectPercentageLiteral,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssStrokeOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssStrokeOpacityQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_stop_opacity_value(items: &[CssLexicalItem]) -> CssStopOpacityQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssStopOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssStopOpacityUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssStopOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssStopOpacityUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssStopOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssStopOpacityUnsupportedReason::FunctionValue,
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
        return CssStopOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssStopOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { .. } => {
            CssStopOpacityQualificationOutcome::Qualified(CssStopOpacityValue::DirectNumberLiteral)
        }
        CssTokenKind::Percentage { .. } => CssStopOpacityQualificationOutcome::Qualified(
            CssStopOpacityValue::DirectPercentageLiteral,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssStopOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStopOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssStopOpacityQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_flood_opacity_value(items: &[CssLexicalItem]) -> CssFloodOpacityQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssFloodOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFloodOpacityUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssFloodOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFloodOpacityUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssFloodOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFloodOpacityUnsupportedReason::FunctionValue,
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
        return CssFloodOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssFloodOpacityQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Number { .. } => CssFloodOpacityQualificationOutcome::Qualified(
            CssFloodOpacityValue::DirectNumberLiteral,
        ),
        CssTokenKind::Percentage { .. } => CssFloodOpacityQualificationOutcome::Qualified(
            CssFloodOpacityValue::DirectPercentageLiteral,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssFloodOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFloodOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssFloodOpacityQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn qualify_paint_order_value(items: &[CssLexicalItem]) -> CssPaintOrderQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssPaintOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssPaintOrderUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssPaintOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssPaintOrderUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
    {
        if is_css_wide_keyword(identifier) {
            return CssPaintOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPaintOrderUnsupportedReason::CssWideKeyword,
            );
        }
        if identifier.eq_ignore_ascii_case("normal") {
            return CssPaintOrderQualificationOutcome::Qualified(CssPaintOrderValue::Normal);
        }
    }

    if tokens.is_empty() {
        return CssPaintOrderQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut authored = [CssPaintOrderComponent::Fill; 3];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssPaintOrderQualificationOutcome::InvalidForSelectedValueGrammar;
        };

        let Some((component, slot)) = paint_order_component(identifier) else {
            return CssPaintOrderQualificationOutcome::InvalidForSelectedValueGrammar;
        };

        if occupied_slots & slot != 0 {
            return CssPaintOrderQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    CssPaintOrderQualificationOutcome::Qualified(CssPaintOrderValue::Components(
        CssPaintOrderComponents { authored, count },
    ))
}

fn paint_order_component(identifier: &str) -> Option<(CssPaintOrderComponent, u8)> {
    if identifier.eq_ignore_ascii_case("fill") {
        return Some((CssPaintOrderComponent::Fill, 0b001));
    }
    if identifier.eq_ignore_ascii_case("stroke") {
        return Some((CssPaintOrderComponent::Stroke, 0b010));
    }
    if identifier.eq_ignore_ascii_case("markers") {
        return Some((CssPaintOrderComponent::Markers, 0b100));
    }
    None
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

fn qualify_line_break_value(items: &[CssLexicalItem]) -> CssLineBreakQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssLineBreakQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineBreakUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssLineBreakQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("loose") =>
        {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Loose)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("strict") =>
        {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Strict)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("anywhere") =>
        {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Anywhere)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssLineBreakQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineBreakUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssLineBreakQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_print_color_adjust_value(
    items: &[CssLexicalItem],
) -> CssPrintColorAdjustQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssPrintColorAdjustQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPrintColorAdjustUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssPrintColorAdjustQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("economy") =>
        {
            CssPrintColorAdjustQualificationOutcome::Qualified(CssPrintColorAdjustValue::Economy)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("exact") =>
        {
            CssPrintColorAdjustQualificationOutcome::Qualified(CssPrintColorAdjustValue::Exact)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssPrintColorAdjustQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPrintColorAdjustUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssPrintColorAdjustQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_overflow_wrap_value(items: &[CssLexicalItem]) -> CssOverflowWrapQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssOverflowWrapQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverflowWrapUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssOverflowWrapQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssOverflowWrapQualificationOutcome::Qualified(CssOverflowWrapValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("break-word") =>
        {
            CssOverflowWrapQualificationOutcome::Qualified(CssOverflowWrapValue::BreakWord)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("anywhere") =>
        {
            CssOverflowWrapQualificationOutcome::Qualified(CssOverflowWrapValue::Anywhere)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssOverflowWrapQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverflowWrapUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssOverflowWrapQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_unicode_bidi_value(items: &[CssLexicalItem]) -> CssUnicodeBidiQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssUnicodeBidiQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssUnicodeBidiUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssUnicodeBidiQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("embed") =>
        {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::Embed)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("isolate") =>
        {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::Isolate)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("bidi-override") =>
        {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::BidiOverride)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("isolate-override") =>
        {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::IsolateOverride)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("plaintext") =>
        {
            CssUnicodeBidiQualificationOutcome::Qualified(CssUnicodeBidiValue::Plaintext)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssUnicodeBidiQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssUnicodeBidiUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssUnicodeBidiQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_mask_type_value(items: &[CssLexicalItem]) -> CssMaskTypeQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssMaskTypeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMaskTypeUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssMaskTypeQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("luminance") =>
        {
            CssMaskTypeQualificationOutcome::Qualified(CssMaskTypeValue::Luminance)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("alpha") =>
        {
            CssMaskTypeQualificationOutcome::Qualified(CssMaskTypeValue::Alpha)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssMaskTypeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMaskTypeUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssMaskTypeQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_color_interpolation_filters_value(
    items: &[CssLexicalItem],
) -> CssColorInterpolationFiltersQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssColorInterpolationFiltersQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColorInterpolationFiltersUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssColorInterpolationFiltersQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssColorInterpolationFiltersQualificationOutcome::Qualified(
                CssColorInterpolationFiltersValue::Auto,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("srgb") =>
        {
            CssColorInterpolationFiltersQualificationOutcome::Qualified(
                CssColorInterpolationFiltersValue::Srgb,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("linearrgb") =>
        {
            CssColorInterpolationFiltersQualificationOutcome::Qualified(
                CssColorInterpolationFiltersValue::LinearRgb,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssColorInterpolationFiltersQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColorInterpolationFiltersUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssColorInterpolationFiltersQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_shape_rendering_value(
    items: &[CssLexicalItem],
) -> CssShapeRenderingQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssShapeRenderingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeRenderingUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssShapeRenderingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssShapeRenderingQualificationOutcome::Qualified(CssShapeRenderingValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("optimizespeed") =>
        {
            CssShapeRenderingQualificationOutcome::Qualified(CssShapeRenderingValue::OptimizeSpeed)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("crispedges") =>
        {
            CssShapeRenderingQualificationOutcome::Qualified(CssShapeRenderingValue::CrispEdges)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("geometricprecision") =>
        {
            CssShapeRenderingQualificationOutcome::Qualified(
                CssShapeRenderingValue::GeometricPrecision,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssShapeRenderingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeRenderingUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssShapeRenderingQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_text_rendering_value(items: &[CssLexicalItem]) -> CssTextRenderingQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssTextRenderingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextRenderingUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssTextRenderingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssTextRenderingQualificationOutcome::Qualified(CssTextRenderingValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("optimizespeed") =>
        {
            CssTextRenderingQualificationOutcome::Qualified(CssTextRenderingValue::OptimizeSpeed)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("optimizelegibility") =>
        {
            CssTextRenderingQualificationOutcome::Qualified(
                CssTextRenderingValue::OptimizeLegibility,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("geometricprecision") =>
        {
            CssTextRenderingQualificationOutcome::Qualified(
                CssTextRenderingValue::GeometricPrecision,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssTextRenderingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextRenderingUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssTextRenderingQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_text_anchor_value(items: &[CssLexicalItem]) -> CssTextAnchorQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssTextAnchorQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextAnchorUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssTextAnchorQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("start") =>
        {
            CssTextAnchorQualificationOutcome::Qualified(CssTextAnchorValue::Start)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("middle") =>
        {
            CssTextAnchorQualificationOutcome::Qualified(CssTextAnchorValue::Middle)
        }
        CssSingleKeywordValue::Identifier(identifier) if identifier.eq_ignore_ascii_case("end") => {
            CssTextAnchorQualificationOutcome::Qualified(CssTextAnchorValue::End)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssTextAnchorQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextAnchorUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssTextAnchorQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_forced_color_adjust_value(
    items: &[CssLexicalItem],
) -> CssForcedColorAdjustQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssForcedColorAdjustQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssForcedColorAdjustUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssForcedColorAdjustQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssForcedColorAdjustQualificationOutcome::Qualified(CssForcedColorAdjustValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssForcedColorAdjustQualificationOutcome::Qualified(CssForcedColorAdjustValue::None)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("preserve-parent-color") =>
        {
            CssForcedColorAdjustQualificationOutcome::Qualified(
                CssForcedColorAdjustValue::PreserveParentColor,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssForcedColorAdjustQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssForcedColorAdjustUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssForcedColorAdjustQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_text_align_last_value(items: &[CssLexicalItem]) -> CssTextAlignLastQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssTextAlignLastQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextAlignLastUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssTextAlignLastQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("start") =>
        {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Start)
        }
        CssSingleKeywordValue::Identifier(identifier) if identifier.eq_ignore_ascii_case("end") => {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::End)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("left") =>
        {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Left)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("right") =>
        {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Right)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("center") =>
        {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Center)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("justify") =>
        {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::Justify)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("match-parent") =>
        {
            CssTextAlignLastQualificationOutcome::Qualified(CssTextAlignLastValue::MatchParent)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssTextAlignLastQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextAlignLastUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssTextAlignLastQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_math_style_value(items: &[CssLexicalItem]) -> CssMathStyleQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssMathStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMathStyleUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssMathStyleQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssMathStyleQualificationOutcome::Qualified(CssMathStyleValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("compact") =>
        {
            CssMathStyleQualificationOutcome::Qualified(CssMathStyleValue::Compact)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssMathStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMathStyleUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssMathStyleQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_math_shift_value(items: &[CssLexicalItem]) -> CssMathShiftQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssMathShiftQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMathShiftUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssMathShiftQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("normal") =>
        {
            CssMathShiftQualificationOutcome::Qualified(CssMathShiftValue::Normal)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("compact") =>
        {
            CssMathShiftQualificationOutcome::Qualified(CssMathShiftValue::Compact)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssMathShiftQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssMathShiftUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssMathShiftQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_ruby_align_value(items: &[CssLexicalItem]) -> CssRubyAlignQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssRubyAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyAlignUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssRubyAlignQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("start") =>
        {
            CssRubyAlignQualificationOutcome::Qualified(CssRubyAlignValue::Start)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("center") =>
        {
            CssRubyAlignQualificationOutcome::Qualified(CssRubyAlignValue::Center)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("space-between") =>
        {
            CssRubyAlignQualificationOutcome::Qualified(CssRubyAlignValue::SpaceBetween)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("space-around") =>
        {
            CssRubyAlignQualificationOutcome::Qualified(CssRubyAlignValue::SpaceAround)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssRubyAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyAlignUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssRubyAlignQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_ruby_merge_value(items: &[CssLexicalItem]) -> CssRubyMergeQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssRubyMergeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyMergeUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssRubyMergeQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("separate") =>
        {
            CssRubyMergeQualificationOutcome::Qualified(CssRubyMergeValue::Separate)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("merge") =>
        {
            CssRubyMergeQualificationOutcome::Qualified(CssRubyMergeValue::Merge)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssRubyMergeQualificationOutcome::Qualified(CssRubyMergeValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssRubyMergeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyMergeUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssRubyMergeQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_ruby_position_value(items: &[CssLexicalItem]) -> CssRubyPositionQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssRubyPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssRubyPositionUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssRubyPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssRubyPositionUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
    {
        if is_css_wide_keyword(identifier) {
            return CssRubyPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyPositionUnsupportedReason::CssWideKeyword,
            );
        }
        if identifier.eq_ignore_ascii_case("inter-character") {
            return CssRubyPositionQualificationOutcome::Qualified(
                CssRubyPositionValue::InterCharacter,
            );
        }
    }

    if tokens.is_empty() || tokens.len() > 2 {
        return CssRubyPositionQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut authored = [CssRubyPositionComponent::Alternate; 2];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssRubyPositionQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        let Some((component, slot)) = ruby_position_component(identifier) else {
            return CssRubyPositionQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        if occupied_slots & slot != 0 {
            return CssRubyPositionQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    CssRubyPositionQualificationOutcome::Qualified(CssRubyPositionValue::Components(
        CssRubyPositionComponents { authored, count },
    ))
}

fn ruby_position_component(identifier: &str) -> Option<(CssRubyPositionComponent, u8)> {
    if identifier.eq_ignore_ascii_case("alternate") {
        return Some((CssRubyPositionComponent::Alternate, 0b01));
    }
    if identifier.eq_ignore_ascii_case("over") {
        return Some((CssRubyPositionComponent::Over, 0b10));
    }
    if identifier.eq_ignore_ascii_case("under") {
        return Some((CssRubyPositionComponent::Under, 0b10));
    }
    None
}

fn qualify_ruby_overhang_value(items: &[CssLexicalItem]) -> CssRubyOverhangQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssRubyOverhangQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyOverhangUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssRubyOverhangQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssRubyOverhangQualificationOutcome::Qualified(CssRubyOverhangValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("spaces")
                || identifier.eq_ignore_ascii_case("none") =>
        {
            CssRubyOverhangQualificationOutcome::Qualified(CssRubyOverhangValue::Spaces)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssRubyOverhangQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyOverhangUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssRubyOverhangQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_clip_rule_value(items: &[CssLexicalItem]) -> CssClipRuleQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssClipRuleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssClipRuleUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssClipRuleQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("nonzero") =>
        {
            CssClipRuleQualificationOutcome::Qualified(CssClipRuleValue::Nonzero)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("evenodd") =>
        {
            CssClipRuleQualificationOutcome::Qualified(CssClipRuleValue::Evenodd)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssClipRuleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssClipRuleUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssClipRuleQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_fill_rule_value(items: &[CssLexicalItem]) -> CssFillRuleQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssFillRuleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillRuleUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssFillRuleQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("nonzero") =>
        {
            CssFillRuleQualificationOutcome::Qualified(CssFillRuleValue::Nonzero)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("evenodd") =>
        {
            CssFillRuleQualificationOutcome::Qualified(CssFillRuleValue::Evenodd)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssFillRuleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillRuleUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssFillRuleQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_column_fill_value(items: &[CssLexicalItem]) -> CssColumnFillQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssColumnFillQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColumnFillUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssColumnFillQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssColumnFillQualificationOutcome::Qualified(CssColumnFillValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("balance") =>
        {
            CssColumnFillQualificationOutcome::Qualified(CssColumnFillValue::Balance)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("balance-all") =>
        {
            CssColumnFillQualificationOutcome::Qualified(CssColumnFillValue::BalanceAll)
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssColumnFillQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColumnFillUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssColumnFillQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_text_decoration_skip_ink_value(
    items: &[CssLexicalItem],
) -> CssTextDecorationSkipInkQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssTextDecorationSkipInkQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextDecorationSkipInkUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssTextDecorationSkipInkQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssTextDecorationSkipInkQualificationOutcome::Qualified(
                CssTextDecorationSkipInkValue::Auto,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssTextDecorationSkipInkQualificationOutcome::Qualified(
                CssTextDecorationSkipInkValue::None,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if identifier.eq_ignore_ascii_case("all") => {
            CssTextDecorationSkipInkQualificationOutcome::Qualified(
                CssTextDecorationSkipInkValue::All,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssTextDecorationSkipInkQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextDecorationSkipInkUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssTextDecorationSkipInkQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_contain_value(items: &[CssLexicalItem]) -> CssContainQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssContainQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssContainUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssContainQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssContainUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
    {
        if is_css_wide_keyword(identifier) {
            return CssContainQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssContainUnsupportedReason::CssWideKeyword,
            );
        }
        if identifier.eq_ignore_ascii_case("none") {
            return CssContainQualificationOutcome::Qualified(CssContainValue::None);
        }
        if identifier.eq_ignore_ascii_case("strict") {
            return CssContainQualificationOutcome::Qualified(CssContainValue::Strict);
        }
        if identifier.eq_ignore_ascii_case("content") {
            return CssContainQualificationOutcome::Qualified(CssContainValue::Content);
        }
    }

    if tokens.is_empty() {
        return CssContainQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut authored = [CssContainComponent::Size; 4];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssContainQualificationOutcome::InvalidForSelectedValueGrammar;
        };

        let Some((component, slot)) = contain_component(identifier) else {
            return CssContainQualificationOutcome::InvalidForSelectedValueGrammar;
        };

        if occupied_slots & slot != 0 {
            return CssContainQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    CssContainQualificationOutcome::Qualified(CssContainValue::Components(CssContainComponents {
        authored,
        count,
    }))
}

fn contain_component(identifier: &str) -> Option<(CssContainComponent, u8)> {
    if identifier.eq_ignore_ascii_case("size") {
        return Some((CssContainComponent::Size, 0b0001));
    }
    if identifier.eq_ignore_ascii_case("inline-size") {
        return Some((CssContainComponent::InlineSize, 0b0001));
    }
    if identifier.eq_ignore_ascii_case("layout") {
        return Some((CssContainComponent::Layout, 0b0010));
    }
    if identifier.eq_ignore_ascii_case("style") {
        return Some((CssContainComponent::Style, 0b0100));
    }
    if identifier.eq_ignore_ascii_case("paint") {
        return Some((CssContainComponent::Paint, 0b1000));
    }
    None
}

fn qualify_font_variant_ligatures_value(
    items: &[CssLexicalItem],
) -> CssFontVariantLigaturesQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssFontVariantLigaturesQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFontVariantLigaturesUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssFontVariantLigaturesQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFontVariantLigaturesUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
    {
        if is_css_wide_keyword(identifier) {
            return CssFontVariantLigaturesQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantLigaturesUnsupportedReason::CssWideKeyword,
            );
        }
        if identifier.eq_ignore_ascii_case("normal") {
            return CssFontVariantLigaturesQualificationOutcome::Qualified(
                CssFontVariantLigaturesValue::Normal,
            );
        }
        if identifier.eq_ignore_ascii_case("none") {
            return CssFontVariantLigaturesQualificationOutcome::Qualified(
                CssFontVariantLigaturesValue::None,
            );
        }
    }

    if tokens.is_empty() || tokens.len() > 4 {
        return CssFontVariantLigaturesQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut authored = [CssFontVariantLigaturesComponent::CommonLigatures; 4];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssFontVariantLigaturesQualificationOutcome::InvalidForSelectedValueGrammar;
        };

        let Some((component, slot)) = font_variant_ligatures_component(identifier) else {
            return CssFontVariantLigaturesQualificationOutcome::InvalidForSelectedValueGrammar;
        };

        if occupied_slots & slot != 0 {
            return CssFontVariantLigaturesQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    CssFontVariantLigaturesQualificationOutcome::Qualified(
        CssFontVariantLigaturesValue::Components(CssFontVariantLigaturesComponents {
            authored,
            count,
        }),
    )
}

fn font_variant_ligatures_component(
    identifier: &str,
) -> Option<(CssFontVariantLigaturesComponent, u8)> {
    if identifier.eq_ignore_ascii_case("common-ligatures") {
        return Some((CssFontVariantLigaturesComponent::CommonLigatures, 0b0001));
    }
    if identifier.eq_ignore_ascii_case("no-common-ligatures") {
        return Some((CssFontVariantLigaturesComponent::NoCommonLigatures, 0b0001));
    }
    if identifier.eq_ignore_ascii_case("discretionary-ligatures") {
        return Some((
            CssFontVariantLigaturesComponent::DiscretionaryLigatures,
            0b0010,
        ));
    }
    if identifier.eq_ignore_ascii_case("no-discretionary-ligatures") {
        return Some((
            CssFontVariantLigaturesComponent::NoDiscretionaryLigatures,
            0b0010,
        ));
    }
    if identifier.eq_ignore_ascii_case("historical-ligatures") {
        return Some((
            CssFontVariantLigaturesComponent::HistoricalLigatures,
            0b0100,
        ));
    }
    if identifier.eq_ignore_ascii_case("no-historical-ligatures") {
        return Some((
            CssFontVariantLigaturesComponent::NoHistoricalLigatures,
            0b0100,
        ));
    }
    if identifier.eq_ignore_ascii_case("contextual") {
        return Some((CssFontVariantLigaturesComponent::Contextual, 0b1000));
    }
    if identifier.eq_ignore_ascii_case("no-contextual") {
        return Some((CssFontVariantLigaturesComponent::NoContextual, 0b1000));
    }
    None
}

fn qualify_font_variant_numeric_value(
    items: &[CssLexicalItem],
) -> CssFontVariantNumericQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFontVariantNumericUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssFontVariantNumericUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
    {
        if is_css_wide_keyword(identifier) {
            return CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantNumericUnsupportedReason::CssWideKeyword,
            );
        }
        if identifier.eq_ignore_ascii_case("normal") {
            return CssFontVariantNumericQualificationOutcome::Qualified(
                CssFontVariantNumericValue::Normal,
            );
        }
    }

    if tokens.is_empty() || tokens.len() > 5 {
        return CssFontVariantNumericQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut authored = [CssFontVariantNumericComponent::LiningNums; 5];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssFontVariantNumericQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        let Some((component, slot)) = font_variant_numeric_component(identifier) else {
            return CssFontVariantNumericQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        if occupied_slots & slot != 0 {
            return CssFontVariantNumericQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    CssFontVariantNumericQualificationOutcome::Qualified(CssFontVariantNumericValue::Components(
        CssFontVariantNumericComponents { authored, count },
    ))
}

fn font_variant_numeric_component(
    identifier: &str,
) -> Option<(CssFontVariantNumericComponent, u8)> {
    if identifier.eq_ignore_ascii_case("lining-nums") {
        return Some((CssFontVariantNumericComponent::LiningNums, 0b00001));
    }
    if identifier.eq_ignore_ascii_case("oldstyle-nums") {
        return Some((CssFontVariantNumericComponent::OldstyleNums, 0b00001));
    }
    if identifier.eq_ignore_ascii_case("proportional-nums") {
        return Some((CssFontVariantNumericComponent::ProportionalNums, 0b00010));
    }
    if identifier.eq_ignore_ascii_case("tabular-nums") {
        return Some((CssFontVariantNumericComponent::TabularNums, 0b00010));
    }
    if identifier.eq_ignore_ascii_case("diagonal-fractions") {
        return Some((CssFontVariantNumericComponent::DiagonalFractions, 0b00100));
    }
    if identifier.eq_ignore_ascii_case("stacked-fractions") {
        return Some((CssFontVariantNumericComponent::StackedFractions, 0b00100));
    }
    if identifier.eq_ignore_ascii_case("ordinal") {
        return Some((CssFontVariantNumericComponent::Ordinal, 0b01000));
    }
    if identifier.eq_ignore_ascii_case("slashed-zero") {
        return Some((CssFontVariantNumericComponent::SlashedZero, 0b10000));
    }
    None
}

fn qualify_text_decoration_line_value(
    items: &[CssLexicalItem],
) -> CssTextDecorationLineQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTextDecorationLineQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextDecorationLineUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTextDecorationLineQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextDecorationLineUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
    {
        if is_css_wide_keyword(identifier) {
            return CssTextDecorationLineQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextDecorationLineUnsupportedReason::CssWideKeyword,
            );
        }
        if identifier.eq_ignore_ascii_case("none") {
            return CssTextDecorationLineQualificationOutcome::Qualified(
                CssTextDecorationLineValue::None,
            );
        }
        if identifier.eq_ignore_ascii_case("spelling-error") {
            return CssTextDecorationLineQualificationOutcome::Qualified(
                CssTextDecorationLineValue::SpellingError,
            );
        }
        if identifier.eq_ignore_ascii_case("grammar-error") {
            return CssTextDecorationLineQualificationOutcome::Qualified(
                CssTextDecorationLineValue::GrammarError,
            );
        }
    }

    if tokens.is_empty() || tokens.len() > 4 {
        return CssTextDecorationLineQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut authored = [CssTextDecorationLineComponent::Underline; 4];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssTextDecorationLineQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        let Some((component, slot)) = text_decoration_line_component(identifier) else {
            return CssTextDecorationLineQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        if occupied_slots & slot != 0 {
            return CssTextDecorationLineQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    CssTextDecorationLineQualificationOutcome::Qualified(CssTextDecorationLineValue::Components(
        CssTextDecorationLineComponents { authored, count },
    ))
}

fn text_decoration_line_component(
    identifier: &str,
) -> Option<(CssTextDecorationLineComponent, u8)> {
    if identifier.eq_ignore_ascii_case("underline") {
        return Some((CssTextDecorationLineComponent::Underline, 0b0001));
    }
    if identifier.eq_ignore_ascii_case("overline") {
        return Some((CssTextDecorationLineComponent::Overline, 0b0010));
    }
    if identifier.eq_ignore_ascii_case("line-through") {
        return Some((CssTextDecorationLineComponent::LineThrough, 0b0100));
    }
    if identifier.eq_ignore_ascii_case("blink") {
        return Some((CssTextDecorationLineComponent::Blink, 0b1000));
    }
    None
}

fn qualify_text_transform_value(items: &[CssLexicalItem]) -> CssTextTransformQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTextTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextTransformUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTextTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextTransformUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
    {
        if is_css_wide_keyword(identifier) {
            return CssTextTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextTransformUnsupportedReason::CssWideKeyword,
            );
        }
        if identifier.eq_ignore_ascii_case("none") {
            return CssTextTransformQualificationOutcome::Qualified(CssTextTransformValue::None);
        }
        if identifier.eq_ignore_ascii_case("math-auto") {
            return CssTextTransformQualificationOutcome::Qualified(
                CssTextTransformValue::MathAuto,
            );
        }
    }

    if tokens.is_empty() || tokens.len() > 3 {
        return CssTextTransformQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut authored = [CssTextTransformComponent::Capitalize; 3];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssTextTransformQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        let Some((component, slot)) = text_transform_component(identifier) else {
            return CssTextTransformQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        if occupied_slots & slot != 0 {
            return CssTextTransformQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    CssTextTransformQualificationOutcome::Qualified(CssTextTransformValue::Components(
        CssTextTransformComponents { authored, count },
    ))
}

fn text_transform_component(identifier: &str) -> Option<(CssTextTransformComponent, u8)> {
    if identifier.eq_ignore_ascii_case("capitalize") {
        return Some((CssTextTransformComponent::Capitalize, 0b001));
    }
    if identifier.eq_ignore_ascii_case("uppercase") {
        return Some((CssTextTransformComponent::Uppercase, 0b001));
    }
    if identifier.eq_ignore_ascii_case("lowercase") {
        return Some((CssTextTransformComponent::Lowercase, 0b001));
    }
    if identifier.eq_ignore_ascii_case("full-width") {
        return Some((CssTextTransformComponent::FullWidth, 0b010));
    }
    if identifier.eq_ignore_ascii_case("full-size-kana") {
        return Some((CssTextTransformComponent::FullSizeKana, 0b100));
    }
    None
}

fn qualify_text_emphasis_position_value(
    items: &[CssLexicalItem],
) -> CssTextEmphasisPositionQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTextEmphasisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextEmphasisPositionUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTextEmphasisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextEmphasisPositionUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssTextEmphasisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextEmphasisPositionUnsupportedReason::CssWideKeyword,
        );
    }

    if tokens.is_empty() || tokens.len() > 2 {
        return CssTextEmphasisPositionQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut authored = [CssTextEmphasisPositionComponent::Over; 2];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssTextEmphasisPositionQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        let Some((component, slot)) = text_emphasis_position_component(identifier) else {
            return CssTextEmphasisPositionQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        if occupied_slots & slot != 0 {
            return CssTextEmphasisPositionQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    const VERTICAL_SLOT: u8 = 0b01;
    if occupied_slots & VERTICAL_SLOT == 0 {
        return CssTextEmphasisPositionQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    CssTextEmphasisPositionQualificationOutcome::Qualified(
        CssTextEmphasisPositionValue::Components(CssTextEmphasisPositionComponents {
            authored,
            count,
        }),
    )
}

fn text_emphasis_position_component(
    identifier: &str,
) -> Option<(CssTextEmphasisPositionComponent, u8)> {
    if identifier.eq_ignore_ascii_case("over") {
        return Some((CssTextEmphasisPositionComponent::Over, 0b01));
    }
    if identifier.eq_ignore_ascii_case("under") {
        return Some((CssTextEmphasisPositionComponent::Under, 0b01));
    }
    if identifier.eq_ignore_ascii_case("right") {
        return Some((CssTextEmphasisPositionComponent::Right, 0b10));
    }
    if identifier.eq_ignore_ascii_case("left") {
        return Some((CssTextEmphasisPositionComponent::Left, 0b10));
    }
    None
}

fn qualify_text_underline_position_value(
    items: &[CssLexicalItem],
) -> CssTextUnderlinePositionQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTextUnderlinePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextUnderlinePositionUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTextUnderlinePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextUnderlinePositionUnsupportedReason::WholeValueFunction,
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

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssTextUnderlinePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextUnderlinePositionUnsupportedReason::CssWideKeyword,
        );
    }

    if tokens.is_empty() || tokens.len() > 2 {
        return CssTextUnderlinePositionQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if let [token] = tokens.as_slice()
        && let CssTokenKind::Ident(identifier) = token.kind()
        && identifier.eq_ignore_ascii_case("auto")
    {
        return CssTextUnderlinePositionQualificationOutcome::Qualified(
            CssTextUnderlinePositionValue::Auto,
        );
    }

    let mut authored = [CssTextUnderlinePositionComponent::FromFont; 2];
    let mut count = 0usize;
    let mut occupied_slots = 0u8;

    for token in tokens {
        let CssTokenKind::Ident(identifier) = token.kind() else {
            return CssTextUnderlinePositionQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        let Some((component, slot)) = text_underline_position_component(identifier) else {
            return CssTextUnderlinePositionQualificationOutcome::InvalidForSelectedValueGrammar;
        };
        if occupied_slots & slot != 0 {
            return CssTextUnderlinePositionQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        occupied_slots |= slot;
        authored[count] = component;
        count += 1;
    }

    CssTextUnderlinePositionQualificationOutcome::Qualified(
        CssTextUnderlinePositionValue::Components(CssTextUnderlinePositionComponents {
            authored,
            count,
        }),
    )
}

fn text_underline_position_component(
    identifier: &str,
) -> Option<(CssTextUnderlinePositionComponent, u8)> {
    if identifier.eq_ignore_ascii_case("from-font") {
        return Some((CssTextUnderlinePositionComponent::FromFont, 0b01));
    }
    if identifier.eq_ignore_ascii_case("under") {
        return Some((CssTextUnderlinePositionComponent::Under, 0b01));
    }
    if identifier.eq_ignore_ascii_case("right") {
        return Some((CssTextUnderlinePositionComponent::Right, 0b10));
    }
    if identifier.eq_ignore_ascii_case("left") {
        return Some((CssTextUnderlinePositionComponent::Left, 0b10));
    }
    None
}

fn qualify_overscroll_behavior_value(
    items: &[CssLexicalItem],
) -> CssOverscrollBehaviorQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssOverscrollBehaviorQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOverscrollBehaviorUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssOverscrollBehaviorQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOverscrollBehaviorUnsupportedReason::WholeValueFunction,
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
                CssOverscrollBehaviorQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssOverscrollBehaviorUnsupportedReason::CssWideKeyword,
                )
            }
            CssTokenKind::Ident(identifier) => overscroll_behavior_keyword(identifier)
                .map(|keyword| {
                    CssOverscrollBehaviorQualificationOutcome::Qualified(
                        CssOverscrollBehaviorValue::Single(keyword),
                    )
                })
                .unwrap_or(
                    CssOverscrollBehaviorQualificationOutcome::InvalidForSelectedValueGrammar,
                ),
            _ => CssOverscrollBehaviorQualificationOutcome::InvalidForSelectedValueGrammar,
        },
        [first, second] => match (first.kind(), second.kind()) {
            (CssTokenKind::Ident(first), CssTokenKind::Ident(second)) => {
                match (
                    overscroll_behavior_keyword(first),
                    overscroll_behavior_keyword(second),
                ) {
                    (Some(first), Some(second)) => {
                        CssOverscrollBehaviorQualificationOutcome::Qualified(
                            CssOverscrollBehaviorValue::Pair { first, second },
                        )
                    }
                    _ => CssOverscrollBehaviorQualificationOutcome::InvalidForSelectedValueGrammar,
                }
            }
            _ => CssOverscrollBehaviorQualificationOutcome::InvalidForSelectedValueGrammar,
        },
        _ => CssOverscrollBehaviorQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn overscroll_behavior_keyword(identifier: &str) -> Option<CssOverscrollBehaviorKeyword> {
    if identifier.eq_ignore_ascii_case("contain") {
        return Some(CssOverscrollBehaviorKeyword::Contain);
    }
    if identifier.eq_ignore_ascii_case("none") {
        return Some(CssOverscrollBehaviorKeyword::None);
    }
    if identifier.eq_ignore_ascii_case("auto") {
        return Some(CssOverscrollBehaviorKeyword::Auto);
    }
    if identifier.eq_ignore_ascii_case("chain") {
        return Some(CssOverscrollBehaviorKeyword::Chain);
    }
    None
}

fn qualify_overscroll_behavior_x_value(
    items: &[CssLexicalItem],
) -> CssOverscrollBehaviorXQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssOverscrollBehaviorXQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorXUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssOverscrollBehaviorXQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("contain") =>
        {
            CssOverscrollBehaviorXQualificationOutcome::Qualified(
                CssOverscrollBehaviorXValue::Contain,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssOverscrollBehaviorXQualificationOutcome::Qualified(CssOverscrollBehaviorXValue::None)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssOverscrollBehaviorXQualificationOutcome::Qualified(CssOverscrollBehaviorXValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("chain") =>
        {
            CssOverscrollBehaviorXQualificationOutcome::Qualified(
                CssOverscrollBehaviorXValue::Chain,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssOverscrollBehaviorXQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorXUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssOverscrollBehaviorXQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_overscroll_behavior_y_value(
    items: &[CssLexicalItem],
) -> CssOverscrollBehaviorYQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssOverscrollBehaviorYQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorYUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssOverscrollBehaviorYQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("contain") =>
        {
            CssOverscrollBehaviorYQualificationOutcome::Qualified(
                CssOverscrollBehaviorYValue::Contain,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssOverscrollBehaviorYQualificationOutcome::Qualified(CssOverscrollBehaviorYValue::None)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssOverscrollBehaviorYQualificationOutcome::Qualified(CssOverscrollBehaviorYValue::Auto)
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("chain") =>
        {
            CssOverscrollBehaviorYQualificationOutcome::Qualified(
                CssOverscrollBehaviorYValue::Chain,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssOverscrollBehaviorYQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorYUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssOverscrollBehaviorYQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_overscroll_behavior_inline_value(
    items: &[CssLexicalItem],
) -> CssOverscrollBehaviorInlineQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssOverscrollBehaviorInlineQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorInlineUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssOverscrollBehaviorInlineQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("contain") =>
        {
            CssOverscrollBehaviorInlineQualificationOutcome::Qualified(
                CssOverscrollBehaviorInlineValue::Contain,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssOverscrollBehaviorInlineQualificationOutcome::Qualified(
                CssOverscrollBehaviorInlineValue::None,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssOverscrollBehaviorInlineQualificationOutcome::Qualified(
                CssOverscrollBehaviorInlineValue::Auto,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("chain") =>
        {
            CssOverscrollBehaviorInlineQualificationOutcome::Qualified(
                CssOverscrollBehaviorInlineValue::Chain,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssOverscrollBehaviorInlineQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorInlineUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssOverscrollBehaviorInlineQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

fn qualify_overscroll_behavior_block_value(
    items: &[CssLexicalItem],
) -> CssOverscrollBehaviorBlockQualificationOutcome {
    match classify_single_keyword_value(items) {
        CssSingleKeywordValue::UnsupportedFunction => {
            CssOverscrollBehaviorBlockQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorBlockUnsupportedReason::FunctionValue,
            )
        }
        CssSingleKeywordValue::Invalid => {
            CssOverscrollBehaviorBlockQualificationOutcome::InvalidForSelectedValueGrammar
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("contain") =>
        {
            CssOverscrollBehaviorBlockQualificationOutcome::Qualified(
                CssOverscrollBehaviorBlockValue::Contain,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("none") =>
        {
            CssOverscrollBehaviorBlockQualificationOutcome::Qualified(
                CssOverscrollBehaviorBlockValue::None,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("auto") =>
        {
            CssOverscrollBehaviorBlockQualificationOutcome::Qualified(
                CssOverscrollBehaviorBlockValue::Auto,
            )
        }
        CssSingleKeywordValue::Identifier(identifier)
            if identifier.eq_ignore_ascii_case("chain") =>
        {
            CssOverscrollBehaviorBlockQualificationOutcome::Qualified(
                CssOverscrollBehaviorBlockValue::Chain,
            )
        }
        CssSingleKeywordValue::Identifier(identifier) if is_css_wide_keyword(identifier) => {
            CssOverscrollBehaviorBlockQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorBlockUnsupportedReason::CssWideKeyword,
            )
        }
        CssSingleKeywordValue::Identifier(_) => {
            CssOverscrollBehaviorBlockQualificationOutcome::InvalidForSelectedValueGrammar
        }
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

fn qualify_letter_spacing_value(items: &[CssLexicalItem]) -> CssLetterSpacingQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssLetterSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssLetterSpacingUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssLetterSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssLetterSpacingUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssLetterSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssLetterSpacingUnsupportedReason::FunctionValue,
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
        return CssLetterSpacingQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssLetterSpacingQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("normal") => {
            CssLetterSpacingQualificationOutcome::Qualified(CssLetterSpacingValue::Normal)
        }
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssLetterSpacingQualificationOutcome::Qualified(
                CssLetterSpacingValue::DirectLengthLiteral,
            )
        }
        CssTokenKind::Dimension { unit, .. } if is_css_length_unit(unit) => {
            CssLetterSpacingQualificationOutcome::Qualified(
                CssLetterSpacingValue::DirectLengthLiteral,
            )
        }
        CssTokenKind::Percentage { .. } => CssLetterSpacingQualificationOutcome::Qualified(
            CssLetterSpacingValue::DirectPercentageLiteral,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssLetterSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLetterSpacingUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssLetterSpacingQualificationOutcome::InvalidForSelectedValueGrammar,
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

fn is_css_time_unit(unit: &str) -> bool {
    ["s", "ms"]
        .iter()
        .any(|time_unit| unit.eq_ignore_ascii_case(time_unit))
}

fn is_css_angle_unit(unit: &str) -> bool {
    ["deg", "grad", "rad", "turn"]
        .iter()
        .any(|angle_unit| unit.eq_ignore_ascii_case(angle_unit))
}

fn qualify_page_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> (
    CssPageQualificationOutcome,
    Option<CssPageCustomIdentEvidenceRef>,
) {
    if contains_deferred_substitution_function(items) {
        return (
            CssPageQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPageUnsupportedReason::DeferredSubstitutionFunction,
            ),
            None,
        );
    }

    if is_entire_whole_value_function(items) {
        return (
            CssPageQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPageUnsupportedReason::WholeValueFunction,
            ),
            None,
        );
    }

    let mut tokens = items
        .iter()
        .enumerate()
        .filter_map(|(relative_index, item)| match item {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return (
            CssPageQualificationOutcome::InvalidForSelectedValueGrammar,
            None,
        );
    };
    if tokens.next().is_some() {
        return (
            CssPageQualificationOutcome::InvalidForSelectedValueGrammar,
            None,
        );
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto") => (
            CssPageQualificationOutcome::Qualified(CssPageValue::Auto),
            None,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => (
            CssPageQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPageUnsupportedReason::CssWideKeyword,
            ),
            None,
        ),
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("default") => (
            CssPageQualificationOutcome::InvalidForSelectedValueGrammar,
            None,
        ),
        CssTokenKind::Ident(_) => (
            CssPageQualificationOutcome::Qualified(CssPageValue::CustomIdent),
            Some(CssPageCustomIdentEvidenceRef {
                lexical_item_index: lexical_item_start + relative_index,
            }),
        ),
        _ => (
            CssPageQualificationOutcome::InvalidForSelectedValueGrammar,
            None,
        ),
    }
}

/// Qualifies one selected ordinary `hyphenate-character` declaration's
/// already-retained value window against the authored `auto | <string>`
/// grammar (#580 / #418 comment 5581206679).
///
/// After the shared deferred-substitution and whole-value-Function preflight,
/// exactly one non-trivia retained token decides the outcome: a direct Ident
/// `auto` (ASCII-case-insensitive) qualifies as `Auto` with no String
/// evidence; a direct `CssTokenKind::String(_)` qualifies as
/// `DirectStringLiteral` with an evidence reference to that exact retained
/// token; every other single token (including a wrong token class or a
/// retained `BadString`), zero tokens, or more than one token is
/// `InvalidForSelectedValueGrammar`. This grammar has no ordinary
/// Function-backed branch, so a Function token falls through to the same
/// Invalid outcome rather than a residual `FunctionValue` Unsupported class.
/// The decoded String payload is never copied out of the tokenizer's
/// `CssTokenKind::String(String)`; only its run-local lexical-item position
/// is retained.
fn qualify_hyphenate_character_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> (
    CssHyphenateCharacterQualificationOutcome,
    Option<CssHyphenateCharacterStringEvidenceRef>,
) {
    if contains_deferred_substitution_function(items) {
        return (
            CssHyphenateCharacterQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssHyphenateCharacterUnsupportedReason::DeferredSubstitutionFunction,
            ),
            None,
        );
    }

    if is_entire_whole_value_function(items) {
        return (
            CssHyphenateCharacterQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssHyphenateCharacterUnsupportedReason::WholeValueFunction,
            ),
            None,
        );
    }

    let mut tokens = items
        .iter()
        .enumerate()
        .filter_map(|(relative_index, item)| match item {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return (
            CssHyphenateCharacterQualificationOutcome::InvalidForSelectedValueGrammar,
            None,
        );
    };
    if tokens.next().is_some() {
        return (
            CssHyphenateCharacterQualificationOutcome::InvalidForSelectedValueGrammar,
            None,
        );
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto") => (
            CssHyphenateCharacterQualificationOutcome::Qualified(CssHyphenateCharacterValue::Auto),
            None,
        ),
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => (
            CssHyphenateCharacterQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssHyphenateCharacterUnsupportedReason::CssWideKeyword,
            ),
            None,
        ),
        CssTokenKind::String(_) => (
            CssHyphenateCharacterQualificationOutcome::Qualified(
                CssHyphenateCharacterValue::DirectStringLiteral,
            ),
            Some(CssHyphenateCharacterStringEvidenceRef {
                lexical_item_index: lexical_item_start + relative_index,
            }),
        ),
        _ => (
            CssHyphenateCharacterQualificationOutcome::InvalidForSelectedValueGrammar,
            None,
        ),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssBorderSpacingComponentClass {
    QualifiedLength,
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Partitions an already-retained `border-spacing` declaration value window
/// into ordered top-level components using one left-to-right recognition-time
/// pass. Depth-zero Whitespace/Comment lexical items are separators; Function
/// and bracket openers extend the current component until their matching
/// closer, so nested content never inflates top-level cardinality. A
/// component also ends the instant block depth returns to zero — including a
/// direct token that never opened a block — so a following top-level token
/// always starts its own component even without an intervening separator. An
/// unmatched closer at depth zero does not affect depth tracking and simply
/// remains part of an ordinary top-level component.
fn border_spacing_top_level_components(items: &[CssLexicalItem]) -> Vec<&[CssLexicalItem]> {
    let mut components = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut current_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = current_start.take() {
                    components.push(&items[start..index]);
                }
                continue;
            }
        }

        if current_start.is_none() {
            current_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        // A component is complete the instant its block depth returns to
        // zero, whether that is a direct token that never opened a block or
        // a Function/bracket whose matching closer was just consumed. A
        // following top-level token must start its own component even when
        // no whitespace/comment separates it from this one.
        if block_stack.is_empty()
            && let Some(start) = current_start.take()
        {
            components.push(&items[start..=index]);
        }
    }

    if let Some(start) = current_start {
        components.push(&items[start..]);
    }

    components
}

/// Classifies one already-partitioned top-level `border-spacing` component.
///
/// A Function-headed component is classified by name/placement only; its
/// interior is never parsed. A direct component reuses the accepted direct
/// `<length [0,∞]>` boundary (unitless zero, or a non-negative length unit
/// dimension) already established for other direct-length leaves.
fn classify_border_spacing_component(
    component: &[CssLexicalItem],
) -> CssBorderSpacingComponentClass {
    let mut tokens = component.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(first) = tokens.next() else {
        return CssBorderSpacingComponentClass::Invalid;
    };

    if let CssTokenKind::Function(name) = first.kind() {
        return if is_whole_value_function(name) {
            CssBorderSpacingComponentClass::MisplacedWholeValueFunction
        } else {
            CssBorderSpacingComponentClass::ResidualFunction
        };
    }

    if tokens.next().is_some() {
        return CssBorderSpacingComponentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssBorderSpacingComponentClass::QualifiedLength
        }
        CssTokenKind::Dimension { value, unit, .. }
            if is_css_length_unit(unit) && is_non_negative_direct_number(value) =>
        {
            CssBorderSpacingComponentClass::QualifiedLength
        }
        _ => CssBorderSpacingComponentClass::Invalid,
    }
}

fn qualify_border_spacing_value(items: &[CssLexicalItem]) -> CssBorderSpacingQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssBorderSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssBorderSpacingUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssBorderSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssBorderSpacingUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssBorderSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssBorderSpacingUnsupportedReason::CssWideKeyword,
        );
    }

    let components = border_spacing_top_level_components(items);

    if components.is_empty() || components.len() > 2 {
        return CssBorderSpacingQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut has_residual_function = false;
    for component in &components {
        match classify_border_spacing_component(component) {
            CssBorderSpacingComponentClass::QualifiedLength => {}
            CssBorderSpacingComponentClass::ResidualFunction => {
                has_residual_function = true;
            }
            CssBorderSpacingComponentClass::MisplacedWholeValueFunction
            | CssBorderSpacingComponentClass::Invalid => {
                return CssBorderSpacingQualificationOutcome::InvalidForSelectedValueGrammar;
            }
        }
    }

    if has_residual_function {
        return CssBorderSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssBorderSpacingUnsupportedReason::FunctionValue,
        );
    }

    CssBorderSpacingQualificationOutcome::Qualified(if components.len() == 1 {
        CssBorderSpacingValue::Single
    } else {
        CssBorderSpacingValue::Pair
    })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssAspectRatioComponentClass {
    QualifiedNumber,
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Partitions an already-retained `aspect-ratio` declaration value window into
/// ordered top-level components using one left-to-right recognition-time
/// pass. Depth-zero Whitespace/Comment lexical items are separators; Function
/// and bracket openers extend the current component until their matching
/// closer, so nested content never inflates top-level cardinality. A
/// component also ends the instant block depth returns to zero — including a
/// direct token that never opened a block — so a following top-level token
/// always starts its own component even without an intervening separator.
/// This intentionally duplicates the equivalent `border-spacing` walk rather
/// than sharing it: this leaf keeps its own bounded local recognition.
fn aspect_ratio_top_level_components(items: &[CssLexicalItem]) -> Vec<&[CssLexicalItem]> {
    let mut components = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut current_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = current_start.take() {
                    components.push(&items[start..index]);
                }
                continue;
            }
        }

        if current_start.is_none() {
            current_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = current_start.take()
        {
            components.push(&items[start..=index]);
        }
    }

    if let Some(start) = current_start {
        components.push(&items[start..]);
    }

    components
}

/// Returns the component's only non-whitespace token, or `None` when the
/// component holds zero or more than one such token.
fn aspect_ratio_only_token(component: &[CssLexicalItem]) -> Option<&CssToken> {
    let mut tokens = component.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    let only = tokens.next()?;
    if tokens.next().is_some() {
        return None;
    }
    Some(only)
}

/// A component is the direct `auto` operand only when it is exactly one
/// `Ident("auto")` token; a Function-headed or multi-token component can
/// never satisfy this operand.
fn is_aspect_ratio_auto_component(component: &[CssLexicalItem]) -> bool {
    aspect_ratio_only_token(component)
        .is_some_and(|token| matches!(token.kind(), CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto")))
}

/// A component is the ratio-internal separator only when it is exactly one
/// `Delim('/')` token. A `/` nested inside a comment never reaches this
/// point as a lexical item, so this cannot be fooled by comment trivia that
/// visually resembles a slash.
fn is_aspect_ratio_slash_component(component: &[CssLexicalItem]) -> bool {
    aspect_ratio_only_token(component)
        .is_some_and(|token| matches!(token.kind(), CssTokenKind::Delim('/')))
}

/// Classifies one already-partitioned top-level `<ratio>` numeric-position
/// component. A Function-headed component is classified by name/placement
/// only; its interior is never parsed or evaluated. A direct component
/// reuses the already accepted direct `<number [0,∞]>` boundary unchanged.
fn classify_aspect_ratio_number_component(
    component: &[CssLexicalItem],
) -> CssAspectRatioComponentClass {
    let mut tokens = component.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(first) = tokens.next() else {
        return CssAspectRatioComponentClass::Invalid;
    };

    if let CssTokenKind::Function(name) = first.kind() {
        return if is_whole_value_function(name) {
            CssAspectRatioComponentClass::MisplacedWholeValueFunction
        } else {
            CssAspectRatioComponentClass::ResidualFunction
        };
    }

    if tokens.next().is_some() {
        return CssAspectRatioComponentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Number { value, .. } if is_non_negative_direct_number(value) => {
            CssAspectRatioComponentClass::QualifiedNumber
        }
        _ => CssAspectRatioComponentClass::Invalid,
    }
}

/// Classifies one already-identified contiguous three-component `<ratio>`
/// pair candidate (`[numerator, Delim('/'), denominator]`). A direct
/// decidable failure on either numeric position wins over a residual
/// Function elsewhere in the same pair: this is the load-bearing ordering
/// that keeps `-1 / calc(9)` Invalid rather than Function-Unsupported.
fn classify_aspect_ratio_ratio_pair(
    components: &[&[CssLexicalItem]],
) -> CssAspectRatioComponentClass {
    debug_assert_eq!(components.len(), 3);

    if !is_aspect_ratio_slash_component(components[1]) {
        return CssAspectRatioComponentClass::Invalid;
    }

    let numerator = classify_aspect_ratio_number_component(components[0]);
    let denominator = classify_aspect_ratio_number_component(components[2]);

    match (numerator, denominator) {
        (
            CssAspectRatioComponentClass::QualifiedNumber,
            CssAspectRatioComponentClass::QualifiedNumber,
        ) => CssAspectRatioComponentClass::QualifiedNumber,
        (CssAspectRatioComponentClass::Invalid, _)
        | (_, CssAspectRatioComponentClass::Invalid)
        | (CssAspectRatioComponentClass::MisplacedWholeValueFunction, _)
        | (_, CssAspectRatioComponentClass::MisplacedWholeValueFunction) => {
            CssAspectRatioComponentClass::Invalid
        }
        _ => CssAspectRatioComponentClass::ResidualFunction,
    }
}

/// Qualifies one already-retained `aspect-ratio` ordinary declaration value
/// window against the selected `auto || <ratio>` profile, where
/// `<ratio> = <number [0,∞]> [ / <number [0,∞]> ]?`.
///
/// Classification order is load-bearing: deferred/arbitrary substitution,
/// then whole-value Function, then whole-value CSS-wide keyword, then one
/// property-local depth-balanced operand-grouping walk that identifies the
/// optional direct `auto` operand and one optional contiguous `<ratio>`
/// operand in either authored order, rejecting duplicates and interleaving
/// before any residual Function is allowed to soften a direct failure into
/// a profile-Unsupported outcome.
fn qualify_aspect_ratio_value(items: &[CssLexicalItem]) -> CssAspectRatioQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssAspectRatioQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAspectRatioUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssAspectRatioQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAspectRatioUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssAspectRatioQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAspectRatioUnsupportedReason::CssWideKeyword,
        );
    }

    let components = aspect_ratio_top_level_components(items);

    if components.is_empty() {
        return CssAspectRatioQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let leading_auto = is_aspect_ratio_auto_component(components[0]);
    let trailing_auto =
        components.len() > 1 && is_aspect_ratio_auto_component(components[components.len() - 1]);

    let (has_auto, ratio_components): (bool, &[&[CssLexicalItem]]) = if components.len() == 1 {
        if leading_auto {
            (true, &[])
        } else {
            (false, &components[..])
        }
    } else if leading_auto {
        (true, &components[1..])
    } else if trailing_auto {
        (true, &components[..components.len() - 1])
    } else {
        (false, &components[..])
    };

    // A duplicate `auto`, or an `auto` interleaved inside what would
    // otherwise be a contiguous ratio operand, is never absorbed here: any
    // remaining `auto` component makes the composition malformed.
    if ratio_components
        .iter()
        .any(|component| is_aspect_ratio_auto_component(component))
    {
        return CssAspectRatioQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if ratio_components.is_empty() {
        debug_assert!(has_auto);
        return CssAspectRatioQualificationOutcome::Qualified(CssAspectRatioValue::Auto);
    }

    let (ratio_class, ratio_shape) = match ratio_components.len() {
        1 => (
            classify_aspect_ratio_number_component(ratio_components[0]),
            CssAspectRatioRatioValue::Single,
        ),
        3 => (
            classify_aspect_ratio_ratio_pair(ratio_components),
            CssAspectRatioRatioValue::Pair,
        ),
        _ => {
            return CssAspectRatioQualificationOutcome::InvalidForSelectedValueGrammar;
        }
    };

    match ratio_class {
        CssAspectRatioComponentClass::QualifiedNumber => {
            CssAspectRatioQualificationOutcome::Qualified(if has_auto {
                CssAspectRatioValue::AutoAndRatio(ratio_shape)
            } else {
                CssAspectRatioValue::Ratio(ratio_shape)
            })
        }
        CssAspectRatioComponentClass::ResidualFunction => {
            CssAspectRatioQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAspectRatioUnsupportedReason::FunctionValue,
            )
        }
        CssAspectRatioComponentClass::MisplacedWholeValueFunction
        | CssAspectRatioComponentClass::Invalid => {
            CssAspectRatioQualificationOutcome::InvalidForSelectedValueGrammar
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssOffsetRotateComponentClass {
    KeywordAuto,
    KeywordReverse,
    QualifiedAngle,
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Partitions an already-retained `offset-rotate` declaration value window
/// into ordered top-level components using one left-to-right
/// recognition-time pass. Depth-zero Whitespace/Comment lexical items are
/// separators; Function and bracket openers extend the current component
/// until their matching closer, so nested content never inflates top-level
/// cardinality. This intentionally duplicates the equivalent `aspect-ratio`
/// walk rather than sharing it: this leaf keeps its own bounded local
/// recognition.
fn offset_rotate_top_level_components(items: &[CssLexicalItem]) -> Vec<&[CssLexicalItem]> {
    let mut components = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut current_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = current_start.take() {
                    components.push(&items[start..index]);
                }
                continue;
            }
        }

        if current_start.is_none() {
            current_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = current_start.take()
        {
            components.push(&items[start..=index]);
        }
    }

    if let Some(start) = current_start {
        components.push(&items[start..]);
    }

    components
}

/// Classifies one already-partitioned top-level `offset-rotate` component. A
/// Function-headed component is classified by name/placement only; its
/// interior is never parsed or evaluated. A direct component qualifies as
/// the `auto`/`reverse` keyword operand only when it is exactly one
/// ASCII-case-insensitive `Ident` token matching that keyword, and as the
/// direct `<angle>` operand only when it is exactly one retained `Dimension`
/// token whose decoded unit is ASCII-case-insensitively `deg`, `grad`,
/// `rad`, or `turn`. A unitless Number (including zero) never satisfies the
/// direct `<angle>` operand, unlike the accepted `<length>` unitless-zero
/// accommodation.
fn classify_offset_rotate_component(component: &[CssLexicalItem]) -> CssOffsetRotateComponentClass {
    if let Some(name) = entire_function_name(component) {
        return if is_whole_value_function(name) {
            CssOffsetRotateComponentClass::MisplacedWholeValueFunction
        } else {
            CssOffsetRotateComponentClass::ResidualFunction
        };
    }

    let mut tokens = component.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssOffsetRotateComponentClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssOffsetRotateComponentClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto") => {
            CssOffsetRotateComponentClass::KeywordAuto
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("reverse") => {
            CssOffsetRotateComponentClass::KeywordReverse
        }
        CssTokenKind::Dimension { unit, .. } if is_css_angle_unit(unit) => {
            CssOffsetRotateComponentClass::QualifiedAngle
        }
        _ => CssOffsetRotateComponentClass::Invalid,
    }
}

/// Qualifies one already-retained `offset-rotate` ordinary declaration value
/// window against the selected `[ auto | reverse ] || <angle>` profile.
///
/// Classification order is load-bearing: deferred/arbitrary substitution,
/// then whole-value Function, then whole-value CSS-wide keyword, then one
/// property-local depth-balanced component-grouping walk that identifies at
/// most one direct `auto`/`reverse` keyword component and at most one
/// direct `<angle>`-position component in either authored order. A decisive
/// direct structural failure — more than two top-level components, a
/// duplicate keyword, a duplicate angle-position component, or any
/// otherwise-Invalid or misplaced whole-value-Function component — always
/// wins over a residual Function occupying the angle position, which keeps
/// cases such as `auto reverse calc(45deg)` and `10deg calc(20deg)`
/// `InvalidForSelectedValueGrammar` rather than softened into
/// `FunctionValue` `Unsupported`.
fn qualify_offset_rotate_value(items: &[CssLexicalItem]) -> CssOffsetRotateQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssOffsetRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOffsetRotateUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssOffsetRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOffsetRotateUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssOffsetRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOffsetRotateUnsupportedReason::CssWideKeyword,
        );
    }

    let components = offset_rotate_top_level_components(items);

    if components.is_empty() || components.len() > 2 {
        return CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let classes: Vec<_> = components
        .iter()
        .map(|component| classify_offset_rotate_component(component))
        .collect();

    if classes.iter().any(|class| {
        matches!(
            class,
            CssOffsetRotateComponentClass::Invalid
                | CssOffsetRotateComponentClass::MisplacedWholeValueFunction
        )
    }) {
        return CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let keyword_classes: Vec<_> = classes
        .iter()
        .filter(|class| {
            matches!(
                class,
                CssOffsetRotateComponentClass::KeywordAuto
                    | CssOffsetRotateComponentClass::KeywordReverse
            )
        })
        .collect();
    let angle_classes: Vec<_> = classes
        .iter()
        .filter(|class| {
            matches!(
                class,
                CssOffsetRotateComponentClass::QualifiedAngle
                    | CssOffsetRotateComponentClass::ResidualFunction
            )
        })
        .collect();

    if keyword_classes.len() > 1 || angle_classes.len() > 1 {
        return CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if angle_classes
        .first()
        .is_some_and(|class| matches!(class, CssOffsetRotateComponentClass::ResidualFunction))
    {
        return CssOffsetRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOffsetRotateUnsupportedReason::FunctionValue,
        );
    }

    let has_direct_angle = !angle_classes.is_empty();
    match keyword_classes.first() {
        Some(CssOffsetRotateComponentClass::KeywordAuto) => {
            CssOffsetRotateQualificationOutcome::Qualified(if has_direct_angle {
                CssOffsetRotateValue::AutoAndDirectAngle
            } else {
                CssOffsetRotateValue::Auto
            })
        }
        Some(CssOffsetRotateComponentClass::KeywordReverse) => {
            CssOffsetRotateQualificationOutcome::Qualified(if has_direct_angle {
                CssOffsetRotateValue::ReverseAndDirectAngle
            } else {
                CssOffsetRotateValue::Reverse
            })
        }
        _ => {
            if has_direct_angle {
                CssOffsetRotateQualificationOutcome::Qualified(CssOffsetRotateValue::DirectAngle)
            } else {
                CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar
            }
        }
    }
}

fn animation_play_state_item_value(items: &[CssLexicalItem]) -> Option<CssAnimationPlayStateValue> {
    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let token = tokens.next()?;
    if tokens.next().is_some() {
        return None;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("running") => {
            Some(CssAnimationPlayStateValue::Running)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("paused") => {
            Some(CssAnimationPlayStateValue::Paused)
        }
        _ => None,
    }
}

/// Qualifies one retained `animation-play-state` declaration value
/// against `<single-animation-play-state>#`, where each item is
/// `running | paused`.
///
/// Deferred substitution is checked before list recognition because it
/// can change top-level separator structure. The list walk then splits
/// only on retained depth-zero `Comma` tokens. Commas inside Functions
/// or other balanced blocks remain inside the current item. Leading,
/// trailing, and consecutive top-level commas therefore produce empty
/// items and fail the selected grammar without any raw-source search or
/// reconstruction.
fn qualify_animation_play_state_value(
    items: &[CssLexicalItem],
) -> CssAnimationPlayStateQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssAnimationPlayStateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationPlayStateUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssAnimationPlayStateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationPlayStateUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssAnimationPlayStateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationPlayStateUnsupportedReason::CssWideKeyword,
        );
    }

    let mut values = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            let Some(value) = animation_play_state_item_value(&items[item_start..index]) else {
                return CssAnimationPlayStateQualificationOutcome::InvalidForSelectedValueGrammar;
            };
            values.push(value);
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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

    let Some(value) = animation_play_state_item_value(&items[item_start..]) else {
        return CssAnimationPlayStateQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    values.push(value);

    CssAnimationPlayStateQualificationOutcome::Qualified(values)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssAnimationIterationCountItemClass {
    Qualified(CssAnimationIterationCountValue),
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Classifies one already-comma-segmented `animation-iteration-count` list
/// item. A Function-headed item that consumes the entire item is classified
/// by name/placement only, exactly as the accepted `aspect-ratio` component
/// classifier does; its interior is never parsed or evaluated. A direct item
/// reuses the already accepted direct `<number [0,∞]>` boundary and the
/// direct `infinite` keyword comparison unchanged.
fn classify_animation_iteration_count_item(
    item: &[CssLexicalItem],
) -> CssAnimationIterationCountItemClass {
    if let Some(name) = entire_function_name(item) {
        return if is_whole_value_function(name) {
            CssAnimationIterationCountItemClass::MisplacedWholeValueFunction
        } else {
            CssAnimationIterationCountItemClass::ResidualFunction
        };
    }

    let mut tokens = item.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssAnimationIterationCountItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssAnimationIterationCountItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("infinite") => {
            CssAnimationIterationCountItemClass::Qualified(
                CssAnimationIterationCountValue::Infinite,
            )
        }
        CssTokenKind::Number { value, .. } if is_non_negative_direct_number(value) => {
            CssAnimationIterationCountItemClass::Qualified(
                CssAnimationIterationCountValue::DirectNumberLiteral,
            )
        }
        _ => CssAnimationIterationCountItemClass::Invalid,
    }
}

/// Qualifies one retained `animation-iteration-count` declaration value
/// against `<single-animation-iteration-count>#`, where each item is
/// `infinite | <number [0,∞]>`.
///
/// Deferred substitution and the whole-value Function/CSS-wide-keyword
/// boundaries are checked before list recognition, reusing the accepted
/// #571 top-level comma-list theorem unchanged: the list walk splits only on
/// retained depth-zero `Comma` tokens, and commas inside Functions or other
/// balanced blocks remain inside the current item.
///
/// Unlike `animation-play-state`, a list item here may be function-backed
/// (`<number>` allows `calc()`), so a Function-headed item is not
/// immediately decisive. Every item is classified first; a decisive
/// `Invalid`/`MisplacedWholeValueFunction` item anywhere in the list always
/// outranks a residual `FunctionValue` item, which in turn outranks a fully
/// `Qualified` list. This aggregation order is what keeps cases such as
/// `-1, calc(2)` and `calc(2), -1` `InvalidForSelectedValueGrammar` rather
/// than softened into `FunctionValue` `Unsupported`.
fn qualify_animation_iteration_count_value(
    items: &[CssLexicalItem],
) -> CssAnimationIterationCountQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssAnimationIterationCountQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationIterationCountUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssAnimationIterationCountQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationIterationCountUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssAnimationIterationCountQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationIterationCountUnsupportedReason::CssWideKeyword,
        );
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            item_classes.push(classify_animation_iteration_count_item(
                &items[item_start..index],
            ));
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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
    item_classes.push(classify_animation_iteration_count_item(
        &items[item_start..],
    ));

    if item_classes.iter().any(|class| {
        matches!(
            class,
            CssAnimationIterationCountItemClass::Invalid
                | CssAnimationIterationCountItemClass::MisplacedWholeValueFunction
        )
    }) {
        return CssAnimationIterationCountQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if item_classes
        .iter()
        .any(|class| matches!(class, CssAnimationIterationCountItemClass::ResidualFunction))
    {
        return CssAnimationIterationCountQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationIterationCountUnsupportedReason::FunctionValue,
        );
    }

    let values = item_classes
        .into_iter()
        .map(|class| match class {
            CssAnimationIterationCountItemClass::Qualified(value) => value,
            CssAnimationIterationCountItemClass::ResidualFunction
            | CssAnimationIterationCountItemClass::MisplacedWholeValueFunction
            | CssAnimationIterationCountItemClass::Invalid => {
                unreachable!("non-Qualified item classes are filtered above")
            }
        })
        .collect();

    CssAnimationIterationCountQualificationOutcome::Qualified(values)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssAnimationDelayItemClass {
    Qualified(CssAnimationDelayValue),
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Classifies one already-comma-segmented `animation-delay` list item. A
/// Function-headed item that consumes the entire item is classified by
/// name/placement only, exactly as the accepted `animation-iteration-count`
/// item classifier does; its interior is never parsed or evaluated. A direct
/// item qualifies only when it is exactly one non-trivia `Dimension` token
/// whose decoded unit is ASCII-case-insensitively `s` or `ms`; a unitless
/// Number (including zero) is deliberately not treated as a direct `<time>`
/// literal, unlike the accepted `<length>` unitless-zero accommodation.
fn classify_animation_delay_item(item: &[CssLexicalItem]) -> CssAnimationDelayItemClass {
    if let Some(name) = entire_function_name(item) {
        return if is_whole_value_function(name) {
            CssAnimationDelayItemClass::MisplacedWholeValueFunction
        } else {
            CssAnimationDelayItemClass::ResidualFunction
        };
    }

    let mut tokens = item.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssAnimationDelayItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssAnimationDelayItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Dimension { unit, .. } if is_css_time_unit(unit) => {
            CssAnimationDelayItemClass::Qualified(CssAnimationDelayValue::DirectTimeLiteral)
        }
        _ => CssAnimationDelayItemClass::Invalid,
    }
}

/// Qualifies one retained `animation-delay` declaration value against
/// `<time>#`, where each item is a direct `Dimension` token whose unit is
/// `s` or `ms`.
///
/// Deferred substitution and the whole-value Function/CSS-wide-keyword
/// boundaries are checked before list recognition, reusing the accepted
/// #571/#573 top-level comma-list theorem unchanged: the list walk splits
/// only on retained depth-zero `Comma` tokens, and commas inside Functions
/// or other balanced blocks remain inside the current item.
///
/// Like `animation-iteration-count`, a list item here may be function-backed
/// (`<time>` allows `calc()`), so a Function-headed item is not immediately
/// decisive. Every item is classified first; a decisive
/// `Invalid`/`MisplacedWholeValueFunction` item anywhere in the list always
/// outranks a residual `FunctionValue` item, which in turn outranks a fully
/// `Qualified` list. This aggregation order is what keeps cases such as
/// `1px, calc(2s)` and `calc(2s), 1px` `InvalidForSelectedValueGrammar`
/// rather than softened into `FunctionValue` `Unsupported`. `animation-delay`
/// permits negative time values, so no non-negative range check is applied.
fn qualify_animation_delay_value(
    items: &[CssLexicalItem],
) -> CssAnimationDelayQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssAnimationDelayQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationDelayUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssAnimationDelayQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationDelayUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssAnimationDelayQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationDelayUnsupportedReason::CssWideKeyword,
        );
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            item_classes.push(classify_animation_delay_item(&items[item_start..index]));
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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
    item_classes.push(classify_animation_delay_item(&items[item_start..]));

    if item_classes.iter().any(|class| {
        matches!(
            class,
            CssAnimationDelayItemClass::Invalid
                | CssAnimationDelayItemClass::MisplacedWholeValueFunction
        )
    }) {
        return CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if item_classes
        .iter()
        .any(|class| matches!(class, CssAnimationDelayItemClass::ResidualFunction))
    {
        return CssAnimationDelayQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationDelayUnsupportedReason::FunctionValue,
        );
    }

    let values = item_classes
        .into_iter()
        .map(|class| match class {
            CssAnimationDelayItemClass::Qualified(value) => value,
            CssAnimationDelayItemClass::ResidualFunction
            | CssAnimationDelayItemClass::MisplacedWholeValueFunction
            | CssAnimationDelayItemClass::Invalid => {
                unreachable!("non-Qualified item classes are filtered above")
            }
        })
        .collect();

    CssAnimationDelayQualificationOutcome::Qualified(values)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssTransitionDurationItemClass {
    Qualified(CssTransitionDurationValue),
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Classifies one already-comma-segmented `transition-duration` list item,
/// reusing the accepted `animation-delay` (#575) item-classification shape
/// unchanged except for the added non-negative range check. A direct item
/// qualifies only when it is exactly one non-trivia `Dimension` token whose
/// decoded unit is ASCII-case-insensitively `s` or `ms` and whose retained
/// numeric evidence satisfies the existing exact `is_non_negative_direct_number`
/// theorem; a direct negative non-zero time literal is therefore `Invalid`,
/// not merely out of range. A unitless Number (including zero) is not a
/// direct `<time>` literal, exactly as for `animation-delay`.
fn classify_transition_duration_item(item: &[CssLexicalItem]) -> CssTransitionDurationItemClass {
    if let Some(name) = entire_function_name(item) {
        return if is_whole_value_function(name) {
            CssTransitionDurationItemClass::MisplacedWholeValueFunction
        } else {
            CssTransitionDurationItemClass::ResidualFunction
        };
    }

    let mut tokens = item.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssTransitionDurationItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssTransitionDurationItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Dimension { value, unit, .. }
            if is_css_time_unit(unit) && is_non_negative_direct_number(value) =>
        {
            CssTransitionDurationItemClass::Qualified(CssTransitionDurationValue::DirectTimeLiteral)
        }
        _ => CssTransitionDurationItemClass::Invalid,
    }
}

/// Qualifies one retained `transition-duration` declaration value against
/// `<time [0s,∞]>#`, composing the accepted top-level comma-list theorem,
/// the accepted direct `<time>` Dimension theorem, and the existing exact
/// non-negative authored numeric range theorem.
///
/// Deferred substitution and the whole-value Function/CSS-wide-keyword
/// boundaries are checked before list recognition, reusing the accepted
/// #571/#573/#575 top-level comma-list theorem unchanged: the list walk
/// splits only on retained depth-zero `Comma` tokens, and commas inside
/// Functions or other balanced blocks remain inside the current item.
///
/// A list item here may be function-backed (`<time>` allows `calc()`), so a
/// Function-headed item is not immediately decisive. Every item is
/// classified first; a decisive `Invalid`/`MisplacedWholeValueFunction` item
/// anywhere in the list always outranks a residual `FunctionValue` item,
/// which in turn outranks a fully `Qualified` list. This aggregation order
/// keeps cases such as `-1s, calc(2s)` and `calc(2s), -1s`
/// `InvalidForSelectedValueGrammar` rather than softened into
/// `FunctionValue` `Unsupported`, while `calc(-1s)` alone (with no decisive
/// direct Invalid item elsewhere) remains `FunctionValue` `Unsupported`
/// because this slice never evaluates the Function to learn its result is
/// negative.
fn qualify_transition_duration_value(
    items: &[CssLexicalItem],
) -> CssTransitionDurationQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTransitionDurationQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransitionDurationUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTransitionDurationQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransitionDurationUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssTransitionDurationQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransitionDurationUnsupportedReason::CssWideKeyword,
        );
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            item_classes.push(classify_transition_duration_item(&items[item_start..index]));
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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
    item_classes.push(classify_transition_duration_item(&items[item_start..]));

    if item_classes.iter().any(|class| {
        matches!(
            class,
            CssTransitionDurationItemClass::Invalid
                | CssTransitionDurationItemClass::MisplacedWholeValueFunction
        )
    }) {
        return CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if item_classes
        .iter()
        .any(|class| matches!(class, CssTransitionDurationItemClass::ResidualFunction))
    {
        return CssTransitionDurationQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransitionDurationUnsupportedReason::FunctionValue,
        );
    }

    let values = item_classes
        .into_iter()
        .map(|class| match class {
            CssTransitionDurationItemClass::Qualified(value) => value,
            CssTransitionDurationItemClass::ResidualFunction
            | CssTransitionDurationItemClass::MisplacedWholeValueFunction
            | CssTransitionDurationItemClass::Invalid => {
                unreachable!("non-Qualified item classes are filtered above")
            }
        })
        .collect();

    CssTransitionDurationQualificationOutcome::Qualified(values)
}

enum CssTransitionPropertyItemClass {
    Qualified(
        CssTransitionPropertyItemValue,
        Option<CssTransitionPropertyCustomIdentEvidenceRef>,
    ),
    Invalid,
}

/// Classifies one already-comma-segmented `transition-property` list item.
///
/// Unlike `transition-duration`, this item grammar (`all | <custom-ident>`)
/// has no ordinary Function-backed value branch, so any item that is not
/// exactly one non-trivia direct Ident token -- including any Function-headed
/// item, whether an ordinary Function or a misplaced generic whole-value
/// Function -- is directly `Invalid`. `all` is matched ASCII-case-
/// insensitively as the predefined keyword item and never receives
/// custom-ident evidence. `none`, `default`, and CSS-wide keywords are
/// excluded from list-item position by CSS Transitions / CSS Values and are
/// therefore `Invalid` here, not merely `all`'s Unsupported whole-value
/// sibling. Every other direct Ident is a qualified `<custom-ident>` item
/// whose recognition-time evidence reference is the absolute retained
/// lexical-item index of the exact selected Ident token, reusing the `page`
/// (#578 / #418 comment 5580383097) ownership pattern: the reference is a
/// locator, not the tokenizer-owned decoded identity itself.
fn classify_transition_property_item(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssTransitionPropertyItemClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssTransitionPropertyItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssTransitionPropertyItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("all") => {
            CssTransitionPropertyItemClass::Qualified(CssTransitionPropertyItemValue::All, None)
        }
        CssTokenKind::Ident(identifier)
            if identifier.eq_ignore_ascii_case("none")
                || identifier.eq_ignore_ascii_case("default")
                || is_css_wide_keyword(identifier) =>
        {
            CssTransitionPropertyItemClass::Invalid
        }
        CssTokenKind::Ident(_) => CssTransitionPropertyItemClass::Qualified(
            CssTransitionPropertyItemValue::CustomIdent,
            Some(CssTransitionPropertyCustomIdentEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            }),
        ),
        _ => CssTransitionPropertyItemClass::Invalid,
    }
}

/// Qualifies one retained `transition-property` declaration value against
/// `none | [ all | <custom-ident> ]#`, composing the accepted top-level
/// comma-list theorem (#571/#573/#575/#577) with the accepted open-ended
/// custom-ident evidence-reference ownership theorem proven by `page`.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `page`/`transition-duration`; the depth-balanced
/// walk below never sees their nested fallback commas as outer separators.
/// A sole retained direct `none` Ident, ASCII-case-insensitively, qualifies
/// the dedicated whole-value branch and never reaches list segmentation. A
/// sole CSS-wide keyword preserves the existing whole-value Unsupported
/// boundary. Otherwise every top-level depth-zero-comma-delimited item is
/// classified independently; any decisive `Invalid` item anywhere in the
/// list makes the whole declaration `InvalidForSelectedValueGrammar`,
/// preserving exact authored order and duplicate occurrences in the
/// resulting item vector when every item qualifies. The returned evidence
/// vector is index-aligned with the qualified item vector: `Some` exactly
/// where the item is `CustomIdent`.
fn qualify_transition_property_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> (
    CssTransitionPropertyQualificationOutcome,
    Vec<Option<CssTransitionPropertyCustomIdentEvidenceRef>>,
) {
    if contains_deferred_substitution_function(items) {
        return (
            CssTransitionPropertyQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransitionPropertyUnsupportedReason::DeferredSubstitutionFunction,
            ),
            Vec::new(),
        );
    }

    if is_entire_whole_value_function(items) {
        return (
            CssTransitionPropertyQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransitionPropertyUnsupportedReason::WholeValueFunction,
            ),
            Vec::new(),
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return (
                CssTransitionPropertyQualificationOutcome::Qualified(
                    CssTransitionPropertyValue::None,
                ),
                Vec::new(),
            );
        }
        if is_css_wide_keyword(identifier) {
            return (
                CssTransitionPropertyQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTransitionPropertyUnsupportedReason::CssWideKeyword,
                ),
                Vec::new(),
            );
        }
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            item_classes.push(classify_transition_property_item(
                &items[item_start..index],
                lexical_item_start + item_start,
            ));
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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
    item_classes.push(classify_transition_property_item(
        &items[item_start..],
        lexical_item_start + item_start,
    ));

    if item_classes
        .iter()
        .any(|class| matches!(class, CssTransitionPropertyItemClass::Invalid))
    {
        return (
            CssTransitionPropertyQualificationOutcome::InvalidForSelectedValueGrammar,
            Vec::new(),
        );
    }

    let mut qualified_items = Vec::with_capacity(item_classes.len());
    let mut custom_ident_evidence = Vec::with_capacity(item_classes.len());
    for class in item_classes {
        match class {
            CssTransitionPropertyItemClass::Qualified(value, evidence) => {
                qualified_items.push(value);
                custom_ident_evidence.push(evidence);
            }
            CssTransitionPropertyItemClass::Invalid => {
                unreachable!("Invalid item classes are filtered above")
            }
        }
    }

    (
        CssTransitionPropertyQualificationOutcome::Qualified(CssTransitionPropertyValue::Items(
            qualified_items,
        )),
        custom_ident_evidence,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CssAnimationNameItemClass {
    Qualified(
        CssAnimationNameItemValue,
        Option<CssAnimationNameKeyframesNameEvidenceRef>,
    ),
    Invalid,
}

/// Classifies one already-comma-segmented `animation-name` list item.
///
/// Unlike `transition-property`, `none` is matched here as a repeated item
/// sentinel rather than a dedicated whole-value branch: an unquoted direct
/// Ident `none`, ASCII-case-insensitively, is always `None` regardless of its
/// position in the list. `default` and the CSS-wide keywords remain excluded
/// from list-item position, exactly as for `transition-property`. Every other
/// direct Ident is a qualified open-ended `<custom-ident>` `KeyframesName`
/// item. A quoted String never inherits `none`/keyword semantics from its
/// decoded contents -- `"none"`, `"initial"`, `"default"` are all
/// `KeyframesName` String items -- but an empty direct String is `Invalid`,
/// unlike the accepted `hyphenate-character` empty-String allowance; a
/// retained `BadString` is not `<string>` and is also `Invalid`. This item
/// grammar has no ordinary Function-backed branch, so any Function-headed
/// item, whether an ordinary Function or a misplaced generic whole-value
/// Function, is directly `Invalid` -- it never reaches this match because a
/// Function-headed item always retains more than the single non-trivia token
/// this grammar accepts. Every `KeyframesName` item's recognition-time
/// evidence reference is the absolute retained lexical-item index of the
/// exact selected Ident/String token, reusing the `page` /
/// `transition-property` ownership pattern: the reference is a locator, not
/// the tokenizer-owned decoded identity itself.
fn classify_animation_name_item(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssAnimationNameItemClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssAnimationNameItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssAnimationNameItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("none") => {
            CssAnimationNameItemClass::Qualified(CssAnimationNameItemValue::None, None)
        }
        CssTokenKind::Ident(identifier)
            if identifier.eq_ignore_ascii_case("default") || is_css_wide_keyword(identifier) =>
        {
            CssAnimationNameItemClass::Invalid
        }
        CssTokenKind::Ident(_) => CssAnimationNameItemClass::Qualified(
            CssAnimationNameItemValue::KeyframesName,
            Some(CssAnimationNameKeyframesNameEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            }),
        ),
        CssTokenKind::String(value) if !value.is_empty() => CssAnimationNameItemClass::Qualified(
            CssAnimationNameItemValue::KeyframesName,
            Some(CssAnimationNameKeyframesNameEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            }),
        ),
        _ => CssAnimationNameItemClass::Invalid,
    }
}

/// Qualifies one retained `animation-name` declaration value against
/// `[ none | <keyframes-name> ]#`, where
/// `<keyframes-name> = <custom-ident> | <string>` (#582 / #418 comment
/// 5581930292), composing the accepted top-level comma-list theorem
/// (#571/#573/#575/#577) with the accepted open-ended evidence-reference
/// ownership theorem proven by `page` / `transition-property` /
/// `hyphenate-character`.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `page`/`transition-property`; the depth-balanced
/// walk below never sees their nested fallback commas as outer separators. A
/// sole CSS-wide keyword preserves the existing whole-value Unsupported
/// boundary. Critically, unlike `transition-property`, `none` is deliberately
/// **not** special-cased as a sole-whole-value branch here: `[ none |
/// <keyframes-name> ]#` places `none` inside the repeated item grammar
/// itself, so `none`, `none, none`, and `foo, none` must all reach ordinary
/// list-item classification. Otherwise every top-level depth-zero-comma-
/// delimited item is classified independently; any decisive `Invalid` item
/// anywhere in the list makes the whole declaration
/// `InvalidForSelectedValueGrammar`, preserving exact authored order and
/// duplicate items -- including repeated `None` items and semantically equal
/// but lexically distinct `KeyframesName` items such as `foo` and `"foo"` --
/// in the resulting item vector when every item qualifies. The returned
/// evidence vector is index-aligned with the qualified item vector: `Some`
/// exactly where the item is `KeyframesName`.
fn qualify_animation_name_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> (
    CssAnimationNameQualificationOutcome,
    Vec<Option<CssAnimationNameKeyframesNameEvidenceRef>>,
) {
    if contains_deferred_substitution_function(items) {
        return (
            CssAnimationNameQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAnimationNameUnsupportedReason::DeferredSubstitutionFunction,
            ),
            Vec::new(),
        );
    }

    if is_entire_whole_value_function(items) {
        return (
            CssAnimationNameQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAnimationNameUnsupportedReason::WholeValueFunction,
            ),
            Vec::new(),
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return (
            CssAnimationNameQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAnimationNameUnsupportedReason::CssWideKeyword,
            ),
            Vec::new(),
        );
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            item_classes.push(classify_animation_name_item(
                &items[item_start..index],
                lexical_item_start + item_start,
            ));
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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
    item_classes.push(classify_animation_name_item(
        &items[item_start..],
        lexical_item_start + item_start,
    ));

    if item_classes
        .iter()
        .any(|class| matches!(class, CssAnimationNameItemClass::Invalid))
    {
        return (
            CssAnimationNameQualificationOutcome::InvalidForSelectedValueGrammar,
            Vec::new(),
        );
    }

    let mut qualified_items = Vec::with_capacity(item_classes.len());
    let mut keyframes_name_evidence = Vec::with_capacity(item_classes.len());
    for class in item_classes {
        match class {
            CssAnimationNameItemClass::Qualified(value, evidence) => {
                qualified_items.push(value);
                keyframes_name_evidence.push(evidence);
            }
            CssAnimationNameItemClass::Invalid => {
                unreachable!("Invalid item classes are filtered above")
            }
        }
    }

    (
        CssAnimationNameQualificationOutcome::Qualified(qualified_items),
        keyframes_name_evidence,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssAnchorNameItemClass {
    Qualified(CssAnchorNameDashedIdentEvidenceRef),
    Invalid,
}

/// Classifies one already-comma-segmented `anchor-name` list item.
///
/// An item qualifies iff, after trivia handling, it is exactly one direct
/// `Ident` token whose tokenizer-decoded identifier starts with `--`. This
/// is the interpreted decoded identity, never the raw authored spelling: an
/// escape-authored Ident that decodes to a leading `--` qualifies exactly
/// like an unescaped one. `none` and every CSS-wide keyword are Invalid here
/// without any dedicated exclusion arm, because their decoded identifiers do
/// not start with `--`; there is intentionally no special-cased `--none`
/// handling either, since it is an ordinary dashed-ident whose identity
/// happens to begin with `--`. A quoted `String`, even one decoding to text
/// beginning with `--`, is a different token class and is therefore Invalid
/// -- `Ident("--foo")` and `String("--foo")` are never conflated. This item
/// grammar has no ordinary Function-backed branch, so any Function-headed
/// item is directly `Invalid`. A qualified item's recognition-time evidence
/// reference is the absolute retained lexical-item index of the exact
/// selected Ident token, reusing the `page` / `transition-property` /
/// `animation-name` ownership pattern: the reference is a locator, not the
/// tokenizer-owned decoded identity itself.
fn classify_anchor_name_item(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssAnchorNameItemClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssAnchorNameItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssAnchorNameItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.starts_with("--") => {
            CssAnchorNameItemClass::Qualified(CssAnchorNameDashedIdentEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            })
        }
        _ => CssAnchorNameItemClass::Invalid,
    }
}

/// Qualifies one retained `anchor-name` declaration value against
/// `none | <dashed-ident>#` (#584 / #418 comment 5583004626), composing the
/// accepted top-level comma-list theorem (#571/#573/#575/#577) with the
/// accepted open-ended evidence-reference ownership theorem proven by
/// `page` / `transition-property` / `hyphenate-character` /
/// `animation-name`.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `page`/`transition-property`/`animation-name`; the
/// depth-balanced walk below never sees their nested fallback commas as
/// outer separators. A sole retained direct `none` Ident, ASCII-case-
/// insensitively, qualifies the dedicated whole-value branch and never
/// reaches list segmentation. A sole CSS-wide keyword preserves the existing
/// whole-value Unsupported boundary. Otherwise every top-level depth-zero-
/// comma-delimited item is classified independently; any decisive `Invalid`
/// item anywhere in the list makes the whole declaration
/// `InvalidForSelectedValueGrammar` -- critically, unlike `animation-name`,
/// `none` in list-item position is always `Invalid` here, since this
/// grammar places `none` only in the dedicated whole-value branch, never
/// inside the repeated item list. Exact authored order and duplicate
/// `<dashed-ident>` items are preserved in the resulting evidence vector
/// when every item qualifies.
fn qualify_anchor_name_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssAnchorNameQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssAnchorNameQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnchorNameUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssAnchorNameQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnchorNameUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssAnchorNameQualificationOutcome::Qualified(CssAnchorNameValue::None);
        }
        if is_css_wide_keyword(identifier) {
            return CssAnchorNameQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAnchorNameUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            item_classes.push(classify_anchor_name_item(
                &items[item_start..index],
                lexical_item_start + item_start,
            ));
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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
    item_classes.push(classify_anchor_name_item(
        &items[item_start..],
        lexical_item_start + item_start,
    ));

    if item_classes
        .iter()
        .any(|class| matches!(class, CssAnchorNameItemClass::Invalid))
    {
        return CssAnchorNameQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let names = item_classes
        .into_iter()
        .map(|class| match class {
            CssAnchorNameItemClass::Qualified(evidence) => evidence,
            CssAnchorNameItemClass::Invalid => {
                unreachable!("Invalid item classes are filtered above")
            }
        })
        .collect();

    CssAnchorNameQualificationOutcome::Qualified(CssAnchorNameValue::Names(names))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssContainerNameItemClass {
    Qualified(CssContainerNameCustomIdentEvidenceRef),
    Invalid,
}

/// Classifies one already-partitioned top-level `container-name`
/// `<custom-ident>+` component.
///
/// A component qualifies iff, after trivia handling, it is exactly one
/// direct `Ident` token whose tokenizer-decoded identifier is not `none`,
/// `and`, `not`, or `or` (ASCII-case-insensitively) -- the property-local
/// exclusions -- and not the generic `<custom-ident>` reserved identifier
/// `default` or a CSS-wide keyword. This is the interpreted decoded
/// identity, never the raw authored spelling: an escape-authored Ident
/// decodes through the tokenizer before this comparison runs. Unrelated
/// keywords such as `auto`/`normal` are ordinary qualified names. A
/// component with more than one non-trivia token -- including a Function,
/// a bracketed construct, or a stray `Comma` sharing a component with a
/// neighboring token -- is directly `Invalid`, since this item grammar has
/// no ordinary Function-backed branch and this leaf is delimiter-free `+`
/// repetition, never `#` comma-list repetition. A quoted `String` is a
/// different token class than `Ident` and is therefore also `Invalid`. A
/// qualified item's recognition-time evidence reference is the absolute
/// retained lexical-item index of the exact selected Ident token, reusing
/// the `page` / `transition-property` / `animation-name` / `anchor-name`
/// ownership pattern: the reference is a locator, not the tokenizer-owned
/// decoded identity itself.
fn classify_container_name_item(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssContainerNameItemClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssContainerNameItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssContainerNameItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier)
            if identifier.eq_ignore_ascii_case("none")
                || identifier.eq_ignore_ascii_case("and")
                || identifier.eq_ignore_ascii_case("not")
                || identifier.eq_ignore_ascii_case("or")
                || identifier.eq_ignore_ascii_case("default")
                || is_css_wide_keyword(identifier) =>
        {
            CssContainerNameItemClass::Invalid
        }
        CssTokenKind::Ident(_) => {
            CssContainerNameItemClass::Qualified(CssContainerNameCustomIdentEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            })
        }
        _ => CssContainerNameItemClass::Invalid,
    }
}

/// Qualifies one retained `container-name` declaration value against
/// `none | <custom-ident>+` (#588 / #418 comment 5584554364), composing the
/// accepted delimiter-free top-level-component theorem (`border-spacing` /
/// `offset-rotate`) with the accepted open-ended evidence-reference
/// ownership theorem proven by `page` / `transition-property` /
/// `hyphenate-character` / `animation-name` / `anchor-name`.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `anchor-name`/`offset-rotate`. A sole retained
/// direct `none` Ident, ASCII-case-insensitively, qualifies the dedicated
/// whole-value branch and never reaches component recognition -- `none` is
/// deliberately never a repeated-item sentinel here, unlike
/// `animation-name`. A sole CSS-wide keyword preserves the existing
/// whole-value Unsupported boundary. Otherwise this single left-to-right
/// recognition-time pass partitions the value into ordered top-level
/// components using depth-zero Whitespace/Comment trivia as separators --
/// never raw-source whitespace splitting, since tokenization already
/// supplies component boundaries -- and classifies each component the
/// instant its block depth returns to zero, so a bare `Comma` (this grammar
/// is `+`, not `#`) never behaves as a permitted separator: it either
/// starts its own single-token `Invalid` component or shares a component
/// with a neighboring token, which then fails the single-token
/// `<custom-ident>` test. Any decisive `Invalid` component anywhere makes
/// the whole declaration `InvalidForSelectedValueGrammar`, and an empty
/// component sequence -- the empty value, or a value that is only trivia --
/// is also `InvalidForSelectedValueGrammar`, since `+` requires at least
/// one component. Exact authored order and duplicate `<custom-ident>` items
/// are preserved in the resulting evidence vector when every component
/// qualifies.
fn qualify_container_name_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssContainerNameQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssContainerNameQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssContainerNameUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssContainerNameQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssContainerNameUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssContainerNameQualificationOutcome::Qualified(CssContainerNameValue::None);
        }
        if is_css_wide_keyword(identifier) {
            return CssContainerNameQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssContainerNameUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = item_start.take() {
                    item_classes.push(classify_container_name_item(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if item_start.is_none() {
            item_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = item_start.take()
        {
            item_classes.push(classify_container_name_item(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }
    if let Some(start) = item_start {
        item_classes.push(classify_container_name_item(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    if item_classes.is_empty() {
        return CssContainerNameQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if item_classes
        .iter()
        .any(|class| matches!(class, CssContainerNameItemClass::Invalid))
    {
        return CssContainerNameQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let names = item_classes
        .into_iter()
        .map(|class| match class {
            CssContainerNameItemClass::Qualified(evidence) => evidence,
            CssContainerNameItemClass::Invalid => {
                unreachable!("Invalid item classes are filtered above")
            }
        })
        .collect();

    CssContainerNameQualificationOutcome::Qualified(CssContainerNameValue::Names(names))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssCounterIncrementComponentClass {
    Name(CssCounterIncrementNameEvidenceRef),
    Integer(CssCounterIncrementIntegerEvidenceRef),
    EmbeddedWholeValueFunction,
    ArbitraryFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `counter-increment`
/// component against the direct-authored profile's two required token
/// shapes -- a direct `<counter-name>` `Ident` or a direct `Integer`-typed
/// `Number` -- plus the Function-position boundary from #592 / #418
/// comment 5593320578.
///
/// A component headed by a `Function` token is never a valid name or
/// integer; it is `EmbeddedWholeValueFunction` when its name is one of the
/// recognized generic whole-value-only functions (`is_whole_value_function`)
/// occupying a non-whole-value position -- decisively invalid there, since
/// whole-value syntax has no independent meaning when embedded inside the
/// structured list -- or `ArbitraryFunction` for any other Function,
/// preserving the conservative open envelope for a structurally feasible
/// optional-integer slot without evaluating it. A component with more than
/// one non-trivia token that is not Function-headed -- including a stray
/// top-level `Comma` sharing a component with a neighboring token, since
/// this grammar is `+`, never `#` comma-list repetition -- is directly
/// `Invalid`. `none`, `default`, and CSS-wide keywords are reserved out of
/// `<counter-name>` at this position; unrelated keywords such as
/// `auto`/`normal`/`and`/`not`/`or` remain ordinary qualified names, since
/// this property does not import `container-name`'s additional exclusions.
fn classify_counter_increment_component(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssCounterIncrementComponentClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssCounterIncrementComponentClass::Invalid;
    };

    if matches!(token.kind(), CssTokenKind::Function(_)) {
        return match entire_function_name(item) {
            Some(name) if is_whole_value_function(name) => {
                CssCounterIncrementComponentClass::EmbeddedWholeValueFunction
            }
            Some(_) => CssCounterIncrementComponentClass::ArbitraryFunction,
            None => CssCounterIncrementComponentClass::Invalid,
        };
    }

    if tokens.next().is_some() {
        return CssCounterIncrementComponentClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier)
            if identifier.eq_ignore_ascii_case("none")
                || identifier.eq_ignore_ascii_case("default")
                || is_css_wide_keyword(identifier) =>
        {
            CssCounterIncrementComponentClass::Invalid
        }
        CssTokenKind::Ident(_) => {
            CssCounterIncrementComponentClass::Name(CssCounterIncrementNameEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            })
        }
        CssTokenKind::Number {
            number_type: CssNumberType::Integer,
            ..
        } => CssCounterIncrementComponentClass::Integer(CssCounterIncrementIntegerEvidenceRef {
            lexical_item_index: absolute_item_start + relative_index,
        }),
        _ => CssCounterIncrementComponentClass::Invalid,
    }
}

/// Sequentially groups already-classified top-level `counter-increment`
/// components into ordered `DirectCounterItem`s per the exact algorithm in
/// #592 / #418 comment 5593320578.
///
/// Each item requires a direct `<counter-name>` component; a Function can
/// never satisfy this required position, so it is directly
/// `InvalidForSelectedValueGrammar` there, regardless of whether the
/// Function is otherwise `ArbitraryFunction` or
/// `EmbeddedWholeValueFunction`. Once a name is recognized, the following
/// component (if any) is inspected: a direct `Integer` attaches as that
/// item's explicit authored integer and advances past it; another valid
/// `<counter-name>` leaves the current item's integer authored-absent and
/// is re-examined as the next item's required name (no advance); an
/// `ArbitraryFunction` is a provisional feasible-integer-slot ambiguity --
/// consumed as belonging to the current item and resolved to
/// `UnsupportedBySelectedValueProfile(FunctionValuedIntegerSlot)` only if
/// no later component makes the declaration decisively invalid regardless
/// of what the Function computes to; any other component (a wrong token
/// class, or an `EmbeddedWholeValueFunction`) is directly decisive
/// `InvalidForSelectedValueGrammar`, since whole-value-only function syntax
/// and non-name/non-integer tokens have no meaning in this position either.
/// Decisive invalidity anywhere always outranks a provisional Function
/// ambiguity found earlier.
fn group_counter_increment_components(
    components: &[CssCounterIncrementComponentClass],
) -> CssCounterIncrementQualificationOutcome {
    let mut items = Vec::new();
    let mut has_function_ambiguity = false;
    let mut index = 0;

    while index < components.len() {
        let name = match components[index] {
            CssCounterIncrementComponentClass::Name(name) => name,
            _ => return CssCounterIncrementQualificationOutcome::InvalidForSelectedValueGrammar,
        };
        index += 1;

        if index >= components.len() {
            items.push(CssCounterIncrementItem {
                name,
                explicit_integer: None,
            });
            break;
        }

        match components[index] {
            CssCounterIncrementComponentClass::Integer(integer) => {
                items.push(CssCounterIncrementItem {
                    name,
                    explicit_integer: Some(integer),
                });
                index += 1;
            }
            CssCounterIncrementComponentClass::Name(_) => {
                items.push(CssCounterIncrementItem {
                    name,
                    explicit_integer: None,
                });
                // Do not advance: this component starts the next item.
            }
            CssCounterIncrementComponentClass::ArbitraryFunction => {
                has_function_ambiguity = true;
                index += 1;
            }
            CssCounterIncrementComponentClass::EmbeddedWholeValueFunction
            | CssCounterIncrementComponentClass::Invalid => {
                return CssCounterIncrementQualificationOutcome::InvalidForSelectedValueGrammar;
            }
        }
    }

    if has_function_ambiguity {
        CssCounterIncrementQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterIncrementUnsupportedReason::FunctionValuedIntegerSlot,
        )
    } else {
        CssCounterIncrementQualificationOutcome::Qualified(CssCounterIncrementValue::Items(items))
    }
}

/// Qualifies one retained `counter-increment` declaration value against the
/// narrowed direct-authored profile `none | DirectCounterItem+` (#592 /
/// #418 comment 5593320578).
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `container-name`/`color-scheme`. A sole retained
/// direct `none` Ident, ASCII-case-insensitively, qualifies the dedicated
/// whole-value branch and never reaches component recognition -- `none` is
/// never a repeated-item sentinel here. A sole CSS-wide keyword preserves
/// the existing whole-value Unsupported boundary. Otherwise this single
/// left-to-right recognition-time pass partitions the value into ordered
/// top-level components using depth-zero Whitespace/Comment trivia as
/// separators -- never raw-source whitespace splitting -- classifies each
/// component the instant its block depth returns to zero (reusing the
/// `container-name`/`color-scheme` delimiter-free partitioning theorem so a
/// bare `Comma` never behaves as a permitted separator), and then
/// sequentially groups the classified components into ordered
/// `DirectCounterItem`s via `group_counter_increment_components`. An empty
/// component sequence -- the empty value, or a value that is only trivia --
/// is `InvalidForSelectedValueGrammar`, since `+` requires at least one
/// component.
fn qualify_counter_increment_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssCounterIncrementQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssCounterIncrementQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterIncrementUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssCounterIncrementQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterIncrementUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssCounterIncrementQualificationOutcome::Qualified(
                CssCounterIncrementValue::None,
            );
        }
        if is_css_wide_keyword(identifier) {
            return CssCounterIncrementQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCounterIncrementUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut component_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = item_start.take() {
                    component_classes.push(classify_counter_increment_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if item_start.is_none() {
            item_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = item_start.take()
        {
            component_classes.push(classify_counter_increment_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }
    if let Some(start) = item_start {
        component_classes.push(classify_counter_increment_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    if component_classes.is_empty() {
        return CssCounterIncrementQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    group_counter_increment_components(&component_classes)
}

/// Classifies the direct inner grammar of one grammar-native
/// `reversed(...)` Function component -- `reversed(<counter-name>)` per
/// #594 / #418 comment 5595916114 -- locating the exact tokenizer-owned
/// inner `Ident` when, after grammar trivia (`Comment`/`Whitespace`) is
/// discounted, the content strictly inside the outer Function's own
/// matching parenthesis -- or, at true stylesheet EOF, everything after
/// the Function opener when no matching closing parenthesis is ever
/// retained -- is exactly one `Ident` token satisfying `<counter-name>`
/// (excluding `none`, `default`, and CSS-wide keywords,
/// ASCII-case-insensitively). Any nested bracket, extra token, or wrong
/// token kind inside the parens necessarily changes this count away from
/// exactly one and is therefore rejected here without being separately
/// traversed for a nested name. Depth is tracked over the already
/// depth-zero-partitioned component span passed in by the caller -- never
/// by re-scanning raw source text or retokenizing.
fn reversed_inner_name_evidence(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> Option<CssCounterResetNameEvidenceRef> {
    let mut block_stack = vec![CssValueBlockCloser::Parenthesis];
    let mut inner_tokens: Vec<(usize, &CssToken)> = Vec::new();

    for (relative_index, entry) in item.iter().enumerate().skip(1) {
        let CssLexicalItem::SemanticToken(token) = entry else {
            // Comment trivia inside the function body carries no semantic
            // content.
            continue;
        };

        if matches!(token.kind(), CssTokenKind::Whitespace) {
            continue;
        }

        let closes_outer = matches!(token.kind(), CssTokenKind::RightParenthesis)
            && block_stack.len() == 1
            && block_stack.last() == Some(&CssValueBlockCloser::Parenthesis);

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

        if !closes_outer {
            inner_tokens.push((relative_index, token));
        }
    }

    let mut inner_iter = inner_tokens.into_iter();
    let (relative_index, token) = inner_iter.next()?;
    if inner_iter.next().is_some() {
        return None;
    }

    let CssTokenKind::Ident(identifier) = token.kind() else {
        return None;
    };

    if identifier.eq_ignore_ascii_case("none")
        || identifier.eq_ignore_ascii_case("default")
        || is_css_wide_keyword(identifier)
    {
        return None;
    }

    Some(CssCounterResetNameEvidenceRef {
        lexical_item_index: absolute_item_start + relative_index,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssCounterResetComponentClass {
    Name(CssCounterResetName),
    Integer(CssCounterResetIntegerEvidenceRef),
    EmbeddedWholeValueFunction,
    ArbitraryFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `counter-reset` component
/// against the direct-authored profile's three required token shapes -- a
/// direct `<counter-name>` `Ident`, a direct `Integer`-typed `Number`, or a
/// grammar-native `reversed(<counter-name>)` Function -- plus the
/// Function-position boundary from `counter-increment` (#592 / #418
/// comment 5593320578), reused unchanged (#594 / #418 comment
/// 5595916114).
///
/// A component headed by a `Function` token whose decoded name matches
/// `reversed` ASCII-case-insensitively is inspected by
/// `reversed_inner_name_evidence`: when its direct inner grammar is
/// exactly one valid `<counter-name>` `Ident`, the component is a
/// qualified `Name(Reversed(_))`; otherwise -- including an empty,
/// multi-token, nested-Function, or reserved-identifier body -- it falls
/// through to the same generic, unevaluated `ArbitraryFunction` treatment
/// as any other unrecognized Function, never a decisive Invalid by itself,
/// since the grouping pass already makes an unrecognized-shape Function
/// decisively Invalid at a required name position and a provisional
/// ambiguity at an optional-integer position. A component headed by a
/// Function whose name is one of the recognized generic whole-value-only
/// functions (`is_whole_value_function`) is `EmbeddedWholeValueFunction`.
/// A component with more than one non-trivia token that is not
/// Function-headed -- including a stray top-level `Comma` sharing a
/// component with a neighboring token, since this grammar is `+`, never
/// `#` comma-list repetition -- is directly `Invalid`. `none`, `default`,
/// and CSS-wide keywords are reserved out of `<counter-name>` at this
/// position; unrelated keywords such as `auto`/`normal`/`and`/`not`/`or`
/// remain ordinary qualified names, since this property does not import
/// `container-name`'s additional exclusions.
fn classify_counter_reset_component(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssCounterResetComponentClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssCounterResetComponentClass::Invalid;
    };

    if matches!(token.kind(), CssTokenKind::Function(_)) {
        return match entire_function_name(item) {
            Some(name) if name.eq_ignore_ascii_case("reversed") => {
                match reversed_inner_name_evidence(item, absolute_item_start) {
                    Some(evidence) => {
                        CssCounterResetComponentClass::Name(CssCounterResetName::Reversed(evidence))
                    }
                    None => CssCounterResetComponentClass::ArbitraryFunction,
                }
            }
            Some(name) if is_whole_value_function(name) => {
                CssCounterResetComponentClass::EmbeddedWholeValueFunction
            }
            Some(_) => CssCounterResetComponentClass::ArbitraryFunction,
            None => CssCounterResetComponentClass::Invalid,
        };
    }

    if tokens.next().is_some() {
        return CssCounterResetComponentClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier)
            if identifier.eq_ignore_ascii_case("none")
                || identifier.eq_ignore_ascii_case("default")
                || is_css_wide_keyword(identifier) =>
        {
            CssCounterResetComponentClass::Invalid
        }
        CssTokenKind::Ident(_) => CssCounterResetComponentClass::Name(CssCounterResetName::Direct(
            CssCounterResetNameEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            },
        )),
        CssTokenKind::Number {
            number_type: CssNumberType::Integer,
            ..
        } => CssCounterResetComponentClass::Integer(CssCounterResetIntegerEvidenceRef {
            lexical_item_index: absolute_item_start + relative_index,
        }),
        _ => CssCounterResetComponentClass::Invalid,
    }
}

/// Sequentially groups already-classified top-level `counter-reset`
/// components into ordered `CounterResetItem`s, reusing the exact
/// `counter-increment` algorithm unchanged (#592 / #418 comment
/// 5593320578) over the widened `Name` classification that now also
/// carries the grammar-native `Reversed` branch (#594 / #418 comment
/// 5595916114).
///
/// Each item requires a direct `<counter-name>` `Ident` or a valid
/// `reversed(<counter-name>)` Function; any other component -- including
/// an unrecognized-shape Function, since a Function can never satisfy this
/// required position regardless of what it might evaluate to -- is
/// directly `InvalidForSelectedValueGrammar` there. Once a name is
/// recognized, the following component (if any) is inspected: a direct
/// `Integer` attaches as that item's explicit authored integer and
/// advances past it; another valid name (`Direct` or `Reversed`) leaves
/// the current item's integer authored-absent and is re-examined as the
/// next item's required name (no advance); an `ArbitraryFunction` is a
/// provisional feasible-integer-slot ambiguity -- consumed as belonging to
/// the current item and resolved to
/// `UnsupportedBySelectedValueProfile(FunctionValuedIntegerSlot)` only if
/// no later component makes the declaration decisively invalid regardless
/// of what the Function computes to; any other component (a wrong token
/// class, or an `EmbeddedWholeValueFunction`) is directly decisive
/// `InvalidForSelectedValueGrammar`. Decisive invalidity anywhere always
/// outranks a provisional Function ambiguity found earlier.
fn group_counter_reset_components(
    components: &[CssCounterResetComponentClass],
) -> CssCounterResetQualificationOutcome {
    let mut items = Vec::new();
    let mut has_function_ambiguity = false;
    let mut index = 0;

    while index < components.len() {
        let name = match components[index] {
            CssCounterResetComponentClass::Name(name) => name,
            _ => return CssCounterResetQualificationOutcome::InvalidForSelectedValueGrammar,
        };
        index += 1;

        if index >= components.len() {
            items.push(CssCounterResetItem {
                name,
                explicit_integer: None,
            });
            break;
        }

        match components[index] {
            CssCounterResetComponentClass::Integer(integer) => {
                items.push(CssCounterResetItem {
                    name,
                    explicit_integer: Some(integer),
                });
                index += 1;
            }
            CssCounterResetComponentClass::Name(_) => {
                items.push(CssCounterResetItem {
                    name,
                    explicit_integer: None,
                });
                // Do not advance: this component starts the next item.
            }
            CssCounterResetComponentClass::ArbitraryFunction => {
                has_function_ambiguity = true;
                index += 1;
            }
            CssCounterResetComponentClass::EmbeddedWholeValueFunction
            | CssCounterResetComponentClass::Invalid => {
                return CssCounterResetQualificationOutcome::InvalidForSelectedValueGrammar;
            }
        }
    }

    if has_function_ambiguity {
        CssCounterResetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterResetUnsupportedReason::FunctionValuedIntegerSlot,
        )
    } else {
        CssCounterResetQualificationOutcome::Qualified(CssCounterResetValue::Items(items))
    }
}

/// Qualifies one retained `counter-reset` declaration value against the
/// narrowed direct-authored profile `none | CounterResetItem+` where
/// `CounterResetItem := CssCounterResetName <integer>?` and
/// `CssCounterResetName := <counter-name> | reversed(<counter-name>)`
/// (#594 / #418 comment 5595916114), composing the accepted
/// `counter-increment` partitioning/grouping theorem with the new
/// grammar-native `reversed` Function branch.
///
/// Deferred substitution and the whole-value Function boundary are
/// checked first, exactly as for `counter-increment`, scanning the entire
/// flat retained token stream regardless of nesting depth -- this already
/// covers `reversed(var(--x))` and `reversed(var(--x), foo)` without any
/// reversed()-specific handling, since the existing any-occurrence
/// preflight does not distinguish top-level from nested Function
/// occurrences. A sole retained direct `none` Ident, ASCII-case-
/// insensitively, qualifies the dedicated whole-value branch and never
/// reaches component recognition -- `none` is never a repeated-item
/// sentinel here. A sole CSS-wide keyword preserves the existing
/// whole-value Unsupported boundary. Otherwise this single left-to-right
/// recognition-time pass partitions the value into ordered top-level
/// components using depth-zero Whitespace/Comment trivia as separators --
/// never raw-source whitespace splitting -- classifies each component the
/// instant its block depth returns to zero (reusing the
/// `counter-increment` delimiter-free partitioning theorem so a bare
/// `Comma` never behaves as a permitted separator, and so a true
/// stylesheet EOF inside an unclosed `reversed(...)` still yields one
/// terminal component from parser-owned retained structure, never from a
/// blanket missing-close acceptance), and then sequentially groups the
/// classified components into ordered `CounterResetItem`s via
/// `group_counter_reset_components`. An empty component sequence -- the
/// empty value, or a value that is only trivia -- is
/// `InvalidForSelectedValueGrammar`, since `+` requires at least one
/// component.
fn qualify_counter_reset_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssCounterResetQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssCounterResetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterResetUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssCounterResetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterResetUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssCounterResetQualificationOutcome::Qualified(CssCounterResetValue::None);
        }
        if is_css_wide_keyword(identifier) {
            return CssCounterResetQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCounterResetUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut component_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = item_start.take() {
                    component_classes.push(classify_counter_reset_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if item_start.is_none() {
            item_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = item_start.take()
        {
            component_classes.push(classify_counter_reset_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }
    if let Some(start) = item_start {
        component_classes.push(classify_counter_reset_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    if component_classes.is_empty() {
        return CssCounterResetQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    group_counter_reset_components(&component_classes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssCounterSetComponentClass {
    Name(CssCounterSetNameEvidenceRef),
    Integer(CssCounterSetIntegerEvidenceRef),
    EmbeddedWholeValueFunction,
    ReversedFunction,
    ArbitraryFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `counter-set` component
/// against the direct-authored profile's two required token shapes -- a
/// direct `<counter-name>` `Ident` or a direct `Integer`-typed `Number` --
/// plus the Function-position boundary transferred unchanged from
/// `counter-increment` (#592 / #600). Unlike `counter-reset`, no
/// `reversed(<counter-name>)` grammar branch is recognized here:
/// `counter-set` has no such branch, and current WPT explicitly rejects
/// `counter-set: reversed(chapter)`. Per #600's `reversed()` boundary,
/// `reversed(...)` is nonetheless a *known* CSS Lists grammar-native
/// Function name -- not an unresolved arbitrary Function whose result
/// might one day satisfy `<integer>` -- so it is classified into its own
/// `ReversedFunction` class, decisively invalid at every placement,
/// distinct from the conservative open-ended `ArbitraryFunction` envelope
/// used for a genuinely unknown Function such as `calc(...)`.
///
/// A component headed by a `Function` token is never a valid name or
/// integer; it is `ReversedFunction` when its decoded name matches
/// `reversed` ASCII-case-insensitively (regardless of the Function's inner
/// content, since no `reversed(<counter-name>)` grammar is recognized here
/// to validate against), `EmbeddedWholeValueFunction` when its name is one
/// of the recognized generic whole-value-only functions
/// (`is_whole_value_function`) occupying a non-whole-value position --
/// decisively invalid there, since whole-value syntax has no independent
/// meaning when embedded inside the structured list -- or
/// `ArbitraryFunction` for any other Function, preserving the conservative
/// open envelope for a structurally feasible optional-integer slot without
/// evaluating it. A component with more than one non-trivia token that is
/// not Function-headed -- including a stray top-level `Comma` sharing a
/// component with a neighboring token, since this grammar is `+`, never `#`
/// comma-list repetition -- is directly `Invalid`. `none`, `default`, and
/// CSS-wide keywords are reserved out of `<counter-name>` at this position;
/// unrelated keywords such as `auto`/`normal`/`and`/`not`/`or` remain
/// ordinary qualified names, since this property does not import
/// `container-name`'s additional exclusions.
fn classify_counter_set_component(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssCounterSetComponentClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssCounterSetComponentClass::Invalid;
    };

    if matches!(token.kind(), CssTokenKind::Function(_)) {
        return match entire_function_name(item) {
            Some(name) if name.eq_ignore_ascii_case("reversed") => {
                CssCounterSetComponentClass::ReversedFunction
            }
            Some(name) if is_whole_value_function(name) => {
                CssCounterSetComponentClass::EmbeddedWholeValueFunction
            }
            Some(_) => CssCounterSetComponentClass::ArbitraryFunction,
            None => CssCounterSetComponentClass::Invalid,
        };
    }

    if tokens.next().is_some() {
        return CssCounterSetComponentClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier)
            if identifier.eq_ignore_ascii_case("none")
                || identifier.eq_ignore_ascii_case("default")
                || is_css_wide_keyword(identifier) =>
        {
            CssCounterSetComponentClass::Invalid
        }
        CssTokenKind::Ident(_) => CssCounterSetComponentClass::Name(CssCounterSetNameEvidenceRef {
            lexical_item_index: absolute_item_start + relative_index,
        }),
        CssTokenKind::Number {
            number_type: CssNumberType::Integer,
            ..
        } => CssCounterSetComponentClass::Integer(CssCounterSetIntegerEvidenceRef {
            lexical_item_index: absolute_item_start + relative_index,
        }),
        _ => CssCounterSetComponentClass::Invalid,
    }
}

/// Sequentially groups already-classified top-level `counter-set`
/// components into ordered `DirectCounterItem`s, reusing the exact
/// `counter-increment` algorithm unchanged (#592 / #600) over the widened
/// classification that separates `ReversedFunction` from the generic
/// `ArbitraryFunction` envelope.
///
/// Each item requires a direct `<counter-name>` component; a Function can
/// never satisfy this required position, so it is directly
/// `InvalidForSelectedValueGrammar` there, regardless of whether the
/// Function is otherwise `ArbitraryFunction`, `EmbeddedWholeValueFunction`,
/// or `ReversedFunction`. Once a name is recognized, the following
/// component (if any) is inspected: a direct `Integer` attaches as that
/// item's explicit authored integer and advances past it; another valid
/// `<counter-name>` leaves the current item's integer authored-absent and
/// is re-examined as the next item's required name (no advance); an
/// `ArbitraryFunction` is a provisional feasible-integer-slot ambiguity --
/// consumed as belonging to the current item and resolved to
/// `UnsupportedBySelectedValueProfile(FunctionValuedIntegerSlot)` only if
/// no later component makes the declaration decisively invalid regardless
/// of what the Function computes to; any other component (a wrong token
/// class, an `EmbeddedWholeValueFunction`, or a `ReversedFunction`) is
/// directly decisive `InvalidForSelectedValueGrammar` -- unlike an
/// unresolved `ArbitraryFunction`, `reversed(...)` is a known CSS Lists
/// grammar-native Function with no meaning anywhere in `counter-set`'s
/// grammar, so it is never given the conservative open-ended integer-slot
/// treatment, even when it occupies a structurally feasible optional
/// position. Decisive invalidity anywhere always outranks a provisional
/// Function ambiguity found earlier.
fn group_counter_set_components(
    components: &[CssCounterSetComponentClass],
) -> CssCounterSetQualificationOutcome {
    let mut items = Vec::new();
    let mut has_function_ambiguity = false;
    let mut index = 0;

    while index < components.len() {
        let name = match components[index] {
            CssCounterSetComponentClass::Name(name) => name,
            _ => return CssCounterSetQualificationOutcome::InvalidForSelectedValueGrammar,
        };
        index += 1;

        if index >= components.len() {
            items.push(CssCounterSetItem {
                name,
                explicit_integer: None,
            });
            break;
        }

        match components[index] {
            CssCounterSetComponentClass::Integer(integer) => {
                items.push(CssCounterSetItem {
                    name,
                    explicit_integer: Some(integer),
                });
                index += 1;
            }
            CssCounterSetComponentClass::Name(_) => {
                items.push(CssCounterSetItem {
                    name,
                    explicit_integer: None,
                });
                // Do not advance: this component starts the next item.
            }
            CssCounterSetComponentClass::ArbitraryFunction => {
                has_function_ambiguity = true;
                index += 1;
            }
            CssCounterSetComponentClass::ReversedFunction
            | CssCounterSetComponentClass::EmbeddedWholeValueFunction
            | CssCounterSetComponentClass::Invalid => {
                return CssCounterSetQualificationOutcome::InvalidForSelectedValueGrammar;
            }
        }
    }

    if has_function_ambiguity {
        CssCounterSetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterSetUnsupportedReason::FunctionValuedIntegerSlot,
        )
    } else {
        CssCounterSetQualificationOutcome::Qualified(CssCounterSetValue::Items(items))
    }
}

/// Qualifies one retained `counter-set` declaration value against the
/// narrowed direct-authored profile `none | DirectCounterItem+` (#600),
/// transferring the `counter-increment` partitioning/grouping theorem
/// (#592 / #593) unchanged. `counter-increment`, not `counter-reset`, is
/// the syntactic/mechanical transfer source, since `counter-set` has no
/// `reversed(<counter-name>)` branch.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `counter-increment`. A sole retained direct `none`
/// Ident, ASCII-case-insensitively, qualifies the dedicated whole-value
/// branch and never reaches component recognition -- `none` is never a
/// repeated-item sentinel here. A sole CSS-wide keyword preserves the
/// existing whole-value Unsupported boundary. Otherwise this single
/// left-to-right recognition-time pass partitions the value into ordered
/// top-level components using depth-zero Whitespace/Comment trivia as
/// separators -- never raw-source whitespace splitting -- classifies each
/// component the instant its block depth returns to zero (reusing the
/// `counter-increment` delimiter-free partitioning theorem so a bare
/// `Comma` never behaves as a permitted separator), and then sequentially
/// groups the classified components into ordered `DirectCounterItem`s via
/// `group_counter_set_components`. An empty component sequence -- the empty
/// value, or a value that is only trivia -- is
/// `InvalidForSelectedValueGrammar`, since `+` requires at least one
/// component.
fn qualify_counter_set_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssCounterSetQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssCounterSetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterSetUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssCounterSetQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCounterSetUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssCounterSetQualificationOutcome::Qualified(CssCounterSetValue::None);
        }
        if is_css_wide_keyword(identifier) {
            return CssCounterSetQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCounterSetUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut component_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = item_start.take() {
                    component_classes.push(classify_counter_set_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if item_start.is_none() {
            item_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = item_start.take()
        {
            component_classes.push(classify_counter_set_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }
    if let Some(start) = item_start {
        component_classes.push(classify_counter_set_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    if component_classes.is_empty() {
        return CssCounterSetQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    group_counter_set_components(&component_classes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssColorSchemeItemClass {
    Light,
    Dark,
    Only,
    CustomIdent(CssColorSchemeCustomIdentEvidenceRef),
    Invalid,
}

/// Classifies one already-partitioned top-level `color-scheme` component.
///
/// A component qualifies as `Light`/`Dark` iff, after trivia handling, it
/// is exactly one direct `Ident` token whose tokenizer-decoded identifier
/// is `light`/`dark` (ASCII-case-insensitively). It qualifies as the
/// structural `Only` modifier under the same single-Ident condition when
/// the decoded identifier is `only`. It is `CustomIdent` when the decoded
/// identifier is none of `normal`, `light`, `dark`, `only`, `default`
/// (ASCII-case-insensitively), and not a CSS-wide keyword -- these are the
/// property-specific plus generic `<custom-ident>` exclusions; `none` is
/// deliberately absent from this list, since unlike `container-name` this
/// property does not reserve it. This is the interpreted decoded identity,
/// never the raw authored spelling: an escape-authored Ident decodes
/// through the tokenizer before this comparison runs. A component with
/// more than one non-trivia token -- including a Function, a bracketed
/// construct, or a stray `Comma` sharing a component with a neighboring
/// token -- is directly `Invalid`, since this item grammar has no ordinary
/// Function-backed branch and this leaf is delimiter-free `+` repetition,
/// never `#` comma-list repetition. A quoted `String` is a different token
/// class than `Ident` and is therefore also `Invalid`. A qualified
/// `CustomIdent` component's recognition-time evidence reference is the
/// absolute retained lexical-item index of the exact selected Ident token,
/// reusing the `page` / `transition-property` / `animation-name` /
/// `anchor-name` / `container-name` ownership pattern: the reference is a
/// locator, not the tokenizer-owned decoded identity itself.
fn classify_color_scheme_item(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssColorSchemeItemClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssColorSchemeItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssColorSchemeItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("light") => {
            CssColorSchemeItemClass::Light
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("dark") => {
            CssColorSchemeItemClass::Dark
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("only") => {
            CssColorSchemeItemClass::Only
        }
        CssTokenKind::Ident(identifier)
            if identifier.eq_ignore_ascii_case("normal")
                || identifier.eq_ignore_ascii_case("default")
                || is_css_wide_keyword(identifier) =>
        {
            CssColorSchemeItemClass::Invalid
        }
        CssTokenKind::Ident(_) => {
            CssColorSchemeItemClass::CustomIdent(CssColorSchemeCustomIdentEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            })
        }
        _ => CssColorSchemeItemClass::Invalid,
    }
}

/// Qualifies one retained `color-scheme` declaration value against
/// `normal | [ light | dark | <custom-ident> ]+ && only?` (#590 / #418
/// comment 5585104754), composing the accepted delimiter-free
/// top-level-component partitioning theorem (`container-name`) with a
/// bounded property-local `&&` group-boundary matcher: an optional
/// structural `only` component composes with the *entire* mandatory
/// repeated scheme-item group, and may appear only immediately before or
/// immediately after that group -- never interior to it, never duplicated,
/// and never standing alone in place of the group.
///
/// Deferred substitution and the whole-value Function boundary are
/// checked first, exactly as for `container-name`. A sole retained direct
/// `normal` Ident, ASCII-case-insensitively, qualifies the dedicated
/// standalone branch and never reaches component recognition -- `normal`
/// is deliberately never a repeated-item sentinel or combinable with
/// `only`. A sole CSS-wide keyword preserves the existing whole-value
/// Unsupported boundary. Otherwise this single left-to-right
/// recognition-time pass partitions the value into ordered top-level
/// components using depth-zero Whitespace/Comment trivia as separators --
/// never raw-source whitespace splitting -- and classifies each component
/// the instant its block depth returns to zero, so a bare `Comma` never
/// behaves as a permitted separator. Any decisive `Invalid` component
/// anywhere makes the whole declaration `InvalidForSelectedValueGrammar`,
/// as does an empty component sequence.
///
/// With every component classified and none `Invalid`, at most one
/// component may be `Only`; more than one is `InvalidForSelectedValueGrammar`.
/// A single `Only` component must sit at index `0` or at the final index of
/// the component sequence -- any other position is interior placement and
/// is `InvalidForSelectedValueGrammar`. The remaining components (with the
/// `Only` component, if any, removed) form the mandatory scheme-item
/// group: it must be non-empty, so a sole `only` (with no scheme items
/// left over) is also `InvalidForSelectedValueGrammar`. Exact authored
/// order and duplicate `Light`/`Dark`/`CustomIdent` items are preserved in
/// the resulting `items` vector when the whole declaration qualifies.
fn qualify_color_scheme_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssColorSchemeQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssColorSchemeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssColorSchemeUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssColorSchemeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssColorSchemeUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("normal") {
            return CssColorSchemeQualificationOutcome::Qualified(CssColorSchemeValue::Normal);
        }
        if is_css_wide_keyword(identifier) {
            return CssColorSchemeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColorSchemeUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = item_start.take() {
                    item_classes.push(classify_color_scheme_item(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if item_start.is_none() {
            item_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = item_start.take()
        {
            item_classes.push(classify_color_scheme_item(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }
    if let Some(start) = item_start {
        item_classes.push(classify_color_scheme_item(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    if item_classes.is_empty() {
        return CssColorSchemeQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if item_classes
        .iter()
        .any(|class| matches!(class, CssColorSchemeItemClass::Invalid))
    {
        return CssColorSchemeQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let only_positions: Vec<usize> = item_classes
        .iter()
        .enumerate()
        .filter_map(|(index, class)| {
            matches!(class, CssColorSchemeItemClass::Only).then_some(index)
        })
        .collect();

    let only = match only_positions.as_slice() {
        [] => false,
        [position] => {
            let last_index = item_classes.len() - 1;
            if *position != 0 && *position != last_index {
                return CssColorSchemeQualificationOutcome::InvalidForSelectedValueGrammar;
            }
            true
        }
        _ => return CssColorSchemeQualificationOutcome::InvalidForSelectedValueGrammar,
    };

    let scheme_items: Vec<CssColorSchemeItemValue> = item_classes
        .into_iter()
        .filter_map(|class| match class {
            CssColorSchemeItemClass::Light => Some(CssColorSchemeItemValue::Light),
            CssColorSchemeItemClass::Dark => Some(CssColorSchemeItemValue::Dark),
            CssColorSchemeItemClass::CustomIdent(evidence) => {
                Some(CssColorSchemeItemValue::CustomIdent(evidence))
            }
            CssColorSchemeItemClass::Only => None,
            CssColorSchemeItemClass::Invalid => {
                unreachable!("Invalid item classes are filtered above")
            }
        })
        .collect();

    if scheme_items.is_empty() {
        return CssColorSchemeQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    CssColorSchemeQualificationOutcome::Qualified(CssColorSchemeValue::Schemes {
        items: scheme_items,
        only,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssImageResolutionComponentClass {
    FromImage,
    DirectResolution,
    Snap,
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

fn is_css_resolution_unit(unit: &str) -> bool {
    ["dpi", "dpcm", "dppx", "x"]
        .iter()
        .any(|resolution_unit| unit.eq_ignore_ascii_case(resolution_unit))
}

/// Partitions an already-retained `image-resolution` declaration value
/// window into ordered top-level components using one left-to-right
/// recognition-time pass. Depth-zero Whitespace/Comment lexical items are
/// separators; Function and bracket openers extend the current component
/// until their matching closer, so nested content never inflates top-level
/// cardinality. This intentionally duplicates the equivalent
/// `offset-rotate` walk rather than sharing it: this leaf keeps its own
/// bounded local recognition.
fn image_resolution_top_level_components(items: &[CssLexicalItem]) -> Vec<&[CssLexicalItem]> {
    let mut components = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut current_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = current_start.take() {
                    components.push(&items[start..index]);
                }
                continue;
            }
        }

        if current_start.is_none() {
            current_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = current_start.take()
        {
            components.push(&items[start..=index]);
        }
    }

    if let Some(start) = current_start {
        components.push(&items[start..]);
    }

    components
}

/// Classifies one already-partitioned top-level `image-resolution`
/// component. A Function-headed component is classified by name/placement
/// only; its interior is never parsed or evaluated. A direct component
/// qualifies as the `from-image`/`snap` keyword operand only when it is
/// exactly one ASCII-case-insensitive `Ident` token matching that keyword,
/// and as the direct `<resolution>` operand only when it is exactly one
/// retained `Dimension` token whose decoded unit is ASCII-case-insensitively
/// `dpi`, `dpcm`, `dppx`, or `x` and whose value is not negative-non-zero
/// (signed zero remains qualified, reusing the accepted lexical-zero
/// `is_non_negative_direct_number` policy). A unitless Number never
/// satisfies the direct `<resolution>` operand.
fn classify_image_resolution_component(
    component: &[CssLexicalItem],
) -> CssImageResolutionComponentClass {
    if let Some(name) = entire_function_name(component) {
        return if is_whole_value_function(name) {
            CssImageResolutionComponentClass::MisplacedWholeValueFunction
        } else {
            CssImageResolutionComponentClass::ResidualFunction
        };
    }

    let mut tokens = component.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(token) = tokens.next() else {
        return CssImageResolutionComponentClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssImageResolutionComponentClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("from-image") => {
            CssImageResolutionComponentClass::FromImage
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("snap") => {
            CssImageResolutionComponentClass::Snap
        }
        CssTokenKind::Dimension { value, unit, .. }
            if is_css_resolution_unit(unit) && is_non_negative_direct_number(value) =>
        {
            CssImageResolutionComponentClass::DirectResolution
        }
        _ => CssImageResolutionComponentClass::Invalid,
    }
}

/// Qualifies one already-retained `image-resolution` ordinary declaration
/// value window against the selected `[ from-image || <resolution> ] &&
/// snap?` profile (#596 / #418 comment 5601753463).
///
/// Classification order is load-bearing: deferred/arbitrary substitution,
/// then whole-value Function, then whole-value CSS-wide keyword, then one
/// property-local depth-balanced component-grouping walk. `snap` is an
/// orthogonal structural modifier of the *entire* `[ from-image ||
/// <resolution> ]` group -- reusing the accepted `color-scheme` `only`
/// group-boundary theorem -- so at most one `snap` component may appear,
/// and only at the very start or very end of the top-level component
/// sequence; any other position (splitting the inner group, e.g.
/// `from-image snap 1dpi`) is decisive `InvalidForSelectedValueGrammar`.
/// The remaining (non-`snap`) components form the mandatory inner group: it
/// must be non-empty, contain at most one `from-image` component, and
/// contain at most one component competing for the single `<resolution>`
/// slot (a direct literal and a residual Function compete for that same
/// slot). A decisive direct structural failure -- more than two non-`snap`
/// components, a duplicate `from-image`, two components competing for the
/// `<resolution>` slot, or any otherwise-Invalid or misplaced
/// whole-value-Function component -- always wins over a residual Function
/// occupying the `<resolution>` slot, which keeps cases such as
/// `from-image snap calc(1dppx)` `InvalidForSelectedValueGrammar` rather
/// than softened into `FunctionValue` `Unsupported`.
fn qualify_image_resolution_value(
    items: &[CssLexicalItem],
) -> CssImageResolutionQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssImageResolutionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssImageResolutionUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssImageResolutionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssImageResolutionUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssImageResolutionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssImageResolutionUnsupportedReason::CssWideKeyword,
        );
    }

    let components = image_resolution_top_level_components(items);

    if components.is_empty() {
        return CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let classes: Vec<_> = components
        .iter()
        .map(|component| classify_image_resolution_component(component))
        .collect();

    if classes.iter().any(|class| {
        matches!(
            class,
            CssImageResolutionComponentClass::Invalid
                | CssImageResolutionComponentClass::MisplacedWholeValueFunction
        )
    }) {
        return CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let snap_positions: Vec<usize> = classes
        .iter()
        .enumerate()
        .filter_map(|(index, class)| {
            matches!(class, CssImageResolutionComponentClass::Snap).then_some(index)
        })
        .collect();

    let has_snap = match snap_positions.as_slice() {
        [] => false,
        [position] => {
            let last_index = classes.len() - 1;
            if *position != 0 && *position != last_index {
                return CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar;
            }
            true
        }
        _ => return CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar,
    };

    let group_classes: Vec<_> = classes
        .iter()
        .filter(|class| !matches!(class, CssImageResolutionComponentClass::Snap))
        .collect();

    if group_classes.is_empty() || group_classes.len() > 2 {
        return CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let from_image_count = group_classes
        .iter()
        .filter(|class| matches!(class, CssImageResolutionComponentClass::FromImage))
        .count();
    let resolution_slot_classes: Vec<_> = group_classes
        .iter()
        .filter(|class| {
            matches!(
                class,
                CssImageResolutionComponentClass::DirectResolution
                    | CssImageResolutionComponentClass::ResidualFunction
            )
        })
        .collect();

    if from_image_count > 1 || resolution_slot_classes.len() > 1 {
        return CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if resolution_slot_classes
        .first()
        .is_some_and(|class| matches!(class, CssImageResolutionComponentClass::ResidualFunction))
    {
        return CssImageResolutionQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssImageResolutionUnsupportedReason::FunctionValue,
        );
    }

    let has_from_image = from_image_count == 1;
    let has_direct_resolution = !resolution_slot_classes.is_empty();

    match (has_from_image, has_direct_resolution, has_snap) {
        (true, false, false) => {
            CssImageResolutionQualificationOutcome::Qualified(CssImageResolutionValue::FromImage)
        }
        (false, true, false) => CssImageResolutionQualificationOutcome::Qualified(
            CssImageResolutionValue::DirectResolution,
        ),
        (true, true, false) => CssImageResolutionQualificationOutcome::Qualified(
            CssImageResolutionValue::FromImageAndResolution,
        ),
        (true, false, true) => CssImageResolutionQualificationOutcome::Qualified(
            CssImageResolutionValue::FromImageAndSnap,
        ),
        (false, true, true) => CssImageResolutionQualificationOutcome::Qualified(
            CssImageResolutionValue::DirectResolutionAndSnap,
        ),
        (true, true, true) => CssImageResolutionQualificationOutcome::Qualified(
            CssImageResolutionValue::FromImageAndResolutionAndSnap,
        ),
        (false, false, _) => CssImageResolutionQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssWillChangeItemClass {
    Qualified(CssWillChangeItemValue),
    Invalid,
}

/// Classifies one already-comma-segmented `will-change`
/// `<animateable-feature>` list item.
///
/// An item qualifies iff, after trivia handling, it is exactly one direct
/// `Ident` token. `scroll-position` and `contents` are recognized ASCII
/// case-insensitively as the two dedicated predefined keywords. Every other
/// direct Ident qualifies as an open-ended `<custom-ident>` item unless its
/// decoded identifier is, ASCII-case-insensitively, `will-change`, `none`,
/// `all`, `auto`, `default`, or a CSS-wide keyword -- the property-local
/// exclusions from css-will-change-1 layered on top of the normal
/// `<custom-ident>` exclusions. This test is the interpreted decoded
/// identity, never the raw authored spelling. Critically, this leaf never
/// tests whether the decoded identity names an existing built-in CSS
/// property, resolves an alias, or expands a shorthand -- `Not-A-Property`,
/// `transform`, `TRANSFORM`, `--var`, and `--Foo` are all ordinary qualified
/// `CustomIdent` items, and their exact case-sensitive identity is preserved
/// unchanged. This item grammar has no ordinary Function-backed branch, so
/// any Function-headed item is directly `Invalid`. A qualified `CustomIdent`
/// item's recognition-time evidence reference is the absolute retained
/// lexical-item index of the exact selected Ident token, reusing the `page`
/// / `transition-property` / `anchor-name` / `container-name` /
/// `color-scheme` ownership pattern: the reference is a locator, not the
/// tokenizer-owned decoded identity itself.
fn classify_will_change_item(
    item: &[CssLexicalItem],
    absolute_item_start: usize,
) -> CssWillChangeItemClass {
    let mut tokens = item
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, token)) = tokens.next() else {
        return CssWillChangeItemClass::Invalid;
    };
    if tokens.next().is_some() {
        return CssWillChangeItemClass::Invalid;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("scroll-position") => {
            CssWillChangeItemClass::Qualified(CssWillChangeItemValue::ScrollPosition)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("contents") => {
            CssWillChangeItemClass::Qualified(CssWillChangeItemValue::Contents)
        }
        CssTokenKind::Ident(identifier)
            if identifier.eq_ignore_ascii_case("will-change")
                || identifier.eq_ignore_ascii_case("none")
                || identifier.eq_ignore_ascii_case("all")
                || identifier.eq_ignore_ascii_case("auto")
                || identifier.eq_ignore_ascii_case("default")
                || is_css_wide_keyword(identifier) =>
        {
            CssWillChangeItemClass::Invalid
        }
        CssTokenKind::Ident(_) => CssWillChangeItemClass::Qualified(
            CssWillChangeItemValue::CustomIdent(CssWillChangeCustomIdentEvidenceRef {
                lexical_item_index: absolute_item_start + relative_index,
            }),
        ),
        _ => CssWillChangeItemClass::Invalid,
    }
}

/// Qualifies one retained `will-change` declaration value against
/// `auto | <animateable-feature>#` (#598 / css-will-change-1), composing the
/// accepted top-level comma-list theorem (#571/#573/#575/#577) with the
/// accepted open-ended evidence-reference ownership theorem proven by
/// `page` / `transition-property` / `hyphenate-character` / `animation-name`
/// / `anchor-name` / `container-name` / `color-scheme`.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for `anchor-name`/`container-name`; the depth-balanced
/// walk below never sees their nested fallback commas as outer separators. A
/// sole retained direct `auto` Ident, ASCII-case-insensitively, qualifies
/// the dedicated whole-value branch and never reaches list segmentation --
/// unlike `animation-name`'s `none`, `auto` is deliberately never a
/// repeated-item sentinel here, so `auto, transform`, `transform, auto`, and
/// `auto transform` all fail: either the whole-value single-token check
/// does not match because more than one non-trivia token is present, or the
/// resulting list item itself decodes to the excluded `auto` identity. A
/// sole CSS-wide keyword preserves the existing whole-value Unsupported
/// boundary. Otherwise every top-level depth-zero-comma-delimited item is
/// classified independently; any decisive `Invalid` item anywhere in the
/// list makes the whole declaration `InvalidForSelectedValueGrammar`,
/// preserving exact authored order and duplicate items -- including
/// case-differing duplicates such as `transform, TRANSFORM` -- in the
/// resulting item vector when every item qualifies.
fn qualify_will_change_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssWillChangeQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssWillChangeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssWillChangeUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssWillChangeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssWillChangeUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("auto") {
            return CssWillChangeQualificationOutcome::Qualified(CssWillChangeValue::Auto);
        }
        if is_css_wide_keyword(identifier) {
            return CssWillChangeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssWillChangeUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut item_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            item_classes.push(classify_will_change_item(
                &items[item_start..index],
                lexical_item_start + item_start,
            ));
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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
    item_classes.push(classify_will_change_item(
        &items[item_start..],
        lexical_item_start + item_start,
    ));

    if item_classes
        .iter()
        .any(|class| matches!(class, CssWillChangeItemClass::Invalid))
    {
        return CssWillChangeQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let features = item_classes
        .into_iter()
        .map(|class| match class {
            CssWillChangeItemClass::Qualified(value) => value,
            CssWillChangeItemClass::Invalid => {
                unreachable!("Invalid item classes are filtered above")
            }
        })
        .collect();

    CssWillChangeQualificationOutcome::Qualified(CssWillChangeValue::Features(features))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssScaleComponentClass {
    Number(CssScaleComponentEvidenceRef),
    Percentage(CssScaleComponentEvidenceRef),
    MisplacedWholeValueFunction,
    ResidualFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `scale` component against
/// the direct-authored profile's two accepted token shapes -- a direct
/// `Number` or a direct `Percentage` -- plus the Function-position boundary
/// shared with `border-spacing`/`aspect-ratio` (#602). A component headed by
/// a `Function` token is never a valid direct component; it is
/// `MisplacedWholeValueFunction` when its decoded name is one of the
/// recognized generic whole-value-only functions (`is_whole_value_function`)
/// occupying a non-whole-value position -- decisively invalid there, since
/// whole-value syntax has no independent meaning embedded inside the
/// bounded component sequence -- or `ResidualFunction` for any other
/// Function, preserving the conservative open envelope for a structurally
/// feasible numeric position without evaluating it. A component with more
/// than one non-trivia token that is not Function-headed is directly
/// `Invalid`. No range restriction or machine-number conversion is applied
/// to a direct `Number`/`Percentage` token: exact authored evidence is
/// authoritative for membership.
fn classify_scale_component(
    component: &[CssLexicalItem],
    absolute_component_start: usize,
) -> CssScaleComponentClass {
    let mut tokens =
        component
            .iter()
            .enumerate()
            .filter_map(|(relative_index, entry)| match entry {
                CssLexicalItem::SemanticToken(token)
                    if !matches!(token.kind(), CssTokenKind::Whitespace) =>
                {
                    Some((relative_index, token))
                }
                _ => None,
            });

    let Some((relative_index, first)) = tokens.next() else {
        return CssScaleComponentClass::Invalid;
    };

    if let CssTokenKind::Function(name) = first.kind() {
        return if is_whole_value_function(name) {
            CssScaleComponentClass::MisplacedWholeValueFunction
        } else {
            CssScaleComponentClass::ResidualFunction
        };
    }

    if tokens.next().is_some() {
        return CssScaleComponentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Number { .. } => {
            CssScaleComponentClass::Number(CssScaleComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        CssTokenKind::Percentage { .. } => {
            CssScaleComponentClass::Percentage(CssScaleComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        _ => CssScaleComponentClass::Invalid,
    }
}

/// Qualifies one retained `scale` declaration value against the pin-bounded
/// direct-authored profile `none | [ <number> | <percentage> ]{1,3}` (#602),
/// reusing the accepted `border-spacing`/`aspect-ratio` block-depth-aware
/// top-level component partition and residual-Function-profile theorem.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first. A sole retained direct `none` Ident, ASCII-case-insensitively,
/// qualifies the dedicated whole-value branch and never reaches component
/// partitioning -- `none` is never a numeric component here. A sole
/// CSS-wide keyword preserves the existing whole-value Unsupported
/// boundary. Otherwise this single left-to-right recognition-time pass
/// partitions the value into ordered top-level components using
/// depth-zero Whitespace/Comment trivia as separators -- never a `Comma`,
/// since this grammar is whitespace-separated repetition, not a comma list
/// -- classifying each component the instant its block depth returns to
/// zero. Zero components, more than three components, or any decisively
/// invalid/misplaced component makes the declaration
/// `InvalidForSelectedValueGrammar` regardless of any residual Function
/// found elsewhere -- structural/direct decisive invalidity always
/// outranks a provisional Function ambiguity. Only once every component is
/// confirmed structurally feasible does an unresolved residual Function
/// resolve to `UnsupportedBySelectedValueProfile(FunctionValue)`.
fn qualify_scale_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssScaleQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssScaleQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssScaleUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssScaleQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssScaleUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssScaleQualificationOutcome::Qualified(CssScaleValue::None);
        }
        if is_css_wide_keyword(identifier) {
            return CssScaleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScaleUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut component_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut component_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = component_start.take() {
                    component_classes.push(classify_scale_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if component_start.is_none() {
            component_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = component_start.take()
        {
            component_classes.push(classify_scale_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }
    if let Some(start) = component_start {
        component_classes.push(classify_scale_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    if component_classes.is_empty() || component_classes.len() > 3 {
        return CssScaleQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut has_residual_function = false;
    let mut components = Vec::with_capacity(component_classes.len());
    for class in component_classes {
        match class {
            CssScaleComponentClass::Number(evidence_ref) => {
                components.push(CssScaleComponent {
                    kind: CssScaleComponentKind::Number,
                    evidence_ref,
                });
            }
            CssScaleComponentClass::Percentage(evidence_ref) => {
                components.push(CssScaleComponent {
                    kind: CssScaleComponentKind::Percentage,
                    evidence_ref,
                });
            }
            CssScaleComponentClass::ResidualFunction => {
                has_residual_function = true;
            }
            CssScaleComponentClass::MisplacedWholeValueFunction
            | CssScaleComponentClass::Invalid => {
                return CssScaleQualificationOutcome::InvalidForSelectedValueGrammar;
            }
        }
    }

    if has_residual_function {
        return CssScaleQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssScaleUnsupportedReason::FunctionValue,
        );
    }

    CssScaleQualificationOutcome::Qualified(CssScaleValue::Components(components))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssRotateComponentClass {
    Angle(CssRotateEvidenceRef),
    Number(CssRotateEvidenceRef),
    KeywordAxis(CssRotateAxisKeyword),
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `rotate` component against
/// the token shapes admitted anywhere in the finite direct-shape theorem
/// (#604): a direct `<angle>` (a retained `Dimension` token whose decoded
/// unit is ASCII-case-insensitively `deg`, `grad`, `rad`, or `turn`,
/// reusing the accepted `offset-rotate` direct-angle theorem), a direct
/// `<number>` (a retained `Number` token -- a unitless Number, including
/// zero, never satisfies the `<angle>` shape), a direct keyword axis (`x`,
/// `y`, or `z`, ASCII-case-insensitively), or a Function-headed component
/// classified by placement/identity only, exactly as in
/// `offset-rotate`/`scale`. A component with more than one non-trivia
/// token that is not Function-headed is directly `Invalid`; a Percentage,
/// a wrong Dimension unit, or any other token class never satisfies the
/// `<number>` or `<angle>` shape.
fn classify_rotate_component(
    component: &[CssLexicalItem],
    absolute_component_start: usize,
) -> CssRotateComponentClass {
    let mut tokens =
        component
            .iter()
            .enumerate()
            .filter_map(|(relative_index, entry)| match entry {
                CssLexicalItem::SemanticToken(token)
                    if !matches!(token.kind(), CssTokenKind::Whitespace) =>
                {
                    Some((relative_index, token))
                }
                _ => None,
            });

    let Some((relative_index, first)) = tokens.next() else {
        return CssRotateComponentClass::Invalid;
    };

    if let CssTokenKind::Function(name) = first.kind() {
        return if is_whole_value_function(name) {
            CssRotateComponentClass::MisplacedWholeValueFunction
        } else {
            CssRotateComponentClass::ResidualFunction
        };
    }

    if tokens.next().is_some() {
        return CssRotateComponentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Dimension { unit, .. } if is_css_angle_unit(unit) => {
            CssRotateComponentClass::Angle(CssRotateEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        CssTokenKind::Number { .. } => CssRotateComponentClass::Number(CssRotateEvidenceRef {
            lexical_item_index: absolute_component_start + relative_index,
        }),
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("x") => {
            CssRotateComponentClass::KeywordAxis(CssRotateAxisKeyword::X)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("y") => {
            CssRotateComponentClass::KeywordAxis(CssRotateAxisKeyword::Y)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("z") => {
            CssRotateComponentClass::KeywordAxis(CssRotateAxisKeyword::Z)
        }
        _ => CssRotateComponentClass::Invalid,
    }
}

/// Partitions an already-retained `rotate` declaration value window into
/// ordered top-level components using one left-to-right recognition-time
/// pass and classifies each the instant its block depth returns to zero.
/// Depth-zero Whitespace/Comment lexical items are separators; Function and
/// bracket openers extend the current component until their matching
/// closer, so nested content (e.g. `calc(min(1, 2))`) never inflates
/// top-level cardinality. This intentionally duplicates the equivalent
/// `scale`/`offset-rotate` walk rather than sharing it: this leaf keeps its
/// own bounded local recognition.
fn rotate_top_level_component_classes(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> Vec<CssRotateComponentClass> {
    let mut classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut component_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = component_start.take() {
                    classes.push(classify_rotate_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if component_start.is_none() {
            component_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = component_start.take()
        {
            classes.push(classify_rotate_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }

    if let Some(start) = component_start {
        classes.push(classify_rotate_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    classes
}

/// Qualifies the sole component of a one-component candidate shape against
/// the direct `<angle>`-only branch of the finite theorem (#604). A bare
/// `<number>` (including unitless zero) or a keyword axis alone never
/// satisfies this shape -- the individual `rotate` property grammar
/// contains `<angle>`, not `<angle> | <zero>`, unlike transform functions
/// such as `rotate()`.
fn qualify_rotate_angle_only_shape(
    class: CssRotateComponentClass,
) -> CssRotateQualificationOutcome {
    match class {
        CssRotateComponentClass::Angle(angle) => {
            CssRotateQualificationOutcome::Qualified(CssRotateValue::Rotation(CssRotateRotation {
                axis: None,
                angle,
            }))
        }
        CssRotateComponentClass::ResidualFunction => {
            CssRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRotateUnsupportedReason::FunctionValue,
            )
        }
        _ => CssRotateQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

/// Qualifies a two-component candidate shape against the keyword-axis
/// branch of the finite theorem (#604): exactly one direct keyword axis
/// and one direct `<angle>`, in either authored order. A Function can only
/// stand in the `<angle>` operand -- a literal `x`/`y`/`z` keyword operand
/// is never satisfied by Function-position ambiguity -- so a Function
/// paired with anything other than a concrete keyword axis is decisively
/// Invalid rather than Unsupported.
fn qualify_rotate_keyword_axis_shape(
    first: CssRotateComponentClass,
    second: CssRotateComponentClass,
) -> CssRotateQualificationOutcome {
    match (first, second) {
        (CssRotateComponentClass::KeywordAxis(keyword), CssRotateComponentClass::Angle(angle))
        | (CssRotateComponentClass::Angle(angle), CssRotateComponentClass::KeywordAxis(keyword)) => {
            CssRotateQualificationOutcome::Qualified(CssRotateValue::Rotation(CssRotateRotation {
                axis: Some(CssRotateAxis::Keyword(keyword)),
                angle,
            }))
        }
        (CssRotateComponentClass::KeywordAxis(_), CssRotateComponentClass::ResidualFunction)
        | (CssRotateComponentClass::ResidualFunction, CssRotateComponentClass::KeywordAxis(_)) => {
            CssRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRotateUnsupportedReason::FunctionValue,
            )
        }
        _ => CssRotateQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

/// Qualifies a four-component candidate shape against the vector-axis
/// branch of the finite theorem (#604): exactly three direct `<number>`
/// components forming one axis operand plus one direct `<angle>`
/// component, in either authored order -- `<number> <number> <number>
/// <angle>` or `<angle> <number> <number> <number>` -- never with the
/// angle interleaved among the three numbers. A concrete match (no
/// residual Function needed) is `Qualified` directly. Otherwise, a
/// residual Function is only `UnsupportedBySelectedValueProfile
/// (FunctionValue)` when assigning it to the ambiguous `<number>`/
/// `<angle>` slot it occupies would complete one of the two orderings;
/// any component that can never occupy its position under either ordering
/// (a keyword axis, a misplaced whole-value Function, or any other
/// decisively invalid component) makes the whole shape decisively
/// Invalid, since `is_number_position`/`is_angle_position` below never
/// admit those classes.
fn qualify_rotate_vector_axis_shape(
    first: CssRotateComponentClass,
    second: CssRotateComponentClass,
    third: CssRotateComponentClass,
    fourth: CssRotateComponentClass,
) -> CssRotateQualificationOutcome {
    if let (
        CssRotateComponentClass::Number(x),
        CssRotateComponentClass::Number(y),
        CssRotateComponentClass::Number(z),
        CssRotateComponentClass::Angle(angle),
    ) = (first, second, third, fourth)
    {
        return CssRotateQualificationOutcome::Qualified(CssRotateValue::Rotation(
            CssRotateRotation {
                axis: Some(CssRotateAxis::Vector([x, y, z])),
                angle,
            },
        ));
    }

    if let (
        CssRotateComponentClass::Angle(angle),
        CssRotateComponentClass::Number(x),
        CssRotateComponentClass::Number(y),
        CssRotateComponentClass::Number(z),
    ) = (first, second, third, fourth)
    {
        return CssRotateQualificationOutcome::Qualified(CssRotateValue::Rotation(
            CssRotateRotation {
                axis: Some(CssRotateAxis::Vector([x, y, z])),
                angle,
            },
        ));
    }

    let is_number_position = |class: CssRotateComponentClass| {
        matches!(
            class,
            CssRotateComponentClass::Number(_) | CssRotateComponentClass::ResidualFunction
        )
    };
    let is_angle_position = |class: CssRotateComponentClass| {
        matches!(
            class,
            CssRotateComponentClass::Angle(_) | CssRotateComponentClass::ResidualFunction
        )
    };

    let vector_then_angle_feasible = is_number_position(first)
        && is_number_position(second)
        && is_number_position(third)
        && is_angle_position(fourth);
    let angle_then_vector_feasible = is_angle_position(first)
        && is_number_position(second)
        && is_number_position(third)
        && is_number_position(fourth);

    if vector_then_angle_feasible || angle_then_vector_feasible {
        CssRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssRotateUnsupportedReason::FunctionValue,
        )
    } else {
        CssRotateQualificationOutcome::InvalidForSelectedValueGrammar
    }
}

/// Qualifies one retained `rotate` declaration value against the
/// pin-bounded finite-shape axis-angle profile `none | <angle> | [ x | y |
/// z | <number>{3} ] && <angle>` (#604 / css-transforms-2 `rotate`),
/// reusing the accepted `offset-rotate` direct `<angle>` theorem and the
/// accepted `scale` exact direct `<number>` evidence-reference and
/// block-depth-aware top-level component partition.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for the other selected leaves. A sole retained direct
/// `none` Ident, ASCII-case-insensitively, qualifies the dedicated
/// whole-value branch and never reaches component partitioning -- `none`
/// is never a zero angle, an axis omission, or a zero-vector sentinel. A
/// sole CSS-wide keyword preserves the existing whole-value Unsupported
/// boundary.
///
/// Otherwise the value is partitioned into top-level components and
/// dispatched purely on component count, since after partitioning the
/// grammar's accepted direct shapes are finite: exactly one component
/// (`<angle>`), exactly two components (a keyword axis and an angle, in
/// either order), or exactly four components (an exact three-`<number>`
/// vector axis and an angle, in either order). Any other component count
/// -- zero, three, or more than four -- is decisive
/// `InvalidForSelectedValueGrammar` regardless of any residual Function
/// present, since no accepted shape has that cardinality; this also
/// ensures a later decisive structural failure (e.g. a fifth trailing
/// component) always outranks an earlier provisional Function ambiguity.
fn qualify_rotate_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssRotateQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssRotateUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssRotateUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssRotateQualificationOutcome::Qualified(CssRotateValue::None);
        }
        if is_css_wide_keyword(identifier) {
            return CssRotateQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRotateUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let classes = rotate_top_level_component_classes(items, lexical_item_start);

    match classes.as_slice() {
        [class] => qualify_rotate_angle_only_shape(*class),
        [first, second] => qualify_rotate_keyword_axis_shape(*first, *second),
        [first, second, third, fourth] => {
            qualify_rotate_vector_axis_shape(*first, *second, *third, *fourth)
        }
        _ => CssRotateQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssTranslateComponentClass {
    Length(CssTranslateComponentEvidenceRef),
    Percentage(CssTranslateComponentEvidenceRef),
    MisplacedWholeValueFunction,
    ResidualFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `translate` component
/// against the direct-authored profile's accepted token shapes -- a direct
/// exact-zero `Number`, a `Dimension` with a recognized CSS length unit, or
/// a `Percentage` -- plus the Function-position boundary shared with
/// `scale`/`rotate` (#606). A component headed by a `Function` token is
/// never a direct component; it is `MisplacedWholeValueFunction` when its
/// decoded name is one of the recognized generic whole-value-only functions
/// (`is_whole_value_function`) occupying a non-whole-value position --
/// decisively invalid there -- or `ResidualFunction` for any other
/// Function, preserving the conservative open envelope for a structurally
/// feasible position without evaluating it. A component with more than one
/// non-trivia token that is not Function-headed is directly `Invalid`. A
/// non-zero unitless `Number` and a `Dimension` with an unrecognized unit
/// (e.g. `deg`, `s`, `fr`) both fall through to `Invalid`: this leaf never
/// interprets an arbitrary `Number` as `Length`. No range restriction or
/// machine-number conversion is applied: exact authored evidence is
/// authoritative for membership. This function is positionally unaware --
/// the caller applies the third-slot `Length`-only restriction after
/// partitioning, since only slot placement (not component identity) decides
/// whether a direct `Percentage` is accepted.
fn classify_translate_component(
    component: &[CssLexicalItem],
    absolute_component_start: usize,
) -> CssTranslateComponentClass {
    let mut tokens =
        component
            .iter()
            .enumerate()
            .filter_map(|(relative_index, entry)| match entry {
                CssLexicalItem::SemanticToken(token)
                    if !matches!(token.kind(), CssTokenKind::Whitespace) =>
                {
                    Some((relative_index, token))
                }
                _ => None,
            });

    let Some((relative_index, first)) = tokens.next() else {
        return CssTranslateComponentClass::Invalid;
    };

    if let CssTokenKind::Function(name) = first.kind() {
        return if is_whole_value_function(name) {
            CssTranslateComponentClass::MisplacedWholeValueFunction
        } else {
            CssTranslateComponentClass::ResidualFunction
        };
    }

    if tokens.next().is_some() {
        return CssTranslateComponentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssTranslateComponentClass::Length(CssTranslateComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        CssTokenKind::Dimension { unit, .. } if is_css_length_unit(unit) => {
            CssTranslateComponentClass::Length(CssTranslateComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        CssTokenKind::Percentage { .. } => {
            CssTranslateComponentClass::Percentage(CssTranslateComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        _ => CssTranslateComponentClass::Invalid,
    }
}

/// Qualifies one retained `translate` declaration value against the
/// pin-bounded heterogeneous positional profile `none | <length-percentage>
/// [ <length-percentage> <length>? ]?` (#606), reusing the accepted `scale`
/// block-depth-aware top-level component partition and residual-Function-
/// profile theorem.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first. A sole retained direct `none` Ident, ASCII-case-insensitively,
/// qualifies the dedicated whole-value branch and never reaches component
/// partitioning -- `none` is never a translation component here. A sole
/// CSS-wide keyword preserves the existing whole-value Unsupported
/// boundary. Otherwise this single left-to-right recognition-time pass
/// partitions the value into ordered top-level components using
/// depth-zero Whitespace/Comment trivia as separators -- never a `Comma` --
/// classifying each component the instant its block depth returns to zero.
/// Zero components, more than three components, or any decisively invalid/
/// misplaced component makes the declaration `InvalidForSelectedValueGrammar`
/// regardless of any residual Function found elsewhere.
///
/// The heterogeneous positional pressure is applied here, once partitioning
/// is complete: a direct `Percentage` occupying the third (zero-indexed
/// position two) component is decisive `InvalidForSelectedValueGrammar`,
/// since the grammar's third slot accepts `<length>` only, never
/// `<length-percentage>` -- this rejection applies even when the Percentage
/// is `0%`, which remains authored Percentage syntax rather than a
/// mathematical zero. A direct `Percentage` in the first or second slot
/// qualifies normally. Only once every component is confirmed structurally
/// feasible does an unresolved residual Function resolve to
/// `UnsupportedBySelectedValueProfile(FunctionValue)`.
fn qualify_translate_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssTranslateQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTranslateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTranslateUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTranslateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTranslateUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssTranslateQualificationOutcome::Qualified(CssTranslateValue::None);
        }
        if is_css_wide_keyword(identifier) {
            return CssTranslateQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTranslateUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut component_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut component_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = component_start.take() {
                    component_classes.push(classify_translate_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if component_start.is_none() {
            component_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = component_start.take()
        {
            component_classes.push(classify_translate_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }
    if let Some(start) = component_start {
        component_classes.push(classify_translate_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    if component_classes.is_empty() || component_classes.len() > 3 {
        return CssTranslateQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut has_residual_function = false;
    let mut components = Vec::with_capacity(component_classes.len());
    for (position, class) in component_classes.into_iter().enumerate() {
        match class {
            CssTranslateComponentClass::Length(evidence_ref) => {
                components.push(CssTranslateComponent {
                    kind: CssTranslateComponentKind::Length,
                    evidence_ref,
                });
            }
            CssTranslateComponentClass::Percentage(evidence_ref) => {
                if position == 2 {
                    return CssTranslateQualificationOutcome::InvalidForSelectedValueGrammar;
                }
                components.push(CssTranslateComponent {
                    kind: CssTranslateComponentKind::Percentage,
                    evidence_ref,
                });
            }
            CssTranslateComponentClass::ResidualFunction => {
                has_residual_function = true;
            }
            CssTranslateComponentClass::MisplacedWholeValueFunction
            | CssTranslateComponentClass::Invalid => {
                return CssTranslateQualificationOutcome::InvalidForSelectedValueGrammar;
            }
        }
    }

    if has_residual_function {
        return CssTranslateQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTranslateUnsupportedReason::FunctionValue,
        );
    }

    CssTranslateQualificationOutcome::Qualified(CssTranslateValue::Components(components))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssTransformOriginComponentClass {
    Keyword(CssTransformOriginKeyword),
    Length(CssTransformOriginComponentEvidenceRef),
    Percentage(CssTransformOriginComponentEvidenceRef),
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `transform-origin`
/// component against the direct-authored token shapes admitted anywhere in
/// the finite theorem (#608): one of the five direct position keywords
/// (ASCII-case-insensitively), a direct exact-zero `Number`, a `Dimension`
/// with a recognized CSS length unit, a `Percentage`, or a Function-headed
/// component classified by placement/identity only, exactly as in
/// `translate`/`rotate`/`scale`. This classification is positionally
/// unaware and role-unaware -- the caller applies the ordered/`&&`
/// role-sensitive theorem and the third-slot `<length>`-only restriction
/// after partitioning.
fn classify_transform_origin_component(
    component: &[CssLexicalItem],
    absolute_component_start: usize,
) -> CssTransformOriginComponentClass {
    let mut tokens =
        component
            .iter()
            .enumerate()
            .filter_map(|(relative_index, entry)| match entry {
                CssLexicalItem::SemanticToken(token)
                    if !matches!(token.kind(), CssTokenKind::Whitespace) =>
                {
                    Some((relative_index, token))
                }
                _ => None,
            });

    let Some((relative_index, first)) = tokens.next() else {
        return CssTransformOriginComponentClass::Invalid;
    };

    if let CssTokenKind::Function(name) = first.kind() {
        return if is_whole_value_function(name) {
            CssTransformOriginComponentClass::MisplacedWholeValueFunction
        } else {
            CssTransformOriginComponentClass::ResidualFunction
        };
    }

    if tokens.next().is_some() {
        return CssTransformOriginComponentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("left") => {
            CssTransformOriginComponentClass::Keyword(CssTransformOriginKeyword::Left)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("center") => {
            CssTransformOriginComponentClass::Keyword(CssTransformOriginKeyword::Center)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("right") => {
            CssTransformOriginComponentClass::Keyword(CssTransformOriginKeyword::Right)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("top") => {
            CssTransformOriginComponentClass::Keyword(CssTransformOriginKeyword::Top)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("bottom") => {
            CssTransformOriginComponentClass::Keyword(CssTransformOriginKeyword::Bottom)
        }
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssTransformOriginComponentClass::Length(CssTransformOriginComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        CssTokenKind::Dimension { unit, .. } if is_css_length_unit(unit) => {
            CssTransformOriginComponentClass::Length(CssTransformOriginComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        CssTokenKind::Percentage { .. } => {
            CssTransformOriginComponentClass::Percentage(CssTransformOriginComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        _ => CssTransformOriginComponentClass::Invalid,
    }
}

/// Partitions an already-retained `transform-origin` declaration value
/// window into ordered top-level components using one left-to-right
/// recognition-time pass and classifies each the instant its block depth
/// returns to zero, reusing the `translate`/`rotate`/`scale` block-depth-
/// aware walk. Depth-zero Whitespace/Comment lexical items are separators
/// -- never a `Comma`, since this grammar has no top-level comma list --
/// and Function/bracket openers extend the current component until their
/// matching closer, so nested content (e.g. `calc(min(10px, 20px))`) never
/// inflates top-level cardinality. This intentionally duplicates the
/// equivalent walk rather than sharing it: this leaf keeps its own bounded
/// local recognition.
fn transform_origin_top_level_component_classes(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> Vec<CssTransformOriginComponentClass> {
    let mut classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut component_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = component_start.take() {
                    classes.push(classify_transform_origin_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if component_start.is_none() {
            component_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = component_start.take()
        {
            classes.push(classify_transform_origin_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }

    if let Some(start) = component_start {
        classes.push(classify_transform_origin_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    classes
}

fn transform_origin_direct_component(
    class: CssTransformOriginComponentClass,
) -> Option<CssTransformOriginComponent> {
    match class {
        CssTransformOriginComponentClass::Keyword(keyword) => {
            Some(CssTransformOriginComponent::Keyword(keyword))
        }
        CssTransformOriginComponentClass::Length(evidence) => {
            Some(CssTransformOriginComponent::Length(evidence))
        }
        CssTransformOriginComponentClass::Percentage(evidence) => {
            Some(CssTransformOriginComponent::Percentage(evidence))
        }
        CssTransformOriginComponentClass::ResidualFunction
        | CssTransformOriginComponentClass::MisplacedWholeValueFunction
        | CssTransformOriginComponentClass::Invalid => None,
    }
}

/// Qualifies the sole component of a one-component candidate shape against
/// the direct `H | V | C | LP` branch of the finite theorem (#608): any one
/// of the five direct position keywords, or a direct `<length-percentage>`.
fn qualify_transform_origin_one_component_shape(
    class: CssTransformOriginComponentClass,
) -> CssTransformOriginQualificationOutcome {
    if let Some(component) = transform_origin_direct_component(class) {
        return CssTransformOriginQualificationOutcome::Qualified(
            CssTransformOriginValue::Components(vec![component]),
        );
    }

    match class {
        CssTransformOriginComponentClass::ResidualFunction => {
            CssTransformOriginQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformOriginUnsupportedReason::FunctionValue,
            )
        }
        _ => CssTransformOriginQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

/// A component class occupies the ordered branch's first slot -- `left |
/// right | center | <length-percentage>` -- directly, without a Function.
fn is_transform_origin_ordered_first_slot(class: CssTransformOriginComponentClass) -> bool {
    matches!(
        class,
        CssTransformOriginComponentClass::Keyword(
            CssTransformOriginKeyword::Left
                | CssTransformOriginKeyword::Right
                | CssTransformOriginKeyword::Center
        ) | CssTransformOriginComponentClass::Length(_)
            | CssTransformOriginComponentClass::Percentage(_)
    )
}

/// A component class occupies the ordered branch's second slot -- `top |
/// bottom | center | <length-percentage>` -- directly, without a Function.
fn is_transform_origin_ordered_second_slot(class: CssTransformOriginComponentClass) -> bool {
    matches!(
        class,
        CssTransformOriginComponentClass::Keyword(
            CssTransformOriginKeyword::Top
                | CssTransformOriginKeyword::Bottom
                | CssTransformOriginKeyword::Center
        ) | CssTransformOriginComponentClass::Length(_)
            | CssTransformOriginComponentClass::Percentage(_)
    )
}

fn is_transform_origin_ordered_first_slot_or_function(
    class: CssTransformOriginComponentClass,
) -> bool {
    is_transform_origin_ordered_first_slot(class)
        || matches!(class, CssTransformOriginComponentClass::ResidualFunction)
}

fn is_transform_origin_ordered_second_slot_or_function(
    class: CssTransformOriginComponentClass,
) -> bool {
    is_transform_origin_ordered_second_slot(class)
        || matches!(class, CssTransformOriginComponentClass::ResidualFunction)
}

/// A direct keyword satisfies the reversed `&&` branch's horizontal-or-
/// center role.
fn is_transform_origin_horizontal_or_center(keyword: CssTransformOriginKeyword) -> bool {
    matches!(
        keyword,
        CssTransformOriginKeyword::Left
            | CssTransformOriginKeyword::Right
            | CssTransformOriginKeyword::Center
    )
}

/// A direct keyword satisfies the reversed `&&` branch's vertical-or-
/// center role.
fn is_transform_origin_vertical_or_center(keyword: CssTransformOriginKeyword) -> bool {
    matches!(
        keyword,
        CssTransformOriginKeyword::Top
            | CssTransformOriginKeyword::Bottom
            | CssTransformOriginKeyword::Center
    )
}

/// Resolves a concretely valid two-component direct position pair, or
/// `None` when the pair needs a residual Function or is not role-valid. A
/// pair qualifies iff the ordered branch matches (first component in the
/// ordered first slot, second in the ordered second slot) or, when both
/// components are direct keywords, the reversed keyword-only `&&` branch
/// matches (first keyword satisfies the vertical-or-center role, second
/// the horizontal-or-center role) -- authored source order is always
/// preserved in the returned pair regardless of which branch proved
/// validity, and neither branch identity is exposed in the result.
fn transform_origin_concrete_two_component_pair(
    first: CssTransformOriginComponentClass,
    second: CssTransformOriginComponentClass,
) -> Option<(CssTransformOriginComponent, CssTransformOriginComponent)> {
    let ordered_branch_valid = is_transform_origin_ordered_first_slot(first)
        && is_transform_origin_ordered_second_slot(second);

    let reversed_keyword_branch_valid = match (first, second) {
        (
            CssTransformOriginComponentClass::Keyword(first_keyword),
            CssTransformOriginComponentClass::Keyword(second_keyword),
        ) => {
            is_transform_origin_vertical_or_center(first_keyword)
                && is_transform_origin_horizontal_or_center(second_keyword)
        }
        _ => false,
    };

    if !ordered_branch_valid && !reversed_keyword_branch_valid {
        return None;
    }

    let first_component = transform_origin_direct_component(first)?;
    let second_component = transform_origin_direct_component(second)?;
    Some((first_component, second_component))
}

/// Qualifies a two-component candidate shape against the role-sensitive
/// finite theorem (#608). A Function can only stand in for the ordered
/// branch's own `<length-percentage>` slot -- never a literal keyword role
/// in either branch -- so a Function paired with a keyword that cannot
/// occupy the *other* ordered slot is decisively Invalid rather than
/// Unsupported (e.g. `top calc(20px)`, `calc(20px) left`).
fn qualify_transform_origin_two_component_shape(
    first: CssTransformOriginComponentClass,
    second: CssTransformOriginComponentClass,
) -> CssTransformOriginQualificationOutcome {
    if let Some((first_component, second_component)) =
        transform_origin_concrete_two_component_pair(first, second)
    {
        return CssTransformOriginQualificationOutcome::Qualified(
            CssTransformOriginValue::Components(vec![first_component, second_component]),
        );
    }

    if matches!(
        first,
        CssTransformOriginComponentClass::Invalid
            | CssTransformOriginComponentClass::MisplacedWholeValueFunction
    ) || matches!(
        second,
        CssTransformOriginComponentClass::Invalid
            | CssTransformOriginComponentClass::MisplacedWholeValueFunction
    ) {
        return CssTransformOriginQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let ordered_branch_feasible_with_function =
        is_transform_origin_ordered_first_slot_or_function(first)
            && is_transform_origin_ordered_second_slot_or_function(second);

    if ordered_branch_feasible_with_function {
        CssTransformOriginQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformOriginUnsupportedReason::FunctionValue,
        )
    } else {
        CssTransformOriginQualificationOutcome::InvalidForSelectedValueGrammar
    }
}

/// Qualifies a three-component candidate shape against the finite theorem
/// (#608): `ValidTwoComponentPosition <length>`. The third, Z, slot is
/// `<length>` only -- a direct `Percentage` there (including `0%`), a
/// keyword, or any other decisively invalid/misplaced component is
/// decisive `InvalidForSelectedValueGrammar` regardless of any residual
/// Function found in the first two components, since structural/direct
/// decisive invalidity always outranks a provisional Function ambiguity.
fn qualify_transform_origin_three_component_shape(
    first: CssTransformOriginComponentClass,
    second: CssTransformOriginComponentClass,
    third: CssTransformOriginComponentClass,
) -> CssTransformOriginQualificationOutcome {
    if matches!(
        third,
        CssTransformOriginComponentClass::Percentage(_)
            | CssTransformOriginComponentClass::Keyword(_)
            | CssTransformOriginComponentClass::Invalid
            | CssTransformOriginComponentClass::MisplacedWholeValueFunction
    ) {
        return CssTransformOriginQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if matches!(
        first,
        CssTransformOriginComponentClass::Invalid
            | CssTransformOriginComponentClass::MisplacedWholeValueFunction
    ) || matches!(
        second,
        CssTransformOriginComponentClass::Invalid
            | CssTransformOriginComponentClass::MisplacedWholeValueFunction
    ) {
        return CssTransformOriginQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if let Some((first_component, second_component)) =
        transform_origin_concrete_two_component_pair(first, second)
    {
        return match third {
            CssTransformOriginComponentClass::Length(evidence) => {
                CssTransformOriginQualificationOutcome::Qualified(
                    CssTransformOriginValue::Components(vec![
                        first_component,
                        second_component,
                        CssTransformOriginComponent::Length(evidence),
                    ]),
                )
            }
            CssTransformOriginComponentClass::ResidualFunction => {
                CssTransformOriginQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTransformOriginUnsupportedReason::FunctionValue,
                )
            }
            _ => unreachable!("third slot already filtered to Length or ResidualFunction"),
        };
    }

    let xy_ordered_feasible_with_function =
        is_transform_origin_ordered_first_slot_or_function(first)
            && is_transform_origin_ordered_second_slot_or_function(second);

    if xy_ordered_feasible_with_function {
        CssTransformOriginQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformOriginUnsupportedReason::FunctionValue,
        )
    } else {
        CssTransformOriginQualificationOutcome::InvalidForSelectedValueGrammar
    }
}

/// Qualifies one retained `transform-origin` declaration value against the
/// pin-bounded property-local role-sensitive finite position theorem
/// (#608), reusing the accepted `translate`/`rotate`/`scale` block-depth-
/// aware top-level component partition, the accepted `translate` direct
/// exact-zero-`Number`-as-`Length` and CSS length-`Dimension`/`Percentage`
/// recognition, and the accepted `rotate` residual-Function-viability
/// precedent that a Function can never satisfy a literal keyword slot.
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for the other selected leaves; `transform-origin` has
/// no dedicated whole-value keyword like `none`. A sole CSS-wide keyword
/// preserves the existing whole-value Unsupported boundary. Otherwise the
/// value is partitioned into ordered top-level components and dispatched
/// purely on component count: one, two, or three components reach the
/// role-sensitive finite theorem; zero, or four or more, is decisive
/// `InvalidForSelectedValueGrammar` regardless of any residual Function
/// present, so a later decisive structural failure (e.g. a fourth trailing
/// component from a rejected generic `<position>` edge-offset form) always
/// outranks an earlier provisional Function ambiguity.
fn qualify_transform_origin_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssTransformOriginQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTransformOriginQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformOriginUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTransformOriginQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformOriginUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssTransformOriginQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformOriginUnsupportedReason::CssWideKeyword,
        );
    }

    let classes = transform_origin_top_level_component_classes(items, lexical_item_start);

    match classes.as_slice() {
        [class] => qualify_transform_origin_one_component_shape(*class),
        [first, second] => qualify_transform_origin_two_component_shape(*first, *second),
        [first, second, third] => {
            qualify_transform_origin_three_component_shape(*first, *second, *third)
        }
        _ => CssTransformOriginQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

/// Qualifies one retained `transform-box` declaration value against the
/// pin-bounded exact five-keyword authored grammar (#610):
/// `content-box | border-box | fill-box | stroke-box | view-box`. This
/// proves only authored grammar membership and authored keyword identity --
/// never the transform reference-box selection those keywords go on to
/// influence downstream. Unrelated CSS box-edge keywords (`padding-box`,
/// `margin-box`) and any multi-component or comma-delimited value are
/// decisively `InvalidForSelectedValueGrammar`, exactly as for the other
/// accepted single-keyword leaves.
fn qualify_transform_box_value(items: &[CssLexicalItem]) -> CssTransformBoxQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTransformBoxQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformBoxUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTransformBoxQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformBoxUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssTransformBoxQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformBoxUnsupportedReason::FunctionValue,
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
        return CssTransformBoxQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssTransformBoxQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("content-box") => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::ContentBox)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("border-box") => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::BorderBox)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("fill-box") => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::FillBox)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("stroke-box") => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::StrokeBox)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("view-box") => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::ViewBox)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssTransformBoxQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformBoxUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssTransformBoxQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

/// Qualifies one retained `transform-style` declaration value against the
/// pin-bounded exact two-keyword authored grammar (#612): `flat |
/// preserve-3d`. This proves only authored grammar membership and authored
/// keyword identity -- never the context-forced used value CSS Transforms 2
/// defines for grouping properties (CSSWG #14054 remains unresolved and is
/// not answered here). The historical `auto` keyword and the proposed
/// `detached` keyword (CSSWG #4242) are both outside the pinned grammar and
/// remain decisively `InvalidForSelectedValueGrammar`, exactly as any other
/// multi-component or comma-delimited value is for the other accepted
/// single-keyword leaves.
fn qualify_transform_style_value(
    items: &[CssLexicalItem],
) -> CssTransformStyleQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTransformStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformStyleUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTransformStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformStyleUnsupportedReason::WholeValueFunction,
        );
    }

    if entire_function_name(items).is_some() {
        return CssTransformStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformStyleUnsupportedReason::FunctionValue,
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
        return CssTransformStyleQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssTransformStyleQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("flat") => {
            CssTransformStyleQualificationOutcome::Qualified(CssTransformStyleValue::Flat)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("preserve-3d") => {
            CssTransformStyleQualificationOutcome::Qualified(CssTransformStyleValue::Preserve3d)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssTransformStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformStyleUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssTransformStyleQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

/// Qualifies one retained `caret-animation` declaration value against the
/// pin-bounded exact two-keyword authored grammar (#636): `auto | manual`.
/// This proves only authored grammar membership and authored keyword
/// identity -- never UA-driven caret blink/fade timing, OS/platform caret
/// settings, CSS animation execution, applicability filtering, or
/// inheritance/cascade/computed-value semantics. The historical/proposal-only
/// `none`, `fade`, and `blink` keywords remain outside the pinned grammar and
/// stay decisively `InvalidForSelectedValueGrammar`, exactly as any other
/// multi-component or comma-delimited value is for the other accepted
/// single-keyword leaves. This property has no ordinary Function branch in
/// the selected grammar, so an unrecognized Function (e.g. `foo()`,
/// `calc(1)`) falls through to the same direct grammar mismatch as any other
/// wrong token class.
fn qualify_caret_animation_value(
    items: &[CssLexicalItem],
) -> CssCaretAnimationQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssCaretAnimationQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCaretAnimationUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssCaretAnimationQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCaretAnimationUnsupportedReason::WholeValueFunction,
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
        return CssCaretAnimationQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssCaretAnimationQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto") => {
            CssCaretAnimationQualificationOutcome::Qualified(CssCaretAnimationValue::Auto)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("manual") => {
            CssCaretAnimationQualificationOutcome::Qualified(CssCaretAnimationValue::Manual)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssCaretAnimationQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCaretAnimationUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssCaretAnimationQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

/// Qualifies one retained `caret-shape` declaration value against the
/// pin-bounded exact four-keyword authored grammar (#638): `auto | bar |
/// block | underscore`. This proves only authored grammar membership and
/// authored keyword identity -- never UA-selected/effective caret shape,
/// IME composition-time override, rendered caret geometry, glyph metrics,
/// writing-mode placement, applicability filtering, or
/// inheritance/cascade/computed-value semantics. Aliases and
/// historical-looking near-misses (`none`, `underline`, `vertical`, `rect`)
/// remain outside the pinned grammar and stay decisively
/// `InvalidForSelectedValueGrammar`, exactly as any other multi-component or
/// comma-delimited value is for the other accepted single-keyword leaves.
/// This property has no ordinary Function branch in the selected grammar, so
/// an unrecognized Function (e.g. `foo()`, `calc(1)`) falls through to the
/// same direct grammar mismatch as any other wrong token class.
fn qualify_caret_shape_value(items: &[CssLexicalItem]) -> CssCaretShapeQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssCaretShapeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCaretShapeUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssCaretShapeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssCaretShapeUnsupportedReason::WholeValueFunction,
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
        return CssCaretShapeQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    if tokens.next().is_some() {
        return CssCaretShapeQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("auto") => {
            CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Auto)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("bar") => {
            CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Bar)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("block") => {
            CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Block)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("underscore") => {
            CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Underscore)
        }
        CssTokenKind::Ident(identifier) if is_css_wide_keyword(identifier) => {
            CssCaretShapeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCaretShapeUnsupportedReason::CssWideKeyword,
            )
        }
        _ => CssCaretShapeQualificationOutcome::InvalidForSelectedValueGrammar,
    }
}

fn animation_fill_mode_item_value(items: &[CssLexicalItem]) -> Option<CssAnimationFillModeValue> {
    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let token = tokens.next()?;
    if tokens.next().is_some() {
        return None;
    }

    match token.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("none") => {
            Some(CssAnimationFillModeValue::None)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("forwards") => {
            Some(CssAnimationFillModeValue::Forwards)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("backwards") => {
            Some(CssAnimationFillModeValue::Backwards)
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("both") => {
            Some(CssAnimationFillModeValue::Both)
        }
        _ => None,
    }
}

/// Qualifies one retained `animation-fill-mode` declaration value against
/// `<single-animation-fill-mode>#` (#641), where each item is exactly one
/// direct decoded Ident `none | forwards | backwards | both`.
///
/// Mirrors the accepted `animation-play-state` comma-list mechanics:
/// deferred substitution is checked before list recognition because it can
/// change top-level separator structure, a sole CSS-wide keyword is
/// unsupported only as the entire value, and the list walk splits only on
/// retained depth-zero `Comma` tokens while commas inside Functions or other
/// balanced blocks remain inside the current item. `auto` is not part of
/// this CSS property's grammar (it belongs only to the Web Animations
/// `FillMode` enum) and therefore falls through to
/// `InvalidForSelectedValueGrammar` like any other unrecognized Ident.
fn qualify_animation_fill_mode_value(
    items: &[CssLexicalItem],
) -> CssAnimationFillModeQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssAnimationFillModeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationFillModeUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssAnimationFillModeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationFillModeUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssAnimationFillModeQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssAnimationFillModeUnsupportedReason::CssWideKeyword,
        );
    }

    let mut values = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut item_start = 0usize;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty()
            && matches!(
                item,
                CssLexicalItem::SemanticToken(token)
                    if matches!(token.kind(), CssTokenKind::Comma)
            )
        {
            let Some(value) = animation_fill_mode_item_value(&items[item_start..index]) else {
                return CssAnimationFillModeQualificationOutcome::InvalidForSelectedValueGrammar;
            };
            values.push(value);
            item_start = index + 1;
            continue;
        }

        let CssLexicalItem::SemanticToken(token) = item else {
            continue;
        };
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

    let Some(value) = animation_fill_mode_item_value(&items[item_start..]) else {
        return CssAnimationFillModeQualificationOutcome::InvalidForSelectedValueGrammar;
    };
    values.push(value);

    CssAnimationFillModeQualificationOutcome::Qualified(values)
}

/// Partitions one selected `transform` function component's (`matrix()` or
/// `scale()`) retained function body into ordered semantic argument
/// slots, using delimiter depth measured relative to that function body
/// (#418 / #645).
///
/// This is the depth-scoped capability first proven for `matrix()` (#418)
/// and reused unchanged for `scale()` (#645), since both selected
/// functions share identical body-relative comma-partitioning mechanics;
/// this helper stays scoped to the `transform` property and does not
/// generalize to a cross-property argument parser. The walk starts one
/// item past the retained `Function` token -- which in CSS Syntax already
/// carries the opening parenthesis -- with a block stack seeded by that
/// function's own parenthesis, so `block_stack.len() == 1` is exactly "at
/// this function body's relative depth zero". Only a `Comma` retained at
/// that relative depth zero separates argument slots: a comma inside a
/// nested Function or a nested `(`/`[`/`{` block raises the depth first
/// and therefore stays inside the current slot, never changing the
/// selected function's arity. Every slot boundary is preserved exactly as
/// authored, so an empty authored position survives as its own empty slot
/// for the caller to reject rather than being collapsed away.
///
/// Termination follows retained parser/tokenizer evidence only, never a
/// raw-source scan: the body ends at the retained `RightParenthesis` that
/// returns the stack to relative depth zero, or -- when CSS Syntax
/// function consumption ended the extent at a true stylesheet EOF, so the
/// parser committed the occurrence with no authored closer -- at the end
/// of the retained component itself. No closing parenthesis is ever
/// synthesized, searched for, or reconstructed, and no source offset is
/// inferred.
///
/// A body holding no retained semantic content at all (`matrix()` or
/// `scale()`) yields zero slots rather than one empty slot, matching CSS
/// component-value list semantics; every other shape yields `commas + 1`
/// slots. Callers receive slot ranges relative to `component` and resolve
/// evidence positions by adding the component's own absolute start.
fn transform_function_body_slot_ranges(component: &[CssLexicalItem]) -> Vec<Range<usize>> {
    let Some(function_index) = component.iter().position(|item| {
        matches!(
            item,
            CssLexicalItem::SemanticToken(token)
                if matches!(token.kind(), CssTokenKind::Function(_))
        )
    }) else {
        return Vec::new();
    };

    let mut block_stack = vec![CssValueBlockCloser::Parenthesis];
    let mut slots: Vec<Range<usize>> = Vec::new();
    let mut slot_start = function_index + 1;
    let mut body_end = component.len();

    for (index, entry) in component.iter().enumerate().skip(function_index + 1) {
        let CssLexicalItem::SemanticToken(token) = entry else {
            // Comment trivia never opens, closes, or separates anything.
            continue;
        };

        if block_stack.len() == 1 {
            match token.kind() {
                CssTokenKind::Comma => {
                    slots.push(slot_start..index);
                    slot_start = index + 1;
                    continue;
                }
                CssTokenKind::RightParenthesis => {
                    body_end = index;
                    break;
                }
                _ => {}
            }
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

    slots.push(slot_start..body_end);

    if slots.len() == 1
        && !component[slots[0].clone()]
            .iter()
            .any(|item| matches!(item, CssLexicalItem::SemanticToken(_)))
    {
        return Vec::new();
    }

    slots
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssTransformMatrixArgumentClass {
    Number(CssTransformMatrixArgumentEvidenceRef),
    OpaqueFunction,
    Invalid,
}

/// Classifies one already-partitioned `matrix()` argument slot against the
/// selected profile's single accepted shape -- exactly one direct retained
/// `<number>` token (#418).
///
/// Whitespace and Comment trivia are excluded here exactly as the accepted
/// `counter-reset` inner-grammar recognition excludes them, so trivia
/// around a separator or an argument never changes slot interpretation. An
/// authored-empty slot has no retained semantic token and is decisively
/// `Invalid` -- it is rejected as an argument while still having been
/// preserved as an ordered position by `transform_function_body_slot_ranges`.
///
/// A slot headed by a `Function` token is never a direct `<number>`. It is
/// `OpaqueFunction` -- a structurally feasible numeric position whose
/// validity depends on calculated-value semantics this leaf does not own
/// and never evaluates -- only when the slot is exactly one complete
/// Function extent, which `entire_function_name` establishes from retained
/// structure. A Function followed by further retained material in the same
/// slot is directly visible structural failure and stays decisively
/// `Invalid`, so an unevaluated Function can never mask junk beside it.
/// So does a decoded name that is one of the recognized generic
/// whole-value-only functions occupying a non-whole-value position,
/// mirroring the accepted `scale` misplaced-whole-value boundary.
/// Deferred-substitution functions never reach here:
/// `qualify_transform_value` resolves them first, at whole-value scope and
/// at any nesting depth.
///
/// A direct `Number` qualifies regardless of `CssNumberType`, because
/// `matrix()` takes `<number>` and not `<integer>`; no range restriction
/// and no machine-float conversion is applied, so exact authored evidence
/// stays authoritative for membership. A `Dimension`, `Percentage`,
/// `Ident`, or `String` token is a direct token-category failure and is
/// decisively `Invalid`, as is any slot carrying more than one retained
/// semantic token.
fn classify_matrix_argument(
    slot: &[CssLexicalItem],
    absolute_slot_start: usize,
) -> CssTransformMatrixArgumentClass {
    let mut tokens = slot
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, first)) = tokens.next() else {
        return CssTransformMatrixArgumentClass::Invalid;
    };

    if matches!(first.kind(), CssTokenKind::Function(_)) {
        return match entire_function_name(slot) {
            Some(name) if is_whole_value_function(name) => CssTransformMatrixArgumentClass::Invalid,
            Some(_) => CssTransformMatrixArgumentClass::OpaqueFunction,
            None => CssTransformMatrixArgumentClass::Invalid,
        };
    }

    if tokens.next().is_some() {
        return CssTransformMatrixArgumentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Number { .. } => {
            CssTransformMatrixArgumentClass::Number(CssTransformMatrixArgumentEvidenceRef {
                lexical_item_index: absolute_slot_start + relative_index,
            })
        }
        _ => CssTransformMatrixArgumentClass::Invalid,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssTransformScaleArgumentClass {
    Number(CssTransformScaleArgumentEvidenceRef),
    Percentage(CssTransformScaleArgumentEvidenceRef),
    OpaqueFunction,
    Invalid,
}

/// Classifies one already-partitioned `scale()` argument slot against the
/// selected profile's two accepted shapes -- a direct retained `<number>`
/// or a direct retained `<percentage>` -- mirroring `classify_matrix_argument`
/// exactly except for the added `Percentage` branch (#645).
///
/// Whitespace and Comment trivia are excluded exactly as for `matrix()`
/// arguments, so trivia around a separator never changes slot
/// interpretation. An authored-empty slot has no retained semantic token
/// and is decisively `Invalid`. A slot headed by a `Function` token is
/// `OpaqueFunction` -- a structurally feasible numeric position whose
/// validity depends on calculated-value semantics this leaf does not own
/// -- only when the slot is exactly one complete Function extent; a
/// recognized generic whole-value-only Function name occupying this
/// non-whole-value position is decisively `Invalid` instead, mirroring the
/// accepted `matrix`/longhand-`scale` boundary. A Function followed by
/// further retained material in the same slot is directly visible
/// structural failure and stays decisively `Invalid`.
///
/// A direct `Number` or `Percentage` qualifies regardless of
/// `CssNumberType`; no range restriction and no machine-number conversion
/// is applied, so exact authored evidence stays authoritative for
/// membership and a `Percentage` is never collapsed into a `Number`. A
/// `Dimension`, `Ident`, or `String` token is a direct token-category
/// failure and is decisively `Invalid`, as is any slot carrying more than
/// one retained semantic token.
fn classify_transform_scale_argument(
    slot: &[CssLexicalItem],
    absolute_slot_start: usize,
) -> CssTransformScaleArgumentClass {
    let mut tokens = slot
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, first)) = tokens.next() else {
        return CssTransformScaleArgumentClass::Invalid;
    };

    if matches!(first.kind(), CssTokenKind::Function(_)) {
        return match entire_function_name(slot) {
            Some(name) if is_whole_value_function(name) => CssTransformScaleArgumentClass::Invalid,
            Some(_) => CssTransformScaleArgumentClass::OpaqueFunction,
            None => CssTransformScaleArgumentClass::Invalid,
        };
    }

    if tokens.next().is_some() {
        return CssTransformScaleArgumentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Number { .. } => {
            CssTransformScaleArgumentClass::Number(CssTransformScaleArgumentEvidenceRef {
                lexical_item_index: absolute_slot_start + relative_index,
            })
        }
        CssTokenKind::Percentage { .. } => {
            CssTransformScaleArgumentClass::Percentage(CssTransformScaleArgumentEvidenceRef {
                lexical_item_index: absolute_slot_start + relative_index,
            })
        }
        _ => CssTransformScaleArgumentClass::Invalid,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssTransformTranslate3dArgumentClass {
    Length(CssTransformTranslate3dArgumentEvidenceRef),
    Percentage(CssTransformTranslate3dArgumentEvidenceRef),
    OpaqueFunction,
    Invalid,
}

/// Classifies one already-partitioned `translate3d()` argument slot against
/// the selected profile's position-independent token shapes -- a direct
/// exact-zero `Number`, a `Dimension` with a recognized CSS length unit, or
/// a `Percentage` (#647). The caller applies the Z-slot `Percentage`
/// rejection after classification, since only slot placement -- not token
/// identity -- decides whether a `Percentage` is accepted; this mirrors
/// `classify_translate_component`'s positional unawareness from the
/// accepted longhand `translate` leaf (#606), reusing its exact-zero and
/// recognized-length-unit theorem rather than the longhand's component
/// type, since longhand property placement and `transform` function
/// argument placement remain distinct semantic roles.
///
/// Whitespace and Comment trivia are excluded exactly as for `matrix()`/
/// `scale()` arguments. An authored-empty slot has no retained semantic
/// token and is decisively `Invalid`. A slot headed by a `Function` token
/// is `OpaqueFunction` -- a structurally feasible position whose validity
/// depends on calculated-value semantics this leaf does not own -- only
/// when the slot is exactly one complete Function extent; a recognized
/// generic whole-value-only Function name occupying this non-whole-value
/// position is decisively `Invalid` instead, mirroring the accepted
/// `matrix`/`scale` boundary. A Function followed by further retained
/// material in the same slot is directly visible structural failure and
/// stays decisively `Invalid`.
///
/// A non-zero unitless `Number` and a `Dimension` with an unrecognized unit
/// (e.g. `deg`) both fall through to `Invalid`: this leaf never interprets
/// an arbitrary `Number` as `Length`, and no range restriction or
/// machine-number conversion is applied, so exact authored evidence stays
/// authoritative for membership. A `Dimension`, `Ident`, or `String` token
/// that is not a recognized length is a direct token-category failure and
/// is decisively `Invalid`, as is any slot carrying more than one retained
/// semantic token.
fn classify_transform_translate3d_argument(
    slot: &[CssLexicalItem],
    absolute_slot_start: usize,
) -> CssTransformTranslate3dArgumentClass {
    let mut tokens = slot
        .iter()
        .enumerate()
        .filter_map(|(relative_index, entry)| match entry {
            CssLexicalItem::SemanticToken(token)
                if !matches!(token.kind(), CssTokenKind::Whitespace) =>
            {
                Some((relative_index, token))
            }
            _ => None,
        });

    let Some((relative_index, first)) = tokens.next() else {
        return CssTransformTranslate3dArgumentClass::Invalid;
    };

    if matches!(first.kind(), CssTokenKind::Function(_)) {
        return match entire_function_name(slot) {
            Some(name) if is_whole_value_function(name) => {
                CssTransformTranslate3dArgumentClass::Invalid
            }
            Some(_) => CssTransformTranslate3dArgumentClass::OpaqueFunction,
            None => CssTransformTranslate3dArgumentClass::Invalid,
        };
    }

    if tokens.next().is_some() {
        return CssTransformTranslate3dArgumentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssTransformTranslate3dArgumentClass::Length(
                CssTransformTranslate3dArgumentEvidenceRef {
                    lexical_item_index: absolute_slot_start + relative_index,
                },
            )
        }
        CssTokenKind::Dimension { unit, .. } if is_css_length_unit(unit) => {
            CssTransformTranslate3dArgumentClass::Length(
                CssTransformTranslate3dArgumentEvidenceRef {
                    lexical_item_index: absolute_slot_start + relative_index,
                },
            )
        }
        CssTokenKind::Percentage { .. } => CssTransformTranslate3dArgumentClass::Percentage(
            CssTransformTranslate3dArgumentEvidenceRef {
                lexical_item_index: absolute_slot_start + relative_index,
            },
        ),
        _ => CssTransformTranslate3dArgumentClass::Invalid,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssTransformComponentClass {
    Matrix([CssTransformMatrixArgumentEvidenceRef; 6]),
    Scale(CssTransformScaleArguments),
    Translate3d(CssTransformTranslate3dFunction),
    OpaqueTransformArgument,
    UnselectedTransformFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `transform` component
/// against the selected profile `SelectedTransformFunction := Matrix |
/// Scale | Translate3d` (#418 / #645 / #647).
///
/// `<transform-list>` admits only `<transform-function>` components, so a
/// component that is not exactly one complete Function extent -- a bare
/// `Ident` such as a non-exclusive `none`, a stray top-level `Comma`, a
/// plain parenthesized block, or a Function followed by trailing material
/// -- is decisively `Invalid`. `entire_function_name` supplies that
/// whole-component Function test unchanged.
///
/// A Function named `matrix` ASCII-case-insensitively enters `matrix()`
/// argument qualification. Its directly visible structural shell is
/// decided first: a slot count other than six is decisive `Invalid`
/// before any argument content is consulted, so `matrix(calc(1),0)` and
/// `matrix(calc(1),0,0,1,0,0,2)` stay invalid on directly visible arity
/// rather than being deferred to unsupported calculated-value semantics.
/// Only at exactly six slots are arguments classified, and a decisive
/// argument failure (empty slot, wrong token category, multi-token slot,
/// misplaced whole-value Function) outranks any opaque Function found in a
/// sibling slot -- so `matrix(1,,calc(1),1,0,0)` and
/// `matrix(1px,calc(1),0,1,0,0)` remain `Invalid` regardless of slot order.
///
/// A Function named `scale` ASCII-case-insensitively enters `scale()`
/// argument qualification the same way: a directly visible slot count of
/// zero or more than two is decisive `Invalid` before any argument is
/// classified, so `scale()`, `scale(1,2,3)`, and `scale(calc(1),2,3)` all
/// stay invalid on directly visible arity. Only at exactly one or two
/// slots are arguments classified, with the same decisive-argument-over-
/// opaque-sibling precedence as `matrix()`, so `scale(1px,calc(1))` and
/// `scale(calc(1),1px)` both remain `Invalid` regardless of slot order.
/// The qualified one-vs-two argument cardinality is carried by
/// `CssTransformScaleArguments`, which cannot represent zero, three, or a
/// synthesized argument.
///
/// A Function named `translate3d` ASCII-case-insensitively enters
/// `translate3d()` argument qualification: a directly visible slot count
/// other than three is decisive `Invalid` before any argument is
/// classified, so `translate3d(1px,2px)` and
/// `translate3d(calc(1px),2px,3px,4px)` both stay invalid on directly
/// visible arity. Only at exactly three slots are arguments classified,
/// positionally: the first two (X, Y) accept `Length` or `Percentage`,
/// while the third (Z) accepts `Length` only, so a directly visible
/// `Percentage` in the Z slot is decisive `Invalid` even when an X or Y
/// slot holds an opaque Function -- `translate3d(calc(1px),2px,30%)` and
/// `translate3d(30%,calc(2px),30%)` both remain `Invalid` regardless of
/// which slot holds the opaque Function, since every slot's decisive
/// category failure is checked before any opaque-Function sibling is
/// consulted. The qualified `X`/`Y`/`Z` positional structure is carried by
/// `CssTransformTranslate3dFunction`, whose Z field has no `Percentage`
/// variant to hold, so a Z `Percentage` can never reach a qualified value.
///
/// Any other Function name is `UnselectedTransformFunction`: `rotate(1deg)`,
/// `matrix3d(...)`, `scaleX(...)`, and an unrecognized name alike stay
/// outside selected-profile coverage instead of being decided here,
/// because deciding them would require the full `<transform-function>`
/// dispatch and the length/angle semantics this leaf does not own. A
/// misplaced whole-value Function is the one exception: it has no
/// independent meaning inside a `<transform-list>` and is decisively
/// `Invalid`, mirroring the accepted `scale` boundary.
fn classify_transform_component(
    component: &[CssLexicalItem],
    absolute_component_start: usize,
) -> CssTransformComponentClass {
    let Some(name) = entire_function_name(component) else {
        return CssTransformComponentClass::Invalid;
    };

    if name.eq_ignore_ascii_case("matrix") {
        let slots = transform_function_body_slot_ranges(component);
        if slots.len() != 6 {
            return CssTransformComponentClass::Invalid;
        }

        let mut arguments = Vec::with_capacity(6);
        let mut has_opaque_argument = false;
        for slot in slots {
            let absolute_slot_start = absolute_component_start + slot.start;
            match classify_matrix_argument(&component[slot], absolute_slot_start) {
                CssTransformMatrixArgumentClass::Number(evidence) => arguments.push(evidence),
                CssTransformMatrixArgumentClass::OpaqueFunction => has_opaque_argument = true,
                CssTransformMatrixArgumentClass::Invalid => {
                    return CssTransformComponentClass::Invalid;
                }
            }
        }

        if has_opaque_argument {
            return CssTransformComponentClass::OpaqueTransformArgument;
        }

        return match <[CssTransformMatrixArgumentEvidenceRef; 6]>::try_from(arguments) {
            Ok(arguments) => CssTransformComponentClass::Matrix(arguments),
            Err(_) => CssTransformComponentClass::Invalid,
        };
    }

    if name.eq_ignore_ascii_case("scale") {
        let slots = transform_function_body_slot_ranges(component);
        if slots.is_empty() || slots.len() > 2 {
            return CssTransformComponentClass::Invalid;
        }

        let mut arguments = Vec::with_capacity(slots.len());
        let mut has_opaque_argument = false;
        for slot in slots {
            let absolute_slot_start = absolute_component_start + slot.start;
            match classify_transform_scale_argument(&component[slot], absolute_slot_start) {
                CssTransformScaleArgumentClass::Number(evidence_ref) => {
                    arguments.push(CssTransformScaleArgument {
                        kind: CssTransformScaleArgumentKind::Number,
                        evidence_ref,
                    });
                }
                CssTransformScaleArgumentClass::Percentage(evidence_ref) => {
                    arguments.push(CssTransformScaleArgument {
                        kind: CssTransformScaleArgumentKind::Percentage,
                        evidence_ref,
                    });
                }
                CssTransformScaleArgumentClass::OpaqueFunction => has_opaque_argument = true,
                CssTransformScaleArgumentClass::Invalid => {
                    return CssTransformComponentClass::Invalid;
                }
            }
        }

        if has_opaque_argument {
            return CssTransformComponentClass::OpaqueTransformArgument;
        }

        let arguments = match arguments.len() {
            1 => CssTransformScaleArguments::One(arguments[0]),
            2 => CssTransformScaleArguments::Two(arguments[0], arguments[1]),
            _ => unreachable!("scale slot count is already bounded to 1..=2 above"),
        };

        return CssTransformComponentClass::Scale(arguments);
    }

    if name.eq_ignore_ascii_case("translate3d") {
        let slots = transform_function_body_slot_ranges(component);
        if slots.len() != 3 {
            return CssTransformComponentClass::Invalid;
        }

        let x_start = absolute_component_start + slots[0].start;
        let y_start = absolute_component_start + slots[1].start;
        let z_start = absolute_component_start + slots[2].start;

        let x_class =
            classify_transform_translate3d_argument(&component[slots[0].clone()], x_start);
        let y_class =
            classify_transform_translate3d_argument(&component[slots[1].clone()], y_start);
        let z_class =
            classify_transform_translate3d_argument(&component[slots[2].clone()], z_start);

        // A directly visible category failure in any position is decisive,
        // including a Z-slot `Percentage`, which is never a valid `Length`
        // regardless of what any sibling slot contains (#647).
        let x_invalid = matches!(x_class, CssTransformTranslate3dArgumentClass::Invalid);
        let y_invalid = matches!(y_class, CssTransformTranslate3dArgumentClass::Invalid);
        let z_invalid = matches!(
            z_class,
            CssTransformTranslate3dArgumentClass::Invalid
                | CssTransformTranslate3dArgumentClass::Percentage(_)
        );
        if x_invalid || y_invalid || z_invalid {
            return CssTransformComponentClass::Invalid;
        }

        let has_opaque_argument = matches!(
            x_class,
            CssTransformTranslate3dArgumentClass::OpaqueFunction
        ) || matches!(
            y_class,
            CssTransformTranslate3dArgumentClass::OpaqueFunction
        ) || matches!(
            z_class,
            CssTransformTranslate3dArgumentClass::OpaqueFunction
        );
        if has_opaque_argument {
            return CssTransformComponentClass::OpaqueTransformArgument;
        }

        let to_xy_argument = |class: CssTransformTranslate3dArgumentClass| match class {
            CssTransformTranslate3dArgumentClass::Length(evidence_ref) => {
                CssTransformTranslate3dXyArgument {
                    kind: CssTransformTranslate3dXyArgumentKind::Length,
                    evidence_ref,
                }
            }
            CssTransformTranslate3dArgumentClass::Percentage(evidence_ref) => {
                CssTransformTranslate3dXyArgument {
                    kind: CssTransformTranslate3dXyArgumentKind::Percentage,
                    evidence_ref,
                }
            }
            CssTransformTranslate3dArgumentClass::OpaqueFunction
            | CssTransformTranslate3dArgumentClass::Invalid => {
                unreachable!("opaque and invalid X/Y classes are already handled above")
            }
        };

        let z = match z_class {
            CssTransformTranslate3dArgumentClass::Length(evidence_ref) => evidence_ref,
            CssTransformTranslate3dArgumentClass::Percentage(_)
            | CssTransformTranslate3dArgumentClass::OpaqueFunction
            | CssTransformTranslate3dArgumentClass::Invalid => {
                unreachable!("Z percentage/opaque/invalid classes are already handled above")
            }
        };

        return CssTransformComponentClass::Translate3d(CssTransformTranslate3dFunction {
            x: to_xy_argument(x_class),
            y: to_xy_argument(y_class),
            z,
        });
    }

    if is_whole_value_function(name) {
        CssTransformComponentClass::Invalid
    } else {
        CssTransformComponentClass::UnselectedTransformFunction
    }
}

/// Qualifies one retained `transform` declaration value against the
/// direct-authored profile `QualifiedDirectTransform := none | [
/// matrix(<number>#{6}) | scale([<number> | <percentage>]#{1,2}) |
/// translate3d(<length-percentage>, <length-percentage>, <length>) ]+`
/// (#418 / #645 / #647).
///
/// Outcome precedence follows evidence authority, never scan order.
/// Lower-layer lifecycle evidence is never touched here at all: this
/// function only ever sees a parser-committed ordinary declaration's
/// retained value window, so incomplete, resource-terminated, uncommitted,
/// and unsupported-region evidence stays owned by the tokenizer and parser
/// and is neither upgraded nor reconstructed. Above that, deferred
/// substitution is resolved first, because `var()` and its equivalents can
/// change the surrounding token sequence, separators, and cardinality --
/// the existing any-occurrence preflight already scans regardless of
/// nesting depth, so `matrix(var(--x),0)` is unsupported rather than
/// decided invalid on an arity that substitution may still change. The
/// whole-value Function boundary and the sole CSS-wide keyword boundary
/// are preserved exactly as for the other selected leaves.
///
/// A sole retained direct `none` Ident, ASCII-case-insensitively,
/// qualifies the dedicated whole-value branch and never reaches component
/// partitioning; `none` is never a `<transform-list>` component, so it is
/// decisively invalid combined with anything else, in either order.
///
/// Otherwise this single left-to-right recognition-time pass partitions
/// the value into ordered top-level components using depth-zero
/// Whitespace/Comment trivia as separators -- never raw-source whitespace
/// splitting, and never a `Comma`, since `<transform-list>` is
/// whitespace-separated repetition, so a top-level comma lands in its own
/// component and is decisively invalid there. Components are classified
/// the instant their block depth returns to zero, which also lets two
/// adjacent selected-function components with no authored whitespace
/// partition correctly, and preserves repeated and heterogeneous
/// `matrix()`/`scale()`/`translate3d()` components in exact authored order
/// via `CssTransformFunction`.
///
/// Resolution then applies the fixed precedence: any decisively invalid
/// component makes the declaration `InvalidForSelectedValueGrammar`
/// regardless of what any other component would have contributed;
/// otherwise an outer-level unselected `<transform-function>` is reported
/// before an inner-level opaque `matrix()`/`scale()`/`translate3d()`
/// argument, because the coarser grammar level bounds coverage first. Both
/// flags are collected across the whole component sequence before either
/// is reported, so the outcome never depends on which component was
/// encountered first, and a single shared `FunctionValuedTransformArgument`
/// reason keeps the opaque-argument outcome deterministic regardless of
/// which selected function kind held the opaque argument.
fn qualify_transform_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssTransformQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
    {
        if identifier.eq_ignore_ascii_case("none") {
            return CssTransformQualificationOutcome::Qualified(CssTransformValue::None);
        }
        if is_css_wide_keyword(identifier) {
            return CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformUnsupportedReason::CssWideKeyword,
            );
        }
    }

    let mut component_classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut component_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = component_start.take() {
                    component_classes.push(classify_transform_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if component_start.is_none() {
            component_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = component_start.take()
        {
            component_classes.push(classify_transform_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }
    if let Some(start) = component_start {
        component_classes.push(classify_transform_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    if component_classes.is_empty() {
        return CssTransformQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let mut functions = Vec::with_capacity(component_classes.len());
    let mut has_unselected_function = false;
    let mut has_opaque_argument = false;
    for class in component_classes {
        match class {
            CssTransformComponentClass::Matrix(arguments) => {
                functions.push(CssTransformFunction::Matrix(CssTransformMatrixFunction {
                    arguments,
                }));
            }
            CssTransformComponentClass::Scale(arguments) => {
                functions.push(CssTransformFunction::Scale(CssTransformScaleFunction {
                    arguments,
                }));
            }
            CssTransformComponentClass::Translate3d(function) => {
                functions.push(CssTransformFunction::Translate3d(function));
            }
            CssTransformComponentClass::OpaqueTransformArgument => has_opaque_argument = true,
            CssTransformComponentClass::UnselectedTransformFunction => {
                has_unselected_function = true;
            }
            CssTransformComponentClass::Invalid => {
                return CssTransformQualificationOutcome::InvalidForSelectedValueGrammar;
            }
        }
    }

    if has_unselected_function {
        return CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformUnsupportedReason::UnselectedTransformFunction,
        );
    }

    if has_opaque_argument {
        return CssTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTransformUnsupportedReason::FunctionValuedTransformArgument,
        );
    }

    CssTransformQualificationOutcome::Qualified(CssTransformValue::Functions(functions))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssTextIndentComponentClass {
    Length(CssTextIndentComponentEvidenceRef),
    Percentage(CssTextIndentComponentEvidenceRef),
    Hanging,
    EachLine,
    ResidualFunction,
    MisplacedWholeValueFunction,
    Invalid,
}

/// Classifies one already-partitioned top-level `text-indent` component
/// against the direct authored token shapes admitted anywhere in the finite
/// LP/H/E theorem (#615): a direct exact-zero `Number`, a `Dimension` with a
/// recognized CSS length unit, a `Percentage`, one of the two direct
/// optional keywords (ASCII-case-insensitively), or a Function-headed
/// component classified by placement/identity only, exactly as in
/// `transform-origin`/`translate`/`rotate`/`scale`. This classification is
/// role-unaware -- the caller applies the finite LP/H/E multiplicity theorem
/// after partitioning.
fn classify_text_indent_component(
    component: &[CssLexicalItem],
    absolute_component_start: usize,
) -> CssTextIndentComponentClass {
    let mut tokens =
        component
            .iter()
            .enumerate()
            .filter_map(|(relative_index, entry)| match entry {
                CssLexicalItem::SemanticToken(token)
                    if !matches!(token.kind(), CssTokenKind::Whitespace) =>
                {
                    Some((relative_index, token))
                }
                _ => None,
            });

    let Some((relative_index, first)) = tokens.next() else {
        return CssTextIndentComponentClass::Invalid;
    };

    if let CssTokenKind::Function(name) = first.kind() {
        return if is_whole_value_function(name) {
            CssTextIndentComponentClass::MisplacedWholeValueFunction
        } else {
            CssTextIndentComponentClass::ResidualFunction
        };
    }

    if tokens.next().is_some() {
        return CssTextIndentComponentClass::Invalid;
    }

    match first.kind() {
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("hanging") => {
            CssTextIndentComponentClass::Hanging
        }
        CssTokenKind::Ident(identifier) if identifier.eq_ignore_ascii_case("each-line") => {
            CssTextIndentComponentClass::EachLine
        }
        CssTokenKind::Number { value, .. } if is_direct_zero_numeric_value(value) => {
            CssTextIndentComponentClass::Length(CssTextIndentComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        CssTokenKind::Dimension { unit, .. } if is_css_length_unit(unit) => {
            CssTextIndentComponentClass::Length(CssTextIndentComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        CssTokenKind::Percentage { .. } => {
            CssTextIndentComponentClass::Percentage(CssTextIndentComponentEvidenceRef {
                lexical_item_index: absolute_component_start + relative_index,
            })
        }
        _ => CssTextIndentComponentClass::Invalid,
    }
}

/// Partitions an already-retained `text-indent` declaration value window
/// into ordered top-level components using one left-to-right recognition-
/// time pass and classifies each the instant its block depth returns to
/// zero, reusing the `transform-origin`/`translate`/`rotate`/`scale` block-
/// depth-aware walk. Depth-zero Whitespace/Comment lexical items are
/// separators -- never a `Comma`, since this grammar has no top-level comma
/// list -- and Function/bracket openers extend the current component until
/// their matching closer, so nested content (e.g. `calc(min(10px, 20px))`)
/// never inflates top-level cardinality. This intentionally duplicates the
/// equivalent walk rather than sharing it: this leaf keeps its own bounded
/// local recognition.
fn text_indent_top_level_component_classes(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> Vec<CssTextIndentComponentClass> {
    let mut classes = Vec::new();
    let mut block_stack: Vec<CssValueBlockCloser> = Vec::new();
    let mut component_start: Option<usize> = None;

    for (index, item) in items.iter().enumerate() {
        if block_stack.is_empty() {
            let is_separator = match item {
                CssLexicalItem::Comment(_) => true,
                CssLexicalItem::SemanticToken(token) => {
                    matches!(token.kind(), CssTokenKind::Whitespace)
                }
            };
            if is_separator {
                if let Some(start) = component_start.take() {
                    classes.push(classify_text_indent_component(
                        &items[start..index],
                        lexical_item_start + start,
                    ));
                }
                continue;
            }
        }

        if component_start.is_none() {
            component_start = Some(index);
        }

        if let CssLexicalItem::SemanticToken(token) = item {
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

        if block_stack.is_empty()
            && let Some(start) = component_start.take()
        {
            classes.push(classify_text_indent_component(
                &items[start..=index],
                lexical_item_start + start,
            ));
        }
    }

    if let Some(start) = component_start {
        classes.push(classify_text_indent_component(
            &items[start..],
            lexical_item_start + start,
        ));
    }

    classes
}

/// Qualifies one already-partitioned ordered sequence of top-level
/// `text-indent` component classes against the finite required-typed-
/// anchor-plus-unique-optional-keyword theorem (#615): exactly one direct
/// `<length-percentage>` anchor, zero or one `hanging`, and zero or one
/// `each-line`, in any authored order.
///
/// Any authored cardinality outside one to three, any `Invalid` or
/// `MisplacedWholeValueFunction` component, or any duplicate `hanging`,
/// duplicate `each-line`, or duplicate concrete anchor is decisive
/// `InvalidForSelectedValueGrammar` regardless of any residual Function
/// present -- structural/direct decisive invalidity always outranks a
/// provisional Function ambiguity. Otherwise, a lone concrete anchor
/// qualifies the value with authored order preserved exactly; a lone
/// residual Function standing in for the absent concrete anchor is
/// `UnsupportedBySelectedValueProfile(FunctionValue)`; any other
/// combination (a concrete anchor plus a residual Function competing for
/// the same single role, more than one residual Function, or no anchor at
/// all) is decisive `InvalidForSelectedValueGrammar`.
fn qualify_text_indent_component_classes(
    classes: &[CssTextIndentComponentClass],
) -> CssTextIndentQualificationOutcome {
    if classes.is_empty() || classes.len() > 3 {
        return CssTextIndentQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if classes.iter().any(|class| {
        matches!(
            class,
            CssTextIndentComponentClass::Invalid
                | CssTextIndentComponentClass::MisplacedWholeValueFunction
        )
    }) {
        return CssTextIndentQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    let concrete_anchor_count = classes
        .iter()
        .filter(|class| {
            matches!(
                class,
                CssTextIndentComponentClass::Length(_) | CssTextIndentComponentClass::Percentage(_)
            )
        })
        .count();
    let residual_function_count = classes
        .iter()
        .filter(|class| matches!(class, CssTextIndentComponentClass::ResidualFunction))
        .count();
    let hanging_count = classes
        .iter()
        .filter(|class| matches!(class, CssTextIndentComponentClass::Hanging))
        .count();
    let each_line_count = classes
        .iter()
        .filter(|class| matches!(class, CssTextIndentComponentClass::EachLine))
        .count();

    if hanging_count > 1 || each_line_count > 1 || concrete_anchor_count > 1 {
        return CssTextIndentQualificationOutcome::InvalidForSelectedValueGrammar;
    }

    if concrete_anchor_count == 1 {
        if residual_function_count > 0 {
            return CssTextIndentQualificationOutcome::InvalidForSelectedValueGrammar;
        }
        let components = classes
            .iter()
            .map(|class| match *class {
                CssTextIndentComponentClass::Length(evidence) => {
                    CssTextIndentComponent::Length(evidence)
                }
                CssTextIndentComponentClass::Percentage(evidence) => {
                    CssTextIndentComponent::Percentage(evidence)
                }
                CssTextIndentComponentClass::Hanging => CssTextIndentComponent::Hanging,
                CssTextIndentComponentClass::EachLine => CssTextIndentComponent::EachLine,
                CssTextIndentComponentClass::ResidualFunction
                | CssTextIndentComponentClass::MisplacedWholeValueFunction
                | CssTextIndentComponentClass::Invalid => {
                    unreachable!("residual/misplaced/invalid components are excluded above")
                }
            })
            .collect();
        return CssTextIndentQualificationOutcome::Qualified(CssTextIndentValue::Components(
            components,
        ));
    }

    if residual_function_count == 1 {
        return CssTextIndentQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextIndentUnsupportedReason::FunctionValue,
        );
    }

    CssTextIndentQualificationOutcome::InvalidForSelectedValueGrammar
}

/// Qualifies one retained `text-indent` declaration value against the
/// pin-bounded property-local finite `&&` composition theorem (#615 /
/// css-text-3, css-text-4 `text-indent`):
///
/// ```text
/// [ <length-percentage> ] && hanging? && each-line?
/// ```
///
/// Deferred substitution and the whole-value Function boundary are checked
/// first, exactly as for the other selected leaves; `text-indent` has no
/// dedicated whole-value keyword. A sole CSS-wide keyword preserves the
/// existing whole-value Unsupported boundary; an embedded CSS-wide
/// identifier is never treated as that whole-property boundary and instead
/// falls through to decisive component-level invalidity. Otherwise the
/// value is partitioned into ordered top-level components and dispatched
/// through the finite LP/H/E multiplicity theorem.
fn qualify_text_indent_value(
    items: &[CssLexicalItem],
    lexical_item_start: usize,
) -> CssTextIndentQualificationOutcome {
    if contains_deferred_substitution_function(items) {
        return CssTextIndentQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextIndentUnsupportedReason::DeferredSubstitutionFunction,
        );
    }

    if is_entire_whole_value_function(items) {
        return CssTextIndentQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextIndentUnsupportedReason::WholeValueFunction,
        );
    }

    let mut whole_value_tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });
    if let (Some(only_token), None) = (whole_value_tokens.next(), whole_value_tokens.next())
        && let CssTokenKind::Ident(identifier) = only_token.kind()
        && is_css_wide_keyword(identifier)
    {
        return CssTextIndentQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssTextIndentUnsupportedReason::CssWideKeyword,
        );
    }

    let classes = text_indent_top_level_component_classes(items, lexical_item_start);
    qualify_text_indent_component_classes(&classes)
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
