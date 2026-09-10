use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssExponentSign, CssNumberSign, CssNumericValue, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssScaleComponent, CssScaleComponentKind, CssScaleQualificationOutcome,
    CssScaleUnsupportedReason, CssScaleValue, CssValueQualificationRunResult, run,
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
) -> &CssScaleQualificationOutcome {
    result.scale_observations()[index].outcome()
}

fn assert_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssScaleQualificationOutcome::Qualified(CssScaleValue::None),
        "expected whole-value None sentinel at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssScaleQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssScaleUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssScaleQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified components. Panics if the
/// observation at `index` is not `Qualified(Components(_))`.
fn qualified_components(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &[CssScaleComponent] {
    match outcome_at(result, index) {
        CssScaleQualificationOutcome::Qualified(CssScaleValue::Components(components)) => {
            components
        }
        other => panic!("expected Qualified(Components(_)) at index {index}, got {other:?}"),
    }
}

fn component_kinds(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> Vec<CssScaleComponentKind> {
    qualified_components(result, index)
        .iter()
        .map(|component| component.kind())
        .collect()
}

/// Resolves one qualified component's tokenizer-owned numeric evidence
/// through its run-local evidence reference, without any machine-number
/// conversion. Panics if the evidence does not resolve to a `Number` or
/// `Percentage` token.
fn component_numeric_value(
    result: &CssValueQualificationRunResult,
    index: usize,
    component_index: usize,
) -> &CssNumericValue {
    let components = qualified_components(result, index);
    let evidence = components[component_index].evidence_ref();
    let token = result.scale_component_token(evidence).unwrap_or_else(|| {
        panic!("component evidence at {index}/{component_index} did not resolve")
    });
    match token {
        CssTokenKind::Number { value, .. } | CssTokenKind::Percentage { value } => value,
        other => panic!(
            "component evidence at {index}/{component_index} resolved to {other:?}, not Number/Percentage"
        ),
    }
}

fn sign_and_integer_digits(value: &CssNumericValue) -> (Option<CssNumberSign>, &str) {
    (value.sign(), value.decimal().integer_digits())
}

// A. Dedicated `none`.

#[test]
fn whole_none_qualifies_ascii_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        602100,
        concat!(
            "a{scale:none;}",
            "b{scale:NONE;}",
            "c{scale:NoNe;}",
            "d{scale:\\6e one;}",
        ),
    );

    for index in 0..4 {
        assert_none(&result, index);
    }
}

