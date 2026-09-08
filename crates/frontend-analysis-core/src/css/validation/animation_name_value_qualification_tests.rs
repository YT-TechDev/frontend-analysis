use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssAnimationNameItemValue, CssAnimationNameQualificationObservation,
    CssAnimationNameQualificationOutcome, CssAnimationNameUnsupportedReason,
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

fn qualified_items(kinds: &[CssAnimationNameItemValue]) -> CssAnimationNameQualificationOutcome {
    CssAnimationNameQualificationOutcome::Qualified(kinds.to_vec())
}

fn unsupported(reason: CssAnimationNameUnsupportedReason) -> CssAnimationNameQualificationOutcome {
    CssAnimationNameQualificationOutcome::UnsupportedBySelectedValueProfile(reason)
}

fn assert_expected(
    result: &CssValueQualificationRunResult,
    expected: &[CssAnimationNameQualificationOutcome],
) {
    let actual: Vec<_> = result
        .animation_name_observations()
        .iter()
        .map(|observation| observation.outcome().clone())
        .collect();
    assert_eq!(actual, expected);
}

/// Resolves one observation's ordered `KeyframesName` items to their
/// tokenizer-owned decoded interpreted identity, `None` at each `None`
/// sentinel position.
fn keyframes_name_values<'a>(
    result: &'a CssValueQualificationRunResult,
    observation: &CssAnimationNameQualificationObservation,
) -> Vec<Option<&'a str>> {
    observation
        .keyframes_name_evidence()
        .iter()
        .map(|evidence| {
            evidence.and_then(|evidence| result.animation_name_keyframes_name_value(evidence))
        })
        .collect()
}

/// Resolves one observation's ordered `KeyframesName` items to the exact
/// retained token kind their evidence ref points at (`"ident"` / `"string"`),
/// `None` at each `None` sentinel position.
fn keyframes_name_token_kinds(
    result: &CssValueQualificationRunResult,
    observation: &CssAnimationNameQualificationObservation,
) -> Vec<Option<&'static str>> {
    observation
        .keyframes_name_evidence()
        .iter()
        .map(|evidence| {
            evidence.map(|evidence| {
                let item = &result
                    .upstream_parser_result()
                    .upstream_tokenizer_result()
                    .lexical_items()[evidence.lexical_item_index()];
                let CssLexicalItem::SemanticToken(token) = item else {
                    panic!("animation-name keyframes-name evidence ref pointed to trivia");
                };
                match token.kind() {
                    CssTokenKind::Ident(_) => "ident",
                    CssTokenKind::String(_) => "string",
                    other => panic!(
                        "animation-name keyframes-name evidence ref pointed to unexpected token kind: {other:?}"
                    ),
                }
            })
        })
        .collect()
}

// A. `none` item.

#[test]
fn none_is_a_repeated_item_ascii_case_insensitively_and_escape_equivalent() {
    use CssAnimationNameItemValue::None;

    let result = qualify(
        98300,
        concat!(
            "a{animation-name:none;}",
            "b{animation-name:NONE;}",
            "c{animation-name:NoNe;}",
            "d{animation-name:\\6e one;}",
            "e{animation-name:none,foo;}",
            "f{animation-name:foo,none;}",
            "g{animation-name:none,none;}",
        ),
    );

    use CssAnimationNameItemValue::KeyframesName;
    assert_expected(
        &result,
        &[
            qualified_items(&[None]),
            qualified_items(&[None]),
            qualified_items(&[None]),
            qualified_items(&[None]),
            qualified_items(&[None, KeyframesName]),
            qualified_items(&[KeyframesName, None]),
            qualified_items(&[None, None]),
        ],
    );
    for observation in &result.animation_name_observations()[0..4] {
        assert!(observation.keyframes_name_evidence()[0].is_none());
    }
    let last = &result.animation_name_observations()[6];
    assert!(last.keyframes_name_evidence()[0].is_none());
    assert!(last.keyframes_name_evidence()[1].is_none());
}

// B. Ident keyframes names.

