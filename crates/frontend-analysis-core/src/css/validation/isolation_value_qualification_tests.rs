use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingValue, CssDirectionQualificationOutcome,
    CssDirectionValue, CssIsolationQualificationOutcome, CssIsolationUnsupportedReason,
    CssIsolationValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Isolate,
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
        .isolation_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

fn expected_outcome(expected: ExpectedOutcome) -> CssIsolationQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssIsolationQualificationOutcome::Qualified(CssIsolationValue::Auto)
        }
        ExpectedOutcome::Isolate => {
            CssIsolationQualificationOutcome::Qualified(CssIsolationValue::Isolate)
        }
        ExpectedOutcome::Invalid => {
            CssIsolationQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssIsolationQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssIsolationUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssIsolationQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssIsolationUnsupportedReason::FunctionValue,
            )
        }
    }
}

#[test]
fn handwritten_isolation_matrix_matches_the_selected_normative_profile() {
    let css = concat!(
        "a{isolation:auto;}",
        "b{isolation:isolate;}",
        "c{isolation:ISOLATE;}",
        "d{ISOLATION:Auto;}",
        "e{isolation:foo;}",
        "f{isolation:auto isolate;}",
        "g{isolation:;}",
        "h{isolation:inherit;}",
        "i{isolation:var(--mode);}",
        "j{color:isolate;}",
    );
    let result = qualify(200, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Isolate,
            ExpectedOutcome::Isolate,
            ExpectedOutcome::Auto,
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
        r"a{i\73 olation:/**/i\73 olate/**/!important;}",
        r"b{isolation:a\75 to;}",
    );
    let result = qualify(201, css);

    assert_expected(&result, &[ExpectedOutcome::Isolate, ExpectedOutcome::Auto]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn css_wide_keywords_remain_profile_unsupported_not_authored_invalid() {
    let css = concat!(
        "a{isolation:initial;}",
        "b{isolation:inherit;}",
        "c{isolation:unset;}",
        "d{isolation:revert;}",
        "e{isolation:revert-layer;}",
        "f{isolation:revert-rule;}",
    );
    let result = qualify(202, css);

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
        "a{isolation:var(--mode);}",
        "b{isolation:env(mode);}",
        "c{isolation:attr(data-mode);}",
        "d{isolation:first-valid(auto,isolate);}",
        "e{isolation:cycle(auto,isolate);}",
        "f{isolation:interpolate(0%,0:auto,1:isolate);}",
        "g{isolation:--mode();}",
        "h{isolation:foo();}",
        "i{isolation:calc(1);}",
    );
    let result = qualify(203, css);

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
fn whole_value_functions_require_entire_value_placement() {
    let css = concat!(
        "a{isolation:auto first-valid(isolate);}",
        "b{isolation:first-valid(isolate) auto;}",
        "c{isolation:cycle(auto,isolate) foo();}",
        "d{isolation:auto var(--mode);}",
    );
    let result = qualify(204, css);

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
fn one_run_owns_upstream_evidence_for_all_selected_properties() {
    let result = qualify(
        205,
        "a{direction:ltr;isolation:isolate;box-sizing:border-box;color:red;isolation:auto;direction:rtl;}",
    );

    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 2);
    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 1);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 2);
    assert_eq!(result.isolation_observations()[1].occurrence_index(), 4);
    assert_eq!(result.direction_observations()[1].occurrence_index(), 5);
    assert_eq!(
        result.direction_observations()[0].outcome(),
        CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Ltr)
    );
    assert_eq!(
        result.box_sizing_observations()[0].outcome(),
        CssBoxSizingQualificationOutcome::Qualified(CssBoxSizingValue::BorderBox)
    );
    assert_eq!(
        result.isolation_observations()[0].outcome(),
        CssIsolationQualificationOutcome::Qualified(CssIsolationValue::Isolate)
    );
    assert_eq!(
        result.isolation_observations()[1].outcome(),
        CssIsolationQualificationOutcome::Qualified(CssIsolationValue::Auto)
    );
}

#[test]
fn duplicate_selected_declarations_keep_distinct_run_local_placement() {
    let result = qualify(206, "a{isolation:auto;}b{isolation:auto;}");

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 0);
    assert_eq!(result.isolation_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.isolation_observations()[0].placement().context_id(),
        result.isolation_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_isolation_observations() {
    for (source_id, css) in [
        (210, "@font-face{isolation:isolate;}"),
        (211, "@page{isolation:isolate;}"),
        (212, "@page{@top-left{isolation:isolate;}}"),
        (213, "@keyframes k{from{isolation:isolate;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.isolation_observations().is_empty(),
            "nonordinary declaration context produced an isolation observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_the_committed_isolation_prefix_and_completion() {
    let result = qualify_with_limits(
        220,
        "a{isolation:auto;isolation:isolate;}",
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
        "a{isolation:auto;}",
        "b{isolation:inherit;}",
        "c{isolation:foo;}",
        "d{direction:ltr;}",
        "e{box-sizing:border-box;}",
    );
    let first = qualify(230, css);
    let repeated = qualify(230, css);
    let another_source = qualify(231, css);

    assert_eq!(
        first.isolation_observations(),
        repeated.isolation_observations()
    );
    assert_eq!(
        first.isolation_observations(),
        another_source.isolation_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
    assert_eq!(
        first.box_sizing_observations(),
        repeated.box_sizing_observations()
    );
}
