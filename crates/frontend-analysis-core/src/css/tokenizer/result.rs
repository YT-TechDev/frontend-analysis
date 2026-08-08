use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRangeError, SourceText};

use super::super::token::{CssLexicalItem, CssTokenContractError};
use super::diagnostic::{CssTokenizerDiagnostic, CssTokenizerDiagnosticContractError};
use super::resource::{
    CssTokenizerInvalidConfiguration, CssTokenizerResourceContractError, CssTokenizerResourceKind,
    CssTokenizerResourceLimitEvidence, CssTokenizerResourceUsage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerCompletion {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTokenizerTermination {
    EndOfInput,
    ResourceLimit(CssTokenizerResourceLimitEvidence),
}

#[derive(Debug, Clone)]
pub(crate) struct CssTokenizerRunResult {
    source_id: SourceId,
    leading_bom: Option<SourceAnchor>,
    lexical_items: Vec<CssLexicalItem>,
    diagnostics: Vec<CssTokenizerDiagnostic>,
    processed_prefix: SourceAnchor,
    unprocessed_remainder: SourceAnchor,
    terminal: SourceAnchor,
    completion: CssTokenizerCompletion,
    termination: CssTokenizerTermination,
    resources: CssTokenizerResourceUsage,
}

impl CssTokenizerRunResult {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        source_text: &SourceText,
        leading_bom: Option<SourceAnchor>,
        lexical_items: Vec<CssLexicalItem>,
        diagnostics: Vec<CssTokenizerDiagnostic>,
        processed_prefix: SourceAnchor,
        unprocessed_remainder: SourceAnchor,
        terminal: SourceAnchor,
        completion: CssTokenizerCompletion,
        termination: CssTokenizerTermination,
        resources: CssTokenizerResourceUsage,
    ) -> Result<Self, CssTokenizerRunError> {
        validate_run(
            source_text,
            leading_bom.as_ref(),
            &lexical_items,
            &diagnostics,
            &processed_prefix,
            &unprocessed_remainder,
            &terminal,
            completion,
            &termination,
            resources,
        )?;

        Ok(Self {
            source_id: source_text.id(),
            leading_bom,
            lexical_items,
            diagnostics,
            processed_prefix,
            unprocessed_remainder,
            terminal,
            completion,
            termination,
            resources,
        })
    }

    pub(crate) const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub(crate) fn leading_bom(&self) -> Option<&SourceAnchor> {
        self.leading_bom.as_ref()
    }

    pub(crate) fn lexical_items(&self) -> &[CssLexicalItem] {
        &self.lexical_items
    }

    pub(crate) fn diagnostics(&self) -> &[CssTokenizerDiagnostic] {
        &self.diagnostics
    }

    pub(crate) const fn processed_prefix(&self) -> &SourceAnchor {
        &self.processed_prefix
    }

    pub(crate) const fn unprocessed_remainder(&self) -> &SourceAnchor {
        &self.unprocessed_remainder
    }

    pub(crate) const fn terminal(&self) -> &SourceAnchor {
        &self.terminal
    }

    pub(crate) const fn completion(&self) -> CssTokenizerCompletion {
        self.completion
    }

    pub(crate) const fn termination(&self) -> &CssTokenizerTermination {
        &self.termination
    }

    pub(crate) const fn resources(&self) -> CssTokenizerResourceUsage {
        self.resources
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTokenizerRunError {
    InvalidConfiguration(CssTokenizerInvalidConfiguration),
    InternalInvariantFailure(CssTokenizerInvariantViolation),
}

impl CssTokenizerRunError {
    pub(crate) fn diagnostic_contract_violation(
        index: usize,
        error: CssTokenizerDiagnosticContractError,
    ) -> Self {
        Self::InternalInvariantFailure(CssTokenizerInvariantViolation::DiagnosticContractViolation {
            index,
            error,
        })
    }
}

impl From<CssTokenizerInvalidConfiguration> for CssTokenizerRunError {
    fn from(value: CssTokenizerInvalidConfiguration) -> Self {
        Self::InvalidConfiguration(value)
    }
}

impl From<SourceRangeError> for CssTokenizerRunError {
    fn from(error: SourceRangeError) -> Self {
        Self::InternalInvariantFailure(CssTokenizerInvariantViolation::SourceRangeContractViolation {
            error,
        })
    }
}

impl From<CssTokenContractError> for CssTokenizerRunError {
    fn from(error: CssTokenContractError) -> Self {
        Self::InternalInvariantFailure(CssTokenizerInvariantViolation::LexicalContractViolation {
            error,
        })
    }
}

impl From<CssTokenizerResourceContractError> for CssTokenizerRunError {
    fn from(error: CssTokenizerResourceContractError) -> Self {
        Self::InternalInvariantFailure(CssTokenizerInvariantViolation::ResourceContractViolation {
            error,
        })
    }
}

impl fmt::Display for CssTokenizerRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS tokenizer run failure: {self:?}")
    }
}

impl Error for CssTokenizerRunError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssTokenizerRunEvidenceRole {
    LeadingBom,
    ProcessedPrefix,
    UnprocessedRemainder,
    Terminal,
    LexicalItem { index: usize },
    Diagnostic { index: usize },
    ResourceLimit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CssTokenizerInvariantViolation {
    SourceIdentityMismatch {
        role: CssTokenizerRunEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    SourceRangeContractViolation {
        error: SourceRangeError,
    },
    LexicalContractViolation {
        error: CssTokenContractError,
    },
    ResourceContractViolation {
        error: CssTokenizerResourceContractError,
    },
    MissingLeadingBomEvidence,
    UnexpectedLeadingBomEvidence,
    InvalidLeadingBomRange,
    TerminalMustBeEmpty {
        start: usize,
        end: usize,
    },
    ProcessedPrefixMismatch {
        terminal: usize,
        actual_start: usize,
        actual_end: usize,
    },
    UnprocessedRemainderMismatch {
        terminal: usize,
        source_len: usize,
        actual_start: usize,
        actual_end: usize,
    },
    CompletionTerminationMismatch,
    EndOfInputBeforeSourceEnd {
        terminal: usize,
        source_len: usize,
    },
    ResourceTerminalMismatch {
        terminal: usize,
        resource_start: usize,
        resource_end: usize,
    },
    LexicalGap {
        index: usize,
        expected_start: usize,
        actual_start: usize,
    },
    LexicalOverlapOrOutOfOrder {
        index: usize,
        expected_start: usize,
        actual_start: usize,
    },
    LexicalItemBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    DiagnosticBeyondTerminal {
        index: usize,
        end: usize,
        terminal: usize,
    },
    DiagnosticOrderViolation {
        index: usize,
    },
    DiagnosticContractViolation {
        index: usize,
        error: CssTokenizerDiagnosticContractError,
    },
    ResourceUsageCountMismatch {
        kind: CssTokenizerResourceKind,
        expected: usize,
        actual: usize,
    },
}

#[allow(clippy::too_many_arguments)]
fn validate_run(
    source_text: &SourceText,
    leading_bom: Option<&SourceAnchor>,
    lexical_items: &[CssLexicalItem],
    diagnostics: &[CssTokenizerDiagnostic],
    processed_prefix: &SourceAnchor,
    unprocessed_remainder: &SourceAnchor,
    terminal: &SourceAnchor,
    completion: CssTokenizerCompletion,
    termination: &CssTokenizerTermination,
    resources: CssTokenizerResourceUsage,
) -> Result<(), CssTokenizerRunError> {
    let expected_source = source_text.id();
    require_source(
        expected_source,
        processed_prefix,
        CssTokenizerRunEvidenceRole::ProcessedPrefix,
    )?;
    require_source(
        expected_source,
        unprocessed_remainder,
        CssTokenizerRunEvidenceRole::UnprocessedRemainder,
    )?;
    require_source(
        expected_source,
        terminal,
        CssTokenizerRunEvidenceRole::Terminal,
    )?;

    if !terminal.range().is_empty() {
        return invariant(CssTokenizerInvariantViolation::TerminalMustBeEmpty {
            start: terminal.range().start(),
            end: terminal.range().end(),
        });
    }

    let terminal_offset = terminal.range().start();
    let source_len = source_text.as_str().len();

    if processed_prefix.range().start() != 0 || processed_prefix.range().end() != terminal_offset {
        return invariant(CssTokenizerInvariantViolation::ProcessedPrefixMismatch {
            terminal: terminal_offset,
            actual_start: processed_prefix.range().start(),
            actual_end: processed_prefix.range().end(),
        });
    }
    if unprocessed_remainder.range().start() != terminal_offset
        || unprocessed_remainder.range().end() != source_len
    {
        return invariant(
            CssTokenizerInvariantViolation::UnprocessedRemainderMismatch {
                terminal: terminal_offset,
                source_len,
                actual_start: unprocessed_remainder.range().start(),
                actual_end: unprocessed_remainder.range().end(),
            },
        );
    }

    validate_bom(source_text, leading_bom)?;
    validate_completion(source_len, terminal_offset, completion, termination)?;
    validate_lexical_coverage(expected_source, leading_bom, lexical_items, terminal_offset)?;
    validate_diagnostics(expected_source, diagnostics, lexical_items, terminal_offset)?;
    validate_resource_counts(resources, lexical_items.len(), diagnostics.len())?;

    if let CssTokenizerTermination::ResourceLimit(limit) = termination {
        require_source(
            expected_source,
            limit.location(),
            CssTokenizerRunEvidenceRole::ResourceLimit,
        )?;
        if !same_anchor(terminal, limit.location()) {
            return invariant(CssTokenizerInvariantViolation::ResourceTerminalMismatch {
                terminal: terminal_offset,
                resource_start: limit.location().range().start(),
                resource_end: limit.location().range().end(),
            });
        }
    }

    Ok(())
}

fn validate_bom(
    source_text: &SourceText,
    leading_bom: Option<&SourceAnchor>,
) -> Result<(), CssTokenizerRunError> {
    let source_has_bom = source_text.as_str().starts_with('\u{feff}');
    match (source_has_bom, leading_bom) {
        (true, None) => invariant(CssTokenizerInvariantViolation::MissingLeadingBomEvidence),
        (false, Some(_)) => invariant(CssTokenizerInvariantViolation::UnexpectedLeadingBomEvidence),
        (false, None) => Ok(()),
        (true, Some(anchor)) => {
            require_source(
                source_text.id(),
                anchor,
                CssTokenizerRunEvidenceRole::LeadingBom,
            )?;
            if anchor.range().start() != 0
                || anchor.range().end() != '\u{feff}'.len_utf8()
                || anchor.fragment() != "\u{feff}"
            {
                return invariant(CssTokenizerInvariantViolation::InvalidLeadingBomRange);
            }
            Ok(())
        }
    }
}

fn validate_completion(
    source_len: usize,
    terminal_offset: usize,
    completion: CssTokenizerCompletion,
    termination: &CssTokenizerTermination,
) -> Result<(), CssTokenizerRunError> {
    match (completion, termination) {
        (CssTokenizerCompletion::Complete, CssTokenizerTermination::EndOfInput) => {
            if terminal_offset != source_len {
                return invariant(CssTokenizerInvariantViolation::EndOfInputBeforeSourceEnd {
                    terminal: terminal_offset,
                    source_len,
                });
            }
            Ok(())
        }
        (CssTokenizerCompletion::Incomplete, CssTokenizerTermination::ResourceLimit(_)) => Ok(()),
        _ => invariant(CssTokenizerInvariantViolation::CompletionTerminationMismatch),
    }
}

fn validate_lexical_coverage(
    expected_source: SourceId,
    leading_bom: Option<&SourceAnchor>,
    lexical_items: &[CssLexicalItem],
    terminal_offset: usize,
) -> Result<(), CssTokenizerRunError> {
    let mut expected_start = leading_bom.map_or(0, |bom| bom.range().end());

    for (index, item) in lexical_items.iter().enumerate() {
        let source = item.source();
        require_source(
            expected_source,
            source,
            CssTokenizerRunEvidenceRole::LexicalItem { index },
        )?;
        let actual_start = source.range().start();
        if actual_start > expected_start {
            return invariant(CssTokenizerInvariantViolation::LexicalGap {
                index,
                expected_start,
                actual_start,
            });
        }
        if actual_start < expected_start {
            return invariant(CssTokenizerInvariantViolation::LexicalOverlapOrOutOfOrder {
                index,
                expected_start,
                actual_start,
            });
        }
        if source.range().end() > terminal_offset {
            return invariant(CssTokenizerInvariantViolation::LexicalItemBeyondTerminal {
                index,
                end: source.range().end(),
                terminal: terminal_offset,
            });
        }
        expected_start = source.range().end();
    }

    if expected_start < terminal_offset {
        return invariant(CssTokenizerInvariantViolation::LexicalGap {
            index: lexical_items.len(),
            expected_start,
            actual_start: terminal_offset,
        });
    }

    Ok(())
}

fn validate_diagnostics(
    expected_source: SourceId,
    diagnostics: &[CssTokenizerDiagnostic],
    lexical_items: &[CssLexicalItem],
    terminal_offset: usize,
) -> Result<(), CssTokenizerRunError> {
    let mut previous_key = None;

    for (index, diagnostic) in diagnostics.iter().enumerate() {
        require_source(
            expected_source,
            diagnostic.location(),
            CssTokenizerRunEvidenceRole::Diagnostic { index },
        )?;
        if diagnostic.location().range().end() > terminal_offset {
            return invariant(CssTokenizerInvariantViolation::DiagnosticBeyondTerminal {
                index,
                end: diagnostic.location().range().end(),
                terminal: terminal_offset,
            });
        }

        let key = diagnostic.source_order_key();
        if previous_key.is_some_and(|previous| previous > key) {
            return invariant(CssTokenizerInvariantViolation::DiagnosticOrderViolation { index });
        }
        previous_key = Some(key);

        if let Err(error) = diagnostic.validate_subject(lexical_items) {
            return invariant(
                CssTokenizerInvariantViolation::DiagnosticContractViolation { index, error },
            );
        }
    }

    Ok(())
}

fn validate_resource_counts(
    resources: CssTokenizerResourceUsage,
    lexical_item_count: usize,
    diagnostic_count: usize,
) -> Result<(), CssTokenizerRunError> {
    let lexical_actual = resources.value(CssTokenizerResourceKind::LexicalItems);
    if lexical_actual != lexical_item_count {
        return invariant(CssTokenizerInvariantViolation::ResourceUsageCountMismatch {
            kind: CssTokenizerResourceKind::LexicalItems,
            expected: lexical_item_count,
            actual: lexical_actual,
        });
    }

    let diagnostic_actual = resources.value(CssTokenizerResourceKind::Diagnostics);
    if diagnostic_actual != diagnostic_count {
        return invariant(CssTokenizerInvariantViolation::ResourceUsageCountMismatch {
            kind: CssTokenizerResourceKind::Diagnostics,
            expected: diagnostic_count,
            actual: diagnostic_actual,
        });
    }

    Ok(())
}

fn require_source(
    expected: SourceId,
    anchor: &SourceAnchor,
    role: CssTokenizerRunEvidenceRole,
) -> Result<(), CssTokenizerRunError> {
    let actual = anchor.source_id();
    if actual != expected {
        return invariant(CssTokenizerInvariantViolation::SourceIdentityMismatch {
            role,
            expected,
            actual,
        });
    }
    Ok(())
}

fn invariant<T>(violation: CssTokenizerInvariantViolation) -> Result<T, CssTokenizerRunError> {
    Err(CssTokenizerRunError::InternalInvariantFailure(violation))
}

fn same_anchor(left: &SourceAnchor, right: &SourceAnchor) -> bool {
    left.source_id() == right.source_id() && left.range() == right.range()
}

#[cfg(test)]
mod producer_contract_tests {
    use super::*;
    use crate::css::token::{CssToken, CssTokenKind};
    use crate::css::tokenizer::diagnostic::{
        CssTokenizerDiagnosticCode, CssTokenizerDiagnosticContext, CssTokenizerDiagnosticSubject,
        CssTokenizerRecovery,
    };
    use crate::css::tokenizer::resource::{
        CssTokenizerResourceKind, CssTokenizerResourceLimitEvidence,
    };

    fn source(id: u64, text: &str) -> SourceText {
        SourceText::new(SourceId::new(id), text.to_owned())
    }

    #[test]
    fn source_range_contract_error_preserves_typed_internal_failure() {
        let source = source(1, "é");
        let error = source.anchor(1, 1).unwrap_err();

        assert_eq!(
            CssTokenizerRunError::from(error),
            CssTokenizerRunError::InternalInvariantFailure(
                CssTokenizerInvariantViolation::SourceRangeContractViolation { error }
            )
        );
    }

    #[test]
    fn lexical_contract_error_preserves_typed_internal_failure() {
        let source = source(2, "f");
        let error = CssToken::new(
            &source,
            source.anchor(0, 1).unwrap(),
            CssTokenKind::Function("f".to_owned()),
        )
        .unwrap_err();

        assert_eq!(
            CssTokenizerRunError::from(error.clone()),
            CssTokenizerRunError::InternalInvariantFailure(
                CssTokenizerInvariantViolation::LexicalContractViolation { error }
            )
        );
    }

    #[test]
    fn resource_contract_error_preserves_typed_internal_failure() {
        let source = source(3, "a");
        let error = CssTokenizerResourceLimitEvidence::new(
            &source,
            CssTokenizerResourceKind::LexicalItems,
            1,
            1,
            source.anchor(0, 0).unwrap(),
        )
        .unwrap_err();

        assert_eq!(
            CssTokenizerRunError::from(error),
            CssTokenizerRunError::InternalInvariantFailure(
                CssTokenizerInvariantViolation::ResourceContractViolation { error }
            )
        );
    }

    #[test]
    fn diagnostic_contract_error_uses_existing_indexed_invariant_path() {
        let source = source(4, "\\\n");
        let error = CssTokenizerDiagnostic::new(
            &source,
            CssTokenizerDiagnosticCode::InvalidEscape,
            source.anchor(0, 1).unwrap(),
            CssTokenizerDiagnosticContext::Url,
            CssTokenizerRecovery::EmitBadUrl,
            CssTokenizerDiagnosticSubject::InputLocation,
        )
        .unwrap_err();

        assert_eq!(
            CssTokenizerRunError::diagnostic_contract_violation(7, error),
            CssTokenizerRunError::InternalInvariantFailure(
                CssTokenizerInvariantViolation::DiagnosticContractViolation { index: 7, error }
            )
        );
    }

    #[test]
    fn malformed_css_diagnostic_remains_distinct_from_producer_failure() {
        let source = source(5, "\"");
        let diagnostic = CssTokenizerDiagnostic::new(
            &source,
            CssTokenizerDiagnosticCode::EofInString,
            source.anchor(1, 1).unwrap(),
            CssTokenizerDiagnosticContext::String,
            CssTokenizerRecovery::EmitStringAtEndOfInput,
            CssTokenizerDiagnosticSubject::InputLocation,
        )
        .unwrap();

        assert_eq!(diagnostic.code(), CssTokenizerDiagnosticCode::EofInString);
    }

    #[test]
    fn wrapped_producer_failures_do_not_disclose_retained_source() {
        const SECRET: &str = "producer-contract-secret";
        let source = source(6, SECRET);

        let source_error = source.anchor(1, usize::MAX).unwrap_err();
        let lexical_error = CssToken::new(
            &source,
            source.anchor(0, SECRET.len()).unwrap(),
            CssTokenKind::Function("secret".to_owned()),
        )
        .unwrap_err();
        let resource_error = CssTokenizerResourceLimitEvidence::new(
            &source,
            CssTokenizerResourceKind::SourceBytes,
            1,
            1,
            source.anchor(0, 0).unwrap(),
        )
        .unwrap_err();

        let errors = [
            CssTokenizerRunError::from(source_error),
            CssTokenizerRunError::from(lexical_error),
            CssTokenizerRunError::from(resource_error),
        ];

        for error in errors {
            assert!(!format!("{error:?}").contains(SECRET));
            assert!(!error.to_string().contains(SECRET));
        }
    }
}
