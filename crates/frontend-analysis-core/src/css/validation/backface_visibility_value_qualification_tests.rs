use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBackfaceVisibilityQualificationOutcome, CssBackfaceVisibilityUnsupportedReason,
    CssBackfaceVisibilityValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Visible,
    Hidden,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssBackfaceVisibilityQualificationOutcome {
    match expected {
        ExpectedOutcome::Visible => CssBackfaceVisibilityQualificationOutcome::Qualified(
            CssBackfaceVisibilityValue::Visible,
        ),
        ExpectedOutcome::Hidden => {
            CssBackfaceVisibilityQualificationOutcome::Qualified(CssBackfaceVisibilityValue::Hidden)
        }
        ExpectedOutcome::Invalid => {
            CssBackfaceVisibilityQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssBackfaceVisibilityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBackfaceVisibilityUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssBackfaceVisibilityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBackfaceVisibilityUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .backface_visibility_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_visible_hidden_grammar() {
    let result = qualify(
        1700,
        concat!(
            "a{backface-visibility:visible;}",
            "b{backface-visibility:hidden;}",
            "c{backface-visibility:VISIBLE;}",
            "d{BACKFACE-VISIBILITY:Hidden;}",
            r"e{backface-visibility:\76 isible;}",
            r"f{backface-visibility:h\69 dden;}",
            "g{backface-visibility:auto;}",
            "h{backface-visibility:visible hidden;}",
            "i{backface-visibility:;}",
            "j{backface-visibility:1;}",
            "k{backface-visibility:1px;}",
            "l{backface-visibility:\"hidden\";}",
            "m{color:hidden;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Visible,
            ExpectedOutcome::Hidden,
            ExpectedOutcome::Visible,
            ExpectedOutcome::Hidden,
            ExpectedOutcome::Visible,
            ExpectedOutcome::Hidden,
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
        1701,
        concat!(
            "a{backface-visibility:/**/visible/**/!important;}",
            "b{backface-visibility:/**/hidden/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Visible, ExpectedOutcome::Hidden],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        1702,
        concat!(
            "a{backface-visibility:initial;}",
            "b{backface-visibility:inherit;}",
            "c{backface-visibility:unset;}",
            "d{backface-visibility:revert;}",
            "e{backface-visibility:revert-layer;}",
            "f{backface-visibility:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        1703,
        concat!(
            "a{backface-visibility:var(--mode);}",
            "b{backface-visibility:env(mode);}",
            "c{backface-visibility:attr(data-mode);}",
            "d{backface-visibility:--mode();}",
            "e{backface-visibility:first-valid(visible,hidden);}",
            "f{backface-visibility:cycle(visible,hidden);}",
            "g{backface-visibility:interpolate(0%,0:visible,1:hidden);}",
            "h{backface-visibility:foo();}",
            "i{backface-visibility:calc(1);}",
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
        1704,
        concat!(
            "a{backface-visibility:visible first-valid(hidden);}",
            "b{backface-visibility:first-valid(hidden) visible;}",
            "c{backface-visibility:visible foo();}",
            "d{backface-visibility:foo() hidden;}",
            "e{backface-visibility:visible var(--mode);}",
            "f{backface-visibility:foo(var(--mode));}",
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
fn one_run_interleaves_backface_visibility_with_every_accepted_leaf() {
    let result = qualify(
        1705,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:isolate;}",
            "d{order:1;}",
            "e{column-count:2;}",
            "f{flex-grow:1;}",
            "g{flex-shrink:1;}",
            "h{opacity:.5;}",
            "i{shape-image-threshold:.5;}",
            "j{shape-margin:1px;}",
            "k{line-height:1;}",
            "l{word-spacing:-1px;}",
            "m{text-underline-offset:-10%;}",
            "n{scroll-margin-top:-1px;}",
            "o{border-top-width:thin;}",
            "p{perspective:1px;}",
            "q{z-index:1;}",
            "r{scroll-snap-align:center;}",
            "s{backface-visibility:hidden;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
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
    assert_eq!(result.backface_visibility_observations().len(), 1);
    assert_eq!(
        result.backface_visibility_observations()[0].occurrence_index(),
        18
    );
    assert_expected(&result, &[ExpectedOutcome::Hidden]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        1706,
        "a{backface-visibility:visible;}b{backface-visibility:visible;}",
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Visible, ExpectedOutcome::Visible],
    );
    assert_eq!(
        result.backface_visibility_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.backface_visibility_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.backface_visibility_observations()[0]
            .placement()
            .context_id(),
        result.backface_visibility_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (1710, "@font-face{backface-visibility:hidden;}"),
        (1711, "@page{backface-visibility:hidden;}"),
        (1712, "@page{@top-left{backface-visibility:hidden;}}"),
        (1713, "@keyframes k{from{backface-visibility:hidden;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.backface_visibility_observations().is_empty(),
            "nonordinary declaration context produced a backface-visibility observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        1720,
        "a{backface-visibility:visible;backface-visibility:hidden;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Visible]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{backface-visibility:visible;}",
        "b{backface-visibility:inherit;}",
        "c{backface-visibility:auto;}",
        "d{backface-visibility:var(--mode);}",
    );
    let first = qualify(1730, css);
    let repeated = qualify(1730, css);
    let another_source = qualify(1731, css);

    assert_eq!(
        first.backface_visibility_observations(),
        repeated.backface_visibility_observations()
    );
    assert_eq!(
        first.backface_visibility_observations(),
        another_source.backface_visibility_observations()
    );
}
