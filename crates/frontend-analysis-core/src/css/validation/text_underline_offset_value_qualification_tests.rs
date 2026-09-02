use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextUnderlineOffsetQualificationOutcome, CssTextUnderlineOffsetUnsupportedReason,
    CssTextUnderlineOffsetValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssTextUnderlineOffsetQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssTextUnderlineOffsetQualificationOutcome::Qualified(CssTextUnderlineOffsetValue::Auto)
        }
        ExpectedOutcome::DirectLength => CssTextUnderlineOffsetQualificationOutcome::Qualified(
            CssTextUnderlineOffsetValue::DirectLengthLiteral,
        ),
        ExpectedOutcome::DirectPercentage => CssTextUnderlineOffsetQualificationOutcome::Qualified(
            CssTextUnderlineOffsetValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssTextUnderlineOffsetQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssTextUnderlineOffsetQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextUnderlineOffsetUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssTextUnderlineOffsetQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextUnderlineOffsetUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssTextUnderlineOffsetQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextUnderlineOffsetUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssTextUnderlineOffsetQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTextUnderlineOffsetUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .text_underline_offset_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_auto_unitless_zero_and_signed_percentage_boundaries_are_explicit() {
    let result = qualify(
        1600,
        concat!(
            "a{text-underline-offset:auto;}",
            "b{text-underline-offset:AUTO;}",
            "c{text-underline-offset:\\61uto;}",
            "d{text-underline-offset:0;}",
            "e{text-underline-offset:+0;}",
            "f{text-underline-offset:-0;}",
            "g{text-underline-offset:.0;}",
            "h{text-underline-offset:-.0;}",
            "i{text-underline-offset:0.0;}",
            "j{text-underline-offset:-0.0;}",
            "k{text-underline-offset:0e100;}",
            "l{text-underline-offset:-0e100;}",
            "m{text-underline-offset:1;}",
            "n{text-underline-offset:-10;}",
            "o{text-underline-offset:.5;}",
            "p{text-underline-offset:10e2;}",
            "q{text-underline-offset:0%;}",
            "r{text-underline-offset:+0%;}",
            "s{text-underline-offset:-0%;}",
            "t{text-underline-offset:.0%;}",
            "u{text-underline-offset:-.0%;}",
            "v{text-underline-offset:0e100%;}",
            "w{text-underline-offset:-0e100%;}",
            "x{text-underline-offset:187%;}",
            "y{text-underline-offset:1e100%;}",
            "z{text-underline-offset:-30%;}",
            "aa{text-underline-offset:-.5%;}",
            "ab{text-underline-offset:-1e-999%;}",
            "ac{text-underline-offset:from-font;}",
            "ad{text-underline-offset:none;}",
            "ae{text-underline-offset:\"1px\";}",
            "af{text-underline-offset:;}",
            "ag{color:1px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
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
        css.push_str(&format!(".u{index}{{text-underline-offset:1{unit};}}"));
    }
    css.push_str(".upper{text-underline-offset:1Q;}.escaped{text-underline-offset:1p\\78;}");

    let result = qualify(1601, &css);
    assert_expected(
        &result,
        &vec![ExpectedOutcome::DirectLength; units.len() + 2],
    );
}

#[test]
fn signed_dimension_values_are_qualified_without_range_ordering() {
    let result = qualify(
        1602,
        concat!(
            "a{text-underline-offset:0px;}",
            "b{text-underline-offset:+0px;}",
            "c{text-underline-offset:-0px;}",
            "d{text-underline-offset:-0e100px;}",
            "e{text-underline-offset:53px;}",
            "f{text-underline-offset:.5em;}",
            "g{text-underline-offset:1e100cqi;}",
            "h{text-underline-offset:-10px;}",
            "i{text-underline-offset:-49em;}",
            "j{text-underline-offset:-1e-999px;}",
            "k{text-underline-offset:1deg;}",
            "l{text-underline-offset:-1s;}",
            "m{text-underline-offset:0fr;}",
            "n{text-underline-offset:-1foo;}",
            "o{text-underline-offset:0deg;}",
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
        1603,
        concat!(
            "a{text-underline-offset:/**/auto/**/!important;}",
            "b{text-underline-offset:/**/-10px/**/!important;}",
            "c{text-underline-offset:/**/-30%/**/!important;}",
            "d{text-underline-offset:+ 0;}",
            "e{text-underline-offset:+/**/0;}",
            "f{text-underline-offset:- 1px;}",
            "g{text-underline-offset:-/**/1px;}",
            "h{text-underline-offset:1px 2px;}",
            "i{text-underline-offset:10% 10px;}",
            "j{text-underline-offset:auto 10px;}",
            "k{text-underline-offset:(1px);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
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
        1604,
        concat!(
            "a{text-underline-offset:calc(45% - 0.3em);}",
            "b{text-underline-offset:calc(40em - 10px);}",
            "c{text-underline-offset:calc(-13em + 50px);}",
            "d{text-underline-offset:min(-1px,20%);}",
            "e{text-underline-offset:max(-30%,2px);}",
            "f{text-underline-offset:clamp(-2em,10%,3px);}",
            "g{text-underline-offset:foo();}",
        ),
    );
    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 7]);

    let mixed = qualify(
        1605,
        concat!(
            "a{text-underline-offset:calc(1px) 2px;}",
            "b{text-underline-offset:auto foo();}",
            "c{text-underline-offset:foo() -30%;}",
        ),
    );
    assert_expected(&mixed, &[ExpectedOutcome::Invalid; 3]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        1606,
        concat!(
            "a{text-underline-offset:initial;}",
            "b{text-underline-offset:inherit;}",
            "c{text-underline-offset:unset;}",
            "d{text-underline-offset:revert;}",
            "e{text-underline-offset:revert-layer;}",
            "f{text-underline-offset:revert-rule;}",
            "g{text-underline-offset:-1px initial;}",
            "h{text-underline-offset:initial auto;}",
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
        1607,
        concat!(
            "a{text-underline-offset:var(--s);}",
            "b{text-underline-offset:env(s);}",
            "c{text-underline-offset:attr(data-s);}",
            "d{text-underline-offset:--s();}",
            "e{text-underline-offset:-1px var(--s);}",
            "f{text-underline-offset:auto var(--s);}",
            "g{text-underline-offset:calc(var(--s));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 7]);

    let whole = qualify(
        1608,
        concat!(
            "a{text-underline-offset:first-valid(-1px,-30%);}",
            "b{text-underline-offset:cycle(auto,2px);}",
            "c{text-underline-offset:interpolate(1,0:-1px,1:2px);}",
            "d{text-underline-offset:first-valid(-1px,-30%) 2px;}",
            "e{text-underline-offset:-1px first-valid(-30%,1px);}",
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
        1609,
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
            "r{text-underline-offset:-30%;}",
            "s{direction:rtl;}",
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
    assert_eq!(result.text_underline_offset_observations().len(), 1);

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
    assert_eq!(
        result.text_underline_offset_observations()[0].occurrence_index(),
        17
    );
    assert_eq!(result.direction_observations()[1].occurrence_index(), 18);
    assert_eq!(
        result.text_underline_offset_observations()[0].outcome(),
        CssTextUnderlineOffsetQualificationOutcome::Qualified(
            CssTextUnderlineOffsetValue::DirectPercentageLiteral
        )
    );
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(
        1610,
        "a{text-underline-offset:auto;}b{text-underline-offset:auto;}",
    );
    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_eq!(
        result.text_underline_offset_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.text_underline_offset_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.text_underline_offset_observations()[0]
            .placement()
            .context_id(),
        result.text_underline_offset_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1611, "@font-face{text-underline-offset:-1px;}"),
        (1612, "@page{text-underline-offset:-1px;}"),
        (1613, "@page{@top-left{text-underline-offset:-1px;}}"),
        (1614, "@keyframes k{from{text-underline-offset:-1px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_underline_offset_observations().is_empty(),
            "nonordinary declaration context produced a text-underline-offset observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1615,
        "a{text-underline-offset:-1px;text-underline-offset:-30%;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::DirectLength]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{text-underline-offset:auto;}",
        "b{text-underline-offset:-1px;}",
        "c{text-underline-offset:-30%;}",
        "d{text-underline-offset:calc(45% - 0.3em);}",
        "e{text-underline-offset:var(--s);}",
        "f{scroll-margin-top:-1px;}",
        "g{word-spacing:-10%;}",
        "h{line-height:1.2;}",
        "i{shape-margin:1px;}",
        "j{perspective:1px;}",
        "k{opacity:-50%;}",
        "l{flex-grow:.5;}",
        "m{direction:ltr;}",
        "n{column-count:2;}",
        "o{z-index:auto;}",
        "p{border-top-width:thin;}",
    );
    let first = qualify(1616, css);
    let repeated = qualify(1616, css);
    let another_source = qualify(1617, css);

    assert_eq!(
        first.text_underline_offset_observations(),
        repeated.text_underline_offset_observations()
    );
    assert_eq!(
        first.text_underline_offset_observations(),
        another_source.text_underline_offset_observations()
    );
    assert_eq!(
        first.scroll_margin_top_observations(),
        repeated.scroll_margin_top_observations()
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
