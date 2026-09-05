from pathlib import Path

production_path = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
mod_path = Path("crates/frontend-analysis-core/src/css/validation/mod.rs")
fill_test_path = Path("crates/frontend-analysis-core/src/css/validation/fill_rule_value_qualification_tests.rs")
column_test_path = Path("crates/frontend-analysis-core/src/css/validation/column_fill_value_qualification_tests.rs")


def replace_exact(text, old, new, count=1):
    actual = text.count(old)
    if actual != count:
        raise SystemExit(f"expected {count} occurrence(s), found {actual}: {old[:100]!r}")
    return text.replace(old, new)


production = production_path.read_text()
production = replace_exact(production, "#516/#518).", "#516/#518/#520).")

word_spacing_type_anchor = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWordSpacingValue {
"""
column_fill_types = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

"""
production = replace_exact(
    production,
    word_spacing_type_anchor,
    column_fill_types + word_spacing_type_anchor,
)

production = replace_exact(
    production,
    "    fill_rule_observations: Vec<CssFillRuleQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
    "    fill_rule_observations: Vec<CssFillRuleQualificationObservation>,\n"
    "    column_fill_observations: Vec<CssColumnFillQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
)

getter_anchor = """    pub(crate) fn fill_rule_observations(&self) -> &[CssFillRuleQualificationObservation] {
        &self.fill_rule_observations
    }

"""
getter = """    pub(crate) fn column_fill_observations(&self) -> &[CssColumnFillQualificationObservation] {
        &self.column_fill_observations
    }

"""
production = replace_exact(production, getter_anchor, getter_anchor + getter)

production = replace_exact(
    production,
    "        fill_rule_observations,\n        word_spacing_observations,",
    "        fill_rule_observations,\n        column_fill_observations,\n        word_spacing_observations,",
    count=3,
)

production = replace_exact(
    production,
    "        let mut fill_rule_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
    "        let mut fill_rule_observations = Vec::new();\n"
    "        let mut column_fill_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
)

word_spacing_dispatch_anchor = '            if property_name.eq_ignore_ascii_case("word-spacing") {\n'
column_fill_dispatch = """            if property_name.eq_ignore_ascii_case("column-fill") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                column_fill_observations.push(CssColumnFillQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_column_fill_value(value_items),
                });
                continue;
            }

"""
production = replace_exact(
    production,
    word_spacing_dispatch_anchor,
    column_fill_dispatch + word_spacing_dispatch_anchor,
)

word_spacing_qualifier_anchor = "fn qualify_word_spacing_value(items: &[CssLexicalItem]) -> CssWordSpacingQualificationOutcome {\n"
column_fill_qualifier = """fn qualify_column_fill_value(items: &[CssLexicalItem]) -> CssColumnFillQualificationOutcome {
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

"""
production = replace_exact(
    production,
    word_spacing_qualifier_anchor,
    column_fill_qualifier + word_spacing_qualifier_anchor,
)
production_path.write_text(production)

mod_text = mod_path.read_text()
mod_text = replace_exact(
    mod_text,
    "#[cfg(test)]\nmod column_count_value_qualification_tests;",
    "#[cfg(test)]\nmod column_fill_value_qualification_tests;\n"
    "#[cfg(test)]\nmod column_count_value_qualification_tests;",
)
mod_path.write_text(mod_text)

tests = fill_test_path.read_text()
tests = tests.replace("CssFillRule", "CssColumnFill")
tests = tests.replace("fill_rule", "column_fill")
tests = tests.replace("fill-rule", "column-fill")
tests = tests.replace("ExpectedOutcome::Nonzero", "ExpectedOutcome::Auto")
tests = tests.replace("ExpectedOutcome::Evenodd", "ExpectedOutcome::Balance")
tests = tests.replace("CssColumnFillValue::Nonzero", "CssColumnFillValue::Auto")
tests = tests.replace("CssColumnFillValue::Evenodd", "CssColumnFillValue::Balance")
tests = replace_exact(
    tests,
    "    Nonzero,\n    Evenodd,",
    "    Auto,\n    Balance,\n    BalanceAll,",
)
tests = replace_exact(
    tests,
    """        ExpectedOutcome::Balance => {
            CssColumnFillQualificationOutcome::Qualified(CssColumnFillValue::Balance)
        }
""",
    """        ExpectedOutcome::Balance => {
            CssColumnFillQualificationOutcome::Qualified(CssColumnFillValue::Balance)
        }
        ExpectedOutcome::BalanceAll => {
            CssColumnFillQualificationOutcome::Qualified(CssColumnFillValue::BalanceAll)
        }
""",
)

tests = tests.replace("nonzero", "auto")
tests = tests.replace("evenodd", "balance")

tests = replace_exact(
    tests,
    '            "c{FILL-RULE:EvEnOdD;}",',
    '            "c{COLUMN-FILL:BaLaNcE-AlL;}",',
)
tests = replace_exact(
    tests,
    r'            r"d{column-fill:\6e onzero;}",',
    r'            r"d{column-fill:\61 uto;}",',
)
tests = replace_exact(
    tests,
    r'            r"e{fill-\72 ule:balance;}",',
    r'            r"e{column-\66 ill:balance;}",',
)
tests = replace_exact(
    tests,
    '            "f{column-fill:auto;}",',
    '            "f{column-fill:none;}",',
)
tests = replace_exact(
    tests,
    '            "l{clip-rule:balance;}",',
    '            "l{clip-rule:evenodd;}",',
)

tests = replace_exact(
    tests,
    """            ExpectedOutcome::Auto,
            ExpectedOutcome::Balance,
            ExpectedOutcome::Balance,
            ExpectedOutcome::Auto,
""",
    """            ExpectedOutcome::Auto,
            ExpectedOutcome::Balance,
            ExpectedOutcome::BalanceAll,
            ExpectedOutcome::Auto,
""",
)

tests = replace_exact(
    tests,
    "fn element_kind_and_svg_applicability_are_not_inputs_to_qualification()",
    "fn element_kind_and_multicol_applicability_are_not_inputs_to_qualification()",
)
tests = replace_exact(
    tests,
    """            "path{column-fill:balance;}",
            "div{column-fill:balance;}",
            "svg{column-fill:balance;}",
""",
    """            "div{column-fill:balance;}",
            "section{column-fill:balance;}",
            "span{column-fill:balance;}",
""",
)

tests = replace_exact(
    tests,
    """            "av{clip-rule:balance;}",
            "aw{column-fill:balance;}",
""",
    """            "av{clip-rule:evenodd;}",
            "aw{fill-rule:evenodd;}",
            "ax{column-fill:balance-all;}",
""",
)

tests = replace_exact(
    tests,
    """    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.column_fill_observations().len(), 1);
    assert_eq!(result.column_fill_observations()[0].occurrence_index(), 48);
    assert_expected(&result, &[ExpectedOutcome::Balance]);
""",
    """    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.fill_rule_observations().len(), 1);
    assert_eq!(result.column_fill_observations().len(), 1);
    assert_eq!(result.column_fill_observations()[0].occurrence_index(), 49);
    assert_expected(&result, &[ExpectedOutcome::BalanceAll]);
""",
)

tests = replace_exact(
    tests,
    '        "c{column-fill:auto;}",',
    '        "c{column-fill:none;}",',
)

required_test_fragments = [
    "CssColumnFillQualificationOutcome",
    "CssColumnFillValue::Auto",
    "CssColumnFillValue::Balance",
    "CssColumnFillValue::BalanceAll",
    r"column-fill:\61 uto",
    r"column-\66 ill:balance",
    '"aw{fill-rule:evenodd;}"',
    '"ax{column-fill:balance-all;}"',
    "column_fill_observations()[0].occurrence_index(), 49",
]
for fragment in required_test_fragments:
    if fragment not in tests:
        raise SystemExit(f"missing expected column-fill test fragment: {fragment!r}")

if "Nonzero" in tests or "Evenodd" in tests:
    raise SystemExit("stale fill-rule enum material remained in column-fill test")

column_test_path.write_text(tests)
