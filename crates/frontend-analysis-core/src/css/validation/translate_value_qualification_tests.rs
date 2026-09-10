use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssNumberSign, CssNumericValue, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTranslateComponent, CssTranslateComponentKind, CssTranslateQualificationOutcome,
    CssTranslateUnsupportedReason, CssTranslateValue, CssValueQualificationRunResult, run,
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
) -> &CssTranslateQualificationOutcome {
    result.translate_observations()[index].outcome()
}

fn assert_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssTranslateQualificationOutcome::Qualified(CssTranslateValue::None),
        "expected whole-value None sentinel at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssTranslateQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssTranslateUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssTranslateQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified components. Panics if the
/// observation at `index` is not `Qualified(Components(_))`.
fn qualified_components(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &[CssTranslateComponent] {
    match outcome_at(result, index) {
        CssTranslateQualificationOutcome::Qualified(CssTranslateValue::Components(components)) => {
            components
        }
        other => panic!("expected Qualified(Components(_)) at index {index}, got {other:?}"),
    }
}

fn component_kinds(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> Vec<CssTranslateComponentKind> {
    qualified_components(result, index)
        .iter()
        .map(|component| component.kind())
        .collect()
}

/// Resolves one qualified component's tokenizer-owned numeric evidence
/// through its run-local evidence reference, without any machine-number
/// conversion. Panics if the evidence does not resolve to a `Number`,
/// `Dimension`, or `Percentage` token.
fn component_numeric_value(
    result: &CssValueQualificationRunResult,
    index: usize,
    component_index: usize,
) -> &CssNumericValue {
    let components = qualified_components(result, index);
    let evidence = components[component_index].evidence_ref();
    let token = result
        .translate_component_token(evidence)
        .unwrap_or_else(|| {
            panic!("component evidence at {index}/{component_index} did not resolve")
        });
    match token {
        CssTokenKind::Number { value, .. }
        | CssTokenKind::Dimension { value, .. }
        | CssTokenKind::Percentage { value } => value,
        other => panic!(
            "component evidence at {index}/{component_index} resolved to {other:?}, not Number/Dimension/Percentage"
        ),
    }
}

fn component_token(
    result: &CssValueQualificationRunResult,
    index: usize,
    component_index: usize,
) -> &CssTokenKind {
    let components = qualified_components(result, index);
    let evidence = components[component_index].evidence_ref();
    result
        .translate_component_token(evidence)
        .unwrap_or_else(|| {
            panic!("component evidence at {index}/{component_index} did not resolve")
        })
}

// A. Dedicated `none`.

#[test]
fn whole_none_qualifies_ascii_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        606100,
        concat!(
            "a{translate:none;}",
            "b{translate:NONE;}",
            "c{translate:NoNe;}",
            "d{translate:\\6e one;}",
        ),
    );

    for index in 0..4 {
        assert_none(&result, index);
    }
}

