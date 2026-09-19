//! Candidate-independent exactly-two `IdentifierReference` additive
//! `ExpressionStatement` use-site validation for Issue #764 (durable
//! research: Issue #688 comment `5738838818`; accepted predecessor
//! authorities: Issue #752 / PR #753 ordered two-`IdentifierReference`
//! additive initializer oracle, Issue #754 / PR #755 production ordered
//! two-`IdentifierReference` additive initializer, Issue #756 / PR #757
//! top-level free-standing `IdentifierReference` use-site oracle, Issue
//! #758 / PR #759 production top-level free-standing use-sites, Issue #760
//! / PR #761 one-level Block free-standing `IdentifierReference` use-site
//! oracle, Issue #762 / PR #763 production one-level Block free-standing
//! use-sites).
//!
//! This oracle qualifies only the bounded theorem:
//!
//! ```text
//! SelectedTwoIdentifierReferenceAdditiveExpressionStatement ::=
//!     SelectedAcceptedIdentifierReference
//!     SelectedAdditiveTrivia
//!     SelectedAdditiveOperator
//!     SelectedAdditiveTrivia
//!     SelectedAcceptedIdentifierReference
//!     SelectedStatementTrailingTrivia
//!     AuthoredSemicolon
//!
//! SelectedAdditiveOperator ::= "+" | "-"
//!
//! SelectedAcceptedIdentifierReference ::=
//!     SelectedDirectIdentifierReference
//!   | SelectedEscapedNonReservedIdentifierReference
//! ```
//!
//! placed as a free-standing item of a non-strict `Script`, either directly
//! at the top level or inside one already-selected one-level Block. It does
//! not call production lexical, static-semantics, correspondence,
//! Binding/Scope, aggregate, or runtime evaluation code, and it does not
//! call or import any other candidate-independent Oracle's recognizer or
//! correspondence functions -- this leaf restates its own copy of the
//! primitives it needs, matching the convention already set by every
//! predecessor candidate-independent `IdentifierReference` oracle.
//!
//! Every accepted predecessor authority proves exactly one ingredient in
//! isolation: #752/#753 prove that one bounded additive expression can
//! retain exactly two ordered, independently source-backed
//! `IdentifierReference` facts; #756/#757 prove that one free-standing
//! top-level use-site occurrence owns its own source-name correspondence
//! relation; #760/#761 prove that one free-standing Block-contained
//! use-site occurrence owns its own relation together with exact containing
//! Block ownership and current-Block lexical precedence. No accepted
//! authority yet proves their composition: one free-standing structural
//! `ExpressionStatement` whose own semantic owners are two independently
//! provenance-bearing reference occurrences, each independently queried for
//! source-name correspondence, in exact authored intra-statement order.
//! This is the first free-standing use-site theorem this project
//! independently proves where Statement cardinality and relation
//! cardinality differ. Production must not become the first authority for
//! this combined theorem.
//!
//! The independent lifecycle mirrors every predecessor's `prepare candidate
//! evidence -> authoritative whole-source consumption -> static-neutral
//! accepted witness -> non-refusing relation commit` shape, driven end to
//! end by `recognize_and_accept_selected_script`:
//!
//!   - Layer 1 (`parse_selected_script`): source / placement / cardinality.
//!     Recognizes a whole candidate `Script` as an ordered sequence of
//!     exactly four bounded top-level item shapes -- a `let` binding item,
//!     a `var` binding item, a free-standing exactly-two-operand additive
//!     `ExpressionStatement` use-site item, or one already-selected
//!     one-level Block -- each terminated by an authored `;` (the Block
//!     item is terminated by its own authored `}`). A Block's own contents
//!     are recognized as an ordered sequence of one or more of exactly the
//!     same three non-Block item shapes; a Block containing a nested `{` is
//!     never attempted (bounded to one level, matching #760/#761). A `let`/
//!     `var` binding item's optional initializer independently restates
//!     both the already-accepted single-`SelectedAcceptedIdentifierReference`
//!     shape and the already-accepted #752/#753 two-operand additive shape,
//!     purely so a whole item such as `let x = a + b;` can be recognized
//!     end to end -- neither shape's inner reference is ever retained as a
//!     fact or contributes a Layer 2 relation of its own; the existing,
//!     completely separate initializer relation authority owned by
//!     `selected_variable_statement_name_correspondence.rs` is never
//!     touched. Recognition is whole-source, all-or-nothing at every level:
//!     any unrecognized item aborts the complete parse (`None`), so a
//!     locally valid earlier item -- including a locally valid left operand
//!     whose right operand cannot complete -- can never publish evidence
//!     when a later item or operand fails (whole-source transactionality).
//!
//!   - Static preflight (`preflight_selected_static_semantics`): a bounded,
//!     candidate-independent gate independently restating -- never
//!     importing -- only the existing rules relevant to this shell, exactly
//!     as #760/#761 restates them: a duplicate selected lexical name is
//!     rejected within its own region only; a TopLevel selected lexical
//!     name colliding with any whole-Script selected `var` name is
//!     rejected; a Block's own selected lexical name colliding with that
//!     same Block's own selected `var` name is rejected. Duplicate `var`
//!     contributors alone remain allowed. The two free-standing use-site
//!     operands never contribute a `BoundNames`, `LexicallyDeclaredNames`,
//!     or `VarDeclaredNames` entry, and are never collision-key or Early
//!     Error subjects of their own; a syntactically recognized but
//!     statically rejected candidate never reaches Layer 2.
//!
//!   - Layer 2 (`build_selected_top_level_additive_use_site_correspondence`
//!     and `build_selected_block_additive_use_site_correspondence`):
//!     relation. Each independently derives exactly one correspondence
//!     relation per accepted operand occurrence -- never one relation per
//!     additive Statement, and never one relation for the whole additive
//!     expression -- in exact authored occurrence order, without
//!     deduplication. The two result surfaces remain completely separate:
//!     a top-level free-standing additive use-site relation is never placed
//!     into one combined sequence with a Block-contained relation, and
//!     neither is ever combined with the existing, completely separate
//!     initializer relation surface or with #756/#757's or #760/#761's own
//!     single-occurrence relation surfaces. There is no new global
//!     initializer/top-level/Block relation ordering contract, and no
//!     generic region-polymorphic relation type: the TopLevel relation
//!     carries no containing-Block field at all, and the Block relation's
//!     `containing_block` is always a real Block anchor, never an
//!     `Option<_>` or a `TopLevel` placeholder.
//!
//! `classify_selected_script` binds this whole lifecycle to an executable
//! disposition (`Selected` / `UnsupportedCoverage` / `StaticSemanticsRejected`
//! / `DefinitiveGrammarRejectionEvidence` / `ResourceLimited` /
//! `InternalFailure`), matching the vocabulary already established by every
//! predecessor: an authored-semicolon-only boundary means `a+b` (no
//! semicolon) and `{ a+b }` (close-brace ASI) both independently classify
//! as `UnsupportedCoverage`, never grammar rejection and never composing
//! #717's automatic-semicolon-before-close-brace precedent.
//!
//! `SelectedDirectIdentifierReference` independently restates the
//! already-accepted Issue #237 direct escape-free `IdentifierName`
//! code-point shape minus the unconditionally reserved words.
//! `SelectedEscapedNonReservedIdentifierReference` independently restates
//! the already-accepted Issue #241 `UnicodeEscapeSequence` decode/position/
//! decoded-reserved-word theorem. Selected trivia independently restates
//! the complete already-accepted selected-slice trivia contract established
//! by Issue #742/#743. None of these restatements import their originating
//! oracle's code.
//!
//! The additive operator is recognized as exactly one authored `+` or `-`
//! between the two operands, but -- matching #752/#753's own decision, for
//! the same reason -- no current correspondence consumer needs the operator
//! as retained semantic payload, so it is never retained in any fact or
//! relation this oracle produces (no operator enum, no operator
//! `SourceAnchor`). Likewise no whole Statement, whole Expression, or
//! semicolon `SourceAnchor` is retained: the two exact inner reference
//! occurrences remain the only semantic owners.
//!
//! This is a validation-only leaf: production supports no additive
//! `ExpressionStatement` family at the #764 baseline, so every positive
//! fixture below remains `UnsupportedCoverage` under current production and
//! exists only as independent Oracle evidence for a future, separately
//! authorized production decision. No completion successor file accompanies
//! this leaf: this Issue adds zero production capability, so the frozen
//! `193 / 10 / 183 / {}` completion partition cannot move, matching the
//! precedent set by #742/#743, #746/#747, #752/#753, #756/#757, and
//! #760/#761. This oracle intentionally does not decide whether a future
//! production representation widens an existing use-site payload, adds
//! additive-specific sibling item variants, introduces a new shared bounded
//! carrier, introduces a new broader Script/Block capability layer, or
//! reuses `SelectedIdentifierReferenceInitializer::One|Two` outside
//! initializer ownership: that decision belongs to a separate, later
//! authorized production representation / placement step.
//!
//! Runtime-negative boundary: `NoSelectedSameSourceContributor` is not
//! runtime-unbound, a `ReferenceError`, or a `ResolveBinding` failure.
//! `VisibleSelectedLexicalBinding` is not `ResolveBinding`, `GetValue`, a
//! Reference Record, an Environment Record, or TDZ execution state. Forward
//! lexical source correspondence -- a use-site operand preceding its target
//! declaration in authored order -- is explicitly source correspondence
//! only, never a runtime-TDZ-success claim. This oracle proves same-source
//! selected declaration provenance only.

