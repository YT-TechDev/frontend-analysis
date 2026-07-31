use std::error::Error;
use std::fmt;
use std::rc::Rc;

use crate::RawSourceCoordinate;

/// A caller-supplied, browser-independent source identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceId(u64);

impl SourceId {
    /// Creates an identity from the caller's value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the caller-supplied value.
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A validated half-open range of UTF-8 byte offsets.
///
/// Empty ranges are valid when their shared offset is an in-bounds character
/// boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    start: usize,
    end: usize,
}

impl SourceRange {
    /// Returns the inclusive starting byte offset.
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive ending byte offset.
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns whether the range selects no bytes.
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

struct SourceData {
    id: SourceId,
    text: String,
}

/// Immutable, Core-owned UTF-8 source text.
///
/// Clones and anchors privately share immutable source storage. The selected
/// ownership model is deliberately single-threaded and creates no `Send` or
/// `Sync` guarantee.
#[derive(Clone)]
pub struct SourceText {
    inner: Rc<SourceData>,
}

impl SourceText {
    /// Takes ownership of the exact string under a caller-supplied identity.
    pub fn new(id: SourceId, text: String) -> Self {
        Self {
            inner: Rc::new(SourceData { id, text }),
        }
    }

    /// Returns the caller-supplied source identity.
    pub fn id(&self) -> SourceId {
        self.inner.id
    }

    /// Borrows the exact, unmodified source text.
    pub fn as_str(&self) -> &str {
        &self.inner.text
    }

    /// Validates a half-open UTF-8 byte range and creates an owned anchor.
    ///
    /// Reversal takes precedence over bounds, followed by the start character
    /// boundary and then the end character boundary. Valid empty ranges are
    /// accepted.
    pub fn anchor(&self, start: usize, end: usize) -> Result<SourceAnchor, SourceRangeError> {
        let source_len = self.inner.text.len();
        if start > end {
            return Err(SourceRangeError::ReversedRange { start, end });
        }
        if end > source_len {
            return Err(SourceRangeError::OutOfBounds {
                start,
                end,
                source_len,
            });
        }
        if !self.inner.text.is_char_boundary(start) {
            return Err(SourceRangeError::InvalidStartBoundary { start, end });
        }
        if !self.inner.text.is_char_boundary(end) {
            return Err(SourceRangeError::InvalidEndBoundary { start, end });
        }

        Ok(SourceAnchor {
            source: Rc::clone(&self.inner),
            range: SourceRange { start, end },
        })
    }
}

impl fmt::Debug for SourceText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceText")
            .field("id", &self.inner.id)
            .field("byte_len", &self.inner.text.len())
            .finish()
    }
}

/// An owned reference to an exact fragment of immutable source text.
///
/// The anchor retains its source independently of the originating handle. Its
/// private shared ownership model is deliberately single-threaded and creates
/// no `Send` or `Sync` guarantee.
#[derive(Clone)]
pub struct SourceAnchor {
    source: Rc<SourceData>,
    range: SourceRange,
}

impl SourceAnchor {
    /// Returns the identity of the retained source.
    pub fn source_id(&self) -> SourceId {
        self.source.id
    }

    /// Returns the validated half-open UTF-8 byte range.
    pub fn range(&self) -> SourceRange {
        self.range
    }

    /// Borrows the exact retained substring without allocating or normalizing.
    pub fn fragment(&self) -> &str {
        &self.source.text[self.range.start..self.range.end]
    }

    /// Projects the validated inclusive start endpoint to a raw coordinate.
    ///
    /// The authoritative byte offset and the derived line index and UTF-8 byte
    /// column are zero-based and grammar-neutral. The returned value is
    /// detached from the source text; this anchor remains the retained source
    /// evidence.
    pub fn start_coordinate(&self) -> RawSourceCoordinate {
        RawSourceCoordinate::from_validated_endpoint(
            self.source.id,
            &self.source.text,
            self.range.start,
        )
    }