#[test]
fn none_mixed_with_any_component_is_invalid() {
    let result = qualify(
        602101,
        concat!(
            "a{scale:none 1;}",
            "b{scale:1 none;}",
            "c{scale:none 100%;}",
            "d{scale:100% none;}",
            "e{scale:none none;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// B. One-component Number: sign/zero/fraction/exponent evidence preserved
// exactly, without machine-number conversion.

#[test]
fn direct_number_component_qualifies_with_exact_sign_and_zero_evidence() {
    let result = qualify(
        602102,
        concat!(
            "a{scale:0;}",
            "b{scale:+0;}",
            "c{scale:-0;}",
            "d{scale:1;}",
            "e{scale:+1;}",
            "f{scale:-1;}",
            "g{scale:.5;}",
            "h{scale:-.5;}",
        ),
    );

    for index in 0..8 {
        assert_eq!(
            component_kinds(&result, index),
            [CssScaleComponentKind::Number]
        );
    }

    assert_eq!(
        sign_and_integer_digits(component_numeric_value(&result, 0, 0)),
        (None, "0")
    );
    assert_eq!(
        sign_and_integer_digits(component_numeric_value(&result, 1, 0)),
        (Some(CssNumberSign::Plus), "0")
    );
    assert_eq!(
        sign_and_integer_digits(component_numeric_value(&result, 2, 0)),
        (Some(CssNumberSign::Minus), "0")
    );
    assert_eq!(
        component_numeric_value(&result, 6, 0)
            .decimal()
            .fraction_digits(),
        "5"
    );
    assert_eq!(
        component_numeric_value(&result, 7, 0).sign(),
        Some(CssNumberSign::Minus)
    );
}

#[test]
fn huge_direct_number_magnitude_and_exponent_qualify_without_machine_conversion() {
    let result = qualify(602103, concat!("a{scale:1e100;}", "b{scale:-1e-100000;}",));

    for index in 0..2 {
        assert_eq!(
            component_kinds(&result, index),
            [CssScaleComponentKind::Number]
        );
    }

    let positive = component_numeric_value(&result, 0, 0);
    assert_eq!(positive.decimal().integer_digits(), "1");
    let exponent = positive.decimal().exponent().expect("exponent evidence");
    assert_eq!(exponent.sign(), None);
    assert_eq!(exponent.digits(), "100");

    let negative = component_numeric_value(&result, 1, 0);
    assert_eq!(negative.sign(), Some(CssNumberSign::Minus));
    let exponent = negative.decimal().exponent().expect("exponent evidence");
    assert_eq!(exponent.sign(), Some(CssExponentSign::Minus));
    assert_eq!(exponent.digits(), "100000");
}

// C. One-component Percentage: remains Percentage, never Number.

#[test]
fn direct_percentage_component_qualifies_with_exact_sign_and_zero_evidence() {
    let result = qualify(
        602104,
        concat!(
            "a{scale:0%;}",
            "b{scale:+0%;}",
            "c{scale:-0%;}",
            "d{scale:100%;}",
            "e{scale:-100%;}",
            "f{scale:.5%;}",
            "g{scale:-.5%;}",
        ),
    );

    for index in 0..7 {
        assert_eq!(
            component_kinds(&result, index),
            [CssScaleComponentKind::Percentage]
        );
    }

    assert_eq!(
        sign_and_integer_digits(component_numeric_value(&result, 2, 0)),
        (Some(CssNumberSign::Minus), "0")
    );
    assert_eq!(
        sign_and_integer_digits(component_numeric_value(&result, 3, 0)),
        (None, "100")
    );
}

#[test]
fn large_exponent_percentage_forms_qualify_as_percentage() {
    let result = qualify(602105, "a{scale:1e100%;}b{scale:-1e-100000%;}");

    for index in 0..2 {
        assert_eq!(
            component_kinds(&result, index),
            [CssScaleComponentKind::Percentage]
        );
    }
}

// D. Number vs Percentage distinction -- load-bearing.

#[test]
fn number_and_percentage_are_distinct_authored_kinds_for_the_same_scale_factor() {
    let result = qualify(602106, "a{scale:1;}b{scale:100%;}");

    assert_eq!(component_kinds(&result, 0), [CssScaleComponentKind::Number]);
    assert_eq!(
        component_kinds(&result, 1),
        [CssScaleComponentKind::Percentage]
    );
    assert_ne!(component_kinds(&result, 0), component_kinds(&result, 1));

    // The token evidence itself must remain a distinct variant -- 100% must
    // never be rewritten into a Number(1) token or an interpreted fraction.
    let percentage_evidence = qualified_components(&result, 1)[0].evidence_ref();
    let token = result
        .scale_component_token(percentage_evidence)
        .expect("percentage evidence resolves");
    assert!(matches!(token, CssTokenKind::Percentage { .. }));
    assert!(!matches!(token, CssTokenKind::Number { .. }));
}

// E. 1/2/3 component cardinality, order, and mixed-kind preservation.

#[test]
fn one_two_and_three_component_cardinality_remain_distinct() {
    let result = qualify(602107, "a{scale:1;}b{scale:1 2;}c{scale:1 2 3;}");

    assert_eq!(qualified_components(&result, 0).len(), 1);
    assert_eq!(qualified_components(&result, 1).len(), 2);
    assert_eq!(qualified_components(&result, 2).len(), 3);
}

#[test]
fn mixed_number_and_percentage_components_preserve_exact_order_and_kind() {
    let result = qualify(
        602108,
        concat!(
            "a{scale:1 200%;}",
            "b{scale:100% 2;}",
            "c{scale:1 200% -3;}",
            "d{scale:100% -2 300%;}",
        ),
    );

    assert_eq!(
        component_kinds(&result, 0),
        [
            CssScaleComponentKind::Number,
            CssScaleComponentKind::Percentage
        ]
    );
    assert_eq!(
        component_kinds(&result, 1),
        [
            CssScaleComponentKind::Percentage,
            CssScaleComponentKind::Number
        ]
    );
    assert_eq!(
        component_kinds(&result, 2),
        [
            CssScaleComponentKind::Number,
            CssScaleComponentKind::Percentage,
            CssScaleComponentKind::Number,
        ]
    );
    assert_eq!(
        component_kinds(&result, 3),
        [
            CssScaleComponentKind::Percentage,
            CssScaleComponentKind::Number,
            CssScaleComponentKind::Percentage,
        ]
    );

    assert_eq!(
        sign_and_integer_digits(component_numeric_value(&result, 2, 2)),
        (Some(CssNumberSign::Minus), "3")
    );
}

// F. Omission preservation: `scale: 2` never synthesizes Y; `scale: 2 2`
// never synthesizes Z; `scale: 2 2 1` carries all three explicit.

#[test]
fn authored_omission_is_never_synthesized_as_additional_components() {
    let result = qualify(602109, "a{scale:2;}b{scale:2 2;}c{scale:2 2 1;}");

    assert_eq!(
        qualified_components(&result, 0).len(),
        1,
        "no synthesized Y"
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

    for index in 0..3 {
        assert!(
            component_kinds(&result, index)
                .iter()
                .all(|kind| *kind == CssScaleComponentKind::Number)
        );
    }
}

// G. More than three components is decisive Invalid, even when a residual
// Function is present -- cardinality invalidity is never masked as
// Unsupported.

#[test]
fn more_than_three_components_is_invalid_even_with_a_residual_function() {
    let result = qualify(
        602110,
        concat!(
            "a{scale:1 2 3 4;}",
            "b{scale:100% 2 3 4;}",
            "c{scale:calc(2) 3 4 5;}",
            "d{scale:1 calc(2) 3 4;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// H. Wrong direct token classes.

#[test]
fn wrong_direct_token_classes_are_invalid() {
    let result = qualify(
        602111,
        concat!(
            "a{scale:1px;}",
            "b{scale:1deg;}",
            "c{scale:1s;}",
            "d{scale:1fr;}",
            "e{scale:foo;}",
            "f{scale:\"1\";}",
            "g{scale:#abc;}",
            "h{scale:url(foo);}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// Explicit non-goal seal: the open <length> proposal (CSSWG #5273) must not
// be pre-implemented -- an absolute length remains decisively Invalid under
// the current pinned grammar.
#[test]
fn absolute_length_proposal_is_not_pre_implemented() {
    let result = qualify(602112, "a{scale:100px;}");
    assert_invalid(&result, 0);
}

// I. Commas are invalid -- this grammar is whitespace-separated repetition,
// never a comma list.

#[test]
fn comma_separated_syntax_is_invalid_in_every_spacing_variant() {
    let result = qualify(
        602113,
        concat!(
            "a{scale:1, 2;}",
            "b{scale:1,2;}",
            "c{scale:1 2,3;}",
            "d{scale:,1;}",
            "e{scale:1,;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// J. Residual Function profile: structurally feasible numeric positions
// remain bounded Unsupported, never evaluated.

#[test]
fn residual_function_in_feasible_numeric_position_is_unsupported() {
    let result = qualify(
        602114,
        concat!(
            "a{scale:calc(2);}",
            "b{scale:calc(200%);}",
            "c{scale:calc(100px);}",
            "d{scale:calc(180deg);}",
            "e{scale:arbitrary();}",
            "f{scale:calc(2) 3;}",
            "g{scale:1 calc(200%);}",
            "h{scale:1 calc(200%) 3;}",
        ),
    );

    for index in 0..8 {
        assert_unsupported(&result, index, CssScaleUnsupportedReason::FunctionValue);
    }
}

// K. Decisive Invalid outranks a provisional Function ambiguity found
// earlier in the value -- load-bearing precedence.

#[test]
fn decisive_invalidity_outranks_an_earlier_function_ambiguity() {
    let result = qualify(
        602115,
        concat!(
            "a{scale:calc(2) 3 4 5;}",
            "b{scale:calc(2) junk;}",
            "c{scale:calc(2) 3 junk;}",
            "d{scale:1 calc(2) 3 4;}",
            "e{scale:calc(2), 3;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// L. Nested delimiter isolation: an inner Function comma never inflates
// top-level component cardinality.

#[test]
fn nested_function_comma_is_never_a_top_level_separator() {
    let result = qualify(602116, "a{scale:calc(min(1, 2)) 3;}");
    assert_unsupported(&result, 0, CssScaleUnsupportedReason::FunctionValue);
}

// M. Whole-value Function boundary: recognized only as the entire value;
// embedded, it is misplaced and decisively Invalid.

#[test]
fn whole_value_function_is_unsupported_only_as_the_entire_value() {
    let result = qualify(602117, "a{scale:first-valid(1, 2);}");
    assert_unsupported(&result, 0, CssScaleUnsupportedReason::WholeValueFunction);
}

#[test]
fn embedded_whole_value_only_function_is_invalid_never_softened() {
    let result = qualify(602118, "a{scale:1 first-valid(2);}");
    assert_invalid(&result, 0);
}

// N. Deferred substitution precedes direct classification everywhere,
// reusing the existing shared policy unchanged.

#[test]
fn deferred_substitution_precedes_direct_classification_everywhere() {
    let result = qualify(
        602119,
        concat!(
            "a{scale:var(--x);}",
            "b{scale:1 var(--x);}",
            "c{scale:var(--x) 1;}",
            "d{scale:1 var(--x) 2;}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(
            &result,
            index,
            CssScaleUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// O. CSS-wide keywords: sole value only, never embedded.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        602120,
        concat!(
            "a{scale:initial;}",
            "b{scale:inherit;}",
            "c{scale:unset;}",
            "d{scale:revert;}",
            "e{scale:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(&whole, index, CssScaleUnsupportedReason::CssWideKeyword);
    }

    let combined = qualify(
        602121,
        concat!(
            "a{scale:1 initial;}",
            "b{scale:initial 1;}",
            "c{scale:100% revert;}",
        ),
    );
    for index in 0..3 {
        assert_invalid(&combined, index);
    }
}

// P. Trivia: whitespace/comments never change grouping; empty/trivia-only
// values are invalid.

#[test]
fn comments_between_and_adjacent_to_components_behave_like_whitespace() {
    let spaced = qualify(602122, "a{scale:1 200% -3;}");
    let comment_padded = qualify(602123, "a{scale:/*a*/1/*b*/ /*c*/200%/*d*/ /*e*/-3/*f*/;}");

    for result in [&spaced, &comment_padded] {
        assert_eq!(
            component_kinds(result, 0),
            [
                CssScaleComponentKind::Number,
                CssScaleComponentKind::Percentage,
                CssScaleComponentKind::Number,
            ]
        );
    }
}

#[test]
fn empty_and_trivia_only_values_are_invalid() {
    let result = qualify(602124, concat!("a{scale:;}", "b{scale: /**/  ;}",));
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// Q. `!important` stays outside the authored value window.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(602125, "a{scale:1 200% !important;}");
    assert_eq!(
        component_kinds(&result, 0),
        [
            CssScaleComponentKind::Number,
            CssScaleComponentKind::Percentage
        ]
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// R. Duplicate declarations remain independent observations, never
// collapsed by property name.

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(602126, "a{scale:1;scale:1 2;scale:none;}");

    assert_eq!(result.scale_observations().len(), 3);
    assert_eq!(qualified_components(&result, 0).len(), 1);
    assert_eq!(qualified_components(&result, 1).len(), 2);
    assert_none(&result, 2);
}

// S. Placement: only ordinary declaration placement enters this qualifier.

#[test]
fn nonordinary_contexts_do_not_enter_scale_dispatch() {
    for (source_id, css) in [
        (602127, "@font-face{scale:1;}"),
        (602128, "@page{scale:1;}"),
        (602129, "@keyframes k{from{scale:1;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.scale_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

// T. Incomplete upstream execution: qualification stays constrained by the
// committed prefix; a later occurrence cannot upgrade completion.

#[test]
fn incomplete_upstream_execution_constrains_qualification_to_committed_prefix() {
    let incomplete = qualify_with_limits(
        602130,
        "a{scale:1;}b{scale:1 2;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.scale_observations().len(), 1);
    assert_eq!(qualified_components(&incomplete, 0).len(), 1);
}

// U. Determinism: repeated and cross-source runs of equivalent source
// produce identical observations, including evidence resolution.

#[test]
fn repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{scale:1 200% -3;}",
        "b{scale:none;}",
        "c{scale:var(--x);}",
        "d{scale:none 1;}",
        "e{scale:calc(1);}",
        "f{scale:first-valid(1, 2);}",
    );
    let first = qualify(602131, css);
    let repeated = qualify(602131, css);
    let another_source = qualify(602132, css);

    assert_eq!(first.scale_observations(), repeated.scale_observations());
    assert_eq!(
        first.scale_observations(),
        another_source.scale_observations()
    );
}

// V. Cross-leaf isolation: neighboring accepted leaves are unaffected by
// this dispatch addition.

#[test]
fn cross_leaf_isolation_from_neighboring_qualified_properties() {
    let result = qualify(
        602133,
        concat!(
            "a{scale:1;scale:1 2;}",
            "b{opacity:1;}",
            "c{border-spacing:1px;}",
            "d{aspect-ratio:16/9;}",
            "e{offset-rotate:auto;}",
            "f{counter-set:foo;}",
        ),
    );

    assert_eq!(result.scale_observations().len(), 2);
    assert_eq!(result.scale_observations()[0].occurrence_index(), 0);
    assert_eq!(result.scale_observations()[1].occurrence_index(), 1);
    assert_eq!(result.opacity_observations().len(), 1);
    assert_eq!(result.border_spacing_observations().len(), 1);
    assert_eq!(result.aspect_ratio_observations().len(), 1);
    assert_eq!(result.offset_rotate_observations().len(), 1);
    assert_eq!(result.counter_set_observations().len(), 1);
}

// W. Evidence-ref sanity: evidence never points at trivia, and each
// component's evidence resolves to a token kind matching its authored
// kind.

#[test]
fn evidence_refs_resolve_to_direct_number_and_percentage_tokens_not_trivia() {
    let result = qualify(
        602134,
        "a{scale:\n    /*a*/ 1 /*b*/\n    /*c*/ 200% /*d*/;}",
    );

    let components = qualified_components(&result, 0);
    assert_eq!(components.len(), 2);

    for component in components {
        let token = result
            .scale_component_token(component.evidence_ref())
            .expect("scale evidence resolves");
        match component.kind() {
            CssScaleComponentKind::Number => {
                assert!(matches!(token, CssTokenKind::Number { .. }));
            }
            CssScaleComponentKind::Percentage => {
                assert!(matches!(token, CssTokenKind::Percentage { .. }));
            }
        }
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_components() {
    let result = qualify(
        602135,
        concat!("a{scale:[1 2] 3;}", "b{scale:(1 2) 3;}", "c{scale:1) 2;}",),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// X. Legacy negative-scenarios mirroring the load-bearing falsification
// list from the accepted research: sole `none` is never numeric identity,
// and a residual Function is never evaluated to a value.

#[test]
fn none_is_never_treated_as_a_numeric_identity_or_omission_sentinel() {
    let result = qualify(602136, "a{scale:none;}b{scale:1 none;}");
    assert_none(&result, 0);
    assert_invalid(&result, 1);
}
