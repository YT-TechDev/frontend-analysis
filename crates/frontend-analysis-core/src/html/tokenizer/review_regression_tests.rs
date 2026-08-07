use super::diagnostic::*;
use super::producer::tokenize;
use super::resource::*;
use super::result::*;
use crate::html::token::*;
use crate::{SourceAnchor, SourceId, SourceText};

fn source(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

/// Otherwise-generous limits with `max_retained_interpreted_bytes` lowered
/// to `max_retained`, for exercising the `RetainedInterpretedBytes` boundary
/// on literal markup-prefix recovery paths.
fn retained_limits(max_retained: usize) -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(100, 100, 100, 100, 100, max_retained, 100)
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

fn expect_retained_bytes_resource_limit(
    result: &HtmlTokenizerRunResult,
    expected_limit: usize,
    expected_attempted: usize,
) {
    assert!(result.tokens().is_empty(), "no literal output was retained");
    assert_eq!(result.usage().retained_interpreted_bytes(), 0);
    // The retained-byte budget is preflighted before the diagnostic that
    // would claim `Recovered(EmittedLiteralMarkupPrefix)`: on refusal, no
    // diagnostic is committed for this event at all, so the diagnostic
    // vector never falsely claims a recovery that did not happen.
    assert!(
        result.diagnostics().is_empty(),
        "no diagnostic claims a recovery that the resource limit prevented"
    );
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::RetainedInterpretedBytes
            );
            assert_eq!(resource_limit.limit(), expected_limit);
            assert_eq!(resource_limit.attempted(), expected_attempted);
        }
        _ => panic!("expected a RetainedInterpretedBytes resource-limit completion"),
    }
}

/// `TagOpen(EOF)` recovery must check the retained-byte budget before
/// retaining the literal `"<"` markup prefix.
#[test]
fn tag_open_eof_literal_prefix_respects_zero_retained_byte_limit() {
    let source = source(201, "<");
    let result = tokenize(&source, retained_limits(0));
    expect_retained_bytes_resource_limit(&result, 0, 1);
}

/// `EndTagOpen(EOF)` recovery must check the retained-byte budget before
/// retaining the literal `"</"` markup prefix.
#[test]
fn end_tag_open_eof_literal_prefix_respects_insufficient_retained_byte_limit() {
    let source = source(202, "</");
    let result = tokenize(&source, retained_limits(1));
    expect_retained_bytes_resource_limit(&result, 1, 2);
}

/// Invalid-first-tag-name-character recovery (`InvalidFirstCharacterOfTagName`)
/// must check the retained-byte budget before placing the literal `"<"` into
/// the recovered Data character run.
#[test]
fn invalid_first_tag_name_character_literal_prefix_respects_zero_retained_byte_limit() {
    let source = source(203, "<1");
    let result = tokenize(&source, retained_limits(0));
    expect_retained_bytes_resource_limit(&result, 0, 1);
}

fn only_tag(result: &HtmlTokenizerRunResult) -> &HtmlTagToken {
    match result
        .tokens()
        .iter()
        .find(|token| matches!(token, HtmlToken::Tag(_)))
        .expect("a tag token was emitted")
    {
        HtmlToken::Tag(tag) => tag,
        _ => unreachable!(),
    }
}

/// `AfterAttributeName('=')` must capture the exact `=` span itself:
/// whitespace between an attribute name and `=` must not panic by reaching
/// `equals_span()` with no span recorded.
#[test]
fn whitespace_before_equals_sign_captures_its_own_equals_span() {
    let source = source(301, "<a x =y>");
    let result = tokenize(&source, limits());
    assert!(result.is_clean_complete());
    let tag = only_tag(&result);
    assert_eq!(tag.attributes().len(), 1);
    match tag.attributes()[0].value_syntax() {
        HtmlAttributeValueSyntax::Unquoted { equals, value } => {
            assert_eq!(equals.range().start(), 5);
            assert_eq!(equals.range().end(), 6);
            assert_eq!(value.range().start(), 6);
            assert_eq!(value.range().end(), 7);
        }
        other => panic!("expected an unquoted value syntax, found {other:?}"),
    }
}

