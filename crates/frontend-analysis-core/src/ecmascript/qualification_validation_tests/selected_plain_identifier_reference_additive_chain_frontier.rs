//! Candidate-independent ordered 2..N plain-`IdentifierReference` additive-chain
//! fact-preservation validation for Issue #801 (durable research: Issue #688
//! comment `5775942089`; accepted predecessor authorities: Issue #752 / PR
//! #753 ordered exactly-two `IdentifierReference` additive initializer Oracle,
//! Issue #795 / PR #796 ordered exactly-three `IdentifierReference` additive
//! initializer Oracle, Issue #693 / PR #694 project precedent for
//! independently freezing a `1..N` cardinality theorem after a narrower
//! historical leaf without rewriting that historical Oracle).
//!
//! This oracle qualifies only the unbounded expression-composition family:
//!
//! ```text
//! SelectedPlainIdentifierReferenceAdditiveChain ::=
//!     SelectedAcceptedIdentifierReference
//!     (
//!         SelectedAdditiveTrivia
//!         ("+" | "-")
//!         SelectedAdditiveTrivia
//!         SelectedAcceptedIdentifierReference
//!     )+
//!
//! SelectedAcceptedIdentifierReference ::=
//!     SelectedDirectIdentifierReference
//!   | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! Selected operand cardinality is `2..N`, not `1..N`: a single
//! `IdentifierReference` remains owned by already-accepted single-reference
//! authority and is not reclassified as an additive chain merely to make the
//! cardinality notation prettier. It does not call production lexical,
//! static-semantics, correspondence, Binding/Scope, aggregate, or runtime
//! evaluation code.
//!
//! ## Load-bearing novelty: a genuine iterative continuation model
//!
//! Issue #795 proved a bounded family with exactly two independently owned
//! interior boundaries (`operand1 -> operator1 -> operand2 -> operator2 ->
//! operand3`) but deliberately did not generalize into a loop: the theorem
//! was exactly three operands and two operators, nothing more. This Oracle
//! proves the strictly stronger claim that repeated evidence justifies: the
//! same continuation step -- selected trivia, one authored `+`/`-`, selected
//! trivia, one more accepted `IdentifierReference` -- composes an arbitrary
//! finite number of times, not merely up to three. `recognize_selected_plain_identifier_reference_additive_chain`
//! is therefore a genuine `loop`, not an unrolled fixed sequence of named
//! `first`/`second`/`third`/`fourth` fields: it is deliberately exercised
//! beyond four and five operands (an eight-operand sentinel below) precisely
//! so an accidental fixed cap cannot silently satisfy the suite.
//!
//! `operand_interior_run_end` is the same bounded, single left-to-right
//! forward pass already established by Issue #752/#795 (consuming one direct
//! code point or one syntactically well-formed `UnicodeEscapeSequence`
//! element at a time, stopping only at the first selected-trivia code point
//! or at `+`/`-`), invoked once per operand inside the loop rather than a
//! fixed number of times. Because the scan only stops at a trivia code point
//! or an authored `+`/`-`, it is never a general tokenizer: any other
//! character (`.`, `(`, `=`, `,`, `?`, an update/assignment operator byte,
//! and so on) is silently absorbed into the same run and is judged afterward,
//! at whole-operand-slice granularity, by `recognize_accepted_identifier_reference`
//! -- exactly the mechanism Issue #795's last (fully-remainder) operand
//! already relied on, generalized here to whichever operand turns out to be
//! the chain's last one, since the loop cannot know that in advance. This is
//! never `.find`/`.rfind`, later substring search, source rescanning after
//! recognition, reparse, retokenization, operator-relative endpoint
//! reconstruction, or decoded-length endpoint inference.
//!
//! `SelectedAdditiveTrivia` independently restates the complete
//! already-accepted selected-slice trivia contract established by Issue
//! #742/#743 and restated again by Issue #746/#747, Issue #752/#753, and
//! Issue #795/#796 (the same code-point set the production selected-trivia
//! recognizer accepts): `TAB`, `VT`, `FF`, `BOM`, `LF`, `CR`, `LINE
//! SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen Unicode 17
//! `Space_Separator` property. Comments remain outside this contract and are
//! never accepted as trivia.
//!
//! `SelectedDirectIdentifierReference` independently restates the
//! already-accepted Issue #237 direct escape-free `IdentifierName`
//! code-point shape (`is_id_start`/`is_id_continue` plus `$`/`_`) minus the
//! unconditionally reserved words. `SelectedEscapedNonReservedIdentifierReference`
//! independently restates the already-accepted Issue #241
//! `UnicodeEscapeSequence` decode/position/decoded-reserved-word theorem.
//! Neither restatement imports another Oracle's code: this leaf owns its own
//! copy of the already-accepted primitives it consumes, matching the
//! convention already set by Issue #752/#753 and Issue #795/#796.
//!
//! ## Outer trivia remains outside the naked theorem
//!
//! `operand_interior_run_end` stops precisely at a trivia code point or at
//! `+`/`-`, never anywhere else, so once an operand's run is found, the text
//! immediately following it is always either empty, or begins with trivia
//! and/or `+`/`-`. Leading outer trivia therefore fails immediately (the very
//! first run finds nothing to consume), and trailing outer trivia -- trivia
//! with no following operator -- falls through the same per-boundary
//! `Some('+') | Some('-') => ..., _ => return None` check every interior
//! boundary already uses: trivia trimmed down to nothing, with no operator
//! behind it, is rejected exactly like any other unexpected trailing content,
//! never silently absorbed.
//!
//! ## What is retained, and what is not
//!
//! The Oracle's only retained representation is `PlainIdentifierReferenceAdditiveChainFacts`,
//! an ordered non-empty sequence (`Vec<RecognizedOperand>`, minimum length
//! two) of operand facts, each preserving its own exact authored
//! `SourceAnchor`/fragment, decoded semantic name, and Direct/EscapedNonReserved
//! spelling state, in exact authored left-to-right order with duplicates
//! never deduplicated. This `Vec` is a minimal private test-only shape
//! chosen only because the theorem itself is `2..N`:
//!
//! ```text
//! ORACLE DYNAMIC STORAGE (private Vec<RecognizedOperand>)
//! !=
//! PRODUCTION STORAGE AUTHORITY
//! ```
//!
//! It explicitly does **not** freeze future production representation as
//! `Vec<Fact>`, `first: Fact, rest: Vec<Fact>`, `One | Two | Three | Many`, a
//! recursive occurrence representation, a generic non-empty collection, a
//! generic `Expression` node, or an AST/CST. In particular `One | Two | Three
//! | Many` must remain a viable later production candidate because it can
//! preserve already-accepted allocation-free behavior for cardinalities `<=
//! 3` and add allocation only when a fourth fact is actually required; this
//! Oracle decides none of that. It likewise does not share or merge the
//! initializer and free-standing production carriers, and does not create a
//! shared whole-chain production transaction: those owners retain materially
//! different rollback semantics (per Issue #688 comment `5775942089`) that
//! this validation-only leaf does not resolve. No operator kind, operator
//! `SourceAnchor`, or whole-expression `SourceAnchor` is retained; the
//! authored binary operators are recognized only to prove the exact bounded
//! continuation family.
//!
//! This is a validation-only leaf: production does not support this
//! arbitrary-cardinality `IdentifierReference` additive family at the #801
//! baseline, so every positive fixture below remains `UnsupportedCoverage`
//! under current production and exists only as independent Oracle evidence
//! for a future, separately authorized production decision. No completion
//! successor file accompanies this leaf: this Issue adds zero production
//! capability, so the frozen `193 / 10 / 183 / {}` completion partition
//! cannot move, matching the precedent set by #742/#743, #746/#747,
//! #752/#753, and #795/#796 (which likewise added zero production capability
//! and shipped without one).

