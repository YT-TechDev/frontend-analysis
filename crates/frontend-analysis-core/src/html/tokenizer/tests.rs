use std::panic::{AssertUnwindSafe, catch_unwind};

use super::diagnostic::*;
use super::resource::*;
use super::result::*;
use crate::html::token::*;
use crate::{SourceAnchor, SourceId, SourceText};

fn src(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn anchor(source: &SourceText, start: usize, end: usize) -> SourceAnchor {
    source.anchor(start, end).unwrap()
}

fn limits() -> HtmlTokenizerLimits {
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

fn character(source: &SourceText, start: usize, end: usize, interpreted: &str) -> HtmlToken {
    HtmlToken::Character(
        HtmlCharacterToken::new(anchor(source, start, end), interpreted.to_owned()).unwrap(),
    )
}

fn tag(source: &SourceText, name_end: usize, complete_end: usize, interpreted: &str) -> HtmlToken {
    HtmlToken::Tag(
        HtmlTagToken::new(
            HtmlTagKind::Start,
            anchor(source, 0, complete_end),
            anchor(source, 0, 1),
            HtmlNameEvidence::new(anchor(source, 1, name_end), interpreted.to_owned()).unwrap(),
            Vec::new(),
            None,
            anchor(source, complete_end - 1, complete_end),
        )
        .unwrap(),
    )
}

fn end_tag(
    source: &SourceText,
    name_end: usize,
    complete_end: usize,
    interpreted: &str,
    attributes: Vec<HtmlAttributeEvidence>,
    self_closing_solidus: Option<SourceAnchor>,
) -> HtmlToken {
    end_tag_at(
        source,
        0,
        name_end,
        complete_end,
        interpreted,
        attributes,
        self_closing_solidus,
    )
}

#[allow(clippy::too_many_arguments)]
fn end_tag_at(
    source: &SourceText,
    start: usize,
    name_end: usize,
    complete_end: usize,
    interpreted: &str,
    attributes: Vec<HtmlAttributeEvidence>,
    self_closing_solidus: Option<SourceAnchor>,
) -> HtmlToken {
    HtmlToken::Tag(
        HtmlTagToken::new(
            HtmlTagKind::End,
            anchor(source, start, complete_end),
            anchor(source, start, start + 2),
            HtmlNameEvidence::new(anchor(source, start + 2, name_end), interpreted.to_owned())
                .unwrap(),
            attributes,
            self_closing_solidus,
            anchor(source, complete_end - 1, complete_end),
        )
        .unwrap(),
    )
}

fn boolean_attribute(
    source: &SourceText,
    start: usize,
    end: usize,
    interpreted: &str,
) -> HtmlAttributeEvidence {
    HtmlAttributeEvidence::new(
        anchor(source, start, end),
        HtmlNameEvidence::new(anchor(source, start, end), interpreted.to_owned()).unwrap(),
        HtmlAttributeValueSyntax::Missing,
        String::new(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap()
}

#[allow(clippy::too_many_arguments)]
fn diagnostic_full(
    source: &SourceText,
    code: HtmlTokenizerDiagnosticCode,
    start: usize,
    end: usize,
    context: HtmlTokenizerDiagnosticContext,
    handling: HtmlTokenizerDiagnosticHandling,
    subject: HtmlTokenizerDiagnosticSubject,
) -> HtmlTokenizerDiagnostic {
    HtmlTokenizerDiagnostic::new(
        source,
        code,
        anchor(source, start, end),
        context,
        handling,
        subject,
    )
    .unwrap()
}

fn diagnostic(
    source: &SourceText,
    start: usize,
    end: usize,
    subject: HtmlTokenizerDiagnosticSubject,
) -> HtmlTokenizerDiagnostic {
    HtmlTokenizerDiagnostic::new(
        source,
        HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter,
        anchor(source, start, end),
        HtmlTokenizerDiagnosticContext::Data,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::ReplacedNullWithReplacementCharacter,
        ),
        subject,
    )
    .unwrap()
}

fn usage(
    source: &SourceText,
    transition_steps: usize,
    emitted_tokens: usize,
    diagnostics: usize,
    peak_attributes: usize,
    retained_interpreted_bytes: usize,
    peak_temporary_buffer_bytes: usize,
) -> HtmlTokenizerUsage {
    HtmlTokenizerUsage::new(
        source.as_str().len(),
        transition_steps,
        emitted_tokens,
        diagnostics,
        peak_attributes,
        retained_interpreted_bytes,
        peak_temporary_buffer_bytes,
    )
}

#[test]
fn clean_complete_empty_input_has_one_eof_and_full_coverage() {
    let source = src(1, "");
    let result = HtmlTokenizerRunResult::new(
        &source,
        vec![eof(&source)],
        preprocessing(&source),
        Vec::new(),
        coverage(&source, 0),
        HtmlTokenizerCompletion::Complete,
        limits(),
        usage(&source, 1, 1, 0, 0, 0, 0),
    )
    .unwrap();

    assert!(result.is_clean_complete());
    assert!(!result.is_complete_with_diagnostics());
    assert!(!result.is_incomplete());
    assert_eq!(result.tokens().len(), 1);
    assert!(result.diagnostics().is_empty());
    assert!(result.coverage().is_complete());
    assert_eq!(result.usage().emitted_tokens(), 1);
    assert_eq!(result.limits().max_emitted_tokens(), 1_000);
    assert!(result.preprocessing().skipped_leading_bom().is_none());
    assert!(matches!(
        result.completion(),
        HtmlTokenizerCompletion::Complete
    ));
}

#[test]
fn complete_recovered_result_preserves_diagnostic_and_token_evidence() {
    let source = src(2, "\0é");
    let tokens = vec![
        character(&source, 0, 1, "\u{fffd}"),
        character(&source, 1, 3, "é"),
        eof(&source),
    ];
    let diagnostics = vec![diagnostic(
        &source,
        0,
        1,
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    )];

    let result = HtmlTokenizerRunResult::new(
        &source,
        tokens,
        preprocessing(&source),
        diagnostics,
        coverage(&source, 3),
        HtmlTokenizerCompletion::Complete,
        limits(),
        usage(&source, 3, 3, 1, 0, 5, 0),
    )
    .unwrap();

    assert!(result.is_complete_with_diagnostics());
    assert_eq!(
        result.diagnostics()[0].code(),
        HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter
    );
    assert_eq!(
        result.diagnostics()[0].context(),
        HtmlTokenizerDiagnosticContext::Data
    );
    assert!(matches!(
        result.diagnostics()[0].handling(),
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::ReplacedNullWithReplacementCharacter
        )
    ));
}

#[test]
fn diagnostics_allow_equal_offsets_but_reject_backward_order() {
    let source = src(3, "ab");
    let same_first = diagnostic(&source, 0, 1, HtmlTokenizerDiagnosticSubject::InputLocation);
    let same_second = HtmlTokenizerDiagnostic::new(
        &source,
        HtmlTokenizerDiagnosticCode::ControlCharacterInInputStream,
        anchor(&source, 0, 1),
        HtmlTokenizerDiagnosticContext::InputPreprocessing,
        HtmlTokenizerDiagnosticHandling::Continued,
        HtmlTokenizerDiagnosticSubject::InputLocation,
    )
    .unwrap();

    HtmlTokenizerRunResult::new(
        &source,
        vec![character(&source, 0, 2, "ab"), eof(&source)],
        preprocessing(&source),
        vec![same_first, same_second],
        coverage(&source, 2),
        HtmlTokenizerCompletion::Complete,
        limits(),
        usage(&source, 2, 2, 2, 0, 2, 0),
    )
    .unwrap();

    let later = diagnostic(&source, 1, 2, HtmlTokenizerDiagnosticSubject::InputLocation);
    let earlier = diagnostic(&source, 0, 1, HtmlTokenizerDiagnosticSubject::InputLocation);
    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            vec![character(&source, 0, 2, "ab"), eof(&source)],
            preprocessing(&source),
            vec![later, earlier],
            coverage(&source, 2),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 2, 2, 2, 0, 2, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::DiagnosticOrder
    );
}

#[test]
fn abandoned_regions_and_token_relations_are_validated() {
    let source = src(4, "<a");
    let abandoned = HtmlTokenizerDiagnostic::new(
        &source,
        HtmlTokenizerDiagnosticCode::EofInTag,
        anchor(&source, 2, 2),
        HtmlTokenizerDiagnosticContext::TagName,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::AbandonedIncompleteTagAtEof,
        ),
        HtmlTokenizerDiagnosticSubject::AbandonedInput {
            region: anchor(&source, 0, 2),
        },
    )
    .unwrap();

    let result = HtmlTokenizerRunResult::new(
        &source,
        vec![eof(&source)],
        preprocessing(&source),
        vec![abandoned],
        coverage(&source, 2),
        HtmlTokenizerCompletion::Complete,
        limits(),
        usage(&source, 2, 1, 1, 0, 0, 0),
    )
    .unwrap();
    assert!(result.is_complete_with_diagnostics());

    let invalid = diagnostic(
        &source,
        0,
        1,
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );
    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            vec![eof(&source)],
            preprocessing(&source),
            vec![invalid],
            coverage(&source, 2),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 2, 1, 1, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::DiagnosticMustNotReferenceEof {
            diagnostic_index: 0,
            token_index: 0,
        }
    );
}

