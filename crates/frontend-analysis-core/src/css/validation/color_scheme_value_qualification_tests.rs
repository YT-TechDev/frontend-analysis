use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssColorSchemeItemValue, CssColorSchemeQualificationOutcome, CssColorSchemeUnsupportedReason,
    CssColorSchemeValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

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

fn outcome_at(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &CssColorSchemeQualificationOutcome {
    result.color_scheme_observations()[index].outcome()
}

fn assert_normal(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssColorSchemeQualificationOutcome::Qualified(CssColorSchemeValue::Normal),
        "expected whole-value Normal at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssColorSchemeQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssColorSchemeUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssColorSchemeQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Test-local mirror of one resolved `color-scheme` scheme item, comparable
/// against a handwritten oracle without reproducing the production
/// classifier. `Custom` resolves the item's tokenizer-owned decoded
/// `<custom-ident>` text through the run-local evidence reference.
#[derive(Debug, PartialEq, Eq)]
enum TestSchemeItem<'a> {
    Light,
    Dark,
    Custom(Option<&'a str>),
}

/// Resolves one `Qualified(Schemes { .. })` observation's ordered items and
/// `only` flag. Panics if the observation at `index` is not
/// `Qualified(Schemes { .. })`.
fn schemes_at(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> (Vec<TestSchemeItem<'_>>, bool) {
    match outcome_at(result, index) {
        CssColorSchemeQualificationOutcome::Qualified(CssColorSchemeValue::Schemes {
            items,
            only,
        }) => {
            let resolved = items
                .iter()
                .map(|item| match item {
                    CssColorSchemeItemValue::Light => TestSchemeItem::Light,
                    CssColorSchemeItemValue::Dark => TestSchemeItem::Dark,
                    CssColorSchemeItemValue::CustomIdent(evidence) => {
                        TestSchemeItem::Custom(result.color_scheme_custom_ident_value(*evidence))
                    }
                })
                .collect();
            (resolved, *only)
        }
        other => panic!("expected Qualified(Schemes {{ .. }}) at index {index}, got {other:?}"),
    }
}

/// Resolves one observation's ordered qualified `CustomIdent` items'
/// absolute lexical-item indices, verifying each retained evidence
/// position is a direct `Ident` token -- never trivia, a comma, or a
/// neighboring item.
fn custom_ident_indices(result: &CssValueQualificationRunResult, index: usize) -> Vec<usize> {
    match outcome_at(result, index) {
        CssColorSchemeQualificationOutcome::Qualified(CssColorSchemeValue::Schemes {
            items,
            ..
        }) => items
            .iter()
            .filter_map(|item| match item {
                CssColorSchemeItemValue::CustomIdent(evidence) => Some(*evidence),
                _ => None,
            })
            .map(|evidence| {
                let lexical_index = evidence.lexical_item_index();
                let item = &result
                    .upstream_parser_result()
                    .upstream_tokenizer_result()
                    .lexical_items()[lexical_index];
                let CssLexicalItem::SemanticToken(token) = item else {
                    panic!("color-scheme custom-ident evidence ref pointed to trivia");
                };
                assert!(
                    matches!(token.kind(), CssTokenKind::Ident(_)),
                    "color-scheme custom-ident evidence ref pointed to unexpected token kind: {:?}",
                    token.kind()
                );
                lexical_index
            })
            .collect(),
        other => panic!("expected Qualified(Schemes {{ .. }}) at index {index}, got {other:?}"),
    }
}

// A. Whole `normal`.

#[test]
fn whole_normal_qualifies_ascii_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        99200,
        concat!(
            "a{color-scheme:normal;}",
            "b{color-scheme:NORMAL;}",
            "c{color-scheme:NoRmAl;}",
            "d{color-scheme:\\6e ormal;}",
        ),
    );

    for index in 0..4 {
        assert_normal(&result, index);
    }
}

// B. `normal` never combines with the composite branch.

#[test]
fn normal_is_invalid_when_combined_with_composite_branch() {
    let result = qualify(
        99201,
        concat!(
            "a{color-scheme:normal light;}",
            "b{color-scheme:light normal;}",
            "c{color-scheme:only normal;}",
            "d{color-scheme:normal only;}",
            "e{color-scheme:normal dark;}",
            "f{color-scheme:normal purple;}",
            "g{color-scheme:purple normal;}",
        ),
    );

    for index in 0..7 {
        assert_invalid(&result, index);
    }
}

// C. Light/dark order, duplicates.

#[test]
fn light_dark_keywords_preserve_exact_order_and_multiplicity() {
    let result = qualify(
        99202,
        concat!(
            "a{color-scheme:light;}",
            "b{color-scheme:dark;}",
            "c{color-scheme:light dark;}",
            "d{color-scheme:dark light;}",
            "e{color-scheme:light light;}",
            "f{color-scheme:dark dark;}",
            "g{color-scheme:LIGHT;}",
            "h{color-scheme:Dark;}",
        ),
    );

    assert_eq!(schemes_at(&result, 0), (vec![TestSchemeItem::Light], false));
    assert_eq!(schemes_at(&result, 1), (vec![TestSchemeItem::Dark], false));
    assert_eq!(
        schemes_at(&result, 2),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], false)
    );
    assert_eq!(
        schemes_at(&result, 3),
        (vec![TestSchemeItem::Dark, TestSchemeItem::Light], false)
    );
    assert_eq!(
        schemes_at(&result, 4),
        (vec![TestSchemeItem::Light, TestSchemeItem::Light], false)
    );
    assert_eq!(
        schemes_at(&result, 5),
        (vec![TestSchemeItem::Dark, TestSchemeItem::Dark], false)
    );
    assert_eq!(schemes_at(&result, 6), (vec![TestSchemeItem::Light], false));
    assert_eq!(schemes_at(&result, 7), (vec![TestSchemeItem::Dark], false));
}

