//! Candidate-independent validation of the #140/#172 Core-integrated
//! `analyze_css_source` operation against the existing #138/#139
//! parser gold corpus and generated corpus.
//!
//! This is an additional validation layer, not a replacement for the
//! existing direct parser gate in [`super::parser_conformance_tests`]: that
//! module proves `css::parser::producer::run` against a caller-supplied
//! `CssTokenizerRunResult`, while this module proves the complete
//! `SourceText -> analyze_css_source(...)` Core boundary, including
//! the additional source-evidence/relationship validation #140 owns.
//!
//! Per the #140 architecture clarification, `CSS-PARSER-LIFECYCLE-
//! TOKENIZER-INCOMPLETE-001` is a parser-internal synthetic-upstream
//! lifecycle fixture (its own documentation states a normal generous
//! tokenizer run over its source would complete) and is intentionally
//! excluded from this source-driven Core gate rather than routed through a
//! Core-operation tokenizer-result injection seam; its parser-level
//! validation remains authoritative through the existing direct gate in
//! [`super::parser_conformance_tests`]. This module adds no new fixtures,
//! derives no expected data from production output, and modifies no
//! existing gold.

use crate::css::analysis::analyze_css_source;
use crate::css::parser::producer::run as run_parser;
use crate::css::tokenizer::producer::run as run_tokenizer;
use crate::{SourceId, SourceText};

use super::generated::{assert_deterministic_replay, generated_inputs};
use super::parser_candidate::{
    assert_matches_gold, canonical_signature, generous_parser_limits, generous_tokenizer_limits,
};
use super::parser_fixtures::normative_parser_fixtures;

const SYNTHETIC_LIFECYCLE_FIXTURE_ID: &str = "CSS-PARSER-LIFECYCLE-TOKENIZER-INCOMPLETE-001";

#[test]
fn core_operation_matches_every_source_driven_normative_parser_gold_fixture() {
    let fixtures = normative_parser_fixtures();
    assert_eq!(fixtures.len(), 36);

    let mut executed = 0usize;
    for fixture in &fixtures {
        if fixture.id == SYNTHETIC_LIFECYCLE_FIXTURE_ID {
            continue;
        }
        executed += 1;

        let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
        let result = analyze_css_source(
            &source,
            generous_tokenizer_limits(),
            generous_parser_limits(),
        )
        .unwrap_or_else(|error| panic!("{}: Core operation error {error:?}", fixture.id));

        assert_matches_gold(fixture, &result);
    }
    assert_eq!(
        executed, 35,
        "source-driven Core normative subset must be exactly 35"
    );
}

#[test]
fn core_operation_handles_all_generated_inputs_without_panic_and_preserves_invariants() {
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), 4_096);

    for (index, source) in inputs.iter().enumerate() {
        let primary = SourceText::new(SourceId::new(1), source.clone());
        let result = analyze_css_source(
            &primary,
            generous_tokenizer_limits(),
            generous_parser_limits(),
        )
        .unwrap_or_else(|error| panic!("generated input {index}: Core operation error {error:?}"));

        let upstream_terminal = result
            .upstream_tokenizer_result()
            .terminal()
            .range()
            .start();
        assert!(
            result.terminal().range().start() <= upstream_terminal,
            "generated input {index}: parser terminal exceeds tokenizer terminal"
        );

        let mut previous_end: Option<usize> = None;
        for occurrence in result.occurrences() {
            assert_eq!(
                occurrence.complete().source_id(),
                primary.id(),
                "generated input {index}: occurrence source identity mismatch"
            );
            if let Some(previous_end) = previous_end {
                assert!(
                    occurrence.complete().range().start() >= previous_end,
                    "generated input {index}: occurrence ordering moved backward"
                );
            }
            previous_end = Some(occurrence.complete().range().end());
        }
    }
}

#[test]
fn core_operation_is_deterministic_across_repeats_and_source_ids() {
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), 4_096);

    assert_deterministic_replay(&inputs, |source| {
        let text = SourceText::new(SourceId::new(1), source.to_owned());
        let result =
            analyze_css_source(&text, generous_tokenizer_limits(), generous_parser_limits())
                .expect("generated input must analyze cleanly");
        canonical_signature(&result)
    });

    for source in &inputs {
        let first_text = SourceText::new(SourceId::new(11), source.clone());
        let first = analyze_css_source(
            &first_text,
            generous_tokenizer_limits(),
            generous_parser_limits(),
        )
        .unwrap();

        let second_text = SourceText::new(SourceId::new(22), source.clone());
        let second = analyze_css_source(
            &second_text,
            generous_tokenizer_limits(),
            generous_parser_limits(),
        )
        .unwrap();

        assert_eq!(
            canonical_signature(&first),
            canonical_signature(&second),
            "source {source:?}"
        );
    }
}

/// Proves the Core operation is only an integration validator, not a
/// semantic transformer: for identical source/limits, the direct
/// `tokenizer::producer::run -> parser::producer::run` pipeline and
/// `analyze_css_source` must carry equivalent parser-owned meaning.
/// Pointer/allocation identity is never required.
#[test]
fn core_operation_matches_direct_pipeline_meaning_for_all_generated_inputs() {
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), 4_096);

    for source in &inputs {
        let direct_text = SourceText::new(SourceId::new(1), source.clone());
        let tokenizer_result = run_tokenizer(&direct_text, generous_tokenizer_limits()).unwrap();
        let direct = run_parser(&direct_text, tokenizer_result, generous_parser_limits()).unwrap();

        let core_text = SourceText::new(SourceId::new(1), source.clone());
        let core = analyze_css_source(
            &core_text,
            generous_tokenizer_limits(),
            generous_parser_limits(),
        )
        .unwrap();

        assert_eq!(
            canonical_signature(&direct),
            canonical_signature(&core),
            "source {source:?}"
        );
    }
}