#[test]
fn unsupported_input_before_output_preserves_unprocessed_suffix() {
    let source = src(5, "&x");
    let unsupported = HtmlTokenizerUnsupportedCapability::new(
        &source,
        HtmlTokenizerCapability::CharacterReference {
            context: HtmlCharacterReferenceContext::Data,
        },
        HtmlTokenizerCapabilityAvailability::Deferred,
        HtmlTokenizerUnsupportedTrigger::Input(anchor(&source, 0, 1)),
    )
    .unwrap();

    let result = HtmlTokenizerRunResult::new(
        &source,
        Vec::new(),
        preprocessing(&source),
        Vec::new(),
        coverage(&source, 0),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        )),
        limits(),
        usage(&source, 1, 0, 0, 0, 0, 0),
    )
    .unwrap();

    assert!(result.is_incomplete());
    assert_eq!(result.coverage().unprocessed_suffix().fragment(), "&x");
}

#[test]
fn context_mode_can_stop_after_a_complete_emitted_tag() {
    let source = src(6, "<script>x");
    let emitted = tag(&source, 7, 8, "script");
    let unsupported = HtmlTokenizerUnsupportedCapability::new(
        &source,
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::ScriptData,
        },
        HtmlTokenizerCapabilityAvailability::Deferred,
        HtmlTokenizerUnsupportedTrigger::EmittedToken {
            token_index: 0,
            boundary: anchor(&source, 8, 8),
        },
    )
    .unwrap();

    let result = HtmlTokenizerRunResult::new(
        &source,
        vec![emitted],
        preprocessing(&source),
        Vec::new(),
        coverage(&source, 8),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        )),
        limits(),
        usage(&source, 8, 1, 0, 0, 6, 0),
    )
    .unwrap();

    assert!(result.is_incomplete());
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(
            HtmlTokenizerIncompleteCause::UnsupportedCapability(value),
        ) => {
            assert_eq!(
                value.capability(),
                HtmlTokenizerCapability::ContextDependentTokenizerMode {
                    mode: HtmlTokenizerMode::ScriptData,
                }
            );
            assert_eq!(
                value.availability(),
                HtmlTokenizerCapabilityAvailability::Deferred
            );
        }
        other => panic!("unexpected completion: {other:?}"),
    }
}

