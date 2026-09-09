use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssValueQualificationRunResult, CssWillChangeItemValue, CssWillChangeQualificationOutcome,
    CssWillChangeUnsupportedReason, CssWillChangeValue, run,
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
) -> &CssWillChangeQualificationOutcome {
    result.will_change_observations()[index].outcome()
}

fn assert_auto(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssWillChangeQualificationOutcome::Qualified(CssWillChangeValue::Auto),
        "expected whole-value Auto branch at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssWillChangeQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssWillChangeUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssWillChangeQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Handwritten test-local shape for one resolved `<animateable-feature>`
/// item, independent of the production `CssWillChangeItemValue` shape, so
/// assertions compare interpreted identity rather than internal evidence
/// representation.
#[derive(Debug, PartialEq, Eq)]
enum TestFeature<'a> {
    ScrollPosition,
    Contents,
    CustomIdent(Option<&'a str>),
}

/// Resolves one observation's ordered qualified `<animateable-feature>`
/// items to their interpreted identity. Panics if the observation at
/// `index` is not `Qualified(Features(_))`.
fn features_at<'a>(
    result: &'a CssValueQualificationRunResult,
    index: usize,
) -> Vec<TestFeature<'a>> {
    match outcome_at(result, index) {
        CssWillChangeQualificationOutcome::Qualified(CssWillChangeValue::Features(items)) => items
            .iter()
            .map(|item| match item {
                CssWillChangeItemValue::ScrollPosition => TestFeature::ScrollPosition,
                CssWillChangeItemValue::Contents => TestFeature::Contents,
                CssWillChangeItemValue::CustomIdent(evidence) => {
                    TestFeature::CustomIdent(result.will_change_custom_ident_value(*evidence))
                }
            })
            .collect(),
        other => panic!("expected Qualified(Features(_)) at index {index}, got {other:?}"),
    }
}

fn assert_features_len(result: &CssValueQualificationRunResult, index: usize, expected_len: usize) {
    match outcome_at(result, index) {
        CssWillChangeQualificationOutcome::Qualified(CssWillChangeValue::Features(items)) => {
            assert_eq!(items.len(), expected_len, "at index {index}");
        }
        other => panic!("expected Qualified(Features(_)) at index {index}, got {other:?}"),
    }
}

// A. Whole `auto`, case variants and escape-decoded equivalent.

#[test]
fn whole_auto_qualifies_ascii_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        598000,
        concat!(
            "a{will-change:auto;}",
            "b{will-change:AUTO;}",
            "c{will-change:AuTo;}",
            "d{will-change:\\61 uto;}",
        ),
    );

    for index in 0..4 {
        assert_auto(&result, index);
    }
}

// B. `auto` is never a repeated-item sentinel; it is always Invalid mixed
// with any feature, with or without a comma, in either order.

#[test]
fn auto_mixed_with_features_is_always_invalid() {
    let result = qualify(
        598001,
        concat!(
            "a{will-change:auto transform;}",
            "b{will-change:auto, transform;}",
            "c{will-change:transform, auto;}",
            "d{will-change:contents auto;}",
            "e{will-change:contents, auto;}",
            "f{will-change:auto, auto;}",
            "g{will-change:auto auto;}",
        ),
    );

    for index in 0..7 {
        assert_invalid(&result, index);
    }
}

// C. Dedicated `scroll-position` / `contents` keywords, ASCII case variants.

#[test]
fn dedicated_keywords_qualify_case_insensitively() {
    let result = qualify(
        598002,
        concat!(
            "a{will-change:scroll-position;}",
            "b{will-change:SCROLL-POSITION;}",
            "c{will-change:Scroll-Position;}",
            "d{will-change:contents;}",
            "e{will-change:CONTENTS;}",
            "f{will-change:Contents;}",
            "g{will-change:scroll-position, contents;}",
            "h{will-change:contents, scroll-position;}",
        ),
    );

    for index in 0..6 {
        assert_features_len(&result, index, 1);
    }
    assert_eq!(features_at(&result, 0), [TestFeature::ScrollPosition]);
    assert_eq!(features_at(&result, 3), [TestFeature::Contents]);
    assert_eq!(
        features_at(&result, 6),
        [TestFeature::ScrollPosition, TestFeature::Contents]
    );
    assert_eq!(
        features_at(&result, 7),
        [TestFeature::Contents, TestFeature::ScrollPosition]
    );
}

