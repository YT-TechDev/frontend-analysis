//! Candidate-independent one-level Block `IdentifierReference`
//! `ExpressionStatement` use-site validation for Issue #760 (durable
//! research: Issue #688 comment `5732434770`; accepted predecessor
//! authorities: Issue #756 / PR #757 top-level free-standing
//! `IdentifierReference` use-site oracle and #758 / PR #759 top-level
//! production, Issue #283 / #302 / PR #303 / #304 one-level Block
//! hierarchical Binding / Scope authority, Issue #705-#710 lineage
//! region-aware source-name correspondence and whole-Script selected-`var`
//! contributor provenance).
//!
//! This oracle qualifies only the bounded theorem:
//!
//! ```text
//! SelectedOneLevelBlockIdentifierReferenceExpressionStatement ::=
//!     SelectedAcceptedIdentifierReference
//!     AuthoredSemicolon
//!
//! SelectedAcceptedIdentifierReference ::=
//!     SelectedDirectIdentifierReference
//!   | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! placed as:
//!
//! ```text
//! non-strict Script
//! -> one already-selected one-level Block
//! -> one Block StatementListItem
//! -> free-standing IdentifierReference ExpressionStatement
//! ```
//!
//! It does not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code, and it does not call
//! or import any other candidate-independent Oracle's recognizer or
//! correspondence functions -- each accepted `IdentifierReference` Oracle
//! restates its own copy of the primitives it needs.
//!
//! The accepted predecessor authorities each proved one ingredient in
//! isolation: #756/#757 proved a free-standing top-level use-site with
//! top-level-only source-name correspondence; #302/#303/#304 proved a
//! one-level Block's nearest selected lexical target for an
//! *initializer-owned* reference; #705-#710 proved region-aware
//! correspondence together with whole-Script selected-`var` contributor
//! provenance. No accepted authority yet proves the combined theorem this
//! oracle exists to freeze: a free-standing use-site, with no containing
//! binding, placed inside one already-selected one-level Block, corresponding
//! first to that Block's own selected lexical binding, then to the enclosing
//! TopLevel selected lexical binding, then to every same-source selected
//! `var` contributor across the whole Script (Block-contained or top-level),
//! and otherwise to no selected same-source contributor at all. Production
//! must not become the first authority for this combined theorem.
//!
//! The independent lifecycle mirrors #756/#757's three-stage shape (`prepare
//! candidate evidence -> authoritative whole-source consumption ->
//! static-neutral accepted witness -> non-refusing relation commit`), driven
//! end to end by `recognize_and_accept_selected_script`:
//!
//!   - Layer 1 (`parse_selected_script`): source / placement. Recognizes a
//!     whole candidate `Script` as an ordered sequence of exactly four
//!     bounded top-level item shapes -- a `let` binding item, a `var`
//!     binding item, a free-standing `IdentifierReference`
//!     `ExpressionStatement` use-site item, or one already-selected
//!     one-level Block -- each terminated by an authored `;` (the Block item
//!     is terminated by its own authored `}`). A Block's own contents are
//!     recognized as an ordered sequence of exactly the same three
//!     non-Block item shapes: this oracle's Block is bounded to exactly one
//!     level, so encountering `{` while recognizing a Block's own contents
//!     is never attempted and aborts the whole parse, never silently
//!     skipped or specially rejected. Recognition is whole-source,
//!     all-or-nothing at every level: any unrecognized item -- top-level or
//!     Block-contained -- aborts the complete parse (`None`), so a locally
//!     valid earlier item can never publish evidence when a later item fails
//!     (the whole-source transactionality theorem).
//!
//!   - Static preflight (`preflight_selected_static_semantics`): a bounded,
//!     candidate-independent gate over only the existing rules relevant to
//!     this shell, restated per accepted region-ownership semantics rather
//!     than widened: a duplicate selected lexical name is rejected within
//!     its own region (TopLevel among itself; each Block among its own
//!     items only -- sibling Block lexical names never collide with each
//!     other); a TopLevel selected lexical name colliding with any
//!     whole-Script selected `var` name (TopLevel or Block-contributed) is
//!     rejected, matching the already-accepted Script
//!     `LexicallyDeclaredNames`/`VarDeclaredNames` collision rule now that a
//!     `var` may be Block-contributed; and a Block's own selected lexical
//!     name colliding with that same Block's own selected `var` name is
//!     rejected, matching the already-accepted Block-local collision rule.
//!     Duplicate `var` contributors alone remain allowed, at any region. A
//!     syntactically recognized but statically rejected candidate never
//!     reaches Layer 2, and its rejection is never collapsed into the same
//!     outcome as a candidate whose source shape never matched this theorem
//!     at all. This is not a general Early Error engine and never imports
//!     production static semantics.
//!
//!   - Layer 2 (`build_selected_source_name_correspondence`): relation.
//!     Consumes only the accepted witness (never an arbitrary item list, and
//!     never a production accepted-witness type) and independently derives
//!     exactly one correspondence relation per free-standing use-site item
//!     -- top-level or Block-contained -- in exact authored occurrence
//!     order, without deduplication. Every relation independently retains
//!     its own containing region (`TopLevel` or the exact containing
//!     `Block`'s own authored anchor), so top-level and Block placements
//!     remain distinguishable without inventing one new global
//!     initializer/top-level/Block relation ordering contract: this oracle
//!     never touches the existing, completely separate initializer
//!     correspondence relation owned by
//!     `selected_variable_statement_name_correspondence.rs`.
//!
//! `classify_selected_script` binds this whole lifecycle to an executable
//! disposition (`Selected` / `UnsupportedCoverage` / `StaticSemanticsRejected`
//! / `DefinitiveGrammarRejectionEvidence` / `ResourceLimited` /
//! `InternalFailure`), matching the vocabulary and distinctions already
//! established by #756/#757: a valid-but-unselected close-brace ASI boundary
//! such as `{ a }` independently classifies as `UnsupportedCoverage` (never
//! grammar rejection, and never composing #717's automatic-semicolon-before-
//! close-brace precedent), while a syntactically well-formed but statically
//! rejected shell independently classifies as `StaticSemanticsRejected`
//! instead.
//!
//! `let`/`var` binding items may carry an `= SelectedAcceptedIdentifierReference`
//! initializer purely so the whole-item grammar can be recognized end to
//! end; that inner reference is never retained as a fact and never becomes
//! part of Layer 2's relation stream (existing initializer relation
//! authority remains completely separate and is never imported here).
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
//! This is a validation-only leaf: production supports no Block-contained
//! `ExpressionStatement` family at the #760 baseline (historical
//! `SelectedBlockItem = LexicalDeclaration | Var` remains completely
//! unmodified by this Issue), so every positive fixture below remains
//! `UnsupportedCoverage` under current production and exists only as
//! independent Oracle evidence for a future, separately authorized
//! production decision. No completion successor file accompanies this leaf:
//! this Issue adds zero production capability, so the frozen `193 / 10 / 183
//! / {}` completion partition cannot move, matching the precedent set by
//! #742/#743, #746/#747, #752/#753, and #756/#757. This oracle intentionally
//! does not decide whether a future production representation widens the
//! historical `SelectedBlockItem`, introduces a new use-site-enabled Block
//! representation, or introduces a new broadest Script carrier: that
//! decision belongs to a separate post-Oracle production representation /
//! placement step.
//!
//! Runtime-negative boundary: `NoSelectedSameSourceContributor` is not
//! runtime-unbound, a `ReferenceError`, or a `ResolveBinding` failure.
//! `VisibleSelectedLexicalBinding` is not `ResolveBinding`, `GetValue`, a
//! Reference Record, an Environment Record, or TDZ execution state. Forward
//! lexical source correspondence -- a use-site preceding its target
//! declaration in authored order -- is explicitly source correspondence
//! only, never a runtime-TDZ-success claim. This oracle proves same-source
//! selected declaration provenance only.

use std::collections::HashMap;

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 760;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREDECESSOR_TOP_LEVEL_USE_SITE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_top_level_identifier_reference_expression_statement_use_site_validation_tests.rs"
);
const PREDECESSOR_BLOCK_BINDING_SCOPE_ORACLE_SOURCE: &str =
    include_str!("selected_one_level_block_binding_scope_validation_tests.rs");