/// A second attribute's `=` span must never be the stale span belonging to
/// an earlier attribute: the first attribute consumes `=` directly (via
/// `AttributeName('=')`), the second reaches `=` through
/// `AfterAttributeName('=')` after intervening whitespace, and each must
/// resolve to its own exact position.
#[test]
fn a_later_attributes_equals_span_is_never_a_stale_earlier_one() {
    let source = source(302, "<a x=1 y =2>");
    let result = tokenize(&source, limits());
    assert!(result.is_clean_complete());
    let tag = only_tag(&result);
    assert_eq!(tag.attributes().len(), 2);

    match tag.attributes()[0].value_syntax() {
        HtmlAttributeValueSyntax::Unquoted { equals, .. } => {
            assert_eq!((equals.range().start(), equals.range().end()), (4, 5));
        }
        other => panic!("expected an unquoted value syntax, found {other:?}"),
    }
    match tag.attributes()[1].value_syntax() {
        HtmlAttributeValueSyntax::Unquoted { equals, .. } => {
            assert_eq!((equals.range().start(), equals.range().end()), (9, 10));
        }
        other => panic!("expected an unquoted value syntax, found {other:?}"),
    }
}

fn assert_abandoned(diagnostic: &HtmlTokenizerDiagnostic) {
    assert!(
        matches!(
            diagnostic.subject(),
            HtmlTokenizerDiagnosticSubject::AbandonedInput { .. }
        ),
        "a diagnostic from a tag that will never emit must not keep a \
         forward EmittedToken reference"
    );
}

/// A diagnostic committed while a tag is under construction, followed by an
/// unsupported-capability discovery before that tag ever emits, must not
/// leave the diagnostic's subject dangling: `HtmlTokenizerRunResult::new`
/// would otherwise reject the forward `EmittedToken` reference and the
/// internal `.expect` in `into_result` would panic.
#[test]
fn active_tag_diagnostic_survives_an_unsupported_stop_before_emission() {
    let source = source(303, "<a x x=&y");
    let result = tokenize(&source, limits());
    assert!(result.tokens().is_empty());
    assert!(matches!(
        result.completion(),
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::UnsupportedCapability(_))
    ));
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        HtmlTokenizerDiagnosticCode::DuplicateAttribute
    );
    assert_abandoned(&result.diagnostics()[0]);
}

/// The same class of dangling-subject defect, but for a resource-limit stop
/// (not an unsupported-capability release): a still-active tag is frozen,
/// not released, yet its provisional diagnostics must still be retargeted.
#[test]
fn active_tag_diagnostic_survives_a_resource_limit_stop_before_emission() {
    let source = source(304, "<a x x='y");
    let limits = HtmlTokenizerLimits::new(100, 100, 100, 1, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(result.tokens().is_empty());
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::Diagnostics
            );
        }
        other => panic!("expected a Diagnostics resource-limit completion, found {other:?}"),
    }
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        HtmlTokenizerDiagnosticCode::DuplicateAttribute
    );
    assert_abandoned(&result.diagnostics()[0]);
    // The frozen tag builder's retained bytes ("a", "x", "x", "y") remain
    // visible: a resource-limit stop must not silently discard evidence
    // that the diagnostics-limit refusal itself left untouched.
    assert_eq!(result.usage().retained_interpreted_bytes(), 4);
}

/// A prior token already emitted, plus an active Data run whose own
/// emission is then refused, must leave the Data run's content uncounted
/// as a token but still visible in retained-byte usage: `flush_data_run`
/// must not destroy the run before its token-capacity preflight succeeds.
#[test]
fn data_run_emission_refusal_does_not_destroy_the_pending_run() {
    let source = source(305, "<a>b");
    let limits = HtmlTokenizerLimits::new(100, 100, 1, 100, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert_eq!(result.tokens().len(), 1);
    assert!(matches!(result.tokens()[0], HtmlToken::Tag(_)));
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::EmittedTokens
            );
            assert_eq!(resource_limit.limit(), 1);
            assert_eq!(resource_limit.attempted(), 2);
        }
        other => panic!("expected an EmittedTokens resource-limit completion, found {other:?}"),
    }
    // "a" (committed, in the tag) + "b" (still pending, never destroyed).
    assert_eq!(result.usage().retained_interpreted_bytes(), 2);
}