use crate::{SourceId, SourceText};

use super::super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};
use super::inventory::RULE_UNITS;

const ISSUE_ID: u64 = 801;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("model.rs");
const PREVIOUS_TWO_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE: &str = include_str!(
    "../qualification_selected_two_identifier_reference_additive_initializer_validation_tests.rs"
);
const PREVIOUS_THREE_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE: &str = include_str!(
    "../qualification_selected_three_identifier_reference_additive_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str =
    include_str!("selected_plain_identifier_reference_additive_chain_frontier.rs");
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "ordered 2..N plain-IdentifierReference additive-chain frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "unary operands, non-IdentifierReference operands, parenthesized/member/call ",
    "operands, richer expression tails, comments, or top-level/Block var placement"
);
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];
const SELECTED_OPERAND_CARDINALITY: &str = "2..N";

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
// #237/#752/#795, never imported from any of those Oracles. ---

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
// as Issue #746/#747, Issue #752/#753, and Issue #795/#796 already restate
// it. ---

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

// --- The Issue #801 theorem itself. ---

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

/// The Oracle's only retained representation: an ordered, minimum-length-two
/// sequence of operand facts. This `Vec` shape is a minimal private test
/// representation chosen only because the theorem itself is `2..N`; it is
/// never a claim that future production must use `Vec<Fact>`, `first: Fact,
/// rest: Vec<Fact>`, `One | Two | Three | Many`, a recursive occurrence
/// representation, a generic non-empty collection, a generic `Expression`
/// node, or an AST/CST.
#[derive(Debug, Clone, PartialEq, Eq)]
struct PlainIdentifierReferenceAdditiveChainFacts<'a> {
    operands: Vec<RecognizedOperand<'a>>,
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

