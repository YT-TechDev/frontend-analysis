use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssCaretShapeQualificationOutcome, CssCaretShapeUnsupportedReason, CssCaretShapeValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    Bar,
    Block,
    Underscore,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssCaretShapeQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Auto)
        }
        ExpectedOutcome::Bar => {
            CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Bar)
        }
        ExpectedOutcome::Block => {
            CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Block)
        }
        ExpectedOutcome::Underscore => {
            CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Underscore)
        }
        ExpectedOutcome::Invalid => {
            CssCaretShapeQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssCaretShapeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCaretShapeUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssCaretShapeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCaretShapeUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssCaretShapeQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssCaretShapeUnsupportedReason::WholeValueFunction,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .caret_shape_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

// A: all four direct keywords qualify distinctly, and the four qualified
// variants are pairwise distinct results, not merely all "Qualified".
#[test]
fn all_direct_keywords_qualify_and_remain_pairwise_distinct() {
    let result = qualify(
        63800,
        concat!(
            "a{caret-shape:auto;}",
            "b{caret-shape:bar;}",
            "c{caret-shape:block;}",
            "d{caret-shape:underscore;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Bar,
            ExpectedOutcome::Block,
            ExpectedOutcome::Underscore,
        ],
    );

    let outcomes: Vec<_> = result
        .caret_shape_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    for i in 0..outcomes.len() {
        for j in (i + 1)..outcomes.len() {
            assert_ne!(
                outcomes[i], outcomes[j],
                "authored caret-shape identities unexpectedly collapsed"
            );
        }
    }
}

// B: ASCII case-insensitive authored spelling recognition matches existing
// keyword identity semantics.
#[test]
fn case_insensitive_spelling_qualifies() {
    let result = qualify(
        63801,
        concat!(
            "a{caret-shape:AUTO;}",
            "b{caret-shape:Auto;}",
            "c{caret-shape:aUtO;}",
            "d{caret-shape:BAR;}",
            "e{caret-shape:Bar;}",
            "f{caret-shape:bAr;}",
            "g{caret-shape:BLOCK;}",
            "h{caret-shape:Block;}",
            "i{caret-shape:bLoCk;}",
            "j{caret-shape:UNDERSCORE;}",
            "k{caret-shape:Underscore;}",
            "l{caret-shape:uNdErScOrE;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Bar,
            ExpectedOutcome::Bar,
            ExpectedOutcome::Bar,
            ExpectedOutcome::Block,
            ExpectedOutcome::Block,
            ExpectedOutcome::Block,
            ExpectedOutcome::Underscore,
            ExpectedOutcome::Underscore,
            ExpectedOutcome::Underscore,
        ],
    );
}

// C (escapes): tokenizer-decoded CSS escape forms for each direct keyword,
// and an escaped property name, resolve through the same decoded Ident
// identity -- no manual escape decoding or raw-source scanning is performed
// here.
#[test]
fn escaped_equivalents_qualify_through_decoded_ident_identity() {
    let result = qualify(
        63802,
        concat!(
            r"a{caret-shape:\61 uto;}",
            r"b{caret-shape:\62 ar;}",
            r"c{caret-shape:\62 lock;}",
            r"d{caret-shape:\75 nderscore;}",
            r"e{caret-sh\61 pe:block;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Bar,
            ExpectedOutcome::Block,
            ExpectedOutcome::Underscore,
            ExpectedOutcome::Block,
        ],
    );
}

// D: aliases and historical-looking near-misses are not authored `caret-shape`
// identities and remain permanent regression guards -- `underline` != authored
// `underscore`, `vertical` != authored `bar`, `rect` != authored `block`.
#[test]
fn alias_and_near_miss_values_remain_invalid() {
    let result = qualify(
        63803,
        concat!(
            "a{caret-shape:none;}",
            "b{caret-shape:ba;}",
            "c{caret-shape:underline;}",
            "d{caret-shape:vertical;}",
            "e{caret-shape:rect;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

// E: unknown/typo keyword identifiers remain ordinary Invalid.
#[test]
fn unknown_and_typo_keywords_are_invalid() {
    let result = qualify(
        63804,
        concat!(
            "a{caret-shape:automatic;}",
            "b{caret-shape:box;}",
            "c{caret-shape:line;}",
            "d{caret-shape:underbar;}",
            "e{caret-shape:caret;}",
            "f{caret-shape:foo;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 6]);
}

// F: this is a single-component grammar -- two or more direct keywords in
// one value are invalid in any order, including exact repetition and
// three-token residual syntax. A valid prefix followed by residual syntax is
// never accepted.
#[test]
fn wrong_cardinality_is_invalid() {
    let result = qualify(
        63805,
        concat!(
            "a{caret-shape:auto bar;}",
            "b{caret-shape:bar block;}",
            "c{caret-shape:block underscore;}",
            "d{caret-shape:underscore auto;}",
            "e{caret-shape:auto auto;}",
            "f{caret-shape:bar bar;}",
            "g{caret-shape:block block;}",
            "h{caret-shape:underscore underscore;}",
            "i{caret-shape:underscore auto bar;}",
            "j{caret-shape:block auto block;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 10]);
}

// G: wrong token classes (number, dimension, percentage, string, hash) are
// invalid -- none of these are a decoded Ident at all.
#[test]
fn wrong_token_classes_are_invalid() {
    let result = qualify(
        63806,
        concat!(
            "a{caret-shape:1;}",
            "b{caret-shape:1px;}",
            "c{caret-shape:50%;}",
            "d{caret-shape:\"bar\";}",
            "e{caret-shape:#fff;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

// H: caret-shape has no ordinary Function branch in the selected grammar --
// an unrecognized Function is a direct grammar mismatch, not a generic
// Unsupported(FunctionValue) outcome.
#[test]
fn ordinary_functions_are_invalid_not_unsupported() {
    let result = qualify(
        63807,
        concat!("a{caret-shape:foo();}", "b{caret-shape:calc(1);}"),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 2]);
}

// H (embedded): an ordinary residual Function embedded alongside a direct
// keyword is structurally invalid, not softened into any Unsupported
// outcome.
#[test]
fn embedded_ordinary_functions_are_structurally_invalid() {
    let result = qualify(
        63808,
        concat!("a{caret-shape:auto foo();}", "b{caret-shape:foo() block;}"),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 2]);
}

// I: recognized deferred/arbitrary substitution Functions preserve the
// existing shared Unsupported boundary, distinct from ordinary Invalid and
// from the whole-value Function boundary.
#[test]
fn deferred_substitution_functions_are_unsupported() {
    let result = qualify(
        63809,
        concat!(
            "a{caret-shape:var(--x);}",
            "b{caret-shape:env(foo);}",
            "c{caret-shape:attr(data-x);}",
            "d{caret-shape:if(x);}",
            "e{caret-shape:inherit(x);}",
            "f{caret-shape:ident(x);}",
            "g{caret-shape:random-item(x);}",
            "h{caret-shape:--custom();}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedDeferred; 8]);
}

// J: the sole recognized whole-value-only Function is Unsupported only when
// it occupies the entire value; mixed placement alongside a direct keyword
// remains structurally Invalid rather than being softened into the
// whole-value outcome.
#[test]
fn whole_value_functions_are_unsupported_only_as_the_sole_value() {
    let result = qualify(
        63810,
        concat!(
            "a{caret-shape:first-valid(bar,block);}",
            "b{caret-shape:cycle(bar,block);}",
            "c{caret-shape:interpolate(50%,0:auto,1:block);}",
            "d{caret-shape:auto first-valid(bar,block);}",
            "e{caret-shape:first-valid(auto,block) bar;}",
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

// K: sole CSS-wide keywords use the existing CSS-wide Unsupported outcome.
// Explicit discriminator: `initial` is CSS-wide authored identity and must
// never collapse into direct authored `auto`, even though `auto` is the
// property's initial value.
#[test]
fn css_wide_keywords_are_unsupported_and_initial_never_collapses_into_auto() {
    let result = qualify(
        63811,
        concat!(
            "a{caret-shape:initial;}",
            "b{caret-shape:inherit;}",
            "c{caret-shape:unset;}",
            "d{caret-shape:revert;}",
            "e{caret-shape:revert-layer;}",
            "f{caret-shape:revert-rule;}",
            "g{caret-shape:auto initial;}",
            "h{caret-shape:initial block;}",
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

    let initial_outcome = result.caret_shape_observations()[0].outcome();
    assert_ne!(
        initial_outcome,
        CssCaretShapeQualificationOutcome::Qualified(CssCaretShapeValue::Auto),
        "authored `initial` must not collapse into direct authored `auto`"
    );
}

// L: comments/whitespace surrounding a valid keyword preserve the same
// qualified identity; an empty or trivia-only declaration value is invalid.
#[test]
fn trivia_and_comments_preserve_identity_and_empty_value_is_invalid() {
    let result = qualify(
        63812,
        concat!(
            "a{caret-shape:/**/auto/**/;}",
            "b{caret-shape: block ;}",
            "c{caret-shape:;}",
            "d{caret-shape:/**/;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Block,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

// L: `!important` qualifies the authored value according to the existing
// declaration priority boundary -- priority does not change the outcome.
#[test]
fn important_priority_does_not_change_qualification() {
    let result = qualify(63813, "a{caret-shape:underscore !important;}");

    assert_expected(&result, &[ExpectedOutcome::Underscore]);
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
    let result = qualify(63814, "a{color:red;}");

    assert!(
        result.caret_shape_observations().is_empty(),
        "absent caret-shape declaration unexpectedly produced an observation"
    );
}

// O: multiple `caret-shape` declarations remain independent run-local
// observations with distinct occurrence index and placement -- no cascade or
// property deduplication is performed here, and distinct occurrences remain
// distinct even when their values match.
#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(63815, "a{caret-shape:block;}b{caret-shape:block;}");

    assert_expected(&result, &[ExpectedOutcome::Block, ExpectedOutcome::Block]);
    assert_eq!(result.caret_shape_observations()[0].occurrence_index(), 0);
    assert_eq!(result.caret_shape_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.caret_shape_observations()[0]
            .placement()
            .context_id(),
        result.caret_shape_observations()[1]
            .placement()
            .context_id(),
    );
}

// P: only ordinary declaration placements enter this qualifier --
// nonordinary declaration-shaped contexts produce no observation.
#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (63820, "@font-face{caret-shape:block;}"),
        (63821, "@page{caret-shape:block;}"),
        (63822, "@page{@top-left{caret-shape:block;}}"),
        (63823, "@keyframes k{from{caret-shape:block;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.caret_shape_observations().is_empty(),
            "nonordinary declaration context produced a caret-shape observation for {css:?}"
        );
    }
}

// Q: upstream parser resource stop preserves only the committed retained
// prefix; the qualifier never upgrades parser completion or retries/reparses.
#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        63830,
        "a{caret-shape:auto;caret-shape:block;}",
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
        "a{caret-shape:auto;}",
        "b{caret-shape:inherit;}",
        "c{caret-shape:bar;}",
        "d{caret-shape:var(--x);}",
        "e{caret-color:red;}",
    );
    let first = qualify(63840, css);
    let repeated = qualify(63840, css);
    let another_source = qualify(63841, css);

    assert_eq!(
        first.caret_shape_observations(),
        repeated.caret_shape_observations()
    );
    assert_eq!(
        first.caret_shape_observations(),
        another_source.caret_shape_observations()
    );
}

// N: dispatch isolation -- neighboring/unrelated properties, including the
// similarly-named `caret-color` and `caret-animation` and other accepted
// keyword leaves, never produce a caret-shape observation. Only the actual
// `caret-shape` declaration is observed, at its own run-local occurrence
// index.
#[test]
fn dispatch_isolation_excludes_neighboring_and_unrelated_properties() {
    let result = qualify(
        63850,
        concat!(
            "a{caret-color:red;}",
            "b{caret-animation:manual;}",
            "c{direction:ltr;}",
            "d{paint-order:fill;}",
            "e{unrelated-property:auto;}",
            "f{caret-shape:underscore;}",
        ),
    );

    assert_eq!(result.caret_shape_observations().len(), 1);
    assert_eq!(result.caret_shape_observations()[0].occurrence_index(), 5);
    assert_expected(&result, &[ExpectedOutcome::Underscore]);
}