#[test]
fn every_resource_dimension_has_explicit_limit_evidence() {
    let source = src(7, "x");
    for resource in HtmlTokenizerResource::ALL {
        let (run_limits, run_usage, tokens, processed_end, limit, attempted) = match resource {
            HtmlTokenizerResource::SourceBytes => (
                HtmlTokenizerLimits::new(0, 10, 10, 10, 10, 10, 10),
                usage(&source, 0, 0, 0, 0, 0, 0),
                Vec::new(),
                0,
                0,
                1,
            ),
            HtmlTokenizerResource::TransitionSteps => (
                HtmlTokenizerLimits::new(10, 1, 10, 10, 10, 10, 10),
                usage(&source, 1, 0, 0, 0, 0, 0),
                Vec::new(),
                0,
                1,
                2,
            ),
            HtmlTokenizerResource::EmittedTokens => (
                HtmlTokenizerLimits::new(10, 10, 1, 10, 10, 10, 10),
                usage(&source, 1, 1, 0, 0, 1, 0),
                vec![character(&source, 0, 1, "x")],
                1,
                1,
                2,
            ),
            HtmlTokenizerResource::Diagnostics => (
                HtmlTokenizerLimits::new(10, 10, 10, 1, 10, 10, 10),
                usage(&source, 1, 0, 1, 0, 0, 0),
                Vec::new(),
                1,
                1,
                2,
            ),
            HtmlTokenizerResource::AttributesPerTag => (
                HtmlTokenizerLimits::new(10, 10, 10, 10, 1, 10, 10),
                usage(&source, 1, 0, 0, 1, 0, 0),
                Vec::new(),
                1,
                1,
                2,
            ),
            HtmlTokenizerResource::RetainedInterpretedBytes => (
                HtmlTokenizerLimits::new(10, 10, 10, 10, 10, 1, 10),
                usage(&source, 1, 1, 0, 0, 1, 0),
                vec![character(&source, 0, 1, "x")],
                1,
                1,
                2,
            ),
            HtmlTokenizerResource::TemporaryBufferBytes => (
                HtmlTokenizerLimits::new(10, 10, 10, 10, 10, 10, 1),
                usage(&source, 1, 0, 0, 0, 0, 1),
                Vec::new(),
                1,
                1,
                2,
            ),
        };

        let diagnostics = if resource == HtmlTokenizerResource::Diagnostics {
            vec![diagnostic(
                &source,
                0,
                1,
                HtmlTokenizerDiagnosticSubject::InputLocation,
            )]
        } else {
            Vec::new()
        };
        let at_offset = if resource == HtmlTokenizerResource::SourceBytes {
            0
        } else {
            processed_end
        };
        let limit_evidence = HtmlTokenizerResourceLimit::new(
            &source,
            resource,
            limit,
            attempted,
            anchor(&source, at_offset, at_offset),
        )
        .unwrap();

        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            diagnostics,
            coverage(&source, processed_end),
            HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
                limit_evidence,
            )),
            run_limits,
            run_usage,
        )
        .unwrap();
    }
}

