//! Candidate-independent direct `NullLiteral` initializer validation for Issue #249.
//!
//! This module fixes the direct `null` source family, its collision with maximal
//! direct/escaped `IdentifierName` spelling, selected boundaries, and future
//! lifecycle without calling production lexical, static-semantics, aggregate,
//! runtime-evaluation, or future NullLiteral recognition code.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode::{is_id_continue, is_id_start};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 249;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_BOOLEAN_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_boolean_literal_initializer_validation_tests.rs");
const THIS_SOURCE: &str =
    include_str!("qualification_selected_null_literal_initializer_validation_tests.rs");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ByteRange {
    start: usize,
    end: usize,
}

impl ByteRange {
    const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoundaryKind {
    BindingListComma,
    AuthoredSemicolon,
    AutomaticAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FutureOutcome {
    SelectedAcceptedIncomplete,
    UnsupportedCoverage,
    StaticSemanticsRejected,
    SyntaxRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AuthoredRoute {
    DirectNullLiteral,
    IdentifierReference,
    ReservedIdentifierName,
    Unowned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PositiveFixture {
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    boundary_offset: usize,
    boundary: BoundaryKind,
    expected: FutureOutcome,
}

const fn positive_fixture(
    id: &'static str,
    source: &'static str,
    rhs: &'static str,
    rhs_range: ByteRange,
    boundary_offset: usize,
    boundary: BoundaryKind,
) -> PositiveFixture {
    PositiveFixture {
        id,
        source,
        rhs,
        rhs_range,
        boundary_offset,
        boundary,
        expected: FutureOutcome::SelectedAcceptedIncomplete,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RouteFixture {
    id: &'static str,
    rhs: &'static str,
    expected_route: AuthoredRoute,
    expected_outcome: FutureOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnsupportedReason {
    EscapedNullLikeIdentifierName,
    FormedInvalidContinuation,
    MalformedContinuation,
    OtherReservedDirectRhs,
    MemberExpression,
    CallExpression,
    BinaryExpression,
    AssignmentExpression,
    ConditionalExpression,
    Comment,
    UnexpectedTail,
    ExtraSemicolon,
    RequiresNonEofAsi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UnsupportedFixture {
    id: &'static str,
    source: &'static str,
    rhs_range: ByteRange,
    reason: UnsupportedReason,
    expected: FutureOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticReachabilityFixture {
    id: &'static str,
    source: &'static str,
    rhs_range: ByteRange,
    rule_id: &'static str,
    subject: ByteRange,
    subject_fragment: &'static str,
    control_gold_id: &'static str,
    expected: FutureOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GrammarReachabilityFixture {
    id: &'static str,
    source: &'static str,
    rhs_range: ByteRange,
    subject: ByteRange,
    subject_fragment: &'static str,
    expected: FutureOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EscapeFormation {
    end: usize,
    code_point: u32,
}

const POSITIVE_FIXTURES: &[PositiveFixture] = &[
    positive_fixture(
        "NULL-POSITIVE-SEMICOLON",
        "const x = null;",
        "null",
        ByteRange::new(10, 14),
        14,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "NULL-POSITIVE-EOF",
        "const x = null",
        "null",
        ByteRange::new(10, 14),
        14,
        BoundaryKind::AutomaticAtEof,
    ),
    positive_fixture(
        "NULL-POSITIVE-TRAILING-TRIVIA-EOF",
        "const x = null   \t\n",
        "null",
        ByteRange::new(10, 14),
        19,
        BoundaryKind::AutomaticAtEof,
    ),
    positive_fixture(
        "NULL-POSITIVE-COMMA-FIRST",
        "let x = null, y = foo;",
        "null",
        ByteRange::new(8, 12),
        12,
        BoundaryKind::BindingListComma,
    ),
    positive_fixture(
        "NULL-POSITIVE-COMMA-SECOND",
        "let x = foo, y = null;",
        "null",
        ByteRange::new(17, 21),
        21,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "NULL-POSITIVE-AFTER-DECIMAL",
        "let x = 1, y = null;",
        "null",
        ByteRange::new(15, 19),
        19,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "NULL-POSITIVE-BEFORE-BOOLEAN",
        "const x = null, y = false;",
        "null",
        ByteRange::new(10, 14),
        14,
        BoundaryKind::BindingListComma,
    ),
    positive_fixture(
        "NULL-POSITIVE-AFTER-BOOLEAN",
        "const x = true, y = null;",
        "null",
        ByteRange::new(20, 24),
        24,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "NULL-POSITIVE-AFTER-ESCAPED-IDENTIFIER-REFERENCE",
        r"const x = \u0066oo, y = null;",
        "null",
        ByteRange::new(24, 28),
        28,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "NULL-POSITIVE-TRIVIA-BEFORE-SEMICOLON",
        "const x = null \t\n;",
        "null",
        ByteRange::new(10, 14),
        17,
        BoundaryKind::AuthoredSemicolon,
    ),
];

const ROUTE_FIXTURES: &[RouteFixture] = &[
    RouteFixture {
        id: "NULL-ROUTE-DIRECT",
        rhs: "null",
        expected_route: AuthoredRoute::DirectNullLiteral,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-LONGER-NULLX",
        rhs: "nullx",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-LONGER-NULLVALUE",
        rhs: "nullValue",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-DIRECT-UNICODE-CONTINUATION",
        rhs: "nullπ",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-DIRECT-DIGIT-CONTINUATION",
        rhs: "null0",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-DIRECT-DOLLAR-CONTINUATION",
        rhs: "null$",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-DIRECT-UNDERSCORE-CONTINUATION",
        rhs: "null_",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-FIXED-UES-CONTINUATION",
        rhs: r"null\u0061",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-BRACED-UES-CONTINUATION",
        rhs: r"null\u{61}",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-LONG-LEADING-ZERO-UES-CONTINUATION",
        rhs: r"null\u{00000061}",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-DIGIT-UES-CONTINUATION",
        rhs: r"null\u0030",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-ZWNJ-UES-CONTINUATION",
        rhs: r"null\u200C",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-ZWJ-UES-CONTINUATION",
        rhs: r"null\u200D",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "NULL-ROUTE-ESCAPED-FIRST",
        rhs: r"\u006Eull",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-ESCAPED-MIDDLE",
        rhs: r"n\u0075ll",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-ESCAPED-LATE",
        rhs: r"nu\u006Cll",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-ESCAPED-LAST",
        rhs: r"nul\u006C",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-FORMED-INVALID-HYPHEN",
        rhs: r"null\u002D",
        expected_route: AuthoredRoute::Unowned,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-FORMED-INVALID-SURROGATE-FIXED",
        rhs: r"null\uD800",
        expected_route: AuthoredRoute::Unowned,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-FORMED-INVALID-SURROGATE-BRACED",
        rhs: r"null\u{D800}",
        expected_route: AuthoredRoute::Unowned,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-MALFORMED-CONTINUATION",
        rhs: r"null\u{}",
        expected_route: AuthoredRoute::Unowned,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-NON-CODE-POINT-CONTINUATION",
        rhs: r"null\u{110000}",
        expected_route: AuthoredRoute::Unowned,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "NULL-ROUTE-OTHER-RESERVED-THIS",
        rhs: "this",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
];

const UNSUPPORTED_FIXTURES: &[UnsupportedFixture] = &[
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-ESCAPED-FIRST",
        source: r"const x = \u006Eull;",
        rhs_range: ByteRange::new(10, 19),
        reason: UnsupportedReason::EscapedNullLikeIdentifierName,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-ESCAPED-MIDDLE",
        source: r"const x = n\u0075ll;",
        rhs_range: ByteRange::new(10, 19),
        reason: UnsupportedReason::EscapedNullLikeIdentifierName,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-FORMED-INVALID-HYPHEN",
        source: r"const x = null\u002D;",
        rhs_range: ByteRange::new(10, 20),
        reason: UnsupportedReason::FormedInvalidContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-FORMED-INVALID-SURROGATE-FIXED",
        source: r"const x = null\uD800;",
        rhs_range: ByteRange::new(10, 20),
        reason: UnsupportedReason::FormedInvalidContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-FORMED-INVALID-SURROGATE-BRACED",
        source: r"const x = null\u{D800};",
        rhs_range: ByteRange::new(10, 22),
        reason: UnsupportedReason::FormedInvalidContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-MALFORMED-EMPTY-BRACED",
        source: r"const x = null\u{};",
        rhs_range: ByteRange::new(10, 18),
        reason: UnsupportedReason::MalformedContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-MALFORMED-SHORT",
        source: r"const x = null\u0;",
        rhs_range: ByteRange::new(10, 17),
        reason: UnsupportedReason::MalformedContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-MALFORMED-UNCLOSED",
        source: r"const x = null\u{61",
        rhs_range: ByteRange::new(10, 19),
        reason: UnsupportedReason::MalformedContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-NON-CODE-POINT",
        source: r"const x = null\u{110000};",
        rhs_range: ByteRange::new(10, 24),
        reason: UnsupportedReason::MalformedContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-OTHER-RESERVED-THIS",
        source: "const x = this;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::OtherReservedDirectRhs,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-MEMBER",
        source: "const x = null.foo;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::MemberExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-CALL",
        source: "const x = null();",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::CallExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-BINARY",
        source: "const x = null + x;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::BinaryExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-ASSIGNMENT",
        source: "const x = null = x;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::AssignmentExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-CONDITIONAL",
        source: "const x = null ? x : y;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::ConditionalExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-COMMENT",
        source: "const x = null/*comment*/;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::Comment,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-UNEXPECTED-TAIL",
        source: "const x = null unexpected;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::UnexpectedTail,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-EXTRA-SEMICOLON",
        source: "const x = null;;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::ExtraSemicolon,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "NULL-UNSUPPORTED-NON-EOF-ASI",
        source: "const x = null\nconst y = foo;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::RequiresNonEofAsi,
        expected: FutureOutcome::UnsupportedCoverage,
    },
];

const STATIC_REACHABILITY_FIXTURES: &[StaticReachabilityFixture] = &[
    StaticReachabilityFixture {
        id: "NULL-STATIC-EE01",
        source: r"let \u0030 = null;",
        rhs_range: ByteRange::new(13, 17),
        rule_id: "EE-01-R01",
        subject: ByteRange::new(4, 10),
        subject_fragment: r"\u0030",
        control_gold_id: "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "NULL-STATIC-EE04",
        source: r"let \u0069f = null;",
        rhs_range: ByteRange::new(14, 18),
        rule_id: "EE-04-R08",
        subject: ByteRange::new(4, 11),
        subject_fragment: r"\u0069f",
        control_gold_id: "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "NULL-STATIC-R01",
        source: "let let = null;",
        rhs_range: ByteRange::new(10, 14),
        rule_id: "EE-15-R01",
        subject: ByteRange::new(4, 7),
        subject_fragment: "let",
        control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "NULL-STATIC-R02",
        source: "let x = null, x = foo;",
        rhs_range: ByteRange::new(8, 12),
        rule_id: "EE-15-R02",
        subject: ByteRange::new(14, 15),
        subject_fragment: "x",
        control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "NULL-STATIC-R03",
        source: "const x = null, y;",
        rhs_range: ByteRange::new(10, 14),
        rule_id: "EE-15-R03",
        subject: ByteRange::new(16, 17),
        subject_fragment: "y",
        control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "NULL-STATIC-EE36",
        source: "let x = null; let x = foo;",
        rhs_range: ByteRange::new(8, 12),
        rule_id: "EE-36-R01",
        subject: ByteRange::new(18, 19),
        subject_fragment: "x",
        control_gold_id: "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
];

const GRAMMAR_REACHABILITY_FIXTURES: &[GrammarReachabilityFixture] = &[
    GrammarReachabilityFixture {
        id: "NULL-GRAMMAR-LATER-EMPTY-BRACED",
        source: r"const x = null; let \u{};",
        rhs_range: ByteRange::new(10, 14),
        subject: ByteRange::new(20, 24),
        subject_fragment: r"\u{}",
        expected: FutureOutcome::SyntaxRejected,
    },
    GrammarReachabilityFixture {
        id: "NULL-GRAMMAR-LATER-PART-EMPTY-BRACED",
        source: r"const x = null; let a\u{};",
        rhs_range: ByteRange::new(10, 14),
        subject: ByteRange::new(21, 25),
        subject_fragment: r"\u{}",
        expected: FutureOutcome::SyntaxRejected,
    },
    GrammarReachabilityFixture {
        id: "NULL-GRAMMAR-LATER-UNCLOSED-BRACED",
        source: r"const x = null; let \u{61",
        rhs_range: ByteRange::new(10, 14),
        subject: ByteRange::new(20, 25),
        subject_fragment: r"\u{61",
        expected: FutureOutcome::SyntaxRejected,
    },
];

const UNCONDITIONALLY_RESERVED_WORDS: &[&str] = &[
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

fn slice(text: &str, range: ByteRange) -> &str {
    text.get(range.start..range.end)
        .unwrap_or_else(|| panic!("range {range:?} must be a UTF-8 boundary in {text:?}"))
}

fn is_selected_trivia_for_oracle(code_point: char) -> bool {
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
}

fn is_selected_identifier_start_for_oracle(code_point: u32) -> bool {
    code_point == '$' as u32 || code_point == '_' as u32 || is_id_start(code_point)
}

fn is_selected_identifier_part_for_oracle(code_point: u32) -> bool {
    code_point == '$' as u32 || is_id_continue(code_point)
}

fn hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some((byte - b'0') as u32),
        b'a'..=b'f' => Some((byte - b'a' + 10) as u32),
        b'A'..=b'F' => Some((byte - b'A' + 10) as u32),
        _ => None,
    }
}

fn parse_hex_digits(digits: &[u8]) -> Option<u32> {
    let first_significant = digits.iter().position(|byte| *byte != b'0');
    let significant = match first_significant {
        Some(index) => &digits[index..],
        None => return Some(0),
    };

    if significant.len() > 6 {
        return None;
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value.checked_mul(16)?.checked_add(hex_value(*byte)?)?;
    }
    (value <= 0x10_FFFF).then_some(value)
}

fn formed_unicode_escape_at_for_oracle(source: &str, start: usize) -> Option<EscapeFormation> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return None;
    }

    if bytes.get(start + 2) == Some(&b'{') {
        let digits_start = start + 3;
        let mut end = digits_start;
        while let Some(byte) = bytes.get(end) {
            if *byte == b'}' {
                let digits = bytes.get(digits_start..end)?;
                if digits.is_empty() || digits.iter().any(|byte| hex_value(*byte).is_none()) {
                    return None;
                }
                return Some(EscapeFormation {
                    end: end + 1,
                    code_point: parse_hex_digits(digits)?,
                });
            }
            hex_value(*byte)?;
            end += 1;
        }
        return None;
    }

    let digits = bytes.get(start + 2..start + 6)?;
    if digits.iter().any(|byte| hex_value(*byte).is_none()) {
        return None;
    }

    Some(EscapeFormation {
        end: start + 6,
        code_point: parse_hex_digits(digits)?,
    })
}

fn decode_identifier_name_for_oracle(rhs: &str) -> Option<String> {
    let mut offset = 0_usize;
    let mut first = true;
    let mut decoded = String::new();

    while offset < rhs.len() {
        let (code_point, end) = if rhs.as_bytes().get(offset) == Some(&b'\\') {
            let formation = formed_unicode_escape_at_for_oracle(rhs, offset)?;
            (formation.code_point, formation.end)
        } else {
            let code_point = rhs.get(offset..)?.chars().next()?;
            (code_point as u32, offset + code_point.len_utf8())
        };

        let valid = if first {
            is_selected_identifier_start_for_oracle(code_point)
        } else {
            is_selected_identifier_part_for_oracle(code_point)
        };
        if !valid {
            return None;
        }

        decoded.push(char::from_u32(code_point)?);
        first = false;
        offset = end;
    }

    (!first).then_some(decoded)
}

fn classify_authored_route(rhs: &str) -> AuthoredRoute {
    if rhs == "null" {
        return AuthoredRoute::DirectNullLiteral;
    }

    let Some(decoded) = decode_identifier_name_for_oracle(rhs) else {
        return AuthoredRoute::Unowned;
    };

    if UNCONDITIONALLY_RESERVED_WORDS.contains(&decoded.as_str()) {
        AuthoredRoute::ReservedIdentifierName
    } else {
        AuthoredRoute::IdentifierReference
    }
}

#[test]
fn frozen_authority_candidate_independence_and_historical_oracle_are_exact() {
    assert_eq!(ISSUE_ID, 249);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(
        PREVIOUS_BOOLEAN_ORACLE_SOURCE.contains("BOOLEAN-ROUTE-OTHER-RESERVED-NULL"),
        "accepted Boolean-only oracle must retain its historical direct-null control"
    );
    assert!(
        PREVIOUS_BOOLEAN_ORACLE_SOURCE.contains("BOOLEAN-UNSUPPORTED-OTHER-RESERVED-NULL"),
        "accepted Boolean-only oracle must retain its historical null unsupported control"
    );

    for forbidden_import in [
        concat!("use super::", "selected_lexical_slice"),
        concat!("use super::", "selected_static_semantics"),
        concat!("use super::", "selected_qualification_integration"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("consume_selected_", "null_literal"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden_import),
            "candidate-independent NullLiteral oracle must not import production authority: {forbidden_import}"
        );
    }
}

#[test]
fn direct_null_spelling_and_identifier_name_collision_are_independent() {
    assert!(UNCONDITIONALLY_RESERVED_WORDS.contains(&"null"));

    for fixture in ROUTE_FIXTURES {
        assert_eq!(
            classify_authored_route(fixture.rhs),
            fixture.expected_route,
            "{}",
            fixture.id
        );

        match fixture.expected_route {
            AuthoredRoute::DirectNullLiteral | AuthoredRoute::IdentifierReference => assert_eq!(
                fixture.expected_outcome,
                FutureOutcome::SelectedAcceptedIncomplete,
                "{}",
                fixture.id
            ),
            AuthoredRoute::ReservedIdentifierName | AuthoredRoute::Unowned => assert_eq!(
                fixture.expected_outcome,
                FutureOutcome::UnsupportedCoverage,
                "{}",
                fixture.id
            ),
        }
    }

    assert_eq!(
        decode_identifier_name_for_oracle(r"null\u0061").as_deref(),
        Some("nulla")
    );
    assert_eq!(
        decode_identifier_name_for_oracle(r"\u006Eull").as_deref(),
        Some("null")
    );
    assert_eq!(
        decode_identifier_name_for_oracle(r"n\u0075ll").as_deref(),
        Some("null")
    );
}

#[test]
fn positive_fixtures_pin_authored_ranges_boundaries_and_future_lifecycle() {
    for (index, fixture) in POSITIVE_FIXTURES.iter().enumerate() {
        assert_eq!(
            slice(fixture.source, fixture.rhs_range),
            fixture.rhs,
            "{}",
            fixture.id
        );
        assert_eq!(
            classify_authored_route(fixture.rhs),
            AuthoredRoute::DirectNullLiteral,
            "{}",
            fixture.id
        );
        assert_eq!(
            fixture.expected,
            FutureOutcome::SelectedAcceptedIncomplete,
            "{}",
            fixture.id
        );

        let source = SourceText::new(
            SourceId::new(249_000 + index as u64),
            fixture.source.to_owned(),
        );
        assert_eq!(
            source
                .anchor(fixture.rhs_range.start, fixture.rhs_range.end)
                .expect("NullLiteral RHS range must be a valid authored UTF-8 anchor")
                .fragment(),
            fixture.rhs,
            "{}",
            fixture.id
        );

        let trivia = fixture
            .source
            .get(fixture.rhs_range.end..fixture.boundary_offset)
            .unwrap_or_else(|| panic!("{} must have valid boundary offsets", fixture.id));
        assert!(
            trivia.chars().all(is_selected_trivia_for_oracle),
            "{} has non-selected trivia before boundary: {trivia:?}",
            fixture.id
        );

        match fixture.boundary {
            BoundaryKind::BindingListComma => assert_eq!(
                fixture.source.as_bytes().get(fixture.boundary_offset),
                Some(&b','),
                "{}",
                fixture.id
            ),
            BoundaryKind::AuthoredSemicolon => assert_eq!(
                fixture.source.as_bytes().get(fixture.boundary_offset),
                Some(&b';'),
                "{}",
                fixture.id
            ),
            BoundaryKind::AutomaticAtEof => assert_eq!(
                fixture.boundary_offset,
                fixture.source.len(),
                "{} must reach actual EOF after selected trivia",
                fixture.id
            ),
        }
    }
}

#[test]
fn unsupported_controls_pin_whole_source_disposition_without_local_commit_policy() {
    for (index, fixture) in UNSUPPORTED_FIXTURES.iter().enumerate() {
        let rhs = slice(fixture.source, fixture.rhs_range);
        let source = SourceText::new(
            SourceId::new(249_200 + index as u64),
            fixture.source.to_owned(),
        );
        assert_eq!(
            source
                .anchor(fixture.rhs_range.start, fixture.rhs_range.end)
                .expect("unsupported RHS range must stay authored")
                .fragment(),
            rhs,
            "{}",
            fixture.id
        );
        assert_eq!(
            fixture.expected,
            FutureOutcome::UnsupportedCoverage,
            "{}",
            fixture.id
        );

        match fixture.reason {
            UnsupportedReason::EscapedNullLikeIdentifierName => assert_eq!(
                classify_authored_route(rhs),
                AuthoredRoute::ReservedIdentifierName,
                "{}",
                fixture.id
            ),
            UnsupportedReason::FormedInvalidContinuation
            | UnsupportedReason::MalformedContinuation => {
                assert_eq!(
                    classify_authored_route(rhs),
                    AuthoredRoute::Unowned,
                    "{}",
                    fixture.id
                );
            }
            UnsupportedReason::OtherReservedDirectRhs => assert_eq!(
                classify_authored_route(rhs),
                AuthoredRoute::ReservedIdentifierName,
                "{}",
                fixture.id
            ),
            UnsupportedReason::MemberExpression
            | UnsupportedReason::CallExpression
            | UnsupportedReason::BinaryExpression
            | UnsupportedReason::AssignmentExpression
            | UnsupportedReason::ConditionalExpression
            | UnsupportedReason::Comment
            | UnsupportedReason::UnexpectedTail
            | UnsupportedReason::ExtraSemicolon
            | UnsupportedReason::RequiresNonEofAsi => {
                assert_eq!(rhs, "null", "{}", fixture.id);
                assert_eq!(
                    classify_authored_route(rhs),
                    AuthoredRoute::DirectNullLiteral,
                    "{}",
                    fixture.id
                );
            }
        }
    }
}

#[test]
fn static_reachability_preserves_existing_subjects_and_outcome_family() {
    for (index, fixture) in STATIC_REACHABILITY_FIXTURES.iter().enumerate() {
        assert_eq!(
            slice(fixture.source, fixture.rhs_range),
            "null",
            "{}",
            fixture.id
        );
        assert_eq!(
            classify_authored_route(slice(fixture.source, fixture.rhs_range)),
            AuthoredRoute::DirectNullLiteral,
            "{}",
            fixture.id
        );
        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment,
            "{} / {}",
            fixture.id,
            fixture.rule_id
        );
        assert_eq!(
            fixture.expected,
            FutureOutcome::StaticSemanticsRejected,
            "{}",
            fixture.id
        );
        assert!(
            gold_source(fixture.control_gold_id).is_some(),
            "{} must retain its accepted static control authority",
            fixture.id
        );

        let source = SourceText::new(
            SourceId::new(249_400 + index as u64),
            fixture.source.to_owned(),
        );
        assert_eq!(
            source
                .anchor(fixture.subject.start, fixture.subject.end)
                .expect("static subject must be a valid authored anchor")
                .fragment(),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );
    }
}

#[test]
fn later_existing_grammar_reachability_preserves_authored_subjects() {
    for (index, fixture) in GRAMMAR_REACHABILITY_FIXTURES.iter().enumerate() {
        assert_eq!(
            slice(fixture.source, fixture.rhs_range),
            "null",
            "{}",
            fixture.id
        );
        assert_eq!(
            classify_authored_route(slice(fixture.source, fixture.rhs_range)),
            AuthoredRoute::DirectNullLiteral,
            "{}",
            fixture.id
        );
        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );
        assert_eq!(
            fixture.expected,
            FutureOutcome::SyntaxRejected,
            "{}",
            fixture.id
        );

        let source = SourceText::new(
            SourceId::new(249_500 + index as u64),
            fixture.source.to_owned(),
        );
        assert_eq!(
            source
                .anchor(fixture.subject.start, fixture.subject.end)
                .expect("Grammar subject must be a valid authored anchor")
                .fragment(),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );
    }
}

#[test]
fn future_handoff_stays_incomplete_unsupported_where_unowned_and_validation_only() {
    assert!(
        POSITIVE_FIXTURES
            .iter()
            .all(|fixture| fixture.expected == FutureOutcome::SelectedAcceptedIncomplete)
    );
    assert!(
        UNSUPPORTED_FIXTURES
            .iter()
            .all(|fixture| fixture.expected == FutureOutcome::UnsupportedCoverage)
    );
    assert!(
        STATIC_REACHABILITY_FIXTURES
            .iter()
            .all(|fixture| fixture.expected == FutureOutcome::StaticSemanticsRejected)
    );
    assert!(
        GRAMMAR_REACHABILITY_FIXTURES
            .iter()
            .all(|fixture| fixture.expected == FutureOutcome::SyntaxRejected)
    );

    for forbidden_retained_or_qualified_state in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("CompleteQualification", "Witness"),
        concat!("Null", "Value"),
        concat!("SelectedExpression", "Kind"),
        concat!("NullLiteral", "Kind"),
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden_retained_or_qualified_state),
            "validation must not introduce retained/qualified production state: {forbidden_retained_or_qualified_state}"
        );
    }
}
