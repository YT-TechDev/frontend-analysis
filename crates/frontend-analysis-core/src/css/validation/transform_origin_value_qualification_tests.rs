use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssNumberSign, CssNumericValue, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTransformOriginComponent, CssTransformOriginComponentEvidenceRef, CssTransformOriginKeyword,
    CssTransformOriginQualificationOutcome, CssTransformOriginUnsupportedReason,
    CssTransformOriginValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

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

fn outcome_at(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &CssTransformOriginQualificationOutcome {
    result.transform_origin_observations()[index].outcome()
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssTransformOriginQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssTransformOriginUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssTransformOriginQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified components. Panics if the
/// observation at `index` is not `Qualified(Components(_))`.
fn qualified_components(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &[CssTransformOriginComponent] {
    match outcome_at(result, index) {
        CssTransformOriginQualificationOutcome::Qualified(CssTransformOriginValue::Components(
            components,
        )) => components,
        other => panic!("expected Qualified(Components(_)) at index {index}, got {other:?}"),
    }
}

fn assert_qualified_keywords(
    result: &CssValueQualificationRunResult,
    index: usize,
    expected: &[CssTransformOriginKeyword],
) {
    let components = qualified_components(result, index);
    let actual: Vec<CssTransformOriginKeyword> = components
        .iter()
        .map(|component| match component {
            CssTransformOriginComponent::Keyword(keyword) => *keyword,
            other => panic!("expected Keyword component at index {index}, got {other:?}"),
        })
        .collect();
    assert_eq!(actual, expected, "authored keyword order at index {index}");
}

/// Checks only a value's leading components against expected keywords,
/// leaving any trailing non-keyword (e.g. Z `Length`) component unchecked.
fn assert_leading_keywords(
    result: &CssValueQualificationRunResult,
    index: usize,
    expected: &[CssTransformOriginKeyword],
) {
    let components = qualified_components(result, index);
    for (component_index, expected_keyword) in expected.iter().enumerate() {
        match components[component_index] {
            CssTransformOriginComponent::Keyword(keyword) => {
                assert_eq!(
                    keyword, *expected_keyword,
                    "authored keyword order at index {index}/{component_index}"
                );
            }
            other => panic!(
                "expected Keyword component at index {index}/{component_index}, got {other:?}"
            ),
        }
    }
}

fn numeric_evidence_ref(
    result: &CssValueQualificationRunResult,
    index: usize,
    component_index: usize,
) -> CssTransformOriginComponentEvidenceRef {
    match qualified_components(result, index)[component_index] {
        CssTransformOriginComponent::Length(evidence)
        | CssTransformOriginComponent::Percentage(evidence) => evidence,
        other => {
            panic!("component at {index}/{component_index} is {other:?}, not Length/Percentage")
        }
    }
}

fn component_token(
    result: &CssValueQualificationRunResult,
    index: usize,
    component_index: usize,
) -> &CssTokenKind {
    let evidence = numeric_evidence_ref(result, index, component_index);
    result
        .transform_origin_component_token(evidence)
        .unwrap_or_else(|| {
            panic!("component evidence at {index}/{component_index} did not resolve")
        })
}

fn component_numeric_value(
    result: &CssValueQualificationRunResult,
    index: usize,
    component_index: usize,
) -> &CssNumericValue {
    match component_token(result, index, component_index) {
        CssTokenKind::Number { value, .. }
        | CssTokenKind::Dimension { value, .. }
        | CssTokenKind::Percentage { value } => value,
        other => panic!(
            "component evidence at {index}/{component_index} resolved to {other:?}, not Number/Dimension/Percentage"
        ),
    }
}

// A. One-component keywords: all five, ASCII-case-insensitive and
// escape-equivalent.

#[test]
fn one_component_keywords_qualify_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        608100,
        concat!(
            "a{transform-origin:left;}",
            "b{transform-origin:LEFT;}",
            "c{transform-origin:Left;}",
            "d{transform-origin:\\6c eft;}",
            "e{transform-origin:right;}",
            "f{transform-origin:top;}",
            "g{transform-origin:bottom;}",
            "h{transform-origin:center;}",
        ),
    );

    let expected = [
        CssTransformOriginKeyword::Left,
        CssTransformOriginKeyword::Left,
        CssTransformOriginKeyword::Left,
        CssTransformOriginKeyword::Left,
        CssTransformOriginKeyword::Right,
        CssTransformOriginKeyword::Top,
        CssTransformOriginKeyword::Bottom,
        CssTransformOriginKeyword::Center,
    ];
    for (index, keyword) in expected.into_iter().enumerate() {
        assert_qualified_keywords(&result, index, &[keyword]);
    }
}

// B. One-component direct Length Dimension.

#[test]
fn one_component_direct_length_dimension_qualifies() {
    let result = qualify(
        608101,
        concat!(
            "a{transform-origin:0px;}",
            "b{transform-origin:-0px;}",
            "c{transform-origin:1px;}",
            "d{transform-origin:-10px;}",
            "e{transform-origin:.5em;}",
            "f{transform-origin:-.5em;}",
            "g{transform-origin:1e100cqi;}",
            "h{transform-origin:-1e-999px;}",
        ),
    );

    for index in 0..8 {
        assert_eq!(qualified_components(&result, index).len(), 1);
        assert!(matches!(
            qualified_components(&result, index)[0],
            CssTransformOriginComponent::Length(_)
        ));
        assert!(matches!(
            component_token(&result, index, 0),
            CssTokenKind::Dimension { .. }
        ));
    }
}

// C. Exact-zero unitless Length in every accepted spelling; non-zero
// unitless Number remains Invalid.

#[test]
fn unitless_exact_zero_qualifies_as_length_in_every_accepted_spelling() {
    let result = qualify(
        608102,
        concat!(
            "a{transform-origin:0;}",
            "b{transform-origin:+0;}",
            "c{transform-origin:-0;}",
            "d{transform-origin:.0;}",
            "e{transform-origin:-.0;}",
            "f{transform-origin:0.0;}",
            "g{transform-origin:-0.0;}",
            "h{transform-origin:0e100;}",
            "i{transform-origin:-0e100;}",
        ),
    );

    for index in 0..9 {
        assert!(matches!(
            qualified_components(&result, index)[0],
            CssTransformOriginComponent::Length(_)
        ));
        assert!(matches!(
            component_token(&result, index, 0),
            CssTokenKind::Number { .. }
        ));
    }

    assert_eq!(
        component_numeric_value(&result, 1, 0).sign(),
        Some(CssNumberSign::Plus)
    );
    assert_eq!(
        component_numeric_value(&result, 2, 0).sign(),
        Some(CssNumberSign::Minus)
    );
}

#[test]
fn non_zero_unitless_number_is_invalid() {
    let result = qualify(
        608103,
        concat!(
            "a{transform-origin:1;}",
            "b{transform-origin:-1;}",
            "c{transform-origin:.5;}",
            "d{transform-origin:10e2;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// D. One-component Percentage.

#[test]
fn one_component_percentage_qualifies_and_remains_percentage() {
    let result = qualify(
        608104,
        concat!(
            "a{transform-origin:0%;}",
            "b{transform-origin:-0%;}",
            "c{transform-origin:50%;}",
            "d{transform-origin:-100%;}",
            "e{transform-origin:.5%;}",
            "f{transform-origin:-1e100%;}",
        ),
    );

    for index in 0..6 {
        assert!(matches!(
            qualified_components(&result, index)[0],
            CssTransformOriginComponent::Percentage(_)
        ));
        assert!(matches!(
            component_token(&result, index, 0),
            CssTokenKind::Percentage { .. }
        ));
    }
}

// E. 0 / 0px / 0% authored identity -- load-bearing.

#[test]
fn zero_zero_px_and_zero_percent_remain_authored_distinct() {
    let result = qualify(
        608105,
        concat!(
            "a{transform-origin:0;}",
            "b{transform-origin:0px;}",
            "c{transform-origin:0%;}",
        ),
    );

    assert!(matches!(
        qualified_components(&result, 0)[0],
        CssTransformOriginComponent::Length(_)
    ));
    assert!(matches!(
        component_token(&result, 0, 0),
        CssTokenKind::Number { .. }
    ));

    assert!(matches!(
        qualified_components(&result, 1)[0],
        CssTransformOriginComponent::Length(_)
    ));
    assert!(matches!(
        component_token(&result, 1, 0),
        CssTokenKind::Dimension { .. }
    ));

    assert!(matches!(
        qualified_components(&result, 2)[0],
        CssTransformOriginComponent::Percentage(_)
    ));
    assert!(matches!(
        component_token(&result, 2, 0),
        CssTokenKind::Percentage { .. }
    ));
}

// F. Ordered two-component valid forms.

#[test]
fn ordered_two_component_forms_qualify() {
    let result = qualify(
        608106,
        concat!(
            "a{transform-origin:left 20px;}",
            "b{transform-origin:right 20%;}",
            "c{transform-origin:center 20px;}",
            "d{transform-origin:20px top;}",
            "e{transform-origin:20% center;}",
            "f{transform-origin:20px bottom;}",
            "g{transform-origin:10px 20px;}",
            "h{transform-origin:10px 20%;}",
            "i{transform-origin:10% 20px;}",
            "j{transform-origin:10% 20%;}",
        ),
    );

    for index in 0..10 {
        assert_eq!(qualified_components(&result, index).len(), 2);
    }
}

// G. Critical ordered asymmetry -- load-bearing, role-sensitive boundary.

#[test]
fn ordered_role_sensitive_asymmetry_is_preserved() {
    let result = qualify(
        608107,
        concat!(
            "a{transform-origin:left 20px;}",
            "b{transform-origin:top 20px;}",
            "c{transform-origin:20px top;}",
            "d{transform-origin:20px left;}",
            "e{transform-origin:right 20px;}",
            "f{transform-origin:bottom 20px;}",
            "g{transform-origin:20px bottom;}",
            "h{transform-origin:20px right;}",
        ),
    );

    assert_eq!(qualified_components(&result, 0).len(), 2); // left 20px
    assert_invalid(&result, 1); // top 20px
    assert_eq!(qualified_components(&result, 2).len(), 2); // 20px top
    assert_invalid(&result, 3); // 20px left
    assert_eq!(qualified_components(&result, 4).len(), 2); // right 20px
    assert_invalid(&result, 5); // bottom 20px
    assert_eq!(qualified_components(&result, 6).len(), 2); // 20px bottom
    assert_invalid(&result, 7); // 20px right
}

// H. Keyword && valid reorderings -- authored source order preserved.

#[test]
fn keyword_and_and_reorderings_qualify_with_authored_order_preserved() {
    use CssTransformOriginKeyword::{Bottom, Center, Left, Right, Top};

    let result = qualify(
        608108,
        concat!(
            "a{transform-origin:left top;}",
            "b{transform-origin:top left;}",
            "c{transform-origin:right bottom;}",
            "d{transform-origin:bottom right;}",
            "e{transform-origin:center left;}",
            "f{transform-origin:left center;}",
            "g{transform-origin:center right;}",
            "h{transform-origin:right center;}",
            "i{transform-origin:center top;}",
            "j{transform-origin:top center;}",
            "k{transform-origin:center bottom;}",
            "l{transform-origin:bottom center;}",
            "m{transform-origin:center center;}",
        ),
    );

    let expected: [[CssTransformOriginKeyword; 2]; 13] = [
        [Left, Top],
        [Top, Left],
        [Right, Bottom],
        [Bottom, Right],
        [Center, Left],
        [Left, Center],
        [Center, Right],
        [Right, Center],
        [Center, Top],
        [Top, Center],
        [Center, Bottom],
        [Bottom, Center],
        [Center, Center],
    ];

    for (index, pair) in expected.iter().enumerate() {
        assert_qualified_keywords(&result, index, pair);
    }
}

// I. Keyword same-axis invalidity.

#[test]
fn same_axis_keyword_pairs_are_invalid() {
    let result = qualify(
        608109,
        concat!(
            "a{transform-origin:left right;}",
            "b{transform-origin:right left;}",
            "c{transform-origin:left left;}",
            "d{transform-origin:right right;}",
            "e{transform-origin:top bottom;}",
            "f{transform-origin:bottom top;}",
            "g{transform-origin:top top;}",
            "h{transform-origin:bottom bottom;}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// J. Three-component direct valid forms.

#[test]
fn three_component_direct_forms_qualify_with_z_length() {
    use CssTransformOriginKeyword::{Bottom, Left, Right, Top};

    let result = qualify(
        608110,
        concat!(
            "a{transform-origin:1px 2px 3px;}",
            "b{transform-origin:left top 3px;}",
            "c{transform-origin:top left 3px;}",
            "d{transform-origin:left 20px 3px;}",
            "e{transform-origin:20px top 3px;}",
            "f{transform-origin:center left 6px;}",
            "g{transform-origin:bottom right 7px;}",
            "h{transform-origin:1px 2px 0;}",
        ),
    );

    for index in 0..8 {
        let components = qualified_components(&result, index);
        assert_eq!(components.len(), 3);
        assert!(matches!(
            components[2],
            CssTransformOriginComponent::Length(_)
        ));
    }

    assert_leading_keywords(&result, 1, &[Left, Top]);
    assert_leading_keywords(&result, 2, &[Top, Left]);
    assert!(matches!(
        qualified_components(&result, 6)[1],
        CssTransformOriginComponent::Keyword(Right)
    ));
    assert!(matches!(
        qualified_components(&result, 6)[0],
        CssTransformOriginComponent::Keyword(Bottom)
    ));
}

// K. Third-slot Percentage rejection -- decisively invalid even at zero.

#[test]
fn third_slot_percentage_is_invalid_even_when_zero() {
    let result = qualify(
        608111,
        concat!(
            "a{transform-origin:1px 2px 3%;}",
            "b{transform-origin:1px 2px 0%;}",
            "c{transform-origin:left top 30%;}",
            "d{transform-origin:top left 0%;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// L. No default synthesis -- authored cardinality is exact.

#[test]
fn authored_cardinality_is_exact_with_no_synthesized_defaults() {
    let result = qualify(
        608112,
        concat!(
            "a{transform-origin:left;}",
            "b{transform-origin:left center;}",
            "c{transform-origin:left center 0;}",
        ),
    );

    assert_eq!(
        qualified_components(&result, 0).len(),
        1,
        "no synthesized center"
    );
    assert_eq!(
        qualified_components(&result, 1).len(),
        2,
        "no synthesized 0px"
    );
    assert_eq!(
        qualified_components(&result, 2).len(),
        3,
        "all three explicit"
    );
}

// M. Authored-order distinction.

#[test]
fn authored_keyword_order_distinguishes_otherwise_equivalent_pairs() {
    use CssTransformOriginKeyword::{Center, Left, Top};

    let result = qualify(
        608113,
        concat!(
            "a{transform-origin:top left;}",
            "b{transform-origin:left top;}",
            "c{transform-origin:center left;}",
            "d{transform-origin:left center;}",
        ),
    );

    assert_qualified_keywords(&result, 0, &[Top, Left]);
    assert_qualified_keywords(&result, 1, &[Left, Top]);
    assert_qualified_keywords(&result, 2, &[Center, Left]);
    assert_qualified_keywords(&result, 3, &[Left, Center]);
}

// N. Generic <position> widening rejection.

#[test]
fn generic_position_four_component_forms_are_invalid() {
    let result = qualify(
        608114,
        concat!(
            "a{transform-origin:right 20px bottom 30px;}",
            "b{transform-origin:bottom 10% right 20%;}",
            "c{transform-origin:right 30% top 60px;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// O. Wrong token classes.

#[test]
fn wrong_direct_token_classes_are_invalid() {
    let result = qualify(
        608115,
        concat!(
            "a{transform-origin:foo;}",
            "b{transform-origin:\"left\";}",
            "c{transform-origin:#abc;}",
            "d{transform-origin:1deg;}",
            "e{transform-origin:1s;}",
            "f{transform-origin:1fr;}",
            "g{transform-origin:10px foo;}",
        ),
    );

    for index in 0..7 {
        assert_invalid(&result, index);
    }
}

// P. Residual Function one-component.

#[test]
fn residual_function_one_component_is_unsupported() {
    let result = qualify(
        608116,
        concat!(
            "a{transform-origin:calc(10px);}",
            "b{transform-origin:calc(10%);}",
            "c{transform-origin:calc(10% + 10px);}",
            "d{transform-origin:foo();}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(
            &result,
            index,
            CssTransformOriginUnsupportedReason::FunctionValue,
        );
    }
}

// Q. Residual Function feasible two-component placement.

#[test]
fn residual_function_feasible_two_component_placement_is_unsupported() {
    let result = qualify(
        608117,
        concat!(
            "a{transform-origin:left calc(20px);}",
            "b{transform-origin:right calc(20px);}",
            "c{transform-origin:center calc(20px);}",
            "d{transform-origin:calc(20px) top;}",
            "e{transform-origin:calc(20px) center;}",
            "f{transform-origin:calc(20px) bottom;}",
            "g{transform-origin:calc(10px) calc(20px);}",
        ),
    );

    for index in 0..7 {
        assert_unsupported(
            &result,
            index,
            CssTransformOriginUnsupportedReason::FunctionValue,
        );
    }
}

// R. Residual Function impossible two-component placement -- load-bearing.

#[test]
fn residual_function_impossible_two_component_placement_is_invalid() {
    let result = qualify(
        608118,
        concat!(
            "a{transform-origin:top calc(20px);}",
            "b{transform-origin:bottom calc(20px);}",
            "c{transform-origin:calc(20px) left;}",
            "d{transform-origin:calc(20px) right;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// S. Residual Function feasible three-component placement.

#[test]
fn residual_function_feasible_three_component_placement_is_unsupported() {
    let result = qualify(
        608119,
        concat!(
            "a{transform-origin:left calc(20px) 30px;}",
            "b{transform-origin:calc(10px) top 30px;}",
            "c{transform-origin:left top calc(30px);}",
            "d{transform-origin:top left calc(30px);}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(
            &result,
            index,
            CssTransformOriginUnsupportedReason::FunctionValue,
        );
    }
}

// T. Residual Function impossible three-component placement.

#[test]
fn residual_function_impossible_three_component_placement_is_invalid() {
    let result = qualify(
        608120,
        concat!(
            "a{transform-origin:top calc(20px) 30px;}",
            "b{transform-origin:calc(20px) left 30px;}",
            "c{transform-origin:bottom calc(20px) 30px;}",
            "d{transform-origin:calc(20px) right 30px;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// U. Direct Z invalidity outranks Function uncertainty.

#[test]
fn decisive_third_slot_invalidity_outranks_function_ambiguity() {
    let result = qualify(
        608121,
        concat!(
            "a{transform-origin:left calc(20px) 30%;}",
            "b{transform-origin:calc(10px) top 0%;}",
        ),
    );

    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// V. Four or more components.

#[test]
fn four_or_more_components_are_invalid_even_with_a_residual_function() {
    let result = qualify(
        608122,
        concat!(
            "a{transform-origin:left top 10px 20px;}",
            "b{transform-origin:calc(10px) top 20px 30px;}",
        ),
    );

    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// W. Nested Function delimiter isolation.

#[test]
fn nested_function_comma_is_never_a_top_level_separator() {
    let result = qualify(608123, "a{transform-origin:left calc(min(10px, 20px));}");
    assert_unsupported(
        &result,
        0,
        CssTransformOriginUnsupportedReason::FunctionValue,
    );
}

// X. Top-level comma rejection.

#[test]
fn comma_separated_syntax_is_invalid() {
    let result = qualify(
        608124,
        concat!(
            "a{transform-origin:left, top;}",
            "b{transform-origin:left top, 10px;}",
            "c{transform-origin:10px, 20px;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// Y. Whole-value Function boundary.

#[test]
fn whole_value_function_is_unsupported_only_as_the_entire_value() {
    let result = qualify(608125, "a{transform-origin:first-valid(10px, 20px);}");
    assert_unsupported(
        &result,
        0,
        CssTransformOriginUnsupportedReason::WholeValueFunction,
    );
}

#[test]
fn embedded_whole_value_only_function_is_invalid_never_softened() {
    let result = qualify(
        608126,
        concat!(
            "a{transform-origin:left first-valid(20px);}",
            "b{transform-origin:left top first-valid(30px);}",
        ),
    );
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// Z. Deferred substitution boundary.

#[test]
fn deferred_substitution_precedes_direct_classification_everywhere() {
    let result = qualify(
        608127,
        concat!(
            "a{transform-origin:var(--x);}",
            "b{transform-origin:left var(--x);}",
            "c{transform-origin:var(--x) left;}",
            "d{transform-origin:left var(--x) 20px;}",
            "e{transform-origin:env(safe-area-inset-top);}",
            "f{transform-origin:attr(data-x);}",
            "g{transform-origin:--custom(10px);}",
        ),
    );

    for index in 0..7 {
        assert_unsupported(
            &result,
            index,
            CssTransformOriginUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// AA. CSS-wide keyword boundary.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        608128,
        concat!(
            "a{transform-origin:initial;}",
            "b{transform-origin:inherit;}",
            "c{transform-origin:unset;}",
            "d{transform-origin:revert;}",
            "e{transform-origin:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssTransformOriginUnsupportedReason::CssWideKeyword,
        );
    }

    let combined = qualify(
        608129,
        concat!(
            "a{transform-origin:left initial;}",
            "b{transform-origin:initial top;}",
            "c{transform-origin:left top revert;}",
        ),
    );
    for index in 0..3 {
        assert_invalid(&combined, index);
    }
}

// AB. Trivia: whitespace/comments never change grouping; empty/trivia-only
// values are invalid.

#[test]
fn comments_between_and_adjacent_to_components_behave_like_whitespace() {
    let spaced = qualify(608130, "a{transform-origin:left top 30px;}");
    let comment_padded = qualify(
        608131,
        "a{transform-origin:/*a*/left/*b*/ /*c*/top/*d*/ /*e*/30px/*f*/;}",
    );

    for result in [&spaced, &comment_padded] {
        assert_eq!(qualified_components(result, 0).len(), 3);
    }
}

#[test]
fn empty_and_trivia_only_values_are_invalid() {
    let result = qualify(
        608132,
        concat!("a{transform-origin:;}", "b{transform-origin: /**/  ;}",),
    );
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// AC. `!important` stays outside the authored value window.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(608133, "a{transform-origin:top left 10px !important;}");
    assert_eq!(qualified_components(&result, 0).len(), 3);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// AD. Duplicate declarations remain independent observations.

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(
        608134,
        "a{transform-origin:left;transform-origin:left center;transform-origin:left center 0;}",
    );

    assert_eq!(result.transform_origin_observations().len(), 3);
    assert_eq!(qualified_components(&result, 0).len(), 1);
    assert_eq!(qualified_components(&result, 1).len(), 2);
    assert_eq!(qualified_components(&result, 2).len(), 3);
}

// AE. Placement: only ordinary declaration placement enters this qualifier.

#[test]
fn nonordinary_contexts_do_not_enter_transform_origin_dispatch() {
    for (source_id, css) in [
        (608135, "@font-face{transform-origin:left;}"),
        (608136, "@page{transform-origin:left;}"),
        (608137, "@keyframes k{from{transform-origin:left;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.transform_origin_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

// AF. Incomplete upstream execution constrains qualification to the
// committed prefix.

#[test]
fn incomplete_upstream_execution_constrains_qualification_to_committed_prefix() {
    let incomplete = qualify_with_limits(
        608138,
        "a{transform-origin:left;}b{transform-origin:left center;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.transform_origin_observations().len(), 1);
    assert_eq!(qualified_components(&incomplete, 0).len(), 1);
}

// AG. Determinism.

#[test]
fn repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{transform-origin:top left -30px;}",
        "b{transform-origin:var(--x);}",
        "c{transform-origin:calc(10px);}",
        "d{transform-origin:first-valid(10px, 20px);}",
        "e{transform-origin:left top 30%;}",
    );
    let first = qualify(608139, css);
    let repeated = qualify(608139, css);
    let another_source = qualify(608140, css);

    assert_eq!(
        first.transform_origin_observations(),
        repeated.transform_origin_observations()
    );
    assert_eq!(
        first.transform_origin_observations(),
        another_source.transform_origin_observations()
    );
}

// AH. Cross-leaf isolation.

#[test]
fn cross_leaf_isolation_from_neighboring_qualified_properties() {
    let result = qualify(
        608141,
        concat!(
            "a{transform-origin:left;transform-origin:left center;}",
            "b{translate:10px;}",
            "c{scale:1;}",
            "d{rotate:45deg;}",
            "e{scroll-margin-top:1px;}",
            "f{word-spacing:1px;}",
        ),
    );

    assert_eq!(result.transform_origin_observations().len(), 2);
    assert_eq!(
        result.transform_origin_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.transform_origin_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.scroll_margin_top_observations().len(), 1);
    assert_eq!(result.word_spacing_observations().len(), 1);
}

// Signed-zero preservation: `-0` must resolve with a retained Minus sign,
// never normalized away, when correctly positioned across all three slots.

#[test]
fn signed_zero_is_preserved_across_all_three_slots() {
    let result = qualify(608142, "a{transform-origin:-0 top -0;}");
    let components = qualified_components(&result, 0);
    assert_eq!(components.len(), 3);
    assert!(matches!(
        components[0],
        CssTransformOriginComponent::Length(_)
    ));
    assert!(matches!(
        components[2],
        CssTransformOriginComponent::Length(_)
    ));
    assert_eq!(
        component_numeric_value(&result, 0, 0).sign(),
        Some(CssNumberSign::Minus)
    );
    assert_eq!(
        component_numeric_value(&result, 0, 2).sign(),
        Some(CssNumberSign::Minus)
    );
}

// Evidence-ref sanity: evidence never points at trivia, and unmatched or
// nested block evidence cannot fake top-level components.

#[test]
fn evidence_refs_resolve_to_direct_length_and_percentage_tokens_not_trivia() {
    let result = qualify(
        608143,
        "a{transform-origin:\n    /*a*/ 10px /*b*/\n    /*c*/ 20% /*d*/\n    /*e*/ 0 /*f*/;}",
    );

    let components = qualified_components(&result, 0);
    assert_eq!(components.len(), 3);

    assert!(matches!(
        component_token(&result, 0, 0),
        CssTokenKind::Dimension { .. }
    ));
    assert!(matches!(
        component_token(&result, 0, 1),
        CssTokenKind::Percentage { .. }
    ));
    assert!(matches!(
        component_token(&result, 0, 2),
        CssTokenKind::Number { .. }
    ));
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_components() {
    let result = qualify(
        608144,
        concat!(
            "a{transform-origin:[left top] 30px;}",
            "b{transform-origin:(left top) 30px;}",
            "c{transform-origin:left) top;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// `center` is never assigned both roles simultaneously without a valid
// complete matching theorem -- a sole `center` remains a single
// one-component keyword, never an implicit two-role pair.

#[test]
fn sole_center_is_a_single_one_component_keyword_not_a_dual_role_pair() {
    let result = qualify(608145, "a{transform-origin:center;}");
    assert_qualified_keywords(&result, 0, &[CssTransformOriginKeyword::Center]);
}
