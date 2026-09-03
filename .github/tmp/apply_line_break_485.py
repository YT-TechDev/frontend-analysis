from pathlib import Path

production = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
text = production.read_text()


def replace_exact(old: str, new: str, count: int = 1) -> None:
    global text
    actual = text.count(old)
    assert actual == count, (
        f"anchor count mismatch: expected {count}, got {actual}: {old[:100]!r}"
    )
    text = text.replace(old, new)


replace_exact(
    "//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483).\n",
    "//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483/#485).\n",
)

type_anchor = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWordSpacingValue {
"""
line_break_types = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

"""
replace_exact(type_anchor, line_break_types + type_anchor)

replace_exact(
    "    line_height_observations: Vec<CssLineHeightQualificationObservation>,\n    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,\n",
    "    line_height_observations: Vec<CssLineHeightQualificationObservation>,\n    line_break_observations: Vec<CssLineBreakQualificationObservation>,\n    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,\n",
)

getter_anchor = """    pub(crate) fn word_spacing_observations(&self) -> &[CssWordSpacingQualificationObservation] {
"""
line_break_getter = """    pub(crate) fn line_break_observations(&self) -> &[CssLineBreakQualificationObservation] {
        &self.line_break_observations
    }

"""
replace_exact(getter_anchor, line_break_getter + getter_anchor)

replace_exact(
    "        line_height_observations,\n        word_spacing_observations,\n",
    "        line_height_observations,\n        line_break_observations,\n        word_spacing_observations,\n",
    count=3,
)

replace_exact(
    "        let mut line_height_observations = Vec::new();\n        let mut word_spacing_observations = Vec::new();\n",
    "        let mut line_height_observations = Vec::new();\n        let mut line_break_observations = Vec::new();\n        let mut word_spacing_observations = Vec::new();\n",
)

dispatch_anchor = """            if property_name.eq_ignore_ascii_case("word-spacing") {
"""
line_break_dispatch = """            if property_name.eq_ignore_ascii_case("line-break") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                line_break_observations.push(CssLineBreakQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_line_break_value(value_items),
                });
                continue;
            }

"""
replace_exact(dispatch_anchor, line_break_dispatch + dispatch_anchor)

classifier_anchor = """fn qualify_word_spacing_value(
"""
line_break_classifier = """fn qualify_line_break_value(items: &[CssLexicalItem]) -> CssLineBreakQualificationOutcome {
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

"""
replace_exact(classifier_anchor, line_break_classifier + classifier_anchor)

production.write_text(text)