    /// Projects the validated exclusive end endpoint to a raw coordinate.
    ///
    /// The authoritative byte offset and the derived line index and UTF-8 byte
    /// column are zero-based and grammar-neutral. The returned value is
    /// detached from the source text; this anchor remains the retained source
    /// evidence.
    pub fn end_coordinate(&self) -> RawSourceCoordinate {
        RawSourceCoordinate::from_validated_endpoint(
            self.source.id,
            &self.source.text,
            self.range.end,
        )
    }
}

impl fmt::Debug for SourceAnchor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SourceAnchor")
            .field("source_id", &self.source.id)
            .field("range", &self.range)
            .finish()
    }
}

/// A deterministic failure to validate a requested source range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceRangeError {
    /// The starting offset is greater than the ending offset.
    ReversedRange {
        /// The requested starting byte offset.
        start: usize,
        /// The requested ending byte offset.
        end: usize,
    },
    /// The ending offset exceeds the source byte length.
    OutOfBounds {
        /// The requested starting byte offset.
        start: usize,
        /// The requested ending byte offset.
        end: usize,
        /// The byte length of the source.
        source_len: usize,
    },
    /// The starting offset is not a UTF-8 character boundary.
    InvalidStartBoundary {
        /// The requested starting byte offset.
        start: usize,
        /// The requested ending byte offset.
        end: usize,
    },
    /// The ending offset is not a UTF-8 character boundary.
    InvalidEndBoundary {
        /// The requested starting byte offset.
        start: usize,
        /// The requested ending byte offset.
        end: usize,
    },
}

impl fmt::Display for SourceRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReversedRange { start, end } => {
                write!(formatter, "source range start {start} exceeds end {end}")
            }
            Self::OutOfBounds {
                start,
                end,
                source_len,
            } => write!(
                formatter,
                "source range [{start}, {end}) exceeds source byte length {source_len}"
            ),
            Self::InvalidStartBoundary { start, end } => write!(
                formatter,
                "source range [{start}, {end}) starts inside a UTF-8 character"
            ),
            Self::InvalidEndBoundary { start, end } => write!(
                formatter,
                "source range [{start}, {end}) ends inside a UTF-8 character"
            ),
        }
    }
}

