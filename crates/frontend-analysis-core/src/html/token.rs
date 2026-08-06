use std::error::Error;
use std::fmt;

use crate::{SourceAnchor, SourceId, SourceText};

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
        Ok(Self {
            source,
            interpreted,
        })
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
        Ok(Self {
            source,
            interpreted,
        })
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
            validate_nested(&complete, anchor, role)?;
        }

        let expected_open = match kind {
            HtmlTagKind::Start => "<",
            HtmlTagKind::End => "</",
        };
        exact(
            &open_delimiter,
            HtmlEvidenceRole::OpenDelimiter,
            expected_open,
        )?;
        exact(&close_delimiter, HtmlEvidenceRole::CloseDelimiter, ">")?;
        if open_delimiter.range().start() != complete.range().start()
            || close_delimiter.range().end() != complete.range().end()
        {
            return Err(HtmlTokenContractError::MisalignedBoundary {
                role: HtmlEvidenceRole::Tag,
            });
        }
        ordered(&open_delimiter, name.source(), HtmlEvidenceRole::TagName)?;
        ordered(
            name.source(),
            &close_delimiter,
            HtmlEvidenceRole::CloseDelimiter,
        )?;

        let mut previous = name.source();
        for (index, attribute) in attributes.iter().enumerate() {
            validate_nested(&complete, attribute.complete(), HtmlEvidenceRole::Attribute)?;
            ordered(previous, attribute.complete(), HtmlEvidenceRole::Attribute)?;
            validate_attribute_disposition(&attributes, index)?;
            previous = attribute.complete();
        }

        if let Some(solidus) = &self_closing_solidus {
            validate_nested(&complete, solidus, HtmlEvidenceRole::SelfClosingSolidus)?;
            exact(solidus, HtmlEvidenceRole::SelfClosingSolidus, "/")?;
            ordered(previous, solidus, HtmlEvidenceRole::SelfClosingSolidus)?;
            if solidus.range().end() != close_delimiter.range().start() {
                return Err(HtmlTokenContractError::InvalidSelfClosingPosition);
            }
        } else {
            ordered(previous, &close_delimiter, HtmlEvidenceRole::CloseDelimiter)?;
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
        Ok(Self {
            skipped_leading_bom,
        })
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
    UnexpectedEffectiveDuplicate {
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
        } => validate_quoted(complete, name, equals, open_quote, value, close_quote, "\"")?,
        HtmlAttributeValueSyntax::SingleQuoted {
            equals,
            open_quote,
            value,
            close_quote,
        } => validate_quoted(complete, name, equals, open_quote, value, close_quote, "'")?,
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

fn validate_attribute_disposition(
    attributes: &[HtmlAttributeEvidence],
    attribute_index: usize,
) -> Result<(), HtmlTokenContractError> {
    let attribute = &attributes[attribute_index];
    match attribute.disposition() {
        HtmlAttributeDisposition::Effective => {
            if let Some(first_index) = attributes[..attribute_index]
                .iter()
                .position(|earlier| earlier.name().interpreted() == attribute.name().interpreted())
            {
                return Err(HtmlTokenContractError::UnexpectedEffectiveDuplicate {
                    attribute_index,
                    first_index,
                });
            }
        }
        HtmlAttributeDisposition::DuplicateOf { first_index } => {
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
        }
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

fn non_empty(anchor: &SourceAnchor, role: HtmlEvidenceRole) -> Result<(), HtmlTokenContractError> {
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
    if nested.range().start() < owner.range().start() || nested.range().end() > owner.range().end()
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
