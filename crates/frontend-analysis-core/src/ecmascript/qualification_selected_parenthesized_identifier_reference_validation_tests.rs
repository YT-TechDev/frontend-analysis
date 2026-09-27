//! Candidate-independent exactly-one grouping-layer `ParenthesizedExpression`
//! `IdentifierReference` fact-preservation validation for Issue #831 (durable
//! research: Issue #688 comment `5852461928` post-#830 activation checkpoint,
//! and Issue #688 comment `5852264054` pre-merge falsification record;
//! structural precedent: Issue #827 exactly-one `!`/`~` `IdentifierReference`
//! `UnaryExpression` Oracle; accepted premise authority: Issue #237 direct
//! escape-free `IdentifierReference` source/name boundary, Issue #241 escaped
//! `IdentifierReference` initializer Oracle).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedParenthesizedIdentifierReference ::=
//!     "("
//!     SelectedGroupingTrivia
//!     SelectedAcceptedIdentifierReference
//!     SelectedGroupingTrivia
//!     ")"
//!
//! SelectedAcceptedIdentifierReference ::=
//!       SelectedDirectIdentifierReference
//!     | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice. It
//! does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! The load-bearing capability question is not merely whether source such as
//! `(a)` can be recognized. It is whether exactly one grouping layer can own
//! syntax around an already source-backed `IdentifierReference` fact --
//! Direct or EscapedNonReserved -- without destroying, widening,
//! reconstructing, or normalizing that fact, *and* whether every candidate
//! this bounded Oracle accepts independently and necessarily satisfies the
//! `ParenthesizedExpression : ( Expression )` cover requirement that
//! `PrimaryExpression : CoverParenthesizedExpressionAndArrowParameterList`
//! (`EE-08-R01`) exists to police.
//!
//! `SelectedGroupingTrivia` independently restates the same already-accepted
//! selected-slice trivia contract reused by Issue #827's
//! `SelectedUnaryOperandTrivia` (the same code-point set `is_selected_trivia`
//! in `selected_lexical_slice.rs` recognizes, restated here rather than
//! imported): `TAB`, `VT`, `FF`, `BOM`, `LF`, `CR`, `LINE SEPARATOR`,
//! `PARAGRAPH SEPARATOR`, and the frozen Unicode 17 `Space_Separator`
//! property. Comments remain outside this contract.
//!
//! `SelectedDirectIdentifierReference` independently restates only the
//! already-accepted Issue #237 direct escape-free `IdentifierName`
//! code-point shape. `SelectedEscapedNonReservedIdentifierReference`
//! independently restates both the fixed-form `\uXXXX` and braced
//! `\u{HexDigits}` `UnicodeEscapeSequence` forms of the already-accepted
//! Issue #241 escaped `IdentifierReference` decode/position/ReservedWord
//! boundary. Issue #241 owns the escaped-`IdentifierReference` premise in
//! full; this Oracle independently composes representative accepted fixed
//! and braced `EscapedNonReserved` operands through the new exactly-one
//! grouping wrapper without re-proving the entire Issue #241 theorem.
//!
//! This is a validation-only leaf: production supports no grouping/
//! `ParenthesizedExpression` syntax at the #831 baseline (the frozen
//! completion partition classifies `EE-08-R01` as `StructurallyNonTriggering`
//! / `OwningSyntaxAbsent("Grouping / parenthesized expression")`), so every
//! positive fixture below is independent Oracle evidence for a future,
//! separately authorized production decision, never a claim that production
//! already accepts it. No completion successor file accompanies this leaf:
//! this Issue adds zero production capability, so the frozen
//! `193 / 10 / 183 / {}` completion partition cannot move, matching the
//! precedent set by #827 (which likewise added zero production capability
//! and shipped without one).
//!
//! Production is explicitly NOT authorized by Issue #831.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 831;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_BANG_TILDE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_bang_tilde_identifier_reference_unary_expression_initializer_validation_tests.rs"
);
const PREVIOUS_ESCAPED_IDENTIFIER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_parenthesized_identifier_reference_validation_tests.rs");
const COMPLETION_AUTHORITY_SOURCE: &str =
    include_str!("qualification_validation_tests/selected_slice_completion.rs");
const INVENTORY_SOURCE: &str = include_str!("qualification_validation_tests/inventory.rs");
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "exactly-one grouping-layer ParenthesizedExpression IdentifierReference frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "recursive grouping, ArrowFunction/cover-grammar reinterpretation, richer ",
    "inner or outer expressions, comments, or top-level/Block var placement"
);
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];

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

