use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssNumberSign, CssNumberType, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssCounterIncrementItem, CssCounterIncrementNameEvidenceRef,
    CssCounterIncrementQualificationOutcome, CssCounterIncrementUnsupportedReason,
    CssCounterIncrementValue, CssValueQualificationRunResult, run,
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
) -> &CssCounterIncrementQualificationOutcome {
    result.counter_increment_observations()[index].outcome()
}

fn assert_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterIncrementQualificationOutcome::Qualified(CssCounterIncrementValue::None),
        "expected whole-value None sentinel at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterIncrementQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssCounterIncrementUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterIncrementQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified `DirectCounterItem`s.
/// Panics if the observation at `index` is not `Qualified(Items(_))`.
fn qualified_items(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &[CssCounterIncrementItem] {
    match outcome_at(result, index) {
        CssCounterIncrementQualificationOutcome::Qualified(CssCounterIncrementValue::Items(
            items,
        )) => items,
        other => panic!("expected Qualified(Items(_)) at index {index}, got {other:?}"),
    }
}

/// Resolves one observation's ordered qualified items to their
/// tokenizer-owned decoded `<counter-name>` identity.
fn item_names(result: &CssValueQualificationRunResult, index: usize) -> Vec<Option<&str>> {
    qualified_items(result, index)
        .iter()
        .map(|item| result.counter_increment_name_value(item.name()))
        .collect()
}

/// Resolves one observation's ordered qualified items to whether each has an
/// authored explicit integer -- `false` means authored-absent, never a
/// synthesized default `1`.
fn item_explicit_integer_presence(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> Vec<bool> {
    qualified_items(result, index)
        .iter()
        .map(|item| item.explicit_integer().is_some())
        .collect()
}

fn assert_items_len(result: &CssValueQualificationRunResult, index: usize, expected_len: usize) {
    assert_eq!(
        qualified_items(result, index).len(),
        expected_len,
        "at index {index}"
    );
}

/// Resolves one item's explicit integer evidence to its exact retained
/// tokenizer sign/digit structure, without machine-integer conversion.
/// Panics if the item has no explicit integer.
fn explicit_integer_sign_and_digits(
    result: &CssValueQualificationRunResult,
    index: usize,
    item_index: usize,
) -> (Option<CssNumberSign>, &str) {
    let items = qualified_items(result, index);
    let evidence = items[item_index]
        .explicit_integer()
        .unwrap_or_else(|| panic!("item {item_index} at {index} has no explicit integer"));
    let token = result
        .counter_increment_integer_token(evidence)
        .unwrap_or_else(|| panic!("integer evidence at {index}/{item_index} did not resolve"));
    let CssTokenKind::Number { value, number_type } = token else {
        panic!("integer evidence at {index}/{item_index} did not resolve to a Number token");
    };
    assert_eq!(*number_type, CssNumberType::Integer);
    (value.sign(), value.decimal().integer_digits())
}

// A. Whole `none`.

#[test]
fn whole_none_qualifies_ascii_case_insensitively_and_escape_equivalent() {
    let result = qualify(
        592100,
        concat!(
            "a{counter-increment:none;}",
            "b{counter-increment:NONE;}",
            "c{counter-increment:NoNe;}",
            "d{counter-increment:\\6e one;}",
        ),
    );

    for index in 0..4 {
        assert_none(&result, index);
    }
}

// B. `none` is invalid in the repeated-item position.

#[test]
fn none_is_invalid_in_repeated_item_position() {
    let result = qualify(
        592101,
        concat!(
            "a{counter-increment:none foo;}",
            "b{counter-increment:foo none;}",
            "c{counter-increment:none 1;}",
            "d{counter-increment:none none;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// C. Direct structured repetition: order, duplicates, case sensitivity,
// omission-vs-explicit-integer grouping.

#[test]
fn direct_structured_items_qualify_with_exact_order_multiplicity_and_omission() {
    let result = qualify(
        592102,
        concat!(
            "a{counter-increment:chapter;}",
            "b{counter-increment:chapter 1;}",
            "c{counter-increment:chapter section;}",
            "d{counter-increment:chapter section 2;}",
            "e{counter-increment:chapter 1 section;}",
            "f{counter-increment:chapter 1 section 2;}",
            "g{counter-increment:a 1 b 2 c 3;}",
            "h{counter-increment:Foo foo FOO;}",
        ),
    );

    assert_items_len(&result, 0, 1);
    assert_eq!(item_names(&result, 0), [Some("chapter")]);
    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);

    assert_items_len(&result, 1, 1);
    assert_eq!(item_names(&result, 1), [Some("chapter")]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);

    assert_items_len(&result, 2, 2);
    assert_eq!(item_names(&result, 2), [Some("chapter"), Some("section")]);
    assert_eq!(item_explicit_integer_presence(&result, 2), [false, false]);

    assert_items_len(&result, 3, 2);
    assert_eq!(item_names(&result, 3), [Some("chapter"), Some("section")]);
    assert_eq!(item_explicit_integer_presence(&result, 3), [false, true]);

    assert_items_len(&result, 4, 2);
    assert_eq!(item_names(&result, 4), [Some("chapter"), Some("section")]);
    assert_eq!(item_explicit_integer_presence(&result, 4), [true, false]);

    assert_items_len(&result, 5, 2);
    assert_eq!(item_names(&result, 5), [Some("chapter"), Some("section")]);
    assert_eq!(item_explicit_integer_presence(&result, 5), [true, true]);

    assert_items_len(&result, 6, 3);
    assert_eq!(item_names(&result, 6), [Some("a"), Some("b"), Some("c")]);
    assert_eq!(
        item_explicit_integer_presence(&result, 6),
        [true, true, true]
    );

    assert_items_len(&result, 7, 3);
    assert_eq!(
        item_names(&result, 7),
        [Some("Foo"), Some("foo"), Some("FOO")]
    );
    let names = item_names(&result, 7);
    assert_ne!(names[0], names[1]);
    assert_ne!(names[1], names[2]);
    assert_eq!(
        item_explicit_integer_presence(&result, 7),
        [false, false, false]
    );
}

// C2. Repeated same-name items are preserved, never collapsed/merged/
// deduplicated -- there is no runtime same-counter-name combination here.

#[test]
fn repeated_same_name_items_are_preserved_without_collapsing_or_merging() {
    let result = qualify(592137, "a{counter-increment:foo 1 bar 2 foo -1;}");

    assert_items_len(&result, 0, 3);
    assert_eq!(
        item_names(&result, 0),
        [Some("foo"), Some("bar"), Some("foo")]
    );
    assert_eq!(
        item_explicit_integer_presence(&result, 0),
        [true, true, true]
    );
    assert_eq!(explicit_integer_sign_and_digits(&result, 0, 0), (None, "1"));
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 0, 2),
        (Some(CssNumberSign::Minus), "1")
    );
}

// D. Authored omission is never a synthesized explicit `1`.

#[test]
fn authored_omission_remains_distinct_from_explicit_one() {
    let result = qualify(
        592103,
        concat!("a{counter-increment:foo;}", "b{counter-increment:foo 1;}",),
    );

    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);
    let (sign, digits) = explicit_integer_sign_and_digits(&result, 1, 0);
    assert_eq!(sign, None);
    assert_eq!(digits, "1");
}

// E. Sign/zero direct integer evidence is preserved, never normalized.

#[test]
fn direct_integer_sign_and_zero_spelling_is_preserved_exactly() {
    let result = qualify(
        592104,
        concat!(
            "a{counter-increment:foo 0;}",
            "b{counter-increment:foo +0;}",
            "c{counter-increment:foo -0;}",
            "d{counter-increment:foo 1;}",
            "e{counter-increment:foo +1;}",
            "f{counter-increment:foo -1;}",
        ),
    );

    assert_eq!(explicit_integer_sign_and_digits(&result, 0, 0), (None, "0"));
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 1, 0),
        (Some(CssNumberSign::Plus), "0")
    );
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 2, 0),
        (Some(CssNumberSign::Minus), "0")
    );
    assert_eq!(explicit_integer_sign_and_digits(&result, 3, 0), (None, "1"));
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 4, 0),
        (Some(CssNumberSign::Plus), "1")
    );
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 5, 0),
        (Some(CssNumberSign::Minus), "1")
    );
}

