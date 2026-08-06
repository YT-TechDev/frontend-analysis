use std::collections::BTreeSet;
use std::fmt;

use super::expected::{
    Attribute, AttributeValue, ByteSpan, Completion, DiagnosticHandling, DiagnosticSubject,
    ExpectedRun, Lexeme, Resource, Token,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum FixtureCategory {
    Preprocessing,
    SupportedToken,
    Diagnostic,
    Unsupported,
    Resource,
    Adversarial,
}

impl FixtureCategory {
    const fn prefix(self) -> &'static str {
        match self {
            Self::Preprocessing => "PRE-",
            Self::SupportedToken => "TOK-",
            Self::Diagnostic => "ERR-",
            Self::Unsupported => "UNSUP-",
            Self::Resource => "RES-",
            Self::Adversarial => "ADV-",
        }
    }
}

#[derive(Clone)]
pub(super) struct HtmlTokenizerFixture {
    pub(super) id: &'static str,
    pub(super) category: FixtureCategory,
    pub(super) purpose: &'static str,
    pub(super) source_bytes: &'static [u8],
    pub(super) expected: ExpectedRun,
}

impl fmt::Debug for HtmlTokenizerFixture {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTokenizerFixture")
            .field("id", &self.id)
            .field("category", &self.category)
            .field("purpose", &self.purpose)
            .field("source_byte_len", &self.source_bytes.len())
            .field("expected_token_count", &self.expected.0.tokens.len())
            .field(
                "expected_diagnostic_count",
                &self.expected.0.diagnostics.len(),
            )
            .field("completion", &self.expected.0.completion)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FixtureContractError {
    fixture_id: &'static str,
    path: String,
}

impl FixtureContractError {
    fn new(fixture_id: &'static str, path: impl Into<String>) -> Self {
        Self {
            fixture_id,
            path: path.into(),
        }
    }
}

impl fmt::Display for FixtureContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HTML tokenizer fixture contract violation: {}.{}",
            self.fixture_id, self.path
        )
    }
}

impl std::error::Error for FixtureContractError {}

