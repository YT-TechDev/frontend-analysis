use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBorderTopWidthQualificationOutcome, CssBorderTopWidthUnsupportedReason,
    CssBorderTopWidthValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Thin,
    Medium,
    Thick,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssBorderTopWidthQualificationOutcome {
    match expected {
        ExpectedOutcome::Thin => {
            CssBorderTopWidthQualificationOutcome::Qualified(CssBorderTopWidthValue::Thin)
        }
        ExpectedOutcome::Medium => {
            CssBorderTopWidthQualificationOutcome::Qualified(CssBorderTopWidthValue::Medium)
        }
        ExpectedOutcome::Thick => {
            CssBorderTopWidthQualificationOutcome::Qualified(CssBorderTopWidthValue::Thick)
        }
        ExpectedOutcome::DirectLength => CssBorderTopWidthQualificationOutcome::Qualified(
            CssBorderTopWidthValue::DirectLengthLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssBorderTopWidthQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssBorderTopWidthQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderTopWidthUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssBorderTopWidthQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderTopWidthUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssBorderTopWidthQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderTopWidthUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssBorderTopWidthQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssBorderTopWidthUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .border_top_width_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_keywords_unitless_zero_and_direct_type_boundaries_are_explicit() {
    let result = qualify(
        1100,
        concat!(
            "a{border-top-width:thin;}",
            "b{border-top-width:MEDIUM;}",
            "c{border-top-width:thick;}",
            "d{border-top-width:\\74hin;}",
            "e{border-top-width:0;}",
            "f{border-top-width:+0;}",
            "g{border-top-width:-0;}",
            "h{border-top-width:.0;}",
            "i{border-top-width:-.0;}",
            "j{border-top-width:0.0;}",
            "k{border-top-width:-0.0;}",
            "l{border-top-width:0e100;}",
            "m{border-top-width:-0e100;}",
            "n{border-top-width:1;}",
            "o{border-top-width:-1;}",
            "p{border-top-width:.5;}",
            "q{border-top-width:80%;}",
            "r{border-top-width:none;}",
            "s{border-top-width:auto;}",
            "t{border-top-width:\"1px\";}",
            "u{border-top-width:;}",
            "v{color:1px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Thin,
            ExpectedOutcome::Medium,
            ExpectedOutcome::Thick,
            ExpectedOutcome::Thin,
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
        css.push_str(&format!(".u{index}{{border-top-width:1{unit};}}"));
    }
    css.push_str(".upper{border-top-width:1Q;}.escaped{border-top-width:1p\\78;}");

    let result = qualify(1101, &css);
    assert_expected(
        &result,
        &vec![ExpectedOutcome::DirectLength; units.len() + 2],
    );
}

#[test]
fn direct_dimension_range_and_unit_mismatches_are_source_provable() {
    let result = qualify(
        1102,
        concat!(
            "a{border-top-width:0px;}",
            "b{border-top-width:-0px;}",
            "c{border-top-width:-0e100px;}",
            "d{border-top-width:1px;}",
            "e{border-top-width:.5em;}",
            "f{border-top-width:1e100cqi;}",
            "g{border-top-width:-1px;}",
            "h{border-top-width:-.5em;}",
            "i{border-top-width:-1e-999px;}",
            "j{border-top-width:1deg;}",
            "k{border-top-width:1s;}",
            "l{border-top-width:1fr;}",
            "m{border-top-width:1foo;}",
            "n{border-top-width:0deg;}",
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
        1103,
        concat!(
            "a{border-top-width:/**/medium/**/!important;}",
            "b{border-top-width:/**/-0px/**/!important;}",
            "c{border-top-width:+ 0;}",
            "d{border-top-width:+/**/0;}",
            "e{border-top-width:- 1px;}",
            "f{border-top-width:-/**/1px;}",
            "g{border-top-width:1px 2px;}",
            "h{border-top-width:thin 1px;}",
            "i{border-top-width:1px thick;}",
            "j{border-top-width:(1px);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Medium,
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
fn sole_functions_are_unsupported_without_length_or_numeric_evaluation() {
    let result = qualify(
        1104,
        concat!(
            "a{border-top-width:calc(10px);}",
            "b{border-top-width:calc(-1px);}",
            "c{border-top-width:min(1px,2px);}",
            "d{border-top-width:max(0px,2px);}",
            "e{border-top-width:clamp(0px,1px,2px);}",
            "f{border-top-width:anchor-size(width);}",
            "g{border-top-width:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 7]);

    let mixed = qualify(
        1105,
        concat!(
            "a{border-top-width:calc(1px) 2px;}",
            "b{border-top-width:1px foo();}",
            "c{border-top-width:foo() 1px;}",
        ),
    );
    assert_expected(&mixed, &[ExpectedOutcome::Invalid; 3]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        1106,
        concat!(
            "a{border-top-width:initial;}",
            "b{border-top-width:inherit;}",
            "c{border-top-width:unset;}",
            "d{border-top-width:revert;}",
            "e{border-top-width:revert-layer;}",
            "f{border-top-width:revert-rule;}",
            "g{border-top-width:1px initial;}",
            "h{border-top-width:initial thin;}",
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
        1107,
        concat!(
            "a{border-top-width:var(--w);}",
            "b{border-top-width:env(w);}",
            "c{border-top-width:attr(data-w);}",
            "d{border-top-width:--w();}",
            "e{border-top-width:-1px var(--w);}",
            "f{border-top-width:thin var(--w);}",
            "g{border-top-width:calc(var(--w));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 7]);

    let whole = qualify(
        1108,
        concat!(
            "a{border-top-width:first-valid(1px,thin);}",
            "b{border-top-width:cycle(1px,2px);}",
            "c{border-top-width:interpolate(1px,0:0,1:1);}",
            "d{border-top-width:first-valid(1px,thin) 2px;}",
            "e{border-top-width:1px first-valid(0,1px);}",
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
        1109,
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
            "n{direction:rtl;}",
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
    assert_eq!(result.direction_observations()[1].occurrence_index(), 13);
    assert_eq!(
        result.border_top_width_observations()[0].outcome(),
        CssBorderTopWidthQualificationOutcome::Qualified(CssBorderTopWidthValue::Thick)
    );
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(1110, "a{border-top-width:thin;}b{border-top-width:thin;}");
    assert_expected(&result, &[ExpectedOutcome::Thin, ExpectedOutcome::Thin]);
    assert_eq!(
        result.border_top_width_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.border_top_width_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.border_top_width_observations()[0]
            .placement()
            .context_id(),
        result.border_top_width_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1111, "@font-face{border-top-width:1px;}"),
        (1112, "@page{border-top-width:1px;}"),
        (1113, "@page{@top-left{border-top-width:1px;}}"),
        (1114, "@keyframes k{from{border-top-width:1px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.border_top_width_observations().is_empty(),
            "nonordinary declaration context produced a border-top-width observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1115,
        "a{border-top-width:thin;border-top-width:1px;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::Thin]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{border-top-width:thin;}",
        "b{border-top-width:-0px;}",
        "c{border-top-width:calc(2px);}",
        "d{border-top-width:var(--w);}",
        "e{perspective:1px;}",
        "f{opacity:-50%;}",
        "g{flex-grow:.5;}",
        "h{direction:ltr;}",
        "i{column-count:2;}",
        "j{z-index:auto;}",
    );
    let first = qualify(1116, css);
    let repeated = qualify(1116, css);
    let another_source = qualify(1117, css);

    assert_eq!(
        first.border_top_width_observations(),
        repeated.border_top_width_observations()
    );
    assert_eq!(
        first.border_top_width_observations(),
        another_source.border_top_width_observations()
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
}
