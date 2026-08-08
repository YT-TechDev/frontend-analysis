use crate::{SourceId, SourceText};

use super::token::{
    CssCommentEvidence, CssCommentTermination, CssLexicalItem, CssToken, CssTokenKind,
};
use super::tokenizer::diagnostic::{
    CssTokenizerDiagnostic, CssTokenizerDiagnosticCode, CssTokenizerDiagnosticContext,
    CssTokenizerDiagnosticContractError, CssTokenizerDiagnosticSubject, CssTokenizerRecovery,
};
use super::tokenizer::resource::{
    CssTokenizerInvalidConfiguration, CssTokenizerLimits, CssTokenizerResourceContractError,
    CssTokenizerResourceKind, CssTokenizerResourceLimitEvidence, CssTokenizerResourceUsage,
};
use super::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerInvariantViolation, CssTokenizerRunError,
    CssTokenizerRunEvidenceRole, CssTokenizerRunResult, CssTokenizerTermination,
};

fn source(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn token(source: &SourceText, start: usize, end: usize, kind: CssTokenKind) -> CssLexicalItem {
    CssLexicalItem::SemanticToken(
        CssToken::new(source, source.anchor(start, end).unwrap(), kind).unwrap(),
    )
}

fn usage(item_count: usize, diagnostic_count: usize) -> CssTokenizerResourceUsage {
    CssTokenizerResourceUsage::new(0, 1, item_count, diagnostic_count, 0, 0)
}

fn complete_run(
    source: &SourceText,
    leading_bom: Option<crate::SourceAnchor>,
    lexical_items: Vec<CssLexicalItem>,
    diagnostics: Vec<CssTokenizerDiagnostic>,
) -> Result<CssTokenizerRunResult, CssTokenizerRunError> {
    let len = source.as_str().len();
    let item_count = lexical_items.len();
    let diagnostic_count = diagnostics.len();
    CssTokenizerRunResult::new(
        source,
        leading_bom,
        lexical_items,
        diagnostics,
        source.anchor(0, len).unwrap(),
        source.anchor(len, len).unwrap(),
        source.anchor(len, len).unwrap(),
        CssTokenizerCompletion::Complete,
        CssTokenizerTermination::EndOfInput,
        usage(item_count, diagnostic_count),
    )
}

#[test]
fn complete_clean_run_preserves_full_raw_coverage() {
    let source = source(1, "a");
    let result = complete_run(
        &source,
        None,
        vec![token(&source, 0, 1, CssTokenKind::Ident("a".to_owned()))],
        vec![],
    )
    .unwrap();

    assert_eq!(result.source_id(), SourceId::new(1));
    assert_eq!(result.completion(), CssTokenizerCompletion::Complete);
    assert_eq!(result.termination(), &CssTokenizerTermination::EndOfInput);
    assert_eq!(result.processed_prefix().range().start(), 0);
    assert_eq!(result.processed_prefix().range().end(), 1);
    assert!(result.unprocessed_remainder().range().is_empty());
    assert!(result.terminal().range().is_empty());
    assert!(result.diagnostics().is_empty());
}

#[test]
fn complete_run_may_preserve_recoverable_diagnostic() {
    let source = source(2, "\\");
    let item = token(&source, 0, 1, CssTokenKind::Delim('\\'));
    let diagnostic = CssTokenizerDiagnostic::new(
        &source,
        CssTokenizerDiagnosticCode::InvalidEscape,
        source.anchor(0, 1).unwrap(),
        CssTokenizerDiagnosticContext::TokenStart,
        CssTokenizerRecovery::EmitDelimiter,
        CssTokenizerDiagnosticSubject::LexicalItem { index: 0 },
    )
    .unwrap();

    let result = complete_run(&source, None, vec![item], vec![diagnostic]).unwrap();
    assert_eq!(result.completion(), CssTokenizerCompletion::Complete);
    assert_eq!(result.diagnostics().len(), 1);
}

#[test]
fn every_first_slice_diagnostic_has_one_explicit_handling_contract() {
    let source = source(3, "x");
    let location = source.anchor(0, 0).unwrap();
    let cases = [
        (
            CssTokenizerDiagnosticCode::InvalidEscape,
            CssTokenizerDiagnosticContext::TokenStart,
            CssTokenizerRecovery::EmitDelimiter,
        ),
        (
            CssTokenizerDiagnosticCode::EofInComment,
            CssTokenizerDiagnosticContext::Comment,
            CssTokenizerRecovery::PreserveUnterminatedComment,
        ),
        (
            CssTokenizerDiagnosticCode::EofInString,
            CssTokenizerDiagnosticContext::String,
            CssTokenizerRecovery::EmitStringAtEndOfInput,
        ),
        (
            CssTokenizerDiagnosticCode::NewlineInString,
            CssTokenizerDiagnosticContext::String,
            CssTokenizerRecovery::EmitBadStringAndReconsumeNewline,
        ),
        (
            CssTokenizerDiagnosticCode::EofInUrl,
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitUrlAtEndOfInput,
        ),
        (
            CssTokenizerDiagnosticCode::InvalidUrlCodePoint,
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitBadUrl,
        ),
        (
            CssTokenizerDiagnosticCode::InvalidUrlEscape,
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitBadUrl,
        ),
        (
            CssTokenizerDiagnosticCode::EofInEscape,
            CssTokenizerDiagnosticContext::Escape,
            CssTokenizerRecovery::SubstituteReplacementCharacterForEofEscape,
        ),
    ];

    for (code, context, recovery) in cases {
        let diagnostic = CssTokenizerDiagnostic::new(
            &source,
            code,
            location.clone(),
            context,
            recovery,
            CssTokenizerDiagnosticSubject::InputLocation,
        )
        .unwrap();
        assert_eq!(diagnostic.code(), code);
        assert_eq!(diagnostic.context(), context);
        assert_eq!(diagnostic.recovery(), recovery);
    }
}

#[test]
fn diagnostic_rejects_wrong_handling_and_cross_source_evidence() {
    let other = source(5, "x");
    let source = source(4, "x");

    assert!(matches!(
        CssTokenizerDiagnostic::new(
            &source,
            CssTokenizerDiagnosticCode::InvalidEscape,
            source.anchor(0, 0).unwrap(),
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitBadUrl,
            CssTokenizerDiagnosticSubject::InputLocation,
        )
        .unwrap_err(),
        CssTokenizerDiagnosticContractError::InvalidHandling { .. }
    ));

    assert!(matches!(
        CssTokenizerDiagnostic::new(
            &source,
            CssTokenizerDiagnosticCode::InvalidEscape,
            other.anchor(0, 0).unwrap(),
            CssTokenizerDiagnosticContext::TokenStart,
            CssTokenizerRecovery::EmitDelimiter,
            CssTokenizerDiagnosticSubject::InputLocation,
        )
        .unwrap_err(),
        CssTokenizerDiagnosticContractError::SourceIdentityMismatch { .. }
    ));
}

#[test]
fn malformed_bad_string_and_eof_comment_keep_subject_evidence() {
    let string_source = source(6, "\"x\n");
    let bad_string = token(&string_source, 0, 2, CssTokenKind::BadString);
    let newline = token(&string_source, 2, 3, CssTokenKind::Whitespace);
    let string_diagnostic = CssTokenizerDiagnostic::new(
        &string_source,
        CssTokenizerDiagnosticCode::NewlineInString,
        string_source.anchor(2, 2).unwrap(),
        CssTokenizerDiagnosticContext::String,
        CssTokenizerRecovery::EmitBadStringAndReconsumeNewline,
        CssTokenizerDiagnosticSubject::LexicalItem { index: 0 },
    )
    .unwrap();
    complete_run(
        &string_source,
        None,
        vec![bad_string, newline],
        vec![string_diagnostic],
    )
    .unwrap();

    let comment_source = source(7, "/*x");
    let comment = CssCommentEvidence::new(
        &comment_source,
        comment_source.anchor(0, 3).unwrap(),
        comment_source.anchor(0, 2).unwrap(),
        CssCommentTermination::EndOfInput,
    )
    .unwrap();
    let comment_diagnostic = CssTokenizerDiagnostic::new(
        &comment_source,
        CssTokenizerDiagnosticCode::EofInComment,
        comment_source.anchor(3, 3).unwrap(),
        CssTokenizerDiagnosticContext::Comment,
        CssTokenizerRecovery::PreserveUnterminatedComment,
        CssTokenizerDiagnosticSubject::LexicalItem { index: 0 },
    )
    .unwrap();
    complete_run(
        &comment_source,
        None,
        vec![CssLexicalItem::Comment(comment)],
        vec![comment_diagnostic],
    )
    .unwrap();
}

#[test]
fn resource_limit_is_explicit_for_every_first_slice_resource_kind() {
    let kinds = [
        CssTokenizerResourceKind::SourceBytes,
        CssTokenizerResourceKind::AlgorithmSteps,
        CssTokenizerResourceKind::LexicalItems,
        CssTokenizerResourceKind::Diagnostics,
        CssTokenizerResourceKind::RetainedInterpretedBytes,
        CssTokenizerResourceKind::TemporaryBufferBytes,
    ];

    for (index, kind) in kinds.into_iter().enumerate() {
        let source = source(20 + index as u64, "a");
        let terminal = source.anchor(0, 0).unwrap();
        let limit =
            CssTokenizerResourceLimitEvidence::new(&source, kind, 0, 1, terminal.clone()).unwrap();
        let result = CssTokenizerRunResult::new(
            &source,
            None,
            vec![],
            vec![],
            source.anchor(0, 0).unwrap(),
            source.anchor(0, 1).unwrap(),
            terminal,
            CssTokenizerCompletion::Incomplete,
            CssTokenizerTermination::ResourceLimit(limit),
            usage(0, 0),
        )
        .unwrap();

        assert_eq!(result.completion(), CssTokenizerCompletion::Incomplete);
        match result.termination() {
            CssTokenizerTermination::ResourceLimit(evidence) => {
                assert_eq!(evidence.kind(), kind);
                assert_eq!(evidence.limit(), 0);
                assert_eq!(evidence.attempted(), 1);
            }
            CssTokenizerTermination::EndOfInput => panic!("expected resource termination"),
        }
    }
}

#[test]
fn zero_algorithm_steps_is_invalid_configuration() {
    let error = CssTokenizerLimits::new(0, 0, 0, 0, 0, 0).unwrap_err();
    assert_eq!(error, CssTokenizerInvalidConfiguration::ZeroAlgorithmSteps);
    assert_eq!(
        CssTokenizerRunError::from(error),
        CssTokenizerRunError::InvalidConfiguration(
            CssTokenizerInvalidConfiguration::ZeroAlgorithmSteps
        )
    );
}

#[test]
fn zero_capacities_other_than_algorithm_steps_are_explicitly_constructible() {
    let limits = CssTokenizerLimits::new(0, 1, 0, 0, 0, 0).unwrap();
    assert_eq!(limits.limit(CssTokenizerResourceKind::SourceBytes), 0);
    assert_eq!(limits.limit(CssTokenizerResourceKind::AlgorithmSteps), 1);
    assert_eq!(limits.limit(CssTokenizerResourceKind::LexicalItems), 0);
}

#[test]
fn resource_evidence_requires_attempted_value_to_exceed_limit() {
    let source = source(30, "a");
    assert_eq!(
        CssTokenizerResourceLimitEvidence::new(
            &source,
            CssTokenizerResourceKind::LexicalItems,
            1,
            1,
            source.anchor(0, 0).unwrap(),
        )
        .unwrap_err(),
        CssTokenizerResourceContractError::AttemptDidNotExceedLimit {
            limit: 1,
            attempted: 1,
        }
    );
}

#[test]
fn resource_terminal_location_must_be_an_exact_point() {
    let source = source(301, "a");
    assert_eq!(
        CssTokenizerResourceLimitEvidence::new(
            &source,
            CssTokenizerResourceKind::SourceBytes,
            0,
            1,
            source.anchor(0, 1).unwrap(),
        )
        .unwrap_err(),
        CssTokenizerResourceContractError::LocationMustBePoint { start: 0, end: 1 }
    );
}

#[test]
fn complete_and_incomplete_termination_states_cannot_be_collapsed() {
    let source = source(31, "a");
    let limit = CssTokenizerResourceLimitEvidence::new(
        &source,
        CssTokenizerResourceKind::AlgorithmSteps,
        1,
        2,
        source.anchor(0, 0).unwrap(),
    )
    .unwrap();

    assert_eq!(
        CssTokenizerRunResult::new(
            &source,
            None,
            vec![],
            vec![],
            source.anchor(0, 0).unwrap(),
            source.anchor(0, 1).unwrap(),
            source.anchor(0, 0).unwrap(),
            CssTokenizerCompletion::Complete,
            CssTokenizerTermination::ResourceLimit(limit),
            usage(0, 0),
        )
        .unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::CompletionTerminationMismatch
        )
    );
}