impl HtmlTokenizerFixture {
    pub(super) fn validate(&self) -> Result<(), FixtureContractError> {
        if !self.id.starts_with(self.category.prefix()) {
            return Err(FixtureContractError::new(self.id, "id.category_prefix"));
        }
        if self.purpose.trim().is_empty() {
            return Err(FixtureContractError::new(self.id, "purpose.empty"));
        }
        let source = std::str::from_utf8(self.source_bytes)
            .map_err(|_| FixtureContractError::new(self.id, "source.invalid_utf8"))?;
        let run = &self.expected.0;
        let source_len = self.source_bytes.len();

        validate_span(self.id, source, run.coverage.processed_prefix, "coverage.processed")?;
        validate_span(
            self.id,
            source,
            run.coverage.unprocessed_suffix,
            "coverage.unprocessed",
        )?;
        if run.coverage.processed_prefix.start != 0
            || run.coverage.processed_prefix.end != run.coverage.unprocessed_suffix.start
            || run.coverage.unprocessed_suffix.end != source_len
        {
            return Err(FixtureContractError::new(self.id, "coverage.partition"));
        }

        if let Some(bom) = run.skipped_leading_bom {
            validate_span(self.id, source, bom, "preprocessing.bom")?;
            if bom.start != 0
                || bom.end > run.coverage.processed_prefix.end
                || self.source_bytes.get(bom.start..bom.end) != Some("\u{feff}".as_bytes())
            {
                return Err(FixtureContractError::new(
                    self.id,
                    "preprocessing.bom_evidence",
                ));
            }
        }

        let mut previous_end = 0usize;
        let mut eof_count = 0usize;
        for (index, token) in run.tokens.iter().enumerate() {
            let path = format!("tokens[{index}]");
            validate_token(self.id, source, token, &path)?;
            let span = token.top_level_span();
            if span.start < previous_end {
                return Err(FixtureContractError::new(self.id, format!("{path}.order")));
            }
            if !matches!(token, Token::EndOfFile { .. })
                && span.end > run.coverage.processed_prefix.end
            {
                return Err(FixtureContractError::new(
                    self.id,
                    format!("{path}.outside_processed_prefix"),
                ));
            }
            if matches!(token, Token::EndOfFile { .. }) {
                eof_count += 1;
                if index + 1 != run.tokens.len() || span != ByteSpan::new(source_len, source_len) {
                    return Err(FixtureContractError::new(self.id, "tokens.eof"));
                }
            }
            previous_end = span.end;
        }

        if run.completion.is_complete() {
            if run.coverage.unprocessed_suffix != ByteSpan::new(source_len, source_len)
                || eof_count != 1
            {
                return Err(FixtureContractError::new(self.id, "completion.complete"));
            }
        } else if eof_count != 0 {
            return Err(FixtureContractError::new(
                self.id,
                "completion.incomplete_contains_eof",
            ));
        }

        let mut previous_diagnostic_start = 0usize;
        for (index, diagnostic) in run.diagnostics.iter().enumerate() {
            let path = format!("diagnostics[{index}]");
            validate_span(self.id, source, diagnostic.location, &format!("{path}.location"))?;
            if index > 0 && diagnostic.location.start < previous_diagnostic_start {
                return Err(FixtureContractError::new(self.id, format!("{path}.order")));
            }
            if diagnostic.location.end > run.coverage.processed_prefix.end {
                return Err(FixtureContractError::new(
                    self.id,
                    format!("{path}.outside_processed_prefix"),
                ));
            }
            previous_diagnostic_start = diagnostic.location.start;
            match diagnostic.subject {
                DiagnosticSubject::InputLocation => {}
                DiagnosticSubject::EmittedToken(token_index) => {
                    if token_index >= run.tokens.len()
                        || matches!(run.tokens[token_index], Token::EndOfFile { .. })
                    {
                        return Err(FixtureContractError::new(
                            self.id,
                            format!("{path}.subject_token"),
                        ));
                    }
                }
                DiagnosticSubject::AbandonedInput(region) => {
                    validate_span(self.id, source, region, &format!("{path}.abandoned"))?;
                    if !region.contains(diagnostic.location)
                        || region.end > run.coverage.processed_prefix.end
                    {
                        return Err(FixtureContractError::new(
                            self.id,
                            format!("{path}.abandoned_relation"),
                        ));
                    }
                }
            }
            if matches!(diagnostic.handling, DiagnosticHandling::Stopped)
                && (run.completion.is_complete() || index + 1 != run.diagnostics.len())
            {
                return Err(FixtureContractError::new(
                    self.id,
                    format!("{path}.stopped_semantics"),
                ));
            }
        }

        if run.usage.source_bytes != source_len
            || run.usage.emitted_tokens != run.tokens.len()
            || run.usage.diagnostics != run.diagnostics.len()
        {
            return Err(FixtureContractError::new(self.id, "usage.counts"));
        }
        validate_terminal_policy(self)
    }
}

pub(super) fn validate_corpus(
    fixtures: &[HtmlTokenizerFixture],
) -> Result<(), FixtureContractError> {
    let mut ids = BTreeSet::new();
    for fixture in fixtures {
        if !ids.insert(fixture.id) {
            return Err(FixtureContractError::new(fixture.id, "id.duplicate"));
        }
        fixture.validate()?;
    }
    Ok(())
}

