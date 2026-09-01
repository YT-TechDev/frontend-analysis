use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingUnsupportedReason, CssBoxSizingValue,
    CssDirectionQualificationOutcome, CssDirectionValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    ContentBox,
    BorderBox,
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

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .box_sizing_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

fn expected_outcome(expected: ExpectedOutcome) -> CssBoxSizingQualificationOutcome {
    match expected {
        ExpectedOutcome::ContentBox => {
            CssBoxSizingQualificationOutcome::Qualified(CssBoxSizingValue::ContentBox)
        }
        ExpectedOutcome::BorderBox => {
            CssBoxSizingQualificationOutcome::Qualified(CssBoxSizingValue::BorderBox)
        }
        ExpectedOutcome::Invalid => {
            CssBoxSizingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssBoxSizingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBoxSizingUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssBoxSizingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBoxSizingUnsupportedReason::FunctionValue,
            )
        }
    }
}

#[test]
fn handwritten_box_sizing_matrix_matches_the_selected_normative_profile() {
    let css = concat!(
        "a{box-sizing:content-box;}",
        "b{box-sizing:border-box;}",
        "c{box-sizing:BORDER-BOX;}",
        "d{BOX-SIZING:Content-Box;}",
        "e{box-sizing:foo;}",
        "f{box-sizing:content-box border-box;}",
        "g{box-sizing:;}",
        "h{box-sizing:inherit;}",
        "i{box-sizing:var(--sizing);}",
        "j{color:border-box;}",
    );
    let result = qualify(100, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::ContentBox,
            ExpectedOutcome::BorderBox,
            ExpectedOutcome::BorderBox,
            ExpectedOutcome::ContentBox,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedFunction,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn escaped_identifiers_comments_and_priority_use_retained_lexical_meaning() {
    let css = concat!(
        r"a{b\6f x-sizing:/**/c\6f ntent-box/**/!important;}",
        r"b{box-sizing:b\6f rder-box;}",
    );
    let result = qualify(101, css);

    assert_expected(
        &result,
        &[ExpectedOutcome::ContentBox, ExpectedOutcome::BorderBox],
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn css_wide_keywords_remain_profile_unsupported_not_authored_invalid() {
    let css = concat!(
        "a{box-sizing:initial;}",
        "b{box-sizing:inherit;}",
        "c{box-sizing:unset;}",
        "d{box-sizing:revert;}",
        "e{box-sizing:revert-layer;}",
        "f{box-sizing:revert-rule;}",
    );
    let result = qualify(102, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
        ],
    );
}

#[test]
fn profile_unsupported_functions_fail_open_but_ordinary_functions_are_invalid() {
    let css = concat!(
        "a{box-sizing:var(--sizing);}",
        "b{box-sizing:env(sizing);}",
        "c{box-sizing:attr(data-sizing);}",
        "d{box-sizing:first-valid(content-box,border-box);}",
        "e{box-sizing:cycle(content-box,border-box);}",
        "f{box-sizing:interpolate(0%,0:content-box,1:border-box);}",
        "g{box-sizing:foo();}",
        "h{box-sizing:calc(1);}",
    );
    let result = qualify(103, css);

    assert_expected(
        &result,
        &[
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
fn whole_value_functions_require_entire_value_placement() {
    let css = concat!(
        "a{box-sizing:content-box first-valid(border-box);}",
        "b{box-sizing:first-valid(border-box) content-box;}",
        "c{box-sizing:cycle(content-box,border-box) foo();}",
        "d{box-sizing:content-box var(--sizing);}",
    );
    let result = qualify(104, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::UnsupportedFunction,
        ],
    );
}

#[test]
fn one_run_owns_upstream_evidence_for_both_selected_properties() {
    let result = qualify(
        105,
        "a{direction:ltr;box-sizing:border-box;color:red;direction:rtl;}",
    );

    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 1);
    assert_eq!(result.direction_observations()[1].occurrence_index(), 3);
    assert_eq!(
        result.direction_observations()[0].outcome(),
        CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Ltr)
    );
    assert_eq!(
        result.box_sizing_observations()[0].outcome(),
        CssBoxSizingQualificationOutcome::Qualified(CssBoxSizingValue::BorderBox)
    );
}

#[test]
fn duplicate_selected_declarations_keep_distinct_run_local_placement() {
    let result = qualify(106, "a{box-sizing:content-box;}b{box-sizing:content-box;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::ContentBox, ExpectedOutcome::ContentBox],
    );
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 0);
    assert_eq!(result.box_sizing_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.box_sizing_observations()[0].placement().context_id(),
        result.box_sizing_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_box_sizing_observations() {
    for (source_id, css) in [
        (110, "@font-face{box-sizing:border-box;}"),
        (111, "@page{box-sizing:border-box;}"),
        (112, "@page{@top-left{box-sizing:border-box;}}"),
        (113, "@keyframes k{from{box-sizing:border-box;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.box_sizing_observations().is_empty(),
            "nonordinary declaration context produced a box-sizing observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_the_committed_box_sizing_prefix_and_completion() {
    let result = qualify_with_limits(
        120,
        "a{box-sizing:content-box;box-sizing:border-box;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::ContentBox]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{box-sizing:content-box;}",
        "b{box-sizing:inherit;}",
        "c{box-sizing:foo;}",
        "d{direction:ltr;}",
    );
    let first = qualify(130, css);
    let repeated = qualify(130, css);
    let another_source = qualify(131, css);

    assert_eq!(
        first.box_sizing_observations(),
        repeated.box_sizing_observations()
    );
    assert_eq!(
        first.box_sizing_observations(),
        another_source.box_sizing_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
}
