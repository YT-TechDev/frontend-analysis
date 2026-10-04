//! The exhaustive private tokenizer state enum.
//!
//! The five Doctype* states implement only the selected canonical
//! `<!DOCTYPE html>` family; they are not a general DOCTYPE grammar.
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
    /// Entered from Data or RCDATA on an authored `&`, which has already been
    /// consumed by the single forward cursor but not yet interpreted. The
    /// private return-state owner records which selected context resumes; this
    /// state only chooses the branch and discovers or consumes nothing.
    CharacterReference,
    /// The whole selected Named operation: bounded non-committing discovery,
    /// preparation, evidence construction, matched-source consumption and
    /// commit, as one transition-level step. It is entered by reconsuming the
    /// first identifier scalar, so a matched identifier never costs one outer
    /// transition per authored byte.
    NamedCharacterReference,
    /// The unresolved candidate run, which closes at its own boundary before
    /// the authored delimiter is reconsumed in the selected return state.
    AmbiguousAmpersand,
    /// Entered after the authored `#`; the next unit selects the radix or
    /// recovers the literal `&#` prefix.
    NumericCharacterReference,
    /// Entered after `x`/`X`; the first hexadecimal digit is reconsumed in
    /// [`State::HexadecimalCharacterReference`].
    HexadecimalCharacterReferenceStart,
    DecimalCharacterReference,
    HexadecimalCharacterReference,
    /// Input-free: consumes and examines no unit, yet costs one transition.
    /// It is never dispatched with an input unit.
    NumericCharacterReferenceEnd,
    /// Entered from TagOpen only after bounded non-committing recognition
    /// proved the next seven unconsumed bytes are ASCII-CI `DOCTYPE`. Each
    /// letter is still consumed by the single forward cursor, one unit per
    /// transition.
    DoctypeKeyword,
    /// Requires at least one ASCII whitespace unit after the keyword.
    DoctypeAfterKeyword,
    /// Skips further ASCII whitespace; only the selected name may follow.
    DoctypeBeforeName,
    /// Consumes exactly the four ASCII-CI letters of the selected name `html`.
    DoctypeName,
    /// Skips optional ASCII whitespace and accepts only the closing `>`.
    DoctypeAfterName,
}

/// The tokenizer-private return owner shared by the character reference
/// states. Exactly the two selected contexts are representable: it is not a
/// general return target, and tree construction never owns or imports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CharacterReferenceReturnState {
    Data,
    Rcdata,
}

impl CharacterReferenceReturnState {
    pub(super) fn state(self) -> State {
        match self {
            Self::Data => State::Data,
            Self::Rcdata => State::Rcdata,
        }
    }
}