#[test]
fn invalid_configuration_is_distinct_and_precedes_processing() {
    let source = src(8, "x");
    let transition_limits = HtmlTokenizerLimits::new(10, 0, 1, 1, 1, 1, 1);
    let result = HtmlTokenizerRunResult::new(
        &source,
        Vec::new(),
        preprocessing(&source),
        Vec::new(),
        coverage(&source, 0),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::InvalidConfiguration(
            HtmlTokenizerConfigurationFailure::ZeroTransitionStepLimit,
        )),
        transition_limits,
        usage(&source, 0, 0, 0, 0, 0, 0),
    )
    .unwrap();
    assert!(result.is_incomplete());

    let token_limits = HtmlTokenizerLimits::new(10, 1, 0, 1, 1, 1, 1);
    HtmlTokenizerRunResult::new(
        &source,
        Vec::new(),
        preprocessing(&source),
        Vec::new(),
        coverage(&source, 0),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::InvalidConfiguration(
            HtmlTokenizerConfigurationFailure::ZeroEmittedTokenLimit,
        )),
        token_limits,
        usage(&source, 0, 0, 0, 0, 0, 0),
    )
    .unwrap();
}

#[test]
fn source_mismatch_and_invalid_regions_return_typed_errors_without_panics() {
    let source = src(9, "ab");
    let foreign = src(10, "ab");
    let result = catch_unwind(AssertUnwindSafe(|| {
        HtmlTokenizerDiagnostic::new(
            &source,
            HtmlTokenizerDiagnosticCode::EofInTag,
            anchor(&foreign, 0, 1),
            HtmlTokenizerDiagnosticContext::TagName,
            HtmlTokenizerDiagnosticHandling::Stopped,
            HtmlTokenizerDiagnosticSubject::InputLocation,
        )
    }));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().unwrap_err(),
        HtmlTokenizerDiagnosticContractError::SourceIdentityMismatch {
            role: HtmlDiagnosticEvidenceRole::Location,
            expected: source.id(),
            actual: foreign.id(),
        }
    );

    assert_eq!(
        HtmlTokenizerDiagnostic::new(
            &source,
            HtmlTokenizerDiagnosticCode::EofInTag,
            anchor(&source, 0, 1),
            HtmlTokenizerDiagnosticContext::TagName,
            HtmlTokenizerDiagnosticHandling::Stopped,
            HtmlTokenizerDiagnosticSubject::AbandonedInput {
                region: anchor(&source, 1, 2),
            },
        )
        .unwrap_err(),
        HtmlTokenizerDiagnosticContractError::LocationOutsideAbandonedRegion
    );
}

#[test]
fn coverage_and_completion_contradictions_are_rejected() {
    let source = src(11, "x");
    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            Vec::new(),
            preprocessing(&source),
            Vec::new(),
            coverage(&source, 0),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 1, 0, 0, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::CompleteResultMustContainExactlyOneEof
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            vec![eof(&source)],
            preprocessing(&source),
            Vec::new(),
            coverage(&source, 1),
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::InternalInvariantFailure(
                    HtmlTokenizerInvariantFailure::RunAssembly,
                ),
            ),
            limits(),
            usage(&source, 1, 1, 0, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::IncompleteResultMustNotContainEof
    );
}

#[test]
fn aggregate_debug_output_redacts_source_content() {
    const SECRET: &str = "private-run-result-marker";
    let source = src(12, SECRET);
    let unsupported = HtmlTokenizerUnsupportedCapability::new(
        &source,
        HtmlTokenizerCapability::MarkupDeclaration,
        HtmlTokenizerCapabilityAvailability::Unsupported,
        HtmlTokenizerUnsupportedTrigger::Input(anchor(&source, 0, 1)),
    )
    .unwrap();
    let result = HtmlTokenizerRunResult::new(
        &source,
        Vec::new(),
        preprocessing(&source),
        Vec::new(),
        coverage(&source, 0),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(
            unsupported,
        )),
        limits(),
        usage(&source, 1, 0, 0, 0, 0, 0),
    )
    .unwrap();

    let debug = format!("{result:?}");
    assert!(!debug.contains(SECRET));
    assert!(debug.contains("token_count"));
    assert!(debug.contains("UnsupportedCapability"));
}

