use std::collections::BTreeMap;

use crate::html::token::{HtmlEndOfFileToken, HtmlPreprocessingEvidence, HtmlToken};
use crate::{SourceId, SourceText};

use super::super::resource::{HtmlTokenizerLimits, HtmlTokenizerUsage};
use super::super::result::{
    HtmlTokenizerCompletion, HtmlTokenizerCoverage, HtmlTokenizerRunResult,
};
use super::compare::compare;
use super::corpus::{
    all_candidate_independent_corpus, initial_corpus, supplemental_regression_corpus,
};
use super::expected::{
    ByteSpan, Completion, DiagnosticCode, DiagnosticHandling, DiagnosticSubject, ObservedRun,
    Resource, Token, UnsupportedTrigger,
};
use super::fixture::{FixtureCategory, validate_corpus};
use super::generated::{
    MAX_GENERATED_CASES, MAX_SOURCE_BYTES, generated_inputs, minimization_candidates,
};
use super::observe::observe;
use super::policy::validate_policy;

#[test]
fn initial_inventory_contains_exactly_72_unique_fixtures() {
    let fixtures = initial_corpus();
    assert_eq!(fixtures.len(), 72);
    validate_corpus(&fixtures).unwrap();
    validate_policy(&fixtures).unwrap();

    let counts = fixtures
        .iter()
        .fold(BTreeMap::new(), |mut counts, fixture| {
            *counts.entry(fixture.category).or_insert(0usize) += 1;
            counts
        });
    assert_eq!(counts.get(&FixtureCategory::Preprocessing), Some(&10));
    assert_eq!(counts.get(&FixtureCategory::SupportedToken), Some(&12));
    assert_eq!(counts.get(&FixtureCategory::Diagnostic), Some(&17));
    assert_eq!(counts.get(&FixtureCategory::Unsupported), Some(&14));
    assert_eq!(counts.get(&FixtureCategory::Resource), Some(&9));
    assert_eq!(counts.get(&FixtureCategory::Adversarial), Some(&10));
}

#[test]
fn initial_ids_are_stable_and_contiguous_within_each_category() {
    let fixtures = initial_corpus();
    for (prefix, count) in [
        ("PRE", 10usize),
        ("TOK", 12),
        ("ERR", 17),
        ("UNSUP", 14),
        ("RES", 9),
        ("ADV", 10),
    ] {
        let actual: Vec<&str> = fixtures
            .iter()
            .filter(|fixture| fixture.id.starts_with(prefix))
            .map(|fixture| fixture.id)
            .collect();
        let expected: Vec<String> = (1..=count)
            .map(|index| format!("{prefix}-{index:03}"))
            .collect();
        assert_eq!(
            actual,
            expected.iter().map(String::as_str).collect::<Vec<_>>()
        );
    }
}

#[test]
fn empty_run_observation_is_stable_across_three_runs_and_source_ids() {
    let fixture = initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == "PRE-001")
        .unwrap();

    let first = observed_empty_run(10);
    let second = observed_empty_run(10);
    let third = observed_empty_run(10);
    let alternate_source_id = observed_empty_run(11);
    for observed in [&first, &second, &third, &alternate_source_id] {
        compare(fixture.id, &fixture.expected, observed).unwrap();
    }
    assert_eq!(first, second);
    assert_eq!(second, third);
    assert_eq!(
        first, alternate_source_id,
        "SourceId must not change semantic observation"
    );
}

#[test]
fn preprocessing_fixture_distinguishes_ff_control_null_and_noncharacter() {
    let fixture = initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == "PRE-010")
        .unwrap();
    assert_eq!(
        fixture.source_bytes,
        "\u{000c}\u{0001}\0\u{fdd0}".as_bytes()
    );
    let codes: Vec<DiagnosticCode> = fixture
        .expected
        .0
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect();
    assert_eq!(
        codes,
        vec![
            DiagnosticCode::ControlCharacterInInputStream,
            DiagnosticCode::UnexpectedNullCharacter,
            DiagnosticCode::NoncharacterInInputStream,
        ]
    );
    assert_eq!(
        fixture.expected.0.diagnostics[0].location,
        ByteSpan::new(1, 2)
    );
    assert_eq!(
        fixture.expected.0.diagnostics[1].location,
        ByteSpan::new(2, 3)
    );
    assert_eq!(
        fixture.expected.0.diagnostics[2].location,
        ByteSpan::new(3, 6)
    );
}