/// Finds where one operand ends: one left-to-right forward pass consuming
/// either one direct code point or one syntactically well-formed
/// `UnicodeEscapeSequence` element, stopping at the first selected-trivia
/// code point or at `+`/`-`, or at the end of the candidate. A malformed
/// escape aborts the whole candidate immediately (`None`) rather than
/// silently truncating the run at that point -- the malformed escape never
/// leaks an earlier valid-looking prefix. Position validity of the collected
/// run is judged afterward by `recognize_accepted_identifier_reference`,
/// never here. Because the scan stops only at trivia or `+`/`-`, whichever
/// operand happens to be the chain's last one has its run naturally extend to
/// the end of the candidate -- absorbing any other trailing content (`.`,
/// `(`, `=`, `,`, `?`, and so on) into the same slice, which then fails
/// identifier validation rather than truncating to a shorter accepted
/// prefix. This one function is invoked once per operand from inside
/// `recognize_selected_plain_identifier_reference_additive_chain`'s loop; it
/// is never a general tokenizer and never looks ahead past its own operand.
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

/// Central Issue #801 theorem: a genuine left-to-right iterative recognizer,
/// not an unrolled fixed sequence of named operand fields. Whole-candidate,
/// all-or-nothing: no fact is ever returned unless the complete bounded
/// family -- one accepted `IdentifierReference`, then one-or-more repetitions
/// of (trivia, exactly one authored `+`/`-`, trivia, one more accepted
/// `IdentifierReference`) -- is recognized over the entire candidate, with
/// nothing left over. Operand facts are only ever appended to a local `Vec`
/// during the loop and combined into the returned struct in a single final
/// expression, so a failure partway through the chain can never publish an
/// earlier prepared prefix as a partial result (`?` returns `None` from every
/// fallible step before that final expression is reached). Every authored
/// binary operator is recognized only to prove the exact bounded
/// continuation family; none is ever retained in the returned facts.
fn recognize_selected_plain_identifier_reference_additive_chain(
    candidate: &str,
) -> Option<PlainIdentifierReferenceAdditiveChainFacts<'_>> {
    let mut operands: Vec<RecognizedOperand<'_>> = Vec::new();
    let mut remaining = candidate;

    loop {
        let operand_end = operand_interior_run_end(remaining)?;
        let (operand_slice, after_operand) = remaining.split_at(operand_end);
        let operand = recognize_accepted_identifier_reference(operand_slice)?;
        operands.push(operand);

        if after_operand.is_empty() {
            break;
        }

        let after_trivia = &after_operand[trivia_run_end(after_operand)..];
        match after_trivia.chars().next() {
            Some('+') | Some('-') => {
                let after_operator = &after_trivia[1..];
                remaining = &after_operator[trivia_run_end(after_operator)..];
            }
            // Either trailing trivia with no operator behind it (outer
            // trivia does not belong to this naked theorem), or unexpected
            // trailing content that `operand_interior_run_end` stopped
            // short of absorbing: both remain unaccepted, never a partial
            // prefix.
            _ => return None,
        }
    }

    (operands.len() >= 2).then_some(PlainIdentifierReferenceAdditiveChainFacts { operands })
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 801);
    assert_eq!(SELECTED_OPERAND_CARDINALITY, "2..N");
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
        PREVIOUS_THREE_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 795")
    );
    assert!(
        PREVIOUS_THREE_IDENTIFIER_REFERENCE_ADDITIVE_ORACLE_SOURCE.contains(
            "ordered exactly-three IdentifierReference additive initializer frontier only"
        )
    );
    assert!(
        FRONTIER_SCOPE_NOTE
            .contains("ordered 2..N plain-IdentifierReference additive-chain frontier only")
    );

    for forbidden in [
        concat!("use super::super::", "selected_lexical_slice"),
        concat!("use super::super::", "selected_static_semantics"),
        concat!("use super::super::", "selected_qualification_integration"),
        concat!(
            "use super::super::",
            "selected_variable_statement_name_correspondence"
        ),
        concat!("use super::super::", "selected_binding_scope"),
        concat!(
            "use super::super::",
            "selected_one_level_block_binding_scope"
        ),
        concat!("use super::super::", "selected_binding_identifier"),
        concat!("use super::super::", "qualification;"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("recognize_selected_two_identifier_", "reference_additive"),
        concat!("recognize_selected_three_identifier_", "reference_additive"),
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
        concat!(
            "use super::super::",
            "selected_lexical_slice::is_selected_trivia"
        ),
        // No cross-Oracle import: this leaf restates rather than imports.
        concat!(
            "use super::qualification_selected_two_identifier_",
            "reference_additive_initializer_validation_tests"
        ),
        concat!(
            "use super::qualification_selected_three_identifier_",
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
    // accepted oracle at the same `super::super::unicode` path (this leaf
    // sits one module level deeper, inside `qualification_validation_tests`).
    assert!(
        THIS_ORACLE_SOURCE.contains(
            "use super::super::unicode::{is_id_continue, is_id_start, is_space_separator}"
        )
    );
}

/// Genuine iterative model, not an unrolled fixed-cardinality theorem
/// (acceptance criteria 2, 3, 4; W1): the recognizer is defined with a real
/// `loop`, never named `first`/`second`/`third`/`fourth` fields, and no
/// hardcoded four-or-five-operand cap exists anywhere in the source.
#[test]
fn recognition_is_a_genuine_loop_never_an_unrolled_fixed_cardinality_theorem() {
    assert!(THIS_ORACLE_SOURCE.contains("loop {"));
    // Defined exactly once as a real function; the second match is this
    // assertion's own search-string literal.
    assert_eq!(
        THIS_ORACLE_SOURCE
            .matches("fn operand_interior_run_end(")
            .count(),
        2
    );
    // Each entry is built with `concat!` so the assembled forbidden literal
    // does not appear as one contiguous run inside this very array, which
    // would otherwise always self-match. These check for concrete unrolled
    // named-field storage (as the two/three oracles legitimately use for
    // their own fixed cardinality), not the English words themselves, which
    // this module's own prose uses to explain what it deliberately does not
    // do.
    for forbidden in [
        concat!("four", "th: RecognizedOperand"),
        concat!("fif", "th: RecognizedOperand"),
        concat!("struct", " Four"),
        concat!("struct", " Five"),
        concat!("Operand", "Four"),
        concat!("Operand", "Five"),
        concat!("enum ", "ExactlyFour"),
        concat!("operands.len()", " == 4"),
        concat!("operands.len()", " == 5"),
        concat!("operands.len()", " >= 4 &&"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }
    assert!(THIS_ORACLE_SOURCE.contains("operands.len() >= 2"));
}

/// Required cardinality witnesses (acceptance criterion 6): 2, 3, 4, 5,
/// mixed-operator, and a long 8-operand sentinel all recognize with exactly
/// one fact per authored operand, in exact authored order. The 8-operand
/// sentinel exists to falsify an accidental fixed cap, not as a stress
/// benchmark.
#[test]
fn cardinality_witnesses_from_two_through_a_long_sentinel_all_recognize() {
    let two = recognize_selected_plain_identifier_reference_additive_chain("a+b")
        .expect("a+b must recognize the theorem");
    assert_eq!(two.operands.len(), 2);

    let three = recognize_selected_plain_identifier_reference_additive_chain("a+b+c")
        .expect("a+b+c must recognize the theorem");
    assert_eq!(three.operands.len(), 3);

    let four = recognize_selected_plain_identifier_reference_additive_chain("a+b+c+d")
        .expect("a+b+c+d must recognize the theorem");
    assert_eq!(four.operands.len(), 4);

    let five = recognize_selected_plain_identifier_reference_additive_chain("a+b+c+d+e")
        .expect("a+b+c+d+e must recognize the theorem");
    assert_eq!(five.operands.len(), 5);

    let mixed = recognize_selected_plain_identifier_reference_additive_chain("a-b+c-d+e")
        .expect("a-b+c-d+e must recognize the theorem");
    assert_eq!(mixed.operands.len(), 5);

    let mixed_other_order =
        recognize_selected_plain_identifier_reference_additive_chain("a+b-c+d-e+f")
            .expect("a+b-c+d-e+f must recognize the theorem");
    assert_eq!(mixed_other_order.operands.len(), 6);

    let long_sentinel =
        recognize_selected_plain_identifier_reference_additive_chain("a+b+c+d+e+f+g+h")
            .expect("a+b+c+d+e+f+g+h must recognize the theorem");
    assert_eq!(long_sentinel.operands.len(), 8);

    let expected_names = ["a", "b", "c", "d", "e", "f", "g", "h"];
    for (operand, expected) in long_sentinel.operands.iter().zip(expected_names) {
        assert_eq!(operand.semantic_name, expected);
        assert_eq!(operand.spelling, OperandSpellingKind::Direct);
    }
}

/// Required positive matrix with fixture-owned literal byte ranges for every
/// operand -- never derived by search, rescan, or reparse. Facts are
/// retained in authored left-to-right order, each with its own exact
/// authored anchor (acceptance criteria 8, 9; W1/W9).
#[test]
fn positive_matrix_pins_ordered_facts_with_exact_provenance() {
    struct Fixture {
        text: &'static str,
        operands: &'static [Range],
        expected_names: &'static [&'static str],
    }

    let fixtures = [
        Fixture {
            text: "const x = a + b + c + d;",
            operands: &[Range(10, 11), Range(14, 15), Range(18, 19), Range(22, 23)],
            expected_names: &["a", "b", "c", "d"],
        },
        Fixture {
            text: "const x = foo + bar + baz;",
            operands: &[Range(10, 13), Range(16, 19), Range(22, 25)],
            expected_names: &["foo", "bar", "baz"],
        },
        Fixture {
            text: "const x = a + b - c + d - e;",
            operands: &[
                Range(10, 11),
                Range(14, 15),
                Range(18, 19),
                Range(22, 23),
                Range(26, 27),
            ],
            expected_names: &["a", "b", "c", "d", "e"],
        },
    ];

    for (fixture_index, fixture) in fixtures.iter().enumerate() {
        let mut previous_end = 0;
        for operand in fixture.operands {
            assert!(operand.0 >= previous_end);
            previous_end = operand.1;
        }

        let whole = Range(fixture.operands[0].0, fixture.operands.last().unwrap().1);
        let whole_text = slice(fixture.text, whole);
        let facts = recognize_selected_plain_identifier_reference_additive_chain(whole_text)
            .unwrap_or_else(|| panic!("{whole_text:?} must recognize the theorem"));

        assert_eq!(facts.operands.len(), fixture.operands.len());
        for (index, (operand_range, expected_name)) in fixture
            .operands
            .iter()
            .zip(fixture.expected_names)
            .enumerate()
        {
            assert_eq!(slice(fixture.text, *operand_range), *expected_name);
            assert_eq!(facts.operands[index].authored, *expected_name);
            assert_eq!(facts.operands[index].semantic_name, *expected_name);
            assert_eq!(facts.operands[index].spelling, OperandSpellingKind::Direct);
            assert_eq!(
                authored_anchor(
                    801_000 + fixture_index as u64 * 10 + index as u64,
                    fixture.text,
                    *operand_range
                ),
                *expected_name
            );
        }
    }
}

/// Ordering and duplicate theorem (acceptance criteria 9, 10; W11): authored
/// left-to-right order is load-bearing and never reordered by semantic name;
/// equal semantic names are never deduplicated into fewer facts; and
/// distinct authored spellings that decode to the same semantic name remain
/// distinct authored occurrences.
#[test]
fn authored_order_is_preserved_and_equal_semantic_names_are_never_deduplicated() {
    let reordered = recognize_selected_plain_identifier_reference_additive_chain("c+a+b+d")
        .expect("c+a+b+d must recognize the theorem");
    let reordered_names: Vec<_> = reordered
        .operands
        .iter()
        .map(|operand| operand.semantic_name.as_str())
        .collect();
    assert_eq!(reordered_names, ["c", "a", "b", "d"]);

    let all_duplicate = recognize_selected_plain_identifier_reference_additive_chain("a+a+a+a")
        .expect("a+a+a+a must recognize the theorem");
    assert_eq!(all_duplicate.operands.len(), 4);
    for operand in &all_duplicate.operands {
        assert_eq!(operand.semantic_name, "a");
        assert_eq!(operand.authored, "a");
    }

    // Built with `concat!` over individually escaped fragments so the
    // literal bytes `\`, `u`, `0`, `0`, `6`, `1` survive intact rather than
    // being collapsed into a decoded Unicode scalar by an intermediate write
    // layer, matching the remediation Issue #752/#753 already applied.
    const MIXED_SPELLING_DUPLICATE_SOURCE: &str =
        concat!("a+", "\\", "u0061", "+a+", "\\", "u0061");
    const ESCAPED_A: &str = concat!("\\", "u0061");
    let mixed_spelling_duplicate = recognize_selected_plain_identifier_reference_additive_chain(
        MIXED_SPELLING_DUPLICATE_SOURCE,
    )
    .expect("mixed direct/escaped duplicate source must recognize the theorem");
    assert_eq!(mixed_spelling_duplicate.operands.len(), 4);
    let expected_authored = ["a", ESCAPED_A, "a", ESCAPED_A];
    let expected_spelling = [
        OperandSpellingKind::Direct,
        OperandSpellingKind::EscapedNonReserved,
        OperandSpellingKind::Direct,
        OperandSpellingKind::EscapedNonReserved,
    ];
    for (index, operand) in mixed_spelling_duplicate.operands.iter().enumerate() {
        assert_eq!(operand.authored, expected_authored[index]);
        assert_eq!(operand.spelling, expected_spelling[index]);
        assert_eq!(operand.semantic_name, "a");
    }
    // Every authored occurrence remains distinct evidence even though every
    // semantic name decodes identically (W11).
    assert_ne!(
        mixed_spelling_duplicate.operands[0].authored,
        mixed_spelling_duplicate.operands[1].authored
    );
}

/// Escaped operands at arbitrary positions (acceptance criterion 11):
/// first, each interior position, and the final position, each preserving
/// its own authored spelling distinct from its decoded semantic name.
#[test]
fn escaped_operands_at_arbitrary_positions_preserve_authored_vs_decoded_identity() {
    struct PositionFixture {
        source: &'static str,
        escaped_index: usize,
        expected_authored: &'static str,
    }

    // Built with `concat!` over individually escaped fragments so the
    // literal escape bytes survive intact rather than being collapsed into a
    // decoded Unicode scalar by an intermediate write layer, matching the
    // remediation Issue #752/#753 already applied.
    let fixtures = [
        PositionFixture {
            source: concat!("\\", "u0061", "+b+c+d"),
            escaped_index: 0,
            expected_authored: concat!("\\", "u0061"),
        },
        PositionFixture {
            source: concat!("a+", "\\", "u0062", "+c+d"),
            escaped_index: 1,
            expected_authored: concat!("\\", "u0062"),
        },
        PositionFixture {
            source: concat!("a+b+", "\\", "u0063", "+d"),
            escaped_index: 2,
            expected_authored: concat!("\\", "u0063"),
        },
        PositionFixture {
            source: concat!("a+b+c+", "\\", "u0064"),
            escaped_index: 3,
            expected_authored: concat!("\\", "u0064"),
        },
    ];

    let expected_names = ["a", "b", "c", "d"];
    for fixture in fixtures {
        let facts = recognize_selected_plain_identifier_reference_additive_chain(fixture.source)
            .unwrap_or_else(|| panic!("{:?} must recognize the theorem", fixture.source));
        assert_eq!(facts.operands.len(), 4);
        for (index, operand) in facts.operands.iter().enumerate() {
            assert_eq!(operand.semantic_name, expected_names[index]);
            if index == fixture.escaped_index {
                assert_eq!(operand.authored, fixture.expected_authored);
                assert_eq!(operand.spelling, OperandSpellingKind::EscapedNonReserved);
                assert_ne!(operand.authored, operand.semantic_name);
            } else {
                assert_eq!(operand.authored, expected_names[index]);
                assert_eq!(operand.spelling, OperandSpellingKind::Direct);
            }
        }
    }
}

/// Selected additive trivia composes at every internal boundary, not merely
/// the first (acceptance criterion 7): a mixed asymmetric trivia set spans
/// three separate boundaries in one candidate, and comments never compose as
/// operand trivia at any boundary.
#[test]
fn selected_additive_trivia_matrix_across_every_internal_boundary() {
    for candidate in [
        "a+b+c+d",
        "a + b + c + d",
        "a\t+\tb\t+\tc\t+\td",
        // Asymmetric mixture spanning three separate boundaries at once: TAB
        // before the first `+`, NBSP after it; LINE SEPARATOR before `-`,
        // PARAGRAPH SEPARATOR after it; BOM before the final `+`, VT after
        // it.
        "a\t+\u{00A0}b\u{2028}-\u{2029}c\u{FEFF}+\u{000B}d",
    ] {
        let facts = recognize_selected_plain_identifier_reference_additive_chain(candidate)
            .unwrap_or_else(|| panic!("{candidate:?} must recognize the theorem"));
        let names: Vec<_> = facts
            .operands
            .iter()
            .map(|operand| operand.semantic_name.as_str())
            .collect();
        assert_eq!(names, ["a", "b", "c", "d"]);
    }

    for comment_control in [
        "a/*c*/+b+c+d",
        "a+/*c*/b+c+d",
        "a+b/*c*/+c+d",
        "a+b+/*c*/c+d",
        "a+b+c/*c*/+d",
        "a+b+c+/*c*/d",
    ] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(comment_control).is_none(),
            "{comment_control:?}"
        );
    }

    assert!(!is_selected_additive_trivia('\u{200B}'));
    assert!(is_selected_additive_trivia('\u{00A0}'));
    assert!(
        recognize_selected_plain_identifier_reference_additive_chain("a+\u{200B}b+c+d").is_none()
    );
}

