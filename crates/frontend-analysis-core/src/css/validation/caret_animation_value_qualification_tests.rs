use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssCaretAnimationQualificationOutcome, CssCaretAnimationUnsupportedReason,
    CssCaretAnimationValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Manual,
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferred,
    UnsupportedWholeValue,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssCaretAnimationQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssCaretAnimationQualificationOutcome::Qualified(CssCaretAnimationValue::Auto)
        }
        ExpectedOutcome::Manual => {
            CssCaretAnimationQualificationOutcome::Qualified(CssCaretAnimationValue::Manual)
        }
        ExpectedOutcome::Invalid => {
            CssCaretAnimationQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssCaretAnimationQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCaretAnimationUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssCaretAnimationQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCaretAnimationUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssCaretAnimationQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCaretAnimationUnsupportedReason::WholeValueFunction,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .caret_animation_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

// A + B: both direct keywords qualify distinctly, and the two qualified
// variants are pairwise distinct results, not merely both "Qualified".
#[test]
fn both_direct_keywords_qualify_and_remain_pairwise_distinct() {
    let result = qualify(
        63600,
        concat!("a{caret-animation:auto;}", "b{caret-animation:manual;}"),
    );

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Manual]);

    let outcomes: Vec<_> = result
        .caret_animation_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_ne!(
        outcomes[0], outcomes[1],
        "authored caret-animation identities unexpectedly collapsed"
    );
}

// C: ASCII case-insensitive authored spelling recognition matches existing
// keyword identity semantics.
#[test]
fn case_insensitive_spelling_qualifies() {
    let result = qualify(
        63601,
        concat!(
            "a{caret-animation:AUTO;}",
            "b{caret-animation:Auto;}",
            "c{caret-animation:aUtO;}",
            "d{caret-animation:MANUAL;}",
            "e{caret-animation:Manual;}",
            "f{caret-animation:mAnUaL;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Manual,
            ExpectedOutcome::Manual,
            ExpectedOutcome::Manual,
        ],
    );
}

// C (escapes): tokenizer-decoded CSS escape forms for `auto`/`manual`, and an
// escaped property name, resolve through the same decoded Ident identity --
// no manual escape decoding or raw-source scanning is performed here.
#[test]
fn escaped_equivalents_qualify_through_decoded_ident_identity() {
    let result = qualify(
        63602,
        concat!(
            r"a{caret-animation:\61 uto;}",
            r"b{caret-animation:\6d anual;}",
            r"c{caret-anim\61 tion:manual;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Manual,
            ExpectedOutcome::Manual,
        ],
    );
}

