use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::result::CssTokenizerExecutionCompletion;
use crate::css::validation::{analyze_css_source, CssAnalysisLimits};
use crate::css::value_qualification::{
    CssFontSynthesisPositionQualificationOutcome, CssFontSynthesisPositionUnsupportedReason,
    CssFontSynthesisPositionValue,
};
use crate::SourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    None,
    Invalid,
    UnsupportedCssWide,
    UnsupportedFunction,
}

fn analyze(source_id: u64, source: &str) -> crate::css::value_qualification::CssValueQualificationRunResult {
    analyze_css_source(
        SourceId::new(source_id),
        source,
        CssAnalysisLimits::default(),
    )
    .expect("CSS analysis should succeed")
    .value_qualification_result
}

fn actual_outcomes(
    result: &crate::css::value_qualification::CssValueQualificationRunResult,
) -> Vec<(usize, ExpectedOutcome)> {
    result
        .font_synthesis_position_observations()
        .iter()
        .map(|observation| {
            let outcome = match observation.outcome() {
                CssFontSynthesisPositionQualificationOutcome::Qualified(
                    CssFontSynthesisPositionValue::Auto,
                ) => ExpectedOutcome::Auto,
                CssFontSynthesisPositionQualificationOutcome::Qualified(
                    CssFontSynthesisPositionValue::None,
                ) => ExpectedOutcome::None,
                CssFontSynthesisPositionQualificationOutcome::InvalidForSelectedValueGrammar => {
                    ExpectedOutcome::Invalid
                }
                CssFontSynthesisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontSynthesisPositionUnsupportedReason::CssWideKeyword,
                ) => ExpectedOutcome::UnsupportedCssWide,
                CssFontSynthesisPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssFontSynthesisPositionUnsupportedReason::FunctionValue,
                ) => ExpectedOutcome::UnsupportedFunction,
            };
            (observation.occurrence_index(), outcome)
        })
        .collect()
}

#[test]
fn font_synthesis_position_direct_grammar_is_bounded_and_source_honest() {
    let source = r#"
        a {
            font-synthesis-position: auto;
            font-synthesis-position: none;
            FONT-SYNTHESIS-POSITION: AuTo;
            font-synthesis-position: n\6f ne;
            font-synthesis-position: normal;
            font-synthesis-position: auto none;
            font-synthesis-position: auto, none;
            font-synthesis-position: ;
            font-synthesis-position: 1;
            font-synthesis-position: 1px;
            font-synthesis-position: "auto";
            color: red;
        }
    "#;

    let result = analyze(1, source);
    assert_eq!(
        actual_outcomes(&result),
        vec![
            (0, ExpectedOutcome::Auto),
            (1, ExpectedOutcome::None),
            (2, ExpectedOutcome::Auto),
            (3, ExpectedOutcome::None),
            (4, ExpectedOutcome::Invalid),
            (5, ExpectedOutcome::Invalid),
            (6, ExpectedOutcome::Invalid),
            (7, ExpectedOutcome::Invalid),
            (8, ExpectedOutcome::Invalid),
            (9, ExpectedOutcome::Invalid),
            (10, ExpectedOutcome::Invalid),
        ]
    );
}

#[test]
fn font_synthesis_position_comments_and_priority_do_not_change_value_grammar() {
    let source = r#"
        a {
            font-synthesis-position: /*a*/ auto /*b*/ !important;
            font-synthesis-position: none!important;
        }
    "#;
    let result = analyze(2, source);
    assert_eq!(
        actual_outcomes(&result),
        vec![(0, ExpectedOutcome::Auto), (1, ExpectedOutcome::None)]
    );
}

#[test]
fn font_synthesis_position_css_wide_keywords_are_profile_unsupported() {
    let source = r#"
        a {
            font-synthesis-position: initial;
            font-synthesis-position: inherit;
            font-synthesis-position: unset;
            font-synthesis-position: revert;
            font-synthesis-position: revert-layer;
            font-synthesis-position: revert-rule;
        }
    "#;
    let result = analyze(3, source);
    assert_eq!(
        actual_outcomes(&result),
        vec![
            (0, ExpectedOutcome::UnsupportedCssWide),
            (1, ExpectedOutcome::UnsupportedCssWide),
            (2, ExpectedOutcome::UnsupportedCssWide),
            (3, ExpectedOutcome::UnsupportedCssWide),
            (4, ExpectedOutcome::UnsupportedCssWide),
            (5, ExpectedOutcome::UnsupportedCssWide),
        ]
    );
}

