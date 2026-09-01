use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssDirectionQualificationOutcome, CssDirectionQualificationRunResult,
    CssDirectionUnsupportedReason, CssDirectionValue, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Ltr,
    Rtl,
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

fn qualify(source_id: u64, css: &str) -> CssDirectionQualificationRunResult {
    qualify_with_limits(source_id, css, parser_limits())
}

fn qualify_with_limits(
    source_id: u64,
    css: &str,
    parser_limits: CssParserLimits,
) -> CssDirectionQualificationRunResult {
    let source = SourceText::new(SourceId::new(source_id), css.to_owned());
    let parser_result = analyze_css_source(&source, tokenizer_limits(), parser_limits).unwrap();
    run(parser_result).unwrap()
}

fn assert_expected(result: &CssDirectionQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

fn expected_outcome(expected: ExpectedOutcome) -> CssDirectionQualificationOutcome {
    match expected {
        ExpectedOutcome::Ltr => CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Ltr),
        ExpectedOutcome::Rtl => CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Rtl),
        ExpectedOutcome::Invalid => {
            CssDirectionQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssDirectionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssDirectionUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssDirectionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssDirectionUnsupportedReason::FunctionValue,
            )
        }
    }
}

#[test]
fn handwritten_direction_matrix_matches_the_selected_normative_profile() {
    let css = concat!(
        "a{direction:ltr;}",
        "b{direction:rtl;}",
        "c{direction:RTL;}",
        "d{DIRECTION:lTr;}",
        "e{direction:foo;}",
        "f{direction:ltr rtl;}",
        "g{direction:;}",
        "h{direction:inherit;}",
        "i{direction:var(--dir);}",
        "j{color:ltr;}",
    );
    let result = qualify(1, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Ltr,
            ExpectedOutcome::Rtl,
            ExpectedOutcome::Rtl,
            ExpectedOutcome::Ltr,
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
fn css_wide_keywords_remain_profile_unsupported_not_authored_invalid() {
    let css = concat!(
        "a{direction:initial;}",
        "b{direction:inherit;}",
        "c{direction:unset;}",
        "d{direction:revert;}",
        "e{direction:revert-layer;}",
        "f{direction:revert-rule;}",
    );
    let result = qualify(2, css);

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
fn escaped_identifiers_comments_and_priority_use_retained_lexical_meaning() {
    let css = concat!(
        r"a{d\69 rection:/**/l\74 r/**/!important;}",
        r"b{direction:r\74 l;}",
    );
    let result = qualify(3, css);

    assert_expected(&result, &[ExpectedOutcome::Ltr, ExpectedOutcome::Rtl]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn profile_unsupported_functions_fail_open_but_ordinary_functions_are_invalid() {
    let css = concat!(
        "a{direction:var(--dir);}",
        "b{direction:env(dir);}",
        "c{direction:attr(dir);}",
        "d{direction:first-valid(ltr,rtl);}",
        "e{direction:cycle(ltr,rtl);}",
        "f{direction:interpolate(0%,0:ltr,1:rtl);}",
        "g{direction:foo();}",
        "h{direction:calc(1);}",
    );
    let result = qualify(4, css);

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
fn duplicate_selected_declarations_keep_distinct_run_local_placement() {
    let result = qualify(5, "a{direction:ltr;}b{direction:ltr;}");

    assert_expected(&result, &[ExpectedOutcome::Ltr, ExpectedOutcome::Ltr]);
    assert_eq!(result.observations()[0].occurrence_index(), 0);
    assert_eq!(result.observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.observations()[0].placement().context_id(),
        result.observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_direction_observations() {
    for (source_id, css) in [
        (10, "@font-face{direction:ltr;}"),
        (11, "@page{direction:ltr;}"),
        (12, "@page{@top-left{direction:ltr;}}"),
        (13, "@keyframes k{from{direction:ltr;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.observations().is_empty(),
            "nonordinary declaration context produced a direction observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_the_committed_direction_prefix_and_completion() {
    let result = qualify_with_limits(
        20,
        "a{direction:ltr;direction:rtl;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Ltr]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = "a{direction:ltr;}b{direction:inherit;}c{direction:foo;}";
    let first = qualify(30, css);
    let repeated = qualify(30, css);
    let another_source = qualify(31, css);

    assert_eq!(first.observations(), repeated.observations());
    assert_eq!(first.observations(), another_source.observations());
}
