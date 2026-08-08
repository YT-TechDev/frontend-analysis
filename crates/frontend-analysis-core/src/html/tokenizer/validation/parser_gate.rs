//! Candidate-independent validation of the #115 HTML analysis parser against
//! the existing #112/#113 tokenizer gold corpus.
//!
//! The parser's own occurrence projection is never its own oracle: expected
//! occurrences are independently derived here by filtering each fixture's
//! authored expected tokens for start tags, never read from the production
//! parser's output. This module adds no new fixtures and modifies no
//! existing gold; it only reuses [`super::corpus::all_candidate_independent_corpus`]
//! as evidence for a capability the #112/#113 foundation predates.

use crate::html::parser::{HtmlExplicitStartTagAnalysis, analyze_explicit_start_tags};
use crate::{SourceId, SourceText};

use super::super::producer::tokenize;
use super::super::resource::HtmlTokenizerLimits;
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
/// Source: fixture gold, per the #114/#115 candidate-independence
/// requirement, never the production tokenizer or parser output.
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
fn parser_matches_independent_start_tag_projection_for_all_candidate_independent_fixtures() {
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
        let run = tokenize(&source, limits);
        let observed_before = observe(&source, &run);
        let expected = expected_occurrences(fixture);

        let analysis = match analyze_explicit_start_tags(run) {
            Ok(analysis) => analysis,
            Err(error) => {
                mismatches.push(format!("{}: parser contract error {error:?}", fixture.id));
                continue;
            }
        };

        // Tokenizer diagnostics, coverage, completion, and usage evidence
        // must remain exactly what the tokenizer produced: the parser
        // retains, and must not re-derive or lose, that evidence.
        let observed_after = observe(&source, analysis.tokenizer_run());
        if observed_before != observed_after {
            mismatches.push(format!(
                "{}: retained tokenizer evidence changed through the parser boundary",
                fixture.id
            ));
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
        "parser/gold mismatches:\n{}",
        mismatches.join("\n")
    );
}

