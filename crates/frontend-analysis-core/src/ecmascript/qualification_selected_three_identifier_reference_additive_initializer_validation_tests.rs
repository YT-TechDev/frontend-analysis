//! Candidate-independent ordered three-`IdentifierReference` additive
//! initializer fact-preservation validation for Issue #795 (durable
//! research: Issue #688 comment `5767005572`; accepted predecessor
//! authorities: Issue #752 / PR #753 ordered two-`IdentifierReference`
//! additive initializer Oracle, Issue #789 / PR #790 one-reference/
//! one-plain-decimal heterogeneous additive Oracle establishing that syntax
//! operand cardinality and retained semantic-evidence cardinality are
//! distinct theorems).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedThreeIdentifierReferenceAdditiveInitializer ::=
//!     SelectedAcceptedIdentifierReference
//!     SelectedAdditiveTrivia
//!     SelectedAdditiveOperator
//!     SelectedAdditiveTrivia
//!     SelectedAcceptedIdentifierReference
//!     SelectedAdditiveTrivia
//!     SelectedAdditiveOperator
//!     SelectedAdditiveTrivia
//!     SelectedAcceptedIdentifierReference
//!
//! SelectedAdditiveOperator ::= "+" | "-"
//!
//! SelectedAcceptedIdentifierReference ::=
//!     SelectedDirectIdentifierReference
//!   | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice. It
//! does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code.
//!
//! ## Load-bearing novelty: two independently owned interior boundaries
//!
//! Issue #752's exactly-two Oracle only had to discover where the first
//! (left) operand ended; the entire remaining post-operator suffix could
//! then be validated as the second (right) operand in one step. Exactly-three
//! introduces a strictly stronger source-recognition theorem:
//!
//! ```text
//! operand 1 -> operator 1 -> operand 2 -> operator 2 -> operand 3
//! ```
//!
//! The Oracle must independently own **two** interior operand boundaries --
//! the end of operand 1 and the end of operand 2 -- within one bounded
//! left-to-right recognition lifecycle, never by `.find`/`.rfind`, later
//! substring search, source rescanning after recognition, reparse,
//! retokenization, operator-relative endpoint reconstruction, or
//! decoded-length endpoint inference. `operand_interior_run_end` performs
//! exactly that bounded forward pass and is independently invoked once for
//! each interior boundary; the third (last) operand is never boundary-scanned
//! at all, because it is validated as the complete remaining slice, exactly
//! as Issue #752's right operand was -- the same "no trailing content
//! survives" guarantee, simply applied to the slice following the second
//! operator instead of the first. This is deliberately not generalized into
//! a loop over an arbitrary-N operand list: the theorem is exactly three
//! operands and two operators, nothing more.
//!
//! Issue #789/PR #790 already proved that syntax-operand cardinality and
//! retained-fact cardinality are distinct theorems (two syntax operands can
//! retain fewer than two facts). That result is not reused here: this Oracle
//! proves the opposite direction -- that a bounded source family can retain
//! *exactly three* independently source-backed facts, in exact authored
//! order, which neither #752 nor #789 established.
//!
//! `SelectedAdditiveTrivia` independently restates the complete
//! already-accepted selected-slice trivia contract established by Issue
//! #742/#743 and restated again by Issue #746/#747 and Issue #752/#753 (the
//! same code-point set the production selected-trivia recognizer accepts):
//! `TAB`, `VT`, `FF`, `BOM`, `LF`, `CR`, `LINE SEPARATOR`, `PARAGRAPH
//! SEPARATOR`, and the frozen Unicode 17 `Space_Separator` property. Comments
//! remain outside this contract and are never accepted as trivia.
//!
//! `SelectedDirectIdentifierReference` independently restates the
//! already-accepted Issue #237 direct escape-free `IdentifierName`
//! code-point shape (`is_id_start`/`is_id_continue` plus `$`/`_`) minus the
//! unconditionally reserved words. `SelectedEscapedNonReservedIdentifierReference`
//! independently restates the already-accepted Issue #241
//! `UnicodeEscapeSequence` decode/position/decoded-reserved-word theorem.
//! Neither restatement imports another Oracle's code: this leaf owns its own
//! copy of the already-accepted primitives it consumes, matching the
//! convention already set by Issue #752/#753 relative to Issue #237/#241.
//!
//! ## What is retained, and what is not
//!
//! The Oracle's only retained representation is `ThreeIdentifierReferenceAdditiveFacts`:
//! exactly three ordered operand facts (`first`, `second`, `third`), each
//! preserving its own exact authored `SourceAnchor`/fragment, decoded
//! semantic name, and Direct/EscapedNonReserved spelling state. This is a
//! minimal private test-only shape and explicitly does **not** freeze future
//! production representation as `Three { first, second, third }`, `[Fact; 3]`,
//! a tuple, `Vec<Fact>`, `SmallVec`, a recursive occurrence representation, a
//! generic non-empty collection, a generic `Expression` node, or an AST/CST.
//! ECMA-262's `AdditiveExpression` grammar is recursively (left-recursively)
//! defined, so `a+b+c` has a normative recursive syntax tree; this Oracle
//! deliberately flattens that into three ordered occurrence facts because
//! current demonstrated consumers (`identifier_reference_initializer_facts()`
//! iteration in `selected_binding_scope.rs`,
//! `selected_one_level_block_binding_scope.rs`, and
//! `selected_variable_statement_name_correspondence.rs`, per Issue #688
//! comment `5767005572`) are fact-iteration driven, not variant-driven. That
//! evidence flattening is not a claim that the ECMAScript grammar itself is
//! flat, and it does not select `Three`/`Vec`/arbitrary-N production storage.
//! No operator kind, operator `SourceAnchor`, or whole-expression
//! `SourceAnchor` is retained; the two authored binary operators are
//! recognized only to prove the exact bounded source family.
//!
//! Pinned Test262 addition/subtraction order-of-evaluation tests are
//! corroborative runtime evidence only; they are not the expected-answer
//! authority for this source-provenance/cardinality Oracle.
//!
//! This is a validation-only leaf: production does not support this
//! exactly-three IdentifierReference additive family at the #795 baseline,
//! so every positive fixture below remains `UnsupportedCoverage` under
//! current production and exists only as independent Oracle evidence for
//! a future, separately authorized production decision. No completion
//! successor file accompanies this leaf:
//! this Issue adds zero production capability, so the frozen `193 / 10 / 183
//! / {}` completion partition cannot move, matching the precedent set by
//! #742/#743, #746/#747, and #752/#753 (which likewise added zero production
//! capability and shipped without one).

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 795;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_three_identifier_reference_additive_initializer_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "ordered exactly-three IdentifierReference additive initializer frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "four-or-more operand chains, unary operands, non-IdentifierReference ",
    "operands, parenthesized/member/call operands, richer expression tails, ",
    "comments, or top-level/Block var placement"
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

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