const PREDECESSOR_BLOCK_ORACLE_SOURCE: &str =
    include_str!("qualification_selected_one_level_block_validation_tests.rs");
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_one_level_block_identifier_reference_expression_statement_use_site_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "one-level Block free-standing IdentifierReference ExpressionStatement ",
    "use-site frontier only; later independently qualified owners may ",
    "strengthen classification for recursive/deeper Block nesting, ",
    "function/class body use-sites, Module items, strict-mode widening, ",
    "close-brace or general ASI-terminated use-sites, or richer Expression ",
    "neighbors"
);

/// Preserved distinctly from `UnsupportedCoverage`, matching the minimal
/// symbolic model already accepted by the predecessor candidate-independent
/// `IdentifierReference` oracles (#241, #746/#747, #752/#753, #756/#757) --
/// this leaf adds no further resource machinery beyond that already-accepted
/// minimum.
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];

/// The four failure classifications this oracle keeps textually distinct: a
/// valid-but-unselected close-brace ASI boundary is `UnsupportedCoverage`,
/// never `DefinitiveGrammarRejectionEvidence`.
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
// #237/#756, never imported from either oracle. ---

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

// --- The Issue #760 theorem itself. ---

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

/// The current containing region of one recognized reference or one static
/// rejection cause: either the enclosing top-level `Script`, or the exact
/// containing one-level Block's own authored anchor. Deliberately not an
/// `Option<_>` -- `TopLevel` is a first-class region, never an absent Block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedRegion {
    TopLevel,
    Block(Range),
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
/// to validate. Its containing region is never stored here -- it is
/// determined only by where the fact was recognized (top-level sequence or
/// a specific Block's own item sequence), matching the accepted predecessor
/// convention of keeping the recognized fact itself free of any
/// generic/optional containing-owner field (W5/W6).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedUseSiteFact {
    reference: Range,
    semantic_name: String,
    spelling: IdentifierSpellingKind,
}

/// One non-Block item shape shared by both the top-level sequence and a
/// Block's own contents: a minimal private test representation scoped only
/// to this oracle's exact theorem, never a claim that future production
/// must use this exact enum, a generic `Statement` hierarchy, or any
/// particular Rust type layout.
#[derive(Debug, Clone, PartialEq, Eq)]
enum RecognizedNonBlockItem {
    LexicalBinding(RecognizedBindingFact),
    VarBinding(RecognizedBindingFact),
    UseSite(RecognizedUseSiteFact),
}

/// One recognized one-level Block: its own exact authored anchor (spanning
/// both braces) and its own ordered item sequence. This oracle's Block is
/// bounded to exactly one level: `items` never contains a nested Block
/// variant, and recognizing a Block's own contents never attempts to
/// recognize `{` as a fourth shape (no recursive/deeper Block support).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedBlockFact {
    anchor: Range,
    items: Vec<RecognizedNonBlockItem>,
}

/// The oracle's only Layer 1 retained top-level representation: a private,
/// bounded, four-variant item shape (the three non-Block shapes, plus one
/// already-selected one-level Block).
#[derive(Debug, Clone, PartialEq, Eq)]
enum RecognizedTopLevelItem {
    LexicalBinding(RecognizedBindingFact),
    VarBinding(RecognizedBindingFact),
    UseSite(RecognizedUseSiteFact),
    Block(RecognizedBlockFact),
}

