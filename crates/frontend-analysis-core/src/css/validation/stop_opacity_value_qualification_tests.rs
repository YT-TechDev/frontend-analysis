use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFillOpacityQualificationOutcome, CssFillOpacityValue, CssOpacityQualificationOutcome,
    CssOpacityValue, CssStopOpacityQualificationOutcome, CssStopOpacityUnsupportedReason,
    CssStopOpacityValue, CssStrokeLinecapQualificationOutcome, CssStrokeLinecapValue,
    CssStrokeOpacityQualificationOutcome, CssStrokeOpacityValue, CssValueQualificationRunResult,
    run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssStopOpacityQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectNumber => {
            CssStopOpacityQualificationOutcome::Qualified(CssStopOpacityValue::DirectNumberLiteral)
        }
        ExpectedOutcome::DirectPercentage => CssStopOpacityQualificationOutcome::Qualified(
            CssStopOpacityValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssStopOpacityQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssStopOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStopOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssStopOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStopOpacityUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssStopOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStopOpacityUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssStopOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStopOpacityUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .stop_opacity_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

// Pinned WPT corroboration (svg/pservers/parsing/stop-opacity-valid.svg):
// `-1, 0.5, 3, -100%, 50%, 300%` remain authored-valid direct literals. The
// `[0,1]`/`[0%,100%]` clamp is computed-value behavior and is explicitly out
// of scope for this leaf.
#[test]
fn out_of_range_direct_literals_remain_qualified_not_invalid() {
    let result = qualify(
        1800,
        concat!(
            "a{stop-opacity:-1;}",
            "b{stop-opacity:0.5;}",
            "c{stop-opacity:3;}",
            "d{stop-opacity:-100%;}",
            "e{stop-opacity:50%;}",
            "f{stop-opacity:300%;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

// Required direct Number lexical diversity: sign spellings, leading-dot
// decimals, integers, and exponent spellings (including uppercase `E` and
// zero-magnitude exponents) must all tokenize as one Number token and
// qualify identically -- no range check is introduced.
#[test]
fn direct_number_lexical_diversity_all_qualify_as_direct_number() {
    let result = qualify(
        1801,
        concat!(
            "a{stop-opacity:0;}",
            "b{stop-opacity:+0;}",
            "c{stop-opacity:-0;}",
            "d{stop-opacity:.5;}",
            "e{stop-opacity:-.5;}",
            "f{stop-opacity:1;}",
            "g{stop-opacity:-1;}",
            "h{stop-opacity:3;}",
            "i{stop-opacity:1e0;}",
            "j{stop-opacity:1E0;}",
            "k{stop-opacity:+1e0;}",
            "l{stop-opacity:-1e2;}",
            "m{stop-opacity:0e100;}",
            "n{stop-opacity:-0e100;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::DirectNumber; 14]);
}

// Required direct Percentage lexical diversity: sign spellings, leading-dot
// decimals, and exponent spellings immediately followed by `%` all tokenize
// as one Percentage token and remain distinct from Number identity.
#[test]
fn direct_percentage_lexical_diversity_all_qualify_as_direct_percentage() {
    let result = qualify(
        1802,
        concat!(
            "a{stop-opacity:0%;}",
            "b{stop-opacity:+0%;}",
            "c{stop-opacity:-0%;}",
            "d{stop-opacity:50%;}",
            "e{stop-opacity:100%;}",
            "f{stop-opacity:-100%;}",
            "g{stop-opacity:300%;}",
            "h{stop-opacity:.5%;}",
            "i{stop-opacity:-.5%;}",
            "j{stop-opacity:1e2%;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::DirectPercentage; 10]);
}

// Pinned WPT corroboration (svg/pservers/parsing/stop-opacity-invalid.svg):
// `1.` and `2 3` are authored-invalid, plus the repository's established
// wrong-token-class/cardinality invalid matrix used by the accepted
// `opacity`/`fill-opacity`/`stroke-opacity` leaves.
#[test]
fn wrong_token_class_and_cardinality_mismatches_are_invalid() {
    let result = qualify(
        1803,
        concat!(
            "a{stop-opacity:;}",
            "b{stop-opacity:1.;}",
            "c{stop-opacity:2 3;}",
            "d{stop-opacity:1px;}",
            "e{stop-opacity:\"1\";}",
            "f{stop-opacity:auto;}",
            "g{stop-opacity:#fff;}",
            "h{stop-opacity:1,2;}",
            "i{stop-opacity:1 50%;}",
            "j{stop-opacity:50% 1;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 10]);
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn comments_priority_and_separated_signs_preserve_retained_token_boundaries() {
    let result = qualify(
        1804,
        concat!(
            "a{stop-opacity:/**/300%/**/!important;}",
            "b{stop-opacity:/**/-1/**/!important;}",
            "c{stop-opacity:+ 1;}",
            "d{stop-opacity:+/**/1;}",
            "e{stop-opacity:- 50%;}",
            "f{stop-opacity:-/**/50%;}",
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
fn property_name_is_recognized_ascii_case_insensitively() {
    let result = qualify(
        1805,
        concat!(
            "a{STOP-OPACITY:0.5;}",
            "b{Stop-Opacity:50%;}",
            "c{stop-OPACITY:-1;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectNumber,
        ],
    );
}

#[test]
fn sole_ordinary_function_boundary_matches_the_selected_profile() {
    let result = qualify(
        1806,
        concat!(
            "a{stop-opacity:1 2;}",
            "b{stop-opacity:50% 1;}",
            "c{stop-opacity:1 50%;}",
            "d{stop-opacity:(1);}",
            "e{stop-opacity:calc(.5);}",
            "f{stop-opacity:min(0,1);}",
            "g{stop-opacity:max(0,1);}",
            "h{stop-opacity:clamp(0,.5,1);}",
            "i{stop-opacity:foo();}",
            "j{stop-opacity:foo() 1;}",
            "k{stop-opacity:1 foo();}",
            "l{stop-opacity:1 calc(.5);}",
            "m{stop-opacity:calc(.5) 1;}",
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
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn css_wide_substitution_and_whole_value_boundaries_are_preserved() {
    let result = qualify(
        1807,
        concat!(
            "a{stop-opacity:initial;}",
            "b{stop-opacity:inherit;}",
            "c{stop-opacity:unset;}",
            "d{stop-opacity:revert;}",
            "e{stop-opacity:revert-layer;}",
            "f{stop-opacity:revert-rule;}",
            "g{stop-opacity:1 initial;}",
            "h{stop-opacity:var(--opacity);}",
            "i{stop-opacity:env(opacity);}",
            "j{stop-opacity:attr(data-opacity);}",
            "k{stop-opacity:--opacity();}",
            "l{stop-opacity:-1 var(--opacity);}",
            "m{stop-opacity:calc(var(--opacity));}",
            "n{stop-opacity:first-valid(1,50%);}",
            "o{stop-opacity:cycle(1,50%);}",
            "p{stop-opacity:interpolate(50%,0:0,1:1);}",
            "q{stop-opacity:first-valid(1,50%) 2;}",
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

    // The initial value of `stop-opacity` is `1`, but authored `initial`
    // must never be synthesized into `Qualified(DirectNumberLiteral)`: it
    // is a CSS-wide mechanism, not authored numeric identity.
    for outcome in result
        .stop_opacity_observations()
        .iter()
        .map(|observation| observation.outcome())
    {
        assert_ne!(
            outcome,
            CssStopOpacityQualificationOutcome::Qualified(CssStopOpacityValue::DirectNumberLiteral)
        );
    }
}

#[test]
fn multiple_declarations_preserve_distinct_run_local_occurrence_and_placement() {
    let result = qualify(1808, "a{stop-opacity:.5;stop-opacity:50%;stop-opacity:-1;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectNumber,
        ],
    );
    assert_eq!(result.stop_opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.stop_opacity_observations()[1].occurrence_index(), 1);
    assert_eq!(result.stop_opacity_observations()[2].occurrence_index(), 2);
    assert_eq!(
        result.stop_opacity_observations()[0].outcome(),
        CssStopOpacityQualificationOutcome::Qualified(CssStopOpacityValue::DirectNumberLiteral)
    );
    assert_eq!(
        result.stop_opacity_observations()[1].outcome(),
        CssStopOpacityQualificationOutcome::Qualified(CssStopOpacityValue::DirectPercentageLiteral)
    );
    assert_eq!(
        result.stop_opacity_observations()[2].outcome(),
        CssStopOpacityQualificationOutcome::Qualified(CssStopOpacityValue::DirectNumberLiteral)
    );
}

#[test]
fn duplicate_stop_opacity_declarations_keep_distinct_run_local_placement() {
    let result = qualify(1809, "a{stop-opacity:50%;}b{stop-opacity:50%;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
        ],
    );
    assert_eq!(result.stop_opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.stop_opacity_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.stop_opacity_observations()[0]
            .placement()
            .context_id(),
        result.stop_opacity_observations()[1]
            .placement()
            .context_id(),
    );
}

// Property dispatch isolation: only `stop-opacity` may produce
// `stop-opacity` observations. `opacity`, `fill-opacity`, `stroke-opacity`,
// `stroke-linecap`, and an unrelated property must not leak into or be
// affected by `stop-opacity` dispatch.
#[test]
fn dispatch_is_isolated_from_opacity_fill_opacity_stroke_opacity_stroke_linecap_and_unrelated_properties()
 {
    let result = qualify(
        1810,
        concat!(
            "a{opacity:.5;}",
            "b{fill-opacity:.5;}",
            "c{stroke-opacity:.5;}",
            "d{stop-opacity:.5;}",
            "e{stroke-linecap:round;}",
            "f{color:red;}",
        ),
    );

    assert_eq!(result.opacity_observations().len(), 1);
    assert_eq!(result.fill_opacity_observations().len(), 1);
    assert_eq!(result.stroke_opacity_observations().len(), 1);
    assert_eq!(result.stop_opacity_observations().len(), 1);
    assert_eq!(result.stroke_linecap_observations().len(), 1);

    assert_eq!(result.opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.fill_opacity_observations()[0].occurrence_index(), 1);
    assert_eq!(
        result.stroke_opacity_observations()[0].occurrence_index(),
        2
    );
    assert_eq!(result.stop_opacity_observations()[0].occurrence_index(), 3);
    assert_eq!(
        result.stroke_linecap_observations()[0].occurrence_index(),
        4
    );

    assert_eq!(
        result.opacity_observations()[0].outcome(),
        CssOpacityQualificationOutcome::Qualified(CssOpacityValue::DirectNumberLiteral)
    );
    assert_eq!(
        result.fill_opacity_observations()[0].outcome(),
        CssFillOpacityQualificationOutcome::Qualified(CssFillOpacityValue::DirectNumberLiteral)
    );
    assert_eq!(
        result.stroke_opacity_observations()[0].outcome(),
        CssStrokeOpacityQualificationOutcome::Qualified(CssStrokeOpacityValue::DirectNumberLiteral)
    );
    assert_eq!(
        result.stop_opacity_observations()[0].outcome(),
        CssStopOpacityQualificationOutcome::Qualified(CssStopOpacityValue::DirectNumberLiteral)
    );
    assert_eq!(
        result.stroke_linecap_observations()[0].outcome(),
        CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Round)
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_stop_opacity_observations() {
    for (source_id, css) in [
        (1811, "@font-face{stop-opacity:1;}"),
        (1812, "@page{stop-opacity:1;}"),
        (1813, "@page{@top-left{stop-opacity:1;}}"),
        (1814, "@keyframes k{from{stop-opacity:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.stop_opacity_observations().is_empty(),
            "nonordinary declaration context produced a stop-opacity observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_stop_opacity_prefix() {
    let result = qualify_with_limits(
        1815,
        "a{stop-opacity:300%;stop-opacity:-1;}",
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
fn repeated_and_cross_source_stop_opacity_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{stop-opacity:-1;}",
        "b{stop-opacity:300%;}",
        "c{stop-opacity:calc(2);}",
        "d{stop-opacity:var(--opacity);}",
        "e{opacity:50%;}",
        "f{fill-opacity:.5;}",
        "g{stroke-opacity:.5;}",
        "h{stroke-linecap:butt;}",
    );
    let first = qualify(1816, css);
    let repeated = qualify(1816, css);
    let another_source = qualify(1817, css);

    assert_eq!(
        first.stop_opacity_observations(),
        repeated.stop_opacity_observations()
    );
    assert_eq!(
        first.stop_opacity_observations(),
        another_source.stop_opacity_observations()
    );
    assert_eq!(
        first.opacity_observations(),
        repeated.opacity_observations()
    );
    assert_eq!(
        first.fill_opacity_observations(),
        repeated.fill_opacity_observations()
    );
    assert_eq!(
        first.stroke_opacity_observations(),
        repeated.stroke_opacity_observations()
    );
    assert_eq!(
        first.stroke_linecap_observations(),
        repeated.stroke_linecap_observations()
    );
}
