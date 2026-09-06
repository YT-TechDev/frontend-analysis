use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssRubyAlignQualificationOutcome, CssRubyAlignUnsupportedReason, CssRubyAlignValue,
    CssValueQualificationRunResult, run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedOutcome {
    Start,
    Center,
    SpaceBetween,
    SpaceAround,
    Invalid,
    UnsupportedCssWide,
    UnsupportedFunction,
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
fn expected_outcome(expected: ExpectedOutcome) -> CssRubyAlignQualificationOutcome {
    match expected {
        ExpectedOutcome::Start => {
            CssRubyAlignQualificationOutcome::Qualified(CssRubyAlignValue::Start)
        }
        ExpectedOutcome::Center => {
            CssRubyAlignQualificationOutcome::Qualified(CssRubyAlignValue::Center)
        }
        ExpectedOutcome::SpaceBetween => {
            CssRubyAlignQualificationOutcome::Qualified(CssRubyAlignValue::SpaceBetween)
        }
        ExpectedOutcome::SpaceAround => {
            CssRubyAlignQualificationOutcome::Qualified(CssRubyAlignValue::SpaceAround)
        }
        ExpectedOutcome::Invalid => {
            CssRubyAlignQualificationOutcome::InvalidForSelectedValueGrammar
        }
        ExpectedOutcome::UnsupportedCssWide => {
            CssRubyAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyAlignUnsupportedReason::CssWideKeyword,
            )
        }
        ExpectedOutcome::UnsupportedFunction => {
            CssRubyAlignQualificationOutcome::UnsupportedBySelectedValueProfile(
                CssRubyAlignUnsupportedReason::FunctionValue,
            )
        }
    }
}
fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    let actual: Vec<_> = result
        .ruby_align_observations()
        .iter()
        .map(|o| o.outcome())
        .collect();
    let expected: Vec<_> = expected.iter().copied().map(expected_outcome).collect();
    assert_eq!(actual, expected);
}

#[test]
fn normative_direct_grammar_matches_pinned_wpt_and_rejects_mismatches() {
    let result = qualify(
        3191,
        concat!(
            "a{ruby-align:start;}",
            "b{ruby-align:center;}",
            "c{ruby-align:space-between;}",
            "d{ruby-align:space-around;}",
            "e{ruby-align:auto;}",
            "f{ruby-align:start center;}",
            "g{ruby-align:;}",
            "h{ruby-align:10px;}"
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::Start,
            ExpectedOutcome::Center,
            ExpectedOutcome::SpaceBetween,
            ExpectedOutcome::SpaceAround,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
        ],
    );
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn case_escapes_comments_and_priority_preserve_keyword_meaning() {
    let result = qualify(
        3192,
        concat!(
            "a{RUBY-ALIGN:SpAcE-ArOuNd;}",
            r"b{ruby-align:\73 tart;}",
            r"c{ruby-\61 lign:space-between;}",
            "d{ruby-align:/**/center/**/!important;}"
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::SpaceAround,
            ExpectedOutcome::Start,
            ExpectedOutcome::SpaceBetween,
            ExpectedOutcome::Center,
        ],
    );
    assert!(
        result.upstream_parser_result().occurrences()[3]
            .priority()
            .is_some()
    );
}

#[test]
fn css_wide_keywords_remain_profile_unsupported() {
    let result = qualify(
        3193,
        concat!(
            "a{ruby-align:initial;}",
            "b{ruby-align:inherit;}",
            "c{ruby-align:unset;}",
            "d{ruby-align:revert;}",
            "e{ruby-align:revert-layer;}",
            "f{ruby-align:revert-rule;}"
        ),
    );
    assert_expected(&result, &[ExpectedOutcome::UnsupportedCssWide; 6]);
}

#[test]
fn function_boundaries_preserve_existing_single_keyword_profile() {
    let result = qualify(
        3194,
        concat!(
            "a{ruby-align:var(--align);}",
            "b{ruby-align:env(align);}",
            "c{ruby-align:attr(data-align);}",
            "d{ruby-align:--align();}",
            "e{ruby-align:first-valid(start,center);}",
            "f{ruby-align:cycle(start,center);}",
            "g{ruby-align:interpolate(0%,0:start,1:center);}",
            "h{ruby-align:foo();}",
            "i{ruby-align:calc(1);}",
            "j{ruby-align:start var(--align);}",
            "k{ruby-align:start first-valid(center);}"
        ),
    );
    assert_expected(
        &result,
        &[
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::Invalid,
            ExpectedOutcome::UnsupportedFunction,
            ExpectedOutcome::Invalid,
        ],
    );
}

#[test]
fn applicability_is_not_an_input_and_sibling_ruby_leaves_remain_distinct() {
    let result = qualify(
        3195,
        concat!(
            "ruby{ruby-align:center;}",
            "div{ruby-align:center;}",
            "svg{ruby-align:center;}",
            "a{ruby-merge:merge;}",
            "b{ruby-position:alternate over;}",
            "c{ruby-overhang:spaces;}"
        ),
    );
    assert_expected(&result, &[ExpectedOutcome::Center; 3]);
    assert_eq!(result.ruby_merge_observations().len(), 1);
    assert_eq!(result.ruby_position_observations().len(), 1);
    assert_eq!(result.ruby_overhang_observations().len(), 1);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(3196, "a{ruby-align:start;}b{ruby-align:start;}");
    assert_expected(&result, &[ExpectedOutcome::Start, ExpectedOutcome::Start]);
    assert_eq!(result.ruby_align_observations()[0].occurrence_index(), 0);
    assert_eq!(result.ruby_align_observations()[1].occurrence_index(), 1);
    assert_ne!(
        result.ruby_align_observations()[0].placement().context_id(),
        result.ruby_align_observations()[1].placement().context_id(),
    );
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [
        (3197, "@font-face{ruby-align:center;}"),
        (3198, "@page{ruby-align:center;}"),
    ] {
        let result = qualify(source_id, css);
        assert!(result.ruby_align_observations().is_empty());
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix() {
    let result = qualify_with_limits(
        3199,
        "a{ruby-align:start;}b{ruby-align:center;}",
        parser_limits_with_occurrences(1),
    );
    assert_expected(&result, &[ExpectedOutcome::Start]);
    assert_ne!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = "a{ruby-align:space-between;}b{ruby-align:space-around;}";
    let first = qualify(3200, css);
    let second = qualify(3200, css);
    let other_source = qualify(3201, css);
    let expected = [ExpectedOutcome::SpaceBetween, ExpectedOutcome::SpaceAround];
    assert_expected(&first, &expected);
    assert_expected(&second, &expected);
    assert_expected(&other_source, &expected);
}