// D. Unknown property-shaped, ordinary property-shaped, and
// custom-property-shaped identifiers are all ordinary valid `CustomIdent`
// items -- this leaf never consults a built-in property registry.

#[test]
fn arbitrary_identifiers_qualify_as_custom_ident_without_any_property_registry_lookup() {
    let result = qualify(
        598003,
        concat!(
            "a{will-change:transform;}",
            "b{will-change:background-color;}",
            "c{will-change:Not-A-Property;}",
            "d{will-change:--var;}",
            "e{will-change:--Foo;}",
        ),
    );

    for index in 0..5 {
        assert_features_len(&result, index, 1);
    }
    assert_eq!(
        features_at(&result, 0),
        [TestFeature::CustomIdent(Some("transform"))]
    );
    assert_eq!(
        features_at(&result, 1),
        [TestFeature::CustomIdent(Some("background-color"))]
    );
    assert_eq!(
        features_at(&result, 2),
        [TestFeature::CustomIdent(Some("Not-A-Property"))]
    );
    assert_eq!(
        features_at(&result, 3),
        [TestFeature::CustomIdent(Some("--var"))]
    );
    assert_eq!(
        features_at(&result, 4),
        [TestFeature::CustomIdent(Some("--Foo"))]
    );
}

// E. Case preservation: no lowercasing, no canonicalization, no
// case-insensitive collapsing of distinct `CustomIdent` identities.

#[test]
fn custom_ident_case_is_preserved_and_never_canonicalized() {
    let result = qualify(
        598004,
        concat!(
            "a{will-change:transform;}",
            "b{will-change:TRANSFORM;}",
            "c{will-change:--Foo;}",
            "d{will-change:--foo;}",
        ),
    );

    assert_eq!(
        features_at(&result, 0),
        [TestFeature::CustomIdent(Some("transform"))]
    );
    assert_eq!(
        features_at(&result, 1),
        [TestFeature::CustomIdent(Some("TRANSFORM"))]
    );
    assert_ne!(features_at(&result, 0), features_at(&result, 1));
    assert_eq!(
        features_at(&result, 2),
        [TestFeature::CustomIdent(Some("--Foo"))]
    );
    assert_eq!(
        features_at(&result, 3),
        [TestFeature::CustomIdent(Some("--foo"))]
    );
    assert_ne!(features_at(&result, 2), features_at(&result, 3));
}

// F. Property-local `<custom-ident>` exclusions: `will-change`, `none`,
// `all`, `default` -- alone and embedded in a list.