/// A prior token already emitted, plus a complete active tag whose own
/// emission is then refused, must leave the tag builder untouched (not
/// finalized and discarded): `finish_tag` must not destroy the tag before
/// its token-capacity preflight succeeds.
#[test]
fn tag_emission_refusal_does_not_destroy_the_active_tag() {
    let source = source(306, "a<b>");
    let limits = HtmlTokenizerLimits::new(100, 100, 1, 100, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert_eq!(result.tokens().len(), 1);
    assert!(matches!(result.tokens()[0], HtmlToken::Character(_)));
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::EmittedTokens
            );
            assert_eq!(resource_limit.limit(), 1);
            assert_eq!(resource_limit.attempted(), 2);
        }
        other => panic!("expected an EmittedTokens resource-limit completion, found {other:?}"),
    }
    // "a" (committed, as the Character token) + "b" (the frozen tag name,
    // never destroyed by the refused `finish_tag`).
    assert_eq!(result.usage().retained_interpreted_bytes(), 2);
}

/// The required `EofInTag` diagnostic must be checked before the active tag
/// builder is taken: a diagnostics-limit refusal must leave the builder
/// (and its already-retained bytes) exactly as it was, not discarded.
#[test]
fn eof_in_tag_diagnostic_refusal_does_not_discard_the_active_tag() {
    let source = source(307, "<a");
    let limits = HtmlTokenizerLimits::new(100, 100, 100, 0, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(result.tokens().is_empty());
    assert!(result.diagnostics().is_empty());
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::Diagnostics
            );
            assert_eq!(resource_limit.limit(), 0);
            assert_eq!(resource_limit.attempted(), 1);
        }
        other => panic!("expected a Diagnostics resource-limit completion, found {other:?}"),
    }
    assert_eq!(result.usage().retained_interpreted_bytes(), 1);
}

/// The required `MissingAttributeValue` diagnostic must be checked before
/// the pending attribute name is taken: a diagnostics-limit refusal must
/// leave it untouched, not destructively consumed first.
#[test]
fn missing_attribute_value_diagnostic_refusal_does_not_consume_the_pending_name() {
    let source = source(308, "<a x=>");
    let limits = HtmlTokenizerLimits::new(100, 100, 100, 0, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(result.tokens().is_empty());
    assert!(result.diagnostics().is_empty());
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::Diagnostics
            );
        }
        other => panic!("expected a Diagnostics resource-limit completion, found {other:?}"),
    }
    // "a" (tag name) + "x" (still the pending attribute name).
    assert_eq!(result.usage().retained_interpreted_bytes(), 2);
}

/// The required `DuplicateAttribute` diagnostic must be checked before the
/// pending duplicate attribute name is taken: a diagnostics-limit refusal
/// must leave it untouched.
#[test]
fn duplicate_attribute_diagnostic_refusal_does_not_consume_the_pending_name() {
    let source = source(309, "<a x x");
    let limits = HtmlTokenizerLimits::new(100, 100, 100, 0, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(result.tokens().is_empty());
    assert!(result.diagnostics().is_empty());
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::Diagnostics
            );
        }
        other => panic!("expected a Diagnostics resource-limit completion, found {other:?}"),
    }
    // "a" (tag name) + "x" (first, already committed to attributes) + "x"
    // (second, still the pending, never-taken attribute name).
    assert_eq!(result.usage().retained_interpreted_bytes(), 3);
}

/// `DuplicateAttribute` must be observed (and committed to the diagnostics
/// vector) when the tokenizer leaves the attribute-name state, not at value
/// finalization: otherwise a later value diagnostic (positioned after the
/// name) would be appended first, and `DuplicateAttribute` would then
/// append at the earlier name location, violating the nondecreasing
/// diagnostic-order contract enforced by `HtmlTokenizerRunResult::new`.
#[test]
fn duplicate_attribute_precedes_a_later_diagnostic_in_its_own_value() {
    let source = source(310, "<a x x=\"\0\">");
    let result = tokenize(&source, limits());
    assert!(result.is_complete_with_diagnostics());
    assert_eq!(result.diagnostics().len(), 2);
    assert_eq!(
        result.diagnostics()[0].code(),
        HtmlTokenizerDiagnosticCode::DuplicateAttribute
    );
    assert_eq!(result.diagnostics()[0].location().range().start(), 5);
    assert_eq!(
        result.diagnostics()[1].code(),
        HtmlTokenizerDiagnosticCode::UnexpectedNullCharacter
    );
    assert_eq!(result.diagnostics()[1].location().range().start(), 8);

    let tag = only_tag(&result);
    assert_eq!(tag.attributes().len(), 2);
    assert_eq!(
        tag.attributes()[0].disposition(),
        HtmlAttributeDisposition::Effective
    );
    assert_eq!(
        tag.attributes()[1].disposition(),
        HtmlAttributeDisposition::DuplicateOf { first_index: 0 }
    );
}

