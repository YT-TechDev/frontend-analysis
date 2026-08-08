use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

use super::super::token::CssLexicalItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerDiagnosticCode {
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
pub(crate) enum CssTokenizerDiagnosticContext {
    TokenStart,
    Comment,
    String,
    Url,
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerRecovery {
    EmitDelimiter,
    PreserveUnterminatedComment,
    EmitStringAtEndOfInput,
    EmitBadStringAndReconsumeNewline,
    EmitUrlAtEndOfInput,
    EmitBadUrl,
    SubstituteReplacementCharacterForEofEscape,
}

#[derive(Clone)]
pub(crate) enum CssTokenizerDiagnosticSubject {
    InputLocation,
    LexicalItem { index: usize },
    InputRegion { region: SourceAnchor },
}

impl fmt::Debug for CssTokenizerDiagnosticSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLocation => formatter.write_str("InputLocation"),
            Self::LexicalItem { index } => formatter
                .debug_struct("LexicalItem")
                .field("index", index)
                .finish(),
            Self::InputRegion { region } => formatter
                .debug_struct("InputRegion")
                .field("source_id", &region.source_id())
                .field("range", &region.range())
                .finish(),
        }
    }
}

impl PartialEq for CssTokenizerDiagnosticSubject {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InputLocation, Self::InputLocation) => true,
            (Self::LexicalItem { index: left }, Self::LexicalItem { index: right }) => {
                left == right
            }
            (Self::InputRegion { region: left }, Self::InputRegion { region: right }) => {
                same_anchor(left, right)
            }
            _ => false,
        }
    }
}

impl Eq for CssTokenizerDiagnosticSubject {}

#[derive(Clone)]
pub(crate) struct CssTokenizerDiagnostic {
    code: CssTokenizerDiagnosticCode,
    location: SourceAnchor,
    context: CssTokenizerDiagnosticContext,
    recovery: CssTokenizerRecovery,
    subject: CssTokenizerDiagnosticSubject,
}

impl CssTokenizerDiagnostic {
    pub(crate) fn new(
        source_text: &SourceText,
        code: CssTokenizerDiagnosticCode,
        location: SourceAnchor,
        context: CssTokenizerDiagnosticContext,
        recovery: CssTokenizerRecovery,
        subject: CssTokenizerDiagnosticSubject,
    ) -> Result<Self, CssTokenizerDiagnosticContractError> {
        require_source(
            source_text.id(),
            &location,
            CssTokenizerDiagnosticEvidenceRole::Location,
        )?;

        if let CssTokenizerDiagnosticSubject::InputRegion { region } = &subject {
            require_source(
                source_text.id(),
                region,
                CssTokenizerDiagnosticEvidenceRole::InputRegion,
            )?;
            if !contains_anchor(region, &location) {
                return Err(CssTokenizerDiagnosticContractError::LocationOutsideInputRegion);
            }
        }

        let (expected_context, expected_recovery) = expected_handling(code);
        if context != expected_context || recovery != expected_recovery {
            return Err(CssTokenizerDiagnosticContractError::InvalidHandling {
                code,
                expected_context,
                actual_context: context,
                expected_recovery,
                actual_recovery: recovery,
            });
        }

        Ok(Self {
            code,
            location,
            context,
            recovery,
            subject,
        })
    }

    pub(crate) const fn code(&self) -> CssTokenizerDiagnosticCode {
        self.code
    }

    pub(crate) const fn location(&self) -> &SourceAnchor {
        &self.location
    }

    pub(crate) const fn context(&self) -> CssTokenizerDiagnosticContext {
        self.context
    }

    pub(crate) const fn recovery(&self) -> CssTokenizerRecovery {
        self.recovery
    }

    pub(crate) const fn subject(&self) -> &CssTokenizerDiagnosticSubject {
        &self.subject
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        (self.location.range().start(), self.location.range().end())
    }

