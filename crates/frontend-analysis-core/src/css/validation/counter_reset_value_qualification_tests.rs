use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssNumberSign, CssNumberType, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssCounterResetItem, CssCounterResetName, CssCounterResetQualificationOutcome,
    CssCounterResetUnsupportedReason, CssCounterResetValue, CssValueQualificationRunResult, run,
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
) -> &CssCounterResetQualificationOutcome {
    result.counter_reset_observations()[index].outcome()
}

fn assert_none(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterResetQualificationOutcome::Qualified(CssCounterResetValue::None),
        "expected whole-value None sentinel at index {index}"
    );
}

fn assert_invalid(result: &CssValueQualificationRunResult, index: usize) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterResetQualificationOutcome::InvalidForSelectedValueGrammar,
        "expected InvalidForSelectedValueGrammar at index {index}"
    );
}

fn assert_unsupported(
    result: &CssValueQualificationRunResult,
    index: usize,
    reason: CssCounterResetUnsupportedReason,
) {
    assert_eq!(
        outcome_at(result, index),
        &CssCounterResetQualificationOutcome::UnsupportedBySelectedValueProfile(reason),
        "expected Unsupported({reason:?}) at index {index}"
    );
}

/// Resolves one observation's ordered qualified `CounterResetItem`s.
/// Panics if the observation at `index` is not `Qualified(Items(_))`.
fn qualified_items(
    result: &CssValueQualificationRunResult,
    index: usize,
) -> &[CssCounterResetItem] {
    match outcome_at(result, index) {
        CssCounterResetQualificationOutcome::Qualified(CssCounterResetValue::Items(items)) => items,
        other => panic!("expected Qualified(Items(_)) at index {index}, got {other:?}"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedItemName<'a> {
    Direct(&'a str),
    Reversed(&'a str),
}

/// Resolves one observation's ordered qualified items to their branch
/// (`Direct`/`Reversed`) and tokenizer-owned decoded `<counter-name>`
/// identity in a single hand-checkable value.
fn item_names(result: &CssValueQualificationRunResult, index: usize) -> Vec<ExpectedItemName<'_>> {
    qualified_items(result, index)
        .iter()
        .map(|item| match item.name() {
            CssCounterResetName::Direct(evidence) => ExpectedItemName::Direct(
                result
                    .counter_reset_name_value(evidence)
                    .expect("direct name evidence resolves"),
            ),
            CssCounterResetName::Reversed(evidence) => ExpectedItemName::Reversed(
                result
                    .counter_reset_name_value(evidence)
                    .expect("reversed name evidence resolves"),
            ),
        })
        .collect()
}

/// Resolves one observation's ordered qualified items to whether each has an
/// authored explicit integer -- `false` means authored-absent, never a
/// synthesized default.
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
        .counter_reset_integer_token(evidence)
        .unwrap_or_else(|| panic!("integer evidence at {index}/{item_index} did not resolve"));
    let CssTokenKind::Number { value, number_type } = token else {
        panic!("integer evidence at {index}/{item_index} did not resolve to a Number token");
    };
    assert_eq!(*number_type, CssNumberType::Integer);
    (value.sign(), value.decimal().integer_digits())
}

// 63. Basic whole `none` and basic Direct/Reversed items.

#[test]
fn whole_none_qualifies() {
    let result = qualify(594100, "a{counter-reset:none;}");
    assert_none(&result, 0);
}

#[test]
fn basic_direct_and_reversed_items_qualify_with_expected_omission() {
    let result = qualify(
        594101,
        concat!(
            "a{counter-reset:foo;}",
            "b{counter-reset:foo 0;}",
            "c{counter-reset:foo 1;}",
            "d{counter-reset:foo -1;}",
            "e{counter-reset:foo +1;}",
            "f{counter-reset:reversed(foo);}",
            "g{counter-reset:reversed(foo) 0;}",
            "h{counter-reset:reversed(foo) 1;}",
            "i{counter-reset:reversed(foo) -1;}",
            "j{counter-reset:reversed(foo) +1;}",
        ),
    );

    assert_eq!(item_names(&result, 0), [ExpectedItemName::Direct("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);

    assert_eq!(item_names(&result, 1), [ExpectedItemName::Direct("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);
    assert_eq!(explicit_integer_sign_and_digits(&result, 1, 0), (None, "0"));

    assert_eq!(item_names(&result, 2), [ExpectedItemName::Direct("foo")]);
    assert_eq!(explicit_integer_sign_and_digits(&result, 2, 0), (None, "1"));

    assert_eq!(
        explicit_integer_sign_and_digits(&result, 3, 0),
        (Some(CssNumberSign::Minus), "1")
    );
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 4, 0),
        (Some(CssNumberSign::Plus), "1")
    );

    assert_eq!(item_names(&result, 5), [ExpectedItemName::Reversed("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 5), [false]);

    assert_eq!(item_names(&result, 6), [ExpectedItemName::Reversed("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 6), [true]);
    assert_eq!(explicit_integer_sign_and_digits(&result, 6, 0), (None, "0"));

    assert_eq!(explicit_integer_sign_and_digits(&result, 7, 0), (None, "1"));
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 8, 0),
        (Some(CssNumberSign::Minus), "1")
    );
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 9, 0),
        (Some(CssNumberSign::Plus), "1")
    );
}

// 64. Mixed Direct/Reversed structured repetition.

#[test]
fn mixed_direct_and_reversed_structured_items_qualify_with_exact_order() {
    let result = qualify(
        594102,
        concat!(
            "a{counter-reset:foo reversed(bar);}",
            "b{counter-reset:foo 1 reversed(bar);}",
            "c{counter-reset:reversed(foo) bar;}",
            "d{counter-reset:reversed(foo) 1 bar;}",
            "e{counter-reset:foo 1 reversed(bar) 2 baz -1;}",
            "f{counter-reset:reversed(foo) 1 bar 2 reversed(baz);}",
            "g{counter-reset:foo 1 foo 2;}",
            "h{counter-reset:reversed(foo) 1 reversed(foo) 2;}",
            "i{counter-reset:foo reversed(foo);}",
        ),
    );

    assert_items_len(&result, 0, 2);
    assert_eq!(
        item_names(&result, 0),
        [
            ExpectedItemName::Direct("foo"),
            ExpectedItemName::Reversed("bar")
        ]
    );
    assert_eq!(item_explicit_integer_presence(&result, 0), [false, false]);

    assert_items_len(&result, 1, 2);
    assert_eq!(
        item_names(&result, 1),
        [
            ExpectedItemName::Direct("foo"),
            ExpectedItemName::Reversed("bar")
        ]
    );
    assert_eq!(item_explicit_integer_presence(&result, 1), [true, false]);

    assert_items_len(&result, 2, 2);
    assert_eq!(
        item_names(&result, 2),
        [
            ExpectedItemName::Reversed("foo"),
            ExpectedItemName::Direct("bar")
        ]
    );
    assert_eq!(item_explicit_integer_presence(&result, 2), [false, false]);

    assert_items_len(&result, 3, 2);
    assert_eq!(
        item_names(&result, 3),
        [
            ExpectedItemName::Reversed("foo"),
            ExpectedItemName::Direct("bar")
        ]
    );
    assert_eq!(item_explicit_integer_presence(&result, 3), [true, false]);

    assert_items_len(&result, 4, 3);
    assert_eq!(
        item_names(&result, 4),
        [
            ExpectedItemName::Direct("foo"),
            ExpectedItemName::Reversed("bar"),
            ExpectedItemName::Direct("baz")
        ]
    );
    assert_eq!(
        item_explicit_integer_presence(&result, 4),
        [true, true, true]
    );
    assert_eq!(explicit_integer_sign_and_digits(&result, 4, 0), (None, "1"));
    assert_eq!(explicit_integer_sign_and_digits(&result, 4, 1), (None, "2"));
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 4, 2),
        (Some(CssNumberSign::Minus), "1")
    );

    assert_items_len(&result, 5, 3);
    assert_eq!(
        item_names(&result, 5),
        [
            ExpectedItemName::Reversed("foo"),
            ExpectedItemName::Direct("bar"),
            ExpectedItemName::Reversed("baz")
        ]
    );
    assert_eq!(
        item_explicit_integer_presence(&result, 5),
        [true, true, false]
    );

    assert_items_len(&result, 6, 2);
    assert_eq!(
        item_names(&result, 6),
        [
            ExpectedItemName::Direct("foo"),
            ExpectedItemName::Direct("foo")
        ]
    );
    assert_eq!(item_explicit_integer_presence(&result, 6), [true, true]);

    assert_items_len(&result, 7, 2);
    assert_eq!(
        item_names(&result, 7),
        [
            ExpectedItemName::Reversed("foo"),
            ExpectedItemName::Reversed("foo")
        ]
    );
    assert_eq!(item_explicit_integer_presence(&result, 7), [true, true]);

    assert_items_len(&result, 8, 2);
    assert_eq!(
        item_names(&result, 8),
        [
            ExpectedItemName::Direct("foo"),
            ExpectedItemName::Reversed("foo")
        ]
    );
    assert_eq!(item_explicit_integer_presence(&result, 8), [false, false]);
}

// 65. Function-name recognition is ASCII-case-insensitive; inner identity is
// never lowercased.

#[test]
fn reversed_function_name_recognition_is_ascii_case_insensitive() {
    let result = qualify(
        594103,
        concat!(
            "a{counter-reset:reversed(foo);}",
            "b{counter-reset:REVERSED(foo);}",
            "c{counter-reset:ReVeRsEd(foo);}",
            "d{counter-reset:\\72 eversed(foo);}",
        ),
    );

    for index in 0..4 {
        assert_eq!(
            item_names(&result, index),
            [ExpectedItemName::Reversed("foo")]
        );
    }
}

// 66. Inner Ident case sensitivity is preserved -- distinct from the
// ASCII-insensitive Function-name match.

#[test]
fn reversed_inner_name_case_sensitivity_is_preserved() {
    let result = qualify(
        594104,
        concat!(
            "a{counter-reset:reversed(Foo);}",
            "b{counter-reset:reversed(foo);}",
            "c{counter-reset:reversed(FOO);}",
        ),
    );

    assert_eq!(item_names(&result, 0), [ExpectedItemName::Reversed("Foo")]);
    assert_eq!(item_names(&result, 1), [ExpectedItemName::Reversed("foo")]);
    assert_eq!(item_names(&result, 2), [ExpectedItemName::Reversed("FOO")]);
}

// 67. Unrelated keywords remain valid inside `reversed()`, unlike
// `container-name`'s additional exclusions.

#[test]
fn unreserved_keywords_remain_valid_inside_reversed() {
    let result = qualify(
        594105,
        concat!(
            "a{counter-reset:reversed(auto);}",
            "b{counter-reset:reversed(normal);}",
            "c{counter-reset:reversed(and);}",
            "d{counter-reset:reversed(not);}",
            "e{counter-reset:reversed(or);}",
            "f{counter-reset:reversed(light);}",
            "g{counter-reset:reversed(dark);}",
        ),
    );

    for (index, expected) in ["auto", "normal", "and", "not", "or", "light", "dark"]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            item_names(&result, index),
            [ExpectedItemName::Reversed(expected)]
        );
    }
}

// 68. Reserved identifiers are invalid inside `reversed()`.

#[test]
fn reserved_identifiers_are_invalid_inside_reversed() {
    let result = qualify(
        594106,
        concat!(
            "a{counter-reset:reversed(none);}",
            "b{counter-reset:reversed(default);}",
            "c{counter-reset:reversed(initial);}",
            "d{counter-reset:reversed(inherit);}",
            "e{counter-reset:reversed(unset);}",
            "f{counter-reset:reversed(revert);}",
            "g{counter-reset:reversed(revert-layer);}",
            "h{counter-reset:reversed(revert-rule);}",
        ),
    );

    for index in 0..8 {
        assert_invalid(&result, index);
    }
}

// 69. Malformed `reversed()` inner grammar is invalid.

#[test]
fn malformed_reversed_inner_grammar_is_invalid() {
    let result = qualify(
        594107,
        concat!(
            "a{counter-reset:reversed();}",
            "b{counter-reset:reversed( );}",
            "c{counter-reset:reversed(/*comment*/);}",
            "d{counter-reset:reversed(foo bar);}",
            "e{counter-reset:reversed(foo,bar);}",
            "f{counter-reset:reversed(foo,);}",
            "g{counter-reset:reversed(,foo);}",
            "h{counter-reset:reversed(\"foo\");}",
            "i{counter-reset:reversed(1);}",
            "j{counter-reset:reversed(1px);}",
            "k{counter-reset:reversed(#foo);}",
            "l{counter-reset:reversed(foo());}",
            "m{counter-reset:reversed(calc(1));}",
            "n{counter-reset:reversed(first-valid(foo, bar));}",
            "o{counter-reset:reversed((foo));}",
            "p{counter-reset:reversed([foo]);}",
            "q{counter-reset:reversed({foo});}",
        ),
    );

    for index in 0..17 {
        assert_invalid(&result, index);
    }
}

// 70. Trivia: comments/whitespace inside `reversed()` never shift the
// stored inner evidence off the Ident token.

#[test]
fn reversed_inner_evidence_resolves_to_ident_not_trivia_or_opener() {
    let result = qualify(594108, "a{counter-reset:reversed(/*a*/ Foo /*b*/);}");

    assert_eq!(item_names(&result, 0), [ExpectedItemName::Reversed("Foo")]);

    let items = qualified_items(&result, 0);
    let CssCounterResetName::Reversed(evidence) = items[0].name() else {
        panic!("expected Reversed branch");
    };
    let token = &result
        .upstream_parser_result()
        .upstream_tokenizer_result()
        .lexical_items()[evidence.lexical_item_index()];
    let CssLexicalItem::SemanticToken(token) = token else {
        panic!("reversed inner name evidence pointed to trivia");
    };
    assert!(matches!(token.kind(), CssTokenKind::Ident(_)));
}

// 71. Escaped inner counter names resolve to tokenizer-decoded identity;
// escape spelling that decodes to reserved `none` is rejected via decoded
// identity, not a manual decoder.

#[test]
fn escaped_inner_counter_name_uses_tokenizer_decoded_identity() {
    let result = qualify(
        594109,
        concat!(
            "a{counter-reset:reversed(a\\ 8);}",
            "b{counter-reset:reversed(\\6e one);}",
        ),
    );

    assert_eq!(item_names(&result, 0), [ExpectedItemName::Reversed("a 8")]);
    assert_invalid(&result, 1);
}

// 72. True stylesheet EOF inside an unclosed `reversed(...)` still yields
// one qualified terminal component from parser-owned retained structure,
// never rejected merely because raw source lacks `)`.

#[test]
fn true_stylesheet_eof_inside_unclosed_reversed_still_qualifies() {
    let result = qualify(594110, "a{counter-reset:reversed(foo");

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
    assert_eq!(result.counter_reset_observations().len(), 1);
    assert_eq!(item_names(&result, 0), [ExpectedItemName::Reversed("foo")]);
    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);
}

// 73. Missing-close negative test: without true EOF, a missing early close
// absorbs what would otherwise be a separate later component into the
// Function's own inner content, so it no longer satisfies exactly one
// Ident -- this seals against a blanket "if missing ')' then accept" bug.
// The classification comes from actual retained component structure (the
// real closing `)` is one token later than a naive split would assume),
// never from raw-source `)` presence/absence.

#[test]
fn reversed_missing_early_close_absorbs_following_material_into_inner_grammar() {
    let closed = qualify(594111, "a{counter-reset:reversed(foo) 1;}");
    assert_eq!(item_names(&closed, 0), [ExpectedItemName::Reversed("foo")]);
    assert_eq!(item_explicit_integer_presence(&closed, 0), [true]);
    assert_eq!(explicit_integer_sign_and_digits(&closed, 0, 0), (None, "1"));

    let missing_close = qualify(594112, "a{counter-reset:reversed(foo 1);}");
    assert_invalid(&missing_close, 0);
}

// 74. Authored omission is distinct evidence from an explicit integer, for
// both Direct and Reversed branches -- never synthesized to any default.

#[test]
fn authored_omission_is_distinct_from_explicit_integer_for_both_branches() {
    let result = qualify(
        594113,
        concat!(
            "a{counter-reset:foo;}",
            "b{counter-reset:foo 0;}",
            "c{counter-reset:reversed(foo);}",
            "d{counter-reset:reversed(foo) 0;}",
        ),
    );

    assert_eq!(item_explicit_integer_presence(&result, 0), [false]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);
    assert_eq!(item_explicit_integer_presence(&result, 2), [false]);
    assert_eq!(item_explicit_integer_presence(&result, 3), [true]);
}

// 75. Sign/zero direct integer evidence is preserved exactly for both
// branches (subset already covered by test 63; this focuses on zero/sign
// spelling specifically for Reversed too).

#[test]
fn direct_integer_sign_and_zero_spelling_is_preserved_for_reversed_branch() {
    let result = qualify(
        594114,
        concat!(
            "a{counter-reset:reversed(foo) 0;}",
            "b{counter-reset:reversed(foo) +0;}",
            "c{counter-reset:reversed(foo) -0;}",
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
}

// 76. Very large direct integers qualify without machine-integer
// conversion, after both a Direct and a Reversed name.

#[test]
fn huge_direct_integers_qualify_without_machine_conversion() {
    let huge_positive = "123456789012345678901234567890123456789012345678901234567890";
    let huge_negative_digits = "987654321098765432109876543210987654321098765432109876543210";
    let css = format!(
        "a{{counter-reset:foo {huge_positive};}}b{{counter-reset:reversed(foo) -{huge_negative_digits};}}"
    );
    let result = qualify(594115, &css);

    assert_eq!(
        explicit_integer_sign_and_digits(&result, 0, 0),
        (None, huge_positive)
    );
    assert_eq!(
        explicit_integer_sign_and_digits(&result, 1, 0),
        (Some(CssNumberSign::Minus), huge_negative_digits)
    );
}

// 77. Fraction/exponent numbers are Number-type, not Integer-type: invalid
// after both branches.

#[test]
fn fraction_and_exponent_numbers_are_invalid_after_either_branch() {
    let result = qualify(
        594116,
        concat!(
            "a{counter-reset:foo 1.0;}",
            "b{counter-reset:foo 1e0;}",
            "c{counter-reset:foo +1e0;}",
            "d{counter-reset:reversed(foo) 1.0;}",
            "e{counter-reset:reversed(foo) 1e0;}",
            "f{counter-reset:reversed(foo) +1e0;}",
        ),
    );

    for index in 0..6 {
        assert_invalid(&result, index);
    }
}

// 78. Function position matrix.

#[test]
fn function_position_matrix() {
    let result = qualify(
        594117,
        concat!(
            "a{counter-reset:reversed(foo);}",
            "b{counter-reset:foo();}",
            "c{counter-reset:arbitrary(foo);}",
            "d{counter-reset:reverse(foo);}",
            "e{counter-reset:reversed-counter(foo);}",
            "f{counter-reset:calc(1);}",
            "g{counter-reset:foo 1 bar();}",
            "h{counter-reset:foo calc(1) bar();}",
            "i{counter-reset:foo first-valid(x, y);}",
            "j{counter-reset:reversed(foo) first-valid(x, y);}",
            "k{counter-reset:reversed(first-valid(x, y));}",
            "l{counter-reset:foo calc(1);}",
            "m{counter-reset:foo calc(-2.5);}",
            "n{counter-reset:foo min(1, 2);}",
            "o{counter-reset:reversed(foo) calc(1);}",
            "p{counter-reset:reversed(foo) calc(-2.5);}",
            "q{counter-reset:reversed(foo) min(1, 2);}",
            "r{counter-reset:reversed(foo) calc(1) bar;}",
            "s{counter-reset:reversed(foo) calc(1) reversed(baz);}",
            "t{counter-reset:foo calc(1) reversed(bar);}",
            "u{counter-reset:reversed(foo) calc(1) 2;}",
            "v{counter-reset:first-valid(x, y);}",
        ),
    );

    assert_eq!(item_names(&result, 0), [ExpectedItemName::Reversed("foo")]);

    for index in 1..11 {
        assert_invalid(&result, index);
    }

    for index in 11..17 {
        assert_unsupported(
            &result,
            index,
            CssCounterResetUnsupportedReason::FunctionValuedIntegerSlot,
        );
    }

    for index in 17..20 {
        assert_unsupported(
            &result,
            index,
            CssCounterResetUnsupportedReason::FunctionValuedIntegerSlot,
        );
    }

    assert_invalid(&result, 20);

    assert_unsupported(
        &result,
        21,
        CssCounterResetUnsupportedReason::WholeValueFunction,
    );
}

// 79. Deferred substitution matrix, including nested inside `reversed()`.

#[test]
fn deferred_substitution_matrix() {
    let result = qualify(
        594118,
        concat!(
            "a{counter-reset:var(--x);}",
            "b{counter-reset:foo var(--x);}",
            "c{counter-reset:reversed(var(--x));}",
            "d{counter-reset:reversed(ident(\"foo\"));}",
            "e{counter-reset:reversed(env(foo));}",
            "f{counter-reset:reversed(attr(foo));}",
            "g{counter-reset:reversed(var(--x), foo);}",
        ),
    );

    for index in 0..7 {
        assert_unsupported(
            &result,
            index,
            CssCounterResetUnsupportedReason::DeferredSubstitutionFunction,
        );
    }
}

// 80. CSS-wide keyword matrix: whole-value only, never embedded (including
// embedded inside `reversed()`'s own outer position).

#[test]
fn css_wide_keyword_matrix() {
    let whole = qualify(
        594119,
        concat!(
            "a{counter-reset:initial;}",
            "b{counter-reset:inherit;}",
            "c{counter-reset:unset;}",
            "d{counter-reset:revert;}",
            "e{counter-reset:revert-layer;}",
        ),
    );
    for index in 0..5 {
        assert_unsupported(
            &whole,
            index,
            CssCounterResetUnsupportedReason::CssWideKeyword,
        );
    }

    let embedded = qualify(
        594120,
        concat!(
            "a{counter-reset:foo initial;}",
            "b{counter-reset:initial foo;}",
            "c{counter-reset:reversed(foo) inherit;}",
            "d{counter-reset:initial reversed(foo);}",
        ),
    );
    for index in 0..4 {
        assert_invalid(&embedded, index);
    }
}

// 81. `none` matrix.

#[test]
fn none_matrix() {
    let result = qualify(
        594121,
        concat!(
            "a{counter-reset:none foo;}",
            "b{counter-reset:foo none;}",
            "c{counter-reset:none reversed(foo);}",
            "d{counter-reset:reversed(foo) none;}",
            "e{counter-reset:reversed(none);}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// Comma is invalid at the top level -- this grammar is `+`, not `#`,
// exactly as for `counter-increment`.

#[test]
fn top_level_comma_is_invalid() {
    let result = qualify(
        594131,
        concat!(
            "a{counter-reset:foo, bar;}",
            "b{counter-reset:reversed(foo), bar;}",
            "c{counter-reset:foo, reversed(bar);}",
            "d{counter-reset:foo,;}",
            "e{counter-reset:,foo;}",
        ),
    );

    for index in 0..5 {
        assert_invalid(&result, index);
    }
}

// 82. `!important` stays outside the semantic value window; duplicate
// declaration occurrences preserve authored order without cascade.

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    let result = qualify(594122, "a{counter-reset:reversed(foo) 1 bar 2 !important;}");
    assert_items_len(&result, 0, 2);
    assert_eq!(
        item_names(&result, 0),
        [
            ExpectedItemName::Reversed("foo"),
            ExpectedItemName::Direct("bar")
        ]
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
        594123,
        "a{counter-reset:foo;counter-reset:reversed(foo);counter-reset:bar 2;}",
    );

    assert_eq!(result.counter_reset_observations().len(), 3);
    assert_eq!(item_names(&result, 0), [ExpectedItemName::Direct("foo")]);
    assert_eq!(item_names(&result, 1), [ExpectedItemName::Reversed("foo")]);
    assert_eq!(item_names(&result, 2), [ExpectedItemName::Direct("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 2), [true]);
}

// 83. Lifecycle: ordinary placement selected, nonordinary excluded,
// committed-prefix / Incomplete propagation, deterministic repeated runs.

#[test]
fn nonordinary_contexts_do_not_enter_counter_reset_dispatch() {
    for (source_id, css) in [
        (594124, "@font-face{counter-reset:foo;}"),
        (594125, "@page{counter-reset:foo;}"),
        (594126, "@keyframes k{from{counter-reset:foo;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.counter_reset_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    let incomplete = qualify_with_limits(
        594127,
        "a{counter-reset:foo;}b{counter-reset:reversed(foo) bar;}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_eq!(incomplete.counter_reset_observations().len(), 1);
    assert_eq!(
        item_names(&incomplete, 0),
        [ExpectedItemName::Direct("foo")]
    );

    let css = concat!(
        "a{counter-reset:foo 1 reversed(bar);}",
        "b{counter-reset:none;}",
        "c{counter-reset:var(--x);}",
        "d{counter-reset:none foo;}",
        "e{counter-reset:reversed(foo) calc(1);}",
    );
    let first = qualify(594128, css);
    let repeated = qualify(594128, css);
    let another_source = qualify(594129, css);

    assert_eq!(
        first.counter_reset_observations(),
        repeated.counter_reset_observations()
    );
    assert_eq!(
        first.counter_reset_observations(),
        another_source.counter_reset_observations()
    );
}

// 84. Cross-dispatch separation: the new grammar-native Function branch is
// property-local and must not change Function behavior on other qualified
// leaves, including `counter-increment`'s own Function handling.

#[test]
fn cross_dispatch_separation_from_other_qualified_leaves() {
    let result = qualify(
        594130,
        concat!(
            "a{counter-reset:foo;counter-reset:reversed(bar) 2;}",
            "b{counter-increment:foo;counter-increment:reversed(bar);}",
            "c{container-name:foo;}",
            "d{color-scheme:light;}",
            "e{anchor-name:--foo;}",
            "f{animation-name:foo;}",
            "g{transition-property:width;}",
            "h{offset-rotate:auto;}",
            "i{aspect-ratio:16/9;}",
            "j{border-spacing:1px 2px;}",
        ),
    );

    assert_eq!(result.counter_reset_observations().len(), 2);
    assert_eq!(result.counter_reset_observations()[0].occurrence_index(), 0);
    assert_eq!(result.counter_reset_observations()[1].occurrence_index(), 1);
    assert_eq!(item_names(&result, 0), [ExpectedItemName::Direct("foo")]);
    assert_eq!(item_names(&result, 1), [ExpectedItemName::Reversed("bar")]);
    assert_eq!(item_explicit_integer_presence(&result, 1), [true]);

    // `reversed(bar)` is not grammar-native for `counter-increment`: it is
    // an ordinary unrecognized Function, so `counter-increment:reversed(bar)`
    // alone is `InvalidForSelectedValueGrammar` there, never a qualified
    // Reversed item -- the new branch is strictly property-local.
    assert_eq!(result.counter_increment_observations().len(), 2);

    assert_eq!(result.container_name_observations().len(), 1);
    assert_eq!(result.color_scheme_observations().len(), 1);
    assert_eq!(result.anchor_name_observations().len(), 1);
    assert_eq!(result.animation_name_observations().len(), 1);
    assert_eq!(result.transition_property_observations().len(), 1);
    assert_eq!(result.offset_rotate_observations().len(), 1);
    assert_eq!(result.aspect_ratio_observations().len(), 1);
    assert_eq!(result.border_spacing_observations().len(), 1);
}
