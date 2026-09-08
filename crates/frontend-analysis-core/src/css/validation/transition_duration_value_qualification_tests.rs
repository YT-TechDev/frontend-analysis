use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTransitionDurationQualificationOutcome, CssTransitionDurationUnsupportedReason,
    CssTransitionDurationValue, CssValueQualificationRunResult, run,
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

fn qualified(values: &[CssTransitionDurationValue]) -> CssTransitionDurationQualificationOutcome {
    CssTransitionDurationQualificationOutcome::Qualified(values.to_vec())
}

fn unsupported(
    reason: CssTransitionDurationUnsupportedReason,
) -> CssTransitionDurationQualificationOutcome {
    CssTransitionDurationQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssTransitionDurationQualificationOutcome],
) {
    let actual: Vec<_> = result
        .transition_duration_observations()
        .iter()
        .map(|observation| observation.outcome().clone())
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn direct_non_negative_time_literals_qualify_regardless_of_numeric_spelling_or_unit_case() {
    use CssTransitionDurationValue::DirectTimeLiteral;

    let result = qualify(
        97600,
        concat!(
            "a{transition-duration:0s;}",
            "b{transition-duration:-0s;}",
            "c{transition-duration:+0s;}",
            "d{transition-duration:.0s;}",
            "e{transition-duration:-.0s;}",
            "f{transition-duration:-0.0s;}",
            "g{transition-duration:-0e100ms;}",
            "h{transition-duration:1s;}",
            "i{transition-duration:.5s;}",
            "j{transition-duration:1.25s;}",
            "k{transition-duration:1e2ms;}",
            "l{transition-duration:500ms;}",
            "m{transition-duration:1S;}",
            "n{transition-duration:500MS;}",
            "o{transition-duration:500Ms;}",
            "p{transition-duration:500mS;}",
        ),
    );

    let expected = vec![qualified(&[DirectTimeLiteral]); 16];
    assert_expected(&result, &expected);
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn ordered_lists_preserve_authored_order_and_duplicates() {
    use CssTransitionDurationValue::DirectTimeLiteral;

    let result = qualify(
        97601,
        concat!(
            "a{transition-duration:0s,500ms,2s;}",
            "b{transition-duration:1s,1s,1s;}",
            "c{transition-duration:1S,500MS;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectTimeLiteral, DirectTimeLiteral, DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral, DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral]),
        ],
    );
}

#[test]
fn retained_comma_tokens_and_trivia_define_list_separators() {
    use CssTransitionDurationValue::DirectTimeLiteral;

    let result = qualify(
        97602,
        concat!(
            "a{transition-duration:1s,2s;}",
            "b{transition-duration: 1s , 2s ;}",
            "c{transition-duration:1s/*x*/,/*y*/2s;}",
            "d{transition-duration: /*a*/1s/*b*/,/*c*/2ms/*d*/;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectTimeLiteral, DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral]),
        ],
    );
}

#[test]
fn direct_negative_non_zero_time_literals_are_invalid() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97603,
        concat!(
            "a{transition-duration:-1s;}",
            "b{transition-duration:-.5s;}",
            "c{transition-duration:-0.01ms;}",
            "d{transition-duration:-1e-100s;}",
            "e{transition-duration:1s,-1s;}",
            "f{transition-duration:-1s,1s;}",
        ),
    );

    let expected = vec![Invalid; 6];
    assert_expected(&result, &expected);
}

#[test]
fn unitless_numbers_including_zero_are_invalid() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97604,
        concat!(
            "a{transition-duration:0;}",
            "b{transition-duration:-0;}",
            "c{transition-duration:+0;}",
            "d{transition-duration:.0;}",
            "e{transition-duration:1;}",
            "f{transition-duration:-1;}",
            "g{transition-duration:.5;}",
        ),
    );

    let expected = vec![Invalid; 7];
    assert_expected(&result, &expected);
}

