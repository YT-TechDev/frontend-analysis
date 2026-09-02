use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssScrollMarginTopQualificationOutcome, CssScrollMarginTopUnsupportedReason,
    CssScrollMarginTopValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    DirectLength,
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferred,
    UnsupportedWholeValue,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssScrollMarginTopQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectLength => CssScrollMarginTopQualificationOutcome::Qualified(
            CssScrollMarginTopValue::DirectLengthLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssScrollMarginTopQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssScrollMarginTopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollMarginTopUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssScrollMarginTopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollMarginTopUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssScrollMarginTopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollMarginTopUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssScrollMarginTopQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssScrollMarginTopUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .scroll_margin_top_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_unitless_zero_is_length_but_nonzero_numbers_and_percentages_are_invalid() {
    let result = qualify(
        1500,
        concat!(
            "a{scroll-margin-top:0;}",
            "b{scroll-margin-top:+0;}",
            "c{scroll-margin-top:-0;}",
            "d{scroll-margin-top:.0;}",
            "e{scroll-margin-top:-.0;}",
            "f{scroll-margin-top:0.0;}",
            "g{scroll-margin-top:-0.0;}",
            "h{scroll-margin-top:0e100;}",
            "i{scroll-margin-top:-0e100;}",
            "j{scroll-margin-top:1;}",
            "k{scroll-margin-top:-1;}",
            "l{scroll-margin-top:.5;}",
            "m{scroll-margin-top:10e2;}",
            "n{scroll-margin-top:0%;}",
            "o{scroll-margin-top:-0%;}",
            "p{scroll-margin-top:20%;}",
            "q{scroll-margin-top:-30%;}",
            "r{scroll-margin-top:.5%;}",
            "s{scroll-margin-top:-1e-999%;}",
            "t{scroll-margin-top:auto;}",
            "u{scroll-margin-top:none;}",
            "v{scroll-margin-top:\"1px\";}",
            "w{scroll-margin-top:;}",
            "x{color:1px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
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
fn handwritten_current_css_length_unit_inventory_is_qualified_case_insensitively() {
    let units = [
        "cm", "mm", "q", "in", "pt", "pc", "px", "em", "rem", "ex", "rex", "cap", "rcap", "ch",
        "rch", "ic", "ric", "lh", "rlh", "vw", "vh", "vi", "vb", "vmin", "vmax", "svw", "svh",
        "svi", "svb", "svmin", "svmax", "lvw", "lvh", "lvi", "lvb", "lvmin", "lvmax", "dvw", "dvh",
        "dvi", "dvb", "dvmin", "dvmax", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax",
    ];

    let mut css = String::new();
    for (index, unit) in units.iter().enumerate() {
        css.push_str(&format!(".u{index}{{scroll-margin-top:1{unit};}}"));
    }
    css.push_str(".upper{scroll-margin-top:1Q;}.escaped{scroll-margin-top:1p\\78;}");

    let result = qualify(1501, &css);
    assert_expected(
        &result,
        &vec![ExpectedOutcome::DirectLength; units.len() + 2],
    );
}

#[test]
fn signed_dimension_values_are_qualified_without_range_ordering() {
    let result = qualify(
        1502,
        concat!(
            "a{scroll-margin-top:0px;}",
            "b{scroll-margin-top:+0px;}",
            "c{scroll-margin-top:-0px;}",
            "d{scroll-margin-top:-0e100px;}",
            "e{scroll-margin-top:1px;}",
            "f{scroll-margin-top:.5em;}",
            "g{scroll-margin-top:1e100cqi;}",
            "h{scroll-margin-top:-10px;}",
            "i{scroll-margin-top:-.5em;}",
            "j{scroll-margin-top:-1e-999px;}",
            "k{scroll-margin-top:1deg;}",
            "l{scroll-margin-top:-1s;}",
            "m{scroll-margin-top:0fr;}",
            "n{scroll-margin-top:-1foo;}",
            "o{scroll-margin-top:0deg;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn comments_priority_separated_signs_and_cardinality_preserve_token_boundaries() {
    let result = qualify(
        1503,
        concat!(
            "a{scroll-margin-top:/**/-10px/**/!important;}",
            "b{scroll-margin-top:/**/0/**/!important;}",
            "c{scroll-margin-top:+ 0;}",
            "d{scroll-margin-top:+/**/0;}",
            "e{scroll-margin-top:- 1px;}",
            "f{scroll-margin-top:-/**/1px;}",
            "g{scroll-margin-top:1px 2px;}",
            "h{scroll-margin-top:10% 10px;}",
            "i{scroll-margin-top:(1px);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    for index in 0..2 {
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some()
        );
    }
}

#[test]
fn sole_functions_are_unsupported_without_length_evaluation() {
    let result = qualify(
        1504,
        concat!(
            "a{scroll-margin-top:calc(2em + 3ex);}",
            "b{scroll-margin-top:calc(-10px + 1em);}",
            "c{scroll-margin-top:min(-1px,2px);}",
            "d{scroll-margin-top:max(-10px,2px);}",
            "e{scroll-margin-top:clamp(-2em,1px,3px);}",
            "f{scroll-margin-top:foo();}",
        ),
    );
    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 6]);

    let mixed = qualify(
        1505,
        concat!(
            "a{scroll-margin-top:calc(1px) 2px;}",
            "b{scroll-margin-top:foo() -10px;}",
            "c{scroll-margin-top:-10px foo();}",
        ),
    );
    assert_expected(&mixed, &[ExpectedOutcome::Invalid; 3]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        1506,
        concat!(
            "a{scroll-margin-top:initial;}",
            "b{scroll-margin-top:inherit;}",
            "c{scroll-margin-top:unset;}",
            "d{scroll-margin-top:revert;}",
            "e{scroll-margin-top:revert-layer;}",
            "f{scroll-margin-top:revert-rule;}",
            "g{scroll-margin-top:-1px initial;}",
            "h{scroll-margin-top:initial -1px;}",
        ),
    );
    assert_expected(
        &css_wide,
        &[
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );

    let deferred = qualify(
        1507,
        concat!(
            "a{scroll-margin-top:var(--s);}",
            "b{scroll-margin-top:env(s);}",
            "c{scroll-margin-top:attr(data-s);}",
            "d{scroll-margin-top:--s();}",
            "e{scroll-margin-top:-1px var(--s);}",
            "f{scroll-margin-top:calc(var(--s));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 6]);

    let whole = qualify(
        1508,
        concat!(
            "a{scroll-margin-top:first-valid(-1px,2px);}",
            "b{scroll-margin-top:cycle(-1px,2px);}",
            "c{scroll-margin-top:interpolate(1,0:-1px,1:2px);}",
            "d{scroll-margin-top:first-valid(-1px,2px) 2px;}",
            "e{scroll-margin-top:-1px first-valid(2px,3px);}",
        ),
    );
    assert_expected(
        &whole,
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
fn one_run_owns_upstream_evidence_for_every_selected_value_leaf() {
    let result = qualify(
        1509,
        concat!(
            "a{direction:ltr;}",
            "b{box-sizing:border-box;}",
            "c{isolation:auto;}",
            "d{order:1;}",
            "e{scroll-snap-align:start end;}",
            "f{z-index:auto;}",
            "g{column-count:2;}",
            "h{flex-grow:.5;}",
            "i{flex-shrink:.5;}",
            "j{opacity:120%;}",
            "k{shape-image-threshold:-50%;}",
            "l{perspective:1px;}",
            "m{border-top-width:thick;}",
            "n{shape-margin:37.5%;}",
            "o{line-height:1.2;}",
            "p{word-spacing:-10%;}",
            "q{scroll-margin-top:-10px;}",
            "r{direction:rtl;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.isolation_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
    assert_eq!(result.scroll_snap_align_observations().len(), 1);
    assert_eq!(result.z_index_observations().len(), 1);
    assert_eq!(result.column_count_observations().len(), 1);
    assert_eq!(result.flex_grow_observations().len(), 1);
    assert_eq!(result.flex_shrink_observations().len(), 1);
    assert_eq!(result.opacity_observations().len(), 1);
    assert_eq!(result.shape_image_threshold_observations().len(), 1);
    assert_eq!(result.perspective_observations().len(), 1);
    assert_eq!(result.border_top_width_observations().len(), 1);
    assert_eq!(result.shape_margin_observations().len(), 1);
    assert_eq!(result.line_height_observations().len(), 1);
    assert_eq!(result.word_spacing_observations().len(), 1);
    assert_eq!(result.scroll_margin_top_observations().len(), 1);

    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 1);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 2);
    assert_eq!(result.order_observations()[0].occurrence_index(), 3);
    assert_eq!(
        result.scroll_snap_align_observations()[0].occurrence_index(),
        4
    );
    assert_eq!(result.z_index_observations()[0].occurrence_index(), 5);
    assert_eq!(result.column_count_observations()[0].occurrence_index(), 6);
    assert_eq!(result.flex_grow_observations()[0].occurrence_index(), 7);
    assert_eq!(result.flex_shrink_observations()[0].occurrence_index(), 8);
    assert_eq!(result.opacity_observations()[0].occurrence_index(), 9);
    assert_eq!(
        result.shape_image_threshold_observations()[0].occurrence_index(),
        10
    );
    assert_eq!(result.perspective_observations()[0].occurrence_index(), 11);
    assert_eq!(
        result.border_top_width_observations()[0].occurrence_index(),
        12
    );
    assert_eq!(result.shape_margin_observations()[0].occurrence_index(), 13);
    assert_eq!(result.line_height_observations()[0].occurrence_index(), 14);
    assert_eq!(result.word_spacing_observations()[0].occurrence_index(), 15);
    assert_eq!(
        result.scroll_margin_top_observations()[0].occurrence_index(),
        16
    );
    assert_eq!(result.direction_observations()[1].occurrence_index(), 17);
    assert_eq!(
        result.scroll_margin_top_observations()[0].outcome(),
        CssScrollMarginTopQualificationOutcome::Qualified(
            CssScrollMarginTopValue::DirectLengthLiteral
        )
    );
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(1510, "a{scroll-margin-top:-1px;}b{scroll-margin-top:-1px;}");
    assert_expected(
        &result,
        &[ExpectedOutcome::DirectLength, ExpectedOutcome::DirectLength],
    );
    assert_eq!(
        result.scroll_margin_top_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.scroll_margin_top_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.scroll_margin_top_observations()[0]
            .placement()
            .context_id(),
        result.scroll_margin_top_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1511, "@font-face{scroll-margin-top:-1px;}"),
        (1512, "@page{scroll-margin-top:-1px;}"),
        (1513, "@page{@top-left{scroll-margin-top:-1px;}}"),
        (1514, "@keyframes k{from{scroll-margin-top:-1px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.scroll_margin_top_observations().is_empty(),
            "nonordinary declaration context produced a scroll-margin-top observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1515,
        "a{scroll-margin-top:-1px;scroll-margin-top:20%;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::DirectLength]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{scroll-margin-top:-1px;}",
        "b{scroll-margin-top:20%;}",
        "c{scroll-margin-top:calc(2em + 3ex);}",
        "d{scroll-margin-top:var(--s);}",
        "e{word-spacing:-10%;}",
        "f{line-height:1.2;}",
        "g{shape-margin:1px;}",
        "h{perspective:1px;}",
        "i{opacity:-50%;}",
        "j{flex-grow:.5;}",
        "k{direction:ltr;}",
        "l{column-count:2;}",
        "m{z-index:auto;}",
        "n{border-top-width:thin;}",
    );
    let first = qualify(1516, css);
    let repeated = qualify(1516, css);
    let another_source = qualify(1517, css);

    assert_eq!(
        first.scroll_margin_top_observations(),
        repeated.scroll_margin_top_observations()
    );
    assert_eq!(
        first.scroll_margin_top_observations(),
        another_source.scroll_margin_top_observations()
    );
    assert_eq!(
        first.word_spacing_observations(),
        repeated.word_spacing_observations()
    );
    assert_eq!(
        first.line_height_observations(),
        repeated.line_height_observations()
    );
    assert_eq!(
        first.shape_margin_observations(),
        repeated.shape_margin_observations()
    );
    assert_eq!(
        first.perspective_observations(),
        repeated.perspective_observations()
    );
    assert_eq!(
        first.opacity_observations(),
        repeated.opacity_observations()
    );
    assert_eq!(
        first.flex_grow_observations(),
        repeated.flex_grow_observations()
    );
    assert_eq!(
        first.direction_observations(),
        repeated.direction_observations()
    );
    assert_eq!(
        first.column_count_observations(),
        repeated.column_count_observations()
    );
    assert_eq!(
        first.z_index_observations(),
        repeated.z_index_observations()
    );
    assert_eq!(
        first.border_top_width_observations(),
        repeated.border_top_width_observations()
    );
}
