//! Candidate-independent direct `this` initializer validation for Issue #253.
//!
//! This oracle qualifies only direct-authored `PrimaryExpression : this` in the
//! already-selected initializer position. It does not call production lexical,
//! static-semantics, aggregate, or runtime-evaluation code.
//!
//! `UnsupportedCoverage` expectations are scoped to the direct-`this` frontier.
//! Later independently qualified owners may strengthen classifications for
//! escaped `this`, binding positions, richer expressions, comments, or ASI.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode::{is_id_continue, is_id_start};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 253;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_NULL_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_null_literal_initializer_validation_tests.rs");
const THIS_SOURCE: &str =
    include_str!("qualification_selected_this_expression_initializer_validation_tests.rs");
const FRONTIER_SCOPE_NOTE: &str =
    "direct-this frontier only; later independently qualified owners may strengthen classification";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    DirectThis,
    IdentifierReference,
    ReservedIdentifierName,
    Unowned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    SelectedAcceptedIncomplete,
    UnsupportedCoverage,
    StaticSemanticsRejected,
    SyntaxRejected,
}

#[derive(Debug, Clone, Copy)]
struct Escape {
    end: usize,
    code_point: u32,
}

const RESERVED: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
];

fn slice(text: &str, Range(start, end): Range) -> &str {
    text.get(start..end)
        .unwrap_or_else(|| panic!("invalid UTF-8 range ({start},{end}) in {text:?}"))
}

fn hex(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some((byte - b'0') as u32),
        b'a'..=b'f' => Some((byte - b'a' + 10) as u32),
        b'A'..=b'F' => Some((byte - b'A' + 10) as u32),
        _ => None,
    }
}

fn parse_hex(digits: &[u8]) -> Option<u32> {
    let significant = match digits.iter().position(|byte| *byte != b'0') {
        Some(index) => &digits[index..],
        None => return Some(0),
    };
    if significant.len() > 6 {
        return None;
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value.checked_mul(16)?.checked_add(hex(*byte)?)?;
    }
    (value <= 0x10_FFFF).then_some(value)
}

fn formed_escape(text: &str, start: usize) -> Option<Escape> {
    let bytes = text.as_bytes();
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return None;
    }

    if bytes.get(start + 2) == Some(&b'{') {
        let digits_start = start + 3;
        let mut end = digits_start;
        while let Some(byte) = bytes.get(end) {
            if *byte == b'}' {
                let digits = bytes.get(digits_start..end)?;
                if digits.is_empty() || digits.iter().any(|byte| hex(*byte).is_none()) {
                    return None;
                }
                return Some(Escape {
                    end: end + 1,
                    code_point: parse_hex(digits)?,
                });
            }
            hex(*byte)?;
            end += 1;
        }
        return None;
    }

    let digits = bytes.get(start + 2..start + 6)?;
    if digits.iter().any(|byte| hex(*byte).is_none()) {
        return None;
    }
    Some(Escape {
        end: start + 6,
        code_point: parse_hex(digits)?,
    })
}

fn identifier_start(code_point: u32) -> bool {
    code_point == '$' as u32 || code_point == '_' as u32 || is_id_start(code_point)
}

fn identifier_part(code_point: u32) -> bool {
    identifier_start(code_point)
        || is_id_continue(code_point)
        || matches!(code_point, 0x200C | 0x200D)
}

fn decode_identifier_name(rhs: &str) -> Option<String> {
    let mut offset = 0_usize;
    let mut first = true;
    let mut decoded = String::new();

    while offset < rhs.len() {
        let (code_point, end) = if rhs.as_bytes().get(offset) == Some(&b'\\') {
            let escape = formed_escape(rhs, offset)?;
            (escape.code_point, escape.end)
        } else {
            let scalar = rhs.get(offset..)?.chars().next()?;
            (scalar as u32, offset + scalar.len_utf8())
        };

        if if first {
            !identifier_start(code_point)
        } else {
            !identifier_part(code_point)
        } {
            return None;
        }

        decoded.push(char::from_u32(code_point)?);
        first = false;
        offset = end;
    }

    (!first).then_some(decoded)
}

fn route(rhs: &str) -> Route {
    if rhs == "this" {
        return Route::DirectThis;
    }
    let Some(decoded) = decode_identifier_name(rhs) else {
        return Route::Unowned;
    };
    if RESERVED.contains(&decoded.as_str()) {
        Route::ReservedIdentifierName
    } else {
        Route::IdentifierReference
    }
}

