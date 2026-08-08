//! Lazy, tokenizer-owned CSS input preprocessing cursor.
//!
//! Implements the CSS Syntax Module Level 3 §3.3 preprocessing profile
//! selected by #132: CRLF/CR/FF map to a semantic LF and NUL maps to
//! U+FFFD, while every semantic unit retains its exact raw UTF-8 byte
//! range. No normalized string or O(n) mapping table is constructed.

/// One preprocessed semantic input unit: a semantic scalar plus the exact
/// raw UTF-8 byte range that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssInputUnit {
    semantic: char,
    raw_start: usize,
    raw_end: usize,
}

impl CssInputUnit {
    pub(crate) const fn semantic(self) -> char {
        self.semantic
    }

    pub(crate) const fn raw_start(self) -> usize {
        self.raw_start
    }

    pub(crate) const fn raw_end(self) -> usize {
        self.raw_end
    }
}

/// A checked, lazy preprocessing cursor over retained UTF-8 source.
///
/// Cheap to copy: it holds only a borrowed slice and small position state,
/// never a normalized copy of the source. Lookahead performed through
/// [`CssInputCursor::peek_at`] or by copying the cursor never advances the
/// original; only [`CssInputCursor::consume`] commits movement.
#[derive(Clone, Copy)]
pub(crate) struct CssInputCursor<'a> {
    source: &'a str,
    raw_pos: usize,
    reconsume: Option<CssInputUnit>,
}

impl<'a> CssInputCursor<'a> {
    /// Creates a cursor starting at `start`, which must be a valid UTF-8
    /// boundary (the caller establishes this via any leading-BOM skip).
    pub(crate) const fn new(source: &'a str, start: usize) -> Self {
        Self {
            source,
            raw_pos: start,
            reconsume: None,
        }
    }

    /// Returns the raw byte offset of the next unit `consume` would return.
    pub(crate) fn raw_position(&self) -> usize {
        match self.reconsume {
            Some(unit) => unit.raw_start,
            None => self.raw_pos,
        }
    }

    /// Consumes and returns the next semantic input unit, or `None` at EOF.
    pub(crate) fn consume(&mut self) -> Option<CssInputUnit> {
        if let Some(unit) = self.reconsume.take() {
            self.raw_pos = unit.raw_end;
            return Some(unit);
        }
        self.consume_fresh()
    }

    fn consume_fresh(&mut self) -> Option<CssInputUnit> {
        let rest = self.source.get(self.raw_pos..)?;
        let first = rest.chars().next()?;
        let start = self.raw_pos;
        let first_len = first.len_utf8();

        let unit = match first {
            '\r' => {
                let after = start + first_len;
                if self.source[after..].starts_with('\n') {
                    CssInputUnit {
                        semantic: '\n',
                        raw_start: start,
                        raw_end: after + 1,
                    }
                } else {
                    CssInputUnit {
                        semantic: '\n',
                        raw_start: start,
                        raw_end: after,
                    }
                }
            }
            '\u{000C}' => CssInputUnit {
                semantic: '\n',
                raw_start: start,
                raw_end: start + first_len,
            },
            '\0' => CssInputUnit {
                semantic: '\u{FFFD}',
                raw_start: start,
                raw_end: start + first_len,
            },
            other => CssInputUnit {
                semantic: other,
                raw_start: start,
                raw_end: start + first_len,
            },
        };

        self.raw_pos = unit.raw_end;
        Some(unit)
    }

    /// Restores exactly one previously consumed unit for re-consumption.
    ///
    /// Only one pending reconsume unit may exist at a time; the tokenizer
    /// never needs more, matching the approved input architecture.
    pub(crate) fn reconsume_unit(&mut self, unit: CssInputUnit) {
        self.reconsume = Some(unit);
    }

    /// Bounded, non-committing lookahead: `ahead == 0` is the code point
    /// `consume` would next return, `ahead == 1` the one after, and so on.
    ///
    /// Implemented by stepping a copy of the cursor; the original cursor is
    /// never mutated.
    pub(crate) fn peek_at(&self, ahead: usize) -> Option<char> {
        let mut probe = *self;
        let mut result = None;
        for _ in 0..=ahead {
            result = probe.consume().map(CssInputUnit::semantic);
        }
        result
    }
}
