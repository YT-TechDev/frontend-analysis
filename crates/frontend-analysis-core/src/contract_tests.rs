use crate::{SourceAnchor, SourceId, SourceRangeError, SourceText};

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
