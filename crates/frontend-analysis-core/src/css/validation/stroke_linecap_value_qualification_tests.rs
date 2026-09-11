use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssStrokeLinecapQualificationOutcome, CssStrokeLinecapUnsupportedReason, CssStrokeLinecapValue,
    CssTransformBoxQualificationOutcome, CssTransformBoxValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Butt,
    Round,
    Square,
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
        .stroke_linecap_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

fn expected_outcome(expected: ExpectedOutcome) -> CssStrokeLinecapQualificationOutcome {
    match expected {
        ExpectedOutcome::Butt => {
            CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Butt)
        }
        ExpectedOutcome::Round => {
            CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Round)
        }
        ExpectedOutcome::Square => {
            CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Square)
        }
        ExpectedOutcome::Invalid => {
            CssStrokeLinecapQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssStrokeLinecapQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeLinecapUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssStrokeLinecapQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssStrokeLinecapUnsupportedReason::FunctionValue,
            )
        }
    }
}

// A/B/C/D/E/F: direct keyword identity, case-insensitivity, the WPT-invalid
// boundary (`auto`, `butt round`), reversed/duplicate multi-keyword
// mismatches, the SVGWG #798 future-syntax guard (`short`), the
// Fill-and-Stroke-3 "Perfect-World Syntax" isolation guard (`none`), other
// wrong identifiers, non-identifier token classes, and empty/comma/
// unrelated-property values.
#[test]
fn handwritten_stroke_linecap_matrix_matches_the_selected_normative_profile() {
    let css = concat!(
        "a{stroke-linecap:butt;}",
        "b{stroke-linecap:round;}",
        "c{stroke-linecap:square;}",
        "d{stroke-linecap:BUTT;}",
        "e{stroke-linecap:RoUnD;}",
        "f{STROKE-LINECAP:SQUARE;}",
        "g{stroke-linecap:auto;}",
        "h{stroke-linecap:butt round;}",
        "i{stroke-linecap:round butt;}",
        "j{stroke-linecap:butt butt;}",
        "k{stroke-linecap:round round;}",
        "l{stroke-linecap:square square;}",
        "m{stroke-linecap:short;}",
        "n{stroke-linecap:none;}",
        "o{stroke-linecap:left;}",
        "p{stroke-linecap:right;}",
        "q{stroke-linecap:miter;}",
        "r{stroke-linecap:bevel;}",
        "s{stroke-linecap:1;}",
        "t{stroke-linecap:1px;}",
        "u{stroke-linecap:10%;}",
        "v{stroke-linecap:\"round\";}",
        "w{stroke-linecap:#fff;}",
        "x{stroke-linecap:;}",
        "y{stroke-linecap:butt,round;}",
        "z{color:butt;}",
    );
    let result = qualify(1, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Butt,
            ExpectedOutcome::Round,
            ExpectedOutcome::Square,
            ExpectedOutcome::Butt,
            ExpectedOutcome::Round,
            ExpectedOutcome::Square,
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
        "a{stroke-linecap:initial;}",
        "b{stroke-linecap:inherit;}",
        "c{stroke-linecap:unset;}",
        "d{stroke-linecap:revert;}",
        "e{stroke-linecap:revert-layer;}",
        "f{stroke-linecap:revert-rule;}",
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

    // The initial value is `butt`, but authored `initial` must never be
    // synthesized into `Qualified(Butt)`.
    for outcome in result
        .stroke_linecap_observations()
        .iter()
        .map(|observation| observation.outcome())
    {
        assert_ne!(
            outcome,
            CssStrokeLinecapQualificationOutcome::Qualified(CssStrokeLinecapValue::Butt)
        );
    }
}

#[test]
fn embedded_css_wide_keyword_in_multi_component_value_is_invalid() {
    let css = concat!(
        "a{stroke-linecap:butt initial;}",
        "b{stroke-linecap:inherit round;}",
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
        r"a{stroke-l\69 necap:/**/\62 utt/**/!important;}",
        r"b{stroke-linecap:r\6f und;}",
        r"c{stroke-linecap:squ\61 re;}",
    );
    let result = qualify(4, css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Butt,
            ExpectedOutcome::Round,
            ExpectedOutcome::Square,
        ],
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
        "a{stroke-linecap:var(--linecap);}",
        "b{stroke-linecap:env(foo);}",
        "c{stroke-linecap:attr(data-linecap);}",
        "d{stroke-linecap:--custom();}",
        "e{stroke-linecap:first-valid(butt,round);}",
        "f{stroke-linecap:cycle(butt,round);}",
        "g{stroke-linecap:interpolate(0%,0:butt,1:round);}",
        "h{stroke-linecap:foo();}",
        "i{stroke-linecap:calc(1);}",
        "j{stroke-linecap:calc(1px);}",
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
        "a{stroke-linecap:butt first-valid(round);}",
        "b{stroke-linecap:first-valid(butt) round;}",
        "c{stroke-linecap:cycle(butt,round) foo();}",
        "d{stroke-linecap:butt var(--x);}",
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
    let result = qualify(7, "a{stroke-linecap:butt;stroke-linecap:round;}");

    assert_expected(&result, &[ExpectedOutcome::Butt, ExpectedOutcome::Round]);
    assert_eq!(
        result.stroke_linecap_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.stroke_linecap_observations()[1].occurrence_index(),
        1
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_do_not_become_stroke_linecap_observations() {
    for (source_id, css) in [
        (10, "@font-face{stroke-linecap:butt;}"),
        (11, "@page{stroke-linecap:butt;}"),
        (12, "@page{@top-left{stroke-linecap:butt;}}"),
        (13, "@keyframes k{from{stroke-linecap:butt;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.stroke_linecap_observations().is_empty(),
            "nonordinary declaration context produced a stroke-linecap observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_the_committed_stroke_linecap_prefix_and_completion() {
    let result = qualify_with_limits(
        20,
        "a{stroke-linecap:butt;stroke-linecap:round;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Butt]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{stroke-linecap:butt;}",
        "b{stroke-linecap:inherit;}",
        "c{stroke-linecap:foo;}",
    );
    let first = qualify(30, css);
    let repeated = qualify(30, css);
    let another_source = qualify(31, css);

    assert_eq!(
        first.stroke_linecap_observations(),
        repeated.stroke_linecap_observations()
    );
    assert_eq!(
        first.stroke_linecap_observations(),
        another_source.stroke_linecap_observations()
    );
}

// Cross-leaf/property isolation: `transform-box: stroke-box` shares the
// `stroke` substring and sibling `stroke-*` property names share the
// `stroke-` prefix, but none of them may be misclassified as a
// `stroke-linecap` observation, and `stroke-linecap` must not interfere
// with their own unrelated qualification.
#[test]
fn cross_leaf_isolation_does_not_alter_or_leak_into_sibling_properties() {
    let result = qualify(
        40,
        concat!(
            "a{stroke-linecap:butt;transform-box:stroke-box;}",
            "b{stroke-linejoin:round;}",
            "c{stroke-width:2px;}",
            "d{stroke-opacity:0.5;}",
            "e{stroke-miterlimit:4;}",
            "f{stroke-linecap:round;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Butt, ExpectedOutcome::Round]);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(
        result.transform_box_observations()[0].outcome(),
        CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::StrokeBox)
    );
}

// Perfect-World Syntax isolation: a declaration using the non-operative
// `stroke-cap` property name must not produce a `stroke-linecap`
// observation, and `stroke-cap` is not treated as an alias.
#[test]
fn perfect_world_stroke_cap_property_name_is_not_an_alias() {
    let result = qualify(41, "a{stroke-cap:round;stroke-linecap:round;}");

    assert_expected(&result, &[ExpectedOutcome::Round]);
    assert_eq!(result.stroke_linecap_observations().len(), 1);
}
