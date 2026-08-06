#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ByteSpan {
    pub(super) start: usize,
    pub(super) end: usize,
}

impl ByteSpan {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(super) const fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub(super) const fn contains(self, other: Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Lexeme {
    pub(super) span: ByteSpan,
    pub(super) authored: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenKind {
    StartTag,
    EndTag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AttributeDisposition {
    Effective,
    DuplicateOf(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AttributeValue {
    Missing,
    MissingAfterEquals {
        equals: Lexeme,
        value_boundary: ByteSpan,
    },
    Unquoted {
        equals: Lexeme,
        value: Lexeme,
    },
    DoubleQuoted {
        equals: Lexeme,
        open_quote: Lexeme,
        value: Lexeme,
        close_quote: Lexeme,
    },
    SingleQuoted {
        equals: Lexeme,
        open_quote: Lexeme,
        value: Lexeme,
        close_quote: Lexeme,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Attribute {
    pub(super) complete: Lexeme,
    pub(super) name: Lexeme,
    pub(super) interpreted_name: String,
    pub(super) value: AttributeValue,
    pub(super) interpreted_value: String,
    pub(super) disposition: AttributeDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Token {
    Character {
        source: Lexeme,
        interpreted: String,
    },
    Tag {
        kind: TokenKind,
        complete: Lexeme,
        open_delimiter: Lexeme,
        name: Lexeme,
        interpreted_name: String,
        attributes: Vec<Attribute>,
        self_closing_solidus: Option<Lexeme>,
        close_delimiter: Lexeme,
    },
    EndOfFile {
        at: ByteSpan,
    },
}

impl Token {
    pub(super) fn top_level_span(&self) -> ByteSpan {
        match self {
            Self::Character { source, .. } => source.span,
            Self::Tag { complete, .. } => complete.span,
            Self::EndOfFile { at } => *at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DiagnosticCode {
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
pub(super) enum DiagnosticContext {
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
pub(super) enum RecoveryKind {
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
pub(super) enum DiagnosticHandling {
    Continued,
    Recovered(RecoveryKind),
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DiagnosticSubject {
    InputLocation,
    EmittedToken(usize),
    AbandonedInput(ByteSpan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Diagnostic {
    pub(super) code: DiagnosticCode,
    pub(super) location: ByteSpan,
    pub(super) context: DiagnosticContext,
    pub(super) handling: DiagnosticHandling,
    pub(super) subject: DiagnosticSubject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Availability {
    Deferred,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CharacterReferenceContext {
    Data,
    AttributeValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenizerMode {
    Rcdata,
    RawText,
    ScriptData,
    Plaintext,
    Noscript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Capability {
    CharacterReference(CharacterReferenceContext),
    MarkupDeclaration,
    ProcessingInstruction,
    BogusCommentRecovery,
    ContextDependentTokenizerMode(TokenizerMode),
    TreeConstructionControlledState,
    ForeignContentControl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum UnsupportedTrigger {
    Input(ByteSpan),
    EmittedToken {
        token_index: usize,
        boundary: ByteSpan,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Resource {
    SourceBytes,
    TransitionSteps,
    EmittedTokens,
    Diagnostics,
    AttributesPerTag,
    RetainedInterpretedBytes,
    TemporaryBufferBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConfigurationFailure {
    ZeroTransitionStepLimit,
    ZeroEmittedTokenLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InvariantFailure {
    CursorState,
    ReconsumeState,
    BuilderState,
    EmissionOrder,
    SourceEvidenceConstruction,
    CounterOverflow,
    RunAssembly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Completion {
    Complete,
    Unsupported {
        capability: Capability,
        availability: Availability,
        trigger: UnsupportedTrigger,
    },
    ResourceLimit {
        resource: Resource,
        limit: usize,
        attempted: usize,
        at: ByteSpan,
    },
    InvalidConfiguration(ConfigurationFailure),
    InternalInvariantFailure(InvariantFailure),
}

impl Completion {
    pub(super) const fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Limits {
    pub(super) source_bytes: usize,
    pub(super) transition_steps: usize,
    pub(super) emitted_tokens: usize,
    pub(super) diagnostics: usize,
    pub(super) attributes_per_tag: usize,
    pub(super) retained_interpreted_bytes: usize,
    pub(super) temporary_buffer_bytes: usize,
}

impl Limits {
    pub(super) const fn generous() -> Self {
        Self {
            source_bytes: 1_024,
            transition_steps: 8_192,
            emitted_tokens: 1_024,
            diagnostics: 1_024,
            attributes_per_tag: 256,
            retained_interpreted_bytes: 4_096,
            temporary_buffer_bytes: 1_024,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Usage {
    pub(super) source_bytes: usize,
    pub(super) transition_steps: usize,
    pub(super) emitted_tokens: usize,
    pub(super) diagnostics: usize,
    pub(super) peak_attributes_per_tag: usize,
    pub(super) retained_interpreted_bytes: usize,
    pub(super) peak_temporary_buffer_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Coverage {
    pub(super) processed_prefix: ByteSpan,
    pub(super) unprocessed_suffix: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CanonicalRun {
    pub(super) skipped_leading_bom: Option<ByteSpan>,
    pub(super) tokens: Vec<Token>,
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) coverage: Coverage,
    pub(super) completion: Completion,
    pub(super) limits: Limits,
    pub(super) usage: Usage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExpectedRun(pub(super) CanonicalRun);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ObservedRun(pub(super) CanonicalRun);
