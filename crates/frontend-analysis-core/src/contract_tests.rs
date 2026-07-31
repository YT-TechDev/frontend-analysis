use crate::{
    RawSourceCoordinate, SourceAnchor, SourceId, SourceRange, SourceRangeError, SourceText,
};

fn assert_coordinate_traits<T>()
where
    T: std::fmt::Debug + Clone + Copy + PartialEq + Eq,
{
}

fn coordinate_at(source: &SourceText, offset: usize) -> RawSourceCoordinate {
    source.anchor(offset, offset).unwrap().start_coordinate()
}

fn assert_coordinate_table(source_text: &str, expected: &[(usize, usize, usize)]) {
    let source = SourceText::new(SourceId::new(100), source_text.to_owned());
    for &(byte_offset, line_index, byte_column) in expected {
        let coordinate = coordinate_at(&source, byte_offset);
        assert_eq!(coordinate.source_id(), source.id());
        assert_eq!(coordinate.byte_offset(), byte_offset);
        assert_eq!(coordinate.line_index(), line_index);
        assert_eq!(coordinate.byte_column(), byte_column);
    }
}

#[test]
fn public_ascii_contract_preserves_identity_text_and_ranges() {
    let source = SourceText::new(SourceId::new(41), String::from("abc"));
    assert_eq!(source.id().value(), 41);
    assert_eq!(source.as_str(), "abc");

    for (start, end, expected) in [(0, 3, "abc"), (1, 2, "b")] {
        let anchor = source.anchor(start, end).unwrap();
        assert_eq!(anchor.range().start(), start);
        assert_eq!(anchor.range().end(), end);
        assert!(!anchor.range().is_empty());
        assert_eq!(anchor.fragment(), expected);
    }
}

#[test]
fn valid_empty_ranges_retain_each_utf8_boundary() {
    for (text, boundaries) in [("abc", &[0, 1, 3][..]), ("aéz", &[0, 1, 3, 4][..])] {
        let source = SourceText::new(SourceId::new(1), text.to_owned());
        for &position in boundaries {
            let anchor = source.anchor(position, position).unwrap();
            assert_eq!(anchor.range().start(), position);
            assert_eq!(anchor.range().end(), position);
            assert!(anchor.range().is_empty());
            assert_eq!(anchor.fragment(), "");
        }
    }
}

#[test]
fn invalid_empty_utf8_position_is_an_invalid_start() {
    let source = SourceText::new(SourceId::new(1), "aéz".to_owned());
    assert_eq!(
        source.anchor(2, 2).unwrap_err(),
        SourceRangeError::InvalidStartBoundary { start: 2, end: 2 }
    );
}

#[test]
fn multibyte_utf8_boundary_matrix_uses_byte_offsets() {
    let source = SourceText::new(SourceId::new(2), "aé界🦀".to_owned());
    // Byte boundaries are a=0..1, é=1..3, 界=3..6, and 🦀=6..10.
    for (start, end, fragment) in [(0, 1, "a"), (1, 3, "é"), (3, 6, "界"), (6, 10, "🦀")] {
        let anchor = source.anchor(start, end).unwrap();
        assert_eq!((anchor.range().start(), anchor.range().end()), (start, end));
        assert_eq!(anchor.fragment(), fragment);
    }

    for (start, end) in [(2, 3), (4, 6), (7, 10)] {
        assert_eq!(
            source.anchor(start, end).unwrap_err(),
            SourceRangeError::InvalidStartBoundary { start, end }
        );
    }
    for (start, end) in [(1, 2), (3, 5), (6, 9)] {
        assert_eq!(
            source.anchor(start, end).unwrap_err(),
            SourceRangeError::InvalidEndBoundary { start, end }
        );
    }
}

#[test]
fn validation_error_precedence_is_deterministic() {
    let source = SourceText::new(SourceId::new(3), "aé界".to_owned());
    let cases = [
        ((9, 8), SourceRangeError::ReversedRange { start: 9, end: 8 }),
        (
            (2, 8),
            SourceRangeError::OutOfBounds {
                start: 2,
                end: 8,
                source_len: 6,
            },
        ),
        (
            (2, 5),
            SourceRangeError::InvalidStartBoundary { start: 2, end: 5 },
        ),
        (
            (1, 5),
            SourceRangeError::InvalidEndBoundary { start: 1, end: 5 },
        ),
        (
            (2, 2),
            SourceRangeError::InvalidStartBoundary { start: 2, end: 2 },
        ),
    ];
    for ((start, end), expected) in cases {
        assert_eq!(source.anchor(start, end).unwrap_err(), expected);
    }
}