#[test]
fn unquoted_identifiers_qualify_without_sibling_keyword_lookup() {
    use CssAnimationNameItemValue::KeyframesName;

    let result = qualify(
        98301,
        concat!(
            "a{animation-name:foo;}",
            "b{animation-name:Both;}",
            "c{animation-name:ease-in;}",
            "d{animation-name:infinite;}",
            "e{animation-name:paused;}",
            "f{animation-name:first,second,third;}",
            "g{animation-name:Foo,foo,FOO;}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified_items(&[KeyframesName]),
            qualified_items(&[KeyframesName]),
            qualified_items(&[KeyframesName]),
            qualified_items(&[KeyframesName]),
            qualified_items(&[KeyframesName]),
            qualified_items(&[KeyframesName, KeyframesName, KeyframesName]),
            qualified_items(&[KeyframesName, KeyframesName, KeyframesName]),
        ],
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[1]),
        [Some("Both")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[2]),
        [Some("ease-in")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[3]),
        [Some("infinite")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[4]),
        [Some("paused")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[5]),
        [Some("first"), Some("second"), Some("third")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[6]),
        [Some("Foo"), Some("foo"), Some("FOO")]
    );
}

#[test]
fn custom_ident_identity_is_case_sensitive_and_escape_equivalent() {
    use CssAnimationNameItemValue::KeyframesName;

    let result = qualify(
        98302,
        concat!(
            "a{animation-name:Foo;}",
            "b{animation-name:foo;}",
            "c{animation-name:FOO;}",
            "d{animation-name:\\46 oo;}",
        ),
    );

    assert_expected(&result, &vec![qualified_items(&[KeyframesName]); 4]);
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[0]),
        [Some("Foo")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[1]),
        [Some("foo")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[2]),
        [Some("FOO")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[3]),
        [Some("Foo")]
    );
}

// C. String names.

#[test]
fn direct_strings_qualify_as_keyframes_names() {
    use CssAnimationNameItemValue::KeyframesName;

    let result = qualify(
        98303,
        concat!(
            "a{animation-name:\"something\";}",
            "b{animation-name:'multi word string';}",
            "c{animation-name:\"none\";}",
            "d{animation-name:\"initial\";}",
            "e{animation-name:\"inherit\";}",
            "f{animation-name:\"revert\";}",
            "g{animation-name:\"revert-layer\";}",
            "h{animation-name:\"unset\";}",
            "i{animation-name:\"default\";}",
            "j{animation-name:\"a,b\";}",
        ),
    );

    assert_expected(&result, &vec![qualified_items(&[KeyframesName]); 10]);
    let expected_values = [
        "something",
        "multi word string",
        "none",
        "initial",
        "inherit",
        "revert",
        "revert-layer",
        "unset",
        "default",
        "a,b",
    ];
    for (observation, expected) in result
        .animation_name_observations()
        .iter()
        .zip(expected_values)
    {
        assert_eq!(
            keyframes_name_values(&result, observation),
            [Some(expected)]
        );
    }
}

#[test]
fn escape_decoded_string_identity_is_tokenizer_owned() {
    use CssAnimationNameItemValue::KeyframesName;

    let result = qualify(98304, "a{animation-name:\"\\1400\";}");

    assert_expected(&result, &[qualified_items(&[KeyframesName])]);
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[0]),
        [Some("\u{1400}")]
    );
    assert_eq!(
        keyframes_name_token_kinds(&result, &result.animation_name_observations()[0]),
        [Some("string")]
    );
}

#[test]
fn comma_inside_string_payload_does_not_split_the_list() {
    use CssAnimationNameItemValue::KeyframesName;

    let result = qualify(98305, "a{animation-name:\"a,b\",foo;}");

    assert_expected(&result, &[qualified_items(&[KeyframesName, KeyframesName])]);
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[0]),
        [Some("a,b"), Some("foo")]
    );
}

// D. Heterogeneous semantic identity.

