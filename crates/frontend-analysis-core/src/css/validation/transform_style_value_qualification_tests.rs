use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTransformStyleQualificationOutcome, CssTransformStyleUnsupportedReason,
    CssTransformStyleValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Flat,
    Preserve3d,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssTransformStyleQualificationOutcome {
    match expected {
        ExpectedOutcome::Flat => {
            CssTransformStyleQualificationOutcome::Qualified(CssTransformStyleValue::Flat)
        }
        ExpectedOutcome::Preserve3d => {
            CssTransformStyleQualificationOutcome::Qualified(CssTransformStyleValue::Preserve3d)
        }
        ExpectedOutcome::Invalid => {
            CssTransformStyleQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssTransformStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformStyleUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssTransformStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformStyleUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssTransformStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformStyleUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssTransformStyleQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformStyleUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .transform_style_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

// A + C + D: both direct keywords qualify distinctly, and ASCII
// case-insensitive / escape-equivalent authored spelling recognition matches
// existing keyword identity semantics (also covering an escaped property
// name).
#[test]
fn both_direct_keywords_qualify_case_insensitively() {
    let result = qualify(
        6200,
        concat!(
            "a{transform-style:flat;}",
            "b{transform-style:preserve-3d;}",
            "c{transform-style:FLAT;}",
            "d{transform-style:PRESERVE-3D;}",
            "e{transform-style:Flat;}",
            "f{transform-style:Preserve-3D;}",
            r"g{transform-style:\66 lat;}",
            r"h{transform-style:\70 reserve-3d;}",
            r"i{transform-st\79 le:preserve-3d;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Flat,
            ExpectedOutcome::Preserve3d,
            ExpectedOutcome::Flat,
            ExpectedOutcome::Preserve3d,
            ExpectedOutcome::Flat,
            ExpectedOutcome::Preserve3d,
            ExpectedOutcome::Flat,
            ExpectedOutcome::Preserve3d,
            ExpectedOutcome::Preserve3d,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

// B: authored identity distinction -- the two qualified variants are
// pairwise distinct results, not merely both "Qualified".
#[test]
fn qualified_authored_identities_remain_pairwise_distinct() {
    let result = qualify(
        6201,
        concat!(
            "a{transform-style:flat;}",
            "b{transform-style:preserve-3d;}",
        ),
    );

    let outcomes: Vec<_> = result
        .transform_style_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(outcomes.len(), 2);
    assert_ne!(
        outcomes[0], outcomes[1],
        "authored transform-style identities unexpectedly collapsed"
    );
    assert_eq!(
        outcomes[0],
        CssTransformStyleQualificationOutcome::Qualified(CssTransformStyleValue::Flat)
    );
    assert_eq!(
        outcomes[1],
        CssTransformStyleQualificationOutcome::Qualified(CssTransformStyleValue::Preserve3d)
    );
}

// E + F: the historical `auto` keyword (pre-rewrite grammar) and the
// proposed `detached` keyword (CSSWG #4242, not part of the pinned grammar)
// both remain ordinary Invalid identifiers -- neither is accepted by this
// leaf.
#[test]
fn historical_auto_and_proposed_detached_are_invalid() {
    let result = qualify(
        6202,
        concat!("a{transform-style:auto;}", "b{transform-style:detached;}",),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Invalid, ExpectedOutcome::Invalid],
    );
}

// G: adjacent/unrelated keywords and an arbitrary wrong identifier remain
// ordinary Invalid identifiers.
#[test]
fn adjacent_and_arbitrary_wrong_identifiers_are_invalid() {
    let result = qualify(
        6203,
        concat!(
            "a{transform-style:visible;}",
            "b{transform-style:hidden;}",
            "c{transform-style:none;}",
            "d{transform-style:foo;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

// H: this is a single-component grammar -- two or more direct keywords in
// one value are invalid, in either order and including exact repetition.
#[test]
fn multiple_direct_keywords_are_invalid() {
    let result = qualify(
        6204,
        concat!(
            "a{transform-style:flat preserve-3d;}",
            "b{transform-style:preserve-3d flat;}",
            "c{transform-style:flat flat;}",
            "d{transform-style:preserve-3d preserve-3d;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

// I: top-level comma-separated forms are invalid -- this never becomes a
// list-valued property.
#[test]
fn top_level_comma_forms_are_invalid() {
    let result = qualify(
        6205,
        concat!(
            "a{transform-style:flat, preserve-3d;}",
            "b{transform-style:flat,;}",
            "c{transform-style:,preserve-3d;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 3]);
}

// L + M + N: ordinary residual Function, recognized deferred/arbitrary
// substitution Function, and the sole recognized whole-value-only Function
// each preserve their own existing shared Unsupported boundary rather than
// being conflated with one another or with ordinary Invalid.
#[test]
fn function_boundaries_remain_distinct_and_fail_open() {
    let result = qualify(
        6206,
        concat!(
            "a{transform-style:foo();}",
            "b{transform-style:calc(1);}",
            "c{transform-style:var(--x);}",
            "d{transform-style:env(foo);}",
            "e{transform-style:attr(data-x);}",
            "f{transform-style:--custom();}",
            "g{transform-style:first-valid(flat,preserve-3d);}",
            "h{transform-style:cycle(flat,preserve-3d);}",
            "i{transform-style:interpolate(50%,0:flat,1:preserve-3d);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
        ],
    );
}

// M (embedded) + structural-invalidity-with-Function: a whole-value-only
// Function embedded alongside a direct keyword, or an ordinary residual
// Function embedded alongside a direct keyword, is structurally invalid, not
// softened into a generic FunctionValue/whole-value outcome.
#[test]
fn embedded_functions_are_structurally_invalid() {
    let result = qualify(
        6207,
        concat!(
            "a{transform-style:flat first-valid(preserve-3d);}",
            "b{transform-style:first-valid(preserve-3d) flat;}",
            "c{transform-style:flat foo();}",
            "d{transform-style:foo() preserve-3d;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

// J + K: sole CSS-wide keywords use the existing CSS-wide Unsupported
// outcome; embedded CSS-wide syntax is structurally invalid rather than
// whole-property CSS-wide handling. `initial` here is direct authored
// evidence for the whole-value CSS-wide keyword, not the same authored
// evidence as a direct `flat` keyword.
#[test]
fn css_wide_keywords_are_unsupported_only_as_the_whole_value() {
    let result = qualify(
        6208,
        concat!(
            "a{transform-style:initial;}",
            "b{transform-style:inherit;}",
            "c{transform-style:unset;}",
            "d{transform-style:revert;}",
            "e{transform-style:revert-layer;}",
            "f{transform-style:revert-rule;}",
            "g{transform-style:flat initial;}",
            "h{transform-style:initial preserve-3d;}",
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

    let initial_outcome = result.transform_style_observations()[0].outcome();
    assert_ne!(
        initial_outcome,
        CssTransformStyleQualificationOutcome::Qualified(CssTransformStyleValue::Flat),
        "authored `initial` must not collapse into direct authored `flat`"
    );
}

// Q: authored absence produces no synthetic `Flat` observation, even though
// `flat` is the property's initial value -- the qualifier observes only
// authored declarations.
#[test]
fn no_declaration_produces_no_synthetic_flat_observation() {
    let result = qualify(6209, "a{color:red;}");

    assert!(
        result.transform_style_observations().is_empty(),
        "absent transform-style declaration unexpectedly produced an observation"
    );
}

// P: load-bearing candidate-independent regression -- authored
// classification never depends on neighboring declarations such as
// `overflow: hidden` or `opacity: .5`, which are exactly the kind of
// grouping-property state CSS Transforms 2 uses only for the downstream used
// value. `Preserve3d` remains the authored result regardless of declaration
// order relative to such properties.
#[test]
fn grouping_property_declarations_do_not_influence_authored_classification() {
    let result = qualify(
        6210,
        concat!(
            "a{transform-style:preserve-3d;overflow:hidden;}",
            "b{opacity:.5;transform-style:preserve-3d;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Preserve3d, ExpectedOutcome::Preserve3d],
    );
}

// R: trivia/comments surrounding a valid keyword retain the same qualified
// identity; an empty or trivia-only declaration value is invalid.
#[test]
fn trivia_and_comments_preserve_identity_and_empty_value_is_invalid() {
    let result = qualify(
        6211,
        concat!(
            "a{transform-style:/**/flat/**/;}",
            "b{transform-style: preserve-3d ;}",
            "c{transform-style:;}",
            "d{transform-style:/**/;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Flat,
            ExpectedOutcome::Preserve3d,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

// S: `!important` qualifies the authored value according to the existing
// declaration priority boundary -- priority does not change the outcome.
#[test]
fn important_priority_does_not_change_qualification() {
    let result = qualify(6212, "a{transform-style:preserve-3d !important;}");

    assert_expected(&result, &[ExpectedOutcome::Preserve3d]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// T: multiple `transform-style` declarations remain independent run-local
// observations -- no cascade or property deduplication is performed here.
#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(
        6213,
        "a{transform-style:flat;}b{transform-style:preserve-3d;}",
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Flat, ExpectedOutcome::Preserve3d],
    );
    assert_eq!(
        result.transform_style_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.transform_style_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.transform_style_observations()[0]
            .placement()
            .context_id(),
        result.transform_style_observations()[1]
            .placement()
            .context_id(),
    );
}

// U: only ordinary declaration placements enter this qualifier --
// nonordinary declaration-shaped contexts produce no observation.
#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (6220, "@font-face{transform-style:preserve-3d;}"),
        (6221, "@page{transform-style:preserve-3d;}"),
        (6222, "@page{@top-left{transform-style:preserve-3d;}}"),
        (6223, "@keyframes k{from{transform-style:preserve-3d;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.transform_style_observations().is_empty(),
            "nonordinary declaration context produced a transform-style observation for {css:?}"
        );
    }
}

// V: upstream parser resource stop preserves only the committed retained
// prefix; the qualifier never upgrades parser completion.
#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        6230,
        "a{transform-style:flat;transform-style:preserve-3d;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Flat]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

// W: repeated qualification of equivalent source, and across distinct
// sources, is semantically deterministic.
#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{transform-style:flat;}",
        "b{transform-style:inherit;}",
        "c{transform-style:preserve-3d;}",
        "d{transform-style:var(--style);}",
        "e{transform-box:border-box;}",
    );
    let first = qualify(6240, css);
    let repeated = qualify(6240, css);
    let another_source = qualify(6241, css);

    assert_eq!(
        first.transform_style_observations(),
        repeated.transform_style_observations()
    );
    assert_eq!(
        first.transform_style_observations(),
        another_source.transform_style_observations()
    );
}

// X: cross-leaf isolation -- one run interleaving `transform-style` with
// `transform-box` and the other accepted transform leaves produces exactly
// one observation per property, with no cross-dispatch between leaves.
#[test]
fn one_run_interleaves_with_neighboring_accepted_leaves_without_cross_dispatch() {
    let result = qualify(
        6250,
        concat!(
            "a{box-sizing:border-box;}",
            "b{transform-origin:left top;}",
            "c{translate:10px;}",
            "d{rotate:45deg;}",
            "e{scale:2;}",
            "f{transform-box:fill-box;}",
            "g{transform-style:preserve-3d;}",
        ),
    );

    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_style_observations().len(), 1);
    assert_eq!(
        result.transform_style_observations()[0].occurrence_index(),
        6
    );
    assert_expected(&result, &[ExpectedOutcome::Preserve3d]);
}