#[test]
fn wrong_dimension_units_are_invalid_including_mixed_list_placement() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97605,
        concat!(
            "a{transition-duration:1px;}",
            "b{transition-duration:1em;}",
            "c{transition-duration:1deg;}",
            "d{transition-duration:1Hz;}",
            "e{transition-duration:1dpi;}",
            "f{transition-duration:1fr;}",
            "g{transition-duration:1s, 1px;}",
            "h{transition-duration:1px, 1s;}",
        ),
    );

    let expected = vec![Invalid; 8];
    assert_expected(&result, &expected);
}

#[test]
fn wrong_token_classes_are_invalid() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97606,
        concat!(
            "a{transition-duration:infinite;}",
            "b{transition-duration:auto;}",
            r#"c{transition-duration:"1s";}"#,
            "d{transition-duration:50%;}",
            "e{transition-duration:1s 2s;}",
        ),
    );

    let expected = vec![Invalid; 5];
    assert_expected(&result, &expected);
}

#[test]
fn empty_items_and_missing_or_extra_separators_are_invalid() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97607,
        concat!(
            "a{transition-duration:;}",
            "b{transition-duration:,1s;}",
            "c{transition-duration:1s,;}",
            "d{transition-duration:1s,,2s;}",
            "e{transition-duration:1s 2s;}",
        ),
    );

    let expected = vec![Invalid; 5];
    assert_expected(&result, &expected);
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssTransitionDurationUnsupportedReason::CssWideKeyword;

    let whole = qualify(
        97608,
        concat!(
            "a{transition-duration:inherit;}",
            "b{transition-duration:initial;}",
            "c{transition-duration:unset;}",
            "d{transition-duration:revert;}",
            "e{transition-duration:revert-layer;}",
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
        97609,
        concat!(
            "a{transition-duration:initial,1s;}",
            "b{transition-duration:1s,initial;}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&mixed, &expected);
}

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    use CssTransitionDurationUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        97610,
        concat!(
            "a{transition-duration:var(--x);}",
            "b{transition-duration:1s,var(--x);}",
            "c{transition-duration:var(--x),1s;}",
            "d{transition-duration:var(--x,1s,2s);}",
        ),
    );

    assert_expected(
        &result,
        &[
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
        ],
    );
}

#[test]
fn function_backed_time_items_are_unsupported_absent_decisive_invalid_evidence() {
    use CssTransitionDurationUnsupportedReason::FunctionValue;

    let result = qualify(
        97611,
        concat!(
            "a{transition-duration:calc(2s);}",
            "b{transition-duration:calc(-1s);}",
            "c{transition-duration:calc(2 * 3s);}",
            "d{transition-duration:1s,calc(2s);}",
            "e{transition-duration:calc(2s),1s;}",
            "f{transition-duration:min(1s,2s),3s;}",
            "g{transition-duration:foo();}",
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
            unsupported(FunctionValue),
        ],
    );
}

#[test]
fn whole_value_function_boundary_does_not_legalize_function_items() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssTransitionDurationUnsupportedReason::WholeValueFunction;

    let whole = qualify(97612, "a{transition-duration:first-valid(1s,2s);}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let items = qualify(
        97613,
        concat!(
            "a{transition-duration:1s,first-valid(2s);}",
            "b{transition-duration:first-valid(1s),2s;}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&items, &expected);
}

#[test]
fn decisive_invalid_evidence_outranks_residual_function_unsupported() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97614,
        concat!(
            "a{transition-duration:-1s,calc(2s);}",
            "b{transition-duration:calc(2s),-1s;}",
            "c{transition-duration:1px,calc(2s);}",
            "d{transition-duration:calc(2s),1px;}",
            "e{transition-duration:1s,,calc(2s);}",
        ),
    );

    let expected = vec![Invalid; 5];
    assert_expected(&result, &expected);
}