/// Recognizes one `let`/`var` binding item starting at `offset`: the
/// keyword, a mandatory trivia boundary, a direct `BindingIdentifier`, an
/// optional `= SelectedAcceptedIdentifierReference` initializer, and a
/// mandatory authored `;`. Returns the recognized fact and the absolute
/// offset immediately after the authored `;`. Reused identically for a
/// top-level binding item and a Block-contained binding item -- the grammar
/// shape itself never differs by region.
fn parse_binding(
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

/// Recognizes one free-standing
/// `SelectedOneLevelBlockIdentifierReferenceExpressionStatement`-shaped
/// use-site item starting at `offset`: the theorem itself, minus its
/// placement (placement is judged only by which sequence this function is
/// called from -- top-level or a Block's own contents). Returns the
/// recognized fact and the absolute offset immediately after the authored
/// `;`. Reused identically for a top-level use-site and a Block-contained
/// use-site.
fn parse_use_site(source: &str, offset: usize) -> Option<(RecognizedUseSiteFact, usize)> {
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

/// Recognizes one non-Block item (`let`/`var` binding or free-standing
/// use-site) at `offset`, in that dispatch order -- matching the already
/// accepted #756/#757 precedence so `{ let a; }` / `{ var a; }` remain
/// recognized as declarations rather than reclassified, while `{ let; }` /
/// `{ varfoo; }` fall through to the free-standing use-site route exactly
/// when the keyword attempt cannot form a complete binding (no generic
/// Statement dispatch is introduced: this is the same three-way attempt
/// order already accepted by the top-level oracle).
fn parse_non_block_item(source: &str, offset: usize) -> Option<(RecognizedNonBlockItem, usize)> {
    if let Some((fact, end)) = parse_binding(source, offset, "let") {
        return Some((RecognizedNonBlockItem::LexicalBinding(fact), end));
    }
    if let Some((fact, end)) = parse_binding(source, offset, "var") {
        return Some((RecognizedNonBlockItem::VarBinding(fact), end));
    }
    let (fact, end) = parse_use_site(source, offset)?;
    Some((RecognizedNonBlockItem::UseSite(fact), end))
}

/// Recognizes one already-selected one-level Block starting at `offset`:
/// an authored `{`, an ordered sequence of zero or more non-Block items,
/// and an authored `}`. Never attempts to recognize a nested `{` as a
/// fourth Block-contained item shape -- a Block containing another Block
/// is never matched, aborting the whole parse (`None`), which is exactly
/// how this oracle keeps itself bounded to one level without any special
/// nested-Block rejection code. Returns the recognized fact (its own
/// anchor spans both braces) and the absolute offset immediately after the
/// authored `}`.
fn parse_block(source: &str, offset: usize) -> Option<(RecognizedBlockFact, usize)> {
    if !source[offset..].starts_with('{') {
        return None;
    }

    let mut cursor = offset + 1;
    cursor += trivia_run_end(&source[cursor..]);

    let mut items = Vec::new();
    while !source[cursor..].starts_with('}') {
        if cursor >= source.len() {
            return None;
        }
        let (item, next_offset) = parse_non_block_item(source, cursor)?;
        items.push(item);
        cursor = next_offset;
        cursor += trivia_run_end(&source[cursor..]);
    }

    let end = cursor + 1;
    Some((
        RecognizedBlockFact {
            anchor: Range(offset, end),
            items,
        },
        end,
    ))
}

/// Central Issue #760 Layer 1 theorem. Whole-source, all-or-nothing at
/// every level: a top-level item sequence is never returned unless every
/// top-level item, in exact authored order, recognizes as one of the four
/// bounded shapes with nothing left over, and a Block item is never
/// returned unless every one of its own contained items recognizes as one
/// of the three bounded non-Block shapes with nothing left over. A single
/// unrecognized item at any level -- a richer Expression neighbor, a
/// nested Block, a function/class body, an escaped ReservedWord, a
/// malformed escape, a close-brace-only ASI boundary, or trailing
/// malformed source -- aborts the complete parse; no earlier valid item is
/// ever returned on its own (whole-source transactionality).
fn parse_selected_script(source: &str) -> Option<Vec<RecognizedTopLevelItem>> {
    let mut offset = trivia_run_end(source);
    let mut items = Vec::new();

    while offset < source.len() {
        let (item, next_offset) = if let Some((block, end)) = parse_block(source, offset) {
            (RecognizedTopLevelItem::Block(block), end)
        } else if let Some((item, end)) = parse_non_block_item(source, offset) {
            let item = match item {
                RecognizedNonBlockItem::LexicalBinding(fact) => {
                    RecognizedTopLevelItem::LexicalBinding(fact)
                }
                RecognizedNonBlockItem::VarBinding(fact) => {
                    RecognizedTopLevelItem::VarBinding(fact)
                }
                RecognizedNonBlockItem::UseSite(fact) => RecognizedTopLevelItem::UseSite(fact),
            };
            (item, end)
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
/// passed the existing static rules relevant to this shell. Layer 2 can
/// only consume this witness, never an arbitrary `Vec<RecognizedTopLevelItem>`
/// -- this is a minimal private representation scoped to this oracle's
/// exact bounded shell, never a claim about a future production
/// accepted-witness type or layout.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AcceptedSelectedScript {
    items: Vec<RecognizedTopLevelItem>,
}

impl AcceptedSelectedScript {
    fn items(&self) -> &[RecognizedTopLevelItem] {
        &self.items
    }
}

/// One independently derived static-preflight rejection reason: the
/// existing static rules relevant to this bounded shell, restated per
/// accepted region-ownership semantics, each carrying the exact anchors and
/// owning region that independently proved it. This is a minimal private
/// representation, never a claim about a future production Early Error type
/// layout.
#[derive(Debug, Clone, PartialEq, Eq)]
enum StaticPreflightRejection {
    DuplicateLexical {
        semantic_name: String,
        first: Range,
        duplicate: Range,
        region: SelectedRegion,
    },
    LexicalVarCollision {
        semantic_name: String,
        lexical: Range,
        var: Range,
        region: SelectedRegion,
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
    Accepted(AcceptedSelectedScript),
    Rejected(StaticPreflightRejection),
}

/// Independently restates -- never imports -- only the existing static
/// rules relevant to this bounded shell:
///
///   - a duplicate selected lexical name is rejected within its own region
///     only (TopLevel among itself; each Block among its own items only --
///     sibling Block lexical names never collide, matching the accepted
///     region-ownership theorem);
///   - a TopLevel selected lexical name colliding with any whole-Script
///     selected `var` name (TopLevel- or Block-contributed) is rejected,
///     matching the already-accepted Script
///     `LexicallyDeclaredNames`/`VarDeclaredNames` collision rule now that a
///     `var` contributor may originate inside a Block;
///   - a Block's own selected lexical name colliding with that same Block's
///     own selected `var` name is rejected, matching the already-accepted
///     Block-local collision rule (a Block's own lexical name never
///     collides with an unrelated outer or sibling `var`).
///
/// Duplicate `var` contributors alone remain allowed, at any region: the
/// first authored anchor per `var` name per region is recorded only for
/// collision detection, never by count. Every pass walks items in authored
/// order, so reported rejection anchors are deterministic regardless of any
/// internal map iteration order. This is not a general Early Error engine:
/// it proves only this bounded shell's own already-accepted rules.
fn preflight_selected_static_semantics(
    items: Vec<RecognizedTopLevelItem>,
) -> StaticPreflightOutcome {
    let mut top_lexical_seen: HashMap<&str, Range> = HashMap::new();
    for item in &items {
        if let RecognizedTopLevelItem::LexicalBinding(fact) = item {
            if let Some(&first) = top_lexical_seen.get(fact.semantic_name.as_str()) {
                return StaticPreflightOutcome::Rejected(
                    StaticPreflightRejection::DuplicateLexical {
                        semantic_name: fact.semantic_name.clone(),
                        first,
                        duplicate: fact.binding,
                        region: SelectedRegion::TopLevel,
                    },
                );
            }
            top_lexical_seen.insert(fact.semantic_name.as_str(), fact.binding);
        }
    }

    for item in &items {
        let RecognizedTopLevelItem::Block(block) = item else {
            continue;
        };
        let mut block_lexical_seen: HashMap<&str, Range> = HashMap::new();
        for block_item in &block.items {
            let RecognizedNonBlockItem::LexicalBinding(fact) = block_item else {
                continue;
            };
            if let Some(&first) = block_lexical_seen.get(fact.semantic_name.as_str()) {
                return StaticPreflightOutcome::Rejected(
                    StaticPreflightRejection::DuplicateLexical {
                        semantic_name: fact.semantic_name.clone(),
                        first,
                        duplicate: fact.binding,
                        region: SelectedRegion::Block(block.anchor),
                    },
                );
            }
            block_lexical_seen.insert(fact.semantic_name.as_str(), fact.binding);
        }
    }

    let mut script_var_contributors: HashMap<&str, Vec<Range>> = HashMap::new();
    for item in &items {
        match item {
            RecognizedTopLevelItem::VarBinding(fact) => {
                script_var_contributors
                    .entry(fact.semantic_name.as_str())
                    .or_default()
                    .push(fact.binding);
            }
            RecognizedTopLevelItem::Block(block) => {
                for block_item in &block.items {
                    if let RecognizedNonBlockItem::VarBinding(fact) = block_item {
                        script_var_contributors
                            .entry(fact.semantic_name.as_str())
                            .or_default()
                            .push(fact.binding);
                    }
                }
            }
            RecognizedTopLevelItem::LexicalBinding(_) | RecognizedTopLevelItem::UseSite(_) => {}
        }
    }

    for item in &items {
        let RecognizedTopLevelItem::LexicalBinding(fact) = item else {
            continue;
        };
        if let Some(contributors) = script_var_contributors.get(fact.semantic_name.as_str()) {
            let var = contributors[0];
            return StaticPreflightOutcome::Rejected(
                StaticPreflightRejection::LexicalVarCollision {
                    semantic_name: fact.semantic_name.clone(),
                    lexical: fact.binding,
                    var,
                    region: SelectedRegion::TopLevel,
                },
            );
        }
    }

    for item in &items {
        let RecognizedTopLevelItem::Block(block) = item else {
            continue;
        };
        let mut block_var: HashMap<&str, Range> = HashMap::new();
        for block_item in &block.items {
            if let RecognizedNonBlockItem::VarBinding(fact) = block_item {
                block_var
                    .entry(fact.semantic_name.as_str())
                    .or_insert(fact.binding);
            }
        }
        for block_item in &block.items {
            let RecognizedNonBlockItem::LexicalBinding(fact) = block_item else {
                continue;
            };
            if let Some(&var) = block_var.get(fact.semantic_name.as_str()) {
                return StaticPreflightOutcome::Rejected(
                    StaticPreflightRejection::LexicalVarCollision {
                        semantic_name: fact.semantic_name.clone(),
                        lexical: fact.binding,
                        var,
                        region: SelectedRegion::Block(block.anchor),
                    },
                );
            }
        }
    }

    StaticPreflightOutcome::Accepted(AcceptedSelectedScript { items })
}

/// The oracle's full independent lifecycle front door: source/placement
/// recognition (Layer 1), then the bounded static preflight gate,
/// producing the only shape Layer 2 may consume. This convenience accessor
/// discards the specific rejection reason on failure;
/// `classify_selected_script` is the entry point that preserves it.
fn recognize_and_accept_selected_script(source: &str) -> Option<AcceptedSelectedScript> {
    let items = parse_selected_script(source)?;
    match preflight_selected_static_semantics(items) {
        StaticPreflightOutcome::Accepted(accepted) => Some(accepted),
        StaticPreflightOutcome::Rejected(_) => None,
    }
}

/// The oracle's executable disposition classification, binding a fixture
/// directly to its outcome rather than only naming these symbols in a
/// disconnected list. A valid-but-unselected close-brace ASI boundary such
/// as `{ a }` independently classifies as `UnsupportedCoverage`, never
/// `DefinitiveGrammarRejectionEvidence`. A syntactically well-formed but
/// statically rejected shell independently classifies as
/// `StaticSemanticsRejected`, never downgraded to `UnsupportedCoverage`.
/// This bounded recognizer never independently proves definitive grammar
/// rejection -- doing so would require duplicating production's
/// already-owned grammar-evidence machinery, which this oracle must not do
/// -- so it never constructs that variant in practice; the variant exists
/// so the type itself preserves the project's failure vocabulary
/// distinction, matching the already-accepted
/// `ResourceLimited`/`InternalFailure` symbolic minimum used by every
/// predecessor candidate-independent `IdentifierReference` oracle.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedScriptDisposition {
    Selected(AcceptedSelectedScript),
    UnsupportedCoverage,
    StaticSemanticsRejected(StaticPreflightRejection),
    DefinitiveGrammarRejectionEvidence(Range),
    ResourceLimited,
    InternalFailure,
}

fn classify_selected_script(source: &str) -> SelectedScriptDisposition {
    let Some(items) = parse_selected_script(source) else {
        return SelectedScriptDisposition::UnsupportedCoverage;
    };

    match preflight_selected_static_semantics(items) {
        StaticPreflightOutcome::Accepted(accepted) => SelectedScriptDisposition::Selected(accepted),
        StaticPreflightOutcome::Rejected(rejection) => {
            SelectedScriptDisposition::StaticSemanticsRejected(rejection)
        }
    }
}

/// One independently derived Layer 2 source-name correspondence relation.
/// Exactly the theorem's vocabulary (`VisibleSelectedLexicalBinding`,
/// `SameSourceSelectedVarNameContributors`, `NoSelectedSameSourceContributor`)
/// and nothing else: no `Statement`/whole-expression/semicolon/item-ordinal
/// anchor, and no field mirroring the existing initializer relation's
/// distinct load-bearing declaration-owner field (W1/W2/W5/W6/W18).
#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedSourceNameCorrespondence {
    VisibleSelectedLexicalBinding {
        binding: Range,
        region: SelectedRegion,
    },
    SameSourceSelectedVarNameContributors {
        contributors: Vec<Range>,
    },
    NoSelectedSameSourceContributor,
}

/// One retained free-standing use-site relation: the reference occurrence,
/// its exact containing region (`TopLevel` or its exact containing Block's
/// own anchor -- W7/W8/W9 firewall: no whole Statement/Expression/semicolon
/// anchor), its semantic name, and its source-name correspondence. There is
/// no containing declarator/binding field (W5/W6).
#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedUseSiteRelation {
    containing_region: SelectedRegion,
    reference: Range,
    semantic_name: String,
    correspondence: SelectedSourceNameCorrespondence,
}

/// Computes one relation for a single recognized use-site fact against the
/// already-collected TopLevel lexical map, whole-Script `var` contributor
/// map, and -- when the use-site is Block-contained -- that specific
/// Block's own lexical map. The lexical precedence theorem is applied
/// exactly in the required order: current Block selected lexical binding,
/// then enclosing TopLevel selected lexical binding, then all same-source
/// selected `var`-name contributors, then `NoSelectedSameSourceContributor`
/// (W11/W12 firewall: an outer lexical binding never wins over a same-Block
/// lexical binding of the same name, regardless of authored order between
/// the use-site and its target).
fn build_relation(
    use_site: &RecognizedUseSiteFact,
    containing_region: SelectedRegion,
    current_block_lexical: Option<&HashMap<&str, Range>>,
    top_lexical: &HashMap<&str, Range>,
    script_var_contributors: &HashMap<&str, Vec<Range>>,
) -> SelectedUseSiteRelation {
    let name = use_site.semantic_name.as_str();

    let correspondence = if let Some(binding) = current_block_lexical.and_then(|m| m.get(name)) {
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: *binding,
            region: containing_region,
        }
    } else if let Some(binding) = top_lexical.get(name) {
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: *binding,
            region: SelectedRegion::TopLevel,
        }
    } else if let Some(contributors) = script_var_contributors.get(name) {
        SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: contributors.clone(),
        }
    } else {
        SelectedSourceNameCorrespondence::NoSelectedSameSourceContributor
    };

    SelectedUseSiteRelation {
        containing_region,
        reference: use_site.reference,
        semantic_name: use_site.semantic_name.clone(),
        correspondence,
    }
}

