use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssExponentSign, CssNumberSign, CssNumericValue, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssRotateAxis, CssRotateAxisKeyword, CssRotateEvidenceRef, CssRotateQualificationOutcome,
    CssRotateRotation, CssRotateUnsupportedReason, CssRotateValue, CssValueQualificationRunResult,
    run,
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
) -> CssRotateQualificationOutcome {
    result.rotate_observations()[index].outcome()
}

fn assert_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        CssRotateQualificationOutcome::Qualified(CssRotateValue::None),
        "expected whole-value None sentinel at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        CssRotateQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssRotateUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        CssRotateQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's qualified `Rotation` structure. Panics if the
/// observation at `index` is not `Qualified(Rotation(_))`.
fn rotation_at(result: &CssValueQualificationRunResult, index: usize) -> CssRotateRotation {
    match outcome_at(result, index) {
        CssRotateQualificationOutcome::Qualified(CssRotateValue::Rotation(rotation)) => rotation,
        other => panic!("expected Qualified(Rotation(_)) at index {index}, got {other:?}"),
    }
}

fn angle_at(result: &CssValueQualificationRunResult, index: usize) -> CssRotateEvidenceRef {
    rotation_at(result, index).angle()
}

fn assert_axis_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        rotation_at(result, index).axis(),
        None,
        "expected authored axis omission at index {index}"
    );
}

fn assert_axis_keyword(
    result: &CssValueQualificationRunResult,
    index: usize,
    keyword: CssRotateAxisKeyword,
) {
    assert_eq!(
        rotation_at(result, index).axis(),
        Some(CssRotateAxis::Keyword(keyword)),
        "expected keyword axis {keyword:?} at index {index}"
    );
}

/// Resolves one observation's qualified vector-axis evidence. Panics if the
/// observation at `index` is not a `Vector` axis.
fn vector_axis_at(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> [CssRotateEvidenceRef; 3] {
    match rotation_at(result, index).axis() {
        Some(CssRotateAxis::Vector(evidence)) => evidence,
        other => panic!("expected Vector axis at index {index}, got {other:?}"),
    }
}

/// Resolves one qualified angle evidence reference to its exact
/// tokenizer-owned `Dimension` value/unit, without any unit conversion or
/// numeric normalization.
fn angle_dimension(
    result: &CssValueQualificationRunResult,
    angle: CssRotateEvidenceRef,
) -> (&CssNumericValue, &str) {
    match result
        .rotate_evidence_token(angle)
        .expect("angle evidence resolves")
    {
        CssTokenKind::Dimension { value, unit, .. } => (value, unit.as_str()),
        other => panic!("angle evidence resolved to {other:?}, not Dimension"),
    }
}

/// Resolves one qualified vector-axis component evidence reference to its
/// exact tokenizer-owned `Number` value, without machine-number conversion.
fn vector_component_value(
    result: &CssValueQualificationRunResult,
    evidence: CssRotateEvidenceRef,
) -> &CssNumericValue {
    match result
        .rotate_evidence_token(evidence)
        .expect("vector component evidence resolves")
    {
        CssTokenKind::Number { value, .. } => value,
        other => panic!("vector component evidence resolved to {other:?}, not Number"),
    }
}

fn sign_and_integer_digits(value: &CssNumericValue) -> (Option<CssNumberSign>, &str) {
    (value.sign(), value.decimal().integer_digits())
}

// A. Dedicated `none`.

#[test]
fn whole_none_qualifies_ascii_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        604100,
        concat!(
            "a{rotate:none;}",
            "b{rotate:NONE;}",
            "c{rotate:NoNe;}",
            "d{rotate:\\6e one;}",
        ),
    );

    for index in 0..4 {
        assert_none(&result, index);
    }
}

