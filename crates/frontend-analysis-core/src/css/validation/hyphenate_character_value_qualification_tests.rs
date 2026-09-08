use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssHyphenateCharacterQualificationOutcome, CssHyphenateCharacterUnsupportedReason,
    CssHyphenateCharacterValue, CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    QualifiedString,
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferred,
    UnsupportedWholeValue,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssHyphenateCharacterQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => {
            CssHyphenateCharacterQualificationOutcome::Qualified(CssHyphenateCharacterValue::Auto)
        }
        ExpectedOutcome::QualifiedString => CssHyphenateCharacterQualificationOutcome::Qualified(
            CssHyphenateCharacterValue::DirectStringLiteral,
        ),
        ExpectedOutcome::Invalid => {
            CssHyphenateCharacterQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssHyphenateCharacterQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssHyphenateCharacterUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssHyphenateCharacterQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssHyphenateCharacterUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssHyphenateCharacterQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssHyphenateCharacterUnsupportedReason::WholeValueFunction,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .hyphenate_character_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

fn string_values(result: &CssValueQualificationRunResult) -> Vec<Option<&str>> {
    result
        .hyphenate_character_observations()
        .iter()
        .map(|observation| result.hyphenate_character_string_value(observation))
        .collect()
}

#[test]
fn auto_is_ascii_case_insensitive_and_escape_equivalent() {
    let result = qualify(
        1040,
        concat!(
            "a{hyphenate-character:auto;}",
            "b{hyphenate-character:AUTO;}",
            "c{hyphenate-character:AuTo;}",
            "d{hyphenate-character:\\61 uto;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Auto; 4]);
    assert_eq!(string_values(&result), [None, None, None, None]);
}

#[test]
fn direct_strings_qualify_including_empty_and_multi_character() {
    let result = qualify(
        1041,
        concat!(
            "a{hyphenate-character:\"=\";}",
            "b{hyphenate-character:'/-/';}",
            "c{hyphenate-character:\"\";}",
            "d{hyphenate-character:'';}",
            "e{hyphenate-character:\"hello\";}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::QualifiedString; 5]);
    assert_eq!(
        string_values(&result),
        [Some("="), Some("/-/"), Some(""), Some(""), Some("hello"),]
    );
}

#[test]
fn escape_decoded_string_identity_is_tokenizer_owned() {
    let result = qualify(1042, "a{hyphenate-character:\"\\1400\";}");

    assert_expected(&result, &[ExpectedOutcome::QualifiedString]);
    assert_eq!(string_values(&result), [Some("\u{1400}")]);

    let observation = &result.hyphenate_character_observations()[0];
    let evidence = observation
        .string_evidence()
        .expect("qualified direct String must retain its recognition-time evidence ref");
    let item = &result
        .upstream_parser_result()
        .upstream_tokenizer_result()
        .lexical_items()[evidence.lexical_item_index()];
    let token = match item {
        CssLexicalItem::SemanticToken(token) => token,
        _ => panic!("hyphenate-character string evidence ref did not point to a semantic token"),
    };
    let CssTokenKind::String(value) = token.kind() else {
        panic!("hyphenate-character string evidence ref did not point to a String token");
    };
    assert_eq!(value, "\u{1400}");
}

#[test]
fn quoted_keyword_looking_strings_remain_string_not_keyword() {
    let result = qualify(
        1043,
        concat!(
            "a{hyphenate-character:\"auto\";}",
            "b{hyphenate-character:\"initial\";}",
            "c{hyphenate-character:\"none\";}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::QualifiedString; 3]);
    assert_eq!(
        string_values(&result),
        [Some("auto"), Some("initial"), Some("none")]
    );
}

#[test]
fn quote_delimiter_style_is_not_semantic_value_kind() {
    let result = qualify(
        1044,
        concat!(
            "a{hyphenate-character:\"-\";}",
            "b{hyphenate-character:'-';}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::QualifiedString; 2]);
    assert_eq!(string_values(&result), [Some("-"), Some("-")]);
}

#[test]
fn auto_versus_quoted_keyword_contrast_is_load_bearing() {
    let result = qualify(
        1045,
        concat!(
            "a{hyphenate-character:auto;}",
            "b{hyphenate-character:\"auto\";}",
            "c{hyphenate-character:initial;}",
            "d{hyphenate-character:\"initial\";}",
            "e{hyphenate-character:none;}",
            "f{hyphenate-character:\"none\";}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::QualifiedString,
            ExpectedOutcome::UnsupportedCssWide,
            ExpectedOutcome::QualifiedString,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::QualifiedString,
        ],
    );
    assert_eq!(
        string_values(&result),
        [
            None,
            Some("auto"),
            None,
            Some("initial"),
            None,
            Some("none"),
        ]
    );
}

#[test]
fn css_wide_keywords_are_unsupported_but_quoted_forms_qualify() {
    let result = qualify(
        1046,
        concat!(
            "a{hyphenate-character:initial;}",
            "b{hyphenate-character:INHERIT;}",
            "c{hyphenate-character:unset;}",
            "d{hyphenate-character:revert;}",
            "e{hyphenate-character:revert-layer;}",
            "f{hyphenate-character:\"initial\";}",
            "g{hyphenate-character:\"inherit\";}",
            "h{hyphenate-character:\"unset\";}",
            "i{hyphenate-character:\"revert\";}",
            "j{hyphenate-character:\"revert-layer\";}",
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
            ExpectedOutcome::QualifiedString,
            ExpectedOutcome::QualifiedString,
            ExpectedOutcome::QualifiedString,
            ExpectedOutcome::QualifiedString,
            ExpectedOutcome::QualifiedString,
        ],
    );
}

#[test]
fn direct_invalid_identifiers_and_wrong_token_classes() {
    let result = qualify(
        1047,
        concat!(
            "a{hyphenate-character:;}",
            "b{hyphenate-character:normal;}",
            "c{hyphenate-character:manual;}",
            "d{hyphenate-character:none;}",
            "e{hyphenate-character:foo;}",
            "f{hyphenate-character:1400;}",
            "g{hyphenate-character:1px;}",
            "h{hyphenate-character:50%;}",
            "i{hyphenate-character:#fff;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 9]);
}

#[test]
fn multi_token_values_never_pick_a_first_valid_component() {
    let result = qualify(
        1048,
        concat!(
            "a{hyphenate-character:auto auto;}",
            "b{hyphenate-character:\"-\" \"=\";}",
            "c{hyphenate-character:auto \"-\";}",
            "d{hyphenate-character:\"-\" auto;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 4]);
}

#[test]
fn ordinary_functions_are_invalid_not_function_value_unsupported() {
    let result = qualify(
        1049,
        concat!(
            "a{hyphenate-character:foo();}",
            "b{hyphenate-character:calc(1);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 2]);
}

#[test]
fn deferred_and_whole_value_functions_preserve_fail_open_boundaries() {
    let result = qualify(
        1050,
        concat!(
            "a{hyphenate-character:var(--x);}",
            "b{hyphenate-character:env(foo);}",
            "c{hyphenate-character:first-valid(auto, \"-\");}",
            "d{hyphenate-character:\"-\" first-valid(auto, \"=\");}",
            "e{hyphenate-character:first-valid(auto, \"=\") \"-\";}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn bad_string_is_not_direct_string() {
    // A literal newline before the closing quote forces the tokenizer to
    // retain `CssTokenKind::BadString` rather than `String`, per the
    // accepted tokenizer recovery boundary (#580 section on BadString).
    let result = qualify(1051, "a{hyphenate-character:\"x\n;}");

    assert_expected(&result, &[ExpectedOutcome::Invalid]);
    assert_eq!(string_values(&result), [None]);
}

#[test]
fn eof_in_string_recovery_still_yields_retained_string_evidence() {
    // No closing quote, brace, or semicolon: the tokenizer reaches end of
    // input while inside the string and, per the accepted EOF-in-string
    // recovery boundary, still commits `CssTokenKind::String(_)` (not
    // `BadString`) with a diagnostic. The qualifier must respect that
    // authoritative retained token kind rather than re-judging the syntax.
    let result = qualify(1052, "a{hyphenate-character:\"x");

    assert_expected(&result, &[ExpectedOutcome::QualifiedString]);
    assert_eq!(string_values(&result), [Some("x")]);
}

#[test]
fn comments_and_priority_do_not_alter_string_payload_or_evidence_target() {
    let result = qualify(
        1053,
        concat!(
            "a{hyphenate-character:/* before */\"\\1400\"/* after */!important;}",
            "b{hyphenate-character:/* a */auto/* b */!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::QualifiedString, ExpectedOutcome::Auto],
    );
    assert_eq!(string_values(&result), [Some("\u{1400}"), None]);
    assert!(
        result.upstream_parser_result().occurrences()[0]
            .priority()
            .is_some()
    );
    assert!(
        result.upstream_parser_result().occurrences()[1]
            .priority()
            .is_some()
    );

    let evidence = result.hyphenate_character_observations()[0]
        .string_evidence()
        .expect("qualified direct String must retain its recognition-time evidence ref");
    let item = &result
        .upstream_parser_result()
        .upstream_tokenizer_result()
        .lexical_items()[evidence.lexical_item_index()];
    match item {
        CssLexicalItem::SemanticToken(token) => {
            assert!(matches!(token.kind(), CssTokenKind::String(_)));
        }
        _ => panic!("hyphenate-character string evidence ref pointed to trivia, not a token"),
    }
}

#[test]
fn auto_never_fabricates_string_evidence() {
    let result = qualify(1054, "a{hyphenate-character:auto;}");

    assert_expected(&result, &[ExpectedOutcome::Auto]);
    assert!(
        result.hyphenate_character_observations()[0]
            .string_evidence()
            .is_none()
    );
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement_and_evidence() {
    let result = qualify(
        1055,
        concat!(
            "a{hyphenate-character:\"-\";",
            "hyphenate-character:auto;",
            "hyphenate-character:\"\";}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::QualifiedString,
            ExpectedOutcome::Auto,
            ExpectedOutcome::QualifiedString,
        ],
    );
    assert_eq!(string_values(&result), [Some("-"), None, Some("")]);
    assert_eq!(
        result.hyphenate_character_observations()[0].occurrence_index(),
        0
    );
    assert_eq!(
        result.hyphenate_character_observations()[1].occurrence_index(),
        1
    );
    assert_eq!(
        result.hyphenate_character_observations()[2].occurrence_index(),
        2
    );
    assert!(
        result.hyphenate_character_observations()[1]
            .string_evidence()
            .is_none()
    );
    assert_ne!(
        result.hyphenate_character_observations()[0]
            .string_evidence()
            .unwrap()
            .lexical_item_index(),
        result.hyphenate_character_observations()[2]
            .string_evidence()
            .unwrap()
            .lexical_item_index(),
    );
}

#[test]
fn nonordinary_contexts_do_not_become_property_observations() {
    for (source_id, css) in [
        (1056, "@font-face{hyphenate-character:\"-\";}"),
        (1057, "@page{hyphenate-character:\"-\";}"),
        (1058, "@keyframes k{from{hyphenate-character:\"-\";}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.hyphenate_character_observations().is_empty(),
            "nonordinary declaration context produced a hyphenate-character observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix_and_incomplete_completion() {
    let result = qualify_with_limits(
        1059,
        "a{hyphenate-character:\"-\";hyphenate-character:auto;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::QualifiedString]);
    assert_eq!(string_values(&result), [Some("-")]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{hyphenate-character:auto;}",
        "b{hyphenate-character:\"-\";}",
        "c{hyphenate-character:\"\\1400\";}",
        "d{hyphenate-character:none;}",
        "e{hyphenate-character:var(--x);}",
    );
    let first = qualify(1060, css);
    let repeated = qualify(1060, css);
    let another_source = qualify(1061, css);

    assert_eq!(
        first.hyphenate_character_observations(),
        repeated.hyphenate_character_observations()
    );
    assert_eq!(
        first.hyphenate_character_observations(),
        another_source.hyphenate_character_observations()
    );
    assert_eq!(
        string_values(&first),
        [None, Some("-"), Some("\u{1400}"), None, None]
    );
    assert_eq!(
        string_values(&repeated),
        [None, Some("-"), Some("\u{1400}"), None, None]
    );
    assert_eq!(
        string_values(&another_source),
        [None, Some("-"), Some("\u{1400}"), None, None]
    );
}

#[test]
fn adversarial_multi_component_and_placement_cases_stay_invalid() {
    let result = qualify(
        1062,
        concat!(
            "a{hyphenate-character:\"\" \"\";}",
            "b{hyphenate-character:auto \"\";}",
            "c{hyphenate-character:\"\" auto;}",
            "d{hyphenate-character:(\"-\");}",
            "e{hyphenate-character:foo(\"-\");}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 5]);
}
