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
fn handwritten_fixed_slot_boundary_matches_pinned_wpt_and_derived_theorem() {
    use CssTextDecorationLineComponent::{Blink, LineThrough, Overline, Underline};
    let result = qualify(
        86_000,
        concat!(
            "a{text-decoration-line:none;}",
            "b{text-decoration-line:spelling-error;}",
            "c{text-decoration-line:grammar-error;}",
            "d{text-decoration-line:underline;}",
            "e{text-decoration-line:overline;}",
            "f{text-decoration-line:line-through;}",
            "g{text-decoration-line:blink;}",
            "h{text-decoration-line:underline overline line-through blink;}",
            "i{text-decoration-line:underline underline;}",
            "j{text-decoration-line:blink line-through blink;}",
            "k{text-decoration-line:none underline;}",
            "l{text-decoration-line:spelling-error overline;}",
            "m{text-decoration-line:spelling-error grammar-error;}",
            "n{text-decoration-line:auto;}",
            "o{text-decoration-line:0;}",
            "p{text-decoration-line:\"underline\";}",
            "q{text-decoration-line:;}",
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
            ExpectedOutcome::Components(&[Underline, Overline, LineThrough, Blink]),
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
        86_001,
        concat!(
            "a{text-decoration-line:overline underline;}",
            "b{text-decoration-line:underline overline;}",
            "c{text-decoration-line:blink line-through underline;}",
            "d{text-decoration-line:underline line-through blink;}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Overline, Underline]),
            ExpectedOutcome::Components(&[Underline, Overline]),
            ExpectedOutcome::Components(&[Blink, LineThrough, Underline]),
            ExpectedOutcome::Components(&[Underline, LineThrough, Blink]),
        ],
    );
}

#[test]
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssTextDecorationLineComponent::{Blink, Overline, Underline};
    let result = qualify(
        86_002,
        concat!(
            "a{TEXT-DECORATION-LINE:UNDERLINE OVERLINE;}",
            r"b{text-decoration-line:\75 nderline;}",
            r"c{text-decoratio\6e -line:overline;}",
            "d{text-decoration-line:/**/blink/**/underline/**/!important;}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Underline, Overline]),
            ExpectedOutcome::Components(&[Underline]),
            ExpectedOutcome::Components(&[Overline]),
            ExpectedOutcome::Components(&[Blink, Underline]),
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
        86_003,
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
        86_004,
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
        86_005,
        concat!(
            "a{font-variant-numeric:oldstyle-nums tabular-nums;}",
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
    let result = qualify(
        86_006,
        "a{text-decoration-line:spelling-error;}b{text-decoration-line:spelling-error;}",
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::SpellingError,
            ExpectedOutcome::SpellingError,
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
        (86_010, "@font-face{text-decoration-line:underline;}"),
        (86_011, "@page{text-decoration-line:underline;}"),
        (86_012, "@page{@top-left{text-decoration-line:underline;}}"),
        (
            86_013,
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
        86_020,
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
    let first = qualify(86_021, css);
    let repeated = qualify(86_021, css);
    assert_eq!(
        first.text_decoration_line_observations(),
        repeated.text_decoration_line_observations()
    );

    let other_source = qualify(86_022, css);
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

#[test]
fn direct_values_do_not_collapse_into_each_other() {
    let result = qualify(
        86_030,
        concat!(
            "a{text-decoration-line:none;}",
            "b{text-decoration-line:spelling-error;}",
            "c{text-decoration-line:grammar-error;}",
            "d{text-decoration-line:underline;}",
            "e{text-decoration-line:overline;}",
            "f{text-decoration-line:line-through;}",
            "g{text-decoration-line:blink;}",
        ),
    );
    let observations = result.text_decoration_line_observations();
    for i in 0..observations.len() {
        for j in (i + 1)..observations.len() {
            assert_ne!(
                observations[i].outcome(),
                observations[j].outcome(),
                "observations {i} and {j} unexpectedly collapsed"
            );
        }
    }
}

#[test]
fn all_six_unordered_pairs_qualify_in_both_authored_orders() {
    use CssTextDecorationLineComponent::{Blink, LineThrough, Overline, Underline};
    let result = qualify(
        86_031,
        concat!(
            "a{text-decoration-line:underline overline;}",
            "b{text-decoration-line:overline underline;}",
            "c{text-decoration-line:underline line-through;}",
            "d{text-decoration-line:line-through underline;}",
            "e{text-decoration-line:underline blink;}",
            "f{text-decoration-line:blink underline;}",
            "g{text-decoration-line:overline line-through;}",
            "h{text-decoration-line:line-through overline;}",
            "i{text-decoration-line:overline blink;}",
            "j{text-decoration-line:blink overline;}",
            "k{text-decoration-line:line-through blink;}",
            "l{text-decoration-line:blink line-through;}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Underline, Overline]),
            ExpectedOutcome::Components(&[Overline, Underline]),
            ExpectedOutcome::Components(&[Underline, LineThrough]),
            ExpectedOutcome::Components(&[LineThrough, Underline]),
            ExpectedOutcome::Components(&[Underline, Blink]),
            ExpectedOutcome::Components(&[Blink, Underline]),
            ExpectedOutcome::Components(&[Overline, LineThrough]),
            ExpectedOutcome::Components(&[LineThrough, Overline]),
            ExpectedOutcome::Components(&[Overline, Blink]),
            ExpectedOutcome::Components(&[Blink, Overline]),
            ExpectedOutcome::Components(&[LineThrough, Blink]),
            ExpectedOutcome::Components(&[Blink, LineThrough]),
        ],
    );
}

#[test]
fn three_keyword_subset_qualifies_in_every_authored_permutation() {
    use CssTextDecorationLineComponent::{Blink, Overline, Underline};
    let result = qualify(
        86_032,
        concat!(
            "a{text-decoration-line:underline overline blink;}",
            "b{text-decoration-line:underline blink overline;}",
            "c{text-decoration-line:overline underline blink;}",
            "d{text-decoration-line:overline blink underline;}",
            "e{text-decoration-line:blink underline overline;}",
            "f{text-decoration-line:blink overline underline;}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Underline, Overline, Blink]),
            ExpectedOutcome::Components(&[Underline, Blink, Overline]),
            ExpectedOutcome::Components(&[Overline, Underline, Blink]),
            ExpectedOutcome::Components(&[Overline, Blink, Underline]),
            ExpectedOutcome::Components(&[Blink, Underline, Overline]),
            ExpectedOutcome::Components(&[Blink, Overline, Underline]),
        ],
    );
}

#[test]
fn four_keyword_reverse_permutation_also_qualifies() {
    use CssTextDecorationLineComponent::{Blink, LineThrough, Overline, Underline};
    let result = qualify(
        86_033,
        "a{text-decoration-line:blink line-through overline underline;}",
    );
    assert_expected(
        &result,
        &[ExpectedOutcome::Components(&[
            Blink,
            LineThrough,
            Overline,
            Underline,
        ])],
    );
}

#[test]
fn additional_duplicate_keyword_combinations_are_invalid() {
    let result = qualify(
        86_034,
        concat!(
            "a{text-decoration-line:overline overline;}",
            "b{text-decoration-line:line-through line-through;}",
            "c{text-decoration-line:overline underline overline;}",
            "d{text-decoration-line:underline overline line-through blink blink;}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn additional_exclusive_singleton_mixing_is_invalid_in_either_order() {
    let result = qualify(
        86_035,
        concat!(
            "a{text-decoration-line:underline none;}",
            "b{text-decoration-line:none spelling-error;}",
            "c{text-decoration-line:overline spelling-error;}",
            "d{text-decoration-line:grammar-error blink;}",
            "e{text-decoration-line:blink grammar-error;}",
            "f{text-decoration-line:grammar-error spelling-error;}",
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
fn unrecognized_identifiers_are_invalid() {
    let result = qualify(
        86_036,
        concat!(
            "a{text-decoration-line:normal;}",
            "b{text-decoration-line:under-line;}",
            "c{text-decoration-line:over-line;}",
            "d{text-decoration-line:linethrough;}",
            "e{text-decoration-line:foo;}",
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
        ],
    );
}

#[test]
fn additional_embedded_css_wide_forms_are_invalid() {
    let result = qualify(
        86_037,
        concat!(
            "a{text-decoration-line:initial overline;}",
            "b{text-decoration-line:none inherit;}",
            "c{text-decoration-line:spelling-error revert;}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn additional_deferred_substitution_functions_fail_open() {
    let result = qualify(
        86_038,
        concat!(
            "a{text-decoration-line:env(foo);}",
            "b{text-decoration-line:attr(data-x);}",
            "c{text-decoration-line:--custom();}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
            ExpectedOutcome::UnsupportedDeferredFunction,
        ],
    );
}

#[test]
fn embedded_whole_value_function_is_invalid_regardless_of_position() {
    let result = qualify(
        86_039,
        "a{text-decoration-line:first-valid(underline) overline;}",
    );
    assert_expected(&result, &[ExpectedOutcome::Invalid]);
}

#[test]
fn additional_embedded_ordinary_functions_are_invalid() {
    let result = qualify(
        86_040,
        concat!(
            "a{text-decoration-line:foo() overline;}",
            "b{text-decoration-line:blink calc(1);}",
            "c{text-decoration-line:spelling-error foo();}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn comma_delimited_forms_are_invalid() {
    let result = qualify(
        86_041,
        concat!(
            "a{text-decoration-line:underline, overline;}",
            "b{text-decoration-line:underline,;}",
            "c{text-decoration-line:,overline;}",
            "d{text-decoration-line:underline overline,;}",
            "e{text-decoration-line:none, underline;}",
            "f{text-decoration-line:spelling-error, grammar-error;}",
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
fn case_insensitive_recognition_covers_remaining_direct_keywords() {
    use CssTextDecorationLineComponent::{Blink, LineThrough};
    let result = qualify(
        86_042,
        concat!(
            "a{text-decoration-line:NONE;}",
            "b{text-decoration-line:BLINK;}",
            "c{text-decoration-line:Line-Through;}",
            "d{text-decoration-line:SPELLING-ERROR;}",
            "e{text-decoration-line:Grammar-Error;}",
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::None,
            ExpectedOutcome::Components(&[Blink]),
            ExpectedOutcome::Components(&[LineThrough]),
            ExpectedOutcome::SpellingError,
            ExpectedOutcome::GrammarError,
        ],
    );
}