#[test]
fn equal_offset_diagnostics_preserve_transition_order() {
    let fixture = initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == "ADV-008")
        .unwrap();
    assert_eq!(fixture.expected.0.diagnostics.len(), 2);
    assert_eq!(
        fixture.expected.0.diagnostics[0].code,
        DiagnosticCode::UnexpectedEqualsSignBeforeAttributeName
    );
    assert_eq!(
        fixture.expected.0.diagnostics[1].code,
        DiagnosticCode::UnexpectedCharacterInAttributeName
    );
    assert_eq!(
        fixture.expected.0.diagnostics[0].location,
        fixture.expected.0.diagnostics[1].location
    );
}

#[test]
fn structural_comparison_reports_the_first_owned_path() {
    let fixture = initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == "PRE-001")
        .unwrap();
    let mut observed = observed_empty_run(12);
    observed.0.tokens[0] = Token::EndOfFile {
        at: ByteSpan::new(1, 1),
    };

    let mismatch = compare(fixture.id, &fixture.expected, &observed).unwrap_err();
    assert_eq!(mismatch.path(), "expected.tokens[0].at");
}

#[test]
fn bounded_generator_is_reproducible_valid_and_ordered() {
    let first = generated_inputs();
    let second = generated_inputs();
    assert_eq!(first, second);
    assert_eq!(first.len(), MAX_GENERATED_CASES);
    assert!(first.iter().all(|value| value.len() <= MAX_SOURCE_BYTES));
    assert!(
        first
            .iter()
            .all(|value| std::str::from_utf8(value.as_bytes()).is_ok())
    );
    assert!(first.iter().any(|value| value.contains('\0')));
    assert!(first.iter().any(|value| value.contains('界')));
}

#[test]
fn minimization_order_is_stable_shortest_first_and_utf8_safe() {
    let first = minimization_candidates("é<a界>");
    let second = minimization_candidates("é<a界>");
    assert_eq!(first, second);
    assert_eq!(first.first().map(String::as_str), Some(""));
    assert!(first.windows(2).all(|pair| {
        pair[0].len() < pair[1].len()
            || (pair[0].len() == pair[1].len() && pair[0].as_bytes() <= pair[1].as_bytes())
    }));
    assert!(
        first
            .iter()
            .all(|value| value.is_char_boundary(value.len()))
    );
}

#[test]
fn fixture_debug_redacts_authored_source_content() {
    let fixture = initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == "ADV-010")
        .unwrap();
    let debug = format!("{fixture:?}");
    assert!(!debug.contains("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
    assert!(debug.contains("source_byte_len"));
}

#[test]
fn malformed_gold_fixture_is_rejected_before_candidate_execution() {
    let mut fixture = initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == "PRE-001")
        .unwrap();
    fixture.expected.0.coverage.unprocessed_suffix = ByteSpan::new(0, 1);
    let error = fixture.validate().unwrap_err().to_string();
    assert!(error.contains("PRE-001.coverage"));
    assert!(!error.contains("source_bytes"));
}

#[test]
fn malformed_unsupported_trigger_is_rejected_by_policy_validation() {
    let mut fixture = initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == "UNSUP-001")
        .unwrap();
    let Completion::Unsupported { trigger, .. } = &mut fixture.expected.0.completion else {
        panic!("UNSUP-001 must remain unsupported");
    };
    *trigger = UnsupportedTrigger::Input(ByteSpan::new(1, 2));
    let error = validate_policy(std::slice::from_ref(&fixture))
        .unwrap_err()
        .to_string();
    assert!(error.contains("UNSUP-001.completion.unsupported.input_boundary"));
}

#[test]
fn understated_usage_is_rejected_by_policy_validation() {
    let mut fixture = initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == "TOK-008")
        .unwrap();
    fixture.expected.0.usage.peak_attributes_per_tag = 0;
    let error = validate_policy(std::slice::from_ref(&fixture))
        .unwrap_err()
        .to_string();
    assert!(error.contains("TOK-008.usage.peak_attributes_per_tag"));
}

fn find(id: &str) -> super::fixture::HtmlTokenizerFixture {
    initial_corpus()
        .into_iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing fixture {id}"))
}

