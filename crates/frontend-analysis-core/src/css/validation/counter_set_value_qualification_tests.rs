use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssNumberSign, CssNumberType, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssCounterSetItem, CssCounterSetNameEvidenceRef, CssCounterSetQualificationOutcome,
    CssCounterSetUnsupportedReason, CssCounterSetValue, CssValueQualificationRunResult, run,
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
) -> &CssCounterSetQualificationOutcome {
    result.counter_set_observations()[index].outcome()
}

fn assert_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterSetQualificationOutcome::Qualified(CssCounterSetValue::None),
        "expected whole-value None sentinel at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterSetQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssCounterSetUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterSetQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified `DirectCounterItem`s.
/// Panics if the observation at `index` is not `Qualified(Items(_))`.
fn qualified_items(result: &CssValueQualificationRunResult, index: usize) -> &[CssCounterSetItem] {
    match outcome_at(result, index) {
        CssCounterSetQualificationOutcome::Qualified(CssCounterSetValue::Items(items)) => items,
        other => panic!("expected Qualified(Items(_)) at index {index}, got {other:?}"),
    }
}

/// Resolves one observation's ordered qualified items to their
/// tokenizer-owned decoded `<counter-name>` identity.
fn item_names(result: &CssValueQualificationRunResult, index: usize) -> Vec<Option<&str>> {
    qualified_items(result, index)
        .iter()
        .map(|item| result.counter_set_name_value(item.name()))
        .collect()
}

/// Resolves one observation's ordered qualified items to whether each has an
/// authored explicit integer -- `false` means authored-absent, never a
/// synthesized default `0`.
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
        .counter_set_integer_token(evidence)
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
        600100,
        concat!(
            "a{counter-set:none;}",
            "b{counter-set:NONE;}",
            "c{counter-set:NoNe;}",
            "d{counter-set:\\6e one;}",
        ),
    );

    for index in 0..4 {
        assert_none(&result, index);
    }
}