use std::collections::HashMap;

use crate::{SourceId, SourceText};

use super::unicode::{is_id_continue, is_id_start, is_space_separator};
use super::unicode_generated::{
    ECMA262_SNAPSHOT as FROZEN_ECMA262_SNAPSHOT, UNICODE_VERSION as FROZEN_UNICODE_VERSION,
};

const ISSUE_ID: u64 = 764;
const ECMA_262_EDITION: &str = "ECMA-262, 17th edition, 2026";
const ECMA_262_SNAPSHOT: &str = "d89c03f2db8a597bc915b363a6518d0cc8acdbc0";
const UNICODE_VERSION: &str = "17.0.0";
const MODEL_SOURCE: &str = include_str!("qualification_validation_tests/model.rs");
const PREDECESSOR_ADDITIVE_INITIALIZER_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_initializer_validation_tests.rs"
);
const PREDECESSOR_TOP_LEVEL_USE_SITE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_top_level_identifier_reference_expression_statement_use_site_validation_tests.rs"
);
const PREDECESSOR_BLOCK_USE_SITE_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_one_level_block_identifier_reference_expression_statement_use_site_validation_tests.rs"
);
const THIS_ORACLE_SOURCE: &str = include_str!(
    "qualification_selected_two_identifier_reference_additive_expression_statement_use_site_validation_tests.rs"
);
const FRONTIER_SCOPE_NOTE: &str = concat!(
    "exactly-two IdentifierReference additive free-standing ExpressionStatement ",
    "use-site frontier only, across accepted TopLevel and one-level Block ",
    "placements; later independently qualified owners may strengthen ",
    "classification for third-or-more additive operands, unary/parenthesized/",
    "member/call operands, richer Expression neighbors, close-brace or EOF ",
    "ASI-terminated use-sites, or recursive/deeper Block nesting"
);

/// Preserved distinctly from `UnsupportedCoverage`, matching the minimal
/// symbolic model already accepted by every predecessor candidate-independent
/// `IdentifierReference` oracle -- this leaf adds no further resource
/// machinery beyond that already-accepted minimum.
const PROCESSING_FAILURES: &[&str] = &["ResourceLimited", "InternalFailure"];

/// The four failure classifications this oracle keeps textually distinct: a
/// valid-but-unselected ASI boundary is `UnsupportedCoverage`, never
/// `DefinitiveGrammarRejectionEvidence`.
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
// #237/#752/#756/#760, never imported from any of them. ---

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
/// words -- also reused as this oracle's bounded `BindingIdentifier`
/// policy, since no fixture below requires an escaped or reserved-word
/// declarator.
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

/// `SelectedEscapedNonReservedIdentifierReference`: decodes a whole
/// candidate spelling that must contain at least one authored
/// `UnicodeEscapeSequence`, judging start/part position validity per
/// decoded element and rejecting a decoded-reserved whole name --
/// independently restated from Issue #241.
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

// --- The Issue #764 theorem itself. ---

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
/// escape-free `IdentifierName` code-point shape only (this oracle's
/// bounded declarator policy never accepts an escaped or reserved-word
/// declarator).
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
/// at the first code point that cannot continue an identifier-shaped run
/// (including trivia and the `+`/`-` additive operator, since neither is a
/// direct identifier part). A malformed escape aborts the whole run
/// immediately (`None`) rather than silently truncating at that point --
/// the malformed escape never leaks an earlier valid-looking prefix.
/// Position validity of the collected run is judged afterward by
/// `recognize_accepted_identifier_reference`, never here.
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

/// The current containing region of one recognized relation: either the
/// enclosing top-level `Script`, or the exact containing one-level Block's
/// own authored anchor. Deliberately not an `Option<_>` -- `TopLevel` is a
/// first-class region, never an absent Block (matching #760/#761).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SelectedRegion {
    TopLevel,
    Block(Range),
}

/// One retained `let`/`var` binding fact: exactly the declarator name and
/// its own exact authored anchor. Its optional initializer -- either
/// `SelectedAcceptedIdentifierReference` or the already-accepted #752/#753
/// two-operand additive shape -- is recognized only to prove the whole
/// bounded item grammar (never retained as a fact, and never contributes a
/// Layer 2 relation of its own).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedBindingFact {
    semantic_name: String,
    binding: Range,
}

/// One retained additive use-site operand: this oracle's per-occurrence
/// semantic owner. Its containing region and containing Block, when
/// applicable, are never stored here -- determined only by where the
/// enclosing item was recognized, matching the accepted predecessor
/// convention of keeping the recognized fact itself free of any
/// generic/optional containing-owner field (W5/W6/W11).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedUseSiteOperand {
    reference: Range,
    semantic_name: String,
    spelling: IdentifierSpellingKind,
}

/// One retained free-standing additive use-site fact: the theorem this
/// oracle exists to validate. Exactly two ordered operand facts and nothing
/// else -- no retained operator enum or `SourceAnchor` (W14), no
/// third-operand slot (W15/W16), and no single combined expression-owned
/// relation field (W2/W12/W13).
#[derive(Debug, Clone, PartialEq, Eq)]
struct RecognizedAdditiveUseSiteFact {
    left: RecognizedUseSiteOperand,
    right: RecognizedUseSiteOperand,
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
    AdditiveUseSite(RecognizedAdditiveUseSiteFact),
}

/// One recognized one-level Block: its own exact authored anchor (spanning
/// both braces) and its own ordered item sequence. This oracle's Block is
/// bounded to exactly one level: `items` never contains a nested Block
/// variant, and recognizing a Block's own contents never attempts to
/// recognize `{` as a fourth shape (no recursive/deeper Block support,
/// W24).
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
    AdditiveUseSite(RecognizedAdditiveUseSiteFact),
    Block(RecognizedBlockFact),
}

/// Skips one `let`/`var` binding initializer RHS starting at `offset`:
/// either `SelectedAcceptedIdentifierReference` alone (the already-accepted
/// single-reference initializer shape), or the already-accepted #752/#753
/// `SelectedTwoIdentifierReferenceAdditiveInitializer` shape. Independently
/// restated -- never imported -- purely so a surrounding whole item such as
/// `let x = a + b;` can be recognized end to end; neither the single
/// reference nor either additive operand is ever retained as a fact or
/// contributes a Layer 2 relation of its own (the existing, completely
/// separate initializer relation authority owned by
/// `selected_variable_statement_name_correspondence.rs` is never touched).
/// Returns the absolute offset immediately after the recognized RHS.
fn skip_selected_binding_initializer(source: &str, offset: usize) -> Option<usize> {
    let first_len = identifier_reference_run_end(&source[offset..])?;
    let first_slice = &source[offset..offset + first_len];
    recognize_accepted_identifier_reference(first_slice)?;
    let mut cursor = offset + first_len;

    let after_first_trivia_len = trivia_run_end(&source[cursor..]);
    let after_first = &source[cursor + after_first_trivia_len..];
    if matches!(after_first.as_bytes().first(), Some(b'+') | Some(b'-')) {
        let mut inner = cursor + after_first_trivia_len + 1;
        inner += trivia_run_end(&source[inner..]);
        let second_len = identifier_reference_run_end(&source[inner..])?;
        let second_slice = &source[inner..inner + second_len];
        recognize_accepted_identifier_reference(second_slice)?;
        cursor = inner + second_len;
    }

    Some(cursor)
}

