//! Candidate-independent direct `BooleanLiteral` initializer validation for Issue #245.
//!
//! This module fixes the direct `true` / `false` source family, its collision
//! with maximal direct/escaped `IdentifierName` spelling, selected boundaries,
//! and future lifecycle without calling production lexical, static-semantics,
//! aggregate, runtime-evaluation, or future Boolean recognition code.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode::{is_id_continue, is_id_start};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 245;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const THIS_SOURCE: &str =
    include_str!("qualification_selected_boolean_literal_initializer_validation_tests.rs");

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
    DirectBooleanLiteral,
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
    EscapedBooleanLikeIdentifierName,
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
        "BOOLEAN-POSITIVE-TRUE-SEMICOLON",
        "const x = true;",
        "true",
        ByteRange::new(10, 14),
        14,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-FALSE-SEMICOLON",
        "const x = false;",
        "false",
        ByteRange::new(10, 15),
        15,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-TRUE-EOF",
        "const x = true",
        "true",
        ByteRange::new(10, 14),
        14,
        BoundaryKind::AutomaticAtEof,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-FALSE-TRAILING-TRIVIA-EOF",
        "const x = false   \t\n",
        "false",
        ByteRange::new(10, 15),
        20,
        BoundaryKind::AutomaticAtEof,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-TRUE-COMMA-FIRST",
        "let x = true, y = foo;",
        "true",
        ByteRange::new(8, 12),
        12,
        BoundaryKind::BindingListComma,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-FALSE-COMMA-SECOND",
        "let x = foo, y = false;",
        "false",
        ByteRange::new(17, 22),
        22,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-TRUE-AFTER-DECIMAL",
        "let x = 1, y = true;",
        "true",
        ByteRange::new(15, 19),
        19,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-TWO-BOOLEAN-BINDINGS-FIRST",
        "const x = true, y = false;",
        "true",
        ByteRange::new(10, 14),
        14,
        BoundaryKind::BindingListComma,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-TWO-BOOLEAN-BINDINGS-SECOND",
        "const x = true, y = false;",
        "false",
        ByteRange::new(20, 25),
        25,
        BoundaryKind::AuthoredSemicolon,
    ),
    positive_fixture(
        "BOOLEAN-POSITIVE-TRIVIA-BEFORE-SEMICOLON",
        "const x = true \t\n;",
        "true",
        ByteRange::new(10, 14),
        17,
        BoundaryKind::AuthoredSemicolon,
    ),
];

