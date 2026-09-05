use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssContainComponent, CssContainQualificationOutcome, CssContainUnsupportedReason,
    CssContainValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    None,
    Strict,
    Content,
    Components(&'static [CssContainComponent]),
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
    let actual = result.contain_observations();
    assert_eq!(actual.len(), expected.len());

    for (observation, expected) in actual.iter().zip(expected.iter().copied()) {
        match (observation.outcome(), expected) {
            (
                CssContainQualificationOutcome::Qualified(CssContainValue::None),
                ExpectedOutcome::None,
            )
            | (
                CssContainQualificationOutcome::Qualified(CssContainValue::Strict),
                ExpectedOutcome::Strict,
            )
            | (
                CssContainQualificationOutcome::Qualified(CssContainValue::Content),
                ExpectedOutcome::Content,
            ) => {}
            (
                CssContainQualificationOutcome::Qualified(CssContainValue::Components(components)),
                ExpectedOutcome::Components(expected),
            ) => assert_eq!(components.authored_components(), expected),
            (
                CssContainQualificationOutcome::InvalidForSelectedValueGrammar,
                ExpectedOutcome::Invalid,
            ) => {}
            (
                CssContainQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssContainUnsupportedReason::CssWideKeyword,
                ),
                ExpectedOutcome::UnsupportedCssWide,
            ) => {}
            (
                CssContainQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssContainUnsupportedReason::DeferredSubstitutionFunction,
                ),
                ExpectedOutcome::UnsupportedDeferredFunction,
            ) => {}
            (
                CssContainQualificationOutcome::UnsupportedBySelectedValueProfile(
                    CssContainUnsupportedReason::WholeValueFunction,
                ),
                ExpectedOutcome::UnsupportedWholeValueFunction,
            ) => {}
            (actual, expected) => {
                panic!("unexpected contain outcome: {actual:?}, expected {expected:?}")
            }
        }
    }
}

