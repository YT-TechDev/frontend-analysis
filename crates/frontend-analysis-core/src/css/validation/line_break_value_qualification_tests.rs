use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssLineBreakQualificationOutcome, CssLineBreakUnsupportedReason, CssLineBreakValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Loose,
    Normal,
    Strict,
    Anywhere,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssLineBreakQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Auto)
        }
        ExpectedOutcome::Loose => {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Loose)
        }
        ExpectedOutcome::Normal => {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Normal)
        }
        ExpectedOutcome::Strict => {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Strict)
        }
        ExpectedOutcome::Anywhere => {
            CssLineBreakQualificationOutcome::Qualified(CssLineBreakValue::Anywhere)
        }
        ExpectedOutcome::Invalid => {
            CssLineBreakQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssLineBreakQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineBreakUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssLineBreakQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineBreakUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .line_break_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_full_line_break_grammar() {
    let result = qualify(
        2500,
        concat!(
            "a{line-break:auto;}",
            "b{line-break:loose;}",
            "c{line-break:normal;}",
            "d{line-break:strict;}",
            "e{line-break:anywhere;}",
            "f{LINE-BREAK:StRiCt;}",
            r"g{line-break:\6c oose;}",
            r"h{line-break:\61 nywhere;}",
            "i{line-break:none;}",
            "j{line-break:auto loose;}",
            "k{line-break:strict normal;}",
            "l{line-break:anywhere anywhere;}",
            "m{line-break:after-white-space;}",
            "n{line-break:auto,strict;}",
            "o{line-break:;}",
            "p{line-break:1;}",
            "q{line-break:1px;}",
            "r{line-break:\"strict\";}",
            "s{color:strict;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Loose,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Strict,
            ExpectedOutcome::Anywhere,
            ExpectedOutcome::Strict,
            ExpectedOutcome::Loose,
            ExpectedOutcome::Anywhere,
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
fn comments_and_priority_preserve_decoded_keyword_meaning() {
    let result = qualify(
        2501,
        concat!(
            "a{line-break:/**/loose/**/!important;}",
            "b{line-break:/**/strict/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Loose, ExpectedOutcome::Strict],
    );
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2502,
        concat!(
            "a{line-break:initial;}",
            "b{line-break:inherit;}",
            "c{line-break:unset;}",
            "d{line-break:revert;}",
            "e{line-break:revert-layer;}",
            "f{line-break:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2503,
        concat!(
            "a{line-break:var(--line-break);}",
            "b{line-break:env(line-break);}",
            "c{line-break:attr(data-line-break);}",
            "d{line-break:--line-break();}",
            "e{line-break:first-valid(strict,normal);}",
            "f{line-break:cycle(loose,strict);}",
            "g{line-break:interpolate(0%,0:auto,1:anywhere);}",
            "h{line-break:foo();}",
            "i{line-break:calc(1);}",
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
        2504,
        concat!(
            "a{line-break:auto first-valid(strict);}",
            "b{line-break:first-valid(strict) auto;}",
            "c{line-break:loose foo();}",
            "d{line-break:foo() anywhere;}",
            "e{line-break:normal var(--line-break);}",
            "f{line-break:foo(var(--line-break));}",
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
fn one_run_interleaves_line_break_with_every_accepted_leaf() {
    let result = qualify(
        2505,
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
            "aa{font-variant-position:super;}",
            "ab{font-synthesis-weight:none;}",
            "ac{font-synthesis-small-caps:none;}",
            "ad{font-synthesis-position:none;}",
            "ae{font-variant-emoji:emoji;}",
            "af{font-variant-caps:unicase;}",
            "ag{line-break:anywhere;}",
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
    assert_eq!(result.font_variant_position_observations().len(), 1);
    assert_eq!(result.font_synthesis_weight_observations().len(), 1);
    assert_eq!(result.font_synthesis_small_caps_observations().len(), 1);
    assert_eq!(result.font_synthesis_position_observations().len(), 1);
    assert_eq!(result.font_variant_emoji_observations().len(), 1);
    assert_eq!(result.font_variant_caps_observations().len(), 1);
    assert_eq!(result.line_break_observations().len(), 1);
    assert_eq!(result.line_break_observations()[0].occurrence_index(), 32);
    assert_expected(&result, &[ExpectedOutcome::Anywhere]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2506,
        "a{line-break:strict;}b{line-break:strict;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Strict, ExpectedOutcome::Strict]);
    assert_eq!(result.line_break_observations()[0].occurrence_index(), 0);
    assert_eq!(result.line_break_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.line_break_observations()[0].placement().context_id(),
        result.line_break_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2510, "@font-face{line-break:strict;}"),
        (2511, "@page{line-break:strict;}"),
        (2512, "@page{@top-left{line-break:strict;}}"),
        (2513, "@keyframes k{from{line-break:strict;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.line_break_observations().is_empty(),
            "nonordinary declaration context produced a line-break observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2520,
        "a{line-break:auto;line-break:strict;}",
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
        "a{line-break:auto;}",
        "b{line-break:inherit;}",
        "c{line-break:none;}",
        "d{line-break:var(--line-break);}",
    );
    let first = qualify(2530, css);
    let repeated = qualify(2530, css);
    let another_source = qualify(2531, css);

    assert_eq!(
        first.line_break_observations(),
        repeated.line_break_observations()
    );
    assert_eq!(
        first.line_break_observations(),
        another_source.line_break_observations()
    );
}
