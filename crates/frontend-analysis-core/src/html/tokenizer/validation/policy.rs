use std::fmt;

use super::expected::{
    Attribute, ByteSpan, CanonicalRun, Completion, ConfigurationFailure, Resource, Token,
};
use super::fixture::HtmlTokenizerFixture;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FixturePolicyError {
    fixture_id: &'static str,
    path: &'static str,
}

impl FixturePolicyError {
    fn new(fixture_id: &'static str, path: &'static str) -> Self {
        Self { fixture_id, path }
    }
}

impl fmt::Display for FixturePolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HTML tokenizer fixture policy violation: {}.{}",
            self.fixture_id, self.path
        )
    }
}

impl std::error::Error for FixturePolicyError {}

pub(super) fn validate_policy(
    fixtures: &[HtmlTokenizerFixture],
) -> Result<(), FixturePolicyError> {
    for fixture in fixtures {
        validate_fixture_policy(fixture)?;
    }
    Ok(())
}

fn validate_fixture_policy(fixture: &HtmlTokenizerFixture) -> Result<(), FixturePolicyError> {
    let run = &fixture.expected.0;
    let source = std::str::from_utf8(fixture.source_bytes)
        .map_err(|_| FixturePolicyError::new(fixture.id, "source.invalid_utf8"))?;

    let minimum_attributes = run
        .tokens
        .iter()
        .filter_map(|token| match token {
            Token::Tag { attributes, .. } => Some(attributes.len()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    if run.usage.peak_attributes_per_tag < minimum_attributes {
        return Err(FixturePolicyError::new(
            fixture.id,
            "usage.peak_attributes_per_tag",
        ));
    }

    let minimum_interpreted = run.tokens.iter().try_fold(0usize, |total, token| {
        total
            .checked_add(token_interpreted_bytes(token))
            .ok_or_else(|| FixturePolicyError::new(fixture.id, "usage.counter_overflow"))
    })?;
    if run.usage.retained_interpreted_bytes < minimum_interpreted {
        return Err(FixturePolicyError::new(
            fixture.id,
            "usage.retained_interpreted_bytes",
        ));
    }

    let exceeded_resource = match run.completion {
        Completion::ResourceLimit { resource, .. } => Some(resource),
        _ => None,
    };
    for resource in resources() {
        if resource == Resource::SourceBytes && exceeded_resource == Some(Resource::SourceBytes) {
            continue;
        }
        if usage_for(run, resource) > limit_for(run, resource) {
            return Err(FixturePolicyError::new(fixture.id, "usage.exceeds_limit"));
        }
    }

    let configured_failure = configuration_failure(run);
    match &run.completion {
        Completion::Complete => {
            if configured_failure.is_some() {
                return Err(FixturePolicyError::new(
                    fixture.id,
                    "completion.configuration_mismatch",
                ));
            }
        }
        Completion::Unsupported { trigger, .. } => {
            if configured_failure.is_some() {
                return Err(FixturePolicyError::new(
                    fixture.id,
                    "completion.configuration_mismatch",
                ));
            }
            match trigger {
                super::expected::UnsupportedTrigger::Input(span) => {
                    validate_span(fixture.id, source, *span, "completion.unsupported.input_span")?;
                    if span.start != run.coverage.processed_prefix.end {
                        return Err(FixturePolicyError::new(
                            fixture.id,
                            "completion.unsupported.input_boundary",
                        ));
                    }
                }
                super::expected::UnsupportedTrigger::EmittedToken {
                    token_index,
                    boundary,
                } => {
                    validate_span(
                        fixture.id,
                        source,
                        *boundary,
                        "completion.unsupported.token_boundary",
                    )?;
                    let Some(token) = run.tokens.get(*token_index) else {
                        return Err(FixturePolicyError::new(
                            fixture.id,
                            "completion.unsupported.token_index",
                        ));
                    };
                    if matches!(token, Token::EndOfFile { .. })
                        || !boundary.is_empty()
                        || boundary.start != token.top_level_span().end
                        || boundary.end > run.coverage.processed_prefix.end
                    {
                        return Err(FixturePolicyError::new(
                            fixture.id,
                            "completion.unsupported.token_relation",
                        ));
                    }
                }
            }
        }
        Completion::ResourceLimit {
            resource,
            limit,
            attempted,
            at,
        } => {
            if configured_failure.is_some() {
                return Err(FixturePolicyError::new(
                    fixture.id,
                    "completion.configuration_mismatch",
                ));
            }
            validate_span(fixture.id, source, *at, "completion.resource.at")?;
            if *limit != limit_for(run, *resource) || attempted <= limit {
                return Err(FixturePolicyError::new(
                    fixture.id,
                    "completion.resource.policy",
                ));
            }
            if at.start > run.coverage.processed_prefix.end {
                return Err(FixturePolicyError::new(
                    fixture.id,
                    "completion.resource.location",
                ));
            }
            if *resource == Resource::SourceBytes {
                if *attempted != source.len()
                    || run.coverage.processed_prefix.end != 0
                    || *at != ByteSpan::new(0, 0)
                {
                    return Err(FixturePolicyError::new(
                        fixture.id,
                        "completion.resource.source_bytes",
                    ));
                }
            } else {
                let committed = usage_for(run, *resource);
                let exact_increment = matches!(
                    resource,
                    Resource::TransitionSteps
                        | Resource::EmittedTokens
                        | Resource::Diagnostics
                        | Resource::AttributesPerTag
                );
                let expected_attempted = committed.checked_add(1).ok_or_else(|| {
                    FixturePolicyError::new(fixture.id, "completion.resource.counter_overflow")
                })?;
                if *attempted <= committed || (exact_increment && *attempted != expected_attempted)
                {
                    return Err(FixturePolicyError::new(
                        fixture.id,
                        "completion.resource.attempted",
                    ));
                }
            }
        }
        Completion::InvalidConfiguration(actual) => {
            if configured_failure != Some(*actual)
                || run.coverage.processed_prefix.end != 0
                || run.skipped_leading_bom.is_some()
                || !run.tokens.is_empty()
                || !run.diagnostics.is_empty()
                || run.usage.transition_steps != 0
                || run.usage.emitted_tokens != 0
                || run.usage.diagnostics != 0
                || run.usage.peak_attributes_per_tag != 0
                || run.usage.retained_interpreted_bytes != 0
                || run.usage.peak_temporary_buffer_bytes != 0
            {
                return Err(FixturePolicyError::new(
                    fixture.id,
                    "completion.invalid_configuration",
                ));
            }
        }
        Completion::InternalInvariantFailure(_) => {
            if configured_failure.is_some() {
                return Err(FixturePolicyError::new(
                    fixture.id,
                    "completion.configuration_mismatch",
                ));
            }
        }
    }
    Ok(())
}

fn configuration_failure(run: &CanonicalRun) -> Option<ConfigurationFailure> {
    if run.limits.transition_steps == 0 {
        Some(ConfigurationFailure::ZeroTransitionStepLimit)
    } else if run.limits.emitted_tokens == 0 {
        Some(ConfigurationFailure::ZeroEmittedTokenLimit)
    } else {
        None
    }
}

fn resources() -> [Resource; 7] {
    [
        Resource::SourceBytes,
        Resource::TransitionSteps,
        Resource::EmittedTokens,
        Resource::Diagnostics,
        Resource::AttributesPerTag,
        Resource::RetainedInterpretedBytes,
        Resource::TemporaryBufferBytes,
    ]
}

fn limit_for(run: &CanonicalRun, resource: Resource) -> usize {
    match resource {
        Resource::SourceBytes => run.limits.source_bytes,
        Resource::TransitionSteps => run.limits.transition_steps,
        Resource::EmittedTokens => run.limits.emitted_tokens,
        Resource::Diagnostics => run.limits.diagnostics,
        Resource::AttributesPerTag => run.limits.attributes_per_tag,
        Resource::RetainedInterpretedBytes => run.limits.retained_interpreted_bytes,
        Resource::TemporaryBufferBytes => run.limits.temporary_buffer_bytes,
    }
}

fn usage_for(run: &CanonicalRun, resource: Resource) -> usize {
    match resource {
        Resource::SourceBytes => run.usage.source_bytes,
        Resource::TransitionSteps => run.usage.transition_steps,
        Resource::EmittedTokens => run.usage.emitted_tokens,
        Resource::Diagnostics => run.usage.diagnostics,
        Resource::AttributesPerTag => run.usage.peak_attributes_per_tag,
        Resource::RetainedInterpretedBytes => run.usage.retained_interpreted_bytes,
        Resource::TemporaryBufferBytes => run.usage.peak_temporary_buffer_bytes,
    }
}

fn token_interpreted_bytes(token: &Token) -> usize {
    match token {
        Token::Character { interpreted, .. } => interpreted.len(),
        Token::Tag {
            interpreted_name,
            attributes,
            ..
        } => interpreted_name.len() + attributes.iter().map(attribute_interpreted_bytes).sum::<usize>(),
        Token::EndOfFile { .. } => 0,
    }
}

fn attribute_interpreted_bytes(attribute: &Attribute) -> usize {
    attribute.interpreted_name.len() + attribute.interpreted_value.len()
}

fn validate_span(
    fixture_id: &'static str,
    source: &str,
    span: ByteSpan,
    path: &'static str,
) -> Result<(), FixturePolicyError> {
    if span.start > span.end
        || span.end > source.len()
        || !source.is_char_boundary(span.start)
        || !source.is_char_boundary(span.end)
    {
        return Err(FixturePolicyError::new(fixture_id, path));
    }
    Ok(())
}
