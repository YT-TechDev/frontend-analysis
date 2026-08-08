use std::cmp::Ordering;

use crate::{SourceId, SourceText};

use super::token::{
    CssCommentEvidence, CssCommentTermination, CssDecimalExponent, CssDecimalValue,
    CssExponentSign, CssHashType, CssLexicalItem, CssLexicalOrderError, CssNumberSign,
    CssNumberType, CssNumericValue, CssToken, CssTokenContractError, CssTokenEvidenceRole,
    CssTokenKind,
};

fn source(id: u64, text: &str) -> SourceText {
    SourceText::new(SourceId::new(id), text.to_owned())
}

fn integer_value(digits: &str, sign: Option<CssNumberSign>) -> CssNumericValue {
    CssNumericValue::new(
        CssDecimalValue::new(digits.to_owned(), String::new(), None).unwrap(),
        sign,
    )
}

fn number_value(
    integer_digits: &str,
    fraction_digits: &str,
    exponent: Option<(Option<CssExponentSign>, &str)>,
    sign: Option<CssNumberSign>,
) -> CssNumericValue {
    let exponent =
        exponent.map(|(sign, digits)| CssDecimalExponent::new(sign, digits.to_owned()).unwrap());
    CssNumericValue::new(
        CssDecimalValue::new(
            integer_digits.to_owned(),
            fraction_digits.to_owned(),
            exponent,
        )
        .unwrap(),
        sign,
    )
}

fn whole_token(id: u64, raw: &str, kind: CssTokenKind) -> CssToken {
    let source = source(id, raw);
    let anchor = source.anchor(0, raw.len()).unwrap();
    CssToken::new(&source, anchor, kind).unwrap()
}

#[test]
fn first_capability_token_vocabulary_constructs_with_source_evidence() {
    let cases = vec![
        (r"c\6F lor", CssTokenKind::Ident("color".to_owned())),
        (r"f\6fo(", CssTokenKind::Function("foo".to_owned())),
        (r"@m\65 dia", CssTokenKind::AtKeyword("media".to_owned())),
        (
            r"#i\64",
            CssTokenKind::Hash {
                value: "id".to_owned(),
                hash_type: CssHashType::Id,
            },
        ),
        (
            "#123",
            CssTokenKind::Hash {
                value: "123".to_owned(),
                hash_type: CssHashType::Unrestricted,
            },
        ),
        ("\"a\\62\"", CssTokenKind::String("ab".to_owned())),
        (r"url(a\62)", CssTokenKind::Url("ab".to_owned())),
        ("url(a\"x)", CssTokenKind::BadUrl),
        ("!", CssTokenKind::Delim('!')),
        (
            "+12",
            CssTokenKind::Number {
                value: integer_value("12", Some(CssNumberSign::Plus)),
                number_type: CssNumberType::Integer,
            },
        ),
        (
            ".5%",
            CssTokenKind::Percentage {
                value: number_value("0", "5", None, None),
            },
        ),
        (
            "-1.25e+2px",
            CssTokenKind::Dimension {
                value: number_value(
                    "1",
                    "25",
                    Some((Some(CssExponentSign::Plus), "2")),
                    Some(CssNumberSign::Minus),
                ),
                number_type: CssNumberType::Number,
                unit: "px".to_owned(),
            },
        ),
        (" \r\n\t\u{000C}", CssTokenKind::Whitespace),
        ("<!--", CssTokenKind::Cdo),
        ("-->", CssTokenKind::Cdc),
        (":", CssTokenKind::Colon),
        (";", CssTokenKind::Semicolon),
        (",", CssTokenKind::Comma),
        ("[", CssTokenKind::LeftSquareBracket),
        ("]", CssTokenKind::RightSquareBracket),
        ("(", CssTokenKind::LeftParenthesis),
        (")", CssTokenKind::RightParenthesis),
        ("{", CssTokenKind::LeftCurlyBracket),
        ("}", CssTokenKind::RightCurlyBracket),
    ];

    for (index, (raw, kind)) in cases.into_iter().enumerate() {
        let token = whole_token(index as u64 + 1, raw, kind);
        assert_eq!(token.source().fragment(), raw);
        assert_eq!(token.source().range().start(), 0);
        assert_eq!(token.source().range().end(), raw.len());
    }
}

#[test]
fn raw_spelling_and_interpreted_ident_value_remain_distinct() {
    let source = source(30, r"c\6F lor");
    let token = CssToken::new(
        &source,
        source.anchor(0, 8).unwrap(),
        CssTokenKind::Ident("color".to_owned()),
    )
    .unwrap();

    assert_eq!(token.source().fragment(), r"c\6F lor");
    assert_eq!(token.source().range().start(), 0);
    assert_eq!(token.source().range().end(), 8);
    assert_eq!(token.kind(), &CssTokenKind::Ident("color".to_owned()));
}

