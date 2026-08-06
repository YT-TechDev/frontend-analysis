use std::panic::{AssertUnwindSafe, catch_unwind};

use super::token::*;
use crate::{SourceAnchor, SourceId, SourceText};

fn source(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn anchor(source: &SourceText, start: usize, end: usize) -> SourceAnchor {
    source.anchor(start, end).unwrap()
}

fn name(source: &SourceText, start: usize, end: usize, interpreted: &str) -> HtmlNameEvidence {
    HtmlNameEvidence::new(anchor(source, start, end), interpreted.to_owned()).unwrap()
}

fn boolean_attribute(
    source: &SourceText,
    start: usize,
    end: usize,
    interpreted: &str,
    disposition: HtmlAttributeDisposition,
) -> HtmlAttributeEvidence {
    HtmlAttributeEvidence::new(
        anchor(source, start, end),
        name(source, start, end, interpreted),
        HtmlAttributeValueSyntax::Missing,
        String::new(),
        disposition,
    )
    .unwrap()
}

#[allow(clippy::too_many_arguments)]
fn tag(
    source: &SourceText,
    kind: HtmlTagKind,
    complete: (usize, usize),
    opener: (usize, usize),
    tag_name: (usize, usize, &str),
    attributes: Vec<HtmlAttributeEvidence>,
    solidus: Option<(usize, usize)>,
    closer: (usize, usize),
) -> Result<HtmlTagToken, HtmlTokenContractError> {
    HtmlTagToken::new(
        kind,
        anchor(source, complete.0, complete.1),
        anchor(source, opener.0, opener.1),
        name(source, tag_name.0, tag_name.1, tag_name.2),
        attributes,
        solidus.map(|range| anchor(source, range.0, range.1)),
        anchor(source, closer.0, closer.1),
    )
}

#[test]
fn character_tokens_preserve_raw_and_interpreted_evidence() {
    let source = source(1, "aé\r\n\0");
    let token = HtmlCharacterToken::new(
        anchor(&source, 0, source.as_str().len()),
        "aé\n\u{fffd}".to_owned(),
    )
    .unwrap();

    assert_eq!(token.source().fragment(), "aé\r\n\0");
    assert_eq!(token.interpreted(), "aé\n\u{fffd}");

    assert_eq!(
        HtmlCharacterToken::new(anchor(&source, 0, 0), "x".to_owned()).unwrap_err(),
        HtmlTokenContractError::EmptySourceRange {
            role: HtmlEvidenceRole::Character,
        }
    );
    assert_eq!(
        HtmlCharacterToken::new(anchor(&source, 0, 1), String::new()).unwrap_err(),
        HtmlTokenContractError::EmptyInterpretedValue {
            role: HtmlEvidenceRole::Character,
        }
    );
}

#[test]
fn start_tags_preserve_names_attributes_and_duplicates_in_source_order() {
    let source = source(2, "<DIV ID=x id='y'>");
    let first = HtmlAttributeEvidence::new(
        anchor(&source, 5, 9),
        name(&source, 5, 7, "id"),
        HtmlAttributeValueSyntax::Unquoted {
            equals: anchor(&source, 7, 8),
            value: anchor(&source, 8, 9),
        },
        "x".to_owned(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap();
    let duplicate = HtmlAttributeEvidence::new(
        anchor(&source, 10, 16),
        name(&source, 10, 12, "id"),
        HtmlAttributeValueSyntax::SingleQuoted {
            equals: anchor(&source, 12, 13),
            open_quote: anchor(&source, 13, 14),
            value: anchor(&source, 14, 15),
            close_quote: anchor(&source, 15, 16),
        },
        "y".to_owned(),
        HtmlAttributeDisposition::DuplicateOf { first_index: 0 },
    )
    .unwrap();

    let tag = tag(
        &source,
        HtmlTagKind::Start,
        (0, 17),
        (0, 1),
        (1, 4, "div"),
        vec![first, duplicate],
        None,
        (16, 17),
    )
    .unwrap();

    assert_eq!(tag.kind(), HtmlTagKind::Start);
    assert_eq!(tag.complete().fragment(), "<DIV ID=x id='y'>");
    assert_eq!(tag.open_delimiter().fragment(), "<");
    assert_eq!(tag.name().source().fragment(), "DIV");
    assert_eq!(tag.name().interpreted(), "div");
    assert_eq!(tag.attributes()[0].complete().fragment(), "ID=x");
    assert_eq!(tag.attributes()[0].interpreted_value(), "x");
    assert_eq!(tag.attributes()[1].complete().fragment(), "id='y'");
    assert_eq!(
        tag.attributes()[1].disposition(),
        HtmlAttributeDisposition::DuplicateOf { first_index: 0 }
    );
    assert_eq!(tag.close_delimiter().fragment(), ">");
}

#[test]
fn attribute_value_forms_are_not_collapsed_into_optionality() {
    let boolean_source = source(3, "disabled");
    let boolean = boolean_attribute(
        &boolean_source,
        0,
        8,
        "disabled",
        HtmlAttributeDisposition::Effective,
    );
    assert!(matches!(
        boolean.value_syntax(),
        HtmlAttributeValueSyntax::Missing
    ));
    assert_eq!(boolean.interpreted_value(), "");

    let missing_source = source(4, "foo=   ");
    let missing = HtmlAttributeEvidence::new(
        anchor(&missing_source, 0, 7),
        name(&missing_source, 0, 3, "foo"),
        HtmlAttributeValueSyntax::MissingAfterEquals {
            equals: anchor(&missing_source, 3, 4),
            value_boundary: anchor(&missing_source, 7, 7),
        },
        String::new(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap();
    assert!(matches!(
        missing.value_syntax(),
        HtmlAttributeValueSyntax::MissingAfterEquals { .. }
    ));

    let unquoted_source = source(5, "foo=é");
    let unquoted = HtmlAttributeEvidence::new(
        anchor(&unquoted_source, 0, 6),
        name(&unquoted_source, 0, 3, "foo"),
        HtmlAttributeValueSyntax::Unquoted {
            equals: anchor(&unquoted_source, 3, 4),
            value: anchor(&unquoted_source, 4, 6),
        },
        "é".to_owned(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap();
    assert!(matches!(
        unquoted.value_syntax(),
        HtmlAttributeValueSyntax::Unquoted { .. }
    ));

    let empty_quoted_source = source(6, "foo=\"\"");
    let empty_quoted = HtmlAttributeEvidence::new(
        anchor(&empty_quoted_source, 0, 6),
        name(&empty_quoted_source, 0, 3, "foo"),
        HtmlAttributeValueSyntax::DoubleQuoted {
            equals: anchor(&empty_quoted_source, 3, 4),
            open_quote: anchor(&empty_quoted_source, 4, 5),
            value: anchor(&empty_quoted_source, 5, 5),
            close_quote: anchor(&empty_quoted_source, 5, 6),
        },
        String::new(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap();
    assert!(matches!(
        empty_quoted.value_syntax(),
        HtmlAttributeValueSyntax::DoubleQuoted { .. }
    ));

    let single_quoted_source = source(7, "foo='y'");
    let single_quoted = HtmlAttributeEvidence::new(
        anchor(&single_quoted_source, 0, 7),
        name(&single_quoted_source, 0, 3, "foo"),
        HtmlAttributeValueSyntax::SingleQuoted {
            equals: anchor(&single_quoted_source, 3, 4),
            open_quote: anchor(&single_quoted_source, 4, 5),
            value: anchor(&single_quoted_source, 5, 6),
            close_quote: anchor(&single_quoted_source, 6, 7),
        },
        "y".to_owned(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap();
    assert!(matches!(
        single_quoted.value_syntax(),
        HtmlAttributeValueSyntax::SingleQuoted { .. }
    ));
}

#[test]
fn end_tag_and_self_closing_delimiters_are_exact() {
    let end_source = source(8, "</DIV>");
    let end = tag(
        &end_source,
        HtmlTagKind::End,
        (0, 6),
        (0, 2),
        (2, 5, "div"),
        Vec::new(),
        None,
        (5, 6),
    )
    .unwrap();
    assert_eq!(end.open_delimiter().fragment(), "</");

    let self_closing_source = source(9, "<img/>");
    let self_closing = tag(
        &self_closing_source,
        HtmlTagKind::Start,
        (0, 6),
        (0, 1),
        (1, 4, "img"),
        Vec::new(),
        Some((4, 5)),
        (5, 6),
    )
    .unwrap();
    assert_eq!(self_closing.self_closing_solidus().unwrap().fragment(), "/");
}

#[test]
fn source_mismatch_outside_ranges_and_wrong_delimiters_are_rejected() {
    let owner = source(10, "<a>");
    let foreign = source(11, "a");
    assert_eq!(
        HtmlTagToken::new(
            HtmlTagKind::Start,
            anchor(&owner, 0, 3),
            anchor(&owner, 0, 1),
            name(&foreign, 0, 1, "a"),
            Vec::new(),
            None,
            anchor(&owner, 2, 3),
        )
        .unwrap_err(),
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::TagName,
            expected: owner.id(),
            actual: foreign.id(),
        }
    );

    let outside = source(12, "x<a>y");
    assert_eq!(
        HtmlTagToken::new(
            HtmlTagKind::Start,
            anchor(&outside, 1, 4),
            anchor(&outside, 1, 2),
            name(&outside, 0, 1, "x"),
            Vec::new(),
            None,
            anchor(&outside, 3, 4),
        )
        .unwrap_err(),
        HtmlTokenContractError::RangeOutsideOwner {
            role: HtmlEvidenceRole::TagName,
        }
    );

    let wrong = source(13, "[a>");
    assert_eq!(
        tag(
            &wrong,
            HtmlTagKind::Start,
            (0, 3),
            (0, 1),
            (1, 2, "a"),
            Vec::new(),
            None,
            (2, 3),
        )
        .unwrap_err(),
        HtmlTokenContractError::WrongAuthoredFragment {
            role: HtmlEvidenceRole::OpenDelimiter,
            expected: "<",
        }
    );
}

#[test]
fn invalid_attribute_syntax_and_order_are_rejected() {
    let source = source(14, "foo=");
    assert_eq!(
        HtmlAttributeEvidence::new(
            anchor(&source, 0, 4),
            name(&source, 0, 3, "foo"),
            HtmlAttributeValueSyntax::Unquoted {
                equals: anchor(&source, 3, 4),
                value: anchor(&source, 4, 4),
            },
            String::new(),
            HtmlAttributeDisposition::Effective,
        )
        .unwrap_err(),
        HtmlTokenContractError::UnquotedValueMustBeNonEmpty
    );

    let source = source(15, "foo:bar");
    assert_eq!(
        HtmlAttributeEvidence::new(
            anchor(&source, 0, 7),
            name(&source, 0, 3, "foo"),
            HtmlAttributeValueSyntax::Unquoted {
                equals: anchor(&source, 3, 4),
                value: anchor(&source, 4, 7),
            },
            "bar".to_owned(),
            HtmlAttributeDisposition::Effective,
        )
        .unwrap_err(),
        HtmlTokenContractError::WrongAuthoredFragment {
            role: HtmlEvidenceRole::Equals,
            expected: "=",
        }
    );

    let overlap_source = source(16, "<a x y>");
    let first = boolean_attribute(
        &overlap_source,
        3,
        4,
        "x",
        HtmlAttributeDisposition::Effective,
    );
    let overlapping = boolean_attribute(
        &overlap_source,
        3,
        4,
        "x",
        HtmlAttributeDisposition::DuplicateOf { first_index: 0 },
    );
    assert_eq!(
        tag(
            &overlap_source,
            HtmlTagKind::Start,
            (0, 7),
            (0, 1),
            (1, 2, "a"),
            vec![first, overlapping],
            None,
            (6, 7),
        )
        .unwrap_err(),
        HtmlTokenContractError::InvalidOrder {
            role: HtmlEvidenceRole::Attribute,
        }
    );
}

#[test]
fn duplicate_disposition_must_reference_the_effective_first_occurrence() {
    let source = source(17, "<a x x x>");
    let first = boolean_attribute(
        &source,
        3,
        4,
        "x",
        HtmlAttributeDisposition::Effective,
    );
    let duplicate = boolean_attribute(
        &source,
        5,
        6,
        "x",
        HtmlAttributeDisposition::DuplicateOf { first_index: 0 },
    );
    let invalid_target = boolean_attribute(
        &source,
        7,
        8,
        "x",
        HtmlAttributeDisposition::DuplicateOf { first_index: 1 },
    );
    assert_eq!(
        tag(
            &source,
            HtmlTagKind::Start,
            (0, 9),
            (0, 1),
            (1, 2, "a"),
            vec![first, duplicate, invalid_target],
            None,
            (8, 9),
        )
        .unwrap_err(),
        HtmlTokenContractError::DuplicateTargetMustBeEffective {
            attribute_index: 2,
            first_index: 1,
        }
    );

    let source = source(18, "<a x x>");
    let first = boolean_attribute(
        &source,
        3,
        4,
        "x",
        HtmlAttributeDisposition::Effective,
    );
    let invalid_reference = boolean_attribute(
        &source,
        5,
        6,
        "x",
        HtmlAttributeDisposition::DuplicateOf { first_index: 1 },
    );
    assert_eq!(
        tag(
            &source,
            HtmlTagKind::Start,
            (0, 7),
            (0, 1),
            (1, 2, "a"),
            vec![first, invalid_reference],
            None,
            (6, 7),
        )
        .unwrap_err(),
        HtmlTokenContractError::InvalidDuplicateReference {
            attribute_index: 1,
            first_index: 1,
        }
    );

    let source = source(19, "<a x y>");
    let first = boolean_attribute(
        &source,
        3,
        4,
        "x",
        HtmlAttributeDisposition::Effective,
    );
    let wrong_name = boolean_attribute(
        &source,
        5,
        6,
        "y",
        HtmlAttributeDisposition::DuplicateOf { first_index: 0 },
    );
    assert_eq!(
        tag(
            &source,
            HtmlTagKind::Start,
            (0, 7),
            (0, 1),
            (1, 2, "a"),
            vec![first, wrong_name],
            None,
            (6, 7),
        )
        .unwrap_err(),
        HtmlTokenContractError::DuplicateNameMismatch {
            attribute_index: 1,
            first_index: 0,
        }
    );

    let source = source(20, "<a x x>");
    let first = boolean_attribute(
        &source,
        3,
        4,
        "x",
        HtmlAttributeDisposition::Effective,
    );
    let second_effective = boolean_attribute(
        &source,
        5,
        6,
        "x",
        HtmlAttributeDisposition::Effective,
    );
    assert_eq!(
        tag(
            &source,
            HtmlTagKind::Start,
            (0, 7),
            (0, 1),
            (1, 2, "a"),
            vec![first, second_effective],
            None,
            (6, 7),
        )
        .unwrap_err(),
        HtmlTokenContractError::UnexpectedEffectiveDuplicate {
            attribute_index: 1,
            first_index: 0,
        }
    );
}

#[test]
fn eof_and_leading_bom_evidence_have_exact_boundaries() {
    let source = source(21, "abc");
    let eof = HtmlEndOfFileToken::new(&source, anchor(&source, 3, 3)).unwrap();
    assert!(eof.source().range().is_empty());
    assert_eq!(
        HtmlEndOfFileToken::new(&source, anchor(&source, 2, 2)).unwrap_err(),
        HtmlTokenContractError::EndOfFileNotAtSourceEnd
    );
    assert_eq!(
        HtmlEndOfFileToken::new(&source, anchor(&source, 2, 3)).unwrap_err(),
        HtmlTokenContractError::EndOfFileMustBeEmpty
    );

    let bom_source = source(22, "\u{feff}<a>");
    let evidence =
        HtmlPreprocessingEvidence::new(&bom_source, Some(anchor(&bom_source, 0, 3))).unwrap();
    assert_eq!(
        evidence.skipped_leading_bom().unwrap().fragment(),
        "\u{feff}"
    );

    let misplaced = source(23, "x\u{feff}");
    assert_eq!(
        HtmlPreprocessingEvidence::new(&misplaced, Some(anchor(&misplaced, 1, 4))).unwrap_err(),
        HtmlTokenContractError::LeadingBomNotAtStart
    );
}

#[test]
fn debug_and_errors_do_not_expose_source_or_interpreted_content() {
    const MARKER: &str = "private-token-marker-51f2";
    let source = source(24, MARKER);
    let token =
        HtmlCharacterToken::new(anchor(&source, 0, source.as_str().len()), MARKER.to_owned())
            .unwrap();
    let debug = format!("{token:?}");
    assert!(!debug.contains(MARKER));
    assert!(debug.contains("interpreted_byte_len"));

    let name = HtmlNameEvidence::new(
        anchor(&source, 0, source.as_str().len()),
        MARKER.to_owned(),
    )
    .unwrap();
    assert!(!format!("{name:?}").contains(MARKER));

    let error = HtmlCharacterToken::new(anchor(&source, 0, 0), String::new()).unwrap_err();
    assert!(!format!("{error:?}").contains(MARKER));
    assert!(!error.to_string().contains(MARKER));
}

#[test]
fn invalid_construction_returns_an_error_without_panicking() {
    let source = source(25, "<a/ >");
    let result = catch_unwind(AssertUnwindSafe(|| {
        tag(
            &source,
            HtmlTagKind::Start,
            (0, 5),
            (0, 1),
            (1, 2, "a"),
            Vec::new(),
            Some((2, 3)),
            (4, 5),
        )
    }));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().unwrap_err(),
        HtmlTokenContractError::InvalidSelfClosingPosition
    );
}

#[test]
fn token_variants_preserve_deterministic_vec_order() {
    let source = source(26, "x");
    let tokens = [
        HtmlToken::Character(
            HtmlCharacterToken::new(anchor(&source, 0, 1), "x".to_owned()).unwrap(),
        ),
        HtmlToken::EndOfFile(HtmlEndOfFileToken::new(&source, anchor(&source, 1, 1)).unwrap()),
    ];
    match &tokens[0] {
        HtmlToken::Character(token) => assert_eq!(token.source().range().start(), 0),
        other => panic!("unexpected token: {other:?}"),
    }
    match &tokens[1] {
        HtmlToken::EndOfFile(token) => assert_eq!(token.source().range().start(), 1),
        other => panic!("unexpected token: {other:?}"),
    }

    let tag_source = source(27, "<a>");
    let token = HtmlToken::Tag(
        tag(
            &tag_source,
            HtmlTagKind::Start,
            (0, 3),
            (0, 1),
            (1, 2, "a"),
            Vec::new(),
            None,
            (2, 3),
        )
        .unwrap(),
    );
    match token {
        HtmlToken::Tag(tag) => assert_eq!(tag.complete().fragment(), "<a>"),
        other => panic!("unexpected token: {other:?}"),
    }
}
