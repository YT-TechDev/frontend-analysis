use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceRange, SourceText};

#[derive(Debug, Clone)]
pub(crate) enum HtmlToken {
    Character(HtmlCharacterToken),
    Tag(HtmlTagToken),
    EndOfFile(HtmlEndOfFileToken),
}

#[derive(Clone)]
pub(crate) struct HtmlCharacterToken {
    source: SourceAnchor,
    interpreted: String,
}

impl HtmlCharacterToken {
    pub(crate) fn new(
        source: SourceAnchor,
        interpreted: String,
    ) -> Result<Self, HtmlTokenContractError> {
        non_empty(&source, HtmlEvidenceRole::Character)?;
        if interpreted.is_empty() {
            return Err(HtmlTokenContractError::EmptyInterpretedValue {
                role: HtmlEvidenceRole::Character,
            });
        }
        Ok(Self { source, interpreted })
    }

    pub(crate) fn source(&self) -> &SourceAnchor {
        &self.source
    }

    pub(crate) fn interpreted(&self) -> &str {
        &self.interpreted
    }
}

impl fmt::Debug for HtmlCharacterToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlCharacterToken")
            .field("source_id", &self.source.source_id())
            .field("range", &self.source.range())
            .field("interpreted_byte_len", &self.interpreted.len())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlTagKind {
    Start,
    End,
}

#[derive(Clone)]
pub(crate) struct HtmlNameEvidence {
    source: SourceAnchor,
    interpreted: String,
}

impl HtmlNameEvidence {
    pub(crate) fn new(
        source: SourceAnchor,
        interpreted: String,
    ) -> Result<Self, HtmlTokenContractError> {
        non_empty(&source, HtmlEvidenceRole::Name)?;
        if interpreted.is_empty() {
            return Err(HtmlTokenContractError::EmptyInterpretedValue {
                role: HtmlEvidenceRole::Name,
            });
        }
        Ok(Self { source, interpreted })
    }

    pub(crate) fn source(&self) -> &SourceAnchor {
        &self.source
    }

    pub(crate) fn interpreted(&self) -> &str {
        &self.interpreted
    }
}

impl fmt::Debug for HtmlNameEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlNameEvidence")
            .field("source_id", &self.source.source_id())
            .field("range", &self.source.range())
            .field("interpreted_byte_len", &self.interpreted.len())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum HtmlAttributeValueSyntax {
    Missing,
    MissingAfterEquals {
        equals: SourceAnchor,
        value_boundary: SourceAnchor,
    },
    Unquoted {
        equals: SourceAnchor,
        value: SourceAnchor,
    },
    DoubleQuoted {
        equals: SourceAnchor,
        open_quote: SourceAnchor,
        value: SourceAnchor,
        close_quote: SourceAnchor,
    },
    SingleQuoted {
        equals: SourceAnchor,
        open_quote: SourceAnchor,
        value: SourceAnchor,
        close_quote: SourceAnchor,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlAttributeDisposition {
    Effective,
    DuplicateOf { first_index: usize },
}

#[derive(Clone)]
pub(crate) struct HtmlAttributeEvidence {
    complete: SourceAnchor,
    name: HtmlNameEvidence,
    value_syntax: HtmlAttributeValueSyntax,
    interpreted_value: String,
    disposition: HtmlAttributeDisposition,
}

impl HtmlAttributeEvidence {
    pub(crate) fn new(
        complete: SourceAnchor,
        name: HtmlNameEvidence,
        value_syntax: HtmlAttributeValueSyntax,
        interpreted_value: String,
        disposition: HtmlAttributeDisposition,
    ) -> Result<Self, HtmlTokenContractError> {
        non_empty(&complete, HtmlEvidenceRole::Attribute)?;
        same_source(&complete, name.source(), HtmlEvidenceRole::AttributeName)?;
        contained(&complete, name.source(), HtmlEvidenceRole::AttributeName)?;
        if complete.range().start() != name.source().range().start() {
            return Err(HtmlTokenContractError::MisalignedBoundary {
                role: HtmlEvidenceRole::AttributeName,
            });
        }
        validate_value_syntax(&complete, name.source(), &value_syntax, &interpreted_value)?;
        Ok(Self {
            complete,
            name,
            value_syntax,
            interpreted_value,
            disposition,
        })
    }

    pub(crate) fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    pub(crate) fn name(&self) -> &HtmlNameEvidence {
        &self.name
    }

    pub(crate) fn value_syntax(&self) -> &HtmlAttributeValueSyntax {
        &self.value_syntax
    }

    pub(crate) fn interpreted_value(&self) -> &str {
        &self.interpreted_value
    }

    pub(crate) fn disposition(&self) -> HtmlAttributeDisposition {
        self.disposition
    }
}

impl fmt::Debug for HtmlAttributeEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlAttributeEvidence")
            .field("source_id", &self.complete.source_id())
            .field("range", &self.complete.range())
            .field("name", &self.name)
            .field("value_syntax", &self.value_syntax)
            .field("interpreted_value_byte_len", &self.interpreted_value.len())
            .field("disposition", &self.disposition)
            .finish()
    }
}