#[test]
fn leading_bom_is_run_evidence_and_not_a_lexical_item() {
    let source = source(32, "\u{feff}a");
    let result = complete_run(
        &source,
        Some(source.anchor(0, 3).unwrap()),
        vec![token(&source, 3, 4, CssTokenKind::Ident("a".to_owned()))],
        vec![],
    )
    .unwrap();

    assert_eq!(result.leading_bom().unwrap().range().start(), 0);
    assert_eq!(result.leading_bom().unwrap().range().end(), 3);
    assert_eq!(result.lexical_items().len(), 1);
}

#[test]
fn missing_or_spurious_leading_bom_evidence_is_rejected() {
    let with_bom = source(33, "\u{feff}");
    assert_eq!(
        complete_run(&with_bom, None, vec![], vec![]).unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::MissingLeadingBomEvidence
        )
    );

    let without_bom = source(34, "a");
    assert_eq!(
        complete_run(
            &without_bom,
            Some(without_bom.anchor(0, 1).unwrap()),
            vec![],
            vec![],
        )
        .unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::UnexpectedLeadingBomEvidence
        )
    );
}

#[test]
fn incomplete_result_preserves_exact_processed_prefix_and_remainder() {
    let source = source(35, "ab");
    let item = token(&source, 0, 1, CssTokenKind::Ident("a".to_owned()));
    let terminal = source.anchor(1, 1).unwrap();
    let limit = CssTokenizerResourceLimitEvidence::new(
        &source,
        CssTokenizerResourceKind::AlgorithmSteps,
        10,
        11,
        terminal.clone(),
    )
    .unwrap();

    let result = CssTokenizerRunResult::new(
        &source,
        None,
        vec![item],
        vec![],
        source.anchor(0, 1).unwrap(),
        source.anchor(1, 2).unwrap(),
        terminal,
        CssTokenizerCompletion::Incomplete,
        CssTokenizerTermination::ResourceLimit(limit),
        usage(1, 0),
    )
    .unwrap();

    assert_eq!(result.processed_prefix().fragment(), "a");
    assert_eq!(result.unprocessed_remainder().fragment(), "b");
}

