use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontWeightQualificationOutcome, CssFontWeightUnsupportedReason, CssFontWeightValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Bold,
    Bolder,
    Lighter,
    DirectNumber,
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferred,
    UnsupportedWholeValue,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFontWeightQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::Normal)
        }
        ExpectedOutcome::Bold => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::Bold)
        }
        ExpectedOutcome::Bolder => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::Bolder)
        }
        ExpectedOutcome::Lighter => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::Lighter)
        }
        ExpectedOutcome::DirectNumber => {
            CssFontWeightQualificationOutcome::Qualified(CssFontWeightValue::DirectNumberLiteral)
        }
        ExpectedOutcome::Invalid => {
            CssFontWeightQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFontWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontWeightUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssFontWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontWeightUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssFontWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontWeightUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFontWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontWeightUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .font_weight_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_keywords_and_exact_bounds_are_qualified() {
    let result = qualify(
        900,
        concat!(
            "a{font-weight:normal;}",
            "b{font-weight:BOLD;}",
            "c{font-weight:bolder;}",
            "d{font-weight:LIGHTER;}",
            "e{font-weight:1;}",
            "f{font-weight:01;}",
            "g{font-weight:1.0;}",
            "h{font-weight:10e-1;}",
            "i{font-weight:1000;}",
            "j{font-weight:1000.000000000000;}",
            "k{font-weight:1e3;}",
            "l{font-weight:10000e-1;}",
            "m{font-weight:000.001000000000000000e3;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Bold,
            ExpectedOutcome::Bolder,
            ExpectedOutcome::Lighter,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn direct_numbers_outside_the_closed_range_are_invalid_exactly() {
    let result = qualify(
        901,
        concat!(
            "a{font-weight:0;}",
            "b{font-weight:+0;}",
            "c{font-weight:-0;}",
            "d{font-weight:0e100;}",
            "e{font-weight:-0e100;}",
            "f{font-weight:.999999999999999999999999999999;}",
            "g{font-weight:9.999e-1;}",
            "h{font-weight:999999999999999999999e-21;}",
            "i{font-weight:1000.000000000000000000000000001;}",
            "j{font-weight:1001;}",
            "k{font-weight:1.000001e3;}",
            "l{font-weight:-1;}",
            "m{font-weight:-1000;}",
            "n{font-weight:1e1000000;}",
            "o{font-weight:1e-1000000;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 15]);
}

#[test]
fn huge_exponents_are_compared_with_coefficient_scale_without_bounded_exponent_parsing() {
    let positive_cancellation = format!("0.{}1e1000", "0".repeat(999));
    let negative_cancellation = format!("1{}e-1000", "0".repeat(1000));
    let css = format!(
        "a{{font-weight:{positive_cancellation};}}b{{font-weight:{negative_cancellation};}}c{{font-weight:999e999999999999999999;}}d{{font-weight:999e-999999999999999999;}}"
    );
    let result = qualify(902, &css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn comments_priority_and_sign_adjacency_use_retained_number_evidence() {
    let result = qualify(
        903,
        concat!(
            "a{font-weight:/**/400/**/!important;}",
            "b{font-weight:/**/+400/**/!important;}",
            "c{font-weight:+ 400;}",
            "d{font-weight:+/**/400;}",
            "e{font-weight:- 1;}",
            "f{font-weight:-/**/1;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
    assert!(
        result.upstream_parser_result().occurrences()[1]
            .priority()
            .is_some()
    );
}

#[test]
fn sole_numeric_and_ordinary_functions_are_unsupported_without_evaluation() {
    let result = qualify(
        904,
        concat!(
            "a{font-weight:calc(.999);}",
            "b{font-weight:calc(1);}",
            "c{font-weight:calc(1001);}",
            "d{font-weight:calc(-100);}",
            "e{font-weight:min(1,2);}",
            "f{font-weight:max(999,1001);}",
            "g{font-weight:clamp(1,500,1000);}",
            "h{font-weight:calc(1px);}",
            "i{font-weight:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 9]);
}

#[test]
fn deferred_and_whole_value_function_boundaries_remain_fail_open() {
    let result = qualify(
        905,
        concat!(
            "a{font-weight:var(--weight);}",
            "b{font-weight:env(weight);}",
            "c{font-weight:attr(data-weight);}",
            "d{font-weight:--weight();}",
            "e{font-weight:calc(var(--weight));}",
            "f{font-weight:first-valid(400,700);}",
            "g{font-weight:cycle(400,700);}",
            "h{font-weight:interpolate(50%,0:400,1:700);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
        ],
    );
}

#[test]
fn css_wide_keywords_are_unsupported_only_as_the_whole_value() {
    let result = qualify(
        906,
        concat!(
            "a{font-weight:initial;}",
            "b{font-weight:inherit;}",
            "c{font-weight:unset;}",
            "d{font-weight:revert;}",
            "e{font-weight:revert-layer;}",
            "f{font-weight:revert-rule;}",
            "g{font-weight:400 initial;}",
            "h{font-weight:initial 400;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn direct_cardinality_and_token_category_mismatches_are_invalid() {
    let result = qualify(
        907,
        concat!(
            "a{font-weight:400 500;}",
            "b{font-weight:normal 400;}",
            "c{font-weight:10%;}",
            "d{font-weight:400px;}",
            "e{font-weight:\"400\";}",
            "f{font-weight:auto;}",
            "g{font-weight:;}",
            "h{font-weight:calc(1) 400;}",
            "i{font-weight:400 foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 9]);
}

#[test]
fn duplicate_font_weight_declarations_keep_distinct_run_local_placement() {
    let result = qualify(908, "a{font-weight:400;}b{font-weight:400;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::DirectNumber, ExpectedOutcome::DirectNumber],
    );
    assert_eq!(result.font_weight_observations()[0].occurrence_index(), 0);
    assert_eq!(result.font_weight_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.font_weight_observations()[0]
            .placement()
            .context_id(),
        result.font_weight_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_font_weight_contexts_do_not_become_property_observations() {
    for (source_id, css) in [
        (909, "@font-face{font-weight:400;}"),
        (910, "@page{font-weight:400;}"),
        (911, "@page{@top-left{font-weight:400;}}"),
        (912, "@keyframes k{from{font-weight:400;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_weight_observations().is_empty(),
            "nonordinary declaration context produced a font-weight observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_font_weight_prefix_and_incomplete_completion() {
    let result = qualify_with_limits(
        913,
        "a{font-weight:1;font-weight:1000;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::DirectNumber]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_font_weight_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{font-weight:normal;}",
        "b{font-weight:1;}",
        "c{font-weight:1000.0000000000001;}",
        "d{font-weight:calc(1001);}",
        "e{font-weight:var(--weight);}",
    );
    let first = qualify(914, css);
    let repeated = qualify(914, css);
    let another_source = qualify(915, css);

    assert_eq!(
        first.font_weight_observations(),
        repeated.font_weight_observations()
    );
    assert_eq!(
        first.font_weight_observations(),
        another_source.font_weight_observations()
    );
}