/// Central Issue #760 Layer 2 theorem. Consumes only the bounded static
/// preflight's accepted witness -- never an arbitrary item list, and never
/// a production accepted-witness type -- and derives exactly one relation
/// per free-standing use-site item, top-level or Block-contained, in exact
/// authored occurrence order, without reordering or deduplication
/// (W9/W10/W14). TopLevel lexical bindings and whole-Script `var`
/// contributors are collected from the *whole* accepted item sequence
/// before any correspondence is computed, so a lexical declaration or `var`
/// contributor appearing after a use-site still corresponds (the forward
/// lexical correspondence theorem, W12 adjacent: this is same-source
/// correspondence, never previous-only or execution-order lookup). Each
/// Block's own lexical map is likewise collected from that Block's *whole*
/// own item sequence before any of that Block's own use-sites are
/// resolved, and never leaks into or receives from a sibling Block (W10
/// sibling-lexical-leakage firewall). A `var` contributor is Script-wide
/// regardless of which region contributed it -- unlike lexical visibility,
/// sibling-Block and top-level `var` contributors both remain eligible
/// (acceptance criterion 11).
fn build_selected_source_name_correspondence(
    accepted: &AcceptedSelectedScript,
) -> Vec<SelectedUseSiteRelation> {
    let items = accepted.items();

    let mut top_lexical: HashMap<&str, Range> = HashMap::new();
    let mut script_var_contributors: HashMap<&str, Vec<Range>> = HashMap::new();

    for item in items {
        match item {
            RecognizedTopLevelItem::LexicalBinding(fact) => {
                top_lexical
                    .entry(fact.semantic_name.as_str())
                    .or_insert(fact.binding);
            }
            RecognizedTopLevelItem::VarBinding(fact) => {
                script_var_contributors
                    .entry(fact.semantic_name.as_str())
                    .or_default()
                    .push(fact.binding);
            }
            RecognizedTopLevelItem::Block(block) => {
                for block_item in &block.items {
                    if let RecognizedNonBlockItem::VarBinding(fact) = block_item {
                        script_var_contributors
                            .entry(fact.semantic_name.as_str())
                            .or_default()
                            .push(fact.binding);
                    }
                }
            }
            RecognizedTopLevelItem::UseSite(_) => {}
        }
    }

    let mut relations = Vec::new();
    for item in items {
        match item {
            RecognizedTopLevelItem::UseSite(use_site) => {
                relations.push(build_relation(
                    use_site,
                    SelectedRegion::TopLevel,
                    None,
                    &top_lexical,
                    &script_var_contributors,
                ));
            }
            RecognizedTopLevelItem::Block(block) => {
                let mut block_lexical: HashMap<&str, Range> = HashMap::new();
                for block_item in &block.items {
                    if let RecognizedNonBlockItem::LexicalBinding(fact) = block_item {
                        block_lexical
                            .entry(fact.semantic_name.as_str())
                            .or_insert(fact.binding);
                    }
                }
                for block_item in &block.items {
                    if let RecognizedNonBlockItem::UseSite(use_site) = block_item {
                        relations.push(build_relation(
                            use_site,
                            SelectedRegion::Block(block.anchor),
                            Some(&block_lexical),
                            &top_lexical,
                            &script_var_contributors,
                        ));
                    }
                }
            }
            RecognizedTopLevelItem::LexicalBinding(_) | RecognizedTopLevelItem::VarBinding(_) => {}
        }
    }

    relations
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 760);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREDECESSOR_TOP_LEVEL_USE_SITE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 756"));
    assert!(PREDECESSOR_BLOCK_BINDING_SCOPE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 302"));
    assert!(PREDECESSOR_BLOCK_ORACLE_SOURCE.contains("Issue #283"));
    assert!(FRONTIER_SCOPE_NOTE.contains("one-level Block free-standing IdentifierReference"));
    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));

    for forbidden in [
        // Production hard-zero: no candidate answer is ever derived from a
        // future production recognizer, accepted witness, or
        // correspondence implementation.
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
            "use super::qualification_selected_top_level_identifier_",
            "reference_expression_statement_use_site_validation_tests"
        ),
        concat!(
            "use super::selected_one_level_block_binding_",
            "scope_validation_tests"
        ),
        concat!(
            "use super::qualification_selected_one_level_block_",
            "validation_tests"
        ),
        // Forbidden reconstruction of expected ranges.
        concat!(".", "find("),
        concat!(".", "rfind("),
        // The distinct, load-bearing production field this new relation
        // must never reuse or generalize.
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
        // Recursive/deeper Block support is never introduced.
        concat!("enum Recognized", "NestedBlockItem"),
        concat!("Recognized", "NonBlockItem::Block"),
        // Completion movement is never introduced by a validation-only leaf.
        concat!("TO", "TAL"),
        concat!("NEWLY_", "REACHABLE"),
        concat!("Qual", "ified"),
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

