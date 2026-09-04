from pathlib import Path

path = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
text = path.read_text()


def replace_once(old: str, new: str) -> None:
    global text
    count = text.count(old)
    assert count == 1, f"expected exactly one anchor, got {count}: {old[:100]!r}"
    text = text.replace(old, new, 1)


def replace_exact_count(old: str, new: str, expected: int) -> None:
    global text
    count = text.count(old)
    assert count == expected, f"expected {expected} anchors, got {count}: {old[:100]!r}"
    text = text.replace(old, new)


replace_once(
    "semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483/#485/#487/#489).",
    "semantic Leaves (#413/#414/#416/#419/#422/#424/#426/#428/#432/#434/#436/#438/#440/#442/#444/#446/#448/#450/#452/#454/#457/#459/#463/#465/#467/#469/#471/#473/#475/#477/#479/#481/#483/#485/#487/#489/#491).",
)

overflow_wrap_observation = '''impl CssOverflowWrapQualificationObservation {
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
pub(crate) enum CssWordSpacingValue'''
unicode_bidi_types = '''impl CssOverflowWrapQualificationObservation {
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
pub(crate) enum CssWordSpacingValue'''
replace_once(overflow_wrap_observation, unicode_bidi_types)

replace_once(
    "    overflow_wrap_observations: Vec<CssOverflowWrapQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
    "    overflow_wrap_observations: Vec<CssOverflowWrapQualificationObservation>,\n"
    "    unicode_bidi_observations: Vec<CssUnicodeBidiQualificationObservation>,\n"
    "    word_spacing_observations: Vec<CssWordSpacingQualificationObservation>,",
)

replace_once(
    '''    pub(crate) fn overflow_wrap_observations(&self) -> &[CssOverflowWrapQualificationObservation] {
        &self.overflow_wrap_observations
    }

    pub(crate) fn word_spacing_observations(&self) -> &[CssWordSpacingQualificationObservation] {''',
    '''    pub(crate) fn overflow_wrap_observations(&self) -> &[CssOverflowWrapQualificationObservation] {
        &self.overflow_wrap_observations
    }

    pub(crate) fn unicode_bidi_observations(&self) -> &[CssUnicodeBidiQualificationObservation] {
        &self.unicode_bidi_observations
    }

    pub(crate) fn word_spacing_observations(&self) -> &[CssWordSpacingQualificationObservation] {''',
)

replace_exact_count(
    "        overflow_wrap_observations,\n        word_spacing_observations,",
    "        overflow_wrap_observations,\n        unicode_bidi_observations,\n        word_spacing_observations,",
    3,
)

replace_once(
    "        let mut overflow_wrap_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
    "        let mut overflow_wrap_observations = Vec::new();\n"
    "        let mut unicode_bidi_observations = Vec::new();\n"
    "        let mut word_spacing_observations = Vec::new();",
)

overflow_dispatch = '''            if property_name.eq_ignore_ascii_case("overflow-wrap") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overflow_wrap_observations.push(CssOverflowWrapQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_overflow_wrap_value(value_items),
                });
                continue;
            }

            if property_name.eq_ignore_ascii_case("word-spacing") {'''
unicode_dispatch = '''            if property_name.eq_ignore_ascii_case("overflow-wrap") {
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

            if property_name.eq_ignore_ascii_case("word-spacing") {'''
replace_once(overflow_dispatch, unicode_dispatch)

unicode_classifier = r'''
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
'''
replace_once(
    "\nfn qualify_word_spacing_value(items: &[CssLexicalItem]) -> CssWordSpacingQualificationOutcome {",
    unicode_classifier
    + "\nfn qualify_word_spacing_value(items: &[CssLexicalItem]) -> CssWordSpacingQualificationOutcome {",
)

path.write_text(text)