#[test]
fn transition_steps_are_not_derivable_from_utf8_byte_length() {
    let fixture = find("PRE-003");
    assert_eq!(fixture.source_bytes.len(), 5);
    assert_eq!(fixture.expected.0.tokens.len(), 2);
    assert_eq!(fixture.expected.0.usage.transition_steps, 3);
    assert_ne!(
        fixture.expected.0.usage.transition_steps,
        fixture.source_bytes.len()
    );
}

#[test]
fn transition_steps_are_not_derivable_from_crlf_byte_count() {
    let lf = find("PRE-006");
    let cr = find("PRE-007");
    let crlf = find("PRE-008");
    assert_eq!(lf.source_bytes.len(), 1);
    assert_eq!(cr.source_bytes.len(), 1);
    assert_eq!(crlf.source_bytes.len(), 2);
    assert_eq!(lf.expected.0.usage.transition_steps, 2);
    assert_eq!(cr.expected.0.usage.transition_steps, 2);
    assert_eq!(crlf.expected.0.usage.transition_steps, 2);
}

#[test]
fn transition_steps_include_one_reconsume_instruction_for_invalid_tag_open() {
    // TagOpen examines '1', issues one reconsume instruction, and Data then
    // examines the same authored input unit a second time. EOF is step four.
    let fixture = find("ERR-005");
    assert_eq!(fixture.source_bytes.len(), 2);
    assert_eq!(fixture.expected.0.usage.transition_steps, 4);
}

#[test]
fn tag_open_eof_emission_remains_one_state_dispatch() {
    // TagOpen(EOF) emits both the literal '<' and EOF in one state dispatch.
    let fixture = find("ERR-004");
    assert_eq!(fixture.expected.0.usage.transition_steps, 2);
    assert_eq!(fixture.expected.0.tokens.len(), 2);
}

#[test]
fn transition_steps_are_not_derivable_from_diagnostic_count_alone() {
    let noncharacter = find("ERR-001");
    let control = find("ERR-002");
    assert_eq!(noncharacter.expected.0.diagnostics.len(), 1);
    assert_eq!(control.expected.0.diagnostics.len(), 1);
    assert_ne!(noncharacter.source_bytes.len(), control.source_bytes.len());
    assert_eq!(noncharacter.expected.0.usage.transition_steps, 2);
    assert_eq!(control.expected.0.usage.transition_steps, 2);
}

#[test]
fn transition_step_commits_even_when_token_emission_is_refused() {
    let fixture = find("RES-003");
    assert_eq!(fixture.expected.0.usage.emitted_tokens, 1);
    assert_eq!(fixture.expected.0.usage.transition_steps, 2);
}

#[test]
fn transition_step_commits_before_resource_refusal_without_partial_mutation() {
    let attrs = find("RES-005");
    assert_eq!(attrs.expected.0.usage.transition_steps, 9);
    assert_eq!(attrs.expected.0.usage.peak_attributes_per_tag, 1);
    assert_eq!(attrs.expected.0.limits.attributes_per_tag, 1);

    let retained = find("RES-007");
    assert_eq!(retained.expected.0.usage.transition_steps, 3);
    assert_eq!(retained.expected.0.usage.retained_interpreted_bytes, 0);
    assert_eq!(retained.expected.0.limits.retained_interpreted_bytes, 0);
    assert_eq!(retained.expected.0.usage.peak_temporary_buffer_bytes, 0);
    assert_eq!(retained.expected.0.limits.temporary_buffer_bytes, 1_024);
    assert!(matches!(
        &retained.expected.0.completion,
        Completion::ResourceLimit {
            resource: Resource::RetainedInterpretedBytes,
            attempted: 1,
            ..
        }
    ));
}

#[test]
fn no_initial_fixture_claims_temporary_buffer_execution_coverage() {
    assert!(!initial_corpus().iter().any(|fixture| matches!(
        &fixture.expected.0.completion,
        Completion::ResourceLimit {
            resource: Resource::TemporaryBufferBytes,
            ..
        }
    )));
}

#[test]
fn transition_steps_are_not_derivable_from_the_retired_sum_heuristic() {
    for (id, old_heuristic) in [
        ("PRE-003", 7usize),
        ("PRE-008", 4),
        ("TOK-009", 9),
        ("ERR-013", 13),
        ("RES-005", 6),
        ("RES-007", 1),
    ] {
        let fixture = find(id);
        assert_ne!(fixture.expected.0.usage.transition_steps, old_heuristic);
    }
}