impl Error for SourceRangeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_identity_preserves_caller_value() {
        let first = SourceId::new(42);
        assert_eq!(first.value(), 42);
        assert_eq!(first, SourceId::new(42));
        assert_ne!(first, SourceId::new(43));
    }

    #[test]
    fn source_text_preserves_exact_input() {
        let exact = " \u{feff}line\r\né e\u{301} \n".to_owned();
        let source = SourceText::new(SourceId::new(1), exact.clone());
        assert_eq!(source.as_str(), exact);
        assert!(source.as_str().contains('\u{feff}'));
        assert!(source.as_str().contains("\r\n"));
        assert!(source.as_str().contains("é e\u{301}"));
        assert!(source.as_str().starts_with(' '));
        assert!(source.as_str().ends_with('\n'));
    }

    #[test]
    fn ascii_ranges_include_valid_empty_ranges() {
        let source = SourceText::new(SourceId::new(1), "abc".to_owned());
        assert_eq!(source.anchor(0, 3).unwrap().fragment(), "abc");
        assert_eq!(source.anchor(1, 2).unwrap().fragment(), "b");

        let start_empty = source.anchor(0, 0).unwrap();
        assert!(start_empty.range().is_empty());
        assert_eq!(start_empty.fragment(), "");

        let end_empty = source.anchor(3, 3).unwrap();
        assert!(end_empty.range().is_empty());
        assert_eq!(end_empty.fragment(), "");
    }

    #[test]
    fn multibyte_ranges_require_character_boundaries() {
        let source = SourceText::new(SourceId::new(1), "aéz".to_owned());
        assert_eq!(source.anchor(1, 3).unwrap().fragment(), "é");
        assert_eq!(source.anchor(3, 4).unwrap().fragment(), "z");
        assert_eq!(
            source.anchor(2, 3).unwrap_err(),
            SourceRangeError::InvalidStartBoundary { start: 2, end: 3 }
        );
        assert_eq!(
            source.anchor(1, 2).unwrap_err(),
            SourceRangeError::InvalidEndBoundary { start: 1, end: 2 }
        );
        assert_eq!(source.anchor(3, 3).unwrap().fragment(), "");
        assert_eq!(
            source.anchor(2, 2).unwrap_err(),
            SourceRangeError::InvalidStartBoundary { start: 2, end: 2 }
        );
    }

    #[test]
    fn validation_errors_follow_required_precedence() {
        let source = SourceText::new(SourceId::new(1), "aéé".to_owned());
        assert_eq!(
            source.anchor(8, 7).unwrap_err(),
            SourceRangeError::ReversedRange { start: 8, end: 7 }
        );
        assert_eq!(
            source.anchor(2, 8).unwrap_err(),
            SourceRangeError::OutOfBounds {
                start: 2,
                end: 8,
                source_len: 5,
            }
        );
        assert_eq!(
            source.anchor(2, 4).unwrap_err(),
            SourceRangeError::InvalidStartBoundary { start: 2, end: 4 }
        );
        assert_eq!(
            source.anchor(1, 2).unwrap_err(),
            SourceRangeError::InvalidEndBoundary { start: 1, end: 2 }
        );
    }

    #[test]
    fn anchor_owns_source_independently_of_original_handles() {
        let caller_text = String::from("retained");
        let source = SourceText::new(SourceId::new(7), caller_text);
        let anchor = source.anchor(0, 8).unwrap();
        drop(source);

        assert_eq!(anchor.source_id(), SourceId::new(7));
        assert_eq!(anchor.fragment(), "retained");
    }

    #[test]
    fn clones_and_anchors_share_source_storage() {
        let source = SourceText::new(SourceId::new(1), "shared".to_owned());
        let source_clone = source.clone();
        let first = source.anchor(0, 1).unwrap();
        let second = source.anchor(1, 2).unwrap();
        let first_clone = first.clone();

        assert!(Rc::ptr_eq(&source.inner, &source_clone.inner));
        assert!(Rc::ptr_eq(&source.inner, &first.source));
        assert!(Rc::ptr_eq(&first.source, &second.source));
        assert!(Rc::ptr_eq(&first.source, &first_clone.source));
    }

    #[test]
    fn repeated_requests_are_deterministic() {
        let source = SourceText::new(SourceId::new(1), "aéz".to_owned());
        let first = source.anchor(1, 3).unwrap();
        let second = source.anchor(1, 3).unwrap();
        assert_eq!(first.fragment(), second.fragment());
        assert_eq!(first.range(), second.range());
        assert_eq!(
            source.anchor(2, 3).unwrap_err(),
            source.anchor(2, 3).unwrap_err()
        );
    }

    #[test]
    fn debug_and_errors_do_not_expose_source_content() {
        const SECRET: &str = "complete-source-secret";
        let source = SourceText::new(SourceId::new(9), SECRET.to_owned());
        let anchor = source.anchor(0, 8).unwrap();
        let error = source.anchor(2, usize::MAX).unwrap_err();

        let source_debug = format!("{source:?}");
        assert!(source_debug.contains("SourceId(9)"));
        assert!(source_debug.contains("byte_len"));
        assert!(!source_debug.contains(SECRET));

        let anchor_debug = format!("{anchor:?}");
        assert!(anchor_debug.contains("SourceId(9)"));
        assert!(anchor_debug.contains("SourceRange"));
        assert!(!anchor_debug.contains(anchor.fragment()));

        assert!(!format!("{error:?}").contains(SECRET));
        assert!(!error.to_string().contains(SECRET));

        fn accepts_error(_: &dyn Error) {}
        accepts_error(&error);
    }
}
