use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssContainerNameQualificationOutcome, CssContainerNameUnsupportedReason, CssContainerNameValue,
    CssValueQualificationRunResult, run,
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
) -> &CssContainerNameQualificationOutcome {
    result.container_name_observations()[index].outcome()
}

fn assert_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssContainerNameQualificationOutcome::Qualified(CssContainerNameValue::None),
        "expected whole-value None sentinel at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssContainerNameQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssContainerNameUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssContainerNameQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified `<custom-ident>` items to
/// their tokenizer-owned decoded interpreted identity. Panics if the
/// observation at `index` is not `Qualified(Names(_))`.
fn custom_ident_values(result: &CssValueQualificationRunResult, index: usize) -> Vec<Option<&str>> {
    match outcome_at(result, index) {
        CssContainerNameQualificationOutcome::Qualified(CssContainerNameValue::Names(names)) => {
            names
                .iter()
                .map(|evidence| result.container_name_custom_ident_value(*evidence))
                .collect()
        }
        other => panic!("expected Qualified(Names(_)) at index {index}, got {other:?}"),
    }
}

/// Resolves one observation's ordered qualified items' absolute lexical-item
/// indices, verifying each retained evidence position is a direct `Ident`
/// token -- never trivia, a comma, or a neighboring item.
fn custom_ident_indices(result: &CssValueQualificationRunResult, index: usize) -> Vec<usize> {
    match outcome_at(result, index) {
        CssContainerNameQualificationOutcome::Qualified(CssContainerNameValue::Names(names)) => {
            names
                .iter()
                .map(|evidence| {
                    let lexical_index = evidence.lexical_item_index();
                    let item = &result
                        .upstream_parser_result()
                        .upstream_tokenizer_result()
                        .lexical_items()[lexical_index];
                    let CssLexicalItem::SemanticToken(token) = item else {
                        panic!("container-name custom-ident evidence ref pointed to trivia");
                    };
                    assert!(
                        matches!(token.kind(), CssTokenKind::Ident(_)),
                        "container-name custom-ident evidence ref pointed to unexpected token kind: {:?}",
                        token.kind()
                    );
                    lexical_index
                })
                .collect()
        }
        other => panic!("expected Qualified(Names(_)) at index {index}, got {other:?}"),
    }
}

fn assert_names_len(result: &CssValueQualificationRunResult, index: usize, expected_len: usize) {
    match outcome_at(result, index) {
        CssContainerNameQualificationOutcome::Qualified(CssContainerNameValue::Names(names)) => {
            assert_eq!(names.len(), expected_len, "at index {index}");
        }
        other => panic!("expected Qualified(Names(_)) at index {index}, got {other:?}"),
    }
}

// A. Whole `none`.

#[test]
fn whole_none_qualifies_ascii_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        99100,
        concat!(
            "a{container-name:none;}",
            "b{container-name:NONE;}",
            "c{container-name:NoNe;}",
            "d{container-name:\\6e one;}",
        ),
    );

    for index in 0..4 {
        assert_none(&result, index);
        assert_names_len_zero_for_none(&result, index);
    }
}

fn assert_names_len_zero_for_none(result: &CssValueQualificationRunResult, index: usize) {
    match outcome_at(result, index) {
        CssContainerNameQualificationOutcome::Qualified(CssContainerNameValue::None) => {}
        other => panic!("expected Qualified(None) at index {index}, got {other:?}"),
    }
}

// B. `none` is invalid inside the repeated-name branch.

