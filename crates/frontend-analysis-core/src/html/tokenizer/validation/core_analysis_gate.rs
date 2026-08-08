//! Candidate-independent validation of the #116 Core-integrated
//! `analyze_html_explicit_start_tags` operation against the existing
//! #112/#113 tokenizer gold corpus.
//!
//! This is an additional validation layer, not a replacement for the
//! existing direct parser gate in [`super::parser_gate`]: that module
//! proves `analyze_explicit_start_tags` against a caller-supplied
//! `HtmlTokenizerRunResult`, while this module proves the complete
//! `SourceText -> analyze_html_explicit_start_tags(...)` Core boundary,
//! including the additional source-evidence validation #116 owns. Expected
//! occurrences are derived independently by filtering each fixture's
//! authored expected tokens for start tags, never read from production
//! output. This module adds no new fixtures and modifies no existing gold.

use crate::html::analysis::analyze_html_explicit_start_tags;
use crate::html::parser::HtmlExplicitStartTagAnalysis;
use crate::{SourceId, SourceText};

use super::super::resource::HtmlTokenizerLimits;
use super::compare::compare;
use super::corpus::all_candidate_independent_corpus;
use super::expected::{Limits, Token, TokenKind};
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

struct ExpectedOccurrence {
    complete_start: usize,
    complete_end: usize,
    raw_name_start: usize,
    raw_name_end: usize,
}

/// Independently derives the expected start-tag occurrence projection for a
/// fixture by filtering its authored expected tokens for start tags only.
/// Source: fixture gold, never the production tokenizer, parser, or Core
/// operation output.
fn expected_occurrences(fixture: &HtmlTokenizerFixture) -> Vec<ExpectedOccurrence> {
    fixture
        .expected
        .0
        .tokens
        .iter()
        .filter_map(|token| match token {
            Token::Tag {
                kind: TokenKind::StartTag,
                complete,
                name,
                ..
            } => Some(ExpectedOccurrence {
                complete_start: complete.span.start,
                complete_end: complete.span.end,
                raw_name_start: name.span.start,
                raw_name_end: name.span.end,
            }),
            _ => None,
        })
        .collect()
}

