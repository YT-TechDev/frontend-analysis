//! Connects the real production tokenizer to the #112 observation and
//! comparison layer, and runs the deterministic bounded generator against it.
//!
//! This module performs no source rescanning and repairs no observation: it
//! only executes `producer::tokenize` and hands the result to the existing
//! one-way observation adapter and structural comparator.

use crate::{SourceId, SourceText};

use super::super::producer::tokenize;
use super::super::resource::HtmlTokenizerLimits;
use super::compare::compare;
use super::corpus::all_candidate_independent_corpus;
use super::expected::{Limits, ObservedRun};
use super::fixture::HtmlTokenizerFixture;
use super::generated::{MAX_GENERATED_CASES, MAX_SOURCE_BYTES, generated_inputs};
use super::observe::observe;

fn to_html_limits(limits: Limits) -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(
        limits.source_bytes,
        limits.transition_steps,
        limits.emitted_tokens,
        limits.diagnostics,
        limits.attributes_per_tag,
        limits.retained_interpreted_bytes,
        limits.temporary_buffer_bytes,
    )
}

fn run_fixture(fixture: &HtmlTokenizerFixture, source_id: u64) -> ObservedRun {
    let text =
        String::from_utf8(fixture.source_bytes.to_vec()).expect("fixture source is valid UTF-8");
    let source = SourceText::new(SourceId::new(source_id), text);
    let limits = to_html_limits(fixture.expected.0.limits);
    let result = tokenize(&source, limits);
    observe(&source, &result)
}

#[test]
fn all_candidate_independent_fixtures_match_the_production_tokenizer() {
    let fixtures = all_candidate_independent_corpus();
    const INITIAL_CORPUS_COUNT: usize = 72;
    const SUPPLEMENTAL_REGRESSION_COUNT: usize = 4;
    assert_eq!(
        fixtures.len(),
        INITIAL_CORPUS_COUNT + SUPPLEMENTAL_REGRESSION_COUNT
    );
    let mut mismatches = Vec::new();
    for fixture in &fixtures {
        let observed = run_fixture(fixture, 1);
        if let Err(mismatch) = compare(fixture.id, &fixture.expected, &observed) {
            mismatches.push(mismatch.to_string());
        }
    }
    assert!(
        mismatches.is_empty(),
        "candidate/gold mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn fixture_observations_are_deterministic_across_repeats_and_source_ids() {
    // Equality checked with plain `if`, not `assert_eq!`: `ObservedRun`
    // embeds authored source bytes and interpreted source-derived strings,
    // and `assert_eq!` would print both full Debug payloads on failure.
    // Only the fixture ID is reported.
    let fixtures = all_candidate_independent_corpus();
    for fixture in &fixtures {
        let first = run_fixture(fixture, 101);
        let second = run_fixture(fixture, 101);
        let third = run_fixture(fixture, 101);
        let alternate_source_id = run_fixture(fixture, 202);
        if first != second {
            panic!("{}: run 1 vs run 2 differ", fixture.id);
        }
        if second != third {
            panic!("{}: run 2 vs run 3 differ", fixture.id);
        }
        if first != alternate_source_id {
            panic!(
                "{}: SourceId must not change semantic observation",
                fixture.id
            );
        }
    }
}

#[test]
fn generated_inputs_terminate_without_panic_and_are_deterministic() {
    let limits = to_html_limits(Limits::generous());
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), MAX_GENERATED_CASES);

    // Executed directly, with no `catch_unwind` boundary: a production
    // tokenizer panic on any generated case must fail this test naturally
    // rather than being downgraded to ordinary case reporting.
    let mut failures = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        assert!(input.len() <= MAX_SOURCE_BYTES);
        let first_source = SourceText::new(SourceId::new(1), input.clone());
        let second_source = SourceText::new(SourceId::new(1), input.clone());

        let first_result = tokenize(&first_source, limits);
        let first_observed = observe(&first_source, &first_result);
        let second_result = tokenize(&second_source, limits);
        let second_observed = observe(&second_source, &second_result);

        if first_observed != second_observed {
            // Structural identifiers only: case index and source byte
            // length. Authored source content must never appear in a
            // validation failure message.
            failures.push(format!(
                "case {index} (source byte length {}) is not deterministic",
                input.len()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "generated-input failures:\n{}",
        failures.join("\n")
    );
}