#[test]
fn err_006_and_unsup_004_share_the_same_tag_open_question_mark_prefix() {
    // ERR-006 ("<?") and UNSUP-004 ("<?x>") both dispatch TagOpen on '?'.
    // The pinned WHATWG Tag open state emits
    // UnexpectedQuestionMarkInsteadOfTagName unconditionally on '?' before
    // deferring to the unsupported processing-instruction boundary, so both
    // fixtures must record an identical diagnostic and completion for that
    // shared dispatch, independent of what follows in the source.
    let err_006 = find("ERR-006");
    let unsup_004 = find("UNSUP-004");

    for fixture in [&err_006, &unsup_004] {
        assert_eq!(fixture.expected.0.usage.transition_steps, 2);
        assert_eq!(fixture.expected.0.diagnostics.len(), 1);

        let diagnostic = &fixture.expected.0.diagnostics[0];
        assert_eq!(
            diagnostic.code,
            DiagnosticCode::UnexpectedQuestionMarkInsteadOfTagName
        );
        assert_eq!(diagnostic.location, ByteSpan::new(1, 2));
        assert_eq!(
            diagnostic.context,
            super::expected::DiagnosticContext::TagOpen
        );
        assert_eq!(
            diagnostic.handling,
            super::expected::DiagnosticHandling::Stopped
        );
        assert_eq!(
            diagnostic.subject,
            super::expected::DiagnosticSubject::InputLocation
        );

        assert_eq!(fixture.expected.0.coverage.processed_prefix.end, 2);

        let Completion::Unsupported {
            capability,
            availability,
            trigger,
        } = &fixture.expected.0.completion
        else {
            panic!("{}: completion must remain Unsupported", fixture.id);
        };
        assert_eq!(
            *capability,
            super::expected::Capability::ProcessingInstruction
        );
        assert_eq!(*availability, super::expected::Availability::Deferred);
        assert_eq!(*trigger, UnsupportedTrigger::Input(ByteSpan::new(2, 2)));
    }

    assert_eq!(
        unsup_004.expected.0.coverage.unprocessed_suffix,
        ByteSpan::new(2, 4)
    );
}

fn find_regression(id: &str) -> super::fixture::HtmlTokenizerFixture {
    supplemental_regression_corpus()
        .into_iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing regression fixture {id}"))
}

#[test]
fn supplemental_regression_corpus_contains_exactly_three_stable_reg_113_fixtures() {
    let fixtures = supplemental_regression_corpus();
    assert_eq!(fixtures.len(), 3);
    validate_corpus(&fixtures).unwrap();
    validate_policy(&fixtures).unwrap();

    let mut ids: Vec<&str> = fixtures.iter().map(|fixture| fixture.id).collect();
    let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len(), "supplemental IDs must be unique");
    for id in &ids {
        assert!(id.starts_with("REG-113-"), "{id} must start with REG-113-");
        assert!(
            fixtures
                .iter()
                .filter(|f| f.category == FixtureCategory::Regression)
                .count()
                == 3,
            "all supplemental fixtures must use FixtureCategory::Regression"
        );
    }
    ids.sort_unstable();
    assert_eq!(
        ids,
        vec![
            "REG-113-end-tag-attributes-emission-refusal",
            "REG-113-end-tag-trailing-solidus-emission-refusal",
            "REG-113-missing-attribute-value-emission-refusal",
        ]
    );

    // Deterministic and self-validating: rebuilding the corpus twice yields
    // an identical, independently re-validating result.
    let rebuilt = supplemental_regression_corpus();
    assert_eq!(rebuilt.len(), fixtures.len());
    validate_corpus(&rebuilt).unwrap();
    validate_policy(&rebuilt).unwrap();
}

#[test]
fn initial_corpus_is_unaffected_by_the_supplemental_regression_layer() {
    // The historical 72-fixture initial inventory must remain unchanged in
    // both count and category composition after adding the REG- layer.
    assert_eq!(initial_corpus().len(), 72);
}

#[test]
fn aggregate_candidate_independent_corpus_is_75_with_unique_ids() {
    let aggregate = all_candidate_independent_corpus();
    assert_eq!(aggregate.len(), 75, "72 initial + 3 supplemental");

    let ids: Vec<&str> = aggregate.iter().map(|fixture| fixture.id).collect();
    let unique: std::collections::BTreeSet<&str> = ids.iter().copied().collect();
    assert_eq!(
        unique.len(),
        ids.len(),
        "aggregate corpus must contain no duplicate IDs across both layers"
    );

    // Deterministic concatenation: initial corpus first, then supplemental.
    assert_eq!(aggregate[71].id, "ADV-010");
    assert_eq!(
        aggregate[72].id,
        "REG-113-end-tag-attributes-emission-refusal"
    );

    validate_corpus(&aggregate).unwrap();
    validate_policy(&aggregate).unwrap();
}