/// Required positive direct/escaped family (acceptance criteria 2, 3, 4):
/// every accepted spelling family, placed inside one already-selected
/// one-level Block, produces exactly one free-standing use-site item with
/// its own exact authored anchor, decoded semantic name, spelling
/// classification, exact containing Block anchor, and
/// `NoSelectedSameSourceContributor` correspondence when no declaration is
/// in scope. No range is ever discovered by search, rescan, reparse, or
/// decoded-length inference: every range below is fixture-owned and
/// independently reconciled through Core `SourceText`.
#[test]
fn positive_direct_escaped_family_pins_exact_block_provenance_and_no_match() {
    struct PositiveFixture {
        source: &'static str,
        block: Range,
        reference: Range,
        expected_authored: &'static str,
        expected_semantic_name: &'static str,
        expected_spelling: IdentifierSpellingKind,
    }

    let fixtures = [
        // A: bare Block use-site, the theorem's own representative source.
        PositiveFixture {
            source: "{ a; }",
            block: Range(0, 6),
            reference: Range(2, 3),
            expected_authored: "a",
            expected_semantic_name: "a",
            expected_spelling: IdentifierSpellingKind::Direct,
        },
        // B: escaped authored provenance, classic four-hex-digit form.
        PositiveFixture {
            source: concat!("{ ", "\\", "u0061", "; }"),
            block: Range(0, 11),
            reference: Range(2, 8),
            expected_authored: concat!("\\", "u0061"),
            expected_semantic_name: "a",
            expected_spelling: IdentifierSpellingKind::EscapedNonReserved,
        },
        // B (representative braced form already accepted by existing
        // IdentifierReference authority).
        PositiveFixture {
            source: concat!("{ ", "\\", "u{61}", "; }"),
            block: Range(0, 11),
            reference: Range(2, 8),
            expected_authored: concat!("\\", "u{61}"),
            expected_semantic_name: "a",
            expected_spelling: IdentifierSpellingKind::EscapedNonReserved,
        },
        PositiveFixture {
            source: concat!("{ f", "\\", "u006F", "o; }"),
            block: Range(0, 13),
            reference: Range(2, 10),
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
        assert_eq!(slice(fixture.source, fixture.block), fixture.source);

        let accepted = recognize_and_accept_selected_script(fixture.source)
            .unwrap_or_else(|| panic!("{:?} must recognize the theorem", fixture.source));
        let items = accepted.items();
        assert_eq!(items.len(), 1);
        let RecognizedTopLevelItem::Block(block) = &items[0] else {
            panic!(
                "{:?} must recognize one already-selected one-level Block",
                fixture.source
            );
        };
        assert_eq!(block.anchor, fixture.block);
        assert_eq!(block.items.len(), 1);
        let RecognizedNonBlockItem::UseSite(use_site) = &block.items[0] else {
            panic!(
                "{:?} must recognize a free-standing Block use-site item",
                fixture.source
            );
        };
        assert_eq!(use_site.reference, fixture.reference);
        assert_eq!(use_site.semantic_name, fixture.expected_semantic_name);
        assert_eq!(use_site.spelling, fixture.expected_spelling);

        if fixture.expected_spelling == IdentifierSpellingKind::EscapedNonReserved {
            assert_ne!(fixture.expected_authored, fixture.expected_semantic_name);
        }

        assert_eq!(
            authored_anchor(760_000 + index as u64, fixture.source, fixture.reference),
            fixture.expected_authored
        );

        let relations = build_selected_source_name_correspondence(&accepted);
        assert_eq!(relations.len(), 1);
        assert_eq!(
            relations[0].containing_region,
            SelectedRegion::Block(fixture.block)
        );
        assert_eq!(relations[0].reference, fixture.reference);
        assert_eq!(relations[0].semantic_name, fixture.expected_semantic_name);
        assert_eq!(
            relations[0].correspondence,
            SelectedSourceNameCorrespondence::NoSelectedSameSourceContributor
        );
    }
}

/// Current-Block lexical precedence (acceptance criterion 7): a
/// free-standing Block use-site corresponds to the exact Block-local
/// lexical binding of the same semantic name.
#[test]
fn current_block_lexical_target_is_independently_validated() {
    let source = "{ let a; a; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`{ let a; a; }` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 1);
    let RecognizedTopLevelItem::Block(block) = &items[0] else {
        panic!("must recognize one Block");
    };
    assert_eq!(block.anchor, Range(0, 13));
    assert_eq!(block.items.len(), 2);
    let RecognizedNonBlockItem::LexicalBinding(binding) = &block.items[0] else {
        panic!("first Block item must be the lexical binding");
    };
    assert_eq!(binding.binding, Range(6, 7));

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].reference, Range(9, 10));
    assert_eq!(
        relations[0].containing_region,
        SelectedRegion::Block(block.anchor)
    );
    assert_eq!(
        relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(6, 7),
            region: SelectedRegion::Block(block.anchor),
        }
    );
}

/// Outer TopLevel lexical fallback (acceptance criterion 8): a Block use-site
/// with no matching Block-local lexical binding corresponds to the exact
/// enclosing TopLevel selected lexical binding.
#[test]
fn outer_top_level_lexical_fallback_is_independently_validated() {
    let source = "let a;\n{ a; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`let a; { a; }` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 2);
    let RecognizedTopLevelItem::LexicalBinding(top_binding) = &items[0] else {
        panic!("first item must be the TopLevel lexical binding");
    };
    assert_eq!(top_binding.binding, Range(4, 5));
    let RecognizedTopLevelItem::Block(block) = &items[1] else {
        panic!("second item must be the Block");
    };
    assert_eq!(block.anchor, Range(7, 13));

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].reference, Range(9, 10));
    assert_eq!(
        relations[0].containing_region,
        SelectedRegion::Block(block.anchor)
    );
    assert_eq!(
        relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5),
            region: SelectedRegion::TopLevel,
        }
    );
}

/// Forward inner lexical shadowing (acceptance criteria 9, W12): a
/// later-authored same-Block lexical binding is the source-correspondence
/// target, never the outer TopLevel binding of the same name -- this is
/// source correspondence, never a runtime-TDZ-success claim.
#[test]
fn forward_inner_lexical_shadows_outer_top_level() {
    let source = "let a;\n{ a; let a; }";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`let a; { a; let a; }` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 2);
    let RecognizedTopLevelItem::Block(block) = &items[1] else {
        panic!("second item must be the Block");
    };
    assert_eq!(block.anchor, Range(7, 20));
    assert_eq!(block.items.len(), 2);
    let RecognizedNonBlockItem::UseSite(use_site) = &block.items[0] else {
        panic!("first Block item must be the use-site");
    };
    assert_eq!(use_site.reference, Range(9, 10));
    let RecognizedNonBlockItem::LexicalBinding(inner_binding) = &block.items[1] else {
        panic!("second Block item must be the later inner lexical binding");
    };
    assert_eq!(inner_binding.binding, Range(16, 17));

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].reference, Range(9, 10));
    assert_eq!(
        relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(16, 17),
            region: SelectedRegion::Block(block.anchor),
        },
        "the later same-Block lexical binding must shadow the outer TopLevel binding"
    );
}

/// Sibling Block lexical exclusion (acceptance criterion 10, W10): a
/// sibling Block's lexical binding never leaks into another Block's
/// correspondence.
#[test]
fn sibling_block_lexical_binding_never_leaks() {
    let source = "{ let a; }\n{ a; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`{ let a; } { a; }` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 2);
    let RecognizedTopLevelItem::Block(first_block) = &items[0] else {
        panic!("first item must be a Block");
    };
    assert_eq!(first_block.anchor, Range(0, 10));
    let RecognizedTopLevelItem::Block(second_block) = &items[1] else {
        panic!("second item must be a Block");
    };
    assert_eq!(second_block.anchor, Range(11, 17));

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].reference, Range(13, 14));
    assert_eq!(
        relations[0].containing_region,
        SelectedRegion::Block(second_block.anchor)
    );
    assert_eq!(
        relations[0].correspondence,
        SelectedSourceNameCorrespondence::NoSelectedSameSourceContributor,
        "the first Block's own lexical `a` must never be a target for the second Block's use-site"
    );
}

/// Later TopLevel lexical source correspondence (acceptance criterion 9
/// adjacent, forward TopLevel fallback): a Block use-site with no matching
/// Block-local lexical binding corresponds to a TopLevel lexical binding
/// authored after the Block.
#[test]
fn later_top_level_lexical_forward_correspondence_is_independently_validated() {
    let source = "{ a; }\nlet a;";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`{ a; } let a;` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 2);
    let RecognizedTopLevelItem::Block(block) = &items[0] else {
        panic!("first item must be the Block");
    };
    assert_eq!(block.anchor, Range(0, 6));
    let RecognizedTopLevelItem::LexicalBinding(later_binding) = &items[1] else {
        panic!("second item must be the later TopLevel lexical binding");
    };
    assert_eq!(later_binding.binding, Range(11, 12));

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].reference, Range(2, 3));
    assert_eq!(
        relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(11, 12),
            region: SelectedRegion::TopLevel,
        }
    );
}

