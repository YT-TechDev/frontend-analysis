from pathlib import Path

production = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
text = production.read_text()


def replace_exact(old: str, new: str, count: int = 1) -> None:
    global text
    actual = text.count(old)
    assert actual == count, (
        f"anchor count mismatch: expected {count}, got {actual}: {old[:160]!r}"
    )
    text = text.replace(old, new)


replace_exact(
    "//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483/#485/#487).\n",
    "//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483/#485/#487/#489).\n",
)

type_anchor = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWordSpacingValue {
"""
new_types = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

"""
replace_exact(type_anchor, new_types + type_anchor)

replace_exact(
    "    line_break_observations: Vec<CssLineBreakQualificationObservation>,\n"
    "    print_color_adjust_observations: Vec<CssPrintColorAdjustQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,\n",
    "    line_break_observations: Vec<CssLineBreakQualificationObservation>,\n"
    "    print_color_adjust_observations: Vec<CssPrintColorAdjustQualificationObservation>,\n"
    "    overflow_wrap_observations: Vec<CssOverflowWrapQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,\n",
)

getter_anchor = """    pub(crate) fn word_spacing_observations(&self) -> &[CssWordSpacingQualificationObservation] {
"""
new_getter = """    pub(crate) fn overflow_wrap_observations(
        &self,
    ) -> &[CssOverflowWrapQualificationObservation] {
        &self.overflow_wrap_observations
    }

"""
replace_exact(getter_anchor, new_getter + getter_anchor)

replace_exact(
    "        line_height_observations,\n"
    "        line_break_observations,\n"
    "        print_color_adjust_observations,\n"
    "        word_spacing_observations,\n",
    "        line_height_observations,\n"
    "        line_break_observations,\n"
    "        print_color_adjust_observations,\n"
    "        overflow_wrap_observations,\n"
    "        word_spacing_observations,\n",
    count=2,
)
replace_exact(
    "            line_height_observations,\n"
    "            line_break_observations,\n"
    "            print_color_adjust_observations,\n"
    "            word_spacing_observations,\n",
    "            line_height_observations,\n"
    "            line_break_observations,\n"
    "            print_color_adjust_observations,\n"
    "            overflow_wrap_observations,\n"
    "            word_spacing_observations,\n",
)

replace_exact(
    "        let mut line_break_observations = Vec::new();\n"
    "        let mut print_color_adjust_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();\n",
    "        let mut line_break_observations = Vec::new();\n"
    "        let mut print_color_adjust_observations = Vec::new();\n"
    "        let mut overflow_wrap_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();\n",
)

dispatch_anchor = """            if property_name.eq_ignore_ascii_case("word-spacing") {
"""
new_dispatch = """            if property_name.eq_ignore_ascii_case("overflow-wrap") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overflow_wrap_observations.push(CssOverflowWrapQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_overflow_wrap_value(value_items),
                });
                continue;
            }

"""
replace_exact(dispatch_anchor, new_dispatch + dispatch_anchor)

classifier_anchor = """fn qualify_word_spacing_value(
"""
new_classifier = """fn qualify_overflow_wrap_value(
    items: &[CssLexicalItem],
) -> CssOverflowWrapQualificationOutcome {
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

"""
replace_exact(classifier_anchor, new_classifier + classifier_anchor)

production.write_text(text)
