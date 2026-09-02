use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssShapeMarginQualificationOutcome, CssShapeMarginUnsupportedReason, CssShapeMarginValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
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

fn expected_outcome(expected: ExpectedOutcome) -> CssShapeMarginQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectLength => {
            CssShapeMarginQualificationOutcome::Qualified(CssShapeMarginValue::DirectLengthLiteral)
        }
        ExpectedOutcome::DirectPercentage => CssShapeMarginQualificationOutcome::Qualified(
            CssShapeMarginValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssShapeMarginQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssShapeMarginQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeMarginUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssShapeMarginQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeMarginUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssShapeMarginQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeMarginUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssShapeMarginQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeMarginUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .shape_margin_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_unitless_zero_percentage_and_direct_type_boundaries_are_explicit() {
    let result = qualify(
        1200,
        concat!(
            "a{shape-margin:0;}",
            "b{shape-margin:+0;}",
            "c{shape-margin:-0;}",
            "d{shape-margin:.0;}",
            "e{shape-margin:-.0;}",
            "f{shape-margin:0.0;}",
            "g{shape-margin:-0.0;}",
            "h{shape-margin:0e100;}",
            "i{shape-margin:-0e100;}",
            "j{shape-margin:37.5%;}",
            "k{shape-margin:0%;}",
            "l{shape-margin:+0%;}",
            "m{shape-margin:-0%;}",
            "n{shape-margin:.0%;}",
            "o{shape-margin:-.0%;}",
            "p{shape-margin:0e100%;}",
            "q{shape-margin:-0e100%;}",
            "r{shape-margin:100%;}",
            "s{shape-margin:1e100%;}",
            "t{shape-margin:-1%;}",
            "u{shape-margin:-.5%;}",
            "v{shape-margin:-1e-999%;}",
            "w{shape-margin:1;}",
            "x{shape-margin:-1;}",
            "y{shape-margin:.5;}",
            "z{shape-margin:none;}",
            "aa{shape-margin:auto;}",
            "ab{shape-margin:\"1px\";}",
            "ac{shape-margin:;}",
            "ad{color:1px;}",
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
        css.push_str(&format!(".u{index}{{shape-margin:1{unit};}}"));
    }
    css.push_str(".upper{shape-margin:1Q;}.escaped{shape-margin:1p\\78;}");

    let result = qualify(1201, &css);
    assert_expected(
        &result,
        &vec![ExpectedOutcome::DirectLength; units.len() + 2],
    );
}

#[test]
fn direct_dimension_range_and_unit_mismatches_are_source_provable() {
    let result = qualify(
        1202,
        concat!(
            "a{shape-margin:0px;}",
            "b{shape-margin:-0px;}",
            "c{shape-margin:-0e100px;}",
            "d{shape-margin:1px;}",
            "e{shape-margin:.5em;}",
            "f{shape-margin:1e100cqi;}",
            "g{shape-margin:-1px;}",
            "h{shape-margin:-.5em;}",
            "i{shape-margin:-1e-999px;}",
            "j{shape-margin:1deg;}",
            "k{shape-margin:1s;}",
            "l{shape-margin:1fr;}",
            "m{shape-margin:1foo;}",
            "n{shape-margin:0deg;}",
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
}

#[test]
fn comments_priority_separated_signs_and_cardinality_preserve_token_boundaries() {
    let result = qualify(
        1203,
        concat!(
            "a{shape-margin:/**/37.5%/**/!important;}",
            "b{shape-margin:/**/-0px/**/!important;}",
            "c{shape-margin:+ 0%;}",
            "d{shape-margin:+/**/0%;}",
            "e{shape-margin:- 1px;}",
            "f{shape-margin:-/**/1px;}",
            "g{shape-margin:1px 2%;}",
            "h{shape-margin:37% 1px;}",
            "i{shape-margin:1px 37%;}",
            "j{shape-margin:(1px);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectLength,
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
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
    assert!(
        result.upstream_parser_result().occurrences()[1]
            .priority()
            .is_some()
    );
}

#[test]
fn sole_functions_are_unsupported_without_length_percentage_evaluation() {
    let result = qualify(
        1204,
        concat!(
            "a{shape-margin:calc(10px);}",
            "b{shape-margin:calc(10%);}",
            "c{shape-margin:calc(-1%);}",
            "d{shape-margin:min(1px,2%);}",
            "e{shape-margin:max(0%,2px);}",
            "f{shape-margin:clamp(0px,1%,2px);}",
            "g{shape-margin:anchor-size(width);}",
            "h{shape-margin:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 8]);

    let mixed = qualify(
        1205,
        concat!(
            "a{shape-margin:calc(1px) 2%;}",
            "b{shape-margin:1px foo();}",
            "c{shape-margin:foo() 1%;}",
        ),
    );
    assert_expected(&mixed, &[ExpectedOutcome::Invalid; 3]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        1206,
        concat!(
            "a{shape-margin:initial;}",
            "b{shape-margin:inherit;}",
            "c{shape-margin:unset;}",
            "d{shape-margin:revert;}",
            "e{shape-margin:revert-layer;}",
            "f{shape-margin:revert-rule;}",
            "g{shape-margin:1px initial;}",
            "h{shape-margin:initial 37%;}",
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
        1207,
        concat!(
            "a{shape-margin:var(--m);}",
            "b{shape-margin:env(m);}",
            "c{shape-margin:attr(data-m);}",
            "d{shape-margin:--m();}",
            "e{shape-margin:-1% var(--m);}",
            "f{shape-margin:1px var(--m);}",
            "g{shape-margin:calc(var(--m));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 7]);

    let whole = qualify(
        1208,
        concat!(
            "a{shape-margin:first-valid(1px,20%);}",
            "b{shape-margin:cycle(1px,2%);}",
            "c{shape-margin:interpolate(1px,0:0,1:1);}",
            "d{shape-margin:first-valid(1px,20%) 2px;}",
            "e{shape-margin:1px first-valid(0%,1px);}",
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
        1209,
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
            "o{direction:rtl;}",
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
    assert_eq!(result.direction_observations()[1].occurrence_index(), 14);
    assert_eq!(
        result.shape_margin_observations()[0].outcome(),
        CssShapeMarginQualificationOutcome::Qualified(CssShapeMarginValue::DirectPercentageLiteral)
    );
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(1210, "a{shape-margin:1px;}b{shape-margin:1px;}");
    assert_expected(
        &result,
        &[ExpectedOutcome::DirectLength, ExpectedOutcome::DirectLength],
    );
    assert_eq!(result.shape_margin_observations()[0].occurrence_index(), 0);
    assert_eq!(result.shape_margin_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.shape_margin_observations()[0]
            .placement()
            .context_id(),
        result.shape_margin_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1211, "@font-face{shape-margin:1px;}"),
        (1212, "@page{shape-margin:1px;}"),
        (1213, "@page{@top-left{shape-margin:1px;}}"),
        (1214, "@keyframes k{from{shape-margin:1px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.shape_margin_observations().is_empty(),
            "nonordinary declaration context produced a shape-margin observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1215,
        "a{shape-margin:37.5%;shape-margin:1px;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::DirectPercentage]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{shape-margin:37.5%;}",
        "b{shape-margin:-0px;}",
        "c{shape-margin:calc(2%);}",
        "d{shape-margin:var(--m);}",
        "e{perspective:1px;}",
        "f{opacity:-50%;}",
        "g{flex-grow:.5;}",
        "h{direction:ltr;}",
        "i{column-count:2;}",
        "j{z-index:auto;}",
        "k{border-top-width:thin;}",
    );
    let first = qualify(1216, css);
    let repeated = qualify(1216, css);
    let another_source = qualify(1217, css);

    assert_eq!(
        first.shape_margin_observations(),
        repeated.shape_margin_observations()
    );
    assert_eq!(
        first.shape_margin_observations(),
        another_source.shape_margin_observations()
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