#[test]
fn multibyte_utf8_anchor_is_byte_based_and_interpreted_value_is_preserved() {
    let source = source(31, "é");
    let token = CssToken::new(
        &source,
        source.anchor(0, 2).unwrap(),
        CssTokenKind::Ident("é".to_owned()),
    )
    .unwrap();

    assert_eq!(token.source().range().start(), 0);
    assert_eq!(token.source().range().end(), 2);
    assert_eq!(token.source().fragment(), "é");
}

#[test]
fn preprocessing_sensitive_raw_evidence_can_differ_from_semantic_payload() {
    let whitespace = source(32, "\r\n");
    let whitespace_token = CssToken::new(
        &whitespace,
        whitespace.anchor(0, 2).unwrap(),
        CssTokenKind::Whitespace,
    )
    .unwrap();
    assert_eq!(whitespace_token.source().fragment().as_bytes(), b"\r\n");

    let nul = source(33, "\0");
    let replacement = CssToken::new(
        &nul,
        nul.anchor(0, 1).unwrap(),
        CssTokenKind::Ident("\u{FFFD}".to_owned()),
    )
    .unwrap();
    assert_eq!(replacement.source().fragment().as_bytes(), b"\0");
    assert_eq!(
        replacement.kind(),
        &CssTokenKind::Ident("\u{FFFD}".to_owned())
    );
}

#[test]
fn decimal_components_preserve_large_exponents_without_float_conversion() {
    let exponent_digits = "9".repeat(256);
    let exponent =
        CssDecimalExponent::new(Some(CssExponentSign::Minus), exponent_digits.clone()).unwrap();
    let decimal = CssDecimalValue::new(
        "123456789012345678901234567890".to_owned(),
        "00000000000000000001".to_owned(),
        Some(exponent),
    )
    .unwrap();
    let value = CssNumericValue::new(decimal, Some(CssNumberSign::Minus));

    assert_eq!(value.sign(), Some(CssNumberSign::Minus));
    assert_eq!(
        value.decimal().integer_digits(),
        "123456789012345678901234567890"
    );
    assert_eq!(value.decimal().fraction_digits(), "00000000000000000001");
    let exponent = value.decimal().exponent().unwrap();
    assert_eq!(exponent.sign(), Some(CssExponentSign::Minus));
    assert_eq!(exponent.digits(), exponent_digits);
    assert_eq!(value.required_number_type(), CssNumberType::Number);
}

#[test]
fn integer_and_number_type_are_structurally_distinguished() {
    let integer = integer_value("12", None);
    assert_eq!(integer.required_number_type(), CssNumberType::Integer);

    let decimal = number_value("12", "5", None, None);
    assert_eq!(decimal.required_number_type(), CssNumberType::Number);

    let exponent = number_value("12", "", Some((Some(CssExponentSign::Plus), "3")), None);
    assert_eq!(exponent.required_number_type(), CssNumberType::Number);
}