/// `EndTagWithAttributes` must be reported once per end tag, not once per
/// authored attribute, matching the pinned WHATWG "end tag token emitted
/// with attributes" single parse error.
#[test]
fn end_tag_with_multiple_attributes_reports_one_diagnostic() {
    let source = source(311, "</a x y>");
    let result = tokenize(&source, limits());
    assert!(result.is_complete_with_diagnostics());
    let tag = only_tag(&result);
    assert_eq!(tag.attributes().len(), 2);
    let end_tag_diagnostics: Vec<_> = result
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == HtmlTokenizerDiagnosticCode::EndTagWithAttributes)
        .collect();
    assert_eq!(end_tag_diagnostics.len(), 1);
    assert_eq!(end_tag_diagnostics[0].location().range().start(), 4);
    assert_eq!(end_tag_diagnostics[0].location().range().end(), 5);
}

/// `EndTagWithAttributes` must also be reported for a single-attribute
/// emitted end tag, not only the multi-attribute case above.
#[test]
fn end_tag_with_single_attribute_reports_the_diagnostic() {
    let source = source(312, "</a x>");
    let result = tokenize(&source, limits());
    assert!(result.is_complete_with_diagnostics());
    let end_tag_diagnostics: Vec<_> = result
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code() == HtmlTokenizerDiagnosticCode::EndTagWithAttributes)
        .collect();
    assert_eq!(end_tag_diagnostics.len(), 1);
}

/// `EndTagWithAttributes` is emission-conditioned (#111): its evidence is
/// held as pending evidence on the tag builder and must not appear at all
/// when the corresponding end-tag token emission is refused, even though
/// the abandoned builder held authored attribute evidence. Reproduces
/// `REG-113-end-tag-attributes-emission-refusal` as a focused regression
/// independent of the candidate-independent gold corpus.
#[test]
fn end_tag_with_attributes_does_not_appear_when_emission_is_refused() {
    let source = source(313, "z</a x>");
    let limits = HtmlTokenizerLimits::new(100, 100, 1, 100, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(
        !result
            .tokens()
            .iter()
            .any(|token| matches!(token, HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End))
    );
    assert!(result.diagnostics().is_empty());
}

/// `EndTagWithTrailingSolidus` must be reported for an emitted end tag
/// carrying authored trailing-solidus evidence.
#[test]
fn end_tag_with_trailing_solidus_reports_the_diagnostic() {
    let source = source(314, "</a/>");
    let result = tokenize(&source, limits());
    assert!(result.is_complete_with_diagnostics());
    let solidus_diagnostics: Vec<_> = result
        .diagnostics()
        .iter()
        .filter(|diagnostic| {
            diagnostic.code() == HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus
        })
        .collect();
    assert_eq!(solidus_diagnostics.len(), 1);
}

/// `EndTagWithTrailingSolidus` is emission-conditioned (#111) and must not
/// appear when the corresponding end-tag token emission is refused.
/// Reproduces `REG-113-end-tag-trailing-solidus-emission-refusal` as a
/// focused regression independent of the candidate-independent gold corpus.
#[test]
fn end_tag_with_trailing_solidus_does_not_appear_when_emission_is_refused() {
    let source = source(315, "z</a/>");
    let limits = HtmlTokenizerLimits::new(100, 100, 1, 100, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(
        !result
            .tokens()
            .iter()
            .any(|token| matches!(token, HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End))
    );
    assert!(result.diagnostics().is_empty());
}

/// `MissingAttributeValue` is observation-conditioned (#111): unlike the two
/// emission-conditioned codes above, it must remain in the final result
/// with an `AbandonedInput` subject even when the top-level token emission
/// is separately refused, because `Recovered(CompletedTagWithMissingAttributeValue)`
/// denotes builder/token-construction completion, not emitted-vector
/// insertion. Reproduces `REG-113-missing-attribute-value-emission-refusal`
/// as a focused regression independent of the candidate-independent gold
/// corpus.
#[test]
fn missing_attribute_value_survives_emission_refusal_with_abandoned_input() {
    let source = source(316, "z<a b=>");
    let limits = HtmlTokenizerLimits::new(100, 100, 1, 100, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(
        !result
            .tokens()
            .iter()
            .any(|token| matches!(token, HtmlToken::Tag(_)))
    );
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        HtmlTokenizerDiagnosticCode::MissingAttributeValue
    );
    assert_abandoned(&result.diagnostics()[0]);
}

/// A multi-diagnostic end tag must preserve the #111 diagnostic-order
/// invariant even though `EndTagWithAttributes`'s pending emission-
/// conditioned evidence is materialized only once the tag successfully
/// emits — chronologically after a later, already-committed
/// `UnexpectedSolidusInTag` diagnostic recognized in between. The
/// emission-conditioned diagnostic must still be inserted back at its
/// earlier source location rather than appended after the later one.
#[test]
fn end_tag_with_attributes_preserves_diagnostic_order_against_a_later_diagnostic() {
    let source = source(317, "</a x/y>");
    let result = tokenize(&source, limits());
    assert!(result.is_complete_with_diagnostics());
    assert_eq!(result.diagnostics().len(), 2);
    assert_eq!(
        result.diagnostics()[0].code(),
        HtmlTokenizerDiagnosticCode::EndTagWithAttributes
    );
    assert_eq!(result.diagnostics()[0].location().range().start(), 4);
    assert_eq!(
        result.diagnostics()[1].code(),
        HtmlTokenizerDiagnosticCode::UnexpectedSolidusInTag
    );
    assert_eq!(result.diagnostics()[1].location().range().start(), 5);
    let tag = only_tag(&result);
    assert_eq!(tag.attributes().len(), 2);
}

/// A tag's successful emission and its pending emission-conditioned
/// diagnostic evidence are one compound operation: a `Diagnostics`-limit
/// refusal on the pending evidence must leave the tag builder frozen (not
/// finalized and discarded) and commit no diagnostic at all, matching the
/// same all-or-nothing shape as every other refusal in `finish_tag`.
#[test]
fn pending_emission_diagnostic_refusal_does_not_emit_the_tag_or_the_diagnostic() {
    let source = source(318, "</a x>");
    let limits = HtmlTokenizerLimits::new(100, 100, 100, 0, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(result.tokens().is_empty());
    assert!(result.diagnostics().is_empty());
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::Diagnostics
            );
            assert_eq!(resource_limit.limit(), 0);
            assert_eq!(resource_limit.attempted(), 1);
        }
        other => panic!("expected a Diagnostics resource-limit completion, found {other:?}"),
    }
    // "a" (tag name) + "x" (attribute name): the frozen builder's retained
    // bytes, never destroyed by the refused compound operation.
    assert_eq!(result.usage().retained_interpreted_bytes(), 2);
}