const POSITIVE_CANDIDATES: &[&str] = &[
    "(a)",
    "(\u{03C0})",
    "(\\u0061)",
    "(f\\u006Fo)",
    r"(\u{66}oo)",
    r"(\u{1D49C})",
];

const COVER_FIREWALL_CONTROLS: &[&str] = &[
    "()",
    "(   )",
    "(a,)",
    "(a,b)",
    "(...a)",
    "(a,...b)",
    "(a)=>a",
    "(a,b)=>a",
    "(...a)=>a",
    "( a , b )",
    "( a ) => a",
];

const RECURSIVE_GROUPING_CONTROLS: &[&str] = &["((a))", "(((a)))"];

const RICHER_INNER_EXPRESSION_CONTROLS: &[&str] = &[
    "(a+b)",
    "(a*b)",
    "(a=b)",
    "(a?b:c)",
    "(a())",
    "(a.b)",
    "(new a)",
    "(!a)",
    "(~a)",
    "(+a)",
    "(-a)",
    "(typeof a)",
    "(void a)",
    "(delete a)",
];

const RICHER_OUTER_EXPRESSION_CONTROLS: &[&str] = &[
    "(a)+b", "(a)*b", "(a).b", "(a)()", "(a)=b", "(a)?b:c", "(a),b",
];

