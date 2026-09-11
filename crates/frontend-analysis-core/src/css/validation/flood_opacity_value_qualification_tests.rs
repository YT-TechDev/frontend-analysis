use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFillOpacityQualificationOutcome, CssFillOpacityValue, CssFloodOpacityQualificationOutcome,
    CssFloodOpacityUnsupportedReason, CssFloodOpacityValue, CssOpacityQualificationOutcome,
    CssOpacityValue, CssStopOpacityQualificationOutcome, CssStopOpacityValue,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFloodOpacityQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectNumber => CssFloodOpacityQualificationOutcome::Qualified(
            CssFloodOpacityValue::DirectNumberLiteral,
        ),
        ExpectedOutcome::DirectPercentage => CssFloodOpacityQualificationOutcome::Qualified(
            CssFloodOpacityValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssFloodOpacityQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFloodOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFloodOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssFloodOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFloodOpacityUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssFloodOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFloodOpacityUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFloodOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFloodOpacityUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .flood_opacity_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

// Pinned WPT corroboration (css/filter-effects/parsing/flood-opacity-valid.svg):
// `-1, 0.5, 3, -100%, 50%, 300%` remain authored-valid direct literals. The
// `[0,1]`/`[0%,100%]` clamp is computed-value behavior and is explicitly out
// of scope for this leaf.
#[test]
fn out_of_range_direct_literals_remain_qualified_not_invalid() {
    let result = qualify(
        63200,
        concat!(
            "a{flood-opacity:-1;}",
            "b{flood-opacity:0.5;}",
            "c{flood-opacity:3;}",
            "d{flood-opacity:-100%;}",
            "e{flood-opacity:50%;}",
            "f{flood-opacity:300%;}",
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
        63201,
        concat!(
            "a{flood-opacity:0;}",
            "b{flood-opacity:+0;}",
            "c{flood-opacity:-0;}",
            "d{flood-opacity:.5;}",
            "e{flood-opacity:-.5;}",
            "f{flood-opacity:1;}",
            "g{flood-opacity:-1;}",
            "h{flood-opacity:3;}",
            "i{flood-opacity:1e0;}",
            "j{flood-opacity:1E0;}",
            "k{flood-opacity:+1e0;}",
            "l{flood-opacity:-1e2;}",
            "m{flood-opacity:0e100;}",
            "n{flood-opacity:-0e100;}",
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
        63202,
        concat!(
            "a{flood-opacity:0%;}",
            "b{flood-opacity:+0%;}",
            "c{flood-opacity:-0%;}",
            "d{flood-opacity:50%;}",
            "e{flood-opacity:100%;}",
            "f{flood-opacity:-100%;}",
            "g{flood-opacity:300%;}",
            "h{flood-opacity:.5%;}",
            "i{flood-opacity:-.5%;}",
            "j{flood-opacity:1e2%;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::DirectPercentage; 10]);
}

// Pinned WPT corroboration (css/filter-effects/parsing/flood-opacity-invalid.svg):
// `1.` and `2 3` are authored-invalid, plus the repository's established
// wrong-token-class/cardinality invalid matrix used by the accepted
// `opacity`/`fill-opacity`/`stroke-opacity`/`stop-opacity` leaves.
#[test]
fn wrong_token_class_and_cardinality_mismatches_are_invalid() {
    let result = qualify(
        63203,
        concat!(
            "a{flood-opacity:;}",
            "b{flood-opacity:1.;}",
            "c{flood-opacity:2 3;}",
            "d{flood-opacity:1px;}",
            "e{flood-opacity:\"1\";}",
            "f{flood-opacity:auto;}",
            "g{flood-opacity:#fff;}",
            "h{flood-opacity:1,2;}",
            "i{flood-opacity:1 50%;}",
            "j{flood-opacity:50% 1;}",
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
        63204,
        concat!(
            "a{flood-opacity:/**/300%/**/!important;}",
            "b{flood-opacity:/**/-1/**/!important;}",
            "c{flood-opacity:+ 1;}",
            "d{flood-opacity:+/**/1;}",
            "e{flood-opacity:- 50%;}",
            "f{flood-opacity:-/**/50%;}",
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
        63205,
        concat!(
            "a{FLOOD-OPACITY:0.5;}",
            "b{Flood-Opacity:50%;}",
            "c{flood-OPACITY:-1;}",
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

// Pinned WPT math discriminator (css/filter-effects/parsing/flood-opacity-valid.svg):
// `calc(0.5 * sign(10em - 1px))` is CSS-valid but is not evaluated by this
// leaf; it must remain the selected sole-ordinary-function unsupported
// boundary rather than `Qualified` or `InvalidForSelectedValueGrammar`.
#[test]
fn sole_ordinary_function_boundary_matches_the_selected_profile() {
    let result = qualify(
        63206,
        concat!(
            "a{flood-opacity:1 2;}",
            "b{flood-opacity:50% 1;}",
            "c{flood-opacity:1 50%;}",
            "d{flood-opacity:(1);}",
            "e{flood-opacity:calc(.5);}",
            "f{flood-opacity:min(0,1);}",
            "g{flood-opacity:max(0,1);}",
            "h{flood-opacity:clamp(0,.5,1);}",
            "i{flood-opacity:foo();}",
            "j{flood-opacity:foo() 1;}",
            "k{flood-opacity:1 foo();}",
            "l{flood-opacity:1 calc(.5);}",
            "m{flood-opacity:calc(.5) 1;}",
            "n{flood-opacity:calc(0.5 * sign(10em - 1px));}",
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
            ExpectedOutcome::UnsupportedFunction,
        ],
    );
}

#[test]
fn css_wide_substitution_and_whole_value_boundaries_are_preserved() {
    let result = qualify(
        63207,
        concat!(
            "a{flood-opacity:initial;}",
            "b{flood-opacity:inherit;}",
            "c{flood-opacity:unset;}",
            "d{flood-opacity:revert;}",
            "e{flood-opacity:revert-layer;}",
            "f{flood-opacity:revert-rule;}",
            "g{flood-opacity:1 initial;}",
            "h{flood-opacity:var(--opacity);}",
            "i{flood-opacity:env(opacity);}",
            "j{flood-opacity:attr(data-opacity);}",
            "k{flood-opacity:--opacity();}",
            "l{flood-opacity:-1 var(--opacity);}",
            "m{flood-opacity:calc(var(--opacity));}",
            "n{flood-opacity:first-valid(1,50%);}",
            "o{flood-opacity:cycle(1,50%);}",
            "p{flood-opacity:interpolate(50%,0:0,1:1);}",
            "q{flood-opacity:first-valid(1,50%) 2;}",
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

    // The initial value of `flood-opacity` is `1`, but authored `initial`
    // must never be synthesized into `Qualified(DirectNumberLiteral)`: it
    // is a CSS-wide mechanism, not authored numeric identity.
    for outcome in result
        .flood_opacity_observations()
        .iter()
        .map(|observation| observation.outcome())
    {
        assert_ne!(
            outcome,
            CssFloodOpacityQualificationOutcome::Qualified(
                CssFloodOpacityValue::DirectNumberLiteral
            )
        );
    }
}

#[test]
fn multiple_declarations_preserve_distinct_run_local_occurrence_and_placement() {
    let result = qualify(
        63208,
        "a{flood-opacity:.5;flood-opacity:50%;flood-opacity:-1;}",
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectNumber,
        ],
    );
    assert_eq!(result.flood_opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.flood_opacity_observations()[1].occurrence_index(), 1);
    assert_eq!(result.flood_opacity_observations()[2].occurrence_index(), 2);
    assert_eq!(
        result.flood_opacity_observations()[0].outcome(),
        CssFloodOpacityQualificationOutcome::Qualified(CssFloodOpacityValue::DirectNumberLiteral)
    );
    assert_eq!(
        result.flood_opacity_observations()[1].outcome(),
        CssFloodOpacityQualificationOutcome::Qualified(
            CssFloodOpacityValue::DirectPercentageLiteral
        )
    );
    assert_eq!(
        result.flood_opacity_observations()[2].outcome(),
        CssFloodOpacityQualificationOutcome::Qualified(CssFloodOpacityValue::DirectNumberLiteral)
    );
}

#[test]
fn duplicate_flood_opacity_declarations_keep_distinct_run_local_placement() {
    let result = qualify(63209, "a{flood-opacity:50%;}b{flood-opacity:50%;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
        ],
    );
    assert_eq!(result.flood_opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.flood_opacity_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.flood_opacity_observations()[0]
            .placement()
            .context_id(),
        result.flood_opacity_observations()[1]
            .placement()
            .context_id(),
    );
}

// Property dispatch isolation: only `flood-opacity` may produce
// `flood-opacity` observations. `opacity`, `fill-opacity`, `stroke-opacity`,
// `stop-opacity`, and an unrelated property must not leak into or be
// affected by `flood-opacity` dispatch.
#[test]
fn dispatch_is_isolated_from_opacity_fill_opacity_stroke_opacity_stop_opacity_and_unrelated_properties()
 {
    let result = qualify(
        63210,
        concat!(
            "a{opacity:.5;}",
            "b{fill-opacity:.5;}",
            "c{stroke-opacity:.5;}",
            "d{stop-opacity:.5;}",
            "e{flood-opacity:.5;}",
            "f{color:red;}",
        ),
    );

    assert_eq!(result.opacity_observations().len(), 1);
    assert_eq!(result.fill_opacity_observations().len(), 1);
    assert_eq!(result.stroke_opacity_observations().len(), 1);
    assert_eq!(result.stop_opacity_observations().len(), 1);
    assert_eq!(result.flood_opacity_observations().len(), 1);

    assert_eq!(result.opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.fill_opacity_observations()[0].occurrence_index(), 1);
    assert_eq!(
        result.stroke_opacity_observations()[0].occurrence_index(),
        2
    );
    assert_eq!(result.stop_opacity_observations()[0].occurrence_index(), 3);
    assert_eq!(result.flood_opacity_observations()[0].occurrence_index(), 4);

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
        result.flood_opacity_observations()[0].outcome(),
        CssFloodOpacityQualificationOutcome::Qualified(CssFloodOpacityValue::DirectNumberLiteral)
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_flood_opacity_observations() {
    for (source_id, css) in [
        (63211, "@font-face{flood-opacity:1;}"),
        (63212, "@page{flood-opacity:1;}"),
        (63213, "@page{@top-left{flood-opacity:1;}}"),
        (63214, "@keyframes k{from{flood-opacity:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.flood_opacity_observations().is_empty(),
            "nonordinary declaration context produced a flood-opacity observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_flood_opacity_prefix() {
    let result = qualify_with_limits(
        63215,
        "a{flood-opacity:300%;flood-opacity:-1;}",
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
fn repeated_and_cross_source_flood_opacity_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{flood-opacity:-1;}",
        "b{flood-opacity:300%;}",
        "c{flood-opacity:calc(2);}",
        "d{flood-opacity:var(--opacity);}",
        "e{opacity:50%;}",
        "f{fill-opacity:.5;}",
        "g{stroke-opacity:.5;}",
        "h{stop-opacity:.5;}",
    );
    let first = qualify(63216, css);
    let repeated = qualify(63216, css);
    let another_source = qualify(63217, css);

    assert_eq!(
        first.flood_opacity_observations(),
        repeated.flood_opacity_observations()
    );
    assert_eq!(
        first.flood_opacity_observations(),
        another_source.flood_opacity_observations()
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
        first.stop_opacity_observations(),
        repeated.stop_opacity_observations()
    );
}
