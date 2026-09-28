//! Candidate-independent bounded `delete` `IdentifierReference`
//! `UnaryExpression` fact-preservation and EE-11 non-triggering validation
//! for Issue #839 (durable research: Issue #688 comment `5857601911`
//! zero-base bounded `delete` frontier selection, Issue #688 comment
//! `5857970069` strictness/EE-11-attribution falsification, Issue #688
//! comment `5858046424` representation/ambient-strictness/Oracle-architecture
//! falsification; structural precedent: Issue #827 exactly-one `!`/`~`
//! `IdentifierReference` `UnaryExpression` Oracle, Issue #831 exactly-one
//! grouping-layer `ParenthesizedExpression` `IdentifierReference` Oracle;
//! accepted premise authority: Issue #237 direct escape-free
//! `IdentifierReference` source/name boundary, Issue #241 escaped
//! `IdentifierReference` initializer Oracle).
//!
//! This oracle qualifies only the bounded expression-composition family:
//!
//! ```text
//! SelectedDeleteIdentifierReferenceUnaryExpression ::=
//!     "delete"
//!     SelectedRequiredDeleteOperandTrivia
//!     SelectedAcceptedIdentifierReference
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
//! Unlike Issue #827/#831, this frontier crosses the frozen `EE-11` Early
//! Error container for the first time, so this Oracle carries a second
//! bounded responsibility beyond source composition: independently modeling
//! -- as fixture/context authority, never by reparsing or a Directive
//! Prologue parser -- that every candidate this theorem admits leaves both
//! `EE-11-R01` and `EE-11-R02` structurally non-triggering under the current
//! selected non-strict envelope, while a `StrictControl` fixture context
//! (backed by the pinned normative text and pinned Test262 challenge cases,
//! never by production strict-mode detection) does trigger them. See "B. EE-11
//! context model" below.
//!
//! ## A. Source-composition responsibility
//!
//! `SelectedRequiredDeleteOperandTrivia` independently restates the complete
//! already-accepted selected-slice trivia contract established by Issue
//! #742/#743 and reused by Issue #746/#827/#831 (the same code-point set
//! `is_selected_trivia` in `selected_lexical_slice.rs` recognizes, restated
//! here rather than imported): `TAB`, `VT`, `FF`, `BOM`, `LF`, `CR`, `LINE
//! SEPARATOR`, `PARAGRAPH SEPARATOR`, and the frozen Unicode 17
//! `Space_Separator` property. Comments remain outside this contract. Unlike
//! `!`/`~`, `delete` is a word-like keyword token, so this Oracle additionally
//! proves *direct keyword ownership* under the already-settled
//! maximal-`IdentifierName` principle: if the code point (direct or escaped)
//! immediately following the six-byte ASCII `"delete"` prefix would itself
//! continue an `IdentifierName`, the whole candidate remains one longer
//! `IdentifierReference` (`deletea`, `deleteπ`, `delete\u0061`,
//! `delete\u{61}`) and is never split into keyword + operand. An escaped
//! keyword spelling (`\u0064elete a`) is never a direct `delete` token: the
//! ASCII-literal-prefix test at the very start of recognition excludes it
//! structurally, without a second production keyword scanner. Because the
//! bounded operand is exactly one `IdentifierReference`, selected trivia
//! between keyword and operand must be non-empty -- this is a property of
//! *this* bounded theorem's operand shape, not a general `delete` lexical
//! requirement (general ECMAScript already accepts `delete(a)`/`delete!a`
//! without a separator; both remain outside this theorem, not syntax errors).
//!
//! `SelectedDirectIdentifierReference` independently restates only the
//! already-accepted Issue #237 direct escape-free `IdentifierName` code-point
//! shape. `SelectedEscapedNonReservedIdentifierReference` independently
//! restates both the fixed-form `\uXXXX` and braced `\u{HexDigits}`
//! `UnicodeEscapeSequence` forms of the already-accepted Issue #241 escaped
//! `IdentifierReference` decode/position/ReservedWord boundary. Issue #241
//! owns the escaped-`IdentifierReference` premise in full; this Oracle
//! independently composes representative accepted operands through the new
//! exactly-one `delete` wrapper without re-proving the entire Issue #241
//! theorem, and without introducing a second Unicode escape decoder.
//!
//! ## B. EE-11 context model
//!
//! A small validation-only context vocabulary -- never a second ECMAScript
//! parser, never Directive Prologue parsing, never runtime `Reference`
//! semantics -- models exactly the semantic interaction axis this frontier
//! newly crosses:
//!
//! ```text
//! DeleteContext:
//!     NonStrictSelectedScript
//!     StrictControl
//!
//! DeleteOperandClass:
//!     IdentifierReference
//!     ParenthesizedIdentifierReferenceControl
//!     PrivateReferenceControl
//!     OutsideSelectedTheorem
//! ```
//!
//! `NonStrictSelectedScript` is independent fixture/context authority, not a
//! reparse: the load-bearing premise (falsification result 1 of Issue #688
//! comment `5857970069`) is not merely `ParseGoal = Script`, but that the
//! current selected envelope structurally cannot establish strict mode --
//! current selected Script carriers admit no free-standing `StringLiteral`
//! `ExpressionStatement` (escape-free `StringLiteral` is currently accepted
//! only as an initializer atom, never a statement, so it can never form a
//! Directive Prologue), no Module goal, no Function/Class owner, and no
//! direct-eval context. `StrictControl` is an explicit fixture-authored
//! control context, backed by the pinned ECMA-262 Directive Prologue /
//! `IsStrict` semantics and the pinned Test262 `identifier-strict.js` /
//! `identifier-strict-recursive.js` challenge cases (`onlyStrict`,
//! `delete test262identifier;` and `delete ((identifier));` respectively),
//! never by scanning source for a `"use strict"` directive.
//!
//! `IdentifierReference` and `ParenthesizedIdentifierReferenceControl` model,
//! respectively, the direct `EE-11-R01` operand shape and the recursive
//! `EE-11-R02` propagation through
//! `CoverParenthesizedExpressionAndArrowParameterList` (falsification result 3
//! of comment `5857970069`: grouped `delete` is not unconditionally invalid;
//! the recursive rule matters only when the covered operand ultimately
//! reaches a forbidden strict `IdentifierReference`). Neither Direct nor
//! EscapedNonReserved provenance changes `EE-11-R01` applicability once both
//! form an `IdentifierReference` (falsification result 2): `delete a` and
//! `delete \u0061` are equivalent `EE-11-R01` controls. Grouping itself
//! remains entirely outside this bounded theorem's admitted operand shape --
//! `ParenthesizedIdentifierReferenceControl` fixtures exist only to prove the
//! semantic distinction, never as an admitted `delete` operand.
//! `PrivateReferenceControl` is structurally outside this first selected
//! theorem: no private-name/Class frontier is authorized, and this Oracle
//! proves structural exclusion rather than implementing private-name
//! semantics. `OutsideSelectedTheorem` is the broader-valid-but-outside
//! classification (`delete(a)`, `delete!a`, `delete typeof a`, ...): these
//! remain valid broader ECMAScript that this bounded Oracle simply declines,
//! never globally invalid syntax.
//!
//! ## Representation hypothesis and profile firewall
//!
//! The smallest viable retained evidence for a later, separately authorized
//! production decision remains the existing inner `SelectedIdentifierReferenceFact`
//! only -- never `delete` operator identity, never a whole `UnaryExpression`
//! `SourceAnchor`, never runtime `delete` state. This survives the Issue
//! #688 comment `5858046424` representation and ambient-strictness
//! falsification: dropping `delete` operator identity now does not force a
//! later rescan, because a Script's Directive Prologue is bounded by its
//! *initial* sequence of `StringLiteral` `ExpressionStatement`s -- a later
//! `"use strict"` can never retroactively strictify an earlier already-erased
//! `delete` occurrence. This hypothesis is explicitly profile-scoped: it is
//! authority *only* for the current non-strict selected Script carrier, never
//! for a future strict Script, Module, Class, strict Function, or strict
//! direct-eval `delete` frontier, each of which must reopen
//! representation/static-semantics design before reusing this erased
//! representation.
//!
//! ## Completion hypothesis
//!
//! This leaf adds zero production capability (validation-only, matching the
//! precedent set by #746/#827/#831, none of which shipped a completion
//! successor for the same reason). If every admitted candidate independently
//! proves `IsStrict == false`, operand shape exactly `IdentifierReference`,
//! no private target, and no `CoverParenthesizedExpressionAndArrowParameterList`
//! operand, then `EE-11-R01`/`EE-11-R02` remain non-triggering for this
//! bounded theorem and the frozen `193 / 10 / 183 / {}` completion partition
//! is unchanged. Historical completion authority is not modified by this
//! Oracle.

