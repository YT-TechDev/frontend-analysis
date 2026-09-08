use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssOffsetRotateQualificationOutcome, CssOffsetRotateUnsupportedReason, CssOffsetRotateValue,
    CssValueQualificationRunResult, run,
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

fn qualified(value: CssOffsetRotateValue) -> CssOffsetRotateQualificationOutcome {
    CssOffsetRotateQualificationOutcome::Qualified(value)
}

fn unsupported(reason: CssOffsetRotateUnsupportedReason) -> CssOffsetRotateQualificationOutcome {
    CssOffsetRotateQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssOffsetRotateQualificationOutcome],
) {
    let actual: Vec<_> = result
        .offset_rotate_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn direct_angle_literals_qualify_regardless_of_sign_magnitude_or_unit_case() {
    use CssOffsetRotateValue::DirectAngle;

    let result = qualify(
        58600,
        concat!(
            "a{offset-rotate:0deg;}",
            "b{offset-rotate:-0deg;}",
            "c{offset-rotate:+0deg;}",
            "d{offset-rotate:45deg;}",
            "e{offset-rotate:-400deg;}",
            "f{offset-rotate:100grad;}",
            "g{offset-rotate:1.57rad;}",
            "h{offset-rotate:.5turn;}",
            "i{offset-rotate:-1deg;}",
            "j{offset-rotate:-.5turn;}",
            "k{offset-rotate:-3.14rad;}",
            "l{offset-rotate:45DEG;}",
            "m{offset-rotate:1RaD;}",
            "n{offset-rotate:.5TuRn;}",
            "o{offset-rotate:100GRAD;}",
            "p{offset-rotate:1e2deg;}",
        ),
    );

    let expected = vec![qualified(DirectAngle); 16];
    assert_expected(&result, &expected);
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn unitless_zero_and_plain_numbers_are_invalid() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        58601,
        concat!(
            "a{offset-rotate:0;}",
            "b{offset-rotate:+0;}",
            "c{offset-rotate:-0;}",
            "d{offset-rotate:1;}",
            "e{offset-rotate:.5;}",
        ),
    );

    let expected = vec![Invalid; 5];
    assert_expected(&result, &expected);
}

#[test]
fn wrong_dimension_units_are_invalid() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        58602,
        concat!(
            "a{offset-rotate:1px;}",
            "b{offset-rotate:1em;}",
            "c{offset-rotate:1s;}",
            "d{offset-rotate:1ms;}",
            "e{offset-rotate:1hz;}",
            "f{offset-rotate:1dpi;}",
        ),
    );

    let expected = vec![Invalid; 6];
    assert_expected(&result, &expected);
}

#[test]
fn wrong_token_classes_are_invalid() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        58603,
        concat!(
            "a{offset-rotate:50%;}",
            r#"b{offset-rotate:"45deg";}"#,
            "c{offset-rotate:none;}",
            "d{offset-rotate:normal;}",
            "e{offset-rotate:fixed;}",
            "f{offset-rotate:foo;}",
            "g{offset-rotate:url(foo);}",
            "h{offset-rotate:#fff;}",
        ),
    );

    let expected = vec![Invalid; 8];
    assert_expected(&result, &expected);
}

#[test]
fn sole_keyword_components_qualify_ascii_case_insensitively() {
    use CssOffsetRotateValue::{Auto, Reverse};

    let result = qualify(
        58604,
        concat!(
            "a{offset-rotate:auto;}",
            "b{offset-rotate:AUTO;}",
            "c{offset-rotate:Auto;}",
            "d{offset-rotate:reverse;}",
            "e{offset-rotate:REVERSE;}",
            "f{offset-rotate:Reverse;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(Auto),
            qualified(Auto),
            qualified(Auto),
            qualified(Reverse),
            qualified(Reverse),
            qualified(Reverse),
        ],
    );
}

#[test]
fn keyword_and_angle_composition_is_authored_order_independent() {
    use CssOffsetRotateValue::{AutoAndDirectAngle, ReverseAndDirectAngle};

    let result = qualify(
        58605,
        concat!(
            "a{offset-rotate:auto 5turn;}",
            "b{offset-rotate:5turn auto;}",
            "c{offset-rotate:reverse -90deg;}",
            "d{offset-rotate:-90deg reverse;}",
            "e{offset-rotate:auto 45deg;}",
            "f{offset-rotate:45deg auto;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(AutoAndDirectAngle),
            qualified(AutoAndDirectAngle),
            qualified(ReverseAndDirectAngle),
            qualified(ReverseAndDirectAngle),
            qualified(AutoAndDirectAngle),
            qualified(AutoAndDirectAngle),
        ],
    );
}