#[test]
fn none_embedded_or_repeated_is_invalid() {
    let result = qualify(
        604101,
        concat!(
            "a{rotate:none 30deg;}",
            "b{rotate:30deg none;}",
            "c{rotate:none x 30deg;}",
            "d{rotate:x 30deg none;}",
            "e{rotate:none none;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// B. Direct angle-only: all four units, sign/case/exponent evidence.

#[test]
fn direct_angle_only_qualifies_for_all_units_regardless_of_sign_or_case() {
    let result = qualify(
        604102,
        concat!(
            "a{rotate:0deg;}",
            "b{rotate:-0deg;}",
            "c{rotate:+0deg;}",
            "d{rotate:45deg;}",
            "e{rotate:-400deg;}",
            "f{rotate:100grad;}",
            "g{rotate:1.57rad;}",
            "h{rotate:.5turn;}",
            "i{rotate:-1.57rad;}",
            "j{rotate:-.5turn;}",
            "k{rotate:45DEG;}",
            "l{rotate:1RaD;}",
            "m{rotate:.5TuRn;}",
            "n{rotate:100GRAD;}",
            "o{rotate:1e2deg;}",
        ),
    );

    for index in 0..15 {
        assert_axis_none(&result, index);
    }

    let (value, unit) = angle_dimension(&result, angle_at(&result, 1));
    assert_eq!(value.sign(), Some(CssNumberSign::Minus));
    assert_eq!(unit, "deg");

    let (_, unit) = angle_dimension(&result, angle_at(&result, 10));
    assert_eq!(unit, "DEG");

    let (value, unit) = angle_dimension(&result, angle_at(&result, 14));
    assert_eq!(unit, "deg");
    let exponent = value.decimal().exponent().expect("exponent evidence");
    assert_eq!(exponent.digits(), "2");
}

// C. Bare-zero boundary -- load-bearing.

#[test]
fn bare_unitless_zero_is_invalid_but_zero_with_unit_qualifies() {
    let result = qualify(
        604103,
        concat!(
            "a{rotate:0;}",
            "b{rotate:+0;}",
            "c{rotate:-0;}",
            "d{rotate:0deg;}",
        ),
    );

    assert_invalid(&result, 0);
    assert_invalid(&result, 1);
    assert_invalid(&result, 2);
    assert_axis_none(&result, 3);
}

// D. Wrong angle Dimensions.

#[test]
fn wrong_dimension_units_are_invalid() {
    let result = qualify(
        604104,
        concat!(
            "a{rotate:1px;}",
            "b{rotate:1s;}",
            "c{rotate:1hz;}",
            "d{rotate:1dpi;}",
            "e{rotate:1em;}",
            "f{rotate:1ms;}",
        ),
    );

    for index in 0..6 {
        assert_invalid(&result, index);
    }
}

// E. Keyword-axis before angle.

#[test]
fn keyword_axis_before_angle_qualifies() {
    let result = qualify(
        604105,
        concat!(
            "a{rotate:x 30deg;}",
            "b{rotate:y -1rad;}",
            "c{rotate:z .5turn;}",
        ),
    );

    assert_axis_keyword(&result, 0, CssRotateAxisKeyword::X);
    assert_axis_keyword(&result, 1, CssRotateAxisKeyword::Y);
    assert_axis_keyword(&result, 2, CssRotateAxisKeyword::Z);
}

// F. Keyword-axis after angle.

#[test]
fn keyword_axis_after_angle_qualifies() {
    let result = qualify(
        604106,
        concat!(
            "a{rotate:30deg x;}",
            "b{rotate:-1rad y;}",
            "c{rotate:.5turn z;}",
        ),
    );

    assert_axis_keyword(&result, 0, CssRotateAxisKeyword::X);
    assert_axis_keyword(&result, 1, CssRotateAxisKeyword::Y);
    assert_axis_keyword(&result, 2, CssRotateAxisKeyword::Z);
}

// G. Keyword axis identity: ASCII case and escape-equivalent forms.

#[test]
fn keyword_axis_identity_is_ascii_case_insensitive_and_escape_equivalent() {
    let result = qualify(
        604107,
        concat!(
            "a{rotate:X 30deg;}",
            "b{rotate:30deg Y;}",
            "c{rotate:\\78  30deg;}",
        ),
    );

    assert_axis_keyword(&result, 0, CssRotateAxisKeyword::X);
    assert_axis_keyword(&result, 1, CssRotateAxisKeyword::Y);
    assert_axis_keyword(&result, 2, CssRotateAxisKeyword::X);
}

// H. Keyword axis incomplete/duplicate.

#[test]
fn keyword_axis_incomplete_or_duplicate_is_invalid() {
    let result = qualify(
        604108,
        concat!(
            "a{rotate:x;}",
            "b{rotate:y;}",
            "c{rotate:z;}",
            "d{rotate:x y 30deg;}",
            "e{rotate:45deg x y;}",
            "f{rotate:x x 30deg;}",
            "g{rotate:30deg z z;}",
        ),
    );

    for index in 0..7 {
        assert_invalid(&result, index);
    }
}

// I. Vector-axis angle-last.

#[test]
fn vector_axis_angle_last_qualifies() {
    let result = qualify(
        604109,
        concat!(
            "a{rotate:1 0 0 30deg;}",
            "b{rotate:0 1 0 30deg;}",
            "c{rotate:0 0 1 30deg;}",
            "d{rotate:0.5 0 0 400grad;}",
            "e{rotate:-1 0 0 -30deg;}",
            "f{rotate:0 0 0 30deg;}",
        ),
    );

    for index in 0..6 {
        let _ = vector_axis_at(&result, index);
    }
}

// J. Vector-axis angle-first.

#[test]
fn vector_axis_angle_first_qualifies() {
    let result = qualify(
        604110,
        concat!(
            "a{rotate:30deg 1 0 0;}",
            "b{rotate:30deg 0 1 0;}",
            "c{rotate:30deg 0 0 1;}",
            "d{rotate:400grad 0.5 0 0;}",
            "e{rotate:-30deg -1 0 0;}",
            "f{rotate:30deg 0 0 0;}",
        ),
    );

    for index in 0..6 {
        let _ = vector_axis_at(&result, index);
    }
}

// K. Exact vector Number evidence: sign/zero/fraction/exponent preserved
// without machine-number conversion.

#[test]
fn vector_number_evidence_preserves_exact_sign_zero_and_order() {
    let result = qualify(
        604111,
        concat!(
            "a{rotate:1 0 0 30deg;}",
            "b{rotate:0.5 0 0 30deg;}",
            "c{rotate:-1 0 0 30deg;}",
            "d{rotate:-0 +0 -0 30deg;}",
            "e{rotate:1e100 0 -1e-100000 30deg;}",
        ),
    );

    let evidence = vector_axis_at(&result, 0);
    assert_eq!(
        sign_and_integer_digits(vector_component_value(&result, evidence[0])),
        (None, "1")
    );

    let evidence = vector_axis_at(&result, 1);
    assert_eq!(
        vector_component_value(&result, evidence[0])
            .decimal()
            .fraction_digits(),
        "5"
    );

    let evidence = vector_axis_at(&result, 2);
    assert_eq!(
        sign_and_integer_digits(vector_component_value(&result, evidence[0])),
        (Some(CssNumberSign::Minus), "1")
    );

    let evidence = vector_axis_at(&result, 3);
    assert_eq!(
        vector_component_value(&result, evidence[0]).sign(),
        Some(CssNumberSign::Minus)
    );
    assert_eq!(
        vector_component_value(&result, evidence[1]).sign(),
        Some(CssNumberSign::Plus)
    );
    assert_eq!(
        vector_component_value(&result, evidence[2]).sign(),
        Some(CssNumberSign::Minus)
    );

    let evidence = vector_axis_at(&result, 4);
    let first = vector_component_value(&result, evidence[0]);
    assert_eq!(first.decimal().integer_digits(), "1");
    let exponent = first.decimal().exponent().expect("exponent evidence");
    assert_eq!(exponent.sign(), None);
    assert_eq!(exponent.digits(), "100");

    let third = vector_component_value(&result, evidence[2]);
    assert_eq!(third.sign(), Some(CssNumberSign::Minus));
    let exponent = third.decimal().exponent().expect("exponent evidence");
    assert_eq!(exponent.sign(), Some(CssExponentSign::Minus));
    assert_eq!(exponent.digits(), "100000");
}

// L. Zero-vector acceptance -- load-bearing.

#[test]
fn zero_vector_qualifies_in_either_order() {
    let result = qualify(
        604112,
        concat!("a{rotate:0 0 0 30deg;}", "b{rotate:30deg 0 0 0;}",),
    );

    let _ = vector_axis_at(&result, 0);
    let _ = vector_axis_at(&result, 1);
}

// M. Authored axis distinctions -- no synthetic Z, no vector/keyword
// canonicalization.

#[test]
fn angle_only_keyword_axis_and_vector_axis_remain_distinct_authored_structures() {
    let result = qualify(
        604113,
        concat!(
            "a{rotate:30deg;}",
            "b{rotate:z 30deg;}",
            "c{rotate:0 0 1 30deg;}",
        ),
    );

    assert_axis_none(&result, 0);
    assert_axis_keyword(&result, 1, CssRotateAxisKeyword::Z);
    let _ = vector_axis_at(&result, 2);
}

#[test]
fn keyword_axis_and_distinct_vector_forms_never_canonicalize_to_each_other() {
    let result = qualify(
        604114,
        concat!(
            "a{rotate:x 30deg;}",
            "b{rotate:1 0 0 30deg;}",
            "c{rotate:0.5 0 0 30deg;}",
            "d{rotate:-1 0 0 30deg;}",
        ),
    );

    assert_axis_keyword(&result, 0, CssRotateAxisKeyword::X);
    let vector_one = vector_axis_at(&result, 1);
    let vector_half = vector_axis_at(&result, 2);
    let vector_negative_one = vector_axis_at(&result, 3);

    assert_eq!(
        sign_and_integer_digits(vector_component_value(&result, vector_one[0])),
        (None, "1")
    );
    assert_eq!(
        vector_component_value(&result, vector_half[0])
            .decimal()
            .fraction_digits(),
        "5"
    );
    assert_eq!(
        sign_and_integer_digits(vector_component_value(&result, vector_negative_one[0])),
        (Some(CssNumberSign::Minus), "1")
    );
}

// N. Incomplete vector shapes.

#[test]
fn incomplete_vector_shapes_are_invalid() {
    let result = qualify(
        604115,
        concat!(
            "a{rotate:1;}",
            "b{rotate:1 2;}",
            "c{rotate:1 2 3;}",
            "d{rotate:1 30deg;}",
            "e{rotate:1 2 30deg;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// O. Too many vector Number components.

#[test]
fn too_many_vector_components_are_invalid() {
    let result = qualify(
        604116,
        concat!("a{rotate:1 0 0 2 30deg;}", "b{rotate:30deg 1 0 0 2;}",),
    );

    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// P. Vector wrong token class.

#[test]
fn vector_wrong_token_classes_are_invalid() {
    let result = qualify(
        604117,
        concat!(
            "a{rotate:1% 0 0 30deg;}",
            "b{rotate:1 0% 0 30deg;}",
            "c{rotate:1 0 0% 30deg;}",
            "d{rotate:1px 0 0 30deg;}",
            "e{rotate:1 1px 0 30deg;}",
            "f{rotate:foo 0 0 30deg;}",
            "g{rotate:1 1deg 0 30deg;}",
        ),
    );

    for index in 0..7 {
        assert_invalid(&result, index);
    }
}

// Q. `&&` grouping / no interleaving -- load-bearing.

#[test]
fn angle_never_interleaves_inside_the_vector() {
    let result = qualify(
        604118,
        concat!(
            "a{rotate:1 0 0 30deg;}",
            "b{rotate:30deg 1 0 0;}",
            "c{rotate:1 30deg 0 0;}",
            "d{rotate:1 0 30deg 0;}",
        ),
    );

    let _ = vector_axis_at(&result, 0);
    let _ = vector_axis_at(&result, 1);
    assert_invalid(&result, 2);
    assert_invalid(&result, 3);
}

// R. Residual Function: single angle shape.

#[test]
fn residual_function_in_single_angle_shape_is_unsupported() {
    let result = qualify(604119, "a{rotate:calc(30deg);}");
    assert_unsupported(&result, 0, CssRotateUnsupportedReason::FunctionValue);
}

// S. Residual Function: keyword-axis shape.

#[test]
fn residual_function_in_keyword_axis_shape_is_unsupported() {
    let result = qualify(
        604120,
        concat!("a{rotate:x calc(30deg);}", "b{rotate:calc(30deg) x;}",),
    );

    for index in 0..2 {
        assert_unsupported(&result, index, CssRotateUnsupportedReason::FunctionValue);
    }
}

// T. Residual Function: vector Number slot.

#[test]
fn residual_function_in_vector_number_slot_is_unsupported() {
    let result = qualify(
        604121,
        concat!(
            "a{rotate:calc(1) 0 0 30deg;}",
            "b{rotate:1 calc(0) 0 30deg;}",
            "c{rotate:1 0 calc(0) 30deg;}",
        ),
    );

    for index in 0..3 {
        assert_unsupported(&result, index, CssRotateUnsupportedReason::FunctionValue);
    }
}

// U. Residual Function: vector angle slot.

#[test]
fn residual_function_in_vector_angle_slot_is_unsupported() {
    let result = qualify(
        604122,
        concat!(
            "a{rotate:1 0 0 calc(30deg);}",
            "b{rotate:calc(30deg) 1 0 0;}",
        ),
    );

    for index in 0..2 {
        assert_unsupported(&result, index, CssRotateUnsupportedReason::FunctionValue);
    }
}

// V. Structurally impossible Function assignments -- one of the most
// important candidate-independent tests: these must be decisively Invalid,
// never softened to Unsupported.

#[test]
fn structurally_impossible_function_assignments_are_invalid_not_unsupported() {
    let result = qualify(
        604123,
        concat!(
            "a{rotate:calc(1) 30deg;}",
            "b{rotate:calc(1) 0 30deg;}",
            "c{rotate:calc(1) 0 0;}",
            "d{rotate:1 calc(2) 30deg;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// W. Later decisive invalidity beats earlier Function ambiguity.

#[test]
fn later_decisive_invalidity_outranks_earlier_function_ambiguity() {
    let result = qualify(
        604124,
        concat!(
            "a{rotate:calc(1) 0 0 30deg extra;}",
            "b{rotate:calc(1) 0 0 30deg junk more;}",
        ),
    );

    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// X. Nested Function delimiter isolation.

#[test]
fn nested_function_comma_is_never_a_top_level_separator() {
    let result = qualify(604125, "a{rotate:calc(min(1, 2)) 0 0 30deg;}");
    assert_unsupported(&result, 0, CssRotateUnsupportedReason::FunctionValue);
}

// Y. Whole-value Function boundary.

#[test]
fn whole_value_function_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(604126, "a{rotate:first-valid(30deg, 45deg);}");
    assert_unsupported(&whole, 0, CssRotateUnsupportedReason::WholeValueFunction);

    let embedded = qualify(
        604127,
        concat!(
            "a{rotate:x first-valid(30deg);}",
            "b{rotate:1 0 0 first-valid(30deg);}",
        ),
    );
    for index in 0..2 {
        assert_invalid(&embedded, index);
    }
}

// Z. Deferred substitution precedes direct classification everywhere.

#[test]
fn deferred_substitution_precedes_direct_classification_everywhere() {
    let result = qualify(
        604128,
        concat!(
            "a{rotate:var(--x);}",
            "b{rotate:x var(--x);}",
            "c{rotate:var(--x) x;}",
            "d{rotate:1 0 0 var(--x);}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(
            &result,
            index,
            CssRotateUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// AA. CSS-wide keyword: sole value only, never embedded.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        604129,
        concat!(
            "a{rotate:initial;}",
            "b{rotate:inherit;}",
            "c{rotate:unset;}",
            "d{rotate:revert;}",
            "e{rotate:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(&whole, index, CssRotateUnsupportedReason::CssWideKeyword);
    }

    let embedded = qualify(
        604130,
        concat!(
            "a{rotate:30deg initial;}",
            "b{rotate:initial 30deg;}",
            "c{rotate:x inherit 30deg;}",
            "d{rotate:1 0 0 revert;}",
        ),
    );
    for index in 0..4 {
        assert_invalid(&embedded, index);
    }
}

// AB. Comma syntax is invalid.

#[test]
fn comma_separated_syntax_is_invalid() {
    let result = qualify(
        604131,
        concat!(
            "a{rotate:x, 30deg;}",
            "b{rotate:30deg, x;}",
            "c{rotate:1, 0, 0, 30deg;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// AC. Trivia/comments; empty/trivia-only values.

#[test]
fn comments_between_and_adjacent_to_components_behave_like_whitespace() {
    let spaced = qualify(604132, "a{rotate:1 0 0 30deg;}");
    let comment_padded = qualify(
        604133,
        "a{rotate:/*a*/1/*b*/ /*c*/0/*d*/ /*e*/0/*f*/ /*g*/30deg/*h*/;}",
    );

    for result in [&spaced, &comment_padded] {
        let _ = vector_axis_at(result, 0);
    }
}

#[test]
fn empty_and_trivia_only_values_are_invalid() {
    let result = qualify(604134, concat!("a{rotate:;}", "b{rotate: /**/  ;}",));
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// AD. `!important` stays outside the authored value window.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(604135, "a{rotate:1 0 0 30deg !important;}");
    let _ = vector_axis_at(&result, 0);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// AE. Repeated declarations remain independent observations.

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(604136, "a{rotate:30deg;rotate:x 30deg;rotate:none;}");

    assert_eq!(result.rotate_observations().len(), 3);
    assert_axis_none(&result, 0);
    assert_axis_keyword(&result, 1, CssRotateAxisKeyword::X);
    assert_none(&result, 2);
}

// AF. Placement: only ordinary declaration placement enters this qualifier.

#[test]
fn nonordinary_contexts_do_not_enter_rotate_dispatch() {
    for (source_id, css) in [
        (604137, "@font-face{rotate:30deg;}"),
        (604138, "@page{rotate:30deg;}"),
        (604139, "@keyframes k{from{rotate:30deg;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.rotate_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

// AG. Incomplete upstream execution constrains qualification to the
// committed prefix.

#[test]
fn incomplete_upstream_execution_constrains_qualification_to_committed_prefix() {
    let incomplete = qualify_with_limits(
        604140,
        "a{rotate:30deg;}b{rotate:x 30deg;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.rotate_observations().len(), 1);
    assert_axis_none(&incomplete, 0);
}

// AH. Determinism: repeated and cross-source runs of equivalent source
// produce identical observations, including evidence resolution.

#[test]
fn repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{rotate:1 0 0 30deg;}",
        "b{rotate:none;}",
        "c{rotate:var(--x);}",
        "d{rotate:none x;}",
        "e{rotate:calc(30deg);}",
        "f{rotate:first-valid(30deg);}",
    );
    let first = qualify(604141, css);
    let repeated = qualify(604141, css);
    let another_source = qualify(604142, css);

    assert_eq!(first.rotate_observations(), repeated.rotate_observations());
    assert_eq!(
        first.rotate_observations(),
        another_source.rotate_observations()
    );
}

// AI. Cross-leaf isolation: neighboring accepted leaves are unaffected by
// this dispatch addition.

#[test]
fn cross_leaf_isolation_from_neighboring_qualified_properties() {
    let result = qualify(
        604143,
        concat!(
            "a{rotate:30deg;rotate:x 30deg;}",
            "b{scale:1;}",
            "c{offset-rotate:auto;}",
            "d{counter-set:foo;}",
            "e{image-resolution:2dppx;}",
        ),
    );

    assert_eq!(result.rotate_observations().len(), 2);
    assert_eq!(result.rotate_observations()[0].occurrence_index(), 0);
    assert_eq!(result.rotate_observations()[1].occurrence_index(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.offset_rotate_observations().len(), 1);
    assert_eq!(result.counter_set_observations().len(), 1);
    assert_eq!(result.image_resolution_observations().len(), 1);
}

// Adversarial sealing: unmatched/nested block evidence never fakes a
// top-level component, mirroring the accepted `scale`/`offset-rotate`
// seal.

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_components() {
    let result = qualify(
        604144,
        concat!(
            "a{rotate:[1 0 0] 30deg;}",
            "b{rotate:(1 0 0) 30deg;}",
            "c{rotate:x) 30deg;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}
