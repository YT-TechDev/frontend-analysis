//! Bounded declaration-value qualification for selected post-freeze CSS
//! semantic Leaves (#413/#414/#416).
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
use super::token::{CssLexicalItem, CssTokenKind};
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
    let (direction_observations, box_sizing_observations) = {
        let tokenizer_result = parser_result.upstream_tokenizer_result();
        let mut cursor = LexicalWindowCursor::new(tokenizer_result);
        let mut direction_observations = Vec::new();
        let mut box_sizing_observations = Vec::new();

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
            }
        }

        (direction_observations, box_sizing_observations)
    };

    Ok(CssValueQualificationRunResult {
        upstream_parser_result: parser_result,
        direction_observations,
        box_sizing_observations,
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
    let mut contains_deferred_substitution_function = false;

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
            CssTokenKind::Function(name) => {
                contains_deferred_substitution_function |= is_deferred_substitution_function(name);
                only_identifier = None;
            }
            _ => {
                only_identifier = None;
            }
        }
    }

    // Arbitrary substitution functions make parse-time grammar validity
    // deferred wherever they occur. Generic `<whole-value>` functions cross
    // this slice's boundary only when that function occupies the entire value.
    if contains_deferred_substitution_function || is_entire_whole_value_function(items) {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CssValueBlockCloser {
    Parenthesis,
    SquareBracket,
    CurlyBracket,
}

fn is_entire_whole_value_function(items: &[CssLexicalItem]) -> bool {
    let mut tokens = items.iter().filter_map(|item| match item {
        CssLexicalItem::SemanticToken(token)
            if !matches!(token.kind(), CssTokenKind::Whitespace) =>
        {
            Some(token)
        }
        _ => None,
    });

    let Some(first) = tokens.next() else {
        return false;
    };
    let CssTokenKind::Function(name) = first.kind() else {
        return false;
    };
    if !is_whole_value_function(name) {
        return false;
    }

    let mut block_stack = vec![CssValueBlockCloser::Parenthesis];
    for token in tokens {
        if block_stack.is_empty() {
            return false;
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

    true
}

fn is_deferred_substitution_function(name: &str) -> bool {
    [
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
