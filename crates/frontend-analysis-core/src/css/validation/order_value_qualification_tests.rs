use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingValue, CssDirectionQualificationOutcome,
    CssDirectionValue, CssIsolationQualificationOutcome, CssIsolationValue,
    CssOrderQualificationOutcome, CssOrderUnsupportedReason, CssOrderValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    DirectInteger,
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

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .order_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

fn expected_outcome(expected: ExpectedOutcome) -> CssOrderQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectInteger => {
            CssOrderQualificationOutcome::Qualified(CssOrderValue::DirectIntegerLiteral)
        }
        ExpectedOutcome::Invalid => CssOrderQualificationOutcome::InvalidForSelectedValueGrammar,
        ExpectedOutcome::UnsupportedCssWide => {
            CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOrderUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOrderUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOrderUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOrderUnsupportedReason::FunctionValue,
            )
        }
    }
}

#[test]
fn handwritten_order_literal_matrix_matches_the_selected_direct_integer_profile() {
    let css = concat!(
        "a{order:0;}",
        "b{order:1;}",
        "c{order:-1;}",
        "d{order:+1;}",
        "e{order:01;}",
        "f{order:-0;}",
        "g{order:999999999999999999999999999999999999;}",
        "h{order:1.0;}",
        "i{order:1e0;}",
        "j{order:1px;}",
        "k{order:10%;}",
        "l{order:foo;}",
        "m{order:\"1\";}",
        "n{order:1 2;}",
        "o{color:1;}",
    );
    let result = qualify(300, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::Invalid,
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
fn comments_priority_and_sign_adjacency_use_retained_token_structure() {
    let css = concat!(
        "a{order:/**/+1/**/!important;}",
        "b{order:-1/**/;}",
        "c{order:+/**/1;}",
    );
    let result = qualify(301, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::Invalid,
        ],
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let css = concat!(
        "a{order:initial;}",
        "b{order:inherit;}",
        "c{order:unset;}",
        "d{order:revert;}",
        "e{order:revert-layer;}",
        "f{order:revert-rule;}",
    );
    let result = qualify(302, css);

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
fn sole_ordinary_functions_are_unsupported_without_a_numeric_function_registry() {
    let css = concat!(
        "a{order:calc(1);}",
        "b{order:calc(1px);}",
        "c{order:min(1,2);}",
        "d{order:min(1,1px);}",
        "e{order:sibling-index();}",
        "f{order:sibling-count();}",
        "g{order:random(1,10);}",
        "h{order:foo();}",
    );
    let result = qualify(303, css);

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
            ExpectedOutcome::UnsupportedFunction,
        ],
    );
}

#[test]
fn ordinary_function_unsupported_is_whole_selected_value_shape_aware() {
    let css = concat!(
        "a{order:foo() 1;}",
        "b{order:1 foo();}",
        "c{order:calc(1) 2;}",
        "d{order:(foo());}",
    );
    let result = qualify(304, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn deferred_substitution_remains_fail_open_wherever_it_occurs() {
    let css = concat!(
        "a{order:var(--order);}",
        "b{order:env(order);}",
        "c{order:attr(data-order);}",
        "d{order:--order();}",
        "e{order:1 var(--order);}",
        "f{order:calc(var(--order));}",
    );
    let result = qualify(305, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
        ],
    );
}

#[test]
fn generic_whole_value_functions_keep_their_placement_boundary() {
    let css = concat!(
        "a{order:first-valid(1,2);}",
        "b{order:cycle(1,2);}",
        "c{order:interpolate(50%,0:1,1:2);}",
        "d{order:first-valid(1,2) 3;}",
        "e{order:3 cycle(1,2);}",
    );
    let result = qualify(306, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn one_run_owns_upstream_evidence_for_keyword_and_numeric_leaves() {
    let result = qualify(
        307,
        "a{direction:ltr;order:1;isolation:auto;box-sizing:border-box;order:calc(1);direction:rtl;}",
    );

    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 2);

    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.order_observations()[0].occurrence_index(), 1);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 2);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 3);
    assert_eq!(result.order_observations()[1].occurrence_index(), 4);
    assert_eq!(result.direction_observations()[1].occurrence_index(), 5);

    assert_eq!(
        result.direction_observations()[0].outcome(),
        CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Ltr)
    );
    assert_eq!(
        result.order_observations()[0].outcome(),
        CssOrderQualificationOutcome::Qualified(CssOrderValue::DirectIntegerLiteral)
    );
    assert_eq!(
        result.isolation_observations()[0].outcome(),
        CssIsolationQualificationOutcome::Qualified(CssIsolationValue::Auto)
    );
    assert_eq!(
        result.box_sizing_observations()[0].outcome(),
        CssBoxSizingQualificationOutcome::Qualified(CssBoxSizingValue::BorderBox)
    );
    assert_eq!(
        result.order_observations()[1].outcome(),
        CssOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
            CssOrderUnsupportedReason::FunctionValue,
        )
    );
}

#[test]
fn duplicate_order_declarations_keep_distinct_run_local_placement() {
    let result = qualify(308, "a{order:1;}b{order:1;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::DirectInteger, ExpectedOutcome::DirectInteger],
    );
    assert_eq!(result.order_observations()[0].occurrence_index(), 0);
    assert_eq!(result.order_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.order_observations()[0].placement().context_id(),
        result.order_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_order_observations() {
    for (source_id, css) in [
        (310, "@font-face{order:1;}"),
        (311, "@page{order:1;}"),
        (312, "@page{@top-left{order:1;}}"),
        (313, "@keyframes k{from{order:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.order_observations().is_empty(),
            "nonordinary declaration context produced an order observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_the_committed_order_prefix_and_completion() {
    let result = qualify_with_limits(
        320,
        "a{order:1;order:2;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::DirectInteger]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_order_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{order:1;}",
        "b{order:calc(1);}",
        "c{order:1.0;}",
        "d{order:var(--order);}",
        "e{direction:ltr;}",
        "f{box-sizing:border-box;}",
        "g{isolation:auto;}",
    );
    let first = qualify(330, css);
    let repeated = qualify(330, css);
    let another_source = qualify(331, css);

    assert_eq!(first.order_observations(), repeated.order_observations());
    assert_eq!(first.order_observations(), another_source.order_observations());
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
    assert_eq!(
        first.box_sizing_observations(),
        repeated.box_sizing_observations()
    );
    assert_eq!(
        first.isolation_observations(),
        repeated.isolation_observations()
    );
}
