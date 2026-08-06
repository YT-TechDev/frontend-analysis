use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};
use super::helpers::*;

#[rustfmt::skip]
pub(super) fn add_adversarial(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(complete_text(
        "ADV-001",
        FixtureCategory::Adversarial,
        "reconsume does not duplicate or omit authored bytes",
        "<1",
        "<1",
        vec![diagnostic(DiagnosticCode::InvalidFirstCharacterOfTagName, 1, 2, DiagnosticContext::TagOpen, DiagnosticHandling::Recovered(RecoveryKind::EmittedLiteralMarkupPrefix), DiagnosticSubject::InputLocation)],
        None,
        // Data(<)+TagOpen('1',reconsume Data)+Data(reconsume '1')+Data(EOF) = 4
        4,
    ));
    fixtures.push(complete_text(
        "ADV-002",
        FixtureCategory::Adversarial,
        "literal markup recovery for an invalid tag-name starter",
        "<>",
        "<>",
        vec![diagnostic(DiagnosticCode::InvalidFirstCharacterOfTagName, 1, 2, DiagnosticContext::TagOpen, DiagnosticHandling::Recovered(RecoveryKind::EmittedLiteralMarkupPrefix), DiagnosticSubject::InputLocation)],
        None,
        // Data(<)+TagOpen('>',reconsume Data)+Data(reconsume '>')+Data(EOF) = 4
        4,
    ));
    let adv3 = "<a x=1 y='2' z=\"3\">";
    fixtures.push(complete_tag_fixture(
        "ADV-003", "multiple adjacent attributes with mixed quoting", adv3,
        TokenKind::StartTag, 0, 19, 0, 1, 1, 2, "a",
        vec![
            attribute_unquoted(adv3, 3, 6, 3, 4, "x", 4, 5, 5, 6, "1", AttributeDisposition::Effective),
            attribute_single(adv3, 7, 12, 7, 8, "y", 8, 9, 9, 10, 10, 11, 11, 12, "2", AttributeDisposition::Effective),
            attribute_double(adv3, 13, 18, 13, 14, "z", 14, 15, 15, 16, 16, 17, 17, 18, "3", AttributeDisposition::Effective),
        ],
        None, 18, 19, Vec::new(),
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(x,create,reconsume)
        // +AttrName(reconsume x)+AttrName(=)+BeforeAttrValue(1,reconsume Unquoted)+Unquoted(reconsume 1)
        // +Unquoted(sp,BeforeAttrName)+BeforeAttrName(y,create,reconsume)+AttrName(reconsume y)+AttrName(=)
        // +BeforeAttrValue(')+SingleQuoted(2,append)+SingleQuoted(',close)+AfterAttrValueQuoted(sp)
        // +BeforeAttrName(z,create,reconsume)+AttrName(reconsume z)+AttrName(=)+BeforeAttrValue(")
        // +DoubleQuoted(3,append)+DoubleQuoted(",close)+AfterAttrValueQuoted(>,emit)+Data(EOF) = 25
        25,
    ));
    let adv4 = "<a X x>";
    fixtures.push(complete_tag_fixture(
        "ADV-004", "duplicate attributes compare interpreted names across authored case", adv4,
        TokenKind::StartTag, 0, 7, 0, 1, 1, 2, "a",
        vec![
            attribute_missing(adv4, 3, 4, 3, 4, "x", AttributeDisposition::Effective),
            attribute_missing(adv4, 5, 6, 5, 6, "x", AttributeDisposition::DuplicateOf(0)),
        ],
        None, 6, 7,
        vec![diagnostic(DiagnosticCode::DuplicateAttribute, 5, 6, DiagnosticContext::AttributeName, DiagnosticHandling::Recovered(RecoveryKind::PreservedDuplicateAttributeOccurrence), DiagnosticSubject::EmittedToken(0))],
        // same shape as ERR-015 = 13
        13,
    ));
    let adv5 = "<a\0>";
    fixtures.push(complete_tag_fixture(
        "ADV-005", "NUL in tag name retains raw byte and replacement interpretation", adv5,
        TokenKind::StartTag, 0, 4, 0, 1, 1, 3, "a\u{fffd}", Vec::new(), None, 3, 4,
        vec![diagnostic(DiagnosticCode::UnexpectedNullCharacter, 2, 3, DiagnosticContext::TagName, DiagnosticHandling::Recovered(RecoveryKind::ReplacedNullWithReplacementCharacter), DiagnosticSubject::EmittedToken(0))],
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(NUL,append)+TagName(>,emit)+Data(EOF) = 6
        6,
    ));
    let adv6 = "<a x\0=y\0>";
    fixtures.push(complete_tag_fixture(
        "ADV-006", "NUL in attribute name and value", adv6,
        TokenKind::StartTag, 0, 9, 0, 1, 1, 2, "a",
        vec![attribute_unquoted(adv6, 3, 8, 3, 5, "x\u{fffd}", 5, 6, 6, 8, "y\u{fffd}", AttributeDisposition::Effective)],
        None, 8, 9,
        vec![
            diagnostic(DiagnosticCode::UnexpectedNullCharacter, 4, 5, DiagnosticContext::AttributeName, DiagnosticHandling::Recovered(RecoveryKind::ReplacedNullWithReplacementCharacter), DiagnosticSubject::EmittedToken(0)),
            diagnostic(DiagnosticCode::UnexpectedNullCharacter, 7, 8, DiagnosticContext::AttributeValueUnquoted, DiagnosticHandling::Recovered(RecoveryKind::ReplacedNullWithReplacementCharacter), DiagnosticSubject::EmittedToken(0)),
        ],
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(x,create,reconsume)
        // +AttrName(reconsume x)+AttrName(NUL,append)+AttrName(=)+BeforeAttrValue(y,reconsume Unquoted)
        // +Unquoted(reconsume y)+Unquoted(NUL,append)+Unquoted(>,emit)+Data(EOF) = 13
        13,
    ));
    let adv7 = "é<a x=界>ß";
    fixtures.push(complete(
        "ADV-007",
        FixtureCategory::Adversarial,
        "multi-byte scalars adjacent to captured token boundaries",
        adv7,
        vec![
            character(adv7, 0, 2, "é"),
            tag(adv7, TokenKind::StartTag, 2, 11, 2, 3, 3, 4, "a", vec![attribute_unquoted(adv7, 5, 10, 5, 6, "x", 6, 7, 7, 10, "界", AttributeDisposition::Effective)], None, 10, 11),
            character(adv7, 11, 13, "ß"),
            eof(13),
        ],
        Vec::new(),
        None,
        1,
        // Data(é)+Data(<)+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)
        // +BeforeAttrName(x,create,reconsume)+AttrName(reconsume x)+AttrName(=)
        // +BeforeAttrValue(界,reconsume Unquoted)+Unquoted(reconsume 界)+Unquoted(>,emit)
        // +Data(ß)+Data(EOF) = 13
        13,
    ));
    let adv8 = "<a =>";
    fixtures.push(complete_tag_fixture(
        "ADV-008", "equal-offset diagnostics preserve transition observation order", adv8,
        TokenKind::StartTag, 0, 5, 0, 1, 1, 2, "a",
        vec![attribute_missing(adv8, 3, 4, 3, 4, "=", AttributeDisposition::Effective)],
        None, 4, 5,
        vec![
            diagnostic(DiagnosticCode::UnexpectedEqualsSignBeforeAttributeName, 3, 4, DiagnosticContext::BeforeAttributeName, DiagnosticHandling::Recovered(RecoveryKind::StartedAttributeAtUnexpectedEqualsSign), DiagnosticSubject::EmittedToken(0)),
            diagnostic(DiagnosticCode::UnexpectedCharacterInAttributeName, 3, 4, DiagnosticContext::AttributeName, DiagnosticHandling::Continued, DiagnosticSubject::EmittedToken(0)),
        ],
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(=,start attr '=')
        // +AttrName(>,reconsume AfterAttrName)+AfterAttrName(reconsume >,emit)+Data(EOF) = 8
        8,
    ));
    let adv9 = "<a x='";
    fixtures.push(complete(
        "ADV-009",
        FixtureCategory::Adversarial,
        "EOF at an active quoted-value builder boundary abandons the tag",
        adv9,
        vec![eof(6)],
        vec![diagnostic(DiagnosticCode::EofInTag, 6, 6, DiagnosticContext::AttributeValueSingleQuoted, DiagnosticHandling::Recovered(RecoveryKind::AbandonedIncompleteTagAtEof), DiagnosticSubject::AbandonedInput(ByteSpan::new(0, 6)))],
        None,
        0,
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(x,create,reconsume)
        // +AttrName(reconsume x)+AttrName(=)+BeforeAttrValue(')+SingleQuoted(EOF,EofInTag) = 9
        9,
    ));
    let adv10 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fixtures.push(complete_text(
        "ADV-010",
        FixtureCategory::Adversarial,
        "long bounded input has deterministic counters without a performance claim",
        adv10,
        adv10,
        Vec::new(),
        None,
        // 64 Data('a') steps + Data(EOF) = 65
        65,
    ));
}