use crate::{SourceId, SourceText};

use super::qualification_validation_tests::gold_source;
use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 839;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const TEST262_SNAPSHOT: &str = "3655e7464de3d52643ecddd4b5f9f4f3e7f62398";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_BANG_TILDE_UNARY_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_bang_tilde_identifier_reference_unary_expression_initializer_validation_tests.rs"
);
const PREVIOUS_PARENTHESIZED_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_parenthesized_identifier_reference_validation_tests.rs");
const PREVIOUS_ESCAPED_IDENTIFIER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_escaped_identifier_reference_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_delete_identifier_reference_unary_expression_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "exactly-one direct delete keyword IdentifierReference UnaryExpression frontier only; ",
    "later independently qualified owners may strengthen classification for ",
    "recursive/mixed unary, other unary operators, non-IdentifierReference ",
    "operands, parenthesized/private-reference operands, richer expression ",
    "tails, comments, strict Script/Module/Class/Function/direct-eval ",
    "contexts, or top-level/Block var placement"
);
const PROFILE_FIREWALL_NOTE: &str = concat!(
    "this non-strict representation and EE-11-non-triggering theorem is ",
    "authority only for the current non-strict selected Script carrier; it ",
    "is not reusable authority for strict Script, Module, Class, strict ",
    "Function, strict direct-eval, or any other strict-capable delete analysis"
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontierOutcome {
    SelectedAcceptedIncomplete,
    /// Broader-valid-but-outside-this-theorem ECMAScript (the Issue's
    /// `OutsideSelectedTheorem`), and malformed/incomplete lower-layer
    /// operand evidence that must never be upgraded into a stronger
    /// rejection category merely because it follows `delete`.
    UnsupportedCoverage,
    StaticSemanticsRejected,
    SyntaxRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedLexicalBindingOrder {
    Before,
    Same,
    After,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectedBindingScopeTarget {
    SameSourceSelectedLexicalBinding(ExpectedLexicalBindingOrder),
    NoSameSourceSelectedLexicalBinding,
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
/// an `IdentifierReference`. Never a `BindingIdentifier` policy, an escaped
/// form, or a broader `PrimaryExpression`.
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
/// its code point and the byte offset immediately after it. Both forms are
/// part of the single already-accepted Issue #241 escaped-`IdentifierReference`
/// premise this Oracle composes, never re-derives, through the new
/// exactly-one `delete` wrapper.
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
/// keyword/trivia-prefixed spelling.
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

/// Independently restates the complete already-accepted
/// `SelectedRequiredDeleteOperandTrivia` code-point set established by Issue
/// #742/#743 and reused by Issue #746/#827/#831: the frozen
/// `is_selected_trivia` code-point set plus the frozen Unicode 17
/// `Space_Separator` property. Not a call into the production
/// selected-trivia recognizer.
fn is_selected_delete_operand_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

/// Returns whether decoding exactly one identifier element (a direct code
/// point, or a `\uXXXX`/`\u{HexDigits}` escape) starting at byte offset `0`
/// of `text` yields a valid `IdentifierPart` continuation code point. Used
/// only to prove maximal-munch ownership of the direct `"delete"` prefix
/// (Issue #839 section 11): it is never a second decoder for operand
/// content, which is always independently decoded afterward by the single
/// already-accepted `decode_selected_escaped_identifier` /
/// `is_escape_free_identifier_name` route.
fn continues_as_identifier_part(text: &str) -> bool {
    let Some(first) = text.chars().next() else {
        return false;
    };
    if first != '\\' {
        return is_direct_identifier_part(first);
    }
    match decode_unicode_escape_at(text.as_bytes(), 0) {
        Ok((code_point, _)) => is_selected_identifier_part_code_point(code_point),
        Err(_) => false,
    }
}

/// `SelectedDirectDeleteKeyword` boundary: proves direct authored ownership
/// of the exact ASCII `"delete"` keyword token. Applies the already-settled
/// maximal-`IdentifierName` principle -- if the code point immediately
/// following the six-byte prefix would itself continue an `IdentifierName`,
/// the whole candidate remains one longer `IdentifierReference` and this is
/// not a `delete` keyword boundary at all (Issue #839 section 11). An
/// escaped keyword spelling such as `\u0064elete` is excluded structurally
/// by the literal-ASCII-prefix test, never by normalizing an escape into a
/// keyword.
fn selected_direct_delete_keyword_boundary(candidate: &str) -> Option<usize> {
    if !candidate.as_bytes().starts_with(b"delete") {
        return None;
    }
    let after = &candidate[6..];
    if continues_as_identifier_part(after) {
        return None;
    }
    Some(6)
}

/// Central Issue #839 theorem: exactly one direct authored `"delete"`
/// keyword boundary, followed by one or more already-accepted selected
/// trivia code points (non-empty, because the bounded operand is exactly one
/// `IdentifierReference` and needs a separator from the keyword), followed by
/// exactly one complete Direct or EscapedNonReserved `IdentifierReference`
/// occupying the rest of the candidate. Whole-string: no richer tail, no
/// comment separator, no grouping/private-reference/other-operand shape
/// survives -- every broader-valid-but-outside and malformed/invalid control
/// fails structurally through the same two checks (keyword boundary, then
/// non-empty-trivia-then-single-IdentifierReference) without any special
/// casing per control.
///
/// The trivia scan advances a UTF-8 byte offset by each accepted code
/// point's own `len_utf8()` as it is examined -- a single forward pass that
/// is this oracle's owned recognition, never a later search, rescan, or
/// reparse over already-classified source.
fn is_selected_delete_identifier_reference_unary_expression(candidate: &str) -> bool {
    let Some(keyword_end) = selected_direct_delete_keyword_boundary(candidate) else {
        return false;
    };
    let after_keyword = &candidate[keyword_end..];
    let mut operand_start = 0usize;
    for code_point in after_keyword.chars() {
        if !is_selected_delete_operand_trivia(code_point) {
            break;
        }
        operand_start += code_point.len_utf8();
    }
    if operand_start == 0 {
        return false;
    }
    let operand = &after_keyword[operand_start..];
    is_selected_accepted_identifier_reference(operand)
}

/// Connects the independently computed Oracle recognition result to a
/// `FrontierOutcome`, so a fixture-owned expected outcome is checked against
/// actual recognition of that exact candidate rather than asserted as a
/// standalone constant. A recognized bounded candidate is
/// `SelectedAcceptedIncomplete`; a bounded recognizer decline for a
/// broader-valid/outside or lower-layer unsupported candidate is
/// `UnsupportedCoverage` -- this Oracle never invents `SyntaxRejected`
/// merely because recognition declined.
fn classify_frontier_outcome(candidate: &str) -> FrontierOutcome {
    if is_selected_delete_identifier_reference_unary_expression(candidate) {
        FrontierOutcome::SelectedAcceptedIncomplete
    } else {
        FrontierOutcome::UnsupportedCoverage
    }
}

/// Independently restates only the already-accepted premise that a
/// selected `IdentifierReference` composes with existing lexical Binding/
/// Scope meaning by comparing declaration-order positions -- the same
/// conceptual relation the production Binding/Scope analysis in
/// `selected_binding_scope.rs` derives, restated here from a fixture-owned
/// authored binding inventory rather than by calling production.
fn expected_binding_scope_target(
    binding_inventory: &[&str],
    containing_binding_name: &str,
    reference_semantic_name: &str,
) -> ExpectedBindingScopeTarget {
    let containing_position = binding_inventory
        .iter()
        .position(|name| *name == containing_binding_name)
        .expect("fixture-owned containing binding must be present in the inventory");

    match binding_inventory
        .iter()
        .position(|name| *name == reference_semantic_name)
    {
        None => ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
        Some(target_position) => {
            let order = match target_position.cmp(&containing_position) {
                std::cmp::Ordering::Less => ExpectedLexicalBindingOrder::Before,
                std::cmp::Ordering::Equal => ExpectedLexicalBindingOrder::Same,
                std::cmp::Ordering::Greater => ExpectedLexicalBindingOrder::After,
            };
            ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(order)
        }
    }
}

// ---------------------------------------------------------------------------
// B. EE-11 context model.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeleteContext {
    NonStrictSelectedScript,
    StrictControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeleteOperandClass {
    IdentifierReference,
    ParenthesizedIdentifierReferenceControl,
    PrivateReferenceControl,
    OutsideSelectedTheorem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ee11Outcome {
    NoRejection,
    R01Rejection,
    R02RecursivelyReachesR01Rejection,
    StructurallyOutsideTheorem,
}

/// Fixture/context-authority truth table for the exact `EE-11-R01`/`EE-11-R02`
/// interaction this bounded theorem newly crosses (Issue #839 section "B. EE-11
/// context responsibility"; Issue #688 comment `5857970069` falsification
/// results 1-3). This is never derived by reparsing source or by a Directive
/// Prologue parser: `DeleteContext` is independent fixture/context authority,
/// backed by the pinned ECMA-262 Directive Prologue / `IsStrict` semantics and
/// the pinned Test262 `identifier-strict.js` / `identifier-strict-recursive.js`
/// challenge cases.
fn expected_ee11_outcome(context: DeleteContext, operand: DeleteOperandClass) -> Ee11Outcome {
    match operand {
        DeleteOperandClass::PrivateReferenceControl
        | DeleteOperandClass::OutsideSelectedTheorem => Ee11Outcome::StructurallyOutsideTheorem,
        DeleteOperandClass::IdentifierReference => match context {
            DeleteContext::NonStrictSelectedScript => Ee11Outcome::NoRejection,
            DeleteContext::StrictControl => Ee11Outcome::R01Rejection,
        },
        DeleteOperandClass::ParenthesizedIdentifierReferenceControl => match context {
            DeleteContext::NonStrictSelectedScript => Ee11Outcome::NoRejection,
            DeleteContext::StrictControl => Ee11Outcome::R02RecursivelyReachesR01Rejection,
        },
    }
}

/// One fixture-authored row connecting a concrete candidate to its
/// `DeleteContext`, its `DeleteOperandClass`, whether the bounded
/// source-composition recognizer independently admits that exact candidate
/// text, and the expected `EE-11` outcome for that context/class pairing.
/// `source_selected` is proven directly against
/// `is_selected_delete_identifier_reference_unary_expression` -- never
/// assumed -- so a candidate's own recognition result backs its row, rather
/// than a constant shared across unrelated candidates.
struct Ee11Fixture {
    candidate: &'static str,
    context: DeleteContext,
    operand_class: DeleteOperandClass,
    source_selected: bool,
    expected_ee11: Ee11Outcome,
}

/// The connected candidate/context/operand-class/outcome matrix (Issue #839
/// review remediation, finding 3). `StrictControl` rows never feed a whole
/// `"use strict"`-prefixed source through the non-strict-envelope
/// recognizer -- `candidate` stays the bare `delete` + operand fragment, and
/// `context` is independent fixture authority layered on top, exactly as
/// `DeleteContext`'s own contract requires; `"use strict"` is never scanned
/// to derive it.
const EE11_SEMANTIC_MATRIX: &[Ee11Fixture] = &[
    Ee11Fixture {
        candidate: "delete a",
        context: DeleteContext::NonStrictSelectedScript,
        operand_class: DeleteOperandClass::IdentifierReference,
        source_selected: true,
        expected_ee11: Ee11Outcome::NoRejection,
    },
    Ee11Fixture {
        candidate: "delete \\u0061",
        context: DeleteContext::NonStrictSelectedScript,
        operand_class: DeleteOperandClass::IdentifierReference,
        source_selected: true,
        expected_ee11: Ee11Outcome::NoRejection,
    },
    Ee11Fixture {
        candidate: "delete a",
        context: DeleteContext::StrictControl,
        operand_class: DeleteOperandClass::IdentifierReference,
        source_selected: true,
        expected_ee11: Ee11Outcome::R01Rejection,
    },
    Ee11Fixture {
        candidate: "delete \\u0061",
        context: DeleteContext::StrictControl,
        operand_class: DeleteOperandClass::IdentifierReference,
        source_selected: true,
        expected_ee11: Ee11Outcome::R01Rejection,
    },
    Ee11Fixture {
        candidate: "delete ((a))",
        context: DeleteContext::NonStrictSelectedScript,
        operand_class: DeleteOperandClass::ParenthesizedIdentifierReferenceControl,
        source_selected: false,
        expected_ee11: Ee11Outcome::NoRejection,
    },
    Ee11Fixture {
        candidate: "delete ((a))",
        context: DeleteContext::StrictControl,
        operand_class: DeleteOperandClass::ParenthesizedIdentifierReferenceControl,
        source_selected: false,
        expected_ee11: Ee11Outcome::R02RecursivelyReachesR01Rejection,
    },
    Ee11Fixture {
        candidate: "delete obj.#x",
        context: DeleteContext::NonStrictSelectedScript,
        operand_class: DeleteOperandClass::PrivateReferenceControl,
        source_selected: false,
        expected_ee11: Ee11Outcome::StructurallyOutsideTheorem,
    },
    Ee11Fixture {
        candidate: "delete obj.#x",
        context: DeleteContext::StrictControl,
        operand_class: DeleteOperandClass::PrivateReferenceControl,
        source_selected: false,
        expected_ee11: Ee11Outcome::StructurallyOutsideTheorem,
    },
    Ee11Fixture {
        candidate: "delete(a)",
        context: DeleteContext::NonStrictSelectedScript,
        operand_class: DeleteOperandClass::OutsideSelectedTheorem,
        source_selected: false,
        expected_ee11: Ee11Outcome::StructurallyOutsideTheorem,
    },
];

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 839);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert!(MODEL_SOURCE.contains(TEST262_SNAPSHOT));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREVIOUS_BANG_TILDE_UNARY_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 827"));
    assert!(PREVIOUS_PARENTHESIZED_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 831"));
    assert!(PREVIOUS_ESCAPED_IDENTIFIER_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 241"));
    assert!(FRONTIER_SCOPE_NOTE.contains(
        "exactly-one direct delete keyword IdentifierReference UnaryExpression frontier only"
    ));
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));
    assert!(
        PROFILE_FIREWALL_NOTE
            .contains("authority only for the current non-strict selected Script carrier")
    );
    assert!(PROFILE_FIREWALL_NOTE.contains("not reusable authority for strict Script"));

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
        concat!("use super::", "qualification::"),
        concat!("recognize_selected_", "lexical_slice"),
        concat!("consume_", "keyword("),
        concat!("consume_selected_", "identifier_reference"),
        concat!("analyze_selected_binding_", "scope("),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_selected_block_var_", "statement"),
        concat!("parse_unary_", "expression"),
        concat!("parse_primary_", "expression"),
        concat!("parse_", "expression("),
        // Candidate independence for trivia recognition specifically: the
        // production selected-trivia *recognizer* is never imported or
        // called, only the stable normative Unicode `Space_Separator`
        // property primitive (`is_space_separator`) is reused.
        concat!("is_selected_", "trivia("),
        concat!("skip_selected_", "trivia("),
        concat!("use super::", "selected_lexical_slice::is_selected_trivia"),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
        // Runtime `delete`/`Reference` firewall (call-syntax, so genuine doc
        // prose mentioning these normative terms is unaffected).
        concat!("Get", "Value("),
        concat!("Resolve", "Binding("),
        concat!("Delete", "Binding("),
        concat!("fn parse_directive_", "prologue"),
        concat!("struct Directive", "Prologue"),
        concat!("fn is_", "strict_directive"),
        concat!("struct Reference", "Record"),
        concat!("struct Environment", "Record"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // `is_id_start` / `is_id_continue` / `is_space_separator` are stable,
    // frozen normative Unicode 17 primitives this oracle is independently
    // entitled to consult (also already used by predecessor accepted
    // oracles at the same `super::unicode` path); none of them is the
    // production selected-trivia recognizer or a production expression
    // parser.
    assert!(
        THIS_ORACLE_SOURCE
            .contains("use super::unicode::{is_id_continue, is_id_start, is_space_separator}")
    );
}

/// Required positive matrix: fixture-owned literal byte ranges for keyword,
/// inner reference, and the complete `delete` occurrence -- never derived by
/// search, rescan, or reparse. The inner reference range never includes the
/// keyword byte range; the decoded semantic name and Direct/EscapedNonReserved
/// provenance are preserved exactly.
#[test]
fn exact_positive_matrix_pins_fixture_owned_keyword_and_reference_ranges() {
    struct PositiveRow {
        source: &'static str,
        keyword: Range,
        reference: Range,
        whole: Range,
        expected_reference: &'static str,
        expected_name: &'static str,
        expected_provenance: IdentifierReferenceProvenance,
        expected_outcome: FrontierOutcome,
    }

    let rows = [
        PositiveRow {
            source: "const x = delete a;",
            keyword: Range(10, 16),
            reference: Range(17, 18),
            whole: Range(10, 18),
            expected_reference: "a",
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::Direct,
            expected_outcome: FrontierOutcome::SelectedAcceptedIncomplete,
        },
        PositiveRow {
            source: "const x = delete \u{03C0};",
            keyword: Range(10, 16),
            reference: Range(17, 19),
            whole: Range(10, 19),
            expected_reference: "\u{03C0}",
            expected_name: "\u{03C0}",
            expected_provenance: IdentifierReferenceProvenance::Direct,
            expected_outcome: FrontierOutcome::SelectedAcceptedIncomplete,
        },
        PositiveRow {
            source: "const x = delete \u{1D49C};",
            keyword: Range(10, 16),
            reference: Range(17, 21),
            whole: Range(10, 21),
            expected_reference: "\u{1D49C}",
            expected_name: "\u{1D49C}",
            expected_provenance: IdentifierReferenceProvenance::Direct,
            expected_outcome: FrontierOutcome::SelectedAcceptedIncomplete,
        },
        PositiveRow {
            source: "const x = delete \\u0061;",
            keyword: Range(10, 16),
            reference: Range(17, 23),
            whole: Range(10, 23),
            expected_reference: "\\u0061",
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
            expected_outcome: FrontierOutcome::SelectedAcceptedIncomplete,
        },
        PositiveRow {
            source: "const x = delete f\\u006Fo;",
            keyword: Range(10, 16),
            reference: Range(17, 25),
            whole: Range(10, 25),
            expected_reference: "f\\u006Fo",
            expected_name: "foo",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
            expected_outcome: FrontierOutcome::SelectedAcceptedIncomplete,
        },
        PositiveRow {
            source: "const x = delete \\u{66}oo;",
            keyword: Range(10, 16),
            reference: Range(17, 25),
            whole: Range(10, 25),
            expected_reference: "\\u{66}oo",
            expected_name: "foo",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
            expected_outcome: FrontierOutcome::SelectedAcceptedIncomplete,
        },
        PositiveRow {
            source: "const x = delete \\u{1D49C};",
            keyword: Range(10, 16),
            reference: Range(17, 26),
            whole: Range(10, 26),
            expected_reference: "\\u{1D49C}",
            expected_name: "\u{1D49C}",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
            expected_outcome: FrontierOutcome::SelectedAcceptedIncomplete,
        },
    ];

    for (index, row) in rows.iter().enumerate() {
        assert_eq!(slice(row.source, row.keyword), "delete");
        assert_eq!(slice(row.source, row.reference), row.expected_reference);
        // The inner reference range never includes the keyword byte range,
        // and starts strictly after it (a non-empty trivia gap separates
        // them).
        assert!(row.reference.0 > row.keyword.1);
        assert_eq!(row.whole.0, row.keyword.0);
        assert_eq!(row.whole.1, row.reference.1);

        let whole_text = slice(row.source, row.whole);
        assert!(
            is_selected_delete_identifier_reference_unary_expression(whole_text),
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
            authored_anchor(839_000 + index as u64, row.source, row.whole),
            whole_text
        );
        // The inner reference anchor is exactly the (possibly escaped)
        // spelling, never the keyword/trivia-prefixed spelling, and the
        // decoded semantic name never includes the keyword or trivia.
        assert_eq!(
            authored_anchor(839_050 + index as u64, row.source, row.reference),
            row.expected_reference
        );

        assert_eq!(classify_frontier_outcome(whole_text), row.expected_outcome);
    }
}

/// Selected trivia matrix (bare candidates, offsets start at 0): ASCII
/// `TAB`/`LF`, and the frozen Unicode 17 `Space_Separator` property beyond
/// ASCII space (NBSP) all compose as the required non-empty operand trivia;
/// comments never do, even though comments are a valid ECMAScript separator
/// in general (they remain outside this selected trivia theorem, not a
/// syntax error).
#[test]
fn selected_trivia_matrix_between_keyword_and_operand() {
    struct TriviaRow {
        candidate: &'static str,
        keyword: Range,
        trivia: Range,
        reference: Range,
    }

    let rows = [
        TriviaRow {
            candidate: "delete\ta",
            keyword: Range(0, 6),
            trivia: Range(6, 7),
            reference: Range(7, 8),
        },
        TriviaRow {
            candidate: "delete\na",
            keyword: Range(0, 6),
            trivia: Range(6, 7),
            reference: Range(7, 8),
        },
        TriviaRow {
            candidate: "delete a",
            keyword: Range(0, 6),
            trivia: Range(6, 7),
            reference: Range(7, 8),
        },
        TriviaRow {
            candidate: "delete\u{00A0}a",
            keyword: Range(0, 6),
            trivia: Range(6, 8),
            reference: Range(8, 9),
        },
    ];

    for row in &rows {
        assert_eq!(slice(row.candidate, row.keyword), "delete");
        assert_eq!(slice(row.candidate, row.reference), "a");
        assert!(row.trivia.1 > row.trivia.0, "trivia must be non-empty");
        assert_eq!(row.trivia.0, row.keyword.1);
        assert_eq!(row.trivia.1, row.reference.0);
        assert!(
            is_selected_delete_identifier_reference_unary_expression(row.candidate),
            "{:?}",
            row.candidate
        );
        assert_eq!(
            classify_frontier_outcome(row.candidate),
            FrontierOutcome::SelectedAcceptedIncomplete,
            "{:?}",
            row.candidate
        );
    }

    // Comments never compose as required operand trivia, even though they
    // are a valid ECMAScript separator in general -- this candidate remains
    // `OutsideSelectedTheorem`, not `SyntaxRejected`.
    for comment_control in ["delete/*c*/a", "delete//c\na"] {
        assert!(
            !is_selected_delete_identifier_reference_unary_expression(comment_control),
            "{comment_control:?}"
        );
        assert_eq!(
            classify_frontier_outcome(comment_control),
            FrontierOutcome::UnsupportedCoverage,
            "{comment_control:?}"
        );
    }
}

/// Non-strict `IdentifierReference` policy composition (representative
/// categories only, not a re-test of Issue #237/#241): the current-profile
/// name-policy families that are already accepted `IdentifierReference`s in
/// the fixed non-strict selected envelope all compose as valid `delete`
/// operands, both directly authored and EscapedNonReserved. These are
/// *not* substitutes for the clean `EE-11-R01` strict-attribution controls
/// below (Issue #688 comment `5857970069` falsification result 4: some of
/// these names independently change grammar policy under strict mode, so a
/// strict failure of e.g. `delete let` would not cleanly attribute to
/// `EE-11-R01` itself).
#[test]
fn non_strict_identifier_reference_policy_composition_matrix() {
    const ACCEPTED_DIRECT_OPERANDS: &[&str] =
        &["let", "static", "yield", "await", "eval", "arguments"];
    for operand in ACCEPTED_DIRECT_OPERANDS {
        assert_eq!(
            classify_selected_accepted_identifier_reference(operand),
            Some(((*operand).to_owned(), IdentifierReferenceProvenance::Direct)),
            "{operand:?}"
        );
        let candidate = format!("delete {operand}");
        assert!(
            is_selected_delete_identifier_reference_unary_expression(&candidate),
            "{candidate:?}"
        );
    }

    const ACCEPTED_ESCAPED_OPERANDS: &[(&str, &str)] = &[
        ("\\u006Cet", "let"),
        ("\\u0079ield", "yield"),
        ("a\\u0077ait", "await"),
        ("\\u0065val", "eval"),
        ("\\u0061rguments", "arguments"),
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
        let candidate = format!("delete {rhs}");
        assert!(
            is_selected_delete_identifier_reference_unary_expression(&candidate),
            "{candidate:?}"
        );
    }

    assert_eq!(UNCONDITIONALLY_RESERVED_WORDS.len(), 36);
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"yield"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"await"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"eval"));
    assert!(!UNCONDITIONALLY_RESERVED_WORDS.contains(&"arguments"));
    assert!(UNCONDITIONALLY_RESERVED_WORDS.contains(&"delete"));
}

/// Maximal-`IdentifierName` firewall (Issue #839 section 11): these authored
/// spellings are NOT split into `delete` keyword + operand. Where accepted
/// by the already-settled `IdentifierReference` theorem, they remain one
/// maximal authored `IdentifierReference`. An escaped keyword spelling is
/// never a direct `delete` token.
#[test]
fn maximal_identifier_name_firewall_never_splits_keyword_and_operand() {
    struct MaximalRow {
        candidate: &'static str,
        expected_name: &'static str,
        expected_provenance: IdentifierReferenceProvenance,
    }

    let rows = [
        MaximalRow {
            candidate: "deletea",
            expected_name: "deletea",
            expected_provenance: IdentifierReferenceProvenance::Direct,
        },
        MaximalRow {
            candidate: "delete\u{03C0}",
            expected_name: "delete\u{03C0}",
            expected_provenance: IdentifierReferenceProvenance::Direct,
        },
        MaximalRow {
            candidate: "delete\\u0061",
            expected_name: "deletea",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
        },
        MaximalRow {
            candidate: "delete\\u{61}",
            expected_name: "deletea",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
        },
    ];

    for row in &rows {
        // Never recognized as the delete-keyword wrapper.
        assert!(
            !is_selected_delete_identifier_reference_unary_expression(row.candidate),
            "{:?}",
            row.candidate
        );
        assert!(
            selected_direct_delete_keyword_boundary(row.candidate).is_none(),
            "{:?}",
            row.candidate
        );
        // Instead independently proven to remain one maximal accepted
        // IdentifierReference.
        assert_eq!(
            classify_selected_accepted_identifier_reference(row.candidate),
            Some((row.expected_name.to_owned(), row.expected_provenance)),
            "{:?}",
            row.candidate
        );
        assert_eq!(
            classify_frontier_outcome(row.candidate),
            FrontierOutcome::UnsupportedCoverage,
            "{:?}",
            row.candidate
        );
    }

    // An escaped keyword spelling is never a direct `delete` token: the
    // literal-ASCII-prefix test excludes it before any operand/trivia
    // classification is attempted.
    assert!(selected_direct_delete_keyword_boundary("\\u0064elete a").is_none());
    assert!(!is_selected_delete_identifier_reference_unary_expression(
        "\\u0064elete a"
    ));
    assert_eq!(
        classify_frontier_outcome("\\u0064elete a"),
        FrontierOutcome::UnsupportedCoverage
    );
}

/// Broader-valid-but-outside controls (Issue #839 section 12): these remain
/// valid broader ECMAScript outside this exact bounded theorem, never
/// globally invalid syntax. `OutsideSelectedTheorem != SyntaxRejected`.
#[test]
fn broader_valid_but_outside_controls_are_not_syntax_rejected() {
    const OUTSIDE_CONTROLS: &[&str] = &[
        "delete(a)",
        "delete((a))",
        "delete!a",
        "delete~a",
        "delete+a",
        "delete-a",
        "delete typeof a",
        "delete void a",
        "delete delete a",
        "delete/*c*/a",
    ];
    for control in OUTSIDE_CONTROLS {
        assert!(
            !is_selected_delete_identifier_reference_unary_expression(control),
            "{control:?}"
        );
        assert_eq!(
            classify_frontier_outcome(control),
            FrontierOutcome::UnsupportedCoverage,
            "{control:?}"
        );
    }

    // The keyword boundary itself still holds for the punctuator-adjacent
    // controls (maximal munch does not extend into punctuation); only the
    // required non-empty trivia / single-IdentifierReference operand shape
    // is what excludes them.
    for control in ["delete(a)", "delete!a", "delete~a", "delete+a", "delete-a"] {
        assert!(
            selected_direct_delete_keyword_boundary(control).is_some(),
            "{control:?}"
        );
    }
}

/// Malformed / invalid operand controls (Issue #839 section 13): already-owned
/// `IdentifierReference`/`Identifier` distinctions are preserved verbatim,
/// through the single already-accepted decoder, never a delete-specific
/// second decoder, and never upgraded into a stronger rejection category.
#[test]
fn malformed_invalid_operand_controls_reuse_owned_decode_failures() {
    struct MalformedRow {
        operand: &'static str,
        expected_failure: DecodeFailure,
    }

    let rows = [
        MalformedRow {
            operand: "\\u0069f",
            expected_failure: DecodeFailure::DecodedReserved,
        },
        MalformedRow {
            operand: "\\u{69}f",
            expected_failure: DecodeFailure::DecodedReserved,
        },
        MalformedRow {
            operand: "\\u{}",
            expected_failure: DecodeFailure::MalformedEscape,
        },
        MalformedRow {
            operand: "\\u{G}",
            expected_failure: DecodeFailure::MalformedEscape,
        },
        MalformedRow {
            operand: "\\u{110000}",
            expected_failure: DecodeFailure::NonCodePoint,
        },
        MalformedRow {
            operand: "\\u0030",
            expected_failure: DecodeFailure::InvalidStart,
        },
    ];

    for row in &rows {
        assert_eq!(
            decode_selected_escaped_identifier(row.operand),
            Err(row.expected_failure),
            "{:?}",
            row.operand
        );
        assert_eq!(
            classify_selected_accepted_identifier_reference(row.operand),
            None,
            "{:?}",
            row.operand
        );
        let candidate = format!("delete {}", row.operand);
        assert!(
            !is_selected_delete_identifier_reference_unary_expression(&candidate),
            "{candidate:?}"
        );
        // These remain UnsupportedCoverage (processing declines the
        // candidate), never a stronger SyntaxRejected claim invented by
        // this Oracle -- the whole `delete UnaryExpression` production does
        // not exist yet to reject anything.
        assert_eq!(
            classify_frontier_outcome(&candidate),
            FrontierOutcome::UnsupportedCoverage,
            "{candidate:?}"
        );
    }

    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
}

/// Private-reference controls (Issue #839 section "EE-11 context
/// responsibility" / section 12): private target alternatives remain
/// structurally outside this first selected theorem. This Oracle proves
/// structural exclusion, never private-name semantics.
#[test]
fn private_reference_controls_remain_structurally_outside() {
    for control in ["delete obj.#x", "delete obj?.#x"] {
        assert!(
            !is_selected_delete_identifier_reference_unary_expression(control),
            "{control:?}"
        );
    }

    for context in [
        DeleteContext::NonStrictSelectedScript,
        DeleteContext::StrictControl,
    ] {
        assert_eq!(
            expected_ee11_outcome(context, DeleteOperandClass::PrivateReferenceControl),
            Ee11Outcome::StructurallyOutsideTheorem
        );
    }
}

/// EE-11 context model matrix (Issue #839 section "B. EE-11 context
/// responsibility" and "EE-11 attribution requirements"): the exact
/// semantic interaction this bounded theorem newly crosses, proven as a
/// fixture/context truth table -- never by reparsing source, never by a
/// Directive Prologue parser.
#[test]
fn ee11_context_model_matrix_proves_non_triggering_and_strict_controls() {
    assert_eq!(
        expected_ee11_outcome(
            DeleteContext::NonStrictSelectedScript,
            DeleteOperandClass::IdentifierReference
        ),
        Ee11Outcome::NoRejection
    );
    assert_eq!(
        expected_ee11_outcome(
            DeleteContext::StrictControl,
            DeleteOperandClass::IdentifierReference
        ),
        Ee11Outcome::R01Rejection
    );
    assert_eq!(
        expected_ee11_outcome(
            DeleteContext::StrictControl,
            DeleteOperandClass::ParenthesizedIdentifierReferenceControl
        ),
        Ee11Outcome::R02RecursivelyReachesR01Rejection
    );
    assert_eq!(
        expected_ee11_outcome(
            DeleteContext::NonStrictSelectedScript,
            DeleteOperandClass::ParenthesizedIdentifierReferenceControl
        ),
        Ee11Outcome::NoRejection
    );
    assert_eq!(
        expected_ee11_outcome(
            DeleteContext::NonStrictSelectedScript,
            DeleteOperandClass::OutsideSelectedTheorem
        ),
        Ee11Outcome::StructurallyOutsideTheorem
    );

    // Every row of the connected candidate/context/operand-class/outcome
    // matrix independently proves its own recognition result and its own
    // EE-11 outcome, rather than a shared candidate loop reusing constant
    // context/operand-class arguments.
    for row in EE11_SEMANTIC_MATRIX {
        assert_eq!(
            is_selected_delete_identifier_reference_unary_expression(row.candidate),
            row.source_selected,
            "{:?}",
            row.candidate
        );
        assert_eq!(
            expected_ee11_outcome(row.context, row.operand_class),
            row.expected_ee11,
            "{:?}",
            row.candidate
        );
        // Fixture-class consistency invariant: for this exact bounded
        // matrix, `source_selected` and `operand_class` are never authored
        // independently of each other. Only the `IdentifierReference`
        // operand class is admitted by the bounded source-composition
        // theorem; `ParenthesizedIdentifierReferenceControl`,
        // `PrivateReferenceControl`, and `OutsideSelectedTheorem` are all
        // structurally declined. Without this, a row could accidentally
        // pair `source_selected = false` with `operand_class =
        // IdentifierReference` (or vice versa) and still pass the checks
        // above, since more than one operand class yields `NoRejection` in
        // the non-strict context.
        assert_eq!(
            row.source_selected,
            row.operand_class == DeleteOperandClass::IdentifierReference,
            "{:?}",
            row.candidate
        );
    }
}

/// Clean `EE-11-R01` attribution controls (Issue #688 comment `5857970069`
/// falsification results 2/4): ordinary names whose `IdentifierReference`
/// validity is stable across the strictness contrast, for both Direct and
/// EscapedNonReserved provenance -- direct vs. escaped provenance plays no
/// role in `EE-11-R01` applicability once both form an `IdentifierReference`.
/// `StrictControl` fixtures are corroborated by the pinned Test262
/// `identifier-strict.js` (`onlyStrict`, `delete test262identifier;`) case;
/// this Oracle never derives strictness by scanning the fixture text for a
/// directive.
#[test]
fn clean_ee11_r01_strict_attribution_controls_are_provenance_independent() {
    const STRICT_PREFIX: &str = "\"use strict\";\n";

    // Pull exactly the IdentifierReference-class rows of the connected
    // matrix -- both Direct and EscapedNonReserved, both contexts -- so
    // each candidate's own recognition and own EE-11 outcome are checked
    // against that same candidate's own row, never a constant shared across
    // unrelated candidates.
    let clean_r01_rows = EE11_SEMANTIC_MATRIX
        .iter()
        .filter(|row| row.operand_class == DeleteOperandClass::IdentifierReference);

    for row in clean_r01_rows {
        assert_eq!(
            is_selected_delete_identifier_reference_unary_expression(row.candidate),
            row.source_selected,
            "{:?}",
            row.candidate
        );
        assert_eq!(
            expected_ee11_outcome(row.context, row.operand_class),
            row.expected_ee11,
            "{:?}",
            row.candidate
        );

        if row.context == DeleteContext::StrictControl {
            // The strict-control fixture pairs this exact accepted
            // candidate with an explicit fixture-authored `StrictControl`
            // context; it is never fed through the (non-strict-envelope-
            // scoped) recognizer above as a claim about production
            // strict-mode source, and `"use strict"` is never scanned to
            // derive the context.
            let strict_control_fixture = format!("{STRICT_PREFIX}{};", row.candidate);
            assert!(strict_control_fixture.starts_with(STRICT_PREFIX));
            assert!(strict_control_fixture.contains(row.candidate));
        }
    }

    // Strict-sensitive Identifier-policy names (e.g. `let`) are deliberately
    // excluded as the *sole* proof of R01: their independent strict-policy
    // rejection would not cleanly attribute to EE-11-R01 itself
    // (falsification result 4). This test uses only ordinary, strictness-
    // stable names (`a`, its escaped spelling) as the clean control pair.
    for row in EE11_SEMANTIC_MATRIX
        .iter()
        .filter(|row| row.operand_class == DeleteOperandClass::IdentifierReference)
    {
        assert!(row.candidate == "delete a" || row.candidate == "delete \\u0061");
    }
}

/// `EE-11-R02` recursive attribution controls (Issue #688 comment
/// `5857970069` falsification result 3): grouped `delete` is not
/// unconditionally invalid. The recursive rule matters only when the
/// covered operand ultimately reaches a forbidden strict `IdentifierReference`.
/// Grouping remains entirely outside this bounded theorem's admitted operand
/// shape in both contexts -- these fixtures are semantic controls only.
/// Corroborated by the pinned Test262 `identifier-strict-recursive.js`
/// (`onlyStrict`, `delete ((identifier));`) case.
#[test]
fn ee11_r02_recursive_controls_are_conditional_not_unconditional() {
    for grouped in ["delete (a)", "delete ((a))"] {
        // Grouping is never an admitted operand of this bounded theorem in
        // either context.
        assert!(!is_selected_delete_identifier_reference_unary_expression(
            grouped
        ));
    }

    assert_eq!(
        expected_ee11_outcome(
            DeleteContext::NonStrictSelectedScript,
            DeleteOperandClass::ParenthesizedIdentifierReferenceControl
        ),
        Ee11Outcome::NoRejection,
        "non-strict grouped delete is not rejected merely from grouping"
    );
    assert_eq!(
        expected_ee11_outcome(
            DeleteContext::StrictControl,
            DeleteOperandClass::ParenthesizedIdentifierReferenceControl
        ),
        Ee11Outcome::R02RecursivelyReachesR01Rejection,
        "strict grouped delete recursively reaches the forbidden strict IdentifierReference"
    );
}

/// Downstream source-name correspondence challenge (Issue #839 section 16):
/// for a representative future relation `let a; const x = delete a;`, the
/// exact retained relation is the containing binding, the inner authored
/// reference, the decoded semantic name, and the same-source lexical
/// binding correspondence -- never a claim about `GetValue`, runtime
/// `ResolveBinding`, `DeleteBinding` result, or delete success/failure.
#[test]
fn downstream_source_name_correspondence_is_independently_frozen_without_runtime_claims() {
    struct CorrespondenceFixture {
        id: &'static str,
        source: &'static str,
        binding_inventory: &'static [&'static str],
        containing_binding_name: &'static str,
        containing_binding_range: Range,
        keyword_range: Range,
        reference_range: Range,
        expected_name: &'static str,
        expected_provenance: IdentifierReferenceProvenance,
        target_binding_range: Option<Range>,
        expected: ExpectedBindingScopeTarget,
    }

    let fixtures = [
        CorrespondenceFixture {
            id: "DELETE-CORRESPONDENCE-BEFORE-001",
            source: "let a;\nconst x = delete a;",
            binding_inventory: &["a", "x"],
            containing_binding_name: "x",
            containing_binding_range: Range(13, 14),
            keyword_range: Range(17, 23),
            reference_range: Range(24, 25),
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::Direct,
            target_binding_range: Some(Range(4, 5)),
            expected: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Before,
            ),
        },
        CorrespondenceFixture {
            id: "DELETE-CORRESPONDENCE-NOTARGET-001",
            source: "const x = delete z;",
            binding_inventory: &["x"],
            containing_binding_name: "x",
            containing_binding_range: Range(6, 7),
            keyword_range: Range(10, 16),
            reference_range: Range(17, 18),
            expected_name: "z",
            expected_provenance: IdentifierReferenceProvenance::Direct,
            target_binding_range: None,
            expected: ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding,
        },
        CorrespondenceFixture {
            id: "DELETE-CORRESPONDENCE-ESCAPED-BEFORE-001",
            source: "let a;\nconst x = delete \\u0061;",
            binding_inventory: &["a", "x"],
            containing_binding_name: "x",
            containing_binding_range: Range(13, 14),
            keyword_range: Range(17, 23),
            reference_range: Range(24, 30),
            expected_name: "a",
            expected_provenance: IdentifierReferenceProvenance::EscapedNonReserved,
            target_binding_range: Some(Range(4, 5)),
            expected: ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
                ExpectedLexicalBindingOrder::Before,
            ),
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        assert_eq!(slice(fixture.source, fixture.keyword_range), "delete");
        let whole = Range(fixture.keyword_range.0, fixture.reference_range.1);
        let whole_text = slice(fixture.source, whole);
        assert!(
            is_selected_delete_identifier_reference_unary_expression(whole_text),
            "{}: {whole_text:?}",
            fixture.id
        );

        assert_eq!(
            slice(fixture.source, fixture.containing_binding_range),
            fixture.containing_binding_name,
            "{}",
            fixture.id
        );

        // `reference_authored` is the raw authored source fragment (which,
        // for an escaped operand, is the escape spelling itself, e.g.
        // `\u0061`, never the decoded name). `reference_semantic_name` is
        // the independently decoded semantic name (Finding 4): these two
        // are never conflated -- the authored fragment backs
        // SourceAnchor/source-fragment assertions, the decoded name backs
        // binding/correspondence identity.
        let reference_authored = slice(fixture.source, fixture.reference_range);
        let (decoded_name, provenance) = classify_selected_accepted_identifier_reference(
            reference_authored,
        )
        .unwrap_or_else(|| {
            panic!(
                "{}: authored reference must be an accepted IdentifierReference",
                fixture.id
            )
        });
        assert_eq!(decoded_name, fixture.expected_name, "{}", fixture.id);
        assert_eq!(provenance, fixture.expected_provenance, "{}", fixture.id);
        let reference_semantic_name = decoded_name.as_str();

        // The inner reference anchor never includes the delete keyword, and
        // exactly reproduces the raw authored fragment -- never the decoded
        // name, never derived by byte-length inference or source searching.
        assert!(fixture.reference_range.0 > fixture.keyword_range.1);
        assert_eq!(
            authored_anchor(
                839_150 + index as u64,
                fixture.source,
                fixture.reference_range
            ),
            reference_authored,
            "{}",
            fixture.id
        );

        let computed = expected_binding_scope_target(
            fixture.binding_inventory,
            fixture.containing_binding_name,
            reference_semantic_name,
        );
        assert_eq!(computed, fixture.expected, "{}", fixture.id);

        match (fixture.target_binding_range, computed) {
            (
                Some(target_range),
                ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(_),
            ) => {
                assert_eq!(
                    slice(fixture.source, target_range),
                    reference_semantic_name,
                    "{}",
                    fixture.id
                );
                assert_eq!(
                    authored_anchor(839_200 + index as u64, fixture.source, target_range),
                    reference_semantic_name,
                    "{}",
                    fixture.id
                );
            }
            (None, ExpectedBindingScopeTarget::NoSameSourceSelectedLexicalBinding) => {}
            _ => panic!("{}: target range and expected target disagree", fixture.id),
        }
    }

    // No runtime vocabulary appears anywhere in this oracle: the
    // correspondence relation is a source-name relation, never a claim that
    // GetValue occurred, delete succeeded, DeleteBinding returned a value,
    // or the name is runtime-resolvable, and a no-target result is never
    // described as a runtime unresolvable-reference exception claim.
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("Reference", "Error")));
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("delete_", "succeeded")));
    assert!(!THIS_ORACLE_SOURCE.contains(concat!("runtime_", "resolvable")));
}