/// Outer trivia firewall (acceptance criterion 19): leading and trailing
/// statement/declaration-level trivia does not belong to the naked theorem
/// and is never silently absorbed, whatever internal trivia the same
/// candidate otherwise accepts.
#[test]
fn outer_leading_and_trailing_trivia_remain_outside_the_naked_theorem() {
    assert!(recognize_selected_plain_identifier_reference_additive_chain("a+b+c+d").is_some());
    for control in [
        " a+b+c+d",
        "a+b+c+d ",
        " a+b+c+d ",
        "\ta+b+c+d",
        "a+b+c+d\n",
    ] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
}

/// Whole-candidate transactionality and no-prefix-truncation (acceptance
/// criteria 5, 14, 15; W10): an incomplete chain never publishes a prepared
/// prefix, and a richer/malformed tail on an otherwise-valid-looking longer
/// chain never truncates to a shorter accepted result.
#[test]
fn whole_candidate_transactionality_never_commits_a_partial_or_truncated_result() {
    for control in [
        "a+",
        "a+b+",
        "a+b+c+",
        "a+b+c+d+",
        "+b+c+d",
        "a++c+d",
        "a+b+c unexpected",
        "a+b+c+d.e",
        "a+b+c+d()",
        "a+b+c+d=e",
        "a+b+c+d,e",
        "a+b+c+d?e:f",
    ] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(control).is_none(),
            "{control:?}"
        );
    }

    // The complete bounded family itself remains independently valid, and a
    // longer chain is never truncated to a shorter accepted prefix: because
    // the return type is `Option`, a rejected candidate structurally cannot
    // yield a partial `Vec` of fewer facts.
    assert!(recognize_selected_plain_identifier_reference_additive_chain("a+b+c+d").is_some());
    assert!(recognize_selected_plain_identifier_reference_additive_chain("a+b+c+d+e").is_some());
}

