use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssNumberSign, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextIndentComponent, CssTextIndentQualificationOutcome, CssTextIndentUnsupportedReason,
    CssTextIndentValue, CssValueQualificationRunResult, run,
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
) -> &CssTextIndentQualificationOutcome {
    result.text_indent_observations()[index].outcome()
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssTextIndentQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssTextIndentUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssTextIndentQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified components. Panics if the
/// observation at `index` is not `Qualified(Components(_))`.
fn qualified_components(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &[CssTextIndentComponent] {
    match outcome_at(result, index) {
        CssTextIndentQualificationOutcome::Qualified(CssTextIndentValue::Components(
            components,
        )) => components,
        other => panic!("expected Qualified(Components(_)) at index {index}, got {other:?}"),
    }
}

/// One component's role identity, ignoring its run-local evidence reference,
/// for authored-order assertions that do not care which exact tokenizer
/// index backed a `Length`/`Percentage` component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextIndentComponentRole {
    Length,
    Percentage,
    Hanging,
    EachLine,
}

fn component_roles(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> Vec<TextIndentComponentRole> {
    qualified_components(result, index)
        .iter()
        .map(|component| match component {
            CssTextIndentComponent::Length(_) => TextIndentComponentRole::Length,
            CssTextIndentComponent::Percentage(_) => TextIndentComponentRole::Percentage,
            CssTextIndentComponent::Hanging => TextIndentComponentRole::Hanging,
            CssTextIndentComponent::EachLine => TextIndentComponentRole::EachLine,
        })
        .collect()
}

fn assert_component_order(
    result: &CssValueQualificationRunResult,
    index: usize,
    expected: &[TextIndentComponentRole],
) {
    assert_eq!(
        component_roles(result, index),
        expected,
        "authored component order at index {index}"
    );
}

fn component_token(
    result: &CssValueQualificationRunResult,
    index: usize,
    component_index: usize,
) -> &CssTokenKind {
    let evidence = match qualified_components(result, index)[component_index] {
        CssTextIndentComponent::Length(evidence) | CssTextIndentComponent::Percentage(evidence) => {
            evidence
        }
        other => {
            panic!("component at {index}/{component_index} is {other:?}, not Length/Percentage")
        }
    };
    result
        .text_indent_component_token(evidence)
        .unwrap_or_else(|| {
            panic!("component evidence at {index}/{component_index} did not resolve")
        })
}

// A. Direct Length qualifies, including negative values and exact unitless
// zero.

#[test]
fn direct_length_qualifies_including_negative_and_exact_zero() {
    let result = qualify(
        615100,
        concat!(
            "a{text-indent:10px;}",
            "b{text-indent:-30px;}",
            "c{text-indent:0;}",
        ),
    );

    for index in 0..3 {
        assert_component_order(&result, index, &[TextIndentComponentRole::Length]);
    }
    assert!(matches!(
        component_token(&result, 0, 0),
        CssTokenKind::Dimension { .. }
    ));
    assert!(matches!(
        component_token(&result, 1, 0),
        CssTokenKind::Dimension { .. }
    ));
    assert!(matches!(
        component_token(&result, 2, 0),
        CssTokenKind::Number { .. }
    ));
}

// B. Direct Percentage qualifies, including negative values and exact zero.

#[test]
fn direct_percentage_qualifies_including_negative_and_exact_zero() {
    let result = qualify(
        615101,
        concat!(
            "a{text-indent:20%;}",
            "b{text-indent:-40%;}",
            "c{text-indent:0%;}",
        ),
    );

    for index in 0..3 {
        assert_component_order(&result, index, &[TextIndentComponentRole::Percentage]);
        assert!(matches!(
            component_token(&result, index, 0),
            CssTokenKind::Percentage { .. }
        ));
    }
}

// C. Unitless nonzero Number is invalid; never interpreted as a Length.

#[test]
fn unitless_nonzero_number_is_invalid() {
    let result = qualify(
        615102,
        concat!(
            "a{text-indent:10;}",
            "b{text-indent:-10;}",
            "c{text-indent:1.5;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// D. Authored zero identity: `0`, `0px`, and `0%` retain distinct run-local
// evidence -- load-bearing.

#[test]
fn zero_zero_px_and_zero_percent_remain_authored_distinct() {
    let result = qualify(
        615103,
        concat!(
            "a{text-indent:0;}",
            "b{text-indent:0px;}",
            "c{text-indent:0%;}",
        ),
    );

    assert!(matches!(
        qualified_components(&result, 0)[0],
        CssTextIndentComponent::Length(_)
    ));
    assert!(matches!(
        component_token(&result, 0, 0),
        CssTokenKind::Number { .. }
    ));

    assert!(matches!(
        qualified_components(&result, 1)[0],
        CssTextIndentComponent::Length(_)
    ));
    assert!(matches!(
        component_token(&result, 1, 0),
        CssTokenKind::Dimension { .. }
    ));

    assert!(matches!(
        qualified_components(&result, 2)[0],
        CssTextIndentComponent::Percentage(_)
    ));
    assert!(matches!(
        component_token(&result, 2, 0),
        CssTokenKind::Percentage { .. }
    ));
}

// E. Optional `hanging`: both authored orders qualify, order preserved.

#[test]
fn optional_hanging_qualifies_in_both_authored_orders() {
    let result = qualify(
        615104,
        concat!(
            "a{text-indent:10px hanging;}",
            "b{text-indent:hanging 10px;}",
        ),
    );

    assert_component_order(
        &result,
        0,
        &[
            TextIndentComponentRole::Length,
            TextIndentComponentRole::Hanging,
        ],
    );
    assert_component_order(
        &result,
        1,
        &[
            TextIndentComponentRole::Hanging,
            TextIndentComponentRole::Length,
        ],
    );
}

// F. Optional `each-line`: both authored orders qualify, order preserved.

#[test]
fn optional_each_line_qualifies_in_both_authored_orders() {
    let result = qualify(
        615105,
        concat!(
            "a{text-indent:20% each-line;}",
            "b{text-indent:each-line 20%;}",
        ),
    );

    assert_component_order(
        &result,
        0,
        &[
            TextIndentComponentRole::Percentage,
            TextIndentComponentRole::EachLine,
        ],
    );
    assert_component_order(
        &result,
        1,
        &[
            TextIndentComponentRole::EachLine,
            TextIndentComponentRole::Percentage,
        ],
    );
}

// G. All six three-component permutations qualify independently with exact
// authored order preserved -- load-bearing.

#[test]
fn all_six_three_component_permutations_qualify_with_authored_order_preserved() {
    let result = qualify(
        615106,
        concat!(
            "a{text-indent:10px hanging each-line;}",
            "b{text-indent:10px each-line hanging;}",
            "c{text-indent:hanging 10px each-line;}",
            "d{text-indent:hanging each-line 10px;}",
            "e{text-indent:each-line 10px hanging;}",
            "f{text-indent:each-line hanging 10px;}",
        ),
    );

    use TextIndentComponentRole::{EachLine, Hanging, Length};

    assert_component_order(&result, 0, &[Length, Hanging, EachLine]);
    assert_component_order(&result, 1, &[Length, EachLine, Hanging]);
    assert_component_order(&result, 2, &[Hanging, Length, EachLine]);
    assert_component_order(&result, 3, &[Hanging, EachLine, Length]);
    assert_component_order(&result, 4, &[EachLine, Length, Hanging]);
    assert_component_order(&result, 5, &[EachLine, Hanging, Length]);
}

// H. The required `<length-percentage>` anchor cannot be omitted: `hanging`
// and `each-line` never stand alone or together.

#[test]
fn required_length_percentage_anchor_cannot_be_omitted() {
    let result = qualify(
        615107,
        concat!(
            "a{text-indent:hanging;}",
            "b{text-indent:each-line;}",
            "c{text-indent:hanging each-line;}",
            "d{text-indent:each-line hanging;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// I. Duplicate `<length-percentage>` anchor is invalid, regardless of the
// concrete Length/Percentage mix.

#[test]
fn duplicate_length_percentage_anchor_is_invalid() {
    let result = qualify(
        615108,
        concat!(
            "a{text-indent:10px 20px;}",
            "b{text-indent:10px 20%;}",
            "c{text-indent:20% 10px;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// J. Duplicate optional keyword is invalid.

#[test]
fn duplicate_optional_keyword_is_invalid() {
    let result = qualify(
        615109,
        concat!(
            "a{text-indent:10px hanging hanging;}",
            "b{text-indent:10px each-line each-line;}",
            "c{text-indent:hanging 20% hanging;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// K. Four-or-more direct components are invalid; no fourth component can
// qualify.

#[test]
fn four_or_more_components_are_invalid() {
    let result = qualify(
        615110,
        concat!(
            "a{text-indent:10px hanging each-line hanging;}",
            "b{text-indent:10px 20px hanging each-line;}",
        ),
    );

    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// L. Wrong identifiers are invalid.

#[test]
fn wrong_identifiers_are_invalid() {
    let result = qualify(
        615111,
        concat!(
            "a{text-indent:auto;}",
            "b{text-indent:normal;}",
            "c{text-indent:indent;}",
            "d{text-indent:foo;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// M. Top-level commas/delimiters are invalid.

#[test]
fn comma_separated_syntax_is_invalid() {
    let result = qualify(
        615112,
        concat!(
            "a{text-indent:10px, hanging;}",
            "b{text-indent:10px, each-line;}",
            "c{text-indent:10px hanging, each-line;}",
            "d{text-indent:10px,;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// N. CSS-wide keyword boundary: sole is Unsupported, embedded is Invalid.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        615113,
        concat!(
            "a{text-indent:initial;}",
            "b{text-indent:inherit;}",
            "c{text-indent:unset;}",
            "d{text-indent:revert;}",
            "e{text-indent:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssTextIndentUnsupportedReason::CssWideKeyword,
        );
    }

    let embedded = qualify(
        615114,
        concat!(
            "a{text-indent:10px initial;}",
            "b{text-indent:initial 10px;}",
            "c{text-indent:hanging inherit 20%;}",
        ),
    );
    for index in 0..3 {
        assert_invalid(&embedded, index);
    }
}

// O. Deferred substitution boundary.

#[test]
fn deferred_substitution_functions_are_unsupported() {
    let result = qualify(
        615115,
        concat!(
            "a{text-indent:var(--x);}",
            "b{text-indent:hanging var(--x);}",
            "c{text-indent:env(safe-area-inset-top);}",
            "d{text-indent:attr(data-x);}",
            "e{text-indent:--custom(10px);}",
        ),
    );

    for index in 0..5 {
        assert_unsupported(
            &result,
            index,
            CssTextIndentUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// P. Whole-value Function boundary: sole is Unsupported, embedded is
// decisively invalid, never softened.

#[test]
fn whole_value_function_is_unsupported_only_as_the_entire_value() {
    let result = qualify(615116, "a{text-indent:first-valid(10px, 20px);}");
    assert_unsupported(
        &result,
        0,
        CssTextIndentUnsupportedReason::WholeValueFunction,
    );
}

#[test]
fn embedded_whole_value_only_function_is_invalid_never_softened() {
    let result = qualify(
        615117,
        concat!(
            "a{text-indent:10px first-valid(20px);}",
            "b{text-indent:hanging first-valid(20px);}",
            "c{text-indent:first-valid(20px) each-line;}",
        ),
    );
    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// Q. Ordinary residual Function with a feasible LP assignment is
// Unsupported(FunctionValue).

#[test]
fn residual_function_with_feasible_assignment_is_unsupported() {
    let result = qualify(
        615118,
        concat!(
            "a{text-indent:calc(10px);}",
            "b{text-indent:hanging calc(10px);}",
            "c{text-indent:calc(10px) each-line;}",
            "d{text-indent:each-line hanging calc(10px);}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(
            &result,
            index,
            CssTextIndentUnsupportedReason::FunctionValue,
        );
    }
}

// R. Ordinary residual Function with no feasible LP assignment is
// decisively invalid -- load-bearing.

#[test]
fn residual_function_with_impossible_assignment_is_invalid() {
    let result = qualify(
        615119,
        concat!(
            "a{text-indent:10px calc(10px);}",
            "b{text-indent:calc(10px) 20%;}",
            "c{text-indent:calc(10px) calc(20px);}",
            "d{text-indent:hanging hanging calc(10px);}",
            "e{text-indent:calc(10px) each-line each-line;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// S. Case-insensitive and escape-equivalent keyword recognition.

#[test]
fn keywords_are_case_insensitive_and_escape_equivalent() {
    let result = qualify(
        615120,
        concat!(
            "a{text-indent:HANGING 10px;}",
            "b{text-indent:10px EACH-LINE;}",
            "c{text-indent:Each-Line Hanging 10px;}",
            "d{text-indent:\\68 anging 10px;}",
            "e{text-indent:10px \\65 ach-line;}",
        ),
    );

    use TextIndentComponentRole::{EachLine, Hanging, Length};

    assert_component_order(&result, 0, &[Hanging, Length]);
    assert_component_order(&result, 1, &[Length, EachLine]);
    assert_component_order(&result, 2, &[EachLine, Hanging, Length]);
    assert_component_order(&result, 3, &[Hanging, Length]);
    assert_component_order(&result, 4, &[Length, EachLine]);
}

// T. Authored order vs. canonicalization -- load-bearing: the qualifier
// never rewrites a reordered authored sequence into a canonical
// `<length-percentage> hanging each-line` shape.

#[test]
fn authored_order_is_never_canonicalized() {
    let result = qualify(
        615121,
        concat!(
            "a{text-indent:hanging 10px;}",
            "b{text-indent:10px hanging;}",
            "c{text-indent:each-line hanging 10px;}",
            "d{text-indent:10px hanging each-line;}",
        ),
    );

    use TextIndentComponentRole::{EachLine, Hanging, Length};

    let hanging_first = component_roles(&result, 0);
    let length_first = component_roles(&result, 1);
    assert_eq!(hanging_first, vec![Hanging, Length]);
    assert_eq!(length_first, vec![Length, Hanging]);
    assert_ne!(
        hanging_first, length_first,
        "authored orders must remain distinguishable, never canonicalized"
    );

    let each_line_first = component_roles(&result, 2);
    let length_led = component_roles(&result, 3);
    assert_eq!(each_line_first, vec![EachLine, Hanging, Length]);
    assert_eq!(length_led, vec![Length, Hanging, EachLine]);
    assert_ne!(
        each_line_first, length_led,
        "authored three-component orders must remain distinguishable, never canonicalized"
    );
}

// U. Trivia: whitespace/comments never change grouping or classification;
// empty/trivia-only values are invalid.

#[test]
fn comments_between_and_adjacent_to_components_behave_like_whitespace() {
    let spaced = qualify(615122, "a{text-indent:10px hanging each-line;}");
    let comment_padded = qualify(
        615123,
        "a{text-indent:/*a*/10px/*b*/ /*c*/hanging/*d*/ /*e*/each-line/*f*/;}",
    );

    use TextIndentComponentRole::{EachLine, Hanging, Length};

    for result in [&spaced, &comment_padded] {
        assert_component_order(result, 0, &[Length, Hanging, EachLine]);
    }
}

#[test]
fn empty_and_trivia_only_values_are_invalid() {
    let result = qualify(
        615124,
        concat!("a{text-indent:;}", "b{text-indent: /**/  ;}",),
    );
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// V. `!important` stays outside the authored value window.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(615125, "a{text-indent:hanging 10px !important;}");
    use TextIndentComponentRole::{Hanging, Length};
    assert_component_order(&result, 0, &[Hanging, Length]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// W. Duplicate declarations remain independent observations, without
// cascade/deduplication.

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(
        615126,
        "a{text-indent:10px;text-indent:hanging 10px;text-indent:10px hanging each-line;}",
    );

    assert_eq!(result.text_indent_observations().len(), 3);
    assert_eq!(qualified_components(&result, 0).len(), 1);
    assert_eq!(qualified_components(&result, 1).len(), 2);
    assert_eq!(qualified_components(&result, 2).len(), 3);
}

// X. Placement: only ordinary declaration placement enters this qualifier.

#[test]
fn nonordinary_contexts_do_not_enter_text_indent_dispatch() {
    for (source_id, css) in [
        (615127, "@font-face{text-indent:10px;}"),
        (615128, "@page{text-indent:10px;}"),
        (615129, "@keyframes k{from{text-indent:10px;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_indent_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

// Y. Incomplete upstream execution constrains qualification to the
// committed prefix.

#[test]
fn incomplete_upstream_execution_constrains_qualification_to_committed_prefix() {
    let incomplete = qualify_with_limits(
        615130,
        "a{text-indent:10px;}b{text-indent:hanging 10px;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.text_indent_observations().len(), 1);
    assert_eq!(qualified_components(&incomplete, 0).len(), 1);
}

// Z. Determinism.

#[test]
fn repeated_and_cross_source_runs_are_deterministic() {
    let css = concat!(
        "a{text-indent:hanging 10px each-line;}",
        "b{text-indent:var(--x);}",
        "c{text-indent:calc(10px);}",
        "d{text-indent:first-valid(10px, 20px);}",
        "e{text-indent:each-line -30%;}",
    );
    let first = qualify(615131, css);
    let repeated = qualify(615131, css);
    let another_source = qualify(615132, css);

    assert_eq!(
        first.text_indent_observations(),
        repeated.text_indent_observations()
    );
    assert_eq!(
        first.text_indent_observations(),
        another_source.text_indent_observations()
    );
}

// AA. Cross-leaf isolation.

#[test]
fn cross_leaf_isolation_from_neighboring_qualified_properties() {
    let result = qualify(
        615133,
        concat!(
            "a{text-indent:10px;text-indent:hanging 10px;}",
            "b{transform-origin:left;}",
            "c{transform-style:flat;}",
            "d{transform-box:border-box;}",
            "e{translate:10px;}",
            "f{word-spacing:1px;}",
            "g{line-height:1;}",
        ),
    );

    assert_eq!(result.text_indent_observations().len(), 2);
    assert_eq!(result.text_indent_observations()[0].occurrence_index(), 0);
    assert_eq!(result.text_indent_observations()[1].occurrence_index(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.word_spacing_observations().len(), 1);
    assert_eq!(result.line_height_observations().len(), 1);
}

// Negative-value sign preservation: negative Length/Percentage evidence
// resolves with a retained Minus sign, never normalized away.

#[test]
fn negative_length_and_percentage_signs_are_preserved() {
    let result = qualify(
        615134,
        concat!("a{text-indent:-30px;}", "b{text-indent:-40%;}"),
    );

    let length_sign = match component_token(&result, 0, 0) {
        CssTokenKind::Dimension { value, .. } => value.sign(),
        other => panic!("expected Dimension token, got {other:?}"),
    };
    assert_eq!(length_sign, Some(CssNumberSign::Minus));

    let percentage_sign = match component_token(&result, 1, 0) {
        CssTokenKind::Percentage { value } => value.sign(),
        other => panic!("expected Percentage token, got {other:?}"),
    };
    assert_eq!(percentage_sign, Some(CssNumberSign::Minus));
}
