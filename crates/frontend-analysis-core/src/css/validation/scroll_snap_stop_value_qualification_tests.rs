use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssScrollSnapStopQualificationOutcome, CssScrollSnapStopUnsupportedReason,
    CssScrollSnapStopValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Always,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssScrollSnapStopQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssScrollSnapStopQualificationOutcome::Qualified(CssScrollSnapStopValue::Normal)
        }
        ExpectedOutcome::Always => {
            CssScrollSnapStopQualificationOutcome::Qualified(CssScrollSnapStopValue::Always)
        }
        ExpectedOutcome::Invalid => {
            CssScrollSnapStopQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssScrollSnapStopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollSnapStopUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssScrollSnapStopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollSnapStopUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .scroll_snap_stop_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_normal_always_grammar() {
    let result = qualify(
        1800,
        concat!(
            "a{scroll-snap-stop:normal;}",
            "b{scroll-snap-stop:always;}",
            "c{scroll-snap-stop:NORMAL;}",
            "d{SCROLL-SNAP-STOP:Always;}",
            r"e{scroll-snap-stop:n\6f rmal;}",
            r"f{scroll-snap-stop:\61 lways;}",
            "g{scroll-snap-stop:auto;}",
            "h{scroll-snap-stop:normal always;}",
            "i{scroll-snap-stop:;}",
            "j{scroll-snap-stop:1;}",
            "k{scroll-snap-stop:1px;}",
            "l{scroll-snap-stop:\"always\";}",
            "m{color:always;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Always,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Always,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Always,
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
fn comments_and_priority_preserve_decoded_keyword_meaning() {
    let result = qualify(
        1801,
        concat!(
            "a{scroll-snap-stop:/**/normal/**/!important;}",
            "b{scroll-snap-stop:/**/always/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Always]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        1802,
        concat!(
            "a{scroll-snap-stop:initial;}",
            "b{scroll-snap-stop:inherit;}",
            "c{scroll-snap-stop:unset;}",
            "d{scroll-snap-stop:revert;}",
            "e{scroll-snap-stop:revert-layer;}",
            "f{scroll-snap-stop:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        1803,
        concat!(
            "a{scroll-snap-stop:var(--stop);}",
            "b{scroll-snap-stop:env(stop);}",
            "c{scroll-snap-stop:attr(data-stop);}",
            "d{scroll-snap-stop:--stop();}",
            "e{scroll-snap-stop:first-valid(normal,always);}",
            "f{scroll-snap-stop:cycle(normal,always);}",
            "g{scroll-snap-stop:interpolate(0%,0:normal,1:always);}",
            "h{scroll-snap-stop:foo();}",
            "i{scroll-snap-stop:calc(1);}",
        ),
    );

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
fn function_placement_preserves_keyword_leaf_fail_open_boundary() {
    let result = qualify(
        1804,
        concat!(
            "a{scroll-snap-stop:normal first-valid(always);}",
            "b{scroll-snap-stop:first-valid(always) normal;}",
            "c{scroll-snap-stop:normal foo();}",
            "d{scroll-snap-stop:foo() always;}",
            "e{scroll-snap-stop:normal var(--stop);}",
            "f{scroll-snap-stop:foo(var(--stop));}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
        ],
    );
}

#[test]
fn one_run_interleaves_scroll_snap_stop_with_every_accepted_leaf() {
    let result = qualify(
        1805,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:isolate;}",
            "d{backface-visibility:hidden;}",
            "e{order:1;}",
            "f{column-count:2;}",
            "g{flex-grow:1;}",
            "h{flex-shrink:1;}",
            "i{opacity:.5;}",
            "j{shape-image-threshold:.5;}",
            "k{shape-margin:1px;}",
            "l{line-height:1;}",
            "m{word-spacing:-1px;}",
            "n{text-underline-offset:-10%;}",
            "o{scroll-margin-top:-1px;}",
            "p{border-top-width:thin;}",
            "q{perspective:1px;}",
            "r{z-index:1;}",
            "s{scroll-snap-align:center;}",
            "t{scroll-snap-stop:always;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.backface_visibility_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
    assert_eq!(result.column_count_observations().len(), 1);
    assert_eq!(result.flex_grow_observations().len(), 1);
    assert_eq!(result.flex_shrink_observations().len(), 1);
    assert_eq!(result.opacity_observations().len(), 1);
    assert_eq!(result.shape_image_threshold_observations().len(), 1);
    assert_eq!(result.shape_margin_observations().len(), 1);
    assert_eq!(result.line_height_observations().len(), 1);
    assert_eq!(result.word_spacing_observations().len(), 1);
    assert_eq!(result.text_underline_offset_observations().len(), 1);
    assert_eq!(result.scroll_margin_top_observations().len(), 1);
    assert_eq!(result.border_top_width_observations().len(), 1);
    assert_eq!(result.perspective_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.scroll_snap_align_observations().len(), 1);
    assert_eq!(result.scroll_snap_stop_observations().len(), 1);
    assert_eq!(
        result.scroll_snap_stop_observations()[0].occurrence_index(),
        19
    );
    assert_expected(&result, &[ExpectedOutcome::Always]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        1806,
        "a{scroll-snap-stop:normal;}b{scroll-snap-stop:normal;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Normal]);
    assert_eq!(
        result.scroll_snap_stop_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.scroll_snap_stop_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.scroll_snap_stop_observations()[0]
            .placement()
            .context_id(),
        result.scroll_snap_stop_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (1810, "@font-face{scroll-snap-stop:always;}"),
        (1811, "@page{scroll-snap-stop:always;}"),
        (1812, "@page{@top-left{scroll-snap-stop:always;}}"),
        (1813, "@keyframes k{from{scroll-snap-stop:always;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.scroll_snap_stop_observations().is_empty(),
            "nonordinary declaration context produced a scroll-snap-stop observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        1820,
        "a{scroll-snap-stop:normal;scroll-snap-stop:always;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Normal]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{scroll-snap-stop:normal;}",
        "b{scroll-snap-stop:inherit;}",
        "c{scroll-snap-stop:auto;}",
        "d{scroll-snap-stop:var(--stop);}",
    );
    let first = qualify(1830, css);
    let repeated = qualify(1830, css);
    let another_source = qualify(1831, css);

    assert_eq!(
        first.scroll_snap_stop_observations(),
        repeated.scroll_snap_stop_observations()
    );
    assert_eq!(
        first.scroll_snap_stop_observations(),
        another_source.scroll_snap_stop_observations()
    );
}
