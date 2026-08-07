//! Private, narrow builders for incomplete tags and attributes.
//!
//! Builders own only temporary construction state and raw offsets captured
//! at recognition time. They never expose an incomplete production token;
//! finalization into an approved #110 token contract happens only once an
//! attribute's value syntax or a tag's closing delimiter is fully known, and
//! borrows `SourceText` just long enough to build the anchors.

use crate::SourceText;
use crate::html::token::{
    HtmlAttributeDisposition, HtmlAttributeEvidence, HtmlAttributeValueSyntax, HtmlNameEvidence,
    HtmlTagKind, HtmlTagToken,
};

use super::super::diagnostic::{
    HtmlTokenizerDiagnosticCode, HtmlTokenizerDiagnosticContext, HtmlTokenizerDiagnosticHandling,
};

/// Emission-conditioned diagnostic evidence recognized while a tag is under
/// construction, held privately on the builder until the corresponding
/// end-tag token actually emits. `EndTagWithAttributes` and
/// `EndTagWithTrailingSolidus` are the only emission-conditioned codes
/// (#111): their underlying parse-error fact exists only once the end-tag
/// token emission commits. If the tag never emits, this evidence is simply
/// dropped along with the builder rather than becoming a diagnostic.
pub(super) struct PendingEmissionDiagnostic {
    pub(super) code: HtmlTokenizerDiagnosticCode,
    pub(super) location: (usize, usize),
    pub(super) context: HtmlTokenizerDiagnosticContext,
    pub(super) handling: HtmlTokenizerDiagnosticHandling,
    /// Number of diagnostics already committed to the run's diagnostics
    /// vector at the moment this evidence was recognized. If the tag emits,
    /// this evidence is inserted back at exactly this position rather than
    /// appended at the end, since diagnostics committed afterward (with
    /// equal or later source starts) must not be reordered.
    pub(super) insertion_index: usize,
}

/// The name portion of an attribute under construction.
pub(super) struct AttributeNameBuilder {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) interpreted: String,
    /// The duplicate-vs-effective disposition, determined once when the
    /// tokenizer leaves the attribute-name state (per the pinned WHATWG
    /// snapshot) and read back unchanged at value finalization. `None`
    /// while the name is still being examined.
    pub(super) disposition: Option<HtmlAttributeDisposition>,
}

impl AttributeNameBuilder {
    pub(super) fn new(start: usize) -> Self {
        Self {
            start,
            end: start,
            interpreted: String::new(),
            disposition: None,
        }
    }

    pub(super) fn push(&mut self, ch: char, raw_end: usize) {
        self.interpreted.push(ch);
        self.end = raw_end;
    }
}

/// The value portion of an attribute under construction, once its syntax
/// kind is known (unquoted or quoted).
pub(super) enum AttributeValueBuilder {
    Unquoted {
        equals: (usize, usize),
        value_start: usize,
        value_end: usize,
        interpreted: String,
    },
    Quoted {
        double: bool,
        equals: (usize, usize),
        open_quote: (usize, usize),
        value_start: usize,
        value_end: usize,
        interpreted: String,
    },
}

impl AttributeValueBuilder {
    pub(super) fn new_unquoted(equals: (usize, usize), start: usize) -> Self {
        Self::Unquoted {
            equals,
            value_start: start,
            value_end: start,
            interpreted: String::new(),
        }
    }

    pub(super) fn new_quoted(
        double: bool,
        equals: (usize, usize),
        open_quote: (usize, usize),
    ) -> Self {
        let start = open_quote.1;
        Self::Quoted {
            double,
            equals,
            open_quote,
            value_start: start,
            value_end: start,
            interpreted: String::new(),
        }
    }

    pub(super) fn push(&mut self, ch: char, raw_end: usize) {
        match self {
            Self::Unquoted {
                value_end,
                interpreted,
                ..
            }
            | Self::Quoted {
                value_end,
                interpreted,
                ..
            } => {
                interpreted.push(ch);
                *value_end = raw_end;
            }
        }
    }

    pub(super) fn interpreted_len(&self) -> usize {
        match self {
            Self::Unquoted { interpreted, .. } | Self::Quoted { interpreted, .. } => {
                interpreted.len()
            }
        }
    }
}