#[test]
fn exact_source_and_selected_fragments_are_preserved() {
    let exact = String::from(" \u{feff}A\r\né e\u{301} 🦀\0 tail ");
    let bytes = exact.as_bytes().to_vec();
    let source = SourceText::new(SourceId::new(4), exact.clone());
    assert_eq!(source.as_str(), exact);
    assert_eq!(source.as_str().as_bytes(), bytes);

    for (needle, expected) in [
        ("\u{feff}", "\u{feff}"),
        ("\r\n", "\r\n"),
        ("e\u{301}", "e\u{301}"),
        ("🦀", "🦀"),
        ("\0", "\0"),
    ] {
        let start = exact.find(needle).unwrap();
        let anchor = source.anchor(start, start + needle.len()).unwrap();
        assert_eq!(anchor.fragment(), expected);
    }
}

fn retained_anchor() -> SourceAnchor {
    let caller_owned = String::from("retained");
    SourceText::new(SourceId::new(7), caller_owned)
        .anchor(0, 8)
        .unwrap()
}

#[test]
fn anchor_is_independent_of_caller_lifetime() {
    let anchor = retained_anchor();
    assert_eq!(anchor.source_id(), SourceId::new(7));
    assert_eq!((anchor.range().start(), anchor.range().end()), (0, 8));
    assert_eq!(anchor.fragment(), "retained");
}

#[test]
fn clones_and_anchors_retain_publicly_observable_ownership() {
    let original = SourceText::new(SourceId::new(8), "shared text".to_owned());
    let cloned_source = original.clone();
    let from_original = original.anchor(0, 6).unwrap();
    let from_clone = cloned_source.anchor(7, 11).unwrap();
    let retained_anchor = from_original.clone();
    drop(original);
    drop(cloned_source);
    drop(from_original);

    assert_eq!(retained_anchor.source_id(), SourceId::new(8));
    assert_eq!(retained_anchor.fragment(), "shared");
    assert_eq!(from_clone.source_id(), SourceId::new(8));
    assert_eq!(from_clone.fragment(), "text");
}

#[test]
fn equal_content_retains_distinct_caller_provenance() {
    let first = SourceText::new(SourceId::new(10), "same".to_owned());
    let second = SourceText::new(SourceId::new(11), "same".to_owned());
    let first_anchor = first.anchor(0, 4).unwrap();
    let second_anchor = second.anchor(0, 4).unwrap();
    assert_eq!(first_anchor.fragment(), second_anchor.fragment());
    assert_eq!(first_anchor.range(), second_anchor.range());
    assert_ne!(first_anchor.source_id(), second_anchor.source_id());
}

#[test]
fn repeated_valid_and_invalid_requests_are_deterministic() {
    let source = SourceText::new(SourceId::new(12), "aéz".to_owned());
    for _ in 0..3 {
        let anchor = source.anchor(1, 3).unwrap();
        assert_eq!(anchor.source_id(), SourceId::new(12));
        assert_eq!((anchor.range().start(), anchor.range().end()), (1, 3));
        assert_eq!(anchor.fragment(), "é");
        assert_eq!(
            source.anchor(2, 3).unwrap_err(),
            SourceRangeError::InvalidStartBoundary { start: 2, end: 3 }
        );
    }
}

#[test]
fn raw_coordinate_is_reachable_with_only_approved_public_traits() {
    assert_coordinate_traits::<RawSourceCoordinate>();

    let source = SourceText::new(SourceId::new(20), "a\r\nb".to_owned());
    let anchor: SourceAnchor = source.anchor(1, 3).unwrap();
    let range: SourceRange = anchor.range();
    let start: RawSourceCoordinate = anchor.start_coordinate();
    let end: RawSourceCoordinate = anchor.end_coordinate();

    assert_eq!((range.start(), range.end()), (1, 3));
    assert_eq!((start.source_id(), start.byte_offset()), (source.id(), 1));
    assert_eq!((start.line_index(), start.byte_column()), (0, 1));
    assert_eq!((end.source_id(), end.byte_offset()), (source.id(), 3));
    assert_eq!((end.line_index(), end.byte_column()), (1, 0));
}

#[test]
fn raw_newline_tables_cover_every_valid_boundary() {
    for (source, expected) in [
        ("", &[(0, 0, 0)][..]),
        ("a", &[(0, 0, 0), (1, 0, 1)]),
        ("\n", &[(0, 0, 0), (1, 1, 0)]),
        ("\r", &[(0, 0, 0), (1, 1, 0)]),
        ("\r\n", &[(0, 0, 0), (1, 0, 1), (2, 1, 0)]),
        (
            "a\r\nb",
            &[(0, 0, 0), (1, 0, 1), (2, 0, 2), (3, 1, 0), (4, 1, 1)],
        ),
        (
            "\r\r\n\n",
            &[(0, 0, 0), (1, 1, 0), (2, 1, 1), (3, 2, 0), (4, 3, 0)],
        ),
    ] {
        assert_coordinate_table(source, expected);
    }
}

