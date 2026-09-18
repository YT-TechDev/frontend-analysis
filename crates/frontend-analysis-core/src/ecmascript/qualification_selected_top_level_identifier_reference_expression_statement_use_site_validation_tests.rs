//! Candidate-independent top-level `IdentifierReference` `ExpressionStatement`
//! use-site validation for Issue #756 (durable research: Issue #688 comment
//! `5729112465`; accepted predecessor authorities: Issue #752 / PR #753
//! ordered two-`IdentifierReference` additive initializer oracle, Issue #754
//! / PR #755 ordered two-`IdentifierReference` additive production, Issue
//! #241 candidate-independent escaped `IdentifierReference` authored-source /
//! decoded-name theorem).
//!
//! This oracle qualifies only the bounded theorem:
//!
//! ```text
//! SelectedTopLevelIdentifierReferenceExpressionStatement ::=
//!     SelectedAcceptedIdentifierReference
//!     AuthoredSemicolon
//!
//! SelectedAcceptedIdentifierReference ::=
//!     SelectedDirectIdentifierReference
//!   | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! placed as a top-level item of a non-strict `Script`, alongside the
//! already-accepted bounded `let`/`var` binding items this oracle also
//! independently recognizes so a free-standing use-site has a same-source
//! declaration context to correspond against. It does not call production
//! lexical, static-semantics, correspondence, Binding/Scope, aggregate, or
//! runtime evaluation code.
//!
//! Every predecessor accepted `IdentifierReference` oracle recognized a
//! reference owned by a declaration initializer. This is the first oracle
//! whose selected reference occurrence is a free-standing use-site: it is
//! not read from inside any binding's initializer, and its semantic owner is
//! the exact occurrence itself plus its accepted top-level placement, never
//! the existing initializer relation's distinct declaration-owner field. The
//! analytical payoff is a dedicated
//! *source-name correspondence* relation -- not runtime binding resolution
//! -- proving, for every accepted occurrence, whether it corresponds to a
//! selected same-source top-level lexical binding, to every selected
//! same-source `var` contributor in authored order, or to no selected
//! same-source contributor at all.
//!
//! The independent lifecycle has three stages, matching the research
//! theorem's `prepare candidate evidence -> authoritative whole-source
//! consumption -> static-neutral accepted witness -> non-refusing relation
//! commit` shape, and is driven end to end by
//! `recognize_and_accept_selected_top_level_script`:
//!
//!   - Layer 1 (`parse_selected_top_level_script`): source / placement.
//!     Recognizes a whole candidate `Script` as an ordered sequence of
//!     exactly three bounded top-level item shapes -- a `let` binding item,
//!     a `var` binding item, or a free-standing `IdentifierReference`
//!     `ExpressionStatement` use-site item -- each terminated by an
//!     authored `;`. Recognition is whole-source, all-or-nothing: any
//!     unrecognized item aborts the complete parse (`None`), so a locally
//!     valid earlier item can never publish evidence when a later item
//!     fails (the whole-source transactionality theorem).
//!
//!   - Static preflight (`preflight_selected_top_level_static_semantics`):
//!     a bounded, candidate-independent gate over only the two existing
//!     top-level rules relevant to this shell -- a duplicate selected
//!     lexical name is rejected, and a selected lexical name colliding with
//!     a selected top-level `var` name is rejected (duplicate `var`
//!     contributors alone remain allowed) -- returning either the oracle's
//!     only accepted witness, `AcceptedSelectedTopLevelScript`, or a
//!     `StaticPreflightRejection` carrying the exact rule and anchors that
//!     independently proved the rejection. A syntactically recognized but
//!     statically rejected candidate never reaches Layer 2, and its
//!     rejection is never collapsed into the same outcome as a candidate
//!     whose source shape never matched this theorem at all: the two are
//!     materially distinct causes and remain distinct dispositions. This is
//!     not a general Early Error engine and never imports production
//!     static semantics.
//!
//!   - Layer 2 (`build_selected_source_name_correspondence`): relation.
//!     Consumes only the accepted witness (never an arbitrary item list,
//!     and never a production accepted-witness type) and independently
//!     derives exactly one correspondence relation per free-standing
//!     use-site item, in exact authored occurrence order, without
//!     deduplication.
//!
//! `classify_selected_top_level_script` binds this whole lifecycle to an
//! executable disposition (`Selected` / `UnsupportedCoverage` /
//! `StaticSemanticsRejected` / `DefinitiveGrammarRejectionEvidence` /
//! `ResourceLimited` / `InternalFailure`), so a fixture such as a
//! valid-but-unselected ASI boundary is independently proven to classify as
//! `UnsupportedCoverage` while a syntactically well-formed but statically
//! rejected shell -- a duplicate selected lexical name, or a selected
//! lexical/`var` name collision -- is independently proven to classify as
//! `StaticSemanticsRejected` instead, rather than either case being merely
//! named in a disconnected symbolic list or collapsed into the other.
//!
//! `let`/`var` binding items may carry an `= SelectedAcceptedIdentifierReference`
//! initializer purely so the whole-item grammar can be recognized end to
//! end; that inner reference is never retained as a fact and never becomes
//! part of Layer 2's relation stream, so this oracle never invents a
//! combined initializer/use-site relation theorem (existing initializer
//! relation authority, e.g. `selected_variable_statement_name_correspondence.rs`,
//! remains completely separate and is never imported here).
//!
//! `SelectedDirectIdentifierReference` independently restates the
//! already-accepted Issue #237 direct escape-free `IdentifierName`
//! code-point shape (`is_id_start`/`is_id_continue` plus `$`/`_`) minus the
//! unconditionally reserved words. `SelectedEscapedNonReservedIdentifierReference`
//! independently restates the already-accepted Issue #241
//! `UnicodeEscapeSequence` decode/position/decoded-reserved-word theorem.
//! Selected trivia independently restates the complete already-accepted
//! selected-slice trivia contract established by Issue #742/#743. None of
//! these restatements import their originating oracle's code, matching the
//! convention already set by every predecessor candidate-independent
//! `IdentifierReference` oracle: each leaf owns its own copy of the
//! already-accepted primitives it consumes.
//!
//! This is a validation-only leaf: production supports no `ExpressionStatement`
//! family at the #756 baseline, so every positive fixture below remains
//! `UnsupportedCoverage` under current production and exists only as
//! independent Oracle evidence for a future, separately authorized
//! production decision. No completion successor file accompanies this leaf:
//! this Issue adds zero production capability, so the frozen `193 / 10 / 183
//! / {}` completion partition cannot move, matching the precedent set by
//! #742/#743, #746/#747, and #752/#753.
//!
//! Runtime-negative boundary: `NoSelectedSameSourceContributor` is not
//! runtime-unbound, a `ReferenceError`, or a `ResolveBinding` failure.
//! `VisibleSelectedLexicalBinding` is not `ResolveBinding`,
//! `GetValue`, a Reference Record, an Environment Record, or TDZ execution
//! state. This oracle proves same-source selected declaration provenance
//! only.

use std::collections::HashMap;

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 756;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREVIOUS_ORDERED_TWO_IDENTIFIER_REFERENCE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_initializer_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_top_level_identifier_reference_expression_statement_use_site_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "top-level non-strict Script free-standing IdentifierReference ",
    "ExpressionStatement use-site frontier only; later independently ",
    "qualified owners may strengthen classification for nested Block/",
    "function/class body use-sites, Module items, strict-mode widening, ",
    "ASI-terminated use-sites, or richer Expression neighbors"
);

