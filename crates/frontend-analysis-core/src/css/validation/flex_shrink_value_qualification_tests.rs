use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingValue, CssColumnCountQualificationOutcome,
    CssColumnCountValue, CssDirectionQualificationOutcome, CssDirectionValue,
    CssFlexGrowQualificationOutcome, CssFlexGrowValue, CssFlexShrinkQualificationOutcome,
    CssFlexShrinkUnsupportedReason, CssFlexShrinkValue, CssIsolationQualificationOutcome,
    CssIsolationValue, CssOrderQualificationOutcome, CssOrderValue, CssScrollSnapAlignKeyword,
    CssScrollSnapAlignQualificationOutcome, CssScrollSnapAlignValue, CssValueQualificationRunResult,
    CssZIndexQualificationOutcome, CssZIndexValue, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFlexShrinkQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectNumber => {
            CssFlexShrinkQualificationOutcome::Qualified(CssFlexShrinkValue::DirectNumberLiteral)
        }
        ExpectedOutcome::Invalid => {
            CssFlexShrinkQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFlexShrinkQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexShrinkUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssFlexShrinkQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexShrinkUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssFlexShrinkQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexShrinkUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFlexShrinkQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexShrinkUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .flex_shrink_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_number_matrix_reuses_non_negative_number_theorem() {
    let result = qualify(
        800,
        concat!(
            "a{flex-shrink:0;}",
            "b{flex-shrink:+0;}",
            "c{flex-shrink:-0;}",
            "d{flex-shrink:.0;}",
            "e{flex-shrink:-.0;}",
            "f{flex-shrink:0.0;}",
            "g{flex-shrink:-0.0;}",
            "h{flex-shrink:0e100;}",
            "i{flex-shrink:-0e100;}",
            "j{flex-shrink:1;}",
            "k{flex-shrink:+1;}",
            "l{flex-shrink:.5;}",
            "m{flex-shrink:+.5;}",
            "n{flex-shrink:1.0;}",
            "o{flex-shrink:1e0;}",
            "p{flex-shrink:23.4e5;}",
            "q{flex-shrink:+.678E9;}",
            "r{flex-shrink:-1;}",
            "s{flex-shrink:-.5;}",
            "t{flex-shrink:-1.0;}",
            "u{flex-shrink:-1e0;}",
            "v{flex-shrink:-1e-999999;}",
            "w{flex-shrink:1px;}",
            "x{flex-shrink:10%;}",
            "y{flex-shrink:\"1\";}",
            "z{flex-shrink:auto;}",
            "aa{flex-shrink:;}",
            "ab{color:1;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
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
fn comments_priority_and_sign_adjacency_use_retained_number_evidence() {
    let result = qualify(
        801,
        concat!(
            "a{flex-shrink:/**/.5/**/!important;}",
            "b{flex-shrink:/**/-0/**/!important;}",
            "c{flex-shrink:+ 1;}",
            "d{flex-shrink:+/**/1;}",
            "e{flex-shrink:- 1;}",
            "f{flex-shrink:-/**/1;}",
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
fn direct_cardinality_mismatches_are_invalid() {
    let result = qualify(
        802,
        concat!(
            "a{flex-shrink:1 2;}",
            "b{flex-shrink:0 .5;}",
            "c{flex-shrink:auto 1;}",
            "d{flex-shrink:(1);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

#[test]
fn sole_ordinary_functions_are_unsupported_without_numeric_evaluation() {
    let result = qualify(
        803,
        concat!(
            "a{flex-shrink:calc(1);}",
            "b{flex-shrink:calc(-1);}",
            "c{flex-shrink:min(-1,1);}",
            "d{flex-shrink:max(-1,1);}",
            "e{flex-shrink:clamp(-1,.5,1);}",
            "f{flex-shrink:sibling-index();}",
            "g{flex-shrink:sibling-count();}",
            "h{flex-shrink:calc(1px);}",
            "i{flex-shrink:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 9]);
}

#[test]
fn ordinary_function_plus_component_is_directly_invalid() {
    let result = qualify(
        804,
        concat!(
            "a{flex-shrink:foo() 1;}",
            "b{flex-shrink:1 foo();}",
            "c{flex-shrink:calc(1) 2;}",
            "d{flex-shrink:(foo());}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

#[test]
fn css_wide_keywords_are_unsupported_only_as_the_whole_value() {
    let result = qualify(
        805,
        concat!(
            "a{flex-shrink:initial;}",
            "b{flex-shrink:inherit;}",
            "c{flex-shrink:unset;}",
            "d{flex-shrink:revert;}",
            "e{flex-shrink:revert-layer;}",
            "f{flex-shrink:revert-rule;}",
            "g{flex-shrink:1 initial;}",
            "h{flex-shrink:initial 1;}",
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
fn deferred_substitution_remains_conservatively_fail_open_wherever_it_occurs() {
    let result = qualify(
        806,
        concat!(
            "a{flex-shrink:var(--shrink);}",
            "b{flex-shrink:env(shrink);}",
            "c{flex-shrink:attr(data-shrink);}",
            "d{flex-shrink:--shrink();}",
            "e{flex-shrink:-1 var(--shrink);}",
            "f{flex-shrink:1 var(--shrink);}",
            "g{flex-shrink:calc(var(--shrink));}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 7]);
}

#[test]
fn generic_whole_value_functions_keep_entire_value_placement_boundary() {
    let result = qualify(
        807,
        concat!(
            "a{flex-shrink:first-valid(1,0);}",
            "b{flex-shrink:cycle(1,0);}",
            "c{flex-shrink:interpolate(50%,0:0,1:1);}",
            "d{flex-shrink:first-valid(1,0) 2;}",
            "e{flex-shrink:1 first-valid(0,1);}",
        ),
    );

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
fn one_run_owns_upstream_evidence_for_all_selected_value_leaves() {
    use CssScrollSnapAlignKeyword::{End, Start};

    let result = qualify(
        808,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:auto;}",
            "d{order:1;}",
            "e{scroll-snap-align:start end;}",
            "f{z-index:auto;}",
            "g{column-count:2;}",
            "h{flex-grow:.5;}",
            "i{flex-shrink:.5;}",
            "j{direction:rtl;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
    assert_eq!(result.scroll_snap_align_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.column_count_observations().len(), 1);
    assert_eq!(result.flex_grow_observations().len(), 1);
    assert_eq!(result.flex_shrink_observations().len(), 1);

    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 1);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 2);
    assert_eq!(result.order_observations()[0].occurrence_index(), 3);
    assert_eq!(result.scroll_snap_align_observations()[0].occurrence_index(), 4);
    assert_eq!(result.z_index_observations()[0].occurrence_index(), 5);
    assert_eq!(result.column_count_observations()[0].occurrence_index(), 6);
    assert_eq!(result.flex_grow_observations()[0].occurrence_index(), 7);
    assert_eq!(result.flex_shrink_observations()[0].occurrence_index(), 8);
    assert_eq!(result.direction_observations()[1].occurrence_index(), 9);

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
        CssIsolationQualificationOutcome::Qualified(CssIsolationValue::Auto)
    );
    assert_eq!(
        result.order_observations()[0].outcome(),
        CssOrderQualificationOutcome::Qualified(CssOrderValue::DirectIntegerLiteral)
    );
    assert_eq!(
        result.scroll_snap_align_observations()[0].outcome(),
        CssScrollSnapAlignQualificationOutcome::Qualified(CssScrollSnapAlignValue::Pair {
            first: Start,
            second: End,
        })
    );
    assert_eq!(
        result.z_index_observations()[0].outcome(),
        CssZIndexQualificationOutcome::Qualified(CssZIndexValue::Auto)
    );
    assert_eq!(
        result.column_count_observations()[0].outcome(),
        CssColumnCountQualificationOutcome::Qualified(CssColumnCountValue::DirectIntegerLiteral)
    );
    assert_eq!(
        result.flex_grow_observations()[0].outcome(),
        CssFlexGrowQualificationOutcome::Qualified(CssFlexGrowValue::DirectNumberLiteral)
    );
    assert_eq!(
        result.flex_shrink_observations()[0].outcome(),
        CssFlexShrinkQualificationOutcome::Qualified(CssFlexShrinkValue::DirectNumberLiteral)
    );
}

#[test]
fn duplicate_flex_shrink_declarations_keep_distinct_run_local_placement() {
    let result = qualify(809, "a{flex-shrink:.5;}b{flex-shrink:.5;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::DirectNumber, ExpectedOutcome::DirectNumber],
    );
    assert_eq!(result.flex_shrink_observations()[0].occurrence_index(), 0);
    assert_eq!(result.flex_shrink_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.flex_shrink_observations()[0].placement().context_id(),
        result.flex_shrink_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_flex_shrink_observations() {
    for (source_id, css) in [
        (810, "@font-face{flex-shrink:1;}"),
        (811, "@page{flex-shrink:1;}"),
        (812, "@page{@top-left{flex-shrink:1;}}"),
        (813, "@keyframes k{from{flex-shrink:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.flex_shrink_observations().is_empty(),
            "nonordinary declaration context produced a flex-shrink observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_flex_shrink_prefix_and_incomplete_completion() {
    let result = qualify_with_limits(
        820,
        "a{flex-shrink:0;flex-shrink:1;}",
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
fn repeated_and_cross_source_flex_shrink_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{flex-shrink:-0;}",
        "b{flex-shrink:.5;}",
        "c{flex-shrink:-.5;}",
        "d{flex-shrink:calc(-1);}",
        "e{flex-shrink:var(--shrink);}",
        "f{flex-grow:.5;}",
        "g{direction:ltr;}",
        "h{column-count:2;}",
        "i{z-index:auto;}",
    );
    let first = qualify(830, css);
    let repeated = qualify(830, css);
    let another_source = qualify(831, css);

    assert_eq!(
        first.flex_shrink_observations(),
        repeated.flex_shrink_observations()
    );
    assert_eq!(
        first.flex_shrink_observations(),
        another_source.flex_shrink_observations()
    );
    assert_eq!(
        first.flex_grow_observations(),
        repeated.flex_grow_observations()
    );
    assert_eq!(first.direction_observations(), repeated.direction_observations());
    assert_eq!(
        first.column_count_observations(),
        repeated.column_count_observations()
    );
    assert_eq!(first.z_index_observations(), repeated.z_index_observations());
}