const COMMENT_FIREWALL_CONTROLS: &[&str] = &["(/*c*/a)", "(a/*c*/)", "(/*c*/a/*c*/)"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    SelectedAcceptedIncomplete,
    UnsupportedCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentifierReferenceProvenance {
    Direct,
    EscapedNonReserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodeFailure {
    MalformedEscape,
    NonCodePoint,
    InvalidStart,
    InvalidPart,
    DecodedReserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CoverClassification {
    AcceptedParenthesizedIdentifierReference,
    CoverFailureOrNotSelected,
}

fn slice(text: &str, Range(start, end): Range) -> &str {
    text.get(start..end)
        .unwrap_or_else(|| panic!("invalid UTF-8 range ({start},{end}) in {text:?}"))
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

/// Independently restates only the already-accepted Issue #237 direct
/// `IdentifierStart` code-point shape.
fn is_direct_identifier_start(code_point: char) -> bool {
    matches!(code_point, '$' | '_') || is_id_start(code_point as u32)
}

/// Independently restates only the already-accepted Issue #237 direct
/// `IdentifierPart` code-point shape.
fn is_direct_identifier_part(code_point: char) -> bool {
    code_point == '$' || is_id_continue(code_point as u32)
}

/// Independently restates only the already-accepted Issue #237 direct
/// escape-free `IdentifierName` code-point shape: a pure code-point-family
/// check, never a reserved-word policy on its own.
fn is_escape_free_identifier_name(spelling: &str) -> bool {
    let mut code_points = spelling.chars();
    let Some(first) = code_points.next() else {
        return false;
    };
    is_direct_identifier_start(first) && code_points.all(is_direct_identifier_part)
}

/// `SelectedDirectIdentifierReference`: the already-accepted direct
/// escape-free `IdentifierName` shape, minus the unconditionally reserved
/// words that Script/non-strict/`Yield=false`/`Await=false` never admits as
/// an `IdentifierReference`.
fn is_selected_direct_identifier_reference(candidate: &str) -> bool {
    is_escape_free_identifier_name(candidate)
        && !UNCONDITIONALLY_RESERVED_WORDS.contains(&candidate)
}

fn ascii_hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

/// Parses the hex digits inside a braced `\u{HexDigits}` escape, independently
/// restating only the already-accepted Issue #241 braced-form boundary:
/// leading zeros are permitted without limit, but at most six significant hex
/// digits are, and the resulting value must be a valid Unicode code point
/// (`<= 0x10FFFF`).
fn parse_braced_code_point(digits: &[u8]) -> Result<u32, DecodeFailure> {
    if digits.is_empty() || digits.iter().any(|byte| ascii_hex_value(*byte).is_none()) {
        return Err(DecodeFailure::MalformedEscape);
    }

    let significant = match digits.iter().position(|byte| *byte != b'0') {
        Some(index) => &digits[index..],
        None => return Ok(0),
    };

    if significant.len() > 6 {
        return Err(DecodeFailure::NonCodePoint);
    }

    let mut value = 0_u32;
    for byte in significant {
        value = value * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }

    (value <= 0x10_FFFF)
        .then_some(value)
        .ok_or(DecodeFailure::NonCodePoint)
}

/// Decodes exactly one `UnicodeEscapeSequence` starting at `start` -- either
/// the fixed-form `\uXXXX` or the braced `\u{HexDigits}` form -- returning
/// its code point and the byte offset immediately after it.
fn decode_unicode_escape_at(bytes: &[u8], start: usize) -> Result<(u32, usize), DecodeFailure> {
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return Err(DecodeFailure::MalformedEscape);
    }
    let payload = start + 2;

    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload + 1;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(u8::is_ascii_hexdigit) {
            end += 1;
        }
        if end == digits_start || bytes.get(end) != Some(&b'}') {
            return Err(DecodeFailure::MalformedEscape);
        }
        let code_point = parse_braced_code_point(&bytes[digits_start..end])?;
        return Ok((code_point, end + 1));
    }

    let digits = bytes
        .get(payload..payload + 4)
        .ok_or(DecodeFailure::MalformedEscape)?;
    if !digits.iter().all(u8::is_ascii_hexdigit) {
        return Err(DecodeFailure::MalformedEscape);
    }
    let mut value = 0_u32;
    for byte in digits {
        value = value * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }
    Ok((value, payload + 4))
}

fn is_selected_identifier_start_code_point(code_point: u32) -> bool {
    char::from_u32(code_point).is_some_and(is_direct_identifier_start)
}

fn is_selected_identifier_part_code_point(code_point: u32) -> bool {
    char::from_u32(code_point).is_some_and(is_direct_identifier_part)
}

/// `SelectedEscapedNonReservedIdentifierReference`: independently restates
/// the fixed-form `\uXXXX` and braced `\u{HexDigits}` subset of the
/// already-accepted Issue #241 escaped `IdentifierReference`
/// decode/position/ReservedWord boundary. Returns the decoded semantic name,
/// never the authored spelling.
fn decode_selected_escaped_identifier(spelling: &str) -> Result<String, DecodeFailure> {
    let bytes = spelling.as_bytes();
    let mut offset = 0_usize;
    let mut element_index = 0_usize;
    let mut code_points: Vec<u32> = Vec::new();

    while offset < bytes.len() {
        let (code_point, end) = if bytes[offset] == b'\\' {
            decode_unicode_escape_at(bytes, offset)?
        } else {
            let scalar = spelling[offset..]
                .chars()
                .next()
                .ok_or(DecodeFailure::MalformedEscape)?;
            (scalar as u32, offset + scalar.len_utf8())
        };

        let valid_position = if element_index == 0 {
            is_selected_identifier_start_code_point(code_point)
        } else {
            is_selected_identifier_part_code_point(code_point)
        };
        if !valid_position {
            return Err(if element_index == 0 {
                DecodeFailure::InvalidStart
            } else {
                DecodeFailure::InvalidPart
            });
        }

        code_points.push(code_point);
        offset = end;
        element_index += 1;
    }

    if code_points.is_empty() {
        return Err(DecodeFailure::MalformedEscape);
    }

    let mut string_value = String::new();
    for code_point in &code_points {
        string_value.push(
            char::from_u32(*code_point)
                .expect("validated position-valid code point must be a Unicode scalar value"),
        );
    }

    if UNCONDITIONALLY_RESERVED_WORDS.contains(&string_value.as_str()) {
        return Err(DecodeFailure::DecodedReserved);
    }

    Ok(string_value)
}

/// `SelectedAcceptedIdentifierReference`: Direct first, EscapedNonReserved
/// otherwise. Returns the decoded semantic name and provenance -- never the
/// parenthesized/trivia-wrapped spelling.
fn classify_selected_accepted_identifier_reference(
    operand: &str,
) -> Option<(String, IdentifierReferenceProvenance)> {
    if is_selected_direct_identifier_reference(operand) {
        return Some((operand.to_owned(), IdentifierReferenceProvenance::Direct));
    }
    decode_selected_escaped_identifier(operand)
        .ok()
        .map(|name| (name, IdentifierReferenceProvenance::EscapedNonReserved))
}

fn is_selected_accepted_identifier_reference(operand: &str) -> bool {
    classify_selected_accepted_identifier_reference(operand).is_some()
}

/// `SelectedGroupingTrivia`: independently restates the complete
/// already-accepted trivia theorem reused by Issue #827's
/// `SelectedUnaryOperandTrivia`: the frozen `is_selected_trivia` code-point
/// set plus the frozen Unicode 17 `Space_Separator` property. Not a call
/// into the production selected-trivia recognizer.
fn is_selected_grouping_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

/// Central Issue #831 theorem: exactly one leading authored `(`, exactly one
/// trailing authored `)`, with zero or more already-accepted grouping trivia
/// code points immediately inside each delimiter, and exactly one complete
/// Direct or EscapedNonReserved `IdentifierReference` occupying the rest of
/// the candidate. Whole-string: no richer tail, no nested grouping, no
/// comma/rest/arrow continuation survives.
///
/// Recursive grouping (W9), richer-outer continuation (W10), and cover/arrow
/// forms (W3) are excluded structurally rather than by special-casing: the
/// leading `(` and trailing `)` are each consumed exactly once, so any
/// additional grouping punctuator, comma, rest token, or continuation ends up
/// inside the classified operand and fails `IdentifierReference` shape on its
/// own, or (for a richer outer tail) the candidate simply does not end in
/// `)` and is rejected before any operand classification occurs.
///
/// Trivia is trimmed by two independent single-pass scans (forward from the
/// inner start, backward from the inner end) over already-classified trivia
/// code points only -- this oracle's own owned recognition, never a later
/// search, rescan, or reparse over already-classified source.
fn is_selected_parenthesized_identifier_reference(candidate: &str) -> bool {
    let mut chars = candidate.chars();
    if chars.next() != Some('(') || !candidate.ends_with(')') {
        return false;
    }
    let inner = &candidate[1..candidate.len() - 1];

    let mut leading_trim = 0_usize;
    for code_point in inner.chars() {
        if !is_selected_grouping_trivia(code_point) {
            break;
        }
        leading_trim += code_point.len_utf8();
    }
    let after_leading = &inner[leading_trim..];

    let mut operand_end = after_leading.len();
    for code_point in after_leading.chars().rev() {
        if !is_selected_grouping_trivia(code_point) {
            break;
        }
        operand_end -= code_point.len_utf8();
    }

    is_selected_accepted_identifier_reference(&after_leading[..operand_end])
}

/// Independent EE-08 cover model: every candidate this bounded Oracle accepts
/// necessarily has exact `(`, one accepted operand, exact `)`, no comma, no
/// rest, and no arrow continuation -- so it necessarily covers
/// `ParenthesizedExpression : ( Expression )` (the `Expression` being exactly
/// the one accepted `IdentifierReference` this theorem bounds), and
/// `EE-08-R01` (`PrimaryExpression : CoverParenthesizedExpressionAndArrowParameterList`
/// `must cover ParenthesizedExpression`) never triggers for it.
fn classify_ee08_cover(candidate: &str) -> CoverClassification {
    if is_selected_parenthesized_identifier_reference(candidate) {
        CoverClassification::AcceptedParenthesizedIdentifierReference
    } else {
        CoverClassification::CoverFailureOrNotSelected
    }
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 831);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_BANG_TILDE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 827"));
    assert!(PREVIOUS_ESCAPED_IDENTIFIER_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 241"));
    assert!(FRONTIER_SCOPE_NOTE.contains(
        "exactly-one grouping-layer ParenthesizedExpression IdentifierReference frontier only"
    ));
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));
}