/// Preserved distinctly from `UnsupportedCoverage`, matching the minimal
/// symbolic model already accepted by the predecessor candidate-independent
/// `IdentifierReference` oracles (#241, #746/#747, #752/#753) -- this leaf
/// adds no further resource machinery beyond that already-accepted minimum.
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];

/// The four failure classifications this oracle keeps textually distinct
/// (acceptance criterion 24): a valid-but-unselected ASI boundary is
/// `UnsupportedCoverage`, never `DefinitiveGrammarRejectionEvidence`.
const FAILURE_CLASSIFICATIONS: &[&str] = &[
    "UnsupportedCoverage",
    "DefinitiveGrammarRejectionEvidence",
    "ResourceLimited",
    "InternalFailure",
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
// #237/#752, never imported from either oracle. ---

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
/// words -- also reused as this oracle's bounded `BindingIdentifier` policy,
/// since no fixture below requires an escaped or reserved-word declarator.
fn is_selected_direct_identifier_reference(candidate: &str) -> bool {
    is_escape_free_identifier_name(candidate)
        && !UNCONDITIONALLY_RESERVED_WORDS.contains(&candidate)
}

// --- Escaped IdentifierReference decode: independently restated from Issue
// #241, never imported from that oracle. ---

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
/// judged afterward once the whole candidate spelling is known.
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

/// `SelectedEscapedNonReservedIdentifierReference`: decodes a whole candidate
/// spelling that must contain at least one authored `UnicodeEscapeSequence`,
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
// as every intervening accepted oracle already restates it. ---

fn is_selected_trivia(code_point: char) -> bool {
    matches!(
        code_point,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{FEFF}' | '\n' | '\r' | '\u{2028}' | '\u{2029}'
    ) || is_space_separator(code_point as u32)
}

fn trivia_run_end(candidate: &str) -> usize {
    let mut offset = 0_usize;
    for code_point in candidate.chars() {
        if !is_selected_trivia(code_point) {
            break;
        }
        offset += code_point.len_utf8();
    }
    offset
}

// --- The Issue #756 theorem itself. ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdentifierSpellingKind {
    Direct,
    EscapedNonReserved,
}

/// Validates one whole candidate spelling against
/// `SelectedAcceptedIdentifierReference`: exactly one of the direct or
/// escaped-non-Reserved routes, over the complete candidate, never a prefix
/// or a later-reconstructed sub-range.
fn recognize_accepted_identifier_reference(
    candidate: &str,
) -> Option<(String, IdentifierSpellingKind)> {
    if candidate.contains('\\') {
        let decoded = decode_selected_escaped_identifier(candidate).ok()?;
        Some((
            decoded.string_value,
            IdentifierSpellingKind::EscapedNonReserved,
        ))
    } else if is_selected_direct_identifier_reference(candidate) {
        Some((candidate.to_owned(), IdentifierSpellingKind::Direct))
    } else {
        None
    }
}

/// Finds where a `BindingIdentifier` run ends: the already-accepted direct
/// escape-free `IdentifierName` code-point shape only (this oracle's bounded
/// declarator policy never accepts an escaped or reserved-word declarator).
fn direct_identifier_run_end(candidate: &str) -> usize {
    let mut chars = candidate.chars();
    let mut offset = match chars.next() {
        Some(code_point) if is_direct_identifier_start(code_point) => code_point.len_utf8(),
        _ => return 0,
    };
    for code_point in chars {
        if !is_direct_identifier_part(code_point) {
            break;
        }
        offset += code_point.len_utf8();
    }
    offset
}

/// Finds where a `SelectedAcceptedIdentifierReference` candidate run ends:
/// one left-to-right forward pass consuming either one direct code point or
/// one syntactically well-formed `UnicodeEscapeSequence` element, stopping
/// at the first code point that cannot continue an identifier-shaped run. A
/// malformed escape aborts the whole run immediately (`None`) rather than
/// silently truncating at that point -- the malformed escape never leaks an
/// earlier valid-looking prefix. Position validity of the collected run is
/// judged afterward by `recognize_accepted_identifier_reference`, never
/// here.
fn identifier_reference_run_end(candidate: &str) -> Option<usize> {
    let mut offset = 0_usize;
    while offset < candidate.len() {
        if candidate.as_bytes()[offset] == b'\\' {
            let formation = formed_escape_at(candidate, offset).ok()?;
            offset = formation.end;
            continue;
        }
        let code_point = candidate[offset..].chars().next()?;
        if !(code_point == '$' || code_point == '_' || is_id_continue(code_point as u32)) {
            break;
        }
        offset += code_point.len_utf8();
    }
    (offset > 0).then_some(offset)
}

/// One retained `let`/`var` binding fact: exactly the declarator name and
/// its own exact authored anchor. Its optional `= SelectedAcceptedIdentifierReference`
/// initializer is recognized only to prove the whole bounded item grammar
/// (never retained as a fact, and never contributes a Layer 2 relation of
/// its own -- W17/W24 firewall).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedBindingFact {
    semantic_name: String,
    binding: Range,
}

/// One retained free-standing use-site fact: the theorem this oracle exists
/// to validate.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedUseSiteFact {
    reference: Range,
    semantic_name: String,
    spelling: IdentifierSpellingKind,
}

/// The oracle's only Layer 1 retained representation: a private, bounded,
/// three-variant item shape. This is a minimal private test representation
/// scoped only to this oracle's exact theorem, never a claim that future
/// production must use this exact enum, a generic `Statement` hierarchy, or
/// any particular Rust type layout (W20).
#[derive(Debug, Clone, PartialEq, Eq)]
enum RecognizedTopLevelItem {
    LexicalBinding(RecognizedBindingFact),
    VarBinding(RecognizedBindingFact),
    UseSite(RecognizedUseSiteFact),
}

/// Recognizes one `let`/`var` binding item starting at `offset`: the
/// keyword, a mandatory trivia boundary, a direct `BindingIdentifier`, an
/// optional `= SelectedAcceptedIdentifierReference` initializer, and a
/// mandatory authored `;`. Returns the recognized fact and the absolute
/// offset immediately after the authored `;`.
fn parse_top_level_binding(
    source: &str,
    offset: usize,
    keyword: &'static str,
) -> Option<(RecognizedBindingFact, usize)> {
    let rest = source.get(offset..)?;
    let after_keyword = rest.strip_prefix(keyword)?;
    let boundary = after_keyword.chars().next()?;
    if !is_selected_trivia(boundary) {
        return None;
    }

    let mut cursor = offset + keyword.len();
    cursor += trivia_run_end(&source[cursor..]);

    let name_len = direct_identifier_run_end(&source[cursor..]);
    if name_len == 0 {
        return None;
    }
    let name_start = cursor;
    let name_end = cursor + name_len;
    let name = &source[name_start..name_end];
    if !is_selected_direct_identifier_reference(name) {
        return None;
    }
    cursor = name_end;

    cursor += trivia_run_end(&source[cursor..]);
    if source[cursor..].starts_with('=') {
        cursor += 1;
        cursor += trivia_run_end(&source[cursor..]);
        let rhs_len = identifier_reference_run_end(&source[cursor..])?;
        let rhs = &source[cursor..cursor + rhs_len];
        recognize_accepted_identifier_reference(rhs)?;
        cursor += rhs_len;
        cursor += trivia_run_end(&source[cursor..]);
    }

    if !source[cursor..].starts_with(';') {
        return None;
    }
    let end = cursor + 1;

    Some((
        RecognizedBindingFact {
            semantic_name: name.to_owned(),
            binding: Range(name_start, name_end),
        },
        end,
    ))
}