#[test]
fn ident_and_string_with_equal_decoded_text_share_interpreted_identity_but_distinct_evidence() {
    use CssAnimationNameItemValue::KeyframesName;

    let result = qualify(98306, "a{animation-name:foo,\"foo\",FOO;}");

    assert_expected(
        &result,
        &[qualified_items(&[
            KeyframesName,
            KeyframesName,
            KeyframesName,
        ])],
    );

    let observation = &result.animation_name_observations()[0];
    let values = keyframes_name_values(&result, observation);
    assert_eq!(values, [Some("foo"), Some("foo"), Some("FOO")]);
    assert_eq!(values[0], values[1]);
    assert_ne!(values[1], values[2]);

    let kinds = keyframes_name_token_kinds(&result, observation);
    assert_eq!(kinds, [Some("ident"), Some("string"), Some("ident")]);

    let evidence = observation.keyframes_name_evidence();
    assert_eq!(evidence.len(), 3);
    let indices: Vec<_> = evidence
        .iter()
        .map(|entry| entry.unwrap().lexical_item_index())
        .collect();
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
}

// E. `none` versus `"none"`.

#[test]
fn none_sentinel_and_string_none_remain_distinct_and_case_sensitive() {
    use CssAnimationNameItemValue::{KeyframesName, None as NoneItem};

    let result = qualify(98307, "a{animation-name:none,\"none\",NoNe,\"NoNe\";}");

    assert_expected(
        &result,
        &[qualified_items(&[
            NoneItem,
            KeyframesName,
            NoneItem,
            KeyframesName,
        ])],
    );

    let observation = &result.animation_name_observations()[0];
    assert_eq!(
        keyframes_name_values(&result, observation),
        [None, Some("none"), None, Some("NoNe")]
    );
    let evidence = observation.keyframes_name_evidence();
    assert!(evidence[0].is_none());
    assert!(evidence[1].is_some());
    assert!(evidence[2].is_none());
    assert!(evidence[3].is_some());
}

// F. Empty String.

#[test]
fn empty_string_is_invalid_unlike_hyphenate_character() {
    use CssAnimationNameQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        98308,
        concat!(
            "a{animation-name:\"\";}",
            "b{animation-name:foo,\"\";}",
            "c{animation-name:\"\",foo;}",
        ),
    );

    assert_expected(&result, &vec![Invalid; 3]);
}

// G. Wrong token classes.

#[test]
fn wrong_token_classes_are_invalid() {
    use CssAnimationNameQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        98309,
        concat!(
            "a{animation-name:12;}",
            "b{animation-name:1px;}",
            "c{animation-name:50%;}",
        ),
    );
    assert_expected(&result, &vec![Invalid; 3]);

    // A literal newline before the closing quote forces the tokenizer to
    // retain `CssTokenKind::BadString` rather than `String`, per the
    // accepted tokenizer recovery boundary; `BadString` is not `<string>`.
    let bad_string = qualify(98310, "a{animation-name:\"x\n;}");
    assert_expected(&bad_string, &[Invalid]);
}

// H. Structural invalidity.

#[test]
fn structural_separator_and_missing_comma_cases_are_invalid() {
    use CssAnimationNameQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        98311,
        concat!(
            "a{animation-name:;}",
            "b{animation-name:,foo;}",
            "c{animation-name:foo,;}",
            "d{animation-name:foo,,bar;}",
            "e{animation-name:one two;}",
            "f{animation-name:\"foo\" \"bar\";}",
        ),
    );

    assert_expected(&result, &vec![Invalid; 6]);
}

// I. CSS-wide keywords / `default`.

#[test]
fn css_wide_keywords_and_default_boundary() {
    use CssAnimationNameItemValue::KeyframesName;
    use CssAnimationNameQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationNameUnsupportedReason::CssWideKeyword;

    let whole = qualify(
        98312,
        concat!(
            "a{animation-name:initial;}",
            "b{animation-name:inherit;}",
            "c{animation-name:unset;}",
            "d{animation-name:revert;}",
            "e{animation-name:revert-layer;}",
        ),
    );
    assert_expected(&whole, &vec![unsupported(CssWideKeyword); 5]);

    let list_placement = qualify(
        98313,
        concat!(
            "a{animation-name:foo,initial;}",
            "b{animation-name:initial,foo;}",
            "c{animation-name:foo,inherit;}",
            "d{animation-name:revert,foo;}",
        ),
    );
    assert_expected(&list_placement, &vec![Invalid; 4]);

    let default_boundary = qualify(
        98314,
        concat!(
            "a{animation-name:default;}",
            "b{animation-name:foo,default;}",
        ),
    );
    assert_expected(&default_boundary, &vec![Invalid; 2]);

    let quoted_forms = qualify(
        98315,
        concat!(
            "a{animation-name:\"initial\";}",
            "b{animation-name:\"inherit\";}",
            "c{animation-name:\"default\";}",
        ),
    );
    assert_expected(&quoted_forms, &vec![qualified_items(&[KeyframesName]); 3]);
}

