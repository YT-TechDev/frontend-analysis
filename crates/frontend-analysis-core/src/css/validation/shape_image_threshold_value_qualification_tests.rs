use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssShapeImageThresholdQualificationOutcome, CssShapeImageThresholdUnsupportedReason,
    CssShapeImageThresholdValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    DirectNumber,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssShapeImageThresholdQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectNumber => CssShapeImageThresholdQualificationOutcome::Qualified(
            CssShapeImageThresholdValue::DirectNumberLiteral,
        ),
        ExpectedOutcome::DirectPercentage => CssShapeImageThresholdQualificationOutcome::Qualified(
            CssShapeImageThresholdValue::DirectPercentageLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssShapeImageThresholdQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssShapeImageThresholdQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeImageThresholdUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssShapeImageThresholdQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeImageThresholdUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssShapeImageThresholdQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeImageThresholdUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssShapeImageThresholdQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssShapeImageThresholdUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .shape_image_threshold_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_number_percentage_union_keeps_out_of_range_literals_qualified() {
    let result = qualify(
        940,
        concat!(
            "a{shape-image-threshold:0;}",
            "b{shape-image-threshold:1;}",
            "c{shape-image-threshold:.5;}",
            "d{shape-image-threshold:1.5;}",
            "e{shape-image-threshold:-1;}",
            "f{shape-image-threshold:1e100;}",
            "g{shape-image-threshold:-1e100;}",
            "h{shape-image-threshold:+.25;}",
            "i{shape-image-threshold:0%;}",
            "j{shape-image-threshold:50%;}",
            "k{shape-image-threshold:100%;}",
            "l{shape-image-threshold:120%;}",
            "m{shape-image-threshold:-50%;}",
            "n{shape-image-threshold:+25%;}",
            "o{shape-image-threshold:1px;}",
            "p{shape-image-threshold:\"1\";}",
            "q{shape-image-threshold:auto;}",
            "r{shape-image-threshold:foo;}",
            "s{shape-image-threshold:;}",
            "t{color:1;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
            ExpectedOutcome::DirectNumber,
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
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn comments_priority_and_separated_signs_use_retained_token_boundaries() {
    let result = qualify(
        941,
        concat!(
            "a{shape-image-threshold:/**/120%/**/!important;}",
            "b{shape-image-threshold:/**/-1/**/!important;}",
            "c{shape-image-threshold:+ 1;}",
            "d{shape-image-threshold:+/**/1;}",
            "e{shape-image-threshold:- 50%;}",
            "f{shape-image-threshold:-/**/50%;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectNumber,
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
fn direct_cardinality_and_function_boundaries_are_explicit() {
    let result = qualify(
        942,
        concat!(
            "a{shape-image-threshold:1 2;}",
            "b{shape-image-threshold:50% 1;}",
            "c{shape-image-threshold:1 50%;}",
            "d{shape-image-threshold:(1);}",
            "e{shape-image-threshold:foo() 1;}",
            "f{shape-image-threshold:1 foo();}",
            "g{shape-image-threshold:calc(1) 50%;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 7]);
}

#[test]
fn sole_ordinary_and_numeric_functions_are_unsupported_without_evaluation() {
    let result = qualify(
        943,
        concat!(
            "a{shape-image-threshold:calc(1);}",
            "b{shape-image-threshold:calc(-1);}",
            "c{shape-image-threshold:calc(120%);}",
            "d{shape-image-threshold:min(0,1);}",
            "e{shape-image-threshold:max(-1,2);}",
            "f{shape-image-threshold:clamp(-1,.5,2);}",
            "g{shape-image-threshold:calc(1px);}",
            "h{shape-image-threshold:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 8]);
}

#[test]
fn css_wide_deferred_and_whole_value_provenance_stays_distinct() {
    let css_wide = qualify(
        944,
        concat!(
            "a{shape-image-threshold:initial;}",
            "b{shape-image-threshold:inherit;}",
            "c{shape-image-threshold:unset;}",
            "d{shape-image-threshold:revert;}",
            "e{shape-image-threshold:revert-layer;}",
            "f{shape-image-threshold:revert-rule;}",
            "g{shape-image-threshold:1 initial;}",
            "h{shape-image-threshold:initial 50%;}",
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
        945,
        concat!(
            "a{shape-image-threshold:var(--threshold);}",
            "b{shape-image-threshold:env(threshold);}",
            "c{shape-image-threshold:attr(data-threshold);}",
            "d{shape-image-threshold:--threshold();}",
            "e{shape-image-threshold:-1 var(--threshold);}",
            "f{shape-image-threshold:120% var(--threshold);}",
            "g{shape-image-threshold:calc(var(--threshold));}",
        ),
    );
    assert_expected(&deferred, &[ExpectedOutcome::UnsupportedDeferred; 7]);

    let whole = qualify(
        946,
        concat!(
            "a{shape-image-threshold:first-valid(1,50%);}",
            "b{shape-image-threshold:cycle(1,50%);}",
            "c{shape-image-threshold:interpolate(50%,0:0,1:1);}",
            "d{shape-image-threshold:first-valid(1,50%) 2;}",
            "e{shape-image-threshold:1 first-valid(0,1);}",
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
        947,
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
            "l{direction:rtl;}",
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

    assert_eq!(result.direction_observations()[0].occurrence_index(), 0);
    assert_eq!(result.box_sizing_observations()[0].occurrence_index(), 1);
    assert_eq!(result.isolation_observations()[0].occurrence_index(), 2);
    assert_eq!(result.order_observations()[0].occurrence_index(), 3);
    assert_eq!(result.scroll_snap_align_observations()[0].occurrence_index(), 4);
    assert_eq!(result.z_index_observations()[0].occurrence_index(), 5);
    assert_eq!(result.column_count_observations()[0].occurrence_index(), 6);
    assert_eq!(result.flex_grow_observations()[0].occurrence_index(), 7);
    assert_eq!(result.flex_shrink_observations()[0].occurrence_index(), 8);
    assert_eq!(result.opacity_observations()[0].occurrence_index(), 9);
    assert_eq!(
        result.shape_image_threshold_observations()[0].occurrence_index(),
        10
    );
    assert_eq!(result.direction_observations()[1].occurrence_index(), 11);
    assert_eq!(
        result.shape_image_threshold_observations()[0].outcome(),
        CssShapeImageThresholdQualificationOutcome::Qualified(
            CssShapeImageThresholdValue::DirectPercentageLiteral
        )
    );
}

#[test]
fn duplicate_placements_and_nonordinary_contexts_stay_separate() {
    let result = qualify(
        948,
        "a{shape-image-threshold:50%;}b{shape-image-threshold:50%;}",
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
        ],
    );
    assert_eq!(
        result.shape_image_threshold_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.shape_image_threshold_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.shape_image_threshold_observations()[0]
            .placement()
            .context_id(),
        result.shape_image_threshold_observations()[1]
            .placement()
            .context_id(),
    );

    for (source_id, css) in [
        (949, "@font-face{shape-image-threshold:1;}"),
        (950, "@page{shape-image-threshold:1;}"),
        (951, "@page{@top-left{shape-image-threshold:1;}}"),
        (952, "@keyframes k{from{shape-image-threshold:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.shape_image_threshold_observations().is_empty(),
            "nonordinary declaration context produced a shape-image-threshold observation for {css:?}"
        );
    }
}

#[test]
fn incomplete_prefix_and_repeated_cross_source_runs_preserve_lifecycle_and_determinism() {
    let incomplete = qualify_with_limits(
        953,
        "a{shape-image-threshold:120%;shape-image-threshold:-1;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[ExpectedOutcome::DirectPercentage]);
    assert_eq!(incomplete.upstream_parser_result().occurrences().len(), 1);

    let css = concat!(
        "a{shape-image-threshold:-1;}",
        "b{shape-image-threshold:120%;}",
        "c{shape-image-threshold:calc(2);}",
        "d{shape-image-threshold:var(--threshold);}",
        "e{opacity:-50%;}",
        "f{flex-grow:.5;}",
        "g{flex-shrink:.5;}",
        "h{direction:ltr;}",
        "i{column-count:2;}",
        "j{z-index:auto;}",
    );
    let first = qualify(954, css);
    let repeated = qualify(954, css);
    let another_source = qualify(955, css);

    assert_eq!(
        first.shape_image_threshold_observations(),
        repeated.shape_image_threshold_observations()
    );
    assert_eq!(
        first.shape_image_threshold_observations(),
        another_source.shape_image_threshold_observations()
    );
    assert_eq!(first.opacity_observations(), repeated.opacity_observations());
    assert_eq!(first.flex_grow_observations(), repeated.flex_grow_observations());
    assert_eq!(
        first.flex_shrink_observations(),
        repeated.flex_shrink_observations()
    );
    assert_eq!(first.direction_observations(), repeated.direction_observations());
    assert_eq!(
        first.column_count_observations(),
        repeated.column_count_observations()
    );
    assert_eq!(first.z_index_observations(), repeated.z_index_observations());
}