#[derive(Clone)]
pub(crate) struct HtmlTagToken {
    kind: HtmlTagKind,
    complete: SourceAnchor,
    open_delimiter: SourceAnchor,
    name: HtmlNameEvidence,
    attributes: Vec<HtmlAttributeEvidence>,
    self_closing_solidus: Option<SourceAnchor>,
    close_delimiter: SourceAnchor,
}

impl HtmlTagToken {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        kind: HtmlTagKind,
        complete: SourceAnchor,
        open_delimiter: SourceAnchor,
        name: HtmlNameEvidence,
        attributes: Vec<HtmlAttributeEvidence>,
        self_closing_solidus: Option<SourceAnchor>,
        close_delimiter: SourceAnchor,
    ) -> Result<Self, HtmlTokenContractError> {
        non_empty(&complete, HtmlEvidenceRole::Tag)?;
        for (role, anchor) in [
            (HtmlEvidenceRole::OpenDelimiter, &open_delimiter),
            (HtmlEvidenceRole::TagName, name.source()),
            (HtmlEvidenceRole::CloseDelimiter, &close_delimiter),
        ] {
            same_source(&complete, anchor, role)?;
            contained(&complete, anchor, role)?;
        }

        let expected_open = match kind {
            HtmlTagKind::Start => "<",
            HtmlTagKind::End => "</",
        };
        exact(&open_delimiter, HtmlEvidenceRole::OpenDelimiter, expected_open)?;
        exact(&close_delimiter, HtmlEvidenceRole::CloseDelimiter, ">")?;
        if open_delimiter.range().start() != complete.range().start()
            || close_delimiter.range().end() != complete.range().end()
        {
            return Err(HtmlTokenContractError::MisalignedBoundary {
                role: HtmlEvidenceRole::Tag,
            });
        }
        ordered(&open_delimiter, name.source(), HtmlEvidenceRole::TagName)?;
        ordered(name.source(), &close_delimiter, HtmlEvidenceRole::CloseDelimiter)?;

        let mut previous = name.source().range();
        for (index, attribute) in attributes.iter().enumerate() {
            same_source(&complete, attribute.complete(), HtmlEvidenceRole::Attribute)?;
            contained(&complete, attribute.complete(), HtmlEvidenceRole::Attribute)?;
            if attribute.complete().range().start() < previous.end() {
                return Err(HtmlTokenContractError::InvalidOrder {
                    role: HtmlEvidenceRole::Attribute,
                });
            }
            validate_duplicate(&attributes, index)?;
            previous = attribute.complete().range();
        }

        if let Some(solidus) = &self_closing_solidus {
            same_source(&complete, solidus, HtmlEvidenceRole::SelfClosingSolidus)?;
            contained(&complete, solidus, HtmlEvidenceRole::SelfClosingSolidus)?;
            exact(solidus, HtmlEvidenceRole::SelfClosingSolidus, "/")?;
            if solidus.range().start() < previous.end()
                || solidus.range().end() != close_delimiter.range().start()
            {
                return Err(HtmlTokenContractError::InvalidSelfClosingPosition);
            }
        } else if previous.end() > close_delimiter.range().start() {
            return Err(HtmlTokenContractError::InvalidOrder {
                role: HtmlEvidenceRole::CloseDelimiter,
            });
        }

        Ok(Self {
            kind,
            complete,
            open_delimiter,
            name,
            attributes,
            self_closing_solidus,
            close_delimiter,
        })
    }

    pub(crate) fn kind(&self) -> HtmlTagKind {
        self.kind
    }

    pub(crate) fn complete(&self) -> &SourceAnchor {
        &self.complete
    }

    pub(crate) fn open_delimiter(&self) -> &SourceAnchor {
        &self.open_delimiter
    }

    pub(crate) fn name(&self) -> &HtmlNameEvidence {
        &self.name
    }

    pub(crate) fn attributes(&self) -> &[HtmlAttributeEvidence] {
        &self.attributes
    }

    pub(crate) fn self_closing_solidus(&self) -> Option<&SourceAnchor> {
        self.self_closing_solidus.as_ref()
    }

    pub(crate) fn close_delimiter(&self) -> &SourceAnchor {
        &self.close_delimiter
    }
}