/// Recognizes one free-standing `SelectedTopLevelIdentifierReferenceExpressionStatement`
/// use-site item starting at `offset`: the theorem itself. Returns the
/// recognized fact and the absolute offset immediately after the authored
/// `;`.
fn parse_top_level_use_site(source: &str, offset: usize) -> Option<(RecognizedUseSiteFact, usize)> {
    let run_len = identifier_reference_run_end(&source[offset..])?;
    let candidate = &source[offset..offset + run_len];
    let (semantic_name, spelling) = recognize_accepted_identifier_reference(candidate)?;

    let mut cursor = offset + run_len;
    cursor += trivia_run_end(&source[cursor..]);
    if !source[cursor..].starts_with(';') {
        return None;
    }
    let end = cursor + 1;

    Some((
        RecognizedUseSiteFact {
            reference: Range(offset, offset + run_len),
            semantic_name,
            spelling,
        },
        end,
    ))
}

/// Central Issue #756 Layer 1 theorem. Whole-source, all-or-nothing: an
/// item sequence is never returned unless every top-level item, in exact
/// authored order, recognizes as one of the three bounded shapes with
/// nothing left over. A single unrecognized item -- a richer Expression
/// neighbor, a nested placement, an escaped ReservedWord, a malformed
/// escape, an ASI-only boundary, or trailing malformed source -- aborts the
/// complete parse; no earlier valid item is ever returned on its own
/// (whole-source transactionality).
fn parse_selected_top_level_script(source: &str) -> Option<Vec<RecognizedTopLevelItem>> {
    let mut offset = trivia_run_end(source);
    let mut items = Vec::new();

    while offset < source.len() {
        let (item, next_offset) =
            if let Some((fact, end)) = parse_top_level_binding(source, offset, "let") {
                (RecognizedTopLevelItem::LexicalBinding(fact), end)
            } else if let Some((fact, end)) = parse_top_level_binding(source, offset, "var") {
                (RecognizedTopLevelItem::VarBinding(fact), end)
            } else if let Some((fact, end)) = parse_top_level_use_site(source, offset) {
                (RecognizedTopLevelItem::UseSite(fact), end)
            } else {
                return None;
            };

        items.push(item);
        offset = next_offset;
        offset += trivia_run_end(&source[offset..]);
    }

    Some(items)
}

/// The oracle's only bounded static-preflight accepted witness: a
/// syntactically recognized item sequence that has also independently
/// passed the two existing top-level static rules relevant to this shell.
/// Layer 2 can only consume this witness, never an arbitrary
/// `Vec<RecognizedTopLevelItem>` -- this is a minimal private
/// representation scoped to this oracle's exact bounded shell, never a
/// claim about a future production accepted-witness type or layout.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AcceptedSelectedTopLevelScript {
    items: Vec<RecognizedTopLevelItem>,
}

impl AcceptedSelectedTopLevelScript {
    fn items(&self) -> &[RecognizedTopLevelItem] {
        &self.items
    }
}

/// One independently derived static-preflight rejection reason: exactly the
/// two existing top-level static rules relevant to this bounded shell, each
/// carrying the exact anchors that independently proved it. This is a
/// minimal private representation, never a claim about a future production
/// Early Error type layout.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StaticPreflightRejection {
    DuplicateLexical {
        semantic_name: String,
        first: Range,
        duplicate: Range,
    },
    LexicalVarCollision {
        semantic_name: String,
        lexical: Range,
        var: Range,
    },
}

/// The static preflight gate's own outcome: either the oracle's accepted
/// witness, or a rejection reason. Kept distinct from the syntax-level
/// `Option<Vec<RecognizedTopLevelItem>>` outcome of Layer 1, and from
/// `UnsupportedCoverage`: a statically rejected but syntactically
/// well-formed candidate is a materially different cause than a candidate
/// whose source shape never matched this theorem at all.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StaticPreflightOutcome {
    Accepted(AcceptedSelectedTopLevelScript),
    Rejected(StaticPreflightRejection),
}

/// Independently restates -- never imports -- only the two existing
/// top-level static rules relevant to this bounded shell: a duplicate
/// selected lexical name is rejected (mirroring the already-accepted
/// `LexicallyDeclaredNames` duplicate rule), and a selected lexical name
/// colliding with a selected top-level `var` name is rejected (mirroring
/// the already-accepted lexical/var collision rule). Duplicate `var`
/// contributors alone remain allowed -- the first authored anchor per `var`
/// name is recorded only for collision detection, never by count, so
/// repeated `var` declarators of the same name never trigger rejection on
/// their own. Both passes walk `items` in authored order, so the reported
/// rejection anchors are deterministic regardless of any internal map
/// iteration order. This is not a general Early Error engine: it proves
/// only this bounded shell's own already-accepted rules, and reproduces no
/// unrelated Early Error machinery.
fn preflight_selected_top_level_static_semantics(
    items: Vec<RecognizedTopLevelItem>,
) -> StaticPreflightOutcome {
    let mut lexical_names: HashMap<&str, Range> = HashMap::new();
    let mut var_names: HashMap<&str, Range> = HashMap::new();

    for item in &items {
        match item {
            RecognizedTopLevelItem::LexicalBinding(fact) => {
                if let Some(&first) = lexical_names.get(fact.semantic_name.as_str()) {
                    return StaticPreflightOutcome::Rejected(
                        StaticPreflightRejection::DuplicateLexical {
                            semantic_name: fact.semantic_name.clone(),
                            first,
                            duplicate: fact.binding,
                        },
                    );
                }
                lexical_names.insert(fact.semantic_name.as_str(), fact.binding);
            }
            RecognizedTopLevelItem::VarBinding(fact) => {
                var_names
                    .entry(fact.semantic_name.as_str())
                    .or_insert(fact.binding);
            }
            RecognizedTopLevelItem::UseSite(_) => {}
        }
    }

    for item in &items {
        let name = match item {
            RecognizedTopLevelItem::LexicalBinding(fact) => fact.semantic_name.as_str(),
            RecognizedTopLevelItem::VarBinding(fact) => fact.semantic_name.as_str(),
            RecognizedTopLevelItem::UseSite(_) => continue,
        };
        if let (Some(&lexical), Some(&var)) = (lexical_names.get(name), var_names.get(name)) {
            return StaticPreflightOutcome::Rejected(
                StaticPreflightRejection::LexicalVarCollision {
                    semantic_name: name.to_owned(),
                    lexical,
                    var,
                },
            );
        }
    }

    StaticPreflightOutcome::Accepted(AcceptedSelectedTopLevelScript { items })
}

/// The oracle's full independent lifecycle front door: source/placement
/// recognition (Layer 1), then the bounded static preflight gate,
/// producing the only shape Layer 2 may consume. The free-standing
/// use-site itself contributes no declaration names at either stage --
/// only `let`/`var` binding items are examined by the preflight gate --
/// but the surrounding already-selected declaration shell must still
/// independently pass it before any relation is ever committed. This
/// convenience accessor discards the specific rejection reason on failure;
/// `classify_selected_top_level_script` is the entry point that preserves
/// it.
fn recognize_and_accept_selected_top_level_script(
    source: &str,
) -> Option<AcceptedSelectedTopLevelScript> {
    let items = parse_selected_top_level_script(source)?;
    match preflight_selected_top_level_static_semantics(items) {
        StaticPreflightOutcome::Accepted(accepted) => Some(accepted),
        StaticPreflightOutcome::Rejected(_) => None,
    }
}

