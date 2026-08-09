//! Parser-private lexical-index cursor over the tokenizer's existing
//! `CssLexicalItem` sequence (#139).
//!
//! The cursor never flattens or copies the semantic-token stream: it walks
//! the upstream tokenizer result's `lexical_items()` slice by index, passed
//! in on every call rather than borrowed for the cursor's own lifetime. This
//! keeps the cursor a plain, minimal, `Copy` snapshot (a single index) so the
//! producer can own the upstream `CssTokenizerRunResult` by value throughout
//! the run and still move it, unborrowed, into the final `CssParserRunResult`
//! at the end.
//!
//! Resource accounting (charging `AlgorithmSteps` per lexical-item visit,
//! including comment visits) is a producer concern, not a cursor concern:
//! the cursor exposes only a raw, unweighted one-item-at-a-time view so the
//! producer can charge exactly once per visited lexical item.

use super::super::token::{CssLexicalItem, CssToken};
use super::super::tokenizer::result::{
    CssTokenizerCompletion, CssTokenizerRunResult, CssTokenizerTermination,
};

/// The raw next lexical item at the cursor's current position: a
/// grammar-invisible comment, a semantic token, true end of input, or the
/// upstream tokenizer's non-EndOfInput terminal (resource limit or other
/// incompleteness).
///
/// `TrueEndOfInput` and `UpstreamTokenizerTerminal` are structurally
/// distinct: this enum can only report `TrueEndOfInput` when the upstream
/// tokenizer itself completed at true `EndOfInput`, so a tokenizer-incomplete
/// terminal can never be mistaken for ordinary EOF by any caller.
pub(crate) enum CssParserRawPosition<'t> {
    Comment,
    SemanticToken {
        lexical_index: usize,
        token: &'t CssToken,
    },
    TrueEndOfInput,
    UpstreamTokenizerTerminal,
}

/// A parser-private, opaque cursor position over `CssLexicalItem`s.
///
/// Deliberately minimal (a single lexical index) so it is cheap to snapshot
/// and restore as part of the producer's transactional checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssParserCursor {
    lexical_index: usize,
}

impl CssParserCursor {
    pub(crate) const fn new() -> Self {
        Self { lexical_index: 0 }
    }

    /// Returns the underlying lexical index this cursor currently observes.
    pub(crate) const fn lexical_index(&self) -> usize {
        self.lexical_index
    }

    /// Snapshots the current position. The snapshot is a plain `Copy` value;
    /// restoring it is the cursor's only rollback primitive.
    pub(crate) const fn checkpoint(&self) -> Self {
        *self
    }

    /// Restores a previously taken snapshot.
    pub(crate) fn restore(&mut self, checkpoint: Self) {
        *self = checkpoint;
    }

    /// Looks at the lexical item at the current index without advancing.
    pub(crate) fn peek_raw<'t>(
        &self,
        upstream: &'t CssTokenizerRunResult,
    ) -> CssParserRawPosition<'t> {
        match upstream.lexical_items().get(self.lexical_index) {
            Some(CssLexicalItem::Comment(_)) => CssParserRawPosition::Comment,
            Some(CssLexicalItem::SemanticToken(token)) => CssParserRawPosition::SemanticToken {
                lexical_index: self.lexical_index,
                token,
            },
            None if upstream_ended_at_true_eof(upstream) => CssParserRawPosition::TrueEndOfInput,
            None => CssParserRawPosition::UpstreamTokenizerTerminal,
        }
    }

    /// Advances past exactly the lexical item last observed via `peek_raw`
    /// at the current index (comment or semantic token). No arbitrary index
    /// assignment is exposed; this is the only forward-movement primitive.
    pub(crate) fn advance(&mut self) {
        self.lexical_index += 1;
    }
}

