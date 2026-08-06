use crate::{SourceAnchor, SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerDiagnosticCode {
    NoncharacterInInputStream,
    ControlCharacterInInputStream,
    UnexpectedNullCharacter,
    EofBeforeTagName,
    InvalidFirstCharacterOfTagName,
    UnexpectedQuestionMarkInsteadOfTagName,
    MissingEndTagName,
    EofInTag,
    UnexpectedEqualsSignBeforeAttributeName,
    UnexpectedCharacterInAttributeName,
    MissingAttributeValue,
    UnexpectedCharacterInUnquotedAttributeValue,
    MissingWhitespaceBetweenAttributes,
    UnexpectedSolidusInTag,
    DuplicateAttribute,
    EndTagWithAttributes,
    EndTagWithTrailingSolidus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerDiagnosticContext {
    InputPreprocessing,
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerRecoveryKind {
    ReplacedNullWithReplacementCharacter,
    EmittedLiteralMarkupPrefix,
    IgnoredUnexpectedInput,
    StartedAttributeAtUnexpectedEqualsSign,
    ReconsumedInData,
    ReconsumedBeforeAttributeName,
    CompletedTagWithMissingAttributeValue,
    PreservedDuplicateAttributeOccurrence,
    PreservedEndTagLexicalEvidence,
    AbandonedIncompleteTagAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerDiagnosticHandling {
    Continued,
    Recovered(HtmlTokenizerRecoveryKind),
    Stopped,
}

#[derive(Clone)]
pub(crate) enum HtmlTokenizerDiagnosticSubject {
    InputLocation,
    EmittedToken { token_index: usize },
    AbandonedInput { region: SourceAnchor },
}

impl std::fmt::Debug for HtmlTokenizerDiagnosticSubject {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputLocation => formatter.write_str("InputLocation"),
            Self::EmittedToken { token_index } => formatter
                .debug_struct("EmittedToken")
                .field("token_index", token_index)
                .finish(),
            Self::AbandonedInput { region } => formatter
                .debug_struct("AbandonedInput")
                .field("source_id", &region.source_id())
                .field("range", &region.range())
                .finish(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct HtmlTokenizerDiagnostic {
    code: HtmlTokenizerDiagnosticCode,
    location: SourceAnchor,
    context: HtmlTokenizerDiagnosticContext,
    handling: HtmlTokenizerDiagnosticHandling,
    subject: HtmlTokenizerDiagnosticSubject,
}

impl HtmlTokenizerDiagnostic {
    pub(crate) fn new(
        source_text: &SourceText,
        code: HtmlTokenizerDiagnosticCode,
        location: SourceAnchor,
        context: HtmlTokenizerDiagnosticContext,
        handling: HtmlTokenizerDiagnosticHandling,
        subject: HtmlTokenizerDiagnosticSubject,
    ) -> Result<Self, HtmlTokenizerDiagnosticContractError> {
        validate_source(
            source_text.id(),
            &location,
            HtmlDiagnosticEvidenceRole::Location,
        )?;
        if let HtmlTokenizerDiagnosticSubject::AbandonedInput { region } = &subject {
            validate_source(
                source_text.id(),
                region,
                HtmlDiagnosticEvidenceRole::AbandonedRegion,
            )?;
            if location.range().start() < region.range().start()
                || location.range().end() > region.range().end()
            {
                return Err(HtmlTokenizerDiagnosticContractError::LocationOutsideAbandonedRegion);
            }
        }

        Ok(Self {
            code,
            location,
            context,
            handling,
            subject,
        })
    }

    pub(crate) fn code(&self) -> HtmlTokenizerDiagnosticCode {
        self.code
    }

    pub(crate) fn location(&self) -> &SourceAnchor {
        &self.location
    }

    pub(crate) fn context(&self) -> HtmlTokenizerDiagnosticContext {
        self.context
    }

    pub(crate) fn handling(&self) -> HtmlTokenizerDiagnosticHandling {
        self.handling
    }

    pub(crate) fn subject(&self) -> &HtmlTokenizerDiagnosticSubject {
        &self.subject
    }
}

impl std::fmt::Debug for HtmlTokenizerDiagnostic {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HtmlTokenizerDiagnostic")
            .field("code", &self.code)
            .field("source_id", &self.location.source_id())
            .field("range", &self.location.range())
            .field("context", &self.context)
            .field("handling", &self.handling)
            .field("subject", &self.subject)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlDiagnosticEvidenceRole {
    Location,
    AbandonedRegion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerDiagnosticContractError {
    SourceIdentityMismatch {
        role: HtmlDiagnosticEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    LocationOutsideAbandonedRegion,
}

impl std::fmt::Display for HtmlTokenizerDiagnosticContractError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "HTML tokenizer diagnostic contract violation: {self:?}"
        )
    }
}

impl std::error::Error for HtmlTokenizerDiagnosticContractError {}

fn validate_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: HtmlDiagnosticEvidenceRole,
) -> Result<(), HtmlTokenizerDiagnosticContractError> {
    if anchor.source_id() != expected {
        return Err(
            HtmlTokenizerDiagnosticContractError::SourceIdentityMismatch {
                role,
                expected,
                actual: anchor.source_id(),
            },
        );
    }
    Ok(())
}
