use crate::{SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum GoldGroup {
    InputPreprocessing,
    Comments,
    IdentEscape,
    Strings,
    Urls,
    Numeric,
    Fixed,
    Historical,
    Lifecycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GoldRange {
    pub(super) start: usize,
    pub(super) end: usize,
}

impl GoldRange {
    pub(super) const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub(super) const fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub(super) const fn contains(self, other: Self) -> bool {
        other.start >= self.start && other.end <= self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldHashType {
    Id,
    Unrestricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldNumberType {
    Integer,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldSign {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GoldNumber {
    pub(super) sign: Option<GoldSign>,
    pub(super) integer_digits: &'static str,
    pub(super) fraction_digits: &'static str,
    pub(super) exponent_sign: Option<GoldSign>,
    pub(super) exponent_digits: Option<&'static str>,
    pub(super) number_type: GoldNumberType,
}

impl GoldNumber {
    pub(super) const fn integer(sign: Option<GoldSign>, digits: &'static str) -> Self {
        Self {
            sign,
            integer_digits: digits,
            fraction_digits: "",
            exponent_sign: None,
            exponent_digits: None,
            number_type: GoldNumberType::Integer,
        }
    }

    pub(super) const fn number(
        sign: Option<GoldSign>,
        integer_digits: &'static str,
        fraction_digits: &'static str,
        exponent_sign: Option<GoldSign>,
        exponent_digits: Option<&'static str>,
    ) -> Self {
        Self {
            sign,
            integer_digits,
            fraction_digits,
            exponent_sign,
            exponent_digits,
            number_type: GoldNumberType::Number,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GoldTokenKind {
    Ident(&'static str),
    Function(&'static str),
    AtKeyword(&'static str),
    Hash {
        value: &'static str,
        hash_type: GoldHashType,
    },
    String(&'static str),
    BadString,
    Url(&'static str),
    BadUrl,
    Delim(char),
    Number(GoldNumber),
    Percentage(GoldNumber),
    Dimension {
        number: GoldNumber,
        unit: &'static str,
    },
    Whitespace,
    Cdo,
    Cdc,
    Colon,
    Semicolon,
    Comma,
    LeftSquareBracket,
    RightSquareBracket,
    LeftParenthesis,
    RightParenthesis,
    LeftCurlyBracket,
    RightCurlyBracket,
}

impl GoldTokenKind {
    pub(super) const fn class_name(&self) -> &'static str {
        match self {
            Self::Ident(_) => "ident",
            Self::Function(_) => "function",
            Self::AtKeyword(_) => "at-keyword",
            Self::Hash { .. } => "hash",
            Self::String(_) => "string",
            Self::BadString => "bad-string",
            Self::Url(_) => "url",
            Self::BadUrl => "bad-url",
            Self::Delim(_) => "delim",
            Self::Number(_) => "number",
            Self::Percentage(_) => "percentage",
            Self::Dimension { .. } => "dimension",
            Self::Whitespace => "whitespace",
            Self::Cdo => "cdo",
            Self::Cdc => "cdc",
            Self::Colon => "colon",
            Self::Semicolon => "semicolon",
            Self::Comma => "comma",
            Self::LeftSquareBracket => "left-square-bracket",
            Self::RightSquareBracket => "right-square-bracket",
            Self::LeftParenthesis => "left-parenthesis",
            Self::RightParenthesis => "right-parenthesis",
            Self::LeftCurlyBracket => "left-curly-bracket",
            Self::RightCurlyBracket => "right-curly-bracket",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldCommentTermination {
    Closed { close: GoldRange },
    EndOfInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GoldLexicalItem {
    Token {
        range: GoldRange,
        kind: GoldTokenKind,
    },
    Comment {
        range: GoldRange,
        open: GoldRange,
        termination: GoldCommentTermination,
    },
}

impl GoldLexicalItem {
    pub(super) const fn range(&self) -> GoldRange {
        match self {
            Self::Token { range, .. } | Self::Comment { range, .. } => *range,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum GoldDiagnosticCode {
    InvalidEscape,
    EofInComment,
    EofInString,
    NewlineInString,
    EofInUrl,
    InvalidUrlCodePoint,
    InvalidUrlEscape,
    EofInEscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldDiagnosticContext {
    TokenStart,
    Comment,
    String,
    Url,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldRecovery {
    EmitDelimiter,
    PreserveUnterminatedComment,
    EmitStringAtEndOfInput,
    EmitBadStringAndReconsumeNewline,
    EmitUrlAtEndOfInput,
    EmitBadUrl,
    SubstituteReplacementCharacterForEofEscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldDiagnosticSubject {
    InputLocation,
    LexicalItem(usize),
    InputRegion(GoldRange),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GoldDiagnostic {
    pub(super) code: GoldDiagnosticCode,
    pub(super) location: GoldRange,
    pub(super) context: GoldDiagnosticContext,
    pub(super) recovery: GoldRecovery,
    pub(super) subject: GoldDiagnosticSubject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GoldPreprocessedUnit {
    pub(super) semantic: char,
    pub(super) raw: GoldRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum GoldResourceKind {
    SourceBytes,
    AlgorithmSteps,
    LexicalItems,
    Diagnostics,
    RetainedInterpretedBytes,
    TemporaryBufferBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldCompletion {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldTermination {
    EndOfInput,
    ResourceLimit {
        kind: GoldResourceKind,
        limit: usize,
        attempted: usize,
        terminal: GoldRange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GoldFixture {
    pub(super) id: &'static str,
    pub(super) group: GoldGroup,
    pub(super) source_id: u64,
    pub(super) source: &'static str,
    pub(super) byte_len: usize,
    pub(super) leading_bom: Option<GoldRange>,
    pub(super) items: Vec<GoldLexicalItem>,
    pub(super) diagnostics: Vec<GoldDiagnostic>,
    pub(super) preprocessed_units: Vec<GoldPreprocessedUnit>,
    pub(super) terminal: GoldRange,
    pub(super) completion: GoldCompletion,
    pub(super) termination: GoldTermination,
}

impl GoldFixture {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn complete(
        id: &'static str,
        group: GoldGroup,
        source_id: u64,
        source: &'static str,
        byte_len: usize,
        leading_bom: Option<GoldRange>,
        items: Vec<GoldLexicalItem>,
        diagnostics: Vec<GoldDiagnostic>,
        preprocessed_units: Vec<GoldPreprocessedUnit>,
    ) -> Self {
        let terminal = GoldRange::new(byte_len, byte_len);
        Self {
            id,
            group,
            source_id,
            source,
            byte_len,
            leading_bom,
            items,
            diagnostics,
            preprocessed_units,
            terminal,
            completion: GoldCompletion::Complete,
            termination: GoldTermination::EndOfInput,
        }
    }

    pub(super) fn resource_limited(
        id: &'static str,
        source_id: u64,
        source: &'static str,
        byte_len: usize,
        kind: GoldResourceKind,
        limit: usize,
        attempted: usize,
    ) -> Self {
        let terminal = GoldRange::new(0, 0);
        Self {
            id,
            group: GoldGroup::Lifecycle,
            source_id,
            source,
            byte_len,
            leading_bom: None,
            items: Vec::new(),
            diagnostics: Vec::new(),
            preprocessed_units: Vec::new(),
            terminal,
            completion: GoldCompletion::Incomplete,
            termination: GoldTermination::ResourceLimit {
                kind,
                limit,
                attempted,
                terminal,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GoldValidationError {
    ByteLengthMismatch,
    InvalidRange,
    MissingLeadingBom,
    UnexpectedLeadingBom,
    InvalidLeadingBom,
    LexicalGapOrOverlap,
    InvalidCommentOpen,
    InvalidCommentClose,
    UnterminatedCommentNotAtEof,
    DiagnosticOrder,
    InvalidDiagnosticSubject,
    InvalidDiagnosticHandling,
    InvalidPreprocessedUnitOrder,
    PreprocessingCoverageGap,
    CompletionTerminationMismatch,
    InvalidTerminal,
    ResourceAttemptDidNotExceedLimit,
}

pub(super) fn validate_fixture(fixture: &GoldFixture) -> Result<(), GoldValidationError> {
    if fixture.source.len() != fixture.byte_len {
        return Err(GoldValidationError::ByteLengthMismatch);
    }

    let source = SourceText::new(SourceId::new(fixture.source_id), fixture.source.to_owned());
    validate_bom(&source, fixture.leading_bom)?;

    if !fixture.terminal.is_empty()
        || source
            .anchor(fixture.terminal.start, fixture.terminal.end)
            .is_err()
    {
        return Err(GoldValidationError::InvalidTerminal);
    }

    match (fixture.completion, fixture.termination) {
        (GoldCompletion::Complete, GoldTermination::EndOfInput) => {
            if fixture.terminal.start != fixture.byte_len {
                return Err(GoldValidationError::InvalidTerminal);
            }
        }
        (
            GoldCompletion::Incomplete,
            GoldTermination::ResourceLimit {
                limit,
                attempted,
                terminal,
                ..
            },
        ) => {
            if terminal != fixture.terminal {
                return Err(GoldValidationError::InvalidTerminal);
            }
            if attempted <= limit {
                return Err(GoldValidationError::ResourceAttemptDidNotExceedLimit);
            }
        }
        _ => return Err(GoldValidationError::CompletionTerminationMismatch),
    }

    let mut expected_start = fixture.leading_bom.map_or(0, |range| range.end);
    for item in &fixture.items {
        let range = item.range();
        if source.anchor(range.start, range.end).is_err() || range.is_empty() {
            return Err(GoldValidationError::InvalidRange);
        }
        if range.start != expected_start || range.end > fixture.terminal.start {
            return Err(GoldValidationError::LexicalGapOrOverlap);
        }
        validate_item(&source, item)?;
        expected_start = range.end;
    }
    if expected_start != fixture.terminal.start {
        return Err(GoldValidationError::LexicalGapOrOverlap);
    }

    let mut previous = None;
    for diagnostic in &fixture.diagnostics {
        if source
            .anchor(diagnostic.location.start, diagnostic.location.end)
            .is_err()
            || diagnostic.location.end > fixture.terminal.start
        {
            return Err(GoldValidationError::InvalidRange);
        }
        let key = (diagnostic.location.start, diagnostic.location.end);
        if previous.is_some_and(|value| value > key) {
            return Err(GoldValidationError::DiagnosticOrder);
        }
        previous = Some(key);
        validate_diagnostic(diagnostic, &fixture.items)?;
    }

    let mut previous_unit_end = None;
    for unit in &fixture.preprocessed_units {
        if source.anchor(unit.raw.start, unit.raw.end).is_err() || unit.raw.is_empty() {
            return Err(GoldValidationError::InvalidRange);
        }
        if previous_unit_end.is_some_and(|end| unit.raw.start < end) {
            return Err(GoldValidationError::InvalidPreprocessedUnitOrder);
        }
        previous_unit_end = Some(unit.raw.end);
    }

    if fixture.group == GoldGroup::InputPreprocessing {
        let mut expected_unit_start = fixture.leading_bom.map_or(0, |range| range.end);
        for unit in &fixture.preprocessed_units {
            if unit.raw.start != expected_unit_start {
                return Err(GoldValidationError::PreprocessingCoverageGap);
            }
            expected_unit_start = unit.raw.end;
        }
        if expected_unit_start != fixture.byte_len {
            return Err(GoldValidationError::PreprocessingCoverageGap);
        }
    }

    Ok(())
}

fn validate_bom(
    source: &SourceText,
    leading_bom: Option<GoldRange>,
) -> Result<(), GoldValidationError> {
    let has_bom = source.as_str().starts_with('\u{feff}');
    match (has_bom, leading_bom) {
        (true, None) => Err(GoldValidationError::MissingLeadingBom),
        (false, Some(_)) => Err(GoldValidationError::UnexpectedLeadingBom),
        (false, None) => Ok(()),
        (true, Some(range)) => {
            if range != GoldRange::new(0, 3) {
                return Err(GoldValidationError::InvalidLeadingBom);
            }
            let anchor = source
                .anchor(range.start, range.end)
                .map_err(|_| GoldValidationError::InvalidLeadingBom)?;
            if anchor.fragment() != "\u{feff}" {
                return Err(GoldValidationError::InvalidLeadingBom);
            }
            Ok(())
        }
    }
}

fn validate_item(source: &SourceText, item: &GoldLexicalItem) -> Result<(), GoldValidationError> {
    let GoldLexicalItem::Comment {
        range,
        open,
        termination,
    } = item
    else {
        return Ok(());
    };

    let open_anchor = source
        .anchor(open.start, open.end)
        .map_err(|_| GoldValidationError::InvalidCommentOpen)?;
    if open.start != range.start || open_anchor.fragment() != "/*" {
        return Err(GoldValidationError::InvalidCommentOpen);
    }

    match termination {
        GoldCommentTermination::Closed { close } => {
            let close_anchor = source
                .anchor(close.start, close.end)
                .map_err(|_| GoldValidationError::InvalidCommentClose)?;
            if close.end != range.end || close_anchor.fragment() != "*/" {
                return Err(GoldValidationError::InvalidCommentClose);
            }
        }
        GoldCommentTermination::EndOfInput => {
            if range.end != source.as_str().len() {
                return Err(GoldValidationError::UnterminatedCommentNotAtEof);
            }
        }
    }
    Ok(())
}

fn validate_diagnostic(
    diagnostic: &GoldDiagnostic,
    items: &[GoldLexicalItem],
) -> Result<(), GoldValidationError> {
    let (context, recovery) = expected_handling(diagnostic.code);
    if diagnostic.context != context || diagnostic.recovery != recovery {
        return Err(GoldValidationError::InvalidDiagnosticHandling);
    }

    match diagnostic.subject {
        GoldDiagnosticSubject::InputLocation => Ok(()),
        GoldDiagnosticSubject::LexicalItem(index) => {
            let Some(item) = items.get(index) else {
                return Err(GoldValidationError::InvalidDiagnosticSubject);
            };
            if !item.range().contains(diagnostic.location) {
                return Err(GoldValidationError::InvalidDiagnosticSubject);
            }
            Ok(())
        }
        GoldDiagnosticSubject::InputRegion(region) => {
            if !region.contains(diagnostic.location) {
                return Err(GoldValidationError::InvalidDiagnosticSubject);
            }
            Ok(())
        }
    }
}

const fn expected_handling(code: GoldDiagnosticCode) -> (GoldDiagnosticContext, GoldRecovery) {
    match code {
        GoldDiagnosticCode::InvalidEscape => (
            GoldDiagnosticContext::TokenStart,
            GoldRecovery::EmitDelimiter,
        ),
        GoldDiagnosticCode::EofInComment => (
            GoldDiagnosticContext::Comment,
            GoldRecovery::PreserveUnterminatedComment,
        ),
        GoldDiagnosticCode::EofInString => (
            GoldDiagnosticContext::String,
            GoldRecovery::EmitStringAtEndOfInput,
        ),
        GoldDiagnosticCode::NewlineInString => (
            GoldDiagnosticContext::String,
            GoldRecovery::EmitBadStringAndReconsumeNewline,
        ),
        GoldDiagnosticCode::EofInUrl => (
            GoldDiagnosticContext::Url,
            GoldRecovery::EmitUrlAtEndOfInput,
        ),
        GoldDiagnosticCode::InvalidUrlCodePoint | GoldDiagnosticCode::InvalidUrlEscape => {
            (GoldDiagnosticContext::Url, GoldRecovery::EmitBadUrl)
        }
        GoldDiagnosticCode::EofInEscape => (
            GoldDiagnosticContext::Escape,
            GoldRecovery::SubstituteReplacementCharacterForEofEscape,
        ),
    }
}