#[test]
fn first_capability_vocabularies_are_finite_and_explicit() {
    let diagnostic_codes = [
        HtmlTokenizerDiagnosticCode::NoncharacterInInputStream,
        HtmlTokenizerDiagnosticCode::ControlCharacterInInputStream,
        HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter,
        HtmlTokenizerDiagnosticCode::EofBeforeTagName,
        HtmlTokenizerDiagnosticCode::InvalidFirstCharacterOfTagName,
        HtmlTokenizerDiagnosticCode::UnexpectedQuestionMarkInsteadOfTagName,
        HtmlTokenizerDiagnosticCode::MissingEndTagName,
        HtmlTokenizerDiagnosticCode::EofInTag,
        HtmlTokenizerDiagnosticCode::UnexpectedEqualsSignBeforeAttributeName,
        HtmlTokenizerDiagnosticCode::UnexpectedCharacterInAttributeName,
        HtmlTokenizerDiagnosticCode::MissingAttributeValue,
        HtmlTokenizerDiagnosticCode::UnexpectedCharacterInUnquotedAttributeValue,
        HtmlTokenizerDiagnosticCode::MissingWhitespaceBetweenAttributes,
        HtmlTokenizerDiagnosticCode::UnexpectedSolidusInTag,
        HtmlTokenizerDiagnosticCode::DuplicateAttribute,
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
    ];
    assert_eq!(diagnostic_codes.len(), 17);

    let contexts = [
        HtmlTokenizerDiagnosticContext::InputPreprocessing,
        HtmlTokenizerDiagnosticContext::Data,
        HtmlTokenizerDiagnosticContext::TagOpen,
        HtmlTokenizerDiagnosticContext::EndTagOpen,
        HtmlTokenizerDiagnosticContext::TagName,
        HtmlTokenizerDiagnosticContext::BeforeAttributeName,
        HtmlTokenizerDiagnosticContext::AttributeName,
        HtmlTokenizerDiagnosticContext::AfterAttributeName,
        HtmlTokenizerDiagnosticContext::BeforeAttributeValue,
        HtmlTokenizerDiagnosticContext::AttributeValueDoubleQuoted,
        HtmlTokenizerDiagnosticContext::AttributeValueSingleQuoted,
        HtmlTokenizerDiagnosticContext::AttributeValueUnquoted,
        HtmlTokenizerDiagnosticContext::AfterAttributeValueQuoted,
        HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
    ];
    assert_eq!(contexts.len(), 14);

    let recovery = [
        HtmlTokenizerRecoveryKind::ReplacedNullWithReplacementCharacter,
        HtmlTokenizerRecoveryKind::EmittedLiteralMarkupPrefix,
        HtmlTokenizerRecoveryKind::IgnoredUnexpectedInput,
        HtmlTokenizerRecoveryKind::StartedAttributeAtUnexpectedEqualsSign,
        HtmlTokenizerRecoveryKind::ReconsumedInData,
        HtmlTokenizerRecoveryKind::ReconsumedBeforeAttributeName,
        HtmlTokenizerRecoveryKind::CompletedTagWithMissingAttributeValue,
        HtmlTokenizerRecoveryKind::PreservedDuplicateAttributeOccurrence,
        HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        HtmlTokenizerRecoveryKind::AbandonedIncompleteTagAtEof,
    ];
    assert_eq!(recovery.len(), 10);

    let capabilities = [
        HtmlTokenizerCapability::CharacterReference {
            context: HtmlCharacterReferenceContext::Data,
        },
        HtmlTokenizerCapability::CharacterReference {
            context: HtmlCharacterReferenceContext::AttributeValue,
        },
        HtmlTokenizerCapability::MarkupDeclaration,
        HtmlTokenizerCapability::ProcessingInstruction,
        HtmlTokenizerCapability::BogusCommentRecovery,
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::Rcdata,
        },
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::RawText,
        },
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::ScriptData,
        },
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::Plaintext,
        },
        HtmlTokenizerCapability::ContextDependentTokenizerMode {
            mode: HtmlTokenizerMode::Noscript,
        },
        HtmlTokenizerCapability::TreeConstructionControlledState,
        HtmlTokenizerCapability::ForeignContentControl,
    ];
    assert_eq!(capabilities.len(), 12);

    let invariant_failures = [
        HtmlTokenizerInvariantFailure::CursorState,
        HtmlTokenizerInvariantFailure::ReconsumeState,
        HtmlTokenizerInvariantFailure::BuilderState,
        HtmlTokenizerInvariantFailure::EmissionOrder,
        HtmlTokenizerInvariantFailure::SourceEvidenceConstruction,
        HtmlTokenizerInvariantFailure::CounterOverflow,
        HtmlTokenizerInvariantFailure::RunAssembly,
    ];
    assert_eq!(invariant_failures.len(), 7);
}