/// Whole-Script selected-`var` contributor provenance (acceptance criteria
/// 11, 12): a top-level `var` contributor, a same-Block `var` contributor,
/// and a sibling-Block `var` contributor all remain eligible same-source
/// targets under the accepted whole-Script provenance theorem -- unlike
/// lexical visibility, `var` contributor eligibility is never Block-scoped.
#[test]
fn whole_script_var_contributor_provenance_covers_every_region() {
    // H: top-level var contributor.
    let top_level_source = "var a;\n{ a; }";
    let top_level_accepted = recognize_and_accept_selected_script(top_level_source)
        .expect("`var a; { a; }` must recognize");
    let top_level_relations = build_selected_source_name_correspondence(&top_level_accepted);
    assert_eq!(top_level_relations.len(), 1);
    assert_eq!(
        top_level_relations[0].correspondence,
        SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(4, 5)],
        }
    );

    // I: same-Block var contributor.
    let same_block_source = "{ var a; a; }";
    let same_block_accepted = recognize_and_accept_selected_script(same_block_source)
        .expect("`{ var a; a; }` must recognize");
    let same_block_relations = build_selected_source_name_correspondence(&same_block_accepted);
    assert_eq!(same_block_relations.len(), 1);
    assert_eq!(
        same_block_relations[0].correspondence,
        SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(6, 7)],
        }
    );

    // J: sibling-Block var contributor -- proves sibling lexical exclusion
    // rules are never reused for `var` (acceptance criterion 12).
    let sibling_block_source = "{ a; }\n{ var a; }";
    let sibling_block_accepted = recognize_and_accept_selected_script(sibling_block_source)
        .expect("`{ a; } { var a; }` must recognize");
    let sibling_block_relations =
        build_selected_source_name_correspondence(&sibling_block_accepted);
    assert_eq!(sibling_block_relations.len(), 1);
    assert_eq!(sibling_block_relations[0].reference, Range(2, 3));
    assert_eq!(
        sibling_block_relations[0].correspondence,
        SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(13, 14)],
        },
        "a sibling Block's own var contributor must remain eligible under whole-Script provenance"
    );
}

/// Lexical precedence over a same-source `var` contributor: a Block
/// use-site corresponds to a visible selected lexical binding, never to an
/// unrelated `var` contributor of the same name.
#[test]
fn lexical_precedence_over_var_contributor_is_independently_validated() {
    let source = "let a;\nvar x = a;\n{ a; }";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`let a; var x = a; { a; }` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 3);
    let RecognizedTopLevelItem::VarBinding(x_binding) = &items[1] else {
        panic!("second item must be the `x` var binding");
    };
    assert_eq!(x_binding.semantic_name, "x");

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].reference, Range(20, 21));
    assert_eq!(
        relations[0].correspondence,
        SelectedSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5),
            region: SelectedRegion::TopLevel,
        }
    );
}

/// Duplicate use-site occurrence theorem (acceptance criterion 14, W14):
/// two authored occurrences of the same semantic name in the same Block
/// remain two distinct relations, never collapsed into one.
#[test]
fn duplicate_block_use_site_occurrences_are_never_deduplicated() {
    let source = "{ a; a; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`{ a; a; }` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 1);
    let RecognizedTopLevelItem::Block(block) = &items[0] else {
        panic!("must recognize one Block");
    };
    assert_eq!(block.items.len(), 2);

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_ne!(relations[0].reference, relations[1].reference);
    assert_eq!(relations[0].reference, Range(2, 3));
    assert_eq!(relations[1].reference, Range(5, 6));
    assert_eq!(relations[0].semantic_name, relations[1].semantic_name);
    assert_eq!(relations[0].correspondence, relations[1].correspondence);
}

/// Distinct Block ownership (acceptance criterion 15): two distinct Blocks
/// retain two distinct reference occurrences, each with its own exact
/// containing Block region.
#[test]
fn distinct_blocks_retain_distinct_region_ownership() {
    let source = "{ a; }\n{ a; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`{ a; } { a; }` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 2);
    let RecognizedTopLevelItem::Block(first_block) = &items[0] else {
        panic!("first item must be a Block");
    };
    let RecognizedTopLevelItem::Block(second_block) = &items[1] else {
        panic!("second item must be a Block");
    };
    assert_ne!(first_block.anchor, second_block.anchor);

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].reference, Range(2, 3));
    assert_eq!(
        relations[0].containing_region,
        SelectedRegion::Block(first_block.anchor)
    );
    assert_eq!(relations[1].reference, Range(9, 10));
    assert_eq!(
        relations[1].containing_region,
        SelectedRegion::Block(second_block.anchor)
    );
    for relation in &relations {
        assert_eq!(
            relation.correspondence,
            SelectedSourceNameCorrespondence::NoSelectedSameSourceContributor
        );
    }
}

/// TopLevel and Block surfaces remain semantically distinct (acceptance
/// criterion 16, W15): all covered occurrences across both surfaces are
/// validated in exact authored order, without inventing one new global
/// initializer/top-level/Block relation ordering contract -- each relation
/// still independently carries only its own containing region.
#[test]
fn top_level_and_block_surfaces_remain_distinguishable_in_authored_order() {
    let source = "a;\n{ a; }\na;";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`a; { a; } a;` must recognize");
    let items = accepted.items();
    assert_eq!(items.len(), 3);

    let relations = build_selected_source_name_correspondence(&accepted);
    assert_eq!(relations.len(), 3);
    assert_eq!(relations[0].containing_region, SelectedRegion::TopLevel);
    assert_eq!(relations[0].reference, Range(0, 1));
    let RecognizedTopLevelItem::Block(block) = &items[1] else {
        panic!("second item must be the Block");
    };
    assert_eq!(
        relations[1].containing_region,
        SelectedRegion::Block(block.anchor)
    );
    assert_eq!(relations[1].reference, Range(5, 6));
    assert_eq!(relations[2].containing_region, SelectedRegion::TopLevel);
    assert_eq!(relations[2].reference, Range(10, 11));
    for relation in &relations {
        assert_eq!(
            relation.correspondence,
            SelectedSourceNameCorrespondence::NoSelectedSameSourceContributor
        );
    }
}

/// Dispatch discriminators (acceptance criterion 17): `{ let; }` and
/// `{ varfoo; }` are independently challenged as selected use-site
/// candidates -- the keyword attempt cannot form a complete binding, so
/// each falls through to the free-standing use-site route with its whole
/// spelling as the semantic name -- while `{ let a; }` and `{ var a; }`
/// remain recognized as the existing declaration items, never reclassified
/// (acceptance criterion 18).
#[test]
fn dispatch_discriminators_never_reclassify_existing_declarations() {
    let let_candidate = "{ let; }";
    let let_accepted = recognize_and_accept_selected_script(let_candidate)
        .expect("`{ let; }` must recognize as a selected use-site candidate");
    let let_items = let_accepted.items();
    assert_eq!(let_items.len(), 1);
    let RecognizedTopLevelItem::Block(block) = &let_items[0] else {
        panic!("must recognize one Block");
    };
    assert_eq!(block.items.len(), 1);
    let RecognizedNonBlockItem::UseSite(use_site) = &block.items[0] else {
        panic!("`{{ let; }}` must fall through to the free-standing use-site route");
    };
    assert_eq!(use_site.semantic_name, "let");
    assert_eq!(use_site.reference, Range(2, 5));

    let varfoo_candidate = "{ varfoo; }";
    let varfoo_accepted = recognize_and_accept_selected_script(varfoo_candidate)
        .expect("`{ varfoo; }` must recognize as a selected use-site candidate");
    let varfoo_items = varfoo_accepted.items();
    let RecognizedTopLevelItem::Block(varfoo_block) = &varfoo_items[0] else {
        panic!("must recognize one Block");
    };
    let RecognizedNonBlockItem::UseSite(varfoo_use_site) = &varfoo_block.items[0] else {
        panic!("`{{ varfoo; }}` must fall through to the free-standing use-site route");
    };
    assert_eq!(varfoo_use_site.semantic_name, "varfoo");
    assert_eq!(varfoo_use_site.reference, Range(2, 8));

    let existing_let_declaration = "{ let a; }";
    let existing_let_accepted = recognize_and_accept_selected_script(existing_let_declaration)
        .expect("`{ let a; }` must recognize as the existing LexicalDeclaration");
    let existing_let_items = existing_let_accepted.items();
    let RecognizedTopLevelItem::Block(existing_let_block) = &existing_let_items[0] else {
        panic!("must recognize one Block");
    };
    assert!(matches!(
        existing_let_block.items[0],
        RecognizedNonBlockItem::LexicalBinding(_)
    ));
    assert_eq!(
        build_selected_source_name_correspondence(&existing_let_accepted).len(),
        0,
        "`{{ let a; }}` contributes no use-site relation"
    );

    let existing_var_declaration = "{ var a; }";
    let existing_var_accepted = recognize_and_accept_selected_script(existing_var_declaration)
        .expect("`{ var a; }` must recognize as the existing VariableStatement");
    let existing_var_items = existing_var_accepted.items();
    let RecognizedTopLevelItem::Block(existing_var_block) = &existing_var_items[0] else {
        panic!("must recognize one Block");
    };
    assert!(matches!(
        existing_var_block.items[0],
        RecognizedNonBlockItem::VarBinding(_)
    ));
    assert_eq!(
        build_selected_source_name_correspondence(&existing_var_accepted).len(),
        0,
        "`{{ var a; }}` contributes no use-site relation"
    );
}