// F. Very large direct integers qualify without machine-integer conversion.

#[test]
fn huge_direct_integers_qualify_without_machine_conversion() {
    let huge_positive = "123456789012345678901234567890123456789012345678901234567890";
    let huge_negative_digits = "987654321098765432109876543210987654321098765432109876543210";
    let css = format!(
        "a{{counter-increment:foo {huge_positive};}}b{{counter-increment:foo -{huge_negative_digits};}}"
    );
    let result = qualify(592105, &css);

    assert_eq!(
        explicit_integer_sign_and_digits(&result, 0, 0),
        (None, huge_positive)
    );
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 1, 0),
        (Some(CssNumberSign::Minus), huge_negative_digits)
    );
}

// G. Escape-authored counter name plus explicit direct integer.

#[test]
fn escape_authored_counter_name_qualifies_via_tokenizer_decoded_identity() {
    let result = qualify(592106, "a{counter-increment:a\\ 8 9;}");

    assert_items_len(&result, 0, 1);
    assert_eq!(item_names(&result, 0), [Some("a 8")]);
    assert_eq!(item_explicit_integer_presence(&result, 0), [true]);
    let (sign, digits) = explicit_integer_sign_and_digits(&result, 0, 0);
    assert_eq!(sign, None);
    assert_eq!(digits, "9");
}

