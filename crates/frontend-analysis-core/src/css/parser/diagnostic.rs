//! Parser-owned diagnostic vocabulary (#138).
//!
//! Deliberately finite: tokenizer lexical diagnostics remain owned by the
//! retained `CssTokenizerRunResult` and are never duplicated here. No
//! selector, property, descriptor, CSSOM, or cascade diagnostics exist in
//! this first slice.

use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CssParserDiagnosticCode {
    /// A top-level item could not be interpreted as a supported qualified
    /// rule.
    InvalidStylesheetQualifiedRule,
    /// A supported-context block item failed both declaration interpretation
    /// and qualified-rule fallback. Durable only after both fail.
    InvalidBlockItem,
}

#[derive(Clone)]
pub(crate) struct CssParserDiagnostic {
    code: CssParserDiagnosticCode,
    location: SourceAnchor,
}

impl CssParserDiagnostic {
    pub(crate) fn new(
        source_text: &SourceText,
        code: CssParserDiagnosticCode,
        location: SourceAnchor,
    ) -> Result<Self, CssParserDiagnosticContractError> {
        let expected = source_text.id();
        let actual = location.source_id();
        if actual != expected {
            return Err(CssParserDiagnosticContractError::SourceIdentityMismatch {
                expected,
                actual,
            });
        }
        if location.range().is_empty() {
            return Err(CssParserDiagnosticContractError::EmptyLocation);
        }

        Ok(Self { code, location })
    }

    pub(crate) const fn code(&self) -> CssParserDiagnosticCode {
        self.code
    }

    pub(crate) const fn location(&self) -> &SourceAnchor {
        &self.location
    }

    pub(crate) fn source_order_key(&self) -> (usize, usize) {
        (self.location.range().start(), self.location.range().end())
    }
}

impl PartialEq for CssParserDiagnostic {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && same_anchor(&self.location, &other.location)
    }
}

impl Eq for CssParserDiagnostic {}

impl fmt::Debug for CssParserDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CssParserDiagnostic")
            .field("code", &self.code)
            .field("source_id", &self.location.source_id())
            .field("location", &self.location.range())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssParserDiagnosticContractError {
    SourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    EmptyLocation,
}

impl fmt::Display for CssParserDiagnosticContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS parser diagnostic contract violation: {self:?}"
        )
    }
}

impl Error for CssParserDiagnosticContractError {}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(id: u64, text: &str) -> SourceText {
        SourceText::new(SourceId::new(id), text.to_owned())
    }

    #[test]
    fn diagnostic_constructs_with_non_empty_same_source_location() {
        let text = source(1, "a{@x}");
        let diagnostic = CssParserDiagnostic::new(
            &text,
            CssParserDiagnosticCode::InvalidBlockItem,
            text.anchor(2, 4).unwrap(),
        )
        .unwrap();
        assert_eq!(diagnostic.code(), CssParserDiagnosticCode::InvalidBlockItem);
        assert_eq!(diagnostic.source_order_key(), (2, 4));
    }

    #[test]
    fn empty_location_is_rejected() {
        let text = source(2, "a{}");
        let result = CssParserDiagnostic::new(
            &text,
            CssParserDiagnosticCode::InvalidStylesheetQualifiedRule,
            text.anchor(2, 2).unwrap(),
        );
        assert_eq!(result, Err(CssParserDiagnosticContractError::EmptyLocation));
    }

    #[test]
    fn cross_source_location_is_rejected() {
        let text = source(3, "a{}");
        let other = source(4, "a{}");
        let result = CssParserDiagnostic::new(
            &text,
            CssParserDiagnosticCode::InvalidBlockItem,
            other.anchor(0, 1).unwrap(),
        );
        assert!(matches!(
            result,
            Err(CssParserDiagnosticContractError::SourceIdentityMismatch { .. })
        ));
    }

    #[test]
    fn debug_output_does_not_disclose_authored_source() {
        const SECRET: &str = "top-secret-invalid-region";
        let text = source(5, SECRET);
        let diagnostic = CssParserDiagnostic::new(
            &text,
            CssParserDiagnosticCode::InvalidBlockItem,
            text.anchor(0, SECRET.len()).unwrap(),
        )
        .unwrap();
        assert!(!format!("{diagnostic:?}").contains(SECRET));
    }
}
