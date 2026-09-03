use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssEmptyCellsQualificationOutcome, CssEmptyCellsUnsupportedReason, CssEmptyCellsValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Show,
    Hide,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssEmptyCellsQualificationOutcome {
    match expected {
        ExpectedOutcome::Show => {
            CssEmptyCellsQualificationOutcome::Qualified(CssEmptyCellsValue::Show)
        }
        ExpectedOutcome::Hide => {
            CssEmptyCellsQualificationOutcome::Qualified(CssEmptyCellsValue::Hide)
        }
        ExpectedOutcome::Invalid => {
            CssEmptyCellsQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssEmptyCellsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssEmptyCellsUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssEmptyCellsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssEmptyCellsUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .empty_cells_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_show_hide_grammar() {
    let result = qualify(
        1840,
        concat!(
            "a{empty-cells:show;}",
            "b{empty-cells:hide;}",
            "c{empty-cells:SHOW;}",
            "d{EMPTY-CELLS:Hide;}",
            r"e{empty-cells:sh\6f w;}",
            r"f{empty-cells:h\69 de;}",
            "g{empty-cells:auto;}",
            "h{empty-cells:show hide;}",
            "i{empty-cells:;}",
            "j{empty-cells:1;}",
            "k{empty-cells:1px;}",
            "l{empty-cells:\"hide\";}",
            "m{color:hide;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Show,
            ExpectedOutcome::Hide,
            ExpectedOutcome::Show,
            ExpectedOutcome::Hide,
            ExpectedOutcome::Show,
            ExpectedOutcome::Hide,
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
        1841,
        concat!(
            "a{empty-cells:/**/show/**/!important;}",
            "b{empty-cells:/**/hide/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Show, ExpectedOutcome::Hide]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        1842,
        concat!(
            "a{empty-cells:initial;}",
            "b{empty-cells:inherit;}",
            "c{empty-cells:unset;}",
            "d{empty-cells:revert;}",
            "e{empty-cells:revert-layer;}",
            "f{empty-cells:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        1843,
        concat!(
            "a{empty-cells:var(--cells);}",
            "b{empty-cells:env(cells);}",
            "c{empty-cells:attr(data-cells);}",
            "d{empty-cells:--cells();}",
            "e{empty-cells:first-valid(show,hide);}",
            "f{empty-cells:cycle(show,hide);}",
            "g{empty-cells:interpolate(0%,0:show,1:hide);}",
            "h{empty-cells:foo();}",
            "i{empty-cells:calc(1);}",
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
        1844,
        concat!(
            "a{empty-cells:show first-valid(hide);}",
            "b{empty-cells:first-valid(hide) show;}",
            "c{empty-cells:show foo();}",
            "d{empty-cells:foo() hide;}",
            "e{empty-cells:show var(--cells);}",
            "f{empty-cells:foo(var(--cells));}",
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
fn one_run_interleaves_empty_cells_with_every_accepted_leaf() {
    let result = qualify(
        1845,
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
            "u{empty-cells:hide;}",
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
    assert_eq!(result.empty_cells_observations().len(), 1);
    assert_eq!(result.empty_cells_observations()[0].occurrence_index(), 20);
    assert_expected(&result, &[ExpectedOutcome::Hide]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(1846, "a{empty-cells:show;}b{empty-cells:show;}");

    assert_expected(&result, &[ExpectedOutcome::Show, ExpectedOutcome::Show]);
    assert_eq!(result.empty_cells_observations()[0].occurrence_index(), 0);
    assert_eq!(result.empty_cells_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.empty_cells_observations()[0]
            .placement()
            .context_id(),
        result.empty_cells_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (1850, "@font-face{empty-cells:hide;}"),
        (1851, "@page{empty-cells:hide;}"),
        (1852, "@page{@top-left{empty-cells:hide;}}"),
        (1853, "@keyframes k{from{empty-cells:hide;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.empty_cells_observations().is_empty(),
            "nonordinary declaration context produced an empty-cells observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        1860,
        "a{empty-cells:show;empty-cells:hide;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Show]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{empty-cells:show;}",
        "b{empty-cells:inherit;}",
        "c{empty-cells:auto;}",
        "d{empty-cells:var(--cells);}",
    );
    let first = qualify(1870, css);
    let repeated = qualify(1870, css);
    let another_source = qualify(1871, css);

    assert_eq!(
        first.empty_cells_observations(),
        repeated.empty_cells_observations()
    );
    assert_eq!(
        first.empty_cells_observations(),
        another_source.empty_cells_observations()
    );
}
