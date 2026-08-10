//! #139 production parser wiring against the #138 candidate-independent
//! parser gold.
//!
//! Gold remains independent: fixtures are authored in
//! `parser_fixtures.rs`/`parser_gold.rs` without reference to production
//! output, and this module only reads production results to compare them
//! against that already-fixed gold.

use super::generated::{assert_deterministic_replay, generated_inputs};
use super::gold::GoldRange;
use super::parser_candidate::{
    assert_matches_gold, canonical_signature, run_fixture,
    run_fixture_with_synthetic_upstream_incomplete,
};
use super::parser_fixtures::normative_parser_fixtures;
use super::parser_gold::{ParserGoldFixture, ParserGoldGroup};

use crate::css::parser::evidence::CssParserRecoveryTermination;
use crate::css::parser::producer::run as run_parser;
use crate::css::parser::result::{CssParserExecutionCompletion, CssParserTermination};
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::{SourceId, SourceText};

use super::parser_candidate::{generous_parser_limits, generous_tokenizer_limits};

#[test]
fn production_parser_matches_every_normative_parser_gold_fixture() {
    let fixtures = normative_parser_fixtures();
    assert_eq!(fixtures.len(), 36);

    for fixture in &fixtures {
        let id = fixture.id;
        let result = if id == "CSS-PARSER-LIFECYCLE-TOKENIZER-INCOMPLETE-001" {
            run_fixture_with_synthetic_upstream_incomplete(fixture)
        } else {
            run_fixture(fixture)
        };
        assert_matches_gold(fixture, &result);
    }
}

#[test]
fn historical_escaped_property_provenance_regression_matches_exactly() {
    let source = SourceText::new(SourceId::new(50_001), r"a{c\6F lor/**/:red;}".to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits()).unwrap();
    let result = run_parser(&source, tokenizer_result, generous_parser_limits()).unwrap();

    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(
        (
            occurrence.complete().range().start(),
            occurrence.complete().range().end()
        ),
        (2, 19)
    );
    assert_eq!(
        (
            occurrence.property_name().range().start(),
            occurrence.property_name().range().end()
        ),
        (2, 10)
    );
    assert_eq!(
        (
            occurrence.colon().range().start(),
            occurrence.colon().range().end()
        ),
        (14, 15)
    );
    assert_eq!(
        (
            occurrence.value().range().start(),
            occurrence.value().range().end()
        ),
        (15, 18)
    );
    assert!(occurrence.priority().is_none());
}

#[test]
fn historical_silent_priority_loss_regression_matches_exactly_with_no_byte_loss() {
    let source = SourceText::new(
        SourceId::new(50_002),
        r"a{color:red !/**/Im\70 ortant;}".to_owned(),
    );
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits()).unwrap();
    let result = run_parser(&source, tokenizer_result, generous_parser_limits()).unwrap();

    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(
        (
            occurrence.complete().range().start(),
            occurrence.complete().range().end()
        ),
        (2, 30)
    );
    assert_eq!(
        (
            occurrence.value().range().start(),
            occurrence.value().range().end()
        ),
        (8, 11)
    );
    let priority = occurrence
        .priority()
        .expect("valid priority evidence expected");
    assert_eq!(
        (
            priority.complete().range().start(),
            priority.complete().range().end()
        ),
        (12, 29)
    );
    assert_eq!(
        (
            priority.bang().range().start(),
            priority.bang().range().end()
        ),
        (12, 13)
    );
    assert_eq!(
        (
            priority.important_ident().range().start(),
            priority.important_ident().range().end()
        ),
        (17, 29)
    );
    assert_eq!(
        occurrence.complete().range().end() - occurrence.complete().range().start(),
        28
    );
}

fn parse(source_id: u64, text: &str) -> crate::css::parser::result::CssParserRunResult {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    let tokenizer_result = run_tokenizer(&source, generous_tokenizer_limits()).unwrap();
    run_parser(&source, tokenizer_result, generous_parser_limits()).unwrap()
}

#[test]
fn production_parser_terminates_deterministically_over_generated_corpus() {
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), 4_096);

    for source in &inputs {
        let text = SourceText::new(SourceId::new(0), source.clone());
        let tokenizer_result =
            run_tokenizer(&text, generous_tokenizer_limits()).unwrap_or_else(|error| {
                panic!("generated input {source:?} tokenizer error: {error:?}")
            });
        let result =
            run_parser(&text, tokenizer_result, generous_parser_limits()).unwrap_or_else(|error| {
                panic!("generated input {source:?} produced a parser run error: {error:?}")
            });
        drop(result);
    }

    assert_deterministic_replay(&inputs, |source| {
        let text = SourceText::new(SourceId::new(1), source.to_owned());
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits())
            .expect("generated input must tokenize cleanly");
        let result = run_parser(&text, tokenizer_result, generous_parser_limits())
            .expect("generated input must parse cleanly");
        canonical_signature(&result)
    });
}

