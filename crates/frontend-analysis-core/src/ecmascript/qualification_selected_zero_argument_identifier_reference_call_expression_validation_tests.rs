//! Candidate-independent bounded zero-argument `IdentifierReference`
//! `CallExpression` source-shape and callee fact-preservation validation for
//! Issue #849 (durable research: Issue #688 comment `5896253950` post-#848
//! zero-base frontier authority; structural precedent: Issue #831 exactly-one
//! grouping-layer `ParenthesizedExpression` `IdentifierReference` Oracle and
//! Issue #839 bounded `delete` `IdentifierReference` Oracle; accepted premise
//! authority: Issue #237 direct escape-free `IdentifierReference`
//! source/name boundary, Issue #241 escaped `IdentifierReference`
//! initializer Oracle).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedZeroArgumentIdentifierReferenceCallExpression ::=
//!     SelectedAcceptedIdentifierReference
//!     SelectedCallTrivia
//!     "("
//!     SelectedCallTrivia
//!     ")"
//!
//! SelectedCallTrivia ::= ZERO-OR-MORE existing selected trivia
//!
//! SelectedAcceptedIdentifierReference ::=
//!       SelectedDirectIdentifierReference
//!     | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! in the existing selected top-level `LexicalDeclaration+` Script slice
//! carrier. The carrier is a validation-harness choice, not a claim that the
//! theorem is `LexicalDeclaration`-specific. This oracle does not call
//! production lexical, static-semantics, correspondence, Binding/Scope,
//! aggregate, or runtime evaluation code.
//!
//! ## Normative basis (pinned ECMA-262, `d89c03f2...`)
//!
//! `CallExpression : CoverCallExpressionAndAsyncArrowHead` covers
//! `CallMemberExpression : MemberExpression Arguments`, and
//! `Arguments : ( )` is the empty argument list. Neither production carries
//! a `[no LineTerminator here]` restriction, so a `LineTerminator` between
//! the callee and `(` is ordinary call continuation: the ASI rule inserts a
//! semicolon only before an *offending* token that no production allows, and
//! `(` after a complete `IdentifierReference` is allowed by the call
//! grammar. That grammar-level reading -- not the production ASI/terminator
//! logic -- is this Oracle's authority for `a\n()`.
//!
//! The pinned Left-Hand-Side Expressions Early Error clause contains only
//! `OptionalChain : ?. TemplateLiteral | OptionalChain TemplateLiteral` and
//! `ImportMeta` (frozen `EE-09-R01`/`EE-09-R02`); the only Early Errors that
//! mention `CoverCallExpressionAndAsyncArrowHead` belong to
//! `AsyncArrowFunction : CoverCallExpressionAndAsyncArrowHead => AsyncConciseBody`
//! (frozen `EE-35`) and require the `=>` continuation. The
//! `AssignmentTargetType` of a `CallExpression` is `web-compat` or `invalid`
//! (a normative-optional host step), but it is consumed only by assignment,
//! update, and for-in/of operators, none of which this theorem admits.
//! Section "C. Early Error reachability model" restates that reasoning
//! against the frozen inventory instead of asserting it.
//!
//! Pinned Test262 is not cited as evidence by this Oracle.
//!
//! ## Vocabulary
//!
//! The repository's existing frontier vocabulary is reused:
//! `SelectedAcceptedIncomplete` is the Issue's `Matched` (a valid bounded
//! zero-argument call, with the callee `IdentifierReference` evidence), and
//! `UnsupportedCoverage` is the Issue's `NotSelected / OutsideBoundedTheorem`.
//! `ResourceLimited` / `InternalFailure` stay a distinct processing-failure
//! identity: this Oracle's recognizer is a total, allocation-bounded function
//! over one candidate and never produces either, and neither may be folded
//! into ordinary non-selection.
//!
//! `CallEvidence` below is *Oracle-side* evidence used to seal delimiter
//! ownership and trivia boundaries. It creates no requirement that a future
//! production consumer store a call anchor, delimiter anchors, an `Arguments`
//! identity, a call operator, or a call result: the production-facing theorem
//! is only "retain exactly the existing callee `IdentifierReference` fact".
//!
//! `SelectedCallTrivia` independently restates the same already-accepted
//! selected-slice trivia contract reused by Issue #827/#831/#839 (the same
//! code-point set `is_selected_trivia` in `selected_lexical_slice.rs`
//! recognizes, restated here rather than imported): `TAB`, `VT`, `FF`, `BOM`,
//! `LF`, `CR`, `LINE SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen
//! Unicode 17 `Space_Separator` property. Comments remain outside.
//!
//! This is a validation-only leaf: production supports no call syntax at the
//! #849 baseline, so every positive fixture below is independent Oracle
//! evidence for a future, separately authorized production decision, never a
//! claim that production already accepts it. No runtime, direct-eval,
//! `ResolveBinding`, `IsCallable`, `GetValue`, `this`, or return-value claim
//! is made: `eval()` is only source syntax here.
//!
//! Production is explicitly NOT authorized by Issue #849.

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 849;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_PARENTHESIZED_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_parenthesized_identifier_reference_validation_tests.rs");
const PREVIOUS_DELETE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_delete_identifier_reference_unary_expression_validation_tests.rs"
);
const PREVIOUS_ESCAPED_IDENTIFIER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_zero_argument_identifier_reference_call_expression_validation_tests.rs"
);
const COMPLETION_AUTHORITY_SOURCE: &str =
    include_str!("qualification_validation_tests/selected_slice_completion.rs");
const INVENTORY_SOURCE: &str = include_str!("qualification_validation_tests/inventory.rs");
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "exactly-one zero-argument IdentifierReference CallExpression frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "non-empty Arguments, chained/member/optional/new/super calls, richer ",
    "outer expressions, comments, AsyncArrowFunction/cover-grammar ",
    "reinterpretation, runtime call semantics, or top-level/Block var placement"
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

/// Every code point of the selected trivia contract that this Oracle sweeps
/// in both trivia positions: the eight explicit selected code points plus a
/// representative spread of the frozen Unicode 17 `Space_Separator` property.
const SELECTED_TRIVIA_SWEEP: &[char] = &[
    '\u{0009}', '\u{000B}', '\u{000C}', '\u{0020}', '\u{00A0}', '\u{FEFF}', '\u{000A}', '\u{000D}',
    '\u{2028}', '\u{2029}', '\u{1680}', '\u{2000}', '\u{200A}', '\u{202F}', '\u{205F}', '\u{3000}',
];