// J. Ordinary Functions.

#[test]
fn ordinary_function_items_are_invalid_never_softened_to_unsupported() {
    use CssAnimationNameQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        98316,
        concat!(
            "a{animation-name:foo();}",
            "b{animation-name:calc(1);}",
            "c{animation-name:foo,bar();}",
            "d{animation-name:bar(),foo;}",
        ),
    );

    assert_expected(&result, &vec![Invalid; 4]);
}

// K. Generic whole-value Function placement.

#[test]
fn whole_value_generic_function_boundary_does_not_legalize_function_items() {
    use CssAnimationNameQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;
    use CssAnimationNameUnsupportedReason::WholeValueFunction;

    let whole = qualify(98317, "a{animation-name:first-valid(foo, \"bar\");}");
    assert_expected(&whole, &[unsupported(WholeValueFunction)]);

    let items = qualify(
        98318,
        concat!(
            "a{animation-name:foo,first-valid(\"bar\");}",
            "b{animation-name:first-valid(foo),\"bar\";}",
        ),
    );
    assert_expected(&items, &vec![Invalid; 2]);
}

// L. Deferred substitution.

#[test]
fn deferred_substitution_precedes_comma_list_recognition_everywhere() {
    use CssAnimationNameUnsupportedReason::DeferredSubstitutionFunction;

    let result = qualify(
        98319,
        concat!(
            "a{animation-name:var(--x);}",
            "b{animation-name:foo,var(--x);}",
            "c{animation-name:var(--x),foo;}",
            "d{animation-name:var(--x,foo,\"bar\");}",
        ),
    );

    assert_expected(&result, &vec![unsupported(DeferredSubstitutionFunction); 4]);
}

// M. Trivia / priority.

#[test]
fn trivia_around_separators_does_not_shift_keyframes_name_evidence() {
    use CssAnimationNameItemValue::{KeyframesName, None as NoneItem};

    let result = qualify(
        98320,
        concat!(
            "a{animation-name:foo,bar;}",
            "b{animation-name: foo , bar ;}",
            "c{animation-name:foo/*a*/,/*b*/bar;}",
            "d{animation-name:/*x*/foo/*y*/,/*z*/none/*w*/;}",
            "e{animation-name:/*a*/ Foo /*b*/,/*c*/ none /*d*/,/*e*/ \"bar\";}",
        ),
    );

    assert_expected(
        &result,
        &[
            qualified_items(&[KeyframesName, KeyframesName]),
            qualified_items(&[KeyframesName, KeyframesName]),
            qualified_items(&[KeyframesName, KeyframesName]),
            qualified_items(&[KeyframesName, NoneItem]),
            qualified_items(&[KeyframesName, NoneItem, KeyframesName]),
        ],
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[0]),
        [Some("foo"), Some("bar")]
    );
    assert_eq!(
        keyframes_name_values(&result, &result.animation_name_observations()[4]),
        [Some("Foo"), None, Some("bar")]
    );
}

