use std::fmt::Debug;
use std::panic::{AssertUnwindSafe, catch_unwind};

use super::token::*;
use crate::{SourceAnchor, SourceId, SourceText};

fn src(id: u64, text: &str) -> SourceText {
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
) -> HtmlAttributeEvidence {
    HtmlAttributeEvidence::new(
        anchor(source, start, end),
        name(source, start, end, interpreted),
        HtmlAttributeValueSyntax::Missing,
        String::new(),
        HtmlAttributeDisposition::Effective,
    )
    .unwrap()
}

fn assert_contract_error<T: Debug>(
    operation: impl FnOnce() -> Result<T, HtmlTokenContractError>,
    expected: HtmlTokenContractError,
) {
    let result = catch_unwind(AssertUnwindSafe(operation));
    assert!(result.is_ok(), "invalid model input must not panic");
    assert_eq!(result.unwrap().unwrap_err(), expected);
}

#[test]
fn character_preprocessing_matrix_is_explicit() {
    for (index, (raw, interpreted)) in [
        ("\n", "\n"),
        ("\r", "\n"),
        ("\r\n", "\n"),
        ("\0", "\u{fffd}"),
        ("é", "é"),
    ]
    .into_iter()
    .enumerate()
    {
        let source = src(100 + index as u64, raw);
        let token = HtmlCharacterToken::new(
            anchor(&source, 0, source.as_str().len()),
            interpreted.to_owned(),
        )
        .unwrap();
        assert_eq!(token.source().fragment(), raw);
        assert_eq!(token.interpreted(), interpreted);
    }
}

