use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};
use super::helpers::*;

pub(super) fn add_supported_tokens(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(complete_text(
        "TOK-001",
        FixtureCategory::SupportedToken,
        "contiguous Data-state character data",
        "abc",
        "abc",
        Vec::new(),
        None,
    ));
    fixtures.push(complete_text(
        "TOK-002",
        FixtureCategory::SupportedToken,
        "multi-byte character data retains byte coordinates",
        "é界ß",
        "é界ß",
        Vec::new(),
        None,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-003", "simple lowercase start tag", "<a>", TokenKind::StartTag,
        0, 3, 0, 1, 1, 2, "a", Vec::new(), None, 2, 3, Vec::new(),
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-004", "mixed-case start tag with ASCII-lowercased interpretation", "<DiV>",
        TokenKind::StartTag, 0, 5, 0, 1, 1, 4, "div", Vec::new(), None, 4, 5, Vec::new(),
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-005", "simple end tag", "</a>", TokenKind::EndTag,
        0, 4, 0, 2, 2, 3, "a", Vec::new(), None, 3, 4, Vec::new(),
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-006", "mixed-case end tag", "</DIV>", TokenKind::EndTag,
        0, 6, 0, 2, 2, 5, "div", Vec::new(), None, 5, 6, Vec::new(),
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-007", "whitespace between tag name and close delimiter", "<a >",
        TokenKind::StartTag, 0, 4, 0, 1, 1, 2, "a", Vec::new(), None, 3, 4, Vec::new(),
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-008", "boolean attribute", "<a x>", TokenKind::StartTag,
        0, 5, 0, 1, 1, 2, "a",
        vec![attribute_missing("<a x>", 3, 4, 3, 4, "x", AttributeDisposition::Effective)],
        None, 4, 5, Vec::new(),
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-009", "unquoted attribute value", "<a x=y>", TokenKind::StartTag,
        0, 7, 0, 1, 1, 2, "a",
        vec![attribute_unquoted(
            "<a x=y>", 3, 6, 3, 4, "x", 4, 5, 5, 6, "y", AttributeDisposition::Effective,
        )],
        None, 6, 7, Vec::new(),
    ));
    let tok10 = "<a x=\"\" y=\"z\">";
    fixtures.push(complete_tag_fixture(
        "TOK-010", "double-quoted empty and non-empty attribute values", tok10,
        TokenKind::StartTag, 0, 14, 0, 1, 1, 2, "a",
        vec![
            attribute_double(tok10, 3, 7, 3, 4, "x", 4, 5, 5, 6, 6, 6, 6, 7, "", AttributeDisposition::Effective),
            attribute_double(tok10, 8, 13, 8, 9, "y", 9, 10, 10, 11, 11, 12, 12, 13, "z", AttributeDisposition::Effective),
        ],
        None, 13, 14, Vec::new(),
    ));
    let tok11 = "<a x='' y='z'>";
    fixtures.push(complete_tag_fixture(
        "TOK-011", "single-quoted empty and non-empty attribute values", tok11,
        TokenKind::StartTag, 0, 14, 0, 1, 1, 2, "a",
        vec![
            attribute_single(tok11, 3, 7, 3, 4, "x", 4, 5, 5, 6, 6, 6, 6, 7, "", AttributeDisposition::Effective),
            attribute_single(tok11, 8, 13, 8, 9, "y", 9, 10, 10, 11, 11, 12, 12, 13, "z", AttributeDisposition::Effective),
        ],
        None, 13, 14, Vec::new(),
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-012", "self-closing start tag with exact solidus evidence", "<a/>",
        TokenKind::StartTag, 0, 4, 0, 1, 1, 2, "a", Vec::new(), Some((2, 3)), 3, 4, Vec::new(),
    ));
}
