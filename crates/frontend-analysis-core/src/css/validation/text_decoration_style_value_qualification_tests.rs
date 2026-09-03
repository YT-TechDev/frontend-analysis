use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextDecorationStyleQualificationOutcome, CssTextDecorationStyleUnsupportedReason,
    CssTextDecorationStyleValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssTextDecorationStyleQualificationOutcome {
    match expected {
        ExpectedOutcome::Solid => CssTextDecorationStyleQualificationOutcome::Qualified(
            CssTextDecorationStyleValue::Solid,
        ),
        ExpectedOutcome::Double => CssTextDecorationStyleQualificationOutcome::Qualified(
            CssTextDecorationStyleValue::Double,
        ),
        ExpectedOutcome::Dotted => CssTextDecorationStyleQualificationOutcome::Qualified(
            CssTextDecorationStyleValue::Dotted,
        ),
        ExpectedOutcome::Dashed => CssTextDecorationStyleQualificationOutcome::Qualified(
            CssTextDecorationStyleValue::Dashed,
        ),
        ExpectedOutcome::Wavy => {
            CssTextDecorationStyleQualificationOutcome::Qualified(CssTextDecorationStyleValue::Wavy)
        }
        ExpectedOutcome::Invalid => {
            CssTextDecorationStyleQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssTextDecorationStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextDecorationStyleUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssTextDecorationStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextDecorationStyleUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .text_decoration_style_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_direct_keyword_boundary_matches_five_keyword_grammar() {
    let result = qualify(
        1880,
        concat!(
            "a{text-decoration-style:solid;}",
            "b{text-decoration-style:double;}",
            "c{text-decoration-style:dotted;}",
            "d{text-decoration-style:dashed;}",
            "e{text-decoration-style:wavy;}",
            "f{text-decoration-style:SOLID;}",
            "g{TEXT-DECORATION-STYLE:WaVy;}",
            r"h{text-decoration-style:s\6f lid;}",
            r"i{text-decoration-style:d\6f uble;}",
            "j{text-decoration-style:groove;}",
            "k{text-decoration-style:solid wavy;}",
            "l{text-decoration-style:;}",
            "m{text-decoration-style:1;}",
            "n{text-decoration-style:1px;}",
            "o{text-decoration-style:\"solid\";}",
            "p{color:wavy;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Solid,
            ExpectedOutcome::Double,
            ExpectedOutcome::Dotted,
            ExpectedOutcome::Dashed,
            ExpectedOutcome::Wavy,
            ExpectedOutcome::Solid,
            ExpectedOutcome::Wavy,
            ExpectedOutcome::Solid,
            ExpectedOutcome::Double,
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
        1881,
        concat!(
            "a{text-decoration-style:/**/dotted/**/!important;}",
            "b{text-decoration-style:/**/dashed/**/!important;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Dotted, ExpectedOutcome::Dashed]);
    for occurrence in result.upstream_parser_result().occurrences() {
        assert!(occurrence.priority().is_some());
    }
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        1882,
        concat!(
            "a{text-decoration-style:initial;}",
            "b{text-decoration-style:inherit;}",
            "c{text-decoration-style:unset;}",
            "d{text-decoration-style:revert;}",
            "e{text-decoration-style:revert-layer;}",
            "f{text-decoration-style:revert-rule;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn deferred_and_whole_value_functions_fail_open_but_ordinary_functions_are_invalid() {
    let result = qualify(
        1883,
        concat!(
            "a{text-decoration-style:var(--style);}",
            "b{text-decoration-style:env(style);}",
            "c{text-decoration-style:attr(data-style);}",
            "d{text-decoration-style:--style();}",
            "e{text-decoration-style:first-valid(solid,wavy);}",
            "f{text-decoration-style:cycle(solid,wavy);}",
            "g{text-decoration-style:interpolate(0%,0:solid,1:wavy);}",
            "h{text-decoration-style:foo();}",
            "i{text-decoration-style:calc(1);}",
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
        1884,
        concat!(
            "a{text-decoration-style:solid first-valid(wavy);}",
            "b{text-decoration-style:first-valid(wavy) solid;}",
            "c{text-decoration-style:dotted foo();}",
            "d{text-decoration-style:foo() dashed;}",
            "e{text-decoration-style:wavy var(--style);}",
            "f{text-decoration-style:foo(var(--style));}",
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
fn one_run_interleaves_text_decoration_style_with_every_accepted_leaf() {
    let result = qualify(
        1885,
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
    assert_eq!(
        result.text_decoration_style_observations()[0].occurrence_index(),
        21
    );
    assert_expected(&result, &[ExpectedOutcome::Wavy]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        1886,
        "a{text-decoration-style:solid;}b{text-decoration-style:solid;}",
    );

    assert_expected(&result, &[ExpectedOutcome::Solid, ExpectedOutcome::Solid]);
    assert_eq!(
        result.text_decoration_style_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.text_decoration_style_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.text_decoration_style_observations()[0]
            .placement()
            .context_id(),
        result.text_decoration_style_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (1890, "@font-face{text-decoration-style:wavy;}"),
        (1891, "@page{text-decoration-style:wavy;}"),
        (1892, "@page{@top-left{text-decoration-style:wavy;}}"),
        (1893, "@keyframes k{from{text-decoration-style:wavy;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_decoration_style_observations().is_empty(),
            "nonordinary declaration context produced a text-decoration-style observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        1900,
        "a{text-decoration-style:solid;text-decoration-style:wavy;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Solid]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{text-decoration-style:solid;}",
        "b{text-decoration-style:inherit;}",
        "c{text-decoration-style:groove;}",
        "d{text-decoration-style:var(--style);}",
    );
    let first = qualify(1910, css);
    let repeated = qualify(1910, css);
    let another_source = qualify(1911, css);

    assert_eq!(
        first.text_decoration_style_observations(),
        repeated.text_decoration_style_observations()
    );
    assert_eq!(
        first.text_decoration_style_observations(),
        another_source.text_decoration_style_observations()
    );
}
