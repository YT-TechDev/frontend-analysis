use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFillOpacityQualificationOutcome, CssFillOpacityValue, CssOpacityQualificationOutcome,
    CssOpacityValue, CssStrokeLinecapQualificationOutcome, CssStrokeLinecapValue,
    CssStrokeOpacityQualificationOutcome, CssStrokeOpacityUnsupportedReason, CssStrokeOpacityValue,
    CssValueQualificationRunResult, run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssStrokeOpacityQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectNumber => CssStrokeOpacityQualificationOutcome::Qualified(
            CssStrokeOpacityValue::DirectNumberLiteral,
        ),
        ExpectedOutcome::DirectPercentage => CssStrokeOpacityQualificationOutcome::Qualified(
            CssStrokeOpacityValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssStrokeOpacityQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssStrokeOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssStrokeOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeOpacityUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssStrokeOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeOpacityUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssStrokeOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeOpacityUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .stroke_opacity_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

// Pinned WPT corroboration (svg/painting/parsing/stroke-opacity-valid.svg):
// `-1, 0.5, 3, -100%, 50%, 300%` remain authored-valid direct literals. The
// `[0,1]`/`[0%,100%]` clamp is computed-value behavior and is explicitly out
// of scope for this leaf.
#[test]
fn out_of_range_direct_literals_remain_qualified_not_invalid() {
    let result = qualify(
        1700,
        concat!(
            "a{stroke-opacity:-1;}",
            "b{stroke-opacity:0.5;}",
            "c{stroke-opacity:3;}",
            "d{stroke-opacity:-100%;}",
            "e{stroke-opacity:50%;}",
            "f{stroke-opacity:300%;}",
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
        1701,
        concat!(
            "a{stroke-opacity:0;}",
            "b{stroke-opacity:+0;}",
            "c{stroke-opacity:-0;}",
            "d{stroke-opacity:.5;}",
            "e{stroke-opacity:-.5;}",
            "f{stroke-opacity:1;}",
            "g{stroke-opacity:-1;}",
            "h{stroke-opacity:3;}",
            "i{stroke-opacity:1e0;}",
            "j{stroke-opacity:1E0;}",
            "k{stroke-opacity:+1e0;}",
            "l{stroke-opacity:-1e2;}",
            "m{stroke-opacity:0e100;}",
            "n{stroke-opacity:-0e100;}",
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
        1702,
        concat!(
            "a{stroke-opacity:0%;}",
            "b{stroke-opacity:+0%;}",
            "c{stroke-opacity:-0%;}",
            "d{stroke-opacity:50%;}",
            "e{stroke-opacity:100%;}",
            "f{stroke-opacity:-100%;}",
            "g{stroke-opacity:300%;}",
            "h{stroke-opacity:.5%;}",
            "i{stroke-opacity:-.5%;}",
            "j{stroke-opacity:1e2%;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::DirectPercentage; 10]);
}

// Pinned WPT corroboration (svg/painting/parsing/stroke-opacity-invalid.svg):
// `1.` and `2 3` are authored-invalid, plus the repository's established
// wrong-token-class/cardinality invalid matrix used by the accepted
// `opacity`/`fill-opacity` leaves.
#[test]
fn wrong_token_class_and_cardinality_mismatches_are_invalid() {
    let result = qualify(
        1703,
        concat!(
            "a{stroke-opacity:;}",
            "b{stroke-opacity:1.;}",
            "c{stroke-opacity:2 3;}",
            "d{stroke-opacity:1px;}",
            "e{stroke-opacity:\"1\";}",
            "f{stroke-opacity:auto;}",
            "g{stroke-opacity:#fff;}",
            "h{stroke-opacity:1,2;}",
            "i{stroke-opacity:1 50%;}",
            "j{stroke-opacity:50% 1;}",
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
        1704,
        concat!(
            "a{stroke-opacity:/**/300%/**/!important;}",
            "b{stroke-opacity:/**/-1/**/!important;}",
            "c{stroke-opacity:+ 1;}",
            "d{stroke-opacity:+/**/1;}",
            "e{stroke-opacity:- 50%;}",
            "f{stroke-opacity:-/**/50%;}",
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
        1705,
        concat!(
            "a{STROKE-OPACITY:0.5;}",
            "b{Stroke-Opacity:50%;}",
            "c{stroke-OPACITY:-1;}",
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
        1706,
        concat!(
            "a{stroke-opacity:1 2;}",
            "b{stroke-opacity:50% 1;}",
            "c{stroke-opacity:1 50%;}",
            "d{stroke-opacity:(1);}",
            "e{stroke-opacity:calc(.5);}",
            "f{stroke-opacity:min(0,1);}",
            "g{stroke-opacity:max(0,1);}",
            "h{stroke-opacity:clamp(0,.5,1);}",
            "i{stroke-opacity:foo();}",
            "j{stroke-opacity:foo() 1;}",
            "k{stroke-opacity:1 foo();}",
            "l{stroke-opacity:1 calc(.5);}",
            "m{stroke-opacity:calc(.5) 1;}",
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
        1707,
        concat!(
            "a{stroke-opacity:initial;}",
            "b{stroke-opacity:inherit;}",
            "c{stroke-opacity:unset;}",
            "d{stroke-opacity:revert;}",
            "e{stroke-opacity:revert-layer;}",
            "f{stroke-opacity:revert-rule;}",
            "g{stroke-opacity:1 initial;}",
            "h{stroke-opacity:var(--opacity);}",
            "i{stroke-opacity:env(opacity);}",
            "j{stroke-opacity:attr(data-opacity);}",
            "k{stroke-opacity:--opacity();}",
            "l{stroke-opacity:-1 var(--opacity);}",
            "m{stroke-opacity:calc(var(--opacity));}",
            "n{stroke-opacity:first-valid(1,50%);}",
            "o{stroke-opacity:cycle(1,50%);}",
            "p{stroke-opacity:interpolate(50%,0:0,1:1);}",
            "q{stroke-opacity:first-valid(1,50%) 2;}",
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

    // The initial value of `stroke-opacity` is `1`, but authored `initial`
    // must never be synthesized into `Qualified(DirectNumberLiteral)`: it
    // is a CSS-wide mechanism, not authored numeric identity.
    for outcome in result
        .stroke_opacity_observations()
        .iter()
        .map(|observation| observation.outcome())
    {
        assert_ne!(
            outcome,
            CssStrokeOpacityQualificationOutcome::Qualified(
                CssStrokeOpacityValue::DirectNumberLiteral
            )
        );
    }
}

#[test]
fn multiple_declarations_preserve_distinct_run_local_occurrence_and_placement() {
    let result = qualify(
        1708,
        "a{stroke-opacity:.5;stroke-opacity:50%;stroke-opacity:-1;}",
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectNumber,
        ],
    );
    assert_eq!(
        result.stroke_opacity_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.stroke_opacity_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(
        result.stroke_opacity_observations()[2].occurrence_index(),
        2
    );
    assert_eq!(
        result.stroke_opacity_observations()[0].outcome(),
        CssStrokeOpacityQualificationOutcome::Qualified(CssStrokeOpacityValue::DirectNumberLiteral)
    );
    assert_eq!(
        result.stroke_opacity_observations()[1].outcome(),
        CssStrokeOpacityQualificationOutcome::Qualified(
            CssStrokeOpacityValue::DirectPercentageLiteral
        )
    );
    assert_eq!(
        result.stroke_opacity_observations()[2].outcome(),
        CssStrokeOpacityQualificationOutcome::Qualified(CssStrokeOpacityValue::DirectNumberLiteral)
    );
}

#[test]
fn duplicate_stroke_opacity_declarations_keep_distinct_run_local_placement() {
    let result = qualify(1709, "a{stroke-opacity:50%;}b{stroke-opacity:50%;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
        ],
    );
    assert_eq!(
        result.stroke_opacity_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.stroke_opacity_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.stroke_opacity_observations()[0]
            .placement()
            .context_id(),
        result.stroke_opacity_observations()[1]
            .placement()
            .context_id(),
    );
}

// Property dispatch isolation: only `stroke-opacity` may produce
// `stroke-opacity` observations. `opacity`, `fill-opacity`,
// `stroke-linecap`, and an unrelated property must not leak into or be
// affected by `stroke-opacity` dispatch, and prefix matching over the
// `stroke-` family is not authorized.
#[test]
fn dispatch_is_isolated_from_opacity_fill_opacity_stroke_linecap_and_unrelated_properties() {
    let result = qualify(
        1710,
        concat!(
            "a{opacity:.5;}",
            "b{fill-opacity:.5;}",
            "c{stroke-opacity:.5;}",
            "d{stroke-linecap:round;}",
            "e{color:red;}",
        ),
    );

    assert_eq!(result.opacity_observations().len(), 1);
    assert_eq!(result.fill_opacity_observations().len(), 1);
    assert_eq!(result.stroke_opacity_observations().len(), 1);
    assert_eq!(result.stroke_linecap_observations().len(), 1);

    assert_eq!(result.opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.fill_opacity_observations()[0].occurrence_index(), 1);
    assert_eq!(
        result.stroke_opacity_observations()[0].occurrence_index(),
        2
    );
    assert_eq!(
        result.stroke_linecap_observations()[0].occurrence_index(),
        3
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
        result.stroke_linecap_observations()[0].outcome(),
        CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Round)
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_stroke_opacity_observations() {
    for (source_id, css) in [
        (1711, "@font-face{stroke-opacity:1;}"),
        (1712, "@page{stroke-opacity:1;}"),
        (1713, "@page{@top-left{stroke-opacity:1;}}"),
        (1714, "@keyframes k{from{stroke-opacity:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.stroke_opacity_observations().is_empty(),
            "nonordinary declaration context produced a stroke-opacity observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_stroke_opacity_prefix() {
    let result = qualify_with_limits(
        1715,
        "a{stroke-opacity:300%;stroke-opacity:-1;}",
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
fn repeated_and_cross_source_stroke_opacity_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{stroke-opacity:-1;}",
        "b{stroke-opacity:300%;}",
        "c{stroke-opacity:calc(2);}",
        "d{stroke-opacity:var(--opacity);}",
        "e{opacity:50%;}",
        "f{fill-opacity:.5;}",
        "g{stroke-linecap:butt;}",
    );
    let first = qualify(1716, css);
    let repeated = qualify(1716, css);
    let another_source = qualify(1717, css);

    assert_eq!(
        first.stroke_opacity_observations(),
        repeated.stroke_opacity_observations()
    );
    assert_eq!(
        first.stroke_opacity_observations(),
        another_source.stroke_opacity_observations()
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
        first.stroke_linecap_observations(),
        repeated.stroke_linecap_observations()
    );
}
