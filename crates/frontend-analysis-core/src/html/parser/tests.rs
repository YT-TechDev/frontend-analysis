use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::{
    HtmlCharacterToken, HtmlEndOfFileToken, HtmlNameEvidence, HtmlPreprocessingEvidence,
    HtmlTagKind, HtmlTagToken, HtmlToken,
};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::{HtmlTokenizerLimits, HtmlTokenizerUsage};
use super::super::tokenizer::result::{
    HtmlTokenizerCompletion, HtmlTokenizerCoverage, HtmlTokenizerRunResult,
};
use super::*;

fn src(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn anchor(source: &SourceText, start: usize, end: usize) -> SourceAnchor {
    source.anchor(start, end).unwrap()
}

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_000, 1_000, 1_000, 1_000, 1_000, 1_000, 1_000)
}

fn coverage(source: &SourceText, processed_end: usize) -> HtmlTokenizerCoverage {
    HtmlTokenizerCoverage::new(
        source,
        anchor(source, 0, processed_end),
        anchor(source, processed_end, source.as_str().len()),
    )
    .unwrap()
}

fn preprocessing(source: &SourceText) -> HtmlPreprocessingEvidence {
    HtmlPreprocessingEvidence::new(source, None).unwrap()
}

fn eof(source: &SourceText) -> HtmlToken {
    let end = source.as_str().len();
    HtmlToken::EndOfFile(HtmlEndOfFileToken::new(source, anchor(source, end, end)).unwrap())
}

fn start_tag_at(
    source: &SourceText,
    start: usize,
    name_end: usize,
    complete_end: usize,
    interpreted: &str,
) -> HtmlToken {
    HtmlToken::Tag(
        HtmlTagToken::new(
            HtmlTagKind::Start,
            anchor(source, start, complete_end),
            anchor(source, start, start + 1),
            HtmlNameEvidence::new(anchor(source, start + 1, name_end), interpreted.to_owned())
                .unwrap(),
            Vec::new(),
            None,
            anchor(source, complete_end - 1, complete_end),
        )
        .unwrap(),
    )
}

fn end_tag_at(
    source: &SourceText,
    start: usize,
    name_end: usize,
    complete_end: usize,
    interpreted: &str,
) -> HtmlToken {
    HtmlToken::Tag(
        HtmlTagToken::new(
            HtmlTagKind::End,
            anchor(source, start, complete_end),
            anchor(source, start, start + 2),
            HtmlNameEvidence::new(anchor(source, start + 2, name_end), interpreted.to_owned())
                .unwrap(),
            Vec::new(),
            None,
            anchor(source, complete_end - 1, complete_end),
        )
        .unwrap(),
    )
}

fn character_at(source: &SourceText, start: usize, end: usize, interpreted: &str) -> HtmlToken {
    HtmlToken::Character(
        HtmlCharacterToken::new(anchor(source, start, end), interpreted.to_owned()).unwrap(),
    )
}

/// Builds a `Complete` tokenizer run out of already-constructed tokens
/// (excluding EOF, which is appended automatically), with generous limits
/// and usage evidence loose enough to satisfy `HtmlTokenizerRunResult`'s
/// self-validation without needing exact transition/byte accounting.
fn complete_run(source: &SourceText, mut tokens: Vec<HtmlToken>) -> HtmlTokenizerRunResult {
    tokens.push(eof(source));
    let token_count = tokens.len();
    let end = source.as_str().len();
    HtmlTokenizerRunResult::new(
        source,
        tokens,
        preprocessing(source),
        Vec::new(),
        coverage(source, end),
        HtmlTokenizerCompletion::Complete,
        generous_limits(),
        HtmlTokenizerUsage::new(end, token_count, token_count, 0, 0, 100, 0),
    )
    .unwrap()
}

fn tokenize_source(source: &SourceText) -> HtmlTokenizerRunResult {
    tokenize(source, generous_limits())
}

fn start_tag_occurrences(
    analysis: &HtmlExplicitStartTagAnalysis,
) -> &[HtmlExplicitStartTagOccurrence] {
    analysis.occurrences()
}

