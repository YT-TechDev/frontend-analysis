use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssValueQualificationRunResult, CssWordSpacingQualificationOutcome,
    CssWordSpacingUnsupportedReason, CssWordSpacingValue, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    DirectLength,
    DirectPercentage,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssWordSpacingQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssWordSpacingQualificationOutcome::Qualified(CssWordSpacingValue::Normal)
        }
        ExpectedOutcome::DirectLength => {
            CssWordSpacingQualificationOutcome::Qualified(CssWordSpacingValue::DirectLengthLiteral)
        }
        ExpectedOutcome::DirectPercentage => CssWordSpacingQualificationOutcome::Qualified(
            CssWordSpacingValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssWordSpacingQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssWordSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssWordSpacingUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssWordSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssWordSpacingUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssWordSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssWordSpacingUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssWordSpacingQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssWordSpacingUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .word_spacing_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_normal_unitless_zero_and_signed_percentage_boundaries_are_explicit() {
    let result = qualify(
        1400,
        concat!(
            "a{word-spacing:normal;}",
            "b{word-spacing:NORMAL;}",
            "c{word-spacing:\\6eormal;}",
            "d{word-spacing:0;}",
            "e{word-spacing:+0;}",
            "f{word-spacing:-0;}",
            "g{word-spacing:.0;}",
            "h{word-spacing:-.0;}",
            "i{word-spacing:0.0;}",
            "j{word-spacing:-0.0;}",
            "k{word-spacing:0e100;}",
            "l{word-spacing:-0e100;}",
            "m{word-spacing:1;}",
            "n{word-spacing:-1;}",
            "o{word-spacing:.5;}",
            "p{word-spacing:37.5%;}",
            "q{word-spacing:0%;}",
            "r{word-spacing:+0%;}",
            "s{word-spacing:-0%;}",
            "t{word-spacing:.0%;}",
            "u{word-spacing:-.0%;}",
            "v{word-spacing:0e100%;}",
            "w{word-spacing:-0e100%;}",
            "x{word-spacing:100%;}",
            "y{word-spacing:1e100%;}",
            "z{word-spacing:-1%;}",
            "aa{word-spacing:-.5%;}",
            "ab{word-spacing:-1e-999%;}",
            "ac{word-spacing:auto;}",
            "ad{word-spacing:none;}",
            "ae{word-spacing:\"1px\";}",
            "af{word-spacing:;}",
            "ag{color:1px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Normal,
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
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
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
        css.push_str(&format!(".u{index}{{word-spacing:1{unit};}}"));
    }
    css.push_str(".upper{word-spacing:1Q;}.escaped{word-spacing:1p\\78;}");

    let result = qualify(1401, &css);
    assert_expected(
        &result,
        &vec![ExpectedOutcome::DirectLength; units.len() + 2],
    );
}

#[test]
fn signed_dimension_values_are_qualified_without_range_ordering() {
    let result = qualify(
        1402,
        concat!(
            "a{word-spacing:0px;}",
            "b{word-spacing:+0px;}",
            "c{word-spacing:-0px;}",
            "d{word-spacing:-0e100px;}",
            "e{word-spacing:1px;}",
            "f{word-spacing:.5em;}",
            "g{word-spacing:1e100cqi;}",
            "h{word-spacing:-1px;}",
            "i{word-spacing:-.5em;}",
            "j{word-spacing:-1e-999px;}",
            "k{word-spacing:1deg;}",
            "l{word-spacing:-1s;}",
            "m{word-spacing:0fr;}",
            "n{word-spacing:-1foo;}",
            "o{word-spacing:0deg;}",
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
        1403,
        concat!(
            "a{word-spacing:/**/normal/**/!important;}",
            "b{word-spacing:/**/-20px/**/!important;}",
            "c{word-spacing:/**/-10%/**/!important;}",
            "d{word-spacing:+ 0;}",
            "e{word-spacing:+/**/0;}",
            "f{word-spacing:- 1px;}",
            "g{word-spacing:-/**/1px;}",
            "h{word-spacing:1px 2px;}",
            "i{word-spacing:10% 10px;}",
            "j{word-spacing:normal 10px;}",
            "k{word-spacing:(1px);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::DirectLength,
            ExpectedOutcome::DirectPercentage,
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
    for index in 0..3 {
        assert!(
            result.upstream_parser_result().occurrences()[index]
                .priority()
                .is_some()
        );
    }
}

#[test]
fn sole_functions_are_unsupported_without_length_percentage_evaluation() {
    let result = qualify(
        1404,
        concat!(
            "a{word-spacing:calc(2em + 3ex);}",
            "b{word-spacing:calc(2ch - 30%);}",
            "c{word-spacing:calc(40% + 50px);}",
            "d{word-spacing:min(-1px,20%);}",
            "e{word-spacing:max(-10%,2px);}",
            "f{word-spacing:clamp(-2em,10%,3px);}",
            "g{word-spacing:foo();}",
        ),
    );
    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 7]);

    let mixed = qualify(
        1405,
        concat!(
            "a{word-spacing:calc(1px) 2px;}",
            "b{word-spacing:normal foo();}",
            "c{word-spacing:foo() -10%;}",
        ),
    );
    assert_expected(&mixed, &[ExpectedOutcome::Invalid; 3]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        1406,
        concat!(
            "a{word-spacing:initial;}",
            "b{word-spacing:inherit;}",
            "c{word-spacing:unset;}",
            "d{word-spacing:revert;}",
            "e{word-spacing:revert-layer;}",
            "f{word-spacing:revert-rule;}",
            "g{word-spacing:-1px initial;}",
            "h{word-spacing:initial normal;}",
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
        1407,
        concat!(
            "a{word-spacing:var(--s);}",
            "b{word-spacing:env(s);}",
            "c{word-spacing:attr(data-s);}",
            "d{word-spacing:--s();}",
            "e{word-spacing:-1px var(--s);}",
            "f{word-spacing:normal var(--s);}",
            "g{word-spacing:calc(var(--s));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 7]);

    let whole = qualify(
        1408,
        concat!(
            "a{word-spacing:first-valid(-1px,-10%);}",
            "b{word-spacing:cycle(-1px,2px);}",
            "c{word-spacing:interpolate(1,0:-1px,1:2px);}",
            "d{word-spacing:first-valid(-1px,-10%) 2px;}",
            "e{word-spacing:-1px first-valid(-10%,1px);}",
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
        1409,
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
            "q{direction:rtl;}",
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
    assert_eq!(result.direction_observations()[1].occurrence_index(), 16);
    assert_eq!(
        result.word_spacing_observations()[0].outcome(),
        CssWordSpacingQualificationOutcome::Qualified(CssWordSpacingValue::DirectPercentageLiteral)
    );
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(1410, "a{word-spacing:normal;}b{word-spacing:normal;}");
    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Normal]);
    assert_eq!(result.word_spacing_observations()[0].occurrence_index(), 0);
    assert_eq!(result.word_spacing_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.word_spacing_observations()[0]
            .placement()
            .context_id(),
        result.word_spacing_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1411, "@font-face{word-spacing:-1px;}"),
        (1412, "@page{word-spacing:-1px;}"),
        (1413, "@page{@top-left{word-spacing:-1px;}}"),
        (1414, "@keyframes k{from{word-spacing:-1px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.word_spacing_observations().is_empty(),
            "nonordinary declaration context produced a word-spacing observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1415,
        "a{word-spacing:-1px;word-spacing:-10%;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::DirectLength]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{word-spacing:normal;}",
        "b{word-spacing:-1px;}",
        "c{word-spacing:-10%;}",
        "d{word-spacing:calc(2em + 3ex);}",
        "e{word-spacing:var(--s);}",
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
    let first = qualify(1416, css);
    let repeated = qualify(1416, css);
    let another_source = qualify(1417, css);

    assert_eq!(
        first.word_spacing_observations(),
        repeated.word_spacing_observations()
    );
    assert_eq!(
        first.word_spacing_observations(),
        another_source.word_spacing_observations()
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