#[test]
fn production_parser_is_deterministic_across_distinct_source_ids() {
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), 4_096);
    for source in &inputs {
        let first_text = SourceText::new(SourceId::new(11), source.clone());
        let first_tokenizer = run_tokenizer(&first_text, generous_tokenizer_limits()).unwrap();
        let first = run_parser(&first_text, first_tokenizer, generous_parser_limits()).unwrap();

        let second_text = SourceText::new(SourceId::new(22), source.clone());
        let second_tokenizer = run_tokenizer(&second_text, generous_tokenizer_limits()).unwrap();
        let second = run_parser(&second_text, second_tokenizer, generous_parser_limits()).unwrap();

        assert_eq!(
            canonical_signature(&first),
            canonical_signature(&second),
            "source {source:?}"
        );
    }
}

// ---------------------------------------------------------------------
// Focused rollback / provenance regressions beyond gold comparison.
// ---------------------------------------------------------------------

/// #167 supersedes the old whole-remainder outcome: `color:green{...}`
/// structurally rolls back declaration recognition, then the qualified-rule
/// fallback recognizes "color:green" as a nested rule prelude and retains a
/// real child context, so the inner "color:blue" declaration is a supported
/// occurrence with zero leaked declaration diagnostic/recovery evidence.
#[test]
fn declaration_shaped_start_that_becomes_a_nested_rule_leaves_zero_speculative_evidence() {
    let result = parse(60_001, "a{color:green{color:blue;}}");
    assert_eq!(result.occurrences().len(), 1);
    assert!(result.parser_diagnostics().is_empty());
    assert!(result.recovery_records().is_empty());
    assert!(result.unsupported_regions().is_empty());
    assert_eq!(result.context_records().len(), 2);
}

/// #167 supersedes the old whole-remainder outcome: the nested qualified
/// rule is now a real child context, so all three declarations (outer,
/// child, outer) are supported occurrences.
#[test]
fn declaration_then_nested_rule_then_declaration_retains_all_three_occurrences() {
    let result = parse(60_002, "a{color:red;b{color:blue;}color:green;}");
    assert_eq!(result.occurrences().len(), 3);
    assert!(result.unsupported_regions().is_empty());
    assert_eq!(result.context_records().len(), 2);
}

/// #167 top-level retained qualified-rule context ordinals stay
/// source-relative across other materialized root items: a committed
/// top-level unsupported at-rule occupies the unretained root item position
/// between `a` and `b`, so `b` receives root ordinal 2, not 1.
#[test]
fn top_level_at_rule_between_qualified_rules_advances_root_ordinal() {
    let result = parse(60_011, "a{}@media{}b{}");
    assert_eq!(result.unsupported_regions().len(), 1);
    assert_eq!(result.context_records().len(), 2);

    let a = &result.context_records()[0];
    assert!(a.parent().is_none());
    assert_eq!(a.item_ordinal().value(), 0);

    let b = &result.context_records()[1];
    assert!(b.parent().is_none());
    assert_eq!(b.item_ordinal().value(), 2);
}

#[test]
fn declaration_then_nested_at_rule_produces_exactly_one_supported_occurrence() {
    let result = parse(60_003, "a{color:red;@media screen{color:blue;}}");
    assert_eq!(result.occurrences().len(), 1);
    assert_eq!(result.unsupported_regions().len(), 1);
}

#[test]
fn missing_colon_then_valid_declaration_recovers_and_continues() {
    let result = parse(60_004, "a{color red;background:blue;}");
    assert_eq!(result.occurrences().len(), 1);
    assert_eq!(result.parser_diagnostics().len(), 1);
    assert_eq!(result.recovery_records().len(), 1);
}

#[test]
fn malformed_start_then_valid_declaration_recovers_and_continues() {
    let result = parse(60_005, "a{:red;color:blue;}");
    assert_eq!(result.occurrences().len(), 1);
    assert_eq!(result.parser_diagnostics().len(), 1);
    assert_eq!(result.recovery_records().len(), 1);
}

#[test]
fn noncustom_sole_brace_value_is_a_declaration() {
    let result = parse(60_006, "a{color:{red};}");
    assert_eq!(result.occurrences().len(), 1);
    assert!(result.unsupported_regions().is_empty());
}

#[test]
fn custom_property_brace_value_is_always_a_declaration() {
    let result = parse(60_007, "a{--x:{a:b};}");
    assert_eq!(result.occurrences().len(), 1);
    assert!(result.unsupported_regions().is_empty());
}

// ---------------------------------------------------------------------
// #158 discard / #159 EOF-recovery focused production regressions.
// ---------------------------------------------------------------------

