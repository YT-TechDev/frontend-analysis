//! The exhaustive private tokenizer state enum.
//!
//! TC-S9 extends the established Data-context subset with only the four
//! RAWTEXT states required by the selected InHead `<style>` lifecycle, and
//! TC-S10 adds only the four RCDATA states plus the three character-reference
//! states required by the selected InHead `<title>` lifecycle. This remains
//! private lexical implementation state; tree construction never owns or
//! imports it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum State {
    Data,
    TagOpen,
    EndTagOpen,
    TagName,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    RawText,
    RawTextLessThanSign,
    RawTextEndTagOpen,
    RawTextEndTagName,
    Rcdata,
    RcdataLessThanSign,
    RcdataEndTagOpen,
    RcdataEndTagName,
    /// Entered from RCDATA on an authored `&`, which has already been
    /// consumed by the single forward cursor but not yet interpreted. This
    /// state only chooses the branch; it discovers and consumes nothing.
    CharacterReference,
    /// The whole selected Named operation: bounded non-committing discovery,
    /// preparation, evidence construction, matched-source consumption and
    /// commit, as one transition-level step. It is entered by reconsuming the
    /// first identifier scalar, so a matched identifier never costs one outer
    /// transition per authored byte.
    NamedCharacterReference,
    /// The unresolved candidate run, which closes at its own boundary before
    /// the authored delimiter is reconsumed in RCDATA.
    AmbiguousAmpersand,
}