#[test]
fn every_nested_evidence_class_rejects_foreign_source_identity() {
    let attribute_source = src(110, "foo=\"x\"");
    let foreign_attribute_source = src(111, "foo=\"x\"");

    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&attribute_source, 0, 7),
                name(&foreign_attribute_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::Missing,
                String::new(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::AttributeName,
            expected: attribute_source.id(),
            actual: foreign_attribute_source.id(),
        },
    );

    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&attribute_source, 0, 7),
                name(&attribute_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::DoubleQuoted {
                    equals: anchor(&foreign_attribute_source, 3, 4),
                    open_quote: anchor(&attribute_source, 4, 5),
                    value: anchor(&attribute_source, 5, 6),
                    close_quote: anchor(&attribute_source, 6, 7),
                },
                "x".to_owned(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::Equals,
            expected: attribute_source.id(),
            actual: foreign_attribute_source.id(),
        },
    );

    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&attribute_source, 0, 7),
                name(&attribute_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::DoubleQuoted {
                    equals: anchor(&attribute_source, 3, 4),
                    open_quote: anchor(&foreign_attribute_source, 4, 5),
                    value: anchor(&attribute_source, 5, 6),
                    close_quote: anchor(&attribute_source, 6, 7),
                },
                "x".to_owned(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::OpenQuote,
            expected: attribute_source.id(),
            actual: foreign_attribute_source.id(),
        },
    );

    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&attribute_source, 0, 7),
                name(&attribute_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::DoubleQuoted {
                    equals: anchor(&attribute_source, 3, 4),
                    open_quote: anchor(&attribute_source, 4, 5),
                    value: anchor(&foreign_attribute_source, 5, 6),
                    close_quote: anchor(&attribute_source, 6, 7),
                },
                "x".to_owned(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::Value,
            expected: attribute_source.id(),
            actual: foreign_attribute_source.id(),
        },
    );

    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&attribute_source, 0, 7),
                name(&attribute_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::DoubleQuoted {
                    equals: anchor(&attribute_source, 3, 4),
                    open_quote: anchor(&attribute_source, 4, 5),
                    value: anchor(&attribute_source, 5, 6),
                    close_quote: anchor(&foreign_attribute_source, 6, 7),
                },
                "x".to_owned(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::CloseQuote,
            expected: attribute_source.id(),
            actual: foreign_attribute_source.id(),
        },
    );

    let missing_source = src(112, "foo= ");
    let foreign_missing_source = src(113, "foo= ");
    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&missing_source, 0, 5),
                name(&missing_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::MissingAfterEquals {
                    equals: anchor(&missing_source, 3, 4),
                    value_boundary: anchor(&foreign_missing_source, 5, 5),
                },
                String::new(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::ValueBoundary,
            expected: missing_source.id(),
            actual: foreign_missing_source.id(),
        },
    );

    let tag_source = src(114, "<a x/>");
    let foreign_tag_source = src(115, "<a x/>");
    let foreign_attribute = boolean_attribute(&foreign_tag_source, 3, 4, "x");

    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&tag_source, 0, 6),
                anchor(&foreign_tag_source, 0, 1),
                name(&tag_source, 1, 2, "a"),
                Vec::new(),
                None,
                anchor(&tag_source, 5, 6),
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::OpenDelimiter,
            expected: tag_source.id(),
            actual: foreign_tag_source.id(),
        },
    );

    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&tag_source, 0, 6),
                anchor(&tag_source, 0, 1),
                name(&foreign_tag_source, 1, 2, "a"),
                Vec::new(),
                None,
                anchor(&tag_source, 5, 6),
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::TagName,
            expected: tag_source.id(),
            actual: foreign_tag_source.id(),
        },
    );

    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&tag_source, 0, 6),
                anchor(&tag_source, 0, 1),
                name(&tag_source, 1, 2, "a"),
                vec![foreign_attribute],
                None,
                anchor(&tag_source, 5, 6),
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::Attribute,
            expected: tag_source.id(),
            actual: foreign_tag_source.id(),
        },
    );

    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&tag_source, 0, 6),
                anchor(&tag_source, 0, 1),
                name(&tag_source, 1, 2, "a"),
                Vec::new(),
                Some(anchor(&foreign_tag_source, 4, 5)),
                anchor(&tag_source, 5, 6),
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::SelfClosingSolidus,
            expected: tag_source.id(),
            actual: foreign_tag_source.id(),
        },
    );

    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&tag_source, 0, 6),
                anchor(&tag_source, 0, 1),
                name(&tag_source, 1, 2, "a"),
                Vec::new(),
                None,
                anchor(&foreign_tag_source, 5, 6),
            )
        },
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::CloseDelimiter,
            expected: tag_source.id(),
            actual: foreign_tag_source.id(),
        },
    );

    assert_contract_error(
        || HtmlEndOfFileToken::new(&tag_source, anchor(&foreign_tag_source, 6, 6)),
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::EndOfFile,
            expected: tag_source.id(),
            actual: foreign_tag_source.id(),
        },
    );

    let bom_source = src(116, "\u{feff}");
    let foreign_bom_source = src(117, "\u{feff}");
    assert_contract_error(
        || HtmlPreprocessingEvidence::new(&bom_source, Some(anchor(&foreign_bom_source, 0, 3))),
        HtmlTokenContractError::SourceIdentityMismatch {
            role: HtmlEvidenceRole::LeadingBom,
            expected: bom_source.id(),
            actual: foreign_bom_source.id(),
        },
    );
}

