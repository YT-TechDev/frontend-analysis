use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};
use super::helpers::*;

#[rustfmt::skip]
pub(super) fn add_diagnostics(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(complete_text(
        "ERR-001",
        FixtureCategory::Diagnostic,
        "NoncharacterInInputStream diagnostic",
        "\u{fdd0}",
        "\u{fdd0}",
        vec![diagnostic(DiagnosticCode::NoncharacterInInputStream, 0, 3, DiagnosticContext::InputPreprocessing, DiagnosticHandling::Continued, DiagnosticSubject::InputLocation)],
        None,
    ));
    fixtures.push(complete_text(
        "ERR-002",
        FixtureCategory::Diagnostic,
        "ControlCharacterInInputStream diagnostic",
        "\u{0001}",
        "\u{0001}",
        vec![diagnostic(DiagnosticCode::ControlCharacterInInputStream, 0, 1, DiagnosticContext::InputPreprocessing, DiagnosticHandling::Continued, DiagnosticSubject::InputLocation)],
        None,
    ));
    fixtures.push(complete_text(
        "ERR-003",
        FixtureCategory::Diagnostic,
        "UnexpectedNullCharacter replacement",
        "\0",
        "\u{fffd}",
        vec![diagnostic(DiagnosticCode::UnexpectedNullCharacter, 0, 1, DiagnosticContext::Data, DiagnosticHandling::Recovered(RecoveryKind::ReplacedNullWithReplacementCharacter), DiagnosticSubject::InputLocation)],
        None,
    ));
    fixtures.push(complete_text(
        "ERR-004",
        FixtureCategory::Diagnostic,
        "EOF before a tag name emits the literal markup prefix",
        "<",
        "<",
        vec![diagnostic(DiagnosticCode::EofBeforeTagName, 1, 1, DiagnosticContext::TagOpen, DiagnosticHandling::Recovered(RecoveryKind::EmittedLiteralMarkupPrefix), DiagnosticSubject::InputLocation)],
        None,
    ));
    fixtures.push(complete_text(
        "ERR-005",
        FixtureCategory::Diagnostic,
        "invalid first tag-name character emits the literal prefix and reconsumes in Data",
        "<1",
        "<1",
        vec![diagnostic(DiagnosticCode::InvalidFirstCharacterOfTagName, 1, 2, DiagnosticContext::TagOpen, DiagnosticHandling::Recovered(RecoveryKind::EmittedLiteralMarkupPrefix), DiagnosticSubject::InputLocation)],
        None,
    ));
    fixtures.push(incomplete(
        "ERR-006",
        FixtureCategory::Diagnostic,
        "question mark after tag open records the parse error before unsupported PI recovery",
        "<?",
        2,
        Vec::new(),
        vec![diagnostic(DiagnosticCode::UnexpectedQuestionMarkInsteadOfTagName, 1, 2, DiagnosticContext::TagOpen, DiagnosticHandling::Stopped, DiagnosticSubject::InputLocation)],
        Completion::Unsupported {
            capability: Capability::ProcessingInstruction,
            availability: Availability::Deferred,
            trigger: UnsupportedTrigger::Input(ByteSpan::new(2, 2)),
        },
        Limits::generous(),
        usage("<?", 2, 0, 1, 0, 0, 0),
    ));
    fixtures.push(complete(
        "ERR-007",
        FixtureCategory::Diagnostic,
        "missing end-tag name",
        "</>",
        vec![eof(3)],
        vec![diagnostic(DiagnosticCode::MissingEndTagName, 2, 3, DiagnosticContext::EndTagOpen, DiagnosticHandling::Recovered(RecoveryKind::IgnoredUnexpectedInput), DiagnosticSubject::InputLocation)],
        None,
        0,
    ));
    fixtures.push(complete(
        "ERR-008",
        FixtureCategory::Diagnostic,
        "EOF in tag abandons the incomplete builder",
        "<a",
        vec![eof(2)],
        vec![diagnostic(DiagnosticCode::EofInTag, 2, 2, DiagnosticContext::TagName, DiagnosticHandling::Recovered(RecoveryKind::AbandonedIncompleteTagAtEof), DiagnosticSubject::AbandonedInput(ByteSpan::new(0, 2)))],
        None,
        0,
    ));

    let err9 = "<a =x>";
    fixtures.push(complete_tag_fixture(
        "ERR-009", "equals sign starts an attribute name and is reconsumed in Attribute name", err9,
        TokenKind::StartTag, 0, 6, 0, 1, 1, 2, "a",
        vec![attribute_missing(err9, 3, 5, 3, 5, "=x", AttributeDisposition::Effective)],
        None, 5, 6,
        vec![
            diagnostic(DiagnosticCode::UnexpectedEqualsSignBeforeAttributeName, 3, 4, DiagnosticContext::BeforeAttributeName, DiagnosticHandling::Recovered(RecoveryKind::StartedAttributeAtUnexpectedEqualsSign), DiagnosticSubject::EmittedToken(0)),
            diagnostic(DiagnosticCode::UnexpectedCharacterInAttributeName, 3, 4, DiagnosticContext::AttributeName, DiagnosticHandling::Continued, DiagnosticSubject::EmittedToken(0)),
        ],
    ));
    let err10 = "<a x\"y>";
    fixtures.push(complete_tag_fixture(
        "ERR-010", "unexpected quote in an attribute name", err10,
        TokenKind::StartTag, 0, 7, 0, 1, 1, 2, "a",
        vec![attribute_missing(err10, 3, 6, 3, 6, "x\"y", AttributeDisposition::Effective)],
        None, 6, 7,
        vec![diagnostic(DiagnosticCode::UnexpectedCharacterInAttributeName, 4, 5, DiagnosticContext::AttributeName, DiagnosticHandling::Continued, DiagnosticSubject::EmittedToken(0))],
    ));
    let err11 = "<a x=>";
    fixtures.push(complete_tag_fixture(
        "ERR-011", "missing attribute value before close delimiter", err11,
        TokenKind::StartTag, 0, 6, 0, 1, 1, 2, "a",
        vec![attribute_missing_after_equals(err11, 3, 5, 3, 4, "x", 4, 5, 5, AttributeDisposition::Effective)],
        None, 5, 6,
        vec![diagnostic(DiagnosticCode::MissingAttributeValue, 5, 6, DiagnosticContext::BeforeAttributeValue, DiagnosticHandling::Recovered(RecoveryKind::CompletedTagWithMissingAttributeValue), DiagnosticSubject::EmittedToken(0))],
    ));
    let err12 = "<a x=a\"b>";
    fixtures.push(complete_tag_fixture(
        "ERR-012", "unexpected character in an unquoted attribute value", err12,
        TokenKind::StartTag, 0, 9, 0, 1, 1, 2, "a",
        vec![attribute_unquoted(err12, 3, 8, 3, 4, "x", 4, 5, 5, 8, "a\"b", AttributeDisposition::Effective)],
        None, 8, 9,
        vec![diagnostic(DiagnosticCode::UnexpectedCharacterInUnquotedAttributeValue, 6, 7, DiagnosticContext::AttributeValueUnquoted, DiagnosticHandling::Continued, DiagnosticSubject::EmittedToken(0))],
    ));
    let err13 = "<a x=\"y\"z>";
    fixtures.push(complete_tag_fixture(
        "ERR-013", "missing whitespace between adjacent attributes", err13,
        TokenKind::StartTag, 0, 10, 0, 1, 1, 2, "a",
        vec![
            attribute_double(err13, 3, 8, 3, 4, "x", 4, 5, 5, 6, 6, 7, 7, 8, "y", AttributeDisposition::Effective),
            attribute_missing(err13, 8, 9, 8, 9, "z", AttributeDisposition::Effective),
        ],
        None, 9, 10,
        vec![diagnostic(DiagnosticCode::MissingWhitespaceBetweenAttributes, 8, 9, DiagnosticContext::AfterAttributeValueQuoted, DiagnosticHandling::Recovered(RecoveryKind::ReconsumedBeforeAttributeName), DiagnosticSubject::EmittedToken(0))],
    ));
    let err14 = "<a /x>";
    fixtures.push(complete_tag_fixture(
        "ERR-014", "unexpected solidus is reconsumed before an attribute name", err14,
        TokenKind::StartTag, 0, 6, 0, 1, 1, 2, "a",
        vec![attribute_missing(err14, 4, 5, 4, 5, "x", AttributeDisposition::Effective)],
        None, 5, 6,
        vec![diagnostic(DiagnosticCode::UnexpectedSolidusInTag, 3, 4, DiagnosticContext::SelfClosingStartTag, DiagnosticHandling::Recovered(RecoveryKind::ReconsumedBeforeAttributeName), DiagnosticSubject::EmittedToken(0))],
    ));
    let err15 = "<a x x>";
    fixtures.push(complete_tag_fixture(
        "ERR-015", "duplicate attribute occurrence remains authored evidence", err15,
        TokenKind::StartTag, 0, 7, 0, 1, 1, 2, "a",
        vec![
            attribute_missing(err15, 3, 4, 3, 4, "x", AttributeDisposition::Effective),
            attribute_missing(err15, 5, 6, 5, 6, "x", AttributeDisposition::DuplicateOf(0)),
        ],
        None, 6, 7,
        vec![diagnostic(DiagnosticCode::DuplicateAttribute, 5, 6, DiagnosticContext::AttributeName, DiagnosticHandling::Recovered(RecoveryKind::PreservedDuplicateAttributeOccurrence), DiagnosticSubject::EmittedToken(0))],
    ));
    let err16 = "</a x>";
    fixtures.push(complete_tag_fixture(
        "ERR-016", "end tag with authored attribute evidence", err16,
        TokenKind::EndTag, 0, 6, 0, 2, 2, 3, "a",
        vec![attribute_missing(err16, 4, 5, 4, 5, "x", AttributeDisposition::Effective)],
        None, 5, 6,
        vec![diagnostic(DiagnosticCode::EndTagWithAttributes, 4, 5, DiagnosticContext::AttributeName, DiagnosticHandling::Recovered(RecoveryKind::PreservedEndTagLexicalEvidence), DiagnosticSubject::EmittedToken(0))],
    ));
    fixtures.push(complete_tag_fixture(
        "ERR-017", "end tag with trailing self-closing solidus", "</a/>",
        TokenKind::EndTag, 0, 5, 0, 2, 2, 3, "a", Vec::new(), Some((3, 4)), 4, 5,
        vec![diagnostic(DiagnosticCode::EndTagWithTrailingSolidus, 3, 4, DiagnosticContext::SelfClosingStartTag, DiagnosticHandling::Recovered(RecoveryKind::PreservedEndTagLexicalEvidence), DiagnosticSubject::EmittedToken(0))],
    ));
}
