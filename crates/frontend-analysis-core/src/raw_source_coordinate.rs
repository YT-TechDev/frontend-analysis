use crate::SourceId;

/// A raw, grammar-neutral projection of a validated source endpoint.
///
/// The byte offset is authoritative. The line index and UTF-8 byte column are
/// zero-based values derived from the exact source bytes. This copied numeric
/// evidence is detached from source text; [`crate::SourceAnchor`] remains the
/// retained source evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawSourceCoordinate {
    source_id: SourceId,
    byte_offset: usize,
    line_index: usize,
    byte_column: usize,
}

impl RawSourceCoordinate {
    pub(crate) fn from_validated_endpoint(
        source_id: SourceId,
        source: &str,
        byte_offset: usize,
    ) -> Self {
        let (line_index, byte_column) = scan(source.as_bytes(), byte_offset);
        Self {
            source_id,
            byte_offset,
            line_index,
            byte_column,
        }
    }

    /// Returns the identity of the source from which the endpoint was projected.
    pub fn source_id(self) -> SourceId {
        self.source_id
    }

    /// Returns the authoritative zero-based UTF-8 byte offset.
    pub fn byte_offset(self) -> usize {
        self.byte_offset
    }

    /// Returns the derived zero-based raw line index.
    pub fn line_index(self) -> usize {
        self.line_index
    }

    /// Returns the zero-based number of UTF-8 bytes from the raw line start.
    pub fn byte_column(self) -> usize {
        self.byte_column
    }
}

fn scan(source: &[u8], byte_offset: usize) -> (usize, usize) {
    let mut position = 0;
    let mut line_index = 0;
    let mut byte_column = 0;

    while position < byte_offset {
        match source[position] {
            b'\r' if source.get(position + 1) == Some(&b'\n') => {
                if position + 1 < byte_offset {
                    position += 2;
                    line_index += 1;
                    byte_column = 0;
                } else {
                    position += 1;
                    byte_column += 1;
                }
            }
            b'\r' | b'\n' => {
                position += 1;
                line_index += 1;
                byte_column = 0;
            }
            _ => {
                position += 1;
                byte_column += 1;
            }
        }
    }

    (line_index, byte_column)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceText;

    fn assert_table(source: &str, expected: &[(usize, usize, usize)]) {
        for &(byte_offset, line_index, byte_column) in expected {
            let coordinate =
                RawSourceCoordinate::from_validated_endpoint(SourceId::new(1), source, byte_offset);
            assert_eq!(coordinate.byte_offset(), byte_offset);
            assert_eq!(coordinate.line_index(), line_index);
            assert_eq!(coordinate.byte_column(), byte_column);
        }
    }

    #[test]
    fn required_coordinate_tables_are_exact() {
        assert_table("", &[(0, 0, 0)]);
        assert_table("a", &[(0, 0, 0), (1, 0, 1)]);
        assert_table("\n", &[(0, 0, 0), (1, 1, 0)]);
        assert_table("\r", &[(0, 0, 0), (1, 1, 0)]);
        assert_table("\r\n", &[(0, 0, 0), (1, 0, 1), (2, 1, 0)]);
        assert_table(
            "a\r\nb",
            &[(0, 0, 0), (1, 0, 1), (2, 0, 2), (3, 1, 0), (4, 1, 1)],
        );
        assert_table(
            "\r\r\n\n",
            &[(0, 0, 0), (1, 1, 0), (2, 1, 1), (3, 2, 0), (4, 3, 0)],
        );
    }

    #[test]
    fn multibyte_characters_advance_columns_by_utf8_bytes() {
        assert_table("éあ😀", &[(0, 0, 0), (2, 0, 2), (5, 0, 5), (9, 0, 9)]);
    }

    #[test]
    fn only_approved_raw_newlines_increment_the_line() {
        for content in ['\u{000c}', '\u{000b}', '\u{0085}', '\u{2028}', '\u{2029}'] {
            let source = content.to_string();
            assert_table(&source, &[(source.len(), 0, source.len())]);
        }
    }

    #[test]
    fn endpoint_projections_preserve_identity_offsets_and_empty_anchors() {
        let empty_source = SourceText::new(SourceId::new(41), String::new());
        let empty_source_anchor = empty_source.anchor(0, 0).unwrap();
        assert_eq!(
            empty_source_anchor.start_coordinate(),
            RawSourceCoordinate::from_validated_endpoint(SourceId::new(41), "", 0)
        );
        assert_eq!(
            empty_source_anchor.start_coordinate(),
            empty_source_anchor.end_coordinate()
        );

        let source = SourceText::new(SourceId::new(42), "a\r\nb\n\r".to_owned());
        let anchor = source.anchor(1, 3).unwrap();
        let start = anchor.start_coordinate();
        let end = anchor.end_coordinate();

        assert_eq!(start.source_id(), SourceId::new(42));
        assert_eq!(start.byte_offset(), 1);
        assert_eq!((start.line_index(), start.byte_column()), (0, 1));
        assert_eq!(end.source_id(), SourceId::new(42));
        assert_eq!(end.byte_offset(), 3);
        assert_eq!((end.line_index(), end.byte_column()), (1, 0));

        for offset in [0, 1, 3, 5, 6] {
            let empty = source.anchor(offset, offset).unwrap();
            assert_eq!(empty.start_coordinate(), empty.end_coordinate());
        }
    }

    #[test]
    fn trailing_newline_coordinates_are_exact() {
        assert_table("text\n", &[(4, 0, 4), (5, 1, 0)]);
        assert_table("text\r", &[(4, 0, 4), (5, 1, 0)]);
    }

    #[test]
    fn repeated_copy_and_clone_projections_are_exact() {
        let source = SourceText::new(SourceId::new(7), "a\r\nb".to_owned());
        let anchor = source.anchor(3, 4).unwrap();
        let coordinate = anchor.start_coordinate();
        let copied = coordinate;
        #[allow(clippy::clone_on_copy)]
        let cloned = coordinate.clone();

        assert_eq!(coordinate, anchor.start_coordinate());
        assert_eq!(copied, coordinate);
        assert_eq!(cloned, coordinate);
    }

    #[test]
    fn coordinate_is_detached_from_source_and_anchor_handles() {
        let coordinate = {
            let source = SourceText::new(SourceId::new(8), "retained".to_owned());
            let anchor = source.anchor(2, 5).unwrap();
            anchor.start_coordinate()
        };

        assert_eq!(coordinate.source_id(), SourceId::new(8));
        assert_eq!(coordinate.byte_offset(), 2);
        assert_eq!((coordinate.line_index(), coordinate.byte_column()), (0, 2));
    }

    #[test]
    fn debug_exposes_only_coordinate_evidence() {
        const SECRET: &str = "complete-source-secret";
        let source = SourceText::new(SourceId::new(9), SECRET.to_owned());
        let anchor = source.anchor(3, 9).unwrap();
        let debug = format!("{:?}", anchor.start_coordinate());

        assert!(debug.contains("SourceId(9)"));
        assert!(debug.contains("byte_offset"));
        assert!(debug.contains("line_index"));
        assert!(debug.contains("byte_column"));
        assert!(!debug.contains(SECRET));
        assert!(!debug.contains(anchor.fragment()));
    }
}