/// Cardinality firewall (acceptance criterion 1): a single `IdentifierReference`
/// remains outside this exact `2..N` theorem, owned instead by already-accepted
/// single-reference authority.
#[test]
fn single_reference_cardinality_remains_outside_the_two_to_n_theorem() {
    for control in ["a", "foo", r"a"] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
    assert!(recognize_selected_plain_identifier_reference_additive_chain("a+b").is_some());
}

/// Unary / additive ambiguity firewall (acceptance criterion 16; W12):
/// leading unary, right-unary, and doubled-punctuator variants remain
/// outside this plain-reference theorem at any boundary.
#[test]
fn unary_and_additive_ambiguity_firewall_keeps_signed_and_doubled_forms_outside() {
    for control in [
        "+a+b+c+d",
        "-a-b-c-d",
        "a+-b+c+d",
        "a-+b+c+d",
        "a+ +b+c+d",
        "a- -b+c+d",
        "a+b+-c+d",
        "a+b-+c+d",
        "a++b+c+d",
        "a+b++c+d",
        "a--b+c+d",
        "a+b--c+d",
    ] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
}

/// Heterogeneous operand firewall (acceptance criterion 17): numeric,
/// boolean, `null`, `this`, and string-literal operands remain outside at
/// any position.
#[test]
fn heterogeneous_operand_firewall_keeps_non_identifier_operands_outside() {
    for control in [
        "a+b+c+1",
        "a+b+1+d",
        "1+b+c+d",
        "a+b+c+true",
        "a+b+null+d",
        "this+b+c+d",
        "\"a\"+b+c+d",
    ] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
}

