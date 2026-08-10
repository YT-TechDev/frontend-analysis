//! Focused #140 Core-integration tests: native demonstration, historical
//! regressions, preprocessing/raw-provenance, lifecycle/resource
//! propagation, recovery/discard/unsupported propagation, Core-boundary
//! corruption, and error non-disclosure.
//!
//! Candidate-independent gold/generated-corpus validation lives in
//! `super::super::validation::core_analysis_gate` instead, alongside the
//! existing #138/#139 parser validation layers.

use super::*;
use crate::css::parser::evidence::CssParserRecoveryTermination;
use crate::css::parser::result::{CssParserExecutionCompletion, CssParserTermination};
use crate::css::tokenizer::result::{CssTokenizerCompletion, CssTokenizerTermination};

fn generous_tokenizer_limits() -> CssTokenizerLimits {
    CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
}

fn generous_parser_limits() -> CssParserLimits {
    CssParserLimits::new(
        1 << 20,
        1 << 12,
        1 << 12,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
        1 << 16,
    )
    .unwrap()
}

fn span(anchor: &SourceAnchor) -> (usize, usize) {
    (anchor.range().start(), anchor.range().end())
}

// ---------------------------------------------------------------------
// Native demonstration.
// ---------------------------------------------------------------------

#[test]
fn native_demonstration_multibyte_declaration_survives_source_handle_drop() {
    let result = {
        let source = SourceText::new(SourceId::new(101), "a{--é:中;}".to_owned());
        analyze_css_declarations(
            &source,
            generous_tokenizer_limits(),
            generous_parser_limits(),
        )
        .unwrap()
    };

    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(span(occurrence.complete()), (2, 11));
    assert_eq!(span(occurrence.property_name()), (2, 6));
    assert_eq!(occurrence.property_name().fragment(), "--é");
    assert_eq!(span(occurrence.colon()), (6, 7));
    assert_eq!(span(occurrence.value()), (7, 10));
    assert_eq!(occurrence.value().fragment(), "中");
    assert!(occurrence.priority().is_none());
}

// ---------------------------------------------------------------------
// Historical regressions, preserved exactly through the Core operation.
// ---------------------------------------------------------------------

#[test]
fn historical_escaped_property_provenance_regression_matches_exactly_through_core() {
    let source = SourceText::new(SourceId::new(50_101), r"a{c\6F lor/**/:red;}".to_owned());
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(span(occurrence.complete()), (2, 19));
    assert_eq!(span(occurrence.property_name()), (2, 10));
    assert_eq!(span(occurrence.colon()), (14, 15));
    assert_eq!(span(occurrence.value()), (15, 18));
    assert!(occurrence.priority().is_none());
}

#[test]
fn historical_silent_priority_loss_regression_matches_exactly_through_core() {
    let source = SourceText::new(
        SourceId::new(50_102),
        r"a{color:red !/**/Im\70 ortant;}".to_owned(),
    );
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(span(occurrence.complete()), (2, 30));
    assert_eq!(span(occurrence.value()), (8, 11));
    let priority = occurrence
        .priority()
        .expect("valid priority evidence expected");
    assert_eq!(span(priority.complete()), (12, 29));
    assert_eq!(span(priority.bang()), (12, 13));
    assert_eq!(span(priority.important_ident()), (17, 29));
}

// ---------------------------------------------------------------------
// Preprocessing / raw-provenance evidence, preserved through Core.
// ---------------------------------------------------------------------

#[test]
fn preprocessing_bom_before_first_rule_preserves_exact_post_bom_offsets_through_core() {
    let source = SourceText::new(SourceId::new(61_101), "\u{feff}a{color:red;}".to_owned());
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(occurrence.property_name().range().start(), 5);
    assert_eq!(occurrence.property_name().range().end(), 10);
}

#[test]
fn preprocessing_cr_lf_crlf_ff_do_not_shift_neighboring_declaration_endpoints_through_core() {
    let source = SourceText::new(
        SourceId::new(61_102),
        "a{a1:1;\ra2:2;\na3:3;\r\na4:4;\x0ca5:5;}".to_owned(),
    );
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 5);
    let expected_starts = [2, 8, 14, 21, 27];
    for (occurrence, expected_start) in result.occurrences().iter().zip(expected_starts) {
        assert_eq!(occurrence.complete().range().start(), expected_start);
    }
}

#[test]
fn preprocessing_nul_byte_in_value_yields_ordinary_byte_exact_evidence_through_core() {
    let source = SourceText::new(SourceId::new(61_103), "a{color:\0;}".to_owned());
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    assert_eq!(span(result.occurrences()[0].value()), (8, 9));
}

