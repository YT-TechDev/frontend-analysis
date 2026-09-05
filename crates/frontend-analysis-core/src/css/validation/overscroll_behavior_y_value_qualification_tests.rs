use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssOverscrollBehaviorYQualificationOutcome, CssOverscrollBehaviorYUnsupportedReason,
    CssOverscrollBehaviorYValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Contain,
    None,
    Auto,
    Chain,
    Invalid,
    UnsupportedCssWide,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssOverscrollBehaviorYQualificationOutcome {
    match expected {
        ExpectedOutcome::Contain => CssOverscrollBehaviorYQualificationOutcome::Qualified(
            CssOverscrollBehaviorYValue::Contain,
        ),
        ExpectedOutcome::None => {
            CssOverscrollBehaviorYQualificationOutcome::Qualified(CssOverscrollBehaviorYValue::None)
        }
        ExpectedOutcome::Auto => {
            CssOverscrollBehaviorYQualificationOutcome::Qualified(CssOverscrollBehaviorYValue::Auto)
        }
        ExpectedOutcome::Chain => CssOverscrollBehaviorYQualificationOutcome::Qualified(
            CssOverscrollBehaviorYValue::Chain,
        ),
        ExpectedOutcome::Invalid => {
            CssOverscrollBehaviorYQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssOverscrollBehaviorYQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorYUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssOverscrollBehaviorYQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorYUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .overscroll_behavior_y_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_pinned_wpt() {
    let result = qualify(
        3340,
        concat!(
            "a{overscroll-behavior-y:contain;}",
            "b{overscroll-behavior-y:none;}",
            "c{overscroll-behavior-y:auto;}",
            "d{OVERSCROLL-BEHAVIOR-Y:ChAiN;}",
            r"e{overscroll-behavior-y:\63 ontain;}",
            r"f{overscroll-behavior-\79 :none;}",
            "g{overscroll-behavior-y:normal;}",
            "h{overscroll-behavior-y:contain none;}",
            "i{overscroll-behavior-y:0;}",
            "j{overscroll-behavior-y:1px;}",
            "k{overscroll-behavior-y:\"auto\";}",
            "l{overscroll-behavior-y:;}",
            "m{clip-rule:evenodd;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Contain,
            ExpectedOutcome::None,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Chain,
            ExpectedOutcome::Contain,
            ExpectedOutcome::None,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn comments_and_priority_preserve_decoded_keyword_meaning_and_source_placement() {
    let result = qualify(
        3341,
        concat!(
            "a{overscroll-behavior-y:/**/contain/**/!important;}",
            "b{overscroll-behavior-y:/**/chain/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Contain, ExpectedOutcome::Chain]);
    for observation in result.overscroll_behavior_y_observations() {
        let occurrence =
            &result.upstream_parser_result().occurrences()[observation.occurrence_index()];
        assert_eq!(observation.placement(), occurrence.placement());
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3342,
        concat!(
            "a{overscroll-behavior-y:initial;}",
            "b{overscroll-behavior-y:inherit;}",
            "c{overscroll-behavior-y:unset;}",
            "d{overscroll-behavior-y:revert;}",
            "e{overscroll-behavior-y:revert-layer;}",
            "f{overscroll-behavior-y:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3343,
        concat!(
            "a{overscroll-behavior-y:var(--behavior);}",
            "b{overscroll-behavior-y:env(behavior);}",
            "c{overscroll-behavior-y:attr(data-behavior);}",
            "d{overscroll-behavior-y:--behavior();}",
            "e{overscroll-behavior-y:first-valid(contain,none);}",
            "f{overscroll-behavior-y:cycle(contain,none);}",
            "g{overscroll-behavior-y:interpolate(0%,0:contain,1:none);}",
            "h{overscroll-behavior-y:foo();}",
            "i{overscroll-behavior-y:calc(1);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn mixed_function_placement_preserves_single_keyword_boundary() {
    let result = qualify(
        3344,
        concat!(
            "a{overscroll-behavior-y:contain first-valid(none);}",
            "b{overscroll-behavior-y:first-valid(auto) chain;}",
            "c{overscroll-behavior-y:auto foo();}",
            "d{overscroll-behavior-y:foo() none;}",
            "e{overscroll-behavior-y:chain var(--behavior);}",
            "f{overscroll-behavior-y:foo(var(--behavior));}",
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
        ],
    );
}

#[test]
fn scroll_container_applicability_is_not_an_input_to_qualification() {
    let result = qualify(
        3345,
        concat!(
            "span{overscroll-behavior-y:contain;}",
            "div{overscroll-behavior-y:contain;}",
            "section{overscroll-behavior-y:contain;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Contain; 3]);
    let indices: Vec<_> = result
        .overscroll_behavior_y_observations()
        .iter()
        .map(|observation| observation.occurrence_index())
        .collect();
    assert_eq!(indices, [0, 1, 2]);
}

#[test]
fn one_run_interleaves_with_existing_accepted_leaves_without_cross_dispatch() {
    let result = qualify(
        3346,
        concat!(
            "a{direction:ltr;}",
            "b{z-index:1;}",
            "c{text-decoration-skip-ink:all;}",
            "d{overscroll-behavior-x:chain;}",
            "e{overscroll-behavior-y:chain;}",
            "f{clip-rule:evenodd;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.text_decoration_skip_ink_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);
    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);
    assert_eq!(
        result.overscroll_behavior_y_observations()[0].occurrence_index(),
        4
    );
    assert_expected(&result, &[ExpectedOutcome::Chain]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        3347,
        "a{overscroll-behavior-y:auto;}b{overscroll-behavior-y:auto;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_ne!(
        result.overscroll_behavior_y_observations()[0]
            .placement()
            .context_id(),
        result.overscroll_behavior_y_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3350, "@font-face{overscroll-behavior-y:none;}"),
        (3351, "@page{overscroll-behavior-y:none;}"),
        (3352, "@page{@top-left{overscroll-behavior-y:none;}}"),
        (3353, "@keyframes k{from{overscroll-behavior-y:none;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.overscroll_behavior_y_observations().is_empty(),
            concat!(
                "nonordinary declaration context produced an ",
                "overscroll-behavior-y observation for {:?}"
            ),
            css
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3360,
        "a{overscroll-behavior-y:auto;overscroll-behavior-y:none;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Auto]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{overscroll-behavior-y:contain;}",
        "b{overscroll-behavior-y:inherit;}",
        "c{overscroll-behavior-y:chain;}",
        "d{overscroll-behavior-y:var(--behavior);}",
        "e{clip-rule:none;}",
    );
    let first = qualify(3370, css);
    let repeated = qualify(3370, css);
    let another_source = qualify(3371, css);

    assert_eq!(
        first.overscroll_behavior_y_observations(),
        repeated.overscroll_behavior_y_observations()
    );
    assert_eq!(
        first.overscroll_behavior_y_observations(),
        another_source.overscroll_behavior_y_observations()
    );
}