#[test]
fn property_local_exclusions_are_invalid_alone_and_embedded() {
    let result = qualify(
        598005,
        concat!(
            "a{will-change:will-change;}",
            "b{will-change:none;}",
            "c{will-change:all;}",
            "d{will-change:default;}",
            "e{will-change:transform,will-change;}",
            "f{will-change:none,transform;}",
            "g{will-change:transform,all;}",
            "h{will-change:transform,default;}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// G. Ordered mixed lists, duplicate items, and case-differing duplicates
// are all preserved -- never deduplicated, sorted, or collapsed.

#[test]
fn ordered_mixed_lists_and_duplicates_are_preserved_exactly() {
    let result = qualify(
        598006,
        concat!(
            "a{will-change:transform, opacity;}",
            "b{will-change:opacity, transform;}",
            "c{will-change:transform, transform;}",
            "d{will-change:TRANSFORM, transform;}",
            "e{will-change:scroll-position, contents, transform;}",
            "f{will-change:Not-A-Property, transform;}",
            "g{will-change:transform, --var;}",
        ),
    );

    assert_eq!(
        features_at(&result, 0),
        [
            TestFeature::CustomIdent(Some("transform")),
            TestFeature::CustomIdent(Some("opacity")),
        ]
    );
    assert_eq!(
        features_at(&result, 1),
        [
            TestFeature::CustomIdent(Some("opacity")),
            TestFeature::CustomIdent(Some("transform")),
        ]
    );
    assert_ne!(features_at(&result, 0), features_at(&result, 1));
    assert_eq!(
        features_at(&result, 2),
        [
            TestFeature::CustomIdent(Some("transform")),
            TestFeature::CustomIdent(Some("transform")),
        ]
    );
    assert_eq!(
        features_at(&result, 3),
        [
            TestFeature::CustomIdent(Some("TRANSFORM")),
            TestFeature::CustomIdent(Some("transform")),
        ]
    );
    assert_ne!(features_at(&result, 3)[0], features_at(&result, 3)[1]);
    assert_eq!(
        features_at(&result, 4),
        [
            TestFeature::ScrollPosition,
            TestFeature::Contents,
            TestFeature::CustomIdent(Some("transform")),
        ]
    );
    assert_eq!(
        features_at(&result, 5),
        [
            TestFeature::CustomIdent(Some("Not-A-Property")),
            TestFeature::CustomIdent(Some("transform")),
        ]
    );
    assert_eq!(
        features_at(&result, 6),
        [
            TestFeature::CustomIdent(Some("transform")),
            TestFeature::CustomIdent(Some("--var")),
        ]
    );
}

// G2. Evidence-locator integrity: each `CustomIdent` evidence reference
// points at the exact retained `Ident` token, never at trivia, a comma, a
// neighboring item, or a copied payload.

#[test]
fn custom_ident_evidence_refs_point_to_the_exact_retained_ident_tokens() {
    let result = qualify(598025, "a{will-change:--Foo,scroll-position,--Foo;}");

    assert_features_len(&result, 0, 3);
    let items = match outcome_at(&result, 0) {
        CssWillChangeQualificationOutcome::Qualified(CssWillChangeValue::Features(items)) => items,
        other => panic!("expected Qualified(Features(_)), got {other:?}"),
    };

    let mut custom_ident_indices = Vec::new();
    for item in items {
        if let CssWillChangeItemValue::CustomIdent(evidence) = item {
            let lexical_index = evidence.lexical_item_index();
            let lexical_item = &result
                .upstream_parser_result()
                .upstream_tokenizer_result()
                .lexical_items()[lexical_index];
            let CssLexicalItem::SemanticToken(token) = lexical_item else {
                panic!("will-change CustomIdent evidence ref pointed to trivia");
            };
            assert!(
                matches!(token.kind(), CssTokenKind::Ident(_)),
                "will-change CustomIdent evidence ref pointed to unexpected token kind: {:?}",
                token.kind()
            );
            custom_ident_indices.push(lexical_index);
        }
    }
    assert_eq!(custom_ident_indices.len(), 2);
    assert!(custom_ident_indices[0] < custom_ident_indices[1]);
}

// H. CSS-wide keyword: Unsupported only as the entire value; embedded in
// list position it is Invalid.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        598007,
        concat!(
            "a{will-change:initial;}",
            "b{will-change:inherit;}",
            "c{will-change:unset;}",
            "d{will-change:revert;}",
            "e{will-change:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssWillChangeUnsupportedReason::CssWideKeyword,
        );
    }

    let embedded = qualify(
        598008,
        concat!(
            "a{will-change:transform,initial;}",
            "b{will-change:initial,transform;}",
            "c{will-change:transform,inherit;}",
        ),
    );
    for index in 0..3 {
        assert_invalid(&embedded, index);
    }
}

// I. Ordinary Function items are decisively Invalid, never softened to
// generic Unsupported.

#[test]
fn ordinary_function_items_are_invalid_never_softened_to_unsupported() {
    let result = qualify(
        598009,
        concat!(
            "a{will-change:foo();}",
            "b{will-change:calc(1);}",
            "c{will-change:transform,bar();}",
            "d{will-change:bar(),transform;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// J. A recognized generic whole-value Function is Unsupported only when it
// occupies the entire value; embedded in a list it is Invalid, never
// upgraded to whole-value Unsupported.

#[test]
fn whole_value_generic_function_boundary_does_not_legalize_function_items() {
    let whole = qualify(598010, "a{will-change:first-valid(transform, opacity);}");
    assert_unsupported(
        &whole,
        0,
        CssWillChangeUnsupportedReason::WholeValueFunction,
    );

    let embedded = qualify(
        598011,
        concat!(
            "a{will-change:transform,first-valid(opacity);}",
            "b{will-change:first-valid(transform),opacity;}",
        ),
    );
    for index in 0..2 {
        assert_invalid(&embedded, index);
    }
}

// K. Deferred substitution remains Unsupported wherever it occurs, and
// nested commas inside its arguments never become outer separators.

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    let result = qualify(
        598012,
        concat!(
            "a{will-change:var(--x);}",
            "b{will-change:transform,var(--x);}",
            "c{will-change:var(--x),transform;}",
            "d{will-change:var(--x,--y,--z);}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(
            &result,
            index,
            CssWillChangeUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// L. Nested commas inside an ordinary (non-deferred) Function do not
// become outer top-level separators either.

#[test]
fn nested_ordinary_function_commas_do_not_split_as_outer_separators() {
    let result = qualify(
        598013,
        concat!(
            "a{will-change:foo(transform,opacity),contents;}",
            "b{will-change:(transform,opacity),contents;}",
            "c{will-change:transform),opacity;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// M. Leading comma, trailing comma, doubled/empty comma-list items.

#[test]
fn empty_and_malformed_comma_list_items_are_invalid() {
    let result = qualify(
        598014,
        concat!(
            "a{will-change:;}",
            "b{will-change:,transform;}",
            "c{will-change:transform,;}",
            "d{will-change:transform,,opacity;}",
            "e{will-change:transform opacity;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// N. Non-Ident token classes are invalid: String, Number, Dimension,
// Percentage, Hash, and bracket/block-shaped constructs.

#[test]
fn non_ident_token_classes_are_invalid() {
    let result = qualify(
        598015,
        concat!(
            "a{will-change:\"transform\";}",
            "b{will-change:100;}",
            "c{will-change:100px;}",
            "d{will-change:100%;}",
            "e{will-change:#fff;}",
            "f{will-change:[transform];}",
            "g{will-change:{transform};}",
        ),
    );

    for index in 0..7 {
        assert_invalid(&result, index);
    }
}

// O. `!important` priority is outside the semantic value window.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(598016, "a{will-change:transform !important;}");
    assert_features_len(&result, 0, 1);
    assert_eq!(
        features_at(&result, 0),
        [TestFeature::CustomIdent(Some("transform"))]
    );
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// P. Duplicate declarations remain distinct observations in authored order,
// without any cascade winner selection.

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(
        598017,
        "a{will-change:transform;will-change:auto;will-change:opacity,opacity;}",
    );

    assert_eq!(result.will_change_observations().len(), 3);
    assert_eq!(
        features_at(&result, 0),
        [TestFeature::CustomIdent(Some("transform"))]
    );
    assert_auto(&result, 1);
    assert_eq!(
        features_at(&result, 2),
        [
            TestFeature::CustomIdent(Some("opacity")),
            TestFeature::CustomIdent(Some("opacity")),
        ]
    );
}

// Q. Ordinary placement contract: nonordinary contexts never enter
// `will-change` dispatch.

#[test]
fn nonordinary_contexts_do_not_enter_will_change_dispatch() {
    for (source_id, css) in [
        (598018, "@font-face{will-change:transform;}"),
        (598019, "@page{will-change:transform;}"),
        (598020, "@keyframes k{from{will-change:transform;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.will_change_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

// R. Cross-dispatch isolation against accepted neighboring leaves.

#[test]
fn duplicate_occurrences_and_existing_leaf_dispatch_remain_separate() {
    let result = qualify(
        598021,
        concat!(
            "a{will-change:transform;will-change:auto;}",
            "b{anchor-name:--foo;}",
            "c{container-name:foo;}",
            "d{color-scheme:light;}",
            "e{animation-name:foo;}",
            "f{transition-property:width;}",
        ),
    );

    assert_eq!(result.will_change_observations().len(), 2);
    assert_eq!(result.will_change_observations()[0].occurrence_index(), 0);
    assert_eq!(result.will_change_observations()[1].occurrence_index(), 1);
    assert_eq!(
        features_at(&result, 0),
        [TestFeature::CustomIdent(Some("transform"))]
    );
    assert_auto(&result, 1);
    assert_eq!(result.anchor_name_observations().len(), 1);
    assert_eq!(result.container_name_observations().len(), 1);
    assert_eq!(result.color_scheme_observations().len(), 1);
    assert_eq!(result.animation_name_observations().len(), 1);
    assert_eq!(result.transition_property_observations().len(), 1);
}

// S. Resource / committed prefix, and repeated/cross-source determinism.

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    let incomplete = qualify_with_limits(
        598022,
        "a{will-change:transform;}b{will-change:transform,opacity;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.will_change_observations().len(), 1);
    assert_eq!(
        features_at(&incomplete, 0),
        [TestFeature::CustomIdent(Some("transform"))]
    );

    let css = concat!(
        "a{will-change:transform,opacity;}",
        "b{will-change:auto;}",
        "c{will-change:var(--x);}",
        "d{will-change:auto,transform;}",
    );
    let first = qualify(598023, css);
    let repeated = qualify(598023, css);
    let another_source = qualify(598024, css);

    assert_eq!(
        first.will_change_observations(),
        repeated.will_change_observations()
    );
    assert_eq!(
        first.will_change_observations(),
        another_source.will_change_observations()
    );
}