#[test]
fn cross_source_lexical_evidence_is_rejected() {
    let expected = source(36, "a");
    let other = source(37, "a");
    let item = token(&other, 0, 1, CssTokenKind::Ident("a".to_owned()));

    assert!(matches!(
        complete_run(&expected, None, vec![item], vec![]).unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::SourceIdentityMismatch {
                role: CssTokenizerRunEvidenceRole::LexicalItem { index: 0 },
                ..
            }
        )
    ));
}

#[test]
fn corrupt_diagnostic_lexical_index_is_rejected_at_collection_boundary() {
    let source = source(38, "a");
    let item = token(&source, 0, 1, CssTokenKind::Ident("a".to_owned()));
    let diagnostic = CssTokenizerDiagnostic::new(
        &source,
        CssTokenizerDiagnosticCode::EofInEscape,
        source.anchor(1, 1).unwrap(),
        CssTokenizerDiagnosticContext::Escape,
        CssTokenizerRecovery::SubstituteReplacementCharacterForEofEscape,
        CssTokenizerDiagnosticSubject::LexicalItem { index: 1 },
    )
    .unwrap();

    assert!(matches!(
        complete_run(&source, None, vec![item], vec![diagnostic]).unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::DiagnosticContractViolation {
                index: 0,
                error: CssTokenizerDiagnosticContractError::LexicalItemSubjectOutOfBounds {
                    index: 1,
                    item_count: 1,
                },
            }
        )
    ));
}

