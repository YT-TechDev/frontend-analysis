use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssLineHeightQualificationOutcome, CssLineHeightUnsupportedReason, CssLineHeightValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    DirectNumber,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssLineHeightQualificationOutcome {
    match expected {
        ExpectedOutcome::Normal => {
            CssLineHeightQualificationOutcome::Qualified(CssLineHeightValue::Normal)
        }
        ExpectedOutcome::DirectNumber => {
            CssLineHeightQualificationOutcome::Qualified(CssLineHeightValue::DirectNumberLiteral)
        }
        ExpectedOutcome::DirectLength => {
            CssLineHeightQualificationOutcome::Qualified(CssLineHeightValue::DirectLengthLiteral)
        }
        ExpectedOutcome::DirectPercentage => CssLineHeightQualificationOutcome::Qualified(
            CssLineHeightValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssLineHeightQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssLineHeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineHeightUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssLineHeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineHeightUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssLineHeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineHeightUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssLineHeightQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssLineHeightUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .line_height_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_keyword_number_percentage_and_ambiguous_zero_boundaries_are_explicit() {
    let result = qualify(
        1300,
        concat!(
            "a{line-height:normal;}",
            "b{line-height:NORMAL;}",
            "c{line-height:\\6eormal;}",
            "d{line-height:0;}",
            "e{line-height:+0;}",
            "f{line-height:-0;}",
            "g{line-height:.0;}",
            "h{line-height:-.0;}",
            "i{line-height:0.0;}",
            "j{line-height:-0.0;}",
            "k{line-height:0e100;}",
            "l{line-height:-0e100;}",
            "m{line-height:1;}",
            "n{line-height:.5;}",
            "o{line-height:1e100;}",
            "p{line-height:-1;}",
            "q{line-height:-.5;}",
            "r{line-height:-1e-999;}",
            "s{line-height:0%;}",
            "t{line-height:+0%;}",
            "u{line-height:-0%;}",
            "v{line-height:37.5%;}",
            "w{line-height:200%;}",
            "x{line-height:-1%;}",
            "y{line-height:-.5%;}",
            "z{line-height:auto;}",
            "aa{line-height:none;}",
            "ab{line-height:\"1\";}",
            "ac{line-height:;}",
            "ad{color:1;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Normal,
            ExpectedOutcome::Normal,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
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
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );

    assert_eq!(
        result.line_height_observations()[3].outcome(),
        CssLineHeightQualificationOutcome::Qualified(CssLineHeightValue::DirectNumberLiteral)
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
        css.push_str(&format!(".u{index}{{line-height:1{unit};}}"));
    }
    css.push_str(".upper{line-height:1Q;}.escaped{line-height:1p\\78;}");

    let result = qualify(1301, &css);
    assert_expected(
        &result,
        &vec![ExpectedOutcome::DirectLength; units.len() + 2],
    );
}

#[test]
fn direct_dimension_range_unit_and_zero_branch_mismatches_are_source_provable() {
    let result = qualify(
        1302,
        concat!(
            "a{line-height:0px;}",
            "b{line-height:-0px;}",
            "c{line-height:-0e100px;}",
            "d{line-height:1px;}",
            "e{line-height:.5em;}",
            "f{line-height:1e100cqi;}",
            "g{line-height:-1px;}",
            "h{line-height:-.5em;}",
            "i{line-height:-1e-999px;}",
            "j{line-height:1deg;}",
            "k{line-height:1s;}",
            "l{line-height:1fr;}",
            "m{line-height:1foo;}",
            "n{line-height:0deg;}",
            "o{line-height:0;}",
            "p{line-height:0%;}",
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
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectPercentage,
        ],
    );
}

#[test]
fn comments_priority_separated_signs_and_cardinality_preserve_token_boundaries() {
    let result = qualify(
        1303,
        concat!(
            "a{line-height:/**/normal/**/!important;}",
            "b{line-height:/**/-0/**/!important;}",
            "c{line-height:+ 0;}",
            "d{line-height:+/**/0;}",
            "e{line-height:- 1px;}",
            "f{line-height:-/**/1px;}",
            "g{line-height:1 2px;}",
            "h{line-height:37% 1;}",
            "i{line-height:normal 1px;}",
            "j{line-height:(1);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::DirectNumber,
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
fn sole_functions_are_unsupported_without_number_or_length_percentage_evaluation() {
    let result = qualify(
        1304,
        concat!(
            "a{line-height:calc(2);}",
            "b{line-height:calc(200% + 10px);}",
            "c{line-height:calc(-1);}",
            "d{line-height:min(1px,200%);}",
            "e{line-height:max(0%,2px);}",
            "f{line-height:clamp(0px,1,2em);}",
            "g{line-height:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 7]);

    let mixed = qualify(
        1305,
        concat!(
            "a{line-height:calc(1) 2px;}",
            "b{line-height:normal foo();}",
            "c{line-height:foo() 1%;}",
        ),
    );
    assert_expected(&mixed, &[ExpectedOutcome::Invalid; 3]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        1306,
        concat!(
            "a{line-height:initial;}",
            "b{line-height:inherit;}",
            "c{line-height:unset;}",
            "d{line-height:revert;}",
            "e{line-height:revert-layer;}",
            "f{line-height:revert-rule;}",
            "g{line-height:1 initial;}",
            "h{line-height:initial normal;}",
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
        1307,
        concat!(
            "a{line-height:var(--h);}",
            "b{line-height:env(h);}",
            "c{line-height:attr(data-h);}",
            "d{line-height:--h();}",
            "e{line-height:-1 var(--h);}",
            "f{line-height:normal var(--h);}",
            "g{line-height:calc(var(--h));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 7]);

    let whole = qualify(
        1308,
        concat!(
            "a{line-height:first-valid(1,120%);}",
            "b{line-height:cycle(1,2);}",
            "c{line-height:interpolate(1,0:0,1:1);}",
            "d{line-height:first-valid(1,120%) 2px;}",
            "e{line-height:1 first-valid(0%,1px);}",
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
        1309,
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
            "p{direction:rtl;}",
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
    assert_eq!(result.direction_observations()[1].occurrence_index(), 15);
    assert_eq!(
        result.line_height_observations()[0].outcome(),
        CssLineHeightQualificationOutcome::Qualified(CssLineHeightValue::DirectNumberLiteral)
    );
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(1310, "a{line-height:normal;}b{line-height:normal;}");
    assert_expected(&result, &[ExpectedOutcome::Normal, ExpectedOutcome::Normal]);
    assert_eq!(result.line_height_observations()[0].occurrence_index(), 0);
    assert_eq!(result.line_height_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.line_height_observations()[0]
            .placement()
            .context_id(),
        result.line_height_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1311, "@font-face{line-height:1;}"),
        (1312, "@page{line-height:1;}"),
        (1313, "@page{@top-left{line-height:1;}}"),
        (1314, "@keyframes k{from{line-height:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.line_height_observations().is_empty(),
            "nonordinary declaration context produced a line-height observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1315,
        "a{line-height:normal;line-height:1.2;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::Normal]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{line-height:normal;}",
        "b{line-height:-0;}",
        "c{line-height:37.5%;}",
        "d{line-height:calc(2);}",
        "e{line-height:var(--h);}",
        "f{shape-margin:1px;}",
        "g{perspective:1px;}",
        "h{opacity:-50%;}",
        "i{flex-grow:.5;}",
        "j{direction:ltr;}",
        "k{column-count:2;}",
        "l{z-index:auto;}",
        "m{border-top-width:thin;}",
    );
    let first = qualify(1316, css);
    let repeated = qualify(1316, css);
    let another_source = qualify(1317, css);

    assert_eq!(
        first.line_height_observations(),
        repeated.line_height_observations()
    );
    assert_eq!(
        first.line_height_observations(),
        another_source.line_height_observations()
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