/// The oracle's executable disposition classification, binding a fixture
/// directly to its outcome rather than only naming these symbols in a
/// disconnected list. A valid-but-unselected ASI boundary independently
/// classifies as `UnsupportedCoverage`, never `DefinitiveGrammarRejectionEvidence`.
/// A syntactically well-formed but statically rejected shell independently
/// classifies as `StaticSemanticsRejected`, never downgraded to
/// `UnsupportedCoverage`: these are materially distinct causes -- a source
/// shape outside the selected theorem entirely, versus a source shape
/// recognized by the theorem but independently known to violate an
/// already-accepted static rule. This bounded recognizer never
/// independently proves definitive grammar rejection -- doing so would
/// require duplicating production's already-owned grammar-evidence
/// machinery, which this oracle must not do -- so it never constructs that
/// variant in practice; the variant exists so the type itself preserves the
/// project's failure vocabulary distinction, matching the already-accepted
/// `ResourceLimited`/`InternalFailure` symbolic minimum
/// used by every predecessor candidate-independent `IdentifierReference`
/// oracle.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedTopLevelScriptDisposition {
    Selected(AcceptedSelectedTopLevelScript),
    UnsupportedCoverage,
    StaticSemanticsRejected(StaticPreflightRejection),
    DefinitiveGrammarRejectionEvidence(Range),
    ResourceLimited,
    InternalFailure,
}

fn classify_selected_top_level_script(source: &str) -> SelectedTopLevelScriptDisposition {
    let Some(items) = parse_selected_top_level_script(source) else {
        return SelectedTopLevelScriptDisposition::UnsupportedCoverage;
    };

    match preflight_selected_top_level_static_semantics(items) {
        StaticPreflightOutcome::Accepted(accepted) => {
            SelectedTopLevelScriptDisposition::Selected(accepted)
        }
        StaticPreflightOutcome::Rejected(rejection) => {
            SelectedTopLevelScriptDisposition::StaticSemanticsRejected(rejection)
        }
    }
}

/// One independently derived Layer 2 source-name correspondence relation.
/// Exactly the theorem's vocabulary (`VisibleSelectedLexicalBinding`,
/// `SameSourceSelectedVarNameContributors`, `NoSelectedSameSourceContributor`)
/// and nothing else: no `Statement`/whole-expression/semicolon/item-ordinal
/// anchor, and no field mirroring the existing initializer relation's
/// distinct load-bearing declaration-owner field (#688 comment
/// `5729112465`; W1/W2/W5/W6/W18).
#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedSourceNameCorrespondence {
    VisibleSelectedLexicalBinding { binding: Range },
    SameSourceSelectedVarNameContributors { contributors: Vec<Range> },
    NoSelectedSameSourceContributor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedUseSiteRelation {
    reference: Range,
    semantic_name: String,
    correspondence: SelectedSourceNameCorrespondence,
}

/// Central Issue #756 Layer 2 theorem. Consumes only the bounded static
/// preflight's accepted witness -- never an arbitrary item list, and never
/// a production accepted-witness type -- and derives exactly one relation
/// per free-standing use-site item, in exact authored occurrence order,
/// without reordering or deduplication (W9/W10). Lexical bindings are
/// collected from the *whole* accepted item sequence before any
/// correspondence is computed, so a lexical declaration appearing after a
/// use-site still corresponds (the forward lexical correspondence theorem,
/// W8 adjacent: this is same-source correspondence, never previous-only or
/// execution-order lookup). A visible selected lexical binding always takes
/// precedence over a same-source `var` contributor of the same name.
fn build_selected_source_name_correspondence(
    accepted: &AcceptedSelectedTopLevelScript,
) -> Vec<SelectedUseSiteRelation> {
    let items = accepted.items();
    let mut lexical_bindings: HashMap<&str, Range> = HashMap::new();
    let mut var_contributors: HashMap<&str, Vec<Range>> = HashMap::new();

    for item in items {
        match item {
            RecognizedTopLevelItem::LexicalBinding(fact) => {
                lexical_bindings
                    .entry(fact.semantic_name.as_str())
                    .or_insert(fact.binding);
            }
            RecognizedTopLevelItem::VarBinding(fact) => {
                var_contributors
                    .entry(fact.semantic_name.as_str())
                    .or_default()
                    .push(fact.binding);
            }
            RecognizedTopLevelItem::UseSite(_) => {}
        }
    }

    let mut relations = Vec::new();
    for item in items {
        let RecognizedTopLevelItem::UseSite(use_site) = item else {
            continue;
        };

        let correspondence = if let Some(binding) = lexical_bindings
            .get(use_site.semantic_name.as_str())
            .copied()
        {
            SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding { binding }
        } else if let Some(contributors) = var_contributors.get(use_site.semantic_name.as_str()) {
            SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
                contributors: contributors.clone(),
            }
        } else {
            SelectedSourceNameCorrespondence::NoSelectedSameSourceContributor
        };

        relations.push(SelectedUseSiteRelation {
            reference: use_site.reference,
            semantic_name: use_site.semantic_name.clone(),
            correspondence,
        });
    }

    relations
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 756);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(
        PREVIOUS_ORDERED_TWO_IDENTIFIER_REFERENCE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 752")
    );
    assert!(FRONTIER_SCOPE_NOTE.contains("free-standing IdentifierReference"));
    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));

    for forbidden in [
        // Production hard-zero (issue #756 section 35): no candidate answer
        // is ever derived from a future production recognizer, accepted
        // witness, or correspondence implementation.
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
        concat!("evaluate_selected_static_", "semantics("),
        concat!("attempt_selected_", "qualification("),
        concat!("analyze_selected_binding_", "scope("),
        concat!(
            "analyze_selected_variable_statement_name_",
            "correspondence("
        ),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_expression_", "statement"),
        // Candidate independence for trivia recognition specifically: only
        // the stable normative Unicode `Space_Separator` property primitive
        // is reused, never the production selected-trivia recognizer.
        concat!("skip_selected_", "trivia("),
        concat!("use super::selected_lexical_", "slice::is_selected_trivia"),
        // No cross-Oracle import: this leaf restates rather than imports.
        concat!(
            "use super::qualification_selected_two_identifier_",
            "reference_additive_initializer_validation_tests"
        ),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
        // The distinct, load-bearing production field this new relation
        // must never reuse or generalize (#688 comment 5729112465).
        concat!("containing_", "binding"),
        concat!("containing_", "source"),
        // No generic Statement/Expression/reference-use architecture.
        concat!("enum Selected", "Statement"),
        concat!("enum Selected", "Expression"),
        concat!("struct Statement", "SourceAnchor"),
        concat!("struct WholeExpression", "SourceAnchor"),
        concat!("struct Semicolon", "SourceAnchor"),
        concat!("struct ScriptItem", "SourceAnchor"),
        concat!("item_", "ordinal"),
        // Runtime resolution vocabulary never appears as executable code
        // (this oracle's module documentation explicitly names these terms
        // in prose per the runtime-negative boundary requirement, so the
        // check here is deliberately code-shaped rather than prose-shaped).
        concat!("Resolve", "Binding("),
        concat!("Get", "Value("),
        concat!("Reference", "Record"),
        concat!("Environment", "Record"),
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
}

