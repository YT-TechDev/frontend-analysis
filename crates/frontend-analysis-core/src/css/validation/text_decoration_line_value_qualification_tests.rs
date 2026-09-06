use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTextDecorationLineComponent, CssTextDecorationLineQualificationOutcome,
    CssTextDecorationLineUnsupportedReason, CssTextDecorationLineValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    None,
    SpellingError,
    GrammarError,
    Components(&'static [CssTextDecorationLineComponent]),
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
    let actual = result.text_decoration_line_observations();
    assert_eq!(actual.len(), expected.len());

    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssTextDecorationLineQualificationOutcome::Qualified(
                    CssTextDecorationLineValue::None,
                ),
                ExpectedOutcome::None,
            ) => {}
            (
                CssTextDecorationLineQualificationOutcome::Qualified(
                    CssTextDecorationLineValue::SpellingError,
                ),
                ExpectedOutcome::SpellingError,
            ) => {}
            (
                CssTextDecorationLineQualificationOutcome::Qualified(
                    CssTextDecorationLineValue::GrammarError,
                ),
                ExpectedOutcome::GrammarError,
            ) => {}
            (
                CssTextDecorationLineQualificationOutcome::Qualified(
                    CssTextDecorationLineValue::Components(components),
                ),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssTextDecorationLineQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssTextDecorationLineQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextDecorationLineUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssTextDecorationLineQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextDecorationLineUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssTextDecorationLineQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssTextDecorationLineUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => {
                panic!("unexpected text-decoration-line outcome: {actual:?}, expected {expected:?}")
            }
        }
    }
}