#[test]
fn none_mixed_with_any_component_is_invalid() {
    let result = qualify(
        606101,
        concat!(
            "a{translate:none 0;}",
            "b{translate:0 none;}",
            "c{translate:none 10px;}",
            "d{translate:10px none;}",
            "e{translate:none none;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// B. One-component direct Length Dimension: positive/negative, various
// recognized units, without machine-number conversion.

#[test]
fn direct_length_dimension_component_qualifies_regardless_of_sign() {
    let result = qualify(
        606102,
        concat!(
            "a{translate:0px;}",
            "b{translate:-0px;}",
            "c{translate:1px;}",
            "d{translate:-10px;}",
            "e{translate:.5em;}",
            "f{translate:-.5em;}",
            "g{translate:1e100cqi;}",
            "h{translate:-1e-999px;}",
        ),
    );

    for index in 0..8 {
        assert_eq!(
            component_kinds(&result, index),
            [CssTranslateComponentKind::Length]
        );
        assert!(matches!(
            component_token(&result, index, 0),
            CssTokenKind::Dimension { .. }
        ));
    }
}

// C. One-component exact-zero Length: unitless numeric zero in every
// accepted spelling, preserved without machine-number conversion; non-zero
// unitless Numbers remain Invalid.

#[test]
fn unitless_exact_zero_qualifies_as_length_in_every_accepted_spelling() {
    let result = qualify(
        606103,
        concat!(
            "a{translate:0;}",
            "b{translate:+0;}",
            "c{translate:-0;}",
            "d{translate:.0;}",
            "e{translate:-.0;}",
            "f{translate:0.0;}",
            "g{translate:-0.0;}",
            "h{translate:0e100;}",
            "i{translate:-0e100;}",
            "j{translate:0e-999;}",
            "k{translate:-0e-999;}",
        ),
    );

    for index in 0..11 {
        assert_eq!(
            component_kinds(&result, index),
            [CssTranslateComponentKind::Length]
        );
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
fn non_zero_unitless_number_is_invalid_in_every_slot() {
    let result = qualify(
        606104,
        concat!(
            "a{translate:1;}",
            "b{translate:-1;}",
            "c{translate:.5;}",
            "d{translate:10e2;}",
            "e{translate:1 0;}",
            "f{translate:0 1;}",
            "g{translate:0 0 1;}",
        ),
    );

    for index in 0..7 {
        assert_invalid(&result, index);
    }
}

// D. One-component Percentage: remains Percentage, never Length.

#[test]
fn direct_percentage_component_qualifies_and_remains_percentage() {
    let result = qualify(
        606105,
        concat!(
            "a{translate:0%;}",
            "b{translate:-0%;}",
            "c{translate:100%;}",
            "d{translate:-100%;}",
            "e{translate:.5%;}",
            "f{translate:-.5%;}",
        ),
    );

    for index in 0..6 {
        assert_eq!(
            component_kinds(&result, index),
            [CssTranslateComponentKind::Percentage]
        );
        assert!(matches!(
            component_token(&result, index, 0),
            CssTokenKind::Percentage { .. }
        ));
    }
}

// E. Authored 0 / 0px / 0% distinction -- load-bearing.

#[test]
fn zero_zero_px_and_zero_percent_remain_authored_distinct() {
    let result = qualify(
        606106,
        concat!("a{translate:0;}", "b{translate:0px;}", "c{translate:0%;}",),
    );

    assert_eq!(
        component_kinds(&result, 0),
        [CssTranslateComponentKind::Length]
    );
    assert!(matches!(
        component_token(&result, 0, 0),
        CssTokenKind::Number { .. }
    ));

    assert_eq!(
        component_kinds(&result, 1),
        [CssTranslateComponentKind::Length]
    );
    assert!(matches!(
        component_token(&result, 1, 0),
        CssTokenKind::Dimension { .. }
    ));

    assert_eq!(
        component_kinds(&result, 2),
        [CssTranslateComponentKind::Percentage]
    );
    assert!(matches!(
        component_token(&result, 2, 0),
        CssTokenKind::Percentage { .. }
    ));
}

// F. Two-component XY combinations: all four Length/Percentage pairings
// qualify, preserving exact order and kind.

#[test]
fn two_component_xy_combinations_all_qualify_with_exact_order_and_kind() {
    let result = qualify(
        606107,
        concat!(
            "a{translate:10px 20px;}",
            "b{translate:10px 20%;}",
            "c{translate:10% 20px;}",
            "d{translate:10% 20%;}",
            "e{translate:0 0;}",
            "f{translate:0 0%;}",
            "g{translate:0% 0;}",
            "h{translate:0% 0%;}",
        ),
    );

    use CssTranslateComponentKind::{Length, Percentage};
    let expected: [[CssTranslateComponentKind; 2]; 8] = [
        [Length, Length],
        [Length, Percentage],
        [Percentage, Length],
        [Percentage, Percentage],
        [Length, Length],
        [Length, Percentage],
        [Percentage, Length],
        [Percentage, Percentage],
    ];

    for (index, expected_kinds) in expected.iter().enumerate() {
        assert_eq!(component_kinds(&result, index), expected_kinds.as_slice());
    }
}

// G. Three-component direct combinations: XY Length/Percentage mixes with Z
// Length only.

#[test]
fn three_component_forms_qualify_with_z_length_only() {
    let result = qualify(
        606108,
        concat!(
            "a{translate:10px 20px 30px;}",
            "b{translate:10px 20% 30px;}",
            "c{translate:10% 20px 30px;}",
            "d{translate:10% 20% 30px;}",
            "e{translate:1px 2px 0;}",
            "f{translate:1px 2px -3em;}",
            "g{translate:0% 0% 0;}",
        ),
    );

    use CssTranslateComponentKind::{Length, Percentage};
    let expected: [[CssTranslateComponentKind; 3]; 7] = [
        [Length, Length, Length],
        [Length, Percentage, Length],
        [Percentage, Length, Length],
        [Percentage, Percentage, Length],
        [Length, Length, Length],
        [Length, Length, Length],
        [Percentage, Percentage, Length],
    ];

    for (index, expected_kinds) in expected.iter().enumerate() {
        let kinds = component_kinds(&result, index);
        assert_eq!(kinds, expected_kinds.as_slice());
        assert_eq!(kinds[2], CssTranslateComponentKind::Length);
    }
}

// H. Third-slot Percentage rejection -- one of the highest-risk theorems.
// A Percentage equal to mathematical zero does not become <length> in Z.

#[test]
fn third_slot_percentage_is_invalid_even_when_zero() {
    let result = qualify(
        606109,
        concat!(
            "a{translate:10px 20px 30%;}",
            "b{translate:10px 20px 0%;}",
            "c{translate:10% 20% 0%;}",
            "d{translate:0 0 0%;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// I. Exact cardinality / no default synthesis.

#[test]
fn authored_cardinality_is_exact_with_no_synthesized_defaults() {
    let result = qualify(
        606110,
        concat!(
            "a{translate:10px;}",
            "b{translate:10px 0;}",
            "c{translate:10px 0 0;}",
        ),
    );

    assert_eq!(
        qualified_components(&result, 0).len(),
        1,
        "no synthesized Y or Z"
    );
    assert_eq!(
        qualified_components(&result, 1).len(),
        2,
        "no synthesized Z"
    );
    assert_eq!(
        qualified_components(&result, 2).len(),
        3,
        "all three explicit"
    );
}

// J. Wrong direct token classes.

#[test]
fn wrong_direct_token_classes_are_invalid() {
    let result = qualify(
        606111,
        concat!(
            "a{translate:foo;}",
            "b{translate:\"10px\";}",
            "c{translate:#abc;}",
            "d{translate:1deg;}",
            "e{translate:1s;}",
            "f{translate:1fr;}",
            "g{translate:10px foo;}",
            "h{translate:10px 20px foo;}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// K. Top-level comma rejection -- this grammar is whitespace-separated
// repetition, never a comma list.

#[test]
fn comma_separated_syntax_is_invalid_in_every_spacing_variant() {
    let result = qualify(
        606112,
        concat!(
            "a{translate:10px, 20px;}",
            "b{translate:10px,20px;}",
            "c{translate:10px 20px,30px;}",
            "d{translate:,10px;}",
            "e{translate:10px,;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// L. Residual Function in slot 1: structurally feasible -> Unsupported,
// never evaluated.

#[test]
fn residual_function_in_slot_one_is_unsupported() {
    let result = qualify(
        606113,
        concat!(
            "a{translate:calc(10px);}",
            "b{translate:calc(10%);}",
            "c{translate:calc(10% + 10px);}",
            "d{translate:foo();}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(&result, index, CssTranslateUnsupportedReason::FunctionValue);
    }
}

// M. Residual Function in slot 2.

#[test]
fn residual_function_in_slot_two_is_unsupported() {
    let result = qualify(
        606114,
        concat!(
            "a{translate:10px calc(20px);}",
            "b{translate:10px calc(20%);}",
            "c{translate:10% calc(20px);}",
            "d{translate:calc(10px) calc(20px);}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(&result, index, CssTranslateUnsupportedReason::FunctionValue);
    }
}

// N. Residual Function in slot 3: remains provisional Unsupported, even a
// mixed length+percentage calculation that CSS math would ultimately
// resolve to a standards-invalid type -- this leaf does not evaluate
// calculation result types.

#[test]
fn residual_function_in_slot_three_is_unsupported_without_calc_evaluation() {
    let result = qualify(
        606115,
        concat!(
            "a{translate:10px 20px calc(30px);}",
            "b{translate:10% 20% calc(30px);}",
            "c{translate:10px 20px calc(30px + 30%);}",
        ),
    );

    for index in 0..3 {
        assert_unsupported(&result, index, CssTranslateUnsupportedReason::FunctionValue);
    }
}

// O. Decisive Invalid outranks Function uncertainty -- load-bearing
// precedence: do not resolve Unsupported on first residual Function
// sighting when other retained evidence independently proves the
// declaration invalid.

#[test]
fn decisive_invalidity_outranks_an_earlier_or_later_function_ambiguity() {
    let result = qualify(
        606116,
        concat!(
            "a{translate:calc(10px) 20px 30%;}",
            "b{translate:calc(10px) 20px 0%;}",
            "c{translate:calc(10px) 20px junk;}",
            "d{translate:calc(10px) 20px 30px extra;}",
            "e{translate:10px calc(20px) 30%;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// P. More than three ordinary components is decisive Invalid, even with a
// residual Function present -- cardinality invalidity is never masked as
// Unsupported.

#[test]
fn more_than_three_components_is_invalid_even_with_a_residual_function() {
    let result = qualify(
        606117,
        concat!(
            "a{translate:10px 20px 30px 40px;}",
            "b{translate:calc(10px) 20px 30px 40px;}",
        ),
    );

    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// Q. Nested Function delimiter isolation: an inner Function comma never
// inflates outer component cardinality.

#[test]
fn nested_function_comma_is_never_a_top_level_separator() {
    let result = qualify(606118, "a{translate:calc(min(10px, 20px)) 30px;}");
    assert_unsupported(&result, 0, CssTranslateUnsupportedReason::FunctionValue);
}

// R. Whole-value Function boundary: recognized only as the entire value;
// embedded, it is misplaced and decisively Invalid.

#[test]
fn whole_value_function_is_unsupported_only_as_the_entire_value() {
    let result = qualify(606119, "a{translate:first-valid(10px, 20px);}");
    assert_unsupported(
        &result,
        0,
        CssTranslateUnsupportedReason::WholeValueFunction,
    );
}

#[test]
fn embedded_whole_value_only_function_is_invalid_never_softened() {
    let result = qualify(
        606120,
        concat!(
            "a{translate:10px first-valid(20px);}",
            "b{translate:10px 20px first-valid(30px);}",
        ),
    );
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// S. Deferred substitution precedes direct classification everywhere,
// reusing the existing shared policy unchanged.

#[test]
fn deferred_substitution_precedes_direct_classification_everywhere() {
    let result = qualify(
        606121,
        concat!(
            "a{translate:var(--x);}",
            "b{translate:10px var(--x);}",
            "c{translate:var(--x) 10px;}",
            "d{translate:10px var(--x) 20px;}",
            "e{translate:env(safe-area-inset-top);}",
            "f{translate:attr(data-x);}",
            "g{translate:--custom(10px);}",
        ),
    );

    for index in 0..7 {
        assert_unsupported(
            &result,
            index,
            CssTranslateUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// T. CSS-wide keywords: sole value only, never embedded.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        606122,
        concat!(
            "a{translate:initial;}",
            "b{translate:inherit;}",
            "c{translate:unset;}",
            "d{translate:revert;}",
            "e{translate:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(&whole, index, CssTranslateUnsupportedReason::CssWideKeyword);
    }

    let combined = qualify(
        606123,
        concat!(
            "a{translate:10px initial;}",
            "b{translate:initial 10px;}",
            "c{translate:10px 20px revert;}",
        ),
    );
    for index in 0..3 {
        assert_invalid(&combined, index);
    }
}

// U. Trivia: whitespace/comments never change grouping; empty/trivia-only
// values are invalid.

#[test]
fn comments_between_and_adjacent_to_components_behave_like_whitespace() {
    let spaced = qualify(606124, "a{translate:10px 20% 30px;}");
    let comment_padded = qualify(
        606125,
        "a{translate:/*a*/10px/*b*/ /*c*/20%/*d*/ /*e*/30px/*f*/;}",
    );

    for result in [&spaced, &comment_padded] {
        assert_eq!(
            component_kinds(result, 0),
            [
                CssTranslateComponentKind::Length,
                CssTranslateComponentKind::Percentage,
                CssTranslateComponentKind::Length,
            ]
        );
    }
}

#[test]
fn empty_and_trivia_only_values_are_invalid() {
    let result = qualify(606126, concat!("a{translate:;}", "b{translate: /**/  ;}",));
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// V. `!important` stays outside the authored value window.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(606127, "a{translate:10px 20% 30px !important;}");
    assert_eq!(
        component_kinds(&result, 0),
        [
            CssTranslateComponentKind::Length,
            CssTranslateComponentKind::Percentage,
            CssTranslateComponentKind::Length,
        ]
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// W. Duplicate declarations remain independent observations, never
// collapsed by property name.

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(606128, "a{translate:10px;translate:10px 0;translate:none;}");

    assert_eq!(result.translate_observations().len(), 3);
    assert_eq!(qualified_components(&result, 0).len(), 1);
    assert_eq!(qualified_components(&result, 1).len(), 2);
    assert_none(&result, 2);
}

// X. Placement: only ordinary declaration placement enters this qualifier.

#[test]
fn nonordinary_contexts_do_not_enter_translate_dispatch() {
    for (source_id, css) in [
        (606129, "@font-face{translate:10px;}"),
        (606130, "@page{translate:10px;}"),
        (606131, "@keyframes k{from{translate:10px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.translate_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

// Y. Incomplete upstream execution: qualification stays constrained by the
// committed prefix; a later occurrence cannot upgrade completion.

#[test]
fn incomplete_upstream_execution_constrains_qualification_to_committed_prefix() {
    let incomplete = qualify_with_limits(
        606132,
        "a{translate:10px;}b{translate:10px 0;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.translate_observations().len(), 1);
    assert_eq!(qualified_components(&incomplete, 0).len(), 1);
}

// Z. Determinism: repeated and cross-source runs of equivalent source
// produce identical observations, including evidence resolution.

#[test]
fn repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{translate:10px 20% -30px;}",
        "b{translate:none;}",
        "c{translate:var(--x);}",
        "d{translate:none 10px;}",
        "e{translate:calc(10px);}",
        "f{translate:first-valid(10px, 20px);}",
        "g{translate:10px 20px 30%;}",
    );
    let first = qualify(606133, css);
    let repeated = qualify(606133, css);
    let another_source = qualify(606134, css);

    assert_eq!(
        first.translate_observations(),
        repeated.translate_observations()
    );
    assert_eq!(
        first.translate_observations(),
        another_source.translate_observations()
    );
}

// AA. Cross-leaf isolation: neighboring accepted leaves are unaffected by
// this dispatch addition.

#[test]
fn cross_leaf_isolation_from_neighboring_qualified_properties() {
    let result = qualify(
        606135,
        concat!(
            "a{translate:10px;translate:10px 0;}",
            "b{scale:1;}",
            "c{rotate:45deg;}",
            "d{scroll-margin-top:1px;}",
            "e{word-spacing:1px;}",
            "f{shape-margin:1px;}",
            "g{perspective:100px;}",
        ),
    );

    assert_eq!(result.translate_observations().len(), 2);
    assert_eq!(result.translate_observations()[0].occurrence_index(), 0);
    assert_eq!(result.translate_observations()[1].occurrence_index(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.scroll_margin_top_observations().len(), 1);
    assert_eq!(result.word_spacing_observations().len(), 1);
    assert_eq!(result.shape_margin_observations().len(), 1);
    assert_eq!(result.perspective_observations().len(), 1);
}

// Evidence-ref sanity: evidence never points at trivia, and each
// component's evidence resolves to a token kind matching its authored kind.

#[test]
fn evidence_refs_resolve_to_direct_length_and_percentage_tokens_not_trivia() {
    let result = qualify(
        606136,
        "a{translate:\n    /*a*/ 10px /*b*/\n    /*c*/ 20% /*d*/\n    /*e*/ 0 /*f*/;}",
    );

    let components = qualified_components(&result, 0);
    assert_eq!(components.len(), 3);

    let token0 = result
        .translate_component_token(components[0].evidence_ref())
        .expect("translate evidence resolves");
    assert!(matches!(token0, CssTokenKind::Dimension { .. }));

    let token1 = result
        .translate_component_token(components[1].evidence_ref())
        .expect("translate evidence resolves");
    assert!(matches!(token1, CssTokenKind::Percentage { .. }));

    let token2 = result
        .translate_component_token(components[2].evidence_ref())
        .expect("translate evidence resolves");
    assert!(matches!(token2, CssTokenKind::Number { .. }));
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_components() {
    let result = qualify(
        606137,
        concat!(
            "a{translate:[10px 20px] 30px;}",
            "b{translate:(10px 20px) 30px;}",
            "c{translate:10px) 20px;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// `none` is never a translation component -- never treated as a numeric
// identity or omission sentinel.

#[test]
fn none_is_never_treated_as_a_translation_identity_or_omission_sentinel() {
    let result = qualify(606138, "a{translate:none;}b{translate:10px none;}");
    assert_none(&result, 0);
    assert_invalid(&result, 1);
}

// Signed-zero preservation: `-0` must resolve with a retained Minus sign,
// never normalized away, when correctly positioned across all three slots.

#[test]
fn signed_zero_is_preserved_across_all_three_slots() {
    let result = qualify(606139, "a{translate:-0 20% -0;}");
    assert_eq!(
        component_kinds(&result, 0),
        [
            CssTranslateComponentKind::Length,
            CssTranslateComponentKind::Percentage,
            CssTranslateComponentKind::Length,
        ]
    );
    assert_eq!(
        component_numeric_value(&result, 0, 0).sign(),
        Some(CssNumberSign::Minus)
    );
    assert_eq!(
        component_numeric_value(&result, 0, 2).sign(),
        Some(CssNumberSign::Minus)
    );
}