/// AuthoredSemicolon theorem and the close-brace ASI firewall together
/// (acceptance criteria 5, 6): only an authored `;` terminator is selected.
/// `{ a }` remains `UnsupportedCoverage`, never grammar rejection, and this
/// oracle never composes #717's automatic-semicolon-before-close-brace
/// precedent.
#[test]
fn authored_semicolon_theorem_and_close_brace_asi_boundary_remain_unsupported_coverage() {
    match classify_selected_script("{ a; }") {
        SelectedScriptDisposition::Selected(accepted) => {
            assert_eq!(accepted.items().len(), 1);
        }
        other => panic!("`{{ a; }}` must classify as Selected, got {other:?}"),
    }

    assert_eq!(
        classify_selected_script("{ a }"),
        SelectedScriptDisposition::UnsupportedCoverage,
        "`{{ a }}` is a valid-but-unselected close-brace ASI boundary, not part of this theorem, \
         and must never classify as DefinitiveGrammarRejectionEvidence"
    );

    // The failure is caused only by the missing authored terminator, never
    // by an invalid identifier shape: the same leading text independently
    // recognizes as a valid `SelectedAcceptedIdentifierReference` on its
    // own.
    assert!(recognize_accepted_identifier_reference("a").is_some());

    assert!(FAILURE_CLASSIFICATIONS.contains(&"UnsupportedCoverage"));
    assert!(FAILURE_CLASSIFICATIONS.contains(&"DefinitiveGrammarRejectionEvidence"));
    assert_ne!(
        FAILURE_CLASSIFICATIONS[0], FAILURE_CLASSIFICATIONS[1],
        "UnsupportedCoverage and DefinitiveGrammarRejectionEvidence must stay distinct"
    );

    assert!(THIS_ORACLE_SOURCE.contains("struct SelectedUseSiteRelation"));
    for forbidden in [
        concat!("semi", "colon: Range"),
        concat!("semicolon_", "anchor"),
        concat!("termin", "ator: Range"),
        concat!("AutomaticBefore", "BlockClose"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }
}

/// Placement firewall (acceptance criteria 1, 19): recursive/deeper Block
/// nesting, function/class bodies, and strict-mode-widened forms remain
/// outside without any generic `StatementList` architecture or special
/// nested-Block rejection code in this oracle.
#[test]
fn placement_firewall_keeps_deeper_nesting_and_strict_mode_forms_outside() {
    for source in [
        "{ { a; } }",
        "function f() { a; }",
        "class C { m() { a; } }",
        "\"use strict\";\n{ a; }",
    ] {
        assert!(
            parse_selected_script(source).is_none(),
            "{source:?} must remain outside the bounded one-level theorem"
        );
    }

    // The bounded theorem itself remains independently valid; only the
    // deeper/strict-mode neighbors above are unowned.
    assert!(parse_selected_script("{ a; }").is_some());
}

/// General-expression firewall (acceptance criterion 19 adjacent, W16):
/// richer Expression neighbors remain outside this theorem without any
/// generic Expression parser, and no valid leading `IdentifierReference`
/// prefix is ever accepted from a richer expression.
#[test]
fn general_expression_neighbor_firewall_keeps_richer_syntax_outside() {
    for source in [
        "{ a+b; }",
        "{ +a; }",
        "{ -a; }",
        "{ (a); }",
        "{ a.b; }",
        "{ a[b]; }",
        "{ a(); }",
        "{ a=b; }",
        "{ a ? b : c; }",
        "{ a && b; }",
        "{ a, b; }",
        "{ new a; }",
    ] {
        assert!(parse_selected_script(source).is_none(), "{source:?}");
    }

    assert!(parse_selected_script("{ a; }").is_some());
}

/// Escaped `ReservedWord` boundary: a decoded unconditionally reserved word
/// never becomes a selected free-standing Block use-site.
#[test]
fn escaped_reserved_word_firewall_emits_no_accepted_block_use_site_relation() {
    let source = concat!("{ ", "\\", "u0069", "f; }");
    assert!(
        parse_selected_script(source).is_none(),
        "{source:?} decodes to the ReservedWord `if`"
    );
    assert_eq!(
        decode_selected_escaped_identifier(concat!("\\", "u0069", "f")),
        Err(DecodeFailure::DecodedReserved)
    );
}

/// Malformed / invalid escape firewall: no accepted use-site relation leaks
/// from a malformed or position-invalid candidate, and no valid-prefix
/// truncation ever occurs, whether embedded in a Block or judged directly.
#[test]
fn malformed_and_invalid_escape_firewall_never_leaks_a_block_use_site_relation() {
    // Built with individually concatenated fragments (never one contiguous
    // raw `\uXXXX` literal) so the literal bytes `\`, `u`, and the exact
    // hex digits survive intact through the tooling pipeline rather than
    // being collapsed into a decoded Unicode scalar by an intermediate
    // writer.
    let leading_zero_digit = concat!("{ ", "\\", "u0030", "; }");
    let mid_hyphen = concat!("{ a", "\\", "u002D", "b; }");
    for source in [
        "{ \\u{}; }",
        "{ \\uD800; }",
        "{ \\u{110000}; }",
        leading_zero_digit,
        mid_hyphen,
    ] {
        assert!(parse_selected_script(source).is_none(), "{source:?}");
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

    assert!(leading_zero_digit.contains('{'));
    assert!(leading_zero_digit.starts_with("{ \\"));
    assert_eq!(leading_zero_digit.as_bytes()[3], b'u');
    assert!(mid_hyphen.contains('\\'));
    assert_eq!(mid_hyphen.len(), "{ a\\u002Db; }".len());
}

/// Whole-source transactionality (acceptance criterion 26, W21): a locally
/// valid earlier Block use-site never publishes relation evidence when
/// later source prevents whole-source acceptance.
#[test]
fn whole_source_transactionality_never_leaks_earlier_valid_block_relations() {
    let valid_prefix = "{ a; }";
    assert!(parse_selected_script(valid_prefix).is_some());

    for source in ["{ a; }\n???", "{ a; }\nlet x = ;"] {
        assert!(
            parse_selected_script(source).is_none(),
            "{source:?} must publish no relation evidence at all, not even for the valid \
             `{{ a; }}` prefix"
        );
    }
}

/// Static-semantics prerequisite (acceptance criterion 20): an
/// already-known static rejection -- here, the accepted Script
/// lexical/`var` collision rule now that a `var` contributor may be
/// Block-contained -- suppresses the new downstream relation entirely, and
/// remains a static rejection, never downgraded to `UnsupportedCoverage`.
#[test]
fn known_static_rejection_suppresses_downstream_block_use_site_relation() {
    let source = "let a;\n{\nvar a;\na;\n}";
    assert!(
        parse_selected_script(source).is_some(),
        "the shell must be syntactically recognized by Layer 1 alone"
    );
    assert!(recognize_and_accept_selected_script(source).is_none());

    match classify_selected_script(source) {
        SelectedScriptDisposition::StaticSemanticsRejected(
            StaticPreflightRejection::LexicalVarCollision {
                semantic_name,
                lexical,
                var,
                region,
            },
        ) => {
            assert_eq!(semantic_name, "a");
            assert_eq!(lexical, Range(4, 5));
            assert_eq!(var, Range(13, 14));
            assert_eq!(region, SelectedRegion::TopLevel);
        }
        other => panic!(
            "{source:?} must classify as StaticSemanticsRejected(LexicalVarCollision), got {other:?}"
        ),
    }
}

/// Region-local duplicate lexical rejection and region-local Block/`var`
/// collision (acceptance criteria 10, 20 adjacent): duplicate detection and
/// the Block-local lexical/`var` collision rule each stay within their own
/// region, never merged across sibling Blocks.
#[test]
fn region_local_duplicate_lexical_and_block_local_var_collision_are_independently_validated() {
    let duplicate_in_block = "{ let a; let a; }";
    match classify_selected_script(duplicate_in_block) {
        SelectedScriptDisposition::StaticSemanticsRejected(
            StaticPreflightRejection::DuplicateLexical {
                semantic_name,
                first,
                duplicate,
                region,
            },
        ) => {
            assert_eq!(semantic_name, "a");
            assert_eq!(first, Range(6, 7));
            assert_eq!(duplicate, Range(13, 14));
            assert_eq!(region, SelectedRegion::Block(Range(0, 17)));
        }
        other => panic!(
            "{duplicate_in_block:?} must classify as StaticSemanticsRejected(DuplicateLexical), \
             got {other:?}"
        ),
    }

    // Sibling Blocks each independently reusing the same lexical name never
    // collide with each other.
    let sibling_duplicate_allowed = "{ let a; }\n{ let a; }";
    assert!(matches!(
        classify_selected_script(sibling_duplicate_allowed),
        SelectedScriptDisposition::Selected(_)
    ));

    // Block-local lexical/`var` collision: a Block's own lexical name
    // colliding with that same Block's own `var` name is rejected, even
    // though an *outer* `var` of the same name never collides with a
    // Block's own lexical name (acceptance criterion 10 adjacent).
    let block_local_collision = "{ let a; var a; }";
    match classify_selected_script(block_local_collision) {
        SelectedScriptDisposition::StaticSemanticsRejected(
            StaticPreflightRejection::LexicalVarCollision {
                semantic_name,
                lexical,
                var,
                region,
            },
        ) => {
            assert_eq!(semantic_name, "a");
            assert_eq!(lexical, Range(6, 7));
            assert_eq!(var, Range(13, 14));
            assert_eq!(region, SelectedRegion::Block(Range(0, 17)));
        }
        other => panic!(
            "{block_local_collision:?} must classify as StaticSemanticsRejected(LexicalVarCollision), \
             got {other:?}"
        ),
    }

    let outer_var_block_lexical_no_collision = "var a;\n{ let a; }";
    assert!(
        matches!(
            classify_selected_script(outer_var_block_lexical_no_collision),
            SelectedScriptDisposition::Selected(_)
        ),
        "an outer `var` must never collide with an unrelated Block's own lexical binding"
    );

    // Duplicate `var` contributors alone remain allowed, at any region.
    let duplicate_var_allowed = "{ var a; var a; a; }";
    let duplicate_var_accepted = match classify_selected_script(duplicate_var_allowed) {
        SelectedScriptDisposition::Selected(accepted) => accepted,
        other => panic!(
            "`{{ var a; var a; a; }}` must classify as Selected (duplicate var contributors \
             alone are allowed), got {other:?}"
        ),
    };
    let duplicate_var_relations =
        build_selected_source_name_correspondence(&duplicate_var_accepted);
    assert_eq!(duplicate_var_relations.len(), 1);
    assert_eq!(
        duplicate_var_relations[0].correspondence,
        SelectedSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(6, 7), Range(13, 14)],
        }
    );
}