/// Richer expression / precedence / grouping firewall (acceptance criterion
/// 18): grouping, member access, call, multiplicative precedence, and
/// assignment tails all remain outside without any dedicated
/// grouping/member/call/precedence recognition code.
#[test]
fn richer_expression_precedence_and_grouping_firewall_remains_unowned() {
    for control in [
        "(a)+b+c+d",
        "a+(b)+c+d",
        "a.b+c+d",
        "a+b.c+d",
        "a()+b+c+d",
        "a+b()+c+d",
        "a*b+c+d",
        "a+b*c+d",
        "a=b+c+d",
        "a+b+c=d",
    ] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
    assert!(recognize_selected_plain_identifier_reference_additive_chain("a+b+c+d").is_some());
}

/// Escaped `ReservedWord` firewall (acceptance criterion 12): a decoded
/// unconditionally reserved word never becomes a selected operand at the
/// first, an interior, or the final position.
#[test]
fn escaped_reserved_word_firewall_keeps_decoded_reserved_operand_outside_all_positions() {
    for control in [
        concat!("\\", "u0069f+b+c+d"),
        concat!("a+", "\\", "u0069f+c+d"),
        concat!("a+b+", "\\", "u0069f+d"),
        concat!("a+b+c+", "\\", "u0069f"),
    ] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{69}f"),
        Err(DecodeFailure::DecodedReserved)
    );
}

