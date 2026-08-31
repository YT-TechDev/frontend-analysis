//! Core-integrated CSS selector qualification operation (#184).
//!
//! This is the normal production entry path for `CoreV1`: structural evidence
//! is first produced and reconciled by the existing `css::analysis` boundary,
//! then the exact returned parser result is moved into the selector stage.

use std::error::Error;
use std::fmt;

use crate::SourceText;

use super::super::analysis::{CssAnalysisError, analyze_css_source};
use super::super::parser::resource::CssParserLimits;
use super::super::tokenizer::resource::CssTokenizerLimits;
use super::producer::{CssSelectorProducerError, run as run_selector};
use super::resource::CssSelectorLimits;
use super::result::CssSelectorQualificationRunResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssSelectorAnalysisError {
    Structural(CssAnalysisError),
    Selector(CssSelectorProducerError),
}

impl From<CssAnalysisError> for CssSelectorAnalysisError {
    fn from(error: CssAnalysisError) -> Self {
        Self::Structural(error)
    }
}

impl From<CssSelectorProducerError> for CssSelectorAnalysisError {
    fn from(error: CssSelectorProducerError) -> Self {
        Self::Selector(error)
    }
}

impl fmt::Display for CssSelectorAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS selector analysis failure: {self:?}")
    }
}

impl Error for CssSelectorAnalysisError {}

/// Runs the existing tokenizer/parser/Core source-evidence operation and then
/// executes bounded `CoreV1` selector qualification over the exact validated
/// parser result.
pub(crate) fn analyze_css_selectors(
    source: &SourceText,
    tokenizer_limits: CssTokenizerLimits,
    parser_limits: CssParserLimits,
    selector_limits: CssSelectorLimits,
) -> Result<CssSelectorQualificationRunResult, CssSelectorAnalysisError> {
    let parser_result = analyze_css_source(source, tokenizer_limits, parser_limits)?;
    run_selector(source, parser_result, selector_limits).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::selector::result::CssSelectorQualificationOutcome;
    use crate::{SourceId, SourceText};

    fn tokenizer_limits() -> CssTokenizerLimits {
        CssTokenizerLimits::new(4096, 100_000, 8192, 1024, 8192, 8192).unwrap()
    }

    fn parser_limits() -> CssParserLimits {
        CssParserLimits::new(100_000, 256, 256, 8192, 1024, 1024, 1024, 1024, 8192).unwrap()
    }

    #[test]
    fn core_entry_returns_selector_result_owning_the_structural_run() {
        let source = SourceText::new(SourceId::new(1), "a{p:v;}".to_owned());
        let result = analyze_css_selectors(
            &source,
            tokenizer_limits(),
            parser_limits(),
            CssSelectorLimits::new(100_000, 64, 1024, 65_536).unwrap(),
        )
        .unwrap();

        assert_eq!(result.upstream_parser_result().context_records().len(), 1);
        assert!(matches!(
            result.observations()[0].outcome(),
            CssSelectorQualificationOutcome::QualifiedBySelectedGrammar
        ));
    }
}
