use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextUnderlinePositionComponent, CssTextUnderlinePositionQualificationOutcome,
    CssTextUnderlinePositionUnsupportedReason, CssTextUnderlinePositionValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

/// Selected grammar under test: CSS Text Decoration Level 4
/// `auto | [ from-font | under ] || [ left | right ]`. `auto` is an
/// exclusive singleton; the component branch admits one or two authored
/// direct identifiers with at most one occupying Slot A
/// (`from-font`/`under`) and at most one occupying Slot B
/// (`left`/`right`), authored order preserved exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Components(&'static [CssTextUnderlinePositionComponent]),
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
    let actual = result.text_underline_position_observations();
    assert_eq!(actual.len(), expected.len());
    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssTextUnderlinePositionQualificationOutcome::Qualified(
                    CssTextUnderlinePositionValue::Auto,
                ),
                ExpectedOutcome::Auto,
            ) => {}
            (
                CssTextUnderlinePositionQualificationOutcome::Qualified(
                    CssTextUnderlinePositionValue::Components(components),
                ),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssTextUnderlinePositionQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssTextUnderlinePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextUnderlinePositionUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssTextUnderlinePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextUnderlinePositionUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssTextUnderlinePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextUnderlinePositionUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => {
                panic!(
                    "unexpected text-underline-position outcome: {actual:?}, expected {expected:?}"
                )
            }
        }
    }
}

#[test]
fn auto_is_an_exclusive_singleton() {
    let result = qualify(
        62_100,
        concat!(
            "a{text-underline-position:auto;}",
            "b{text-underline-position:auto left;}",
            "c{text-underline-position:left auto;}",
            "d{text-underline-position:auto right;}",
            "e{text-underline-position:right auto;}",
            "f{text-underline-position:auto under;}",
            "g{text-underline-position:under auto;}",
            "h{text-underline-position:auto from-font;}",
            "i{text-underline-position:from-font auto;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
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
fn every_direct_singleton_is_qualified_without_auto_synthesis_or_font_downgrade() {
    use CssTextUnderlinePositionComponent::{FromFont, Left, Right, Under};

    let result = qualify(
        62_101,
        concat!(
            "a{text-underline-position:from-font;}",
            "b{text-underline-position:under;}",
            "c{text-underline-position:left;}",
            "d{text-underline-position:right;}",
        ),
    );

    // `left`/`right` must qualify as a bare one-element authored sequence
    // ([Left]/[Right]) rather than an implied-auto pair ([Auto, Left]);
    // `Components` structurally cannot carry `Auto`, so this equality is
    // itself the load-bearing non-synthesis assertion. `from-font` is
    // directly `Qualified`, never `Unsupported`, because it needs no font
    // metrics to be authored-recognized.
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[FromFont]),
            ExpectedOutcome::Components(&[Under]),
            ExpectedOutcome::Components(&[Left]),
            ExpectedOutcome::Components(&[Right]),
        ],
    );
}

#[test]
fn every_valid_pair_is_qualified_in_both_authored_orders() {
    use CssTextUnderlinePositionComponent::{FromFont, Left, Right, Under};

    let result = qualify(
        62_102,
        concat!(
            "a{text-underline-position:from-font left;}",
            "b{text-underline-position:left from-font;}",
            "c{text-underline-position:from-font right;}",
            "d{text-underline-position:right from-font;}",
            "e{text-underline-position:under left;}",
            "f{text-underline-position:left under;}",
            "g{text-underline-position:under right;}",
            "h{text-underline-position:right under;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[FromFont, Left]),
            ExpectedOutcome::Components(&[Left, FromFont]),
            ExpectedOutcome::Components(&[FromFont, Right]),
            ExpectedOutcome::Components(&[Right, FromFont]),
            ExpectedOutcome::Components(&[Under, Left]),
            ExpectedOutcome::Components(&[Left, Under]),
            ExpectedOutcome::Components(&[Under, Right]),
            ExpectedOutcome::Components(&[Right, Under]),
        ],
    );

    // Authored order is a distinct identity, not a canonicalized set.
    assert_ne!(
        result.text_underline_position_observations()[0].outcome(),
        result.text_underline_position_observations()[1].outcome(),
    );
    assert_ne!(
        result.text_underline_position_observations()[6].outcome(),
        result.text_underline_position_observations()[7].outcome(),
    );
}