#[test]
fn number_type_mismatch_is_rejected() {
    let source = source(34, "1.5");
    let error = CssToken::new(
        &source,
        source.anchor(0, 3).unwrap(),
        CssTokenKind::Number {
            value: number_value("1", "5", None, None),
            number_type: CssNumberType::Integer,
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        CssTokenContractError::InvalidNumberType {
            expected: CssNumberType::Number,
            actual: CssNumberType::Integer,
        }
    );
}

#[test]
fn invalid_decimal_components_are_rejected() {
    assert_eq!(
        CssDecimalValue::new(String::new(), String::new(), None).unwrap_err(),
        CssTokenContractError::EmptyInterpretedValue {
            role: CssTokenEvidenceRole::IntegerDigits,
        }
    );
    assert_eq!(
        CssDecimalValue::new("1a".to_owned(), String::new(), None).unwrap_err(),
        CssTokenContractError::InvalidAsciiDigits {
            role: CssTokenEvidenceRole::IntegerDigits,
        }
    );
    assert_eq!(
        CssDecimalExponent::new(None, "x".to_owned()).unwrap_err(),
        CssTokenContractError::InvalidAsciiDigits {
            role: CssTokenEvidenceRole::ExponentDigits,
        }
    );
}

#[test]
fn empty_token_source_and_wrong_fixed_spelling_are_rejected() {
    let source = source(35, ";");
    assert_eq!(
        CssToken::new(
            &source,
            source.anchor(0, 0).unwrap(),
            CssTokenKind::Semicolon,
        )
        .unwrap_err(),
        CssTokenContractError::EmptySourceRange {
            role: CssTokenEvidenceRole::Token,
        }
    );

    assert_eq!(
        CssToken::new(&source, source.anchor(0, 1).unwrap(), CssTokenKind::Colon,).unwrap_err(),
        CssTokenContractError::FixedSpellingMismatch {
            role: CssTokenEvidenceRole::FixedToken,
            expected: ":",
        }
    );
}

#[test]
fn source_identity_mismatch_is_rejected() {
    let expected = source(36, "a");
    let other = source(37, "a");

    assert_eq!(
        CssToken::new(
            &expected,
            other.anchor(0, 1).unwrap(),
            CssTokenKind::Ident("a".to_owned()),
        )
        .unwrap_err(),
        CssTokenContractError::SourceIdentityMismatch {
            role: CssTokenEvidenceRole::Token,
            expected: SourceId::new(36),
            actual: SourceId::new(37),
        }
    );
}

#[test]
fn source_contract_rejects_out_of_bounds_and_mid_codepoint_ranges() {
    let source = source(38, "é");
    assert!(source.anchor(0, 3).is_err());
    assert!(source.anchor(1, 2).is_err());
}

#[test]
fn impossible_local_token_relationships_are_rejected() {
    let function_source = source(39, "foo");
    assert_eq!(
        CssToken::new(
            &function_source,
            function_source.anchor(0, 3).unwrap(),
            CssTokenKind::Function("foo".to_owned()),
        )
        .unwrap_err(),
        CssTokenContractError::InvalidFunctionBoundary
    );

    let at_source = source(40, "media");
    assert_eq!(
        CssToken::new(
            &at_source,
            at_source.anchor(0, 5).unwrap(),
            CssTokenKind::AtKeyword("media".to_owned()),
        )
        .unwrap_err(),
        CssTokenContractError::InvalidAtKeywordBoundary
    );

    let hash_source = source(41, "id");
    assert_eq!(
        CssToken::new(
            &hash_source,
            hash_source.anchor(0, 2).unwrap(),
            CssTokenKind::Hash {
                value: "id".to_owned(),
                hash_type: CssHashType::Unrestricted,
            },
        )
        .unwrap_err(),
        CssTokenContractError::InvalidHashBoundary
    );

    let string_source = source(42, "text");
    assert_eq!(
        CssToken::new(
            &string_source,
            string_source.anchor(0, 4).unwrap(),
            CssTokenKind::String("text".to_owned()),
        )
        .unwrap_err(),
        CssTokenContractError::InvalidStringBoundary
    );

    let dimension_source = source(43, "1px");
    assert_eq!(
        CssToken::new(
            &dimension_source,
            dimension_source.anchor(0, 3).unwrap(),
            CssTokenKind::Dimension {
                value: integer_value("1", None),
                number_type: CssNumberType::Integer,
                unit: String::new(),
            },
        )
        .unwrap_err(),
        CssTokenContractError::InvalidDimensionUnit
    );
}

#[test]
fn closed_comment_preserves_complete_open_and_close_evidence() {
    let source = source(44, "/*α*/");
    let len = source.as_str().len();
    let comment = CssCommentEvidence::new(
        &source,
        source.anchor(0, len).unwrap(),
        source.anchor(0, 2).unwrap(),
        CssCommentTermination::Closed {
            close_delimiter: source.anchor(len - 2, len).unwrap(),
        },
    )
    .unwrap();

    assert_eq!(comment.complete().fragment(), "/*α*/");
    assert_eq!(comment.open_delimiter().fragment(), "/*");
    match comment.termination() {
        CssCommentTermination::Closed { close_delimiter } => {
            assert_eq!(close_delimiter.fragment(), "*/");
        }
        CssCommentTermination::EndOfInput => panic!("expected closed comment"),
    }
}

#[test]
fn eof_comment_preserves_authored_range_through_source_end() {
    let source = source(45, "/*unterminated");
    let len = source.as_str().len();
    let comment = CssCommentEvidence::new(
        &source,
        source.anchor(0, len).unwrap(),
        source.anchor(0, 2).unwrap(),
        CssCommentTermination::EndOfInput,
    )
    .unwrap();

    assert_eq!(comment.complete().range().end(), len);
    assert_eq!(comment.termination(), &CssCommentTermination::EndOfInput);
}

#[test]
fn eof_comment_that_does_not_reach_source_end_is_rejected() {
    let source = source(46, "/*x*/tail");
    assert_eq!(
        CssCommentEvidence::new(
            &source,
            source.anchor(0, 5).unwrap(),
            source.anchor(0, 2).unwrap(),
            CssCommentTermination::EndOfInput,
        )
        .unwrap_err(),
        CssTokenContractError::CommentEndOfInputNotAtSourceEnd {
            end: 5,
            source_len: 9,
        }
    );
}

#[test]
fn comment_component_source_mismatch_is_rejected() {
    let other = source(48, "/**/");
    let source = source(47, "/**/");

    assert_eq!(
        CssCommentEvidence::new(
            &source,
            source.anchor(0, 4).unwrap(),
            other.anchor(0, 2).unwrap(),
            CssCommentTermination::Closed {
                close_delimiter: source.anchor(2, 4).unwrap(),
            },
        )
        .unwrap_err(),
        CssTokenContractError::SourceIdentityMismatch {
            role: CssTokenEvidenceRole::CommentOpenDelimiter,
            expected: SourceId::new(47),
            actual: SourceId::new(48),
        }
    );
}

#[test]
fn comment_delimiters_must_align_with_complete_range() {
    let source = source(49, "x/**/");
    assert_eq!(
        CssCommentEvidence::new(
            &source,
            source.anchor(0, 5).unwrap(),
            source.anchor(1, 3).unwrap(),
            CssCommentTermination::Closed {
                close_delimiter: source.anchor(3, 5).unwrap(),
            },
        )
        .unwrap_err(),
        CssTokenContractError::MisalignedBoundary {
            role: CssTokenEvidenceRole::CommentOpenDelimiter,
        }
    );
}

#[test]
fn lexical_items_preserve_source_order_for_consecutive_comments() {
    let source = source(50, "/**//**/");
    let first = CssCommentEvidence::new(
        &source,
        source.anchor(0, 4).unwrap(),
        source.anchor(0, 2).unwrap(),
        CssCommentTermination::Closed {
            close_delimiter: source.anchor(2, 4).unwrap(),
        },
    )
    .unwrap();
    let second = CssCommentEvidence::new(
        &source,
        source.anchor(4, 8).unwrap(),
        source.anchor(4, 6).unwrap(),
        CssCommentTermination::Closed {
            close_delimiter: source.anchor(6, 8).unwrap(),
        },
    )
    .unwrap();

    let first = CssLexicalItem::Comment(first);
    let second = CssLexicalItem::Comment(second);

    assert_eq!(first.compare_source_order(&second).unwrap(), Ordering::Less);
    assert_eq!(first.source_order_key(), (50, 0, 4));
    assert_eq!(second.source_order_key(), (50, 4, 8));
}

#[test]
fn lexical_order_rejects_cross_source_comparison() {
    let first =
        CssLexicalItem::SemanticToken(whole_token(51, "a", CssTokenKind::Ident("a".to_owned())));
    let second =
        CssLexicalItem::SemanticToken(whole_token(52, "b", CssTokenKind::Ident("b".to_owned())));

    assert_eq!(
        first.compare_source_order(&second).unwrap_err(),
        CssLexicalOrderError::SourceIdentityMismatch {
            expected: SourceId::new(51),
            actual: SourceId::new(52),
        }
    );
}

#[test]
fn equality_depends_on_source_identity_range_and_semantics_not_allocation_identity() {
    let left_source = source(53, "a");
    let right_source = source(53, "a");

    let left = CssToken::new(
        &left_source,
        left_source.anchor(0, 1).unwrap(),
        CssTokenKind::Ident("a".to_owned()),
    )
    .unwrap();
    let right = CssToken::new(
        &right_source,
        right_source.anchor(0, 1).unwrap(),
        CssTokenKind::Ident("a".to_owned()),
    )
    .unwrap();

    assert_eq!(left, right);
}

#[test]
fn debug_output_does_not_dump_authored_or_decoded_strings() {
    let source = source(54, r"s\65 cret");
    let token = CssToken::new(
        &source,
        source.anchor(0, source.as_str().len()).unwrap(),
        CssTokenKind::Ident("secret".to_owned()),
    )
    .unwrap();

    let token_debug = format!("{token:?}");
    let kind_debug = format!("{:?}", token.kind());

    assert!(!token_debug.contains("secret"));
    assert!(!token_debug.contains(r"s\65 cret"));
    assert!(!kind_debug.contains("secret"));
    assert!(token_debug.contains("interpreted_byte_len"));
}

#[test]
fn empty_required_interpreted_values_are_rejected() {
    let source = source(57, "a");
    assert_eq!(
        CssToken::new(
            &source,
            source.anchor(0, 1).unwrap(),
            CssTokenKind::Ident(String::new()),
        )
        .unwrap_err(),
        CssTokenContractError::EmptyInterpretedValue {
            role: CssTokenEvidenceRole::IdentValue,
        }
    );
}

#[test]
fn malformed_token_kinds_remain_explicit_instead_of_becoming_absence() {
    let bad_string_source = source(55, "\"x\n");
    let bad_string = CssToken::new(
        &bad_string_source,
        bad_string_source.anchor(0, 2).unwrap(),
        CssTokenKind::BadString,
    )
    .unwrap();

    let bad_url = whole_token(56, "url(x\"y)", CssTokenKind::BadUrl);

    assert_eq!(bad_string.kind(), &CssTokenKind::BadString);
    assert_eq!(bad_url.kind(), &CssTokenKind::BadUrl);
}