#[test]
fn handwritten_unordered_boundary_matches_pinned_wpt_and_derived_theorem() {
    use CssContainComponent::{InlineSize, Layout, Paint, Size, Style};

    let result = qualify(
        85_340,
        concat!(
            "a{contain:none;}",
            "b{contain:strict;}",
            "c{contain:content;}",
            "d{contain:size;}",
            "e{contain:inline-size;}",
            "f{contain:layout;}",
            "g{contain:style;}",
            "h{contain:paint;}",
            "i{contain:layout size;}",
            "j{contain:paint style;}",
            "k{contain:layout style paint;}",
            "l{contain:layout paint style size;}",
            "m{contain:layout inline-size;}",
            "n{contain:size layout;}",
            "o{contain:paint layout size style;}",
            "p{contain:layout layout;}",
            "q{contain:paint layout style paint;}",
            "r{contain:size layout size;}",
            "s{contain:inline-size inline-size;}",
            "t{contain:size inline-size;}",
            "u{contain:inline-size size;}",
            "v{contain:none none;}",
            "w{contain:strict layout;}",
            "x{contain:paint content;}",
            "y{contain:auto;}",
            "z{contain:0;}",
            "aa{contain:\"layout\";}",
            "ab{contain:;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::None,
            ExpectedOutcome::Strict,
            ExpectedOutcome::Content,
            ExpectedOutcome::Components(&[Size]),
            ExpectedOutcome::Components(&[InlineSize]),
            ExpectedOutcome::Components(&[Layout]),
            ExpectedOutcome::Components(&[Style]),
            ExpectedOutcome::Components(&[Paint]),
            ExpectedOutcome::Components(&[Layout, Size]),
            ExpectedOutcome::Components(&[Paint, Style]),
            ExpectedOutcome::Components(&[Layout, Style, Paint]),
            ExpectedOutcome::Components(&[Layout, Paint, Style, Size]),
            ExpectedOutcome::Components(&[Layout, InlineSize]),
            ExpectedOutcome::Components(&[Size, Layout]),
            ExpectedOutcome::Components(&[Paint, Layout, Size, Style]),
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
fn authored_order_and_standalone_identity_are_preserved_without_normalization() {
    use CssContainComponent::{Layout, Paint, Size, Style};

    let result = qualify(
        85_341,
        concat!(
            "a{contain:layout size;}",
            "b{contain:size layout;}",
            "c{contain:paint style;}",
            "d{contain:style paint;}",
            "e{contain:strict;}",
            "f{contain:content;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Layout, Size]),
            ExpectedOutcome::Components(&[Size, Layout]),
            ExpectedOutcome::Components(&[Paint, Style]),
            ExpectedOutcome::Components(&[Style, Paint]),
            ExpectedOutcome::Strict,
            ExpectedOutcome::Content,
        ],
    );
}

#[test]
fn case_escapes_comments_and_priority_preserve_authored_components_and_placement() {
    use CssContainComponent::{InlineSize, Layout, Paint, Size, Style};

    let result = qualify(
        85_342,
        concat!(
            "a{CONTAIN:LAYOUT SIZE;}",
            r"b{contain:\73 ize;}",
            r"c{contain:l\61 yout;}",
            r"d{contain:inline-\73 ize;}",
            r"e{cont\61 in:paint;}",
            "f{contain:/**/paint/**/style/**/!important;}",
            "g{contain:layout/**/size;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Layout, Size]),
            ExpectedOutcome::Components(&[Size]),
            ExpectedOutcome::Components(&[Layout]),
            ExpectedOutcome::Components(&[InlineSize]),
            ExpectedOutcome::Components(&[Paint]),
            ExpectedOutcome::Components(&[Paint, Style]),
            ExpectedOutcome::Components(&[Layout, Size]),
        ],
    );

    let priority_observation = &result.contain_observations()[5];
    let occurrence =
        &result.upstream_parser_result().occurrences()[priority_observation.occurrence_index()];
    assert_eq!(priority_observation.placement(), occurrence.placement());
    assert!(occurrence.priority().is_some());
}

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_single_value() {
    let result = qualify(
        85_343,
        concat!(
            "a{contain:initial;}",
            "b{contain:inherit;}",
            "c{contain:unset;}",
            "d{contain:revert;}",
            "e{contain:revert-layer;}",
            "f{contain:revert-rule;}",
            "g{contain:inherit layout;}",
            "h{contain:layout inherit;}",
            "i{contain:initial paint;}",
            "j{contain:initial revert;}",
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
}

#[test]
fn deferred_functions_fail_open_anywhere_while_other_functions_keep_existing_boundaries() {
    let result = qualify(
        85_344,
        concat!(
            "a{contain:var(--containment);}",
            "b{contain:layout var(--containment);}",
            "c{contain:var(--containment) layout;}",
            "d{contain:foo(var(--containment));}",
            "e{contain:first-valid(layout,paint);}",
            "f{contain:cycle(layout,paint);}",
            "g{contain:interpolate(0%,0:layout,1:paint);}",
            "h{contain:layout first-valid(paint);}",
            "i{contain:first-valid(layout) paint;}",
            "j{contain:foo();}",
            "k{contain:layout foo();}",
            "l{contain:foo() layout;}",
            "m{contain:calc(1);}",
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
fn applicability_is_not_an_input_to_authored_value_qualification() {
    use CssContainComponent::{InlineSize, Layout, Paint, Size};

    let result = qualify(
        85_345,
        concat!(
            "span{contain:size layout;}",
            "table{contain:inline-size paint;}",
            "svg{contain:layout;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Size, Layout]),
            ExpectedOutcome::Components(&[InlineSize, Paint]),
            ExpectedOutcome::Components(&[Layout]),
        ],
    );
}

#[test]
fn property_name_value_keyword_collision_and_cross_dispatch_remain_isolated() {
    use CssContainComponent::{Layout, Size};

    let result = qualify(
        85_346,
        concat!(
            "a{direction:ltr;}",
            "b{overscroll-behavior:contain chain;}",
            "c{contain:layout size;}",
            "d{overscroll-behavior-x:contain;}",
            "e{clip-rule:evenodd;}",
        ),
    );

    assert_eq!(result.direction_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(result.overscroll_behavior_x_observations().len(), 1);
    assert_eq!(result.clip_rule_observations().len(), 1);
    assert_eq!(result.contain_observations()[0].occurrence_index(), 2);
    assert_expected(&result, &[ExpectedOutcome::Components(&[Layout, Size])]);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    use CssContainComponent::Layout;

    let result = qualify(85_347, "a{contain:layout;}b{contain:layout;}");

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Components(&[Layout]),
            ExpectedOutcome::Components(&[Layout]),
        ],
    );
    assert_ne!(
        result.contain_observations()[0].placement().context_id(),
        result.contain_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (85_350, "@font-face{contain:size;}"),
        (85_351, "@page{contain:size;}"),
        (85_352, "@page{@top-left{contain:size;}}"),
        (85_353, "@keyframes k{from{contain:size;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.contain_observations().is_empty(),
            "nonordinary declaration context produced a contain observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    use CssContainComponent::{Layout, Size};

    let result = qualify_with_limits(
        85_360,
        "a{contain:layout size;contain:size layout;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::Components(&[Layout, Size])]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{contain:layout size;}",
        "b{contain:strict;}",
        "c{contain:inherit;}",
        "d{contain:var(--containment);}",
        "e{overscroll-behavior:contain chain;}",
    );
    let first = qualify(85_370, css);
    let repeated = qualify(85_370, css);
    let another_source = qualify(85_371, css);

    assert_eq!(
        first.contain_observations(),
        repeated.contain_observations()
    );
    assert_eq!(
        first.contain_observations(),
        another_source.contain_observations()
    );
}