#[test]
fn residual_function_unsupported_survives_alongside_qualified_direct_items() {
    use CssTransitionDurationUnsupportedReason::FunctionValue;

    let result = qualify(97615, "a{transition-duration:1s,calc(-1s),500ms;}");
    assert_expected(&result, &[unsupported(FunctionValue)]);
}

#[test]
fn nested_function_commas_do_not_split_the_outer_list() {
    use CssTransitionDurationUnsupportedReason::FunctionValue;

    let result = qualify(97616, "a{transition-duration:min(1s, 2s), 3s;}");
    assert_expected(&result, &[unsupported(FunctionValue)]);

    let calc_nested = qualify(97617, "a{transition-duration:calc(1s + max(2s, 3s)), 4ms;}");
    assert_expected(&calc_nested, &[unsupported(FunctionValue)]);
}

#[test]
fn important_priority_is_outside_the_semantic_list_window() {
    use CssTransitionDurationValue::DirectTimeLiteral;

    let result = qualify(97618, "a{transition-duration:0s,1s !important;}");
    assert_expected(
        &result,
        &[qualified(&[DirectTimeLiteral, DirectTimeLiteral])],
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn duplicate_occurrences_and_existing_leaf_dispatch_remain_separate() {
    use CssTransitionDurationValue::DirectTimeLiteral;

    let result = qualify(
        97619,
        concat!(
            "a{transition-duration:1s;transition-duration:500ms,2s,100ms;}",
            "b{animation-delay:1s;}",
            "c{animation-iteration-count:0,infinite;}",
            "d{animation-play-state:running,paused;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral, DirectTimeLiteral]),
        ],
    );
    assert_eq!(result.transition_duration_observations().len(), 2);
    assert_eq!(
        result.transition_duration_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.transition_duration_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(result.animation_delay_observations().len(), 1);
    assert_eq!(result.animation_iteration_count_observations().len(), 1);
    assert_eq!(result.animation_play_state_observations().len(), 1);
}

#[test]
fn transition_duration_negative_time_diverges_from_accepted_animation_delay_regression() {
    use crate::css::value_qualification::{
        CssAnimationDelayQualificationOutcome, CssAnimationDelayValue,
    };
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(97620, "a{animation-delay:-1s;transition-duration:-1s;}");

    assert_eq!(
        result.animation_delay_observations()[0].outcome().clone(),
        CssAnimationDelayQualificationOutcome::Qualified(vec![
            CssAnimationDelayValue::DirectTimeLiteral
        ])
    );
    assert_expected(&result, &[Invalid]);
}

#[test]
fn nonordinary_contexts_do_not_enter_transition_duration_dispatch() {
    for (source_id, css) in [
        (97621, "@font-face{transition-duration:1s;}"),
        (97622, "@page{transition-duration:1s;}"),
        (97623, "@keyframes k{from{transition-duration:1s;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.transition_duration_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    use CssTransitionDurationQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        97624,
        concat!(
            "a{transition-duration:calc(1s,2s) 3s;}",
            "b{transition-duration:(1s,2s),3s;}",
            "c{transition-duration:1s),2s;}",
        ),
    );
    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssTransitionDurationValue::DirectTimeLiteral;

    let incomplete = qualify_with_limits(
        97625,
        "a{transition-duration:1s;}b{transition-duration:0s,1s;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified(&[DirectTimeLiteral])]);

    let css = concat!(
        "a{transition-duration:0s,1s;}",
        "b{transition-duration:0s,,1s;}",
        "c{transition-duration:var(--x);}",
        "d{transition-duration:-1s;}",
    );
    let first = qualify(97626, css);
    let repeated = qualify(97626, css);
    let another_source = qualify(97627, css);

    assert_eq!(
        first.transition_duration_observations(),
        repeated.transition_duration_observations()
    );
    assert_eq!(
        first.transition_duration_observations(),
        another_source.transition_duration_observations()
    );
}