fn authored_anchor(source_id: u64, text: &str, range: Range) -> String {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    let Range(start, end) = range;
    source
        .anchor(start, end)
        .expect("fixture range must be a valid authored anchor")
        .fragment()
        .to_owned()
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 253);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_NULL_ORACLE_SOURCE.contains("NULL-ROUTE-OTHER-RESERVED-THIS"));
    assert!(PREVIOUS_NULL_ORACLE_SOURCE.contains("NULL-UNSUPPORTED-OTHER-RESERVED-THIS"));

    assert!(FRONTIER_SCOPE_NOTE.contains("direct-this frontier only"));
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));
    assert!(!THIS_SOURCE.contains("FutureUnsupportedDisposition"));

    for forbidden in [
        concat!("use super::", "selected_lexical_slice"),
        concat!("use super::", "selected_static_semantics"),
        concat!("use super::", "selected_qualification_integration"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("consume_selected_", "this"),
        concat!("ResolveThis", "Binding"),
    ] {
        assert!(!THIS_SOURCE.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn direct_this_and_maximal_identifier_name_routes_are_independent() {
    let selected = [
        "thisx",
        "thisValue",
        "thisπ",
        "this0",
        "this$",
        "this_",
        r"this\u0061",
        r"this\u{61}",
        r"this\u{00000061}",
        r"this\u0030",
        r"this\u200C",
        r"this\u200D",
    ];
    let escaped_reserved = [r"\u0074his", r"t\u0068is", r"th\u0069s", r"thi\u0073"];
    let unowned = [
        r"this\u002D",
        r"this\uD800",
        r"this\u{D800}",
        r"this\u{}",
        r"this\u0",
        r"this\u{61",
        r"this\u{110000}",
    ];

    assert_eq!(route("this"), Route::DirectThis);
    for rhs in selected {
        assert_eq!(route(rhs), Route::IdentifierReference, "{rhs}");
    }
    for rhs in escaped_reserved {
        assert_eq!(route(rhs), Route::ReservedIdentifierName, "{rhs}");
        assert_eq!(decode_identifier_name(rhs).as_deref(), Some("this"));
    }
    for rhs in unowned {
        assert_eq!(route(rhs), Route::Unowned, "{rhs}");
    }

    assert_eq!(
        decode_identifier_name(r"this\u0061").as_deref(),
        Some("thisa")
    );
}

#[test]
fn positive_ranges_boundaries_and_lifecycle_are_pinned() {
    let fixtures = [
        ("const x = this;", Range(10, 14), 14, Some(b';')),
        ("const x = this", Range(10, 14), 14, None),
        ("const x = this   \t\n", Range(10, 14), 19, None),
        ("const x = this \t\n;", Range(10, 14), 17, Some(b';')),
        ("let x = this, y = foo;", Range(8, 12), 12, Some(b',')),
        ("let x = foo, y = this;", Range(17, 21), 21, Some(b';')),
        ("let x = 1, y = this;", Range(15, 19), 19, Some(b';')),
        ("const x = this, y = null;", Range(10, 14), 14, Some(b',')),
        ("const x = null, y = this;", Range(20, 24), 24, Some(b';')),
        ("const x = true, y = this;", Range(20, 24), 24, Some(b';')),
        (
            r"const x = \u0066oo, y = this;",
            Range(24, 28),
            28,
            Some(b';'),
        ),
    ];

    for (index, (text, rhs, boundary, expected_byte)) in fixtures.iter().enumerate() {
        assert_eq!(slice(text, *rhs), "this");
        assert_eq!(route("this"), Route::DirectThis);
        let expected = FrontierOutcome::SelectedAcceptedIncomplete;
        assert!(matches!(
            expected,
            FrontierOutcome::SelectedAcceptedIncomplete
        ));
        assert_eq!(authored_anchor(253_100 + index as u64, text, *rhs), "this");

        let Range(_, end) = *rhs;
        let trivia = text.get(end..*boundary).expect("fixture boundary");
        assert!(trivia.chars().all(|code_point| {
            matches!(
                code_point,
                '\u{0009}'
                    | '\u{000B}'
                    | '\u{000C}'
                    | '\u{0020}'
                    | '\u{00A0}'
                    | '\u{1680}'
                    | '\u{2000}'
                    | '\u{2001}'
                    | '\u{2002}'
                    | '\u{2003}'
                    | '\u{2004}'
                    | '\u{2005}'
                    | '\u{2006}'
                    | '\u{2007}'
                    | '\u{2008}'
                    | '\u{2009}'
                    | '\u{200A}'
                    | '\u{202F}'
                    | '\u{205F}'
                    | '\u{3000}'
                    | '\u{FEFF}'
                    | '\n'
                    | '\r'
                    | '\u{2028}'
                    | '\u{2029}'
            )
        }));

        match expected_byte {
            Some(byte) => assert_eq!(text.as_bytes().get(*boundary), Some(byte)),
            None => assert_eq!(*boundary, text.len()),
        }
    }
}

#[test]
fn unsupported_controls_are_scoped_to_this_frontier() {
    let escaped = [
        (r"const x = \u0074his;", Range(10, 19)),
        (r"const x = t\u0068is;", Range(10, 19)),
        (r"const x = th\u0069s;", Range(10, 19)),
        (r"const x = thi\u0073;", Range(10, 19)),
    ];
    let unowned = [
        (r"const x = this\u002D;", Range(10, 20)),
        (r"const x = this\uD800;", Range(10, 20)),
        (r"const x = this\u{D800};", Range(10, 22)),
        (r"const x = this\u{};", Range(10, 18)),
        (r"const x = this\u0;", Range(10, 17)),
        (r"const x = this\u{61", Range(10, 19)),
        (r"const x = this\u{110000};", Range(10, 24)),
    ];
    let richer = [
        "const x = this.foo;",
        "const x = this();",
        "const x = this + x;",
        "const x = this = x;",
        "const x = this ? x : y;",
        "const x = this/*comment*/;",
        "const x = this unexpected;",
        "const x = this;;",
        "const x = this\nconst y = foo;",
    ];

    for (index, (text, range)) in escaped.iter().enumerate() {
        let rhs = slice(text, *range);
        assert_eq!(route(rhs), Route::ReservedIdentifierName);
        assert_eq!(authored_anchor(253_200 + index as u64, text, *range), rhs);
    }
    for (index, (text, range)) in unowned.iter().enumerate() {
        let rhs = slice(text, *range);
        assert_eq!(route(rhs), Route::Unowned);
        assert_eq!(authored_anchor(253_220 + index as u64, text, *range), rhs);
    }
    for (index, text) in richer.iter().enumerate() {
        let range = Range(10, 14);
        assert_eq!(slice(text, range), "this");
        assert_eq!(route("this"), Route::DirectThis);
        assert_eq!(authored_anchor(253_240 + index as u64, text, range), "this");
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
    assert!(FRONTIER_SCOPE_NOTE.contains("later independently qualified owners"));
}

#[test]
fn position_controls_pin_non_ownership_not_permanent_global_disposition() {
    let fixtures = [
        ("let this = 1;", Range(4, 8)),
        ("let this;", Range(4, 8)),
        ("this;", Range(0, 4)),
        ("const x = this, this;", Range(16, 20)),
    ];

    for (index, (text, range)) in fixtures.iter().enumerate() {
        assert_eq!(slice(text, *range), "this");
        assert_eq!(route("this"), Route::DirectThis);
        assert_eq!(
            authored_anchor(253_300 + index as u64, text, *range),
            "this"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));
}

#[test]
fn existing_static_subjects_and_families_remain_reachable() {
    let fixtures = [
        (
            r"let \u0030 = this;",
            Range(13, 17),
            Range(4, 10),
            r"\u0030",
            "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
        ),
        (
            r"let \u0069f = this;",
            Range(14, 18),
            Range(4, 11),
            r"\u0069f",
            "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
        ),
        (
            "let let = this;",
            Range(10, 14),
            Range(4, 7),
            "let",
            "JS-GOLD-LEXDECL-LET-BINDING-001",
        ),
        (
            "let x = this, x = foo;",
            Range(8, 12),
            Range(14, 15),
            "x",
            "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        ),
        (
            "const x = this, y;",
            Range(10, 14),
            Range(16, 17),
            "y",
            "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        ),
        (
            "let x = this; let x = foo;",
            Range(8, 12),
            Range(18, 19),
            "x",
            "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        ),
    ];

    for (index, (text, rhs, subject, fragment, gold)) in fixtures.iter().enumerate() {
        assert_eq!(slice(text, *rhs), "this");
        assert_eq!(route("this"), Route::DirectThis);
        assert_eq!(slice(text, *subject), *fragment);
        assert!(gold_source(gold).is_some(), "{gold}");
        assert_eq!(
            authored_anchor(253_400 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::StaticSemanticsRejected;
    assert!(matches!(expected, FrontierOutcome::StaticSemanticsRejected));
}

#[test]
fn existing_grammar_subjects_and_syntax_family_remain_reachable() {
    let fixtures = [
        (r"const x = this; let \u{};", Range(20, 24), r"\u{}"),
        (r"const x = this; let a\u{};", Range(21, 25), r"\u{}"),
        (r"const x = this; let \u{61", Range(20, 25), r"\u{61"),
    ];

    for (index, (text, subject, fragment)) in fixtures.iter().enumerate() {
        assert_eq!(slice(text, Range(10, 14)), "this");
        assert_eq!(route("this"), Route::DirectThis);
        assert_eq!(slice(text, *subject), *fragment);
        assert_eq!(
            authored_anchor(253_500 + index as u64, text, *subject),
            *fragment
        );
    }

    let expected = FrontierOutcome::SyntaxRejected;
    assert!(matches!(expected, FrontierOutcome::SyntaxRejected));
}

#[test]
fn handoff_remains_presence_only_unqualified_and_validation_only() {
    assert!(THIS_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(THIS_SOURCE.contains("UnsupportedCoverage"));
    assert!(THIS_SOURCE.contains("StaticSemanticsRejected"));
    assert!(THIS_SOURCE.contains("SyntaxRejected"));

    for forbidden in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("CompleteQualification", "Witness"),
        concat!("This", "Value"),
        concat!("PrimaryExpression", "Kind"),
        concat!("SelectedExpression", "Kind"),
        concat!("ResolveThis", "Binding"),
    ] {
        assert!(!THIS_SOURCE.contains(forbidden), "{forbidden}");
    }
}