const ROUTE_FIXTURES: &[RouteFixture] = &[
    RouteFixture {
        id: "BOOLEAN-ROUTE-DIRECT-TRUE",
        rhs: "true",
        expected_route: AuthoredRoute::DirectBooleanLiteral,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-DIRECT-FALSE",
        rhs: "false",
        expected_route: AuthoredRoute::DirectBooleanLiteral,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-LONGER-TRUEX",
        rhs: "truex",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-LONGER-FALSEVALUE",
        rhs: "falseValue",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-TRUE-FORMED-UES-CONTINUATION",
        rhs: r"true\u0061",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-FALSE-FORMED-UES-CONTINUATION",
        rhs: r"false\u0061",
        expected_route: AuthoredRoute::IdentifierReference,
        expected_outcome: FutureOutcome::SelectedAcceptedIncomplete,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-ESCAPED-TRUE",
        rhs: r"\u0074rue",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-ESCAPED-FALSE",
        rhs: r"\u0066alse",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-MIXED-ESCAPED-TRUE",
        rhs: r"t\u0072ue",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-MIXED-ESCAPED-FALSE",
        rhs: r"f\u0061lse",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-OTHER-RESERVED-NULL",
        rhs: "null",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-OTHER-RESERVED-THIS",
        rhs: "this",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-OTHER-RESERVED-IF",
        rhs: "if",
        expected_route: AuthoredRoute::ReservedIdentifierName,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
    RouteFixture {
        id: "BOOLEAN-ROUTE-MALFORMED-CONTINUATION",
        rhs: r"true\u{}",
        expected_route: AuthoredRoute::Unowned,
        expected_outcome: FutureOutcome::UnsupportedCoverage,
    },
];

const UNSUPPORTED_FIXTURES: &[UnsupportedFixture] = &[
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-ESCAPED-TRUE",
        source: r"const x = \u0074rue;",
        rhs_range: ByteRange::new(10, 19),
        reason: UnsupportedReason::EscapedBooleanLikeIdentifierName,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-ESCAPED-FALSE",
        source: r"const x = \u0066alse;",
        rhs_range: ByteRange::new(10, 20),
        reason: UnsupportedReason::EscapedBooleanLikeIdentifierName,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-MALFORMED-CONTINUATION-EMPTY-BRACED",
        source: r"const x = true\u{};",
        rhs_range: ByteRange::new(10, 18),
        reason: UnsupportedReason::MalformedContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-MALFORMED-CONTINUATION-SHORT",
        source: r"const x = false\u0;",
        rhs_range: ByteRange::new(10, 18),
        reason: UnsupportedReason::MalformedContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-MALFORMED-CONTINUATION-UNCLOSED",
        source: r"const x = true\u{61;",
        rhs_range: ByteRange::new(10, 19),
        reason: UnsupportedReason::MalformedContinuation,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-OTHER-RESERVED-NULL",
        source: "const x = null;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::OtherReservedDirectRhs,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-OTHER-RESERVED-THIS",
        source: "const x = this;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::OtherReservedDirectRhs,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-MEMBER",
        source: "const x = true.foo;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::MemberExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-CALL",
        source: "const x = false();",
        rhs_range: ByteRange::new(10, 15),
        reason: UnsupportedReason::CallExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-BINARY",
        source: "const x = true + x;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::BinaryExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-ASSIGNMENT",
        source: "const x = false = x;",
        rhs_range: ByteRange::new(10, 15),
        reason: UnsupportedReason::AssignmentExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-CONDITIONAL",
        source: "const x = true ? x : y;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::ConditionalExpression,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-COMMENT",
        source: "const x = true/*comment*/;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::Comment,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-UNEXPECTED-TAIL",
        source: "const x = true unexpected;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::UnexpectedTail,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-EXTRA-SEMICOLON",
        source: "const x = true;;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::ExtraSemicolon,
        expected: FutureOutcome::UnsupportedCoverage,
    },
    UnsupportedFixture {
        id: "BOOLEAN-UNSUPPORTED-NON-EOF-ASI",
        source: "const x = true\nconst y = foo;",
        rhs_range: ByteRange::new(10, 14),
        reason: UnsupportedReason::RequiresNonEofAsi,
        expected: FutureOutcome::UnsupportedCoverage,
    },
];