// D. Custom-ident items: heterogeneous order, duplicates, case sensitivity,
// and unreserved keyword-looking names.

#[test]
fn custom_ident_items_qualify_case_sensitively_with_exact_order_and_multiplicity() {
    let result = qualify(
        99203,
        concat!(
            "a{color-scheme:purple;}",
            "b{color-scheme:light purple;}",
            "c{color-scheme:purple dark interesting;}",
            "d{color-scheme:Foo foo FOO;}",
            "e{color-scheme:none;}",
            "f{color-scheme:light none;}",
            "g{color-scheme:auto;}",
            "h{color-scheme:and;}",
            "i{color-scheme:not;}",
            "j{color-scheme:or;}",
        ),
    );

    assert_eq!(
        schemes_at(&result, 0),
        (vec![TestSchemeItem::Custom(Some("purple"))], false)
    );
    assert_eq!(
        schemes_at(&result, 1),
        (
            vec![
                TestSchemeItem::Light,
                TestSchemeItem::Custom(Some("purple"))
            ],
            false
        )
    );
    assert_eq!(
        schemes_at(&result, 2),
        (
            vec![
                TestSchemeItem::Custom(Some("purple")),
                TestSchemeItem::Dark,
                TestSchemeItem::Custom(Some("interesting")),
            ],
            false
        )
    );
    let (foo_items, foo_only) = schemes_at(&result, 3);
    assert!(!foo_only);
    assert_eq!(
        foo_items,
        vec![
            TestSchemeItem::Custom(Some("Foo")),
            TestSchemeItem::Custom(Some("foo")),
            TestSchemeItem::Custom(Some("FOO")),
        ]
    );
    assert_ne!(foo_items[0], foo_items[1]);
    assert_ne!(foo_items[1], foo_items[2]);
    assert_eq!(
        schemes_at(&result, 4),
        (vec![TestSchemeItem::Custom(Some("none"))], false)
    );
    assert_eq!(
        schemes_at(&result, 5),
        (
            vec![TestSchemeItem::Light, TestSchemeItem::Custom(Some("none"))],
            false
        )
    );
    assert_eq!(
        schemes_at(&result, 6),
        (vec![TestSchemeItem::Custom(Some("auto"))], false)
    );
    assert_eq!(
        schemes_at(&result, 7),
        (vec![TestSchemeItem::Custom(Some("and"))], false)
    );
    assert_eq!(
        schemes_at(&result, 8),
        (vec![TestSchemeItem::Custom(Some("not"))], false)
    );
    assert_eq!(
        schemes_at(&result, 9),
        (vec![TestSchemeItem::Custom(Some("or"))], false)
    );
}