#[test]
fn none_is_invalid_in_repeated_name_position() {
    let result = qualify(
        99101,
        concat!(
            "a{container-name:none foo;}",
            "b{container-name:foo none;}",
            "c{container-name:none none;}",
            "d{container-name:foo none bar;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// C. Direct custom-ident names, order, duplicates, case sensitivity.

#[test]
fn direct_custom_ident_names_qualify_case_sensitively_with_exact_order_and_multiplicity() {
    let result = qualify(
        99102,
        concat!(
            "a{container-name:foo;}",
            "b{container-name:BAR;}",
            "c{container-name:foo bar;}",
            "d{container-name:foo bar baz;}",
            "e{container-name:foo foo;}",
            "f{container-name:Foo foo FOO;}",
            "g{container-name:auto;}",
            "h{container-name:normal;}",
            "i{container-name:auto normal;}",
        ),
    );

    assert_eq!(custom_ident_values(&result, 0), [Some("foo")]);
    assert_eq!(custom_ident_values(&result, 1), [Some("BAR")]);
    assert_eq!(custom_ident_values(&result, 2), [Some("foo"), Some("bar")]);
    assert_eq!(
        custom_ident_values(&result, 3),
        [Some("foo"), Some("bar"), Some("baz")]
    );
    assert_eq!(custom_ident_values(&result, 4), [Some("foo"), Some("foo")]);
    assert_eq!(
        custom_ident_values(&result, 5),
        [Some("Foo"), Some("foo"), Some("FOO")]
    );
    assert_ne!(
        custom_ident_values(&result, 5)[0],
        custom_ident_values(&result, 5)[1]
    );
    assert_ne!(
        custom_ident_values(&result, 5)[1],
        custom_ident_values(&result, 5)[2]
    );
    assert_eq!(custom_ident_values(&result, 6), [Some("auto")]);
    assert_eq!(custom_ident_values(&result, 7), [Some("normal")]);
    assert_eq!(
        custom_ident_values(&result, 8),
        [Some("auto"), Some("normal")]
    );
}

// D. Escape-authored spelling decodes through the tokenizer.

#[test]
fn escape_authored_identifier_qualifies_via_tokenizer_decoded_identity() {
    let result = qualify(99103, "a{container-name:\\!escaped;}");

    assert_names_len(&result, 0, 1);
    assert_eq!(custom_ident_values(&result, 0), [Some("!escaped")]);
    let _ = custom_ident_indices(&result, 0);
}

// E. Property-local reserved identifiers: and/not/or.

#[test]
fn and_not_or_are_invalid_in_any_ascii_case_and_any_position() {
    let result = qualify(
        99104,
        concat!(
            "a{container-name:and;}",
            "b{container-name:not;}",
            "c{container-name:or;}",
            "d{container-name:And;}",
            "e{container-name:NOT;}",
            "f{container-name:oR;}",
            "g{container-name:foo and;}",
            "h{container-name:not foo;}",
            "i{container-name:foo OR bar;}",
        ),
    );

    for index in 0..9 {
        assert_invalid(&result, index);
    }
}

// F. Generic reserved identifier: default.

#[test]
fn default_is_invalid_never_softened_to_css_wide_unsupported() {
    let result = qualify(
        99105,
        concat!(
            "a{container-name:default;}",
            "b{container-name:foo default;}",
        ),
    );

    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// G. auto/normal remain valid names, not globally reserved.

#[test]
fn auto_and_normal_are_not_accidentally_reserved() {
    let result = qualify(
        99106,
        concat!("a{container-name:auto;}", "b{container-name:normal;}",),
    );

    assert_eq!(custom_ident_values(&result, 0), [Some("auto")]);
    assert_eq!(custom_ident_values(&result, 1), [Some("normal")]);
}

// H. Comma is invalid -- this is `+`, not `#`.

#[test]
fn comma_separated_syntax_is_invalid_in_every_spacing_variant() {
    let result = qualify(
        99107,
        concat!(
            "a{container-name:foo, bar;}",
            "b{container-name:foo,bar;}",
            "c{container-name:foo,;}",
            "d{container-name:,foo;}",
            "e{container-name:foo,,bar;}",
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
        99108,
        concat!(
            "a{container-name:\"foo\";}",
            "b{container-name:\"none\";}",
            "c{container-name:\"auto\";}",
            "d{container-name:1;}",
            "e{container-name:1px;}",
            "f{container-name:50%;}",
            "g{container-name:#fff;}",
            "h{container-name:url(foo);}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// J. Empty value.

#[test]
fn empty_value_is_invalid() {
    let result = qualify(99109, "a{container-name:;}");
    assert_invalid(&result, 0);
}

// K. Ordinary Functions have no direct branch.

#[test]
fn ordinary_function_items_are_invalid_never_softened_to_function_value_unsupported() {
    let result = qualify(
        99110,
        concat!(
            "a{container-name:foo();}",
            "b{container-name:calc(1);}",
            "c{container-name:foo bar();}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// L. Generic whole-value Function placement.

#[test]
fn whole_value_generic_function_boundary_does_not_legalize_embedded_placement() {
    let whole = qualify(99111, "a{container-name:first-valid(foo, bar);}");
    assert_unsupported(
        &whole,
        0,
        CssContainerNameUnsupportedReason::WholeValueFunction,
    );

    let embedded = qualify(
        99112,
        concat!(
            "a{container-name:foo first-valid(bar);}",
            "b{container-name:first-valid(foo) bar;}",
        ),
    );
    for index in 0..2 {
        assert_invalid(&embedded, index);
    }
}

// M. Deferred substitution precedes direct repetition classification.

#[test]
fn deferred_substitution_precedes_direct_repetition_classification_everywhere() {
    let result = qualify(
        99113,
        concat!(
            "a{container-name:var(--x);}",
            "b{container-name:foo var(--x);}",
            "c{container-name:ident(\"foo\");}",
        ),
    );

    for index in 0..3 {
        assert_unsupported(
            &result,
            index,
            CssContainerNameUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// N. CSS-wide keyword boundary.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        99114,
        concat!(
            "a{container-name:initial;}",
            "b{container-name:inherit;}",
            "c{container-name:unset;}",
            "d{container-name:revert;}",
            "e{container-name:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssContainerNameUnsupportedReason::CssWideKeyword,
        );
    }

    let combined = qualify(
        99115,
        concat!(
            "a{container-name:foo initial;}",
            "b{container-name:initial foo;}",
            "c{container-name:foo inherit;}",
        ),
    );
    for index in 0..3 {
        assert_invalid(&combined, index);
    }
}

// O. Ordered evidence refs through trivia.

#[test]
fn ordered_evidence_refs_point_to_the_exact_retained_ident_tokens_through_trivia() {
    let result = qualify(
        99116,
        concat!(
            "a{container-name:\n",
            "    /*a*/ Foo /*b*/\n",
            "    /*c*/ foo /*d*/\n",
            "    /*e*/ Foo;}",
        ),
    );

    assert_names_len(&result, 0, 3);
    assert_eq!(
        custom_ident_values(&result, 0),
        [Some("Foo"), Some("foo"), Some("Foo")]
    );
    assert_ne!(
        custom_ident_values(&result, 0)[0],
        custom_ident_values(&result, 0)[1]
    );
    let indices = custom_ident_indices(&result, 0);
    assert_eq!(indices.len(), 3);
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn dense_whitespace_and_comment_variants_produce_identical_ordered_names() {
    let spaced = qualify(99117, "a{container-name:foo bar;}");
    let multiline = qualify(99118, "a{container-name:\n    foo\n    bar;}");
    let comment_adjacent = qualify(99119, "a{container-name:foo/*a*/bar;}");
    let comment_padded = qualify(99120, "a{container-name:/*a*/foo/*b*/ /*c*/bar/*d*/;}");

    for result in [&spaced, &multiline, &comment_adjacent, &comment_padded] {
        assert_eq!(custom_ident_values(result, 0), [Some("foo"), Some("bar")]);
    }
}

// P. `!important` and duplicate declaration occurrences.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(99121, "a{container-name:foo bar !important;}");
    assert_names_len(&result, 0, 2);
    assert_eq!(custom_ident_values(&result, 0), [Some("foo"), Some("bar")]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(
        99122,
        "a{container-name:foo;container-name:none;container-name:bar baz;}",
    );

    assert_eq!(result.container_name_observations().len(), 3);
    assert_eq!(custom_ident_values(&result, 0), [Some("foo")]);
    assert_none(&result, 1);
    assert_eq!(custom_ident_values(&result, 2), [Some("bar"), Some("baz")]);
}

// Q. Cross-dispatch separation.

#[test]
fn cross_dispatch_separation_from_other_qualified_leaves() {
    let result = qualify(
        99123,
        concat!(
            "a{container-name:foo;container-name:bar baz;}",
            "b{anchor-name:--foo;}",
            "c{animation-name:foo;}",
            "d{transition-property:width;}",
            "e{contain:layout;}",
            "f{offset-rotate:auto;}",
        ),
    );

    assert_eq!(result.container_name_observations().len(), 2);
    assert_eq!(
        result.container_name_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.container_name_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(custom_ident_values(&result, 0), [Some("foo")]);
    assert_eq!(custom_ident_values(&result, 1), [Some("bar"), Some("baz")]);
    assert_eq!(result.anchor_name_observations().len(), 1);
    assert_eq!(result.animation_name_observations().len(), 1);
    assert_eq!(result.transition_property_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(result.offset_rotate_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_container_name_dispatch() {
    for (source_id, css) in [
        (99124, "@font-face{container-name:foo;}"),
        (99125, "@page{container-name:foo;}"),
        (99126, "@keyframes k{from{container-name:foo;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.container_name_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_names() {
    let result = qualify(
        99127,
        concat!(
            "a{container-name:foo(bar baz) qux;}",
            "b{container-name:(foo bar) qux;}",
            "c{container-name:foo) bar;}",
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
        99128,
        "a{container-name:foo;}b{container-name:foo bar;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.container_name_observations().len(), 1);
    assert_eq!(custom_ident_values(&incomplete, 0), [Some("foo")]);

    let css = concat!(
        "a{container-name:foo bar;}",
        "b{container-name:none;}",
        "c{container-name:var(--x);}",
        "d{container-name:none foo;}",
    );
    let first = qualify(99129, css);
    let repeated = qualify(99129, css);
    let another_source = qualify(99130, css);

    assert_eq!(
        first.container_name_observations(),
        repeated.container_name_observations()
    );
    assert_eq!(
        first.container_name_observations(),
        another_source.container_name_observations()
    );
}