#[test]
fn cardinality_and_duplicate_component_failures_are_invalid() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        58606,
        concat!(
            "a{offset-rotate:auto reverse;}",
            "b{offset-rotate:reverse auto;}",
            "c{offset-rotate:auto auto;}",
            "d{offset-rotate:reverse reverse;}",
            "e{offset-rotate:10deg 20deg;}",
            "f{offset-rotate:10deg 20rad;}",
            "g{offset-rotate:reverse 30deg auto;}",
            "h{offset-rotate:30deg reverse 40deg;}",
            "i{offset-rotate:45deg reverse 90deg;}",
        ),
    );

    let expected = vec![Invalid; 9];
    assert_expected(&result, &expected);
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssOffsetRotateUnsupportedReason::CssWideKeyword;

    let whole = qualify(
        58607,
        concat!(
            "a{offset-rotate:initial;}",
            "b{offset-rotate:inherit;}",
            "c{offset-rotate:unset;}",
            "d{offset-rotate:revert;}",
            "e{offset-rotate:revert-layer;}",
        ),
    );
    assert_expected(
        &whole,
        &[
            unsupported(CssWideKeyword),
            unsupported(CssWideKeyword),
            unsupported(CssWideKeyword),
            unsupported(CssWideKeyword),
            unsupported(CssWideKeyword),
        ],
    );

    let mixed = qualify(
        58608,
        concat!(
            "a{offset-rotate:initial 45deg;}",
            "b{offset-rotate:auto inherit;}",
            "c{offset-rotate:45deg revert;}",
        ),
    );
    let expected = vec![Invalid; 3];
    assert_expected(&mixed, &expected);
}

#[test]
fn deferred_substitution_precedes_structural_recognition_everywhere() {
    use CssOffsetRotateUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        58609,
        concat!(
            "a{offset-rotate:var(--x);}",
            "b{offset-rotate:auto var(--x);}",
            "c{offset-rotate:var(--x) reverse;}",
        ),
    );

    assert_expected(
        &result,
        &[
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
            unsupported(DeferredSubstitutionFunction),
        ],
    );
}

#[test]
fn numeric_function_in_angle_position_is_unsupported_absent_decisive_invalid_evidence() {
    use CssOffsetRotateUnsupportedReason::FunctionValue;

    let result = qualify(
        58610,
        concat!(
            "a{offset-rotate:calc(45deg);}",
            "b{offset-rotate:min(10deg, 20deg);}",
            "c{offset-rotate:max(10deg, 20deg);}",
            "d{offset-rotate:clamp(0deg, 20deg, 90deg);}",
            "e{offset-rotate:auto calc(45deg);}",
            "f{offset-rotate:calc(45deg) reverse;}",
            "g{offset-rotate:reverse calc(-90deg);}",
            "h{offset-rotate:calc(-90deg) reverse;}",
        ),
    );

    let expected = vec![unsupported(FunctionValue); 8];
    assert_expected(&result, &expected);
}

#[test]
fn decisive_structural_invalidity_outranks_residual_function_unsupported() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        58611,
        concat!(
            "a{offset-rotate:auto reverse calc(45deg);}",
            "b{offset-rotate:10deg calc(20deg);}",
            "c{offset-rotate:calc(20deg) 10deg;}",
        ),
    );

    let expected = vec![Invalid; 3];
    assert_expected(&result, &expected);
}

#[test]
fn generic_whole_value_function_is_unsupported_only_at_whole_value_placement() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssOffsetRotateUnsupportedReason::WholeValueFunction;

    let whole = qualify(58612, "a{offset-rotate:first-valid(45deg, 90deg);}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let embedded = qualify(
        58613,
        concat!(
            "a{offset-rotate:auto first-valid(45deg);}",
            "b{offset-rotate:first-valid(45deg) reverse;}",
        ),
    );
    let expected = vec![Invalid; 2];
    assert_expected(&embedded, &expected);
}

