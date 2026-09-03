use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontKerningQualificationOutcome, CssFontKerningUnsupportedReason, CssFontKerningValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Normal,
    None,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFontKerningQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssFontKerningQualificationOutcome::Qualified(CssFontKerningValue::Auto)
        }
        ExpectedOutcome::Normal => {
            CssFontKerningQualificationOutcome::Qualified(CssFontKerningValue::Normal)
        }
        ExpectedOutcome::None => {
            CssFontKerningQualificationOutcome::Qualified(CssFontKerningValue::None)
        }
        ExpectedOutcome::Invalid => {
            CssFontKerningQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFontKerningQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontKerningUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFontKerningQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontKerningUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .font_kerning_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_three_keyword_grammar() {
    let result = qualify(
        2100,
        concat!(
            "a{font-kerning:auto;}",
            "b{font-kerning:normal;}",
            "c{font-kerning:none;}",
            "d{font-kerning:AUTO;}",
            "e{FONT-KERNING:NoRmAl;}",
            r"f{font-kerning:\61 uto;}",
            r"g{font-kerning:n\6f rmal;}",
            "h{font-kerning:manual;}",
            "i{font-kerning:normal auto;}",
            "j{font-kerning:none, auto;}",
            "k{font-kerning:;}",
            "l{font-kerning:1;}",
            "m{font-kerning:1px;}",
            "n{font-kerning:\"normal\";}",
            "o{color:normal;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Normal,
            ExpectedOutcome::None,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Normal,
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
fn comments_and_priority_preserve_decoded_keyword_meaning() {
    let result = qualify(
        2101,
        concat!(
            "a{font-kerning:/**/auto/**/!important;}",
            "b{font-kerning:/**/none/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::None]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2102,
        concat!(
            "a{font-kerning:initial;}",
            "b{font-kerning:inherit;}",
            "c{font-kerning:unset;}",
            "d{font-kerning:revert;}",
            "e{font-kerning:revert-layer;}",
            "f{font-kerning:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2103,
        concat!(
            "a{font-kerning:var(--kerning);}",
            "b{font-kerning:env(kerning);}",
            "c{font-kerning:attr(data-kerning);}",
            "d{font-kerning:--kerning();}",
            "e{font-kerning:first-valid(auto,normal);}",
            "f{font-kerning:cycle(auto,none);}",
            "g{font-kerning:interpolate(0%,0:auto,1:none);}",
            "h{font-kerning:foo();}",
            "i{font-kerning:calc(1);}",
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
        2104,
        concat!(
            "a{font-kerning:auto first-valid(normal);}",
            "b{font-kerning:first-valid(normal) auto;}",
            "c{font-kerning:none foo();}",
            "d{font-kerning:foo() normal;}",
            "e{font-kerning:normal var(--kerning);}",
            "f{font-kerning:foo(var(--kerning));}",
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
fn one_run_interleaves_font_kerning_with_every_accepted_leaf() {
    let result = qualify(
        2105,
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
            "v{text-decoration-style:wavy;}",
            "w{table-layout:fixed;}",
            "x{border-collapse:collapse;}",
            "y{box-decoration-break:clone;}",
            "z{font-kerning:normal;}",
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
    assert_eq!(result.text_decoration_style_observations().len(), 1);
    assert_eq!(result.table_layout_observations().len(), 1);
    assert_eq!(result.border_collapse_observations().len(), 1);
    assert_eq!(result.box_decoration_break_observations().len(), 1);
    assert_eq!(result.font_kerning_observations().len(), 1);
    assert_eq!(result.font_kerning_observations()[0].occurrence_index(), 25);
    assert_expected(&result, &[ExpectedOutcome::Normal]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(2106, "a{font-kerning:normal;}b{font-kerning:normal;}");

    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Normal]);
    assert_eq!(result.font_kerning_observations()[0].occurrence_index(), 0);
    assert_eq!(result.font_kerning_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.font_kerning_observations()[0].placement().context_id(),
        result.font_kerning_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2110, "@font-face{font-kerning:normal;}"),
        (2111, "@page{font-kerning:normal;}"),
        (2112, "@page{@top-left{font-kerning:normal;}}"),
        (2113, "@keyframes k{from{font-kerning:normal;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_kerning_observations().is_empty(),
            "nonordinary declaration context produced a font-kerning observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2120,
        "a{font-kerning:auto;font-kerning:none;}",
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
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{font-kerning:auto;}",
        "b{font-kerning:inherit;}",
        "c{font-kerning:manual;}",
        "d{font-kerning:var(--kerning);}",
    );
    let first = qualify(2130, css);
    let repeated = qualify(2130, css);
    let another_source = qualify(2131, css);

    assert_eq!(
        first.font_kerning_observations(),
        repeated.font_kerning_observations()
    );
    assert_eq!(
        first.font_kerning_observations(),
        another_source.font_kerning_observations()
    );
}