// 1. Multiple start tags with character data: deterministic ordered
// occurrences, interleaved with character data that is not projected.
#[test]
fn multiple_start_tags_with_character_data_are_ordered_and_distinct() {
    let source = src(1, "text<div>x<span>y");
    let run = tokenize_source(&source);
    assert!(run.is_clean_complete());

    let analysis = analyze_explicit_start_tags(run).unwrap();
    let occurrences = start_tag_occurrences(&analysis);
    assert_eq!(occurrences.len(), 2);

    assert_eq!(
        occurrences[0].complete().range(),
        anchor(&source, 4, 9).range()
    );
    assert_eq!(occurrences[0].raw_name().fragment(), "div");
    assert_eq!(
        occurrences[1].complete().range(),
        anchor(&source, 10, 16).range()
    );
    assert_eq!(occurrences[1].raw_name().fragment(), "span");

    // Containment of raw_name within complete holds for every produced
    // occurrence, proven directly rather than assumed.
    for occurrence in occurrences {
        assert!(occurrence.raw_name().range().start() >= occurrence.complete().range().start());
        assert!(occurrence.raw_name().range().end() <= occurrence.complete().range().end());
    }
}

// 2. Duplicate raw spellings at different offsets remain distinct
// occurrences with distinct complete ranges.
#[test]
fn duplicate_raw_spellings_remain_distinct_occurrences() {
    let source = src(2, "<div><div>");
    let run = tokenize_source(&source);
    let analysis = analyze_explicit_start_tags(run).unwrap();
    let occurrences = start_tag_occurrences(&analysis);

    assert_eq!(occurrences.len(), 2);
    assert_eq!(occurrences[0].raw_name().fragment(), "div");
    assert_eq!(occurrences[1].raw_name().fragment(), "div");
    assert_ne!(
        occurrences[0].complete().range(),
        occurrences[1].complete().range()
    );
    assert_eq!(
        occurrences[0].complete().range(),
        anchor(&source, 0, 5).range()
    );
    assert_eq!(
        occurrences[1].complete().range(),
        anchor(&source, 5, 10).range()
    );
}

// 3. Raw spelling preserves authored case and is not replaced by the
// interpreted (lowercased) tag name.
#[test]
fn raw_name_preserves_authored_mixed_case_spelling() {
    let source = src(3, "<DiV>");
    let run = tokenize_source(&source);
    let analysis = analyze_explicit_start_tags(run).unwrap();
    let occurrences = start_tag_occurrences(&analysis);

    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0].raw_name().fragment(), "DiV");
}

// 4. Multibyte UTF-8 content before and between tags keeps byte ranges
// exact.
#[test]
fn multibyte_utf8_surrounding_markup_keeps_exact_byte_ranges() {
    let source = src(4, "é<div>日");
    let run = tokenize_source(&source);
    let analysis = analyze_explicit_start_tags(run).unwrap();
    let occurrences = start_tag_occurrences(&analysis);

    assert_eq!(occurrences.len(), 1);
    assert_eq!(
        occurrences[0].complete().range(),
        anchor(&source, 2, 7).range()
    );
    assert_eq!(
        occurrences[0].raw_name().range(),
        anchor(&source, 3, 6).range()
    );
    assert_eq!(occurrences[0].raw_name().fragment(), "div");
}

// 5. Complete tokenizer runs with diagnostics still propagate the
// diagnostic evidence through the retained tokenizer run.
#[test]
fn complete_with_diagnostics_preserves_diagnostics_through_retained_run() {
    let source = src(5, "\0<div>");
    let run = tokenize_source(&source);
    assert!(run.is_complete_with_diagnostics());

    let analysis = analyze_explicit_start_tags(run).unwrap();
    assert_eq!(start_tag_occurrences(&analysis).len(), 1);
    assert!(analysis.tokenizer_run().is_complete_with_diagnostics());
    assert!(!analysis.tokenizer_run().diagnostics().is_empty());
}

// 6. An unsupported capability discovered after a recognized occurrence
// keeps that occurrence available while the result remains incomplete.
#[test]
fn unsupported_after_occurrence_keeps_prior_occurrence_but_stays_incomplete() {
    let source = src(6, "<title>x");
    let run = tokenize_source(&source);
    assert!(run.is_incomplete());

    let analysis = analyze_explicit_start_tags(run).unwrap();
    let occurrences = start_tag_occurrences(&analysis);
    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0].raw_name().fragment(), "title");
    assert!(analysis.tokenizer_run().is_incomplete());
}

// 7. Zero occurrences from an unsupported run before any start tag must not
// become clean absence.
#[test]
fn unsupported_before_any_occurrence_stays_incomplete_not_clean_absence() {
    let source = src(7, "&x");
    let run = tokenize_source(&source);
    assert!(run.is_incomplete());

    let analysis = analyze_explicit_start_tags(run).unwrap();
    assert!(start_tag_occurrences(&analysis).is_empty());
    assert!(analysis.tokenizer_run().is_incomplete());
}

