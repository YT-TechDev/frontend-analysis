use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssBoxSizingQualificationOutcome, CssBoxSizingValue, CssDirectionQualificationOutcome,
    CssDirectionValue, CssListStylePositionQualificationOutcome,
    CssListStylePositionUnsupportedReason, CssListStylePositionValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Inside,
    Outside,
    Invalid,
    UnsupportedCssWide,
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

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .list_style_position_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

fn expected_outcome(expected: ExpectedOutcome) -> CssListStylePositionQualificationOutcome {
    match expected {
        ExpectedOutcome::Inside => {
            CssListStylePositionQualificationOutcome::Qualified(CssListStylePositionValue::Inside)
        }
        ExpectedOutcome::Outside => {
            CssListStylePositionQualificationOutcome::Qualified(CssListStylePositionValue::Outside)
        }
        ExpectedOutcome::Invalid => {
            CssListStylePositionQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssListStylePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssListStylePositionUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssListStylePositionQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssListStylePositionUnsupportedReason::FunctionValue,
            )
        }
    }
}

// A/B/C/D/E: direct keyword identity, case-insensitivity, the WPT-invalid
// boundary (`auto`, `inside outside`), wrong identifiers (including the
// #4209 future-syntax guard `outside match-parent`), duplicate-keyword
// sequences, non-identifier token classes, and empty/unrelated-property
// values.
#[test]
fn handwritten_list_style_position_matrix_matches_the_selected_normative_profile() {
    let css = concat!(
        "a{list-style-position:inside;}",
        "b{list-style-position:outside;}",
        "c{list-style-position:OUTSIDE;}",
        "d{LIST-STYLE-POSITION:InSiDe;}",
        "e{list-style-position:auto;}",
        "f{list-style-position:none;}",
        "g{list-style-position:left;}",
        "h{list-style-position:right;}",
        "i{list-style-position:match-self;}",
        "j{list-style-position:match-parent;}",
        "k{list-style-position:outside match-parent;}",
        "l{list-style-position:inside outside;}",
        "m{list-style-position:outside inside;}",
        "n{list-style-position:inside inside;}",
        "o{list-style-position:outside outside;}",
        "p{list-style-position:1;}",
        "q{list-style-position:1px;}",
        "r{list-style-position:10%;}",
        "s{list-style-position:\"inside\";}",
        "t{list-style-position:#fff;}",
        "u{list-style-position:;}",
        "v{list-style-position:inside,outside;}",
        "w{color:inside;}",
    );
    let result = qualify(1, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Inside,
            ExpectedOutcome::Outside,
            ExpectedOutcome::Outside,
            ExpectedOutcome::Inside,
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
fn css_wide_keywords_remain_profile_unsupported_not_authored_invalid() {
    let css = concat!(
        "a{list-style-position:initial;}",
        "b{list-style-position:inherit;}",
        "c{list-style-position:unset;}",
        "d{list-style-position:revert;}",
        "e{list-style-position:revert-layer;}",
        "f{list-style-position:revert-rule;}",
    );
    let result = qualify(2, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::UnsupportedCssWide,
        ],
    );

    // The initial value is `outside`, but authored `initial` must never be
    // synthesized into `Qualified(Outside)`.
    for outcome in result
        .list_style_position_observations()
        .iter()
        .map(|observation| observation.outcome())
    {
        assert_ne!(
            outcome,
            CssListStylePositionQualificationOutcome::Qualified(CssListStylePositionValue::Outside)
        );
    }
}

#[test]
fn embedded_css_wide_keyword_in_multi_component_value_is_invalid() {
    let css = concat!(
        "a{list-style-position:inside initial;}",
        "b{list-style-position:inherit outside;}",
    );
    let result = qualify(3, css);

    assert_expected(
        &result,
        &[ExpectedOutcome::Invalid, ExpectedOutcome::Invalid],
    );
}

#[test]
fn escaped_identifiers_comments_and_priority_use_retained_lexical_meaning() {
    let css = concat!(
        r"a{l\69 st-style-position:/**/\69 nside/**/!important;}",
        r"b{list-style-position:o\75 tside;}",
    );
    let result = qualify(4, css);

    assert_expected(
        &result,
        &[ExpectedOutcome::Inside, ExpectedOutcome::Outside],
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn profile_unsupported_functions_fail_open_but_ordinary_functions_are_invalid() {
    let css = concat!(
        "a{list-style-position:var(--list-position);}",
        "b{list-style-position:env(foo);}",
        "c{list-style-position:attr(data-position);}",
        "d{list-style-position:--custom();}",
        "e{list-style-position:first-valid(inside,outside);}",
        "f{list-style-position:cycle(inside,outside);}",
        "g{list-style-position:interpolate(0%,0:inside,1:outside);}",
        "h{list-style-position:foo();}",
        "i{list-style-position:calc(1);}",
        "j{list-style-position:calc(1px);}",
    );
    let result = qualify(5, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn whole_value_functions_require_entire_value_placement() {
    let css = concat!(
        "a{list-style-position:inside first-valid(outside);}",
        "b{list-style-position:first-valid(inside) outside;}",
        "c{list-style-position:cycle(inside,outside) foo();}",
        "d{list-style-position:inside var(--x);}",
    );
    let result = qualify(6, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::UnsupportedFunction,
        ],
    );
}

#[test]
fn duplicate_selected_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        7,
        "a{list-style-position:inside;list-style-position:outside;}",
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Inside, ExpectedOutcome::Outside],
    );
    assert_eq!(
        result.list_style_position_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.list_style_position_observations()[1].occurrence_index(),
        1
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_list_style_position_observations() {
    for (source_id, css) in [
        (10, "@font-face{list-style-position:inside;}"),
        (11, "@page{list-style-position:inside;}"),
        (12, "@page{@top-left{list-style-position:inside;}}"),
        (13, "@keyframes k{from{list-style-position:inside;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.list_style_position_observations().is_empty(),
            "nonordinary declaration context produced a list-style-position observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_the_committed_list_style_position_prefix_and_completion() {
    let result = qualify_with_limits(
        20,
        "a{list-style-position:inside;list-style-position:outside;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Inside]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{list-style-position:inside;}",
        "b{list-style-position:inherit;}",
        "c{list-style-position:foo;}",
    );
    let first = qualify(30, css);
    let repeated = qualify(30, css);
    let another_source = qualify(31, css);

    assert_eq!(
        first.list_style_position_observations(),
        repeated.list_style_position_observations()
    );
    assert_eq!(
        first.list_style_position_observations(),
        another_source.list_style_position_observations()
    );
}

#[test]
fn cross_leaf_isolation_does_not_alter_direction_or_box_sizing_observations() {
    let result = qualify(
        40,
        concat!(
            "a{direction:ltr;list-style-position:inside;box-sizing:border-box;}",
            "b{list-style-position:outside;direction:rtl;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Inside, ExpectedOutcome::Outside],
    );
    assert_eq!(result.direction_observations().len(), 2);
    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(
        result.direction_observations()[0].outcome(),
        CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Ltr)
    );
    assert_eq!(
        result.direction_observations()[1].outcome(),
        CssDirectionQualificationOutcome::Qualified(CssDirectionValue::Rtl)
    );
    assert_eq!(
        result.box_sizing_observations()[0].outcome(),
        CssBoxSizingQualificationOutcome::Qualified(CssBoxSizingValue::BorderBox)
    );
}
