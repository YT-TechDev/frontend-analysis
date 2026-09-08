use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssAnimationIterationCountQualificationOutcome, CssAnimationIterationCountUnsupportedReason,
    CssAnimationIterationCountValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

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

fn qualified(
    values: &[CssAnimationIterationCountValue],
) -> CssAnimationIterationCountQualificationOutcome {
    CssAnimationIterationCountQualificationOutcome::Qualified(values.to_vec())
}

fn unsupported(
    reason: CssAnimationIterationCountUnsupportedReason,
) -> CssAnimationIterationCountQualificationOutcome {
    CssAnimationIterationCountQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssAnimationIterationCountQualificationOutcome],
) {
    let actual: Vec<_> = result
        .animation_iteration_count_observations()
        .iter()
        .map(|observation| observation.outcome().clone())
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn direct_single_and_mixed_list_items_preserve_authored_order() {
    use CssAnimationIterationCountValue::{DirectNumberLiteral, Infinite};

    let result = qualify(
        85720,
        concat!(
            "a{animation-iteration-count:0;}",
            "b{animation-iteration-count:4;}",
            "c{animation-iteration-count:4.5;}",
            "d{animation-iteration-count:+2e3;}",
            "e{animation-iteration-count:infinite;}",
            "f{animation-iteration-count:0, infinite, 3.5;}",
            "g{animation-iteration-count:3.5, 3.5;}",
            "h{animation-iteration-count:INFINITE;}",
            "i{animation-iteration-count:InFiNiTe;}",
            "j{animation-iteration-count:inf\\69 nite;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral]),
            qualified(&[Infinite]),
            qualified(&[DirectNumberLiteral, Infinite, DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral, DirectNumberLiteral]),
            qualified(&[Infinite]),
            qualified(&[Infinite]),
            qualified(&[Infinite]),
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn retained_comma_tokens_and_trivia_define_list_separators() {
    use CssAnimationIterationCountValue::{DirectNumberLiteral, Infinite};

    let result = qualify(
        85721,
        concat!(
            "a{animation-iteration-count:0/**/,/**/infinite;}",
            "b{animation-iteration-count: 0 , infinite , 3 ;}",
            "c{animation-iteration-count:0/*,*/ , /*,*/infinite;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectNumberLiteral, Infinite]),
            qualified(&[DirectNumberLiteral, Infinite, DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral, Infinite]),
        ],
    );
}

#[test]
fn signed_zero_spellings_remain_qualified_direct_zero() {
    use CssAnimationIterationCountValue::DirectNumberLiteral;

    let result = qualify(
        85722,
        concat!(
            "a{animation-iteration-count:-0;}",
            "b{animation-iteration-count:-.0;}",
            "c{animation-iteration-count:-0.0;}",
            "d{animation-iteration-count:-0e100;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral]),
        ],
    );
}

#[test]
fn negative_non_zero_numbers_are_invalid() {
    use CssAnimationIterationCountQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        85723,
        concat!(
            "a{animation-iteration-count:-1;}",
            "b{animation-iteration-count:-.5;}",
            "c{animation-iteration-count:-0.01;}",
        ),
    );

    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn empty_items_missing_separators_and_wrong_direct_items_are_invalid() {
    use CssAnimationIterationCountQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        85724,
        concat!(
            "a{animation-iteration-count:;}",
            "b{animation-iteration-count:,1;}",
            "c{animation-iteration-count:1,;}",
            "d{animation-iteration-count:1,,2;}",
            "e{animation-iteration-count:1 2;}",
            "f{animation-iteration-count:auto;}",
            "g{animation-iteration-count:infinity;}",
            "h{animation-iteration-count:1px;}",
            "i{animation-iteration-count:10%;}",
            "j{animation-iteration-count:1 / 2;}",
        ),
    );

    let expected = vec![Invalid; 10];
    assert_expected(&result, &expected);
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    use CssAnimationIterationCountQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationIterationCountUnsupportedReason::CssWideKeyword;

    let whole = qualify(
        85725,
        concat!(
            "a{animation-iteration-count:inherit;}",
            "b{animation-iteration-count:initial;}",
            "c{animation-iteration-count:unset;}",
            "d{animation-iteration-count:revert;}",
            "e{animation-iteration-count:revert-layer;}",
        ),
    );
    assert_expected(
        &whole,
        &[
            unsupported(CssWideKeyword),
            unsupported(CssWideKeyword),
            unsupported(CssWideKeyword),
            unsupported(CssWideKeyword),
            unsupported(CssWideKeyword),
        ],
    );

    let mixed = qualify(
        85726,
        concat!(
            "a{animation-iteration-count:initial,1;}",
            "b{animation-iteration-count:1,initial;}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&mixed, &expected);
}

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    use CssAnimationIterationCountUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        85727,
        concat!(
            "a{animation-iteration-count:var(--x);}",
            "b{animation-iteration-count:1,var(--x);}",
            "c{animation-iteration-count:var(--x),1;}",
            "d{animation-iteration-count:var(--x,1,2);}",
            "e{animation-iteration-count:1,var(--x,-1);}",
        ),
    );

    assert_expected(
        &result,
        &[
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
        ],
    );
}

#[test]
fn function_backed_numeric_items_are_unsupported_absent_decisive_invalid_evidence() {
    use CssAnimationIterationCountUnsupportedReason::FunctionValue;

    let result = qualify(
        85728,
        concat!(
            "a{animation-iteration-count:calc(2);}",
            "b{animation-iteration-count:1,calc(2);}",
            "c{animation-iteration-count:calc(2),infinite;}",
            "d{animation-iteration-count:min(1, 2),infinite;}",
            "e{animation-iteration-count:foo();}",
            "f{animation-iteration-count:infinite,calc(2),3;}",
        ),
    );

    assert_expected(
        &result,
        &[
            unsupported(FunctionValue),
            unsupported(FunctionValue),
            unsupported(FunctionValue),
            unsupported(FunctionValue),
            unsupported(FunctionValue),
            unsupported(FunctionValue),
        ],
    );
}

#[test]
fn whole_value_function_boundary_does_not_legalize_function_items() {
    use CssAnimationIterationCountQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationIterationCountUnsupportedReason::WholeValueFunction;

    let whole = qualify(85729, "a{animation-iteration-count:first-valid(1, 2);}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let items = qualify(
        85730,
        concat!(
            "a{animation-iteration-count:1,first-valid(2);}",
            "b{animation-iteration-count:first-valid(2),1;}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&items, &expected);
}

#[test]
fn decisive_invalid_evidence_outranks_residual_function_unsupported() {
    use CssAnimationIterationCountQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        85731,
        concat!(
            "a{animation-iteration-count:-1,calc(2);}",
            "b{animation-iteration-count:calc(2),-1;}",
            "c{animation-iteration-count:1,,calc(2);}",
        ),
    );

    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn nested_function_commas_do_not_split_the_outer_list() {
    use CssAnimationIterationCountUnsupportedReason::FunctionValue;

    let result = qualify(85732, "a{animation-iteration-count:min(1, 2), infinite;}");
    assert_expected(&result, &[unsupported(FunctionValue)]);
}

#[test]
fn important_priority_is_outside_the_semantic_list_window() {
    use CssAnimationIterationCountValue::{DirectNumberLiteral, Infinite};

    let result = qualify(85733, "a{animation-iteration-count:0,infinite !important;}");
    assert_expected(&result, &[qualified(&[DirectNumberLiteral, Infinite])]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn duplicate_occurrences_and_existing_leaf_dispatch_remain_separate() {
    use CssAnimationIterationCountValue::{DirectNumberLiteral, Infinite};

    let result = qualify(
        85734,
        concat!(
            "a{animation-iteration-count:3;animation-iteration-count:0,infinite;}",
            "b{animation-play-state:running,paused;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectNumberLiteral]),
            qualified(&[DirectNumberLiteral, Infinite]),
        ],
    );
    assert_eq!(result.animation_iteration_count_observations().len(), 2);
    assert_eq!(
        result.animation_iteration_count_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.animation_iteration_count_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(result.animation_play_state_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_animation_iteration_count_dispatch() {
    for (source_id, css) in [
        (85735, "@font-face{animation-iteration-count:3;}"),
        (85736, "@page{animation-iteration-count:3;}"),
        (85737, "@keyframes k{from{animation-iteration-count:3;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.animation_iteration_count_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    use CssAnimationIterationCountQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        85738,
        concat!(
            "a{animation-iteration-count:1),infinite;}",
            "b{animation-iteration-count:(1,infinite);}",
            "c{animation-iteration-count:foo(1,infinite),infinite);}",
        ),
    );
    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssAnimationIterationCountValue::DirectNumberLiteral;

    let incomplete = qualify_with_limits(
        85739,
        "a{animation-iteration-count:3;}b{animation-iteration-count:0,infinite;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified(&[DirectNumberLiteral])]);

    let css = concat!(
        "a{animation-iteration-count:0,infinite;}",
        "b{animation-iteration-count:0,,infinite;}",
        "c{animation-iteration-count:var(--x);}",
    );
    let first = qualify(85740, css);
    let repeated = qualify(85740, css);
    let another_source = qualify(85741, css);

    assert_eq!(
        first.animation_iteration_count_observations(),
        repeated.animation_iteration_count_observations()
    );
    assert_eq!(
        first.animation_iteration_count_observations(),
        another_source.animation_iteration_count_observations()
    );
}
