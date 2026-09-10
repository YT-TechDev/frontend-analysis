use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssTransformBoxQualificationOutcome, CssTransformBoxUnsupportedReason, CssTransformBoxValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    ContentBox,
    BorderBox,
    FillBox,
    StrokeBox,
    ViewBox,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssTransformBoxQualificationOutcome {
    match expected {
        ExpectedOutcome::ContentBox => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::ContentBox)
        }
        ExpectedOutcome::BorderBox => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::BorderBox)
        }
        ExpectedOutcome::FillBox => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::FillBox)
        }
        ExpectedOutcome::StrokeBox => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::StrokeBox)
        }
        ExpectedOutcome::ViewBox => {
            CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::ViewBox)
        }
        ExpectedOutcome::Invalid => {
            CssTransformBoxQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssTransformBoxQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformBoxUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssTransformBoxQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformBoxUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssTransformBoxQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformBoxUnsupportedReason::WholeValueFunction,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssTransformBoxQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssTransformBoxUnsupportedReason::FunctionValue,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .transform_box_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

// A + B: all five direct keywords qualify distinctly, and ASCII
// case-insensitive / escape-equivalent authored spelling recognition matches
// existing keyword identity semantics.
#[test]
fn all_five_direct_keywords_qualify_case_insensitively() {
    let result = qualify(
        6100,
        concat!(
            "a{transform-box:content-box;}",
            "b{transform-box:border-box;}",
            "c{transform-box:fill-box;}",
            "d{transform-box:stroke-box;}",
            "e{transform-box:view-box;}",
            "f{transform-box:CONTENT-BOX;}",
            "g{transform-box:Border-Box;}",
            "h{transform-box:FILL-BOX;}",
            "i{transform-box:Stroke-Box;}",
            "j{transform-box:VIEW-BOX;}",
            r"k{transform-box:\63 ontent-box;}",
            r"l{transform-\62 ox:border-box;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::ContentBox,
            ExpectedOutcome::BorderBox,
            ExpectedOutcome::FillBox,
            ExpectedOutcome::StrokeBox,
            ExpectedOutcome::ViewBox,
            ExpectedOutcome::ContentBox,
            ExpectedOutcome::BorderBox,
            ExpectedOutcome::FillBox,
            ExpectedOutcome::StrokeBox,
            ExpectedOutcome::ViewBox,
            ExpectedOutcome::ContentBox,
            ExpectedOutcome::BorderBox,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

// C: authored identity distinction -- the five qualified variants are
// pairwise distinct results, not merely all "Qualified".
#[test]
fn qualified_authored_identities_remain_pairwise_distinct() {
    let result = qualify(
        6101,
        concat!(
            "a{transform-box:content-box;}",
            "b{transform-box:border-box;}",
            "c{transform-box:fill-box;}",
            "d{transform-box:stroke-box;}",
            "e{transform-box:view-box;}",
        ),
    );

    let outcomes: Vec<_> = result
        .transform_box_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    assert_eq!(outcomes.len(), 5);

    for (i, first) in outcomes.iter().enumerate() {
        for (j, second) in outcomes.iter().enumerate() {
            if i != j {
                assert_ne!(
                    first, second,
                    "authored transform-box identities at {i} and {j} unexpectedly collapsed"
                );
            }
        }
    }

    assert_eq!(
        outcomes[0],
        CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::ContentBox)
    );
    assert_ne!(
        outcomes[0],
        CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::FillBox)
    );
    assert_ne!(
        outcomes[1],
        CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::StrokeBox)
    );
    assert_ne!(
        outcomes[1],
        CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::ViewBox)
    );
    assert_ne!(
        outcomes[3],
        CssTransformBoxQualificationOutcome::Qualified(CssTransformBoxValue::ViewBox)
    );
}

