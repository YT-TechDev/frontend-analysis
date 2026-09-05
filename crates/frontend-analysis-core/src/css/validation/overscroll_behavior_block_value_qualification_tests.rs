use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssOverscrollBehaviorBlockQualificationOutcome, CssOverscrollBehaviorBlockUnsupportedReason,
    CssOverscrollBehaviorBlockValue, CssValueQualificationRunResult, run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssOverscrollBehaviorBlockQualificationOutcome {
    match expected {
        ExpectedOutcome::Contain => CssOverscrollBehaviorBlockQualificationOutcome::Qualified(
            CssOverscrollBehaviorBlockValue::Contain,
        ),
        ExpectedOutcome::None => CssOverscrollBehaviorBlockQualificationOutcome::Qualified(
            CssOverscrollBehaviorBlockValue::None,
        ),
        ExpectedOutcome::Auto => CssOverscrollBehaviorBlockQualificationOutcome::Qualified(
            CssOverscrollBehaviorBlockValue::Auto,
        ),
        ExpectedOutcome::Chain => CssOverscrollBehaviorBlockQualificationOutcome::Qualified(
            CssOverscrollBehaviorBlockValue::Chain,
        ),
        ExpectedOutcome::Invalid => {
            CssOverscrollBehaviorBlockQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssOverscrollBehaviorBlockQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorBlockUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssOverscrollBehaviorBlockQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOverscrollBehaviorBlockUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .overscroll_behavior_block_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_pinned_wpt() {
    let result = qualify(
        3420,
        concat!(
            "a{overscroll-behavior-block:contain;}",
            "b{overscroll-behavior-block:none;}",
            "c{overscroll-behavior-block:auto;}",
            "d{OVERSCROLL-BEHAVIOR-BLOCK:ChAiN;}",
            r"e{overscroll-behavior-block:\63 ontain;}",
            r"f{overscroll-behavior-\62 lock:none;}",
            "g{overscroll-behavior-block:normal;}",
            "h{overscroll-behavior-block:contain none;}",
            "i{overscroll-behavior-block:0;}",
            "j{overscroll-behavior-block:1px;}",
            "k{overscroll-behavior-block:\"auto\";}",
            "l{overscroll-behavior-block:;}",
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
        3421,
        concat!(
            "a{overscroll-behavior-block:/**/contain/**/!important;}",
            "b{overscroll-behavior-block:/**/chain/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Contain, ExpectedOutcome::Chain]);
    for observation in result.overscroll_behavior_block_observations() {
        let occurrence =
            &result.upstream_parser_result().occurrences()[observation.occurrence_index()];
        assert_eq!(observation.placement(), occurrence.placement());
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3422,
        concat!(
            "a{overscroll-behavior-block:initial;}",
            "b{overscroll-behavior-block:inherit;}",
            "c{overscroll-behavior-block:unset;}",
            "d{overscroll-behavior-block:revert;}",
            "e{overscroll-behavior-block:revert-layer;}",
            "f{overscroll-behavior-block:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        3423,
        concat!(
            "a{overscroll-behavior-block:var(--behavior);}",
            "b{overscroll-behavior-block:env(behavior);}",
            "c{overscroll-behavior-block:attr(data-behavior);}",
            "d{overscroll-behavior-block:--behavior();}",
            "e{overscroll-behavior-block:first-valid(contain,none);}",
            "f{overscroll-behavior-block:cycle(contain,none);}",
            "g{overscroll-behavior-block:interpolate(0%,0:contain,1:none);}",
            "h{overscroll-behavior-block:foo();}",
            "i{overscroll-behavior-block:calc(1);}",
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
        3424,
        concat!(
            "a{overscroll-behavior-block:contain first-valid(none);}",
            "b{overscroll-behavior-block:first-valid(auto) chain;}",
            "c{overscroll-behavior-block:auto foo();}",
            "d{overscroll-behavior-block:foo() none;}",
            "e{overscroll-behavior-block:chain var(--behavior);}",
            "f{overscroll-behavior-block:foo(var(--behavior));}",
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
        3425,
        concat!(
            "span{overscroll-behavior-block:contain;}",
            "div{overscroll-behavior-block:contain;}",
            "section{overscroll-behavior-block:contain;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Contain; 3]);
    let indices: Vec<_> = result
        .overscroll_behavior_block_observations()
        .iter()
        .map(|observation| observation.occurrence_index())
        .collect();
    assert_eq!(indices, [0, 1, 2]);
}

#[test]
fn one_run_interleaves_with_existing_accepted_leaves_without_cross_dispatch() {
    let result = qualify(
        3426,
        concat!(
            "a{direction:ltr;}",
            "b{z-index:1;}",
            "c{text-decoration-skip-ink:all;}",
            "d{overscroll-behavior-x:chain;}",
            "e{overscroll-behavior-y:chain;}",
            "f{overscroll-behavior-inline:chain;}",
            "g{overscroll-behavior-block:chain;}",
            "h{clip-rule:evenodd;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.text_decoration_skip_ink_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_y_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_inline_observations().len(), 1);
    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_block_observations().len(), 1);
    assert_eq!(
        result.overscroll_behavior_block_observations()[0].occurrence_index(),
        6
    );
    assert_expected(&result, &[ExpectedOutcome::Chain]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        3427,
        "a{overscroll-behavior-block:auto;}b{overscroll-behavior-block:auto;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_ne!(
        result.overscroll_behavior_block_observations()[0]
            .placement()
            .context_id(),
        result.overscroll_behavior_block_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3430, "@font-face{overscroll-behavior-block:none;}"),
        (3431, "@page{overscroll-behavior-block:none;}"),
        (3432, "@page{@top-left{overscroll-behavior-block:none;}}"),
        (3433, "@keyframes k{from{overscroll-behavior-block:none;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.overscroll_behavior_block_observations().is_empty(),
            concat!(
                "nonordinary declaration context produced an ",
                "overscroll-behavior-block observation for {:?}"
            ),
            css
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        3440,
        "a{overscroll-behavior-block:auto;overscroll-behavior-block:none;}",
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
        "a{overscroll-behavior-block:contain;}",
        "b{overscroll-behavior-block:inherit;}",
        "c{overscroll-behavior-block:chain;}",
        "d{overscroll-behavior-block:var(--behavior);}",
        "e{clip-rule:none;}",
    );
    let first = qualify(3450, css);
    let repeated = qualify(3450, css);
    let another_source = qualify(3451, css);

    assert_eq!(
        first.overscroll_behavior_block_observations(),
        repeated.overscroll_behavior_block_observations()
    );
    assert_eq!(
        first.overscroll_behavior_block_observations(),
        another_source.overscroll_behavior_block_observations()
    );
}
