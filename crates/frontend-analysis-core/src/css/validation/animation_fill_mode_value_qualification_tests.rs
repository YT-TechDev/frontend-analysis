use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssAnimationFillModeQualificationOutcome, CssAnimationFillModeUnsupportedReason,
    CssAnimationFillModeValue, CssValueQualificationRunResult, run,
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

fn qualified(values: &[CssAnimationFillModeValue]) -> CssAnimationFillModeQualificationOutcome {
    CssAnimationFillModeQualificationOutcome::Qualified(values.to_vec())
}

fn unsupported(
    reason: CssAnimationFillModeUnsupportedReason,
) -> CssAnimationFillModeQualificationOutcome {
    CssAnimationFillModeQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssAnimationFillModeQualificationOutcome],
) {
    let actual: Vec<_> = result
        .animation_fill_mode_observations()
        .iter()
        .map(|observation| observation.outcome().clone())
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn direct_single_and_repeated_list_items_preserve_authored_order() {
    use CssAnimationFillModeValue::{Backwards, Both, Forwards, None};

    let result = qualify(
        6410,
        concat!(
            "a{animation-fill-mode:none;}",
            "b{animation-fill-mode:forwards;}",
            "c{animation-fill-mode:backwards;}",
            "d{animation-fill-mode:both;}",
            "e{animation-fill-mode:both,backwards;}",
            "f{animation-fill-mode:none,forwards,backwards,both;}",
            "g{animation-fill-mode:both,none,both;}",
            "h{animation-fill-mode:backwards,forwards,backwards;}",
            "i{animation-fill-mode:NONE,FoRwArDs,BACKWARDS,BoTh;}",
            "j{animation-fill-mode:n\\6f ne,f\\6f rwards;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[None]),
            qualified(&[Forwards]),
            qualified(&[Backwards]),
            qualified(&[Both]),
            qualified(&[Both, Backwards]),
            qualified(&[None, Forwards, Backwards, Both]),
            qualified(&[Both, None, Both]),
            qualified(&[Backwards, Forwards, Backwards]),
            qualified(&[None, Forwards, Backwards, Both]),
            qualified(&[None, Forwards]),
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn retained_comma_tokens_and_trivia_define_list_separators() {
    use CssAnimationFillModeValue::{Backwards, Both, Forwards, None};

    let result = qualify(
        6411,
        concat!(
            "a{animation-fill-mode:none/**/,/**/both;}",
            "b{animation-fill-mode: none , forwards , both ;}",
            "c{animation-fill-mode:backwards/*,*/ , /*,*/forwards;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(&[None, Both]),
            qualified(&[None, Forwards, Both]),
            qualified(&[Backwards, Forwards]),
        ],
    );
}

#[test]
fn empty_items_missing_separators_and_wrong_direct_items_are_invalid() {
    use CssAnimationFillModeQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        6412,
        concat!(
            "a{animation-fill-mode:;}",
            "b{animation-fill-mode:,none;}",
            "c{animation-fill-mode:none,;}",
            "d{animation-fill-mode:none,,both;}",
            "e{animation-fill-mode:,,both;}",
            "f{animation-fill-mode:both,,;}",
            "g{animation-fill-mode:forwards backwards;}",
            "h{animation-fill-mode:none both;}",
            "i{animation-fill-mode:both none forwards;}",
            "j{animation-fill-mode:none / both;}",
        ),
    );

    let expected = vec![Invalid; 10];
    assert_expected(&result, &expected);
}

#[test]
fn auto_and_unknown_or_wrong_token_class_items_are_invalid() {
    use CssAnimationFillModeQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        6413,
        concat!(
            "a{animation-fill-mode:auto;}",
            "b{animation-fill-mode:none,auto;}",
            "c{animation-fill-mode:auto,both;}",
            "d{animation-fill-mode:normal;}",
            "e{animation-fill-mode:running;}",
            "f{animation-fill-mode:paused;}",
            "g{animation-fill-mode:fill;}",
            "h{animation-fill-mode:forward;}",
            "i{animation-fill-mode:backward;}",
            "j{animation-fill-mode:1;}",
            "k{animation-fill-mode:1px;}",
            "l{animation-fill-mode:50%;}",
            "m{animation-fill-mode:\"none\";}",
            "n{animation-fill-mode:#fff;}",
        ),
    );

    let expected = vec![Invalid; 14];
    assert_expected(&result, &expected);
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    use CssAnimationFillModeQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationFillModeUnsupportedReason::CssWideKeyword;
    use CssAnimationFillModeValue::None;

    let whole = qualify(
        6414,
        concat!(
            "a{animation-fill-mode:inherit;}",
            "b{animation-fill-mode:initial;}",
            "c{animation-fill-mode:unset;}",
            "d{animation-fill-mode:revert;}",
            "e{animation-fill-mode:revert-layer;}",
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

    // `initial` never collapses into the property's initial value identity.
    let initial_only = qualify(
        6415,
        "a{animation-fill-mode:initial;}b{animation-fill-mode:none;}",
    );
    assert_ne!(
        initial_only.animation_fill_mode_observations()[0].outcome(),
        &qualified(&[None])
    );
    assert_expected(
        &initial_only,
        &[unsupported(CssWideKeyword), qualified(&[None])],
    );

    let mixed = qualify(
        6416,
        concat!(
            "a{animation-fill-mode:both,initial;}",
            "b{animation-fill-mode:initial,both;}",
            "c{animation-fill-mode:none,inherit;}",
            "d{animation-fill-mode:revert,forwards;}",
        ),
    );
    let expected = vec![Invalid; 4];
    assert_expected(&mixed, &expected);
}

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    use CssAnimationFillModeUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        6417,
        concat!(
            "a{animation-fill-mode:var(--x);}",
            "b{animation-fill-mode:none,var(--x);}",
            "c{animation-fill-mode:var(--x),both;}",
            "d{animation-fill-mode:none,var(--x,both);}",
            "e{animation-fill-mode:foo(var(--x),both);}",
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
    use CssAnimationFillModeQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationFillModeUnsupportedReason::WholeValueFunction;

    let whole = qualify(6418, "a{animation-fill-mode:first-valid(none,both);}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let items = qualify(
        6419,
        concat!(
            "a{animation-fill-mode:none,first-valid(both);}",
            "b{animation-fill-mode:first-valid(none),both;}",
            "c{animation-fill-mode:foo(none,both);}",
            "d{animation-fill-mode:calc(1);}",
            "e{animation-fill-mode:none,foo();}",
            "f{animation-fill-mode:both,calc(1);}",
            "g{animation-fill-mode:foo(a,b),none;}",
        ),
    );
    let expected = vec![Invalid; 7];
    assert_expected(&items, &expected);
}

#[test]
fn important_priority_is_outside_the_semantic_list_window() {
    use CssAnimationFillModeValue::{Both, None};

    let result = qualify(6420, "a{animation-fill-mode:none,both !important;}");
    assert_expected(&result, &[qualified(&[None, Both])]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn duplicate_occurrences_and_existing_leaf_dispatch_remain_separate() {
    use CssAnimationFillModeValue::{Both, Forwards, None};

    let result = qualify(
        6421,
        concat!(
            "a{animation-fill-mode:none;animation-fill-mode:both,forwards;}",
            "b{animation-play-state:running;}",
            "c{animation-delay:1s;}",
            "d{animation-iteration-count:2;}",
            "e{transition-duration:1s;}",
            "f{caret-shape:auto;}",
            "g{unrelated-property:1px;}",
        ),
    );

    assert_expected(&result, &[qualified(&[None]), qualified(&[Both, Forwards])]);
    assert_eq!(result.animation_fill_mode_observations().len(), 2);
    assert_eq!(
        result.animation_fill_mode_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.animation_fill_mode_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(result.animation_play_state_observations().len(), 1);
    assert_eq!(result.animation_delay_observations().len(), 1);
    assert_eq!(result.animation_iteration_count_observations().len(), 1);
    assert_eq!(result.caret_shape_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_animation_fill_mode_dispatch() {
    for (source_id, css) in [
        (6422, "@font-face{animation-fill-mode:none;}"),
        (6423, "@page{animation-fill-mode:none;}"),
        (6424, "@keyframes k{from{animation-fill-mode:none;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.animation_fill_mode_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    use CssAnimationFillModeQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        6425,
        concat!(
            "a{animation-fill-mode:both),forwards;}",
            "b{animation-fill-mode:(none,both);}",
            "c{animation-fill-mode:foo(none,both),both;}",
        ),
    );
    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssAnimationFillModeValue::None;

    let incomplete = qualify_with_limits(
        6426,
        "a{animation-fill-mode:none;}b{animation-fill-mode:both,forwards;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified(&[None])]);

    let css = concat!(
        "a{animation-fill-mode:none,both;}",
        "b{animation-fill-mode:none,,both;}",
        "c{animation-fill-mode:var(--x);}",
    );
    let first = qualify(6427, css);
    let repeated = qualify(6427, css);
    let another_source = qualify(6428, css);

    assert_eq!(
        first.animation_fill_mode_observations(),
        repeated.animation_fill_mode_observations()
    );
    assert_eq!(
        first.animation_fill_mode_observations(),
        another_source.animation_fill_mode_observations()
    );
}