const STATIC_REACHABILITY_FIXTURES: &[StaticReachabilityFixture] = &[
    StaticReachabilityFixture {
        id: "BOOLEAN-STATIC-EE01",
        source: r"let \u0030 = true;",
        rhs_range: ByteRange::new(13, 17),
        rule_id: "EE-01-R01",
        subject: ByteRange::new(4, 10),
        subject_fragment: r"\u0030",
        control_gold_id: "JS-GOLD-IDENTIFIER-ESCAPED-START-DIGIT-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "BOOLEAN-STATIC-EE04",
        source: r"let \u0069f = true;",
        rhs_range: ByteRange::new(14, 18),
        rule_id: "EE-04-R08",
        subject: ByteRange::new(4, 11),
        subject_fragment: r"\u0069f",
        control_gold_id: "JS-GOLD-IDENTIFIER-ESCAPED-RESERVED-WORD-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "BOOLEAN-STATIC-R01",
        source: "let let = true;",
        rhs_range: ByteRange::new(10, 14),
        rule_id: "EE-15-R01",
        subject: ByteRange::new(4, 7),
        subject_fragment: "let",
        control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "BOOLEAN-STATIC-R02",
        source: "let x = true, x = foo;",
        rhs_range: ByteRange::new(8, 12),
        rule_id: "EE-15-R02",
        subject: ByteRange::new(14, 15),
        subject_fragment: "x",
        control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "BOOLEAN-STATIC-R03",
        source: "const x = true, y;",
        rhs_range: ByteRange::new(10, 14),
        rule_id: "EE-15-R03",
        subject: ByteRange::new(16, 17),
        subject_fragment: "y",
        control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        expected: FutureOutcome::StaticSemanticsRejected,
    },
    StaticReachabilityFixture {
        id: "BOOLEAN-STATIC-EE36",
        source: "let x = true; let x = foo;",
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
        id: "BOOLEAN-GRAMMAR-LATER-EMPTY-BRACED",
        source: r"const x = true; let \u{};",
        rhs_range: ByteRange::new(10, 14),
        subject: ByteRange::new(20, 24),
        subject_fragment: r"\u{}",
        expected: FutureOutcome::SyntaxRejected,
    },
    GrammarReachabilityFixture {
        id: "BOOLEAN-GRAMMAR-LATER-PART-EMPTY-BRACED",
        source: r"const x = true; let a\u{};",
        rhs_range: ByteRange::new(10, 14),
        subject: ByteRange::new(21, 25),
        subject_fragment: r"\u{}",
        expected: FutureOutcome::SyntaxRejected,
    },
    GrammarReachabilityFixture {
        id: "BOOLEAN-GRAMMAR-LATER-UNCLOSED-BRACED",
        source: r"const x = true; let \u{61",
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
            if hex_value(*byte).is_none() {
                return None;
            }
            end += 1;
        }
        return None;
    }

    let digits = bytes.get(start + 2..start + 6)?;
    if digits.len() != 4 || digits.iter().any(|byte| hex_value(*byte).is_none()) {
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
    if matches!(rhs, "true" | "false") {
        return AuthoredRoute::DirectBooleanLiteral;
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
fn frozen_authority_and_candidate_independence_are_exact() {
    assert_eq!(ISSUE_ID, 245);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    for forbidden_import in [
        "use super::selected_lexical_slice",
        "use super::selected_static_semantics",
        "use super::selected_qualification_integration",
        "parse_selected_binding_identifier",
        "consume_selected_identifier_reference",
    ] {
        assert!(
            !THIS_SOURCE.contains(forbidden_import),
            "candidate-independent Boolean oracle must not import production authority: {forbidden_import}"
        );
    }
}

#[test]
fn direct_boolean_spelling_and_identifier_name_collision_are_independent() {
    assert!(UNCONDITIONALLY_RESERVED_WORDS.contains(&"true"));
    assert!(UNCONDITIONALLY_RESERVED_WORDS.contains(&"false"));

    for fixture in ROUTE_FIXTURES {
        assert_eq!(
            classify_authored_route(fixture.rhs),
            fixture.expected_route,
            "{}",
            fixture.id
        );

        match fixture.expected_route {
            AuthoredRoute::DirectBooleanLiteral | AuthoredRoute::IdentifierReference => {
                assert_eq!(
                    fixture.expected_outcome,
                    FutureOutcome::SelectedAcceptedIncomplete,
                    "{}",
                    fixture.id
                );
            }
            AuthoredRoute::ReservedIdentifierName | AuthoredRoute::Unowned => assert_eq!(
                fixture.expected_outcome,
                FutureOutcome::UnsupportedCoverage,
                "{}",
                fixture.id
            ),
        }
    }

    assert_eq!(
        decode_identifier_name_for_oracle(r"true\u0061").as_deref(),
        Some("truea")
    );
    assert_eq!(
        decode_identifier_name_for_oracle(r"false\u0061").as_deref(),
        Some("falsea")
    );
    assert_eq!(
        decode_identifier_name_for_oracle(r"\u0074rue").as_deref(),
        Some("true")
    );
    assert_eq!(
        decode_identifier_name_for_oracle(r"\u0066alse").as_deref(),
        Some("false")
    );
}

#[test]
fn positive_fixtures_pin_authored_ranges_boundaries_and_future_lifecycle() {
    for (index, fixture) in POSITIVE_FIXTURES.iter().enumerate() {
        assert_eq!(slice(fixture.source, fixture.rhs_range), fixture.rhs, "{}", fixture.id);
        assert_eq!(
            classify_authored_route(fixture.rhs),
            AuthoredRoute::DirectBooleanLiteral,
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
            SourceId::new(245_000 + index as u64),
            fixture.source.to_owned(),
        );
        assert_eq!(
            source
                .anchor(fixture.rhs_range.start, fixture.rhs_range.end)
                .expect("Boolean RHS range must be a valid authored UTF-8 anchor")
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
            SourceId::new(245_200 + index as u64),
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
            UnsupportedReason::EscapedBooleanLikeIdentifierName => assert_eq!(
                classify_authored_route(rhs),
                AuthoredRoute::ReservedIdentifierName,
                "{}",
                fixture.id
            ),
            UnsupportedReason::MalformedContinuation => {
                assert_eq!(classify_authored_route(rhs), AuthoredRoute::Unowned, "{}", fixture.id);
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
                assert!(matches!(rhs, "true" | "false"), "{}", fixture.id);
                assert_eq!(
                    classify_authored_route(rhs),
                    AuthoredRoute::DirectBooleanLiteral,
                    "{}",
                    fixture.id
                );
            }
        }
    }

    assert!(THIS_SOURCE.contains("whole-source future disposition = UnsupportedCoverage"));
    assert!(THIS_SOURCE.contains("Local recognizer commit semantics belong to the later production architecture gate"));
}

#[test]
fn static_reachability_preserves_existing_subjects_and_outcome_family() {
    for (index, fixture) in STATIC_REACHABILITY_FIXTURES.iter().enumerate() {
        assert_eq!(slice(fixture.source, fixture.rhs_range), "true", "{}", fixture.id);
        assert_eq!(
            classify_authored_route(slice(fixture.source, fixture.rhs_range)),
            AuthoredRoute::DirectBooleanLiteral,
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
            SourceId::new(245_400 + index as u64),
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
        assert_eq!(slice(fixture.source, fixture.rhs_range), "true", "{}", fixture.id);
        assert_eq!(
            classify_authored_route(slice(fixture.source, fixture.rhs_range)),
            AuthoredRoute::DirectBooleanLiteral,
            "{}",
            fixture.id
        );
        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment,
            "{}",
            fixture.id
        );
        assert_eq!(fixture.expected, FutureOutcome::SyntaxRejected, "{}", fixture.id);

        let source = SourceText::new(
            SourceId::new(245_500 + index as u64),
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
fn future_handoff_stays_presence_only_incomplete_and_validation_only() {
    assert!(POSITIVE_FIXTURES
        .iter()
        .all(|fixture| fixture.expected == FutureOutcome::SelectedAcceptedIncomplete));
    assert!(UNSUPPORTED_FIXTURES
        .iter()
        .all(|fixture| fixture.expected == FutureOutcome::UnsupportedCoverage));
    assert!(STATIC_REACHABILITY_FIXTURES
        .iter()
        .all(|fixture| fixture.expected == FutureOutcome::StaticSemanticsRejected));
    assert!(GRAMMAR_REACHABILITY_FIXTURES
        .iter()
        .all(|fixture| fixture.expected == FutureOutcome::SyntaxRejected));

    assert!(!THIS_SOURCE.contains("ExpectedQualification::Qualified"));
    assert!(!THIS_SOURCE.contains("CompleteQualificationWitness"));
    assert!(!THIS_SOURCE.contains("BooleanValue"));
    assert!(!THIS_SOURCE.contains("SelectedExpressionKind"));
}
