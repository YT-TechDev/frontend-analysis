use crate::css::analysis::analyze_css_source;
use crate::css::parser::resource::CssParserLimits;
use crate::css::parser::result::CssParserExecutionCompletion;
use crate::css::tokenizer::resource::CssTokenizerLimits;
use crate::css::value_qualification::{
    CssRubyPositionComponent, CssRubyPositionQualificationOutcome,
    CssRubyPositionUnsupportedReason, CssRubyPositionValue, CssValueQualificationRunResult,
    run,
};
use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy)]
enum ExpectedOutcome {
    Components(&'static [CssRubyPositionComponent]),
    InterCharacter,
    Invalid,
    UnsupportedCssWide,
    UnsupportedDeferred,
    UnsupportedWholeValue,
}

fn tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(4096, 100_000, 8192, 1024, 8192, 8192).unwrap()
}
fn parser_limits() -> CssParserLimits { parser_limits_with_occurrences(8192) }
fn parser_limits_with_occurrences(max_declaration_occurrences: usize) -> CssParserLimits {
    CssParserLimits::new(100_000, 256, 256, max_declaration_occurrences, 1024, 1024, 1024, 1024, 8192).unwrap()
}
fn qualify(source_id: u64, css: &str) -> CssValueQualificationRunResult {
    qualify_with_limits(source_id, css, parser_limits())
}
fn qualify_with_limits(source_id: u64, css: &str, parser_limits: CssParserLimits) -> CssValueQualificationRunResult {
    let source = SourceText::new(SourceId::new(source_id), css.to_owned());
    let parser_result = analyze_css_source(&source, tokenizer_limits(), parser_limits).unwrap();
    run(parser_result).unwrap()
}
fn assert_expected(result: &CssValueQualificationRunResult, expected: &[ExpectedOutcome]) {
    assert_eq!(result.ruby_position_observations().len(), expected.len());
    for (observation, expected) in result.ruby_position_observations().iter().zip(expected) {
        match (observation.outcome(), expected) {
            (CssRubyPositionQualificationOutcome::Qualified(CssRubyPositionValue::Components(actual)), ExpectedOutcome::Components(expected)) => assert_eq!(actual.authored_components(), *expected),
            (CssRubyPositionQualificationOutcome::Qualified(CssRubyPositionValue::InterCharacter), ExpectedOutcome::InterCharacter) => {}
            (CssRubyPositionQualificationOutcome::InvalidForSelectedValueGrammar, ExpectedOutcome::Invalid) => {}
            (CssRubyPositionQualificationOutcome::UnsupportedBySelectedValueProfile(CssRubyPositionUnsupportedReason::CssWideKeyword), ExpectedOutcome::UnsupportedCssWide) => {}
            (CssRubyPositionQualificationOutcome::UnsupportedBySelectedValueProfile(CssRubyPositionUnsupportedReason::DeferredSubstitutionFunction), ExpectedOutcome::UnsupportedDeferred) => {}
            (CssRubyPositionQualificationOutcome::UnsupportedBySelectedValueProfile(CssRubyPositionUnsupportedReason::WholeValueFunction), ExpectedOutcome::UnsupportedWholeValue) => {}
            (actual, expected) => panic!("unexpected outcome: {actual:?}, expected {expected:?}"),
        }
    }
}

const ALT: &[CssRubyPositionComponent] = &[CssRubyPositionComponent::Alternate];
const OVER: &[CssRubyPositionComponent] = &[CssRubyPositionComponent::Over];
const UNDER: &[CssRubyPositionComponent] = &[CssRubyPositionComponent::Under];
const ALT_OVER: &[CssRubyPositionComponent] = &[CssRubyPositionComponent::Alternate, CssRubyPositionComponent::Over];
const OVER_ALT: &[CssRubyPositionComponent] = &[CssRubyPositionComponent::Over, CssRubyPositionComponent::Alternate];
const ALT_UNDER: &[CssRubyPositionComponent] = &[CssRubyPositionComponent::Alternate, CssRubyPositionComponent::Under];
const UNDER_ALT: &[CssRubyPositionComponent] = &[CssRubyPositionComponent::Under, CssRubyPositionComponent::Alternate];

#[test]
fn normative_direct_grammar_preserves_authored_order_and_rejects_stale_wpt_auto() {
    let result = qualify(3180, concat!(
        "a{ruby-position:alternate;}", "b{ruby-position:over;}", "c{ruby-position:under;}",
        "d{ruby-position:alternate over;}", "e{ruby-position:over alternate;}",
        "f{ruby-position:alternate under;}", "g{ruby-position:under alternate;}",
        "h{ruby-position:inter-character;}", "i{ruby-position:auto;}", "j{ruby-position:center;}"
    ));
    assert_expected(&result, &[
        ExpectedOutcome::Components(ALT), ExpectedOutcome::Components(OVER), ExpectedOutcome::Components(UNDER),
        ExpectedOutcome::Components(ALT_OVER), ExpectedOutcome::Components(OVER_ALT),
        ExpectedOutcome::Components(ALT_UNDER), ExpectedOutcome::Components(UNDER_ALT),
        ExpectedOutcome::InterCharacter, ExpectedOutcome::Invalid, ExpectedOutcome::Invalid,
    ]);
    assert_eq!(result.execution_completion(), CssParserExecutionCompletion::Complete);
}

