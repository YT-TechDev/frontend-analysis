use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingValue, CssColumnCountQualificationOutcome,
    CssColumnCountUnsupportedReason, CssColumnCountValue, CssDirectionQualificationOutcome,
    CssDirectionValue, CssIsolationQualificationOutcome, CssIsolationValue,
    CssOrderQualificationOutcome, CssOrderValue, CssScrollSnapAlignKeyword,
    CssScrollSnapAlignQualificationOutcome, CssScrollSnapAlignValue,
    CssValueQualificationRunResult, CssZIndexQualificationOutcome, CssZIndexValue, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssColumnCountQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssColumnCountQualificationOutcome::Qualified(CssColumnCountValue::Auto)
        }
        ExpectedOutcome::DirectInteger => CssColumnCountQualificationOutcome::Qualified(
            CssColumnCountValue::DirectIntegerLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssColumnCountQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssColumnCountQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColumnCountUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssColumnCountQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColumnCountUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssColumnCountQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColumnCountUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssColumnCountQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssColumnCountUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .column_count_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_profile_matrix_covers_positive_range_and_direct_mismatches() {
    let result = qualify(
        600,
        concat!(
            "a{column-count:auto;}",
            "b{column-count:1;}",
            "c{column-count:+1;}",
            "d{column-count:01;}",
            "e{column-count:0001;}",
            "f{column-count:999999999999999999999999999999999999999999;}",
            "g{column-count:0;}",
            "h{column-count:+0;}",
            "i{column-count:-0;}",
            "j{column-count:00;}",
            "k{column-count:-1;}",
            "l{column-count:-999999999999999999999999999999999999999999;}",
            "m{column-count:1.0;}",
            "n{column-count:1e0;}",
            "o{column-count:.5;}",
            "p{column-count:1px;}",
            "q{column-count:10%;}",
            "r{column-count:\"1\";}",
            "s{column-count:foo;}",
            "t{column-count:;}",
            "u{color:1;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
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
fn decoded_auto_comments_priority_and_sign_adjacency_use_retained_evidence() {
    let result = qualify(
        601,
        concat!(
            "a{column-count:AUTO;}",
            r"b{column-count:\61 uto;}",
            "c{column-count:/**/auto/**/!important;}",
            "d{column-count:/**/+1/**/!important;}",
            "e{column-count:+ 1;}",
            "f{column-count:+/**/1;}",
            "g{column-count:- 1;}",
            "h{column-count:-/**/1;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    assert!(
        result.upstream_parser_result().occurrences()[2]
            .priority()
            .is_some()
    );
    assert!(
        result.upstream_parser_result().occurrences()[3]
            .priority()
            .is_some()
    );
}

#[test]
fn direct_cardinality_mismatches_are_invalid() {
    let result = qualify(
        602,
        concat!(
            "a{column-count:auto 1;}",
            "b{column-count:1 auto;}",
            "c{column-count:auto auto;}",
            "d{column-count:1 2;}",
            "e{column-count:(1);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

#[test]
fn sole_ordinary_functions_are_unsupported_without_range_or_type_evaluation() {
    let result = qualify(
        603,
        concat!(
            "a{column-count:calc(1);}",
            "b{column-count:calc(0);}",
            "c{column-count:calc(-1);}",
            "d{column-count:calc(0.5);}",
            "e{column-count:calc(1.5);}",
            "f{column-count:min(0,1);}",
            "g{column-count:max(0,1);}",
            "h{column-count:clamp(0,1,2);}",
            "i{column-count:sibling-index();}",
            "j{column-count:sibling-count();}",
            "k{column-count:random(1,10);}",
            "l{column-count:calc(1px);}",
            "m{column-count:min(1,1px);}",
            "n{column-count:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 14]);
}

#[test]
fn ordinary_function_envelope_is_whole_selected_value_shape_aware() {
    let result = qualify(
        604,
        concat!(
            "a{column-count:foo() 1;}",
            "b{column-count:1 foo();}",
            "c{column-count:calc(1) auto;}",
            "d{column-count:auto calc(1);}",
            "e{column-count:(foo());}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

#[test]
fn css_wide_keywords_are_unsupported_only_as_the_whole_value() {
    let result = qualify(
        605,
        concat!(
            "a{column-count:initial;}",
            "b{column-count:inherit;}",
            "c{column-count:unset;}",
            "d{column-count:revert;}",
            "e{column-count:revert-layer;}",
            "f{column-count:revert-rule;}",
            "g{column-count:auto initial;}",
            "h{column-count:initial 1;}",
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
        606,
        concat!(
            "a{column-count:var(--count);}",
            "b{column-count:env(count);}",
            "c{column-count:attr(data-count);}",
            "d{column-count:--count();}",
            "e{column-count:0 var(--count);}",
            "f{column-count:auto var(--count);}",
            "g{column-count:calc(var(--count));}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 7]);
}

#[test]
fn generic_whole_value_functions_keep_their_entire_value_placement_boundary() {
    let result = qualify(
        607,
        concat!(
            "a{column-count:first-valid(auto,1);}",
            "b{column-count:cycle(auto,1);}",
            "c{column-count:interpolate(50%,0:auto,1:1);}",
            "d{column-count:first-valid(auto,1) 2;}",
            "e{column-count:auto first-valid(1,2);}",
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
        608,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:auto;}",
            "d{order:1;}",
            "e{scroll-snap-align:start end;}",
            "f{z-index:auto;}",
            "g{column-count:2;}",
            "h{direction:rtl;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
    assert_eq!(result.scroll_snap_align_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.column_count_observations().len(), 1);

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
    assert_eq!(result.direction_observations()[1].occurrence_index(), 7);

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
}

#[test]
fn duplicate_column_count_declarations_keep_distinct_run_local_placement() {
    let result = qualify(609, "a{column-count:1;}b{column-count:1;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
        ],
    );
    assert_eq!(result.column_count_observations()[0].occurrence_index(), 0);
    assert_eq!(result.column_count_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.column_count_observations()[0]
            .placement()
            .context_id(),
        result.column_count_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_column_count_observations() {
    for (source_id, css) in [
        (610, "@font-face{column-count:1;}"),
        (611, "@page{column-count:1;}"),
        (612, "@page{@top-left{column-count:1;}}"),
        (613, "@keyframes k{from{column-count:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.column_count_observations().is_empty(),
            "nonordinary declaration context produced a column-count observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_column_count_prefix_and_incomplete_completion() {
    let result = qualify_with_limits(
        620,
        "a{column-count:auto;column-count:1;}",
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
fn repeated_and_cross_source_column_count_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{column-count:auto;}",
        "b{column-count:1;}",
        "c{column-count:0;}",
        "d{column-count:calc(0);}",
        "e{column-count:var(--count);}",
        "f{direction:ltr;}",
        "g{order:1;}",
        "h{z-index:auto;}",
    );
    let first = qualify(630, css);
    let repeated = qualify(630, css);
    let another_source = qualify(631, css);

    assert_eq!(
        first.column_count_observations(),
        repeated.column_count_observations()
    );
    assert_eq!(
        first.column_count_observations(),
        another_source.column_count_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
    assert_eq!(first.order_observations(), repeated.order_observations());
    assert_eq!(
        first.z_index_observations(),
        repeated.z_index_observations()
    );
}
