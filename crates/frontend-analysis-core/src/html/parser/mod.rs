//! First source-backed HTML analysis-parser capability.
//!
//! Projects a validated [`HtmlTokenizerRunResult`] into explicit authored
//! start-tag occurrences, per the model approved in #114. This is a
//! capability-specific projection, not tree construction: matching, nesting,
//! and synthesized structure remain out of scope for this slice.

use std::error::Error;
use std::fmt;

use crate::SourceAnchor;

use super::token::{HtmlTagKind, HtmlToken};
use super::tokenizer::result::HtmlTokenizerRunResult;

#[cfg(test)]
mod tests;

/// One explicit authored start-tag occurrence recognized in the retained
/// source, traceable to the validated tokenizer token that produced it.
#[derive(Clone)]
pub(crate) struct HtmlExplicitStartTagOccurrence {
    origin_token_index: usize,
    complete: SourceAnchor,
    raw_name: SourceAnchor,
}

impl HtmlExplicitStartTagOccurrence {
    /// Internal provenance/traceability only; not a stable external
    /// identifier. Semantic authored identity is the retained source
    /// identity plus [`Self::complete`]'s range.
    pub(crate) fn origin_token_index(&self) -> usize {
        self.origin_token_index
    }

    /// The complete authored start-tag range, cloned from the originating
    /// tokenizer token's [`HtmlTagToken::complete`](super::token::HtmlTagToken::complete).
    pub(crate) fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    /// The exact raw tag-name range and authored spelling, cloned from the
    /// originating tokenizer token's name evidence. Never the interpreted
    /// (e.g. lowercased) spelling.
    pub(crate) fn raw_name(&self) -> &SourceAnchor {
        &self.raw_name
    }
}

impl fmt::Debug for HtmlExplicitStartTagOccurrence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlExplicitStartTagOccurrence")
            .field("origin_token_index", &self.origin_token_index)
            .field("source_id", &self.complete.source_id())
            .field("complete_range", &self.complete.range())
            .field("raw_name_range", &self.raw_name.range())
            .finish()
    }
}

/// The result of projecting a validated tokenizer run into explicit authored
/// start-tag occurrences. Retains the tokenizer run by value so tokenizer
/// diagnostics, coverage, completion, and resource evidence remain available
/// without duplication.
pub(crate) struct HtmlExplicitStartTagAnalysis {
    tokenizer_run: HtmlTokenizerRunResult,
    occurrences: Vec<HtmlExplicitStartTagOccurrence>,
}

impl HtmlExplicitStartTagAnalysis {
    fn new(
        tokenizer_run: HtmlTokenizerRunResult,
        occurrences: Vec<HtmlExplicitStartTagOccurrence>,
    ) -> Result<Self, HtmlAnalysisParserContractError> {
        validate_occurrences(&tokenizer_run, &occurrences)?;
        Ok(Self {
            tokenizer_run,
            occurrences,
        })
    }

    /// The retained validated tokenizer run. Tokenizer diagnostics,
    /// coverage, completion, and resource evidence remain authoritative
    /// here rather than being re-encoded into a parser-specific duplicate.
    pub(crate) fn tokenizer_run(&self) -> &HtmlTokenizerRunResult {
        &self.tokenizer_run
    }

    /// Explicit authored start-tag occurrences, in deterministic
    /// tokenizer/source order.
    pub(crate) fn occurrences(&self) -> &[HtmlExplicitStartTagOccurrence] {
        &self.occurrences
    }
}

impl fmt::Debug for HtmlExplicitStartTagAnalysis {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlExplicitStartTagAnalysis")
            .field("tokenizer_run", &self.tokenizer_run)
            .field("occurrence_count", &self.occurrences.len())
            .finish()
    }
}

/// A violated relationship between projected occurrences and the validated
/// tokenizer result that produced them. Distinct from tokenizer diagnostics:
/// this is a parser-boundary invariant failure, not tokenizer-observed
/// input evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlAnalysisParserContractError {
    InvalidOriginTokenIndex {
        occurrence_index: usize,
        origin_token_index: usize,
    },
    OriginTokenNotTag {
        occurrence_index: usize,
        origin_token_index: usize,
    },
    OriginTokenNotStartTag {
        occurrence_index: usize,
        origin_token_index: usize,
    },
    CompleteEvidenceMismatch {
        occurrence_index: usize,
        origin_token_index: usize,
    },
    RawNameEvidenceMismatch {
        occurrence_index: usize,
        origin_token_index: usize,
    },
    OccurrenceOrderViolation {
        occurrence_index: usize,
    },
    OccurrenceInventoryMismatch {
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for HtmlAnalysisParserContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "HTML analysis parser contract violation: {self:?}"
        )
    }
}