/// The disposition enum preserves the project's failure vocabulary as a
/// real, executable distinction, not only as a name in a symbolic list.
#[test]
fn disposition_enum_preserves_the_failure_vocabulary_distinction() {
    let sample_rejection = StaticPreflightRejection::DuplicateLexical {
        semantic_name: "a".to_owned(),
        first: Range(0, 0),
        duplicate: Range(0, 0),
        region: SelectedRegion::TopLevel,
    };

    assert_ne!(
        SelectedScriptDisposition::UnsupportedCoverage,
        SelectedScriptDisposition::ResourceLimited
    );
    assert_ne!(
        SelectedScriptDisposition::UnsupportedCoverage,
        SelectedScriptDisposition::InternalFailure
    );
    assert_ne!(
        SelectedScriptDisposition::ResourceLimited,
        SelectedScriptDisposition::InternalFailure
    );
    assert_ne!(
        SelectedScriptDisposition::UnsupportedCoverage,
        SelectedScriptDisposition::DefinitiveGrammarRejectionEvidence(Range(0, 0))
    );
    assert_ne!(
        SelectedScriptDisposition::UnsupportedCoverage,
        SelectedScriptDisposition::StaticSemanticsRejected(sample_rejection.clone())
    );
    assert_ne!(
        SelectedScriptDisposition::StaticSemanticsRejected(sample_rejection.clone()),
        SelectedScriptDisposition::DefinitiveGrammarRejectionEvidence(Range(0, 0))
    );
    assert_ne!(
        SelectedScriptDisposition::StaticSemanticsRejected(sample_rejection.clone()),
        SelectedScriptDisposition::ResourceLimited
    );
    assert_ne!(
        SelectedScriptDisposition::StaticSemanticsRejected(sample_rejection),
        SelectedScriptDisposition::InternalFailure
    );

    assert_ne!(
        StaticPreflightRejection::DuplicateLexical {
            semantic_name: "a".to_owned(),
            first: Range(0, 0),
            duplicate: Range(0, 0),
            region: SelectedRegion::TopLevel,
        },
        StaticPreflightRejection::LexicalVarCollision {
            semantic_name: "a".to_owned(),
            lexical: Range(0, 0),
            var: Range(0, 0),
            region: SelectedRegion::TopLevel,
        }
    );

    assert_ne!(SelectedRegion::TopLevel, SelectedRegion::Block(Range(0, 1)));

    assert_eq!(
        classify_selected_script("{ a + b; }"),
        SelectedScriptDisposition::UnsupportedCoverage
    );
}

/// Resource / failure semantics: the project's
/// `ResourceLimited`/`InternalFailure` lifecycle states remain distinct
/// from `UnsupportedCoverage` and from `DefinitiveGrammarRejectionEvidence`,
/// matching the minimal symbolic model already accepted by the predecessor
/// candidate-independent `IdentifierReference` oracles -- this leaf adds no
/// further resource machinery beyond that already-accepted minimum.
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
/// `Statement`/`Expression`/reference-use abstraction, no completion
/// movement, no production representation decision, and no runtime
/// resolution/evaluation vocabulary is introduced.
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
        concat!("SelectedBlockItem", "::"),
    ] {
        assert!(!THIS_ORACLE_SOURCE.contains(forbidden), "{forbidden}");
    }

    assert!(THIS_ORACLE_SOURCE.contains("VisibleSelectedLexicalBinding"));
    assert!(THIS_ORACLE_SOURCE.contains("SameSourceSelectedVarNameContributors"));
    assert!(THIS_ORACLE_SOURCE.contains("NoSelectedSameSourceContributor"));

    // Static neutrality: this oracle never computes or claims BoundNames,
    // LexicallyDeclaredNames, or VarDeclaredNames contribution for the
    // free-standing use-site itself -- its relation carries no such field,
    // and no production static-semantics path is ever consulted (checked
    // separately by the forbidden-import list in
    // `authority_independence_and_frontier_scope_are_exact`).
    assert!(THIS_ORACLE_SOURCE.contains("struct RecognizedUseSiteFact"));

    assert!(FRONTIER_SCOPE_NOTE.contains("frontier only"));
}
