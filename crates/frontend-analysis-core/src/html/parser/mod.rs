//! First source-backed HTML analysis-parser capability.
//!
//! Projects a validated [`HtmlTokenizerRunResult`] into explicit authored
//! start-tag occurrences, per the model approved in #114. This is a
//! capability-specific projection, not tree construction: matching, nesting,
//! and synthesized structure remain out of scope for this slice.
//!
//! [`analyze_explicit_start_tags`] performs exactly one traversal of
//! [`HtmlTokenizerRunResult::tokens`]. Every occurrence is constructed
//! directly from the single token being visited during that traversal, so
//! the relationships #114 requires to hold between an occurrence and its
//! originating token are established by construction rather than by a
//! second pass that re-fetches and re-compares tokenizer evidence:
//!
//! - the occurrence's `origin_token_index` is the current enumerated index;
//! - the visited token is known to be `HtmlToken::Tag` because that is what
//!   was just pattern-matched;
//! - the visited tag's kind is known to be `Start` because that is what was
//!   just checked;
//! - `complete` and `raw_name` are literal clones of that same tag's own
//!   `complete()` and `name().source()`, so they cannot diverge from it;
//! - `raw_name` remains contained in `complete` transitively, through the
//!   containment [`HtmlTagToken::new`](super::token::HtmlTagToken::new)
//!   already enforces for every validated tokenizer token;
//! - origin indexes strictly increase, because they are read off a forward
//!   `enumerate()` over the token slice and an occurrence is only appended
//!   while visiting the token it describes.
//!
//! The one relationship not fixed purely by local control flow is the final
//! occurrence-vector inventory (no start tag missing, no extra entry); that
//! is still established by construction (one occurrence is appended for
//! every, and only every, encountered `Start` tag), but is additionally
//! proven with two counters accumulated during the same traversal, compared
//! once after the loop, with no second scan of the token slice.

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
///
/// Under the corrected single-pass construction (see the module
/// documentation), every relationship except final inventory is established
/// by construction and cannot independently diverge, so this vocabulary
/// intentionally no longer contains the wrong-token-kind,
/// wrong-origin-index, and evidence-mismatch variants an earlier two-pass
/// design used: those checks compared an occurrence against a token
/// re-fetched by index in a second pass, and that second pass no longer
/// exists. `OccurrenceInventoryMismatch` remains as an explicit,
/// zero-extra-traversal proof that one occurrence was appended for every,
/// and only every, encountered `Start` tag; it is not reachable through any
/// current call path, since both counters it compares are always advanced
/// together, but it protects that guarantee against a future edit to the
/// traversal that decouples them, without requiring a second token scan to
/// check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlAnalysisParserContractError {
    OccurrenceInventoryMismatch { expected: usize, actual: usize },
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
/// performs exactly one traversal of its tokens, in order. Every
/// [`HtmlTagKind::Start`] tag projects one occurrence, constructed directly
/// from the tag being visited; end tags, character data, and EOF are
/// consumed as validated input but not projected. No source rescan, second
/// token-slice scan, endpoint reconstruction, or token replay occurs. Parser
/// result completeness never exceeds the retained tokenizer run's
/// completeness, since it derives entirely from tokens already present in
/// that run.
pub(crate) fn analyze_explicit_start_tags(
    tokenizer_run: HtmlTokenizerRunResult,
) -> Result<HtmlExplicitStartTagAnalysis, HtmlAnalysisParserContractError> {
    let mut occurrences = Vec::new();
    let mut start_tag_count: usize = 0;

    for (origin_token_index, token) in tokenizer_run.tokens().iter().enumerate() {
        let HtmlToken::Tag(tag) = token else {
            continue;
        };
        if tag.kind() != HtmlTagKind::Start {
            continue;
        }

        start_tag_count += 1;
        occurrences.push(HtmlExplicitStartTagOccurrence {
            origin_token_index,
            complete: tag.complete().clone(),
            raw_name: tag.name().source().clone(),
        });
    }

    if start_tag_count != occurrences.len() {
        return Err(
            HtmlAnalysisParserContractError::OccurrenceInventoryMismatch {
                expected: start_tag_count,
                actual: occurrences.len(),
            },
        );
    }

    Ok(HtmlExplicitStartTagAnalysis {
        tokenizer_run,
        occurrences,
    })
}