#[test]
fn crlf_interior_and_completed_boundaries_are_distinct() {
    let source = SourceText::new(SourceId::new(21), "\r\n".to_owned());
    let interior = coordinate_at(&source, 1);
    let completed = coordinate_at(&source, 2);

    assert_eq!((interior.line_index(), interior.byte_column()), (0, 1));
    assert_eq!((completed.line_index(), completed.byte_column()), (1, 0));
}

#[test]
fn empty_anchors_and_trailing_terminators_project_deterministically() {
    for (source, boundaries) in [
        ("", &[0][..]),
        ("aé\r\n", &[0, 1, 3, 4, 5]),
        ("a\n", &[2]),
        ("a\r", &[2]),
        ("a\r\n", &[3]),
        ("\r\r\n\n", &[4]),
    ] {
        let source = SourceText::new(SourceId::new(22), source.to_owned());
        for &offset in boundaries {
            let anchor = source.anchor(offset, offset).unwrap();
            assert_eq!(anchor.start_coordinate(), anchor.end_coordinate());
            assert_eq!(anchor.start_coordinate(), anchor.start_coordinate());
        }
    }

    for (source, expected_line) in [("a\n", 1), ("a\r", 1), ("a\r\n", 1), ("\r\r\n\n", 3)] {
        let source = SourceText::new(SourceId::new(23), source.to_owned());
        let eof = coordinate_at(&source, source.as_str().len());
        assert_eq!(eof.line_index(), expected_line);
        assert_eq!(eof.byte_column(), 0);
    }
}

#[test]
fn utf8_byte_columns_are_not_unicode_or_display_columns() {
    assert_coordinate_table("éあ😀", &[(0, 0, 0), (2, 0, 2), (5, 0, 5), (9, 0, 9)]);

    for separator in ['\u{000c}', '\u{000b}', '\u{0085}', '\u{2028}', '\u{2029}'] {
        let text = separator.to_string();
        let source = SourceText::new(SourceId::new(24), text.clone());
        let end = coordinate_at(&source, text.len());
        assert_eq!(end.line_index(), 0);
        assert_eq!(end.byte_column(), text.len());
    }
}

#[test]
fn coordinate_provenance_copy_clone_and_lifecycle_are_deterministic() {
    let detached = {
        let first = SourceText::new(SourceId::new(25), "same\ntext".to_owned());
        let second = SourceText::new(SourceId::new(26), "same\ntext".to_owned());
        let first_anchor = first.anchor(5, 9).unwrap();
        let second_anchor = second.anchor(5, 9).unwrap();
        let coordinate = first_anchor.start_coordinate();
        let copied = coordinate;
        #[allow(clippy::clone_on_copy)]
        let cloned = coordinate.clone();

        assert_ne!(coordinate, second_anchor.start_coordinate());
        assert_eq!(coordinate, first_anchor.start_coordinate());
        assert_eq!(copied, coordinate);
        assert_eq!(cloned, coordinate);
        coordinate
    };

    assert_eq!(detached.source_id(), SourceId::new(25));
    assert_eq!(detached.byte_offset(), 5);
    assert_eq!((detached.line_index(), detached.byte_column()), (1, 0));
}

#[test]
fn coordinate_debug_does_not_expose_source_content() {
    const MARKER: &str = "private-marker-7f31";
    let source = SourceText::new(SourceId::new(27), format!("prefix-{MARKER}-suffix"));
    let anchor = source.anchor(7, 26).unwrap();
    let debug = format!("{:?}", anchor.start_coordinate());

    assert!(!debug.contains(source.as_str()));
    assert!(!debug.contains(anchor.fragment()));
    assert!(!debug.contains(MARKER));
}

#[test]
fn invalid_ranges_fail_before_coordinate_projection() {
    let source = SourceText::new(SourceId::new(28), "aé".to_owned());
    assert_eq!(
        source.anchor(5, 4).unwrap_err(),
        SourceRangeError::ReversedRange { start: 5, end: 4 }
    );
    assert_eq!(
        source.anchor(2, 5).unwrap_err(),
        SourceRangeError::OutOfBounds {
            start: 2,
            end: 5,
            source_len: 3,
        }
    );
    assert_eq!(
        source.anchor(2, 3).unwrap_err(),
        SourceRangeError::InvalidStartBoundary { start: 2, end: 3 }
    );
    assert_eq!(
        source.anchor(1, 2).unwrap_err(),
        SourceRangeError::InvalidEndBoundary { start: 1, end: 2 }
    );
}