#[test]
fn equal_offset_diagnostics_preserve_producer_order() {
    let source = source(39, "a");
    let item = token(&source, 0, 1, CssTokenKind::Ident("a".to_owned()));
    let location = source.anchor(1, 1).unwrap();
    let first = CssTokenizerDiagnostic::new(
        &source,
        CssTokenizerDiagnosticCode::EofInString,
        location.clone(),
        CssTokenizerDiagnosticContext::String,
        CssTokenizerRecovery::EmitStringAtEndOfInput,
        CssTokenizerDiagnosticSubject::InputLocation,
    )
    .unwrap();
    let second = CssTokenizerDiagnostic::new(
        &source,
        CssTokenizerDiagnosticCode::EofInEscape,
        location,
        CssTokenizerDiagnosticContext::Escape,
        CssTokenizerRecovery::SubstituteReplacementCharacterForEofEscape,
        CssTokenizerDiagnosticSubject::InputLocation,
    )
    .unwrap();

    let result = complete_run(&source, None, vec![item], vec![first, second]).unwrap();
    assert_eq!(
        result.diagnostics()[0].code(),
        CssTokenizerDiagnosticCode::EofInString
    );
    assert_eq!(
        result.diagnostics()[1].code(),
        CssTokenizerDiagnosticCode::EofInEscape
    );
}