/// Recognizes one `let`/`var` binding item starting at `offset`: the
/// keyword, a mandatory trivia boundary, a direct `BindingIdentifier`, an
/// optional `= <initializer>` (see `skip_selected_binding_initializer`),
/// and a mandatory authored `;`. Returns the recognized fact and the
/// absolute offset immediately after the authored `;`. Reused identically
/// for a top-level binding item and a Block-contained binding item.
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
        cursor = skip_selected_binding_initializer(source, cursor)?;
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
/// `SelectedTwoIdentifierReferenceAdditiveExpressionStatement`-shaped
/// use-site item starting at `offset`: the theorem itself, minus its
/// placement (placement is judged only by which sequence this function is
/// called from -- top-level or a Block's own contents). Requires exactly
/// two accepted operands joined by exactly one authored `+` or `-`, with
/// trivia permitted around the operator and before the mandatory authored
/// `;`. A malformed, invalid-position, escaped-ReservedWord, or otherwise
/// unrecognized right operand aborts the whole item (`None`); the already
/// locally valid left operand never leaks on its own (W21). Returns the
/// recognized fact and the absolute offset immediately after the authored
/// `;`.
fn parse_additive_use_site(
    source: &str,
    offset: usize,
) -> Option<(RecognizedAdditiveUseSiteFact, usize)> {
    let left_run_len = identifier_reference_run_end(&source[offset..])?;
    let left_slice = &source[offset..offset + left_run_len];
    let (left_semantic_name, left_spelling) = recognize_accepted_identifier_reference(left_slice)?;
    let left = RecognizedUseSiteOperand {
        reference: Range(offset, offset + left_run_len),
        semantic_name: left_semantic_name,
        spelling: left_spelling,
    };

    let mut cursor = offset + left_run_len;
    cursor += trivia_run_end(&source[cursor..]);
    match source.as_bytes().get(cursor) {
        Some(b'+') | Some(b'-') => {}
        _ => return None,
    }
    cursor += 1;
    cursor += trivia_run_end(&source[cursor..]);

    let right_run_len = identifier_reference_run_end(&source[cursor..])?;
    let right_slice = &source[cursor..cursor + right_run_len];
    let (right_semantic_name, right_spelling) =
        recognize_accepted_identifier_reference(right_slice)?;
    let right = RecognizedUseSiteOperand {
        reference: Range(cursor, cursor + right_run_len),
        semantic_name: right_semantic_name,
        spelling: right_spelling,
    };
    cursor += right_run_len;

    cursor += trivia_run_end(&source[cursor..]);
    if !source[cursor..].starts_with(';') {
        return None;
    }
    let end = cursor + 1;

    Some((RecognizedAdditiveUseSiteFact { left, right }, end))
}

/// Recognizes one non-Block item (`let`/`var` binding or free-standing
/// additive use-site) at `offset`, in that dispatch order -- matching the
/// already accepted #756/#757/#760/#761 precedence so `{ let a; }` /
/// `{ var a; }` remain recognized as declarations rather than
/// reclassified, while a keyword attempt that cannot form a complete
/// binding falls through to the additive-use-site route (no generic
/// Statement dispatch is introduced).
fn parse_non_block_item(source: &str, offset: usize) -> Option<(RecognizedNonBlockItem, usize)> {
    if let Some((fact, end)) = parse_binding(source, offset, "let") {
        return Some((RecognizedNonBlockItem::LexicalBinding(fact), end));
    }
    if let Some((fact, end)) = parse_binding(source, offset, "var") {
        return Some((RecognizedNonBlockItem::VarBinding(fact), end));
    }
    let (fact, end) = parse_additive_use_site(source, offset)?;
    Some((RecognizedNonBlockItem::AdditiveUseSite(fact), end))
}

/// Recognizes one already-selected one-level Block starting at `offset`:
/// an authored `{`, an ordered sequence of one or more non-Block items, and
/// an authored `}`. An immediately closed Block (`{}`) never recognizes,
/// matching #760/#761's own `UnsupportedCoverage` treatment. Never
/// attempts to recognize a nested `{` as a fourth Block-contained item
/// shape -- a Block containing another Block is never matched, aborting
/// the whole parse (`None`), which is exactly how this oracle keeps itself
/// bounded to one level (W24). Returns the recognized fact (its own anchor
/// spans both braces) and the absolute offset immediately after the
/// authored `}`.
fn parse_block(source: &str, offset: usize) -> Option<(RecognizedBlockFact, usize)> {
    if !source[offset..].starts_with('{') {
        return None;
    }

    let mut cursor = offset + 1;
    cursor += trivia_run_end(&source[cursor..]);

    if source[cursor..].starts_with('}') {
        return None;
    }

    let mut items = Vec::new();
    loop {
        if cursor >= source.len() {
            return None;
        }
        let (item, next_offset) = parse_non_block_item(source, cursor)?;
        items.push(item);
        cursor = next_offset;
        cursor += trivia_run_end(&source[cursor..]);
        if source[cursor..].starts_with('}') {
            break;
        }
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

/// Central Issue #764 Layer 1 theorem. Whole-source, all-or-nothing at
/// every level: a top-level item sequence is never returned unless every
/// top-level item, in exact authored order, recognizes as one of the four
/// bounded shapes with nothing left over, and a Block item is never
/// returned unless every one of its own contained items recognizes as one
/// of the three bounded non-Block shapes with nothing left over. A single
/// unrecognized item at any level -- a richer Expression neighbor, a
/// third-or-more additive operand, a nested Block, an escaped
/// ReservedWord, a malformed escape, an ASI-only boundary, or trailing
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
                RecognizedNonBlockItem::AdditiveUseSite(fact) => {
                    RecognizedTopLevelItem::AdditiveUseSite(fact)
                }
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
/// only consume this witness, never an arbitrary `Vec<RecognizedTopLevelItem>`.
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
/// owning region that independently proved it.
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
/// rules relevant to this bounded shell, exactly as #760/#761 restates
/// them:
///
///   - a duplicate selected lexical name is rejected within its own region
///     only (TopLevel among itself; each Block among its own items only);
///   - a TopLevel selected lexical name colliding with any whole-Script
///     selected `var` name (TopLevel- or Block-contributed) is rejected;
///   - a Block's own selected lexical name colliding with that same
///     Block's own selected `var` name is rejected.
///
/// Duplicate `var` contributors alone remain allowed, at any region. Every
/// pass walks items in authored order, so reported rejection anchors are
/// deterministic regardless of any internal map iteration order. This is
/// not a general Early Error engine: it proves only this bounded shell's
/// own already-accepted rules. The two free-standing additive use-site
/// operands never contribute a name to any of these maps.
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
            RecognizedTopLevelItem::LexicalBinding(_)
            | RecognizedTopLevelItem::AdditiveUseSite(_) => {}
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

/// The oracle's full independent lifecycle front door: source/placement/
/// cardinality recognition (Layer 1), then the bounded static preflight
/// gate, producing the only shape Layer 2 may consume. This convenience
/// accessor discards the specific rejection reason on failure;
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
/// disconnected list. A valid-but-unselected ASI boundary such as `a+b`
/// (EOF) or `{ a+b }` (close-brace) independently classifies as
/// `UnsupportedCoverage`, never `DefinitiveGrammarRejectionEvidence`. A
/// syntactically well-formed but statically rejected shell independently
/// classifies as `StaticSemanticsRejected`, never downgraded to
/// `UnsupportedCoverage`. This bounded recognizer never independently
/// proves definitive grammar rejection, so it never constructs that
/// variant in practice; the variant exists so the type itself preserves the
/// project's failure vocabulary distinction, matching the already-accepted
/// `ResourceLimited`/`InternalFailure` symbolic minimum.
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

/// One independently derived Layer 2 TopLevel source-name correspondence
/// relation. Exactly the theorem's vocabulary and nothing else: no
/// `Statement`/whole-expression/operator/semicolon anchor, and no field
/// mirroring the existing initializer relation's distinct load-bearing
/// declaration-owner field. This is the TopLevel-only correspondence
/// vocabulary, matching #756/#757's own shape exactly (no region field: a
/// TopLevel relation's visible lexical binding is always a TopLevel
/// binding).
#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedTopLevelSourceNameCorrespondence {
    VisibleSelectedLexicalBinding { binding: Range },
    SameSourceSelectedVarNameContributors { contributors: Vec<Range> },
    NoSelectedSameSourceContributor,
}

/// One retained top-level free-standing additive use-site relation: the
/// operand occurrence, its semantic name, and its source-name
/// correspondence. There is no containing-Block field at all (W11): a
/// top-level operand never receives one of these relations, so this type
/// is never given an optional or generic region field merely because this
/// oracle also validates Block placement.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedTopLevelAdditiveUseSiteRelation {
    reference: Range,
    semantic_name: String,
    correspondence: SelectedTopLevelSourceNameCorrespondence,
}

