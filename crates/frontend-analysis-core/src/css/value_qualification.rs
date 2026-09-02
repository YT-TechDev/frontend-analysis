//! Bounded declaration-value qualification for selected post-freeze CSS
//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434).
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
    order_observations: Vec<CssOrderQualificationObservation>,
    column_count_observations: Vec<CssColumnCountQualificationObservation>,
    flex_grow_observations: Vec<CssFlexGrowQualificationObservation>,
    flex_shrink_observations: Vec<CssFlexShrinkQualificationObservation>,
    scroll_snap_align_observations: Vec<CssScrollSnapAlignQualificationObservation>,
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

    pub(crate) fn scroll_snap_align_observations(
        &self,
    ) -> &[CssScrollSnapAlignQualificationObservation] {
        &self.scroll_snap_align_observations
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
        order_observations,
        column_count_observations,
        flex_grow_observations,
        flex_shrink_observations,
        scroll_snap_align_observations,
        z_index_observations,
    ) = {
        let tokenizer_result = parser_result.upstream_tokenizer_result();
        let mut cursor = LexicalWindowCursor::new(tokenizer_result);
        let mut direction_observations = Vec::new();
        let mut box_sizing_observations = Vec::new();
        let mut isolation_observations = Vec::new();
        let mut order_observations = Vec::new();
        let mut column_count_observations = Vec::new();
        let mut flex_grow_observations = Vec::new();
        let mut flex_shrink_observations = Vec::new();
        let mut scroll_snap_align_observations = Vec::new();
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
            }
        }

        (
            direction_observations,
            box_sizing_observations,
            isolation_observations,
            order_observations,
            column_count_observations,
            flex_grow_observations,
            flex_shrink_observations,
            scroll_snap_align_observations,
            z_index_observations,
        )
    };

    Ok(CssValueQualificationRunResult {
        upstream_parser_result: parser_result,
        direction_observations,
        box_sizing_observations,
        isolation_observations,
        order_observations,
        column_count_observations,
        flex_grow_observations,
        flex_shrink_observations,
        scroll_snap_align_observations,
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
