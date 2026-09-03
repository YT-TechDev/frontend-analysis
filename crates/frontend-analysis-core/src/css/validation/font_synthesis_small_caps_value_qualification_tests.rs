use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontSynthesisSmallCapsQualificationOutcome, CssFontSynthesisSmallCapsUnsupportedReason,
    CssFontSynthesisSmallCapsValue, CssValueQualificationRunResult, run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFontSynthesisSmallCapsQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => CssFontSynthesisSmallCapsQualificationOutcome::Qualified(
            CssFontSynthesisSmallCapsValue::Auto,
        ),
        ExpectedOutcome::None => CssFontSynthesisSmallCapsQualificationOutcome::Qualified(
            CssFontSynthesisSmallCapsValue::None,
        ),
        ExpectedOutcome::Invalid => {
            CssFontSynthesisSmallCapsQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFontSynthesisSmallCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisSmallCapsUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFontSynthesisSmallCapsQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontSynthesisSmallCapsUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .font_synthesis_small_caps_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_auto_none_grammar() {
    let result = qualify(
        2220,
        concat!(
            "a{font-synthesis-small-caps:auto;}",
            "b{font-synthesis-small-caps:none;}",
            "c{font-synthesis-small-caps:AUTO;}",
            "d{FONT-SYNTHESIS-SMALL-CAPS:NoNe;}",
            r"e{font-synthesis-small-caps:\61 uto;}",
            r"f{font-synthesis-small-caps:n\6f ne;}",
            "g{font-synthesis-small-caps:normal;}",
            "h{font-synthesis-small-caps:auto none;}",
            "i{font-synthesis-small-caps:none auto;}",
            "j{font-synthesis-small-caps:auto,none;}",
            "k{font-synthesis-small-caps:;}",
            "l{font-synthesis-small-caps:1;}",
            "m{font-synthesis-small-caps:1px;}",
            "n{font-synthesis-small-caps:\"auto\";}",
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
        2221,
        concat!(
            "a{font-synthesis-small-caps:/**/auto/**/!important;}",
            "b{font-synthesis-small-caps:/**/none/**/!important;}",
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
        2222,
        concat!(
            "a{font-synthesis-small-caps:initial;}",
            "b{font-synthesis-small-caps:inherit;}",
            "c{font-synthesis-small-caps:unset;}",
            "d{font-synthesis-small-caps:revert;}",
            "e{font-synthesis-small-caps:revert-layer;}",
            "f{font-synthesis-small-caps:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2223,
        concat!(
            "a{font-synthesis-small-caps:var(--caps);}",
            "b{font-synthesis-small-caps:env(caps);}",
            "c{font-synthesis-small-caps:attr(data-caps);}",
            "d{font-synthesis-small-caps:--caps();}",
            "e{font-synthesis-small-caps:first-valid(auto,none);}",
            "f{font-synthesis-small-caps:cycle(auto,none);}",
            "g{font-synthesis-small-caps:interpolate(0%,0:auto,1:none);}",
            "h{font-synthesis-small-caps:foo();}",
            "i{font-synthesis-small-caps:calc(1);}",
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
        2224,
        concat!(
            "a{font-synthesis-small-caps:auto first-valid(none);}",
            "b{font-synthesis-small-caps:first-valid(none) auto;}",
            "c{font-synthesis-small-caps:none foo();}",
            "d{font-synthesis-small-caps:foo() auto;}",
            "e{font-synthesis-small-caps:auto var(--caps);}",
            "f{font-synthesis-small-caps:foo(var(--caps));}",
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
fn one_run_interleaves_font_synthesis_small_caps_with_every_accepted_leaf() {
    let result = qualify(
        2225,
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
    assert_eq!(
        result.font_synthesis_small_caps_observations()[0].occurrence_index(),
        28
    );
    assert_expected(&result, &[ExpectedOutcome::None]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2226,
        "a{font-synthesis-small-caps:auto;}b{font-synthesis-small-caps:auto;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_eq!(
        result.font_synthesis_small_caps_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.font_synthesis_small_caps_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.font_synthesis_small_caps_observations()[0]
            .placement()
            .context_id(),
        result.font_synthesis_small_caps_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2230, "@font-face{font-synthesis-small-caps:auto;}"),
        (2231, "@page{font-synthesis-small-caps:auto;}"),
        (2232, "@page{@top-left{font-synthesis-small-caps:auto;}}"),
        (2233, "@keyframes k{from{font-synthesis-small-caps:auto;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_synthesis_small_caps_observations().is_empty(),
            "nonordinary declaration context produced a font-synthesis-small-caps observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2240,
        "a{font-synthesis-small-caps:auto;font-synthesis-small-caps:none;}",
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
        "a{font-synthesis-small-caps:auto;}",
        "b{font-synthesis-small-caps:inherit;}",
        "c{font-synthesis-small-caps:normal;}",
        "d{font-synthesis-small-caps:var(--caps);}",
    );
    let first = qualify(2250, css);
    let repeated = qualify(2250, css);
    let another_source = qualify(2251, css);

    assert_eq!(
        first.font_synthesis_small_caps_observations(),
        repeated.font_synthesis_small_caps_observations()
    );
    assert_eq!(
        first.font_synthesis_small_caps_observations(),
        another_source.font_synthesis_small_caps_observations()
    );
}