#[test]
fn font_synthesis_position_deferred_and_whole_value_functions_fail_open() {
    let source = r#"
        a {
            font-synthesis-position: var(--x);
            font-synthesis-position: env(foo);
            font-synthesis-position: attr(data-x);
            font-synthesis-position: --custom();
            font-synthesis-position: first-valid(auto, none);
            font-synthesis-position: cycle(auto, none);
            font-synthesis-position: interpolate(auto, none);
            font-synthesis-position: foo();
            font-synthesis-position: calc(1);
        }
    "#;
    let result = analyze(4, source);
    assert_eq!(
        actual_outcomes(&result),
        vec![
            (0, ExpectedOutcome::UnsupportedFunction),
            (1, ExpectedOutcome::UnsupportedFunction),
            (2, ExpectedOutcome::UnsupportedFunction),
            (3, ExpectedOutcome::UnsupportedFunction),
            (4, ExpectedOutcome::UnsupportedFunction),
            (5, ExpectedOutcome::UnsupportedFunction),
            (6, ExpectedOutcome::UnsupportedFunction),
            (7, ExpectedOutcome::Invalid),
            (8, ExpectedOutcome::Invalid),
        ]
    );
}

#[test]
fn font_synthesis_position_function_placement_boundaries_remain_distinct() {
    let source = r#"
        a {
            font-synthesis-position: auto first-valid(none, auto);
            font-synthesis-position: first-valid(auto, none) auto;
            font-synthesis-position: auto foo();
            font-synthesis-position: foo() auto;
            font-synthesis-position: auto var(--x);
            font-synthesis-position: foo(var(--x));
        }
    "#;
    let result = analyze(5, source);
    assert_eq!(
        actual_outcomes(&result),
        vec![
            (0, ExpectedOutcome::Invalid),
            (1, ExpectedOutcome::Invalid),
            (2, ExpectedOutcome::Invalid),
            (3, ExpectedOutcome::Invalid),
            (4, ExpectedOutcome::UnsupportedFunction),
            (5, ExpectedOutcome::UnsupportedFunction),
        ]
    );
}

#[test]
fn font_synthesis_position_interleaves_with_all_accepted_leaves() {
    let source = r#"
        a {
            direction: ltr;
            box-sizing: border-box;
            isolation: isolate;
            backface-visibility: hidden;
            order: 1;
            column-count: 2;
            flex-grow: 1;
            flex-shrink: 1;
            opacity: 1;
            shape-image-threshold: 1;
            shape-margin: 1px;
            line-height: normal;
            word-spacing: normal;
            text-underline-offset: auto;
            scroll-margin-top: 1px;
            border-top-width: thin;
            perspective: none;
            z-index: auto;
            scroll-snap-align: center;
            scroll-snap-stop: always;
            empty-cells: hide;
            text-decoration-style: solid;
            table-layout: fixed;
            border-collapse: collapse;
            box-decoration-break: clone;
            font-kerning: normal;
            font-variant-position: super;
            font-synthesis-weight: none;
            font-synthesis-small-caps: auto;
            font-synthesis-position: none;
        }
    "#;
    let result = analyze(6, source);
    assert_eq!(actual_outcomes(&result), vec![(29, ExpectedOutcome::None)]);
}

#[test]
fn font_synthesis_position_duplicate_declarations_keep_run_local_identity() {
    let source = r#"
        a {
            font-synthesis-position: auto;
            font-synthesis-position: none;
            font-synthesis-position: auto;
        }
    "#;
    let result = analyze(7, source);
    let observations = result.font_synthesis_position_observations();
    assert_eq!(observations.len(), 3);
    assert_eq!(observations[0].occurrence_index(), 0);
    assert_eq!(observations[1].occurrence_index(), 1);
    assert_eq!(observations[2].occurrence_index(), 2);
    assert_ne!(observations[0].placement(), observations[1].placement());
}

#[test]
fn font_synthesis_position_ignores_nonordinary_declaration_contexts() {
    let source = r#"
        @font-face { font-synthesis-position: auto; }
        @page { font-synthesis-position: none; }
        @page { @top-left { font-synthesis-position: auto; } }
        @keyframes x { from { font-synthesis-position: none; } }
        a { font-synthesis-position: auto; }
    "#;
    let result = analyze(8, source);
    assert_eq!(actual_outcomes(&result), vec![(0, ExpectedOutcome::Auto)]);
}

#[test]
fn font_synthesis_position_preserves_upstream_incomplete_committed_prefix() {
    let source = r#"
        a {
            font-synthesis-position: auto;
            font-synthesis-position: none;
        }
    "#;
    let mut limits = CssAnalysisLimits::default();
    limits.tokenizer.max_semantic_tokens = Some(8);
    let result = analyze_css_source(SourceId::new(9), source, limits)
        .expect("resource-limited CSS analysis should preserve committed prefix")
        .value_qualification_result;

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(
        result.upstream_parser_result()
            .upstream_tokenizer_result()
            .execution_completion(),
        CssTokenizerExecutionCompletion::Incomplete
    );
    assert_eq!(actual_outcomes(&result), vec![(0, ExpectedOutcome::Auto)]);
}

#[test]
fn font_synthesis_position_is_deterministic_across_repeated_and_cross_source_runs() {
    let source = "a { font-synthesis-position: auto; font-synthesis-position: none; }";
    let first = analyze(10, source);
    let second = analyze(10, source);
    let other_source = analyze(11, source);

    assert_eq!(
        first.font_synthesis_position_observations(),
        second.font_synthesis_position_observations()
    );
    assert_eq!(actual_outcomes(&first), actual_outcomes(&other_source));
}
