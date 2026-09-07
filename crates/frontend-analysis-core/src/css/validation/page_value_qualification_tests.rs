use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::token::{CssLexicalItem, CssTokenKind};
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssPageQualificationOutcome, CssPageUnsupportedReason, CssPageValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Auto,
    CustomIdent,
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

fn expected_outcome(expected: ExpectedOutcome) -> CssPageQualificationOutcome {
    match expected {
        ExpectedOutcome::Auto => CssPageQualificationOutcome::Qualified(CssPageValue::Auto),
        ExpectedOutcome::CustomIdent => {
            CssPageQualificationOutcome::Qualified(CssPageValue::CustomIdent)
        }
        ExpectedOutcome::Invalid => CssPageQualificationOutcome::InvalidForSelectedValueGrammar,
        ExpectedOutcome::UnsupportedCssWide => {
            CssPageQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPageUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedDeferred => {
            CssPageQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPageUnsupportedReason::DeferredSubstitutionFunction,
            )
        }
        ExpectedOutcome::UnsupportedWholeValue => {
            CssPageQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssPageUnsupportedReason::WholeValueFunction,
            )
        }
    }
}

fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .page_observations()
        .iter()
        .map(|observation| observation.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

fn custom_ident_values(result: &CssValueQualificationRunResult) -> Vec<&str> {
    result
        .page_observations()
        .iter()
        .filter_map(|observation| result.page_custom_ident_value(observation))
        .collect()
}

#[test]
fn auto_is_ascii_case_insensitive_and_custom_ident_identity_is_case_sensitive() {
    let result = qualify(
        920,
        concat!(
            "a{page:auto;}",
            "b{page:AUTO;}",
            "c{page:AuTo;}",
            "d{page:\\61 uto;}",
            "e{page:Foo;}",
            "f{page:foo;}",
            "g{page:FOO;}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::Auto,
            ExpectedOutcome::CustomIdent,
            ExpectedOutcome::CustomIdent,
            ExpectedOutcome::CustomIdent,
        ],
    );
    assert_eq!(custom_ident_values(&result), ["Foo", "foo", "FOO"]);
}

#[test]
fn escape_equivalent_custom_ident_values_share_interpreted_identity() {
    let result = qualify(
        921,
        concat!("a{page:Foo;}", "b{page:\\46 oo;}", "c{page:\\000046oo;}",),
    );

    assert_expected(&result, &[ExpectedOutcome::CustomIdent; 3]);
    assert_eq!(custom_ident_values(&result), ["Foo", "Foo", "Foo"]);

    for observation in result.page_observations() {
        let evidence = observation
            .custom_ident_evidence()
            .expect("qualified custom-ident must retain its recognition-time evidence ref");
        let item = &result
            .upstream_parser_result()
            .upstream_tokenizer_result()
            .lexical_items()[evidence.lexical_item_index()];
        let token = match item {
            CssLexicalItem::SemanticToken(token) => token,
            _ => panic!("page custom-ident evidence ref did not point to a semantic token"),
        };
        let CssTokenKind::Ident(value) = token.kind() else {
            panic!("page custom-ident evidence ref did not point to an Ident token");
        };
        assert_eq!(value, "Foo");
    }
}

#[test]
fn custom_ident_does_not_unicode_normalize_interpreted_identity() {
    let result = qualify(922, "a{page:é;}b{page:e\u{301};}");

    assert_expected(&result, &[ExpectedOutcome::CustomIdent; 2]);
    assert_eq!(custom_ident_values(&result), ["é", "e\u{301}"]);
    assert_ne!(
        custom_ident_values(&result)[0],
        custom_ident_values(&result)[1]
    );
}

#[test]
fn default_is_reserved_but_other_open_ended_identifiers_remain_qualified() {
    let long_name = format!("page-{}", "x".repeat(1024));
    let css = format!(
        "a{{page:default;}}b{{page:DEFAULT;}}c{{page:\\64 efault;}}d{{page:none;}}e{{page:--foo;}}f{{page:日本語;}}g{{page:{long_name};}}"
    );
    let result = qualify(923, &css);

    assert_expected(
        &result,
        &[
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::CustomIdent,
            ExpectedOutcome::CustomIdent,
            ExpectedOutcome::CustomIdent,
            ExpectedOutcome::CustomIdent,
        ],
    );
    assert_eq!(
        custom_ident_values(&result),
        ["none", "--foo", "日本語", long_name.as_str()]
    );
}

#[test]
fn css_wide_keywords_are_unsupported_including_case_and_escape_equivalents() {
    let result = qualify(
        924,
        concat!(
            "a{page:initial;}",
            "b{page:INHERIT;}",
            "c{page:unset;}",
            "d{page:revert;}",
            "e{page:revert-layer;}",
            "f{page:revert-rule;}",
            "g{page:\\69 nitial;}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 7]);
}

#[test]
fn token_category_cardinality_and_ordinary_function_mismatches_are_invalid() {
    let result = qualify(
        925,
        concat!(
            "a{page:1;}",
            "b{page:10%;}",
            "c{page:1px;}",
            "d{page:\"foo\";}",
            "e{page:;}",
            "f{page:foo bar;}",
            "g{page:foo,bar;}",
            "h{page:foo();}",
            "i{page:calc(1);}",
            "j{page:(foo);}",
        ),
    );

    assert_expected(&result, &[ExpectedOutcome::Invalid; 10]);
}

#[test]
fn deferred_and_whole_value_functions_preserve_fail_open_boundaries() {
    let result = qualify(
        926,
        concat!(
            "a{page:var(--page);}",
            "b{page:env(page);}",
            "c{page:attr(data-page);}",
            "d{page:ident(foo);}",
            "e{page:--page();}",
            "f{page:foo var(--page);}",
            "g{page:first-valid(foo,bar);}",
            "h{page:cycle(foo,bar);}",
            "i{page:interpolate(50%,0:foo,1:bar);}",
            "j{page:foo first-valid(bar,baz);}",
        ),
    );

    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedDeferred,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::UnsupportedWholeValue,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn comments_priority_and_escaped_identity_use_retained_token_evidence() {
    let result = qualify(
        927,
        concat!(
            "a{page:/**/\\46 oo/**/!important;}",
            "b{page:/**/AUTO/**/!important;}",
        ),
    );

    assert_expected(
        &result,
        &[ExpectedOutcome::CustomIdent, ExpectedOutcome::Auto],
    );
    assert_eq!(custom_ident_values(&result), ["Foo"]);
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
}

#[test]
fn duplicate_page_declarations_keep_distinct_run_local_placement_and_evidence() {
    let result = qualify(928, "a{page:Foo;}b{page:Foo;}");

    assert_expected(&result, &[ExpectedOutcome::CustomIdent; 2]);
    assert_eq!(custom_ident_values(&result), ["Foo", "Foo"]);
    assert_eq!(result.page_observations()[0].occurrence_index(), 0);
    assert_eq!(result.page_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.page_observations()[0].placement().context_id(),
        result.page_observations()[1].placement().context_id(),
    );
    assert_ne!(
        result.page_observations()[0]
            .custom_ident_evidence()
            .unwrap()
            .lexical_item_index(),
        result.page_observations()[1]
            .custom_ident_evidence()
            .unwrap()
            .lexical_item_index(),
    );
}

#[test]
fn nonordinary_page_contexts_do_not_become_property_observations() {
    for (source_id, css) in [
        (929, "@font-face{page:Foo;}"),
        (930, "@page{page:Foo;}"),
        (931, "@page{@top-left{page:Foo;}}"),
        (932, "@keyframes k{from{page:Foo;}}"),
    ] {
        let result = qualify(source_id, css);
        assert!(
            result.page_observations().is_empty(),
            "nonordinary declaration context produced a page observation for {css:?}"
        );
    }
}

#[test]
fn parser_resource_stop_preserves_committed_page_prefix_and_incomplete_completion() {
    let result = qualify_with_limits(
        933,
        "a{page:Foo;page:Bar;}",
        parser_limits_with_occurrences(1),
    );

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert_expected(&result, &[ExpectedOutcome::CustomIdent]);
    assert_eq!(custom_ident_values(&result), ["Foo"]);
    assert_eq!(result.upstream_parser_result().occurrences().len(), 1);
}

#[test]
fn repeated_and_cross_source_page_runs_are_semantically_deterministic() {
    let css = concat!(
        "a{page:auto;}",
        "b{page:Foo;}",
        "c{page:\\46 oo;}",
        "d{page:foo;}",
        "e{page:default;}",
        "f{page:var(--page);}",
    );
    let first = qualify(934, css);
    let repeated = qualify(934, css);
    let another_source = qualify(935, css);

    assert_eq!(first.page_observations(), repeated.page_observations());
    assert_eq!(
        first.page_observations(),
        another_source.page_observations()
    );
    assert_eq!(custom_ident_values(&first), ["Foo", "Foo", "foo"]);
    assert_eq!(custom_ident_values(&repeated), ["Foo", "Foo", "foo"]);
    assert_eq!(custom_ident_values(&another_source), ["Foo", "Foo", "foo"]);
}
