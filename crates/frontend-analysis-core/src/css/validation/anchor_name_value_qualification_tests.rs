use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssAnchorNameQualificationOutcome, CssAnchorNameUnsupportedReason, CssAnchorNameValue,
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
) -> &CssAnchorNameQualificationOutcome {
    result.anchor_name_observations()[index].outcome()
}

fn assert_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssAnchorNameQualificationOutcome::Qualified(CssAnchorNameValue::None),
        "expected whole-value None sentinel at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssAnchorNameQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssAnchorNameUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssAnchorNameQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified `<dashed-ident>` items to
/// their tokenizer-owned decoded interpreted identity. Panics if the
/// observation at `index` is not `Qualified(Names(_))`.
fn dashed_ident_values(result: &CssValueQualificationRunResult, index: usize) -> Vec<Option<&str>> {
    match outcome_at(result, index) {
        CssAnchorNameQualificationOutcome::Qualified(CssAnchorNameValue::Names(names)) => names
            .iter()
            .map(|evidence| result.anchor_name_dashed_ident_value(*evidence))
            .collect(),
        other => panic!("expected Qualified(Names(_)) at index {index}, got {other:?}"),
    }
}

/// Resolves one observation's ordered qualified items' absolute lexical-item
/// indices, verifying each retained evidence position is a direct
/// `Ident` token -- never trivia, a comma, or a neighboring item.
fn dashed_ident_indices(result: &CssValueQualificationRunResult, index: usize) -> Vec<usize> {
    match outcome_at(result, index) {
        CssAnchorNameQualificationOutcome::Qualified(CssAnchorNameValue::Names(names)) => names
            .iter()
            .map(|evidence| {
                let lexical_index = evidence.lexical_item_index();
                let item = &result
                    .upstream_parser_result()
                    .upstream_tokenizer_result()
                    .lexical_items()[lexical_index];
                let CssLexicalItem::SemanticToken(token) = item else {
                    panic!("anchor-name dashed-ident evidence ref pointed to trivia");
                };
                assert!(
                    matches!(token.kind(), CssTokenKind::Ident(_)),
                    "anchor-name dashed-ident evidence ref pointed to unexpected token kind: {:?}",
                    token.kind()
                );
                lexical_index
            })
            .collect(),
        other => panic!("expected Qualified(Names(_)) at index {index}, got {other:?}"),
    }
}

fn assert_names_len(result: &CssValueQualificationRunResult, index: usize, expected_len: usize) {
    match outcome_at(result, index) {
        CssAnchorNameQualificationOutcome::Qualified(CssAnchorNameValue::Names(names)) => {
            assert_eq!(names.len(), expected_len, "at index {index}");
        }
        other => panic!("expected Qualified(Names(_)) at index {index}, got {other:?}"),
    }
}

// A. Whole `none`.

#[test]
fn whole_none_qualifies_ascii_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        99000,
        concat!(
            "a{anchor-name:none;}",
            "b{anchor-name:NONE;}",
            "c{anchor-name:NoNe;}",
            "d{anchor-name:\\6e one;}",
        ),
    );

    for index in 0..4 {
        assert_none(&result, index);
    }
}

// B. `none` is invalid inside the list branch.

