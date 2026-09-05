from pathlib import Path

VALUE_PATH = Path("crates/frontend-analysis-core/src/css/value_qualification.rs")
MOD_PATH = Path("crates/frontend-analysis-core/src/css/validation/mod.rs")
TEST_PATH = Path(
    "crates/frontend-analysis-core/src/css/validation/"
    "overscroll_behavior_shorthand_value_qualification_tests.rs"
)

text = VALUE_PATH.read_text()


def replace_exact(old: str, new: str, expected: int = 1) -> None:
    global text
    count = text.count(old)
    if count != expected:
        raise SystemExit(
            f"anchor count mismatch: expected {expected}, got {count}: {old[:120]!r}"
        )
    text = text.replace(old, new)


replace_exact("#524/#526/#528/#530).", "#524/#526/#528/#530/#532).")

type_marker = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssWordSpacingValue {
"""
type_block = """#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

"""
replace_exact(type_marker, type_block + type_marker)

field_marker = """    text_decoration_skip_ink_observations: Vec<CssTextDecorationSkipInkQualificationObservation>,
    overscroll_behavior_x_observations: Vec<CssOverscrollBehaviorXQualificationObservation>,
"""
replace_exact(
    field_marker,
    """    text_decoration_skip_ink_observations: Vec<CssTextDecorationSkipInkQualificationObservation>,
    overscroll_behavior_observations: Vec<CssOverscrollBehaviorQualificationObservation>,
    overscroll_behavior_x_observations: Vec<CssOverscrollBehaviorXQualificationObservation>,
""",
)

getter_marker = """    pub(crate) fn overscroll_behavior_x_observations(
"""
getter = """    pub(crate) fn overscroll_behavior_observations(
        &self,
    ) -> &[CssOverscrollBehaviorQualificationObservation] {
        &self.overscroll_behavior_observations
    }

"""
replace_exact(getter_marker, getter + getter_marker)

tuple_marker = """        text_decoration_skip_ink_observations,
        overscroll_behavior_x_observations,
"""
replace_exact(
    tuple_marker,
    """        text_decoration_skip_ink_observations,
        overscroll_behavior_observations,
        overscroll_behavior_x_observations,
""",
    expected=3,
)

init_marker = """        let mut text_decoration_skip_ink_observations = Vec::new();
        let mut overscroll_behavior_x_observations = Vec::new();
"""
replace_exact(
    init_marker,
    """        let mut text_decoration_skip_ink_observations = Vec::new();
        let mut overscroll_behavior_observations = Vec::new();
        let mut overscroll_behavior_x_observations = Vec::new();
""",
)

dispatch_marker = """            if property_name.eq_ignore_ascii_case(\"overscroll-behavior-x\") {
"""
dispatch = """            if property_name.eq_ignore_ascii_case(\"overscroll-behavior\") {
                let value_range = cursor.window_for(occurrence.value())?;
                let value_items = &tokenizer_result.lexical_items()[value_range];
                overscroll_behavior_observations.push(CssOverscrollBehaviorQualificationObservation {
                    occurrence_index,
                    placement: occurrence.placement(),
                    outcome: qualify_overscroll_behavior_value(value_items),
                });
                continue;
            }

"""
replace_exact(dispatch_marker, dispatch + dispatch_marker)

qualifier_marker = """fn qualify_overscroll_behavior_x_value(
"""
qualifier = """fn qualify_overscroll_behavior_value(
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
                .unwrap_or(CssOverscrollBehaviorQualificationOutcome::InvalidForSelectedValueGrammar),
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
    if identifier.eq_ignore_ascii_case(\"contain\") {
        return Some(CssOverscrollBehaviorKeyword::Contain);
    }
    if identifier.eq_ignore_ascii_case(\"none\") {
        return Some(CssOverscrollBehaviorKeyword::None);
    }
    if identifier.eq_ignore_ascii_case(\"auto\") {
        return Some(CssOverscrollBehaviorKeyword::Auto);
    }
    if identifier.eq_ignore_ascii_case(\"chain\") {
        return Some(CssOverscrollBehaviorKeyword::Chain);
    }
    None
}

"""
replace_exact(qualifier_marker, qualifier + qualifier_marker)

VALUE_PATH.write_text(text)

mod_text = MOD_PATH.read_text()
mod_old = """#[cfg(test)]
mod overscroll_behavior_inline_value_qualification_tests;
#[cfg(test)]
mod overscroll_behavior_x_value_qualification_tests;
"""
mod_new = """#[cfg(test)]
mod overscroll_behavior_inline_value_qualification_tests;
#[cfg(test)]
mod overscroll_behavior_shorthand_value_qualification_tests;
#[cfg(test)]
mod overscroll_behavior_x_value_qualification_tests;
"""
if mod_text.count(mod_old) != 1:
    raise SystemExit("validation mod anchor mismatch")
MOD_PATH.write_text(mod_text.replace(mod_old, mod_new))

TEST_PATH.write_text(
r'''use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssOverscrollBehaviorKeyword, CssOverscrollBehaviorQualificationOutcome,
    CssOverscrollBehaviorUnsupportedReason, CssOverscrollBehaviorValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Single(CssOverscrollBehaviorKeyword),
    Pair(CssOverscrollBehaviorKeyword, CssOverscrollBehaviorKeyword),
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

fn expected_outcome(expected: ExpectedOutcome) -> CssOverscrollBehaviorQualificationOutcome {
    match expected {
        ExpectedOutcome::Single(keyword) => CssOverscrollBehaviorQualificationOutcome::Qualified(
            CssOverscrollBehaviorValue::Single(keyword),
        ),
        ExpectedOutcome::Pair(first, second) => {
            CssOverscrollBehaviorQualificationOutcome::Qualified(
                CssOverscrollBehaviorValue::Pair { first, second },
            )
        }
        ExpectedOutcome::Invalid => {
            CssOverscrollBehaviorQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssOverscrollBehaviorQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferredFunction => {
            CssOverscrollBehaviorQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValueFunction => {
            CssOverscrollBehaviorQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorUnsupportedReason::WholeValueFunction,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .overscroll_behavior_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_one_and_two_keyword_boundary_matches_pinned_wpt() {
    use CssOverscrollBehaviorKeyword::{Auto, Chain, Contain, None};

    let result = qualify(
        3460,
        concat!(
            "a{overscroll-behavior:contain;}",
            "b{overscroll-behavior:none;}",
            "c{overscroll-behavior:auto;}",
            "d{OVERSCROLL-BEHAVIOR:ChAiN;}",
            r"e{overscroll-behavior:\63 ontain;}",
            r"f{overscroll-\62 ehavior:none;}",
            "g{overscroll-behavior:contain none;}",
            "h{overscroll-behavior:none auto;}",
            "i{overscroll-behavior:auto contain;}",
            "j{overscroll-behavior:chain auto;}",
            "k{overscroll-behavior:contain contain;}",
            "l{overscroll-behavior:normal;}",
            "m{overscroll-behavior:0;}",
            "n{overscroll-behavior:contain contain contain;}",
            "o{overscroll-behavior:;}",
            "p{overscroll-behavior:\"auto\";}",
            "q{overscroll-behavior-x:contain;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Single(Contain),
            ExpectedOutcome::Single(None),
            ExpectedOutcome::Single(Auto),
            ExpectedOutcome::Single(Chain),
            ExpectedOutcome::Single(Contain),
            ExpectedOutcome::Single(None),
            ExpectedOutcome::Pair(Contain, None),
            ExpectedOutcome::Pair(None, Auto),
            ExpectedOutcome::Pair(Auto, Contain),
            ExpectedOutcome::Pair(Chain, Auto),
            ExpectedOutcome::Pair(Contain, Contain),
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
fn authored_arity_is_preserved_without_shorthand_normalization() {
    use CssOverscrollBehaviorKeyword::Contain;

    let result = qualify(
        3461,
        concat!(
            "a{overscroll-behavior:contain;}",
            "b{overscroll-behavior:contain contain;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Single(Contain),
            ExpectedOutcome::Pair(Contain, Contain),
        ],
    );
}

#[test]
fn comments_and_priority_preserve_authored_components_and_source_placement() {
    use CssOverscrollBehaviorKeyword::{Chain, Contain, None};

    let result = qualify(
        3462,
        concat!(
            "a{overscroll-behavior:/**/contain/**/none/**/!important;}",
            "b{overscroll-behavior:/**/chain/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Pair(Contain, None),
            ExpectedOutcome::Single(Chain),
        ],
    );
    for observation in result.overscroll_behavior_observations() {
        let occurrence =
            &result.upstream_parser_result().occurrences()[observation.occurrence_index()];
        assert_eq!(observation.placement(), occurrence.placement());
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        3463,
        concat!(
            "a{overscroll-behavior:initial;}",
            "b{overscroll-behavior:inherit;}",
            "c{overscroll-behavior:unset;}",
            "d{overscroll-behavior:revert;}",
            "e{overscroll-behavior:revert-layer;}",
            "f{overscroll-behavior:revert-rule;}",
            "g{overscroll-behavior:inherit contain;}",
            "h{overscroll-behavior:contain inherit;}",
            "i{overscroll-behavior:initial revert;}",
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
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn deferred_functions_fail_open_anywhere_while_whole_value_functions_require_whole_placement() {
    let result = qualify(
        3464,
        concat!(
            "a{overscroll-behavior:var(--behavior);}",
            "b{overscroll-behavior:contain var(--behavior);}",
            "c{overscroll-behavior:foo(var(--behavior));}",
            "d{overscroll-behavior:first-valid(contain,none);}",
            "e{overscroll-behavior:cycle(contain,none);}",
            "f{overscroll-behavior:interpolate(0%,0:contain,1:none);}",
            "g{overscroll-behavior:contain first-valid(none);}",
            "h{overscroll-behavior:first-valid(auto) chain;}",
            "i{overscroll-behavior:foo();}",
            "j{overscroll-behavior:calc(1);}",
            "k{overscroll-behavior:contain foo();}",
            "l{overscroll-behavior:foo() none;}",
        ),
    );

    assert_expected(
        &result,
        &[
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
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn scroll_container_applicability_is_not_an_input_to_qualification() {
    use CssOverscrollBehaviorKeyword::{Auto, Chain, Contain, None};

    let result = qualify(
        3465,
        concat!(
            "span{overscroll-behavior:contain none;}",
            "div{overscroll-behavior:auto chain;}",
            "section{overscroll-behavior:none;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Pair(Contain, None),
            ExpectedOutcome::Pair(Auto, Chain),
            ExpectedOutcome::Single(None),
        ],
    );
}

#[test]
fn one_run_interleaves_shorthand_and_longhands_without_cross_dispatch() {
    use CssOverscrollBehaviorKeyword::{Chain, Contain};

    let result = qualify(
        3466,
        concat!(
            "a{direction:ltr;}",
            "b{overscroll-behavior-x:chain;}",
            "c{overscroll-behavior-y:chain;}",
            "d{overscroll-behavior-inline:chain;}",
            "e{overscroll-behavior-block:chain;}",
            "f{overscroll-behavior:contain chain;}",
            "g{clip-rule:evenodd;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_inline_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_block_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_observations().len(), 1);
    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_observations()[0].occurrence_index(), 5);
    assert_expected(&result, &[ExpectedOutcome::Pair(Contain, Chain)]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    use CssOverscrollBehaviorKeyword::Auto;

    let result = qualify(
        3467,
        "a{overscroll-behavior:auto;}b{overscroll-behavior:auto;}",
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Single(Auto), ExpectedOutcome::Single(Auto)],
    );
    assert_ne!(
        result.overscroll_behavior_observations()[0]
            .placement()
            .context_id(),
        result.overscroll_behavior_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3470, "@font-face{overscroll-behavior:none;}"),
        (3471, "@page{overscroll-behavior:none;}"),
        (3472, "@page{@top-left{overscroll-behavior:none;}}"),
        (3473, "@keyframes k{from{overscroll-behavior:none;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.overscroll_behavior_observations().is_empty(),
            "nonordinary declaration context produced an overscroll-behavior observation for {:?}",
            css
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssOverscrollBehaviorKeyword::{Auto, None};

    let result = qualify_with_limits(
        3480,
        "a{overscroll-behavior:auto none;overscroll-behavior:none auto;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Pair(Auto, None)]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{overscroll-behavior:contain none;}",
        "b{overscroll-behavior:inherit;}",
        "c{overscroll-behavior:chain;}",
        "d{overscroll-behavior:var(--behavior);}",
        "e{clip-rule:none;}",
    );
    let first = qualify(3490, css);
    let repeated = qualify(3490, css);
    let another_source = qualify(3491, css);

    assert_eq!(
        first.overscroll_behavior_observations(),
        repeated.overscroll_behavior_observations()
    );
    assert_eq!(
        first.overscroll_behavior_observations(),
        another_source.overscroll_behavior_observations()
    );
}
'''
)