fn validate_terminal_policy(fixture: &HtmlTokenizerFixture) -> Result<(), FixtureContractError> {
    let run = &fixture.expected.0;
    let configuration_failure = if run.limits.transition_steps == 0 {
        Some(super::expected::ConfigurationFailure::ZeroTransitionStepLimit)
    } else if run.limits.emitted_tokens == 0 {
        Some(super::expected::ConfigurationFailure::ZeroEmittedTokenLimit)
    } else {
        None
    };
    match (&run.completion, configuration_failure) {
        (Completion::InvalidConfiguration(actual), Some(expected)) if *actual == expected => {}
        (Completion::InvalidConfiguration(_), _) | (_, Some(_)) => {
            return Err(FixtureContractError::new(
                fixture.id,
                "completion.configuration_policy",
            ));
        }
        (
            Completion::ResourceLimit {
                resource,
                limit,
                attempted,
                at,
            },
            None,
        ) => {
            validate_span(fixture.id, std::str::from_utf8(fixture.source_bytes).unwrap(), *at, "completion.resource.at")?;
            if attempted <= limit || *limit != limit_for(run, *resource) {
                return Err(FixtureContractError::new(
                    fixture.id,
                    "completion.resource_policy",
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn limit_for(run: &super::expected::CanonicalRun, resource: Resource) -> usize {
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

fn validate_token(
    fixture_id: &'static str,
    source: &str,
    token: &Token,
    path: &str,
) -> Result<(), FixtureContractError> {
    match token {
        Token::Character {
            source: lexeme,
            interpreted,
        } => {
            validate_lexeme(fixture_id, source, lexeme, &format!("{path}.source"))?;
            if lexeme.span.is_empty() || interpreted.is_empty() {
                return Err(FixtureContractError::new(
                    fixture_id,
                    format!("{path}.empty_character"),
                ));
            }
        }
        Token::Tag {
            complete,
            open_delimiter,
            name,
            interpreted_name,
            attributes,
            self_closing_solidus,
            close_delimiter,
            ..
        } => {
            validate_lexeme(fixture_id, source, complete, &format!("{path}.complete"))?;
            for (suffix, lexeme) in [
                ("open_delimiter", open_delimiter),
                ("name", name),
                ("close_delimiter", close_delimiter),
            ] {
                validate_nested_lexeme(
                    fixture_id,
                    source,
                    complete.span,
                    lexeme,
                    &format!("{path}.{suffix}"),
                )?;
            }
            if interpreted_name.is_empty()
                || complete.span.start != open_delimiter.span.start
                || complete.span.end != close_delimiter.span.end
                || open_delimiter.span.end > name.span.start
                || name.span.end > close_delimiter.span.start
            {
                return Err(FixtureContractError::new(
                    fixture_id,
                    format!("{path}.tag_boundaries"),
                ));
            }
            let mut previous_end = name.span.end;
            for (attribute_index, attribute) in attributes.iter().enumerate() {
                validate_attribute(
                    fixture_id,
                    source,
                    complete.span,
                    attribute,
                    &format!("{path}.attributes[{attribute_index}]"),
                )?;
                if attribute.complete.span.start < previous_end {
                    return Err(FixtureContractError::new(
                        fixture_id,
                        format!("{path}.attributes[{attribute_index}].order"),
                    ));
                }
                previous_end = attribute.complete.span.end;
            }
            if let Some(solidus) = self_closing_solidus {
                validate_nested_lexeme(
                    fixture_id,
                    source,
                    complete.span,
                    solidus,
                    &format!("{path}.self_closing_solidus"),
                )?;
                if solidus.authored != b"/" || solidus.span.end != close_delimiter.span.start {
                    return Err(FixtureContractError::new(
                        fixture_id,
                        format!("{path}.self_closing_position"),
                    ));
                }
            }
        }
        Token::EndOfFile { at } => validate_span(fixture_id, source, *at, path)?,
    }
    Ok(())
}

fn validate_attribute(
    fixture_id: &'static str,
    source: &str,
    owner: ByteSpan,
    attribute: &Attribute,
    path: &str,
) -> Result<(), FixtureContractError> {
    validate_nested_lexeme(fixture_id, source, owner, &attribute.complete, path)?;
    validate_nested_lexeme(
        fixture_id,
        source,
        attribute.complete.span,
        &attribute.name,
        &format!("{path}.name"),
    )?;
    if attribute.interpreted_name.is_empty()
        || attribute.complete.span.start != attribute.name.span.start
    {
        return Err(FixtureContractError::new(
            fixture_id,
            format!("{path}.name_boundary"),
        ));
    }
    match &attribute.value {
        AttributeValue::Missing => {
            if !attribute.interpreted_value.is_empty()
                || attribute.complete.span.end != attribute.name.span.end
            {
                return Err(FixtureContractError::new(
                    fixture_id,
                    format!("{path}.missing_value"),
                ));
            }
        }
        AttributeValue::MissingAfterEquals {
            equals,
            value_boundary,
        } => {
            validate_value_part(fixture_id, source, attribute.complete.span, equals, path)?;
            validate_span(fixture_id, source, *value_boundary, &format!("{path}.boundary"))?;
            if equals.authored != b"="
                || !value_boundary.is_empty()
                || value_boundary.end != attribute.complete.span.end
                || !attribute.interpreted_value.is_empty()
            {
                return Err(FixtureContractError::new(
                    fixture_id,
                    format!("{path}.missing_after_equals"),
                ));
            }
        }
        AttributeValue::Unquoted { equals, value } => {
            validate_value_part(fixture_id, source, attribute.complete.span, equals, path)?;
            validate_value_part(fixture_id, source, attribute.complete.span, value, path)?;
            if value.span.is_empty() {
                return Err(FixtureContractError::new(
                    fixture_id,
                    format!("{path}.unquoted_empty"),
                ));
            }
        }
        AttributeValue::DoubleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => {
            for part in [equals, open_quote, value, close_quote] {
                validate_value_part(fixture_id, source, attribute.complete.span, part, path)?;
            }
            if equals.authored != b"=" || open_quote.authored != b"\"" || close_quote.authored != b"\"" {
                return Err(FixtureContractError::new(fixture_id, format!("{path}.double_quote")));
            }
        }
        AttributeValue::SingleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => {
            for part in [equals, open_quote, value, close_quote] {
                validate_value_part(fixture_id, source, attribute.complete.span, part, path)?;
            }
            if equals.authored != b"=" || open_quote.authored != b"'" || close_quote.authored != b"'" {
                return Err(FixtureContractError::new(fixture_id, format!("{path}.single_quote")));
            }
        }
    }
    Ok(())
}

fn validate_value_part(
    fixture_id: &'static str,
    source: &str,
    owner: ByteSpan,
    lexeme: &Lexeme,
    path: &str,
) -> Result<(), FixtureContractError> {
    validate_nested_lexeme(fixture_id, source, owner, lexeme, path)
}

fn validate_nested_lexeme(
    fixture_id: &'static str,
    source: &str,
    owner: ByteSpan,
    lexeme: &Lexeme,
    path: &str,
) -> Result<(), FixtureContractError> {
    validate_lexeme(fixture_id, source, lexeme, path)?;
    if !owner.contains(lexeme.span) {
        return Err(FixtureContractError::new(
            fixture_id,
            format!("{path}.outside_owner"),
        ));
    }
    Ok(())
}

fn validate_lexeme(
    fixture_id: &'static str,
    source: &str,
    lexeme: &Lexeme,
    path: &str,
) -> Result<(), FixtureContractError> {
    validate_span(fixture_id, source, lexeme.span, path)?;
    if source.as_bytes().get(lexeme.span.start..lexeme.span.end) != Some(lexeme.authored.as_slice()) {
        return Err(FixtureContractError::new(
            fixture_id,
            format!("{path}.authored_bytes"),
        ));
    }
    Ok(())
}

fn validate_span(
    fixture_id: &'static str,
    source: &str,
    span: ByteSpan,
    path: &str,
) -> Result<(), FixtureContractError> {
    if span.start > span.end
        || span.end > source.len()
        || !source.is_char_boundary(span.start)
        || !source.is_char_boundary(span.end)
    {
        return Err(FixtureContractError::new(
            fixture_id,
            format!("{path}.invalid_span"),
        ));
    }
    Ok(())
}
