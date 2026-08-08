use crate::{SourceId, SourceRangeError, SourceText};

use super::*;

fn src(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_000, 1_000, 1_000, 1_000, 1_000, 1_000, 1_000)
}

// Native UTF-8/raw-spelling vertical-slice demonstration, per the #116
// approved decision. Expected byte ranges are computed independently
// (é is a 2-byte UTF-8 scalar; every other authored character is one ASCII
// byte) rather than derived from any production value.
#[test]
fn native_utf8_vertical_slice_reports_exact_occurrences_and_raw_spelling() {
    let source = src(1, "é<DiV>x<span>");
    let analysis = analyze_html_explicit_start_tags(&source, generous_limits()).unwrap();
    assert!(analysis.tokenizer_run().is_clean_complete());

    let occurrences = analysis.occurrences();
    assert_eq!(occurrences.len(), 2);

    assert_eq!(occurrences[0].complete().source_id(), SourceId::new(1));
    assert_eq!(
        (
            occurrences[0].complete().range().start(),
            occurrences[0].complete().range().end()
        ),
        (2, 7)
    );
    assert_eq!(
        (
            occurrences[0].raw_name().range().start(),
            occurrences[0].raw_name().range().end()
        ),
        (3, 6)
    );
    assert_eq!(occurrences[0].raw_name().fragment(), "DiV");

    assert_eq!(occurrences[1].complete().source_id(), SourceId::new(1));
    assert_eq!(
        (
            occurrences[1].complete().range().start(),
            occurrences[1].complete().range().end()
        ),
        (8, 14)
    );
    assert_eq!(
        (
            occurrences[1].raw_name().range().start(),
            occurrences[1].raw_name().range().end()
        ),
        (9, 13)
    );
    assert_eq!(occurrences[1].raw_name().fragment(), "span");
}

// Raw-coordinate projection: exact SourceId, zero-based byte offsets, line,
// and UTF-8 byte column, independently computed. Coordinates remain owned
// by the existing SourceAnchor/RawSourceCoordinate contract; this operation
// introduces no line/column recalculation of its own.
#[test]
fn occurrence_anchors_project_expected_raw_coordinates() {
    let source = src(7, "a\n<div>");
    let analysis = analyze_html_explicit_start_tags(&source, generous_limits()).unwrap();
    let occurrence = &analysis.occurrences()[0];

    let start = occurrence.complete().start_coordinate();
    assert_eq!(start.source_id(), SourceId::new(7));
    assert_eq!(start.byte_offset(), 2);
    assert_eq!((start.line_index(), start.byte_column()), (1, 0));

    let end = occurrence.complete().end_coordinate();
    assert_eq!(end.source_id(), SourceId::new(7));
    assert_eq!(end.byte_offset(), 7);
    assert_eq!((end.line_index(), end.byte_column()), (1, 5));
}

// Result-lifetime demonstration: the returned analysis remains valid,
// source-backed evidence and all, after the caller's original SourceText
// handle leaves scope, per the existing owned SourceAnchor contract.
#[test]
fn analysis_result_outlives_dropped_source_handle() {
    let analysis = {
        let source = src(42, "<div>x");
        analyze_html_explicit_start_tags(&source, generous_limits()).unwrap()
    };

    let occurrence = &analysis.occurrences()[0];
    assert_eq!(occurrence.complete().source_id(), SourceId::new(42));
    assert_eq!(occurrence.complete().fragment(), "<div>");
    assert_eq!(occurrence.raw_name().fragment(), "div");
}

#[test]
fn complete_clean_run_propagates_through_core_operation() {
    let source = src(2, "<div>x<span>");
    let analysis = analyze_html_explicit_start_tags(&source, generous_limits()).unwrap();
    assert!(analysis.tokenizer_run().is_clean_complete());
    assert_eq!(analysis.occurrences().len(), 2);
}

#[test]
fn complete_with_diagnostics_propagates_through_core_operation() {
    let source = src(3, "\0<div>");
    let analysis = analyze_html_explicit_start_tags(&source, generous_limits()).unwrap();
    assert!(analysis.tokenizer_run().is_complete_with_diagnostics());
    assert!(!analysis.tokenizer_run().diagnostics().is_empty());
    assert_eq!(analysis.occurrences().len(), 1);
}

#[test]
fn unsupported_before_any_occurrence_remains_incomplete() {
    let source = src(4, "&x");
    let analysis = analyze_html_explicit_start_tags(&source, generous_limits()).unwrap();
    assert!(analysis.tokenizer_run().is_incomplete());
    assert!(analysis.occurrences().is_empty());
}

#[test]
fn unsupported_after_prior_occurrence_remains_incomplete() {
    let source = src(5, "<title>x");
    let analysis = analyze_html_explicit_start_tags(&source, generous_limits()).unwrap();
    assert!(analysis.tokenizer_run().is_incomplete());
    let occurrences = analysis.occurrences();
    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0].raw_name().fragment(), "title");
}

#[test]
fn resource_limit_before_any_occurrence_remains_incomplete() {
    let source = src(6, "<div>x");
    let limits = HtmlTokenizerLimits::new(3, 1_000, 1_000, 1_000, 1_000, 1_000, 1_000);
    let analysis = analyze_html_explicit_start_tags(&source, limits).unwrap();
    assert!(analysis.tokenizer_run().is_incomplete());
    assert!(analysis.occurrences().is_empty());
}

#[test]
fn resource_limit_after_prior_occurrence_preserves_evidence() {
    let source = src(8, "<div>x");
    let limits = HtmlTokenizerLimits::new(1_000, 1_000, 1, 1_000, 1_000, 1_000, 1_000);
    let analysis = analyze_html_explicit_start_tags(&source, limits).unwrap();
    assert!(analysis.tokenizer_run().is_incomplete());
    let occurrences = analysis.occurrences();
    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0].raw_name().fragment(), "div");
}