#[test]
fn authored_delimiter_matrix_rejects_every_wrong_fragment() {
    let equals_source = src(120, "foo:x");
    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&equals_source, 0, 5),
                name(&equals_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::Unquoted {
                    equals: anchor(&equals_source, 3, 4),
                    value: anchor(&equals_source, 4, 5),
                },
                "x".to_owned(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::WrongAuthoredFragment {
            role: HtmlEvidenceRole::Equals,
            expected: "=",
        },
    );

    let quote_source = src(121, "foo='x'");
    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&quote_source, 0, 7),
                name(&quote_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::DoubleQuoted {
                    equals: anchor(&quote_source, 3, 4),
                    open_quote: anchor(&quote_source, 4, 5),
                    value: anchor(&quote_source, 5, 6),
                    close_quote: anchor(&quote_source, 6, 7),
                },
                "x".to_owned(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::WrongAuthoredFragment {
            role: HtmlEvidenceRole::OpenQuote,
            expected: "\"",
        },
    );

    let start_source = src(122, "[a>");
    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&start_source, 0, 3),
                anchor(&start_source, 0, 1),
                name(&start_source, 1, 2, "a"),
                Vec::new(),
                None,
                anchor(&start_source, 2, 3),
            )
        },
        HtmlTokenContractError::WrongAuthoredFragment {
            role: HtmlEvidenceRole::OpenDelimiter,
            expected: "<",
        },
    );

    let end_source = src(123, "<a>");
    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::End,
                anchor(&end_source, 0, 3),
                anchor(&end_source, 0, 1),
                name(&end_source, 1, 2, "a"),
                Vec::new(),
                None,
                anchor(&end_source, 2, 3),
            )
        },
        HtmlTokenContractError::WrongAuthoredFragment {
            role: HtmlEvidenceRole::OpenDelimiter,
            expected: "</",
        },
    );

    let slash_source = src(124, "<a!>");
    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&slash_source, 0, 4),
                anchor(&slash_source, 0, 1),
                name(&slash_source, 1, 2, "a"),
                Vec::new(),
                Some(anchor(&slash_source, 2, 3)),
                anchor(&slash_source, 3, 4),
            )
        },
        HtmlTokenContractError::WrongAuthoredFragment {
            role: HtmlEvidenceRole::SelfClosingSolidus,
            expected: "/",
        },
    );

    let close_source = src(125, "<a]");
    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&close_source, 0, 3),
                anchor(&close_source, 0, 1),
                name(&close_source, 1, 2, "a"),
                Vec::new(),
                None,
                anchor(&close_source, 2, 3),
            )
        },
        HtmlTokenContractError::WrongAuthoredFragment {
            role: HtmlEvidenceRole::CloseDelimiter,
            expected: ">",
        },
    );
}

#[test]
fn overlap_and_adjacency_failures_are_rejected_without_panics() {
    let tag_source = src(130, "<ab>");
    let overlapping_attribute = boolean_attribute(&tag_source, 1, 2, "a");
    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&tag_source, 0, 4),
                anchor(&tag_source, 0, 1),
                name(&tag_source, 1, 3, "ab"),
                vec![overlapping_attribute],
                None,
                anchor(&tag_source, 3, 4),
            )
        },
        HtmlTokenContractError::InvalidOrder {
            role: HtmlEvidenceRole::Attribute,
        },
    );

    let attributes_source = src(131, "<a xx>");
    let first = boolean_attribute(&attributes_source, 3, 5, "xx");
    let overlapping = boolean_attribute(&attributes_source, 4, 5, "x");
    assert_contract_error(
        || {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&attributes_source, 0, 6),
                anchor(&attributes_source, 0, 1),
                name(&attributes_source, 1, 2, "a"),
                vec![first, overlapping],
                None,
                anchor(&attributes_source, 5, 6),
            )
        },
        HtmlTokenContractError::InvalidOrder {
            role: HtmlEvidenceRole::Attribute,
        },
    );

    let quoted_source = src(132, "foo=\"x\"");
    assert_contract_error(
        || {
            HtmlAttributeEvidence::new(
                anchor(&quoted_source, 0, 7),
                name(&quoted_source, 0, 3, "foo"),
                HtmlAttributeValueSyntax::DoubleQuoted {
                    equals: anchor(&quoted_source, 3, 4),
                    open_quote: anchor(&quoted_source, 4, 5),
                    value: anchor(&quoted_source, 4, 6),
                    close_quote: anchor(&quoted_source, 6, 7),
                },
                "x".to_owned(),
                HtmlAttributeDisposition::Effective,
            )
        },
        HtmlTokenContractError::MisalignedBoundary {
            role: HtmlEvidenceRole::Value,
        },
    );
}
