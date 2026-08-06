use super::diagnostic::*;
use super::resource::*;
use super::result::*;
use crate::html::token::*;
use crate::{SourceAnchor, SourceId, SourceText};

fn source(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn anchor(source: &SourceText, start: usize, end: usize) -> SourceAnchor {
    source.anchor(start, end).unwrap()
}

fn coverage(source: &SourceText, processed_end: usize) -> HtmlTokenizerCoverage {
    HtmlTokenizerCoverage::new(
        source,
        anchor(source, 0, processed_end),
        anchor(source, processed_end, source.as_str().len()),
    )
    .unwrap()
}

fn limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(100, 100, 100, 100, 100, 100, 100)
}

fn usage(
    source: &SourceText,
    transition_steps: usize,
    emitted_tokens: usize,
    diagnostics: usize,
) -> HtmlTokenizerUsage {
    HtmlTokenizerUsage::new(
        source.as_str().len(),
        transition_steps,
        emitted_tokens,
        diagnostics,
        0,
        0,
        0,
    )
}

fn diagnostic(
    source: &SourceText,
    start: usize,
    end: usize,
    handling: HtmlTokenizerDiagnosticHandling,
) -> HtmlTokenizerDiagnostic {
    HtmlTokenizerDiagnostic::new(
        source,
        HtmlTokenizerDiagnosticCode::EofBeforeTagName,
        anchor(source, start, end),
        HtmlTokenizerDiagnosticContext::TagOpen,
        handling,
        HtmlTokenizerDiagnosticSubject::InputLocation,
    )
    .unwrap()
}

#[test]
fn invalid_limits_require_the_matching_configuration_completion() {
    let source = source(101, "&");
    let invalid_limits = HtmlTokenizerLimits::new(100, 0, 100, 100, 100, 100, 100);
    let unsupported = HtmlTokenizerUnsupportedCapability::new(
        &source,
        HtmlTokenizerCapability::CharacterReference {
            context: HtmlCharacterReferenceContext::Data,
        },
        HtmlTokenizerCapabilityAvailability::Deferred,
        HtmlTokenizerUnsupportedTrigger::Input(anchor(&source, 0, 1)),
    )
    .unwrap();

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            Vec::new(),
            HtmlPreprocessingEvidence::new(&source, None).unwrap(),
            Vec::new(),
            coverage(&source, 0),
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported),
            ),
            invalid_limits,
            usage(&source, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::ConfigurationFailureMismatch
    );
}

#[test]
fn invalid_configuration_precedes_the_source_byte_limit() {
    let source = source(102, "x");
    let invalid_limits = HtmlTokenizerLimits::new(0, 0, 100, 100, 100, 100, 100);

    let result = HtmlTokenizerRunResult::new(
        &source,
        Vec::new(),
        HtmlPreprocessingEvidence::new(&source, None).unwrap(),
        Vec::new(),
        coverage(&source, 0),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::InvalidConfiguration(
            HtmlTokenizerConfigurationFailure::ZeroTransitionStepLimit,
        )),
        invalid_limits,
        usage(&source, 0, 0, 0),
    )
    .unwrap();

    assert!(result.is_incomplete());
}

#[test]
fn stopped_diagnostic_cannot_be_reported_as_complete() {
    let source = source(103, "");
    let eof =
        HtmlToken::EndOfFile(HtmlEndOfFileToken::new(&source, anchor(&source, 0, 0)).unwrap());

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            vec![eof],
            HtmlPreprocessingEvidence::new(&source, None).unwrap(),
            vec![diagnostic(
                &source,
                0,
                0,
                HtmlTokenizerDiagnosticHandling::Stopped,
            )],
            coverage(&source, 0),
            HtmlTokenizerCompletion::Complete,
            limits(),
            usage(&source, 1, 1, 1),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::StoppedDiagnosticRequiresIncompleteResult {
            diagnostic_index: 0,
        }
    );
}

#[test]
fn stopped_diagnostic_must_be_the_last_observed_diagnostic() {
    let source = source(104, "ab");
    let diagnostics = vec![
        diagnostic(&source, 0, 1, HtmlTokenizerDiagnosticHandling::Stopped),
        diagnostic(&source, 1, 2, HtmlTokenizerDiagnosticHandling::Continued),
    ];

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            Vec::new(),
            HtmlPreprocessingEvidence::new(&source, None).unwrap(),
            diagnostics,
            coverage(&source, 2),
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::InternalInvariantFailure(
                    HtmlTokenizerInvariantFailure::RunAssembly,
                ),
            ),
            limits(),
            usage(&source, 2, 0, 2),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::StoppedDiagnosticMustBeLast {
            diagnostic_index: 0,
        }
    );
}

#[test]
fn skipped_bom_must_be_inside_the_processed_prefix() {
    let source = source(105, "\u{feff}x");
    let preprocessing =
        HtmlPreprocessingEvidence::new(&source, Some(anchor(&source, 0, 3))).unwrap();

    assert_eq!(
        HtmlTokenizerRunResult::new(
            &source,
            Vec::new(),
            preprocessing,
            Vec::new(),
            coverage(&source, 0),
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::InternalInvariantFailure(
                    HtmlTokenizerInvariantFailure::RunAssembly,
                ),
            ),
            limits(),
            usage(&source, 0, 0, 0),
        )
        .unwrap_err(),
        HtmlTokenizerRunContractError::PreprocessingOutsideProcessedPrefix
    );
}
