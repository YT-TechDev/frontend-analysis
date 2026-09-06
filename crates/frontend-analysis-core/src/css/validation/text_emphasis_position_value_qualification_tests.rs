use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextEmphasisPositionComponent, CssTextEmphasisPositionQualificationOutcome,
    CssTextEmphasisPositionUnsupportedReason, CssTextEmphasisPositionValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Components(&'static [CssTextEmphasisPositionComponent]),
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
    let actual = result.text_emphasis_position_observations();
    assert_eq!(actual.len(), expected.len());
    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssTextEmphasisPositionQualificationOutcome::Qualified(
                    CssTextEmphasisPositionValue::Components(components),
                ),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssTextEmphasisPositionQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssTextEmphasisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextEmphasisPositionUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssTextEmphasisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextEmphasisPositionUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssTextEmphasisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextEmphasisPositionUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => {
                panic!(
                    "unexpected text-emphasis-position outcome: {actual:?}, expected {expected:?}"
                )
            }
        }
    }
}

#[test]
fn normative_double_ampersand_boundary_matches_research_548() {
    use CssTextEmphasisPositionComponent::{Left, Over, Right, Under};

    let result = qualify(
        88_000,
        concat!(
            "a{text-emphasis-position:over;}",
            "b{text-emphasis-position:under;}",
            "c{text-emphasis-position:over right;}",
            "d{text-emphasis-position:right over;}",
            "e{text-emphasis-position:over left;}",
            "f{text-emphasis-position:left over;}",
            "g{text-emphasis-position:under right;}",
            "h{text-emphasis-position:right under;}",
            "i{text-emphasis-position:under left;}",
            "j{text-emphasis-position:left under;}",
            "k{text-emphasis-position:right;}",
            "l{text-emphasis-position:left;}",
            "m{text-emphasis-position:auto;}",
            "n{text-emphasis-position:;}",
            "o{text-emphasis-position:0;}",
            "p{text-emphasis-position:\"over\";}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Over]),
            ExpectedOutcome::Components(&[Under]),
            ExpectedOutcome::Components(&[Over, Right]),
            ExpectedOutcome::Components(&[Right, Over]),
            ExpectedOutcome::Components(&[Over, Left]),
            ExpectedOutcome::Components(&[Left, Over]),
            ExpectedOutcome::Components(&[Under, Right]),
            ExpectedOutcome::Components(&[Right, Under]),
            ExpectedOutcome::Components(&[Under, Left]),
            ExpectedOutcome::Components(&[Left, Under]),
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
fn authored_order_and_omission_are_preserved_without_cssom_or_default_synthesis() {
    use CssTextEmphasisPositionComponent::{Left, Over, Right, Under};

    let result = qualify(
        88_001,
        concat!(
            "a{text-emphasis-position:right under;}",
            "b{text-emphasis-position:under right;}",
            "c{text-emphasis-position:left over;}",
            "d{text-emphasis-position:over left;}",
            "e{text-emphasis-position:over;}",
            "f{text-emphasis-position:under;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Right, Under]),
            ExpectedOutcome::Components(&[Under, Right]),
            ExpectedOutcome::Components(&[Left, Over]),
            ExpectedOutcome::Components(&[Over, Left]),
            ExpectedOutcome::Components(&[Over]),
            ExpectedOutcome::Components(&[Under]),
        ],
    );
}

#[test]
fn slot_collisions_missing_required_vertical_and_excess_components_are_invalid() {
    let result = qualify(
        88_002,
        concat!(
            "a{text-emphasis-position:over under;}",
            "b{text-emphasis-position:under over;}",
            "c{text-emphasis-position:over over;}",
            "d{text-emphasis-position:right left;}",
            "e{text-emphasis-position:left right;}",
            "f{text-emphasis-position:left left;}",
            "g{text-emphasis-position:right right;}",
            "h{text-emphasis-position:over right left;}",
            "i{text-emphasis-position:right over under;}",
            "j{text-emphasis-position:auto right;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 10]);
}

#[test]
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssTextEmphasisPositionComponent::{Left, Over, Right, Under};

    let result = qualify(
        88_003,
        concat!(
            "a{TEXT-EMPHASIS-POSITION:RIGHT UNDER;}",
            r"b{text-emphasis-position:\6f ver;}",
            r"c{text-emphasis-positio\6e:left under;}",
            "d{text-emphasis-position:/**/over/**/left/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Right, Under]),
            ExpectedOutcome::Components(&[Over]),
            ExpectedOutcome::Components(&[Left, Under]),
            ExpectedOutcome::Components(&[Over, Left]),
        ],
    );

    let priority_observation = &result.text_emphasis_position_observations()[3];
    let occurrence =
        &result.upstream_parser_result().occurrences()[priority_observation.occurrence_index()];
    assert_eq!(priority_observation.placement(), occurrence.placement());
    assert!(occurrence.priority().is_some());
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        88_004,
        concat!(
            "a{text-emphasis-position:initial;}",
            "b{text-emphasis-position:inherit;}",
            "c{text-emphasis-position:unset;}",
            "d{text-emphasis-position:revert;}",
            "e{text-emphasis-position:revert-layer;}",
            "f{text-emphasis-position:revert-rule;}",
            "g{text-emphasis-position:inherit over;}",
            "h{text-emphasis-position:under inherit;}",
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
        88_005,
        concat!(
            "a{text-emphasis-position:var(--position);}",
            "b{text-emphasis-position:over var(--position);}",
            "c{text-emphasis-position:var(--position) left;}",
            "d{text-emphasis-position:foo(var(--position));}",
            "e{text-emphasis-position:first-valid(over,under);}",
            "f{text-emphasis-position:cycle(over,under);}",
            "g{text-emphasis-position:interpolate(0%,0:over,1:under);}",
            "h{text-emphasis-position:over first-valid(left);}",
            "i{text-emphasis-position:foo();}",
            "j{text-emphasis-position:left foo();}",
            "k{text-emphasis-position:calc(1);}",
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
    use CssTextEmphasisPositionComponent::{Left, Under};

    let result = qualify(
        88_006,
        concat!(
            "a{text-transform:uppercase full-width;}",
            "b{text-decoration-line:underline overline;}",
            "table{text-emphasis-position:left under;}",
            "d{text-decoration-style:wavy;}",
        ),
    );

    assert_eq!(result.text_transform_observations().len(), 1);
    assert_eq!(result.text_decoration_line_observations().len(), 1);
    assert_eq!(result.text_emphasis_position_observations().len(), 1);
    assert_eq!(result.text_decoration_style_observations().len(), 1);
    assert_eq!(
        result.text_emphasis_position_observations()[0].occurrence_index(),
        2
    );
    assert_expected(&result, &[ExpectedOutcome::Components(&[Left, Under])]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        88_007,
        "a{text-emphasis-position:over;}b{text-emphasis-position:over;}",
    );

    use CssTextEmphasisPositionComponent::Over;
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Over]),
            ExpectedOutcome::Components(&[Over]),
        ],
    );
    assert_ne!(
        result.text_emphasis_position_observations()[0]
            .placement()
            .context_id(),
        result.text_emphasis_position_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (88_010, "@font-face{text-emphasis-position:over;}"),
        (88_011, "@page{text-emphasis-position:over;}"),
        (88_012, "@page{@top-left{text-emphasis-position:over;}}"),
        (88_013, "@keyframes k{from{text-emphasis-position:over;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_emphasis_position_observations().is_empty(),
            "unexpected observation for {css}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssTextEmphasisPositionComponent::{Left, Under};

    let result = qualify_with_limits(
        88_020,
        concat!(
            "a{text-emphasis-position:left under;",
            "text-emphasis-position:under left;}"
        ),
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Components(&[Left, Under])]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{text-emphasis-position:right under;}",
        "b{text-emphasis-position:over;}",
        "c{text-emphasis-position:left;}",
        "d{text-emphasis-position:auto;}",
        "e{text-emphasis-position:inherit;}",
        "f{text-emphasis-position:var(--position);}",
        "g{text-emphasis-position:over under;}",
    );

    let first = qualify(88_021, css);
    let repeated = qualify(88_021, css);
    assert_eq!(
        first.text_emphasis_position_observations(),
        repeated.text_emphasis_position_observations()
    );

    let other_source = qualify(88_022, css);
    let first_outcomes: Vec<_> = first
        .text_emphasis_position_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let other_outcomes: Vec<_> = other_source
        .text_emphasis_position_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(first_outcomes, other_outcomes);
}