#[test]
fn preprocessing_multibyte_utf8_property_and_value_preserve_exact_byte_ranges_through_core() {
    let source = SourceText::new(SourceId::new(61_104), "a{--é:中;}".to_owned());
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(span(occurrence.property_name()), (2, 6));
    assert_eq!(span(occurrence.value()), (7, 10));
}

#[test]
fn preprocessing_comment_between_property_name_and_colon_stays_inside_complete_only_through_core() {
    let source = SourceText::new(SourceId::new(61_105), "a{color/*gap*/:red;}".to_owned());
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    let occurrence = &result.occurrences()[0];
    assert_eq!(occurrence.property_name().fragment(), "color");
    assert_eq!(occurrence.colon().range().start(), 14);
}

#[test]
fn preprocessing_comment_inside_priority_stays_inside_priority_complete_through_core() {
    let source = SourceText::new(
        SourceId::new(61_106),
        "a{color:red !/*gap*/important;}".to_owned(),
    );
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    let priority = result.occurrences()[0]
        .priority()
        .expect("valid priority expected");
    assert_eq!(priority.bang().fragment(), "!");
    assert_eq!(priority.important_ident().fragment(), "important");
}

// ---------------------------------------------------------------------
// Real lifecycle/resource-limit propagation through the Core operation.
// ---------------------------------------------------------------------

#[test]
fn real_tokenizer_resource_limit_propagates_incomplete_through_core() {
    let source = SourceText::new(SourceId::new(70_001), "a{color:red;}".to_owned());
    // A genuinely small SourceBytes capacity forces the real project-owned
    // tokenizer to terminate with Incomplete + ResourceLimit; no synthetic
    // CssTokenizerRunResult is fabricated.
    let tokenizer_limits =
        CssTokenizerLimits::new(0, 1 << 10, 1 << 10, 1 << 10, 1 << 10, 1 << 10).unwrap();

    let result = analyze_css_declarations(&source, tokenizer_limits, generous_parser_limits())
        .expect("a real upstream-incomplete run remains an Ok CssParserRunResult");

    let upstream = result.upstream_tokenizer_result();
    assert_eq!(upstream.completion(), CssTokenizerCompletion::Incomplete);
    assert!(matches!(
        upstream.termination(),
        CssTokenizerTermination::ResourceLimit(_)
    ));

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert!(matches!(
        result.termination(),
        CssParserTermination::UpstreamTokenizerIncomplete
    ));
    assert!(result.terminal().range().start() <= upstream.terminal().range().start());
    assert!(result.occurrences().is_empty());
    assert!(result.recovery_records().is_empty());
}

#[test]
fn parser_resource_limit_propagates_incomplete_through_core_without_becoming_err() {
    let source = SourceText::new(SourceId::new(70_002), "a{color:red;}".to_owned());
    let parser_limits = CssParserLimits::new(
        1 << 10,
        1 << 10,
        1 << 10,
        0,
        1 << 10,
        1 << 10,
        1 << 10,
        1 << 10,
        1 << 10,
    )
    .unwrap();

    let result = analyze_css_declarations(&source, generous_tokenizer_limits(), parser_limits)
        .expect("a parser resource limit is a valid CssParserRunResult, not a Core Err");

    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Incomplete
    );
    assert!(matches!(
        result.termination(),
        CssParserTermination::ParserResourceLimit(_)
    ));
    assert!(result.occurrences().is_empty());
}

// ---------------------------------------------------------------------
// Recovery / discard / unsupported / true-EOF propagation through Core.
// ---------------------------------------------------------------------

#[test]
fn recovery_propagates_through_core() {
    let source = SourceText::new(
        SourceId::new(70_003),
        "a{color red;background:blue;}".to_owned(),
    );
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    assert_eq!(result.parser_diagnostics().len(), 1);
    assert_eq!(result.recovery_records().len(), 1);
}

#[test]
fn discard_propagates_through_core() {
    let source = SourceText::new(SourceId::new(70_004), "--foo:bar{color:red;}".to_owned());
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert!(result.occurrences().is_empty());
    assert_eq!(result.discard_records().len(), 1);
}