#[test]
fn invalid_configuration_propagates_through_core_operation() {
    let source = src(9, "<div>x");
    let limits = HtmlTokenizerLimits::new(1_000, 0, 1_000, 1_000, 1_000, 1_000, 1_000);
    let analysis = analyze_html_explicit_start_tags(&source, limits).unwrap();
    assert!(analysis.tokenizer_run().is_incomplete());
    assert!(analysis.occurrences().is_empty());
}

#[test]
fn repeated_analysis_is_deterministic_across_runs_and_source_ids() {
    let text = "text<div>x<span>y";
    let first = analyze_html_explicit_start_tags(&src(300, text), generous_limits()).unwrap();
    let second = analyze_html_explicit_start_tags(&src(300, text), generous_limits()).unwrap();
    let third = analyze_html_explicit_start_tags(&src(301, text), generous_limits()).unwrap();

    for (a, b) in [(&first, &second), (&first, &third)] {
        let a_occurrences = a.occurrences();
        let b_occurrences = b.occurrences();
        assert_eq!(a_occurrences.len(), b_occurrences.len());
        for (left, right) in a_occurrences.iter().zip(b_occurrences.iter()) {
            assert_eq!(left.origin_token_index(), right.origin_token_index());
            assert_eq!(left.complete().range(), right.complete().range());
            assert_eq!(left.raw_name().range(), right.raw_name().range());
            assert_eq!(left.raw_name().fragment(), right.raw_name().fragment());
        }
    }
}

// Corruption tests: `validate_occurrence_evidence` is exercised directly
// with deliberately inconsistent anchors, since `HtmlExplicitStartTagOccurrence`
// intentionally exposes no public constructor for corrupted evidence.

#[test]
fn foreign_source_id_on_complete_tag_reports_identity_mismatch() {
    let supplied = src(1, "<div>");
    let foreign = src(2, "<div>");
    let complete = foreign.anchor(0, 5).unwrap();
    let raw_name = supplied.anchor(1, 4).unwrap();

    let error = validate_occurrence_evidence(&supplied, 0, &complete, &raw_name).unwrap_err();
    assert_eq!(
        error,
        HtmlStartTagAnalysisError::OccurrenceSourceIdentityMismatch {
            occurrence_index: 0,
            role: HtmlOccurrenceEvidenceRole::CompleteTag,
            expected: SourceId::new(1),
            actual: SourceId::new(2),
        }
    );
}

#[test]
fn foreign_source_id_on_raw_name_reports_identity_mismatch() {
    let supplied = src(1, "<div>");
    let foreign = src(2, "<div>");
    let complete = supplied.anchor(0, 5).unwrap();
    let raw_name = foreign.anchor(1, 4).unwrap();

    let error = validate_occurrence_evidence(&supplied, 0, &complete, &raw_name).unwrap_err();
    assert_eq!(
        error,
        HtmlStartTagAnalysisError::OccurrenceSourceIdentityMismatch {
            occurrence_index: 0,
            role: HtmlOccurrenceEvidenceRole::RawName,
            expected: SourceId::new(1),
            actual: SourceId::new(2),
        }
    );
}

#[test]
fn same_source_id_with_out_of_bounds_range_reports_range_invalid() {
    let supplied = src(5, "<a>");
    let longer = src(5, "<div>x");
    let complete = longer.anchor(0, 5).unwrap();
    let raw_name = longer.anchor(1, 4).unwrap();

    let error = validate_occurrence_evidence(&supplied, 0, &complete, &raw_name).unwrap_err();
    assert_eq!(
        error,
        HtmlStartTagAnalysisError::OccurrenceSourceRangeInvalid {
            occurrence_index: 0,
            role: HtmlOccurrenceEvidenceRole::CompleteTag,
            error: SourceRangeError::OutOfBounds {
                start: 0,
                end: 5,
                source_len: 3,
            },
        }
    );
}

#[test]
fn same_source_id_with_differing_content_reports_content_mismatch() {
    let supplied = src(9, "<div>");
    let other = src(9, "<span");
    let complete = other.anchor(0, 5).unwrap();
    let raw_name = other.anchor(1, 4).unwrap();

    let error = validate_occurrence_evidence(&supplied, 0, &complete, &raw_name).unwrap_err();
    assert_eq!(
        error,
        HtmlStartTagAnalysisError::OccurrenceSourceContentMismatch {
            occurrence_index: 0,
            role: HtmlOccurrenceEvidenceRole::CompleteTag,
        }
    );
}

#[test]
fn raw_name_outside_complete_reports_containment_violation() {
    let source = src(1, "<div><span>");
    let complete = source.anchor(0, 5).unwrap();
    let raw_name = source.anchor(6, 10).unwrap();

    let error = validate_occurrence_evidence(&source, 0, &complete, &raw_name).unwrap_err();
    assert_eq!(
        error,
        HtmlStartTagAnalysisError::OccurrenceContainmentViolation {
            occurrence_index: 0
        }
    );
}

#[test]
fn error_debug_output_redacts_source_content() {
    const SECRET: &str = "private-analysis-marker";
    let supplied = src(1, &format!("<{SECRET}>"));
    let foreign = src(2, &format!("<{SECRET}>"));
    let complete = foreign.anchor(0, supplied.as_str().len()).unwrap();
    let raw_name = supplied.anchor(1, 1 + SECRET.len()).unwrap();

    let error = validate_occurrence_evidence(&supplied, 0, &complete, &raw_name).unwrap_err();
    let debug = format!("{error:?}");
    assert!(!debug.contains(SECRET));
    assert!(!error.to_string().contains(SECRET));
}
