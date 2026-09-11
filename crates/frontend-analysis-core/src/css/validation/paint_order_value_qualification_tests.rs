use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssPaintOrderComponent, CssPaintOrderQualificationOutcome, CssPaintOrderUnsupportedReason,
    CssPaintOrderValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Normal,
    Components(&'static [CssPaintOrderComponent]),
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferredFunction,
    UnsupportedWholeValueFunction,
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
    let actual = result.paint_order_observations();
    assert_eq!(actual.len(), expected.len());

    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssPaintOrderQualificationOutcome::Qualified(CssPaintOrderValue::Normal),
                ExpectedOutcome::Normal,
            ) => {}
            (
                CssPaintOrderQualificationOutcome::Qualified(CssPaintOrderValue::Components(
                    components,
                )),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssPaintOrderQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssPaintOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssPaintOrderUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssPaintOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssPaintOrderUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssPaintOrderQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssPaintOrderUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => {
                panic!("unexpected paint-order outcome: {actual:?}, expected {expected:?}")
            }
        }
    }
}

fn paint_order_components(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> Vec<CssPaintOrderComponent> {
    match result.paint_order_observations()[index].outcome() {
        CssPaintOrderQualificationOutcome::Qualified(CssPaintOrderValue::Components(
            components,
        )) => components.authored_components().to_vec(),
        other => panic!("expected a Components outcome, got {other:?}"),
    }
}

#[test]
fn handwritten_full_direct_grammar_matches_pinned_wpt_and_derived_theorem() {
    use CssPaintOrderComponent::{Fill, Markers, Stroke};

    let result = qualify(
        86_400,
        concat!(
            "a{paint-order:normal;}",
            "b{paint-order:fill;}",
            "c{paint-order:stroke;}",
            "d{paint-order:markers;}",
            "e{paint-order:fill stroke;}",
            "f{paint-order:fill markers;}",
            "g{paint-order:stroke fill;}",
            "h{paint-order:stroke markers;}",
            "i{paint-order:markers fill;}",
            "j{paint-order:markers stroke;}",
            "k{paint-order:fill stroke markers;}",
            "l{paint-order:fill markers stroke;}",
            "m{paint-order:stroke fill markers;}",
            "n{paint-order:stroke markers fill;}",
            "o{paint-order:markers fill stroke;}",
            "p{paint-order:markers stroke fill;}",
            "q{paint-order:;}",
            "r{paint-order:normal fill;}",
            "s{paint-order:fill normal;}",
            "t{paint-order:normal stroke;}",
            "u{paint-order:stroke normal;}",
            "v{paint-order:normal markers;}",
            "w{paint-order:markers normal;}",
            "x{paint-order:fill fill;}",
            "y{paint-order:stroke stroke;}",
            "z{paint-order:markers markers;}",
            "aa{paint-order:fill stroke fill;}",
            "ab{paint-order:fill stroke markers fill;}",
            "ac{paint-order:unknown;}",
            "ad{paint-order:1;}",
            "ae{paint-order:1px;}",
            "af{paint-order:50%;}",
            "ag{paint-order:\"fill\";}",
            "ah{paint-order:#fff;}",
            "ai{paint-order:fill, stroke;}",
            "aj{paint-order:foo();}",
            "ak{paint-order:calc(1);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Components(&[Fill]),
            ExpectedOutcome::Components(&[Stroke]),
            ExpectedOutcome::Components(&[Markers]),
            ExpectedOutcome::Components(&[Fill, Stroke]),
            ExpectedOutcome::Components(&[Fill, Markers]),
            ExpectedOutcome::Components(&[Stroke, Fill]),
            ExpectedOutcome::Components(&[Stroke, Markers]),
            ExpectedOutcome::Components(&[Markers, Fill]),
            ExpectedOutcome::Components(&[Markers, Stroke]),
            ExpectedOutcome::Components(&[Fill, Stroke, Markers]),
            ExpectedOutcome::Components(&[Fill, Markers, Stroke]),
            ExpectedOutcome::Components(&[Stroke, Fill, Markers]),
            ExpectedOutcome::Components(&[Stroke, Markers, Fill]),
            ExpectedOutcome::Components(&[Markers, Fill, Stroke]),
            ExpectedOutcome::Components(&[Markers, Stroke, Fill]),
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
fn authored_order_and_cardinality_are_preserved_without_omission_or_collapse() {
    use CssPaintOrderComponent::{Fill, Markers, Stroke};

    let result = qualify(
        86_401,
        concat!(
            "a{paint-order:fill;}",
            "b{paint-order:fill stroke;}",
            "c{paint-order:fill stroke markers;}",
            "d{paint-order:stroke;}",
            "e{paint-order:stroke fill markers;}",
            "f{paint-order:markers;}",
            "g{paint-order:markers fill stroke;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Fill]),
            ExpectedOutcome::Components(&[Fill, Stroke]),
            ExpectedOutcome::Components(&[Fill, Stroke, Markers]),
            ExpectedOutcome::Components(&[Stroke]),
            ExpectedOutcome::Components(&[Stroke, Fill, Markers]),
            ExpectedOutcome::Components(&[Markers]),
            ExpectedOutcome::Components(&[Markers, Fill, Stroke]),
        ],
    );

    // Explicit guards: neither omitted-operation synthesis (SVG2 effective
    // paint order) nor WPT shortest-serialization collapse may rewrite one
    // authored form into another inside this qualification layer.
    let fill_only = paint_order_components(&result, 0);
    let fill_stroke = paint_order_components(&result, 1);
    let fill_stroke_markers = paint_order_components(&result, 2);
    let stroke_only = paint_order_components(&result, 3);
    let stroke_fill_markers = paint_order_components(&result, 4);
    let markers_only = paint_order_components(&result, 5);
    let markers_fill_stroke = paint_order_components(&result, 6);

    assert_ne!(fill_only, fill_stroke);
    assert_ne!(fill_only, fill_stroke_markers);
    assert_ne!(fill_stroke, fill_stroke_markers);
    assert_ne!(stroke_only, stroke_fill_markers);
    assert_ne!(markers_only, markers_fill_stroke);
}

#[test]
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssPaintOrderComponent::{Fill, Markers, Stroke};

    let result = qualify(
        86_402,
        concat!(
            "a{PAINT-ORDER:NORMAL;}",
            "b{paint-order:FILL STROKE;}",
            r"c{paint-order:\73 troke;}",
            r"d{paint-order:m\61 rkers;}",
            r"e{p\61 int-order:fill;}",
            "f{paint-order:/**/fill/**/stroke/**/!important;}",
            "g{paint-order:fill/**/markers;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Normal,
            ExpectedOutcome::Components(&[Fill, Stroke]),
            ExpectedOutcome::Components(&[Stroke]),
            ExpectedOutcome::Components(&[Markers]),
            ExpectedOutcome::Components(&[Fill]),
            ExpectedOutcome::Components(&[Fill, Stroke]),
            ExpectedOutcome::Components(&[Fill, Markers]),
        ],
    );

    let priority_observation = &result.paint_order_observations()[5];
    let occurrence =
        &result.upstream_parser_result().occurrences()[priority_observation.occurrence_index()];
    assert_eq!(priority_observation.placement(), occurrence.placement());
    assert!(occurrence.priority().is_some());
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value_and_never_synthesizes_normal() {
    let result = qualify(
        86_403,
        concat!(
            "a{paint-order:initial;}",
            "b{paint-order:inherit;}",
            "c{paint-order:unset;}",
            "d{paint-order:revert;}",
            "e{paint-order:revert-layer;}",
            "f{paint-order:revert-rule;}",
            "g{paint-order:inherit fill;}",
            "h{paint-order:fill inherit;}",
            "i{paint-order:initial stroke;}",
            "j{paint-order:initial revert;}",
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
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );

    // `initial` must never qualify as `Normal` merely because the property's
    // initial value is `normal`.
    for observation in result.paint_order_observations().iter().take(1) {
        assert_ne!(
            observation.outcome(),
            CssPaintOrderQualificationOutcome::Qualified(CssPaintOrderValue::Normal)
        );
    }
}

#[test]
fn deferred_and_whole_value_functions_follow_established_profile_while_ordinary_functions_stay_invalid()
 {
    let result = qualify(
        86_404,
        concat!(
            "a{paint-order:var(--po);}",
            "b{paint-order:fill var(--po);}",
            "c{paint-order:var(--po) fill;}",
            "d{paint-order:foo(var(--po));}",
            "e{paint-order:first-valid(fill,stroke);}",
            "f{paint-order:cycle(fill,stroke);}",
            "g{paint-order:interpolate(0%,0:fill,1:stroke);}",
            "h{paint-order:fill first-valid(stroke);}",
            "i{paint-order:first-valid(fill) stroke;}",
            "j{paint-order:foo();}",
            "k{paint-order:fill foo();}",
            "l{paint-order:foo() fill;}",
            "m{paint-order:calc(1);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::UnsupportedWholeValueFunction,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn property_name_dispatch_is_isolated_from_sibling_and_unrelated_properties() {
    use CssPaintOrderComponent::{Fill, Stroke};

    let result = qualify(
        86_405,
        concat!(
            "a{direction:ltr;}",
            "b{contain:layout style;}",
            "c{paint-order:fill stroke;}",
            "d{stroke-linecap:round;}",
            "e{flood-opacity:0.5;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(result.paint_order_observations().len(), 1);
    assert_eq!(result.stroke_linecap_observations().len(), 1);
    assert_eq!(result.flood_opacity_observations().len(), 1);
    assert_eq!(result.paint_order_observations()[0].occurrence_index(), 2);
    assert_expected(&result, &[ExpectedOutcome::Components(&[Fill, Stroke])]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    use CssPaintOrderComponent::Fill;

    let result = qualify(86_406, "a{paint-order:fill;}b{paint-order:fill;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Fill]),
            ExpectedOutcome::Components(&[Fill]),
        ],
    );
    assert_ne!(
        result.paint_order_observations()[0]
            .placement()
            .context_id(),
        result.paint_order_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (86_410, "@font-face{paint-order:fill;}"),
        (86_411, "@page{paint-order:fill;}"),
        (86_412, "@page{@top-left{paint-order:fill;}}"),
        (86_413, "@keyframes k{from{paint-order:fill;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.paint_order_observations().is_empty(),
            "nonordinary declaration context produced a paint-order observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssPaintOrderComponent::{Fill, Stroke};

    let result = qualify_with_limits(
        86_420,
        "a{paint-order:fill stroke;paint-order:stroke fill;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Components(&[Fill, Stroke])]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{paint-order:fill stroke markers;}",
        "b{paint-order:normal;}",
        "c{paint-order:inherit;}",
        "d{paint-order:var(--po);}",
        "e{contain:layout;}",
    );
    let first = qualify(86_430, css);
    let repeated = qualify(86_430, css);
    let another_source = qualify(86_431, css);

    assert_eq!(
        first.paint_order_observations(),
        repeated.paint_order_observations()
    );
    assert_eq!(
        first.paint_order_observations(),
        another_source.paint_order_observations()
    );
}
