use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssOpacityQualificationOutcome, CssOpacityUnsupportedReason, CssOpacityValue,
    CssValueQualificationRunResult, run,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssOpacityQualificationOutcome {
    match expected {
        ExpectedOutcome::DirectNumber => {
            CssOpacityQualificationOutcome::Qualified(CssOpacityValue::DirectNumberLiteral)
        }
        ExpectedOutcome::DirectPercentage => {
            CssOpacityQualificationOutcome::Qualified(CssOpacityValue::DirectPercentageLiteral)
        }
        ExpectedOutcome::Invalid => CssOpacityQualificationOutcome::InvalidForSelectedValueGrammar,
        ExpectedOutcome::UnsupportedCssWide => {
            CssOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOpacityUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOpacityUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOpacityUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssOpacityQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssOpacityUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .opacity_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_number_percentage_union_keeps_out_of_range_literals_qualified() {
    let result = qualify(
        900,
        concat!(
            "a{opacity:0;}",
            "b{opacity:1;}",
            "c{opacity:.5;}",
            "d{opacity:1.5;}",
            "e{opacity:-1;}",
            "f{opacity:1e100;}",
            "g{opacity:-1e100;}",
            "h{opacity:+.25;}",
            "i{opacity:0%;}",
            "j{opacity:50%;}",
            "k{opacity:100%;}",
            "l{opacity:120%;}",
            "m{opacity:-50%;}",
            "n{opacity:+25%;}",
            "o{opacity:1px;}",
            "p{opacity:\"1\";}",
            "q{opacity:auto;}",
            "r{opacity:foo;}",
            "s{opacity:;}",
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
        901,
        concat!(
            "a{opacity:/**/120%/**/!important;}",
            "b{opacity:/**/-1/**/!important;}",
            "c{opacity:+ 1;}",
            "d{opacity:+/**/1;}",
            "e{opacity:- 50%;}",
            "f{opacity:-/**/50%;}",
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
fn direct_cardinality_mismatches_are_invalid() {
    let result = qualify(
        902,
        concat!(
            "a{opacity:1 2;}",
            "b{opacity:50% 1;}",
            "c{opacity:1 50%;}",
            "d{opacity:(1);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

#[test]
fn sole_ordinary_and_numeric_functions_are_unsupported_without_evaluation() {
    let result = qualify(
        903,
        concat!(
            "a{opacity:calc(1);}",
            "b{opacity:calc(-1);}",
            "c{opacity:calc(120%);}",
            "d{opacity:min(0,1);}",
            "e{opacity:max(-1,2);}",
            "f{opacity:clamp(-1,.5,2);}",
            "g{opacity:calc(1px);}",
            "h{opacity:foo();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedFunction; 8]);
}

#[test]
fn ordinary_function_plus_component_is_directly_invalid() {
    let result = qualify(
        904,
        concat!(
            "a{opacity:foo() 1;}",
            "b{opacity:1 foo();}",
            "c{opacity:calc(1) 50%;}",
            "d{opacity:(foo());}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

#[test]
fn css_wide_keywords_are_unsupported_only_as_the_whole_value() {
    let result = qualify(
        905,
        concat!(
            "a{opacity:initial;}",
            "b{opacity:inherit;}",
            "c{opacity:unset;}",
            "d{opacity:revert;}",
            "e{opacity:revert-layer;}",
            "f{opacity:revert-rule;}",
            "g{opacity:1 initial;}",
            "h{opacity:initial 50%;}",
        ),
    );

    assert_expected(
        &result,
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
}

#[test]
fn deferred_substitution_remains_conservatively_fail_open_wherever_it_occurs() {
    let result = qualify(
        906,
        concat!(
            "a{opacity:var(--opacity);}",
            "b{opacity:env(opacity);}",
            "c{opacity:attr(data-opacity);}",
            "d{opacity:--opacity();}",
            "e{opacity:-1 var(--opacity);}",
            "f{opacity:120% var(--opacity);}",
            "g{opacity:calc(var(--opacity));}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 7]);
}

#[test]
fn generic_whole_value_functions_keep_entire_value_placement_boundary() {
    let result = qualify(
        907,
        concat!(
            "a{opacity:first-valid(1,50%);}",
            "b{opacity:cycle(1,50%);}",
            "c{opacity:interpolate(50%,0:0,1:1);}",
            "d{opacity:first-valid(1,50%) 2;}",
            "e{opacity:1 first-valid(0,1);}",
        ),
    );

    assert_expected(
        &result,
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
        908,
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
            "k{direction:rtl;}",
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
    assert_eq!(result.direction_observations()[1].occurrence_index(), 10);
    assert_eq!(
        result.opacity_observations()[0].outcome(),
        CssOpacityQualificationOutcome::Qualified(CssOpacityValue::DirectPercentageLiteral)
    );
}

#[test]
fn duplicate_opacity_declarations_keep_distinct_run_local_placement() {
    let result = qualify(909, "a{opacity:50%;}b{opacity:50%;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::DirectPercentage,
            ExpectedOutcome::DirectPercentage,
        ],
    );
    assert_eq!(result.opacity_observations()[0].occurrence_index(), 0);
    assert_eq!(result.opacity_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.opacity_observations()[0].placement().context_id(),
        result.opacity_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_opacity_observations() {
    for (source_id, css) in [
        (910, "@font-face{opacity:1;}"),
        (911, "@page{opacity:1;}"),
        (912, "@page{@top-left{opacity:1;}}"),
        (913, "@keyframes k{from{opacity:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.opacity_observations().is_empty(),
            "nonordinary declaration context produced an opacity observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_opacity_prefix_and_incomplete_completion() {
    let result = qualify_with_limits(
        920,
        "a{opacity:120%;opacity:-1;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::DirectPercentage]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_opacity_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{opacity:-1;}",
        "b{opacity:120%;}",
        "c{opacity:calc(2);}",
        "d{opacity:var(--opacity);}",
        "e{flex-grow:.5;}",
        "f{flex-shrink:.5;}",
        "g{direction:ltr;}",
        "h{column-count:2;}",
        "i{z-index:auto;}",
    );
    let first = qualify(930, css);
    let repeated = qualify(930, css);
    let another_source = qualify(931, css);

    assert_eq!(
        first.opacity_observations(),
        repeated.opacity_observations()
    );
    assert_eq!(
        first.opacity_observations(),
        another_source.opacity_observations()
    );
    assert_eq!(
        first.flex_grow_observations(),
        repeated.flex_grow_observations()
    );
    assert_eq!(
        first.flex_shrink_observations(),
        repeated.flex_shrink_observations()
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