/// Malformed escape firewall (acceptance criterion 13; W10): a malformed
/// escape at the first, an interior, or the final position never leaks a
/// truncated prefix or a valid sibling fact -- the whole candidate is
/// rejected.
#[test]
fn malformed_escape_firewall_never_leaks_a_sibling_fact_at_any_position() {
    for control in [r"\u{}+b+c+d", r"a+\u{}+c+d", r"a+b+\u{}+d", r"a+b+c+\u{}"] {
        assert!(
            recognize_selected_plain_identifier_reference_additive_chain(control).is_none(),
            "{control:?}"
        );
    }
    assert_eq!(
        decode_selected_escaped_identifier(r"\u{}"),
        Err(DecodeFailure::MalformedEscape)
    );
}

/// Resource / failure semantics (W19): the project's
/// `ResourceLimited`/`InternalFailure` lifecycle states remain distinct from
/// `UnsupportedCoverage`, matching the minimal symbolic model already
/// accepted by the predecessor candidate-independent `IdentifierReference`
/// oracles (#241, #746/#747, #752/#753, #795/#796) -- this leaf adds no
/// further resource machinery beyond that already-accepted minimum.
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
/// node, AST/CST, token tape, runtime/static vocabulary, or frozen
/// production collection-representation choice is introduced (acceptance
/// criteria 20, 21, 22; W6/W7/W8/W16/W17/W18).
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
        concat!("enum One", "Two", "Three", "Many"),
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

    assert!(THIS_ORACLE_SOURCE.contains("struct PlainIdentifierReferenceAdditiveChainFacts"));
    assert!(THIS_ORACLE_SOURCE.contains("operands: Vec<RecognizedOperand"));
    assert!(THIS_ORACLE_SOURCE.contains("ORACLE DYNAMIC STORAGE"));
    assert!(THIS_ORACLE_SOURCE.contains("PRODUCTION STORAGE AUTHORITY"));
    assert!(THIS_ORACLE_SOURCE.contains("does **not** freeze future production representation"));
    assert!(THIS_ORACLE_SOURCE.contains("One | Two | Three | Many"));
    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}