/// One independently derived Layer 2 Block source-name correspondence
/// relation. Carries `region` on its lexical variant (current Block vs.
/// TopLevel), matching #760/#761's own shape exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectedBlockSourceNameCorrespondence {
    VisibleSelectedLexicalBinding {
        binding: Range,
        region: SelectedRegion,
    },
    SameSourceSelectedVarNameContributors {
        contributors: Vec<Range>,
    },
    NoSelectedSameSourceContributor,
}

/// One retained Block-contained free-standing additive use-site relation:
/// the operand occurrence, its exact containing Block's own anchor, its
/// semantic name, and its source-name correspondence. This relation
/// surface is Block-only -- a TopLevel operand never receives one of these
/// relations, so `containing_block` is always a real Block anchor, never a
/// `TopLevel` placeholder or an `Option<_>`.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedBlockAdditiveUseSiteRelation {
    containing_block: Range,
    reference: Range,
    semantic_name: String,
    correspondence: SelectedBlockSourceNameCorrespondence,
}

/// Collects the whole-Script declaration context both Layer 2 builders
/// share: every TopLevel selected lexical binding (never Block-contributed
/// -- lexical visibility is region-scoped), and every selected `var`
/// contributor across the whole Script, TopLevel- or Block-contributed
/// (`var` provenance is Script-wide, matching #760/#761). Collected from
/// the *whole* accepted item sequence before any correspondence is
/// computed, so a declaration appearing after a use-site operand still
/// corresponds (the forward source correspondence theorem).
fn collect_top_level_lexical_and_script_var_contributors(
    items: &[RecognizedTopLevelItem],
) -> (HashMap<&str, Range>, HashMap<&str, Vec<Range>>) {
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
            RecognizedTopLevelItem::AdditiveUseSite(_) => {}
        }
    }

    (top_lexical, script_var_contributors)
}

/// The already-accepted TopLevel correspondence precedence: TopLevel
/// selected lexical binding, then all selected same-source `var`
/// contributors, then `NoSelectedSameSourceContributor`.
fn top_level_correspondence(
    name: &str,
    top_lexical: &HashMap<&str, Range>,
    script_var_contributors: &HashMap<&str, Vec<Range>>,
) -> SelectedTopLevelSourceNameCorrespondence {
    if let Some(binding) = top_lexical.get(name) {
        SelectedTopLevelSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: *binding,
        }
    } else if let Some(contributors) = script_var_contributors.get(name) {
        SelectedTopLevelSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: contributors.clone(),
        }
    } else {
        SelectedTopLevelSourceNameCorrespondence::NoSelectedSameSourceContributor
    }
}

/// The already-accepted Block correspondence precedence: current Block
/// selected lexical binding, then enclosing TopLevel selected lexical
/// binding, then all selected same-source `var` contributors, then
/// `NoSelectedSameSourceContributor`. A same-Block lexical binding always
/// wins over an outer lexical binding of the same name, and sibling Block
/// lexical bindings never leak in (`current_block_lexical` is that exact
/// Block's own map only).
fn block_correspondence(
    name: &str,
    current_block_lexical: &HashMap<&str, Range>,
    containing_block: Range,
    top_lexical: &HashMap<&str, Range>,
    script_var_contributors: &HashMap<&str, Vec<Range>>,
) -> SelectedBlockSourceNameCorrespondence {
    if let Some(binding) = current_block_lexical.get(name) {
        SelectedBlockSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: *binding,
            region: SelectedRegion::Block(containing_block),
        }
    } else if let Some(binding) = top_lexical.get(name) {
        SelectedBlockSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: *binding,
            region: SelectedRegion::TopLevel,
        }
    } else if let Some(contributors) = script_var_contributors.get(name) {
        SelectedBlockSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: contributors.clone(),
        }
    } else {
        SelectedBlockSourceNameCorrespondence::NoSelectedSameSourceContributor
    }
}

/// Central Issue #764 Layer 2 TopLevel theorem. Consumes only the bounded
/// static preflight's accepted witness and derives exactly two relations
/// per top-level additive use-site item -- one per operand, in exact
/// authored left-then-right order -- in exact authored item occurrence
/// order, without reordering or deduplication (W1/W8/W10). A
/// Block-contained additive use-site item never receives a relation from
/// this builder (that distinct, completely separate Block-only relation
/// surface belongs only to
/// `build_selected_block_additive_use_site_correspondence`); no relation
/// this builder returns is ever placed into one combined sequence with a
/// Block or initializer-owned relation (W9/W10).
fn build_selected_top_level_additive_use_site_correspondence(
    accepted: &AcceptedSelectedScript,
) -> Vec<SelectedTopLevelAdditiveUseSiteRelation> {
    let items = accepted.items();
    let (top_lexical, script_var_contributors) =
        collect_top_level_lexical_and_script_var_contributors(items);

    let mut relations = Vec::new();
    for item in items {
        let RecognizedTopLevelItem::AdditiveUseSite(fact) = item else {
            continue;
        };
        for operand in [&fact.left, &fact.right] {
            let correspondence = top_level_correspondence(
                &operand.semantic_name,
                &top_lexical,
                &script_var_contributors,
            );
            relations.push(SelectedTopLevelAdditiveUseSiteRelation {
                reference: operand.reference,
                semantic_name: operand.semantic_name.clone(),
                correspondence,
            });
        }
    }

    relations
}

/// Central Issue #764 Layer 2 Block theorem. Consumes only the bounded
/// static preflight's accepted witness and derives exactly two relations
/// per Block-contained additive use-site item -- one per operand, in exact
/// authored left-then-right order -- in exact authored occurrence order
/// within its own containing Block, without reordering or deduplication.
/// Each Block's own lexical map is collected from that Block's *whole* own
/// item sequence before any of that Block's own operands are resolved, and
/// never leaks into or receives from a sibling Block (W10 sibling-lexical-
/// leakage firewall). A top-level additive use-site item never receives a
/// relation from this builder (that distinct, completely separate
/// TopLevel-only relation surface belongs only to
/// `build_selected_top_level_additive_use_site_correspondence`).
fn build_selected_block_additive_use_site_correspondence(
    accepted: &AcceptedSelectedScript,
) -> Vec<SelectedBlockAdditiveUseSiteRelation> {
    let items = accepted.items();
    let (top_lexical, script_var_contributors) =
        collect_top_level_lexical_and_script_var_contributors(items);

    let mut relations = Vec::new();
    for item in items {
        let RecognizedTopLevelItem::Block(block) = item else {
            continue;
        };

        let mut block_lexical: HashMap<&str, Range> = HashMap::new();
        for block_item in &block.items {
            if let RecognizedNonBlockItem::LexicalBinding(fact) = block_item {
                block_lexical
                    .entry(fact.semantic_name.as_str())
                    .or_insert(fact.binding);
            }
        }

        for block_item in &block.items {
            let RecognizedNonBlockItem::AdditiveUseSite(fact) = block_item else {
                continue;
            };
            for operand in [&fact.left, &fact.right] {
                let correspondence = block_correspondence(
                    &operand.semantic_name,
                    &block_lexical,
                    block.anchor,
                    &top_lexical,
                    &script_var_contributors,
                );
                relations.push(SelectedBlockAdditiveUseSiteRelation {
                    containing_block: block.anchor,
                    reference: operand.reference,
                    semantic_name: operand.semantic_name.clone(),
                    correspondence,
                });
            }
        }
    }

    relations
}