#[test]
fn trivia_and_important_priority_do_not_alter_component_semantics() {
    use CssOffsetRotateValue::AutoAndDirectAngle;

    let result = qualify(
        58614,
        concat!(
            "a{offset-rotate:/*a*/auto/*b*/45deg;}",
            "b{offset-rotate:45deg/*x*/auto;}",
            "c{offset-rotate: auto  45deg ;}",
            "d{offset-rotate:auto 45deg !important;}",
        ),
    );

    let expected = vec![qualified(AutoAndDirectAngle); 4];
    assert_expected(&result, &expected);
    assert!(
        result.upstream_parser_result().occurrences()[3]
            .priority()
            .is_some()
    );
}

#[test]
fn empty_value_is_invalid() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(58615, "a{offset-rotate: ;}");
    assert_expected(&result, &[Invalid]);
}

#[test]
fn duplicate_occurrences_preserve_authored_order_without_cascade_selection() {
    use CssOffsetRotateValue::{Auto, DirectAngle, Reverse};

    let result = qualify(
        58616,
        concat!(
            "a{offset-rotate:auto;offset-rotate:45deg;offset-rotate:reverse 90deg;}",
            "b{offset-rotate:reverse;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified(Auto),
            qualified(DirectAngle),
            qualified(CssOffsetRotateValue::ReverseAndDirectAngle),
            qualified(Reverse),
        ],
    );
    assert_eq!(result.offset_rotate_observations().len(), 4);
    assert_eq!(result.offset_rotate_observations()[0].occurrence_index(), 0);
    assert_eq!(result.offset_rotate_observations()[2].occurrence_index(), 2);
}

#[test]
fn nonordinary_contexts_do_not_enter_offset_rotate_dispatch() {
    for (source_id, css) in [
        (58617, "@font-face{offset-rotate:45deg;}"),
        (58618, "@page{offset-rotate:45deg;}"),
        (58619, "@keyframes k{from{offset-rotate:45deg;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.offset_rotate_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_components() {
    use CssOffsetRotateQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssOffsetRotateUnsupportedReason::FunctionValue;

    let result = qualify(
        58620,
        concat!(
            // A nested comma inside the Function's balanced block never
            // leaks out to split or duplicate top-level components.
            "a{offset-rotate:calc(45deg, 90deg) auto;}",
            // A bare (non-Function) parenthesis block is multi-token and
            // therefore never a valid direct keyword/angle component.
            "b{offset-rotate:(45deg) auto;}",
            // A stray unmatched closing token after a valid keyword still
            // starts its own trailing component; it is never ignored.
            "c{offset-rotate:auto);}",
        ),
    );
    assert_expected(&result, &[unsupported(FunctionValue), Invalid, Invalid]);
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssOffsetRotateValue::Auto;

    let incomplete = qualify_with_limits(
        58621,
        "a{offset-rotate:auto;}b{offset-rotate:45deg;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified(Auto)]);

    let css = concat!(
        "a{offset-rotate:auto 45deg;}",
        "b{offset-rotate:var(--x);}",
        "c{offset-rotate:-400deg;}",
    );
    let first = qualify(58622, css);
    let repeated = qualify(58622, css);
    let another_source = qualify(58623, css);

    assert_eq!(
        first.offset_rotate_observations(),
        repeated.offset_rotate_observations()
    );
    assert_eq!(
        first.offset_rotate_observations(),
        another_source.offset_rotate_observations()
    );
}

#[test]
fn existing_leaf_dispatch_remains_isolated_from_offset_rotate() {
    use crate::css::value_qualification::{
        CssAspectRatioQualificationOutcome, CssAspectRatioValue,
    };

    let result = qualify(
        58624,
        concat!(
            "a{offset-rotate:auto;aspect-ratio:auto;}",
            "b{anchor-name:--foo;}",
        ),
    );

    assert_eq!(result.offset_rotate_observations().len(), 1);
    assert_eq!(result.aspect_ratio_observations().len(), 1);
    assert_eq!(result.anchor_name_observations().len(), 1);
    assert_eq!(
        result.aspect_ratio_observations()[0].outcome(),
        CssAspectRatioQualificationOutcome::Qualified(CssAspectRatioValue::Auto)
    );
}
