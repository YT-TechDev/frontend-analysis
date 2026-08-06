use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};

#[allow(clippy::too_many_arguments)]
pub(super) fn complete_tag_fixture(
    id: &'static str,
    purpose: &'static str,
    source: &'static str,
    kind: TokenKind,
    complete_start: usize,
    complete_end: usize,
    open_start: usize,
    open_end: usize,
    name_start: usize,
    name_end: usize,
    interpreted_name: &'static str,
    attributes: Vec<Attribute>,
    solidus: Option<(usize, usize)>,
    close_start: usize,
    close_end: usize,
    diagnostics: Vec<Diagnostic>,
    transition_steps: usize,
) -> HtmlTokenizerFixture {
    let token = tag(
        source,
        kind,
        complete_start,
        complete_end,
        open_start,
        open_end,
        name_start,
        name_end,
        interpreted_name,
        attributes,
        solidus,
        close_start,
        close_end,
    );
    let peak_attributes = match &token {
        Token::Tag { attributes, .. } => attributes.len(),
        _ => 0,
    };
    let category = if id.starts_with("TOK-") {
        FixtureCategory::SupportedToken
    } else if id.starts_with("ERR-") {
        FixtureCategory::Diagnostic
    } else {
        FixtureCategory::Adversarial
    };
    complete(
        id,
        category,
        purpose,
        source,
        vec![token, eof(source.len())],
        diagnostics,
        None,
        peak_attributes,
        transition_steps,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn complete_text(
    id: &'static str,
    category: FixtureCategory,
    purpose: &'static str,
    source: &'static str,
    interpreted: &'static str,
    diagnostics: Vec<Diagnostic>,
    bom: Option<ByteSpan>,
    transition_steps: usize,
) -> HtmlTokenizerFixture {
    let mut tokens = Vec::new();
    if !source.is_empty() {
        tokens.push(character(source, 0, source.len(), interpreted));
    }
    tokens.push(eof(source.len()));
    complete(
        id,
        category,
        purpose,
        source,
        tokens,
        diagnostics,
        bom,
        0,
        transition_steps,
    )
}

/// `transition_steps` is authored independently from candidate output: it is
/// the count of attempted specification-state transitions (including
/// reconsume) derived directly from the pinned WHATWG states approved in
/// #109/#111, per the audit table in `transition_audit.rs`. It MUST NOT be
/// derived from source byte length, token count, or diagnostic count.
#[allow(clippy::too_many_arguments)]
pub(super) fn complete(
    id: &'static str,
    category: FixtureCategory,
    purpose: &'static str,
    source: &'static str,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    bom: Option<ByteSpan>,
    peak_attributes: usize,
    transition_steps: usize,
) -> HtmlTokenizerFixture {
    let interpreted_bytes = tokens.iter().map(token_interpreted_bytes).sum();
    let run = CanonicalRun {
        skipped_leading_bom: bom,
        coverage: Coverage {
            processed_prefix: ByteSpan::new(0, source.len()),
            unprocessed_suffix: ByteSpan::new(source.len(), source.len()),
        },
        usage: usage(
            source,
            transition_steps,
            tokens.len(),
            diagnostics.len(),
            peak_attributes,
            interpreted_bytes,
            0,
        ),
        tokens,
        diagnostics,
        completion: Completion::Complete,
        limits: Limits::generous(),
    };
    fixture(id, category, purpose, source, run)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn incomplete(
    id: &'static str,
    category: FixtureCategory,
    purpose: &'static str,
    source: &'static str,
    processed_end: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    completion: Completion,
    limits: Limits,
    usage: Usage,
) -> HtmlTokenizerFixture {
    fixture(
        id,
        category,
        purpose,
        source,
        CanonicalRun {
            skipped_leading_bom: None,
            tokens,
            diagnostics,
            coverage: Coverage {
                processed_prefix: ByteSpan::new(0, processed_end),
                unprocessed_suffix: ByteSpan::new(processed_end, source.len()),
            },
            completion,
            limits,
            usage,
        },
    )
}

pub(super) fn fixture(
    id: &'static str,
    category: FixtureCategory,
    purpose: &'static str,
    source: &'static str,
    run: CanonicalRun,
) -> HtmlTokenizerFixture {
    HtmlTokenizerFixture {
        id,
        category,
        purpose,
        source_bytes: source.as_bytes(),
        expected: ExpectedRun(run),
    }
}

/// `transition_steps` is authored independently from candidate output, per
/// the audit table in `transition_audit.rs`. Coverage (`processed_end`) and
/// `transition_steps` are orthogonal: a capability-boundary discovery step
/// still counts as one attempted transition even where the triggering bytes
/// remain in the unprocessed suffix (#111 coverage rule).
#[allow(clippy::too_many_arguments)]
pub(super) fn unsupported_input(
    id: &'static str,
    purpose: &'static str,
    source: &'static str,
    processed_end: usize,
    capability: Capability,
    availability: Availability,
    trigger: ByteSpan,
    transition_steps: usize,
) -> HtmlTokenizerFixture {
    incomplete(
        id,
        FixtureCategory::Unsupported,
        purpose,
        source,
        processed_end,
        Vec::new(),
        Vec::new(),
        Completion::Unsupported {
            capability,
            availability,
            trigger: UnsupportedTrigger::Input(trigger),
        },
        Limits::generous(),
        usage(source, transition_steps, 0, 0, 0, 0, 0),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resource_fixture(
    id: &'static str,
    purpose: &'static str,
    source: &'static str,
    processed_end: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    resource: Resource,
    limit: usize,
    attempted: usize,
    at: ByteSpan,
    usage: Usage,
) -> HtmlTokenizerFixture {
    let mut limits = Limits::generous();
    match resource {
        Resource::SourceBytes => limits.source_bytes = limit,
        Resource::TransitionSteps => limits.transition_steps = limit,
        Resource::EmittedTokens => limits.emitted_tokens = limit,
        Resource::Diagnostics => limits.diagnostics = limit,
        Resource::AttributesPerTag => limits.attributes_per_tag = limit,
        Resource::RetainedInterpretedBytes => limits.retained_interpreted_bytes = limit,
        Resource::TemporaryBufferBytes => limits.temporary_buffer_bytes = limit,
    }
    incomplete(
        id,
        FixtureCategory::Resource,
        purpose,
        source,
        processed_end,
        tokens,
        diagnostics,
        Completion::ResourceLimit {
            resource,
            limit,
            attempted,
            at,
        },
        limits,
        usage,
    )
}

pub(super) fn character(
    source: &'static str,
    start: usize,
    end: usize,
    interpreted: &str,
) -> Token {
    Token::Character {
        source: lexeme(source, start, end),
        interpreted: interpreted.to_owned(),
    }
}

pub(super) fn eof(at: usize) -> Token {
    Token::EndOfFile {
        at: ByteSpan::new(at, at),
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn tag(
    source: &'static str,
    kind: TokenKind,
    complete_start: usize,
    complete_end: usize,
    open_start: usize,
    open_end: usize,
    name_start: usize,
    name_end: usize,
    interpreted_name: &str,
    attributes: Vec<Attribute>,
    solidus: Option<(usize, usize)>,
    close_start: usize,
    close_end: usize,
) -> Token {
    Token::Tag {
        kind,
        complete: lexeme(source, complete_start, complete_end),
        open_delimiter: lexeme(source, open_start, open_end),
        name: lexeme(source, name_start, name_end),
        interpreted_name: interpreted_name.to_owned(),
        attributes,
        self_closing_solidus: solidus.map(|(start, end)| lexeme(source, start, end)),
        close_delimiter: lexeme(source, close_start, close_end),
    }
}

pub(super) fn attribute_missing(
    source: &'static str,
    complete_start: usize,
    complete_end: usize,
    name_start: usize,
    name_end: usize,
    interpreted_name: &str,
    disposition: AttributeDisposition,
) -> Attribute {
    Attribute {
        complete: lexeme(source, complete_start, complete_end),
        name: lexeme(source, name_start, name_end),
        interpreted_name: interpreted_name.to_owned(),
        value: AttributeValue::Missing,
        interpreted_value: String::new(),
        disposition,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn attribute_missing_after_equals(
    source: &'static str,
    complete_start: usize,
    complete_end: usize,
    name_start: usize,
    name_end: usize,
    interpreted_name: &str,
    equals_start: usize,
    equals_end: usize,
    boundary: usize,
    disposition: AttributeDisposition,
) -> Attribute {
    Attribute {
        complete: lexeme(source, complete_start, complete_end),
        name: lexeme(source, name_start, name_end),
        interpreted_name: interpreted_name.to_owned(),
        value: AttributeValue::MissingAfterEquals {
            equals: lexeme(source, equals_start, equals_end),
            value_boundary: ByteSpan::new(boundary, boundary),
        },
        interpreted_value: String::new(),
        disposition,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn attribute_unquoted(
    source: &'static str,
    complete_start: usize,
    complete_end: usize,
    name_start: usize,
    name_end: usize,
    interpreted_name: &str,
    equals_start: usize,
    equals_end: usize,
    value_start: usize,
    value_end: usize,
    interpreted_value: &str,
    disposition: AttributeDisposition,
) -> Attribute {
    Attribute {
        complete: lexeme(source, complete_start, complete_end),
        name: lexeme(source, name_start, name_end),
        interpreted_name: interpreted_name.to_owned(),
        value: AttributeValue::Unquoted {
            equals: lexeme(source, equals_start, equals_end),
            value: lexeme(source, value_start, value_end),
        },
        interpreted_value: interpreted_value.to_owned(),
        disposition,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn attribute_double(
    source: &'static str,
    complete_start: usize,
    complete_end: usize,
    name_start: usize,
    name_end: usize,
    interpreted_name: &str,
    equals_start: usize,
    equals_end: usize,
    open_start: usize,
    open_end: usize,
    value_start: usize,
    value_end: usize,
    close_start: usize,
    close_end: usize,
    interpreted_value: &str,
    disposition: AttributeDisposition,
) -> Attribute {
    Attribute {
        complete: lexeme(source, complete_start, complete_end),
        name: lexeme(source, name_start, name_end),
        interpreted_name: interpreted_name.to_owned(),
        value: AttributeValue::DoubleQuoted {
            equals: lexeme(source, equals_start, equals_end),
            open_quote: lexeme(source, open_start, open_end),
            value: lexeme(source, value_start, value_end),
            close_quote: lexeme(source, close_start, close_end),
        },
        interpreted_value: interpreted_value.to_owned(),
        disposition,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn attribute_single(
    source: &'static str,
    complete_start: usize,
    complete_end: usize,
    name_start: usize,
    name_end: usize,
    interpreted_name: &str,
    equals_start: usize,
    equals_end: usize,
    open_start: usize,
    open_end: usize,
    value_start: usize,
    value_end: usize,
    close_start: usize,
    close_end: usize,
    interpreted_value: &str,
    disposition: AttributeDisposition,
) -> Attribute {
    Attribute {
        complete: lexeme(source, complete_start, complete_end),
        name: lexeme(source, name_start, name_end),
        interpreted_name: interpreted_name.to_owned(),
        value: AttributeValue::SingleQuoted {
            equals: lexeme(source, equals_start, equals_end),
            open_quote: lexeme(source, open_start, open_end),
            value: lexeme(source, value_start, value_end),
            close_quote: lexeme(source, close_start, close_end),
        },
        interpreted_value: interpreted_value.to_owned(),
        disposition,
    }
}

pub(super) fn lexeme(source: &'static str, start: usize, end: usize) -> Lexeme {
    Lexeme {
        span: ByteSpan::new(start, end),
        authored: source.as_bytes()[start..end].to_vec(),
    }
}

pub(super) fn diagnostic(
    code: DiagnosticCode,
    start: usize,
    end: usize,
    context: DiagnosticContext,
    handling: DiagnosticHandling,
    subject: DiagnosticSubject,
) -> Diagnostic {
    Diagnostic {
        code,
        location: ByteSpan::new(start, end),
        context,
        handling,
        subject,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn usage(
    source: &str,
    transition_steps: usize,
    emitted_tokens: usize,
    diagnostics: usize,
    peak_attributes_per_tag: usize,
    retained_interpreted_bytes: usize,
    peak_temporary_buffer_bytes: usize,
) -> Usage {
    Usage {
        source_bytes: source.len(),
        transition_steps,
        emitted_tokens,
        diagnostics,
        peak_attributes_per_tag,
        retained_interpreted_bytes,
        peak_temporary_buffer_bytes,
    }
}

fn token_interpreted_bytes(token: &Token) -> usize {
    match token {
        Token::Character { interpreted, .. } => interpreted.len(),
        Token::Tag {
            interpreted_name,
            attributes,
            ..
        } => {
            interpreted_name.len()
                + attributes
                    .iter()
                    .map(|attribute| {
                        attribute.interpreted_name.len() + attribute.interpreted_value.len()
                    })
                    .sum::<usize>()
        }
        Token::EndOfFile { .. } => 0,
    }
}