/// Required positive direct/escaped family (acceptance criteria 3, 4, 5):
/// every accepted spelling family produces exactly one free-standing
/// use-site item with its own exact authored anchor, decoded semantic name,
/// spelling classification, and `NoSelectedSameSourceContributor`
/// correspondence when no declaration is in scope. No range is ever
/// discovered by search, rescan, reparse, or decoded-length inference: every
/// range below is fixture-owned and independently reconciled through Core
/// `SourceText`.
#[test]
fn positive_direct_escaped_family_pins_exact_provenance_and_no_match() {
    struct PositiveFixture {
        source: &'static str,
        reference: Range,
        expected_authored: &'static str,
        expected_semantic_name: &'static str,
        expected_spelling: IdentifierSpellingKind,
    }

    let fixtures = [
        PositiveFixture {
            source: "a;",
            reference: Range(0, 1),
            expected_authored: "a",
            expected_semantic_name: "a",
            expected_spelling: IdentifierSpellingKind::Direct,
        },
        PositiveFixture {
            source: concat!("\\", "u0061", ";"),
            reference: Range(0, 6),
            expected_authored: concat!("\\", "u0061"),
            expected_semantic_name: "a",
            expected_spelling: IdentifierSpellingKind::EscapedNonReserved,
        },
        PositiveFixture {
            source: concat!("\\", "u{61}", ";"),
            reference: Range(0, 6),
            expected_authored: concat!("\\", "u{61}"),
            expected_semantic_name: "a",
            expected_spelling: IdentifierSpellingKind::EscapedNonReserved,
        },
        PositiveFixture {
            source: concat!("f", "\\", "u006F", "o;"),
            reference: Range(0, 8),
            expected_authored: concat!("f", "\\", "u006F", "o"),
            expected_semantic_name: "foo",
            expected_spelling: IdentifierSpellingKind::EscapedNonReserved,
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        assert_eq!(
            slice(fixture.source, fixture.reference),
            fixture.expected_authored
        );

        let accepted = recognize_and_accept_selected_top_level_script(fixture.source)
            .unwrap_or_else(|| panic!("{:?} must recognize the theorem", fixture.source));
        let items = accepted.items();
        assert_eq!(items.len(), 1);
        let RecognizedTopLevelItem::UseSite(use_site) = &items[0] else {
            panic!(
                "{:?} must recognize a free-standing use-site item",
                fixture.source
            );
        };
        assert_eq!(use_site.reference, fixture.reference);
        assert_eq!(use_site.semantic_name, fixture.expected_semantic_name);
        assert_eq!(use_site.spelling, fixture.expected_spelling);

        // Authored spelling never becomes decoded identity (W12): an
        // escaped occurrence's authored fragment and semantic name differ
        // exactly when it is escaped.
        if fixture.expected_spelling == IdentifierSpellingKind::EscapedNonReserved {
            assert_ne!(fixture.expected_authored, fixture.expected_semantic_name);
        }

        assert_eq!(
            authored_anchor(756_000 + index as u64, fixture.source, fixture.reference),
            fixture.expected_authored
        );

        let relations = build_selected_source_name_correspondence(&accepted);
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].reference, fixture.reference);
        assert_eq!(relations[0].semantic_name, fixture.expected_semantic_name);
        assert_eq!(
            relations[0].correspondence,
            SelectedSourceNameCorrespondence::NoSelectedSameSourceContributor
        );
    }
}

/// Visible lexical binding correspondence and the forward lexical
/// correspondence falsifier together (acceptance criteria 12, 15): a
/// free-standing use-site corresponds to a selected top-level lexical
/// binding of the same semantic name whether that binding is authored
/// before or after the use-site. Same-source correspondence is
/// deliberately not previous-only lookup, not runtime TDZ interpretation,
/// and not execution-order resolution (W8 adjacent).
#[test]
fn lexical_correspondence_and_forward_declaration_are_both_independently_validated() {
    let backward_source = "let a;\na;";
    let backward_accepted = recognize_and_accept_selected_top_level_script(backward_source)
        .expect("`let a; a;` must recognize the theorem");
    let backward_items = backward_accepted.items();
    assert_eq!(backward_items.len(), 2);
    let RecognizedTopLevelItem::LexicalBinding(binding) = &backward_items[0] else {
        panic!("first item must be a lexical binding");
    };
    assert_eq!(binding.binding, Range(4, 5));
    assert_eq!(slice(backward_source, binding.binding), "a");

    let backward_relations = build_selected_source_name_correspondence(&backward_accepted);
    assert_eq!(backward_relations.len(), 1);
    assert_eq!(backward_relations[0].reference, Range(7, 8));
    assert_eq!(
        backward_relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5)
        }
    );

    // Load-bearing forward falsifier: the use-site precedes its lexical
    // declaration in authored order, yet source-name correspondence still
    // identifies it -- this is source correspondence, not runtime
    // resolution, and must never be rejected because the declaration
    // appears later in source.
    let forward_source = "a;\nlet a;";
    let forward_accepted = recognize_and_accept_selected_top_level_script(forward_source)
        .expect("`a; let a;` must recognize the theorem");
    let forward_items = forward_accepted.items();
    assert_eq!(forward_items.len(), 2);
    let RecognizedTopLevelItem::UseSite(use_site) = &forward_items[0] else {
        panic!("first item must be the free-standing use-site");
    };
    assert_eq!(use_site.reference, Range(0, 1));
    let RecognizedTopLevelItem::LexicalBinding(later_binding) = &forward_items[1] else {
        panic!("second item must be the lexical binding");
    };
    assert_eq!(later_binding.binding, Range(7, 8));

    let forward_relations = build_selected_source_name_correspondence(&forward_accepted);
    assert_eq!(forward_relations.len(), 1);
    assert_eq!(forward_relations[0].reference, Range(0, 1));
    assert_eq!(
        forward_relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(7, 8)
        }
    );
}

/// Authored use-site occurrence ordering (acceptance criterion 16, W9):
/// relations follow exact authored free-standing use-site order, never
/// target declaration order, semantic name, or any other reordering.
#[test]
fn authored_use_site_occurrence_order_is_preserved_not_target_declaration_order() {
    let source = "let b;\nlet a;\na;\nb;";
    let accepted = recognize_and_accept_selected_top_level_script(source)
        .expect("`let b; let a; a; b;` must recognize the theorem");
    assert_eq!(accepted.items().len(), 4);

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].semantic_name, "a");
    assert_eq!(relations[1].semantic_name, "b");
    // The `a` use-site is authored before the `b` use-site even though its
    // target `let a` declaration is authored after `let b`.
    assert!(relations[0].reference.0 < relations[1].reference.0);
}

/// Duplicate use-site occurrence theorem (acceptance criterion 17, W10):
/// two authored occurrences of the same semantic name against the same
/// target remain two distinct relations, never collapsed into one.
#[test]
fn duplicate_use_site_occurrences_are_never_deduplicated() {
    let source = "let a;\na;\na;";
    let accepted = recognize_and_accept_selected_top_level_script(source)
        .expect("`let a; a; a;` must recognize the theorem");
    assert_eq!(accepted.items().len(), 3);

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_ne!(relations[0].reference, relations[1].reference);
    assert_eq!(relations[0].semantic_name, relations[1].semantic_name);
    assert_eq!(relations[0].correspondence, relations[1].correspondence);
}

