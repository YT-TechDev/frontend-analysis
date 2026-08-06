use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};
use super::helpers::*;

pub(super) fn add_adversarial(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(complete_text(
        "ADV-001", FixtureCategory::Adversarial, "reconsume does not duplicate or omit authored bytes",
        "<1", "<1",
        vec![diagnostic(DiagnosticCode::InvalidFirstCharacterOfTagName, 1, 2, DiagnosticContext::TagOpen, DiagnosticHandling::Recovered(RecoveryKind::ReconsumedInData), DiagnosticSubject::InputLocation)], None,
    ));
    fixtures.push(complete_text(
        "ADV-002", FixtureCategory::Adversarial, "literal markup recovery for an invalid tag-name starter",
        "<>", "<>",
        vec![diagnostic(DiagnosticCode::InvalidFirstCharacterOfTagName, 1, 2, DiagnosticContext::TagOpen, DiagnosticHandling::Recovered(RecoveryKind::EmittedLiteralMarkupPrefix), DiagnosticSubject::InputLocation)], None,
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
    ));
    let adv5 = "<a\0>";
    fixtures.push(complete_tag_fixture(
        "ADV-005", "NUL in tag name retains raw byte and replacement interpretation", adv5,
        TokenKind::StartTag, 0, 4, 0, 1, 1, 3, "a\u{fffd}", Vec::new(), None, 3, 4,
        vec![diagnostic(DiagnosticCode::UnexpectedNullCharacter, 2, 3, DiagnosticContext::TagName, DiagnosticHandling::Recovered(RecoveryKind::ReplacedNullWithReplacementCharacter), DiagnosticSubject::EmittedToken(0))],
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
    ));
    let adv7 = "é<a x=界>ß";
    fixtures.push(complete(
        "ADV-007", FixtureCategory::Adversarial, "multi-byte scalars adjacent to captured token boundaries", adv7,
        vec![
            character(adv7, 0, 2, "é"),
            tag(adv7, TokenKind::StartTag, 2, 11, 2, 3, 3, 4, "a", vec![attribute_unquoted(adv7, 5, 10, 5, 6, "x", 6, 7, 7, 10, "界", AttributeDisposition::Effective)], None, 10, 11),
            character(adv7, 11, 13, "ß"),
            eof(13),
        ],
        Vec::new(), None, 1,
    ));
    fixtures.push(complete_text(
        "ADV-008", FixtureCategory::Adversarial, "equal-offset diagnostics preserve observation order", "\0", "\u{fffd}",
        vec![
            diagnostic(DiagnosticCode::ControlCharacterInInputStream, 0, 1, DiagnosticContext::InputPreprocessing, DiagnosticHandling::Continued, DiagnosticSubject::InputLocation),
            diagnostic(DiagnosticCode::UnexpectedNullCharacter, 0, 1, DiagnosticContext::Data, DiagnosticHandling::Recovered(RecoveryKind::ReplacedNullWithReplacementCharacter), DiagnosticSubject::InputLocation),
        ],
        None,
    ));
    let adv9 = "<a x='";
    fixtures.push(complete(
        "ADV-009", FixtureCategory::Adversarial, "EOF at an active quoted-value builder boundary abandons the tag", adv9,
        vec![eof(6)],
        vec![diagnostic(DiagnosticCode::EofInTag, 6, 6, DiagnosticContext::AttributeValueSingleQuoted, DiagnosticHandling::Recovered(RecoveryKind::AbandonedIncompleteTagAtEof), DiagnosticSubject::AbandonedInput(ByteSpan::new(0, 6)))],
        None, 0,
    ));
    let adv10 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fixtures.push(complete_text(
        "ADV-010", FixtureCategory::Adversarial,
        "long bounded input has deterministic counters without a performance claim",
        adv10, adv10, Vec::new(), None,
    ));
}