#[test]
fn decreasing_diagnostic_source_order_is_rejected() {
    let source = source(40, "ab");
    let items = vec![
        token(&source, 0, 1, CssTokenKind::Ident("a".to_owned())),
        token(&source, 1, 2, CssTokenKind::Ident("b".to_owned())),
    ];
    let later = CssTokenizerDiagnostic::new(
        &source,
        CssTokenizerDiagnosticCode::EofInEscape,
        source.anchor(2, 2).unwrap(),
        CssTokenizerDiagnosticContext::Escape,
        CssTokenizerRecovery::SubstituteReplacementCharacterForEofEscape,
        CssTokenizerDiagnosticSubject::InputLocation,
    )
    .unwrap();
    let earlier = CssTokenizerDiagnostic::new(
        &source,
        CssTokenizerDiagnosticCode::InvalidEscape,
        source.anchor(0, 0).unwrap(),
        CssTokenizerDiagnosticContext::TokenStart,
        CssTokenizerRecovery::EmitDelimiter,
        CssTokenizerDiagnosticSubject::InputLocation,
    )
    .unwrap();

    assert_eq!(
        complete_run(&source, None, items, vec![later, earlier]).unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::DiagnosticOrderViolation { index: 1 }
        )
    );
}

#[test]
fn lexical_gaps_and_overlap_are_rejected() {
    let source = source(41, "ab");
    let gap_item = token(&source, 1, 2, CssTokenKind::Ident("b".to_owned()));
    assert!(matches!(
        complete_run(&source, None, vec![gap_item], vec![]).unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::LexicalGap { index: 0, .. }
        )
    ));

    let first = token(&source, 0, 1, CssTokenKind::Ident("a".to_owned()));
    let overlap = token(&source, 0, 2, CssTokenKind::Ident("ab".to_owned()));
    assert!(matches!(
        complete_run(&source, None, vec![first, overlap], vec![]).unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::LexicalOverlapOrOutOfOrder { index: 1, .. }
        )
    ));
}