/// A start or end tag under construction, owning its name, finalized
/// attributes, and pending in-progress attribute (if any).
pub(super) struct TagBuilder {
    pub(super) kind: HtmlTagKind,
    pub(super) tag_start: usize,
    /// The exact raw span of the opening delimiter (`<` or `</`) captured at
    /// the moment each of its characters was examined by the cursor, not
    /// reconstructed afterward from tag kind and a fixed width.
    pub(super) open_delimiter: (usize, usize),
    pub(super) name_start: usize,
    pub(super) name_end: usize,
    pub(super) interpreted_name: String,
    pub(super) attributes: Vec<HtmlAttributeEvidence>,
    pub(super) pending_name: Option<AttributeNameBuilder>,
    pub(super) pending_value: Option<AttributeValueBuilder>,
    /// The exact raw span of `=` consumed for the current pending attribute,
    /// captured at recognition time by every transition that enters
    /// `BeforeAttributeValue`. Owned per-tag so it cannot survive into a
    /// different tag's attribute lifecycle; always overwritten before use
    /// because `BeforeAttributeValue` is reachable only immediately after a
    /// `=` consumption.
    pub(super) pending_equals: Option<(usize, usize)>,
    pub(super) self_closing_solidus: Option<(usize, usize)>,
    /// Index into the run's diagnostics list where this tag's construction
    /// began, so an EOF abandonment can retarget its provisional
    /// `EmittedToken` subjects to `AbandonedInput`.
    pub(super) diagnostics_start: usize,
    /// Emission-conditioned diagnostic evidence recognized so far, held
    /// privately until (and unless) this tag's token actually emits. See
    /// [`PendingEmissionDiagnostic`].
    pub(super) pending_emission_diagnostics: Vec<PendingEmissionDiagnostic>,
}

impl TagBuilder {
    pub(super) fn new(
        kind: HtmlTagKind,
        tag_start: usize,
        open_delimiter: (usize, usize),
        name_start: usize,
        diagnostics_start: usize,
    ) -> Self {
        Self {
            kind,
            tag_start,
            open_delimiter,
            name_start,
            name_end: name_start,
            interpreted_name: String::new(),
            attributes: Vec::new(),
            pending_name: None,
            pending_value: None,
            pending_equals: None,
            self_closing_solidus: None,
            diagnostics_start,
            pending_emission_diagnostics: Vec::new(),
        }
    }

    pub(super) fn push_name(&mut self, ch: char, raw_end: usize) {
        self.interpreted_name.push(ch);
        self.name_end = raw_end;
    }

    pub(super) fn duplicate_disposition(&self, interpreted_name: &str) -> HtmlAttributeDisposition {
        match self
            .attributes
            .iter()
            .position(|existing| existing.name().interpreted() == interpreted_name)
        {
            Some(first_index) => HtmlAttributeDisposition::DuplicateOf { first_index },
            None => HtmlAttributeDisposition::Effective,
        }
    }

    /// Interpreted bytes currently retained by this builder: the tag name
    /// plus every finalized attribute plus any pending name/value under
    /// construction. Returns `None` on checked accumulation overflow rather
    /// than panicking or wrapping.
    pub(super) fn retained_interpreted_bytes(&self) -> Option<usize> {
        let mut total = self.interpreted_name.len();
        for attribute in &self.attributes {
            total = total.checked_add(attribute.name().interpreted().len())?;
            total = total.checked_add(attribute.interpreted_value().len())?;
        }
        if let Some(name) = &self.pending_name {
            total = total.checked_add(name.interpreted.len())?;
        }
        if let Some(value) = &self.pending_value {
            total = total.checked_add(value.interpreted_len())?;
        }
        Some(total)
    }
}

/// Builds a finalized `HtmlAttributeEvidence` for a `Missing` value (no `=`
/// ever appeared after the attribute name).
pub(super) fn finalize_missing_attribute(
    source: &SourceText,
    name: &AttributeNameBuilder,
    disposition: HtmlAttributeDisposition,
) -> HtmlAttributeEvidence {
    let complete = source
        .anchor(name.start, name.end)
        .expect("valid attribute name range");
    let name_anchor = source
        .anchor(name.start, name.end)
        .expect("valid attribute name range");
    let name_evidence = HtmlNameEvidence::new(name_anchor, name.interpreted.clone())
        .expect("non-empty attribute name");
    HtmlAttributeEvidence::new(
        complete,
        name_evidence,
        HtmlAttributeValueSyntax::Missing,
        String::new(),
        disposition,
    )
    .expect("valid missing-value attribute evidence")
}

/// Builds a finalized `HtmlAttributeEvidence` where `=` appeared but no
/// value followed before the tag closed.
pub(super) fn finalize_missing_after_equals_attribute(
    source: &SourceText,
    name: &AttributeNameBuilder,
    equals: (usize, usize),
    boundary: usize,
    disposition: HtmlAttributeDisposition,
) -> HtmlAttributeEvidence {
    let complete = source
        .anchor(name.start, boundary)
        .expect("valid attribute range");
    let name_anchor = source
        .anchor(name.start, name.end)
        .expect("valid attribute name range");
    let name_evidence = HtmlNameEvidence::new(name_anchor, name.interpreted.clone())
        .expect("non-empty attribute name");
    let equals_anchor = source
        .anchor(equals.0, equals.1)
        .expect("valid equals range");
    let boundary_anchor = source
        .anchor(boundary, boundary)
        .expect("valid boundary range");
    HtmlAttributeEvidence::new(
        complete,
        name_evidence,
        HtmlAttributeValueSyntax::MissingAfterEquals {
            equals: equals_anchor,
            value_boundary: boundary_anchor,
        },
        String::new(),
        disposition,
    )
    .expect("valid missing-after-equals attribute evidence")
}

