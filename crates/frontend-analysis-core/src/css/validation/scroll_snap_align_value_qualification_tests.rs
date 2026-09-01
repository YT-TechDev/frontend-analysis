use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingValue, CssDirectionQualificationOutcome,
    CssDirectionValue, CssIsolationQualificationOutcome, CssIsolationValue,
    CssOrderQualificationOutcome, CssOrderValue, CssScrollSnapAlignKeyword,
    CssScrollSnapAlignQualificationOutcome, CssScrollSnapAlignUnsupportedReason,
    CssScrollSnapAlignValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Single(CssScrollSnapAlignKeyword),
    Pair(CssScrollSnapAlignKeyword, CssScrollSnapAlignKeyword),
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferred,
    UnsupportedWholeValue,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssScrollSnapAlignQualificationOutcome {
    match expected {
        ExpectedOutcome::Single(keyword) => CssScrollSnapAlignQualificationOutcome::Qualified(
            CssScrollSnapAlignValue::Single(keyword),
        ),
        ExpectedOutcome::Pair(first, second) => {
            CssScrollSnapAlignQualificationOutcome::Qualified(CssScrollSnapAlignValue::Pair {
                first,
                second,
            })
        }
        ExpectedOutcome::Invalid => {
            CssScrollSnapAlignQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssScrollSnapAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollSnapAlignUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssScrollSnapAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollSnapAlignUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssScrollSnapAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollSnapAlignUnsupportedReason::WholeValueFunction,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .scroll_snap_align_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_single_and_pair_matrix_preserves_authored_arity_and_order() {
    use CssScrollSnapAlignKeyword::{Center, End, None, Start};

    let result = qualify(
        400,
        concat!(
            "a{scroll-snap-align:none;}",
            "b{scroll-snap-align:start;}",
            "c{scroll-snap-align:end;}",
            "d{scroll-snap-align:center;}",
            "e{scroll-snap-align:start none;}",
            "f{scroll-snap-align:center end;}",
            "g{scroll-snap-align:start start;}",
            "h{scroll-snap-align:end center;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Single(None),
            ExpectedOutcome::Single(Start),
            ExpectedOutcome::Single(End),
            ExpectedOutcome::Single(Center),
            ExpectedOutcome::Pair(Start, None),
            ExpectedOutcome::Pair(Center, End),
            ExpectedOutcome::Pair(Start, Start),
            ExpectedOutcome::Pair(End, Center),
        ],
    );
    assert_ne!(
        result.scroll_snap_align_observations()[1].outcome(),
        result.scroll_snap_align_observations()[6].outcome(),
        "authored single `start` must not collapse with authored pair `start start`"
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn direct_mismatches_are_invalid_without_computed_or_runtime_reasoning() {
    let result = qualify(
        401,
        concat!(
            "a{scroll-snap-align:auto;}",
            "b{scroll-snap-align:start invalid;}",
            "c{scroll-snap-align:start end center;}",
            "d{scroll-snap-align:1;}",
            "e{scroll-snap-align:\"start\";}",
            "f{scroll-snap-align:foo();}",
            "g{scroll-snap-align:calc(1);}",
            "h{scroll-snap-align:(start);}",
            "i{scroll-snap-align:;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 9]);
}

#[test]
fn css_wide_keywords_are_profile_unsupported_only_as_the_whole_value() {
    let result = qualify(
        402,
        concat!(
            "a{scroll-snap-align:initial;}",
            "b{scroll-snap-align:inherit;}",
            "c{scroll-snap-align:unset;}",
            "d{scroll-snap-align:revert;}",
            "e{scroll-snap-align:revert-layer;}",
            "f{scroll-snap-align:revert-rule;}",
            "g{scroll-snap-align:start initial;}",
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
        ],
    );
}

#[test]
fn deferred_substitution_remains_fail_open_wherever_it_occurs() {
    let result = qualify(
        403,
        concat!(
            "a{scroll-snap-align:var(--align);}",
            "b{scroll-snap-align:env(align);}",
            "c{scroll-snap-align:attr(data-align);}",
            "d{scroll-snap-align:--align();}",
            "e{scroll-snap-align:start var(--align);}",
            "f{scroll-snap-align:foo(var(--align));}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 6]);
}

#[test]
fn whole_value_functions_keep_their_entire_value_placement_boundary() {
    let result = qualify(
        404,
        concat!(
            "a{scroll-snap-align:first-valid(start,end);}",
            "b{scroll-snap-align:cycle(start,end);}",
            "c{scroll-snap-align:interpolate(50%,0:start,1:end);}",
            "d{scroll-snap-align:start first-valid(end,center);}",
            "e{scroll-snap-align:cycle(start,end) center;}",
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
fn decoded_identifiers_comments_case_and_priority_are_qualified_from_retained_evidence() {
    use CssScrollSnapAlignKeyword::{Center, End, Start};

    let result = qualify(
        405,
        concat!(
            "a{scroll-snap-align:START;}",
            r"b{scroll-snap-align:st\61 rt;}",
            "c{scroll-snap-align:center/**/end!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Single(Start),
            ExpectedOutcome::Single(Start),
            ExpectedOutcome::Pair(Center, End),
        ],
    );
    assert!(
        result.upstream_parser_result().occurrences()[2]
            .priority()
            .is_some()
    );
}

#[test]
fn one_run_owns_upstream_evidence_for_existing_leaves_and_scroll_snap_align() {
    use CssScrollSnapAlignKeyword::{End, Start};

    let result = qualify(
        406,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:auto;}",
            "d{order:1;}",
            "e{scroll-snap-align:start end;}",
            "f{direction:rtl;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
    assert_eq!(result.scroll_snap_align_observations().len(), 1);

    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 1);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 2);
    assert_eq!(result.order_observations()[0].occurrence_index(), 3);
    assert_eq!(
        result.scroll_snap_align_observations()[0].occurrence_index(),
        4
    );
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
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        407,
        "a{scroll-snap-align:start;}b{scroll-snap-align:start;}",
    );

    assert_eq!(result.scroll_snap_align_observations().len(), 2);
    assert_eq!(
        result.scroll_snap_align_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.scroll_snap_align_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.scroll_snap_align_observations()[0]
            .placement()
            .context_id(),
        result.scroll_snap_align_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (410, "@font-face{scroll-snap-align:start;}"),
        (411, "@page{scroll-snap-align:start;}"),
        (412, "@page{@top-left{scroll-snap-align:start;}}"),
        (413, "@keyframes k{from{scroll-snap-align:start;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.scroll_snap_align_observations().is_empty(),
            "nonordinary declaration context produced an observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_incomplete_completion() {
    use CssScrollSnapAlignKeyword::Start;

    let result = qualify_with_limits(
        420,
        "a{scroll-snap-align:start;scroll-snap-align:end;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Single(Start)]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{scroll-snap-align:start end;}",
        "b{scroll-snap-align:var(--align);}",
        "c{scroll-snap-align:auto;}",
        "d{direction:ltr;}",
        "e{order:1;}",
    );
    let first = qualify(430, css);
    let repeated = qualify(430, css);
    let another_source = qualify(431, css);

    assert_eq!(
        first.scroll_snap_align_observations(),
        repeated.scroll_snap_align_observations()
    );
    assert_eq!(
        first.scroll_snap_align_observations(),
        another_source.scroll_snap_align_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
    assert_eq!(first.order_observations(), repeated.order_observations());
}