/// Existing initializer relation remains completely separate (acceptance
/// criterion 18, W17): a declaration's `= a` initializer reference is
/// consumed only to recognize the whole bounded item grammar and never
/// becomes a Layer 2 relation of its own. Exactly one relation is produced
/// -- the free-standing `a;` occurrence -- never a combined
/// initializer/use-site stream.
#[test]
fn existing_initializer_relation_remains_separate_from_free_standing_use_site() {
    let source = "let a;\nlet x = a;\na;";
    let accepted = recognize_and_accept_selected_top_level_script(source)
        .expect("`let a; let x = a; a;` must recognize the theorem");
    let items = accepted.items();
    assert_eq!(items.len(), 3);

    let RecognizedTopLevelItem::LexicalBinding(x_binding) = &items[1] else {
        panic!("second item must be the `x` lexical binding");
    };
    assert_eq!(x_binding.semantic_name, "x");

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].semantic_name, "a");
    // The final free-standing `a;`, not the initializer-owned `a` inside
    // `let x = a;`.
    assert_eq!(relations[0].reference, Range(18, 19));
}

/// `var` contributor theorem and multiple-contributor authored order
/// (acceptance criteria 13, 14, W10 adjacent): every same-source selected
/// `var` contributor is retained, in exact authored contributor order,
/// without deduplication.
#[test]
fn var_contributor_theorem_and_multiple_contributor_authored_order() {
    let single_source = "var a;\na;";
    let single_accepted = recognize_and_accept_selected_top_level_script(single_source)
        .expect("`var a; a;` must recognize the theorem");
    let single_relations = build_selected_source_name_correspondence(&single_accepted);
    assert_eq!(single_relations.len(), 1);
    assert_eq!(
        single_relations[0].correspondence,
        SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(4, 5)],
        }
    );

    let multi_source = "var a;\nvar a;\na;";
    let multi_accepted = recognize_and_accept_selected_top_level_script(multi_source)
        .expect("`var a; var a; a;` must recognize the theorem");
    let multi_relations = build_selected_source_name_correspondence(&multi_accepted);
    assert_eq!(multi_relations.len(), 1);
    assert_eq!(
        multi_relations[0].correspondence,
        SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(4, 5), Range(11, 12)],
        }
    );
}

/// Lexical precedence over a same-source `var` contributor (acceptance
/// criterion 12 adjacent): the free-standing use-site corresponds to the
/// selected visible lexical binding, never to an unrelated `var`
/// contributor's initializer relation, and never to a runtime concept.
#[test]
fn lexical_precedence_over_var_contributor_is_independently_validated() {
    let source = "let a;\nvar x = a;\na;";
    let accepted = recognize_and_accept_selected_top_level_script(source)
        .expect("`let a; var x = a; a;` must recognize the theorem");
    let items = accepted.items();
    assert_eq!(items.len(), 3);

    let RecognizedTopLevelItem::VarBinding(x_binding) = &items[1] else {
        panic!("second item must be the `x` var binding");
    };
    assert_eq!(x_binding.semantic_name, "x");

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].reference, Range(18, 19));
    assert_eq!(
        relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5)
        }
    );
}

/// Static preflight gate (Issue #756 remediation): the surrounding
/// already-selected declaration shell must independently pass the two
/// existing top-level static rules relevant to it before any relation is
/// ever committed, even though the free-standing use-site itself
/// contributes no declaration names. A duplicate selected lexical name is
/// rejected; a selected lexical name colliding with a selected top-level
/// `var` name is rejected in either authored order; duplicate `var`
/// contributors alone remain allowed. Ordinary positive and forward
/// correspondence remain unaffected by the gate.
#[test]
fn static_preflight_gate_rejects_duplicate_lexical_and_lexical_var_collision_shells() {
    // Required counterexamples: no accepted witness, and therefore no
    // relation, is produced for any of these statically rejected shells,
    // even though each is syntactically well-formed under Layer 1 alone.
    // Each is directly bound to `StaticSemanticsRejected`, never downgraded
    // to `UnsupportedCoverage`: this is a source shape the theorem
    // recognized but independently knows violates an already-accepted
    // static rule, a materially distinct cause from a source shape outside
    // the theorem entirely.
    let duplicate_lexical_source = "let a;\nlet a;\na;";
    assert!(parse_selected_top_level_script(duplicate_lexical_source).is_some());
    assert!(recognize_and_accept_selected_top_level_script(duplicate_lexical_source).is_none());
    match classify_selected_top_level_script(duplicate_lexical_source) {
        SelectedTopLevelScriptDisposition::StaticSemanticsRejected(
            StaticPreflightRejection::DuplicateLexical {
                semantic_name,
                first,
                duplicate,
            },
        ) => {
            assert_eq!(semantic_name, "a");
            assert_eq!(first, Range(4, 5));
            assert_eq!(duplicate, Range(11, 12));
        }
        other => panic!(
            "{duplicate_lexical_source:?} must classify as StaticSemanticsRejected(DuplicateLexical), got {other:?}"
        ),
    }

    for (source, expected_lexical, expected_var) in [
        ("let a;\nvar a;\na;", Range(4, 5), Range(11, 12)),
        ("var a;\nlet a;\na;", Range(11, 12), Range(4, 5)),
    ] {
        assert!(
            parse_selected_top_level_script(source).is_some(),
            "{source:?} must be syntactically recognized by Layer 1 alone"
        );
        assert!(
            recognize_and_accept_selected_top_level_script(source).is_none(),
            "{source:?} must be rejected by the static preflight gate"
        );
        match classify_selected_top_level_script(source) {
            SelectedTopLevelScriptDisposition::StaticSemanticsRejected(
                StaticPreflightRejection::LexicalVarCollision {
                    semantic_name,
                    lexical,
                    var,
                },
            ) => {
                assert_eq!(semantic_name, "a");
                assert_eq!(lexical, expected_lexical);
                assert_eq!(var, expected_var);
            }
            other => panic!(
                "{source:?} must classify as StaticSemanticsRejected(LexicalVarCollision), got {other:?}"
            ),
        }
    }

    // Preserved acceptance: duplicate `var` contributors alone remain
    // allowed, with both anchors retained in authored contributor order,
    // and the fixture is directly bound to `Selected`, not merely to
    // `recognize_and_accept_selected_top_level_script(..).is_some()`.
    let preserved_source = "var a;\nvar a;\na;";
    let preserved_accepted = match classify_selected_top_level_script(preserved_source) {
        SelectedTopLevelScriptDisposition::Selected(accepted) => accepted,
        other => panic!(
            "`var a; var a; a;` must classify as Selected (duplicate var contributors alone are allowed), got {other:?}"
        ),
    };
    let preserved_relations = build_selected_source_name_correspondence(&preserved_accepted);
    assert_eq!(preserved_relations.len(), 1);
    assert_eq!(
        preserved_relations[0].correspondence,
        SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(4, 5), Range(11, 12)],
        }
    );

    // Preserved ordinary positive and forward source correspondence: the
    // static preflight gate never rejects a shell with no colliding or
    // duplicate lexical name, whatever the use-site's authored position
    // relative to its target declaration.
    assert!(recognize_and_accept_selected_top_level_script("let a;\na;").is_some());
    let forward_accepted = recognize_and_accept_selected_top_level_script("a;\nlet a;")
        .expect("`a; let a;` forward correspondence must remain accepted by the gate");
    let forward_relations = build_selected_source_name_correspondence(&forward_accepted);
    assert_eq!(forward_relations.len(), 1);
    assert_eq!(
        forward_relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(7, 8)
        }
    );
}

