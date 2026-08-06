use super::super::expected::*;
use super::super::fixture::{FixtureCategory, HtmlTokenizerFixture};
use super::helpers::*;

#[rustfmt::skip]
pub(super) fn add_supported_tokens(fixtures: &mut Vec<HtmlTokenizerFixture>) {
    fixtures.push(complete_text(
        "TOK-001",
        FixtureCategory::SupportedToken,
        "contiguous Data-state character data",
        "abc",
        "abc",
        Vec::new(),
        None,
        // Data('a')+Data('b')+Data('c')+Data(EOF) = 4
        4,
    ));
    fixtures.push(complete_text(
        "TOK-002",
        FixtureCategory::SupportedToken,
        "multi-byte character data retains byte coordinates",
        "é界ß",
        "é界ß",
        Vec::new(),
        None,
        // 3 scalar Data transitions + Data(EOF) = 4
        4,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-003",
        "simple lowercase start tag",
        "<a>",
        TokenKind::StartTag,
        0,
        3,
        0,
        1,
        1,
        2,
        "a",
        Vec::new(),
        None,
        2,
        3,
        Vec::new(),
        // Data('<')+TagOpen('a',create,reconsume)+TagName(reconsume 'a')+TagName('>',emit)+Data(EOF) = 5
        5,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-004",
        "mixed-case start tag with ASCII-lowercased interpretation",
        "<DiV>",
        TokenKind::StartTag,
        0,
        5,
        0,
        1,
        1,
        4,
        "div",
        Vec::new(),
        None,
        4,
        5,
        Vec::new(),
        // Data<+TagOpen(D,reconsume)+TagName(D,i,V)+TagName(>,emit)+Data(EOF) = 1+1+3+1+1 = 7
        7,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-005",
        "simple end tag",
        "</a>",
        TokenKind::EndTag,
        0,
        4,
        0,
        2,
        2,
        3,
        "a",
        Vec::new(),
        None,
        3,
        4,
        Vec::new(),
        // Data<+TagOpen(/)+EndTagOpen(a,reconsume)+TagName(reconsume a)+TagName(>,emit)+Data(EOF) = 6
        6,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-006",
        "mixed-case end tag",
        "</DIV>",
        TokenKind::EndTag,
        0,
        6,
        0,
        2,
        2,
        5,
        "div",
        Vec::new(),
        None,
        5,
        6,
        Vec::new(),
        // Data<+TagOpen(/)+EndTagOpen(D,reconsume)+TagName(D,I,V)+TagName(>,emit)+Data(EOF) = 1+1+1+3+1+1 = 8
        8,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-007",
        "whitespace between tag name and close delimiter",
        "<a >",
        TokenKind::StartTag,
        0,
        4,
        0,
        1,
        1,
        2,
        "a",
        Vec::new(),
        None,
        3,
        4,
        Vec::new(),
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp -> BeforeAttrName)
        // +BeforeAttrName(>,reconsume AfterAttrName)+AfterAttrName(reconsume >,emit)+Data(EOF) = 7
        7,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-008",
        "boolean attribute",
        "<a x>",
        TokenKind::StartTag,
        0,
        5,
        0,
        1,
        1,
        2,
        "a",
        vec![attribute_missing("<a x>", 3, 4, 3, 4, "x", AttributeDisposition::Effective)],
        None,
        4,
        5,
        Vec::new(),
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(x,create,reconsume)
        // +AttrName(reconsume x)+AttrName(>,reconsume AfterAttrName)+AfterAttrName(reconsume >,emit)+Data(EOF) = 9
        9,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-009",
        "unquoted attribute value",
        "<a x=y>",
        TokenKind::StartTag,
        0,
        7,
        0,
        1,
        1,
        2,
        "a",
        vec![attribute_unquoted(
            "<a x=y>",
            3,
            6,
            3,
            4,
            "x",
            4,
            5,
            5,
            6,
            "y",
            AttributeDisposition::Effective,
        )],
        None,
        6,
        7,
        Vec::new(),
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(x,create,reconsume)
        // +AttrName(reconsume x)+AttrName(=)+BeforeAttrValue(y,reconsume Unquoted)+Unquoted(reconsume y)
        // +Unquoted(>,emit)+Data(EOF) = 11
        11,
    ));
    let tok10 = "<a x=\"\" y=\"z\">";
    fixtures.push(complete_tag_fixture(
        "TOK-010",
        "double-quoted empty and non-empty attribute values",
        tok10,
        TokenKind::StartTag,
        0,
        14,
        0,
        1,
        1,
        2,
        "a",
        vec![
            attribute_double(tok10, 3, 7, 3, 4, "x", 4, 5, 5, 6, 6, 6, 6, 7, "", AttributeDisposition::Effective),
            attribute_double(tok10, 8, 13, 8, 9, "y", 9, 10, 10, 11, 11, 12, 12, 13, "z", AttributeDisposition::Effective),
        ],
        None,
        13,
        14,
        Vec::new(),
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(sp)+BeforeAttrName(x,create,reconsume)
        // +AttrName(reconsume x)+AttrName(=)+BeforeAttrValue(")+DoubleQuoted(")+AfterAttrValueQuoted(sp)
        // +BeforeAttrName(y,create,reconsume)+AttrName(reconsume y)+AttrName(=)+BeforeAttrValue(")
        // +DoubleQuoted(z)+DoubleQuoted(")+AfterAttrValueQuoted(>,emit)+Data(EOF) = 18
        18,
    ));
    let tok11 = "<a x='' y='z'>";
    fixtures.push(complete_tag_fixture(
        "TOK-011",
        "single-quoted empty and non-empty attribute values",
        tok11,
        TokenKind::StartTag,
        0,
        14,
        0,
        1,
        1,
        2,
        "a",
        vec![
            attribute_single(tok11, 3, 7, 3, 4, "x", 4, 5, 5, 6, 6, 6, 6, 7, "", AttributeDisposition::Effective),
            attribute_single(tok11, 8, 13, 8, 9, "y", 9, 10, 10, 11, 11, 12, 12, 13, "z", AttributeDisposition::Effective),
        ],
        None,
        13,
        14,
        Vec::new(),
        // Same shape as TOK-010 with single quotes = 18
        18,
    ));
    fixtures.push(complete_tag_fixture(
        "TOK-012",
        "self-closing start tag with exact solidus evidence",
        "<a/>",
        TokenKind::StartTag,
        0,
        4,
        0,
        1,
        1,
        2,
        "a",
        Vec::new(),
        Some((2, 3)),
        3,
        4,
        Vec::new(),
        // Data<+TagOpen(a,reconsume)+TagName(reconsume a)+TagName(/)+SelfClosing(>,emit)+Data(EOF) = 6
        6,
     ));
}