#[test]
fn top_level_custom_property_like_prelude_produces_exactly_one_discard_record() {
    let result = parse(60_008, "--foo:bar{color:red;}");
    assert!(result.occurrences().is_empty());
    assert!(result.parser_diagnostics().is_empty());
    assert!(result.recovery_records().is_empty());
    assert!(result.unsupported_regions().is_empty());
    assert_eq!(result.discard_records().len(), 1);

    let discard = &result.discard_records()[0];
    assert_eq!(discard.region().range(), result_range(0, 21));
    assert_eq!(discard.property_name().range(), result_range(0, 5));
    assert_eq!(discard.colon().range(), result_range(5, 6));
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn malformed_block_item_at_true_eof_commits_diagnostic_and_eof_recovery() {
    let result = parse(60_009, "a{color red");
    assert!(result.occurrences().is_empty());
    assert!(result.discard_records().is_empty());
    assert_eq!(result.parser_diagnostics().len(), 1);
    assert_eq!(result.recovery_records().len(), 1);

    let recovery = &result.recovery_records()[0];
    assert_eq!(recovery.region().range(), result_range(2, 11));
    match recovery.termination() {
        CssParserRecoveryTermination::EndOfInput { terminal } => {
            assert_eq!(terminal.range(), result_range(11, 11));
        }
        other => panic!("expected EndOfInput termination, got {other:?}"),
    }
    assert_eq!(result.terminal().range(), result_range(11, 11));
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

#[test]
fn upstream_resource_limited_terminal_coinciding_with_source_end_is_not_true_eof_recovery() {
    // A synthetic upstream tokenizer result whose Incomplete/ResourceLimit
    // terminal numerically coincides with true source end must never be
    // treated as genuine EndOfInput: no EOF recovery, no diagnostic, and no
    // occurrence may be fabricated from it.
    let fixture = ParserGoldFixture::upstream_incomplete(
        "CSS-PARSER-REGRESSION-RESOURCE-LIMIT-NOT-TRUE-EOF-001",
        ParserGoldGroup::Lifecycle,
        60_010,
        "a{color red",
        11,
        GoldRange::new(11, 11),
    );
    let result = run_fixture_with_synthetic_upstream_incomplete(&fixture);

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert!(matches!(
        result.termination(),
        CssParserTermination::UpstreamTokenizerIncomplete
    ));
    assert!(result.recovery_records().is_empty());
    assert!(result.parser_diagnostics().is_empty());
    assert!(result.occurrences().is_empty());
    assert!(result.discard_records().is_empty());
    assert_eq!(result.terminal().range(), result_range(11, 11));
}

// ---------------------------------------------------------------------
// Input / source provenance: every declaration endpoint must be an exact
// raw byte offset, never rediscovered by scanning. These are covered
// implicitly by gold comparison above; the tests below assert the same
// byte-exactness directly and explicitly per provenance category.
// ---------------------------------------------------------------------

#[test]
fn bom_before_first_rule_preserves_exact_post_bom_offsets() {
    let result = parse(61_001, "\u{feff}a{color:red;}");
    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(occurrence.property_name().range().start(), 5);
    assert_eq!(occurrence.property_name().range().end(), 10);
}

#[test]
fn cr_lf_crlf_ff_do_not_shift_neighboring_declaration_endpoints() {
    let result = parse(61_002, "a{a1:1;\ra2:2;\na3:3;\r\na4:4;\x0ca5:5;}");
    assert_eq!(result.occurrences().len(), 5);
    let expected_starts = [2, 8, 14, 21, 27];
    for (occurrence, expected_start) in result.occurrences().iter().zip(expected_starts) {
        assert_eq!(occurrence.complete().range().start(), expected_start);
    }
}

#[test]
fn nul_byte_in_value_yields_ordinary_byte_exact_evidence() {
    let result = parse(61_003, "a{color:\0;}");
    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(occurrence.value().range(), result_range(8, 9));
}

#[test]
fn multibyte_utf8_property_and_value_preserve_exact_byte_ranges() {
    let result = parse(61_004, "a{--é:中;}");
    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(occurrence.property_name().range(), result_range(2, 6));
    assert_eq!(occurrence.value().range(), result_range(7, 10));
}

#[test]
fn escaped_property_name_preserves_authored_spelling_range() {
    let result = parse(61_005, r"a{c\6F lor/**/:red;}");
    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(occurrence.property_name().fragment(), r"c\6F lor");
}

#[test]
fn comment_between_property_name_and_colon_stays_inside_complete_only() {
    let result = parse(61_006, "a{color/*gap*/:red;}");
    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(occurrence.property_name().fragment(), "color");
    assert_eq!(occurrence.colon().range().start(), 14);
}

#[test]
fn comment_inside_priority_stays_inside_priority_complete() {
    let result = parse(61_007, "a{color:red !/*gap*/important;}");
    assert_eq!(result.occurrences().len(), 1);
    let priority = result.occurrences()[0]
        .priority()
        .expect("valid priority expected");
    assert_eq!(priority.bang().fragment(), "!");
    assert_eq!(priority.important_ident().fragment(), "important");
}

fn result_range(start: usize, end: usize) -> crate::SourceRange {
    // `SourceRange` has no public constructor; derive one from a throwaway
    // anchor purely to compare `(start, end)` pairs by value in these tests.
    let text = SourceText::new(SourceId::new(u64::MAX), " ".repeat(end.max(1)));
    text.anchor(start, end).unwrap().range()
}