/// #167 supersedes the old whole-remainder outcome for a nested *qualified
/// rule*: this source now produces zero unsupported regions, so this test
/// uses a nested at-rule instead to keep exercising unsupported-region
/// propagation through Core. #168 further supersedes the old whole-remainder
/// outcome for `@media screen` specifically (now a supported group context),
/// so an unrecognized at-rule name is used to keep this source's nested
/// at-rule genuinely unsupported.
#[test]
fn unsupported_propagates_through_core() {
    let source = SourceText::new(
        SourceId::new(70_005),
        "a{color:red;@unknown-rule{color:blue;}}".to_owned(),
    );
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert_eq!(result.occurrences().len(), 1);
    assert_eq!(result.unsupported_regions().len(), 1);
}

#[test]
fn true_eof_malformed_recovery_propagates_through_core() {
    let source = SourceText::new(SourceId::new(70_006), "a{color red".to_owned());
    let result = analyze_css_declarations(
        &source,
        generous_tokenizer_limits(),
        generous_parser_limits(),
    )
    .unwrap();

    assert!(result.occurrences().is_empty());
    assert_eq!(result.parser_diagnostics().len(), 1);
    assert_eq!(result.recovery_records().len(), 1);
    assert!(matches!(
        result.recovery_records()[0].termination(),
        CssParserRecoveryTermination::EndOfInput { .. }
    ));
    assert_eq!(
        result.execution_completion(),
        CssParserExecutionCompletion::Complete
    );
}

// ---------------------------------------------------------------------
// Core-boundary corruption tests. Production parser output should already
// be valid; these exercise the private `validate_occurrence_evidence`
// boundary directly with deliberately corrupted anchors, mirroring the
// #116 HTML precedent, rather than weakening any production constructor.
// ---------------------------------------------------------------------

#[test]
fn corruption_cross_source_property_name_is_source_identity_mismatch() {
    let source = SourceText::new(SourceId::new(200), "color:red;".to_owned());
    let other = SourceText::new(SourceId::new(201), "color:red;".to_owned());

    let complete = source.anchor(0, 10).unwrap();
    let property_name = other.anchor(0, 5).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: source.anchor(9, 10).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceSourceIdentityMismatch {
                role: CssDeclarationEvidenceRole::PropertyName,
                ..
            }
        )
    ));
}

#[test]
fn corruption_same_source_id_but_range_invalid_for_supplied_source() {
    let short_source = SourceText::new(SourceId::new(301), "abc".to_owned());
    let long_source = SourceText::new(SourceId::new(301), "abcdefghij;".to_owned());

    let complete = long_source.anchor(0, 11).unwrap();
    let property_name = short_source.anchor(0, 1).unwrap();
    let colon = short_source.anchor(1, 2).unwrap();
    let value = short_source.anchor(2, 3).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: short_source.anchor(3, 3).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &short_source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(CssDeclarationAnalysisError::OccurrenceSourceRangeInvalid {
            role: CssDeclarationEvidenceRole::Complete,
            ..
        })
    ));
}

#[test]
fn corruption_same_source_id_and_valid_range_but_different_retained_content() {
    let source = SourceText::new(SourceId::new(302), "color:red;".to_owned());
    let other = SourceText::new(SourceId::new(302), "COLOR:RED;".to_owned());

    let complete = other.anchor(0, 10).unwrap();
    let property_name = source.anchor(0, 5).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: source.anchor(9, 10).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceSourceContentMismatch {
                role: CssDeclarationEvidenceRole::Complete,
                ..
            }
        )
    ));
}

#[test]
fn corruption_property_name_extends_beyond_complete_is_relationship_violation() {
    let source = SourceText::new(SourceId::new(304), "color:red;".to_owned());
    let complete = source.anchor(0, 5).unwrap();
    let property_name = source.anchor(0, 10).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: source.anchor(9, 10).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
                relationship: CssOccurrenceRelationship::PropertyNameContained,
                ..
            }
        )
    ));
}

#[test]
fn corruption_colon_precedes_property_name_end_is_relationship_violation() {
    let source = SourceText::new(SourceId::new(305), "color:red;".to_owned());
    let complete = source.anchor(0, 10).unwrap();
    let property_name = source.anchor(0, 6).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: source.anchor(9, 10).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
                relationship: CssOccurrenceRelationship::PropertyNameEndsAtOrBeforeColonStart,
                ..
            }
        )
    ));
}

#[test]
fn corruption_value_begins_before_colon_end_is_relationship_violation() {
    let source = SourceText::new(SourceId::new(306), "color:red;".to_owned());
    let complete = source.anchor(0, 10).unwrap();
    let property_name = source.anchor(0, 5).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(4, 9).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: source.anchor(9, 10).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
                relationship: CssOccurrenceRelationship::ColonEndsAtOrBeforeValueStart,
                ..
            }
        )
    ));
}