impl Error for HtmlAnalysisParserContractError {}

/// Projects a validated tokenizer run into explicit authored start-tag
/// occurrences.
///
/// Consumes one already-validated [`HtmlTokenizerRunResult`] by value and
/// iterates its tokens exactly once, in order. Every [`HtmlTagKind::Start`]
/// tag projects one occurrence; end tags, character data, and EOF are
/// consumed as validated input but not projected. No source rescan,
/// endpoint reconstruction, or token replay occurs. Parser result
/// completeness never exceeds the retained tokenizer run's completeness,
/// since it derives entirely from tokens already present in that run.
pub(crate) fn analyze_explicit_start_tags(
    tokenizer_run: HtmlTokenizerRunResult,
) -> Result<HtmlExplicitStartTagAnalysis, HtmlAnalysisParserContractError> {
    let occurrences = tokenizer_run
        .tokens()
        .iter()
        .enumerate()
        .filter_map(|(origin_token_index, token)| match token {
            HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::Start => {
                Some(HtmlExplicitStartTagOccurrence {
                    origin_token_index,
                    complete: tag.complete().clone(),
                    raw_name: tag.name().source().clone(),
                })
            }
            _ => None,
        })
        .collect();

    HtmlExplicitStartTagAnalysis::new(tokenizer_run, occurrences)
}

fn validate_occurrences(
    tokenizer_run: &HtmlTokenizerRunResult,
    occurrences: &[HtmlExplicitStartTagOccurrence],
) -> Result<(), HtmlAnalysisParserContractError> {
    let tokens = tokenizer_run.tokens();
    let expected_count = tokens
        .iter()
        .filter(|token| matches!(token, HtmlToken::Tag(tag) if tag.kind() == HtmlTagKind::Start))
        .count();
    if occurrences.len() != expected_count {
        return Err(
            HtmlAnalysisParserContractError::OccurrenceInventoryMismatch {
                expected: expected_count,
                actual: occurrences.len(),
            },
        );
    }

    let mut previous_origin_token_index: Option<usize> = None;
    for (occurrence_index, occurrence) in occurrences.iter().enumerate() {
        let Some(token) = tokens.get(occurrence.origin_token_index) else {
            return Err(HtmlAnalysisParserContractError::InvalidOriginTokenIndex {
                occurrence_index,
                origin_token_index: occurrence.origin_token_index,
            });
        };
        let HtmlToken::Tag(tag) = token else {
            return Err(HtmlAnalysisParserContractError::OriginTokenNotTag {
                occurrence_index,
                origin_token_index: occurrence.origin_token_index,
            });
        };
        if tag.kind() != HtmlTagKind::Start {
            return Err(HtmlAnalysisParserContractError::OriginTokenNotStartTag {
                occurrence_index,
                origin_token_index: occurrence.origin_token_index,
            });
        }
        if tag.complete().source_id() != occurrence.complete.source_id()
            || tag.complete().range() != occurrence.complete.range()
        {
            return Err(HtmlAnalysisParserContractError::CompleteEvidenceMismatch {
                occurrence_index,
                origin_token_index: occurrence.origin_token_index,
            });
        }
        if tag.name().source().source_id() != occurrence.raw_name.source_id()
            || tag.name().source().range() != occurrence.raw_name.range()
        {
            return Err(HtmlAnalysisParserContractError::RawNameEvidenceMismatch {
                occurrence_index,
                origin_token_index: occurrence.origin_token_index,
            });
        }
        // Containment of raw_name within complete is not re-checked here: it
        // is guaranteed transitively, since occurrence.complete now equals
        // tag.complete() exactly, occurrence.raw_name now equals
        // tag.name().source() exactly, and HtmlTagToken::new already
        // enforces that a tag's name is contained within its own complete
        // range for every validated tokenizer token.
        if let Some(previous) = previous_origin_token_index
            && occurrence.origin_token_index <= previous
        {
            return Err(HtmlAnalysisParserContractError::OccurrenceOrderViolation {
                occurrence_index,
            });
        }
        previous_origin_token_index = Some(occurrence.origin_token_index);
    }
    Ok(())
}
