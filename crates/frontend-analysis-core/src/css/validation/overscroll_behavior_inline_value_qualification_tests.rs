use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssOverscrollBehaviorInlineQualificationOutcome, CssOverscrollBehaviorInlineUnsupportedReason,
    CssOverscrollBehaviorInlineValue, CssValueQualificationRunResult, run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssOverscrollBehaviorInlineQualificationOutcome {
    match expected {
        ExpectedOutcome::Contain => CssOverscrollBehaviorInlineQualificationOutcome::Qualified(
            CssOverscrollBehaviorInlineValue::Contain,
        ),
        ExpectedOutcome::None => CssOverscrollBehaviorInlineQualificationOutcome::Qualified(
            CssOverscrollBehaviorInlineValue::None,
        ),
        ExpectedOutcome::Auto => CssOverscrollBehaviorInlineQualificationOutcome::Qualified(
            CssOverscrollBehaviorInlineValue::Auto,
        ),
        ExpectedOutcome::Chain => CssOverscrollBehaviorInlineQualificationOutcome::Qualified(
            CssOverscrollBehaviorInlineValue::Chain,
        ),
        ExpectedOutcome::Invalid => {
            CssOverscrollBehaviorInlineQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssOverscrollBehaviorInlineQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorInlineUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssOverscrollBehaviorInlineQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorInlineUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .overscroll_behavior_inline_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_pinned_wpt() {
    let result = qualify(
        3380,
        concat!(
            "a{overscroll-behavior-inline:contain;}",
            "b{overscroll-behavior-inline:none;}",
            "c{overscroll-behavior-inline:auto;}",
            "d{OVERSCROLL-BEHAVIOR-INLINE:ChAiN;}",
            r"e{overscroll-behavior-inline:\63 ontain;}",
            r"f{overscroll-behavior-\69 nline:none;}",
            "g{overscroll-behavior-inline:normal;}",
            "h{overscroll-behavior-inline:contain none;}",
            "i{overscroll-behavior-inline:0;}",
            "j{overscroll-behavior-inline:1px;}",
            "k{overscroll-behavior-inline:\"auto\";}",
            "l{overscroll-behavior-inline:;}",
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
        3381,
        concat!(
            "a{overscroll-behavior-inline:/**/contain/**/!important;}",
            "b{overscroll-behavior-inline:/**/chain/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Contain, ExpectedOutcome::Chain]);
    for observation in result.overscroll_behavior_inline_observations() {
        let occurrence =
            &result.upstream_parser_result().occurrences()[observation.occurrence_index()];
        assert_eq!(observation.placement(), occurrence.placement());
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3382,
        concat!(
            "a{overscroll-behavior-inline:initial;}",
            "b{overscroll-behavior-inline:inherit;}",
            "c{overscroll-behavior-inline:unset;}",
            "d{overscroll-behavior-inline:revert;}",
            "e{overscroll-behavior-inline:revert-layer;}",
            "f{overscroll-behavior-inline:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3383,
        concat!(
            "a{overscroll-behavior-inline:var(--behavior);}",
            "b{overscroll-behavior-inline:env(behavior);}",
            "c{overscroll-behavior-inline:attr(data-behavior);}",
            "d{overscroll-behavior-inline:--behavior();}",
            "e{overscroll-behavior-inline:first-valid(contain,none);}",
            "f{overscroll-behavior-inline:cycle(contain,none);}",
            "g{overscroll-behavior-inline:interpolate(0%,0:contain,1:none);}",
            "h{overscroll-behavior-inline:foo();}",
            "i{overscroll-behavior-inline:calc(1);}",
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
        3384,
        concat!(
            "a{overscroll-behavior-inline:contain first-valid(none);}",
            "b{overscroll-behavior-inline:first-valid(auto) chain;}",
            "c{overscroll-behavior-inline:auto foo();}",
            "d{overscroll-behavior-inline:foo() none;}",
            "e{overscroll-behavior-inline:chain var(--behavior);}",
            "f{overscroll-behavior-inline:foo(var(--behavior));}",
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
        3385,
        concat!(
            "span{overscroll-behavior-inline:contain;}",
            "div{overscroll-behavior-inline:contain;}",
            "section{overscroll-behavior-inline:contain;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Contain; 3]);
    let indices: Vec<_> = result
        .overscroll_behavior_inline_observations()
        .iter()
        .map(|observation| observation.occurrence_index())
        .collect();
    assert_eq!(indices, [0, 1, 2]);
}

#[test]
fn one_run_interleaves_with_existing_accepted_leaves_without_cross_dispatch() {
    let result = qualify(
        3386,
        concat!(
            "a{direction:ltr;}",
            "b{z-index:1;}",
            "c{text-decoration-skip-ink:all;}",
            "d{overscroll-behavior-x:chain;}",
            "e{overscroll-behavior-y:chain;}",
            "f{overscroll-behavior-inline:chain;}",
            "g{clip-rule:evenodd;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.text_decoration_skip_ink_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);
    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_inline_observations().len(), 1);
    assert_eq!(
        result.overscroll_behavior_inline_observations()[0].occurrence_index(),
        5
    );
    assert_expected(&result, &[ExpectedOutcome::Chain]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        3387,
        "a{overscroll-behavior-inline:auto;}b{overscroll-behavior-inline:auto;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_ne!(
        result.overscroll_behavior_inline_observations()[0]
            .placement()
            .context_id(),
        result.overscroll_behavior_inline_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3390, "@font-face{overscroll-behavior-inline:none;}"),
        (3391, "@page{overscroll-behavior-inline:none;}"),
        (3392, "@page{@top-left{overscroll-behavior-inline:none;}}"),
        (3393, "@keyframes k{from{overscroll-behavior-inline:none;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.overscroll_behavior_inline_observations().is_empty(),
            concat!(
                "nonordinary declaration context produced an ",
                "overscroll-behavior-inline observation for {:?}"
            ),
            css
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3400,
        "a{overscroll-behavior-inline:auto;overscroll-behavior-inline:none;}",
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
        "a{overscroll-behavior-inline:contain;}",
        "b{overscroll-behavior-inline:inherit;}",
        "c{overscroll-behavior-inline:chain;}",
        "d{overscroll-behavior-inline:var(--behavior);}",
        "e{clip-rule:none;}",
    );
    let first = qualify(3410, css);
    let repeated = qualify(3410, css);
    let another_source = qualify(3411, css);

    assert_eq!(
        first.overscroll_behavior_inline_observations(),
        repeated.overscroll_behavior_inline_observations()
    );
    assert_eq!(
        first.overscroll_behavior_inline_observations(),
        another_source.overscroll_behavior_inline_observations()
    );
}
