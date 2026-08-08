//! Candidate-independent validation of the #115 HTML analysis parser against
//! the existing #112/#113 tokenizer gold corpus.
//!
//! The parser's own occurrence projection is never its own oracle: expected
//! occurrences are independently derived here by filtering each fixture's
//! authored expected tokens for start tags, never read from the production
//! parser's output. This module adds no new fixtures and modifies no
//! existing gold; it only reuses [`super::corpus::all_candidate_independent_corpus`]
//! as evidence for a capability the #112/#113 foundation predates.

use crate::html::parser::analyze_explicit_start_tags;
use crate::{SourceId, SourceText};

use super::super::producer::tokenize;
use super::super::resource::HtmlTokenizerLimits;
use super::corpus::all_candidate_independent_corpus;
use super::expected::{Limits, Token, TokenKind};
use super::fixture::HtmlTokenizerFixture;
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
            let signature: Vec<_> = analysis
                .occurrences()
                .iter()
                .map(|occurrence| {
                    (
                        occurrence.origin_token_index(),
                        occurrence.complete().range(),
                        occurrence.raw_name().range(),
                    )
                })
                .collect();
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