// 8. A resource-limited partial run keeps the start tag projected before
// the boundary; completion/coverage stay tokenizer-owned.
#[test]
fn resource_limited_partial_run_preserves_prior_projected_occurrence() {
    let source = src(8, "<div>x");
    let limits = HtmlTokenizerLimits::new(1_000, 1_000, 1, 1_000, 1_000, 1_000, 1_000);
    let run = tokenize(&source, limits);
    assert!(run.is_incomplete());

    let analysis = analyze_explicit_start_tags(run).unwrap();
    let occurrences = start_tag_occurrences(&analysis);
    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0].raw_name().fragment(), "div");
    assert!(analysis.tokenizer_run().is_incomplete());
}

// 9. End-tag-only input produces zero start-tag occurrences without
// projecting an occurrence for the end tag.
#[test]
fn end_tag_only_input_produces_zero_start_tag_occurrences() {
    let source = src(9, "</div>");
    let run = tokenize_source(&source);
    assert!(run.is_clean_complete());

    let analysis = analyze_explicit_start_tags(run).unwrap();
    assert!(start_tag_occurrences(&analysis).is_empty());
}

// 10. Parser-boundary contract corruption: each invalid relationship
// between occurrences and the retained tokenizer run is rejected as a
// typed contract error, not a panic or silent success.

fn single_start_tag_run() -> (SourceText, HtmlTokenizerRunResult) {
    let source = src(100, "<div>");
    let run = complete_run(&source, vec![start_tag_at(&source, 0, 4, 5, "div")]);
    (source, run)
}

#[test]
fn contract_rejects_out_of_bounds_origin_token_index() {
    let (source, run) = single_start_tag_run();
    let occurrence = HtmlExplicitStartTagOccurrence {
        origin_token_index: 5,
        complete: anchor(&source, 0, 5),
        raw_name: anchor(&source, 1, 4),
    };
    assert_eq!(
        HtmlExplicitStartTagAnalysis::new(run, vec![occurrence]).unwrap_err(),
        HtmlAnalysisParserContractError::InvalidOriginTokenIndex {
            occurrence_index: 0,
            origin_token_index: 5,
        }
    );
}

#[test]
fn contract_rejects_origin_token_that_is_not_a_tag() {
    let source = src(101, "x<div>");
    let run = complete_run(
        &source,
        vec![
            character_at(&source, 0, 1, "x"),
            start_tag_at(&source, 1, 5, 6, "div"),
        ],
    );
    let occurrence = HtmlExplicitStartTagOccurrence {
        origin_token_index: 0,
        complete: anchor(&source, 1, 6),
        raw_name: anchor(&source, 2, 5),
    };
    assert_eq!(
        HtmlExplicitStartTagAnalysis::new(run, vec![occurrence]).unwrap_err(),
        HtmlAnalysisParserContractError::OriginTokenNotTag {
            occurrence_index: 0,
            origin_token_index: 0,
        }
    );
}

#[test]
fn contract_rejects_origin_token_that_is_an_end_tag() {
    // Exactly one real start tag exists (`div` at token index 0), so the
    // occurrence-count invariant is satisfied; the single occurrence is
    // deliberately mispointed at the end tag (token index 1) instead, to
    // isolate the token-kind check from the inventory check.
    let source = src(102, "<div></div>");
    let run = complete_run(
        &source,
        vec![
            start_tag_at(&source, 0, 4, 5, "div"),
            end_tag_at(&source, 5, 10, 11, "div"),
        ],
    );
    let occurrence = HtmlExplicitStartTagOccurrence {
        origin_token_index: 1,
        complete: anchor(&source, 5, 11),
        raw_name: anchor(&source, 7, 10),
    };
    assert_eq!(
        HtmlExplicitStartTagAnalysis::new(run, vec![occurrence]).unwrap_err(),
        HtmlAnalysisParserContractError::OriginTokenNotStartTag {
            occurrence_index: 0,
            origin_token_index: 1,
        }
    );
}

#[test]
fn contract_rejects_mismatched_complete_anchor() {
    let (source, run) = single_start_tag_run();
    let occurrence = HtmlExplicitStartTagOccurrence {
        origin_token_index: 0,
        complete: anchor(&source, 0, 4),
        raw_name: anchor(&source, 1, 4),
    };
    assert_eq!(
        HtmlExplicitStartTagAnalysis::new(run, vec![occurrence]).unwrap_err(),
        HtmlAnalysisParserContractError::CompleteEvidenceMismatch {
            occurrence_index: 0,
            origin_token_index: 0,
        }
    );
}

