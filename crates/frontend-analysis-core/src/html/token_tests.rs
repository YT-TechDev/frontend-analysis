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
fn tags_preserve_names_attributes_duplicates_and_delimiters() {
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
    assert_eq!(
        tag.attributes()[1].disposition(),
        HtmlAttributeDisposition::DuplicateOf { first_index: 0 }
    );
    assert_eq!(tag.close_delimiter().fragment(), ">");

    let end_source = source(3, "</DIV>");
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

    let self_closing_source = source(4, "<img/>");
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
fn attribute_value_forms_remain_distinct() {
    let boolean_source = source(5, "disabled");
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

    let missing_source = source(6, "foo=   ");
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

    let unquoted_source = source(7, "foo=é");
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

    let quoted_source = source(8, "foo=\"\" bar='y'");
    let empty = HtmlAttributeEvidence::new(
        anchor(&quoted_source, 0, 6),
        name(&quoted_source, 0, 3, "foo"),
        HtmlAttributeValueSyntax::DoubleQuoted {
            equals: anchor(&quoted_source, 3, 4),
            open_quote: anchor(&quoted_source, 4, 5),
            value: anchor(&quoted_source, 5, 5),
            close_quote: anchor(&quoted_source, 5, 6),
        },
        String::new(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap();
    let single = HtmlAttributeEvidence::new(
        anchor(&quoted_source, 7, 14),
        name(&quoted_source, 7, 10, "bar"),
        HtmlAttributeValueSyntax::SingleQuoted {
            equals: anchor(&quoted_source, 10, 11),
            open_quote: anchor(&quoted_source, 11, 12),
            value: anchor(&quoted_source, 12, 13),
            close_quote: anchor(&quoted_source, 13, 14),
        },
        "y".to_owned(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap();
    assert!(matches!(
        empty.value_syntax(),
        HtmlAttributeValueSyntax::DoubleQuoted { .. }
    ));
    assert!(matches!(
        single.value_syntax(),
        HtmlAttributeValueSyntax::SingleQuoted { .. }
    ));
}

#[test]
fn invalid_source_ranges_delimiters_and_attribute_syntax_are_rejected() {
    let owner = source(9, "<a>");
    let foreign = source(10, "a");
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

    let outside = source(11, "x<a>y");
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

    let wrong = source(12, "[a>");
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

    let empty_unquoted = source(13, "foo=");
    assert_eq!(
        HtmlAttributeEvidence::new(
            anchor(&empty_unquoted, 0, 4),
            name(&empty_unquoted, 0, 3, "foo"),
            HtmlAttributeValueSyntax::Unquoted {
                equals: anchor(&empty_unquoted, 3, 4),
                value: anchor(&empty_unquoted, 4, 4),
            },
            String::new(),
            HtmlAttributeDisposition::Effective,
        )
        .unwrap_err(),
        HtmlTokenContractError::UnquotedValueMustBeNonEmpty
    );
}

#[test]
fn duplicate_and_order_invariants_are_enforced() {
    let source = source(14, "<a x x x>");
    let first = boolean_attribute(&source, 3, 4, "x", HtmlAttributeDisposition::Effective);
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

    let source = source(15, "<a x x>");
    let first = boolean_attribute(&source, 3, 4, "x", HtmlAttributeDisposition::Effective);
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

    let source = source(16, "<a x y>");
    let first = boolean_attribute(&source, 3, 4, "x", HtmlAttributeDisposition::Effective);
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

    let source = source(17, "<a x x>");
    let first = boolean_attribute(&source, 3, 4, "x", HtmlAttributeDisposition::Effective);
    let second = boolean_attribute(&source, 5, 6, "x", HtmlAttributeDisposition::Effective);
    assert_eq!(
        tag(
            &source,
            HtmlTagKind::Start,
            (0, 7),
            (0, 1),
            (1, 2, "a"),
            vec![first, second],
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
fn eof_bom_redaction_and_panic_contracts_hold() {
    let source = source(18, "abc");
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

    let bom_source = source(19, "\u{feff}<a>");
    let evidence =
        HtmlPreprocessingEvidence::new(&bom_source, Some(anchor(&bom_source, 0, 3))).unwrap();
    assert_eq!(
        evidence.skipped_leading_bom().unwrap().fragment(),
        "\u{feff}"
    );

    const MARKER: &str = "private-token-marker-51f2";
    let private = source(20, MARKER);
    let token =
        HtmlCharacterToken::new(anchor(&private, 0, private.as_str().len()), MARKER.to_owned())
            .unwrap();
    let debug = format!("{token:?}");
    assert!(!debug.contains(MARKER));
    assert!(debug.contains("interpreted_byte_len"));
    let error = HtmlCharacterToken::new(anchor(&private, 0, 0), String::new()).unwrap_err();
    assert!(!format!("{error:?}").contains(MARKER));
    assert!(!error.to_string().contains(MARKER));

    let malformed = source(21, "<a/ >");
    let result = catch_unwind(AssertUnwindSafe(|| {
        tag(
            &malformed,
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
    let source = source(22, "x");
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

    let tag_source = source(23, "<a>");
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