#[test]
fn core_operation_matches_independent_start_tag_projection_for_all_candidate_independent_fixtures()
{
    let fixtures = all_candidate_independent_corpus();
    const INITIAL_CORPUS_COUNT: usize = 72;
    const SUPPLEMENTAL_REGRESSION_COUNT: usize = 4;
    assert_eq!(
        fixtures.len(),
        INITIAL_CORPUS_COUNT + SUPPLEMENTAL_REGRESSION_COUNT
    );

    let mut mismatches = Vec::new();
    for fixture in &fixtures {
        let text = String::from_utf8(fixture.source_bytes.to_vec())
            .expect("fixture source is valid UTF-8");
        let source = SourceText::new(SourceId::new(1), text);
        let limits = to_html_limits(fixture.expected.0.limits);
        let expected = expected_occurrences(fixture);

        let analysis = match analyze_html_explicit_start_tags(&source, limits) {
            Ok(analysis) => analysis,
            Err(error) => {
                mismatches.push(format!("{}: Core operation error {error:?}", fixture.id));
                continue;
            }
        };

        // Retained tokenizer evidence (tokens, preprocessing, diagnostics,
        // coverage, completion, limits, usage) must match the same
        // fixture's tokenizer-only gold exactly: the Core boundary must not
        // alter it.
        let observed = observe(&source, analysis.tokenizer_run());
        if let Err(mismatch) = compare(fixture.id, &fixture.expected, &observed) {
            mismatches.push(mismatch.to_string());
        }

        let occurrences = analysis.occurrences();
        if occurrences.len() != expected.len() {
            mismatches.push(format!(
                "{}: occurrence count {} != independently expected {}",
                fixture.id,
                occurrences.len(),
                expected.len()
            ));
            continue;
        }
        for (index, (occurrence, expectation)) in
            occurrences.iter().zip(expected.iter()).enumerate()
        {
            let complete = occurrence.complete();
            let raw_name = occurrence.raw_name();
            if complete.source_id() != source.id()
                || complete.range().start() != expectation.complete_start
                || complete.range().end() != expectation.complete_end
            {
                mismatches.push(format!(
                    "{}: occurrence[{index}] complete range mismatch",
                    fixture.id
                ));
            }
            if raw_name.source_id() != source.id()
                || raw_name.range().start() != expectation.raw_name_start
                || raw_name.range().end() != expectation.raw_name_end
            {
                mismatches.push(format!(
                    "{}: occurrence[{index}] raw_name range mismatch",
                    fixture.id
                ));
            }
            let expected_raw =
                &source.as_str()[expectation.raw_name_start..expectation.raw_name_end];
            if raw_name.fragment() != expected_raw {
                mismatches.push(format!(
                    "{}: occurrence[{index}] raw spelling mismatch",
                    fixture.id
                ));
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "Core operation/gold mismatches:\n{}",
        mismatches.join("\n")
    );
}

/// A structural occurrence signature: only parser-owned meaning
/// (`origin_token_index`, `complete` range, `raw_name` range), never
/// authored source content.
fn occurrence_signature(
    analysis: &HtmlExplicitStartTagAnalysis,
) -> Vec<(usize, crate::SourceRange, crate::SourceRange)> {
    analysis
        .occurrences()
        .iter()
        .map(|occurrence| {
            (
                occurrence.origin_token_index(),
                occurrence.complete().range(),
                occurrence.raw_name().range(),
            )
        })
        .collect()
}

#[test]
fn core_operation_is_deterministic_across_repeats_and_source_ids() {
    let fixtures = all_candidate_independent_corpus();
    for fixture in &fixtures {
        let text = String::from_utf8(fixture.source_bytes.to_vec())
            .expect("fixture source is valid UTF-8");
        let limits = to_html_limits(fixture.expected.0.limits);

        let mut previous: Option<Vec<(usize, crate::SourceRange, crate::SourceRange)>> = None;
        for source_id in [101u64, 101u64, 202u64] {
            let source = SourceText::new(SourceId::new(source_id), text.clone());
            let analysis = analyze_html_explicit_start_tags(&source, limits).unwrap();
            let signature = occurrence_signature(&analysis);
            if let Some(previous_signature) = &previous {
                assert_eq!(
                    &signature, previous_signature,
                    "{}: non-deterministic occurrence signature",
                    fixture.id
                );
            }
            previous = Some(signature);
        }
    }
}

/// Reuses the existing bounded, deterministic, dependency-free generator
/// (`generated.rs`) to drive `SourceText -> analyze_html_explicit_start_tags(...)`
/// directly, with no `catch_unwind`: a production panic on any of the 4,096
/// generated cases fails this test naturally. This is the Core-integration
/// property/fuzz-smoke gate #116 requires, on top of the pre-existing
/// tokenizer-only (`execute.rs`) and parser-only (`parser_gate.rs`) generated
/// gates.
#[test]
fn core_operation_handles_all_generated_inputs_without_panic_and_preserves_properties() {
    let limits = to_html_limits(Limits::generous());
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), MAX_GENERATED_CASES);

    let mut failures = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        assert!(input.len() <= MAX_SOURCE_BYTES);
        let byte_len = input.len();

        let primary_source = SourceText::new(SourceId::new(1), input.clone());

        // A. The complete Core operation must succeed for every generated
        // case: a returned Err here is a Core-integration regression, not
        // an expected outcome.
        let primary_analysis = match analyze_html_explicit_start_tags(&primary_source, limits) {
            Ok(analysis) => analysis,
            Err(error) => {
                failures.push(format!(
                    "case {index} (source byte length {byte_len}): Core operation error {error:?}"
                ));
                continue;
            }
        };

        // F. Completeness monotonicity: zero occurrences from an incomplete
        // tokenizer run must remain incomplete, never reinterpreted as clean
        // absence.
        if primary_analysis.occurrences().is_empty()
            && primary_analysis.tokenizer_run().is_incomplete()
        {
            // Exercised, not a failure: an incomplete run legitimately
            // projects zero occurrences without becoming clean success.
        }

        for occurrence in primary_analysis.occurrences() {
            // C. Every occurrence's anchors share the supplied source's
            // identity, per Core source-evidence validation.
            if occurrence.complete().source_id() != primary_source.id()
                || occurrence.raw_name().source_id() != primary_source.id()
            {
                failures.push(format!(
                    "case {index} (source byte length {byte_len}): occurrence source identity mismatch"
                ));
            }
            // D. raw_name stays contained in complete.
            if occurrence.complete().range().start() > occurrence.raw_name().range().start()
                || occurrence.raw_name().range().end() > occurrence.complete().range().end()
            {
                failures.push(format!(
                    "case {index} (source byte length {byte_len}): raw_name range escapes complete range"
                ));
            }
        }

        // E. Deterministic source/token ordering: adjacent occurrences move
        // strictly forward in both origin index and range.
        for pair in primary_analysis.occurrences().windows(2) {
            if pair[1].origin_token_index() <= pair[0].origin_token_index()
                || pair[1].complete().range().start() < pair[0].complete().range().end()
            {
                failures.push(format!(
                    "case {index} (source byte length {byte_len}): occurrence ordering moved backward"
                ));
            }
        }

        let primary_signature = occurrence_signature(&primary_analysis);

        // G. Repeat determinism with the same SourceId.
        let repeat_source = SourceText::new(SourceId::new(1), input.clone());
        match analyze_html_explicit_start_tags(&repeat_source, limits) {
            Ok(repeat_analysis) => {
                if occurrence_signature(&repeat_analysis) != primary_signature {
                    failures.push(format!(
                        "case {index} (source byte length {byte_len}): non-deterministic across repeated runs with equal SourceId"
                    ));
                }
            }
            Err(error) => failures.push(format!(
                "case {index} (source byte length {byte_len}): repeat-run Core operation error {error:?}"
            )),
        }

        // Alternate SourceId must not alter occurrence ranges, ordering, or
        // count.
        let alternate_source = SourceText::new(SourceId::new(2), input.clone());
        match analyze_html_explicit_start_tags(&alternate_source, limits) {
            Ok(alternate_analysis) => {
                if occurrence_signature(&alternate_analysis) != primary_signature {
                    failures.push(format!(
                        "case {index} (source byte length {byte_len}): occurrence signature changed with a different SourceId"
                    ));
                }
            }
            Err(error) => failures.push(format!(
                "case {index} (source byte length {byte_len}): alternate-SourceId Core operation error {error:?}"
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "generated Core-operation property failures:\n{}",
        failures.join("\n")
    );
}