// --- Direct IdentifierReference policy: independently restated from Issue
// #237/#752, never imported from either Oracle. ---

fn is_direct_identifier_start(code_point: char) -> bool {
    matches!(code_point, '$' | '_') || is_id_start(code_point as u32)
}

fn is_direct_identifier_part(code_point: char) -> bool {
    code_point == '$' || is_id_continue(code_point as u32)
}

fn is_escape_free_identifier_name(spelling: &str) -> bool {
    let mut code_points = spelling.chars();
    let Some(first) = code_points.next() else {
        return false;
    };
    is_direct_identifier_start(first) && code_points.all(is_direct_identifier_part)
}

/// `SelectedDirectIdentifierReference`: the already-accepted direct
/// escape-free `IdentifierName` shape, minus the unconditionally reserved
/// words -- never a `BindingIdentifier` policy.
fn is_selected_direct_identifier_reference(candidate: &str) -> bool {
    is_escape_free_identifier_name(candidate)
        && !UNCONDITIONALLY_RESERVED_WORDS.contains(&candidate)
}

// --- Escaped IdentifierReference decode: independently restated from Issue
// #241, never imported from that Oracle. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodeFailure {
    MalformedEscape,
    NonCodePoint,
    InvalidStart,
    InvalidPart,
    DecodedReserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecodedIdentifier {
    string_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EscapeFormation {
    end: usize,
    code_point: u32,
}

fn ascii_hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a') + 10),
        b'A'..=b'F' => Some(u32::from(byte - b'A') + 10),
        _ => None,
    }
}

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

/// Forms exactly one `UnicodeEscapeSequence` element at byte offset `start`
/// -- syntactic well-formedness only, never position validity, which is
/// judged afterward once the whole operand slice is known.
fn formed_escape_at(spelling: &str, start: usize) -> Result<EscapeFormation, DecodeFailure> {
    let bytes = spelling.as_bytes();
    if bytes.get(start) != Some(&b'\\') || bytes.get(start + 1) != Some(&b'u') {
        return Err(DecodeFailure::MalformedEscape);
    }

    let payload = start + 2;
    if bytes.get(payload) == Some(&b'{') {
        let digits_start = payload + 1;
        let mut end = digits_start;
        while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit()) {
            end += 1;
        }
        if end == digits_start || bytes.get(end) != Some(&b'}') {
            return Err(DecodeFailure::MalformedEscape);
        }
        let code_point = parse_braced_code_point(&bytes[digits_start..end])?;
        return Ok(EscapeFormation {
            end: end + 1,
            code_point,
        });
    }

    let end = payload
        .checked_add(4)
        .ok_or(DecodeFailure::MalformedEscape)?;
    let digits = bytes
        .get(payload..end)
        .ok_or(DecodeFailure::MalformedEscape)?;
    if !digits.iter().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DecodeFailure::MalformedEscape);
    }

    let mut code_point = 0_u32;
    for byte in digits {
        code_point =
            code_point * 16 + ascii_hex_value(*byte).ok_or(DecodeFailure::MalformedEscape)?;
    }

    Ok(EscapeFormation { end, code_point })
}

fn is_selected_identifier_start_for_oracle(code_point: u32) -> bool {
    code_point == u32::from(b'$') || code_point == u32::from(b'_') || is_id_start(code_point)
}

fn is_selected_identifier_part_for_oracle(code_point: u32) -> bool {
    is_selected_identifier_start_for_oracle(code_point)
        || is_id_continue(code_point)
        || matches!(code_point, 0x200C | 0x200D)
}

fn is_unconditionally_reserved_word_for_oracle(spelling: &str) -> bool {
    UNCONDITIONALLY_RESERVED_WORDS.contains(&spelling)
}