#[test]
fn important_priority_is_outside_the_semantic_value_window() {
    use CssAnimationNameItemValue::KeyframesName;

    let result = qualify(98321, "a{animation-name:foo,\"bar\" !important;}");
    assert_expected(&result, &[qualified_items(&[KeyframesName, KeyframesName])]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
}

// N. Evidence ownership.

#[test]
fn none_items_fabricate_no_keyframes_name_evidence() {
    let result = qualify(98322, "a{animation-name:none,none,none;}");
    let observation = &result.animation_name_observations()[0];
    assert!(
        observation
            .keyframes_name_evidence()
            .iter()
            .all(Option::is_none)
    );
}

#[test]
fn keyframes_name_evidence_points_to_the_exact_retained_lexical_item() {
    let result = qualify(98323, "a{animation-name:foo,\"bar\",none,baz;}");
    let observation = &result.animation_name_observations()[0];
    let evidence = observation.keyframes_name_evidence();
    assert_eq!(evidence.len(), 4);
    assert!(evidence[0].is_some());
    assert!(evidence[1].is_some());
    assert!(evidence[2].is_none());
    assert!(evidence[3].is_some());

    let indices: Vec<_> = [0usize, 1, 3]
        .iter()
        .map(|&position| evidence[position].unwrap().lexical_item_index())
        .collect();
    assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));

    for &position in &[0usize, 1, 3] {
        let index = evidence[position].unwrap().lexical_item_index();
        let item = &result
            .upstream_parser_result()
            .upstream_tokenizer_result()
            .lexical_items()[index];
        assert!(matches!(item, CssLexicalItem::SemanticToken(_)));
    }
}

// O. Lifecycle.

#[test]
fn duplicate_occurrences_and_existing_leaf_dispatch_remain_separate() {
    use CssAnimationNameItemValue::KeyframesName;

    let result = qualify(
        98324,
        concat!(
            "a{animation-name:foo;animation-name:\"bar\",none;}",
            "b{page:Foo;}",
            "c{transition-property:width;}",
            "d{hyphenate-character:\"-\";}",
            "e{animation-play-state:running;}",
            "f{animation-iteration-count:1;}",
            "g{animation-delay:1s;}",
            "h{transition-duration:1s;}",
        ),
    );

    assert_eq!(result.animation_name_observations().len(), 2);
    assert_eq!(
        result.animation_name_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.animation_name_observations()[1].occurrence_index(),
        1
    );
    assert_expected(
        &result,
        &[
            qualified_items(&[KeyframesName]),
            qualified_items(&[KeyframesName, CssAnimationNameItemValue::None]),
        ],
    );
    assert_eq!(result.page_observations().len(), 1);
    assert_eq!(result.transition_property_observations().len(), 1);
    assert_eq!(result.hyphenate_character_observations().len(), 1);
    assert_eq!(result.animation_play_state_observations().len(), 1);
    assert_eq!(result.animation_iteration_count_observations().len(), 1);
    assert_eq!(result.animation_delay_observations().len(), 1);
    assert_eq!(result.transition_duration_observations().len(), 1);
}

#[test]
fn nonordinary_contexts_do_not_enter_animation_name_dispatch() {
    for (source_id, css) in [
        (98325, "@font-face{animation-name:foo;}"),
        (98326, "@page{animation-name:foo;}"),
        (98327, "@keyframes k{from{animation-name:foo;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.animation_name_observations().is_empty(),
            "nonordinary context produced observation for {css:?}"
        );
    }
}

#[test]
fn unmatched_or_nested_block_evidence_cannot_fake_top_level_items() {
    use CssAnimationNameQualificationOutcome::InvalidForSelectedValueGrammar as Invalid;

    let result = qualify(
        98328,
        concat!(
            "a{animation-name:foo(bar,baz),qux;}",
            "b{animation-name:(foo,bar),qux;}",
            "c{animation-name:foo),bar;}",
        ),
    );

    assert_expected(&result, &vec![Invalid; 3]);
}

#[test]
fn committed_prefix_and_repeated_cross_source_runs_preserve_lifecycle() {
    use CssAnimationNameItemValue::KeyframesName;

    let incomplete = qualify_with_limits(
        98329,
        "a{animation-name:foo;}b{animation-name:foo,\"bar\";}",
        parser_limits_with_occurrences(1),
    );
    assert_eq!(
        incomplete.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&incomplete, &[qualified_items(&[KeyframesName])]);

    let css = concat!(
        "a{animation-name:foo,\"bar\";}",
        "b{animation-name:none;}",
        "c{animation-name:var(--x);}",
        "d{animation-name:none,foo;}",
    );
    let first = qualify(98330, css);
    let repeated = qualify(98330, css);
    let another_source = qualify(98331, css);

    assert_eq!(
        first.animation_name_observations(),
        repeated.animation_name_observations()
    );
    assert_eq!(
        first.animation_name_observations(),
        another_source.animation_name_observations()
    );
}