/// Builds a finalized `HtmlAttributeEvidence` from a completed value
/// builder (unquoted or quoted).
pub(super) fn finalize_valued_attribute(
    source: &SourceText,
    name: &AttributeNameBuilder,
    value: &AttributeValueBuilder,
    close_quote: Option<(usize, usize)>,
    disposition: HtmlAttributeDisposition,
) -> HtmlAttributeEvidence {
    let name_anchor = source
        .anchor(name.start, name.end)
        .expect("valid attribute name range");
    let name_evidence = HtmlNameEvidence::new(name_anchor, name.interpreted.clone())
        .expect("non-empty attribute name");
    let (complete_end, syntax) = match value {
        AttributeValueBuilder::Unquoted {
            equals,
            value_start,
            value_end,
            ..
        } => (
            *value_end,
            HtmlAttributeValueSyntax::Unquoted {
                equals: source
                    .anchor(equals.0, equals.1)
                    .expect("valid equals range"),
                value: source
                    .anchor(*value_start, *value_end)
                    .expect("valid unquoted value range"),
            },
        ),
        AttributeValueBuilder::Quoted {
            double,
            equals,
            open_quote,
            value_start,
            value_end,
            ..
        } => {
            let (close_start, close_end) = close_quote
                .expect("close quote span captured at recognition time for quoted attribute value");
            let quote_syntax = if *double {
                HtmlAttributeValueSyntax::DoubleQuoted {
                    equals: source
                        .anchor(equals.0, equals.1)
                        .expect("valid equals range"),
                    open_quote: source
                        .anchor(open_quote.0, open_quote.1)
                        .expect("valid open quote range"),
                    value: source
                        .anchor(*value_start, *value_end)
                        .expect("valid quoted value range"),
                    close_quote: source
                        .anchor(close_start, close_end)
                        .expect("valid close quote range"),
                }
            } else {
                HtmlAttributeValueSyntax::SingleQuoted {
                    equals: source
                        .anchor(equals.0, equals.1)
                        .expect("valid equals range"),
                    open_quote: source
                        .anchor(open_quote.0, open_quote.1)
                        .expect("valid open quote range"),
                    value: source
                        .anchor(*value_start, *value_end)
                        .expect("valid quoted value range"),
                    close_quote: source
                        .anchor(close_start, close_end)
                        .expect("valid close quote range"),
                }
            };
            (close_end, quote_syntax)
        }
    };
    let complete = source
        .anchor(name.start, complete_end)
        .expect("valid attribute complete range");
    let interpreted_value = match value {
        AttributeValueBuilder::Unquoted { interpreted, .. }
        | AttributeValueBuilder::Quoted { interpreted, .. } => interpreted.clone(),
    };
    HtmlAttributeEvidence::new(
        complete,
        name_evidence,
        syntax,
        interpreted_value,
        disposition,
    )
    .expect("valid valued attribute evidence")
}

/// Builds the finalized `HtmlTagToken` from a completed builder and its
/// closing `>` delimiter.
pub(super) fn finalize_tag(
    source: &SourceText,
    builder: TagBuilder,
    close_delimiter: (usize, usize),
) -> HtmlTagToken {
    let complete_end = close_delimiter.1;
    let complete = source
        .anchor(builder.tag_start, complete_end)
        .expect("valid tag complete range");
    let open_delimiter = source
        .anchor(builder.open_delimiter.0, builder.open_delimiter.1)
        .expect("valid open delimiter range");
    let name_anchor = source
        .anchor(builder.name_start, builder.name_end)
        .expect("valid tag name range");
    let name_evidence =
        HtmlNameEvidence::new(name_anchor, builder.interpreted_name).expect("non-empty tag name");
    let self_closing_solidus = builder
        .self_closing_solidus
        .map(|(start, end)| source.anchor(start, end).expect("valid solidus range"));
    let close_delimiter_anchor = source
        .anchor(close_delimiter.0, close_delimiter.1)
        .expect("valid close delimiter range");
    HtmlTagToken::new(
        builder.kind,
        complete,
        open_delimiter,
        name_evidence,
        builder.attributes,
        self_closing_solidus,
        close_delimiter_anchor,
    )
    .expect("valid finalized tag token")
}
