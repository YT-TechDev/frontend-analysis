//! Known-valid-UTF-8 input cursor over `SourceText`.
//!
//! Implements the #109 on-the-fly input-unit model: CR/CRLF normalize to one
//! interpreted LF unit while retaining authored raw ranges, a leading BOM is
//! skipped once before Data-state tokenization, and EOF is a conceptual unit
//! at the validated empty end-of-source range. The cursor is the single
//! forward owner of input position; it never searches or rescans.

use crate::SourceText;

use super::super::diagnostic::HtmlTokenizerDiagnosticCode;

/// One preprocessing-observed input unit: an interpreted scalar or EOF.
///
/// Deliberately does not derive `Debug`: `Scalar` carries an authored source
/// character, and no production consumer requires formatting it. #111
/// treats source text as potentially sensitive and prohibits accidental
/// source-content exposure through newly introduced Debug/Display surfaces.
#[derive(Clone, Copy)]
pub(super) enum InputUnit {
    Scalar { ch: char, start: usize, end: usize },
    Eof { at: usize },
}

impl InputUnit {
    pub(super) fn start(self) -> usize {
        match self {
            Self::Scalar { start, .. } => start,
            Self::Eof { at } => at,
        }
    }

    pub(super) fn end(self) -> usize {
        match self {
            Self::Scalar { end, .. } => end,
            Self::Eof { at } => at,
        }
    }
}

pub(super) struct Cursor<'a> {
    source: &'a SourceText,
    next_raw_offset: usize,
}

impl<'a> Cursor<'a> {
    /// Creates a cursor positioned after skipping one leading UTF-8 BOM, if
    /// present, returning the BOM's exact raw range as preprocessing
    /// evidence.
    pub(super) fn new(source: &'a SourceText) -> (Self, Option<(usize, usize)>) {
        const BOM: char = '\u{feff}';
        let text = source.as_str();
        if text.starts_with(BOM) {
            let len = BOM.len_utf8();
            (
                Self {
                    source,
                    next_raw_offset: len,
                },
                Some((0, len)),
            )
        } else {
            (
                Self {
                    source,
                    next_raw_offset: 0,
                },
                None,
            )
        }
    }

    /// A bounded, borrowed, strictly non-committing view of raw source bytes
    /// the authoritative cursor has not consumed yet.
    ///
    /// This is the only observation TC-S10's Named Character Reference
    /// maximum match needs beyond the already-materialized current unit. It
    /// is pure: it advances no position, materializes no input unit, performs
    /// no CR/CRLF normalization, produces no preprocessing observation, and
    /// retains nothing. It therefore cannot become a second source authority
    /// — every byte it reveals is still consumed afterwards through
    /// [`Self::advance`] like any other input.
    ///
    /// Raw bytes, not `&str`, so a `max_len` cut inside a multi-byte scalar
    /// is representable without slicing on a non-boundary. The only consumer
    /// matches ASCII alphanumerics and `;`, which are never continuation
    /// bytes, so a truncated trailing scalar can never extend a match.
    pub(super) fn unconsumed_bytes(&self, max_len: usize) -> &'a [u8] {
        let text = self.source.as_str().as_bytes();
        let start = self.next_raw_offset.min(text.len());
        let end = start.saturating_add(max_len).min(text.len());
        &text[start..end]
    }

    /// Decodes exactly one input unit at the current raw offset and advances
    /// with checked movement. Returns the unit together with an optional
    /// preprocessing diagnostic code discovered while materializing it.
    pub(super) fn advance(&mut self) -> (InputUnit, Option<HtmlTokenizerDiagnosticCode>) {
        let text = self.source.as_str();
        let start = self.next_raw_offset;
        if start >= text.len() {
            return (InputUnit::Eof { at: start }, None);
        }

        let rest = &text[start..];
        let ch = rest.chars().next().expect("non-empty remaining source");
        let raw_len = ch.len_utf8();

        if ch == '\r' {
            let after_cr = start
                .checked_add(raw_len)
                .expect("checked cursor advance within source bounds");
            let ends_crlf = text[after_cr..].starts_with('\n');
            let end = if ends_crlf {
                after_cr
                    .checked_add(1)
                    .expect("checked cursor advance within source bounds")
            } else {
                after_cr
            };
            self.next_raw_offset = end;
            return (
                InputUnit::Scalar {
                    ch: '\n',
                    start,
                    end,
                },
                None,
            );
        }

        let end = start
            .checked_add(raw_len)
            .expect("checked cursor advance within source bounds");
        self.next_raw_offset = end;
        let diagnostic = preprocessing_diagnostic_for(ch);
        (InputUnit::Scalar { ch, start, end }, diagnostic)
    }
}

/// Classifies a freshly materialized scalar for input-stream preprocessing
/// diagnostics. NUL is excluded: it is flagged by the current tokenizer
/// state, not preprocessing. ASCII whitespace (tab, LF, FF, CR-as-LF) is
/// excluded from the control-character set per the pinned standard.
fn preprocessing_diagnostic_for(ch: char) -> Option<HtmlTokenizerDiagnosticCode> {
    let cp = ch as u32;
    if is_flagged_control_character(cp) {
        Some(HtmlTokenizerDiagnosticCode::ControlCharacterInInputStream)
    } else if is_noncharacter(cp) {
        Some(HtmlTokenizerDiagnosticCode::NoncharacterInInputStream)
    } else {
        None
    }
}

fn is_flagged_control_character(cp: u32) -> bool {
    matches!(cp, 0x01..=0x08 | 0x0B | 0x0E..=0x1F | 0x7F..=0x9F)
}

fn is_noncharacter(cp: u32) -> bool {
    (0xFDD0..=0xFDEF).contains(&cp) || matches!(cp & 0xFFFF, 0xFFFE | 0xFFFF)
}