#[test]
fn resource_usage_counts_must_match_committed_evidence() {
    let source = source(42, "a");
    let item = token(&source, 0, 1, CssTokenKind::Ident("a".to_owned()));
    let len = source.as_str().len();

    assert_eq!(
        CssTokenizerRunResult::new(
            &source,
            None,
            vec![item],
            vec![],
            source.anchor(0, len).unwrap(),
            source.anchor(len, len).unwrap(),
            source.anchor(len, len).unwrap(),
            CssTokenizerCompletion::Complete,
            CssTokenizerTermination::EndOfInput,
            usage(0, 0),
        )
        .unwrap_err(),
        CssTokenizerRunError::InternalInvariantFailure(
            CssTokenizerInvariantViolation::ResourceUsageCountMismatch {
                kind: CssTokenizerResourceKind::LexicalItems,
                expected: 1,
                actual: 0,
            }
        )
    );
}

#[test]
fn multibyte_locations_remain_utf8_byte_based() {
    let source = source(43, "é");
    let item = token(&source, 0, 2, CssTokenKind::Ident("é".to_owned()));
    let diagnostic = CssTokenizerDiagnostic::new(
        &source,
        CssTokenizerDiagnosticCode::EofInEscape,
        source.anchor(2, 2).unwrap(),
        CssTokenizerDiagnosticContext::Escape,
        CssTokenizerRecovery::SubstituteReplacementCharacterForEofEscape,
        CssTokenizerDiagnosticSubject::InputLocation,
    )
    .unwrap();
    let result = complete_run(&source, None, vec![item], vec![diagnostic]).unwrap();

    assert_eq!(result.terminal().range().start(), 2);
    assert_eq!(result.diagnostics()[0].location().range().start(), 2);
}

#[test]
fn input_region_subject_requires_location_containment() {
    let source = source(44, "ab");
    assert_eq!(
        CssTokenizerDiagnostic::new(
            &source,
            CssTokenizerDiagnosticCode::InvalidEscape,
            source.anchor(2, 2).unwrap(),
            CssTokenizerDiagnosticContext::TokenStart,
            CssTokenizerRecovery::EmitDelimiter,
            CssTokenizerDiagnosticSubject::InputRegion {
                region: source.anchor(0, 1).unwrap(),
            },
        )
        .unwrap_err(),
        CssTokenizerDiagnosticContractError::LocationOutsideInputRegion
    );
}

#[test]
fn debug_and_errors_do_not_disclose_authored_source() {
    const SECRET: &str = "private-secret";
    let source = source(45, SECRET);
    let item = token(
        &source,
        0,
        SECRET.len(),
        CssTokenKind::Ident(SECRET.to_owned()),
    );
    let result = complete_run(&source, None, vec![item], vec![]).unwrap();
    let debug = format!("{result:?}");

    assert!(!debug.contains(SECRET));

    let error = CssTokenizerRunResult::new(
        &source,
        None,
        vec![],
        vec![],
        source.anchor(0, SECRET.len()).unwrap(),
        source.anchor(SECRET.len(), SECRET.len()).unwrap(),
        source.anchor(SECRET.len(), SECRET.len()).unwrap(),
        CssTokenizerCompletion::Complete,
        CssTokenizerTermination::EndOfInput,
        usage(0, 0),
    )
    .unwrap_err();
    assert!(!format!("{error:?}").contains(SECRET));
    assert!(!error.to_string().contains(SECRET));
}