impl fmt::Debug for HtmlTagToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HtmlTagToken")
            .field("kind", &self.kind)
            .field("source_id", &self.complete.source_id())
            .field("range", &self.complete.range())
            .field("name", &self.name)
            .field("attributes", &self.attributes)
            .field(
                "self_closing_solidus_range",
                &self.self_closing_solidus.as_ref().map(SourceAnchor::range),
            )
            .finish()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HtmlEndOfFileToken {
    source: SourceAnchor,
}

impl HtmlEndOfFileToken {
    pub(crate) fn new(
        source_text: &SourceText,
        source: SourceAnchor,
    ) -> Result<Self, HtmlTokenContractError> {
        if source.source_id() != source_text.id() {
            return Err(HtmlTokenContractError::SourceIdentityMismatch {
                role: HtmlEvidenceRole::EndOfFile,
                expected: source_text.id(),
                actual: source.source_id(),
            });
        }
        if !source.range().is_empty() {
            return Err(HtmlTokenContractError::EndOfFileMustBeEmpty);
        }
        if source.range().start() != source_text.as_str().len() {
            return Err(HtmlTokenContractError::EndOfFileNotAtSourceEnd);
        }
        Ok(Self { source })
    }

    pub(crate) fn source(&self) -> &SourceAnchor {
        &self.source
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HtmlPreprocessingEvidence {
    skipped_leading_bom: Option<SourceAnchor>,
}

impl HtmlPreprocessingEvidence {
    pub(crate) fn new(
        source_text: &SourceText,
        skipped_leading_bom: Option<SourceAnchor>,
    ) -> Result<Self, HtmlTokenContractError> {
        if let Some(bom) = &skipped_leading_bom {
            if bom.source_id() != source_text.id() {
                return Err(HtmlTokenContractError::SourceIdentityMismatch {
                    role: HtmlEvidenceRole::LeadingBom,
                    expected: source_text.id(),
                    actual: bom.source_id(),
                });
            }
            exact(bom, HtmlEvidenceRole::LeadingBom, "\u{feff}")?;
            if bom.range().start() != 0 {
                return Err(HtmlTokenContractError::LeadingBomNotAtStart);
            }
        }
        Ok(Self { skipped_leading_bom })
    }

    pub(crate) fn skipped_leading_bom(&self) -> Option<&SourceAnchor> {
        self.skipped_leading_bom.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HtmlEvidenceRole {
    Character,
    Name,
    Tag,
    OpenDelimiter,
    TagName,
    Attribute,
    AttributeName,
    Equals,
    ValueBoundary,
    Value,
    OpenQuote,
    CloseQuote,
    SelfClosingSolidus,
    CloseDelimiter,
    EndOfFile,
    LeadingBom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HtmlTokenContractError {
    SourceIdentityMismatch {
        role: HtmlEvidenceRole,
        expected: SourceId,
        actual: SourceId,
    },
    EmptySourceRange {
        role: HtmlEvidenceRole,
    },
    EmptyInterpretedValue {
        role: HtmlEvidenceRole,
    },
    WrongAuthoredFragment {
        role: HtmlEvidenceRole,
        expected: &'static str,
    },
    RangeOutsideOwner {
        role: HtmlEvidenceRole,
    },
    InvalidOrder {
        role: HtmlEvidenceRole,
    },
    MisalignedBoundary {
        role: HtmlEvidenceRole,
    },
    MissingValueMustInterpretEmpty,
    MissingValueBoundaryMustBeEmpty,
    UnquotedValueMustBeNonEmpty,
    InvalidSelfClosingPosition,
    InvalidDuplicateReference {
        attribute_index: usize,
        first_index: usize,
    },
    DuplicateTargetMustBeEffective {
        attribute_index: usize,
        first_index: usize,
    },
    DuplicateNameMismatch {
        attribute_index: usize,
        first_index: usize,
    },
    EndOfFileMustBeEmpty,
    EndOfFileNotAtSourceEnd,
    LeadingBomNotAtStart,
}

impl fmt::Display for HtmlTokenContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HTML token contract violation: {self:?}")
    }
}

impl Error for HtmlTokenContractError {}

fn validate_value_syntax(
    complete: &SourceAnchor,
    name: &SourceAnchor,
    syntax: &HtmlAttributeValueSyntax,
    interpreted: &str,
) -> Result<(), HtmlTokenContractError> {
    match syntax {
        HtmlAttributeValueSyntax::Missing => {
            if !interpreted.is_empty() {
                return Err(HtmlTokenContractError::MissingValueMustInterpretEmpty);
            }
            if complete.range().end() != name.range().end() {
                return Err(HtmlTokenContractError::MisalignedBoundary {
                    role: HtmlEvidenceRole::Attribute,
                });
            }
        }
        HtmlAttributeValueSyntax::MissingAfterEquals {
            equals,
            value_boundary,
        } => {
            validate_nested(complete, equals, HtmlEvidenceRole::Equals)?;
            validate_nested(complete, value_boundary, HtmlEvidenceRole::ValueBoundary)?;
            exact(equals, HtmlEvidenceRole::Equals, "=")?;
            ordered(name, equals, HtmlEvidenceRole::Equals)?;
            ordered(equals, value_boundary, HtmlEvidenceRole::ValueBoundary)?;
            if !value_boundary.range().is_empty() {
                return Err(HtmlTokenContractError::MissingValueBoundaryMustBeEmpty);
            }
            if value_boundary.range().end() != complete.range().end() {
                return Err(HtmlTokenContractError::MisalignedBoundary {
                    role: HtmlEvidenceRole::ValueBoundary,
                });
            }
            if !interpreted.is_empty() {
                return Err(HtmlTokenContractError::MissingValueMustInterpretEmpty);
            }
        }
        HtmlAttributeValueSyntax::Unquoted { equals, value } => {
            validate_nested(complete, equals, HtmlEvidenceRole::Equals)?;
            validate_nested(complete, value, HtmlEvidenceRole::Value)?;
            exact(equals, HtmlEvidenceRole::Equals, "=")?;
            ordered(name, equals, HtmlEvidenceRole::Equals)?;
            ordered(equals, value, HtmlEvidenceRole::Value)?;
            if value.range().is_empty() {
                return Err(HtmlTokenContractError::UnquotedValueMustBeNonEmpty);
            }
            if value.range().end() != complete.range().end() {
                return Err(HtmlTokenContractError::MisalignedBoundary {
                    role: HtmlEvidenceRole::Value,
                });
            }
        }
        HtmlAttributeValueSyntax::DoubleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => validate_quoted(
            complete,
            name,
            equals,
            open_quote,
            value,
            close_quote,
            "\"",
        )?,
        HtmlAttributeValueSyntax::SingleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => validate_quoted(
            complete,
            name,
            equals,
            open_quote,
            value,
            close_quote,
            "'",
        )?,
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_quoted(
    complete: &SourceAnchor,
    name: &SourceAnchor,
    equals: &SourceAnchor,
    open_quote: &SourceAnchor,
    value: &SourceAnchor,
    close_quote: &SourceAnchor,
    quote: &'static str,
) -> Result<(), HtmlTokenContractError> {
    for (role, anchor) in [
        (HtmlEvidenceRole::Equals, equals),
        (HtmlEvidenceRole::OpenQuote, open_quote),
        (HtmlEvidenceRole::Value, value),
        (HtmlEvidenceRole::CloseQuote, close_quote),
    ] {
        validate_nested(complete, anchor, role)?;
    }
    exact(equals, HtmlEvidenceRole::Equals, "=")?;
    exact(open_quote, HtmlEvidenceRole::OpenQuote, quote)?;
    exact(close_quote, HtmlEvidenceRole::CloseQuote, quote)?;
    ordered(name, equals, HtmlEvidenceRole::Equals)?;
    ordered(equals, open_quote, HtmlEvidenceRole::OpenQuote)?;
    if open_quote.range().end() != value.range().start()
        || value.range().end() != close_quote.range().start()
        || close_quote.range().end() != complete.range().end()
    {
        return Err(HtmlTokenContractError::MisalignedBoundary {
            role: HtmlEvidenceRole::Value,
        });
    }
    Ok(())
}

fn validate_duplicate(
    attributes: &[HtmlAttributeEvidence],
    attribute_index: usize,
) -> Result<(), HtmlTokenContractError> {
    let attribute = &attributes[attribute_index];
    let HtmlAttributeDisposition::DuplicateOf { first_index } = attribute.disposition() else {
        return Ok(());
    };
    if first_index >= attribute_index {
        return Err(HtmlTokenContractError::InvalidDuplicateReference {
            attribute_index,
            first_index,
        });
    }
    let first = &attributes[first_index];
    if first.disposition() != HtmlAttributeDisposition::Effective {
        return Err(HtmlTokenContractError::DuplicateTargetMustBeEffective {
            attribute_index,
            first_index,
        });
    }
    if first.name().interpreted() != attribute.name().interpreted() {
        return Err(HtmlTokenContractError::DuplicateNameMismatch {
            attribute_index,
            first_index,
        });
    }
    Ok(())
}

fn validate_nested(
    owner: &SourceAnchor,
    nested: &SourceAnchor,
    role: HtmlEvidenceRole,
) -> Result<(), HtmlTokenContractError> {
    same_source(owner, nested, role)?;
    contained(owner, nested, role)
}

fn non_empty(
    anchor: &SourceAnchor,
    role: HtmlEvidenceRole,
) -> Result<(), HtmlTokenContractError> {
    if anchor.range().is_empty() {
        return Err(HtmlTokenContractError::EmptySourceRange { role });
    }
    Ok(())
}

fn same_source(
    owner: &SourceAnchor,
    nested: &SourceAnchor,
    role: HtmlEvidenceRole,
) -> Result<(), HtmlTokenContractError> {
    if owner.source_id() != nested.source_id() {
        return Err(HtmlTokenContractError::SourceIdentityMismatch {
            role,
            expected: owner.source_id(),
            actual: nested.source_id(),
        });
    }
    Ok(())
}

fn contained(
    owner: &SourceAnchor,
    nested: &SourceAnchor,
    role: HtmlEvidenceRole,
) -> Result<(), HtmlTokenContractError> {
    if nested.range().start() < owner.range().start()
        || nested.range().end() > owner.range().end()
    {
        return Err(HtmlTokenContractError::RangeOutsideOwner { role });
    }
    Ok(())
}

fn ordered(
    earlier: &SourceAnchor,
    later: &SourceAnchor,
    role: HtmlEvidenceRole,
) -> Result<(), HtmlTokenContractError> {
    if later.range().start() < earlier.range().end() {
        return Err(HtmlTokenContractError::InvalidOrder { role });
    }
    Ok(())
}

fn exact(
    anchor: &SourceAnchor,
    role: HtmlEvidenceRole,
    expected: &'static str,
) -> Result<(), HtmlTokenContractError> {
    if anchor.fragment() != expected {
        return Err(HtmlTokenContractError::WrongAuthoredFragment { role, expected });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

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

    #[test]
    fn character_token_preserves_raw_and_interpreted_evidence() {
        let source = source(1, "aé\r\n\0");
        let token = HtmlCharacterToken::new(
            anchor(&source, 0, source.as_str().len()),
            "aé\n\u{fffd}".to_owned(),
        )
        .unwrap();

        assert_eq!(token.source().fragment(), "aé\r\n\0");
        assert_eq!(token.interpreted(), "aé\n\u{fffd}");
    }

    #[test]
    fn tag_and_attribute_contracts_preserve_authored_evidence() {
        let source = source(2, "<div ID=x id='y'>");
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

        let tag = HtmlTagToken::new(
            HtmlTagKind::Start,
            anchor(&source, 0, 17),
            anchor(&source, 0, 1),
            name(&source, 1, 4, "div"),
            vec![first, duplicate],
            None,
            anchor(&source, 16, 17),
        )
        .unwrap();

        assert_eq!(tag.kind(), HtmlTagKind::Start);
        assert_eq!(tag.complete().fragment(), "<div ID=x id='y'>");
        assert_eq!(tag.open_delimiter().fragment(), "<");
        assert_eq!(tag.name().source().fragment(), "div");
        assert_eq!(tag.attributes()[0].complete().fragment(), "ID=x");
        assert_eq!(tag.attributes()[1].complete().fragment(), "id='y'");
        assert_eq!(tag.close_delimiter().fragment(), ">");
        assert_eq!(
            tag.attributes()[1].disposition(),
            HtmlAttributeDisposition::DuplicateOf { first_index: 0 }
        );
    }

    #[test]
    fn attribute_value_forms_remain_distinct() {
        let boolean_source = source(3, "disabled");
        let boolean = boolean_attribute(&boolean_source, 0, 8, "disabled");
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

        let quoted_source = source(5, "foo=\"\"");
        let quoted = HtmlAttributeEvidence::new(
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
        assert!(matches!(
            quoted.value_syntax(),
            HtmlAttributeValueSyntax::DoubleQuoted { .. }
        ));
    }

    #[test]
    fn end_tag_and_self_closing_tag_delimiters_are_exact() {
        let end_source = source(6, "</DIV>");
        let end = HtmlTagToken::new(
            HtmlTagKind::End,
            anchor(&end_source, 0, 6),
            anchor(&end_source, 0, 2),
            name(&end_source, 2, 5, "div"),
            Vec::new(),
            None,
            anchor(&end_source, 5, 6),
        )
        .unwrap();
        assert_eq!(end.open_delimiter().fragment(), "</");

        let self_closing_source = source(7, "<img/>");
        let self_closing = HtmlTagToken::new(
            HtmlTagKind::Start,
            anchor(&self_closing_source, 0, 6),
            anchor(&self_closing_source, 0, 1),
            name(&self_closing_source, 1, 4, "img"),
            Vec::new(),
            Some(anchor(&self_closing_source, 4, 5)),
            anchor(&self_closing_source, 5, 6),
        )
        .unwrap();
        assert_eq!(
            self_closing.self_closing_solidus().unwrap().fragment(),
            "/"
        );
    }

    #[test]
    fn invalid_nested_sources_and_delimiters_return_typed_errors() {
        let owner = source(8, "<a>");
        let foreign = source(9, "a");
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

        let wrong = source(10, "[a>");
        assert_eq!(
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&wrong, 0, 3),
                anchor(&wrong, 0, 1),
                name(&wrong, 1, 2, "a"),
                Vec::new(),
                None,
                anchor(&wrong, 2, 3),
            )
            .unwrap_err(),
            HtmlTokenContractError::WrongAuthoredFragment {
                role: HtmlEvidenceRole::OpenDelimiter,
                expected: "<",
            }
        );
    }

    #[test]
    fn invalid_attribute_and_duplicate_states_are_rejected() {
        let source = source(11, "foo=");
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

        let duplicate_source = source(12, "<a x x>");
        let first = boolean_attribute(&duplicate_source, 3, 4, "x");
        let invalid = HtmlAttributeEvidence::new(
            anchor(&duplicate_source, 5, 6),
            name(&duplicate_source, 5, 6, "x"),
            HtmlAttributeValueSyntax::Missing,
            String::new(),
            HtmlAttributeDisposition::DuplicateOf { first_index: 1 },
        )
        .unwrap();
        assert_eq!(
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&duplicate_source, 0, 7),
                anchor(&duplicate_source, 0, 1),
                name(&duplicate_source, 1, 2, "a"),
                vec![first, invalid],
                None,
                anchor(&duplicate_source, 6, 7),
            )
            .unwrap_err(),
            HtmlTokenContractError::InvalidDuplicateReference {
                attribute_index: 1,
                first_index: 1,
            }
        );
    }

    #[test]
    fn eof_and_leading_bom_have_exact_boundaries() {
        let source = source(13, "abc");
        let eof = HtmlEndOfFileToken::new(&source, anchor(&source, 3, 3)).unwrap();
        assert!(eof.source().range().is_empty());
        assert_eq!(
            HtmlEndOfFileToken::new(&source, anchor(&source, 2, 2)).unwrap_err(),
            HtmlTokenContractError::EndOfFileNotAtSourceEnd
        );

        let bom_source = source(14, "\u{feff}<a>");
        let evidence = HtmlPreprocessingEvidence::new(
            &bom_source,
            Some(anchor(&bom_source, 0, 3)),
        )
        .unwrap();
        assert_eq!(
            evidence.skipped_leading_bom().unwrap().fragment(),
            "\u{feff}"
        );
    }

    #[test]
    fn debug_and_errors_redact_source_and_interpreted_content() {
        const MARKER: &str = "private-token-marker-51f2";
        let source = source(15, MARKER);
        let token = HtmlCharacterToken::new(
            anchor(&source, 0, source.as_str().len()),
            MARKER.to_owned(),
        )
        .unwrap();
        let debug = format!("{token:?}");
        assert!(!debug.contains(MARKER));
        assert!(debug.contains("interpreted_byte_len"));

        let error = HtmlCharacterToken::new(anchor(&source, 0, 0), String::new()).unwrap_err();
        assert!(!format!("{error:?}").contains(MARKER));
        assert!(!error.to_string().contains(MARKER));
    }

    #[test]
    fn invalid_construction_does_not_panic() {
        let source = source(16, "<a/ >");
        let result = catch_unwind(AssertUnwindSafe(|| {
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&source, 0, 5),
                anchor(&source, 0, 1),
                name(&source, 1, 2, "a"),
                Vec::new(),
                Some(anchor(&source, 2, 3)),
                anchor(&source, 4, 5),
            )
        }));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().unwrap_err(),
            HtmlTokenContractError::InvalidSelfClosingPosition
        );
    }

    #[test]
    fn token_variants_preserve_run_local_vec_order() {
        let source = source(17, "x");
        let tokens = [
            HtmlToken::Character(
                HtmlCharacterToken::new(anchor(&source, 0, 1), "x".to_owned()).unwrap(),
            ),
            HtmlToken::EndOfFile(
                HtmlEndOfFileToken::new(&source, anchor(&source, 1, 1)).unwrap(),
            ),
        ];
        assert!(matches!(&tokens[0], HtmlToken::Character(_)));
        assert!(matches!(&tokens[1], HtmlToken::EndOfFile(_)));

        let tag_source = source(18, "<a>");
        let tag = HtmlToken::Tag(
            HtmlTagToken::new(
                HtmlTagKind::Start,
                anchor(&tag_source, 0, 3),
                anchor(&tag_source, 0, 1),
                name(&tag_source, 1, 2, "a"),
                Vec::new(),
                None,
                anchor(&tag_source, 2, 3),
            )
            .unwrap(),
        );
        assert!(matches!(tag, HtmlToken::Tag(_)));
    }
}