#[test]
fn observation_conditioned_diagnostic_survives_top_level_non_emission() {
    // Approved `MissingAttributeValue` observation, matching the `ERR-011`
    // shape (`<a x=>`: `>` observed in `BeforeAttributeValue` after `=`),
    // prefixed with an already-emitted character token so this fixture
    // exercises non-emission of the *tag*, not of the diagnostic-adjacent
    // evidence itself.
    let source = src(13, "z<a b=>");
    let tokens = vec![character(&source, 0, 1, "z")];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::MissingAttributeValue,
        6,
        7,
        HtmlTokenizerDiagnosticContext::BeforeAttributeValue,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::CompletedTagWithMissingAttributeValue,
        ),
        HtmlTokenizerDiagnosticSubject::AbandonedInput {
            region: anchor(&source, 1, 7),
        },
    );
    let limit_evidence = HtmlTokenizerResourceLimit::new(
        &source,
        HtmlTokenizerResource::EmittedTokens,
        1,
        2,
        anchor(&source, 7, 7),
    )
    .unwrap();

    let result = HtmlTokenizerRunResult::new(
        &source,
        tokens,
        preprocessing(&source),
        vec![diagnostic],
        coverage(&source, 7),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            limit_evidence,
        )),
        HtmlTokenizerLimits::new(1_000, 1_000, 1, 1_000, 1_000, 1_000, 1_000),
        // 9 attempted transitions: Data('z'), Data('<'), TagOpen('a', reconsume),
        // TagName(reconsumed 'a'), TagName(space), BeforeAttributeName('b',
        // create, reconsume), AttributeName(reconsumed 'b'), AttributeName('='),
        // BeforeAttributeValue('>') — the dispatch where MissingAttributeValue is
        // observed, CompletedTagWithMissingAttributeValue commits, and the
        // separate top-level tag emission is refused by the EmittedTokens
        // resource limit. The refusal commits this ninth transition without a
        // trailing EOF dispatch, since the run stops incomplete. Attribute `b`
        // was recognized and finalized in the active tag builder before that
        // emission refusal, so peak_attributes_per_tag is 1.
        usage(&source, 9, 1, 1, 1, 3, 0),
    )
    .unwrap();

    assert!(result.is_incomplete());
    assert_eq!(result.usage().transition_steps(), 9);
    assert_eq!(result.usage().peak_attributes_per_tag(), 1);
    assert_eq!(
        result.diagnostics()[0].handling(),
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::CompletedTagWithMissingAttributeValue
        )
    );
    assert!(matches!(
        result.diagnostics()[0].subject(),
        HtmlTokenizerDiagnosticSubject::AbandonedInput { .. }
    ));
}

#[test]
fn end_tag_with_attributes_accepts_emitted_end_tag_with_authored_attributes() {
    let source = src(14, "</a b>");
    let tokens = vec![
        end_tag(
            &source,
            3,
            6,
            "a",
            vec![boolean_attribute(&source, 4, 5, "b")],
            None,
        ),
        eof(&source),
    ];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        4,
        5,
        HtmlTokenizerDiagnosticContext::AttributeName,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );

    let result = HtmlTokenizerRunResult::new(
        &source,
        tokens,
        preprocessing(&source),
        vec![diagnostic],
        coverage(&source, 6),
        HtmlTokenizerCompletion::Complete,
        limits(),
        usage(&source, 6, 2, 1, 1, 2, 0),
    )
    .unwrap();

    assert!(result.is_complete_with_diagnostics());
}

#[test]
fn end_tag_with_attributes_rejects_emitted_end_tag_without_attributes() {
    let source = src(15, "</a>");
    let tokens = vec![end_tag(&source, 3, 4, "a", Vec::new(), None), eof(&source)];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        2,
        3,
        HtmlTokenizerDiagnosticContext::AttributeName,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 4),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 4, 2, 1, 0, 1, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedEvidenceMissing {
            diagnostic_index: 0,
            token_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        }
    );
}

#[test]
fn end_tag_with_attributes_rejects_relation_to_start_tag_token() {
    let source = src(16, "<a>");
    let tokens = vec![tag(&source, 2, 3, "a"), eof(&source)];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        1,
        2,
        HtmlTokenizerDiagnosticContext::AttributeName,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 3),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 3, 2, 1, 0, 1, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedTokenMustBeEndTag {
            diagnostic_index: 0,
            token_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        }
    );
}

#[test]
fn end_tag_with_attributes_rejects_abandoned_input_subject() {
    let source = src(17, "</a>");
    let tokens = vec![eof(&source)];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        2,
        3,
        HtmlTokenizerDiagnosticContext::AttributeName,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::AbandonedInput {
            region: anchor(&source, 0, 4),
        },
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 4),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 4, 1, 1, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedSubjectMustBeEmittedToken {
            diagnostic_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        }
    );
}