// D: adjacent CSS box keywords from unrelated grammars are rejected -- this
// leaf never widens into generic box-edge syntax.
#[test]
fn adjacent_box_keywords_outside_the_pinned_grammar_are_invalid() {
    let result = qualify(
        6102,
        concat!(
            "a{transform-box:padding-box;}",
            "b{transform-box:margin-box;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::Invalid, ExpectedOutcome::Invalid],
    );
}

// E: arbitrary wrong identifiers remain ordinary Invalid identifiers.
#[test]
fn arbitrary_wrong_identifiers_are_invalid() {
    let result = qualify(
        6103,
        concat!(
            "a{transform-box:foo;}",
            "b{transform-box:reference-box;}",
            "c{transform-box:viewport-box;}",
            "d{transform-box:paint-box;}",
            "e{transform-box:coord-box;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}

// F: this is a single-component grammar -- two or more direct keywords in
// one value are invalid, in either order and including exact repetition.
#[test]
fn multiple_direct_keywords_are_invalid() {
    let result = qualify(
        6104,
        concat!(
            "a{transform-box:content-box border-box;}",
            "b{transform-box:fill-box view-box;}",
            "c{transform-box:view-box content-box;}",
            "d{transform-box:content-box content-box;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

// G: top-level comma-separated forms are invalid -- this never becomes a
// list-valued property.
#[test]
fn top_level_comma_forms_are_invalid() {
    let result = qualify(
        6105,
        concat!(
            "a{transform-box:content-box, border-box;}",
            "b{transform-box:content-box,;}",
            "c{transform-box:,content-box;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 3]);
}

// H + I + J: ordinary residual Function, recognized deferred/arbitrary
// substitution Function, and the sole recognized whole-value-only Function
// each preserve their own existing shared Unsupported boundary rather than
// being conflated with one another or with ordinary Invalid.
#[test]
fn function_boundaries_remain_distinct_and_fail_open() {
    let result = qualify(
        6106,
        concat!(
            "a{transform-box:foo();}",
            "b{transform-box:calc(1);}",
            "c{transform-box:var(--x);}",
            "d{transform-box:env(foo);}",
            "e{transform-box:attr(data-x);}",
            "f{transform-box:--custom();}",
            "g{transform-box:first-valid(content-box,border-box);}",
            "h{transform-box:cycle(content-box,border-box);}",
            "i{transform-box:interpolate(50%,0:content-box,1:border-box);}",
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

// J (embedded) + direct-Invalid-outranks-uncertainty: a whole-value-only
// Function embedded alongside a direct keyword is structurally invalid, not
// softened into a generic FunctionValue/whole-value outcome.
#[test]
fn embedded_whole_value_function_is_structurally_invalid() {
    let result = qualify(
        6107,
        concat!(
            "a{transform-box:content-box first-valid(border-box);}",
            "b{transform-box:first-valid(border-box) content-box;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 2]);
}

// K: sole CSS-wide keywords use the existing CSS-wide Unsupported outcome;
// embedded CSS-wide syntax is structurally invalid rather than whole-property
// CSS-wide handling.
#[test]
fn css_wide_keywords_are_unsupported_only_as_the_whole_value() {
    let result = qualify(
        6108,
        concat!(
            "a{transform-box:initial;}",
            "b{transform-box:inherit;}",
            "c{transform-box:unset;}",
            "d{transform-box:revert;}",
            "e{transform-box:revert-layer;}",
            "f{transform-box:revert-rule;}",
            "g{transform-box:content-box initial;}",
            "h{transform-box:initial border-box;}",
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

// L: trivia/comments surrounding a valid keyword retain the same qualified
// identity; an empty or trivia-only declaration value is invalid.
#[test]
fn trivia_and_comments_preserve_identity_and_empty_value_is_invalid() {
    let result = qualify(
        6109,
        concat!(
            "a{transform-box:/**/content-box/**/;}",
            "b{transform-box: stroke-box ;}",
            "c{transform-box:;}",
            "d{transform-box:/**/;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::ContentBox,
            ExpectedOutcome::StrokeBox,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

// M: `!important` qualifies the authored value according to the existing
// declaration priority boundary -- priority does not change the outcome.
#[test]
fn important_priority_does_not_change_qualification() {
    let result = qualify(6110, "a{transform-box:stroke-box !important;}");

    assert_expected(&result, &[ExpectedOutcome::StrokeBox]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// N: multiple `transform-box` declarations remain independent run-local
// observations -- no cascade or property deduplication is performed here.
#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(6111, "a{transform-box:view-box;}b{transform-box:view-box;}");

    assert_expected(
        &result,
        &[ExpectedOutcome::ViewBox, ExpectedOutcome::ViewBox],
    );
    assert_eq!(result.transform_box_observations()[0].occurrence_index(), 0);
    assert_eq!(result.transform_box_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.transform_box_observations()[0]
            .placement()
            .context_id(),
        result.transform_box_observations()[1]
            .placement()
            .context_id(),
    );
}

// O: only ordinary declaration placements enter this qualifier -- nonordinary
// declaration-shaped contexts produce no observation.
#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (6120, "@font-face{transform-box:view-box;}"),
        (6121, "@page{transform-box:view-box;}"),
        (6122, "@page{@top-left{transform-box:view-box;}}"),
        (6123, "@keyframes k{from{transform-box:view-box;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.transform_box_observations().is_empty(),
            "nonordinary declaration context produced a transform-box observation for {css:?}"
        );
    }
}

// P: upstream parser resource stop preserves only the committed retained
// prefix; the qualifier never upgrades parser completion.
#[test]
fn parser_resource_stop_preserves_committed_prefix_and_completion() {
    let result = qualify_with_limits(
        6130,
        "a{transform-box:content-box;transform-box:border-box;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::ContentBox]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

// Q: repeated qualification of equivalent source, and across distinct
// sources, is semantically deterministic.
#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{transform-box:content-box;}",
        "b{transform-box:inherit;}",
        "c{transform-box:view-box;}",
        "d{transform-box:var(--box);}",
        "e{box-sizing:border-box;}",
    );
    let first = qualify(6140, css);
    let repeated = qualify(6140, css);
    let another_source = qualify(6141, css);

    assert_eq!(
        first.transform_box_observations(),
        repeated.transform_box_observations()
    );
    assert_eq!(
        first.transform_box_observations(),
        another_source.transform_box_observations()
    );
}

// R: cross-leaf isolation -- one run interleaving `transform-box` with
// `box-sizing` and the other accepted transform leaves produces exactly one
// observation per property, with no cross-dispatch between leaves.
#[test]
fn one_run_interleaves_with_neighboring_accepted_leaves_without_cross_dispatch() {
    let result = qualify(
        6150,
        concat!(
            "a{box-sizing:border-box;}",
            "b{transform-origin:left top;}",
            "c{translate:10px;}",
            "d{rotate:45deg;}",
            "e{scale:2;}",
            "f{transform-box:fill-box;}",
        ),
    );

    assert_eq!(result.box_sizing_observations().len(), 1);
    assert_eq!(result.transform_origin_observations().len(), 1);
    assert_eq!(result.translate_observations().len(), 1);
    assert_eq!(result.rotate_observations().len(), 1);
    assert_eq!(result.scale_observations().len(), 1);
    assert_eq!(result.transform_box_observations().len(), 1);
    assert_eq!(result.transform_box_observations()[0].occurrence_index(), 5);
    assert_expected(&result, &[ExpectedOutcome::FillBox]);
}