/// No-selected-same-source-contributor theorem (acceptance criterion 14):
/// this means only that no contributor exists inside the selected
/// same-source model. It explicitly does not mean runtime-unresolvable, a
/// `ReferenceError`, a global-property miss, or a `ResolveBinding` failure.
#[test]
fn no_selected_same_source_contributor_is_not_runtime_unbound() {
    let accepted = recognize_and_accept_selected_top_level_script("a;")
        .expect("`a;` must recognize the theorem");
    assert_eq!(accepted.items().len(), 1);

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(
        relations[0].correspondence,
        SelectedSourceNameCorrespondence::NoSelectedSameSourceContributor
    );

    // The `NoSelectedSameSourceContributor` variant carries no payload: it
    // is a same-source-model absence fact only, never a runtime
    // unresolvable/`ReferenceError`/`ResolveBinding`-failure claim (see
    // module documentation for the full runtime-negative boundary).
}

/// Authored-semicolon theorem and the ASI firewall together (acceptance
/// criteria 6, 7): only an authored `;` terminator is selected. A
/// valid-but-unselected ASI boundary -- EOF-terminated or a materially
/// representative line-termination case -- remains `UnsupportedCoverage`,
/// never `DefinitiveGrammarRejectionEvidence`: each control below has a
/// leading identifier-shaped prefix that independently recognizes as a
/// valid `SelectedAcceptedIdentifierReference` on its own, proving the
/// whole-item failure is caused only by the missing authored terminator,
/// not by an invalid identifier shape (W14).
#[test]
fn authored_semicolon_theorem_and_asi_boundaries_remain_unsupported_coverage() {
    // The fixture is directly bound to its executable disposition, not
    // merely to a disconnected symbolic classification list (Issue #756
    // remediation): `a;` independently classifies as `Selected`, and each
    // valid-but-unselected ASI boundary independently classifies as
    // `UnsupportedCoverage`, never `DefinitiveGrammarRejectionEvidence`.
    match classify_selected_top_level_script("a;") {
        SelectedTopLevelScriptDisposition::Selected(accepted) => {
            assert_eq!(accepted.items().len(), 1);
        }
        other => panic!("`a;` must classify as Selected, got {other:?}"),
    }

    for (source, leading_identifier) in [("a", "a"), ("a\nlet b;", "a")] {
        assert_eq!(
            classify_selected_top_level_script(source),
            SelectedTopLevelScriptDisposition::UnsupportedCoverage,
            "{source:?} is a valid-but-unselected ASI boundary, not part of this theorem, \
             and must never classify as DefinitiveGrammarRejectionEvidence"
        );
        // The failure is caused only by the missing authored terminator,
        // never by an invalid identifier shape: the same leading text
        // independently recognizes as a valid
        // `SelectedAcceptedIdentifierReference` on its own.
        assert!(
            recognize_accepted_identifier_reference(leading_identifier).is_some(),
            "{source:?} must fail only for the missing authored semicolon"
        );
    }

    assert!(FAILURE_CLASSIFICATIONS.contains(&"UnsupportedCoverage"));
    assert!(FAILURE_CLASSIFICATIONS.contains(&"DefinitiveGrammarRejectionEvidence"));
    assert_ne!(
        FAILURE_CLASSIFICATIONS[0], FAILURE_CLASSIFICATIONS[1],
        "UnsupportedCoverage and DefinitiveGrammarRejectionEvidence must stay distinct"
    );

    // No semicolon `SourceAnchor` is required by the relation: the relation
    // struct carries exactly `reference`, `semantic_name`, and
    // `correspondence`.
    assert!(THIS_ORACLE_SOURCE.contains("struct SelectedUseSiteRelation"));
    for forbidden in [
        concat!("semi", "colon: Range"),
        concat!("semicolon_", "anchor"),
        concat!("termin", "ator: Range"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }
}

/// The disposition enum preserves the project's failure vocabulary as a
/// real, executable distinction, not only as a name in a symbolic list:
/// `UnsupportedCoverage`, `StaticSemanticsRejected`,
/// `DefinitiveGrammarRejectionEvidence`, `ResourceLimited`, and
/// `InternalFailure` remain pairwise distinct values. In particular,
/// `StaticSemanticsRejected` is never collapsed into `UnsupportedCoverage`:
/// a source shape the theorem recognized but independently knows violates
/// an already-accepted static rule is a materially different cause than a
/// source shape outside the theorem entirely.
#[test]
fn disposition_enum_preserves_the_failure_vocabulary_distinction() {
    let sample_rejection = StaticPreflightRejection::DuplicateLexical {
        semantic_name: "a".to_owned(),
        first: Range(0, 0),
        duplicate: Range(0, 0),
    };

    assert_ne!(
        SelectedTopLevelScriptDisposition::UnsupportedCoverage,
        SelectedTopLevelScriptDisposition::ResourceLimited
    );
    assert_ne!(
        SelectedTopLevelScriptDisposition::UnsupportedCoverage,
        SelectedTopLevelScriptDisposition::InternalFailure
    );
    assert_ne!(
        SelectedTopLevelScriptDisposition::ResourceLimited,
        SelectedTopLevelScriptDisposition::InternalFailure
    );
    assert_ne!(
        SelectedTopLevelScriptDisposition::UnsupportedCoverage,
        SelectedTopLevelScriptDisposition::DefinitiveGrammarRejectionEvidence(Range(0, 0))
    );
    assert_ne!(
        SelectedTopLevelScriptDisposition::UnsupportedCoverage,
        SelectedTopLevelScriptDisposition::StaticSemanticsRejected(sample_rejection.clone())
    );
    assert_ne!(
        SelectedTopLevelScriptDisposition::StaticSemanticsRejected(sample_rejection.clone()),
        SelectedTopLevelScriptDisposition::DefinitiveGrammarRejectionEvidence(Range(0, 0))
    );
    assert_ne!(
        SelectedTopLevelScriptDisposition::StaticSemanticsRejected(sample_rejection.clone()),
        SelectedTopLevelScriptDisposition::ResourceLimited
    );
    assert_ne!(
        SelectedTopLevelScriptDisposition::StaticSemanticsRejected(sample_rejection),
        SelectedTopLevelScriptDisposition::InternalFailure
    );

    // The two static-preflight rejection reasons are themselves distinct.
    assert_ne!(
        StaticPreflightRejection::DuplicateLexical {
            semantic_name: "a".to_owned(),
            first: Range(0, 0),
            duplicate: Range(0, 0),
        },
        StaticPreflightRejection::LexicalVarCollision {
            semantic_name: "a".to_owned(),
            lexical: Range(0, 0),
            var: Range(0, 0),
        }
    );

    // This bounded recognizer never independently constructs
    // `DefinitiveGrammarRejectionEvidence` in practice (doing so would
    // require duplicating production's already-owned grammar-evidence
    // machinery); a source shape entirely outside the theorem classifies as
    // `UnsupportedCoverage`.
    assert_eq!(
        classify_selected_top_level_script("a + b;"),
        SelectedTopLevelScriptDisposition::UnsupportedCoverage
    );
}

/// Escaped `ReservedWord` boundary (acceptance criterion 8, W13): a decoded
/// unconditionally reserved word never becomes a selected free-standing
/// use-site.
#[test]
fn escaped_reserved_word_firewall_emits_no_accepted_use_site_relation() {
    let source = concat!("\\", "u0069", "f;");
    assert!(
        parse_selected_top_level_script(source).is_none(),
        "{source:?} decodes to the ReservedWord `if`"
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0069", "f")),
        Err(DecodeFailure::DecodedReserved)
    );
}

/// Malformed / invalid escape firewall (acceptance criterion 9): no
/// accepted use-site relation leaks from a malformed or position-invalid
/// candidate, and no valid-prefix truncation ever occurs.
#[test]
fn malformed_and_invalid_escape_firewall_never_leaks_a_use_site_relation() {
    // Built with individually concatenated fragments (never one contiguous
    // raw `\uXXXX` literal) so the literal bytes `\`, `u`, and the exact
    // hex digits survive intact through the tooling pipeline rather than
    // being collapsed into a decoded Unicode scalar by an intermediate
    // writer (Issue #756 section 12/28 corruption-resistant fixture
    // pattern; the four-hex-digit classic form is exactly the shape at
    // risk, unlike the already-protected braced positive fixtures above).
    let leading_zero_digit = concat!("\\", "u0030", ";");
    let mid_hyphen = concat!("a", "\\", "u002D", "b;");
    for source in [
        r"\u{};",
        r"\uD800;",
        r"\u{110000};",
        leading_zero_digit,
        mid_hyphen,
    ] {
        assert!(
            parse_selected_top_level_script(source).is_none(),
            "{source:?}"
        );
    }

    assert_eq!(
        decode_selected_escaped_identifier(r"\u{}"),
        Err(DecodeFailure::MalformedEscape)
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
        decode_selected_escaped_identifier(concat!("\\", "u0030")),
        Err(DecodeFailure::InvalidStart)
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("a", "\\", "u002D")),
        Err(DecodeFailure::InvalidPart)
    );

    // Verify the final test source truly contains the intended backslash-u
    // bytes rather than a decoded scalar (Issue #756 section 12 requires
    // this be checked, not merely assumed).
    assert!(leading_zero_digit.starts_with('\\'));
    assert_eq!(leading_zero_digit.as_bytes()[1], b'u');
    assert!(mid_hyphen.contains('\\'));
    assert_eq!(mid_hyphen.len(), "a\\u002Db;".len());
}