#[test]
fn end_tag_with_attributes_rejects_input_location_subject() {
    let source = src(18, "</a>");
    let tokens = vec![eof(&source)];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        2,
        3,
        HtmlTokenizerDiagnosticContext::AttributeName,
        HtmlTokenizerDiagnosticHandling::Continued,
        HtmlTokenizerDiagnosticSubject::InputLocation,
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 4),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 4, 1, 1, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedSubjectMustBeEmittedToken {
            diagnostic_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        }
    );
}

#[test]
fn end_tag_with_trailing_solidus_accepts_emitted_end_tag_with_solidus() {
    let source = src(19, "</a/>");
    let tokens = vec![
        end_tag(&source, 3, 5, "a", Vec::new(), Some(anchor(&source, 3, 4))),
        eof(&source),
    ];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        3,
        4,
        HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );

    let result = HtmlTokenizerRunResult::new(
        &source,
        tokens,
        preprocessing(&source),
        vec![diagnostic],
        coverage(&source, 5),
        HtmlTokenizerCompletion::Complete,
        limits(),
        usage(&source, 5, 2, 1, 0, 1, 0),
    )
    .unwrap();

    assert!(result.is_complete_with_diagnostics());
}

#[test]
fn end_tag_with_trailing_solidus_rejects_emitted_end_tag_without_solidus() {
    let source = src(20, "</a>");
    let tokens = vec![end_tag(&source, 3, 4, "a", Vec::new(), None), eof(&source)];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        2,
        3,
        HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 4),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 4, 2, 1, 0, 1, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedEvidenceMissing {
            diagnostic_index: 0,
            token_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        }
    );
}

#[test]
fn end_tag_with_trailing_solidus_rejects_relation_to_start_tag_token() {
    let source = src(21, "<a>");
    let tokens = vec![tag(&source, 2, 3, "a"), eof(&source)];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        1,
        2,
        HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 3),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 3, 2, 1, 0, 1, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedTokenMustBeEndTag {
            diagnostic_index: 0,
            token_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        }
    );
}

#[test]
fn end_tag_with_trailing_solidus_rejects_abandoned_input_subject() {
    let source = src(22, "</a>");
    let tokens = vec![eof(&source)];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        2,
        3,
        HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::AbandonedInput {
            region: anchor(&source, 0, 4),
        },
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 4),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 4, 1, 1, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedSubjectMustBeEmittedToken {
            diagnostic_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        }
    );
}

#[test]
fn end_tag_with_trailing_solidus_rejects_input_location_subject() {
    let source = src(23, "</a>");
    let tokens = vec![eof(&source)];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        2,
        3,
        HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
        HtmlTokenizerDiagnosticHandling::Continued,
        HtmlTokenizerDiagnosticSubject::InputLocation,
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 4),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 4, 1, 1, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedSubjectMustBeEmittedToken {
            diagnostic_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        }
    );
}

#[test]
fn end_tag_with_attributes_rejects_wrong_qualifying_tag_correlation() {
    let source = src(24, "</a x></b y>");
    let tokens = vec![
        end_tag_at(
            &source,
            0,
            3,
            6,
            "a",
            vec![boolean_attribute(&source, 4, 5, "x")],
            None,
        ),
        end_tag_at(
            &source,
            6,
            9,
            12,
            "b",
            vec![boolean_attribute(&source, 10, 11, "y")],
            None,
        ),
        eof(&source),
    ];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        10,
        11,
        HtmlTokenizerDiagnosticContext::AttributeName,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 12),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 12, 3, 1, 1, 4, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedDiagnosticOutsideReferencedTag {
            diagnostic_index: 0,
            token_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithAttributes,
        }
    );
}

#[test]
fn end_tag_with_trailing_solidus_rejects_wrong_qualifying_tag_correlation() {
    let source = src(25, "</a/></b/>");
    let tokens = vec![
        end_tag_at(
            &source,
            0,
            3,
            5,
            "a",
            Vec::new(),
            Some(anchor(&source, 3, 4)),
        ),
        end_tag_at(
            &source,
            5,
            8,
            10,
            "b",
            Vec::new(),
            Some(anchor(&source, 8, 9)),
        ),
        eof(&source),
    ];
    let diagnostic = diagnostic_full(
        &source,
        HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        8,
        9,
        HtmlTokenizerDiagnosticContext::SelfClosingStartTag,
        HtmlTokenizerDiagnosticHandling::Recovered(
            HtmlTokenizerRecoveryKind::PreservedEndTagLexicalEvidence,
        ),
        HtmlTokenizerDiagnosticSubject::EmittedToken { token_index: 0 },
    );

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            vec![diagnostic],
            coverage(&source, 10),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 10, 3, 1, 0, 2, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::EmissionConditionedDiagnosticOutsideReferencedTag {
            diagnostic_index: 0,
            token_index: 0,
            code: HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus,
        }
    );
}