#[test]
fn authority_independence_and_frontier_scope_are_exact() {
    assert_eq!(ISSUE_ID, 764);
    assert!(MODEL_SOURCE.contains(ECMA_262_EDITION));
    assert!(MODEL_SOURCE.contains(ECMA_262_SNAPSHOT));
    assert!(MODEL_SOURCE.contains(UNICODE_VERSION));
    assert_eq!(FROZEN_ECMA262_SNAPSHOT, ECMA_262_SNAPSHOT);
    assert_eq!(FROZEN_UNICODE_VERSION, UNICODE_VERSION);

    assert!(PREDECESSOR_ADDITIVE_INITIALIZER_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 752"));
    assert!(PREDECESSOR_TOP_LEVEL_USE_SITE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 756"));
    assert!(PREDECESSOR_BLOCK_USE_SITE_ORACLE_SOURCE.contains("ISSUE_ID: u64 = 760"));
    assert!(
        FRONTIER_SCOPE_NOTE
            .contains("exactly-two IdentifierReference additive free-standing ExpressionStatement")
    );
    assert!(FRONTIER_SCOPE_NOTE.contains("may strengthen classification"));

    assert_eq!(PROCESSING_FAILURES, ["ResourceLimited", "InternalFailure"]);
    assert_eq!(
        FAILURE_CLASSIFICATIONS,
        [
            "UnsupportedCoverage",
            "DefinitiveGrammarRejectionEvidence",
            "ResourceLimited",
            "InternalFailure",
        ]
    );
    assert!(!PROCESSING_FAILURES.contains(&"UnsupportedCoverage"));

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
        concat!("consume_selected_", "identifier_reference"),
        concat!("analyze_selected_binding_", "scope("),
        concat!("parse_", "declaration"),
        concat!("parse_variable_", "statement"),
        concat!("parse_selected_block_var_", "statement"),
        concat!("parse_unary_", "expression"),
        concat!("parse_additive_", "expression"),
        // No cross-Oracle import: this leaf restates rather than imports.
        concat!(
            "use super::qualification_selected_two_identifier_reference_additive_",
            "initializer_validation_tests"
        ),
        concat!(
            "use super::qualification_selected_top_level_identifier_reference_",
            "expression_statement_use_site_validation_tests"
        ),
        concat!(
            "use super::qualification_selected_one_level_block_identifier_",
            "reference_expression_statement_use_site_validation_tests"
        ),
        // No production representation frozen by this Oracle (W12/W13/W27).
        concat!("struct Statement", "SourceAnchor"),
        concat!("struct WholeExpression", "SourceAnchor"),
        concat!("struct Operator", "SourceAnchor"),
        concat!("struct Semicolon", "SourceAnchor"),
        concat!("enum Selected", "Statement"),
        concat!("enum Selected", "Expression"),
        // Forbidden reconstruction of expected ranges.
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
}

/// Positive TopLevel Direct/Escaped matrix (acceptance criteria 1, 2, 4;
/// Issue #764 minimum matrix): both operators, direct/direct, escaped/
/// direct, direct/escaped, escaped/escaped, and a mid-identifier escape
/// pair, each with fixture-owned literal byte ranges for left operand and
/// right operand -- never derived by search, rescan, or reparse. Exactly
/// one top-level item is recognized, its two operands retain their own
/// exact authored anchors, `Direct`/`EscapedNonReserved` state, and decoded
/// semantic name, cross-checked against the real `SourceText`/`SourceAnchor`
/// Core primitive.
#[test]
fn positive_top_level_direct_and_escaped_operand_matrix_pins_exact_provenance() {
    // Fixture row: (source, left range, left name, left spelling, right
    // range, right name, right spelling).
    type PositiveMatrixRow = (
        &'static str,
        Range,
        &'static str,
        IdentifierSpellingKind,
        Range,
        &'static str,
        IdentifierSpellingKind,
    );
    const POSITIVE_MATRIX: &[PositiveMatrixRow] = &[
        (
            "a+b;",
            Range(0, 1),
            "a",
            IdentifierSpellingKind::Direct,
            Range(2, 3),
            "b",
            IdentifierSpellingKind::Direct,
        ),
        (
            "a-b;",
            Range(0, 1),
            "a",
            IdentifierSpellingKind::Direct,
            Range(2, 3),
            "b",
            IdentifierSpellingKind::Direct,
        ),
        (
            "a + b;",
            Range(0, 1),
            "a",
            IdentifierSpellingKind::Direct,
            Range(4, 5),
            "b",
            IdentifierSpellingKind::Direct,
        ),
        (
            "\u{5c}u0061+b;",
            Range(0, 6),
            "a",
            IdentifierSpellingKind::EscapedNonReserved,
            Range(7, 8),
            "b",
            IdentifierSpellingKind::Direct,
        ),
        (
            "a+\u{5c}u0062;",
            Range(0, 1),
            "a",
            IdentifierSpellingKind::Direct,
            Range(2, 8),
            "b",
            IdentifierSpellingKind::EscapedNonReserved,
        ),
        (
            "\u{5c}u0061+\u{5c}u0062;",
            Range(0, 6),
            "a",
            IdentifierSpellingKind::EscapedNonReserved,
            Range(7, 13),
            "b",
            IdentifierSpellingKind::EscapedNonReserved,
        ),
        (
            "f\u{5c}u006Fo-b\u{5c}u0061r;",
            Range(0, 8),
            "foo",
            IdentifierSpellingKind::EscapedNonReserved,
            Range(9, 17),
            "bar",
            IdentifierSpellingKind::EscapedNonReserved,
        ),
    ];

    for (index, &(source, left_range, left_name, left_kind, right_range, right_name, right_kind)) in
        POSITIVE_MATRIX.iter().enumerate()
    {
        let disposition = classify_selected_script(source);
        let SelectedScriptDisposition::Selected(accepted) = disposition else {
            panic!("{source:?} must independently classify as Selected");
        };
        let items = accepted.items();
        assert_eq!(items.len(), 1, "{source:?}");
        let RecognizedTopLevelItem::AdditiveUseSite(fact) = &items[0] else {
            panic!("{source:?} must recognize as the additive use-site theorem");
        };

        assert_eq!(fact.left.reference, left_range, "{source:?} left range");
        assert_eq!(fact.left.semantic_name, left_name, "{source:?} left name");
        assert_eq!(fact.left.spelling, left_kind, "{source:?} left spelling");
        assert_eq!(fact.right.reference, right_range, "{source:?} right range");
        assert_eq!(
            fact.right.semantic_name, right_name,
            "{source:?} right name"
        );
        assert_eq!(fact.right.spelling, right_kind, "{source:?} right spelling");

        assert_eq!(
            slice(source, left_range),
            authored_anchor(764_000 + index as u64, source, left_range)
        );
        assert_eq!(
            slice(source, right_range),
            authored_anchor(764_100 + index as u64, source, right_range)
        );
    }
}

/// Positive Block Direct/Escaped matrix (acceptance criteria 1, 2, 3, 4):
/// the same bounded theorem, placed inside one already-selected one-level
/// Block, retaining the Block's own exact authored anchor in addition to
/// the two operand facts.
#[test]
fn positive_block_direct_and_escaped_operand_matrix_pins_exact_provenance() {
    // Fixture row: (source, block anchor, left range, left name, left
    // spelling, right range, right name, right spelling).
    type PositiveMatrixRow = (
        &'static str,
        Range,
        Range,
        &'static str,
        IdentifierSpellingKind,
        Range,
        &'static str,
        IdentifierSpellingKind,
    );
    const POSITIVE_MATRIX: &[PositiveMatrixRow] = &[
        (
            "{ a+b; }",
            Range(0, 8),
            Range(2, 3),
            "a",
            IdentifierSpellingKind::Direct,
            Range(4, 5),
            "b",
            IdentifierSpellingKind::Direct,
        ),
        (
            "{ \u{5c}u0061-b; }",
            Range(0, 13),
            Range(2, 8),
            "a",
            IdentifierSpellingKind::EscapedNonReserved,
            Range(9, 10),
            "b",
            IdentifierSpellingKind::Direct,
        ),
    ];

    for &(
        source,
        block_anchor,
        left_range,
        left_name,
        left_kind,
        right_range,
        right_name,
        right_kind,
    ) in POSITIVE_MATRIX
    {
        let disposition = classify_selected_script(source);
        let SelectedScriptDisposition::Selected(accepted) = disposition else {
            panic!("{source:?} must independently classify as Selected");
        };
        let items = accepted.items();
        assert_eq!(items.len(), 1, "{source:?}");
        let RecognizedTopLevelItem::Block(block) = &items[0] else {
            panic!("{source:?} must recognize as a top-level Block item");
        };
        assert_eq!(block.anchor, block_anchor, "{source:?} block anchor");
        assert_eq!(block.items.len(), 1, "{source:?}");
        let RecognizedNonBlockItem::AdditiveUseSite(fact) = &block.items[0] else {
            panic!("{source:?} must recognize a Block-contained additive use-site");
        };

        assert_eq!(fact.left.reference, left_range, "{source:?} left range");
        assert_eq!(fact.left.semantic_name, left_name, "{source:?} left name");
        assert_eq!(fact.left.spelling, left_kind, "{source:?} left spelling");
        assert_eq!(fact.right.reference, right_range, "{source:?} right range");
        assert_eq!(
            fact.right.semantic_name, right_name,
            "{source:?} right name"
        );
        assert_eq!(fact.right.spelling, right_kind, "{source:?} right spelling");
    }
}

/// TopLevel relation theorem (acceptance criterion 5, W1/W2): one
/// free-standing additive Statement produces exactly two occurrence-owned
/// relations, never one relation for the whole Statement/Expression.
#[test]
fn top_level_relation_theorem_produces_two_ordered_occurrence_owned_relations() {
    let source = "a+b;";
    let accepted = recognize_and_accept_selected_script(source).expect("`a+b;` must be Selected");
    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].reference, Range(0, 1));
    assert_eq!(relations[0].semantic_name, "a");
    assert_eq!(relations[1].reference, Range(2, 3));
    assert_eq!(relations[1].semantic_name, "b");
    assert_eq!(
        relations[0].correspondence,
        SelectedTopLevelSourceNameCorrespondence::NoSelectedSameSourceContributor
    );
    assert_eq!(
        relations[1].correspondence,
        SelectedTopLevelSourceNameCorrespondence::NoSelectedSameSourceContributor
    );

    // The Block-only builder must publish nothing for a source with no Block.
    assert!(build_selected_block_additive_use_site_correspondence(&accepted).is_empty());
}