/// General-expression firewall (acceptance criterion 10, W15): richer
/// Expression neighbors remain outside this theorem without any generic
/// Expression parser, and no valid-prefix `a` is ever accepted from a
/// richer expression.
#[test]
fn general_expression_neighbor_firewall_keeps_richer_syntax_outside() {
    for source in [
        "a + b;",
        "+a;",
        "-a;",
        "(a);",
        "a.b;",
        "a[b];",
        "a();",
        "a = b;",
        "a ? b : c;",
        "a && b;",
        "a, b;",
        "new a;",
    ] {
        assert!(
            parse_selected_top_level_script(source).is_none(),
            "{source:?}"
        );
    }

    // The bounded theorem itself remains independently valid; only the
    // richer neighbors above are unowned.
    assert!(parse_selected_top_level_script("a;").is_some());
}

/// Placement firewall (acceptance criterion 2): this theorem is top-level
/// non-strict `Script` only. Nested Block, function-body, class-body,
/// Module, and strict-mode-widened use-sites remain outside without any
/// generic `StatementList` architecture in this oracle (W16).
#[test]
fn placement_firewall_keeps_nested_and_strict_mode_forms_outside() {
    for source in [
        "{ a; }",
        "function f() { a; }",
        "class C { m() { a; } }",
        "\"use strict\";\na;",
    ] {
        assert!(
            parse_selected_top_level_script(source).is_none(),
            "{source:?}"
        );
    }
}

/// Whole-source transactionality (acceptance criterion 23, W23): a locally
/// valid earlier use-site never publishes relation evidence when later
/// source prevents whole-source acceptance.
#[test]
fn whole_source_transactionality_never_leaks_earlier_valid_relations() {
    let valid_prefix = "let a;\na;";
    assert!(parse_selected_top_level_script(valid_prefix).is_some());

    for source in ["let a;\na;\n???", "let a;\na;\nlet x = ;"] {
        assert!(
            parse_selected_top_level_script(source).is_none(),
            "{source:?} must publish no relation evidence at all, not even for the valid `let a; a;` prefix"
        );
    }
}

/// Resource / failure semantics (acceptance criterion 24, W22): the
/// project's `ResourceLimited`/`InternalFailure` lifecycle states remain
/// distinct from `UnsupportedCoverage` and from
/// `DefinitiveGrammarRejectionEvidence`, matching the minimal symbolic
/// model already accepted by the predecessor candidate-independent
/// `IdentifierReference` oracles -- this leaf adds no further resource
/// machinery beyond that already-accepted minimum.
#[test]
fn resource_and_internal_failure_lifecycle_states_remain_distinct() {
    assert_eq!(PROCESSING_FAILURES, &["ResourceLimited", "InternalFailure"]);
    assert_eq!(FAILURE_CLASSIFICATIONS.len(), 4);
    assert!(THIS_ORACLE_SOURCE.contains("ResourceLimited"));
    assert!(THIS_ORACLE_SOURCE.contains("InternalFailure"));
    for failure in PROCESSING_FAILURES {
        assert_ne!(*failure, "UnsupportedCoverage");
        assert_ne!(*failure, "DefinitiveGrammarRejectionEvidence");
    }
}

/// Freezes the presence-only, validation-only handoff: no existing
/// initializer-relation declaration-owner field widening, no generic
/// `Statement`/`Expression`/reference-use abstraction,
/// no completion movement, and no runtime resolution/evaluation vocabulary
/// is introduced (acceptance criteria 19, 20, 21, 22, 25, 27; W1-W7, W18-W22,
/// W24-W25).
#[test]
fn handoff_remains_validation_only_with_no_runtime_or_generic_architecture() {
    for forbidden in [
        concat!("Get", "Value("),
        concat!("Resolve", "Binding("),
        concat!("Reference", "Record"),
        concat!("Environment", "Record"),
        concat!("Bound", "Names("),
        concat!("VarDeclared", "Names("),
        concat!("LexicallyDeclared", "Names("),
        concat!("GlobalDeclaration", "Instantiation"),
        concat!("hois", "ting"),
        concat!("TO", "TAL"),
        concat!("NEWLY_", "REACHABLE"),
        concat!("Qual", "ified"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    // The relation type carries exactly the theorem's vocabulary.
    assert!(THIS_ORACLE_SOURCE.contains("VisibleSelectedLexicalBinding"));
    assert!(THIS_ORACLE_SOURCE.contains("SameSourceSelectedVarNameContributors"));
    assert!(THIS_ORACLE_SOURCE.contains("NoSelectedSameSourceContributor"));

    // Static neutrality (acceptance criterion 26): this oracle never
    // computes or claims BoundNames, LexicallyDeclaredNames, or
    // VarDeclaredNames contribution for the free-standing use-site -- its
    // relation carries no such field, and no production static-semantics
    // path is ever consulted (checked separately by the forbidden-import
    // list in `authority_independence_and_frontier_scope_are_exact`).
    assert!(THIS_ORACLE_SOURCE.contains("struct RecognizedUseSiteFact"));

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