#[test]
fn contract_rejects_mismatched_raw_name_anchor() {
    let (source, run) = single_start_tag_run();
    let occurrence = HtmlExplicitStartTagOccurrence {
        origin_token_index: 0,
        complete: anchor(&source, 0, 5),
        raw_name: anchor(&source, 1, 3),
    };
    assert_eq!(
        HtmlExplicitStartTagAnalysis::new(run, vec![occurrence]).unwrap_err(),
        HtmlAnalysisParserContractError::RawNameEvidenceMismatch {
            occurrence_index: 0,
            origin_token_index: 0,
        }
    );
}

#[test]
fn contract_rejects_out_of_order_occurrences() {
    let source = src(103, "<div><span>");
    let run = complete_run(
        &source,
        vec![
            start_tag_at(&source, 0, 4, 5, "div"),
            start_tag_at(&source, 5, 10, 11, "span"),
        ],
    );
    let occurrences = vec![
        HtmlExplicitStartTagOccurrence {
            origin_token_index: 1,
            complete: anchor(&source, 5, 11),
            raw_name: anchor(&source, 6, 10),
        },
        HtmlExplicitStartTagOccurrence {
            origin_token_index: 0,
            complete: anchor(&source, 0, 5),
            raw_name: anchor(&source, 1, 4),
        },
    ];
    assert_eq!(
        HtmlExplicitStartTagAnalysis::new(run, occurrences).unwrap_err(),
        HtmlAnalysisParserContractError::OccurrenceOrderViolation {
            occurrence_index: 1
        }
    );
}

#[test]
fn contract_rejects_missing_start_tag_occurrence() {
    let source = src(104, "<div><span>");
    let run = complete_run(
        &source,
        vec![
            start_tag_at(&source, 0, 4, 5, "div"),
            start_tag_at(&source, 5, 10, 11, "span"),
        ],
    );
    let occurrence = HtmlExplicitStartTagOccurrence {
        origin_token_index: 0,
        complete: anchor(&source, 0, 5),
        raw_name: anchor(&source, 1, 4),
    };
    assert_eq!(
        HtmlExplicitStartTagAnalysis::new(run, vec![occurrence]).unwrap_err(),
        HtmlAnalysisParserContractError::OccurrenceInventoryMismatch {
            expected: 2,
            actual: 1,
        }
    );
}

#[test]
fn contract_rejects_extra_occurrence() {
    let (source, run) = single_start_tag_run();
    let occurrence = HtmlExplicitStartTagOccurrence {
        origin_token_index: 0,
        complete: anchor(&source, 0, 5),
        raw_name: anchor(&source, 1, 4),
    };
    assert_eq!(
        HtmlExplicitStartTagAnalysis::new(run, vec![occurrence.clone(), occurrence]).unwrap_err(),
        HtmlAnalysisParserContractError::OccurrenceInventoryMismatch {
            expected: 1,
            actual: 2,
        }
    );
}

// Determinism: equal retained source, configuration, and implementation
// produce equivalent occurrence meaning and order across repeated runs and
// across differing SourceId values.
#[test]
fn repeated_analysis_is_deterministic_across_runs_and_source_ids() {
    let text = "text<div>x<span>y";
    let first = analyze_explicit_start_tags(tokenize_source(&src(200, text))).unwrap();
    let second = analyze_explicit_start_tags(tokenize_source(&src(200, text))).unwrap();
    let third = analyze_explicit_start_tags(tokenize_source(&src(201, text))).unwrap();

    for (a, b) in [(&first, &second), (&first, &third)] {
        let a_occurrences = start_tag_occurrences(a);
        let b_occurrences = start_tag_occurrences(b);
        assert_eq!(a_occurrences.len(), b_occurrences.len());
        for (left, right) in a_occurrences.iter().zip(b_occurrences.iter()) {
            assert_eq!(left.origin_token_index(), right.origin_token_index());
            assert_eq!(left.complete().range(), right.complete().range());
            assert_eq!(left.raw_name().range(), right.raw_name().range());
            assert_eq!(left.raw_name().fragment(), right.raw_name().fragment());
        }
    }
}

// Debug output must not expose retained authored source content.
#[test]
fn debug_output_redacts_source_content() {
    const SECRET: &str = "private-analysis-marker";
    let source = src(9001, &format!("<{SECRET}>"));
    let run = tokenize_source(&source);
    let analysis = analyze_explicit_start_tags(run).unwrap();

    let debug = format!("{analysis:?}");
    assert!(!debug.contains(SECRET));
    assert!(debug.contains("occurrence_count"));

    for occurrence in start_tag_occurrences(&analysis) {
        let occurrence_debug = format!("{occurrence:?}");
        assert!(!occurrence_debug.contains(SECRET));
    }
}