/// Block relation theorem (acceptance criterion 6, W1/W2): both relations
/// share the exact same containing Block anchor but retain distinct
/// reference occurrences.
#[test]
fn block_relation_theorem_two_relations_share_containing_block() {
    let source = "{ a+b; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`{ a+b; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].containing_block, Range(0, 8));
    assert_eq!(relations[1].containing_block, Range(0, 8));
    assert_eq!(relations[0].reference, Range(2, 3));
    assert_eq!(relations[0].semantic_name, "a");
    assert_eq!(relations[1].reference, Range(4, 5));
    assert_eq!(relations[1].semantic_name, "b");

    // The TopLevel-only builder must publish nothing for a Block-only source.
    assert!(build_selected_top_level_additive_use_site_correspondence(&accepted).is_empty());
}

/// Required TopLevel per-operand discriminator (acceptance criteria 7, 8;
/// Issue #764 required discriminator): `let a; var b; a+b;` must prove `a`
/// lexical and `b` a `var` contributor, never collapsed to one shared
/// correspondence result for the whole additive expression.
#[test]
fn top_level_per_operand_independence_lexical_vs_var() {
    let source = "let a;\nvar b;\na+b;";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`let a; var b; a+b;` must be Selected");
    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].reference, Range(14, 15));
    assert_eq!(
        relations[0].correspondence,
        SelectedTopLevelSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5)
        }
    );
    assert_eq!(relations[1].reference, Range(16, 17));
    assert_eq!(
        relations[1].correspondence,
        SelectedTopLevelSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(11, 12)],
        }
    );
}

#[test]
fn top_level_per_operand_independence_var_vs_var() {
    let source = "var a;\nvar b;\na+b;";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`var a; var b; a+b;` must be Selected");
    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(
        relations[0].correspondence,
        SelectedTopLevelSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(4, 5)],
        }
    );
    assert_eq!(
        relations[1].correspondence,
        SelectedTopLevelSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(11, 12)],
        }
    );
}

/// Required TopLevel no-match discriminator (Issue #764 required
/// discriminator): `let a; a+z;` must prove `a` lexical and `z` no selected
/// same-source contributor.
#[test]
fn top_level_per_operand_independence_lexical_vs_no_match() {
    let source = "let a;\na+z;";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`let a; a+z;` must be Selected");
    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].reference, Range(7, 8));
    assert_eq!(
        relations[0].correspondence,
        SelectedTopLevelSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5)
        }
    );
    assert_eq!(relations[1].reference, Range(9, 10));
    assert_eq!(
        relations[1].correspondence,
        SelectedTopLevelSourceNameCorrespondence::NoSelectedSameSourceContributor
    );
}

/// Forward TopLevel source correspondence (acceptance criterion 9): a
/// free-standing use-site operand preceding its target declaration in
/// authored order still corresponds -- source correspondence only, never a
/// runtime-TDZ-success claim.
#[test]
fn top_level_forward_lexical_correspondence() {
    let source = "a+b;\nlet a;\nlet b;";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`a+b; let a; let b;` must be Selected");
    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].reference, Range(0, 1));
    assert_eq!(
        relations[0].correspondence,
        SelectedTopLevelSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(9, 10)
        }
    );
    assert_eq!(relations[1].reference, Range(2, 3));
    assert_eq!(
        relations[1].correspondence,
        SelectedTopLevelSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(16, 17)
        }
    );
}

#[test]
fn top_level_two_lexical_declarations_before_use() {
    let source = "let a;\nlet b;\na+b;";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`let a; let b; a+b;` must be Selected");
    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(
        relations[0].correspondence,
        SelectedTopLevelSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5)
        }
    );
    assert_eq!(
        relations[1].correspondence,
        SelectedTopLevelSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(11, 12)
        }
    );
}

/// Block current-Block lexical precedence over TopLevel (acceptance
/// criterion 7): both operands resolve to the Block's own lexical
/// bindings.
#[test]
fn block_current_block_lexical_precedence_over_top_level() {
    let source = "{ let a; let b; a+b; }";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`{ let a; let b; a+b; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    let block_anchor = Range(0, 22);
    assert_eq!(relations[0].containing_block, block_anchor);
    assert_eq!(relations[0].reference, Range(16, 17));
    assert_eq!(
        relations[0].correspondence,
        SelectedBlockSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(6, 7),
            region: SelectedRegion::Block(block_anchor),
        }
    );
    assert_eq!(relations[1].containing_block, block_anchor);
    assert_eq!(relations[1].reference, Range(18, 19));
    assert_eq!(
        relations[1].correspondence,
        SelectedBlockSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(13, 14),
            region: SelectedRegion::Block(block_anchor),
        }
    );
}

/// Required Block per-operand discriminator (acceptance criterion 7; Issue
/// #764 required discriminator): `let b; { let a; a+b; }` must prove `a`
/// resolves to the current Block's own lexical binding and `b` resolves to
/// the enclosing TopLevel lexical binding -- never collapsed to one shared
/// result.
#[test]
fn block_required_discriminator_current_block_vs_top_level() {
    let source = "let b;\n{ let a; a+b; }";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`let b; { let a; a+b; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    let block_anchor = Range(7, 22);
    assert_eq!(relations[0].reference, Range(16, 17));
    assert_eq!(
        relations[0].correspondence,
        SelectedBlockSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(13, 14),
            region: SelectedRegion::Block(block_anchor),
        }
    );
    assert_eq!(relations[1].reference, Range(18, 19));
    assert_eq!(
        relations[1].correspondence,
        SelectedBlockSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5),
            region: SelectedRegion::TopLevel,
        }
    );
}