// H. Reserved identifiers: `none`/`default`/CSS-wide are excluded from
// `<counter-name>`; unrelated keywords remain eligible, unlike
// `container-name`'s `and`/`not`/`or` exclusions.

#[test]
fn reserved_identifiers_are_invalid_as_counter_names() {
    let result = qualify(
        592107,
        concat!(
            "a{counter-increment:default;}",
            "b{counter-increment:foo default;}",
        ),
    );
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

#[test]
fn unreserved_keywords_remain_valid_counter_names() {
    let result = qualify(
        592108,
        concat!(
            "a{counter-increment:auto;}",
            "b{counter-increment:normal;}",
            "c{counter-increment:and;}",
            "d{counter-increment:not;}",
            "e{counter-increment:or;}",
            "f{counter-increment:light;}",
            "g{counter-increment:dark;}",
        ),
    );

    for (index, expected) in ["auto", "normal", "and", "not", "or", "light", "dark"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(item_names(&result, index), [Some(expected)]);
    }
}

// I. Comma is invalid -- this grammar is `+`, not `#`.

#[test]
fn comma_separated_syntax_is_invalid_in_every_spacing_variant() {
    let result = qualify(
        592109,
        concat!(
            "a{counter-increment:foo, bar;}",
            "b{counter-increment:foo,bar;}",
            "c{counter-increment:foo,;}",
            "d{counter-increment:,foo;}",
            "e{counter-increment:foo,,bar;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// J. Wrong token classes.

#[test]
fn wrong_token_classes_are_invalid() {
    let result = qualify(
        592110,
        concat!(
            "a{counter-increment:\"foo\";}",
            "b{counter-increment:#foo;}",
            "c{counter-increment:1;}",
            "d{counter-increment:1 foo;}",
            "e{counter-increment:foo \"bar\";}",
            "f{counter-increment:foo 1px;}",
            "g{counter-increment:foo 50%;}",
            "h{counter-increment:foo url(x);}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// K. Fraction/exponent numbers are Number-type, not Integer-type: invalid.

#[test]
fn fraction_and_exponent_numbers_are_invalid_never_promoted_to_integer() {
    let result = qualify(
        592111,
        concat!(
            "a{counter-increment:foo 1.0;}",
            "b{counter-increment:foo 1e0;}",
            "c{counter-increment:foo +1e0;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// L. Extra/leading integer is decisive invalidity.

#[test]
fn extra_or_leading_integer_is_invalid() {
    let result = qualify(
        592112,
        concat!(
            "a{counter-increment:foo 1 2;}",
            "b{counter-increment:foo 1 2 bar;}",
            "c{counter-increment:foo 0 -1;}",
            "d{counter-increment:1 foo;}",
            "e{counter-increment:-1 foo;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// M. Empty value.

#[test]
fn empty_value_is_invalid() {
    let result = qualify(592113, "a{counter-increment:;}");
    assert_invalid(&result, 0);
}

// N. Function position matrix -- name-required position.

#[test]
fn function_in_required_name_position_is_invalid() {
    let result = qualify(
        592114,
        concat!(
            "a{counter-increment:foo();}",
            "b{counter-increment:calc(1);}",
            "c{counter-increment:min(1, 2);}",
            "d{counter-increment:foo 1 bar();}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// O. Decisive structural invalidity outranks a provisional Function
// integer-slot ambiguity.

#[test]
fn decisive_invalidity_after_function_integer_slot_outranks_ambiguity() {
    let result = qualify(592115, "a{counter-increment:foo calc(1) 2;}");
    assert_invalid(&result, 0);
}

// P. Embedded recognized whole-value-only function is invalid, never
// softened to whole-value Unsupported -- placement is load-bearing.

#[test]
fn embedded_whole_value_only_function_is_invalid() {
    let result = qualify(592116, "a{counter-increment:foo first-valid(bar);}");
    assert_invalid(&result, 0);
}

// Q. Function occupying a structurally feasible optional-integer slot is
// conservatively Unsupported, never Invalid and never evaluated.

#[test]
fn function_in_feasible_optional_integer_slot_is_unsupported() {
    let result = qualify(
        592117,
        concat!(
            "a{counter-increment:foo calc(1);}",
            "b{counter-increment:foo calc(-2.5);}",
            "c{counter-increment:foo min(1, 2);}",
            "d{counter-increment:foo arbitrary();}",
            "e{counter-increment:foo calc(1) bar;}",
            "f{counter-increment:foo calc(1) bar 2;}",
        ),
    );

    for index in 0..6 {
        assert_unsupported(
            &result,
            index,
            CssCounterIncrementUnsupportedReason::FunctionValuedIntegerSlot,
        );
    }
}

// R. Recognized generic whole-value Function only as the entire value.

#[test]
fn whole_value_function_is_unsupported_only_as_the_entire_value() {
    let result = qualify(592118, "a{counter-increment:first-valid(foo, bar);}");
    assert_unsupported(
        &result,
        0,
        CssCounterIncrementUnsupportedReason::WholeValueFunction,
    );
}

// S. Deferred substitution precedes direct repetition classification
// everywhere in the value.

#[test]
fn deferred_substitution_precedes_direct_repetition_classification_everywhere() {
    let result = qualify(
        592119,
        concat!(
            "a{counter-increment:var(--x);}",
            "b{counter-increment:foo var(--x);}",
            "c{counter-increment:var(--x) 1;}",
            "d{counter-increment:foo 1 var(--x);}",
            "e{counter-increment:ident(\"foo\");}",
            "f{counter-increment:foo ident(\"bar\");}",
        ),
    );

    for index in 0..6 {
        assert_unsupported(
            &result,
            index,
            CssCounterIncrementUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// T. CSS-wide keyword boundary: sole value only, never embedded.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        592120,
        concat!(
            "a{counter-increment:initial;}",
            "b{counter-increment:inherit;}",
            "c{counter-increment:unset;}",
            "d{counter-increment:revert;}",
            "e{counter-increment:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssCounterIncrementUnsupportedReason::CssWideKeyword,
        );
    }

    let combined = qualify(
        592121,
        concat!(
            "a{counter-increment:foo initial;}",
            "b{counter-increment:initial foo;}",
            "c{counter-increment:foo 1 inherit;}",
            "d{counter-increment:inherit 1;}",
        ),
    );
    for index in 0..4 {
        assert_invalid(&combined, index);
    }
}

// U. Trivia: comments/whitespace never break or merge tuple grouping.

#[test]
fn comments_group_direct_tuples_identically_to_whitespace() {
    let result = qualify(
        592122,
        "a{counter-increment:foo/**/1/**/bar/**/2;}b{counter-increment:foo/**/bar;}",
    );

    assert_items_len(&result, 0, 2);
    assert_eq!(item_names(&result, 0), [Some("foo"), Some("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 0), [true, true]);

    assert_items_len(&result, 1, 2);
    assert_eq!(item_names(&result, 1), [Some("foo"), Some("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [false, false]);
}

#[test]
fn dense_whitespace_and_comment_variants_produce_identical_ordered_items() {
    let spaced = qualify(592123, "a{counter-increment:foo 1 bar 2;}");
    let multiline = qualify(
        592124,
        "a{counter-increment:\n    foo\n    1\n    bar\n    2;}",
    );
    let comment_padded = qualify(
        592125,
        "a{counter-increment:/*a*/foo/*b*/ /*c*/1/*d*/ /*e*/bar/*f*/ /*g*/2/*h*/;}",
    );

    for result in [&spaced, &multiline, &comment_padded] {
        assert_eq!(item_names(result, 0), [Some("foo"), Some("bar")]);
        assert_eq!(item_explicit_integer_presence(result, 0), [true, true]);
    }
}

// V. `!important` stays outside the semantic value window; duplicate
// declaration occurrences preserve authored order without cascade.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(592126, "a{counter-increment:foo 1 bar 2 !important;}");
    assert_items_len(&result, 0, 2);
    assert_eq!(item_names(&result, 0), [Some("foo"), Some("bar")]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

#[test]
fn repeated_declarations_preserve_authored_occurrence_order_without_cascade() {
    let result = qualify(
        592127,
        "a{counter-increment:foo;counter-increment:foo 1;counter-increment:bar 2;}",
    );

    assert_eq!(result.counter_increment_observations().len(), 3);
    assert_eq!(item_names(&result, 0), [Some("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);
    assert_eq!(item_names(&result, 1), [Some("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);
    assert_eq!(item_names(&result, 2), [Some("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 2), [true]);
}

// W. Cross-dispatch separation from other qualified leaves.

#[test]
fn cross_dispatch_separation_from_other_qualified_leaves() {
    let result = qualify(
        592128,
        concat!(
            "a{counter-increment:foo;counter-increment:bar 2;}",
            "b{container-name:foo;}",
            "c{color-scheme:light;}",
            "d{anchor-name:--foo;}",
            "e{animation-name:foo;}",
            "f{transition-property:width;}",
            "g{offset-rotate:auto;}",
            "h{order:1;}",
        ),
    );

    assert_eq!(result.counter_increment_observations().len(), 2);
    assert_eq!(
        result.counter_increment_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.counter_increment_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(item_names(&result, 0), [Some("foo")]);
    assert_eq!(item_names(&result, 1), [Some("bar")]);
    assert_eq!(result.container_name_observations().len(), 1);
    assert_eq!(result.color_scheme_observations().len(), 1);
    assert_eq!(result.anchor_name_observations().len(), 1);
    assert_eq!(result.animation_name_observations().len(), 1);
    assert_eq!(result.transition_property_observations().len(), 1);
    assert_eq!(result.offset_rotate_observations().len(), 1);
    assert_eq!(result.order_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_counter_increment_dispatch() {
    for (source_id, css) in [
        (592129, "@font-face{counter-increment:foo;}"),
        (592130, "@page{counter-increment:foo;}"),
        (592131, "@keyframes k{from{counter-increment:foo;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.counter_increment_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    let result = qualify(
        592132,
        concat!(
            "a{counter-increment:foo(bar baz) qux;}",
            "b{counter-increment:(foo bar) qux;}",
            "c{counter-increment:foo) bar;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// X. Resource / committed prefix / determinism.

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    let incomplete = qualify_with_limits(
        592133,
        "a{counter-increment:foo;}b{counter-increment:foo bar;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.counter_increment_observations().len(), 1);
    assert_eq!(item_names(&incomplete, 0), [Some("foo")]);

    let css = concat!(
        "a{counter-increment:foo 1 bar;}",
        "b{counter-increment:none;}",
        "c{counter-increment:var(--x);}",
        "d{counter-increment:none foo;}",
        "e{counter-increment:foo calc(1);}",
    );
    let first = qualify(592134, css);
    let repeated = qualify(592134, css);
    let another_source = qualify(592135, css);

    assert_eq!(
        first.counter_increment_observations(),
        repeated.counter_increment_observations()
    );
    assert_eq!(
        first.counter_increment_observations(),
        another_source.counter_increment_observations()
    );
}

// Y. Regression evidence-ref sanity: evidence never points at trivia.

#[test]
fn evidence_refs_resolve_to_direct_ident_and_integer_tokens_not_trivia() {
    let result = qualify(
        592136,
        "a{counter-increment:\n    /*a*/ foo /*b*/\n    /*c*/ 1 /*d*/\n    /*e*/ bar;}",
    );

    let items = qualified_items(&result, 0);
    assert_eq!(items.len(), 2);

    let name_token = |evidence: CssCounterIncrementNameEvidenceRef| {
        &result
            .upstream_parser_result()
            .upstream_tokenizer_result()
            .lexical_items()[evidence.lexical_item_index()]
    };

    for item in items {
        let CssLexicalItem::SemanticToken(token) = name_token(item.name()) else {
            panic!("counter-increment name evidence pointed to trivia");
        };
        assert!(matches!(token.kind(), CssTokenKind::Ident(_)));
    }

    let explicit = items[0]
        .explicit_integer()
        .expect("first item has explicit integer");
    let integer_token = result
        .counter_increment_integer_token(explicit)
        .expect("integer evidence resolves");
    assert!(matches!(
        integer_token,
        CssTokenKind::Number {
            number_type: CssNumberType::Integer,
            ..
        }
    ));
    assert!(items[1].explicit_integer().is_none());
}