/// `SelectedEscapedNonReservedIdentifierReference`: decodes a whole operand
/// slice that must contain at least one authored `UnicodeEscapeSequence`,
/// judging start/part position validity per decoded element and rejecting a
/// decoded-reserved whole name -- independently restated from Issue #241.
fn decode_selected_escaped_identifier(spelling: &str) -> Result<DecodedIdentifier, DecodeFailure> {
    let mut offset = 0_usize;
    let mut element_index = 0_usize;
    let mut saw_escape = false;
    let mut code_points: Vec<u32> = Vec::new();

    while offset < spelling.len() {
        let (code_point, end, escaped) = if spelling.as_bytes().get(offset) == Some(&b'\\') {
            let formation = formed_escape_at(spelling, offset)?;
            (formation.code_point, formation.end, true)
        } else {
            let scalar = spelling[offset..]
                .chars()
                .next()
                .ok_or(DecodeFailure::MalformedEscape)?;
            (scalar as u32, offset + scalar.len_utf8(), false)
        };

        let valid_position = if element_index == 0 {
            is_selected_identifier_start_for_oracle(code_point)
        } else {
            is_selected_identifier_part_for_oracle(code_point)
        };
        if !valid_position {
            return Err(if element_index == 0 {
                DecodeFailure::InvalidStart
            } else {
                DecodeFailure::InvalidPart
            });
        }

        code_points.push(code_point);
        saw_escape |= escaped;
        offset = end;
        element_index += 1;
    }

    if !saw_escape || code_points.is_empty() {
        return Err(DecodeFailure::MalformedEscape);
    }

    let mut string_value = String::new();
    for code_point in &code_points {
        let scalar = char::from_u32(*code_point)
            .expect("position-valid identifier code point must be a Unicode scalar value");
        string_value.push(scalar);
    }

    if is_unconditionally_reserved_word_for_oracle(&string_value) {
        return Err(DecodeFailure::DecodedReserved);
    }

    Ok(DecodedIdentifier { string_value })
}

// --- Selected trivia: independently restated from Issue #742/#743, exactly
// as Issue #746/#747 and Issue #752/#753 already restate it. ---

fn is_selected_additive_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

fn trivia_run_end(candidate: &str) -> usize {
    let mut offset = 0_usize;
    for code_point in candidate.chars() {
        if !is_selected_additive_trivia(code_point) {
            break;
        }
        offset += code_point.len_utf8();
    }
    offset
}

// --- The Issue #795 theorem itself. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperandSpellingKind {
    Direct,
    EscapedNonReserved,
}

/// One retained operand fact. A private test-only shape, not a claim about
/// future production storage.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedOperand<'a> {
    authored: &'a str,
    semantic_name: String,
    spelling: OperandSpellingKind,
}

/// The Oracle's only retained representation: exactly three ordered operand
/// facts. This struct shape is a minimal private test representation, never
/// a claim that future production must use `Three`, `[Fact; 3]`, a tuple,
/// `Vec<Fact>`, `SmallVec`, a recursive occurrence representation, a generic
/// non-empty collection, a generic `Expression` node, or an AST/CST.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ThreeIdentifierReferenceAdditiveFacts<'a> {
    first: RecognizedOperand<'a>,
    second: RecognizedOperand<'a>,
    third: RecognizedOperand<'a>,
}

/// Validates one whole operand slice against `SelectedAcceptedIdentifierReference`:
/// exactly one of the direct or escaped-non-Reserved routes, over the
/// complete slice, never a prefix or a later-reconstructed sub-range.
fn recognize_accepted_identifier_reference(candidate: &str) -> Option<RecognizedOperand<'_>> {
    if candidate.contains('\\') {
        let decoded = decode_selected_escaped_identifier(candidate).ok()?;
        Some(RecognizedOperand {
            authored: candidate,
            semantic_name: decoded.string_value,
            spelling: OperandSpellingKind::EscapedNonReserved,
        })
    } else if is_selected_direct_identifier_reference(candidate) {
        Some(RecognizedOperand {
            authored: candidate,
            semantic_name: candidate.to_owned(),
            spelling: OperandSpellingKind::Direct,
        })
    } else {
        None
    }
}

/// Finds where an interior operand (one followed by more of the candidate)
/// ends: one left-to-right forward pass consuming either one direct code
/// point or one syntactically well-formed `UnicodeEscapeSequence` element,
/// stopping at the first selected-trivia code point or at `+`/`-`. A
/// malformed escape aborts the whole candidate immediately (`None`) rather
/// than silently truncating the run at that point -- the malformed escape
/// never leaks an earlier valid-looking prefix. Position validity of the
/// collected run is judged afterward by `recognize_accepted_identifier_reference`,
/// never here. This is the Issue #795 novelty over #752: it is invoked
/// independently for both interior boundaries (end of operand 1, end of
/// operand 2) within one recognition lifecycle, never for operand 3, which
/// is instead validated as the complete remaining slice.
fn operand_interior_run_end(candidate: &str) -> Option<usize> {
    let mut offset = 0_usize;
    while offset < candidate.len() {
        if candidate.as_bytes()[offset] == b'\\' {
            let formation = formed_escape_at(candidate, offset).ok()?;
            offset = formation.end;
            continue;
        }
        let code_point = candidate[offset..].chars().next()?;
        if is_selected_additive_trivia(code_point) || code_point == '+' || code_point == '-' {
            break;
        }
        offset += code_point.len_utf8();
    }
    (offset > 0).then_some(offset)
}