/// Completion hard-zero (acceptance criteria 24, 25; W21/W22): this
/// validation-only leaf adds zero production capability, so it must not
/// alter the frozen `193 / 10 / 183` rule-unit partition. The authoritative
/// count assertions already live in `qualification_validation_tests::mod`
/// (`frozen_rule_counts_match_the_independent_per_container_universe`); this
/// is a focused local restatement of the same frozen totals so this leaf
/// carries its own explicit non-movement evidence.
#[test]
fn completion_partition_remains_the_frozen_193_10_183_totals() {
    const EXPECTED_TOTAL_RULE_UNITS: usize = 193;
    const EXPECTED_ENVELOPE_INACTIVE_RULE_UNITS: usize = 10;
    const EXPECTED_ACTIVE_RULE_UNITS: usize = 183;
    assert_eq!(
        EXPECTED_ACTIVE_RULE_UNITS + EXPECTED_ENVELOPE_INACTIVE_RULE_UNITS,
        EXPECTED_TOTAL_RULE_UNITS
    );
    assert_eq!(RULE_UNITS.len(), EXPECTED_TOTAL_RULE_UNITS);
    // Built with `concat!` so the assembled forbidden literal does not
    // appear as one contiguous run inside this very assertion.
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("RuleUnitKind::", "NormativeRule")));
}