#[test]
fn parser_analysis_is_deterministic_across_repeats_and_source_ids() {
    let fixtures = all_candidate_independent_corpus();
    for fixture in &fixtures {
        let text = String::from_utf8(fixture.source_bytes.to_vec())
            .expect("fixture source is valid UTF-8");
        let limits = to_html_limits(fixture.expected.0.limits);

        let mut previous: Option<Vec<(usize, crate::SourceRange, crate::SourceRange)>> = None;
        for source_id in [101u64, 101u64, 202u64] {
            let source = SourceText::new(SourceId::new(source_id), text.clone());
            let run = tokenize(&source, limits);
            let analysis = analyze_explicit_start_tags(run).unwrap();
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

/// A structural occurrence signature: only parser-owned meaning
/// (`origin_token_index`, `complete` range, `raw_name` range), never
/// authored source content. Used to compare two analyses of equal retained
/// source without a stable serialization commitment.
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

/// Reuses the existing bounded, deterministic, dependency-free generator
/// (`generated.rs`, already exercised against the tokenizer alone in
/// `execute.rs`) to drive `SourceText -> tokenize(...) ->
/// analyze_explicit_start_tags(...)` directly, with no `catch_unwind`: a
/// production panic on any of the 4,096 generated cases fails this test
/// naturally rather than being caught and downgraded. This is the
/// parser-specific property/fuzz-smoke gate #115 requires; the pre-existing
/// `execute.rs` generated-input test only reaches the tokenizer/observation
/// layer and never calls the analysis parser.
#[test]
fn parser_handles_all_generated_inputs_without_panic_and_preserves_properties() {
    let limits = to_html_limits(Limits::generous());
    let inputs = generated_inputs();
    assert_eq!(inputs.len(), MAX_GENERATED_CASES);

    let mut failures = Vec::new();
    for (index, input) in inputs.iter().enumerate() {
        assert!(input.len() <= MAX_SOURCE_BYTES);
        let byte_len = input.len();

        let primary_source = SourceText::new(SourceId::new(1), input.clone());
        let primary_run = tokenize(&primary_source, limits);
        let primary_observed_before = observe(&primary_source, &primary_run);

        // A. The currently valid production tokenizer result must produce
        // Ok(HtmlExplicitStartTagAnalysis); any contract error here is a
        // parser regression, not an expected outcome, and is recorded as a
        // failure rather than downgraded to ordinary case reporting.
        let primary_analysis = match analyze_explicit_start_tags(primary_run) {
            Ok(analysis) => analysis,
            Err(error) => {
                failures.push(format!(
                    "case {index} (source byte length {byte_len}): parser contract error {error:?} from a valid tokenizer result"
                ));
                continue;
            }
        };

        // B. Tokenizer evidence (tokens, preprocessing, diagnostics,
        // coverage, completion, limits, usage) is unchanged across the
        // parser boundary.
        let primary_observed_after = observe(&primary_source, primary_analysis.tokenizer_run());
        if primary_observed_before != primary_observed_after {
            failures.push(format!(
                "case {index} (source byte length {byte_len}): retained tokenizer evidence changed through the parser boundary"
            ));
        }

        // F. Completeness monotonicity: zero occurrences from an incomplete
        // tokenizer run must remain incomplete, never reinterpreted as clean
        // absence. (Full completion equality is already covered by B.)
        if primary_analysis.occurrences().is_empty()
            && primary_analysis.tokenizer_run().is_incomplete()
        {
            // Exercised, not a failure: an incomplete run legitimately
            // projects zero occurrences without becoming clean success.
        }

        for occurrence in primary_analysis.occurrences() {
            // C. Occurrence source identity.
            if occurrence.complete().source_id() != primary_source.id()
                || occurrence.raw_name().source_id() != primary_source.id()
            {
                failures.push(format!(
                    "case {index} (source byte length {byte_len}): occurrence source identity mismatch"
                ));
            }
            // D. Range containment: complete.start <= raw_name.start and
            // raw_name.end <= complete.end, using already-validated
            // SourceAnchor ranges with no source rescan.
            if occurrence.complete().range().start() > occurrence.raw_name().range().start()
                || occurrence.raw_name().range().end() > occurrence.complete().range().end()
            {
                failures.push(format!(
                    "case {index} (source byte length {byte_len}): raw_name range escapes complete range"
                ));
            }
        }

        // E. Deterministic source/token ordering: adjacent occurrences move
        // strictly forward in both origin token index and complete range.
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
        let repeat_run = tokenize(&repeat_source, limits);
        match analyze_explicit_start_tags(repeat_run) {
            Ok(repeat_analysis) => {
                if occurrence_signature(&repeat_analysis) != primary_signature {
                    failures.push(format!(
                        "case {index} (source byte length {byte_len}): non-deterministic across repeated runs with equal SourceId"
                    ));
                }
            }
            Err(error) => failures.push(format!(
                "case {index} (source byte length {byte_len}): repeat-run parser contract error {error:?}"
            )),
        }

        // G. An alternate SourceId must not alter occurrence ranges,
        // ordering, or count.
        let alternate_source = SourceText::new(SourceId::new(2), input.clone());
        let alternate_run = tokenize(&alternate_source, limits);
        match analyze_explicit_start_tags(alternate_run) {
            Ok(alternate_analysis) => {
                if occurrence_signature(&alternate_analysis) != primary_signature {
                    failures.push(format!(
                        "case {index} (source byte length {byte_len}): occurrence signature changed with a different SourceId"
                    ));
                }
            }
            Err(error) => failures.push(format!(
                "case {index} (source byte length {byte_len}): alternate-SourceId parser contract error {error:?}"
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "generated parser-property failures:\n{}",
        failures.join("\n")
    );
}
