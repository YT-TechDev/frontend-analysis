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
    CharacterReference,
    NamedCharacterReference,
    AmbiguousAmpersand,
}
