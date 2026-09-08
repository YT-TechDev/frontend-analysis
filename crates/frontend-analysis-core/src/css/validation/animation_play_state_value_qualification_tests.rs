use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssAnimationPlayStateQualificationOutcome, CssAnimationPlayStateUnsupportedReason,
    CssAnimationPlayStateValue, CssValueQualificationRunResult, run,
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

fn qualified(values: &[CssAnimationPlayStateValue]) -> CssAnimationPlayStateQualificationOutcome {
    CssAnimationPlayStateQualificationOutcome::Qualified(values.to_vec())
}

fn unsupported(
    reason: CssAnimationPlayStateUnsupportedReason,
) -> CssAnimationPlayStateQualificationOutcome {
    CssAnimationPlayStateQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssAnimationPlayStateQualificationOutcome],
) {
    let actual: Vec<_> = result
        .animation_play_state_observations()
        .iter()
        .map(|observation| observation.outcome().clone())
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn direct_single_and_repeated_list_items_preserve_authored_order() {
    use CssAnimationPlayStateValue::{Paused, Running};

    let result = qualify(
        5680,
        concat!(
            "a{animation-play-state:running;}",
            "b{animation-play-state:paused;}",
            "c{animation-play-state:running, paused;}",
            "d{animation-play-state:paused,running,paused;}",
            "e{animation-play-state:RUNNING,PaUsEd;}",
            "f{animation-play-state:r\\75 nning,p\\61 used;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[Running]),
            qualified(&[Paused]),
            qualified(&[Running, Paused]),
            qualified(&[Paused, Running, Paused]),
            qualified(&[Running, Paused]),
            qualified(&[Running, Paused]),
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn retained_comma_tokens_and_trivia_define_list_separators() {
    use CssAnimationPlayStateValue::{Paused, Running};

    let result = qualify(
        5681,
        concat!(
            "a{animation-play-state:running/**/,/**/paused;}",
            "b{animation-play-state: running , paused , running ;}",
            "c{animation-play-state:running/*,*/ , /*,*/paused;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[Running, Paused]),
            qualified(&[Running, Paused, Running]),
            qualified(&[Running, Paused]),
        ],
    );
}

#[test]
fn empty_items_missing_separators_and_wrong_direct_items_are_invalid() {
    use CssAnimationPlayStateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        5682,
        concat!(
            "a{animation-play-state:;}",
            "b{animation-play-state:,running;}",
            "c{animation-play-state:running,;}",
            "d{animation-play-state:running,,paused;}",
            "e{animation-play-state:running paused;}",
            "f{animation-play-state:auto;}",
            "g{animation-play-state:running,auto;}",
            "h{animation-play-state:1,paused;}",
            "i{animation-play-state:running/paused;}",
        ),
    );

    let expected = vec![Invalid; 9];
    assert_expected(&result, &expected);
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    use CssAnimationPlayStateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationPlayStateUnsupportedReason::CssWideKeyword;

    let whole = qualify(
        5683,
        concat!(
            "a{animation-play-state:inherit;}",
            "b{animation-play-state:initial;}",
            "c{animation-play-state:unset;}",
            "d{animation-play-state:revert;}",
            "e{animation-play-state:revert-layer;}",
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
        5684,
        concat!(
            "a{animation-play-state:running,inherit;}",
            "b{animation-play-state:initial,paused;}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&mixed, &expected);
}

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    use CssAnimationPlayStateUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        5685,
        concat!(
            "a{animation-play-state:var(--x);}",
            "b{animation-play-state:running,var(--x);}",
            "c{animation-play-state:var(--x),paused;}",
            "d{animation-play-state:running,var(--x,paused);}",
            "e{animation-play-state:foo(var(--x),paused);}",
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
fn whole_value_function_boundary_does_not_legalize_function_items() {
    use CssAnimationPlayStateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationPlayStateUnsupportedReason::WholeValueFunction;

    let whole = qualify(5686, "a{animation-play-state:first-valid(running,paused);}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let items = qualify(
        5687,
        concat!(
            "a{animation-play-state:running,first-valid(paused);}",
            "b{animation-play-state:foo(running,paused);}",
            "c{animation-play-state:foo(a,b),running;}",
        ),
    );
    let expected = vec![Invalid; 3];
    assert_expected(&items, &expected);
}

#[test]
fn important_priority_is_outside_the_semantic_list_window() {
    use CssAnimationPlayStateValue::{Paused, Running};

    let result = qualify(5688, "a{animation-play-state:running,paused !important;}");
    assert_expected(&result, &[qualified(&[Running, Paused])]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn duplicate_occurrences_and_existing_leaf_dispatch_remain_separate() {
    use CssAnimationPlayStateValue::{Paused, Running};

    let result = qualify(
        5689,
        concat!(
            "a{animation-play-state:running;animation-play-state:paused,running;}",
            "b{aspect-ratio:16/9;}",
        ),
    );

    assert_expected(
        &result,
        &[qualified(&[Running]), qualified(&[Paused, Running])],
    );
    assert_eq!(result.animation_play_state_observations().len(), 2);
    assert_eq!(
        result.animation_play_state_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.animation_play_state_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(result.aspect_ratio_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_animation_play_state_dispatch() {
    for (source_id, css) in [
        (5690, "@font-face{animation-play-state:running;}"),
        (5691, "@page{animation-play-state:running;}"),
        (5692, "@keyframes k{from{animation-play-state:running;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.animation_play_state_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    use CssAnimationPlayStateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        5693,
        concat!(
            "a{animation-play-state:running),paused;}",
            "b{animation-play-state:(running,paused);}",
            "c{animation-play-state:foo(running,paused),paused;}",
        ),
    );
    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssAnimationPlayStateValue::Running;

    let incomplete = qualify_with_limits(
        5694,
        "a{animation-play-state:running;}b{animation-play-state:paused,running;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified(&[Running])]);

    let css = concat!(
        "a{animation-play-state:running,paused;}",
        "b{animation-play-state:running,,paused;}",
        "c{animation-play-state:var(--x);}",
    );
    let first = qualify(5695, css);
    let repeated = qualify(5695, css);
    let another_source = qualify(5696, css);

    assert_eq!(
        first.animation_play_state_observations(),
        repeated.animation_play_state_observations()
    );
    assert_eq!(
        first.animation_play_state_observations(),
        another_source.animation_play_state_observations()
    );
}