// E. Reserved identifiers: normal/light/dark/only take keyword semantics,
// never custom-ident; `default` is direct Invalid.

#[test]
fn reserved_identifiers_never_take_custom_ident_semantics() {
    let result = qualify(
        99204,
        concat!("a{color-scheme:default;}", "b{color-scheme:light default;}",),
    );

    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// F. `only` valid edge placement.

#[test]
fn only_modifier_qualifies_before_or_after_the_whole_scheme_group() {
    let result = qualify(
        99205,
        concat!(
            "a{color-scheme:only light;}",
            "b{color-scheme:light only;}",
            "c{color-scheme:only dark;}",
            "d{color-scheme:dark only;}",
            "e{color-scheme:only light dark;}",
            "f{color-scheme:light dark only;}",
            "g{color-scheme:only light light;}",
            "h{color-scheme:light light only;}",
            "i{color-scheme:only purple;}",
            "j{color-scheme:purple only;}",
            "k{color-scheme:only none;}",
            "l{color-scheme:none only;}",
        ),
    );

    assert_eq!(schemes_at(&result, 0), (vec![TestSchemeItem::Light], true));
    assert_eq!(schemes_at(&result, 1), (vec![TestSchemeItem::Light], true));
    assert_eq!(schemes_at(&result, 2), (vec![TestSchemeItem::Dark], true));
    assert_eq!(schemes_at(&result, 3), (vec![TestSchemeItem::Dark], true));
    assert_eq!(
        schemes_at(&result, 4),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], true)
    );
    assert_eq!(
        schemes_at(&result, 5),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], true)
    );
    assert_eq!(
        schemes_at(&result, 6),
        (vec![TestSchemeItem::Light, TestSchemeItem::Light], true)
    );
    assert_eq!(
        schemes_at(&result, 7),
        (vec![TestSchemeItem::Light, TestSchemeItem::Light], true)
    );
    assert_eq!(
        schemes_at(&result, 8),
        (vec![TestSchemeItem::Custom(Some("purple"))], true)
    );
    assert_eq!(
        schemes_at(&result, 9),
        (vec![TestSchemeItem::Custom(Some("purple"))], true)
    );
    assert_eq!(
        schemes_at(&result, 10),
        (vec![TestSchemeItem::Custom(Some("none"))], true)
    );
    assert_eq!(
        schemes_at(&result, 11),
        (vec![TestSchemeItem::Custom(Some("none"))], true)
    );
}

// G. `only` invalid placement: sole, duplicated, or interior.

