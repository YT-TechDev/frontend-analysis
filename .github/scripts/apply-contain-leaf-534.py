from pathlib import Path

VALUE = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
MOD = Path("crates/frontend-analysis-core/src/css/validation/mod.rs")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one anchor, found {count}")
    return text.replace(old, new, 1)


def replace_exact_count(text: str, old: str, new: str, expected: int, label: str) -> str:
    count = text.count(old)
    if count != expected:
        raise RuntimeError(f"{label}: expected {expected} anchors, found {count}")
    return text.replace(old, new)


text = VALUE.read_text()
text = replace_once(
    text,
    "/#530/#532).",
    "/#530/#532/#534).",
    "module issue list",
)

contain_types = r'''#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

'''
text = replace_once(
    text,
    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub(crate) enum CssWordSpacingValue {",
    contain_types + "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub(crate) enum CssWordSpacingValue {",
    "contain types",
)

text = replace_once(
    text,
    "    overscroll_behavior_observations: Vec<CssOverscrollBehaviorQualificationObservation>,\n",
    "    overscroll_behavior_observations: Vec<CssOverscrollBehaviorQualificationObservation>,\n    contain_observations: Vec<CssContainQualificationObservation>,\n",
    "run result field",
)

getter_anchor = '''    pub(crate) fn overscroll_behavior_observations(
        &self,
    ) -> &[CssOverscrollBehaviorQualificationObservation] {
        &self.overscroll_behavior_observations
    }
'''
getter = getter_anchor + '''
    pub(crate) fn contain_observations(&self) -> &[CssContainQualificationObservation] {
        &self.contain_observations
    }
'''
text = replace_once(text, getter_anchor, getter, "contain getter")

text = replace_exact_count(
    text,
    "        overscroll_behavior_observations,\n",
    "        overscroll_behavior_observations,\n        contain_observations,\n",
    3,
    "run tuple wiring",
)

text = replace_once(
    text,
    "        let mut overscroll_behavior_observations = Vec::new();\n",
    "        let mut overscroll_behavior_observations = Vec::new();\n        let mut contain_observations = Vec::new();\n",
    "contain vector",
)

dispatch_anchor = '''            if property_name.eq_ignore_ascii_case("overscroll-behavior") {
                let value_range = cursor.window_for(occurrence.value())?;
'''
contain_dispatch = '''            if property_name.eq_ignore_ascii_case("contain") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                contain_observations.push(CssContainQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_contain_value(value_items),
                });
                continue;
            }

'''
text = replace_once(
    text,
    dispatch_anchor,
    contain_dispatch + dispatch_anchor,
    "contain dispatch",
)

contain_qualifier = r'''fn qualify_contain_value(items: &[CssLexicalItem]) -> CssContainQualificationOutcome {
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

    if let [token] = tokens.as_slice() {
        if let CssTokenKind::Ident(identifier) = token.kind() {
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

    CssContainQualificationOutcome::Qualified(CssContainValue::Components(
        CssContainComponents { authored, count },
    ))
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

'''
text = replace_once(
    text,
    "fn qualify_overscroll_behavior_value(\n",
    contain_qualifier + "fn qualify_overscroll_behavior_value(\n",
    "contain qualifier",
)

VALUE.write_text(text)

mod_text = MOD.read_text()
mod_text = replace_once(
    mod_text,
    "#[cfg(test)]\nmod column_fill_value_qualification_tests;\n#[cfg(test)]\nmod conformance_tests;",
    "#[cfg(test)]\nmod column_fill_value_qualification_tests;\n#[cfg(test)]\nmod contain_value_qualification_tests;\n#[cfg(test)]\nmod conformance_tests;",
    "validation module",
)
MOD.write_text(mod_text)