/// Static-semantics composition (mirrors the Issue #827 precedent): the
/// `delete` keyword and the inner RHS semantic name never become declaration
/// names or static rejection keys. Existing duplicate/collision and
/// missing-initializer authority remains driven only by authored
/// `BindingIdentifier` bindings.
#[test]
fn static_semantics_composition_remains_driven_by_authored_bindings() {
    /// Independently restates only the minimal duplicate-`BoundNames`
    /// premise these existing accepted gold controls exercise -- a pure
    /// authored-name-multiset check, never a call into
    /// `selected_static_semantics.rs`.
    fn has_duplicate_authored_binding_name(binding_names: &[&str]) -> bool {
        for (index, name) in binding_names.iter().enumerate() {
            if binding_names[..index].contains(name) {
                return true;
            }
        }
        false
    }

    /// Independently models, from fixture-owned authored evidence only,
    /// exactly the three narrow ECMA-262 `LexicalDeclaration`/`Script`
    /// static-semantics rules the four fixtures below exercise: a `let`
    /// declaration must not bind the name `let`; a declaration's
    /// `BoundNames` must contain no duplicate; every `const` `LexicalBinding`
    /// must have an `Initializer`. This is a minimal per-rule predicate over
    /// authored fixture data, never a generic static-semantics engine, and
    /// never a call into production.
    fn independently_modeled_lexical_static_rejection(
        bound_names: &[&str],
        const_declarator_has_initializer: Option<&[bool]>,
    ) -> bool {
        if bound_names.contains(&"let") {
            return true;
        }
        if has_duplicate_authored_binding_name(bound_names) {
            return true;
        }
        if let Some(has_initializer) = const_declarator_has_initializer
            && has_initializer.contains(&false)
        {
            return true;
        }
        false
    }

    struct StaticCompositionFixture {
        source: &'static str,
        subject: Range,
        subject_fragment: &'static str,
        control_gold_id: &'static str,
        bound_names: &'static [&'static str],
        const_declarator_has_initializer: Option<&'static [bool]>,
        expected_outcome: FrontierOutcome,
    }

    const FIXTURES: &[StaticCompositionFixture] = &[
        StaticCompositionFixture {
            source: "let let = delete a;",
            subject: Range(4, 7),
            subject_fragment: "let",
            control_gold_id: "JS-GOLD-LEXDECL-LET-BINDING-001",
            bound_names: &["let"],
            const_declarator_has_initializer: None,
            expected_outcome: FrontierOutcome::StaticSemanticsRejected,
        },
        StaticCompositionFixture {
            source: "let x = delete a, x = foo;",
            subject: Range(18, 19),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-LEXDECL-DUPBOUNDNAMES-001",
            bound_names: &["x", "x"],
            const_declarator_has_initializer: None,
            expected_outcome: FrontierOutcome::StaticSemanticsRejected,
        },
        StaticCompositionFixture {
            source: "const x = delete a, y;",
            subject: Range(20, 21),
            subject_fragment: "y",
            control_gold_id: "JS-GOLD-LEXDECL-CONST-MISSING-INIT-001",
            bound_names: &["x", "y"],
            const_declarator_has_initializer: Some(&[true, false]),
            expected_outcome: FrontierOutcome::StaticSemanticsRejected,
        },
        StaticCompositionFixture {
            source: "let x = delete a; let x = foo;",
            subject: Range(22, 23),
            subject_fragment: "x",
            control_gold_id: "JS-GOLD-SCRIPT-DUPLEXICAL-001",
            bound_names: &["x", "x"],
            const_declarator_has_initializer: None,
            expected_outcome: FrontierOutcome::StaticSemanticsRejected,
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
        // Connect the fixture-owned expected outcome to the actual
        // independently modeled static condition, rather than asserting a
        // standalone constant.
        assert_eq!(
            independently_modeled_lexical_static_rejection(
                fixture.bound_names,
                fixture.const_declarator_has_initializer
            ),
            matches!(
                fixture.expected_outcome,
                FrontierOutcome::StaticSemanticsRejected
            ),
            "{}",
            fixture.control_gold_id
        );
    }

    // The RHS reference name never becomes an additional `BoundName`: `a`
    // is declared exactly once (`let a;`) even though it also appears as
    // the delete operand's reference in `x`'s initializer.
    let rhs_not_a_new_bound_name = "let a;\nconst x = delete a;";
    let binding_inventory = ["a", "x"];
    assert_eq!(binding_inventory.len(), 2);
    assert_eq!(
        expected_binding_scope_target(&binding_inventory, "x", "a"),
        ExpectedBindingScopeTarget::SameSourceSelectedLexicalBinding(
            ExpectedLexicalBindingOrder::Before
        )
    );
    assert!(is_selected_delete_identifier_reference_unary_expression(
        slice(rhs_not_a_new_bound_name, Range(17, 25))
    ));
    // Sanity control: this ordinary, non-colliding, fully initialized
    // binding inventory is independently modeled as NOT rejected, proving
    // the predicate above is not vacuously true.
    assert!(!independently_modeled_lexical_static_rejection(
        &binding_inventory,
        None
    ));
}