/// Code points that look like separators but are not selected trivia under
/// the frozen contract (`Cf` format characters, `NEL`, and the Unicode 6.3
/// reclassified `U+180E`). None of them is also an `IdentifierPart`.
const NON_TRIVIA_SEPARATOR_LOOKALIKES: &[char] = &['\u{0085}', '\u{180E}', '\u{200B}', '\u{2060}'];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range(usize, usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    /// The Issue's `Matched`.
    SelectedAcceptedIncomplete,
    /// The Issue's `NotSelected / OutsideBoundedTheorem`. Valid-but-outside
    /// ECMAScript and malformed/incomplete candidates alike land here: this
    /// Oracle never upgrades a decline into a normative `SyntaxError`.
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

/// Callee-atom failures, kept exactly as the already-accepted
/// `IdentifierReference` policy classifies them; never re-labelled as
/// call-specific.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CalleeFailure {
    NoIdentifierStart,
    DirectReservedWord,
    Decode(DecodeFailure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclineReason {
    Callee(CalleeFailure),
    /// After the callee and selected trivia the next code point is not `(`.
    MissingCallOpener,
    /// The candidate ends before the paired `)`.
    IncompleteSuffix,
    /// A code point other than selected trivia or `)` follows `(`
    /// (non-empty arguments, comments, spread, ...).
    UnownedInsideArguments,
    /// A complete zero-argument call prefix exists but the candidate
    /// continues past its `)`.
    TrailingContinuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CalleeEvidence {
    range: Range,
    name: String,
    provenance: IdentifierReferenceProvenance,
}

/// Oracle-side evidence with candidate-relative ranges. `close.1` is the end
/// of the complete zero-argument call prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CallEvidence {
    callee: CalleeEvidence,
    open: Range,
    close: Range,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CallRecognition {
    Matched(CallEvidence),
    Declined(DeclineReason),
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

fn shift(range: Range, base: usize) -> Range {
    Range(range.0 + base, range.1 + base)
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

/// `SelectedAcceptedIdentifierReference` over one *whole* spelling: Direct
/// first, EscapedNonReserved otherwise. This is the bare-atom authority; it
/// owns `a` and `\u0061` on their own and never sees call punctuation.
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

/// `SelectedCallTrivia` code point: the frozen `is_selected_trivia`
/// code-point set plus the frozen Unicode 17 `Space_Separator` property. Not
/// a call into the production selected-trivia recognizer. Comments are not
/// members.
fn is_selected_call_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

/// Consumes zero or more selected trivia code points starting at `from` and
/// returns the offset after the last one.
fn skip_call_trivia(text: &str, from: usize) -> usize {
    let mut end = from;
    for code_point in text[from..].chars() {
        if !is_selected_call_trivia(code_point) {
            break;
        }
        end += code_point.len_utf8();
    }
    end
}

/// Oracle-owned maximal-`IdentifierName` callee scan over the head of
/// `text`: one forward pass consuming identifier-part code points and whole
/// `UnicodeEscapeSequence`s, then classification of exactly the consumed
/// spelling through the restated bare-atom policy. The callee range is what
/// the scan consumed -- never an endpoint reconstructed from a decoded
/// length or a later search.
fn scan_callee(text: &str) -> Result<CalleeEvidence, CalleeFailure> {
    let bytes = text.as_bytes();
    let mut offset = 0_usize;
    let mut has_escape = false;

    while offset < bytes.len() {
        if bytes[offset] == b'\\' {
            let (_, end) =
                decode_unicode_escape_at(bytes, offset).map_err(CalleeFailure::Decode)?;
            has_escape = true;
            offset = end;
            continue;
        }
        let scalar = text[offset..]
            .chars()
            .next()
            .expect("offset is on a code point boundary inside the text");
        if !is_direct_identifier_part(scalar) {
            break;
        }
        offset += scalar.len_utf8();
    }

    if offset == 0 {
        return Err(CalleeFailure::NoIdentifierStart);
    }
    let spelling = &text[..offset];

    let (name, provenance) = if has_escape {
        let name = decode_selected_escaped_identifier(spelling).map_err(CalleeFailure::Decode)?;
        (name, IdentifierReferenceProvenance::EscapedNonReserved)
    } else if !is_escape_free_identifier_name(spelling) {
        return Err(CalleeFailure::Decode(DecodeFailure::InvalidStart));
    } else if UNCONDITIONALLY_RESERVED_WORDS.contains(&spelling) {
        return Err(CalleeFailure::DirectReservedWord);
    } else {
        (spelling.to_owned(), IdentifierReferenceProvenance::Direct)
    };

    Ok(CalleeEvidence {
        range: Range(0, offset),
        name,
        provenance,
    })
}

/// Central Issue #849 theorem, *prefix* form: the head of `text` is exactly
/// one accepted callee, zero or more selected trivia, exactly one `(`, zero
/// or more selected trivia, and exactly one paired `)`. The scan is a single
/// left-to-right pass of this Oracle's own; it never searches, rescans, or
/// reparses. A locally complete prefix is *not* a whole-candidate match: see
/// [`recognize_call`].
fn scan_zero_argument_call_prefix(text: &str) -> Result<CallEvidence, DeclineReason> {
    let callee = scan_callee(text).map_err(DeclineReason::Callee)?;

    let open_at = skip_call_trivia(text, callee.range.1);
    if !text[open_at..].starts_with('(') {
        return Err(DeclineReason::MissingCallOpener);
    }
    let close_at = skip_call_trivia(text, open_at + 1);
    match text[close_at..].chars().next() {
        None => Err(DeclineReason::IncompleteSuffix),
        Some(')') => Ok(CallEvidence {
            callee,
            open: Range(open_at, open_at + 1),
            close: Range(close_at, close_at + 1),
        }),
        Some(_) => Err(DeclineReason::UnownedInsideArguments),
    }
}

/// Whole-candidate recognition (transactional): the candidate is exactly one
/// zero-argument call, ending at its paired `)`. Leading and trailing trivia
/// belong to the carrier, and any continuation past `)` declines the whole
/// candidate without committing the locally complete prefix.
fn recognize_call(candidate: &str) -> CallRecognition {
    match scan_zero_argument_call_prefix(candidate) {
        Err(reason) => CallRecognition::Declined(reason),
        Ok(evidence) if evidence.close.1 == candidate.len() => CallRecognition::Matched(evidence),
        Ok(_) => CallRecognition::Declined(DeclineReason::TrailingContinuation),
    }
}

/// Connects each fixture's expected outcome to actual Oracle recognition of
/// that exact candidate. A recognized bounded candidate is
/// `SelectedAcceptedIncomplete`; every decline is `UnsupportedCoverage`.
fn classify_frontier_outcome(candidate: &str) -> FrontierOutcome {
    match recognize_call(candidate) {
        CallRecognition::Matched(_) => FrontierOutcome::SelectedAcceptedIncomplete,
        CallRecognition::Declined(_) => FrontierOutcome::UnsupportedCoverage,
    }
}

fn declined_reason(candidate: &str) -> DeclineReason {
    match recognize_call(candidate) {
        CallRecognition::Declined(reason) => reason,
        CallRecognition::Matched(evidence) => {
            panic!("{candidate:?} unexpectedly matched: {evidence:?}")
        }
    }
}

fn matched_evidence(candidate: &str) -> CallEvidence {
    match recognize_call(candidate) {
        CallRecognition::Matched(evidence) => evidence,
        CallRecognition::Declined(reason) => {
            panic!("{candidate:?} unexpectedly declined: {reason:?}")
        }
    }
}

// ---------------------------------------------------------------------------
// A. Source-composition fixtures
// ---------------------------------------------------------------------------

/// Fixture-owned literal ranges (never derived by search, rescan, or
/// reparse). `pre_open_trivia` / `inner_trivia` pin the exact authored trivia
/// strings between the callee and `(` and between `(` and `)`.
struct PositiveRow {
    source: &'static str,
    callee: Range,
    open: Range,
    close: Range,
    whole: Range,
    pre_open_trivia: &'static str,
    inner_trivia: &'static str,
    expected_fragment: &'static str,
    expected_name: &'static str,
    expected_provenance: IdentifierReferenceProvenance,
}

const DIRECT: IdentifierReferenceProvenance = IdentifierReferenceProvenance::Direct;
const ESCAPED: IdentifierReferenceProvenance = IdentifierReferenceProvenance::EscapedNonReserved;

const CALLEE_MATRIX: &[PositiveRow] = &[
    PositiveRow {
        source: "const x = a();",
        callee: Range(10, 11),
        open: Range(11, 12),
        close: Range(12, 13),
        whole: Range(10, 13),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = \u{03C0}();",
        callee: Range(10, 12),
        open: Range(12, 13),
        close: Range(13, 14),
        whole: Range(10, 14),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "\u{03C0}",
        expected_name: "\u{03C0}",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = \u{1D49C}();",
        callee: Range(10, 14),
        open: Range(14, 15),
        close: Range(15, 16),
        whole: Range(10, 16),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "\u{1D49C}",
        expected_name: "\u{1D49C}",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = \\u0061();",
        callee: Range(10, 16),
        open: Range(16, 17),
        close: Range(17, 18),
        whole: Range(10, 18),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "\\u0061",
        expected_name: "a",
        expected_provenance: ESCAPED,
    },
    PositiveRow {
        source: "const x = f\\u006Fo();",
        callee: Range(10, 18),
        open: Range(18, 19),
        close: Range(19, 20),
        whole: Range(10, 20),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "f\\u006Fo",
        expected_name: "foo",
        expected_provenance: ESCAPED,
    },
    PositiveRow {
        source: r"const x = \u{66}oo();",
        callee: Range(10, 18),
        open: Range(18, 19),
        close: Range(19, 20),
        whole: Range(10, 20),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: r"\u{66}oo",
        expected_name: "foo",
        expected_provenance: ESCAPED,
    },
    PositiveRow {
        source: r"const x = \u{1D49C}();",
        callee: Range(10, 19),
        open: Range(19, 20),
        close: Range(20, 21),
        whole: Range(10, 21),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: r"\u{1D49C}",
        expected_name: "\u{1D49C}",
        expected_provenance: ESCAPED,
    },
];

/// Trivia between the callee and `(`. The `LineTerminator` row is
/// load-bearing: it remains one call under this theorem and is never an
/// ASI-owned statement split.
const PRE_OPEN_TRIVIA_MATRIX: &[PositiveRow] = &[
    PositiveRow {
        source: "const x = a ();",
        callee: Range(10, 11),
        open: Range(12, 13),
        close: Range(13, 14),
        whole: Range(10, 14),
        pre_open_trivia: " ",
        inner_trivia: "",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = a\t();",
        callee: Range(10, 11),
        open: Range(12, 13),
        close: Range(13, 14),
        whole: Range(10, 14),
        pre_open_trivia: "\t",
        inner_trivia: "",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = a\n();",
        callee: Range(10, 11),
        open: Range(12, 13),
        close: Range(13, 14),
        whole: Range(10, 14),
        pre_open_trivia: "\n",
        inner_trivia: "",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = a\u{00A0}();",
        callee: Range(10, 11),
        open: Range(13, 14),
        close: Range(14, 15),
        whole: Range(10, 15),
        pre_open_trivia: "\u{00A0}",
        inner_trivia: "",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = \\u0061\n();",
        callee: Range(10, 16),
        open: Range(17, 18),
        close: Range(18, 19),
        whole: Range(10, 19),
        pre_open_trivia: "\n",
        inner_trivia: "",
        expected_fragment: "\\u0061",
        expected_name: "a",
        expected_provenance: ESCAPED,
    },
];

/// Trivia inside the empty `Arguments`, and both positions together.
const INNER_TRIVIA_MATRIX: &[PositiveRow] = &[
    PositiveRow {
        source: "const x = a( );",
        callee: Range(10, 11),
        open: Range(11, 12),
        close: Range(13, 14),
        whole: Range(10, 14),
        pre_open_trivia: "",
        inner_trivia: " ",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = a(\t);",
        callee: Range(10, 11),
        open: Range(11, 12),
        close: Range(13, 14),
        whole: Range(10, 14),
        pre_open_trivia: "",
        inner_trivia: "\t",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = a(\n);",
        callee: Range(10, 11),
        open: Range(11, 12),
        close: Range(13, 14),
        whole: Range(10, 14),
        pre_open_trivia: "",
        inner_trivia: "\n",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = a(\u{00A0});",
        callee: Range(10, 11),
        open: Range(11, 12),
        close: Range(14, 15),
        whole: Range(10, 15),
        pre_open_trivia: "",
        inner_trivia: "\u{00A0}",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = a ( );",
        callee: Range(10, 11),
        open: Range(12, 13),
        close: Range(14, 15),
        whole: Range(10, 15),
        pre_open_trivia: " ",
        inner_trivia: " ",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    // Many heterogeneous trivia code points in both positions at once:
    // SP TAB LS NBSP | ( | BOM SP CR LF PS | ).
    PositiveRow {
        source: "const x = a \t\u{2028}\u{00A0}(\u{FEFF} \r\n\u{2029});",
        callee: Range(10, 11),
        open: Range(18, 19),
        close: Range(28, 29),
        whole: Range(10, 29),
        pre_open_trivia: " \t\u{2028}\u{00A0}",
        inner_trivia: "\u{FEFF} \r\n\u{2029}",
        expected_fragment: "a",
        expected_name: "a",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = f\\u006Fo\n(\t);",
        callee: Range(10, 18),
        open: Range(19, 20),
        close: Range(21, 22),
        whole: Range(10, 22),
        pre_open_trivia: "\n",
        inner_trivia: "\t",
        expected_fragment: "f\\u006Fo",
        expected_name: "foo",
        expected_provenance: ESCAPED,
    },
];

/// Callee names with special surrounding grammar or runtime lore. Each is a
/// plain `IdentifierReference` here: source syntax only.
const SPECIAL_NAME_MATRIX: &[PositiveRow] = &[
    PositiveRow {
        source: "const x = eval();",
        callee: Range(10, 14),
        open: Range(14, 15),
        close: Range(15, 16),
        whole: Range(10, 16),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "eval",
        expected_name: "eval",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = arguments();",
        callee: Range(10, 19),
        open: Range(19, 20),
        close: Range(20, 21),
        whole: Range(10, 21),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "arguments",
        expected_name: "arguments",
        expected_provenance: DIRECT,
    },
    // `async` with no `=>` continuation is an ordinary `IdentifierReference`
    // call: the cover production is not reinterpreted, so frozen `EE-35`
    // does not apply.
    PositiveRow {
        source: "const x = async();",
        callee: Range(10, 15),
        open: Range(15, 16),
        close: Range(16, 17),
        whole: Range(10, 17),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "async",
        expected_name: "async",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = yield();",
        callee: Range(10, 15),
        open: Range(15, 16),
        close: Range(16, 17),
        whole: Range(10, 17),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "yield",
        expected_name: "yield",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = await();",
        callee: Range(10, 15),
        open: Range(15, 16),
        close: Range(16, 17),
        whole: Range(10, 17),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "await",
        expected_name: "await",
        expected_provenance: DIRECT,
    },
    // Maximal authored IdentifierName: the callee is never split.
    PositiveRow {
        source: "const x = ab$_();",
        callee: Range(10, 14),
        open: Range(14, 15),
        close: Range(15, 16),
        whole: Range(10, 16),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "ab$_",
        expected_name: "ab$_",
        expected_provenance: DIRECT,
    },
    PositiveRow {
        source: "const x = a\\u0062();",
        callee: Range(10, 17),
        open: Range(17, 18),
        close: Range(18, 19),
        whole: Range(10, 19),
        pre_open_trivia: "",
        inner_trivia: "",
        expected_fragment: "a\\u0062",
        expected_name: "ab",
        expected_provenance: ESCAPED,
    },
];

fn all_positive_rows() -> impl Iterator<Item = &'static PositiveRow> {
    CALLEE_MATRIX
        .iter()
        .chain(PRE_OPEN_TRIVIA_MATRIX)
        .chain(INNER_TRIVIA_MATRIX)
        .chain(SPECIAL_NAME_MATRIX)
}

fn assert_positive_row(id_base: u64, index: usize, row: &PositiveRow) {
    let id = id_base + index as u64;

    // Fixture self-consistency: literal ranges pin exact authored pieces.
    assert_eq!(slice(row.source, row.callee), row.expected_fragment);
    assert_eq!(slice(row.source, row.open), "(");
    assert_eq!(slice(row.source, row.close), ")");
    assert_eq!(
        slice(row.source, Range(row.callee.1, row.open.0)),
        row.pre_open_trivia
    );
    assert_eq!(
        slice(row.source, Range(row.open.1, row.close.0)),
        row.inner_trivia
    );
    assert_eq!(row.whole.0, row.callee.0);
    assert_eq!(row.whole.1, row.close.1);
    // The callee range never includes trivia or either delimiter.
    assert!(row.callee.1 <= row.open.0 && row.open.1 <= row.close.0);
    // Editing transport must not have decoded authored escapes away.
    if row.expected_provenance == ESCAPED {
        assert!(row.expected_fragment.contains('\\'), "{}", row.source);
        assert_ne!(row.expected_fragment, row.expected_name);
    } else {
        assert!(!row.expected_fragment.contains('\\'), "{}", row.source);
        assert_eq!(row.expected_fragment, row.expected_name);
    }

    // Oracle recognition of the whole bounded candidate.
    let whole_text = slice(row.source, row.whole);
    let evidence = matched_evidence(whole_text);
    let base = row.whole.0;
    assert_eq!(
        shift(evidence.callee.range, base),
        row.callee,
        "{whole_text:?}"
    );
    assert_eq!(shift(evidence.open, base), row.open, "{whole_text:?}");
    assert_eq!(shift(evidence.close, base), row.close, "{whole_text:?}");
    assert_eq!(evidence.callee.name, row.expected_name, "{whole_text:?}");
    assert_eq!(
        evidence.callee.provenance, row.expected_provenance,
        "{whole_text:?}"
    );
    assert_eq!(
        classify_frontier_outcome(whole_text),
        FrontierOutcome::SelectedAcceptedIncomplete
    );

    // Cross-check against the bare-atom authority: the callee fragment is
    // exactly one already-accepted IdentifierReference.
    assert_eq!(
        classify_selected_accepted_identifier_reference(row.expected_fragment),
        Some((row.expected_name.to_owned(), row.expected_provenance))
    );

    // Exact authored anchors: the callee anchor is the authored (possibly
    // escaped) spelling, never the call/trivia-wrapped spelling; the whole
    // anchor is the exact bounded call.
    assert_eq!(
        authored_anchor(849_000 + id, row.source, row.callee),
        row.expected_fragment
    );
    assert_eq!(
        authored_anchor(849_500 + id, row.source, row.whole),
        whole_text
    );
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 849);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_PARENTHESIZED_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 831"));
    assert!(PREVIOUS_DELETE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 839"));
    assert!(PREVIOUS_ESCAPED_IDENTIFIER_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 241"));
    assert!(
        FRONTIER_SCOPE_NOTE
            .contains("exactly-one zero-argument IdentifierReference CallExpression frontier only")
    );
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));
}

/// Required Direct and EscapedNonReserved callee matrix: exact authored
/// callee fragment, exact callee anchor, decoded semantic name, provenance,
/// delimiter ownership, and whole-candidate consumption.
#[test]
fn exact_callee_matrix_pins_fixture_owned_ranges_and_provenance() {
    assert_eq!(CALLEE_MATRIX.len(), 7);
    for (index, row) in CALLEE_MATRIX.iter().enumerate() {
        assert_positive_row(1_000, index, row);
        assert_eq!(row.pre_open_trivia, "");
        assert_eq!(row.inner_trivia, "");
    }
}

/// Selected trivia between callee and `(`. `a\n()` is one call.
#[test]
fn selected_trivia_between_callee_and_opener_is_call_continuation() {
    assert_eq!(PRE_OPEN_TRIVIA_MATRIX.len(), 5);
    for (index, row) in PRE_OPEN_TRIVIA_MATRIX.iter().enumerate() {
        assert_positive_row(2_000, index, row);
        assert!(!row.pre_open_trivia.is_empty());
    }

    // The LineTerminator row is a single call, not `a` + `();`.
    let source = "const x = a\n();";
    let evidence = matched_evidence(slice(source, Range(10, 14)));
    assert_eq!(evidence.callee.range, Range(0, 1));
    assert_eq!(evidence.open, Range(2, 3));
    assert_eq!(evidence.close, Range(3, 4));

    // The bare callee alone is the *predecessor* atom, not this theorem; the
    // ASI split reading would leave the unowned `()` fragment.
    assert_eq!(declined_reason("a"), DeclineReason::MissingCallOpener);
    assert_eq!(
        declined_reason("()"),
        DeclineReason::Callee(CalleeFailure::NoIdentifierStart)
    );
}

/// Selected trivia inside the empty `Arguments`, and both positions at once.
#[test]
fn selected_trivia_inside_empty_arguments_is_owned() {
    assert_eq!(INNER_TRIVIA_MATRIX.len(), 7);
    for (index, row) in INNER_TRIVIA_MATRIX.iter().enumerate() {
        assert_positive_row(3_000, index, row);
        assert!(!row.pre_open_trivia.is_empty() || !row.inner_trivia.is_empty());
    }
}

#[test]
fn special_callee_names_are_plain_identifier_references() {
    for (index, row) in SPECIAL_NAME_MATRIX.iter().enumerate() {
        assert_positive_row(4_000, index, row);
    }
}

/// Full selected-trivia sweep, zero-or-more, in both positions, plus the
/// separator look-alikes that the frozen contract does not own.
#[test]
fn selected_trivia_sweep_and_non_trivia_lookalikes() {
    for trivia in SELECTED_TRIVIA_SWEEP {
        assert!(is_selected_call_trivia(*trivia), "{trivia:?}");
        let width = trivia.len_utf8();
        for count in [1_usize, 2] {
            let run = trivia.to_string().repeat(count);
            let n = width * count;

            let before = format!("a{run}()");
            let evidence = matched_evidence(&before);
            assert_eq!(evidence.callee.range, Range(0, 1), "{before:?}");
            assert_eq!(evidence.open, Range(1 + n, 2 + n), "{before:?}");
            assert_eq!(evidence.close, Range(2 + n, 3 + n), "{before:?}");

            let inside = format!("a({run})");
            let evidence = matched_evidence(&inside);
            assert_eq!(evidence.open, Range(1, 2), "{inside:?}");
            assert_eq!(evidence.close, Range(2 + n, 3 + n), "{inside:?}");

            let both = format!("a{run}({run})");
            let evidence = matched_evidence(&both);
            assert_eq!(evidence.open, Range(1 + n, 2 + n), "{both:?}");
            assert_eq!(evidence.close, Range(2 + 2 * n, 3 + 2 * n), "{both:?}");
        }
    }

    for lookalike in NON_TRIVIA_SEPARATOR_LOOKALIKES {
        assert!(!is_selected_call_trivia(*lookalike), "{lookalike:?}");
        assert!(!is_direct_identifier_part(*lookalike), "{lookalike:?}");
        let before = format!("a{lookalike}()");
        assert_eq!(declined_reason(&before), DeclineReason::MissingCallOpener);
        let inside = format!("a({lookalike})");
        assert_eq!(
            declined_reason(&inside),
            DeclineReason::UnownedInsideArguments
        );
    }
}

/// The bounded candidate is exact: leading and trailing trivia belong to the
/// carrier, not to the call.
#[test]
fn candidate_boundary_is_exact_and_carrier_owns_outer_trivia() {
    assert_eq!(
        declined_reason(" f()"),
        DeclineReason::Callee(CalleeFailure::NoIdentifierStart)
    );
    assert_eq!(declined_reason("f() "), DeclineReason::TrailingContinuation);
    assert_eq!(
        declined_reason("f()\n"),
        DeclineReason::TrailingContinuation
    );
    assert_eq!(
        scan_zero_argument_call_prefix("f() ").map(|evidence| evidence.close.1),
        Ok(3)
    );
}

// ---------------------------------------------------------------------------
// B. Firewalls
// ---------------------------------------------------------------------------

/// Whether a source is valid ECMAScript is recorded independently of the
/// Oracle outcome, so "outside this theorem" is never read as `SyntaxError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceValidity {
    /// Valid Script source under the pinned grammar.
    ValidScriptSource,
    /// Grammatically an assignment whose `AssignmentTargetType` is the
    /// normative-optional `web-compat` (host-dependent runtime error) or
    /// `invalid` (early error), depending on the host.
    HostDependentAssignmentTarget,
    /// A pinned-grammar syntax error (incomplete or Script-illegal source).
    SyntaxError,
}

struct OutsideControl {
    source: &'static str,
    validity: SourceValidity,
    expected_reason: DeclineReason,
    /// End of the locally complete zero-argument call prefix, when one
    /// exists. A prefix is never a whole-candidate acceptance.
    prefix_end: Option<usize>,
}

const fn outside(
    source: &'static str,
    validity: SourceValidity,
    expected_reason: DeclineReason,
    prefix_end: Option<usize>,
) -> OutsideControl {
    OutsideControl {
        source,
        validity,
        expected_reason,
        prefix_end,
    }
}

const VALID: SourceValidity = SourceValidity::ValidScriptSource;
const UNOWNED_ARGS: DeclineReason = DeclineReason::UnownedInsideArguments;
const TRAILING: DeclineReason = DeclineReason::TrailingContinuation;
const NO_OPENER: DeclineReason = DeclineReason::MissingCallOpener;
const RESERVED_CALLEE: DeclineReason = DeclineReason::Callee(CalleeFailure::DirectReservedWord);

const NON_EMPTY_ARGUMENT_CONTROLS: &[OutsideControl] = &[
    outside("f(a)", VALID, UNOWNED_ARGS, None),
    outside("f(a,b)", VALID, UNOWNED_ARGS, None),
    outside("f(...a)", VALID, UNOWNED_ARGS, None),
    outside("f(a,)", VALID, UNOWNED_ARGS, None),
    outside("f( a )", VALID, UNOWNED_ARGS, None),
];

const RICHER_OUTER_CONTROLS: &[OutsideControl] = &[
    outside("f()()", VALID, TRAILING, Some(3)),
    outside("f().x", VALID, TRAILING, Some(3)),
    outside("f()[x]", VALID, TRAILING, Some(3)),
    outside("f()+g", VALID, TRAILING, Some(3)),
    outside("f()-g", VALID, TRAILING, Some(3)),
    outside("f()*g", VALID, TRAILING, Some(3)),
    outside("f()/g", VALID, TRAILING, Some(3)),
    outside("f()?g:h", VALID, TRAILING, Some(3)),
    outside(
        "f()=g",
        SourceValidity::HostDependentAssignmentTarget,
        TRAILING,
        Some(3),
    ),
];

const OTHER_CALL_FAMILY_CONTROLS: &[OutsideControl] = &[
    outside("obj.f()", VALID, NO_OPENER, None),
    outside("f?.()", VALID, NO_OPENER, None),
    outside("new f()", VALID, RESERVED_CALLEE, None),
    outside(
        "super()",
        SourceValidity::SyntaxError,
        RESERVED_CALLEE,
        None,
    ),
];

const COMMENT_CONTROLS: &[OutsideControl] = &[
    outside("f/*c*/()", VALID, NO_OPENER, None),
    outside("f(/*c*/)", VALID, UNOWNED_ARGS, None),
    outside("f( /*c*/ )", VALID, UNOWNED_ARGS, None),
];

const INCOMPLETE_SUFFIX_CONTROLS: &[OutsideControl] = &[
    outside(
        "f(",
        SourceValidity::SyntaxError,
        DeclineReason::IncompleteSuffix,
        None,
    ),
    outside(
        "f (",
        SourceValidity::SyntaxError,
        DeclineReason::IncompleteSuffix,
        None,
    ),
    outside(
        "f( ",
        SourceValidity::SyntaxError,
        DeclineReason::IncompleteSuffix,
        None,
    ),
    outside(
        "f(\n",
        SourceValidity::SyntaxError,
        DeclineReason::IncompleteSuffix,
        None,
    ),
];

/// `async` cover neighbors: the locally complete `async()` prefix exists, but
/// the `=>` continuation makes the whole an `AsyncArrowFunction`, which this
/// theorem never owns.
const ASYNC_ARROW_COVER_CONTROLS: &[OutsideControl] = &[
    outside("async()=>x", VALID, TRAILING, Some(7)),
    outside("async() => x", VALID, TRAILING, Some(7)),
    outside("async ( ) => x", VALID, TRAILING, Some(9)),
    outside("f()=>x", SourceValidity::SyntaxError, TRAILING, Some(3)),
];

fn assert_outside_controls(controls: &[OutsideControl]) {
    for control in controls {
        assert_eq!(
            declined_reason(control.source),
            control.expected_reason,
            "{:?}",
            control.source
        );
        assert_eq!(
            classify_frontier_outcome(control.source),
            FrontierOutcome::UnsupportedCoverage,
            "{:?}",
            control.source
        );
        assert_eq!(
            scan_zero_argument_call_prefix(control.source)
                .ok()
                .map(|evidence| evidence.close.1),
            control.prefix_end,
            "{:?}",
            control.source
        );
    }
}

/// Non-empty / richer arguments stay outside; several are valid ECMAScript,
/// and outside-the-theorem is never `SyntaxError`.
#[test]
fn non_empty_argument_firewall_is_not_syntax_rejection() {
    assert_outside_controls(NON_EMPTY_ARGUMENT_CONTROLS);
    for control in NON_EMPTY_ARGUMENT_CONTROLS {
        assert_eq!(control.validity, VALID, "{:?}", control.source);
    }
    // The empty pair is the only owned shape.
    assert!(matches!(recognize_call("f()"), CallRecognition::Matched(_)));
}

/// A locally complete `f()` prefix never authorizes a richer whole source:
/// only the prefix is recognizable, the whole candidate is declined, and no
/// call fact is committed for it.
#[test]
fn richer_outer_continuation_never_promotes_a_local_prefix() {
    assert_outside_controls(RICHER_OUTER_CONTROLS);
    for control in RICHER_OUTER_CONTROLS {
        let prefix = scan_zero_argument_call_prefix(control.source)
            .unwrap_or_else(|reason| panic!("{:?}: {reason:?}", control.source));
        assert_eq!(prefix.callee.name, "f");
        assert_eq!(prefix.close.1, 3);
    }
    // The bare prefix itself remains independently valid.
    assert!(matches!(recognize_call("f()"), CallRecognition::Matched(_)));
    // Validity is recorded separately from the (identical) outcome.
    assert!(
        RICHER_OUTER_CONTROLS
            .iter()
            .any(|control| control.validity == SourceValidity::HostDependentAssignmentTarget)
    );
}

#[test]
fn other_call_families_remain_unowned() {
    assert_outside_controls(OTHER_CALL_FAMILY_CONTROLS);
}

#[test]
fn comments_are_not_selected_trivia() {
    assert_outside_controls(COMMENT_CONTROLS);
    for comment in ["/*c*/", "//c\n"] {
        for candidate in [
            format!("f{comment}()"),
            format!("f({comment})"),
            format!("f{comment}({comment})"),
        ] {
            assert_eq!(
                classify_frontier_outcome(&candidate),
                FrontierOutcome::UnsupportedCoverage,
                "{candidate:?}"
            );
        }
    }
}

/// A failed call candidate never escapes as a successful call fact: a
/// recognized callee prefix without the paired `)` is an ordinary decline.
#[test]
fn incomplete_suffix_rolls_back_without_committing_a_call_fact() {
    assert_outside_controls(INCOMPLETE_SUFFIX_CONTROLS);
    for control in INCOMPLETE_SUFFIX_CONTROLS {
        assert!(scan_zero_argument_call_prefix(control.source).is_err());
        assert_eq!(control.validity, SourceValidity::SyntaxError);
        // The callee alone is still a valid bare atom under its own authority.
        assert!(classify_selected_accepted_identifier_reference("f").is_some());
        assert!(matches!(
            recognize_call(control.source),
            CallRecognition::Declined(DeclineReason::IncompleteSuffix)
        ));
    }
}

/// Cover-grammar / AsyncArrowFunction firewall: the cover-neighbor forms stay
/// outside, while plain `async()` (no `=>`) is an ordinary call and reaches no
/// `EE-35` identity.
#[test]
fn async_arrow_cover_forms_remain_outside_and_plain_calls_do_not_reach_ee35() {
    assert_outside_controls(ASYNC_ARROW_COVER_CONTROLS);
    for control in ASYNC_ARROW_COVER_CONTROLS {
        assert!(control.source.contains("=>"));
    }
    for callee in ["a", "async", "f"] {
        let plain = format!("{callee}()");
        assert!(matches!(
            recognize_call(&plain),
            CallRecognition::Matched(_)
        ));
        assert!(!plain.contains("=>"));
    }
    assert!(INVENTORY_SOURCE.contains(
        "AsyncArrowFunction : CoverCallExpressionAndAsyncArrowHead => AsyncConciseBody / cover must be AsyncArrowHead"
    ));
}

/// Predecessor distinction: the bare atoms belong to the existing
/// `IdentifierReference` authority and never to this call theorem; the call
/// forms belong to this theorem with the same callee fact.
#[test]
fn bare_predecessor_atoms_remain_distinct_from_the_call_theorem() {
    for (bare, call, name, provenance) in [
        ("a", "a()", "a", DIRECT),
        ("\\u0061", "\\u0061()", "a", ESCAPED),
    ] {
        assert_eq!(
            classify_selected_accepted_identifier_reference(bare),
            Some((name.to_owned(), provenance))
        );
        assert_eq!(declined_reason(bare), DeclineReason::MissingCallOpener);
        assert_eq!(
            classify_frontier_outcome(bare),
            FrontierOutcome::UnsupportedCoverage
        );

        let evidence = matched_evidence(call);
        assert_eq!(evidence.callee.range, Range(0, bare.len()));
        assert_eq!(slice(call, evidence.callee.range), bare);
        assert_eq!(
            (evidence.callee.name.as_str(), evidence.callee.provenance),
            (name, provenance)
        );
        // The call spelling is never itself an atom.
        assert_eq!(classify_selected_accepted_identifier_reference(call), None);
    }
}

/// Existing `IdentifierReference` policy boundaries stay callee-atom failures
/// with their original distinct classes; none becomes call-specific.
#[test]
fn identifier_reference_policy_boundaries_are_not_reinterpreted() {
    let cases: &[(&str, DeclineReason)] = &[
        // Escaped ReservedWord.
        (
            "\\u0069f()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::DecodedReserved)),
        ),
        (
            r"\u{69}f()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::DecodedReserved)),
        ),
        (
            "\\u0074his()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::DecodedReserved)),
        ),
        // Malformed escapes.
        (
            "\\u006()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::MalformedEscape)),
        ),
        (
            "\\u0G61()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::MalformedEscape)),
        ),
        (
            r"\u{}()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::MalformedEscape)),
        ),
        (
            r"\u{G}()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::MalformedEscape)),
        ),
        (
            r"\x61()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::MalformedEscape)),
        ),
        // Non-CodePoint escape.
        (
            r"\u{110000}()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::NonCodePoint)),
        ),
        // Invalid IdentifierStart / IdentifierPart, escaped and direct.
        (
            "\\u0030()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::InvalidStart)),
        ),
        (
            "1a()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::InvalidStart)),
        ),
        (
            "a\\u0020()",
            DeclineReason::Callee(CalleeFailure::Decode(DecodeFailure::InvalidPart)),
        ),
        // Direct ReservedWords.
        ("if()", RESERVED_CALLEE),
        ("this()", RESERVED_CALLEE),
        ("true()", RESERVED_CALLEE),
        ("null()", RESERVED_CALLEE),
        ("typeof()", RESERVED_CALLEE),
        ("void()", RESERVED_CALLEE),
        ("delete()", RESERVED_CALLEE),
        // No callee at all.
        (
            "()",
            DeclineReason::Callee(CalleeFailure::NoIdentifierStart),
        ),
        (
            "+a()",
            DeclineReason::Callee(CalleeFailure::NoIdentifierStart),
        ),
        (
            ".a()",
            DeclineReason::Callee(CalleeFailure::NoIdentifierStart),
        ),
        // Maximal-IdentifierName boundaries: the callee is never split, and a
        // separator that is not trivia-then-`(` leaves the call unowned.
        ("a b()", NO_OPENER),
        ("a\u{00A0}b()", NO_OPENER),
        ("a.b()", NO_OPENER),
    ];
    for (candidate, expected) in cases {
        assert_eq!(declined_reason(candidate), *expected, "{candidate:?}");
        assert_eq!(
            classify_frontier_outcome(candidate),
            FrontierOutcome::UnsupportedCoverage,
            "{candidate:?}"
        );
    }

    // Reason classes are distinct and never collapsed into one decline.
    let distinct: std::collections::BTreeSet<String> = cases
        .iter()
        .map(|(candidate, _)| format!("{:?}", declined_reason(candidate)))
        .collect();
    assert!(distinct.len() >= 8, "{distinct:?}");

    // Accepted policy composes through the call unchanged.
    for (candidate, name, provenance) in [
        ("ab()", "ab", DIRECT),
        ("$()", "$", DIRECT),
        ("_()", "_", DIRECT),
        ("\\u0024()", "$", ESCAPED),
        ("\u{03C0}\\u{03C0}()", "\u{03C0}\u{03C0}", ESCAPED),
    ] {
        let evidence = matched_evidence(candidate);
        assert_eq!(evidence.callee.name, name, "{candidate:?}");
        assert_eq!(evidence.callee.provenance, provenance, "{candidate:?}");
    }
}

/// Runtime / direct-eval firewall: `eval()` yields only the authored callee
/// fact and the bounded call-source shape.
#[test]
fn eval_callee_is_source_syntax_only() {
    let evidence = matched_evidence("eval()");
    assert_eq!(evidence.callee.range, Range(0, 4));
    assert_eq!(slice("eval()", evidence.callee.range), "eval");
    assert_eq!(evidence.callee.name, "eval");
    assert_eq!(evidence.callee.provenance, DIRECT);

    // An escaped spelling of `eval` is the same semantic name but keeps its
    // authored escaped fragment and EscapedNonReserved provenance.
    let escaped = matched_evidence("\\u0065val()");
    assert_eq!(escaped.callee.name, "eval");
    assert_eq!(escaped.callee.provenance, ESCAPED);
    assert_eq!(slice("\\u0065val()", escaped.callee.range), "\\u0065val");
}

// ---------------------------------------------------------------------------
// C. Early Error reachability model
// ---------------------------------------------------------------------------

/// Source syntax an Early Error identity's trigger requires, at the
/// granularity needed to decide whether `IdentifierReference` + empty
/// `Arguments` can supply it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyntaxElement {
    // Elements this theorem admits.
    CalleeIdentifierReference,
    EmptyArguments,
    SelectedTrivia,
    // Elements it does not.
    BindingIdentifier,
    LegacyNumericLiteral,
    LegacyStringEscape,
    ObjectLiteral,
    RegExpLiteral,
    TemplateLiteral,
    ParenthesizedGrouping,
    OptionalChain,
    ImportMeta,
    UpdateOperator,
    DeleteOperator,
    AssignmentOperator,
    DestructuringPattern,
    ControlStatement,
    ParameterList,
    FunctionOrClassBoundary,
    ArrowToken,
    SuperKeyword,
    NewTargetMeta,
    PrivateName,
    LabelledStatement,
    BreakOrContinueStatement,
    RegExpPatternSyntax,
}

const ADMITTED_SYNTAX: &[SyntaxElement] = &[
    SyntaxElement::CalleeIdentifierReference,
    SyntaxElement::EmptyArguments,
    SyntaxElement::SelectedTrivia,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reachability {
    /// The trigger is the callee atom itself, already owned by the accepted
    /// bare `IdentifierReference` authority whether or not a call follows.
    CalleeAtomAlreadyOwned,
    /// The trigger is carrier structure (declaration/statement lists) that is
    /// independent of the initializer expression.
    CarrierOwned,
    /// The trigger needs syntax outside the admitted set.
    RequiresAbsent(SyntaxElement),
}

/// Fixture-authored default reachability per frozen Early Error container.
const CONTAINER_REACHABILITY: &[(&str, Reachability)] = &[
    ("EE-01", Reachability::CalleeAtomAlreadyOwned),
    (
        "EE-02",
        Reachability::RequiresAbsent(SyntaxElement::LegacyNumericLiteral),
    ),
    (
        "EE-03",
        Reachability::RequiresAbsent(SyntaxElement::LegacyStringEscape),
    ),
    (
        "EE-04",
        Reachability::RequiresAbsent(SyntaxElement::BindingIdentifier),
    ),
    (
        "EE-05",
        Reachability::RequiresAbsent(SyntaxElement::ObjectLiteral),
    ),
    (
        "EE-06",
        Reachability::RequiresAbsent(SyntaxElement::RegExpLiteral),
    ),
    (
        "EE-07",
        Reachability::RequiresAbsent(SyntaxElement::TemplateLiteral),
    ),
    (
        "EE-08",
        Reachability::RequiresAbsent(SyntaxElement::ParenthesizedGrouping),
    ),
    (
        "EE-09",
        Reachability::RequiresAbsent(SyntaxElement::OptionalChain),
    ),
    (
        "EE-10",
        Reachability::RequiresAbsent(SyntaxElement::UpdateOperator),
    ),
    (
        "EE-11",
        Reachability::RequiresAbsent(SyntaxElement::DeleteOperator),
    ),
    (
        "EE-12",
        Reachability::RequiresAbsent(SyntaxElement::AssignmentOperator),
    ),
    (
        "EE-13",
        Reachability::RequiresAbsent(SyntaxElement::DestructuringPattern),
    ),
    ("EE-14", Reachability::CarrierOwned),
    ("EE-15", Reachability::CarrierOwned),
    (
        "EE-16",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-17",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-18",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-19",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-20",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-21",
        Reachability::RequiresAbsent(SyntaxElement::BreakOrContinueStatement),
    ),
    (
        "EE-22",
        Reachability::RequiresAbsent(SyntaxElement::BreakOrContinueStatement),
    ),
    (
        "EE-23",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-24",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-25",
        Reachability::RequiresAbsent(SyntaxElement::LabelledStatement),
    ),
    (
        "EE-26",
        Reachability::RequiresAbsent(SyntaxElement::ControlStatement),
    ),
    (
        "EE-27",
        Reachability::RequiresAbsent(SyntaxElement::ParameterList),
    ),
    (
        "EE-28",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-29",
        Reachability::RequiresAbsent(SyntaxElement::ArrowToken),
    ),
    (
        "EE-30",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-31",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-32",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-33",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-34",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-35",
        Reachability::RequiresAbsent(SyntaxElement::ArrowToken),
    ),
    ("EE-36", Reachability::CarrierOwned),
    (
        "EE-37",
        Reachability::RequiresAbsent(SyntaxElement::RegExpPatternSyntax),
    ),
];

/// Per-rule refinements of the container defaults.
const RULE_REACHABILITY_OVERRIDES: &[(&str, Reachability)] = &[
    // Callee-atom identities: identical trigger with or without a call.
    ("EE-04-R02", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-R05", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-R06", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-R07", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-R08", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-I01", Reachability::CalleeAtomAlreadyOwned),
    ("EE-04-I02", Reachability::CalleeAtomAlreadyOwned),
    // Same container, different trigger syntax.
    (
        "EE-09-R02",
        Reachability::RequiresAbsent(SyntaxElement::ImportMeta),
    ),
    (
        "EE-33-R10",
        Reachability::RequiresAbsent(SyntaxElement::FunctionOrClassBoundary),
    ),
    (
        "EE-36-R03",
        Reachability::RequiresAbsent(SyntaxElement::SuperKeyword),
    ),
    (
        "EE-36-R04",
        Reachability::RequiresAbsent(SyntaxElement::NewTargetMeta),
    ),
    (
        "EE-36-R05",
        Reachability::RequiresAbsent(SyntaxElement::LabelledStatement),
    ),
    (
        "EE-36-R06",
        Reachability::RequiresAbsent(SyntaxElement::BreakOrContinueStatement),
    ),
    (
        "EE-36-R07",
        Reachability::RequiresAbsent(SyntaxElement::BreakOrContinueStatement),
    ),
    (
        "EE-36-R08",
        Reachability::RequiresAbsent(SyntaxElement::PrivateName),
    ),
];

/// Locator vocabulary that marks a frozen identity as call/cover/super/async
/// neighborhood, independently of the reachability table above.
const CALL_NEIGHBORHOOD_VOCABULARY: &[&str] = &[
    "CallExpression",
    "SuperCall",
    "SuperProperty",
    "HasDirectSuper",
    "Contains super",
    "Contains NewTarget",
    "ContainsArguments",
    "AsyncArrow",
    "AssignmentTargetType",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct InventoryRule {
    id: String,
    container: String,
    active: bool,
    locator: String,
}

/// Reads the frozen inventory *source text* (never the production or
/// completion code) into rule identities.
fn parse_inventory_rules() -> Vec<InventoryRule> {
    let lines: Vec<&str> = INVENTORY_SOURCE.lines().map(str::trim).collect();
    let unquote = |line: &str| line.trim_end_matches(',').trim_matches('"').to_owned();
    let mut rules = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let active = match *line {
            "active_rule(" => true,
            "inactive_rule(" => false,
            _ => continue,
        };
        rules.push(InventoryRule {
            id: unquote(lines[index + 1]),
            container: unquote(lines[index + 2]),
            active,
            locator: unquote(lines[index + 3]),
        });
    }
    rules
}

fn reachability_of(rule: &InventoryRule) -> Reachability {
    if let Some(index) = RULE_REACHABILITY_OVERRIDES
        .iter()
        .position(|(id, _)| *id == rule.id)
    {
        return RULE_REACHABILITY_OVERRIDES[index].1;
    }
    let index = CONTAINER_REACHABILITY
        .iter()
        .position(|(container, _)| *container == rule.container)
        .unwrap_or_else(|| panic!("unaudited container for {rule:?}"));
    CONTAINER_REACHABILITY[index].1
}

fn newly_reachable(rules: &[InventoryRule], admitted: &[SyntaxElement]) -> Vec<String> {
    rules
        .iter()
        .filter(|rule| match reachability_of(rule) {
            Reachability::RequiresAbsent(element) => admitted.contains(&element),
            Reachability::CalleeAtomAlreadyOwned | Reachability::CarrierOwned => false,
        })
        .map(|rule| rule.id.clone())
        .collect()
}

/// Local Early Error question: does selecting exactly `IdentifierReference` +
/// empty `Arguments` make any frozen identity newly required/reachable?
/// Falsifiable: the same model flips identities when a trigger element is
/// admitted (controls below).
#[test]
fn no_frozen_early_error_identity_becomes_newly_reachable() {
    let rules = parse_inventory_rules();

    // The frozen inventory shape is read, not assumed.
    assert_eq!(rules.len(), 193);
    assert_eq!(rules.iter().filter(|rule| rule.active).count(), 183);
    assert_eq!(rules.iter().filter(|rule| !rule.active).count(), 10);
    let mut ids: Vec<&str> = rules.iter().map(|rule| rule.id.as_str()).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 193);
    let mut containers: Vec<&str> = rules.iter().map(|rule| rule.container.as_str()).collect();
    containers.sort_unstable();
    containers.dedup();
    assert_eq!(containers.len(), 37);
    assert_eq!(CONTAINER_REACHABILITY.len(), 37);
    for (container, _) in CONTAINER_REACHABILITY {
        assert!(containers.contains(container), "{container}");
    }
    for (id, _) in RULE_REACHABILITY_OVERRIDES {
        assert!(ids.contains(id), "stale override {id}");
    }
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(seen.len(), 193);"));
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(active, 183);"));
    assert!(COMPLETION_AUTHORITY_SOURCE.contains("assert_eq!(inactive, 10);"));

    // No admitted trigger syntax is required by any identity as "absent".
    assert!(newly_reachable(&rules, ADMITTED_SYNTAX).is_empty());

    // Every identity in the call/cover/super/async neighborhood is classified
    // as requiring absent syntax (never silently owned), and the named
    // identities the Issue calls out are among them.
    let neighborhood: Vec<&InventoryRule> = rules
        .iter()
        .filter(|rule| {
            CALL_NEIGHBORHOOD_VOCABULARY
                .iter()
                .any(|term| rule.locator.contains(term))
        })
        .collect();
    for rule in &neighborhood {
        assert!(
            matches!(reachability_of(rule), Reachability::RequiresAbsent(_)),
            "{rule:?}"
        );
    }
    for required in [
        "EE-35-R02",
        "EE-33-R10",
        "EE-36-R03",
        "EE-12-R03",
        "EE-10-R01",
    ] {
        assert!(
            neighborhood.iter().any(|rule| rule.id == required),
            "{required}"
        );
    }
    // The only container that owns CallExpression-adjacent syntax directly
    // (Left-Hand-Side Expressions, EE-09) requires `?.` templates or
    // `import.meta`, neither of which is admitted.
    for rule in rules.iter().filter(|rule| rule.container == "EE-09") {
        assert!(matches!(
            reachability_of(rule),
            Reachability::RequiresAbsent(SyntaxElement::OptionalChain | SyntaxElement::ImportMeta)
        ));
    }
    // Callee-atom identities are attributed to the atom, not to the call.
    for id in ["EE-01-R01", "EE-01-R02", "EE-04-R08"] {
        let index = rules.iter().position(|rule| rule.id == id).expect(id);
        let rule = &rules[index];
        assert_eq!(reachability_of(rule), Reachability::CalleeAtomAlreadyOwned);
    }
}

/// Model teeth: admitting the corresponding trigger syntax flips exactly the
/// identities that need it, so the empty result above is a finding, not a
/// tautology.
#[test]
fn reachability_model_flips_when_a_trigger_element_is_admitted() {
    let rules = parse_inventory_rules();
    let with = |element: SyntaxElement| {
        let mut admitted = ADMITTED_SYNTAX.to_vec();
        admitted.push(element);
        newly_reachable(&rules, &admitted)
    };

    let arrow = with(SyntaxElement::ArrowToken);
    for id in ["EE-35-R02", "EE-29-R05"] {
        assert!(arrow.iter().any(|flipped| flipped == id), "{id}");
    }
    assert!(with(SyntaxElement::AssignmentOperator).contains(&"EE-12-R03".to_owned()));
    assert!(with(SyntaxElement::UpdateOperator).contains(&"EE-10-R01".to_owned()));
    assert!(with(SyntaxElement::SuperKeyword).contains(&"EE-36-R03".to_owned()));
    assert!(with(SyntaxElement::DeleteOperator).contains(&"EE-11-R01".to_owned()));
    assert!(with(SyntaxElement::OptionalChain).contains(&"EE-09-R01".to_owned()));
    assert!(with(SyntaxElement::ImportMeta).contains(&"EE-09-R02".to_owned()));
}

// ---------------------------------------------------------------------------
// D. Independence and handoff
// ---------------------------------------------------------------------------

/// Oracle meta-independence firewall: rejects imports/calls into production
/// lexical, static-semantics, Binding/Scope, correspondence, or aggregate
/// qualification code, runtime-evaluation vocabulary, and the forbidden
/// substring-search patterns that would let expected ranges be derived
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
        concat!("Is", "Callable("),
        concat!("Perform", "Eval("),
        concat!("Evaluate", "Call("),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(THIS_ORACLE_SOURCE.contains(concat!(
        "use super::",
        "unicode::{is_id_continue, is_id_start, is_space_separator}"
    )));
}

/// Freezes the presence-only, unqualified, validation-only handoff: this leaf
/// composes to `SelectedAcceptedIncomplete` / `UnsupportedCoverage`;
/// `ResourceLimited` / `InternalFailure` remain a distinct processing-failure
/// identity that this total recognizer never produces; and no retained
/// production representation, call anchor, runtime, or general-parser
/// vocabulary is introduced.
#[test]
fn handoff_remains_presence_only_unqualified_and_validation_only() {
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
    assert!(!PROCESSING_FAILURES.contains(&"UnsupportedCoverage"));
    assert!(!PROCESSING_FAILURES.contains(&"SelectedAcceptedIncomplete"));
    for candidate in ["a()", "f(", "f()()", "\\u{}()", "async()=>x", ""] {
        let outcome = format!("{:?}", classify_frontier_outcome(candidate));
        assert!(
            !PROCESSING_FAILURES.contains(&outcome.as_str()),
            "{candidate:?}"
        );
    }

    for forbidden in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("struct Call", "SourceAnchor"),
        concat!("struct Call", "Node"),
        concat!("struct Arguments", "Node"),
        concat!("struct Call", "Fact"),
        concat!("enum Call", "Expression"),
        concat!("enum Left", "HandSideExpression"),
        concat!("enum Member", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("enum Primary", "Expression"),
        concat!("struct Async", "ArrowFunction"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("struct Reference", "Record"),
        concat!("struct Environment", "Record"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}

/// Every positive fixture is reached by exactly one matrix, and all
/// positives pass the same whole-row assertions (guards against a matrix
/// being silently emptied or duplicated).
#[test]
fn positive_fixture_inventory_is_complete_and_unique() {
    let sources: Vec<&str> = all_positive_rows().map(|row| row.source).collect();
    let mut unique = sources.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(sources.len(), unique.len());
    assert_eq!(sources.len(), 7 + 5 + 7 + 7);
    for required in [
        "a()",
        "\u{03C0}()",
        "\u{1D49C}()",
        "\\u0061()",
        "f\\u006Fo()",
        r"\u{66}oo()",
        r"\u{1D49C}()",
        "a ()",
        "a\t()",
        "a\n()",
        "a\u{00A0}()",
        "a( )",
        "a(\t)",
        "a(\n)",
        "a(\u{00A0})",
        "a ( )",
        "eval()",
    ] {
        assert!(
            all_positive_rows().any(|row| slice(row.source, row.whole) == required),
            "{required:?}"
        );
    }
}
