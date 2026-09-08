use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssAspectRatioQualificationOutcome, CssAspectRatioRatioValue, CssAspectRatioUnsupportedReason,
    CssAspectRatioValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    RatioSingle,
    RatioPair,
    AutoRatioSingle,
    AutoRatioPair,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssAspectRatioQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssAspectRatioQualificationOutcome::Qualified(CssAspectRatioValue::Auto)
        }
        ExpectedOutcome::RatioSingle => CssAspectRatioQualificationOutcome::Qualified(
            CssAspectRatioValue::Ratio(CssAspectRatioRatioValue::Single),
        ),
        ExpectedOutcome::RatioPair => CssAspectRatioQualificationOutcome::Qualified(
            CssAspectRatioValue::Ratio(CssAspectRatioRatioValue::Pair),
        ),
        ExpectedOutcome::AutoRatioSingle => CssAspectRatioQualificationOutcome::Qualified(
            CssAspectRatioValue::AutoAndRatio(CssAspectRatioRatioValue::Single),
        ),
        ExpectedOutcome::AutoRatioPair => CssAspectRatioQualificationOutcome::Qualified(
            CssAspectRatioValue::AutoAndRatio(CssAspectRatioRatioValue::Pair),
        ),
        ExpectedOutcome::Invalid => {
            CssAspectRatioQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssAspectRatioQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAspectRatioUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssAspectRatioQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAspectRatioUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssAspectRatioQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAspectRatioUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssAspectRatioQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssAspectRatioUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .aspect_ratio_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_valid_shapes_are_qualified() {
    let result = qualify(
        5660,
        concat!(
            "a{aspect-ratio:auto;}",
            "b{aspect-ratio:16;}",
            "c{aspect-ratio:16 / 9;}",
            "d{aspect-ratio:auto 16;}",
            "e{aspect-ratio:16 auto;}",
            "f{aspect-ratio:auto 16 / 9;}",
            "g{aspect-ratio:16 / 9 auto;}",
            "h{aspect-ratio:0;}",
            "i{aspect-ratio:0 / 0;}",
            "j{aspect-ratio:1 / 0;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::RatioSingle,
            ExpectedOutcome::RatioPair,
            ExpectedOutcome::AutoRatioSingle,
            ExpectedOutcome::AutoRatioSingle,
            ExpectedOutcome::AutoRatioPair,
            ExpectedOutcome::AutoRatioPair,
            ExpectedOutcome::RatioSingle,
            ExpectedOutcome::RatioPair,
            ExpectedOutcome::RatioPair,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn operand_grouping_duplicate_and_interleaving_failures_are_invalid() {
    let result = qualify(
        5661,
        concat!(
            "a{aspect-ratio:auto auto;}",
            "b{aspect-ratio:16 / 9 4 / 3;}",
            "c{aspect-ratio:auto 16 / 9 auto;}",
            "d{aspect-ratio:16 auto / 9;}",
            "e{aspect-ratio:auto / 9;}",
            "f{aspect-ratio:16 / auto;}",
            "g{aspect-ratio:16 / 9 / 4;}",
            "h{aspect-ratio:16 // 9;}",
            "i{aspect-ratio:16 9;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 9]);
}

#[test]
fn total_component_cardinality_alone_cannot_decide_operand_grouping() {
    // Load-bearing pair: both `auto auto` and `auto 16` have exactly two
    // top-level components, but only operand identity/grouping distinguishes
    // the rejected duplicate from the accepted `auto || <ratio>` shape.
    let result = qualify(
        5662,
        concat!("a{aspect-ratio:auto auto;}", "b{aspect-ratio:auto 16;}"),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Invalid, ExpectedOutcome::AutoRatioSingle],
    );
}

#[test]
fn ratio_operand_contiguity_is_required_regardless_of_authored_order() {
    let result = qualify(
        5663,
        concat!(
            "a{aspect-ratio:auto 16 / 9;}",
            "b{aspect-ratio:16 / 9 auto;}",
            "c{aspect-ratio:16 auto / 9;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::AutoRatioPair,
            ExpectedOutcome::AutoRatioPair,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn direct_ratio_component_range_failures_are_invalid() {
    let result = qualify(
        5664,
        concat!(
            "a{aspect-ratio:-1;}",
            "b{aspect-ratio:-1 / 2;}",
            "c{aspect-ratio:1 / -2;}",
            "d{aspect-ratio:auto -1 / 2;}",
            "e{aspect-ratio:-1 / 2 auto;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

#[test]
fn signed_zero_follows_the_existing_direct_number_boundary() {
    // Not a fresh signed-zero investigation: this reuses the already
    // accepted direct `<number [0,∞]>` predicate unchanged, under which a
    // literal all-zero-digit negative spelling remains in range.
    let result = qualify(
        5665,
        concat!(
            "a{aspect-ratio:-0;}",
            "b{aspect-ratio:-0 / 1;}",
            "c{aspect-ratio:1 / -0;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::RatioSingle,
            ExpectedOutcome::RatioPair,
            ExpectedOutcome::RatioPair,
        ],
    );
}

#[test]
fn residual_function_backed_numeric_positions_are_unsupported() {
    let result = qualify(
        5666,
        concat!(
            "a{aspect-ratio:calc(16);}",
            "b{aspect-ratio:calc(16) / 9;}",
            "c{aspect-ratio:16 / calc(9);}",
            "d{aspect-ratio:calc(16) / calc(9);}",
            "e{aspect-ratio:auto calc(16) / 9;}",
            "f{aspect-ratio:calc(16) / 9 auto;}",
            "g{aspect-ratio:calc(16 / 9);}",
            "h{aspect-ratio:auto calc(16 / 9);}",
            "i{aspect-ratio:calc(16 / 9) auto;}",
            "j{aspect-ratio:calc(calc(16)) / 9 auto;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 10]);
}

#[test]
fn direct_invalid_ratio_evidence_beats_residual_function_unsupported() {
    // Load-bearing regression: the numerator's direct out-of-range Number
    // literal is already decidable, so it must not be softened into
    // `UnsupportedBySelectedValueProfile(FunctionValue)` by the residual
    // Function occupying the other numeric position.
    let result = qualify(
        5667,
        concat!(
            "a{aspect-ratio:-1 / calc(9) auto;}",
            "b{aspect-ratio:auto -1 / calc(9);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 2]);
}

#[test]
fn deferred_substitution_preserves_existing_boundary_regardless_of_placement() {
    let result = qualify(
        5668,
        concat!(
            "a{aspect-ratio:var(--x);}",
            "b{aspect-ratio:auto var(--x);}",
            "c{aspect-ratio:var(--x) auto;}",
            "d{aspect-ratio:auto 16 / var(--d);}",
            "e{aspect-ratio:16 / var(--d) auto;}",
            "f{aspect-ratio:auto 16 / 9 var(--x);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 6]);
}

#[test]
fn whole_value_function_boundary_and_operand_position_misplacement() {
    let whole_value = qualify(5669, "a{aspect-ratio:first-valid(auto 16 / 9);}");
    assert_expected(&whole_value, &[ExpectedOutcome::UnsupportedWholeValue]);

    // A recognized whole-value Function is legal only when it occupies the
    // entire declaration value; at an operand position it cannot satisfy
    // `auto` or `<ratio>`, so it is an invalid placement, not a residual
    // Function-Unsupported outcome.
    let misplaced = qualify(
        5670,
        concat!(
            "a{aspect-ratio:auto first-valid(16 / 9);}",
            "b{aspect-ratio:first-valid(auto) 16 / 9;}",
            "c{aspect-ratio:16 / 9 first-valid(auto);}",
        ),
    );
    assert_expected(&misplaced, &[ExpectedOutcome::Invalid; 3]);
}

#[test]
fn css_wide_keywords_apply_only_to_the_entire_value() {
    let whole_value = qualify(
        5671,
        concat!(
            "a{aspect-ratio:inherit;}",
            "b{aspect-ratio:initial;}",
            "c{aspect-ratio:unset;}",
            "d{aspect-ratio:revert;}",
            "e{aspect-ratio:revert-layer;}",
        ),
    );
    assert_expected(&whole_value, &[ExpectedOutcome::UnsupportedCssWide; 5]);

    let misplaced = qualify(
        5672,
        concat!(
            "a{aspect-ratio:auto inherit;}",
            "b{aspect-ratio:inherit auto;}",
            "c{aspect-ratio:16 / 9 inherit;}",
            "d{aspect-ratio:inherit 16 / 9;}",
        ),
    );
    assert_expected(&misplaced, &[ExpectedOutcome::Invalid; 4]);
}

#[test]
fn comments_and_trivia_are_resolved_from_retained_lexical_evidence_not_source_appearance() {
    let result = qualify(
        5673,
        concat!(
            "a{aspect-ratio:auto/**/16/9;}",
            "b{aspect-ratio:16/9/**/auto;}",
            "c{aspect-ratio:auto/**/16 /**/ / /**/ 9;}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::AutoRatioPair,
            ExpectedOutcome::AutoRatioPair,
            ExpectedOutcome::AutoRatioPair,
        ],
    );

    // `16/**//**/9` retains no ratio `Delim('/')` at all: both apparent
    // slash characters belong to comment delimiters, so the two Numbers
    // remain two adjacent top-level components with no ratio-shaped
    // relation between them, which is malformed grouping, not a valid
    // ratio pair.
    let fake_slash = qualify(5674, "a{aspect-ratio:auto/**/16/**//**/9;}");
    assert_expected(&fake_slash, &[ExpectedOutcome::Invalid]);
}

#[test]
fn a_completed_function_component_ends_at_depth_zero_without_swallowing_later_evidence() {
    // Regression pressure equivalent to PR #564: a Function component must
    // end the instant its matching closer returns block depth to zero, so
    // adjacent top-level evidence with no intervening separator is never
    // hidden inside it.
    let result = qualify(
        5675,
        concat!(
            "a{aspect-ratio:calc(16)/9 auto;}",
            "b{aspect-ratio:calc(16)auto;}",
            "c{aspect-ratio:auto calc(16)/9;}",
        ),
    );
    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 3]);

    // An unmatched trailing closer at depth zero remains ordinary top-level
    // evidence; it does not get folded into the preceding Function
    // component, and its own extra component breaks ratio contiguity.
    let unmatched_closer = qualify(5676, "a{aspect-ratio:calc(16)) auto;}");
    assert_expected(&unmatched_closer, &[ExpectedOutcome::Invalid]);
}

#[test]
fn duplicate_auto_grouping_failure_is_not_softened_by_a_residual_function() {
    // A duplicate `auto` is a directly decidable grouping failure; a
    // residual Function occupying the remaining numeric position must not
    // soften it into a Function-Unsupported outcome.
    let result = qualify(5685, "a{aspect-ratio:auto auto calc(9);}");
    assert_expected(&result, &[ExpectedOutcome::Invalid]);
}

#[test]
fn deferred_substitution_is_checked_before_grouping_even_with_visually_duplicated_auto() {
    // Deferred/arbitrary substitution recognition remains prior to operand
    // grouping because a substitution can change the final top-level
    // structure; a duplicate-looking `auto ... auto` source must not be
    // rejected as malformed grouping before this boundary is checked.
    let result = qualify(5686, "a{aspect-ratio:auto var(--x) auto;}");
    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred]);
}

#[test]
fn one_run_owns_upstream_evidence_and_duplicate_placements_stay_separate() {
    let result = qualify(
        5677,
        "a{aspect-ratio:16;}b{aspect-ratio:auto;}c{color:red;}",
    );

    assert_eq!(result.aspect_ratio_observations().len(), 2);
    assert_eq!(result.aspect_ratio_observations()[0].occurrence_index(), 0);
    assert_eq!(result.aspect_ratio_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.aspect_ratio_observations()[0]
            .placement()
            .context_id(),
        result.aspect_ratio_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (5678, "@font-face{aspect-ratio:16;}"),
        (5679, "@page{aspect-ratio:16;}"),
        (5680, "@keyframes k{from{aspect-ratio:16;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.aspect_ratio_observations().is_empty(),
            "nonordinary declaration context produced an aspect-ratio observation for {css:?}"
        );
    }
}

#[test]
fn important_priority_is_excluded_from_the_semantic_value_window() {
    let result = qualify(5681, "a{aspect-ratio:auto 16 !important;}");
    assert_expected(&result, &[ExpectedOutcome::AutoRatioSingle]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        5682,
        "a{aspect-ratio:16;aspect-ratio:auto 16 / 9;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::RatioSingle]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{aspect-ratio:auto 16 / 9;}",
        "b{aspect-ratio:16 auto / 9;}",
        "c{aspect-ratio:calc(16) / 9;}",
        "d{aspect-ratio:var(--x);}",
    );
    let first = qualify(5683, css);
    let repeated = qualify(5683, css);
    let another_source = qualify(5684, css);

    assert_eq!(
        first.aspect_ratio_observations(),
        repeated.aspect_ratio_observations()
    );
    assert_eq!(
        first.aspect_ratio_observations(),
        another_source.aspect_ratio_observations()
    );
}
