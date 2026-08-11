//! Monotonic lexical-window contract for selector qualification (#182).
//!
//! A future qualifier consumes the tokenizer evidence already retained by the
//! same parser run. This helper identifies the lexical slice contained by a
//! parser-owned qualified-rule header without searching or retokenizing raw
//! source and without duplicating the full token stream.

use std::error::Error;
use std::fmt;
use std::ops::Range;

use crate::{SourceAnchor, SourceId};

use super::super::token::CssLexicalItem;
use super::super::tokenizer::result::CssTokenizerRunResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssSelectorLexicalWindowError {
    HeaderSourceIdentityMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    HeaderSourceContentMismatch {
        source_id: SourceId,
    },
    NonMonotonicHeader {
        previous_start: usize,
        actual_start: usize,
    },
    LexicalItemSourceIdentityMismatch {
        index: usize,
        expected: SourceId,
        actual: SourceId,
    },
    HeaderCutsLexicalItem {
        index: usize,
        item_start: usize,
        item_end: usize,
        header_start: usize,
        header_end: usize,
    },
}

impl fmt::Display for CssSelectorLexicalWindowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "CSS selector lexical-window violation: {self:?}")
    }
}

impl Error for CssSelectorLexicalWindowError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CssSelectorLexicalWindow {
    indices: Range<usize>,
}

impl CssSelectorLexicalWindow {
    pub(crate) fn indices(&self) -> Range<usize> {
        self.indices.clone()
    }

    pub(crate) fn items<'a>(
        &self,
        tokenizer_result: &'a CssTokenizerRunResult,
    ) -> &'a [CssLexicalItem] {
        &tokenizer_result.lexical_items()[self.indices.clone()]
    }
}

pub(crate) struct CssSelectorLexicalWindowCursor<'a> {
    tokenizer_result: &'a CssTokenizerRunResult,
    next_index: usize,
    previous_header_start: Option<usize>,
}

impl<'a> CssSelectorLexicalWindowCursor<'a> {
    pub(crate) const fn new(tokenizer_result: &'a CssTokenizerRunResult) -> Self {
        Self {
            tokenizer_result,
            next_index: 0,
            previous_header_start: None,
        }
    }

    /// Returns the exact lexical-item range contained by `header`.
    ///
    /// Headers must be requested in nondecreasing source order. Items before a
    /// later header are consumed monotonically; no full-vector rescan occurs.
    pub(crate) fn window_for(
        &mut self,
        header: &SourceAnchor,
    ) -> Result<CssSelectorLexicalWindow, CssSelectorLexicalWindowError> {
        let expected = self.tokenizer_result.source_id();
        let actual = header.source_id();
        if actual != expected {
            return Err(
                CssSelectorLexicalWindowError::HeaderSourceIdentityMismatch { expected, actual },
            );
        }
        if !self
            .tokenizer_result
            .processed_prefix()
            .retains_exact_source(header)
        {
            return Err(CssSelectorLexicalWindowError::HeaderSourceContentMismatch {
                source_id: expected,
            });
        }

        if let Some(previous_start) = self.previous_header_start
            && header.range().start() < previous_start
        {
            return Err(CssSelectorLexicalWindowError::NonMonotonicHeader {
                previous_start,
                actual_start: header.range().start(),
            });
        }
        self.previous_header_start = Some(header.range().start());

        let items = self.tokenizer_result.lexical_items();
        while self.next_index < items.len() {
            let item = &items[self.next_index];
            validate_item_source(item, self.next_index, expected)?;
            let range = item.source().range();
            if range.end() <= header.range().start() {
                self.next_index += 1;
                continue;
            }
            if range.start() < header.range().start() {
                return Err(cut_error(
                    self.next_index,
                    range.start(),
                    range.end(),
                    header,
                ));
            }
            break;
        }

        let start = self.next_index;
        let mut end = start;
        while end < items.len() {
            let item = &items[end];
            validate_item_source(item, end, expected)?;
            let range = item.source().range();
            if range.start() >= header.range().end() {
                break;
            }
            if range.end() > header.range().end() {
                return Err(cut_error(end, range.start(), range.end(), header));
            }
            end += 1;
        }

        self.next_index = end;
        Ok(CssSelectorLexicalWindow {
            indices: start..end,
        })
    }
}

