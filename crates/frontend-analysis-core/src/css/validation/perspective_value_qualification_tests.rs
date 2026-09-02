use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssPerspectiveQualificationOutcome, CssPerspectiveUnsupportedReason, CssPerspectiveValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    None,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssPerspectiveQualificationOutcome {
    match expected {
        ExpectedOutcome::None => {
            CssPerspectiveQualificationOutcome::Qualified(CssPerspectiveValue::None)
        }
        ExpectedOutcome::DirectLength => {
            CssPerspectiveQualificationOutcome::Qualified(CssPerspectiveValue::DirectLengthLiteral)
        }
        ExpectedOutcome::Invalid => {
            CssPerspectiveQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssPerspectiveQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPerspectiveUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssPerspectiveQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPerspectiveUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssPerspectiveQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPerspectiveUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssPerspectiveQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPerspectiveUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .perspective_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_keyword_unitless_zero_and_direct_type_boundaries_are_explicit() {
    let result = qualify(
        1000,
        concat!(
            "a{perspective:none;}",
            "b{perspective:0;}",
            "c{perspective:+0;}",
            "d{perspective:-0;}",
            "e{perspective:.0;}",
            "f{perspective:+.0;}",
            "g{perspective:-.0;}",
            "h{perspective:0.0;}",
            "i{perspective:-0.0;}",
            "j{perspective:0e100;}",
            "k{perspective:-0e100;}",
            "l{perspective:1;}",
            "m{perspective:-1;}",
            "n{perspective:.5;}",
            "o{perspective:-.5;}",
            "p{perspective:1e0;}",
            "q{perspective:80%;}",
            "r{perspective:0%;}",
            "s{perspective:auto;}",
            "t{perspective:\"1px\";}",
            "u{perspective:;}",
            "v{color:1px;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::None,
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
        css.push_str(&format!(".u{index}{{perspective:1{unit};}}"));
    }
    css.push_str(".upper{perspective:1Q;}.escaped{perspective:1p\\78;}");

    let result = qualify(1001, &css);
    assert_expected(
        &result,
        &vec![ExpectedOutcome::DirectLength; units.len() + 2],
    );
}

#[test]
fn direct_dimension_range_and_unit_mismatches_are_source_provable() {
    let result = qualify(
        1002,
        concat!(
            "a{perspective:0px;}",
            "b{perspective:-0px;}",
            "c{perspective:-0e100px;}",
            "d{perspective:1px;}",
            "e{perspective:.5em;}",
            "f{perspective:1e100cqi;}",
            "g{perspective:-1px;}",
            "h{perspective:-.5em;}",
            "i{perspective:-1e-999px;}",
            "j{perspective:1deg;}",
            "k{perspective:1s;}",
            "l{perspective:1fr;}",
            "m{perspective:1foo;}",
            "n{perspective:0deg;}",
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
        1003,
        concat!(
            "a{perspective:/**/1px/**/!important;}",
            "b{perspective:/**/-0px/**/!important;}",
            "c{perspective:+ 0;}",
            "d{perspective:+/**/0;}",
            "e{perspective:- 1px;}",
            "f{perspective:-/**/1px;}",
            "g{perspective:1px 2px;}",
            "h{perspective:none 1px;}",
            "i{perspective:(1px);}",
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
        1004,
        concat!(
            "a{perspective:calc(10px);}",
            "b{perspective:calc(-1px);}",
            "c{perspective:min(1px,2px);}",
            "d{perspective:max(0px,2px);}",
            "e{perspective:clamp(0px,1px,2px);}",
            "f{perspective:anchor-size(width);}",
            "g{perspective:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 7]);

    let mixed = qualify(
        1005,
        concat!(
            "a{perspective:calc(1px) 2px;}",
            "b{perspective:1px foo();}",
            "c{perspective:foo() 1px;}",
        ),
    );
    assert_expected(&mixed, &[ExpectedOutcome::Invalid; 3]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        1006,
        concat!(
            "a{perspective:initial;}",
            "b{perspective:inherit;}",
            "c{perspective:unset;}",
            "d{perspective:revert;}",
            "e{perspective:revert-layer;}",
            "f{perspective:revert-rule;}",
            "g{perspective:1px initial;}",
            "h{perspective:initial 1px;}",
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
        1007,
        concat!(
            "a{perspective:var(--p);}",
            "b{perspective:env(p);}",
            "c{perspective:attr(data-p);}",
            "d{perspective:--p();}",
            "e{perspective:-1px var(--p);}",
            "f{perspective:1px var(--p);}",
            "g{perspective:calc(var(--p));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 7]);

    let whole = qualify(
        1008,
        concat!(
            "a{perspective:first-valid(1px,none);}",
            "b{perspective:cycle(1px,2px);}",
            "c{perspective:interpolate(1px,0:0,1:1);}",
            "d{perspective:first-valid(1px,none) 2px;}",
            "e{perspective:1px first-valid(0,1px);}",
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
        1009,
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
            "m{direction:rtl;}",
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
    assert_eq!(result.direction_observations()[1].occurrence_index(), 12);
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(1010, "a{perspective:1px;}b{perspective:1px;}");
    assert_expected(
        &result,
        &[ExpectedOutcome::DirectLength, ExpectedOutcome::DirectLength],
    );
    assert_eq!(result.perspective_observations()[0].occurrence_index(), 0);
    assert_eq!(result.perspective_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.perspective_observations()[0]
            .placement()
            .context_id(),
        result.perspective_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (1011, "@font-face{perspective:1px;}"),
        (1012, "@page{perspective:1px;}"),
        (1013, "@page{@top-left{perspective:1px;}}"),
        (1014, "@keyframes k{from{perspective:1px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.perspective_observations().is_empty(),
            "nonordinary declaration context produced a perspective observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        1015,
        "a{perspective:1px;perspective:none;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::DirectLength]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{perspective:none;}",
        "b{perspective:-0px;}",
        "c{perspective:-1px;}",
        "d{perspective:calc(2px);}",
        "e{perspective:var(--p);}",
        "f{shape-image-threshold:120%;}",
        "g{opacity:-50%;}",
        "h{flex-grow:.5;}",
        "i{direction:ltr;}",
        "j{column-count:2;}",
        "k{z-index:auto;}",
    );
    let first = qualify(1016, css);
    let repeated = qualify(1016, css);
    let another_source = qualify(1017, css);

    assert_eq!(
        first.perspective_observations(),
        repeated.perspective_observations()
    );
    assert_eq!(
        first.perspective_observations(),
        another_source.perspective_observations()
    );
    assert_eq!(
        first.shape_image_threshold_observations(),
        repeated.shape_image_threshold_observations()
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