/// Completion hypothesis (Issue #839 section 18): the frozen partition
/// `TOTAL = 193 / required = 10 / complement = 183 / NEWLY_REACHABLE = {}`
/// remains unchanged, because every candidate this bounded theorem admits
/// independently proves `IsStrict == false`, operand shape exactly
/// `IdentifierReference`, no private target, and no
/// `CoverParenthesizedExpressionAndArrowParameterList` operand -- so
/// `EE-11-R01`/`EE-11-R02` remain non-triggering. This Oracle does not
/// modify historical completion authority.
#[test]
fn completion_hypothesis_is_unchanged_pending_independent_proof() {
    const TOTAL_RULE_IDENTITIES: u32 = 193;
    const REQUIRED_REACHABLE: u32 = 10;
    const COMPLEMENT: u32 = 183;
    const NEWLY_REACHABLE: &[&str] = &[];

    assert_eq!(TOTAL_RULE_IDENTITIES, REQUIRED_REACHABLE + COMPLEMENT);
    assert!(NEWLY_REACHABLE.is_empty());

    // Evidence-backed replacement completion theorem: for every row of the
    // connected candidate/context/operand-class/outcome matrix that is (a)
    // an admitted first-leaf candidate (`source_selected`), (b) under the
    // current selected `NonStrictSelectedScript` context, and (c) exactly
    // the `IdentifierReference` operand class (structurally distinct from,
    // and therefore never coinciding with, `PrivateReferenceControl` or
    // `ParenthesizedIdentifierReferenceControl` -- i.e. no private target
    // and no `CoverParenthesizedExpressionAndArrowParameterList` operand) --
    // the independently modeled EE-11 outcome is `NoRejection`. This backs
    // the unchanged completion hypothesis with the matrix itself, rather
    // than asserting it alongside unrelated constants.
    let admitted_first_leaf_rows: Vec<_> = EE11_SEMANTIC_MATRIX
        .iter()
        .filter(|row| {
            row.source_selected
                && row.context == DeleteContext::NonStrictSelectedScript
                && row.operand_class == DeleteOperandClass::IdentifierReference
        })
        .collect();
    assert!(
        !admitted_first_leaf_rows.is_empty(),
        "at least one admitted first-leaf candidate row is required"
    );
    for row in &admitted_first_leaf_rows {
        assert!(is_selected_delete_identifier_reference_unary_expression(
            row.candidate
        ));
        assert_eq!(
            expected_ee11_outcome(row.context, row.operand_class),
            Ee11Outcome::NoRejection,
            "{:?}",
            row.candidate
        );
        assert_eq!(
            row.expected_ee11,
            Ee11Outcome::NoRejection,
            "{:?}",
            row.candidate
        );
    }
}

