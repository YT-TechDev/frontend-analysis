from pathlib import Path

production = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
text = production.read_text()


def replace_exact(old: str, new: str, count: int = 1) -> None:
    global text
    actual = text.count(old)
    assert actual == count, (
        f"anchor count mismatch: expected {count}, got {actual}: {old[:120]!r}"
    )
    text = text.replace(old, new)


replace_exact(
    "//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483/#485).\n",
    "//! semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483/#485/#487).\n",
)

type_anchor = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWordSpacingValue {
"""
new_types = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

"""
replace_exact(type_anchor, new_types + type_anchor)

replace_exact(
    "    line_break_observations: Vec<CssLineBreakQualificationObservation>,\n    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,\n",
    "    line_break_observations: Vec<CssLineBreakQualificationObservation>,\n    print_color_adjust_observations: Vec<CssPrintColorAdjustQualificationObservation>,\n    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,\n",
)

getter_anchor = """    pub(crate) fn word_spacing_observations(&self) -> &[CssWordSpacingQualificationObservation] {
"""
new_getter = """    pub(crate) fn print_color_adjust_observations(
        &self,
    ) -> &[CssPrintColorAdjustQualificationObservation] {
        &self.print_color_adjust_observations
    }

"""
replace_exact(getter_anchor, new_getter + getter_anchor)

replace_exact(
    "        line_height_observations,\n        line_break_observations,\n        word_spacing_observations,\n",
    "        line_height_observations,\n        line_break_observations,\n        print_color_adjust_observations,\n        word_spacing_observations,\n",
    count=2,
)
replace_exact(
    "            line_height_observations,\n            line_break_observations,\n            word_spacing_observations,\n",
    "            line_height_observations,\n            line_break_observations,\n            print_color_adjust_observations,\n            word_spacing_observations,\n",
)

replace_exact(
    "        let mut line_break_observations = Vec::new();\n        let mut word_spacing_observations = Vec::new();\n",
    "        let mut line_break_observations = Vec::new();\n        let mut print_color_adjust_observations = Vec::new();\n        let mut word_spacing_observations = Vec::new();\n",
)

dispatch_anchor = """            if property_name.eq_ignore_ascii_case("word-spacing") {
"""
new_dispatch = """            if property_name.eq_ignore_ascii_case("print-color-adjust") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                print_color_adjust_observations.push(CssPrintColorAdjustQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_print_color_adjust_value(value_items),
                });
                continue;
            }

"""
replace_exact(dispatch_anchor, new_dispatch + dispatch_anchor)

classifier_anchor = """fn qualify_word_spacing_value(items: &[CssLexicalItem]) -> CssWordSpacingQualificationOutcome {
"""
new_classifier = """fn qualify_print_color_adjust_value(
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

"""
replace_exact(classifier_anchor, new_classifier + classifier_anchor)

production.write_text(text)
