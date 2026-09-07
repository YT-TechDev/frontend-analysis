use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBorderSpacingQualificationOutcome, CssBorderSpacingUnsupportedReason, CssBorderSpacingValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Single,
    Pair,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssBorderSpacingQualificationOutcome {
    match expected {
        ExpectedOutcome::Single => {
            CssBorderSpacingQualificationOutcome::Qualified(CssBorderSpacingValue::Single)
        }
        ExpectedOutcome::Pair => {
            CssBorderSpacingQualificationOutcome::Qualified(CssBorderSpacingValue::Pair)
        }
        ExpectedOutcome::Invalid => {
            CssBorderSpacingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssBorderSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderSpacingUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssBorderSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderSpacingUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssBorderSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderSpacingUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssBorderSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderSpacingUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .border_spacing_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_single_and_paired_lengths_are_qualified() {
    let result = qualify(
        1200,
        concat!(
            "a{border-spacing:1px;}",
            "b{border-spacing:1px 2px;}",
            "c{border-spacing:0;}",
            "d{border-spacing:0 0;}",
            "e{border-spacing:-0px 1px;}",
            "f{border-spacing:+1px 2e1px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Single,
            ExpectedOutcome::Pair,
            ExpectedOutcome::Single,
            ExpectedOutcome::Pair,
            ExpectedOutcome::Pair,
            ExpectedOutcome::Pair,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn direct_out_of_range_wrong_token_class_and_cardinality_mismatches_are_invalid() {
    let result = qualify(
        1201,
        concat!(
            "a{border-spacing:-1px 2px;}",
            "b{border-spacing:1px 2%;}",
            "c{border-spacing:1 2px;}",
            "d{border-spacing:1px 2px 3px;}",
            "e{border-spacing:1px / 2px;}",
            "f{border-spacing:1px, 2px;}",
            "g{border-spacing:1px );}",
            "h{border-spacing:\"1px\";}",
            "i{border-spacing:;}",
        ),
    );

    assert_expected(
        &result,
        &[
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
}

#[test]
fn direct_negative_literal_versus_function_backed_negative_result_stays_distinct() {
    // Load-bearing regression pair: a decidable direct out-of-range literal is
    // Invalid, while the same authored magnitude behind an unevaluated
    // Function remains a bounded-profile Unsupported boundary.
    let result = qualify(
        1202,
        concat!(
            "a{border-spacing:-1px 2px;}",
            "b{border-spacing:calc(-1px) 2px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::UnsupportedFunction,
        ],
    );
}

#[test]
fn function_headed_components_in_either_or_both_positions_are_unsupported() {
    let result = qualify(
        1203,
        concat!(
            "a{border-spacing:calc(1px) 2px;}",
            "b{border-spacing:1px calc(2px);}",
            "c{border-spacing:calc(1px) calc(2px);}",
            "d{border-spacing:calc(2) 1px;}",
            "e{border-spacing:min(1px,2px) 2px;}",
            "f{border-spacing:foo(1px) 2px;}",
            "g{border-spacing:sibling-index() 1px;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 7]);
}

#[test]
fn cardinality_and_whole_value_placement_classification_order_is_load_bearing() {
    // Cardinality is decidable before Function support is considered, so a
    // cardinality violation stays Invalid even with a Function present.
    let cardinality = qualify(1204, "a{border-spacing:calc(1px) 2px 3px;}");
    assert_expected(&cardinality, &[ExpectedOutcome::Invalid]);

    // A recognized whole-value Function is legal only when it occupies the
    // entire declaration value; at a non-whole-value component position it is
    // an invalid placement, not a residual-Function Unsupported outcome.
    let misplaced = qualify(
        1205,
        concat!(
            "a{border-spacing:1px first-valid(2px);}",
            "b{border-spacing:first-valid(1px) 2px;}",
        ),
    );
    assert_expected(
        &misplaced,
        &[ExpectedOutcome::Invalid, ExpectedOutcome::Invalid],
    );

    let whole_value = qualify(1206, "a{border-spacing:first-valid(1px 2px);}");
    assert_expected(&whole_value, &[ExpectedOutcome::UnsupportedWholeValue]);
}

#[test]
fn nested_function_content_does_not_inflate_top_level_cardinality() {
    let result = qualify(
        1207,
        concat!(
            "a{border-spacing:calc(calc(1px)) 2px;}",
            "b{border-spacing:calc(var(--a)) 2px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedDeferred,
        ],
    );
}

#[test]
fn deferred_substitution_and_dashed_functions_preserve_existing_boundary() {
    let result = qualify(
        1208,
        concat!(
            "a{border-spacing:var(--gap);}",
            "b{border-spacing:env(gap);}",
            "c{border-spacing:attr(data-gap);}",
            "d{border-spacing:--gap();}",
            "e{border-spacing:1px var(--gap);}",
            "f{border-spacing:var(--gap) 1px;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 6]);
}

#[test]
fn css_wide_keywords_apply_only_to_the_entire_value() {
    let result = qualify(
        1209,
        concat!(
            "a{border-spacing:initial;}",
            "b{border-spacing:inherit;}",
            "c{border-spacing:unset;}",
            "d{border-spacing:revert;}",
            "e{border-spacing:revert-layer;}",
            "f{border-spacing:revert-rule;}",
            "g{border-spacing:1px initial;}",
            "h{border-spacing:initial 1px;}",
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
fn comments_and_important_priority_preserve_component_boundaries() {
    let result = qualify(
        1210,
        concat!(
            "a{border-spacing:/**/1px/**/2px/**/!important;}",
            "b{border-spacing:1px/**/2px;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Pair, ExpectedOutcome::Pair]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn one_run_owns_upstream_evidence_and_duplicate_placements_stay_separate() {
    let result = qualify(
        1211,
        "a{border-spacing:1px;}b{border-spacing:1px 2px;}c{color:red;}",
    );

    assert_eq!(result.border_spacing_observations().len(), 2);
    assert_eq!(
        result.border_spacing_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.border_spacing_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.border_spacing_observations()[0]
            .placement()
            .context_id(),
        result.border_spacing_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1212, "@font-face{border-spacing:1px;}"),
        (1213, "@page{border-spacing:1px;}"),
        (1214, "@keyframes k{from{border-spacing:1px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.border_spacing_observations().is_empty(),
            "nonordinary declaration context produced a border-spacing observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1215,
        "a{border-spacing:1px;border-spacing:1px 2px;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::Single]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{border-spacing:1px 2px;}",
        "b{border-spacing:-1px 2px;}",
        "c{border-spacing:calc(1px) 2px;}",
        "d{border-spacing:var(--gap);}",
    );
    let first = qualify(1216, css);
    let repeated = qualify(1216, css);
    let another_source = qualify(1217, css);

    assert_eq!(
        first.border_spacing_observations(),
        repeated.border_spacing_observations()
    );
    assert_eq!(
        first.border_spacing_observations(),
        another_source.border_spacing_observations()
    );
}
