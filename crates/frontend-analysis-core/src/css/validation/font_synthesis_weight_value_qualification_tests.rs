use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontSynthesisWeightQualificationOutcome, CssFontSynthesisWeightUnsupportedReason,
    CssFontSynthesisWeightValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFontSynthesisWeightQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => CssFontSynthesisWeightQualificationOutcome::Qualified(
            CssFontSynthesisWeightValue::Auto,
        ),
        ExpectedOutcome::None => CssFontSynthesisWeightQualificationOutcome::Qualified(
            CssFontSynthesisWeightValue::None,
        ),
        ExpectedOutcome::Invalid => {
            CssFontSynthesisWeightQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFontSynthesisWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisWeightUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFontSynthesisWeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisWeightUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .font_synthesis_weight_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_auto_none_grammar() {
    let result = qualify(
        2180,
        concat!(
            "a{font-synthesis-weight:auto;}",
            "b{font-synthesis-weight:none;}",
            "c{font-synthesis-weight:AUTO;}",
            "d{FONT-SYNTHESIS-WEIGHT:NoNe;}",
            r"e{font-synthesis-weight:\61 uto;}",
            r"f{font-synthesis-weight:n\6f ne;}",
            "g{font-synthesis-weight:normal;}",
            "h{font-synthesis-weight:auto none;}",
            "i{font-synthesis-weight:none auto;}",
            "j{font-synthesis-weight:auto,none;}",
            "k{font-synthesis-weight:;}",
            "l{font-synthesis-weight:1;}",
            "m{font-synthesis-weight:1px;}",
            "n{font-synthesis-weight:\"auto\";}",
            "o{color:auto;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::None,
            ExpectedOutcome::Auto,
            ExpectedOutcome::None,
            ExpectedOutcome::Auto,
            ExpectedOutcome::None,
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
        2181,
        concat!(
            "a{font-synthesis-weight:/**/auto/**/!important;}",
            "b{font-synthesis-weight:/**/none/**/!important;}",
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
        2182,
        concat!(
            "a{font-synthesis-weight:initial;}",
            "b{font-synthesis-weight:inherit;}",
            "c{font-synthesis-weight:unset;}",
            "d{font-synthesis-weight:revert;}",
            "e{font-synthesis-weight:revert-layer;}",
            "f{font-synthesis-weight:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2183,
        concat!(
            "a{font-synthesis-weight:var(--weight);}",
            "b{font-synthesis-weight:env(weight);}",
            "c{font-synthesis-weight:attr(data-weight);}",
            "d{font-synthesis-weight:--weight();}",
            "e{font-synthesis-weight:first-valid(auto,none);}",
            "f{font-synthesis-weight:cycle(auto,none);}",
            "g{font-synthesis-weight:interpolate(0%,0:auto,1:none);}",
            "h{font-synthesis-weight:foo();}",
            "i{font-synthesis-weight:calc(1);}",
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
        2184,
        concat!(
            "a{font-synthesis-weight:auto first-valid(none);}",
            "b{font-synthesis-weight:first-valid(none) auto;}",
            "c{font-synthesis-weight:none foo();}",
            "d{font-synthesis-weight:foo() auto;}",
            "e{font-synthesis-weight:auto var(--weight);}",
            "f{font-synthesis-weight:foo(var(--weight));}",
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
fn one_run_interleaves_font_synthesis_weight_with_every_accepted_leaf() {
    let result = qualify(
        2185,
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
    assert_eq!(
        result.font_synthesis_weight_observations()[0].occurrence_index(),
        27
    );
    assert_expected(&result, &[ExpectedOutcome::None]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2186,
        "a{font-synthesis-weight:auto;}b{font-synthesis-weight:auto;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_eq!(
        result.font_synthesis_weight_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.font_synthesis_weight_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.font_synthesis_weight_observations()[0]
            .placement()
            .context_id(),
        result.font_synthesis_weight_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2190, "@font-face{font-synthesis-weight:auto;}"),
        (2191, "@page{font-synthesis-weight:auto;}"),
        (2192, "@page{@top-left{font-synthesis-weight:auto;}}"),
        (2193, "@keyframes k{from{font-synthesis-weight:auto;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_synthesis_weight_observations().is_empty(),
            "nonordinary declaration context produced a font-synthesis-weight observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2200,
        "a{font-synthesis-weight:auto;font-synthesis-weight:none;}",
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
        "a{font-synthesis-weight:auto;}",
        "b{font-synthesis-weight:inherit;}",
        "c{font-synthesis-weight:normal;}",
        "d{font-synthesis-weight:var(--weight);}",
    );
    let first = qualify(2210, css);
    let repeated = qualify(2210, css);
    let another_source = qualify(2211, css);

    assert_eq!(
        first.font_synthesis_weight_observations(),
        repeated.font_synthesis_weight_observations()
    );
    assert_eq!(
        first.font_synthesis_weight_observations(),
        another_source.font_synthesis_weight_observations()
    );
}