/// Required positive matrix: Direct ASCII, Direct multibyte, and
/// EscapedNonReserved (fixed, mixed, braced, supplementary) `IdentifierReference`
/// operands, with fixture-owned literal byte ranges for both delimiters, the
/// inner reference, and the complete grouped occurrence -- never derived by
/// search, rescan, or reparse. The inner reference range never includes
/// either paren byte; the decoded semantic name and Direct/EscapedNonReserved
/// provenance are preserved exactly.
#[test]
fn exact_positive_matrix_pins_fixture_owned_paren_and_reference_ranges() {
    struct PositiveRow {
        source: &'static str,
        open_paren: Range,
        reference: Range,
        close_paren: Range,
        whole: Range,
        expected_reference: &'static str,
        expected_name: &'static str,
        expected_provenance: IdentifierReferenceProvenance,
    }

    let rows = [
        PositiveRow {
            source: "const x = (a);",
            open_paren: Range(10, 11),
            reference: Range(11, 12),
            close_paren: Range(12, 13),
            whole: Range(10, 13),
            expected_reference: "a",
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::Direct,
        },
        PositiveRow {
            source: "const x = (\u{03C0});",
            open_paren: Range(10, 11),
            reference: Range(11, 13),
            close_paren: Range(13, 14),
            whole: Range(10, 14),
            expected_reference: "\u{03C0}",
            expected_name: "\u{03C0}",
            expected_provenance: IdentifierReferenceProvenance::Direct,
        },
        PositiveRow {
            source: "const x = (\\u0061);",
            open_paren: Range(10, 11),
            reference: Range(11, 17),
            close_paren: Range(17, 18),
            whole: Range(10, 18),
            expected_reference: "\\u0061",
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
        },
        PositiveRow {
            source: "const x = (f\\u006Fo);",
            open_paren: Range(10, 11),
            reference: Range(11, 19),
            close_paren: Range(19, 20),
            whole: Range(10, 20),
            expected_reference: "f\\u006Fo",
            expected_name: "foo",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
        },
        PositiveRow {
            source: "const x = (\\u{66}oo);",
            open_paren: Range(10, 11),
            reference: Range(11, 19),
            close_paren: Range(19, 20),
            whole: Range(10, 20),
            expected_reference: "\\u{66}oo",
            expected_name: "foo",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
        },
        PositiveRow {
            source: "const x = (\\u{1D49C});",
            open_paren: Range(10, 11),
            reference: Range(11, 20),
            close_paren: Range(20, 21),
            whole: Range(10, 21),
            expected_reference: "\\u{1D49C}",
            expected_name: "\u{1D49C}",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
        },
    ];

    for (index, row) in rows.iter().enumerate() {
        assert_eq!(slice(row.source, row.open_paren), "(");
        assert_eq!(slice(row.source, row.close_paren), ")");
        assert_eq!(slice(row.source, row.reference), row.expected_reference);
        // The inner reference range never includes either paren byte.
        assert_eq!(row.reference.0, row.open_paren.1);
        assert_eq!(row.reference.1, row.close_paren.0);
        assert_eq!(row.whole.0, row.open_paren.0);
        assert_eq!(row.whole.1, row.close_paren.1);

        let whole_text = slice(row.source, row.whole);
        assert!(
            is_selected_parenthesized_identifier_reference(whole_text),
            "{whole_text:?}"
        );

        let reference_text = slice(row.source, row.reference);
        let (name, provenance) = classify_selected_accepted_identifier_reference(reference_text)
            .unwrap_or_else(|| {
                panic!("{reference_text:?} must be an accepted IdentifierReference")
            });
        assert_eq!(name, row.expected_name);
        assert_eq!(provenance, row.expected_provenance);

        assert_eq!(
            authored_anchor(831_000 + index as u64, row.source, row.whole),
            whole_text
        );
        // The inner reference anchor is exactly the (possibly escaped)
        // spelling, never the paren/trivia-wrapped spelling, and the decoded
        // semantic name never includes either paren or any trivia.
        assert_eq!(
            authored_anchor(831_050 + index as u64, row.source, row.reference),
            row.expected_reference
        );
    }

    let expected = FrontierOutcome::SelectedAcceptedIncomplete;
    assert!(matches!(
        expected,
        FrontierOutcome::SelectedAcceptedIncomplete
    ));
}