/// Central Issue #795 theorem. Whole-candidate, all-or-nothing: no fact is
/// ever returned unless the complete nine-part bounded family -- operand 1,
/// trivia, exactly one authored `+`/`-`, trivia, operand 2, trivia, exactly
/// one authored `+`/`-`, trivia, operand 3 -- is recognized over the entire
/// candidate, with nothing left over. Each of the three local `?`-chained
/// steps only ever produces a local binding; the three facts are combined
/// into the returned struct in a single final expression, so a failure while
/// preparing operand 2 or operand 3 can never publish operand 1 (or operand
/// 1 and 2) as a partial result. Both authored binary operators are
/// recognized only to prove the exact bounded source family; neither is ever
/// retained in the returned facts.
fn recognize_selected_three_identifier_reference_additive_initializer(
    candidate: &str,
) -> Option<ThreeIdentifierReferenceAdditiveFacts<'_>> {
    let first_end = operand_interior_run_end(candidate)?;
    let (first_slice, after_first) = candidate.split_at(first_end);
    let first = recognize_accepted_identifier_reference(first_slice)?;

    let after_first_trivia = &after_first[trivia_run_end(after_first)..];
    let mut after_first_trivia_chars = after_first_trivia.chars();
    match after_first_trivia_chars.next() {
        Some('+') | Some('-') => {}
        _ => return None,
    }
    let after_first_operator = &after_first_trivia[1..];

    let after_first_operator_trivia = &after_first_operator[trivia_run_end(after_first_operator)..];
    let second_end = operand_interior_run_end(after_first_operator_trivia)?;
    let (second_slice, after_second) = after_first_operator_trivia.split_at(second_end);
    let second = recognize_accepted_identifier_reference(second_slice)?;

    let after_second_trivia = &after_second[trivia_run_end(after_second)..];
    let mut after_second_trivia_chars = after_second_trivia.chars();
    match after_second_trivia_chars.next() {
        Some('+') | Some('-') => {}
        _ => return None,
    }
    let after_second_operator = &after_second_trivia[1..];

    let third_slice = &after_second_operator[trivia_run_end(after_second_operator)..];
    let third = recognize_accepted_identifier_reference(third_slice)?;

    Some(ThreeIdentifierReferenceAdditiveFacts {
        first,
        second,
        third,
    })
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 795);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(
        PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 752")
    );
    assert!(
        PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE
            .contains("ordered two-IdentifierReference additive initializer frontier only")
    );
    assert!(
        FRONTIER_SCOPE_NOTE.contains(
            "ordered exactly-three IdentifierReference additive initializer frontier only"
        )
    );
    assert!(FRONTIER_SCOPE_NOTE.contains("four-or-more operand chains"));

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
        concat!("use super::", "selected_binding_identifier"),
        concat!("use super::", "qualification;"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("recognize_selected_two_identifier_", "reference_additive"),
        concat!("consume_selected_", "identifier_reference"),
        concat!("analyze_selected_binding_", "scope("),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_selected_block_var_", "statement"),
        concat!("parse_unary_", "expression"),
        concat!("parse_additive_", "expression"),
        // Candidate independence for trivia recognition specifically: only
        // the stable normative Unicode `Space_Separator` property primitive
        // is reused, never the production selected-trivia recognizer.
        concat!("is_selected_", "trivia("),
        concat!("skip_selected_", "trivia("),
        concat!("use super::", "selected_lexical_slice::is_selected_trivia"),
        // No cross-Oracle import: this leaf restates rather than imports.
        concat!(
            "use super::qualification_selected_two_identifier_",
            "reference_additive_initializer_validation_tests"
        ),
        // Forbidden reconstruction of expected ranges/endpoints.
        concat!(".", "find("),
        concat!(".", "rfind("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // `is_id_start` / `is_id_continue` / `is_space_separator` are stable,
    // frozen normative Unicode 17 primitives this oracle is independently
    // entitled to consult, already used the same way by every predecessor
    // accepted oracle at the same `super::unicode` path.
    assert!(
        THIS_ORACLE_SOURCE
            .contains("use super::unicode::{is_id_continue, is_id_start, is_space_separator}")
    );

    // Two-interior-boundary novelty, operationalized: `operand_interior_run_end`
    // is defined exactly once and invoked exactly twice -- for the end of
    // operand 1 and the end of operand 2 -- and never for operand 3, which
    // is instead validated as the entire remaining slice. The fourth match
    // below is this assertion's own search-string literal.
    assert_eq!(
        THIS_ORACLE_SOURCE
            .matches("operand_interior_run_end(")
            .count(),
        4
    );
}

/// Required positive operator-pair matrix (acceptance criteria 1, 2, 3, 4, 6,
/// 8): all four `+`/`-` combinations, over direct ASCII and direct multibyte
/// operands, with fixture-owned literal byte ranges for every operand and
/// operator -- never derived by search, rescan, or reparse. Exactly three
/// facts are retained, in authored left-to-right order, each with its own
/// exact authored anchor (W1/W2/W3/W6/W7).
#[test]
fn positive_operator_pair_matrix_pins_three_ordered_facts_with_exact_provenance() {
    struct OperatorPairFixture {
        text: &'static str,
        first: Range,
        op1: Range,
        second: Range,
        op2: Range,
        third: Range,
        expected_first: &'static str,
        expected_op1: &'static str,
        expected_second: &'static str,
        expected_op2: &'static str,
        expected_third: &'static str,
    }

    let fixtures = [
        OperatorPairFixture {
            text: "const x = a + b + c;",
            first: Range(10, 11),
            op1: Range(12, 13),
            second: Range(14, 15),
            op2: Range(16, 17),
            third: Range(18, 19),
            expected_first: "a",
            expected_op1: "+",
            expected_second: "b",
            expected_op2: "+",
            expected_third: "c",
        },
        OperatorPairFixture {
            text: "const x = a + b - c;",
            first: Range(10, 11),
            op1: Range(12, 13),
            second: Range(14, 15),
            op2: Range(16, 17),
            third: Range(18, 19),
            expected_first: "a",
            expected_op1: "+",
            expected_second: "b",
            expected_op2: "-",
            expected_third: "c",
        },
        OperatorPairFixture {
            text: "const x = a - b + c;",
            first: Range(10, 11),
            op1: Range(12, 13),
            second: Range(14, 15),
            op2: Range(16, 17),
            third: Range(18, 19),
            expected_first: "a",
            expected_op1: "-",
            expected_second: "b",
            expected_op2: "+",
            expected_third: "c",
        },
        OperatorPairFixture {
            text: "const x = a - b - c;",
            first: Range(10, 11),
            op1: Range(12, 13),
            second: Range(14, 15),
            op2: Range(16, 17),
            third: Range(18, 19),
            expected_first: "a",
            expected_op1: "-",
            expected_second: "b",
            expected_op2: "-",
            expected_third: "c",
        },
        OperatorPairFixture {
            text: "const x = foo + bar + baz;",
            first: Range(10, 13),
            op1: Range(14, 15),
            second: Range(16, 19),
            op2: Range(20, 21),
            third: Range(22, 25),
            expected_first: "foo",
            expected_op1: "+",
            expected_second: "bar",
            expected_op2: "+",
            expected_third: "baz",
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        assert_eq!(slice(fixture.text, fixture.first), fixture.expected_first);
        assert_eq!(slice(fixture.text, fixture.op1), fixture.expected_op1);
        assert_eq!(slice(fixture.text, fixture.second), fixture.expected_second);
        assert_eq!(slice(fixture.text, fixture.op2), fixture.expected_op2);
        assert_eq!(slice(fixture.text, fixture.third), fixture.expected_third);
        // No fact ever includes an operator or intervening trivia.
        assert!(fixture.first.1 <= fixture.op1.0);
        assert!(fixture.op1.1 <= fixture.second.0);
        assert!(fixture.second.1 <= fixture.op2.0);
        assert!(fixture.op2.1 <= fixture.third.0);

        let whole = Range(fixture.first.0, fixture.third.1);
        let whole_text = slice(fixture.text, whole);
        let facts = recognize_selected_three_identifier_reference_additive_initializer(whole_text)
            .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(facts.first.authored, fixture.expected_first);
        assert_eq!(facts.first.semantic_name, fixture.expected_first);
        assert_eq!(facts.first.spelling, OperandSpellingKind::Direct);
        assert_eq!(facts.second.authored, fixture.expected_second);
        assert_eq!(facts.second.semantic_name, fixture.expected_second);
        assert_eq!(facts.second.spelling, OperandSpellingKind::Direct);
        assert_eq!(facts.third.authored, fixture.expected_third);
        assert_eq!(facts.third.semantic_name, fixture.expected_third);
        assert_eq!(facts.third.spelling, OperandSpellingKind::Direct);

        assert_eq!(
            authored_anchor(795_000 + index as u64, fixture.text, fixture.first),
            fixture.expected_first
        );
        assert_eq!(
            authored_anchor(795_050 + index as u64, fixture.text, fixture.second),
            fixture.expected_second
        );
        assert_eq!(
            authored_anchor(795_100 + index as u64, fixture.text, fixture.third),
            fixture.expected_third
        );
    }
}

/// Direct/escaped provenance matrix across all three operand positions
/// (acceptance criteria 5, 7; W5): escaped in position 1, position 2,
/// position 3, and all three positions escaped, each with its own exact
/// authored spelling distinct from its decoded semantic name.
#[test]
fn direct_escaped_position_matrix_pins_authored_vs_decoded_identity_per_operand() {
    struct PositionFixture {
        text: &'static str,
        first: Range,
        second: Range,
        third: Range,
        expected_first_authored: &'static str,
        expected_first_spelling: OperandSpellingKind,
        expected_second_authored: &'static str,
        expected_second_spelling: OperandSpellingKind,
        expected_third_authored: &'static str,
        expected_third_spelling: OperandSpellingKind,
    }

    let fixtures = [
        // Escaped in position 1 only.
        PositionFixture {
            text: r"const x = \u{61} + b + c;",
            first: Range(10, 16),
            second: Range(19, 20),
            third: Range(23, 24),
            expected_first_authored: r"\u{61}",
            expected_first_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_second_authored: "b",
            expected_second_spelling: OperandSpellingKind::Direct,
            expected_third_authored: "c",
            expected_third_spelling: OperandSpellingKind::Direct,
        },
        // Escaped in position 2 only -- the interior operand whose *both*
        // boundaries are independently owned by this Oracle's novelty.
        PositionFixture {
            text: r"const x = a + \u{62} + c;",
            first: Range(10, 11),
            second: Range(14, 20),
            third: Range(23, 24),
            expected_first_authored: "a",
            expected_first_spelling: OperandSpellingKind::Direct,
            expected_second_authored: r"\u{62}",
            expected_second_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_third_authored: "c",
            expected_third_spelling: OperandSpellingKind::Direct,
        },
        // Escaped in position 3 only (last operand, full-remainder route).
        PositionFixture {
            text: r"const x = a + b + \u{63};",
            first: Range(10, 11),
            second: Range(14, 15),
            third: Range(18, 24),
            expected_first_authored: "a",
            expected_first_spelling: OperandSpellingKind::Direct,
            expected_second_authored: "b",
            expected_second_spelling: OperandSpellingKind::Direct,
            expected_third_authored: r"\u{63}",
            expected_third_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        // All three positions escaped.
        PositionFixture {
            text: r"const x = \u{61} + \u{62} + \u{63};",
            first: Range(10, 16),
            second: Range(19, 25),
            third: Range(28, 34),
            expected_first_authored: r"\u{61}",
            expected_first_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_second_authored: r"\u{62}",
            expected_second_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_third_authored: r"\u{63}",
            expected_third_spelling: OperandSpellingKind::EscapedNonReserved,
        },
        // Classic four-hex-digit `\uXXXX` form in position 1, restated so the
        // fixed-width `formed_escape_at` branch (and its composition with
        // `operand_interior_run_end`) is exercised by direct authored
        // evidence, matching the remediation Issue #752/#753 already applied.
        PositionFixture {
            text: concat!("const x = ", "\\", "u0061", " + b + c;"),
            first: Range(10, 16),
            second: Range(19, 20),
            third: Range(23, 24),
            expected_first_authored: concat!("\\", "u0061"),
            expected_first_spelling: OperandSpellingKind::EscapedNonReserved,
            expected_second_authored: "b",
            expected_second_spelling: OperandSpellingKind::Direct,
            expected_third_authored: "c",
            expected_third_spelling: OperandSpellingKind::Direct,
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        assert_eq!(
            slice(fixture.text, fixture.first),
            fixture.expected_first_authored
        );
        assert_eq!(
            slice(fixture.text, fixture.second),
            fixture.expected_second_authored
        );
        assert_eq!(
            slice(fixture.text, fixture.third),
            fixture.expected_third_authored
        );

        let whole = Range(fixture.first.0, fixture.third.1);
        let whole_text = slice(fixture.text, whole);
        let facts = recognize_selected_three_identifier_reference_additive_initializer(whole_text)
            .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(facts.first.authored, fixture.expected_first_authored);
        assert_eq!(facts.first.spelling, fixture.expected_first_spelling);
        assert_eq!(facts.second.authored, fixture.expected_second_authored);
        assert_eq!(facts.second.spelling, fixture.expected_second_spelling);
        assert_eq!(facts.third.authored, fixture.expected_third_authored);
        assert_eq!(facts.third.spelling, fixture.expected_third_spelling);

        // Authored spelling never becomes decoded identity (W5): an escaped
        // operand's authored text and semantic name differ exactly when it
        // is escaped.
        if fixture.expected_first_spelling == OperandSpellingKind::EscapedNonReserved {
            assert_ne!(facts.first.authored, facts.first.semantic_name);
        }
        if fixture.expected_second_spelling == OperandSpellingKind::EscapedNonReserved {
            assert_ne!(facts.second.authored, facts.second.semantic_name);
        }
        if fixture.expected_third_spelling == OperandSpellingKind::EscapedNonReserved {
            assert_ne!(facts.third.authored, facts.third.semantic_name);
        }

        assert_eq!(
            authored_anchor(795_200 + index as u64, fixture.text, fixture.first),
            fixture.expected_first_authored
        );
        assert_eq!(
            authored_anchor(795_250 + index as u64, fixture.text, fixture.second),
            fixture.expected_second_authored
        );
        assert_eq!(
            authored_anchor(795_300 + index as u64, fixture.text, fixture.third),
            fixture.expected_third_authored
        );
    }
}

/// Ordering and duplicate theorem (acceptance criteria 3, 5; W3/W4): authored
/// left-to-right order is load-bearing and never reordered by semantic name;
/// equal semantic names never deduplicate into fewer than three facts; and
/// distinct authored spellings that decode to the same semantic name remain
/// three distinct authored occurrences (W6 adjacent).
#[test]
fn authored_order_is_preserved_and_equal_semantic_names_are_never_deduplicated() {
    let reordered_facts =
        recognize_selected_three_identifier_reference_additive_initializer("c+a+b")
            .expect("c+a+b must recognize the theorem");
    assert_eq!(reordered_facts.first.semantic_name, "c");
    assert_eq!(reordered_facts.second.semantic_name, "a");
    assert_eq!(reordered_facts.third.semantic_name, "b");

    let all_duplicate_facts =
        recognize_selected_three_identifier_reference_additive_initializer("a+a+a")
            .expect("a+a+a must recognize the theorem");
    assert_eq!(all_duplicate_facts.first.semantic_name, "a");
    assert_eq!(all_duplicate_facts.second.semantic_name, "a");
    assert_eq!(all_duplicate_facts.third.semantic_name, "a");
    // Three independently owned facts remain even though every semantic name
    // is equal: each field is populated from its own independent operand
    // slice, never merged or collapsed into a smaller set.
    assert_eq!(all_duplicate_facts.first.authored, "a");
    assert_eq!(all_duplicate_facts.second.authored, "a");
    assert_eq!(all_duplicate_facts.third.authored, "a");

    // Mixed authored spelling, equal decoded semantic identity: the middle
    // operand's escaped authored spelling differs from its sibling direct
    // operands' authored spelling even though all three decode to "a".
    let mixed_spelling_facts =
        recognize_selected_three_identifier_reference_additive_initializer(r"a+\u{61}+a")
            .expect(r"a+\u{61}+a must recognize the theorem");
    assert_eq!(mixed_spelling_facts.first.authored, "a");
    assert_eq!(mixed_spelling_facts.second.authored, r"\u{61}");
    assert_eq!(mixed_spelling_facts.third.authored, "a");
    assert_eq!(mixed_spelling_facts.first.semantic_name, "a");
    assert_eq!(mixed_spelling_facts.second.semantic_name, "a");
    assert_eq!(mixed_spelling_facts.third.semantic_name, "a");
    assert_eq!(
        mixed_spelling_facts.first.spelling,
        OperandSpellingKind::Direct
    );
    assert_eq!(
        mixed_spelling_facts.second.spelling,
        OperandSpellingKind::EscapedNonReserved
    );
    assert_eq!(
        mixed_spelling_facts.third.spelling,
        OperandSpellingKind::Direct
    );
    assert_ne!(
        mixed_spelling_facts.second.authored,
        mixed_spelling_facts.first.authored
    );
}

/// Exact selected trivia around both binary operator boundaries (acceptance
/// criterion 9; W6): the already-accepted selected trivia set composes
/// independently on either side of each operator, an asymmetric mixture
/// (TAB/NBSP/LINE SEPARATOR/PARAGRAPH SEPARATOR) composes correctly across
/// both boundaries at once, comments never compose as operand trivia at any
/// of the four comment-firewall positions, and U+200B remains outside.
#[test]
fn selected_additive_trivia_matrix_around_both_operator_boundaries() {
    for candidate in [
        "a+b+c",
        "a + b + c",
        "a\t+\tb\t+\tc",
        // Representative asymmetric mixture spanning both boundaries at
        // once: TAB before `+`, NBSP after `+` before `b`, LINE SEPARATOR
        // before `-`, PARAGRAPH SEPARATOR after `-` before `c`.
        "a\t+\u{00A0}b\u{2028}-\u{2029}c",
    ] {
        let facts = recognize_selected_three_identifier_reference_additive_initializer(candidate)
            .unwrap_or_else(|| panic!("{candidate:?} must recognize the theorem"));
        assert_eq!(facts.first.semantic_name, "a");
        assert_eq!(facts.second.semantic_name, "b");
        assert_eq!(facts.third.semantic_name, "c");
    }

    // Comments never compose as operand trivia at any of the four boundary
    // positions.
    for comment_control in ["a/*c*/+b+c", "a+/*c*/b+c", "a+b/*c*/+c", "a+b+/*c*/c"] {
        assert!(
            recognize_selected_three_identifier_reference_additive_initializer(comment_control)
                .is_none(),
            "{comment_control:?}"
        );
    }

    // Adversarial boundary: ZERO WIDTH SPACE (U+200B) sits immediately
    // outside the frozen `Space_Separator` range and is never accepted
    // merely because it visually resembles spacing.
    assert!(!is_selected_additive_trivia('\u{200B}'));
    assert!(is_selected_additive_trivia('\u{00A0}'));
    assert!(
        recognize_selected_three_identifier_reference_additive_initializer("a+\u{200B}b+c")
            .is_none()
    );
}

/// Cardinality firewall (acceptance criteria 12, 13; W9/W10): one, two, and
/// four-or-more operand chains remain outside this exact bounded theorem; a
/// longer chain is never truncated to its first three operands.
#[test]
fn cardinality_firewall_keeps_one_two_and_four_or_more_operand_chains_outside() {
    for control in ["a", "a+b", "a+b+c+d", "a+b+c+d+e", "a - b - c - d"] {
        assert!(
            recognize_selected_three_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    // The exact bounded family itself remains independently valid; only
    // shorter and longer chains are unowned. Because the return type is
    // `Option`, a four-operand chain structurally cannot yield a partial
    // three-fact result: it can only ever be `None`.
    assert!(recognize_selected_three_identifier_reference_additive_initializer("a+b+c").is_some());
}

/// Whole-candidate transactionality (acceptance criterion 20; W10/W11): no
/// fact is ever committed unless the complete theorem succeeds. A trailing
/// operator, a missing operand, a malformed escape at any of the three
/// operand positions, or unexpected trailing content never yields a partial
/// result; earlier prepared operand evidence is only ever combined into the
/// returned facts in a single final expression.
#[test]
fn whole_candidate_transactionality_never_commits_partial_evidence() {
    for control in [
        "a+b+",
        r"a+b+\u{}",
        r"a+\u{}+c",
        r"\u{}+b+c",
        "a+b+c unexpected",
        "a+b+c+",
        "a+b+",
        "+b+c",
        "a++c",
    ] {
        assert!(
            recognize_selected_three_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    // The complete bounded candidate remains independently valid; only the
    // incomplete/overextended forms above are rejected.
    assert!(recognize_selected_three_identifier_reference_additive_initializer("a+b+c").is_some());
}

/// Punctuator / unary firewall (acceptance criterion 14; W12/W13): `++`/`--`
/// are never split into binary operators, `+=`/`-=` are never mistaken for
/// binary `+`/`-`, and unary-wrapped operands -- however trivia separates a
/// unary punctuator from a following binary one -- remain outside.
#[test]
fn punctuator_and_unary_firewall_keeps_update_assignment_and_unary_forms_outside() {
    for control in [
        "a++b+c", "a--b+c", "a+b++c", "a+b--c", "a+=b+c", "a-=b+c", "a+b+=c", "a+b-=c", "a+-b+c",
        "a-+b+c", "a+b+-c", "a+b-+c", "a+ +b+c", "a- -b+c", "a+b+ +c", "a+b- -c", "+a+b+c",
        "-a+b+c",
    ] {
        assert!(
            recognize_selected_three_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }
}

/// Heterogeneous operand firewall (acceptance criterion 15): numeric,
/// boolean, null, `this`, and string-literal operands remain outside at any
/// of the three positions.
#[test]
fn heterogeneous_operand_firewall_keeps_non_identifier_operands_outside() {
    for control in [
        "a+b+1",
        "a+1+c",
        "1+b+c",
        "a+b+true",
        "a+null+c",
        "this+b+c",
        "\"a\"+b+c",
    ] {
        assert!(
            recognize_selected_three_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }
}

/// Richer expression / precedence / grouping firewall (acceptance criterion
/// 16; W16/W17): grouping, member access, call, multiplicative/exponent
/// precedence, assignment, comma, conditional, and logical tails all remain
/// outside without any dedicated grouping/member/call/precedence recognition
/// code; the exact bounded family itself remains independently valid.
#[test]
fn richer_expression_precedence_and_grouping_firewall_remains_unowned() {
    for control in [
        "(a)+b+c",
        "a+(b)+c",
        "a+b+(c)",
        "a.b+c+d",
        "a+b.c+d",
        "a+b+c.d",
        "a()+b+c",
        "a+b()+c",
        "a+b+c()",
        "a*b+c+d",
        "a+b*c+d",
        "a+b+c*d",
        "a**b+c+d",
        "a=b+c+d",
        "a+b+c=d",
        "a+b+c,d",
        "a+b+c?d:e",
        "a+b||c",
        "a+b&&c",
        "a+b??c",
    ] {
        assert!(
            recognize_selected_three_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    assert!(recognize_selected_three_identifier_reference_additive_initializer("a+b+c").is_some());
}

/// Escaped `ReservedWord` firewall (acceptance criterion 17): a decoded
/// unconditionally reserved word never becomes a selected operand, whatever
/// position among the three it occupies.
#[test]
fn escaped_reserved_word_firewall_keeps_decoded_reserved_operand_outside_all_positions() {
    for control in [
        concat!("\\", "u0069f+b+c"),
        concat!("a+", "\\", "u0069f+c"),
        concat!("a+b+", "\\", "u0069f"),
    ] {
        assert!(
            recognize_selected_three_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    assert_eq!(
        decode_selected_escaped_identifier(r"\u{69}f"),
        Err(DecodeFailure::DecodedReserved)
    );
}

/// Malformed / position-invalid escape firewall (acceptance criterion 18;
/// W10/W11): a malformed or position-invalid escape at any of the three
/// operand positions never leaks a truncated prefix or a valid sibling fact
/// -- the whole candidate is rejected, whichever position fails.
#[test]
fn malformed_and_invalid_escape_firewall_never_leaks_a_sibling_fact_at_any_position() {
    for control in [
        r"\u{}+b+c",
        r"a+\u{}+c",
        r"a+b+\u{}",
        concat!("\\", "u0030+b+c"),
        concat!("a+", "\\", "u0030+c"),
        concat!("a+b+", "\\", "u0030"),
        concat!("a", "\\", "u002Db+c+d"),
        concat!("a+b", "\\", "u002Dc+d"),
        concat!("a+b+c", "\\", "u002Dd"),
        r"\uD800+b+c",
        r"a+\u{110000}+c",
    ] {
        assert!(
            recognize_selected_three_identifier_reference_additive_initializer(control).is_none(),
            "{control:?}"
        );
    }

    // The malformed/invalid-position operand's own independent decode
    // failure is the exact one this firewall exercises, confirming no
    // graceful truncation ever occurs before the whole candidate is judged.
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0030")),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("a", "\\", "u002D")),
        Err(DecodeFailure::InvalidPart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"\uD800"),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{110000}"),
        Err(DecodeFailure::NonCodePoint)
    );
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{}"),
        Err(DecodeFailure::MalformedEscape)
    );
}

/// Resource / failure semantics (acceptance criterion 19; W19): the
/// project's `ResourceLimited`/`InternalFailure` lifecycle states remain
/// distinct from `UnsupportedCoverage`, matching the minimal symbolic model
/// already accepted by the predecessor candidate-independent
/// `IdentifierReference` oracles (#241, #746/#747, #752/#753) -- this leaf
/// adds no further resource machinery beyond that already-accepted minimum.
#[test]
fn resource_and_internal_failure_lifecycle_states_remain_distinct_from_unsupported_coverage() {
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
    assert!(THIS_ORACLE_SOURCE.contains("ResourceLimited"));
    assert!(THIS_ORACLE_SOURCE.contains("InternalFailure"));
    assert_ne!(PROCESSING_FAILURES[0], "UnsupportedCoverage");
    assert_ne!(PROCESSING_FAILURES[1], "UnsupportedCoverage");
}

/// Freezes the presence-only, unqualified, validation-only handoff: no
/// operator enum, operator/whole-expression `SourceAnchor`, binary-expression
/// node, AST/CST, token tape, runtime/static vocabulary, or production
/// collection-representation freeze is introduced (acceptance criteria 21,
/// 22, 23; W14/W15/W20/W21 adjacent).
#[test]
fn handoff_remains_validation_only_with_no_operator_runtime_or_production_representation_freeze() {
    for forbidden in [
        concat!("enum Selected", "AdditiveOperatorKind"),
        concat!("enum Operator", "Kind"),
        concat!("enum Plus", "Minus"),
        concat!("struct Operator", "SourceAnchor"),
        concat!("struct WholeExpression", "SourceAnchor"),
        concat!("struct Binary", "ExpressionNode"),
        concat!("struct Additive", "ExpressionNode"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("enum Three", "Fact"),
        concat!("Vec", "<RecognizedOperand"),
        concat!("Get", "Value("),
        concat!("Resolve", "Binding("),
        concat!("Reference", "Record"),
        concat!("Environment", "Record"),
        concat!("To", "Numeric("),
        concat!("To", "Number("),
        concat!("To", "Primitive("),
        concat!("To", "String("),
        concat!("Bound", "Names("),
        concat!("VarDeclared", "Names("),
        concat!("LexicallyDeclared", "Names("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // No operator value is ever retained in the returned facts: the facts
    // type carries exactly the three operand fields and nothing else.
    assert!(THIS_ORACLE_SOURCE.contains("struct ThreeIdentifierReferenceAdditiveFacts"));
    assert!(THIS_ORACLE_SOURCE.contains("first: RecognizedOperand"));
    assert!(THIS_ORACLE_SOURCE.contains("second: RecognizedOperand"));
    assert!(THIS_ORACLE_SOURCE.contains("third: RecognizedOperand"));

    // Explicit non-freeze documentation is present, matching acceptance
    // criterion 23 and Issue #795 section 5/#688 comment `5767005572`.
    assert!(THIS_ORACLE_SOURCE.contains("does **not** freeze future production representation"));

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
