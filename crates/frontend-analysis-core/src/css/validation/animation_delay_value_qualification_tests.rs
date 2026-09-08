use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssAnimationDelayQualificationOutcome, CssAnimationDelayUnsupportedReason,
    CssAnimationDelayValue, CssValueQualificationRunResult, run,
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

fn qualified(values: &[CssAnimationDelayValue]) -> CssAnimationDelayQualificationOutcome {
    CssAnimationDelayQualificationOutcome::Qualified(values.to_vec())
}

fn unsupported(
    reason: CssAnimationDelayUnsupportedReason,
) -> CssAnimationDelayQualificationOutcome {
    CssAnimationDelayQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssAnimationDelayQualificationOutcome],
) {
    let actual: Vec<_> = result
        .animation_delay_observations()
        .iter()
        .map(|observation| observation.outcome().clone())
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn direct_time_literals_qualify_regardless_of_numeric_spelling_or_unit_case() {
    use CssAnimationDelayValue::DirectTimeLiteral;

    let result = qualify(
        90100,
        concat!(
            "a{animation-delay:0s;}",
            "b{animation-delay:-0s;}",
            "c{animation-delay:+0s;}",
            "d{animation-delay:1s;}",
            "e{animation-delay:-1s;}",
            "f{animation-delay:.5s;}",
            "g{animation-delay:-.5s;}",
            "h{animation-delay:1.25s;}",
            "i{animation-delay:-1.25e2ms;}",
            "j{animation-delay:500ms;}",
            "k{animation-delay:1S;}",
            "l{animation-delay:500MS;}",
            "m{animation-delay:500Ms;}",
            "n{animation-delay:500mS;}",
        ),
    );

    let expected = vec![qualified(&[DirectTimeLiteral]); 14];
    assert_expected(&result, &expected);
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn ordered_lists_preserve_authored_order_and_duplicates() {
    use CssAnimationDelayValue::DirectTimeLiteral;

    let result = qualify(
        90101,
        concat!(
            "a{animation-delay:20s, 10s;}",
            "b{animation-delay:-5ms, 0s, 2.5s;}",
            "c{animation-delay:1S, 500MS;}",
            "d{animation-delay:1s, 1s, 1s;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectTimeLiteral, DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral, DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral, DirectTimeLiteral]),
        ],
    );
}

#[test]
fn retained_comma_tokens_and_trivia_define_list_separators() {
    use CssAnimationDelayValue::DirectTimeLiteral;

    let result = qualify(
        90102,
        concat!(
            "a{animation-delay:1s,2s;}",
            "b{animation-delay: 1s , 2s ;}",
            "c{animation-delay:1s/*x*/,/*y*/2s;}",
            "d{animation-delay: /*a*/1s/*b*/,/*c*/2ms/*d*/;}",
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
fn unitless_numbers_including_zero_are_invalid() {
    use CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        90103,
        concat!(
            "a{animation-delay:0;}",
            "b{animation-delay:-0;}",
            "c{animation-delay:+0;}",
            "d{animation-delay:.0;}",
            "e{animation-delay:1;}",
            "f{animation-delay:-1;}",
            "g{animation-delay:.5;}",
        ),
    );

    let expected = vec![Invalid; 7];
    assert_expected(&result, &expected);
}

#[test]
fn wrong_dimension_units_are_invalid_including_mixed_list_placement() {
    use CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        90104,
        concat!(
            "a{animation-delay:1px;}",
            "b{animation-delay:1em;}",
            "c{animation-delay:1deg;}",
            "d{animation-delay:1Hz;}",
            "e{animation-delay:1dpi;}",
            "f{animation-delay:1fr;}",
            "g{animation-delay:1s, 1px;}",
            "h{animation-delay:1px, 1s;}",
        ),
    );

    let expected = vec![Invalid; 8];
    assert_expected(&result, &expected);
}

#[test]
fn wrong_token_classes_are_invalid() {
    use CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        90105,
        concat!(
            "a{animation-delay:infinite;}",
            "b{animation-delay:auto;}",
            r#"c{animation-delay:"1s";}"#,
            "d{animation-delay:50%;}",
            "e{animation-delay:1s 2s;}",
        ),
    );

    let expected = vec![Invalid; 5];
    assert_expected(&result, &expected);
}

#[test]
fn empty_items_and_missing_or_extra_separators_are_invalid() {
    use CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        90106,
        concat!(
            "a{animation-delay:;}",
            "b{animation-delay:,1s;}",
            "c{animation-delay:1s,;}",
            "d{animation-delay:1s,,2s;}",
            "e{animation-delay:1s 2s;}",
        ),
    );

    let expected = vec![Invalid; 5];
    assert_expected(&result, &expected);
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    use CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationDelayUnsupportedReason::CssWideKeyword;

    let whole = qualify(
        90107,
        concat!(
            "a{animation-delay:inherit;}",
            "b{animation-delay:initial;}",
            "c{animation-delay:unset;}",
            "d{animation-delay:revert;}",
            "e{animation-delay:revert-layer;}",
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
        90108,
        concat!(
            "a{animation-delay:initial,1s;}",
            "b{animation-delay:1s,initial;}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&mixed, &expected);
}

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    use CssAnimationDelayUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        90109,
        concat!(
            "a{animation-delay:var(--x);}",
            "b{animation-delay:1s,var(--x);}",
            "c{animation-delay:var(--x),1s;}",
            "d{animation-delay:var(--x,1s,2s);}",
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
    use CssAnimationDelayUnsupportedReason::FunctionValue;

    let result = qualify(
        90110,
        concat!(
            "a{animation-delay:calc(2s);}",
            "b{animation-delay:calc(2 * 3s);}",
            "c{animation-delay:1s,calc(2s);}",
            "d{animation-delay:calc(2s),1s;}",
            "e{animation-delay:min(1s,2s),3s;}",
            "f{animation-delay:foo();}",
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
    use CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationDelayUnsupportedReason::WholeValueFunction;

    let whole = qualify(90111, "a{animation-delay:first-valid(1s,2s);}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let items = qualify(
        90112,
        concat!(
            "a{animation-delay:1s,first-valid(2s);}",
            "b{animation-delay:first-valid(1s),2s;}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&items, &expected);
}

#[test]
fn decisive_invalid_evidence_outranks_residual_function_unsupported() {
    use CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        90113,
        concat!(
            "a{animation-delay:1px,calc(2s);}",
            "b{animation-delay:calc(2s),1px;}",
            "c{animation-delay:1s,,calc(2s);}",
        ),
    );

    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn residual_function_unsupported_survives_alongside_qualified_direct_items() {
    use CssAnimationDelayUnsupportedReason::FunctionValue;

    let result = qualify(90114, "a{animation-delay:1s,calc(2s),-5ms;}");
    assert_expected(&result, &[unsupported(FunctionValue)]);
}

#[test]
fn nested_function_commas_do_not_split_the_outer_list() {
    use CssAnimationDelayUnsupportedReason::FunctionValue;

    let result = qualify(90115, "a{animation-delay:min(1s, 2s), 3s;}");
    assert_expected(&result, &[unsupported(FunctionValue)]);

    let calc_nested = qualify(90116, "a{animation-delay:calc(1s + max(2s, 3s)), 4ms;}");
    assert_expected(&calc_nested, &[unsupported(FunctionValue)]);
}

#[test]
fn important_priority_is_outside_the_semantic_list_window() {
    use CssAnimationDelayValue::DirectTimeLiteral;

    let result = qualify(90117, "a{animation-delay:0s,1s !important;}");
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
    use CssAnimationDelayValue::DirectTimeLiteral;

    let result = qualify(
        90118,
        concat!(
            "a{animation-delay:1s;animation-delay:-5ms,2s,100ms;}",
            "b{animation-iteration-count:0,infinite;}",
            "c{animation-play-state:running,paused;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[DirectTimeLiteral]),
            qualified(&[DirectTimeLiteral, DirectTimeLiteral, DirectTimeLiteral]),
        ],
    );
    assert_eq!(result.animation_delay_observations().len(), 2);
    assert_eq!(
        result.animation_delay_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.animation_delay_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(result.animation_iteration_count_observations().len(), 1);
    assert_eq!(result.animation_play_state_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_animation_delay_dispatch() {
    for (source_id, css) in [
        (90119, "@font-face{animation-delay:1s;}"),
        (90120, "@page{animation-delay:1s;}"),
        (90121, "@keyframes k{from{animation-delay:1s;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.animation_delay_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    use CssAnimationDelayQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        90122,
        concat!(
            "a{animation-delay:calc(1s,2s) 3s;}",
            "b{animation-delay:(1s,2s),3s;}",
            "c{animation-delay:1s),2s;}",
        ),
    );
    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssAnimationDelayValue::DirectTimeLiteral;

    let incomplete = qualify_with_limits(
        90123,
        "a{animation-delay:1s;}b{animation-delay:0s,1s;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified(&[DirectTimeLiteral])]);

    let css = concat!(
        "a{animation-delay:0s,1s;}",
        "b{animation-delay:0s,,1s;}",
        "c{animation-delay:var(--x);}",
    );
    let first = qualify(90124, css);
    let repeated = qualify(90124, css);
    let another_source = qualify(90125, css);

    assert_eq!(
        first.animation_delay_observations(),
        repeated.animation_delay_observations()
    );
    assert_eq!(
        first.animation_delay_observations(),
        another_source.animation_delay_observations()
    );
}