#[test]
fn end_tag_emission_refusal_regressions_contain_zero_diagnostics_and_no_end_tag_or_eof() {
    for id in [
        "REG-113-end-tag-attributes-emission-refusal",
        "REG-113-end-tag-trailing-solidus-emission-refusal",
    ] {
        let fixture = find_regression(id);
        let run = &fixture.expected.0;
        assert!(
            run.diagnostics.is_empty(),
            "{id} must record zero diagnostics"
        );
        assert_eq!(
            run.tokens.len(),
            1,
            "{id} must contain exactly one prior token"
        );
        assert!(
            matches!(&run.tokens[0], Token::Character { interpreted, .. } if interpreted == "z")
        );
        assert!(
            !run.tokens
                .iter()
                .any(|token| matches!(token, Token::EndOfFile { .. })),
            "{id} must contain no EOF token"
        );
        assert!(
            !run.tokens
                .iter()
                .any(|token| matches!(token, Token::Tag { .. })),
            "{id} must contain no end-tag token"
        );
        assert!(matches!(
            &run.completion,
            Completion::ResourceLimit {
                resource: Resource::EmittedTokens,
                limit: 1,
                attempted: 2,
                ..
            }
        ));
    }
}

#[test]
fn missing_attribute_value_regression_keeps_one_observation_conditioned_diagnostic() {
    let fixture = find_regression("REG-113-missing-attribute-value-emission-refusal");
    let run = &fixture.expected.0;

    assert_eq!(run.tokens.len(), 1, "must contain exactly one prior token");
    assert!(matches!(&run.tokens[0], Token::Character { interpreted, .. } if interpreted == "z"));
    assert!(
        !run.tokens
            .iter()
            .any(|token| matches!(token, Token::EndOfFile { .. }))
    );
    assert!(
        !run.tokens
            .iter()
            .any(|token| matches!(token, Token::Tag { .. }))
    );

    assert_eq!(run.diagnostics.len(), 1);
    let diagnostic = &run.diagnostics[0];
    assert_eq!(diagnostic.code, DiagnosticCode::MissingAttributeValue);
    assert!(matches!(
        diagnostic.handling,
        DiagnosticHandling::Recovered(
            super::expected::RecoveryKind::CompletedTagWithMissingAttributeValue
        )
    ));
    assert!(matches!(
        diagnostic.subject,
        DiagnosticSubject::AbandonedInput(_)
    ));

    assert!(matches!(
        &run.completion,
        Completion::ResourceLimit {
            resource: Resource::EmittedTokens,
            limit: 1,
            attempted: 2,
            ..
        }
    ));
}

#[test]
fn regression_transition_and_usage_values_are_independently_authored() {
    let end_tag_attributes = find_regression("REG-113-end-tag-attributes-emission-refusal");
    assert_eq!(end_tag_attributes.source_bytes, b"z</a x>");
    assert_eq!(end_tag_attributes.expected.0.usage.transition_steps, 10);
    assert_eq!(end_tag_attributes.expected.0.usage.emitted_tokens, 1);
    assert_eq!(end_tag_attributes.expected.0.usage.diagnostics, 0);
    assert_eq!(
        end_tag_attributes.expected.0.usage.peak_attributes_per_tag,
        1
    );
    assert_eq!(
        end_tag_attributes
            .expected
            .0
            .usage
            .retained_interpreted_bytes,
        3
    );
    assert_eq!(
        end_tag_attributes
            .expected
            .0
            .usage
            .peak_temporary_buffer_bytes,
        0
    );
    assert_eq!(
        end_tag_attributes.expected.0.coverage.processed_prefix.end,
        6
    );

    let end_tag_solidus = find_regression("REG-113-end-tag-trailing-solidus-emission-refusal");
    assert_eq!(end_tag_solidus.source_bytes, b"z</a/>");
    assert_eq!(end_tag_solidus.expected.0.usage.transition_steps, 7);
    assert_eq!(end_tag_solidus.expected.0.usage.emitted_tokens, 1);
    assert_eq!(end_tag_solidus.expected.0.usage.diagnostics, 0);
    assert_eq!(end_tag_solidus.expected.0.usage.peak_attributes_per_tag, 0);
    assert_eq!(
        end_tag_solidus.expected.0.usage.retained_interpreted_bytes,
        2
    );
    assert_eq!(
        end_tag_solidus.expected.0.usage.peak_temporary_buffer_bytes,
        0
    );
    assert_eq!(end_tag_solidus.expected.0.coverage.processed_prefix.end, 5);

    let missing_value = find_regression("REG-113-missing-attribute-value-emission-refusal");
    assert_eq!(missing_value.source_bytes, b"z<a b=>");
    assert_eq!(missing_value.expected.0.usage.transition_steps, 9);
    assert_eq!(missing_value.expected.0.usage.emitted_tokens, 1);
    assert_eq!(missing_value.expected.0.usage.diagnostics, 1);
    assert_eq!(missing_value.expected.0.usage.peak_attributes_per_tag, 1);
    assert_eq!(missing_value.expected.0.usage.retained_interpreted_bytes, 3);
    assert_eq!(
        missing_value.expected.0.usage.peak_temporary_buffer_bytes,
        0
    );
    assert_eq!(missing_value.expected.0.coverage.processed_prefix.end, 7);
}

