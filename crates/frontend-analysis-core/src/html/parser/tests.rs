use crate::{SourceAnchor, SourceId, SourceText};

use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;
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

    // Origin indexes strictly increase in tokenizer/source order.
    assert!(occurrences[0].origin_token_index() < occurrences[1].origin_token_index());
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