/// A tag whose successful emission would require materializing more than one
/// pending emission-conditioned diagnostic is still one atomic compound
/// operation: a `Diagnostics`-limit refusal must report the whole rejected
/// operation's would-be usage (`committed + pending_count`), not a fixed
/// single-unit increment, matching the merged #111 atomic multi-diagnostic
/// resource-accounting contract.
#[test]
fn pending_multi_diagnostic_refusal_reports_the_atomic_operations_full_attempted_usage() {
    let source = source(319, "</a x/>");
    let limits = HtmlTokenizerLimits::new(100, 100, 100, 1, 100, 100, 100);
    let result = tokenize(&source, limits);
    assert!(result.tokens().is_empty());
    assert!(result.diagnostics().is_empty());
    assert!(!result.diagnostics().iter().any(|d| d.code()
        == HtmlTokenizerDiagnosticCode::EndTagWithAttributes
        || d.code() == HtmlTokenizerDiagnosticCode::EndTagWithTrailingSolidus));
    assert!(
        !result
            .tokens()
            .iter()
            .any(|token| matches!(token, HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::End))
    );
    match result.completion() {
        HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(
            resource_limit,
        )) => {
            assert_eq!(
                resource_limit.resource(),
                HtmlTokenizerResource::Diagnostics
            );
            assert_eq!(resource_limit.limit(), 1);
            assert_eq!(resource_limit.attempted(), 2);
            assert_eq!(resource_limit.at().range().start(), 6);
            assert_eq!(resource_limit.at().range().end(), 6);
        }
        other => panic!("expected a Diagnostics resource-limit completion, found {other:?}"),
    }
    assert_eq!(result.usage().transition_steps(), 10);
    assert_eq!(result.usage().emitted_tokens(), 0);
    assert_eq!(result.usage().diagnostics(), 0);
    assert_eq!(result.usage().peak_attributes_per_tag(), 1);
    assert_eq!(result.usage().retained_interpreted_bytes(), 2);
    assert_eq!(result.usage().peak_temporary_buffer_bytes(), 0);
}