fn upstream_ended_at_true_eof(upstream: &CssTokenizerRunResult) -> bool {
    upstream.completion() == CssTokenizerCompletion::Complete
        && matches!(upstream.termination(), CssTokenizerTermination::EndOfInput)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::token::{CssCommentEvidence, CssCommentTermination, CssToken, CssTokenKind};
    use crate::css::tokenizer::resource::CssTokenizerResourceUsage;
    use crate::css::tokenizer::result::{CssTokenizerCompletion, CssTokenizerTermination};
    use crate::{SourceId, SourceText};

    fn source(text: &str) -> SourceText {
        SourceText::new(SourceId::new(1), text.to_owned())
    }

    fn complete_run(source: &SourceText, items: Vec<CssLexicalItem>) -> CssTokenizerRunResult {
        let len = source.as_str().len();
        let item_count = items.len();
        CssTokenizerRunResult::new(
            source,
            None,
            items,
            Vec::new(),
            source.anchor(0, len).unwrap(),
            source.anchor(len, len).unwrap(),
            source.anchor(len, len).unwrap(),
            CssTokenizerCompletion::Complete,
            CssTokenizerTermination::EndOfInput,
            CssTokenizerResourceUsage::new(len, 1, item_count, 0, 0, 0),
        )
        .unwrap()
    }

    fn incomplete_run(
        source: &SourceText,
        items: Vec<CssLexicalItem>,
        terminal: usize,
    ) -> CssTokenizerRunResult {
        use crate::css::tokenizer::resource::{
            CssTokenizerResourceKind, CssTokenizerResourceLimitEvidence,
        };
        let len = source.as_str().len();
        let item_count = items.len();
        let evidence = CssTokenizerResourceLimitEvidence::new(
            source,
            CssTokenizerResourceKind::AlgorithmSteps,
            1,
            2,
            source.anchor(terminal, terminal).unwrap(),
        )
        .unwrap();
        CssTokenizerRunResult::new(
            source,
            None,
            items,
            Vec::new(),
            source.anchor(0, terminal).unwrap(),
            source.anchor(terminal, len).unwrap(),
            source.anchor(terminal, terminal).unwrap(),
            CssTokenizerCompletion::Incomplete,
            CssTokenizerTermination::ResourceLimit(evidence),
            CssTokenizerResourceUsage::new(len, 1, item_count, 0, 0, 0),
        )
        .unwrap()
    }

    fn ident_item(source: &SourceText, start: usize, end: usize) -> CssLexicalItem {
        CssLexicalItem::SemanticToken(
            CssToken::new(
                source,
                source.anchor(start, end).unwrap(),
                CssTokenKind::Ident(source.anchor(start, end).unwrap().fragment().to_owned()),
            )
            .unwrap(),
        )
    }

    fn comment_item(source: &SourceText, start: usize, end: usize) -> CssLexicalItem {
        CssLexicalItem::Comment(
            CssCommentEvidence::new(
                source,
                source.anchor(start, end).unwrap(),
                source.anchor(start, start + 2).unwrap(),
                CssCommentTermination::Closed {
                    close_delimiter: source.anchor(end - 2, end).unwrap(),
                },
            )
            .unwrap(),
        )
    }

    #[test]
    fn peek_raw_skips_nothing_and_reports_comment_distinctly() {
        let text = source("/**/a");
        let upstream = complete_run(
            &text,
            vec![comment_item(&text, 0, 4), ident_item(&text, 4, 5)],
        );
        let cursor = CssParserCursor::new();
        assert!(matches!(
            cursor.peek_raw(&upstream),
            CssParserRawPosition::Comment
        ));
    }

    #[test]
    fn advancing_past_a_comment_reaches_the_semantic_token_with_its_lexical_index() {
        let text = source("/**/a");
        let upstream = complete_run(
            &text,
            vec![comment_item(&text, 0, 4), ident_item(&text, 4, 5)],
        );
        let mut cursor = CssParserCursor::new();
        cursor.advance();
        match cursor.peek_raw(&upstream) {
            CssParserRawPosition::SemanticToken { lexical_index, .. } => {
                assert_eq!(lexical_index, 1);
            }
            _ => panic!("expected semantic token"),
        }
    }

    #[test]
    fn true_end_of_input_is_reported_only_when_upstream_completed() {
        let text = source("a");
        let upstream = complete_run(&text, vec![ident_item(&text, 0, 1)]);
        let mut cursor = CssParserCursor::new();
        cursor.advance();
        assert!(matches!(
            cursor.peek_raw(&upstream),
            CssParserRawPosition::TrueEndOfInput
        ));
    }

    #[test]
    fn upstream_tokenizer_terminal_is_distinguished_from_true_eof() {
        let text = source("a");
        let upstream = incomplete_run(&text, vec![ident_item(&text, 0, 1)], 1);
        let mut cursor = CssParserCursor::new();
        cursor.advance();
        assert!(matches!(
            cursor.peek_raw(&upstream),
            CssParserRawPosition::UpstreamTokenizerTerminal
        ));
    }

    #[test]
    fn checkpoint_and_restore_round_trip_the_exact_lexical_index() {
        let mut cursor = CssParserCursor::new();
        cursor.advance();
        cursor.advance();
        let checkpoint = cursor.checkpoint();
        cursor.advance();
        cursor.advance();
        assert_eq!(cursor.lexical_index(), 4);
        cursor.restore(checkpoint);
        assert_eq!(cursor.lexical_index(), 2);
    }

    #[test]
    fn new_cursor_starts_at_index_zero() {
        assert_eq!(CssParserCursor::new().lexical_index(), 0);
    }
}