#[test]
fn diagnostics_resource_limit_accepts_atomic_multi_diagnostic_attempt() {
    // One indivisible logical operation may require more than one
    // diagnostic to commit atomically. The rejected operation therefore
    // reports `attempted = committed + N` for `N >= 1`, not only
    // `committed + 1`: here `committed = 0` and `attempted = 2` for a
    // `max_diagnostics` limit of `1`. No rejected diagnostic appears in the
    // final diagnostics vector because the operation is atomic.
    let source = src(26, "x");
    let run_limits = HtmlTokenizerLimits::new(10, 10, 10, 1, 10, 10, 10);
    let run_usage = usage(&source, 1, 0, 0, 0, 0, 0);
    let limit_evidence = HtmlTokenizerResourceLimit::new(
        &source,
        HtmlTokenizerResource::Diagnostics,
        1,
        2,
        anchor(&source, 1, 1),
    )
    .unwrap();

    let result = HtmlTokenizerRunResult::new(
        &source,
        Vec::new(),
        preprocessing(&source),
        Vec::new(),
        coverage(&source, 1),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            limit_evidence,
        )),
        run_limits,
        run_usage,
    )
    .unwrap();

    assert!(result.is_incomplete());
    assert!(result.diagnostics().is_empty());
    assert_eq!(result.usage().diagnostics(), 0);
}

// `HtmlTokenizerResourceLimit::new` already enforces `attempted > limit`,
// and the terminal resource's committed usage is now independently checked
// against `limit` below. Together those two constructor/aggregate
// invariants imply `attempted > committed` for every valid
// `HtmlTokenizerResourceLimit`: `attempted > limit >= committed`. An
// isolated `attempted <= committed` case therefore cannot be constructed
// without first violating `committed <= limit`, so it is no longer
// independently testable through valid evidence; the committed-over-limit
// regressions below cover the only reachable malformed shape.

#[test]
fn diagnostics_resource_limit_rejects_committed_over_limit() {
    // `committed = 2` already exceeds `limit = 1` before the rejected
    // operation is even considered: the run was over budget before the
    // supposedly rejected attempt. This must be rejected for exceeding the
    // configured limit, independent of the `attempted` value.
    let source = src(27, "xy");
    let diagnostics = vec![
        diagnostic(&source, 0, 1, HtmlTokenizerDiagnosticSubject::InputLocation),
        diagnostic(&source, 1, 2, HtmlTokenizerDiagnosticSubject::InputLocation),
    ];
    let run_limits = HtmlTokenizerLimits::new(10, 10, 10, 1, 10, 10, 10);
    let run_usage = usage(&source, 2, 0, 2, 0, 0, 0);
    let limit_evidence = HtmlTokenizerResourceLimit::new(
        &source,
        HtmlTokenizerResource::Diagnostics,
        1,
        3,
        anchor(&source, 2, 2),
    )
    .unwrap();

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            Vec::new(),
            preprocessing(&source),
            diagnostics,
            coverage(&source, 2),
            HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
                limit_evidence,
            )),
            run_limits,
            run_usage,
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::UsageExceedsLimit {
            resource: HtmlTokenizerResource::Diagnostics,
            usage: 2,
            limit: 1,
        }
    );
}

#[test]
fn emitted_tokens_resource_limit_rejects_committed_over_limit() {
    // The committed-over-limit guard is shared by every non-`SourceBytes`
    // resource, not special-cased to `Diagnostics`. `EmittedTokens` remains
    // an exact-single-increment resource, so this proves the shared guard
    // fires before that stricter check is even reached.
    let source = src(29, "ab");
    let tokens = vec![character(&source, 0, 1, "a"), character(&source, 1, 2, "b")];
    let run_limits = HtmlTokenizerLimits::new(10, 10, 1, 10, 10, 10, 10);
    let run_usage = usage(&source, 2, 2, 0, 0, 2, 0);
    let limit_evidence = HtmlTokenizerResourceLimit::new(
        &source,
        HtmlTokenizerResource::EmittedTokens,
        1,
        3,
        anchor(&source, 2, 2),
    )
    .unwrap();

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            tokens,
            preprocessing(&source),
            Vec::new(),
            coverage(&source, 2),
            HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
                limit_evidence,
            )),
            run_limits,
            run_usage,
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::UsageExceedsLimit {
            resource: HtmlTokenizerResource::EmittedTokens,
            usage: 2,
            limit: 1,
        }
    );
}

#[test]
fn diagnostics_resource_limit_constructor_rejects_attempted_not_exceeding_limit() {
    let source = src(28, "x");
    assert_eq!(
        HtmlTokenizerResourceLimit::new(
            &source,
            HtmlTokenizerResource::Diagnostics,
            1,
            1,
            anchor(&source, 1, 1),
        )
        .unwrap_err(),
        HtmlTokenizerResourceContractError::AttemptedMustExceedLimit {
            limit: 1,
            attempted: 1,
        }
    );
}