fn validate_item_source(
    item: &CssLexicalItem,
    index: usize,
    expected: SourceId,
) -> Result<(), CssSelectorLexicalWindowError> {
    let actual = item.source().source_id();
    if actual != expected {
        return Err(
            CssSelectorLexicalWindowError::LexicalItemSourceIdentityMismatch {
                index,
                expected,
                actual,
            },
        );
    }
    Ok(())
}

fn cut_error(
    index: usize,
    item_start: usize,
    item_end: usize,
    header: &SourceAnchor,
) -> CssSelectorLexicalWindowError {
    CssSelectorLexicalWindowError::HeaderCutsLexicalItem {
        index,
        item_start,
        item_end,
        header_start: header.range().start(),
        header_end: header.range().end(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::token::CssLexicalItem;
    use crate::css::tokenizer::producer::run;
    use crate::css::tokenizer::resource::CssTokenizerLimits;
    use crate::{SourceId, SourceText};

    fn limits() -> CssTokenizerLimits {
        CssTokenizerLimits::new(4096, 20_000, 4096, 256, 4096, 4096).unwrap()
    }

    #[test]
    fn window_reuses_retained_comment_whitespace_and_semantic_token_evidence() {
        let source = SourceText::new(SourceId::new(10), "a/*c*/ > b{}".to_owned());
        let result = run(&source, limits()).unwrap();
        let header_end = source.as_str().find('{').unwrap();
        let header = source.anchor(0, header_end).unwrap();
        let mut cursor = CssSelectorLexicalWindowCursor::new(&result);
        let window = cursor.window_for(&header).unwrap();
        let items = window.items(&result);

        assert!(!items.is_empty());
        assert!(
            items
                .iter()
                .any(|item| matches!(item, CssLexicalItem::Comment(_)))
        );
        assert!(items.iter().all(|item| {
            item.source().range().start() >= header.range().start()
                && item.source().range().end() <= header.range().end()
        }));
    }

    #[test]
    fn later_header_advances_monotonically_without_restarting_at_zero() {
        let source = SourceText::new(SourceId::new(11), "a{} b{}".to_owned());
        let result = run(&source, limits()).unwrap();
        let mut cursor = CssSelectorLexicalWindowCursor::new(&result);
        let first = cursor.window_for(&source.anchor(0, 1).unwrap()).unwrap();
        let second = cursor.window_for(&source.anchor(4, 5).unwrap()).unwrap();

        assert!(first.indices().end <= second.indices().start);
    }

    #[test]
    fn header_with_same_source_id_but_different_retained_bytes_is_rejected() {
        let source = SourceText::new(SourceId::new(12), "a{}".to_owned());
        let different = SourceText::new(SourceId::new(12), "b{}".to_owned());
        let result = run(&source, limits()).unwrap();
        let mut cursor = CssSelectorLexicalWindowCursor::new(&result);

        assert!(matches!(
            cursor.window_for(&different.anchor(0, 1).unwrap()),
            Err(CssSelectorLexicalWindowError::HeaderSourceContentMismatch { .. })
        ));
    }

    #[test]
    fn header_from_another_source_is_rejected() {
        let source = SourceText::new(SourceId::new(12), "a{}".to_owned());
        let foreign = SourceText::new(SourceId::new(13), "a{}".to_owned());
        let result = run(&source, limits()).unwrap();
        let mut cursor = CssSelectorLexicalWindowCursor::new(&result);

        assert!(matches!(
            cursor.window_for(&foreign.anchor(0, 1).unwrap()),
            Err(CssSelectorLexicalWindowError::HeaderSourceIdentityMismatch { .. })
        ));
    }
}