#[test]
fn end_tag_attributes_and_solidus_diagnostics_are_emission_conditioned_by_contrast() {
    // The primary purpose of this supplemental layer: EndTagWithAttributes
    // and EndTagWithTrailingSolidus are emission-conditioned and therefore
    // absent after non-emission (ERR-016/ERR-017 show them present only when
    // the end-tag token actually emits). MissingAttributeValue is
    // observation-conditioned and remains present via AbandonedInput once
    // its builder-level recovery already committed.
    let err_016 = find("ERR-016");
    assert_eq!(
        err_016.expected.0.diagnostics[0].code,
        DiagnosticCode::EndTagWithAttributes
    );
    assert!(matches!(
        err_016.expected.0.diagnostics[0].subject,
        DiagnosticSubject::EmittedToken(_)
    ));

    let err_017 = find("ERR-017");
    assert_eq!(
        err_017.expected.0.diagnostics[0].code,
        DiagnosticCode::EndTagWithTrailingSolidus
    );
    assert!(matches!(
        err_017.expected.0.diagnostics[0].subject,
        DiagnosticSubject::EmittedToken(_)
    ));

    let reg_attributes = find_regression("REG-113-end-tag-attributes-emission-refusal");
    assert!(
        !reg_attributes
            .expected
            .0
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::EndTagWithAttributes),
        "EndTagWithAttributes must not appear when emission is refused"
    );

    let reg_solidus = find_regression("REG-113-end-tag-trailing-solidus-emission-refusal");
    assert!(
        !reg_solidus
            .expected
            .0
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::EndTagWithTrailingSolidus),
        "EndTagWithTrailingSolidus must not appear when emission is refused"
    );

    let reg_missing_value = find_regression("REG-113-missing-attribute-value-emission-refusal");
    assert!(
        reg_missing_value
            .expected
            .0
            .diagnostics
            .iter()
            .any(
                |diagnostic| diagnostic.code == DiagnosticCode::MissingAttributeValue
                    && matches!(diagnostic.subject, DiagnosticSubject::AbandonedInput(_))
            ),
        "MissingAttributeValue must remain present via AbandonedInput"
    );
}

fn observed_empty_run(source_id: u64) -> ObservedRun {
    let source = SourceText::new(SourceId::new(source_id), String::new());
    let eof = HtmlToken::EndOfFile(
        HtmlEndOfFileToken::new(&source, source.anchor(0, 0).unwrap()).unwrap(),
    );
    let coverage = HtmlTokenizerCoverage::new(
        &source,
        source.anchor(0, 0).unwrap(),
        source.anchor(0, 0).unwrap(),
    )
    .unwrap();
    let limits = HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024);
    let usage = HtmlTokenizerUsage::new(0, 1, 1, 0, 0, 0, 0);
    let run = HtmlTokenizerRunResult::new(
        &source,
        vec![eof],
        HtmlPreprocessingEvidence::new(&source, None).unwrap(),
        Vec::new(),
        coverage,
        HtmlTokenizerCompletion::Complete,
        limits,
        usage,
    )
    .unwrap();
    observe(&source, &run)
}
