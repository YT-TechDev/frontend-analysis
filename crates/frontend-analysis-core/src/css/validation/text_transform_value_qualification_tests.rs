use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextTransformComponent, CssTextTransformQualificationOutcome,
    CssTextTransformUnsupportedReason, CssTextTransformValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    None,
    MathAuto,
    Components(&'static [CssTextTransformComponent]),
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
    let actual = result.text_transform_observations();
    assert_eq!(actual.len(), expected.len());
    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssTextTransformQualificationOutcome::Qualified(CssTextTransformValue::None),
                ExpectedOutcome::None,
            ) => {}
            (
                CssTextTransformQualificationOutcome::Qualified(CssTextTransformValue::MathAuto),
                ExpectedOutcome::MathAuto,
            ) => {}
            (
                CssTextTransformQualificationOutcome::Qualified(
                    CssTextTransformValue::Components(components),
                ),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssTextTransformQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssTextTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextTransformUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssTextTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextTransformUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssTextTransformQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextTransformUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => {
                panic!("unexpected text-transform outcome: {actual:?}, expected {expected:?}")
            }
        }
    }
}

#[test]
fn handwritten_fixed_slot_boundary_matches_pinned_wpt_and_derived_theorem() {
    use CssTextTransformComponent::{Capitalize, FullSizeKana, FullWidth, Lowercase, Uppercase};

    let result = qualify(
        87_000,
        concat!(
            "a{text-transform:none;}",
            "b{text-transform:math-auto;}",
            "c{text-transform:capitalize;}",
            "d{text-transform:uppercase;}",
            "e{text-transform:lowercase;}",
            "f{text-transform:full-width;}",
            "g{text-transform:full-size-kana;}",
            "h{text-transform:capitalize full-width full-size-kana;}",
            "i{text-transform:capitalize uppercase;}",
            "j{text-transform:full-width full-width;}",
            "k{text-transform:full-size-kana full-size-kana;}",
            "l{text-transform:none full-width;}",
            "m{text-transform:math-auto uppercase;}",
            "n{text-transform:none math-auto;}",
            "o{text-transform:math-auto math-auto;}",
            "p{text-transform:auto;}",
            "q{text-transform:0;}",
            "r{text-transform:\"uppercase\";}",
            "s{text-transform:;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::None,
            ExpectedOutcome::MathAuto,
            ExpectedOutcome::Components(&[Capitalize]),
            ExpectedOutcome::Components(&[Uppercase]),
            ExpectedOutcome::Components(&[Lowercase]),
            ExpectedOutcome::Components(&[FullWidth]),
            ExpectedOutcome::Components(&[FullSizeKana]),
            ExpectedOutcome::Components(&[Capitalize, FullWidth, FullSizeKana]),
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
            ExpectedOutcome::Invalid,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn authored_order_is_preserved_without_cssom_canonicalization() {
    use CssTextTransformComponent::{Capitalize, FullSizeKana, FullWidth, Lowercase, Uppercase};

    let result = qualify(
        87_001,
        concat!(
            "a{text-transform:full-width lowercase;}",
            "b{text-transform:lowercase full-width;}",
            "c{text-transform:full-size-kana capitalize;}",
            "d{text-transform:full-size-kana full-width uppercase;}",
            "e{text-transform:full-width uppercase full-size-kana;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[FullWidth, Lowercase]),
            ExpectedOutcome::Components(&[Lowercase, FullWidth]),
            ExpectedOutcome::Components(&[FullSizeKana, Capitalize]),
            ExpectedOutcome::Components(&[FullSizeKana, FullWidth, Uppercase]),
            ExpectedOutcome::Components(&[FullWidth, Uppercase, FullSizeKana]),
        ],
    );
}

#[test]
fn every_semantic_slot_rejects_duplicate_or_conflicting_claims() {
    let result = qualify(
        87_002,
        concat!(
            "a{text-transform:capitalize capitalize;}",
            "b{text-transform:uppercase lowercase;}",
            "c{text-transform:lowercase capitalize;}",
            "d{text-transform:full-width full-width;}",
            "e{text-transform:full-size-kana full-size-kana;}",
            "f{text-transform:uppercase full-width lowercase;}",
            "g{text-transform:full-size-kana capitalize full-size-kana;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 7]);
}

#[test]
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssTextTransformComponent::{Capitalize, FullSizeKana, FullWidth, Uppercase};

    let result = qualify(
        87_003,
        concat!(
            "a{TEXT-TRANSFORM:UPPERCASE FULL-WIDTH;}",
            r"b{text-transform:\63 apitalize;}",
            r"c{text-transfor\6d:full-size-kana;}",
            "d{text-transform:/**/full-size-kana/**/capitalize/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Uppercase, FullWidth]),
            ExpectedOutcome::Components(&[Capitalize]),
            ExpectedOutcome::Components(&[FullSizeKana]),
            ExpectedOutcome::Components(&[FullSizeKana, Capitalize]),
        ],
    );

    let priority_observation = &result.text_transform_observations()[3];
    let occurrence =
        &result.upstream_parser_result().occurrences()[priority_observation.occurrence_index()];
    assert_eq!(priority_observation.placement(), occurrence.placement());
    assert!(occurrence.priority().is_some());
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        87_004,
        concat!(
            "a{text-transform:initial;}",
            "b{text-transform:inherit;}",
            "c{text-transform:unset;}",
            "d{text-transform:revert;}",
            "e{text-transform:revert-layer;}",
            "f{text-transform:revert-rule;}",
            "g{text-transform:inherit uppercase;}",
            "h{text-transform:full-width inherit;}",
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
        87_005,
        concat!(
            "a{text-transform:var(--transform);}",
            "b{text-transform:uppercase var(--transform);}",
            "c{text-transform:var(--transform) full-width;}",
            "d{text-transform:foo(var(--transform));}",
            "e{text-transform:first-valid(uppercase,lowercase);}",
            "f{text-transform:cycle(uppercase,lowercase);}",
            "g{text-transform:interpolate(0%,0:uppercase,1:lowercase);}",
            "h{text-transform:uppercase first-valid(lowercase);}",
            "i{text-transform:foo();}",
            "j{text-transform:full-width foo();}",
            "k{text-transform:calc(1);}",
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
    use CssTextTransformComponent::{FullWidth, Uppercase};

    let result = qualify(
        87_006,
        concat!(
            "a{font-variant-numeric:oldstyle-nums tabular-nums;}",
            "b{contain:layout size;}",
            "table{text-transform:full-width uppercase;}",
            "d{text-decoration-line:underline;}",
        ),
    );

    assert_eq!(result.font_variant_numeric_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(result.text_transform_observations().len(), 1);
    assert_eq!(result.text_decoration_line_observations().len(), 1);
    assert_eq!(result.text_transform_observations()[0].occurrence_index(), 2);
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[FullWidth, Uppercase])],
    );
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        87_007,
        "a{text-transform:math-auto;}b{text-transform:math-auto;}",
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::MathAuto, ExpectedOutcome::MathAuto],
    );
    assert_ne!(
        result.text_transform_observations()[0]
            .placement()
            .context_id(),
        result.text_transform_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (87_010, "@font-face{text-transform:uppercase;}"),
        (87_011, "@page{text-transform:uppercase;}"),
        (87_012, "@page{@top-left{text-transform:uppercase;}}"),
        (87_013, "@keyframes k{from{text-transform:uppercase;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_transform_observations().is_empty(),
            "unexpected observation for {css}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssTextTransformComponent::{FullWidth, Uppercase};

    let result = qualify_with_limits(
        87_020,
        concat!(
            "a{text-transform:full-width uppercase;",
            "text-transform:uppercase full-width;}"
        ),
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[FullWidth, Uppercase])],
    );
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{text-transform:full-size-kana full-width lowercase;}",
        "b{text-transform:none;}",
        "c{text-transform:math-auto;}",
        "d{text-transform:inherit;}",
        "e{text-transform:var(--transform);}",
        "f{text-transform:uppercase lowercase;}",
    );

    let first = qualify(87_021, css);
    let repeated = qualify(87_021, css);
    assert_eq!(
        first.text_transform_observations(),
        repeated.text_transform_observations()
    );

    let other_source = qualify(87_022, css);
    let first_outcomes: Vec<_> = first
        .text_transform_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let other_outcomes: Vec<_> = other_source
        .text_transform_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(first_outcomes, other_outcomes);
}