    pub(crate) fn validate_subject(
        &self,
        lexical_items: &[CssLexicalItem],
    ) -> Result<(), CssTokenizerDiagnosticContractError> {
        match &self.subject {
            CssTokenizerDiagnosticSubject::InputLocation => Ok(()),
            CssTokenizerDiagnosticSubject::LexicalItem { index } => {
                let Some(item) = lexical_items.get(*index) else {
                    return Err(
                        CssTokenizerDiagnosticContractError::LexicalItemSubjectOutOfBounds {
                            index: *index,
                            item_count: lexical_items.len(),
                        },
                    );
                };
                if item.source().source_id() != self.location.source_id() {
                    return Err(CssTokenizerDiagnosticContractError::SubjectSourceMismatch {
                        expected: self.location.source_id(),
                        actual: item.source().source_id(),
                    });
                }
                if !contains_anchor(item.source(), &self.location) {
                    return Err(
                        CssTokenizerDiagnosticContractError::LocationOutsideLexicalItem {
                            index: *index,
                        },
                    );
                }
                Ok(())
            }
            CssTokenizerDiagnosticSubject::InputRegion { region } => {
                if !contains_anchor(region, &self.location) {
                    return Err(CssTokenizerDiagnosticContractError::LocationOutsideInputRegion);
                }
                Ok(())
            }
        }
    }
}

impl fmt::Debug for CssTokenizerDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssTokenizerDiagnostic")
            .field("code", &self.code)
            .field("source_id", &self.location.source_id())
            .field("location", &self.location.range())
            .field("context", &self.context)
            .field("recovery", &self.recovery)
            .field("subject", &self.subject)
            .finish()
    }
}

impl PartialEq for CssTokenizerDiagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
            && same_anchor(&self.location, &other.location)
            && self.context == other.context
            && self.recovery == other.recovery
            && self.subject == other.subject
    }
}

impl Eq for CssTokenizerDiagnostic {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerDiagnosticEvidenceRole {
    Location,
    InputRegion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerDiagnosticContractError {
    SourceIdentityMismatch {
        role: CssTokenizerDiagnosticEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    InvalidHandling {
        code: CssTokenizerDiagnosticCode,
        expected_context: CssTokenizerDiagnosticContext,
        actual_context: CssTokenizerDiagnosticContext,
        expected_recovery: CssTokenizerRecovery,
        actual_recovery: CssTokenizerRecovery,
    },
    LexicalItemSubjectOutOfBounds {
        index: usize,
        item_count: usize,
    },
    SubjectSourceMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    LocationOutsideLexicalItem {
        index: usize,
    },
    LocationOutsideInputRegion,
}

impl fmt::Display for CssTokenizerDiagnosticContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS tokenizer diagnostic contract violation: {self:?}"
        )
    }
}

impl Error for CssTokenizerDiagnosticContractError {}

fn expected_handling(
    code: CssTokenizerDiagnosticCode,
) -> (CssTokenizerDiagnosticContext, CssTokenizerRecovery) {
    match code {
        CssTokenizerDiagnosticCode::InvalidEscape => (
            CssTokenizerDiagnosticContext::TokenStart,
            CssTokenizerRecovery::EmitDelimiter,
        ),
        CssTokenizerDiagnosticCode::EofInComment => (
            CssTokenizerDiagnosticContext::Comment,
            CssTokenizerRecovery::PreserveUnterminatedComment,
        ),
        CssTokenizerDiagnosticCode::EofInString => (
            CssTokenizerDiagnosticContext::String,
            CssTokenizerRecovery::EmitStringAtEndOfInput,
        ),
        CssTokenizerDiagnosticCode::NewlineInString => (
            CssTokenizerDiagnosticContext::String,
            CssTokenizerRecovery::EmitBadStringAndReconsumeNewline,
        ),
        CssTokenizerDiagnosticCode::EofInUrl => (
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitUrlAtEndOfInput,
        ),
        CssTokenizerDiagnosticCode::InvalidUrlCodePoint
        | CssTokenizerDiagnosticCode::InvalidUrlEscape => (
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitBadUrl,
        ),
        CssTokenizerDiagnosticCode::EofInEscape => (
            CssTokenizerDiagnosticContext::Escape,
            CssTokenizerRecovery::SubstituteReplacementCharacterForEofEscape,
        ),
    }
}

fn require_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: CssTokenizerDiagnosticEvidenceRole,
) -> Result<(), CssTokenizerDiagnosticContractError> {
    let actual = anchor.source_id();
    if actual != expected {
        return Err(
            CssTokenizerDiagnosticContractError::SourceIdentityMismatch {
                role,
                expected,
                actual,
            },
        );
    }
    Ok(())
}

fn contains_anchor(container: &SourceAnchor, nested: &SourceAnchor) -> bool {
    container.source_id() == nested.source_id()
        && nested.range().start() >= container.range().start()
        && nested.range().end() <= container.range().end()
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}