/// Forward same-Block source correspondence (acceptance criterion 9; Issue
/// #764 required discriminator): `let b; { a+b; let a; }` must prove `a`
/// corresponds to the later same-Block lexical declaration and `b` to the
/// TopLevel lexical declaration.
#[test]
fn block_forward_same_block_lexical_correspondence() {
    let source = "let b;\n{ a+b; let a; }";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`let b; { a+b; let a; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    let block_anchor = Range(7, 22);
    assert_eq!(relations[0].reference, Range(9, 10));
    assert_eq!(
        relations[0].correspondence,
        SelectedBlockSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(18, 19),
            region: SelectedRegion::Block(block_anchor),
        }
    );
    assert_eq!(relations[1].reference, Range(11, 12));
    assert_eq!(
        relations[1].correspondence,
        SelectedBlockSourceNameCorrespondence::VisibleSelectedLexicalBinding {
            binding: Range(4, 5),
            region: SelectedRegion::TopLevel,
        }
    );
}

/// Sibling Block lexical no-leakage firewall (W10 adjacent): an unrelated
/// sibling Block's own lexical `a` must never leak into a later sibling
/// Block's own use-site correspondence.
#[test]
fn block_sibling_lexical_no_leakage() {
    let source = "{ let a; }\n{ a+b; }";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`{ let a; } { a+b; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    let second_block_anchor = Range(11, 19);
    assert_eq!(relations[0].containing_block, second_block_anchor);
    assert_eq!(relations[0].reference, Range(13, 14));
    assert_eq!(
        relations[0].correspondence,
        SelectedBlockSourceNameCorrespondence::NoSelectedSameSourceContributor
    );
    assert_eq!(relations[1].reference, Range(15, 16));
    assert_eq!(
        relations[1].correspondence,
        SelectedBlockSourceNameCorrespondence::NoSelectedSameSourceContributor
    );
}

/// Script-wide `var` provenance for a Block-contained operand (acceptance
/// criteria 6, 8): an outer TopLevel `var` contributor still corresponds
/// for a Block-contained use-site operand.
#[test]
fn block_script_wide_var_provenance_from_outer() {
    let source = "var a;\n{ a+b; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`var a; { a+b; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].reference, Range(9, 10));
    assert_eq!(
        relations[0].correspondence,
        SelectedBlockSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(4, 5)],
        }
    );
    assert_eq!(relations[1].reference, Range(11, 12));
    assert_eq!(
        relations[1].correspondence,
        SelectedBlockSourceNameCorrespondence::NoSelectedSameSourceContributor
    );
}

/// Forward Block-own `var` contributors (acceptance criterion 9 adjacent):
/// both operands corresponding to `var` contributors declared later in the
/// same Block.
#[test]
fn block_own_forward_var_contributors() {
    let source = "{ a+b; var a; var b; }";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`{ a+b; var a; var b; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(
        relations[0].correspondence,
        SelectedBlockSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(11, 12)],
        }
    );
    assert_eq!(
        relations[1].correspondence,
        SelectedBlockSourceNameCorrespondence::SameSourceSelectedVarNameContributors {
            contributors: vec![Range(18, 19)],
        }
    );
}

/// Duplicate occurrence theorem, TopLevel (acceptance criterion 10;
/// W5/W6/W7/W8): two authored occurrences of the same semantic name remain
/// two distinct relations, never collapsed by semantic name, authored
/// spelling, or correspondence result.
#[test]
fn top_level_duplicate_occurrences_not_deduplicated() {
    let source = "a+a;";
    let accepted = recognize_and_accept_selected_script(source).expect("`a+a;` must be Selected");
    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_ne!(relations[0].reference, relations[1].reference);
    assert_eq!(relations[0].semantic_name, relations[1].semantic_name);
    assert_eq!(relations[0].correspondence, relations[1].correspondence);
    assert_eq!(relations[0].reference, Range(0, 1));
    assert_eq!(relations[1].reference, Range(2, 3));
}

/// Duplicate occurrence theorem, Block (acceptance criterion 10).
#[test]
fn block_duplicate_occurrences_not_deduplicated() {
    let source = "{ a+a; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`{ a+a; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    assert_eq!(relations[0].containing_block, relations[1].containing_block);
    assert_ne!(relations[0].reference, relations[1].reference);
    assert_eq!(relations[0].semantic_name, relations[1].semantic_name);
    assert_eq!(relations[0].correspondence, relations[1].correspondence);
    assert_eq!(relations[0].reference, Range(2, 3));
    assert_eq!(relations[1].reference, Range(4, 5));
}

/// Intra-surface authored ordering, TopLevel (acceptance criterion 11):
/// `a+b;\nc-d;` produces relations in exact authored order `a, b, c, d`.
#[test]
fn top_level_intra_surface_authored_order() {
    let source = "a+b;\nc-d;";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`a+b; c-d;` must be Selected");
    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 4);
    let names: Vec<&str> = relations.iter().map(|r| r.semantic_name.as_str()).collect();
    assert_eq!(names, ["a", "b", "c", "d"]);
    assert_eq!(relations[0].reference, Range(0, 1));
    assert_eq!(relations[1].reference, Range(2, 3));
    assert_eq!(relations[2].reference, Range(5, 6));
    assert_eq!(relations[3].reference, Range(7, 8));
}

/// Intra-surface authored ordering, Block (acceptance criterion 11):
/// `{ a+b; c-d; }` produces relations in exact authored order `a, b, c, d`,
/// all sharing the one containing Block.
#[test]
fn block_intra_surface_authored_order() {
    let source = "{ a+b; c-d; }";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`{ a+b; c-d; }` must be Selected");
    let relations = build_selected_block_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 4);
    let names: Vec<&str> = relations.iter().map(|r| r.semantic_name.as_str()).collect();
    assert_eq!(names, ["a", "b", "c", "d"]);
    let block_anchor = Range(0, 13);
    for relation in &relations {
        assert_eq!(relation.containing_block, block_anchor);
    }
    assert_eq!(relations[0].reference, Range(2, 3));
    assert_eq!(relations[1].reference, Range(4, 5));
    assert_eq!(relations[2].reference, Range(7, 8));
    assert_eq!(relations[3].reference, Range(9, 10));
}

/// No cross-surface global ordering theorem (acceptance criterion 12,
/// W9/W10/W11): `a+b;\n{ c-d; }\ne+f;` keeps the TopLevel surface `a, b, e,
/// f` and the Block surface `c, d` completely separate -- never one
/// combined `a, b, c, d, e, f` stream.
#[test]
fn cross_surface_separation_no_combined_ordering() {
    let source = "a+b;\n{ c-d; }\ne+f;";
    let accepted = recognize_and_accept_selected_script(source)
        .expect("`a+b; { c-d; } e+f;` must be Selected");

    let top_level_relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    let top_level_names: Vec<&str> = top_level_relations
        .iter()
        .map(|r| r.semantic_name.as_str())
        .collect();
    assert_eq!(top_level_names, ["a", "b", "e", "f"]);
    assert_eq!(top_level_relations[0].reference, Range(0, 1));
    assert_eq!(top_level_relations[1].reference, Range(2, 3));
    assert_eq!(top_level_relations[2].reference, Range(14, 15));
    assert_eq!(top_level_relations[3].reference, Range(16, 17));

    let block_relations = build_selected_block_additive_use_site_correspondence(&accepted);
    let block_names: Vec<&str> = block_relations
        .iter()
        .map(|r| r.semantic_name.as_str())
        .collect();
    assert_eq!(block_names, ["c", "d"]);
    let block_anchor = Range(5, 13);
    assert_eq!(block_relations[0].containing_block, block_anchor);
    assert_eq!(block_relations[0].reference, Range(7, 8));
    assert_eq!(block_relations[1].containing_block, block_anchor);
    assert_eq!(block_relations[1].reference, Range(9, 10));
}

/// Existing initializer relation surface remains completely separate
/// (acceptance criterion 13, W10/W18): `let x=a+b;\nc+d;` must recognize
/// end to end (independently restating the already-accepted #752/#753
/// initializer shape purely for grammar completeness) while publishing
/// only the free-standing TopLevel surface `c, d` -- the initializer's own
/// `a, b` never become part of this oracle's relation stream.
#[test]
fn initializer_relation_surface_remains_separate_from_free_standing_surface() {
    let source = "let x=a+b;\nc+d;";
    let accepted =
        recognize_and_accept_selected_script(source).expect("`let x=a+b; c+d;` must be Selected");
    let items = accepted.items();
    assert_eq!(items.len(), 2);
    let RecognizedTopLevelItem::LexicalBinding(x_binding) = &items[0] else {
        panic!("first item must be the `x` lexical binding");
    };
    assert_eq!(x_binding.semantic_name, "x");
    assert_eq!(x_binding.binding, Range(4, 5));

    let relations = build_selected_top_level_additive_use_site_correspondence(&accepted);
    assert_eq!(relations.len(), 2);
    let names: Vec<&str> = relations.iter().map(|r| r.semantic_name.as_str()).collect();
    assert_eq!(names, ["c", "d"]);
    assert_eq!(relations[0].reference, Range(11, 12));
    assert_eq!(relations[1].reference, Range(13, 14));
    assert_eq!(
        relations[0].correspondence,
        SelectedTopLevelSourceNameCorrespondence::NoSelectedSameSourceContributor
    );
    assert_eq!(
        relations[1].correspondence,
        SelectedTopLevelSourceNameCorrespondence::NoSelectedSameSourceContributor
    );
}