/// Freezes the presence-only, unqualified, validation-only handoff: this
/// leaf composes to `SelectedAcceptedIncomplete`, `UnsupportedCoverage`,
/// `StaticSemanticsRejected`, and `SyntaxRejected` remain distinct, and no
/// retained production representation, runtime, generic-parser, or
/// private-name vocabulary is introduced. The non-strict profile firewall
/// is explicitly sealed.
#[test]
fn handoff_remains_presence_only_unqualified_and_validation_only() {
    assert!(THIS_ORACLE_SOURCE.contains("SelectedAcceptedIncomplete"));
    assert!(THIS_ORACLE_SOURCE.contains("UnsupportedCoverage"));
    assert!(THIS_ORACLE_SOURCE.contains("StaticSemanticsRejected"));
    assert!(THIS_ORACLE_SOURCE.contains("SyntaxRejected"));
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);

    for forbidden in [
        concat!("ExpectedQualification", "::Qualified"),
        concat!("enum Selected", "Literal"),
        concat!("enum SelectedPrimary", "Expression"),
        concat!("enum SelectedExpression", "Kind"),
        concat!("enum Selected", "UnaryExpressionKind"),
        concat!("struct Selected", "UnaryExpressionNode"),
        concat!("enum Operator", "Kind"),
        concat!("struct Delete", "OperatorSourceAnchor"),
        concat!("struct Unary", "SourceAnchor"),
        concat!("struct Boolean", "Value"),
        concat!("struct Number", "Value"),
        concat!("enum Initializer", "Kind"),
        concat!("enum Primary", "Expression"),
        concat!("enum Expression", "Kind"),
        concat!("struct Ast", "Node"),
        concat!("struct Cst", "Node"),
        concat!("struct Token", "Tape"),
        concat!("struct Private", "Reference"),
        concat!("enum Private", "Name"),
        concat!("To", "Boolean("),
        concat!("To", "Numeric("),
        concat!("To", "Number("),
        concat!("delete_", "result"),
        concat!("delete_", "success"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
    assert!(PROFILE_FIREWALL_NOTE.contains("strict-capable delete analysis"));
}
