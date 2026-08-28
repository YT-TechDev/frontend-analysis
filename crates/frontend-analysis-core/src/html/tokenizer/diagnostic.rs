use crate::{SourceAnchor, SourceId, SourceText};

/// Every first-slice diagnostic code is either *observation-conditioned* or
/// *emission-conditioned*. This distinction governs which terminal subject a
/// diagnostic may carry in a collected [`super::result::HtmlTokenizerRunResult`].
///
/// An observation-conditioned diagnostic's underlying parse-error fact becomes
/// true when the approved specification observation and its committed
/// recovery sub-effect occur; it does not become false merely because a later,
/// independent top-level token emission is refused. It may finish related to
/// [`HtmlTokenizerDiagnosticSubject::AbandonedInput`] when the tag was
/// completed or partially constructed but never emitted.
///
/// An emission-conditioned diagnostic's underlying parse-error fact exists
/// only once the corresponding end-tag token emission actually commits. If
/// that token does not emit, the diagnostic itself must not appear in the
/// final result. [`HtmlTokenizerDiagnosticCode::is_emission_conditioned`]
/// classifies the finite emission-conditioned subset; every other code in
/// this enum is observation-conditioned. This is a project-owned contract
/// clarification: the pinned WHATWG snapshot distinguishes end-tag-token
/// creation from end-tag-token emission and defines both
/// `EndTagWithAttributes` and `EndTagWithTrailingSolidus` as conditions on the
/// emitted end-tag token, but assigns no diagnostic-subject or termination
/// model of its own.
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
    MissingSemicolonAfterCharacterReference,
    UnknownNamedCharacterReference,
}

impl HtmlTokenizerDiagnosticCode {
    /// Returns `true` for the finite subset of first-slice diagnostics whose
    /// underlying parse-error fact requires a committed end-tag token
    /// emission. See the type-level documentation for the observation- vs
    /// emission-conditioned distinction this method enforces.
    pub(crate) fn is_emission_conditioned(self) -> bool {
        matches!(
            self,
            Self::EndTagWithAttributes | Self::EndTagWithTrailingSolidus
        )
    }
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
    NamedCharacterReference,
    AmbiguousAmpersand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTokenizerRecoveryKind {
    ReplacedNullWithReplacementCharacter,
    EmittedLiteralMarkupPrefix,
    IgnoredUnexpectedInput,
    StartedAttributeAtUnexpectedEqualsSign,
    ReconsumedInData,
    ReconsumedBeforeAttributeName,
    /// The malformed attribute/tag evidence required by the approved recovery
    /// was completed at the tokenizer builder/token-construction layer. This
    /// does **not** mean the resulting top-level `HtmlToken` was appended to
    /// the run's emitted-token vector — construction and emission are
    /// distinct effects, and a later independent sub-effect (for example a
    /// token-capacity refusal) may still prevent emission. Because
    /// `MissingAttributeValue` is observation-conditioned, this recovery kind
    /// does not itself require top-level token emission.
    CompletedTagWithMissingAttributeValue,
    /// An observation-conditioned lexical recovery fact: the duplicate
    /// attribute occurrence was preserved in the builder once observed. It
    /// may remain a valid diagnostic subject even when a later, independent
    /// token emission is refused.
    PreservedDuplicateAttributeOccurrence,
    /// The lexical evidence for an end tag's attributes or trailing solidus
    /// may exist in a builder even though the associated diagnostic code
    /// (`EndTagWithAttributes` or `EndTagWithTrailingSolidus`) is
    /// emission-conditioned: the parse-error diagnostic itself cannot appear
    /// in the final result unless the end-tag token actually emitted. Builder
    /// evidence alone is not sufficient for those two codes.
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
    /// Required terminal subject for an emission-conditioned diagnostic code
    /// (see [`HtmlTokenizerDiagnosticCode::is_emission_conditioned`]); also
    /// valid for an observation-conditioned diagnostic related to a token
    /// that did emit.
    EmittedToken {
        token_index: usize,
    },
    /// May describe either an incomplete builder abandoned before token
    /// completion, or source-backed recovered/completed builder evidence
    /// that did not ultimately become an emitted top-level token. The second
    /// meaning is valid only for an observation-conditioned diagnostic: an
    /// emission-conditioned diagnostic's underlying fact does not exist
    /// without a committed token emission, so it must never use this
    /// subject. The region must still satisfy all existing source identity,
    /// range, coverage, and diagnostic-location invariants. Every subject
    /// stored in a collected `HtmlTokenizerRunResult` already represents the
    /// true terminal relationship; producers may use provisional relations
    /// internally, but none may reach the collected result unresolved.
    AbandonedInput {
        region: SourceAnchor,
    },
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