#[test]
fn slot_collisions_and_standalone_mixing_are_invalid() {
    let result = qualify(3181, concat!(
        "a{ruby-position:alternate alternate;}", "b{ruby-position:over under;}",
        "c{ruby-position:under over;}", "d{ruby-position:inter-character alternate;}",
        "e{ruby-position:over inter-character;}", "f{ruby-position:alternate over under;}",
        "g{ruby-position:;}", "h{ruby-position:10px;}"
    ));
    assert_expected(&result, &[ExpectedOutcome::Invalid; 8]);
}

#[test]
fn case_escapes_comments_and_priority_preserve_decoded_authored_components() {
    let result = qualify(3182, concat!(
        "a{RUBY-POSITION:AlTeRnAtE OvEr;}", r"b{ruby-position:\61 lternate under;}",
        r"c{ruby-\70 osition:over alternate;}", "d{ruby-position:/**/inter-character/**/!important;}"
    ));
    assert_expected(&result, &[
        ExpectedOutcome::Components(ALT_OVER), ExpectedOutcome::Components(ALT_UNDER),
        ExpectedOutcome::Components(OVER_ALT), ExpectedOutcome::InterCharacter,
    ]);
    assert!(result.upstream_parser_result().occurrences()[3].priority().is_some());
}

#[test]
fn css_wide_deferred_and_whole_value_boundaries_fail_open_without_upgrading_mixed_functions() {
    let result = qualify(3183, concat!(
        "a{ruby-position:initial;}", "b{ruby-position:inherit;}", "c{ruby-position:revert-layer;}",
        "d{ruby-position:var(--p);}", "e{ruby-position:alternate var(--p);}", "f{ruby-position:--position();}",
        "g{ruby-position:first-valid(over,under);}", "h{ruby-position:cycle(over,under);}",
        "i{ruby-position:interpolate(0%,0:over,1:under);}", "j{ruby-position:alternate first-valid(over);}",
        "k{ruby-position:foo();}", "l{ruby-position:calc(1);}"
    ));
    assert_expected(&result, &[
        ExpectedOutcome::UnsupportedCssWide, ExpectedOutcome::UnsupportedCssWide, ExpectedOutcome::UnsupportedCssWide,
        ExpectedOutcome::UnsupportedDeferred, ExpectedOutcome::UnsupportedDeferred, ExpectedOutcome::UnsupportedDeferred,
        ExpectedOutcome::UnsupportedWholeValue, ExpectedOutcome::UnsupportedWholeValue, ExpectedOutcome::UnsupportedWholeValue,
        ExpectedOutcome::Invalid, ExpectedOutcome::Invalid, ExpectedOutcome::Invalid,
    ]);
}

#[test]
fn applicability_is_not_an_input_and_accepted_sibling_leaves_remain_distinct() {
    let result = qualify(3184, concat!(
        "ruby{ruby-position:alternate over;}", "div{ruby-position:alternate over;}", "svg{ruby-position:alternate over;}",
        "a{ruby-merge:merge;}", "b{ruby-overhang:spaces;}", "c{contain:size layout;}",
        "d{text-transform:uppercase full-width;}", "e{text-emphasis-position:over left;}"
    ));
    assert_expected(&result, &[
        ExpectedOutcome::Components(ALT_OVER), ExpectedOutcome::Components(ALT_OVER), ExpectedOutcome::Components(ALT_OVER),
    ]);
    assert_eq!(result.ruby_merge_observations().len(), 1);
    assert_eq!(result.ruby_overhang_observations().len(), 1);
    assert_eq!(result.contain_observations().len(), 1);
    assert_eq!(result.text_transform_observations().len(), 1);
    assert_eq!(result.text_emphasis_position_observations().len(), 1);
}

#[test]
fn duplicate_declarations_keep_distinct_run_local_placement() {
    let result = qualify(3185, "a{ruby-position:over;}b{ruby-position:over;}");
    assert_expected(&result, &[ExpectedOutcome::Components(OVER), ExpectedOutcome::Components(OVER)]);
    assert_eq!(result.ruby_position_observations()[0].occurrence_index(), 0);
    assert_eq!(result.ruby_position_observations()[1].occurrence_index(), 1);
    assert_ne!(result.ruby_position_observations()[0].placement().context_id(), result.ruby_position_observations()[1].placement().context_id());
}

#[test]
fn nonordinary_declaration_shaped_contexts_are_excluded() {
    for (source_id, css) in [(3186, "@font-face{ruby-position:over;}"), (3187, "@page{ruby-position:over;}")] {
        let result = qualify(source_id, css);
        assert!(result.ruby_position_observations().is_empty());
    }
}

#[test]
fn parser_resource_stop_preserves_committed_prefix() {
    let result = qualify_with_limits(3188, "a{ruby-position:alternate;}b{ruby-position:under;}", parser_limits_with_occurrences(1));
    assert_expected(&result, &[ExpectedOutcome::Components(ALT)]);
    assert_ne!(result.execution_completion(), CssParserExecutionCompletion::Complete);
}

#[test]
fn repeated_and_cross_source_runs_are_semantically_deterministic() {
    let css = "a{ruby-position:under alternate;}b{ruby-position:inter-character;}";
    let first = qualify(3189, css);
    let second = qualify(3189, css);
    let other_source = qualify(3190, css);
    let expected = [ExpectedOutcome::Components(UNDER_ALT), ExpectedOutcome::InterCharacter];
    assert_expected(&first, &expected);
    assert_expected(&second, &expected);
    assert_expected(&other_source, &expected);
}
