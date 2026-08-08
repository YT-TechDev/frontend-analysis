use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssHashType {
    Id,
    Unrestricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssNumberType {
    Integer,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssNumberSign {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssExponentSign {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssDecimalExponent {
    sign: Option<CssExponentSign>,
    digits: String,
}

impl CssDecimalExponent {
    pub(crate) fn new(
        sign: Option<CssExponentSign>,
        digits: String,
    ) -> Result<Self, CssTokenContractError> {
        validate_digits(&digits, CssTokenEvidenceRole::ExponentDigits, false)?;
        Ok(Self { sign, digits })
    }

    pub(crate) const fn sign(&self) -> Option<CssExponentSign> {
        self.sign
    }

    pub(crate) fn digits(&self) -> &str {
        &self.digits
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssDecimalValue {
    integer_digits: String,
    fraction_digits: String,
    exponent: Option<CssDecimalExponent>,
}

impl CssDecimalValue {
    pub(crate) fn new(
        integer_digits: String,
        fraction_digits: String,
        exponent: Option<CssDecimalExponent>,
    ) -> Result<Self, CssTokenContractError> {
        validate_digits(&integer_digits, CssTokenEvidenceRole::IntegerDigits, false)?;
        validate_digits(&fraction_digits, CssTokenEvidenceRole::FractionDigits, true)?;
        Ok(Self {
            integer_digits,
            fraction_digits,
            exponent,
        })
    }

    pub(crate) fn integer_digits(&self) -> &str {
        &self.integer_digits
    }

    pub(crate) fn fraction_digits(&self) -> &str {
        &self.fraction_digits
    }

    pub(crate) fn exponent(&self) -> Option<&CssDecimalExponent> {
        self.exponent.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssNumericValue {
    decimal: CssDecimalValue,
    sign: Option<CssNumberSign>,
}

impl CssNumericValue {
    pub(crate) const fn new(decimal: CssDecimalValue, sign: Option<CssNumberSign>) -> Self {
        Self { decimal, sign }
    }

    pub(crate) const fn decimal(&self) -> &CssDecimalValue {
        &self.decimal
    }

    pub(crate) const fn sign(&self) -> Option<CssNumberSign> {
        self.sign
    }

    pub(crate) fn required_number_type(&self) -> CssNumberType {
        if self.decimal.fraction_digits().is_empty() && self.decimal.exponent().is_none() {
            CssNumberType::Integer
        } else {
            CssNumberType::Number
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum CssTokenKind {
    Ident(String),
    Function(String),
    AtKeyword(String),
    Hash {
        value: String,
        hash_type: CssHashType,
    },
    String(String),
    BadString,
    Url(String),
    BadUrl,
    Delim(char),
    Number {
        value: CssNumericValue,
        number_type: CssNumberType,
    },
    Percentage {
        value: CssNumericValue,
    },
    Dimension {
        value: CssNumericValue,
        number_type: CssNumberType,
        unit: String,
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

impl CssTokenKind {
    fn label(&self) -> &'static str {
        match self {
            Self::Ident(_) => "Ident",
            Self::Function(_) => "Function",
            Self::AtKeyword(_) => "AtKeyword",
            Self::Hash { .. } => "Hash",
            Self::String(_) => "String",
            Self::BadString => "BadString",
            Self::Url(_) => "Url",
            Self::BadUrl => "BadUrl",
            Self::Delim(_) => "Delim",
            Self::Number { .. } => "Number",
            Self::Percentage { .. } => "Percentage",
            Self::Dimension { .. } => "Dimension",
            Self::Whitespace => "Whitespace",
            Self::Cdo => "Cdo",
            Self::Cdc => "Cdc",
            Self::Colon => "Colon",
            Self::Semicolon => "Semicolon",
            Self::Comma => "Comma",
            Self::LeftSquareBracket => "LeftSquareBracket",
            Self::RightSquareBracket => "RightSquareBracket",
            Self::LeftParenthesis => "LeftParenthesis",
            Self::RightParenthesis => "RightParenthesis",
            Self::LeftCurlyBracket => "LeftCurlyBracket",
            Self::RightCurlyBracket => "RightCurlyBracket",
        }
    }

    fn interpreted_byte_len(&self) -> Option<usize> {
        match self {
            Self::Ident(value)
            | Self::Function(value)
            | Self::AtKeyword(value)
            | Self::String(value)
            | Self::Url(value) => Some(value.len()),
            Self::Hash { value, .. } => Some(value.len()),
            Self::Dimension { unit, .. } => Some(unit.len()),
            _ => None,
        }
    }
}

impl fmt::Debug for CssTokenKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssTokenKind")
            .field("variant", &self.label())
            .field("interpreted_byte_len", &self.interpreted_byte_len())
            .finish()
    }
}

#[derive(Clone)]
pub(crate) struct CssToken {
    source: SourceAnchor,
    kind: CssTokenKind,
}

impl CssToken {
    pub(crate) fn new(
        source_text: &SourceText,
        source: SourceAnchor,
        kind: CssTokenKind,
    ) -> Result<Self, CssTokenContractError> {
        same_source(source_text.id(), &source, CssTokenEvidenceRole::Token)?;
        non_empty(&source, CssTokenEvidenceRole::Token)?;
        validate_kind(&source, &kind)?;
        Ok(Self { source, kind })
    }

    pub(crate) const fn source(&self) -> &SourceAnchor {
        &self.source
    }

    pub(crate) const fn kind(&self) -> &CssTokenKind {
        &self.kind
    }
}

impl PartialEq for CssToken {
    fn eq(&self, other: &Self) -> bool {
        same_anchor(&self.source, &other.source) && self.kind == other.kind
    }
}

impl Eq for CssToken {}

impl fmt::Debug for CssToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssToken")
            .field("source_id", &self.source.source_id())
            .field("range", &self.source.range())
            .field("kind", &self.kind)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum CssCommentTermination {
    Closed { close_delimiter: SourceAnchor },
    EndOfInput,
}

impl PartialEq for CssCommentTermination {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Closed {
                    close_delimiter: left,
                },
                Self::Closed {
                    close_delimiter: right,
                },
            ) => same_anchor(left, right),
            (Self::EndOfInput, Self::EndOfInput) => true,
            _ => false,
        }
    }
}

impl Eq for CssCommentTermination {}

#[derive(Clone)]
pub(crate) struct CssCommentEvidence {
    complete: SourceAnchor,
    open_delimiter: SourceAnchor,
    termination: CssCommentTermination,
}

impl CssCommentEvidence {
    pub(crate) fn new(
        source_text: &SourceText,
        complete: SourceAnchor,
        open_delimiter: SourceAnchor,
        termination: CssCommentTermination,
    ) -> Result<Self, CssTokenContractError> {
        same_source(source_text.id(), &complete, CssTokenEvidenceRole::Comment)?;
        same_source(
            source_text.id(),
            &open_delimiter,
            CssTokenEvidenceRole::CommentOpenDelimiter,
        )?;
        non_empty(&complete, CssTokenEvidenceRole::Comment)?;
        exact(
            &open_delimiter,
            CssTokenEvidenceRole::CommentOpenDelimiter,
            "/*",
        )?;
        contained(
            &complete,
            &open_delimiter,
            CssTokenEvidenceRole::CommentOpenDelimiter,
        )?;
        if complete.range().start() != open_delimiter.range().start() {
            return Err(CssTokenContractError::MisalignedBoundary {
                role: CssTokenEvidenceRole::CommentOpenDelimiter,
            });
        }

        match &termination {
            CssCommentTermination::Closed { close_delimiter } => {
                same_source(
                    source_text.id(),
                    close_delimiter,
                    CssTokenEvidenceRole::CommentCloseDelimiter,
                )?;
                exact(
                    close_delimiter,
                    CssTokenEvidenceRole::CommentCloseDelimiter,
                    "*/",
                )?;
                contained(
                    &complete,
                    close_delimiter,
                    CssTokenEvidenceRole::CommentCloseDelimiter,
                )?;
                if complete.range().end() != close_delimiter.range().end() {
                    return Err(CssTokenContractError::MisalignedBoundary {
                        role: CssTokenEvidenceRole::CommentCloseDelimiter,
                    });
                }
                if open_delimiter.range().end() > close_delimiter.range().start() {
                    return Err(CssTokenContractError::EvidenceOutOfOrder {
                        role: CssTokenEvidenceRole::CommentCloseDelimiter,
                    });
                }
            }
            CssCommentTermination::EndOfInput => {
                if complete.range().end() != source_text.as_str().len() {
                    return Err(CssTokenContractError::CommentEndOfInputNotAtSourceEnd {
                        end: complete.range().end(),
                        source_len: source_text.as_str().len(),
                    });
                }
            }
        }

        Ok(Self {
            complete,
            open_delimiter,
            termination,
        })
    }

    pub(crate) const fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    pub(crate) const fn open_delimiter(&self) -> &SourceAnchor {
        &self.open_delimiter
    }

    pub(crate) const fn termination(&self) -> &CssCommentTermination {
        &self.termination
    }
}

impl PartialEq for CssCommentEvidence {
    fn eq(&self, other: &Self) -> bool {
        same_anchor(&self.complete, &other.complete)
            && same_anchor(&self.open_delimiter, &other.open_delimiter)
            && self.termination == other.termination
    }
}

impl Eq for CssCommentEvidence {}

impl fmt::Debug for CssCommentEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssCommentEvidence")
            .field("source_id", &self.complete.source_id())
            .field("range", &self.complete.range())
            .field("termination", &self.termination)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssLexicalItem {
    SemanticToken(CssToken),
    Comment(CssCommentEvidence),
}

impl CssLexicalItem {
    pub(crate) const fn source(&self) -> &SourceAnchor {
        match self {
            Self::SemanticToken(token) => token.source(),
            Self::Comment(comment) => comment.complete(),
        }
    }

    pub(crate) fn source_order_key(&self) -> (u64, usize, usize) {
        let source = self.source();
        (
            source.source_id().value(),
            source.range().start(),
            source.range().end(),
        )
    }

    pub(crate) fn compare_source_order(
        &self,
        other: &Self,
    ) -> Result<Ordering, CssLexicalOrderError> {
        let expected = self.source().source_id();
        let actual = other.source().source_id();
        if expected != actual {
            return Err(CssLexicalOrderError::SourceIdentityMismatch { expected, actual });
        }

        Ok((self.source().range().start(), self.source().range().end())
            .cmp(&(other.source().range().start(), other.source().range().end())))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssLexicalOrderError {
    SourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
}

impl fmt::Display for CssLexicalOrderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS lexical source-order comparison failed: {self:?}"
        )
    }
}

impl Error for CssLexicalOrderError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenEvidenceRole {
    Token,
    FixedToken,
    IdentValue,
    FunctionValue,
    AtKeywordValue,
    HashValue,
    DimensionUnit,
    IntegerDigits,
    FractionDigits,
    ExponentDigits,
    Comment,
    CommentOpenDelimiter,
    CommentCloseDelimiter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTokenContractError {
    SourceIdentityMismatch {
        role: CssTokenEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    EmptySourceRange {
        role: CssTokenEvidenceRole,
    },
    EmptyInterpretedValue {
        role: CssTokenEvidenceRole,
    },
    InvalidAsciiDigits {
        role: CssTokenEvidenceRole,
    },
    FixedSpellingMismatch {
        role: CssTokenEvidenceRole,
        expected: &'static str,
    },
    InvalidWhitespaceSource,
    InvalidFunctionBoundary,
    InvalidAtKeywordBoundary,
    InvalidHashBoundary,
    InvalidStringBoundary,
    InvalidDimensionUnit,
    InvalidNumberType {
        expected: CssNumberType,
        actual: CssNumberType,
    },
    EvidenceOutsideContainer {
        role: CssTokenEvidenceRole,
    },
    EvidenceOutOfOrder {
        role: CssTokenEvidenceRole,
    },
    MisalignedBoundary {
        role: CssTokenEvidenceRole,
    },
    CommentEndOfInputNotAtSourceEnd {
        end: usize,
        source_len: usize,
    },
}

impl fmt::Display for CssTokenContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS token contract violation: {self:?}")
    }
}

impl Error for CssTokenContractError {}

fn validate_kind(source: &SourceAnchor, kind: &CssTokenKind) -> Result<(), CssTokenContractError> {
    match kind {
        CssTokenKind::Ident(value) => required_value(value, CssTokenEvidenceRole::IdentValue),
        CssTokenKind::Function(value) => {
            required_value(value, CssTokenEvidenceRole::FunctionValue)?;
            if !source.fragment().ends_with('(') {
                return Err(CssTokenContractError::InvalidFunctionBoundary);
            }
            Ok(())
        }
        CssTokenKind::AtKeyword(value) => {
            required_value(value, CssTokenEvidenceRole::AtKeywordValue)?;
            if !source.fragment().starts_with('@') {
                return Err(CssTokenContractError::InvalidAtKeywordBoundary);
            }
            Ok(())
        }
        CssTokenKind::Hash { value, .. } => {
            required_value(value, CssTokenEvidenceRole::HashValue)?;
            if !source.fragment().starts_with('#') {
                return Err(CssTokenContractError::InvalidHashBoundary);
            }
            Ok(())
        }
        CssTokenKind::String(_) | CssTokenKind::BadString => {
            if !matches!(source.fragment().chars().next(), Some('"' | '\'')) {
                return Err(CssTokenContractError::InvalidStringBoundary);
            }
            Ok(())
        }
        CssTokenKind::Number { value, number_type } => validate_number_type(value, *number_type),
        CssTokenKind::Dimension {
            value,
            number_type,
            unit,
        } => {
            validate_number_type(value, *number_type)?;
            required_value(unit, CssTokenEvidenceRole::DimensionUnit)
                .map_err(|_| CssTokenContractError::InvalidDimensionUnit)
        }
        CssTokenKind::Whitespace => {
            if !source.fragment().chars().all(is_raw_css_whitespace) {
                return Err(CssTokenContractError::InvalidWhitespaceSource);
            }
            Ok(())
        }
        CssTokenKind::Cdo => exact(source, CssTokenEvidenceRole::FixedToken, "<!--"),
        CssTokenKind::Cdc => exact(source, CssTokenEvidenceRole::FixedToken, "-->"),
        CssTokenKind::Colon => exact(source, CssTokenEvidenceRole::FixedToken, ":"),
        CssTokenKind::Semicolon => exact(source, CssTokenEvidenceRole::FixedToken, ";"),
        CssTokenKind::Comma => exact(source, CssTokenEvidenceRole::FixedToken, ","),
        CssTokenKind::LeftSquareBracket => exact(source, CssTokenEvidenceRole::FixedToken, "["),
        CssTokenKind::RightSquareBracket => exact(source, CssTokenEvidenceRole::FixedToken, "]"),
        CssTokenKind::LeftParenthesis => exact(source, CssTokenEvidenceRole::FixedToken, "("),
        CssTokenKind::RightParenthesis => exact(source, CssTokenEvidenceRole::FixedToken, ")"),
        CssTokenKind::LeftCurlyBracket => exact(source, CssTokenEvidenceRole::FixedToken, "{"),
        CssTokenKind::RightCurlyBracket => exact(source, CssTokenEvidenceRole::FixedToken, "}"),
        CssTokenKind::Url(_)
        | CssTokenKind::BadUrl
        | CssTokenKind::Delim(_)
        | CssTokenKind::Percentage { .. } => Ok(()),
    }
}

fn validate_number_type(
    value: &CssNumericValue,
    actual: CssNumberType,
) -> Result<(), CssTokenContractError> {
    let expected = value.required_number_type();
    if actual != expected {
        return Err(CssTokenContractError::InvalidNumberType { expected, actual });
    }
    Ok(())
}

fn validate_digits(
    value: &str,
    role: CssTokenEvidenceRole,
    allow_empty: bool,
) -> Result<(), CssTokenContractError> {
    if value.is_empty() && !allow_empty {
        return Err(CssTokenContractError::EmptyInterpretedValue { role });
    }
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CssTokenContractError::InvalidAsciiDigits { role });
    }
    Ok(())
}

fn required_value(value: &str, role: CssTokenEvidenceRole) -> Result<(), CssTokenContractError> {
    if value.is_empty() {
        return Err(CssTokenContractError::EmptyInterpretedValue { role });
    }
    Ok(())
}

fn is_raw_css_whitespace(value: char) -> bool {
    matches!(value, '\t' | '\n' | '\u{000C}' | '\r' | ' ')
}

fn same_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: CssTokenEvidenceRole,
) -> Result<(), CssTokenContractError> {
    let actual = anchor.source_id();
    if actual != expected {
        return Err(CssTokenContractError::SourceIdentityMismatch {
            role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn non_empty(
    anchor: &SourceAnchor,
    role: CssTokenEvidenceRole,
) -> Result<(), CssTokenContractError> {
    if anchor.range().is_empty() {
        return Err(CssTokenContractError::EmptySourceRange { role });
    }
    Ok(())
}

fn exact(
    anchor: &SourceAnchor,
    role: CssTokenEvidenceRole,
    expected: &'static str,
) -> Result<(), CssTokenContractError> {
    if anchor.fragment() != expected {
        return Err(CssTokenContractError::FixedSpellingMismatch { role, expected });
    }
    Ok(())
}

fn contained(
    container: &SourceAnchor,
    nested: &SourceAnchor,
    role: CssTokenEvidenceRole,
) -> Result<(), CssTokenContractError> {
    if container.source_id() != nested.source_id()
        || nested.range().start() < container.range().start()
        || nested.range().end() > container.range().end()
    {
        return Err(CssTokenContractError::EvidenceOutsideContainer { role });
    }
    Ok(())
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}
