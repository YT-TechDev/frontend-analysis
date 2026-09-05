from pathlib import Path

VALUE = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
MOD = Path("crates/frontend-analysis-core/src/css/validation/mod.rs")
TEST = Path("crates/frontend-analysis-core/src/css/validation/font_variant_numeric_value_qualification_tests.rs")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, got {count}")
    return text.replace(old, new, 1)


text = VALUE.read_text()
text = replace_once(
    text,
    "/#532/#534/#536).",
    "/#532/#534/#536/#538).",
    "module provenance",
)

observation_anchor = """    pub(crate) const fn outcome(&self) -> CssFontVariantLigaturesQualificationOutcome {
        self.outcome
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
"""

numeric_types = """    pub(crate) const fn outcome(&self) -> CssFontVariantLigaturesQualificationOutcome {
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
"""
text = replace_once(text, observation_anchor, numeric_types, "numeric types")

field = "    font_variant_ligatures_observations: Vec<CssFontVariantLigaturesQualificationObservation>,\n"
text = replace_once(
    text,
    field,
    field + "    font_variant_numeric_observations: Vec<CssFontVariantNumericQualificationObservation>,\n",
    "result field",
)

getter = """    pub(crate) fn font_variant_ligatures_observations(
        &self,
    ) -> &[CssFontVariantLigaturesQualificationObservation] {
        &self.font_variant_ligatures_observations
    }

"""
text = replace_once(
    text,
    getter,
    getter
    + """    pub(crate) fn font_variant_numeric_observations(
        &self,
    ) -> &[CssFontVariantNumericQualificationObservation] {
        &self.font_variant_numeric_observations
    }

""",
    "result getter",
)

var = "        let mut font_variant_ligatures_observations = Vec::new();\n"
text = replace_once(
    text,
    var,
    var + "        let mut font_variant_numeric_observations = Vec::new();\n",
    "observation accumulator",
)

carry = "        font_variant_ligatures_observations,\n"
carry_count = text.count(carry)
if carry_count < 1:
    raise SystemExit("result carry: no matches")
text = text.replace(
    carry,
    carry + "        font_variant_numeric_observations,\n",
)

nested_carry = "            font_variant_ligatures_observations,\n"
# The 12-space tuple arm is distinct from the 8-space replacements above.
if nested_carry in text and "            font_variant_numeric_observations,\n" not in text:
    text = text.replace(
        nested_carry,
        nested_carry + "            font_variant_numeric_observations,\n",
    )

dispatch_anchor = """            if property_name.eq_ignore_ascii_case(\"overscroll-behavior-x\") {
"""
numeric_dispatch = """            if property_name.eq_ignore_ascii_case(\"font-variant-numeric\") {
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

"""
text = replace_once(
    text,
    dispatch_anchor,
    numeric_dispatch + dispatch_anchor,
    "property dispatch",
)

qualifier_anchor = "fn qualify_overscroll_behavior_value("
numeric_qualifier = r'''fn qualify_font_variant_numeric_value(
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

    CssFontVariantNumericQualificationOutcome::Qualified(
        CssFontVariantNumericValue::Components(CssFontVariantNumericComponents {
            authored,
            count,
        }),
    )
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

'''
text = replace_once(
    text,
    qualifier_anchor,
    numeric_qualifier + qualifier_anchor,
    "qualifier insertion",
)
VALUE.write_text(text)

mod_text = MOD.read_text()
mod_anchor = """#[cfg(test)]
mod font_variant_ligatures_value_qualification_tests;
#[cfg(test)]
mod font_variant_position_value_qualification_tests;
"""
mod_text = replace_once(
    mod_text,
    mod_anchor,
    """#[cfg(test)]
mod font_variant_ligatures_value_qualification_tests;
#[cfg(test)]
mod font_variant_numeric_value_qualification_tests;
#[cfg(test)]
mod font_variant_position_value_qualification_tests;
""",
    "test module registration",
)
MOD.write_text(mod_text)

