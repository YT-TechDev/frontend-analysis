use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssOverscrollBehaviorXQualificationOutcome, CssOverscrollBehaviorXUnsupportedReason,
    CssOverscrollBehaviorXValue, CssValueQualificationRunResult, run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssOverscrollBehaviorXQualificationOutcome {
    match expected {
        ExpectedOutcome::Contain => CssOverscrollBehaviorXQualificationOutcome::Qualified(
            CssOverscrollBehaviorXValue::Contain,
        ),
        ExpectedOutcome::None => CssOverscrollBehaviorXQualificationOutcome::Qualified(
            CssOverscrollBehaviorXValue::None,
        ),
        ExpectedOutcome::Auto => CssOverscrollBehaviorXQualificationOutcome::Qualified(
            CssOverscrollBehaviorXValue::Auto,
        ),
        ExpectedOutcome::Chain => CssOverscrollBehaviorXQualificationOutcome::Qualified(
            CssOverscrollBehaviorXValue::Chain,
        ),
        ExpectedOutcome::Invalid => {
            CssOverscrollBehaviorXQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssOverscrollBehaviorXQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorXUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssOverscrollBehaviorXQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorXUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .overscroll_behavior_x_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_pinned_wpt() {
    let result = qualify(
        3300,
        concat!(
            "a{overscroll-behavior-x:contain;}",
            "b{overscroll-behavior-x:none;}",
            "c{overscroll-behavior-x:auto;}",
            "d{OVERSCROLL-BEHAVIOR-X:ChAiN;}",
            r"e{overscroll-behavior-x:\63 ontain;}",
            r"f{overscroll-behavior-\78 :none;}",
            "g{overscroll-behavior-x:normal;}",
            "h{overscroll-behavior-x:contain none;}",
            "i{overscroll-behavior-x:0;}",
            "j{overscroll-behavior-x:1px;}",
            "k{overscroll-behavior-x:\"auto\";}",
            "l{overscroll-behavior-x:;}",
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
        3301,
        concat!(
            "a{overscroll-behavior-x:/**/contain/**/!important;}",
            "b{overscroll-behavior-x:/**/chain/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Contain, ExpectedOutcome::Chain],
    );
    for observation in result.overscroll_behavior_x_observations() {
        let occurrence =
            &result.upstream_parser_result().occurrences()[observation.occurrence_index()];
        assert_eq!(observation.placement(), occurrence.placement());
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3302,
        concat!(
            "a{overscroll-behavior-x:initial;}",
            "b{overscroll-behavior-x:inherit;}",
            "c{overscroll-behavior-x:unset;}",
            "d{overscroll-behavior-x:revert;}",
            "e{overscroll-behavior-x:revert-layer;}",
            "f{overscroll-behavior-x:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3303,
        concat!(
            "a{overscroll-behavior-x:var(--behavior);}",
            "b{overscroll-behavior-x:env(behavior);}",
            "c{overscroll-behavior-x:attr(data-behavior);}",
            "d{overscroll-behavior-x:--behavior();}",
            "e{overscroll-behavior-x:first-valid(contain,none);}",
            "f{overscroll-behavior-x:cycle(contain,none);}",
            "g{overscroll-behavior-x:interpolate(0%,0:contain,1:none);}",
            "h{overscroll-behavior-x:foo();}",
            "i{overscroll-behavior-x:calc(1);}",
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
        3304,
        concat!(
            "a{overscroll-behavior-x:contain first-valid(none);}",
            "b{overscroll-behavior-x:first-valid(auto) chain;}",
            "c{overscroll-behavior-x:auto foo();}",
            "d{overscroll-behavior-x:foo() none;}",
            "e{overscroll-behavior-x:chain var(--behavior);}",
            "f{overscroll-behavior-x:foo(var(--behavior));}",
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
        3305,
        concat!(
            "span{overscroll-behavior-x:contain;}",
            "div{overscroll-behavior-x:contain;}",
            "section{overscroll-behavior-x:contain;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Contain; 3]);
    let indices: Vec<_> = result
        .overscroll_behavior_x_observations()
        .iter()
        .map(|observation| observation.occurrence_index())
        .collect();
    assert_eq!(indices, [0, 1, 2]);
}

#[test]
fn one_run_interleaves_with_existing_accepted_leaves_without_cross_dispatch() {
    let result = qualify(
        3306,
        concat!(
            "a{direction:ltr;}",
            "b{z-index:1;}",
            "c{text-decoration-skip-ink:all;}",
            "d{overscroll-behavior-x:chain;}",
            "e{clip-rule:evenodd;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.text_decoration_skip_ink_observations().len(), 1);
    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);
    assert_eq!(
        result.overscroll_behavior_x_observations()[0].occurrence_index(),
        3
    );
    assert_expected(&result, &[ExpectedOutcome::Chain]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        3307,
        "a{overscroll-behavior-x:auto;}b{overscroll-behavior-x:auto;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_ne!(
        result.overscroll_behavior_x_observations()[0]
            .placement()
            .context_id(),
        result.overscroll_behavior_x_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3310, "@font-face{overscroll-behavior-x:none;}"),
        (3311, "@page{overscroll-behavior-x:none;}"),
        (3312, "@page{@top-left{overscroll-behavior-x:none;}}"),
        (3313, "@keyframes k{from{overscroll-behavior-x:none;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.overscroll_behavior_x_observations().is_empty(),
            concat!(
                "nonordinary declaration context produced an ",
                "overscroll-behavior-x observation for {css:?}"
            )
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3320,
        "a{overscroll-behavior-x:auto;overscroll-behavior-x:none;}",
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
        "a{overscroll-behavior-x:contain;}",
        "b{overscroll-behavior-x:inherit;}",
        "c{overscroll-behavior-x:chain;}",
        "d{overscroll-behavior-x:var(--behavior);}",
        "e{clip-rule:none;}",
    );
    let first = qualify(3330, css);
    let repeated = qualify(3330, css);
    let another_source = qualify(3331, css);

    assert_eq!(
        first.overscroll_behavior_x_observations(),
        repeated.overscroll_behavior_x_observations()
    );
    assert_eq!(
        first.overscroll_behavior_x_observations(),
        another_source.overscroll_behavior_x_observations()
    );
}