#[test]
fn none_is_invalid_in_repeated_item_position() {
    let result = qualify(
        600101,
        concat!(
            "a{counter-set:none foo;}",
            "b{counter-set:foo none;}",
            "c{counter-set:none 0;}",
            "d{counter-set:none none;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// B. Direct structured repetition: order, duplicates, case sensitivity,
// omission-vs-explicit-integer grouping.

#[test]
fn direct_structured_items_qualify_with_exact_order_multiplicity_and_omission() {
    let result = qualify(
        600102,
        concat!(
            "a{counter-set:foo;}",
            "b{counter-set:foo 0;}",
            "c{counter-set:foo -1;}",
            "d{counter-set:foo +1;}",
            "e{counter-set:foo bar;}",
            "f{counter-set:foo 1 bar;}",
            "g{counter-set:foo bar 2;}",
            "h{counter-set:foo 1 bar 2;}",
        ),
    );

    assert_items_len(&result, 0, 1);
    assert_eq!(item_names(&result, 0), [Some("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);

    assert_items_len(&result, 1, 1);
    assert_eq!(item_names(&result, 1), [Some("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);
    assert_eq!(explicit_integer_sign_and_digits(&result, 1, 0), (None, "0"));

    assert_items_len(&result, 2, 1);
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 2, 0),
        (Some(CssNumberSign::Minus), "1")
    );

    assert_items_len(&result, 3, 1);
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 3, 0),
        (Some(CssNumberSign::Plus), "1")
    );

    assert_items_len(&result, 4, 2);
    assert_eq!(item_names(&result, 4), [Some("foo"), Some("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 4), [false, false]);

    assert_items_len(&result, 5, 2);
    assert_eq!(item_names(&result, 5), [Some("foo"), Some("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 5), [true, false]);

    assert_items_len(&result, 6, 2);
    assert_eq!(item_names(&result, 6), [Some("foo"), Some("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 6), [false, true]);

    assert_items_len(&result, 7, 2);
    assert_eq!(item_names(&result, 7), [Some("foo"), Some("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 7), [true, true]);
}

// C. Repeated same-name items are preserved, never collapsed/merged/
// deduplicated -- there is no runtime same-counter-name combination here.

#[test]
fn repeated_same_name_items_are_preserved_without_collapsing_or_merging() {
    let result = qualify(600103, "a{counter-set:foo 1 bar 2 foo -1;}");

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

// D. Omission vs explicit zero -- the load-bearing authored-vs-interpreted
// boundary. `foo` must never become authored `foo 0`.

#[test]
fn authored_omission_remains_distinct_from_explicit_zero() {
    let result = qualify(
        600104,
        concat!("a{counter-set:foo;}", "b{counter-set:foo 0;}",),
    );

    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);
    let (sign, digits) = explicit_integer_sign_and_digits(&result, 1, 0);
    assert_eq!(sign, None);
    assert_eq!(digits, "0");
}

#[test]
fn explicit_plus_zero_and_minus_zero_are_retained_as_explicit_integer_evidence() {
    let result = qualify(
        600105,
        concat!("a{counter-set:foo +0;}", "b{counter-set:foo -0;}",),
    );

    assert_eq!(item_explicit_integer_presence(&result, 0), [true]);
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 0, 0),
        (Some(CssNumberSign::Plus), "0")
    );

    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 1, 0),
        (Some(CssNumberSign::Minus), "0")
    );
}

// E. Sign/zero direct integer evidence is preserved, never normalized.

#[test]
fn direct_integer_sign_and_zero_spelling_is_preserved_exactly() {
    let result = qualify(
        600106,
        concat!(
            "a{counter-set:foo 0;}",
            "b{counter-set:foo +0;}",
            "c{counter-set:foo -0;}",
            "d{counter-set:foo 1;}",
            "e{counter-set:foo +1;}",
            "f{counter-set:foo -1;}",
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
        "a{{counter-set:foo {huge_positive};}}b{{counter-set:foo -{huge_negative_digits};}}"
    );
    let result = qualify(600107, &css);

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
    let result = qualify(600108, "a{counter-set:a\\ 8 9;}");

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
        600109,
        concat!("a{counter-set:default;}", "b{counter-set:foo default;}",),
    );
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

#[test]
fn unreserved_keywords_remain_valid_counter_names() {
    let result = qualify(
        600110,
        concat!(
            "a{counter-set:auto;}",
            "b{counter-set:normal;}",
            "c{counter-set:and;}",
            "d{counter-set:not;}",
            "e{counter-set:or;}",
            "f{counter-set:light;}",
            "g{counter-set:dark;}",
        ),
    );

    for (index, expected) in ["auto", "normal", "and", "not", "or", "light", "dark"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(item_names(&result, index), [Some(expected)]);
    }
}

#[test]
fn case_distinct_names_are_never_normalized_or_deduplicated() {
    let result = qualify(600111, "a{counter-set:Foo foo FOO;}");

    assert_items_len(&result, 0, 3);
    let names = item_names(&result, 0);
    assert_eq!(names, [Some("Foo"), Some("foo"), Some("FOO")]);
    assert_ne!(names[0], names[1]);
    assert_ne!(names[1], names[2]);
    assert_eq!(
        item_explicit_integer_presence(&result, 0),
        [false, false, false]
    );
}

// I. `reversed()` is never transferred from `counter-reset`, at any
// placement -- required name position or a structurally feasible
// optional-integer position alike. Unlike a genuinely unresolved Function
// such as `calc(...)`, `reversed(...)` is a known CSS Lists grammar-native
// Function with no meaning anywhere in `counter-set`'s grammar, so it is
// decisively Invalid rather than an integer-slot ambiguity.

#[test]
fn reversed_function_is_invalid_at_every_placement() {
    let result = qualify(
        600112,
        concat!(
            "a{counter-set:reversed(foo);}",
            "b{counter-set:reversed(none);}",
            "c{counter-set:foo reversed(bar);}",
            "d{counter-set:foo 1 reversed(bar);}",
            "e{counter-set:reversed(foo) 1;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// I2. A genuine, unresolved Function-valued integer-slot ambiguity earlier
// in the value can never mask a later decisive `reversed(...)` invalidity
// -- decisive Invalid always outranks a provisional ambiguity, whichever
// component produced it.

#[test]
fn earlier_function_ambiguity_never_masks_a_later_reversed_invalidity() {
    let result = qualify(600144, "a{counter-set:foo calc(1) reversed(bar);}");
    assert_invalid(&result, 0);
}

// I3. Genuinely unresolved Functions in a feasible integer slot remain the
// bounded `FunctionValuedIntegerSlot` Unsupported outcome, unaffected by
// the `reversed()` boundary above.

#[test]
fn unresolved_functions_in_feasible_integer_slot_remain_unsupported() {
    let result = qualify(
        600145,
        concat!(
            "a{counter-set:foo calc(1);}",
            "b{counter-set:foo arbitrary();}",
        ),
    );
    for index in 0..2 {
        assert_unsupported(
            &result,
            index,
            CssCounterSetUnsupportedReason::FunctionValuedIntegerSlot,
        );
    }
}

// J. Comma is invalid -- this grammar is `+`, not `#`.

#[test]
fn comma_separated_syntax_is_invalid_in_every_spacing_variant() {
    let result = qualify(
        600113,
        concat!(
            "a{counter-set:foo, bar;}",
            "b{counter-set:foo,bar;}",
            "c{counter-set:foo,;}",
            "d{counter-set:,foo;}",
            "e{counter-set:foo,,bar;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// K. Wrong token classes.

#[test]
fn wrong_token_classes_are_invalid() {
    let result = qualify(
        600114,
        concat!(
            "a{counter-set:\"foo\";}",
            "b{counter-set:#foo;}",
            "c{counter-set:3;}",
            "d{counter-set:99 foo;}",
            "e{counter-set:foo \"bar\";}",
            "f{counter-set:foo 1px;}",
            "g{counter-set:foo 50%;}",
            "h{counter-set:foo url(x);}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// L. Fraction/exponent numbers are Number-type, not Integer-type: invalid.

#[test]
fn fraction_and_exponent_numbers_are_invalid_never_promoted_to_integer() {
    let result = qualify(
        600115,
        concat!(
            "a{counter-set:foo 3.14;}",
            "b{counter-set:3.14;}",
            "c{counter-set:foo 1e0;}",
            "d{counter-set:foo +1e0;}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// M. Extra/leading integer is decisive invalidity.

#[test]
fn extra_or_leading_integer_is_invalid() {
    let result = qualify(
        600116,
        concat!(
            "a{counter-set:foo 1 2;}",
            "b{counter-set:foo 1 2 bar;}",
            "c{counter-set:foo 0 -1;}",
            "d{counter-set:1 foo;}",
            "e{counter-set:-1 foo;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// N. Empty / trivia-only value.

#[test]
fn empty_and_trivia_only_values_are_invalid() {
    let result = qualify(
        600117,
        concat!("a{counter-set:;}", "b{counter-set: /**/  ;}",),
    );
    for index in 0..2 {
        assert_invalid(&result, index);
    }
}

// O. Function position matrix -- name-required position.

#[test]
fn function_in_required_name_position_is_invalid() {
    let result = qualify(
        600118,
        concat!(
            "a{counter-set:foo();}",
            "b{counter-set:calc(1);}",
            "c{counter-set:min(1, 2);}",
            "d{counter-set:foo 1 bar();}",
        ),
    );

    for index in 0..4 {
        assert_invalid(&result, index);
    }
}

// P. Decisive structural invalidity outranks a provisional Function
// integer-slot ambiguity.

#[test]
fn decisive_invalidity_after_function_integer_slot_outranks_ambiguity() {
    let result = qualify(600119, "a{counter-set:foo calc(1) 2;}");
    assert_invalid(&result, 0);
}

// Q. Embedded recognized whole-value-only function is invalid, never
// softened to whole-value Unsupported -- placement is load-bearing.

#[test]
fn embedded_whole_value_only_function_is_invalid() {
    let result = qualify(600120, "a{counter-set:foo first-valid(bar);}");
    assert_invalid(&result, 0);
}

// R. Function occupying a structurally feasible optional-integer slot is
// conservatively Unsupported, never Invalid and never evaluated.

#[test]
fn function_in_feasible_optional_integer_slot_is_unsupported() {
    let result = qualify(
        600121,
        concat!(
            "a{counter-set:foo calc(1);}",
            "b{counter-set:foo calc(-2.5);}",
            "c{counter-set:foo min(1, 2);}",
            "d{counter-set:foo arbitrary();}",
            "e{counter-set:foo calc(1) bar;}",
            "f{counter-set:foo calc(1) bar 2;}",
            "g{counter-set:section calc(10 + (5 * sign(2cqw - 10px)));}",
        ),
    );

    for index in 0..7 {
        assert_unsupported(
            &result,
            index,
            CssCounterSetUnsupportedReason::FunctionValuedIntegerSlot,
        );
    }
}

// S. Recognized generic whole-value Function only as the entire value; the
// nested comma inside it is never a top-level separator.

#[test]
fn whole_value_function_is_unsupported_only_as_the_entire_value() {
    let result = qualify(600122, "a{counter-set:first-valid(foo, bar);}");
    assert_unsupported(
        &result,
        0,
        CssCounterSetUnsupportedReason::WholeValueFunction,
    );
}

#[test]
fn nested_function_comma_is_never_a_top_level_repetition_separator() {
    let result = qualify(600123, "a{counter-set:foo calc(min(1, 2));}");
    assert_unsupported(
        &result,
        0,
        CssCounterSetUnsupportedReason::FunctionValuedIntegerSlot,
    );
}

// T. Deferred substitution precedes direct repetition classification
// everywhere in the value.

#[test]
fn deferred_substitution_precedes_direct_repetition_classification_everywhere() {
    let result = qualify(
        600124,
        concat!(
            "a{counter-set:var(--x);}",
            "b{counter-set:foo var(--x);}",
            "c{counter-set:var(--x) 1;}",
            "d{counter-set:foo 1 var(--x);}",
            "e{counter-set:ident(\"foo\");}",
            "f{counter-set:foo ident(\"bar\");}",
        ),
    );

    for index in 0..6 {
        assert_unsupported(
            &result,
            index,
            CssCounterSetUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// U. CSS-wide keyword boundary: sole value only, never embedded.

#[test]
fn css_wide_keyword_is_unsupported_only_as_the_entire_value() {
    let whole = qualify(
        600125,
        concat!(
            "a{counter-set:initial;}",
            "b{counter-set:inherit;}",
            "c{counter-set:unset;}",
            "d{counter-set:revert;}",
            "e{counter-set:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssCounterSetUnsupportedReason::CssWideKeyword,
        );
    }

    let combined = qualify(
        600126,
        concat!(
            "a{counter-set:foo initial;}",
            "b{counter-set:initial foo;}",
            "c{counter-set:foo 1 inherit;}",
            "d{counter-set:inherit 1;}",
        ),
    );
    for index in 0..4 {
        assert_invalid(&combined, index);
    }
}

// V. Trivia: comments/whitespace never break or merge tuple grouping.

#[test]
fn comments_group_direct_tuples_identically_to_whitespace() {
    let result = qualify(
        600127,
        "a{counter-set:foo/**/1/**/bar/**/2;}b{counter-set:foo/**/bar;}",
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
    let spaced = qualify(600128, "a{counter-set:foo 1 bar 2;}");
    let multiline = qualify(600129, "a{counter-set:\n    foo\n    1\n    bar\n    2;}");
    let comment_padded = qualify(
        600130,
        "a{counter-set:/*a*/foo/*b*/ /*c*/1/*d*/ /*e*/bar/*f*/ /*g*/2/*h*/;}",
    );

    for result in [&spaced, &multiline, &comment_padded] {
        assert_eq!(item_names(result, 0), [Some("foo"), Some("bar")]);
        assert_eq!(item_explicit_integer_presence(result, 0), [true, true]);
    }
}

// W. `!important` stays outside the semantic value window; duplicate
// declaration occurrences preserve authored order without cascade.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(600131, "a{counter-set:foo 1 bar 2 !important;}");
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
        600132,
        "a{counter-set:foo;counter-set:foo 0;counter-set:bar 2;}",
    );

    assert_eq!(result.counter_set_observations().len(), 3);
    assert_eq!(item_names(&result, 0), [Some("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);
    assert_eq!(item_names(&result, 1), [Some("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);
    assert_eq!(item_names(&result, 2), [Some("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 2), [true]);
}

// X. Cross-dispatch separation from other qualified leaves, including the
// two sibling counter properties.

#[test]
fn cross_dispatch_separation_from_other_qualified_leaves() {
    let result = qualify(
        600133,
        concat!(
            "a{counter-set:foo;counter-set:bar 2;}",
            "b{counter-increment:foo;}",
            "c{counter-reset:foo;}",
            "d{counter-reset:reversed(foo);}",
            "e{container-name:foo;}",
            "f{color-scheme:light;}",
            "g{will-change:transform;}",
        ),
    );

    assert_eq!(result.counter_set_observations().len(), 2);
    assert_eq!(result.counter_set_observations()[0].occurrence_index(), 0);
    assert_eq!(result.counter_set_observations()[1].occurrence_index(), 1);
    assert_eq!(item_names(&result, 0), [Some("foo")]);
    assert_eq!(item_names(&result, 1), [Some("bar")]);
    assert_eq!(result.counter_increment_observations().len(), 1);
    assert_eq!(result.counter_reset_observations().len(), 2);
    assert_eq!(result.container_name_observations().len(), 1);
    assert_eq!(result.color_scheme_observations().len(), 1);
    assert_eq!(result.will_change_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_counter_set_dispatch() {
    for (source_id, css) in [
        (600134, "@font-face{counter-set:foo;}"),
        (600135, "@page{counter-set:foo;}"),
        (600136, "@keyframes k{from{counter-set:foo;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.counter_set_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    let result = qualify(
        600137,
        concat!(
            "a{counter-set:foo(bar baz) qux;}",
            "b{counter-set:(foo bar) qux;}",
            "c{counter-set:foo) bar;}",
        ),
    );

    for index in 0..3 {
        assert_invalid(&result, index);
    }
}

// Y. Resource / committed prefix / determinism.

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    let incomplete = qualify_with_limits(
        600138,
        "a{counter-set:foo;}b{counter-set:foo bar;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.counter_set_observations().len(), 1);
    assert_eq!(item_names(&incomplete, 0), [Some("foo")]);

    let css = concat!(
        "a{counter-set:foo 1 bar;}",
        "b{counter-set:none;}",
        "c{counter-set:var(--x);}",
        "d{counter-set:none foo;}",
        "e{counter-set:foo calc(1);}",
        "f{counter-set:reversed(foo);}",
    );
    let first = qualify(600139, css);
    let repeated = qualify(600139, css);
    let another_source = qualify(600140, css);

    assert_eq!(
        first.counter_set_observations(),
        repeated.counter_set_observations()
    );
    assert_eq!(
        first.counter_set_observations(),
        another_source.counter_set_observations()
    );
}

// Z. Regression evidence-ref sanity: evidence never points at trivia.

#[test]
fn evidence_refs_resolve_to_direct_ident_and_integer_tokens_not_trivia() {
    let result = qualify(
        600141,
        "a{counter-set:\n    /*a*/ foo /*b*/\n    /*c*/ 0 /*d*/\n    /*e*/ bar;}",
    );

    let items = qualified_items(&result, 0);
    assert_eq!(items.len(), 2);

    let name_token = |evidence: CssCounterSetNameEvidenceRef| {
        &result
            .upstream_parser_result()
            .upstream_tokenizer_result()
            .lexical_items()[evidence.lexical_item_index()]
    };

    for item in items {
        let CssLexicalItem::SemanticToken(token) = name_token(item.name()) else {
            panic!("counter-set name evidence pointed to trivia");
        };
        assert!(matches!(token.kind(), CssTokenKind::Ident(_)));
    }

    let explicit = items[0]
        .explicit_integer()
        .expect("first item has explicit integer");
    let integer_token = result
        .counter_set_integer_token(explicit)
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