// D: historical/proposal-only values remain permanent regression guards --
// `none` (the WG's initially-discussed concept), and `fade`/`blink`
// (mentioned only as possible convenience values) are all outside the
// current pinned grammar.
#[test]
fn historical_and_proposal_only_values_remain_invalid() {
    let result = qualify(
        63603,
        concat!(
            "a{caret-animation:none;}",
            "b{caret-animation:fade;}",
            "c{caret-animation:blink;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 3]);
}

// E: typo / unknown keyword identifiers remain ordinary Invalid.
#[test]
fn typo_and_unknown_keywords_are_invalid() {
    let result = qualify(
        63604,
        concat!(
            "a{caret-animation:manua;}",
            "b{caret-animation:automatic;}",
            "c{caret-animation:blinking;}",
            "d{caret-animation:foo;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

// F: this is a single-component grammar -- two or more direct keywords in
// one value are invalid, in either order, including exact repetition and
// three-token residual syntax. A valid prefix followed by residual syntax is
// never accepted.
#[test]
fn wrong_cardinality_is_invalid() {
    let result = qualify(
        63605,
        concat!(
            "a{caret-animation:auto manual;}",
            "b{caret-animation:manual auto;}",
            "c{caret-animation:auto auto;}",
            "d{caret-animation:manual manual;}",
            "e{caret-animation:manual auto manual;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

// G: wrong token classes (number, dimension, percentage, string, hash) are
// invalid -- none of these are a decoded Ident at all.
#[test]
fn wrong_token_classes_are_invalid() {
    let result = qualify(
        63606,
        concat!(
            "a{caret-animation:1;}",
            "b{caret-animation:1px;}",
            "c{caret-animation:50%;}",
            "d{caret-animation:\"auto\";}",
            "e{caret-animation:#fff;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

// H: caret-animation has no ordinary Function branch in the selected
// grammar -- an unrecognized Function is a direct grammar mismatch, not a
// generic Unsupported(FunctionValue) outcome.
#[test]
fn ordinary_functions_are_invalid_not_unsupported() {
    let result = qualify(
        63607,
        concat!("a{caret-animation:foo();}", "b{caret-animation:calc(1);}"),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 2]);
}

// J: recognized deferred/arbitrary substitution Functions preserve the
// existing shared Unsupported boundary, distinct from ordinary Invalid and
// from the whole-value Function boundary.
#[test]
fn deferred_substitution_functions_are_unsupported() {
    let result = qualify(
        63608,
        concat!(
            "a{caret-animation:var(--x);}",
            "b{caret-animation:env(foo);}",
            "c{caret-animation:attr(data-x);}",
            "d{caret-animation:if(x);}",
            "e{caret-animation:inherit(x);}",
            "f{caret-animation:ident(x);}",
            "g{caret-animation:random-item(x);}",
            "h{caret-animation:--custom();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 8]);
}

// K: the sole recognized whole-value-only Function is Unsupported only when
// it occupies the entire value; mixed placement alongside a direct keyword
// remains structurally Invalid rather than being softened into the
// whole-value outcome.
#[test]
fn whole_value_functions_are_unsupported_only_as_the_sole_value() {
    let result = qualify(
        63609,
        concat!(
            "a{caret-animation:first-valid(auto,manual);}",
            "b{caret-animation:cycle(auto,manual);}",
            "c{caret-animation:interpolate(50%,0:auto,1:manual);}",
            "d{caret-animation:auto first-valid(manual);}",
            "e{caret-animation:first-valid(manual) auto;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

// H (embedded): an ordinary residual Function embedded alongside a direct
// keyword is structurally invalid, not softened into any Unsupported
// outcome.
#[test]
fn embedded_ordinary_functions_are_structurally_invalid() {
    let result = qualify(
        63610,
        concat!(
            "a{caret-animation:auto foo();}",
            "b{caret-animation:foo() manual;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 2]);
}

// I: sole CSS-wide keywords use the existing CSS-wide Unsupported outcome.
// Explicit discriminator: `initial` is CSS-wide authored identity and must
// never collapse into direct authored `auto`, even though `auto` is the
// property's initial value.
#[test]
fn css_wide_keywords_are_unsupported_and_initial_never_collapses_into_auto() {
    let result = qualify(
        63611,
        concat!(
            "a{caret-animation:initial;}",
            "b{caret-animation:inherit;}",
            "c{caret-animation:unset;}",
            "d{caret-animation:revert;}",
            "e{caret-animation:revert-layer;}",
            "f{caret-animation:revert-rule;}",
            "g{caret-animation:auto initial;}",
            "h{caret-animation:initial manual;}",
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

    let initial_outcome = result.caret_animation_observations()[0].outcome();
    assert_ne!(
        initial_outcome,
        CssCaretAnimationQualificationOutcome::Qualified(CssCaretAnimationValue::Auto),
        "authored `initial` must not collapse into direct authored `auto`"
    );
}

// L: comments/whitespace surrounding a valid keyword preserve the same
// qualified identity; an empty or trivia-only declaration value is invalid.
#[test]
fn trivia_and_comments_preserve_identity_and_empty_value_is_invalid() {
    let result = qualify(
        63612,
        concat!(
            "a{caret-animation:/**/auto/**/;}",
            "b{caret-animation: manual ;}",
            "c{caret-animation:;}",
            "d{caret-animation:/**/;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Manual,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

// L: `!important` qualifies the authored value according to the existing
// declaration priority boundary -- priority does not change the outcome.
#[test]
fn important_priority_does_not_change_qualification() {
    let result = qualify(63613, "a{caret-animation:auto !important;}");

    assert_expected(&result, &[ExpectedOutcome::Auto]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// Q: no declaration produces no synthetic `Auto` observation, even though
// `auto` is the property's initial value -- the qualifier observes only
// authored declarations.
#[test]
fn no_declaration_produces_no_synthetic_auto_observation() {
    let result = qualify(63614, "a{color:red;}");

    assert!(
        result.caret_animation_observations().is_empty(),
        "absent caret-animation declaration unexpectedly produced an observation"
    );
}

// O: multiple `caret-animation` declarations remain independent run-local
// observations with distinct occurrence index and placement -- no cascade
// or property deduplication is performed here, and distinct occurrences
// remain distinct even when their values match.
#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(63615, "a{caret-animation:auto;}b{caret-animation:auto;}");

    assert_expected(&result, &[ExpectedOutcome::Auto, ExpectedOutcome::Auto]);
    assert_eq!(
        result.caret_animation_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.caret_animation_observations()[1].occurrence_index(),
        1
    );
    assert_ne!(
        result.caret_animation_observations()[0]
            .placement()
            .context_id(),
        result.caret_animation_observations()[1]
            .placement()
            .context_id(),
    );
}

// P: only ordinary declaration placements enter this qualifier --
// nonordinary declaration-shaped contexts produce no observation.
#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (63620, "@font-face{caret-animation:manual;}"),
        (63621, "@page{caret-animation:manual;}"),
        (63622, "@page{@top-left{caret-animation:manual;}}"),
        (63623, "@keyframes k{from{caret-animation:manual;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.caret_animation_observations().is_empty(),
            "nonordinary declaration context produced a caret-animation observation for {css:?}"
        );
    }
}

// Q: upstream parser resource stop preserves only the committed retained
// prefix; the qualifier never upgrades parser completion or retries/reparses.
#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        63630,
        "a{caret-animation:auto;caret-animation:manual;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Auto]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

// R: repeated qualification of equivalent source, and across distinct
// sources, is semantically deterministic.
#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{caret-animation:auto;}",
        "b{caret-animation:inherit;}",
        "c{caret-animation:manual;}",
        "d{caret-animation:var(--x);}",
        "e{caret-color:red;}",
    );
    let first = qualify(63640, css);
    let repeated = qualify(63640, css);
    let another_source = qualify(63641, css);

    assert_eq!(
        first.caret_animation_observations(),
        repeated.caret_animation_observations()
    );
    assert_eq!(
        first.caret_animation_observations(),
        another_source.caret_animation_observations()
    );
}

// N: dispatch isolation -- neighboring/unrelated properties, including the
// similarly-named `caret-color` and `caret-shape` (neither implemented as a
// selected leaf) and other accepted keyword leaves, never produce a
// caret-animation observation. Only the actual `caret-animation`
// declaration is observed, at its own run-local occurrence index.
#[test]
fn dispatch_isolation_excludes_neighboring_and_unrelated_properties() {
    let result = qualify(
        63650,
        concat!(
            "a{caret-color:red;}",
            "b{caret-shape:block;}",
            "c{direction:ltr;}",
            "d{paint-order:fill;}",
            "e{unrelated-property:auto;}",
            "f{caret-animation:manual;}",
        ),
    );

    assert_eq!(result.caret_animation_observations().len(), 1);
    assert_eq!(
        result.caret_animation_observations()[0].occurrence_index(),
        5
    );
    assert_expected(&result, &[ExpectedOutcome::Manual]);
}
