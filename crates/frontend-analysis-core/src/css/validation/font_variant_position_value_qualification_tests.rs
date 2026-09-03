use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssFontVariantPositionQualificationOutcome, CssFontVariantPositionUnsupportedReason,
    CssFontVariantPositionValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Sub,
    Super,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssFontVariantPositionQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => CssFontVariantPositionQualificationOutcome::Qualified(
            CssFontVariantPositionValue::Normal,
        ),
        ExpectedOutcome::Sub => {
            CssFontVariantPositionQualificationOutcome::Qualified(CssFontVariantPositionValue::Sub)
        }
        ExpectedOutcome::Super => CssFontVariantPositionQualificationOutcome::Qualified(
            CssFontVariantPositionValue::Super,
        ),
        ExpectedOutcome::Invalid => {
            CssFontVariantPositionQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssFontVariantPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantPositionUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssFontVariantPositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssFontVariantPositionUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .font_variant_position_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_three_keyword_grammar() {
    let result = qualify(
        2140,
        concat!(
            "a{font-variant-position:normal;}",
            "b{font-variant-position:sub;}",
            "c{font-variant-position:super;}",
            "d{font-variant-position:NORMAL;}",
            "e{FONT-VARIANT-POSITION:SuB;}",
            r"f{font-variant-position:\6e ormal;}",
            r"g{font-variant-position:s\75 per;}",
            "h{font-variant-position:auto;}",
            "i{font-variant-position:super sub;}",
            "j{font-variant-position:sub, super;}",
            "k{font-variant-position:;}",
            "l{font-variant-position:1;}",
            "m{font-variant-position:1px;}",
            "n{font-variant-position:\"super\";}",
            "o{color:super;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Sub,
            ExpectedOutcome::Super,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Sub,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Super,
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
        2141,
        concat!(
            "a{font-variant-position:/**/normal/**/!important;}",
            "b{font-variant-position:/**/super/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Super]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        2142,
        concat!(
            "a{font-variant-position:initial;}",
            "b{font-variant-position:inherit;}",
            "c{font-variant-position:unset;}",
            "d{font-variant-position:revert;}",
            "e{font-variant-position:revert-layer;}",
            "f{font-variant-position:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        2143,
        concat!(
            "a{font-variant-position:var(--position);}",
            "b{font-variant-position:env(position);}",
            "c{font-variant-position:attr(data-position);}",
            "d{font-variant-position:--position();}",
            "e{font-variant-position:first-valid(normal,sub);}",
            "f{font-variant-position:cycle(normal,super);}",
            "g{font-variant-position:interpolate(0%,0:normal,1:super);}",
            "h{font-variant-position:foo();}",
            "i{font-variant-position:calc(1);}",
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
        2144,
        concat!(
            "a{font-variant-position:normal first-valid(sub);}",
            "b{font-variant-position:first-valid(sub) normal;}",
            "c{font-variant-position:super foo();}",
            "d{font-variant-position:foo() sub;}",
            "e{font-variant-position:sub var(--position);}",
            "f{font-variant-position:foo(var(--position));}",
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
fn one_run_interleaves_font_variant_position_with_every_accepted_leaf() {
    let result = qualify(
        2145,
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
    assert_eq!(
        result.font_variant_position_observations()[0].occurrence_index(),
        26
    );
    assert_expected(&result, &[ExpectedOutcome::Super]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        2146,
        "a{font-variant-position:normal;}b{font-variant-position:normal;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Normal]);
    assert_eq!(
        result.font_variant_position_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.font_variant_position_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.font_variant_position_observations()[0]
            .placement()
            .context_id(),
        result.font_variant_position_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (2150, "@font-face{font-variant-position:normal;}"),
        (2151, "@page{font-variant-position:normal;}"),
        (2152, "@page{@top-left{font-variant-position:normal;}}"),
        (2153, "@keyframes k{from{font-variant-position:normal;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.font_variant_position_observations().is_empty(),
            "nonordinary declaration context produced a font-variant-position observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        2160,
        "a{font-variant-position:normal;font-variant-position:super;}",
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
        "a{font-variant-position:normal;}",
        "b{font-variant-position:inherit;}",
        "c{font-variant-position:auto;}",
        "d{font-variant-position:var(--position);}",
    );
    let first = qualify(2170, css);
    let repeated = qualify(2170, css);
    let another_source = qualify(2171, css);

    assert_eq!(
        first.font_variant_position_observations(),
        repeated.font_variant_position_observations()
    );
    assert_eq!(
        first.font_variant_position_observations(),
        another_source.font_variant_position_observations()
    );
}
