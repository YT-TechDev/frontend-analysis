use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingValue, CssDirectionQualificationOutcome,
    CssDirectionValue, CssIsolationQualificationOutcome, CssIsolationValue,
    CssOrderQualificationOutcome, CssOrderValue, CssScrollSnapAlignKeyword,
    CssScrollSnapAlignQualificationOutcome, CssScrollSnapAlignValue,
    CssValueQualificationRunResult, CssZIndexQualificationOutcome, CssZIndexUnsupportedReason,
    CssZIndexValue, run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssZIndexQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => CssZIndexQualificationOutcome::Qualified(CssZIndexValue::Auto),
        ExpectedOutcome::DirectInteger => {
            CssZIndexQualificationOutcome::Qualified(CssZIndexValue::DirectIntegerLiteral)
        }
        ExpectedOutcome::Invalid => CssZIndexQualificationOutcome::InvalidForSelectedValueGrammar,
        ExpectedOutcome::UnsupportedCssWide => {
            CssZIndexQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssZIndexUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssZIndexQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssZIndexUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssZIndexQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssZIndexUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssZIndexQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssZIndexUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .z_index_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_profile_matrix_covers_auto_integer_and_direct_mismatches() {
    let result = qualify(
        500,
        concat!(
            "a{z-index:auto;}",
            "b{z-index:0;}",
            "c{z-index:1;}",
            "d{z-index:-1;}",
            "e{z-index:+1;}",
            "f{z-index:01;}",
            "g{z-index:000;}",
            "h{z-index:-0;}",
            "i{z-index:999999999999999999999999999999999999999999;}",
            "j{z-index:foo;}",
            "k{z-index:1.0;}",
            "l{z-index:1e0;}",
            "m{z-index:.5;}",
            "n{z-index:1px;}",
            "o{z-index:10%;}",
            "p{z-index:\"1\";}",
            "q{z-index:;}",
            "r{color:1;}",
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
        501,
        concat!(
            "a{z-index:AUTO;}",
            r"b{z-index:\61 uto;}",
            "c{z-index:/**/auto/**/!important;}",
            "d{z-index:/**/+1/**/!important;}",
            "e{z-index:+ 1;}",
            "f{z-index:+/**/1;}",
            "g{z-index:- 1;}",
            "h{z-index:-/**/1;}",
            r"i{z-index:\31;}",
            "j{z-index:１;}",
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
        502,
        concat!(
            "a{z-index:auto 1;}",
            "b{z-index:1 auto;}",
            "c{z-index:auto auto;}",
            "d{z-index:1 2;}",
            "e{z-index:(auto);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

#[test]
fn sole_ordinary_functions_are_unsupported_without_claiming_function_validity() {
    let result = qualify(
        503,
        concat!(
            "a{z-index:calc(1);}",
            "b{z-index:calc(1.5);}",
            "c{z-index:min(1,2);}",
            "d{z-index:max(1,2);}",
            "e{z-index:clamp(0,1,2);}",
            "f{z-index:sibling-index();}",
            "g{z-index:sibling-count();}",
            "h{z-index:random(1,10);}",
            "i{z-index:calc(1px);}",
            "j{z-index:min(1,1px);}",
            "k{z-index:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 11]);
}

#[test]
fn ordinary_function_envelope_is_whole_selected_value_shape_aware() {
    let result = qualify(
        504,
        concat!(
            "a{z-index:foo() 1;}",
            "b{z-index:1 foo();}",
            "c{z-index:calc(1) auto;}",
            "d{z-index:auto calc(1);}",
            "e{z-index:(foo());}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

#[test]
fn css_wide_keywords_are_unsupported_only_as_the_whole_value() {
    let result = qualify(
        505,
        concat!(
            "a{z-index:initial;}",
            "b{z-index:inherit;}",
            "c{z-index:unset;}",
            "d{z-index:revert;}",
            "e{z-index:revert-layer;}",
            "f{z-index:revert-rule;}",
            "g{z-index:auto initial;}",
            "h{z-index:initial 1;}",
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
fn deferred_substitution_remains_fail_open_wherever_it_occurs() {
    let result = qualify(
        506,
        concat!(
            "a{z-index:var(--z);}",
            "b{z-index:env(z);}",
            "c{z-index:attr(data-z);}",
            "d{z-index:--z();}",
            "e{z-index:1 var(--z);}",
            "f{z-index:auto var(--z);}",
            "g{z-index:calc(var(--z));}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 7]);
}

#[test]
fn generic_whole_value_functions_keep_their_entire_value_placement_boundary() {
    let result = qualify(
        507,
        concat!(
            "a{z-index:first-valid(auto,1);}",
            "b{z-index:cycle(auto,1);}",
            "c{z-index:interpolate(50%,0:auto,1:1);}",
            "d{z-index:first-valid(auto,1) 2;}",
            "e{z-index:auto first-valid(1,2);}",
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
        508,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:auto;}",
            "d{order:1;}",
            "e{scroll-snap-align:start end;}",
            "f{z-index:auto;}",
            "g{z-index:-1;}",
            "h{direction:rtl;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
    assert_eq!(result.scroll_snap_align_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 2);

    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 1);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 2);
    assert_eq!(result.order_observations()[0].occurrence_index(), 3);
    assert_eq!(
        result.scroll_snap_align_observations()[0].occurrence_index(),
        4
    );
    assert_eq!(result.z_index_observations()[0].occurrence_index(), 5);
    assert_eq!(result.z_index_observations()[1].occurrence_index(), 6);
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
        result.z_index_observations()[1].outcome(),
        CssZIndexQualificationOutcome::Qualified(CssZIndexValue::DirectIntegerLiteral)
    );
}

#[test]
fn duplicate_z_index_declarations_keep_distinct_run_local_placement() {
    let result = qualify(509, "a{z-index:1;}b{z-index:1;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectInteger,
            ExpectedOutcome::DirectInteger,
        ],
    );
    assert_eq!(result.z_index_observations()[0].occurrence_index(), 0);
    assert_eq!(result.z_index_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.z_index_observations()[0].placement().context_id(),
        result.z_index_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_z_index_observations() {
    for (source_id, css) in [
        (510, "@font-face{z-index:1;}"),
        (511, "@page{z-index:1;}"),
        (512, "@page{@top-left{z-index:1;}}"),
        (513, "@keyframes k{from{z-index:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.z_index_observations().is_empty(),
            "nonordinary declaration context produced a z-index observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_z_index_prefix_and_incomplete_completion() {
    let result = qualify_with_limits(
        520,
        "a{z-index:auto;z-index:1;}",
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
fn repeated_and_cross_source_z_index_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{z-index:auto;}",
        "b{z-index:-1;}",
        "c{z-index:calc(1);}",
        "d{z-index:1.0;}",
        "e{z-index:var(--z);}",
        "f{direction:ltr;}",
        "g{order:1;}",
        "h{scroll-snap-align:start end;}",
    );
    let first = qualify(530, css);
    let repeated = qualify(530, css);
    let another_source = qualify(531, css);

    assert_eq!(
        first.z_index_observations(),
        repeated.z_index_observations()
    );
    assert_eq!(
        first.z_index_observations(),
        another_source.z_index_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
    assert_eq!(first.order_observations(), repeated.order_observations());
    assert_eq!(
        first.scroll_snap_align_observations(),
        repeated.scroll_snap_align_observations()
    );
}
