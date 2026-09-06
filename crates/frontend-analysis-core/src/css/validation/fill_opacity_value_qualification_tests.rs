use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFillOpacityQualificationOutcome, CssFillOpacityUnsupportedReason, CssFillOpacityValue,
    CssOpacityQualificationOutcome, CssOpacityValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    DirectNumber,
    DirectPercentage,
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferred,
    UnsupportedWholeValue,
    UnsupportedFunction,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFillOpacityQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectNumber => {
            CssFillOpacityQualificationOutcome::Qualified(CssFillOpacityValue::DirectNumberLiteral)
        }
        ExpectedOutcome::DirectPercentage => CssFillOpacityQualificationOutcome::Qualified(
            CssFillOpacityValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssFillOpacityQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFillOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssFillOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillOpacityUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssFillOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillOpacityUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFillOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFillOpacityUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .fill_opacity_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_opacity_reference_grammar_keeps_out_of_range_literals_qualified() {
    let result = qualify(
        1600,
        concat!(
            "a{fill-opacity:0;}",
            "b{fill-opacity:1;}",
            "c{fill-opacity:.5;}",
            "d{fill-opacity:1.5;}",
            "e{fill-opacity:-1;}",
            "f{fill-opacity:3;}",
            "g{fill-opacity:1e100;}",
            "h{fill-opacity:-1e100;}",
            "i{fill-opacity:+.25;}",
            "j{fill-opacity:0%;}",
            "k{fill-opacity:50%;}",
            "l{fill-opacity:100%;}",
            "m{fill-opacity:300%;}",
            "n{fill-opacity:-100%;}",
            "o{fill-opacity:+25%;}",
            "p{fill-opacity:1px;}",
            "q{fill-opacity:\"1\";}",
            "r{fill-opacity:auto;}",
            "s{fill-opacity:foo;}",
            "t{fill-opacity:;}",
            "u{color:1;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
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
fn comments_priority_and_separated_signs_preserve_retained_token_boundaries() {
    let result = qualify(
        1601,
        concat!(
            "a{fill-opacity:/**/300%/**/!important;}",
            "b{fill-opacity:/**/-1/**/!important;}",
            "c{fill-opacity:+ 1;}",
            "d{fill-opacity:+/**/1;}",
            "e{fill-opacity:- 50%;}",
            "f{fill-opacity:-/**/50%;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
    assert!(
        result.upstream_parser_result().occurrences()[1]
            .priority()
            .is_some()
    );
}

#[test]
fn direct_cardinality_and_ordinary_function_boundaries_match_opacity() {
    let result = qualify(
        1602,
        concat!(
            "a{fill-opacity:1 2;}",
            "b{fill-opacity:50% 1;}",
            "c{fill-opacity:1 50%;}",
            "d{fill-opacity:(1);}",
            "e{fill-opacity:calc(1);}",
            "f{fill-opacity:min(0,1);}",
            "g{fill-opacity:max(-1,2);}",
            "h{fill-opacity:clamp(-1,.5,2);}",
            "i{fill-opacity:foo();}",
            "j{fill-opacity:foo() 1;}",
            "k{fill-opacity:1 foo();}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn css_wide_substitution_and_whole_value_boundaries_are_preserved() {
    let result = qualify(
        1603,
        concat!(
            "a{fill-opacity:initial;}",
            "b{fill-opacity:inherit;}",
            "c{fill-opacity:unset;}",
            "d{fill-opacity:revert;}",
            "e{fill-opacity:revert-layer;}",
            "f{fill-opacity:revert-rule;}",
            "g{fill-opacity:1 initial;}",
            "h{fill-opacity:var(--opacity);}",
            "i{fill-opacity:env(opacity);}",
            "j{fill-opacity:attr(data-opacity);}",
            "k{fill-opacity:--opacity();}",
            "l{fill-opacity:-1 var(--opacity);}",
            "m{fill-opacity:calc(var(--opacity));}",
            "n{fill-opacity:first-valid(1,50%);}",
            "o{fill-opacity:cycle(1,50%);}",
            "p{fill-opacity:interpolate(50%,0:0,1:1);}",
            "q{fill-opacity:first-valid(1,50%) 2;}",
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
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn opacity_and_fill_opacity_keep_property_specific_observation_ownership() {
    let result = qualify(
        1604,
        concat!(
            "a{opacity:120%;}",
            "b{fill-opacity:-1;}",
            "c{opacity:.5;}",
            "d{fill-opacity:300%;}",
        ),
    );

    assert_eq!(result.opacity_observations().len(), 2);
    assert_eq!(result.fill_opacity_observations().len(), 2);
    assert_eq!(result.opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.fill_opacity_observations()[0].occurrence_index(), 1);
    assert_eq!(result.opacity_observations()[1].occurrence_index(), 2);
    assert_eq!(result.fill_opacity_observations()[1].occurrence_index(), 3);
    assert_eq!(
        result.opacity_observations()[0].outcome(),
        CssOpacityQualificationOutcome::Qualified(CssOpacityValue::DirectPercentageLiteral)
    );
    assert_eq!(
        result.fill_opacity_observations()[0].outcome(),
        CssFillOpacityQualificationOutcome::Qualified(CssFillOpacityValue::DirectNumberLiteral)
    );
}

#[test]
fn duplicate_fill_opacity_declarations_keep_distinct_run_local_placement() {
    let result = qualify(1605, "a{fill-opacity:50%;}b{fill-opacity:50%;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
        ],
    );
    assert_eq!(result.fill_opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.fill_opacity_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.fill_opacity_observations()[0]
            .placement()
            .context_id(),
        result.fill_opacity_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_fill_opacity_observations() {
    for (source_id, css) in [
        (1606, "@font-face{fill-opacity:1;}"),
        (1607, "@page{fill-opacity:1;}"),
        (1608, "@page{@top-left{fill-opacity:1;}}"),
        (1609, "@keyframes k{from{fill-opacity:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.fill_opacity_observations().is_empty(),
            "nonordinary declaration context produced a fill-opacity observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_fill_opacity_prefix() {
    let result = qualify_with_limits(
        1610,
        "a{fill-opacity:300%;fill-opacity:-1;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::DirectPercentage]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_fill_opacity_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{fill-opacity:-1;}",
        "b{fill-opacity:300%;}",
        "c{fill-opacity:calc(2);}",
        "d{fill-opacity:var(--opacity);}",
        "e{opacity:50%;}",
        "f{direction:ltr;}",
    );
    let first = qualify(1611, css);
    let repeated = qualify(1611, css);
    let another_source = qualify(1612, css);

    assert_eq!(
        first.fill_opacity_observations(),
        repeated.fill_opacity_observations()
    );
    assert_eq!(
        first.fill_opacity_observations(),
        another_source.fill_opacity_observations()
    );
    assert_eq!(
        first.opacity_observations(),
        repeated.opacity_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
}