#[test]
fn same_slot_conflicts_duplicates_and_overfull_or_malformed_structures_are_invalid() {
    let result = qualify(
        62_103,
        concat!(
            // Slot A conflicts.
            "a{text-underline-position:from-font under;}",
            "b{text-underline-position:under from-font;}",
            // Slot B conflicts.
            "c{text-underline-position:left right;}",
            "d{text-underline-position:right left;}",
            // Exact duplicates.
            "e{text-underline-position:from-font from-font;}",
            "f{text-underline-position:under under;}",
            "g{text-underline-position:left left;}",
            "h{text-underline-position:right right;}",
            // Three or more direct components.
            "i{text-underline-position:from-font under left;}",
            "j{text-underline-position:left right from-font;}",
            "k{text-underline-position:from-font left right under;}",
            // Empty value.
            "l{text-underline-position:;}",
            // Wrong token classes.
            "m{text-underline-position:0;}",
            "n{text-underline-position:1px;}",
            "o{text-underline-position:\"left\";}",
            "p{text-underline-position:#fff;}",
            "q{text-underline-position:foo;}",
            // Comma-separated forms.
            "r{text-underline-position:from-font,left;}",
            "s{text-underline-position:left,right;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 19]);
}

#[test]
fn case_insensitive_and_escaped_identifiers_are_recognized() {
    use CssTextUnderlinePositionComponent::{FromFont, Left, Right, Under};

    let result = qualify(
        62_104,
        concat!(
            "a{TEXT-UNDERLINE-POSITION:RIGHT UNDER;}",
            r"b{text-underline-position:\75 nder;}",
            r"c{text-underline-positio\6e:left under;}",
            "d{text-underline-position:From-Font Left;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Right, Under]),
            ExpectedOutcome::Components(&[Under]),
            ExpectedOutcome::Components(&[Left, Under]),
            ExpectedOutcome::Components(&[FromFont, Left]),
        ],
    );
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        62_105,
        concat!(
            "a{text-underline-position:initial;}",
            "b{text-underline-position:inherit;}",
            "c{text-underline-position:unset;}",
            "d{text-underline-position:revert;}",
            "e{text-underline-position:revert-layer;}",
            "f{text-underline-position:revert-rule;}",
            "g{text-underline-position:inherit left;}",
            "h{text-underline-position:left inherit;}",
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
fn deferred_and_whole_value_functions_fail_open_while_ordinary_functions_stay_invalid() {
    let result = qualify(
        62_106,
        concat!(
            "a{text-underline-position:var(--pos);}",
            "b{text-underline-position:left var(--pos);}",
            "c{text-underline-position:var(--pos) left;}",
            "d{text-underline-position:env(pos);}",
            "e{text-underline-position:first-valid(left,right);}",
            "f{text-underline-position:cycle(left,right);}",
            "g{text-underline-position:interpolate(0%,0:left,1:right);}",
            "h{text-underline-position:left first-valid(right);}",
            "i{text-underline-position:foo();}",
            "j{text-underline-position:left foo();}",
            "k{text-underline-position:calc(1);}",
        ),
    );

    // Sole/embedded ordinary residual `Function`s (`foo()`, `calc(1)`) are
    // `Invalid` for this no-Function-slot grammar, never reclassified as a
    // typed `FunctionValue` unsupported reason.
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
fn trivia_comments_and_priority_preserve_authored_components_and_placement() {
    use CssTextUnderlinePositionComponent::{FromFont, Left, Right, Under};

    let result = qualify(
        62_107,
        concat!(
            "a{text-underline-position:/**/from-font/**/left/**/!important;}",
            "b{text-underline-position:/**/auto/**/!important;}",
            "c{text-underline-position:/**/right/**/under/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[FromFont, Left]),
            ExpectedOutcome::Auto,
            ExpectedOutcome::Components(&[Right, Under]),
        ],
    );

    for index in 0..3 {
        let observation = &result.text_underline_position_observations()[index];
        let occurrence =
            &result.upstream_parser_result().occurrences()[observation.occurrence_index()];
        assert_eq!(observation.placement(), occurrence.placement());
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn duplicate_declarations_are_retained_independently_in_source_order() {
    use CssTextUnderlinePositionComponent::{Right, Under};

    let result = qualify(
        62_108,
        "a{text-underline-position:right under;text-underline-position:under right;}",
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Right, Under]),
            ExpectedOutcome::Components(&[Under, Right]),
        ],
    );
    assert_eq!(
        result.text_underline_position_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.text_underline_position_observations()[1].occurrence_index(),
        1
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (62_110, "@font-face{text-underline-position:auto;}"),
        (62_111, "@page{text-underline-position:auto;}"),
        (62_112, "@page{@top-left{text-underline-position:auto;}}"),
        (62_113, "@keyframes k{from{text-underline-position:auto;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_underline_position_observations().is_empty(),
            "unexpected observation for {css}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssTextUnderlinePositionComponent::{Left, Under};

    let result = qualify_with_limits(
        62_120,
        concat!(
            "a{text-underline-position:left under;",
            "text-underline-position:under left;}"
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
        "a{text-underline-position:right under;}",
        "b{text-underline-position:from-font;}",
        "c{text-underline-position:left;}",
        "d{text-underline-position:auto;}",
        "e{text-underline-position:inherit;}",
        "f{text-underline-position:var(--pos);}",
        "g{text-underline-position:from-font under;}",
    );

    let first = qualify(62_121, css);
    let repeated = qualify(62_121, css);
    assert_eq!(
        first.text_underline_position_observations(),
        repeated.text_underline_position_observations()
    );

    let other_source = qualify(62_122, css);
    let first_outcomes: Vec<_> = first
        .text_underline_position_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let other_outcomes: Vec<_> = other_source
        .text_underline_position_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(first_outcomes, other_outcomes);
}

#[test]
fn neighboring_accepted_leaves_stay_isolated_and_unaffected() {
    use CssTextUnderlinePositionComponent::{FromFont, Left};

    let result = qualify(
        62_123,
        concat!(
            "a{text-emphasis-position:over right;}",
            "b{text-transform:uppercase full-width;}",
            "c{text-decoration-line:underline overline;}",
            "d{text-underline-offset:auto;}",
            "e{letter-spacing:normal;}",
            "table{text-underline-position:from-font left;}",
        ),
    );

    assert_eq!(result.text_emphasis_position_observations().len(), 1);
    assert_eq!(result.text_transform_observations().len(), 1);
    assert_eq!(result.text_decoration_line_observations().len(), 1);
    assert_eq!(result.text_underline_offset_observations().len(), 1);
    assert_eq!(result.letter_spacing_observations().len(), 1);
    assert_eq!(result.text_underline_position_observations().len(), 1);
    assert_eq!(
        result.text_underline_position_observations()[0].occurrence_index(),
        5
    );
    assert_expected(&result, &[ExpectedOutcome::Components(&[FromFont, Left])]);
}