/// Cardinality firewall (acceptance criterion 14, W15/W16): third-or-more
/// additive operand chains never truncate to the first two; the whole
/// source remains `UnsupportedCoverage`.
#[test]
fn cardinality_firewall_rejects_three_or_more_operands() {
    for source in ["a;", "a+b+c;", "a-b-c;", "a+b-c;", "a-b+c;", "{ a+b+c; }"] {
        assert_eq!(
            classify_selected_script(source),
            SelectedScriptDisposition::UnsupportedCoverage,
            "{source:?}"
        );
    }
}

/// Operand firewall (acceptance criterion 15, W17/W18/W19): unary,
/// literal, grouping, member, and call neighbors never become an accepted
/// operand for this theorem.
#[test]
fn operand_firewall_rejects_unary_literal_grouping_member_call() {
    for source in [
        "+a+b;", "-a+b;", "a++b;", "a+-b;", "1+b;", "a+1;", "true+b;", "a+null;", "this+b;",
        "\"a\"+b;", "(a)+b;", "a+(b);", "a.b+c;", "a+b.c;", "a()+b;", "a+b();",
    ] {
        assert_eq!(
            classify_selected_script(source),
            SelectedScriptDisposition::UnsupportedCoverage,
            "{source:?}"
        );
    }
}

/// Richer-expression / precedence firewall (acceptance criterion 15
/// continued): a valid `a+b` prefix must never commit from a richer
/// unsupported syntax tail.
#[test]
fn richer_expression_firewall_rejects_precedence_and_assignment_neighbors() {
    for source in [
        "a*b;", "a+b*c;", "a*b+c;", "a**b+c;", "a+b**c;", "a=b+c;", "a+=b;", "a?b:c;", "a||b;",
        "a&&b;", "a??b;", "a,b;",
    ] {
        assert_eq!(
            classify_selected_script(source),
            SelectedScriptDisposition::UnsupportedCoverage,
            "{source:?}"
        );
    }
}

/// Escaped ReservedWord and malformed-escape boundary firewall (acceptance
/// criterion 16, W20/W21): neither operand may be an escaped ReservedWord
/// IdentifierName, and a malformed/invalid-position/non-CodePoint escape
/// never leaks an earlier valid-looking operand.
#[test]
fn escaped_reserved_word_and_malformed_boundary_firewall() {
    for source in [
        // Escaped ReservedWord operands (decoded "if").
        "\u{5c}u0069f+b;",
        "a+\u{5c}u0069f;",
        // Malformed escape: too few hex digits before the terminator.
        "\u{5c}u006+b;",
        "a+\u{5c}u006;",
        // Invalid start position: decoded digit cannot start an identifier.
        "\u{5c}u0031+b;",
        // Invalid part position: decoded space cannot continue an identifier.
        "a\u{5c}u0020+b;",
        // Non-CodePoint: braced value exceeds 0x10FFFF.
        "\u{5c}u{110000}+b;",
        // Surrogate half: not a valid identifier start code point.
        "\u{5c}uD800+b;",
    ] {
        assert_eq!(
            classify_selected_script(source),
            SelectedScriptDisposition::UnsupportedCoverage,
            "{source:?}"
        );
    }
}

/// Authored-semicolon boundary firewall (acceptance criterion 17, W22/W23):
/// this theorem selects only `AuthoredSemicolon`; EOF and close-brace ASI
/// remain `UnsupportedCoverage`, never composing #717's automatic-
/// semicolon-before-close-brace precedent.
#[test]
fn authored_semicolon_boundary_firewall_rejects_asi_forms() {
    for source in ["a+b", "{ a+b }"] {
        assert_eq!(
            classify_selected_script(source),
            SelectedScriptDisposition::UnsupportedCoverage,
            "{source:?}"
        );
    }
}

/// Recursive Block firewall (acceptance criterion 18, W24): a Block
/// containing another Block never recognizes; this oracle does not widen
/// region topology beyond the accepted one level.
#[test]
fn recursive_block_firewall_rejects_nested_block() {
    assert_eq!(
        classify_selected_script("{ { a+b; } }"),
        SelectedScriptDisposition::UnsupportedCoverage
    );
}

/// Static neutrality (acceptance criterion 19): the two free-standing
/// operands never contribute a `BoundNames`/`LexicallyDeclaredNames`/
/// `VarDeclaredNames` entry and are never collision-key or Early Error
/// subjects of their own -- reusing the same semantic name across multiple
/// free-standing additive statements, or within one statement, never
/// triggers static rejection on its own.
#[test]
fn static_neutrality_no_new_early_error_from_reused_use_site_names() {
    for source in ["a+a;", "a+b;\na+b;", "{ a+a; }", "{ a+b; }\n{ a+b; }"] {
        assert!(
            matches!(
                classify_selected_script(source),
                SelectedScriptDisposition::Selected(_)
            ),
            "{source:?}"
        );
    }
}

/// Static-rejection prerequisite (acceptance criterion 19, W-static
/// suppression): a source where the additive use-site is locally valid but
/// an existing declaration static rule rejects the whole source must
/// suppress all free-standing relation publication -- never downgraded to
/// `UnsupportedCoverage`.
#[test]
fn static_rejection_prerequisite_suppresses_all_relation_publication() {
    let source = "let a;\nlet a;\na+b;";
    match classify_selected_script(source) {
        SelectedScriptDisposition::StaticSemanticsRejected(
            StaticPreflightRejection::DuplicateLexical {
                semantic_name,
                first,
                duplicate,
                region,
            },
        ) => {
            assert_eq!(semantic_name, "a");
            assert_eq!(first, Range(4, 5));
            assert_eq!(duplicate, Range(11, 12));
            assert_eq!(region, SelectedRegion::TopLevel);
        }
        other => panic!("expected StaticSemanticsRejected, got {other:?}"),
    }

    assert!(recognize_and_accept_selected_script(source).is_none());
}

/// Whole-source transactionality, TopLevel bad tail (acceptance criteria
/// 19 adjacent, W21): a locally valid first additive statement never
/// leaks when a later top-level item fails to recognize.
#[test]
fn whole_source_transactionality_top_level_bad_tail() {
    assert_eq!(
        classify_selected_script("a+b;\n???"),
        SelectedScriptDisposition::UnsupportedCoverage
    );
}

/// Whole-source transactionality, TopLevel bad initializer.
#[test]
fn whole_source_transactionality_top_level_bad_initializer() {
    assert_eq!(
        classify_selected_script("a+b;\nlet x = ;"),
        SelectedScriptDisposition::UnsupportedCoverage
    );
}

/// Whole-source transactionality, Block bad tail: a locally valid Block-
/// contained additive statement never leaks when a later Block-contained
/// item fails to recognize.
#[test]
fn whole_source_transactionality_block_bad_tail() {
    assert_eq!(
        classify_selected_script("{ a+b; ??? }"),
        SelectedScriptDisposition::UnsupportedCoverage
    );
}

/// Whole-source transactionality, Block bad initializer.
#[test]
fn whole_source_transactionality_block_bad_initializer() {
    assert_eq!(
        classify_selected_script("{ a+b; let x = ; }"),
        SelectedScriptDisposition::UnsupportedCoverage
    );
}

/// Processing-failure classification distinctness (Issue #764 processing-
/// failure distinction): `ResourceLimited` and `InternalFailure` remain
/// textually distinct from `UnsupportedCoverage`, matching the minimal
/// symbolic model already accepted by every predecessor oracle. This
/// oracle constructs neither variant in practice (doing so would require
/// duplicating production's own resource/internal-failure machinery), but
/// the disposition type itself preserves the distinction.
#[test]
fn processing_failure_classifications_remain_symbolically_distinct() {
    for failure in PROCESSING_FAILURES {
        assert_ne!(*failure, "UnsupportedCoverage");
        assert!(FAILURE_CLASSIFICATIONS.contains(failure));
    }
    assert!(FAILURE_CLASSIFICATIONS.contains(&"ResourceLimited"));
    assert!(FAILURE_CLASSIFICATIONS.contains(&"InternalFailure"));
    assert!(FAILURE_CLASSIFICATIONS.contains(&"UnsupportedCoverage"));
}