#[test]
fn handwritten_boundary_matches_pinned_wpt_and_derived_theorem() {
    use CssTextDecorationLineComponent::{Blink, LineThrough, Overline, Underline};

    let result = qualify(
        85_700,
        concat!(
            "a{text-decoration-line:none;}",
            "b{text-decoration-line:spelling-error;}",
            "c{text-decoration-line:grammar-error;}",
            "d{text-decoration-line:underline;}",
            "e{text-decoration-line:overline;}",
            "f{text-decoration-line:line-through;}",
            "g{text-decoration-line:blink;}",
            "h{text-decoration-line:underline overline;}",
            "i{text-decoration-line:overline underline;}",
            "j{text-decoration-line:blink line-through underline overline;}",
            "k{text-decoration-line:underline underline;}",
            "l{text-decoration-line:blink line-through blink;}",
            "m{text-decoration-line:none underline;}",
            "n{text-decoration-line:spelling-error overline;}",
            "o{text-decoration-line:spelling-error grammar-error;}",
            "p{text-decoration-line:grammar-error blink;}",
            "q{text-decoration-line:auto;}",
            "r{text-decoration-line:0;}",
            "s{text-decoration-line:\"underline\";}",
            "t{text-decoration-line:;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::None,
            ExpectedOutcome::SpellingError,
            ExpectedOutcome::GrammarError,
            ExpectedOutcome::Components(&[Underline]),
            ExpectedOutcome::Components(&[Overline]),
            ExpectedOutcome::Components(&[LineThrough]),
            ExpectedOutcome::Components(&[Blink]),
            ExpectedOutcome::Components(&[Underline, Overline]),
            ExpectedOutcome::Components(&[Overline, Underline]),
            ExpectedOutcome::Components(&[Blink, LineThrough, Underline, Overline]),
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
fn authored_order_is_preserved_without_cssom_canonicalization() {
    use CssTextDecorationLineComponent::{Blink, LineThrough, Overline, Underline};

    let result = qualify(
        85_701,
        concat!(
            "a{text-decoration-line:underline overline;}",
            "b{text-decoration-line:overline underline;}",
            "c{text-decoration-line:blink line-through underline;}",
            "d{text-decoration-line:underline line-through blink;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Underline, Overline]),
            ExpectedOutcome::Components(&[Overline, Underline]),
            ExpectedOutcome::Components(&[Blink, LineThrough, Underline]),
            ExpectedOutcome::Components(&[Underline, LineThrough, Blink]),
        ],
    );
}

#[test]
fn all_composite_slots_reject_duplicate_claims() {
    let result = qualify(
        85_702,
        concat!(
            "a{text-decoration-line:underline underline;}",
            "b{text-decoration-line:overline overline;}",
            "c{text-decoration-line:line-through line-through;}",
            "d{text-decoration-line:blink blink;}",
            "e{text-decoration-line:underline overline underline;}",
            "f{text-decoration-line:overline blink line-through blink;}",
        ),
    );

    assert_expected(
        &result,
        &[
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
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssTextDecorationLineComponent::{Blink, Overline, Underline};

    let result = qualify(
        85_703,
        concat!(
            "a{TEXT-DECORATION-LINE:UNDERLINE OVERLINE;}",
            r"b{text-decoration-line:\75 nderline;}",
            r"c{text-decoration-li\6e e:blink;}",
            "d{text-decoration-line:/**/blink/**/overline/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Underline, Overline]),
            ExpectedOutcome::Components(&[Underline]),
            ExpectedOutcome::Components(&[Blink]),
            ExpectedOutcome::Components(&[Blink, Overline]),
        ],
    );

    let priority_observation = &result.text_decoration_line_observations()[3];
    let occurrence =
        &result.upstream_parser_result().occurrences()[priority_observation.occurrence_index()];
    assert_eq!(priority_observation.placement(), occurrence.placement());
    assert!(occurrence.priority().is_some());
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        85_704,
        concat!(
            "a{text-decoration-line:initial;}",
            "b{text-decoration-line:inherit;}",
            "c{text-decoration-line:unset;}",
            "d{text-decoration-line:revert;}",
            "e{text-decoration-line:revert-layer;}",
            "f{text-decoration-line:revert-rule;}",
            "g{text-decoration-line:inherit underline;}",
            "h{text-decoration-line:underline inherit;}",
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
}

#[test]
fn deferred_functions_fail_open_anywhere_while_other_functions_keep_existing_boundaries() {
    let result = qualify(
        85_705,
        concat!(
            "a{text-decoration-line:var(--line);}",
            "b{text-decoration-line:underline var(--line);}",
            "c{text-decoration-line:var(--line) overline;}",
            "d{text-decoration-line:foo(var(--line));}",
            "e{text-decoration-line:first-valid(underline,overline);}",
            "f{text-decoration-line:cycle(underline,overline);}",
            "g{text-decoration-line:interpolate(0%,0:underline,1:overline);}",
            "h{text-decoration-line:underline first-valid(overline);}",
            "i{text-decoration-line:foo();}",
            "j{text-decoration-line:underline foo();}",
            "k{text-decoration-line:calc(1);}",
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
        ],
    );
}

#[test]
fn applicability_and_cross_dispatch_remain_isolated() {
    use CssTextDecorationLineComponent::{Overline, Underline};

    let result = qualify(
        85_706,
        concat!(
            "a{font-variant-numeric:oldstyle-nums;}",
            "b{contain:layout size;}",
            "table{text-decoration-line:overline underline;}",
            "d{text-decoration-style:wavy;}",
        ),
    );

    assert_eq!(result.font_variant_numeric_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(result.text_decoration_line_observations().len(), 1);
    assert_eq!(result.text_decoration_style_observations().len(), 1);
    assert_eq!(
        result.text_decoration_line_observations()[0].occurrence_index(),
        2
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[Overline, Underline])],
    );
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    use CssTextDecorationLineComponent::Underline;

    let result = qualify(
        85_707,
        "a{text-decoration-line:underline;}b{text-decoration-line:underline;}",
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Underline]),
            ExpectedOutcome::Components(&[Underline]),
        ],
    );
    assert_ne!(
        result.text_decoration_line_observations()[0]
            .placement()
            .context_id(),
        result.text_decoration_line_observations()[1]
            .placement()
            .context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (85_710, "@font-face{text-decoration-line:underline;}"),
        (85_711, "@page{text-decoration-line:underline;}"),
        (85_712, "@page{@top-left{text-decoration-line:underline;}}"),
        (
            85_713,
            "@keyframes k{from{text-decoration-line:underline;}}",
        ),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.text_decoration_line_observations().is_empty(),
            "unexpected observation for {css}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssTextDecorationLineComponent::{Overline, Underline};

    let result = qualify_with_limits(
        85_720,
        concat!(
            "a{text-decoration-line:overline underline;",
            "text-decoration-line:underline overline;}"
        ),
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[Overline, Underline])],
    );
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{text-decoration-line:overline underline blink;}",
        "b{text-decoration-line:none;}",
        "c{text-decoration-line:inherit;}",
        "d{text-decoration-line:var(--line);}",
        "e{text-decoration-line:underline underline;}",
    );

    let first = qualify(85_721, css);
    let repeated = qualify(85_721, css);
    assert_eq!(
        first.text_decoration_line_observations(),
        repeated.text_decoration_line_observations()
    );

    let other_source = qualify(85_722, css);
    let first_outcomes: Vec<_> = first
        .text_decoration_line_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let other_outcomes: Vec<_> = other_source
        .text_decoration_line_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(first_outcomes, other_outcomes);
}