#[test]
fn only_modifier_is_invalid_when_sole_duplicated_or_interior() {
    let result = qualify(
        99206,
        concat!(
            "a{color-scheme:only;}",
            "b{color-scheme:only only;}",
            "c{color-scheme:light only dark;}",
            "d{color-scheme:only light only;}",
            "e{color-scheme:light only only;}",
            "f{color-scheme:only only light;}",
            "g{color-scheme:purple only light;}",
            "h{color-scheme:light only purple;}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// H. Comma is invalid -- this is `+`, not `#`.

#[test]
fn comma_separated_syntax_is_invalid_in_every_spacing_variant() {
    let result = qualify(
        99207,
        concat!(
            "a{color-scheme:light, dark;}",
            "b{color-scheme:light,dark;}",
            "c{color-scheme:light,;}",
            "d{color-scheme:,light;}",
            "e{color-scheme:light,,dark;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// I. Wrong token classes.

#[test]
fn wrong_token_classes_are_invalid() {
    let result = qualify(
        99208,
        concat!(
            "a{color-scheme:\"light\";}",
            "b{color-scheme:\"none\";}",
            "c{color-scheme:\"purple\";}",
            "d{color-scheme:1;}",
            "e{color-scheme:1px;}",
            "f{color-scheme:50%;}",
            "g{color-scheme:#fff;}",
            "h{color-scheme:url(foo);}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// J. Empty value.

#[test]
fn empty_value_is_invalid() {
    let result = qualify(99209, "a{color-scheme:;}");
    assert_invalid(&result, 0);
}

// K. Ordinary Functions have no direct branch.

#[test]
fn ordinary_function_items_are_invalid_never_softened_to_function_value_unsupported() {
    let result = qualify(
        99210,
        concat!(
            "a{color-scheme:foo();}",
            "b{color-scheme:calc(1);}",
            "c{color-scheme:light foo();}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// L. Deferred substitution precedes direct classification.

#[test]
fn deferred_substitution_precedes_direct_classification_everywhere() {
    let result = qualify(
        99211,
        concat!(
            "a{color-scheme:var(--x);}",
            "b{color-scheme:light var(--x);}",
            "c{color-scheme:var(--x) only;}",
            "d{color-scheme:ident(\"purple\");}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(
            &result,
            index,
            CssColorSchemeUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// M. Generic whole-value Function placement.

#[test]
fn whole_value_generic_function_boundary_does_not_legalize_embedded_placement() {
    let whole = qualify(99212, "a{color-scheme:first-valid(light, dark);}");
    assert_unsupported(
        &whole,
        0,
        CssColorSchemeUnsupportedReason::WholeValueFunction,
    );

    let embedded = qualify(
        99213,
        concat!(
            "a{color-scheme:light first-valid(dark);}",
            "b{color-scheme:first-valid(light) dark;}",
        ),
    );
    for index in 0..2 {
        assert_invalid(&embedded, index);
    }
}

// N. CSS-wide boundary: sole whole-value Unsupported, embedded Invalid.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        99214,
        concat!(
            "a{color-scheme:initial;}",
            "b{color-scheme:inherit;}",
            "c{color-scheme:unset;}",
            "d{color-scheme:revert;}",
            "e{color-scheme:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssColorSchemeUnsupportedReason::CssWideKeyword,
        );
    }

    let embedded = qualify(
        99215,
        concat!(
            "a{color-scheme:light inherit;}",
            "b{color-scheme:initial dark;}",
            "c{color-scheme:only inherit;}",
            "d{color-scheme:dark unset;}",
        ),
    );
    for index in 0..4 {
        assert_invalid(&embedded, index);
    }
}

// O. Ordered evidence refs through trivia; escape-authored identifiers.

#[test]
fn ordered_evidence_refs_point_to_the_exact_retained_ident_tokens_through_trivia() {
    let result = qualify(
        99216,
        concat!(
            "a{color-scheme:\n",
            "    /*a*/ Purple /*b*/\n",
            "    /*c*/ dark /*d*/\n",
            "    /*e*/ purple;}",
        ),
    );

    let (items, only) = schemes_at(&result, 0);
    assert!(!only);
    assert_eq!(
        items,
        vec![
            TestSchemeItem::Custom(Some("Purple")),
            TestSchemeItem::Dark,
            TestSchemeItem::Custom(Some("purple")),
        ]
    );
    assert_ne!(items[0], items[2]);
    let indices = custom_ident_indices(&result, 0);
    assert_eq!(indices.len(), 2);
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn dense_whitespace_and_comment_variants_produce_identical_ordered_schemes() {
    let spaced = qualify(99217, "a{color-scheme:only light dark;}");
    let multiline = qualify(99218, "a{color-scheme:\n    only\n    light\n    dark;}");
    let comment_adjacent = qualify(99219, "a{color-scheme:only/*a*/light dark;}");
    let comment_padded = qualify(99220, "a{color-scheme:light dark/*a*/ /*b*/only/*c*/;}");

    assert_eq!(
        schemes_at(&spaced, 0),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], true)
    );
    assert_eq!(
        schemes_at(&multiline, 0),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], true)
    );
    assert_eq!(
        schemes_at(&comment_adjacent, 0),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], true)
    );
    assert_eq!(
        schemes_at(&comment_padded, 0),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], true)
    );
}

#[test]
fn escape_authored_identifiers_decode_through_the_tokenizer() {
    let result = qualify(
        99221,
        concat!("a{color-scheme:\\6c ight;}", "b{color-scheme:\\!escaped;}",),
    );

    assert_eq!(schemes_at(&result, 0), (vec![TestSchemeItem::Light], false));
    assert_eq!(
        schemes_at(&result, 1),
        (vec![TestSchemeItem::Custom(Some("!escaped"))], false)
    );
}

// P. `!important` and duplicate declaration occurrences.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(99222, "a{color-scheme:light dark only !important;}");
    assert_eq!(
        schemes_at(&result, 0),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], true)
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(
        99223,
        "a{color-scheme:normal;color-scheme:light dark;color-scheme:only purple;}",
    );

    assert_eq!(result.color_scheme_observations().len(), 3);
    assert_normal(&result, 0);
    assert_eq!(
        schemes_at(&result, 1),
        (vec![TestSchemeItem::Light, TestSchemeItem::Dark], false)
    );
    assert_eq!(
        schemes_at(&result, 2),
        (vec![TestSchemeItem::Custom(Some("purple"))], true)
    );
}

// Q. Cross-dispatch separation.

#[test]
fn cross_dispatch_separation_from_other_qualified_leaves() {
    let result = qualify(
        99224,
        concat!(
            "a{color-scheme:light;color-scheme:dark only;}",
            "b{container-name:foo;}",
            "c{anchor-name:--foo;}",
            "d{animation-name:foo;}",
            "e{transition-property:width;}",
            "f{text-emphasis-position:over left;}",
            "g{offset-rotate:auto;}",
        ),
    );

    assert_eq!(result.color_scheme_observations().len(), 2);
    assert_eq!(result.color_scheme_observations()[0].occurrence_index(), 0);
    assert_eq!(result.color_scheme_observations()[1].occurrence_index(), 1);
    assert_eq!(schemes_at(&result, 0), (vec![TestSchemeItem::Light], false));
    assert_eq!(schemes_at(&result, 1), (vec![TestSchemeItem::Dark], true));
    assert_eq!(result.container_name_observations().len(), 1);
    assert_eq!(result.anchor_name_observations().len(), 1);
    assert_eq!(result.animation_name_observations().len(), 1);
    assert_eq!(result.transition_property_observations().len(), 1);
    assert_eq!(result.text_emphasis_position_observations().len(), 1);
    assert_eq!(result.offset_rotate_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_color_scheme_dispatch() {
    for (source_id, css) in [
        (99225, "@font-face{color-scheme:light;}"),
        (99226, "@page{color-scheme:light;}"),
        (99227, "@keyframes k{from{color-scheme:light;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.color_scheme_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    let result = qualify(
        99228,
        concat!(
            "a{color-scheme:foo(light dark) only;}",
            "b{color-scheme:(light dark) only;}",
            "c{color-scheme:light) dark;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// R. Resource / committed prefix / determinism.

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    let incomplete = qualify_with_limits(
        99229,
        "a{color-scheme:light;}b{color-scheme:light dark only;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.color_scheme_observations().len(), 1);
    assert_eq!(
        schemes_at(&incomplete, 0),
        (vec![TestSchemeItem::Light], false)
    );

    let css = concat!(
        "a{color-scheme:only light dark;}",
        "b{color-scheme:normal;}",
        "c{color-scheme:var(--x);}",
        "d{color-scheme:normal light;}",
    );
    let first = qualify(99230, css);
    let repeated = qualify(99230, css);
    let another_source = qualify(99231, css);

    assert_eq!(
        first.color_scheme_observations(),
        repeated.color_scheme_observations()
    );
    assert_eq!(
        first.color_scheme_observations(),
        another_source.color_scheme_observations()
    );
}
