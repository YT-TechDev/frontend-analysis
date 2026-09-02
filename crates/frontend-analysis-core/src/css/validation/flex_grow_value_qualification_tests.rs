use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingValue, CssColumnCountQualificationOutcome,
    CssColumnCountValue, CssDirectionQualificationOutcome, CssDirectionValue,
    CssFlexGrowQualificationOutcome, CssFlexGrowUnsupportedReason, CssFlexGrowValue,
    CssIsolationQualificationOutcome, CssIsolationValue, CssOrderQualificationOutcome,
    CssOrderValue, CssScrollSnapAlignKeyword, CssScrollSnapAlignQualificationOutcome,
    CssScrollSnapAlignValue, CssValueQualificationRunResult, CssZIndexQualificationOutcome,
    CssZIndexValue, run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFlexGrowQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectNumber => {
            CssFlexGrowQualificationOutcome::Qualified(CssFlexGrowValue::DirectNumberLiteral)
        }
        ExpectedOutcome::Invalid => {
            CssFlexGrowQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFlexGrowQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexGrowUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssFlexGrowQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexGrowUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssFlexGrowQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexGrowUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFlexGrowQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFlexGrowUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .flex_grow_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_number_matrix_covers_zero_fraction_exponent_and_range() {
    let result = qualify(
        700,
        concat!(
            "a{flex-grow:0;}",
            "b{flex-grow:+0;}",
            "c{flex-grow:-0;}",
            "d{flex-grow:.0;}",
            "e{flex-grow:-.0;}",
            "f{flex-grow:0.0;}",
            "g{flex-grow:-0.0;}",
            "h{flex-grow:0e100;}",
            "i{flex-grow:-0e100;}",
            "j{flex-grow:1;}",
            "k{flex-grow:+1;}",
            "l{flex-grow:.5;}",
            "m{flex-grow:+.5;}",
            "n{flex-grow:1.0;}",
            "o{flex-grow:1e0;}",
            "p{flex-grow:23.4e5;}",
            "q{flex-grow:+.678E9;}",
            "r{flex-grow:-1;}",
            "s{flex-grow:-.5;}",
            "t{flex-grow:-1.0;}",
            "u{flex-grow:-1e0;}",
            "v{flex-grow:-1e-999999;}",
            "w{flex-grow:1px;}",
            "x{flex-grow:10%;}",
            "y{flex-grow:\"1\";}",
            "z{flex-grow:auto;}",
            "aa{flex-grow:;}",
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
        701,
        concat!(
            "a{flex-grow:/**/.5/**/!important;}",
            "b{flex-grow:/**/-0/**/!important;}",
            "c{flex-grow:+ 1;}",
            "d{flex-grow:+/**/1;}",
            "e{flex-grow:- 1;}",
            "f{flex-grow:-/**/1;}",
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
        702,
        concat!(
            "a{flex-grow:1 2;}",
            "b{flex-grow:0 .5;}",
            "c{flex-grow:auto 1;}",
            "d{flex-grow:(1);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

#[test]
fn sole_ordinary_functions_are_unsupported_without_numeric_evaluation() {
    let result = qualify(
        703,
        concat!(
            "a{flex-grow:calc(1);}",
            "b{flex-grow:calc(-1);}",
            "c{flex-grow:min(-1,1);}",
            "d{flex-grow:max(-1,1);}",
            "e{flex-grow:clamp(-1,.5,1);}",
            "f{flex-grow:sibling-index();}",
            "g{flex-grow:sibling-count();}",
            "h{flex-grow:calc(1px);}",
            "i{flex-grow:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 9]);
}

#[test]
fn ordinary_function_envelope_is_whole_selected_value_shape_aware() {
    let result = qualify(
        704,
        concat!(
            "a{flex-grow:foo() 1;}",
            "b{flex-grow:1 foo();}",
            "c{flex-grow:calc(1) 2;}",
            "d{flex-grow:(foo());}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

#[test]
fn css_wide_keywords_are_unsupported_only_as_the_whole_value() {
    let result = qualify(
        705,
        concat!(
            "a{flex-grow:initial;}",
            "b{flex-grow:inherit;}",
            "c{flex-grow:unset;}",
            "d{flex-grow:revert;}",
            "e{flex-grow:revert-layer;}",
            "f{flex-grow:revert-rule;}",
            "g{flex-grow:1 initial;}",
            "h{flex-grow:initial 1;}",
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
        706,
        concat!(
            "a{flex-grow:var(--grow);}",
            "b{flex-grow:env(grow);}",
            "c{flex-grow:attr(data-grow);}",
            "d{flex-grow:--grow();}",
            "e{flex-grow:-1 var(--grow);}",
            "f{flex-grow:1 var(--grow);}",
            "g{flex-grow:calc(var(--grow));}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 7]);
}

#[test]
fn generic_whole_value_functions_keep_their_entire_value_placement_boundary() {
    let result = qualify(
        707,
        concat!(
            "a{flex-grow:first-valid(1,0);}",
            "b{flex-grow:cycle(1,0);}",
            "c{flex-grow:interpolate(50%,0:0,1:1);}",
            "d{flex-grow:first-valid(1,0) 2;}",
            "e{flex-grow:1 first-valid(0,1);}",
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
        708,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:auto;}",
            "d{order:1;}",
            "e{scroll-snap-align:start end;}",
            "f{z-index:auto;}",
            "g{column-count:2;}",
            "h{flex-grow:.5;}",
            "i{direction:rtl;}",
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

    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 1);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 2);
    assert_eq!(result.order_observations()[0].occurrence_index(), 3);
    assert_eq!(
        result.scroll_snap_align_observations()[0].occurrence_index(),
        4
    );
    assert_eq!(result.z_index_observations()[0].occurrence_index(), 5);
    assert_eq!(result.column_count_observations()[0].occurrence_index(), 6);
    assert_eq!(result.flex_grow_observations()[0].occurrence_index(), 7);
    assert_eq!(result.direction_observations()[1].occurrence_index(), 8);

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
}

#[test]
fn duplicate_flex_grow_declarations_keep_distinct_run_local_placement() {
    let result = qualify(709, "a{flex-grow:.5;}b{flex-grow:.5;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::DirectNumber, ExpectedOutcome::DirectNumber],
    );
    assert_eq!(result.flex_grow_observations()[0].occurrence_index(), 0);
    assert_eq!(result.flex_grow_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.flex_grow_observations()[0].placement().context_id(),
        result.flex_grow_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_flex_grow_observations() {
    for (source_id, css) in [
        (710, "@font-face{flex-grow:1;}"),
        (711, "@page{flex-grow:1;}"),
        (712, "@page{@top-left{flex-grow:1;}}"),
        (713, "@keyframes k{from{flex-grow:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.flex_grow_observations().is_empty(),
            "nonordinary declaration context produced a flex-grow observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_flex_grow_prefix_and_incomplete_completion() {
    let result = qualify_with_limits(
        720,
        "a{flex-grow:0;flex-grow:1;}",
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
fn repeated_and_cross_source_flex_grow_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{flex-grow:-0;}",
        "b{flex-grow:.5;}",
        "c{flex-grow:-.5;}",
        "d{flex-grow:calc(-1);}",
        "e{flex-grow:var(--grow);}",
        "f{direction:ltr;}",
        "g{column-count:2;}",
        "h{z-index:auto;}",
    );
    let first = qualify(730, css);
    let repeated = qualify(730, css);
    let another_source = qualify(731, css);

    assert_eq!(first.flex_grow_observations(), repeated.flex_grow_observations());
    assert_eq!(
        first.flex_grow_observations(),
        another_source.flex_grow_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
    assert_eq!(
        first.column_count_observations(),
        repeated.column_count_observations()
    );
    assert_eq!(
        first.z_index_observations(),
        repeated.z_index_observations()
    );
}