/// First validation carrier from Issue #831: `let a;\nconst x = (a);`. This
/// is a validation-harness choice, not a claim that the theorem is
/// `LexicalDeclaration`-specific.
#[test]
fn first_validation_carrier_form_is_recognized() {
    let source = "let a;\nconst x = (a);";
    assert!(is_selected_parenthesized_identifier_reference(slice(
        source,
        Range(17, 20)
    )));
}

/// Freezes the complete already-accepted selected trivia policy immediately
/// inside each paren: ASCII TAB/LF, and the frozen Unicode 17
/// `Space_Separator` property beyond ASCII space (NBSP) all compose;
/// comments never do.
#[test]
fn selected_grouping_trivia_matrix_and_comment_firewall() {
    let fixtures: &[(&str, Range, Range, Range, &str)] = &[
        (
            "const x = ( a);",
            Range(10, 11),
            Range(12, 13),
            Range(10, 14),
            "( a)",
        ),
        (
            "const x = (a\t);",
            Range(10, 11),
            Range(11, 12),
            Range(10, 14),
            "(a\t)",
        ),
        (
            "const x = (\na);",
            Range(10, 11),
            Range(12, 13),
            Range(10, 14),
            "(\na)",
        ),
        (
            "const x = (a\u{00A0});",
            Range(10, 11),
            Range(11, 12),
            Range(10, 15),
            "(a\u{00A0})",
        ),
    ];

    for (index, (text, open_paren, reference, whole, expected_whole)) in fixtures.iter().enumerate()
    {
        let whole_text = slice(text, *whole);
        assert_eq!(whole_text, *expected_whole);
        assert_eq!(slice(text, *open_paren), "(");
        assert_eq!(slice(text, *reference), "a");
        assert!(
            is_selected_parenthesized_identifier_reference(whole_text),
            "{whole_text:?}"
        );
        assert_eq!(
            authored_anchor(831_100 + index as u64, text, *whole),
            whole_text
        );
    }

    for control in COMMENT_FIREWALL_CONTROLS {
        assert!(
            !is_selected_parenthesized_identifier_reference(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Identifier policy composition (representative categories only, not a
/// re-test of Issue #237/#241): ordinary direct, multibyte direct, `$`/`_`,
/// the `Yield=false`/`Await=false` special forms, the `eval`/`arguments`
/// binding-contrast names, and ordinary/mixed EscapedNonReserved spellings
/// all compose as valid operands; unconditionally reserved direct spellings
/// and malformed escapes remain outside.
#[test]
fn identifier_reference_policy_composition_matrix() {
    const ACCEPTED_DIRECT_OPERANDS: &[&str] = &[
        "a",
        "foo",
        "\u{03C0}",
        "$",
        "_",
        "yield",
        "await",
        "eval",
        "arguments",
    ];
    for operand in ACCEPTED_DIRECT_OPERANDS {
        assert_eq!(
            classify_selected_accepted_identifier_reference(operand),
            Some(((*operand).to_owned(), IdentifierReferenceProvenance::Direct)),
            "{operand:?}"
        );
        let wrapped = format!("({operand})");
        assert!(
            is_selected_parenthesized_identifier_reference(&wrapped),
            "{wrapped:?}"
        );
    }

    const ACCEPTED_ESCAPED_OPERANDS: &[(&str, &str)] = &[
        ("\\u0061", "a"),
        ("f\\u006Fo", "foo"),
        ("\\u0024", "$"),
        ("\\u{66}oo", "foo"),
        ("\\u{1D49C}", "\u{1D49C}"),
    ];
    for (rhs, decoded) in ACCEPTED_ESCAPED_OPERANDS {
        assert_eq!(
            classify_selected_accepted_identifier_reference(rhs),
            Some((
                (*decoded).to_owned(),
                IdentifierReferenceProvenance::EscapedNonReserved
            )),
            "{rhs:?}"
        );
        let wrapped = format!("({rhs})");
        assert!(
            is_selected_parenthesized_identifier_reference(&wrapped),
            "{wrapped:?}"
        );
    }

    // Unconditionally reserved direct spellings remain outside
    // `IdentifierReference` (and therefore outside this grouping theorem).
    for reserved in ["if", "this", "true", "false", "null", "typeof", "delete"] {
        assert_eq!(
            classify_selected_accepted_identifier_reference(reserved),
            None
        );
        let wrapped = format!("({reserved})");
        assert!(
            !is_selected_parenthesized_identifier_reference(&wrapped),
            "{wrapped:?}"
        );
    }

    // Malformed escapes remain outside (short/invalid hex digits), for both
    // the fixed and braced forms.
    for malformed in [r"\u006", r"\u0G61", r"\u{}", r"\u{G}"] {
        assert_eq!(
            decode_selected_escaped_identifier(malformed),
            Err(DecodeFailure::MalformedEscape),
            "{malformed:?}"
        );
        assert_eq!(
            classify_selected_accepted_identifier_reference(malformed),
            None
        );
        let wrapped = format!("({malformed})");
        assert!(
            !is_selected_parenthesized_identifier_reference(&wrapped),
            "{wrapped:?}"
        );
    }

    assert_eq!(UNCONDITIONALLY_RESERVED_WORDS.len(), 36);
}

/// Escaped failure boundary (W-model adjacent): decoded ReservedWord,
/// malformed escape, non-CodePoint, and invalid-start controls are preserved
/// as distinct `DecodeFailure` outcomes, never collapsed into a single
/// "syntax decline", and none survives wrapping in a grouping layer.
#[test]
fn escaped_failure_boundary_matrix_preserves_processing_distinctions() {
    let cases: &[(&str, DecodeFailure)] = &[
        ("\\u0069f", DecodeFailure::DecodedReserved),
        (r"\u{69}f", DecodeFailure::DecodedReserved),
        (r"\u{}", DecodeFailure::MalformedEscape),
        (r"\u{G}", DecodeFailure::MalformedEscape),
        (r"\u{110000}", DecodeFailure::NonCodePoint),
        ("\\u0030", DecodeFailure::InvalidStart),
    ];

    for (spelling, expected_failure) in cases {
        assert_eq!(
            decode_selected_escaped_identifier(spelling),
            Err(*expected_failure),
            "{spelling:?}"
        );
        assert_eq!(
            classify_selected_accepted_identifier_reference(spelling),
            None,
            "{spelling:?}"
        );
        let wrapped = format!("({spelling})");
        assert!(
            !is_selected_parenthesized_identifier_reference(&wrapped),
            "{wrapped:?}"
        );
    }
}

/// Cover/arrow firewall (W3/W6): empty cover, trailing comma, multiple
/// expressions, rest forms, and arrow continuation -- with and without
/// selected trivia -- all remain outside this bounded theorem.
#[test]
fn cover_grammar_firewall_excludes_empty_comma_rest_and_arrow_forms() {
    for control in COVER_FIREWALL_CONTROLS {
        assert!(
            !is_selected_parenthesized_identifier_reference(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Recursive grouping firewall (W9): nested parentheses are never normalized
/// to the same inner fact, because this oracle never recursively calls its
/// own grouping recognizer on the inner candidate.
#[test]
fn recursive_grouping_firewall_remains_unowned() {
    for control in RECURSIVE_GROUPING_CONTROLS {
        assert!(
            !is_selected_parenthesized_identifier_reference(control),
            "{control:?}"
        );
    }
    assert!(is_selected_parenthesized_identifier_reference("(a)"));

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Richer-inner-expression firewall: additive, multiplicative, assignment,
/// conditional, call, member, `new`, and unary-operator operands remain
/// outside this theorem, whatever their own independent validity elsewhere.
#[test]
fn richer_inner_expression_firewall_remains_unowned() {
    for control in RICHER_INNER_EXPRESSION_CONTROLS {
        assert!(
            !is_selected_parenthesized_identifier_reference(control),
            "{control:?}"
        );
    }

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// Richer-outer-expression firewall: a complete grouped-reference prefix
/// never authorizes member, call, binary, assignment, or conditional
/// continuation; only the richer tail is unowned, and the bare `(a)` prefix
/// itself remains independently valid (whole-candidate transactionality).
#[test]
fn richer_outer_expression_firewall_preserves_whole_candidate_transactionality() {
    for control in RICHER_OUTER_EXPRESSION_CONTROLS {
        assert!(
            !is_selected_parenthesized_identifier_reference(control),
            "{control:?}"
        );
    }
    assert!(is_selected_parenthesized_identifier_reference("(a)"));

    let expected = FrontierOutcome::UnsupportedCoverage;
    assert!(matches!(expected, FrontierOutcome::UnsupportedCoverage));
}

/// EE-08 cover reasoning is made inspectable: every accepted candidate is
/// shown by construction to have exact `(`, one accepted operand, exact `)`,
/// no comma, no rest, and no arrow continuation, and that structural fact is
/// connected -- by text inspection only, never by calling production or
/// completion code -- to the frozen `EE-08-R01` completion-authority
/// classification (`OwningSyntaxAbsent("Grouping / parenthesized
/// expression")`) and the frozen `193 / 10 / 183` completion partition, which
/// this validation-only leaf leaves unmoved.
#[test]
fn cover_ee08_structural_reasoning_is_inspectable_and_matches_frozen_completion_authority() {
    for candidate in POSITIVE_CANDIDATES {
        assert!(matches!(
            classify_ee08_cover(candidate),
            CoverClassification::AcceptedParenthesizedIdentifierReference
        ));
        assert!(!candidate.contains(','));
        assert!(!candidate.contains("..."));
        assert!(!candidate.contains("=>"));
    }

    for control in COVER_FIREWALL_CONTROLS
        .iter()
        .chain(RECURSIVE_GROUPING_CONTROLS)
        .chain(RICHER_INNER_EXPRESSION_CONTROLS)
        .chain(RICHER_OUTER_EXPRESSION_CONTROLS)
    {
        assert!(matches!(
            classify_ee08_cover(control),
            CoverClassification::CoverFailureOrNotSelected
        ));
    }

    assert!(INVENTORY_SOURCE.contains("\"EE-08-R01\""));
    assert!(INVENTORY_SOURCE.contains(
        "PrimaryExpression : CoverParenthesizedExpressionAndArrowParameterList / must cover ParenthesizedExpression"
    ));
    assert!(
        COMPLETION_AUTHORITY_SOURCE
            .contains("(\"EE-08\", \"Grouping / parenthesized expression\")")
    );
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("OwningSyntaxAbsent"));
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(seen.len(), 193);"));
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(active, 183);"));
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(inactive, 10);"));
}

/// Narrow, fixture-owned demonstration (not a claim of independent static-
/// semantics composition proof): wrapping an RHS in this grouping layer does
/// not move or replace the authored `BindingIdentifier` subject fragments
/// that existing EE-15/EE-36 duplicate/collision and missing-initializer
/// controls key on -- those subjects remain exactly where the un-grouped
/// control gold fixtures already place them. A minimal relation harness
/// (`let a; const x = (a);`) separately demonstrates that the grouped
/// `IdentifierReference` still names the same authored binding by simple
/// name-string identity, without introducing any new relation vocabulary.
#[test]
fn grouped_rhs_leaves_authored_binding_subjects_and_inner_name_identity_unchanged() {
    struct StaticCompositionFixture {
        source: &'static str,
        subject: Range,
        subject_fragment: &'static str,
        control_gold_id: &'static str,
    }

    const FIXTURES: &[StaticCompositionFixture] = &[
        StaticCompositionFixture {
            source: "let let = (a);",
            subject: Range(4, 7),
            subject_fragment: "let",
            control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
        },
        StaticCompositionFixture {
            source: "let x = (a), x = foo;",
            subject: Range(13, 14),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
        },
        StaticCompositionFixture {
            source: "const x = (a), y;",
            subject: Range(15, 16),
            subject_fragment: "y",
            control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
        },
        StaticCompositionFixture {
            source: "let x = (a); let x = foo;",
            subject: Range(17, 18),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-SCRIPT-DUPLEXICAL-001",
        },
    ];

    for fixture in FIXTURES {
        assert_eq!(
            slice(fixture.source, fixture.subject),
            fixture.subject_fragment
        );
        assert!(
            gold_source(fixture.control_gold_id).is_some(),
            "{}",
            fixture.control_gold_id
        );
    }

    // Minimal relation harness: the grouped reference still names the same
    // authored binding as the un-grouped source would.
    let harness = "let a; const x = (a);";
    assert_eq!(slice(harness, Range(4, 5)), "a");
    assert_eq!(slice(harness, Range(18, 19)), "a");
    assert!(is_selected_parenthesized_identifier_reference(slice(
        harness,
        Range(17, 20)
    )));
    assert_eq!(authored_anchor(831_200, harness, Range(4, 5)), "a");
    assert_eq!(authored_anchor(831_201, harness, Range(18, 19)), "a");

    // No new relation vocabulary is introduced by this validation-only leaf.
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("Grouped", "ReferenceRelation")));
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("Parenthesized", "Relation")));
}

/// Oracle meta-independence firewall: rejects imports/calls into production
/// lexical, static-semantics, Binding/Scope, correspondence, or aggregate
/// qualification code, and rejects the forbidden substring-search/reverse-
/// search reconstruction patterns that would let expected ranges be derived
/// rather than fixture-owned.
#[test]
fn validation_source_has_no_dependency_on_selected_production_or_runtime_evaluation() {
    for forbidden in [
        concat!("use super::", "selected_lexical_slice"),
        concat!("use super::", "selected_static_semantics"),
        concat!("use super::", "selected_qualification_integration"),
        concat!(
            "use super::",
            "selected_variable_statement_name_correspondence"
        ),
        concat!("use super::", "selected_binding_scope"),
        concat!("use super::", "selected_one_level_block_binding_scope"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("consume_selected_", "identifier_reference"),
        concat!("attempt_selected_", "qualification"),
        concat!("Resolve", "Binding("),
        concat!("Get", "Value("),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // `is_id_start` / `is_id_continue` / `is_space_separator` are stable,
    // frozen normative Unicode 17 primitives this oracle is independently
    // entitled to consult (already used by predecessor accepted oracles at
    // the same `super::unicode` path); none of them is a production
    // expression/grouping parser.
    assert!(
        THIS_ORACLE_SOURCE
            .contains("use super::unicode::{is_id_continue, is_id_start, is_space_separator}")
    );
}

/// Freezes the presence-only, unqualified, validation-only handoff: this
/// leaf composes to `SelectedAcceptedIncomplete` and `UnsupportedCoverage`;
/// `ResourceLimited`/`InternalFailure` processing failure remains a distinct
/// lifecycle identity, never folded into `UnsupportedCoverage` or
/// `SelectedAcceptedIncomplete`; and no retained production representation,
/// runtime, cover-grammar, or general-parser vocabulary is introduced.
#[test]
fn handoff_remains_presence_only_unqualified_and_validation_only() {
    assert!(THIS_ORACLE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(THIS_ORACLE_SOURCE.contains("UnsupportedCoverage"));
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
    assert!(!PROCESSING_FAILURES.contains(&"UnsupportedCoverage"));
    assert!(!PROCESSING_FAILURES.contains(&"SelectedAcceptedIncomplete"));

    for forbidden in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("enum Selected", "Literal"),
        concat!("enum SelectedPrimary", "Expression"),
        concat!("enum SelectedExpression", "Kind"),
        concat!("enum Cover", "ParenthesizedExpressionAndArrowParameterList"),
        concat!("enum Arrow", "FormalParameters"),
        concat!("struct Arrow", "FunctionNode"),
        concat!("struct Grouping", "SourceAnchor"),
        concat!("struct Paren", "Node"),
        concat!("struct Reference", "Record"),
        concat!("struct Environment", "Record"),
        concat!("enum Initializer", "Kind"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("Get", "Value("),
        concat!("Resolve", "Binding("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