#[test]
fn none_is_invalid_in_list_item_position() {
    let result = qualify(
        99001,
        concat!(
            "a{anchor-name:none,--foo;}",
            "b{anchor-name:--foo,none;}",
            "c{anchor-name:none,none;}",
            "d{anchor-name:--foo,none,--bar;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// C. `--none` is not `none`.

#[test]
fn dashed_none_is_an_ordinary_dashed_ident_distinct_from_the_none_sentinel() {
    let result = qualify(99002, "a{anchor-name:none;}b{anchor-name:--none;}");

    assert_none(&result, 0);
    assert_names_len(&result, 1, 1);
    assert_eq!(dashed_ident_values(&result, 1), [Some("--none")]);
}

// D. Direct dashed identities.

#[test]
fn direct_dashed_identities_qualify_case_sensitively_with_exact_order_and_multiplicity() {
    let result = qualify(
        99003,
        concat!(
            "a{anchor-name:--foo;}",
            "b{anchor-name:--Foo;}",
            "c{anchor-name:--FOO;}",
            "d{anchor-name:--default;}",
            "e{anchor-name:--initial;}",
            "f{anchor-name:--foo,--bar;}",
            "g{anchor-name:--foo,--bar,--foo;}",
        ),
    );

    assert_eq!(dashed_ident_values(&result, 0), [Some("--foo")]);
    assert_eq!(dashed_ident_values(&result, 1), [Some("--Foo")]);
    assert_eq!(dashed_ident_values(&result, 2), [Some("--FOO")]);
    assert_ne!(
        dashed_ident_values(&result, 0),
        dashed_ident_values(&result, 1)
    );
    assert_ne!(
        dashed_ident_values(&result, 1),
        dashed_ident_values(&result, 2)
    );
    assert_eq!(dashed_ident_values(&result, 3), [Some("--default")]);
    assert_eq!(dashed_ident_values(&result, 4), [Some("--initial")]);
    assert_eq!(
        dashed_ident_values(&result, 5),
        [Some("--foo"), Some("--bar")]
    );
    assert_eq!(
        dashed_ident_values(&result, 6),
        [Some("--foo"), Some("--bar"), Some("--foo")]
    );
}

// E. Escape-authored prefix decodes to a leading `--`.

#[test]
fn escape_authored_spelling_that_decodes_to_a_leading_dash_pair_qualifies() {
    // Neither `\2d` escape literally spells `-` in the authored source; the
    // qualification decision must come from the tokenizer-decoded `Ident`
    // identity, never a raw-source `starts_with("--")` scan.
    let result = qualify(99004, "a{anchor-name:\\2d\\2d foo;}");

    assert_names_len(&result, 0, 1);
    assert_eq!(dashed_ident_values(&result, 0), [Some("--foo")]);
    let _ = dashed_ident_indices(&result, 0);
}

// F. Non-dashed idents are invalid.

#[test]
fn non_dashed_idents_are_invalid_never_falling_back_to_custom_ident() {
    let result = qualify(
        99005,
        concat!(
            "a{anchor-name:foo;}",
            "b{anchor-name:foo-bar;}",
            "c{anchor-name:-foo;}",
            "d{anchor-name:auto;}",
            "e{anchor-name:normal;}",
            "f{anchor-name:default;}",
        ),
    );

    for index in 0..6 {
        assert_invalid(&result, index);
    }
}

// G. Token-class distinctions.

#[test]
fn wrong_token_classes_are_invalid_including_a_string_spelled_like_a_dashed_ident() {
    let result = qualify(
        99006,
        concat!(
            "a{anchor-name:\"--foo\";}",
            "b{anchor-name:100;}",
            "c{anchor-name:100px;}",
            "d{anchor-name:100%;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }

    // A literal newline before the closing quote forces the tokenizer to
    // retain `CssTokenKind::BadString` rather than `String`; `BadString` is
    // never an `Ident` and must not be conflated with one.
    let bad_string = qualify(99007, "a{anchor-name:\"--foo\n;}");
    assert_invalid(&bad_string, 0);
}

// H. Ordered evidence, no dedupe.

#[test]
fn ordered_evidence_refs_point_to_the_exact_retained_ident_tokens() {
    let result = qualify(99008, "a{anchor-name:--Foo,--bar,--Foo;}");

    assert_names_len(&result, 0, 3);
    assert_eq!(
        dashed_ident_values(&result, 0),
        [Some("--Foo"), Some("--bar"), Some("--Foo")]
    );
    let indices = dashed_ident_indices(&result, 0);
    assert_eq!(indices.len(), 3);
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
}

// I. Trivia indexing.

#[test]
fn trivia_around_separators_does_not_shift_dashed_ident_evidence() {
    let result = qualify(
        99009,
        concat!(
            "a{anchor-name:\n",
            "    /*a*/ --Foo /*b*/,\n",
            "    /*c*/ --bar /*d*/,\n",
            "    /*e*/ --Foo;}",
        ),
    );

    assert_names_len(&result, 0, 3);
    assert_eq!(
        dashed_ident_values(&result, 0),
        [Some("--Foo"), Some("--bar"), Some("--Foo")]
    );
    let indices = dashed_ident_indices(&result, 0);
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
}

// J. Structural invalidity.

#[test]
fn structural_separator_and_missing_comma_cases_are_invalid() {
    let result = qualify(
        99010,
        concat!(
            "a{anchor-name:;}",
            "b{anchor-name:,--foo;}",
            "c{anchor-name:--foo,;}",
            "d{anchor-name:--foo,,--bar;}",
            "e{anchor-name:--foo --bar;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// K. CSS-wide keyword boundary.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        99011,
        concat!(
            "a{anchor-name:initial;}",
            "b{anchor-name:inherit;}",
            "c{anchor-name:unset;}",
            "d{anchor-name:revert;}",
            "e{anchor-name:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssAnchorNameUnsupportedReason::CssWideKeyword,
        );
    }

    let list_placement = qualify(
        99012,
        concat!(
            "a{anchor-name:--foo,initial;}",
            "b{anchor-name:initial,--foo;}",
            "c{anchor-name:--foo,inherit;}",
        ),
    );
    for index in 0..3 {
        assert_invalid(&list_placement, index);
    }
}

// L. Functions.

#[test]
fn ordinary_function_items_are_invalid_never_softened_to_unsupported() {
    let result = qualify(
        99013,
        concat!(
            "a{anchor-name:foo();}",
            "b{anchor-name:calc(1);}",
            "c{anchor-name:--foo,bar();}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

#[test]
fn whole_value_generic_function_boundary_does_not_legalize_function_items() {
    let whole = qualify(99014, "a{anchor-name:first-valid(--foo, --bar);}");
    assert_unsupported(
        &whole,
        0,
        CssAnchorNameUnsupportedReason::WholeValueFunction,
    );

    let items = qualify(
        99015,
        concat!(
            "a{anchor-name:--foo,first-valid(--bar);}",
            "b{anchor-name:first-valid(--foo),--bar;}",
        ),
    );
    for index in 0..2 {
        assert_invalid(&items, index);
    }
}

// M. Deferred substitution.

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    let result = qualify(
        99016,
        concat!(
            "a{anchor-name:var(--x);}",
            "b{anchor-name:--foo,var(--x);}",
            "c{anchor-name:var(--x),--foo;}",
            "d{anchor-name:var(--x,--foo,--bar);}",
        ),
    );

    for index in 0..4 {
        assert_unsupported(
            &result,
            index,
            CssAnchorNameUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// N. Comments / important / occurrences.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(99017, "a{anchor-name:--foo !important;}");
    assert_names_len(&result, 0, 1);
    assert_eq!(dashed_ident_values(&result, 0), [Some("--foo")]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(
        99018,
        "a{anchor-name:--a;anchor-name:none;anchor-name:--b,--b;}",
    );

    assert_eq!(result.anchor_name_observations().len(), 3);
    assert_eq!(dashed_ident_values(&result, 0), [Some("--a")]);
    assert_none(&result, 1);
    assert_eq!(dashed_ident_values(&result, 2), [Some("--b"), Some("--b")]);
}

// O. Cross-dispatch.

#[test]
fn duplicate_occurrences_and_existing_leaf_dispatch_remain_separate() {
    let result = qualify(
        99019,
        concat!(
            "a{anchor-name:--foo;anchor-name:--bar,--baz;}",
            "b{page:Foo;}",
            "c{transition-property:width;}",
            "d{hyphenate-character:\"-\";}",
            "e{animation-name:foo;}",
            "f{transition-duration:1s;}",
        ),
    );

    assert_eq!(result.anchor_name_observations().len(), 2);
    assert_eq!(result.anchor_name_observations()[0].occurrence_index(), 0);
    assert_eq!(result.anchor_name_observations()[1].occurrence_index(), 1);
    assert_eq!(dashed_ident_values(&result, 0), [Some("--foo")]);
    assert_eq!(
        dashed_ident_values(&result, 1),
        [Some("--bar"), Some("--baz")]
    );
    assert_eq!(result.page_observations().len(), 1);
    assert_eq!(result.transition_property_observations().len(), 1);
    assert_eq!(result.hyphenate_character_observations().len(), 1);
    assert_eq!(result.animation_name_observations().len(), 1);
    assert_eq!(result.transition_duration_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_anchor_name_dispatch() {
    for (source_id, css) in [
        (99020, "@font-face{anchor-name:--foo;}"),
        (99021, "@page{anchor-name:--foo;}"),
        (99022, "@keyframes k{from{anchor-name:--foo;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.anchor_name_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    let result = qualify(
        99023,
        concat!(
            "a{anchor-name:foo(bar,baz),--qux;}",
            "b{anchor-name:(--foo,--bar),--qux;}",
            "c{anchor-name:--foo),--bar;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// P. Resource / committed prefix.

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    let incomplete = qualify_with_limits(
        99024,
        "a{anchor-name:--foo;}b{anchor-name:--foo,--bar;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.anchor_name_observations().len(), 1);
    assert_eq!(dashed_ident_values(&incomplete, 0), [Some("--foo")]);

    let css = concat!(
        "a{anchor-name:--foo,--bar;}",
        "b{anchor-name:none;}",
        "c{anchor-name:var(--x);}",
        "d{anchor-name:none,--foo;}",
    );
    let first = qualify(99025, css);
    let repeated = qualify(99025, css);
    let another_source = qualify(99026, css);

    assert_eq!(
        first.anchor_name_observations(),
        repeated.anchor_name_observations()
    );
    assert_eq!(
        first.anchor_name_observations(),
        another_source.anchor_name_observations()
    );
}