#[test]
fn corruption_priority_extends_beyond_declaration_complete_is_relationship_violation() {
    let source = SourceText::new(SourceId::new(307), "color:red !important;".to_owned());
    let complete = source.anchor(0, 19).unwrap();
    let property_name = source.anchor(0, 5).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let priority_complete = source.anchor(10, 20).unwrap();
    let bang = source.anchor(10, 11).unwrap();
    let important_ident = source.anchor(11, 20).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: source.anchor(20, 21).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        Some((&priority_complete, &bang, &important_ident)),
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
                relationship: CssOccurrenceRelationship::PriorityContained,
                ..
            }
        )
    ));
}

#[test]
fn corruption_priority_bang_overlaps_important_ident_is_relationship_violation() {
    let source = SourceText::new(SourceId::new(308), "color:red !important;".to_owned());
    let complete = source.anchor(0, 21).unwrap();
    let property_name = source.anchor(0, 5).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let priority_complete = source.anchor(10, 20).unwrap();
    let bang = source.anchor(10, 12).unwrap();
    let important_ident = source.anchor(11, 20).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: source.anchor(20, 21).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        Some((&priority_complete, &bang, &important_ident)),
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
                relationship: CssOccurrenceRelationship::BangEndsAtOrBeforeImportantIdentStart,
                ..
            }
        )
    ));
}

#[test]
fn corruption_right_curly_precedes_complete_end_is_relationship_violation() {
    let source = SourceText::new(SourceId::new(309), "color:red}".to_owned());
    let complete = source.anchor(0, 9).unwrap();
    let property_name = source.anchor(0, 5).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let termination = CssDeclarationTermination::OmittedBeforeRightCurly {
        right_curly: source.anchor(8, 9).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
                relationship: CssOccurrenceRelationship::RightCurlyAtOrAfterCompleteEnd,
                ..
            }
        )
    ));
}

#[test]
fn corruption_eof_terminal_not_at_source_end_is_relationship_violation() {
    let source = SourceText::new(SourceId::new(310), "color:red;more".to_owned());
    let complete = source.anchor(0, 9).unwrap();
    let property_name = source.anchor(0, 5).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let termination = CssDeclarationTermination::OmittedAtEndOfInput {
        terminal: source.anchor(9, 9).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
                relationship: CssOccurrenceRelationship::EofTerminalAtSourceEnd,
                ..
            }
        )
    ));
}

#[test]
fn corruption_eof_terminal_non_empty_is_relationship_violation() {
    let source = SourceText::new(SourceId::new(311), "color:red".to_owned());
    let complete = source.anchor(0, 9).unwrap();
    let property_name = source.anchor(0, 5).unwrap();
    let colon = source.anchor(5, 6).unwrap();
    let value = source.anchor(6, 9).unwrap();
    let termination = CssDeclarationTermination::OmittedAtEndOfInput {
        terminal: source.anchor(8, 9).unwrap(),
    };

    let result = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    );
    assert!(matches!(
        result,
        Err(
            CssDeclarationAnalysisError::OccurrenceRelationshipViolation {
                relationship: CssOccurrenceRelationship::EofTerminalEmpty,
                ..
            }
        )
    ));
}

// ---------------------------------------------------------------------
// Error non-disclosure.
// ---------------------------------------------------------------------

#[test]
fn error_debug_and_display_do_not_disclose_authored_source_content() {
    const SECRET: &str = "top-secret-css-property-name-corruption";
    let source = SourceText::new(SourceId::new(400), format!("{SECRET}:red;"));
    let other = SourceText::new(SourceId::new(401), format!("{SECRET}:red;"));

    let complete = source.anchor(0, SECRET.len() + 5).unwrap();
    // Wrong source deliberately triggers an identity-mismatch error.
    let property_name = other.anchor(0, SECRET.len()).unwrap();
    let colon = source.anchor(SECRET.len(), SECRET.len() + 1).unwrap();
    let value = source.anchor(SECRET.len() + 1, SECRET.len() + 4).unwrap();
    let termination = CssDeclarationTermination::AuthoredSemicolon {
        semicolon: source.anchor(SECRET.len() + 4, SECRET.len() + 5).unwrap(),
    };

    let error = validate_occurrence_evidence(
        &source,
        0,
        &complete,
        &property_name,
        &colon,
        &value,
        None,
        &termination,
    )
    .unwrap_err();

    let debug = format!("{error:?}");
    let display = error.to_string();
    assert!(!debug.contains(SECRET));
    assert!(!display.contains(SECRET));
}