TEST.write_text(r'''use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontVariantNumericComponent, CssFontVariantNumericQualificationOutcome,
    CssFontVariantNumericUnsupportedReason, CssFontVariantNumericValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Components(&'static [CssFontVariantNumericComponent]),
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferredFunction,
    UnsupportedWholeValueFunction,
}

fn tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(4096, 100_000, 8192, 1024, 8192, 8192).unwrap()
}

fn parser_limits() -> CssParserLimits {
    parser_limits_with_occurrences(8192)
}

fn parser_limits_with_occurrences(max_declaration_occurrences: usize) -> CssParserLimits {
    CssParserLimits::new(
        100_000,
        256,
        256,
        max_declaration_occurrences,
        1024,
        1024,
        1024,
        1024,
        8192,
    )
    .unwrap()
}

fn qualify(source_id: u64, css: &str) -> CssValueQualificationRunResult {
    qualify_with_limits(source_id, css, parser_limits())
}

fn qualify_with_limits(
    source_id: u64,
    css: &str,
    parser_limits: CssParserLimits,
) -> CssValueQualificationRunResult {
    let source = SourceText::new(SourceId::new(source_id), css.to_owned());
    let parser_result = analyze_css_source(&source, tokenizer_limits(), parser_limits).unwrap();
    run(parser_result).unwrap()
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual = result.font_variant_numeric_observations();
    assert_eq!(actual.len(), expected.len());

    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssFontVariantNumericQualificationOutcome::Qualified(
                    CssFontVariantNumericValue::Normal,
                ),
                ExpectedOutcome::Normal,
            ) => {}
            (
                CssFontVariantNumericQualificationOutcome::Qualified(
                    CssFontVariantNumericValue::Components(components),
                ),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssFontVariantNumericQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantNumericUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantNumericUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssFontVariantNumericQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontVariantNumericUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => panic!(
                "unexpected font-variant-numeric outcome: {actual:?}, expected {expected:?}"
            ),
        }
    }
}

#[test]
fn handwritten_fixed_slot_boundary_matches_pinned_wpt_and_derived_theorem() {
    use CssFontVariantNumericComponent::{
        DiagonalFractions, LiningNums, OldstyleNums, Ordinal, ProportionalNums, SlashedZero,
        StackedFractions, TabularNums,
    };

    let result = qualify(
        85_600,
        concat!(
            "a{font-variant-numeric:normal;}",
            "b{font-variant-numeric:lining-nums;}",
            "c{font-variant-numeric:oldstyle-nums;}",
            "d{font-variant-numeric:proportional-nums;}",
            "e{font-variant-numeric:tabular-nums;}",
            "f{font-variant-numeric:diagonal-fractions;}",
            "g{font-variant-numeric:stacked-fractions;}",
            "h{font-variant-numeric:ordinal;}",
            "i{font-variant-numeric:slashed-zero;}",
            "j{font-variant-numeric:proportional-nums slashed-zero diagonal-fractions oldstyle-nums ordinal;}",
            "k{font-variant-numeric:lining-nums oldstyle-nums;}",
            "l{font-variant-numeric:proportional-nums tabular-nums;}",
            "m{font-variant-numeric:diagonal-fractions stacked-fractions;}",
            "n{font-variant-numeric:ordinal ordinal;}",
            "o{font-variant-numeric:slashed-zero slashed-zero;}",
            "p{font-variant-numeric:normal lining-nums;}",
            "q{font-variant-numeric:auto;}",
            "r{font-variant-numeric:0;}",
            "s{font-variant-numeric:\"lining-nums\";}",
            "t{font-variant-numeric:;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Components(&[LiningNums]),
            ExpectedOutcome::Components(&[OldstyleNums]),
            ExpectedOutcome::Components(&[ProportionalNums]),
            ExpectedOutcome::Components(&[TabularNums]),
            ExpectedOutcome::Components(&[DiagonalFractions]),
            ExpectedOutcome::Components(&[StackedFractions]),
            ExpectedOutcome::Components(&[Ordinal]),
            ExpectedOutcome::Components(&[SlashedZero]),
            ExpectedOutcome::Components(&[
                ProportionalNums,
                SlashedZero,
                DiagonalFractions,
                OldstyleNums,
                Ordinal,
            ]),
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn authored_order_is_preserved_without_canonicalization() {
    use CssFontVariantNumericComponent::{OldstyleNums, ProportionalNums, SlashedZero, TabularNums};

    let result = qualify(
        85_601,
        concat!(
            "a{font-variant-numeric:proportional-nums oldstyle-nums;}",
            "b{font-variant-numeric:oldstyle-nums proportional-nums;}",
            "c{font-variant-numeric:slashed-zero tabular-nums;}",
            "d{font-variant-numeric:tabular-nums slashed-zero;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[ProportionalNums, OldstyleNums]),
            ExpectedOutcome::Components(&[OldstyleNums, ProportionalNums]),
            ExpectedOutcome::Components(&[SlashedZero, TabularNums]),
            ExpectedOutcome::Components(&[TabularNums, SlashedZero]),
        ],
    );
}

#[test]
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssFontVariantNumericComponent::{LiningNums, OldstyleNums, SlashedZero, TabularNums};

    let result = qualify(
        85_602,
        concat!(
            "a{FONT-VARIANT-NUMERIC:LINING-NUMS TABULAR-NUMS;}",
            r"b{font-variant-numeric:lining-num\73;}",
            r"c{font-variant-numeri\63:oldstyle-nums;}",
            "d{font-variant-numeric:/**/slashed-zero/**/tabular-nums/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[LiningNums, TabularNums]),
            ExpectedOutcome::Components(&[LiningNums]),
            ExpectedOutcome::Components(&[OldstyleNums]),
            ExpectedOutcome::Components(&[SlashedZero, TabularNums]),
        ],
    );

    let priority_observation = &result.font_variant_numeric_observations()[3];
    let occurrence =
        &result.upstream_parser_result().occurrences()[priority_observation.occurrence_index()];
    assert_eq!(priority_observation.placement(), occurrence.placement());
    assert!(occurrence.priority().is_some());
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        85_603,
        concat!(
            "a{font-variant-numeric:initial;}",
            "b{font-variant-numeric:inherit;}",
            "c{font-variant-numeric:unset;}",
            "d{font-variant-numeric:revert;}",
            "e{font-variant-numeric:revert-layer;}",
            "f{font-variant-numeric:revert-rule;}",
            "g{font-variant-numeric:inherit ordinal;}",
            "h{font-variant-numeric:ordinal inherit;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn deferred_functions_fail_open_anywhere_while_other_functions_keep_existing_boundaries() {
    let result = qualify(
        85_604,
        concat!(
            "a{font-variant-numeric:var(--numeric);}",
            "b{font-variant-numeric:lining-nums var(--numeric);}",
            "c{font-variant-numeric:var(--numeric) ordinal;}",
            "d{font-variant-numeric:foo(var(--numeric));}",
            "e{font-variant-numeric:first-valid(lining-nums,ordinal);}",
            "f{font-variant-numeric:cycle(lining-nums,ordinal);}",
            "g{font-variant-numeric:interpolate(0%,0:lining-nums,1:ordinal);}",
            "h{font-variant-numeric:lining-nums first-valid(ordinal);}",
            "i{font-variant-numeric:foo();}",
            "j{font-variant-numeric:lining-nums foo();}",
            "k{font-variant-numeric:calc(1);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn applicability_and_cross_dispatch_remain_isolated() {
    use CssFontVariantNumericComponent::{OldstyleNums, TabularNums};

    let result = qualify(
        85_605,
        concat!(
            "a{font-variant-ligatures:common-ligatures;}",
            "b{contain:layout size;}",
            "table{font-variant-numeric:oldstyle-nums tabular-nums;}",
            "d{font-variant-position:sub;}",
        ),
    );

    assert_eq!(result.font_variant_ligatures_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(result.font_variant_numeric_observations().len(), 1);
    assert_eq!(result.font_variant_position_observations().len(), 1);
    assert_eq!(result.font_variant_numeric_observations()[0].occurrence_index(), 2);
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[OldstyleNums, TabularNums])],
    );
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    use CssFontVariantNumericComponent::Ordinal;

    let result = qualify(
        85_606,
        "a{font-variant-numeric:ordinal;}b{font-variant-numeric:ordinal;}",
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Ordinal]),
            ExpectedOutcome::Components(&[Ordinal]),
        ],
    );
    assert_ne!(
        result.font_variant_numeric_observations()[0]
            .placement()
            .context_id(),
        result.font_variant_numeric_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (85_610, "@font-face{font-variant-numeric:ordinal;}"),
        (85_611, "@page{font-variant-numeric:ordinal;}"),
        (85_612, "@page{@top-left{font-variant-numeric:ordinal;}}"),
        (85_613, "@keyframes k{from{font-variant-numeric:ordinal;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_variant_numeric_observations().is_empty(),
            "unexpected observation for {css}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssFontVariantNumericComponent::{OldstyleNums, TabularNums};

    let result = qualify_with_limits(
        85_620,
        concat!(
            "a{font-variant-numeric:oldstyle-nums tabular-nums;",
            "font-variant-numeric:tabular-nums oldstyle-nums;}"
        ),
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[OldstyleNums, TabularNums])],
    );
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{font-variant-numeric:oldstyle-nums tabular-nums ordinal;}",
        "b{font-variant-numeric:normal;}",
        "c{font-variant-numeric:inherit;}",
        "d{font-variant-numeric:var(--numeric);}",
        "e{font-variant-numeric:lining-nums oldstyle-nums;}",
    );

    let first = qualify(85_621, css);
    let repeated = qualify(85_621, css);
    assert_eq!(
        first.font_variant_numeric_observations(),
        repeated.font_variant_numeric_observations()
    );

    let other_source = qualify(85_622, css);
    let first_outcomes: Vec<_> = first
        .font_variant_numeric_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let other_outcomes: Vec<_> = other_source
        .font_variant_numeric_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(first_outcomes, other_outcomes);
}
''')
