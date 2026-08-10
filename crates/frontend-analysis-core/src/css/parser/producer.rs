//! Bounded source-backed CSS declaration parser producer (#139, #158, #159).
//!
//! Implements the #137/#138-approved first parser capability:
//! `&SourceText + owned CssTokenizerRunResult + CssParserLimits ->
//! Result<CssParserRunResult, CssParserRunError>`.
//!
//! # Deterministic `AlgorithmSteps` charging rule
//!
//! Exactly one `AlgorithmSteps` unit is charged per producer-controlled
//! lexical-item visit (every comment skipped and every semantic token
//! observed by [`Producer::next_semantic`]), plus one additional unit for
//! checkpoint creation/begin. Checkpoint commit and rollback do not charge an
//! additional step. `next_semantic` is the single chokepoint through which
//! every scanning loop (stylesheet dispatch, at-rule body, qualified-rule
//! prelude, qualified-rule fallback replay, nested-remainder/discard-block
//! consumption, declaration value scan) observes lexical items, so no loop
//! iteration capable of continuing execution is ever uncharged, and no state
//! dispatch is charged twice merely because it is reached through a layered
//! helper. A rolled-back speculative declaration attempt's steps are never
//! refunded: replay after rollback charges again from `next_semantic` exactly
//! as a fresh scan would.
//!
//! # Cursor and checkpoint
//!
//! The parser walks the upstream tokenizer's existing `CssLexicalItem`
//! sequence through [`CssParserCursor`] (see `cursor.rs`); no second
//! semantic-token vector is built. At most one speculative checkpoint is
//! active at a time (`MAX_ACTIVE_SPECULATIVE_CHECKPOINT_DEPTH == 1`),
//! snapshotting the cursor position and every durable-evidence vector length
//! (including discard); rollback restores all of it but never the monotonic
//! `AlgorithmSteps` or achieved `PeakComponentDepth` counters already
//! charged. Beginning a second checkpoint, or committing/rolling back
//! without one active, is a typed `CssParserInvariantViolation` rather than
//! a panic.
//!
//! # Component scanning
//!
//! Structural nesting (functions, `()`, `[]`, `{}`) is tracked with an
//! explicit bounded `Vec<ComponentFrameKind>` scratch stack, never
//! recursive Rust calls and never a retained component-value AST.
//! `PeakComponentDepth` is preflighted before every push and updated
//! immediately on success; achieved peak never rolls back.
//!
//! # Custom-property-like top-level qualified-rule discard (#158)
//!
//! A top-level qualified-rule prelude whose first two non-whitespace values
//! are a decoded-`--`-prefixed `Ident` then a `Colon` is structurally
//! discarded per CSS Syntax rather than opening a supported declaration
//! context: its balanced block is consumed with the same bounded iterative
//! scanner and exactly one [`CssParserDiscardEvidence`] commits.
//!
//! # Malformed block item at true end of input (#159)
//!
//! A block item that fails both declaration recognition and qualified-rule
//! fallback and reaches genuine tokenizer end of input commits an
//! `InvalidBlockItem` diagnostic and a [`CssParserRecoveryEvidence`] with an
//! `EndOfInput` termination atomically, using the exact true source-end
//! boundary as the recovery terminal; no semicolon or right curly is
//! fabricated.
//!
//! # Nested group-rule dispatch (#168)
//!
//! [`Producer::handle_nested_at_rule`] replaces #167's whole-remainder
//! outcome as the normal nested-`AtKeyword` production path: it structurally
//! scans exactly one at-rule via [`Producer::scan_nested_at_rule_terminator`]
//! and either enters a supported `GroupRuleBlock` context (only for the
//! finite `@media`/`@supports`/`@container`/`@layer`/`@scope` registry, and
//! only when the bounded per-kind prelude subset qualifies) or commits
//! exactly one context-aware `NestedAtRule` unsupported direct item, then
//! resumes the same parent. `NestedContentRemainder` remains historical/
//! contract vocabulary only; it is no longer produced by this path.

use super::super::declaration::{
    CssDeclarationOccurrence, CssDeclarationPlacement, CssDeclarationPriorityEvidence,
    CssDeclarationRunOrdinal, CssDeclarationTermination, CssDescriptorOccurrence,
    CssDescriptorPlacement,
};
use super::super::token::CssTokenKind;
use super::super::tokenizer::result::CssTokenizerRunResult;
use super::context::{
    CssParserContextId, CssParserContextKind, CssParserContextRecord, CssParserContextTermination,
    CssParserDescriptorRuleKind, CssParserDirectItemOrdinal, CssParserGroupRuleKind,
};
use super::cursor::{CssParserCursor, CssParserRawPosition};
use super::diagnostic::{CssParserDiagnostic, CssParserDiagnosticCode};
use super::evidence::{
    CssParserDiscardEvidence, CssParserDiscardKind, CssParserRecoveryEvidence,
    CssParserRecoveryKind, CssParserRecoveryTermination, CssParserUnsupportedRegion,
};
use super::resource::{
    CssParserLimits, CssParserResourceKind, CssParserResourceLimitEvidence, CssParserResourceUsage,
};
use super::result::{
    CssParserCoverage, CssParserExecutionCompletion, CssParserInvariantViolation,
    CssParserRunError, CssParserRunResult, CssParserTermination, checked_resource_add,
};
use crate::{SourceAnchor, SourceText};

/// Parses `source_text` under `limits` against the already-validated upstream
/// tokenizer evidence, producing the first bounded declaration-analysis
/// capability approved by #137/#138.
pub(crate) fn run(
    source_text: &SourceText,
    upstream_tokenizer_result: CssTokenizerRunResult,
    limits: CssParserLimits,
) -> Result<CssParserRunResult, CssParserRunError> {
    super::result::validate_upstream_boundary(source_text, &upstream_tokenizer_result)?;

    let mut producer = Producer::new(source_text, upstream_tokenizer_result, limits);
    match producer.execute() {
        Ok(()) => producer.finish_complete(),
        Err(Flow::ResourceLimit(signal)) => producer.finish_resource_limited(signal),
        Err(Flow::UpstreamIncomplete) => producer.finish_upstream_incomplete(),
        Err(Flow::Invariant(error)) => Err(error),
    }
}

struct ResourceLimitSignal {
    kind: CssParserResourceKind,
    limit: usize,
    attempted: usize,
}

/// Internal producer control flow: a resource limit (recoverable into an
/// `Incomplete` result), upstream tokenizer incompleteness (recoverable into
/// an `Incomplete` result at the upstream terminal), or a typed internal
/// invariant failure.
enum Flow {
    ResourceLimit(ResourceLimitSignal),
    UpstreamIncomplete,
    Invariant(CssParserRunError),
}

impl From<CssParserRunError> for Flow {
    fn from(error: CssParserRunError) -> Self {
        Self::Invariant(error)
    }
}

/// An owned observation of one lexical-item visit, decoupled from any
/// borrowed reference into the upstream tokenizer result so producer state
/// can be freely mutated afterward without fighting the borrow checker.
enum ParserPosition {
    SemanticToken {
        anchor: SourceAnchor,
        kind: ObservedKind,
    },
    TrueEndOfInput,
    UpstreamTokenizerTerminal,
}

/// The subset of `CssTokenKind` producer control flow actually branches on.
#[derive(Clone)]
enum ObservedKind {
    Ident(String),
    /// The tokenizer-decoded at-keyword value, without the leading `@`
    /// (#168 parser observation only; not a tokenizer semantic change).
    AtKeyword(String),
    Colon,
    Semicolon,
    Whitespace,
    Delim(char),
    Function,
    LeftParenthesis,
    RightParenthesis,
    LeftSquareBracket,
    RightSquareBracket,
    LeftCurlyBracket,
    RightCurlyBracket,
    Cdo,
    Cdc,
    Other,
}

fn observe_kind(kind: &CssTokenKind) -> ObservedKind {
    match kind {
        CssTokenKind::Ident(value) => ObservedKind::Ident(value.clone()),
        CssTokenKind::AtKeyword(value) => ObservedKind::AtKeyword(value.clone()),
        CssTokenKind::Colon => ObservedKind::Colon,
        CssTokenKind::Semicolon => ObservedKind::Semicolon,
        CssTokenKind::Whitespace => ObservedKind::Whitespace,
        CssTokenKind::Delim(value) => ObservedKind::Delim(*value),
        CssTokenKind::Function(_) => ObservedKind::Function,
        CssTokenKind::LeftParenthesis => ObservedKind::LeftParenthesis,
        CssTokenKind::RightParenthesis => ObservedKind::RightParenthesis,
        CssTokenKind::LeftSquareBracket => ObservedKind::LeftSquareBracket,
        CssTokenKind::RightSquareBracket => ObservedKind::RightSquareBracket,
        CssTokenKind::LeftCurlyBracket => ObservedKind::LeftCurlyBracket,
        CssTokenKind::RightCurlyBracket => ObservedKind::RightCurlyBracket,
        CssTokenKind::Cdo => ObservedKind::Cdo,
        CssTokenKind::Cdc => ObservedKind::Cdc,
        _ => ObservedKind::Other,
    }
}

fn is_opener(kind: &ObservedKind) -> bool {
    matches!(
        kind,
        ObservedKind::Function
            | ObservedKind::LeftParenthesis
            | ObservedKind::LeftSquareBracket
            | ObservedKind::LeftCurlyBracket
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ComponentFrameKind {
    Function,
    Parenthesis,
    SquareBracket,
    CurlyBracket,
}

fn opener_frame_kind(kind: &ObservedKind) -> Result<ComponentFrameKind, Flow> {
    match kind {
        ObservedKind::Function => Ok(ComponentFrameKind::Function),
        ObservedKind::LeftParenthesis => Ok(ComponentFrameKind::Parenthesis),
        ObservedKind::LeftSquareBracket => Ok(ComponentFrameKind::SquareBracket),
        ObservedKind::LeftCurlyBracket => Ok(ComponentFrameKind::CurlyBracket),
        _ => Err(invariant_flow(
            CssParserInvariantViolation::ExpectedComponentOpener,
        )),
    }
}

fn invariant_flow(violation: CssParserInvariantViolation) -> Flow {
    Flow::Invariant(CssParserRunError::InternalInvariantFailure(violation))
}

enum BalanceEvent {
    Opened { depth_before: usize },
    ClosedMatching { depth_after: usize },
    Ordinary,
}

/// The raw source-backed parts of a recognized declaration, still lacking
/// its [`CssDeclarationPlacement`]: placement is only known once
/// [`Producer::commit_declaration`] has determined the owning active
/// context and its current item/run counters (#167).
struct DeclarationParts {
    complete: SourceAnchor,
    property_name: SourceAnchor,
    colon: SourceAnchor,
    value: SourceAnchor,
    priority: Option<CssDeclarationPriorityEvidence>,
    termination: CssDeclarationTermination,
}

/// One resolved outcome of the declaration-vs-qualified-rule transaction.
enum DeclarationOutcome {
    Recognized(DeclarationParts),
    NotRecognized,
}

enum AtRuleOutcome {
    Ended { end: usize },
}

/// The identity a newly entered context receives (#168): either an ordinary
/// nested qualified-rule block, or a supported group-rule block carrying its
/// exact authored at-keyword evidence and the tokenizer-decoded value needed
/// (ephemerally) to prove kind correspondence when the record is finalized.
///
/// The shared `RuleBlock` suffix mirrors [`CssParserContextKind`]'s own
/// deliberate naming; `clippy::enum_variant_names` is suppressed for the
/// same reason.
#[allow(clippy::enum_variant_names)]
enum NewContextIdentity {
    QualifiedRuleBlock,
    GroupRuleBlock {
        kind: CssParserGroupRuleKind,
        at_keyword: SourceAnchor,
        decoded_at_keyword: String,
    },
    DescriptorRuleBlock {
        kind: CssParserDescriptorRuleKind,
        at_keyword: SourceAnchor,
        decoded_at_keyword: String,
        property_name: Option<SourceAnchor>,
        decoded_property_name: Option<String>,
    },
}

/// Bounded transient evidence for the #158 custom-property-like top-level
/// qualified-rule prelude exclusion: the leading `Ident` (with its
/// already-decoded value) and the `Colon` immediately following it among the
/// prelude's top-level non-whitespace values.
struct CustomPropertyLikePrelude {
    property_name: SourceAnchor,
    decoded_property_name: String,
    colon: SourceAnchor,
}

enum PreludeOutcome {
    FoundBlockOpener {
        custom_property_like: Option<CustomPropertyLikePrelude>,
    },
    TrueEndOfInputWithoutBlock,
}

enum FallbackOutcome {
    NestedRuleTrigger,
    MalformedEndedAtSemicolon { semicolon: SourceAnchor },
    MalformedEndedAtEnclosingBlock { right_curly: SourceAnchor },
    MalformedEndedAtTrueEof,
}

enum RemainderOutcome {
    EnclosingRightCurly { start: usize, end: usize },
    TrueEndOfInput,
}

/// How a nested at-rule's own scan ends (#168).
enum NestedAtRuleTerminator {
    /// A top-level `{` was reached (cursor already advanced past it).
    /// `qualifies` is `true` only when a candidate registry kind was known
    /// and its bounded prelude subset was satisfied.
    Block {
        block_opener: SourceAnchor,
        qualifies: bool,
    },
    /// A top-level authored `;` was reached (cursor already advanced past
    /// it): the `@layer foo;` statement form, or any other registry/unknown
    /// at-rule's semicolon form.
    Semicolon {
        semicolon: SourceAnchor,
    },
    /// The enclosing context's own `}` was reached before this at-rule ever
    /// got a block or semicolon of its own (not consumed; the caller treats
    /// the at-rule as ending exactly here).
    EnclosingBlockEnd {
        right_curly: SourceAnchor,
    },
    TrueEndOfInputWithoutBlock,
}

/// Bounded, streaming per-kind #168 prelude qualification. Never retains a
/// decoded prelude string or a component-value AST: each variant tracks only
/// the small constant state needed to prove its own bounded grammar subset,
/// fed one semantic token at a time by
/// [`Producer::scan_nested_at_rule_terminator`].
enum PreludeQualifier {
    Media {
        top_level_count: usize,
        ok: bool,
    },
    Container {
        top_level_count: usize,
        ok: bool,
    },
    Layer {
        expect_ident: bool,
        ok: bool,
        segments: usize,
        any_seen: bool,
    },
    Scope {
        position: u8,
        ok: bool,
    },
    Supports {
        top_level_count: usize,
        top_level_ok: bool,
        inner_position: u8,
        inner_ok: bool,
    },
}

fn is_css_wide_reserved(name: &str) -> bool {
    name.eq_ignore_ascii_case("initial")
        || name.eq_ignore_ascii_case("inherit")
        || name.eq_ignore_ascii_case("unset")
        || name.eq_ignore_ascii_case("revert")
        || name.eq_ignore_ascii_case("revert-layer")
}

fn is_container_reserved(name: &str) -> bool {
    is_css_wide_reserved(name)
        || name.eq_ignore_ascii_case("default")
        || name.eq_ignore_ascii_case("none")
        || name.eq_ignore_ascii_case("and")
        || name.eq_ignore_ascii_case("not")
        || name.eq_ignore_ascii_case("or")
}

impl PreludeQualifier {
    fn new_for(kind: CssParserGroupRuleKind) -> Self {
        match kind {
            CssParserGroupRuleKind::Media => Self::Media {
                top_level_count: 0,
                ok: true,
            },
            CssParserGroupRuleKind::Container => Self::Container {
                top_level_count: 0,
                ok: true,
            },
            CssParserGroupRuleKind::Layer => Self::Layer {
                expect_ident: true,
                ok: true,
                segments: 0,
                any_seen: false,
            },
            CssParserGroupRuleKind::Scope => Self::Scope {
                position: 0,
                ok: true,
            },
            CssParserGroupRuleKind::Supports => Self::Supports {
                top_level_count: 0,
                top_level_ok: true,
                inner_position: 0,
                inner_ok: false,
            },
        }
    }

    /// Feeds one non-whitespace semantic token, observed at `depth_before`
    /// (the component-nesting depth immediately before this token is
    /// balanced): `0` is a token directly in the prelude, `1` is a token
    /// immediately inside the prelude's first (and, for a qualifying
    /// candidate, only) nested component group.
    fn observe(&mut self, depth_before: usize, kind: &ObservedKind) {
        match self {
            Self::Media {
                top_level_count,
                ok,
            } => {
                if depth_before != 0 {
                    return;
                }
                *top_level_count += 1;
                if *top_level_count != 1 {
                    *ok = false;
                    return;
                }
                *ok = matches!(
                    kind,
                    ObservedKind::Ident(name)
                        if name.eq_ignore_ascii_case("all")
                            || name.eq_ignore_ascii_case("screen")
                            || name.eq_ignore_ascii_case("print")
                );
            }
            Self::Container {
                top_level_count,
                ok,
            } => {
                if depth_before != 0 {
                    return;
                }
                *top_level_count += 1;
                if *top_level_count != 1 {
                    *ok = false;
                    return;
                }
                *ok = matches!(kind, ObservedKind::Ident(name) if !is_container_reserved(name));
            }
            Self::Layer {
                expect_ident,
                ok,
                segments,
                any_seen,
            } => {
                if depth_before != 0 {
                    *ok = false;
                    return;
                }
                *any_seen = true;
                if *expect_ident {
                    if let ObservedKind::Ident(name) = kind {
                        *ok = *ok && !is_css_wide_reserved(name);
                        *segments += 1;
                        *expect_ident = false;
                    } else {
                        *ok = false;
                    }
                } else if matches!(kind, ObservedKind::Delim('.')) {
                    *expect_ident = true;
                } else {
                    *ok = false;
                }
            }
            Self::Scope { position, ok } => match (*position, depth_before, kind) {
                (0, 0, ObservedKind::LeftParenthesis) => *position = 1,
                (1, 1, ObservedKind::Delim('&')) => *position = 2,
                (2, 1, ObservedKind::RightParenthesis) => *position = 3,
                _ => *ok = false,
            },
            Self::Supports {
                top_level_count,
                top_level_ok,
                inner_position,
                inner_ok,
            } => {
                if depth_before == 0 {
                    *top_level_count += 1;
                    if *top_level_count != 1 || !matches!(kind, ObservedKind::LeftParenthesis) {
                        *top_level_ok = false;
                    }
                    return;
                }
                if depth_before != 1 || *top_level_count != 1 {
                    return;
                }
                match *inner_position {
                    0 => {
                        if matches!(kind, ObservedKind::Ident(_)) {
                            *inner_position = 1;
                        } else {
                            *inner_position = 2;
                        }
                    }
                    1 => {
                        if matches!(kind, ObservedKind::Colon) {
                            *inner_ok = true;
                        }
                        *inner_position = 2;
                    }
                    _ => {}
                }
            }
        }
    }

    /// Resolves the final qualification verdict once the top-level block
    /// opener has been reached. `any_top_level_seen` distinguishes a
    /// genuinely empty prelude (permitted for `@media`/`@layer`/`@scope`,
    /// never for `@container`/`@supports`) from a prelude whose single
    /// top-level item failed its own shape check.
    fn finish(&self, any_top_level_seen: bool) -> bool {
        match self {
            Self::Media {
                top_level_count,
                ok,
            } => !any_top_level_seen || (*top_level_count == 1 && *ok),
            Self::Container {
                top_level_count,
                ok,
            } => *top_level_count == 1 && *ok,
            Self::Layer {
                expect_ident,
                ok,
                segments,
                any_seen,
            } => !any_seen || (*ok && !*expect_ident && *segments >= 1),
            Self::Scope { position, ok } => *position == 0 || (*ok && *position == 3),
            Self::Supports {
                top_level_count,
                top_level_ok,
                inner_ok,
                ..
            } => *top_level_count == 1 && *top_level_ok && *inner_ok,
        }
    }
}

/// The bounded per-kind #169 root-level descriptor parent-qualification
/// outcome, resolved once the top-level candidate's own block opener has
/// been reached. `FontFace` qualifies only for a semantically empty prelude
/// (no non-whitespace top-level token observed). `Property` qualifies only
/// for exactly one top-level `Ident` whose tokenizer-decoded value satisfies
/// the bounded `<custom-property-name>` subset (`starts_with("--")` and not
/// exactly `"--"`); any further top-level token -- including the comma
/// starting a multi-name `@property --a, --b` prelude -- disqualifies.
enum DescriptorQualification {
    Qualified {
        /// The authored anchor and tokenizer-decoded value of the single
        /// qualifying `@property` custom-property-name, or `None` for
        /// `@font-face`.
        property_name: Option<(SourceAnchor, String)>,
    },
    Unqualified,
}

/// Bounded, streaming per-kind #169 root-level descriptor prelude
/// qualification, fed one non-whitespace top-level semantic token at a time
/// by [`Producer::scan_descriptor_candidate_terminator`]. Never retains a
/// decoded prelude string beyond the single candidate `@property` name, and
/// never a component-value AST.
enum DescriptorPreludeQualifier {
    FontFace {
        any_seen: bool,
    },
    Property {
        first: Option<(SourceAnchor, String)>,
        extra_seen: bool,
    },
}

impl DescriptorPreludeQualifier {
    fn new_for(kind: CssParserDescriptorRuleKind) -> Self {
        match kind {
            CssParserDescriptorRuleKind::FontFace => Self::FontFace { any_seen: false },
            CssParserDescriptorRuleKind::Property => Self::Property {
                first: None,
                extra_seen: false,
            },
        }
    }

    /// Feeds one non-whitespace top-level (`depth_before == 0`) semantic
    /// token. Tokens nested inside a component group never disqualify on
    /// their own: the opener token that introduced that nesting already
    /// counted as this prelude's (dis)qualifying top-level item.
    fn observe(&mut self, depth_before: usize, anchor: &SourceAnchor, kind: &ObservedKind) {
        if depth_before != 0 {
            return;
        }
        match self {
            Self::FontFace { any_seen } => *any_seen = true,
            Self::Property { first, extra_seen } => {
                if first.is_some() {
                    *extra_seen = true;
                } else if let ObservedKind::Ident(name) = kind {
                    *first = Some((anchor.clone(), name.clone()));
                } else {
                    *extra_seen = true;
                }
            }
        }
    }

    fn finish(&self) -> DescriptorQualification {
        match self {
            Self::FontFace { any_seen } => {
                if *any_seen {
                    DescriptorQualification::Unqualified
                } else {
                    DescriptorQualification::Qualified {
                        property_name: None,
                    }
                }
            }
            Self::Property { first, extra_seen } => {
                if *extra_seen {
                    return DescriptorQualification::Unqualified;
                }
                match first {
                    Some((anchor, decoded)) if decoded.starts_with("--") && decoded != "--" => {
                        DescriptorQualification::Qualified {
                            property_name: Some((anchor.clone(), decoded.clone())),
                        }
                    }
                    _ => DescriptorQualification::Unqualified,
                }
            }
        }
    }
}

/// How a stylesheet-root #169 descriptor-candidate at-rule's own scan ends.
/// Deliberately narrower than [`NestedAtRuleTerminator`]: a root-level
/// at-rule has no enclosing block, so there is no `EnclosingBlockEnd`
/// variant.
enum DescriptorCandidateTerminator {
    Block {
        block_opener: SourceAnchor,
        qualification: DescriptorQualification,
    },
    Semicolon {
        semicolon: SourceAnchor,
    },
    TrueEndOfInputWithoutBlock,
}

/// One entry in the bounded iterative active-context stack (#167): parser
/// execution metadata required to continue a structurally recognized
/// qualified-rule block's direct content and to resume its parent once it
/// closes. Distinct from the component-value balancing stack
/// (`component_frames`): this stack tracks authored parser contexts, never
/// function/`()`/`[]`/`{}` balancing.
///
/// Retains no decoded selector/prelude text: `header`/`block_opener` are
/// exact source-backed anchors, and `body_start` plus the eventual
/// termination boundary derive `body` without ever rescanning source.
struct ActiveContextFrame {
    id: CssParserContextId,
    parent: Option<CssParserContextId>,
    item_ordinal: CssParserDirectItemOrdinal,
    /// This frame's own context kind (#168): either `QualifiedRuleBlock` or
    /// a supported `GroupRuleBlock`. Drives both the eventual record
    /// constructor choice and this frame's contribution to a new child's
    /// `nearest_qualified_ancestor`.
    kind: CssParserContextKind,
    /// The exact authored at-keyword anchor, present only for a
    /// `GroupRuleBlock` or `DescriptorRuleBlock` frame.
    at_keyword: Option<SourceAnchor>,
    /// The tokenizer-decoded at-keyword value (without `@`), retained only
    /// long enough to prove kind correspondence at record-construction time
    /// (#168/#169); never part of the finalized [`CssParserContextRecord`].
    decoded_at_keyword: Option<String>,
    /// The exact authored custom-property-name anchor, present only for a
    /// `DescriptorRuleBlock(Property)` frame (#169).
    descriptor_property_name: Option<SourceAnchor>,
    /// The tokenizer-decoded custom-property-name value, retained only long
    /// enough to prove the bounded `<custom-property-name>` subset at
    /// record-construction time (#169); never part of the finalized
    /// [`CssParserContextRecord`].
    descriptor_decoded_property_name: Option<String>,
    /// This frame's own nearest qualified-rule ancestor (#141/#168): `None`
    /// at the implicit stylesheet root's direct children, `Some(parent_id)`
    /// when the direct parent is a `QualifiedRuleBlock`, and inherited from
    /// the parent frame when the direct parent is a `GroupRuleBlock`.
    nearest_qualified_ancestor: Option<CssParserContextId>,
    header: SourceAnchor,
    block_opener: SourceAnchor,
    body_start: usize,
    /// The next direct-item ordinal this context will assign to a
    /// materialized declaration, child context, or nested unsupported
    /// at-rule (#168).
    next_item_ordinal: usize,
    /// The next declaration-run ordinal this context will open.
    next_run_ordinal: usize,
    /// Whether a declaration run is currently open (the most recently
    /// materialized direct item, if any, was a declaration rather than a
    /// child context or nested unsupported at-rule).
    run_open: bool,
    /// The run ordinal of the currently open run, meaningful only while
    /// `run_open` is `true`.
    current_run_ordinal: usize,
}

#[derive(Clone)]
struct ComponentSummary {
    anchor: SourceAnchor,
    simple: SimpleKind,
}

#[derive(Clone)]
enum SimpleKind {
    Delim(char),
    Ident(String),
    CurlyBlock,
    Other,
}

enum ValueTerminator {
    Semicolon(SourceAnchor),
    EnclosingRightCurly(SourceAnchor),
    TrueEndOfInput,
}

/// Bounded O(1) scratch summarizing top-level declaration-value components,
/// per the #137/#139 approved constant-space evidence derivation: only the
/// first component, a rolling window of the last three, and two counters are
/// retained. No `Vec<ComponentValue>` is built.
struct ValueScan {
    first: Option<ComponentSummary>,
    window: [Option<ComponentSummary>; 3],
    non_ws_count: usize,
    curly_block_count: usize,
    terminator: ValueTerminator,
}

fn push_component(
    scan_first: &mut Option<ComponentSummary>,
    window: &mut [Option<ComponentSummary>; 3],
    non_ws_count: &mut usize,
    component: ComponentSummary,
) {
    if scan_first.is_none() {
        *scan_first = Some(component.clone());
    }
    window[0] = window[1].take();
    window[1] = window[2].take();
    window[2] = Some(component);
    *non_ws_count += 1;
}

/// One active speculative checkpoint's transactional snapshot.
///
/// Includes the innermost active context's mutable placement counters
/// (#167): declaration speculation never allocates a context, but
/// `commit_declaration` mutates those counters before the checkpoint
/// commits, so a rollback after a later failure must restore them exactly
/// as it restores every other durable-evidence vector length. A checkpoint
/// is only ever active while at least one context is active (declarations
/// only occur inside a block), so the innermost frame is always present at
/// `begin_checkpoint` time.
struct Checkpoint {
    cursor: CssParserCursor,
    occurrences_len: usize,
    descriptor_occurrences_len: usize,
    diagnostics_len: usize,
    recovery_len: usize,
    unsupported_len: usize,
    discard_len: usize,
    component_frames_len: usize,
    frame_next_item_ordinal: usize,
    frame_next_run_ordinal: usize,
    frame_run_open: bool,
    frame_current_run_ordinal: usize,
}

struct Producer<'a> {
    source_text: &'a SourceText,
    upstream: CssTokenizerRunResult,
    limits: CssParserLimits,
    cursor: CssParserCursor,
    committed_pos: usize,
    occurrences: Vec<CssDeclarationOccurrence>,
    /// Retained #169 descriptor occurrences, counted against the shared
    /// `DeclarationOccurrences` aggregate cap alongside `occurrences` but
    /// never merged into that vector: the two occurrence meanings remain
    /// distinct end to end.
    descriptor_occurrences: Vec<CssDescriptorOccurrence>,
    diagnostics: Vec<CssParserDiagnostic>,
    recovery: Vec<CssParserRecoveryEvidence>,
    unsupported: Vec<CssParserUnsupportedRegion>,
    discard: Vec<CssParserDiscardEvidence>,
    algorithm_steps: usize,
    peak_component_depth: usize,
    component_frames: Vec<ComponentFrameKind>,
    checkpoint: Option<Checkpoint>,
    /// Iterative active-context stack (#167): the last entry is the
    /// innermost currently-open qualified-rule context. Empty at the
    /// implicit stylesheet root. Distinct from `component_frames`.
    active_contexts: Vec<ActiveContextFrame>,
    /// Parent-first reserved context slots, indexed by `CssParserContextId`.
    /// A slot is reserved (pushed as `None`) at context entry, before the
    /// active frame is pushed, and finalized (`Some`) when that context
    /// closes -- by an authored `}` or by run-stop cleanup -- regardless of
    /// how that finalization order relates to reservation order.
    pending_context_records: Vec<Option<CssParserContextRecord>>,
    /// Achieved maximum simultaneous active-context depth. The implicit
    /// stylesheet root is depth 0 and uncounted; never rolls back.
    peak_context_depth: usize,
    /// The next direct-item ordinal to assign at the implicit stylesheet
    /// root's own ordinal scope (distinct from any real context's
    /// `next_item_ordinal`).
    root_next_item_ordinal: usize,
}

impl<'a> Producer<'a> {
    fn new(
        source_text: &'a SourceText,
        upstream: CssTokenizerRunResult,
        limits: CssParserLimits,
    ) -> Self {
        Self {
            source_text,
            upstream,
            limits,
            cursor: CssParserCursor::new(),
            committed_pos: 0,
            occurrences: Vec::new(),
            descriptor_occurrences: Vec::new(),
            diagnostics: Vec::new(),
            recovery: Vec::new(),
            unsupported: Vec::new(),
            discard: Vec::new(),
            algorithm_steps: 0,
            peak_component_depth: 0,
            component_frames: Vec::new(),
            checkpoint: None,
            active_contexts: Vec::new(),
            pending_context_records: Vec::new(),
            peak_context_depth: 0,
            root_next_item_ordinal: 0,
        }
    }

    // -- resource accounting -------------------------------------------

    fn check_limit(&self, kind: CssParserResourceKind, prospective: usize) -> Result<(), Flow> {
        let limit = self.limits.limit(kind);
        if prospective > limit {
            return Err(Flow::ResourceLimit(ResourceLimitSignal {
                kind,
                limit,
                attempted: prospective,
            }));
        }
        Ok(())
    }

    fn charge_step(&mut self) -> Result<(), Flow> {
        let prospective = checked_resource_add(
            CssParserResourceKind::AlgorithmSteps,
            self.algorithm_steps,
            1,
        )?;
        self.check_limit(CssParserResourceKind::AlgorithmSteps, prospective)?;
        self.algorithm_steps = prospective;
        Ok(())
    }

    fn push_frame(&mut self, kind: ComponentFrameKind) -> Result<(), Flow> {
        let prospective = checked_resource_add(
            CssParserResourceKind::PeakComponentDepth,
            self.component_frames.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::PeakComponentDepth, prospective)?;
        self.component_frames.push(kind);
        if prospective > self.peak_component_depth {
            self.peak_component_depth = prospective;
        }
        Ok(())
    }

    fn pop_frame_if_matching(&mut self, closer: &ObservedKind) -> bool {
        let matches_top = matches!(
            (self.component_frames.last(), closer),
            (
                Some(ComponentFrameKind::Function | ComponentFrameKind::Parenthesis),
                ObservedKind::RightParenthesis
            ) | (
                Some(ComponentFrameKind::SquareBracket),
                ObservedKind::RightSquareBracket
            ) | (
                Some(ComponentFrameKind::CurlyBracket),
                ObservedKind::RightCurlyBracket
            )
        );
        if matches_top {
            self.component_frames.pop();
        }
        matches_top
    }

    // -- cursor / lexical visiting ---------------------------------------

    /// The single chokepoint through which every scanning loop observes the
    /// next lexical item: skips grammar-invisible comments (charging one
    /// step per skipped comment) and returns the next semantic token, true
    /// end of input, or the upstream tokenizer's non-EndOfInput terminal.
    fn next_semantic(&mut self) -> Result<ParserPosition, Flow> {
        loop {
            self.charge_step()?;
            match self.cursor.peek_raw(&self.upstream) {
                CssParserRawPosition::Comment => self.cursor.advance(),
                CssParserRawPosition::SemanticToken { token, .. } => {
                    return Ok(ParserPosition::SemanticToken {
                        anchor: token.source().clone(),
                        kind: observe_kind(token.kind()),
                    });
                }
                CssParserRawPosition::TrueEndOfInput => return Ok(ParserPosition::TrueEndOfInput),
                CssParserRawPosition::UpstreamTokenizerTerminal => {
                    return Ok(ParserPosition::UpstreamTokenizerTerminal);
                }
            }
        }
    }

    /// Consumes the token in `kind`/`anchor` most recently returned by
    /// `next_semantic`, updating the bounded component-frame stack.
    fn consume_and_balance(&mut self, kind: &ObservedKind) -> Result<BalanceEvent, Flow> {
        if is_opener(kind) {
            let depth_before = self.component_frames.len();
            self.push_frame(opener_frame_kind(kind)?)?;
            self.cursor.advance();
            return Ok(BalanceEvent::Opened { depth_before });
        }
        if matches!(
            kind,
            ObservedKind::RightParenthesis
                | ObservedKind::RightSquareBracket
                | ObservedKind::RightCurlyBracket
        ) {
            self.cursor.advance();
            return Ok(if self.pop_frame_if_matching(kind) {
                BalanceEvent::ClosedMatching {
                    depth_after: self.component_frames.len(),
                }
            } else {
                BalanceEvent::Ordinary
            });
        }
        self.cursor.advance();
        Ok(BalanceEvent::Ordinary)
    }

    // -- checkpoint --------------------------------------------------------

    fn begin_checkpoint(&mut self) -> Result<(), Flow> {
        self.charge_step()?;
        if self.checkpoint.is_some() {
            return Err(invariant_flow(
                CssParserInvariantViolation::CheckpointAlreadyActive,
            ));
        }
        let frame = self.active_contexts.last().ok_or_else(|| {
            invariant_flow(CssParserInvariantViolation::DeclarationOutsideActiveContext)
        })?;
        self.checkpoint = Some(Checkpoint {
            cursor: self.cursor.checkpoint(),
            occurrences_len: self.occurrences.len(),
            descriptor_occurrences_len: self.descriptor_occurrences.len(),
            diagnostics_len: self.diagnostics.len(),
            recovery_len: self.recovery.len(),
            unsupported_len: self.unsupported.len(),
            discard_len: self.discard.len(),
            component_frames_len: self.component_frames.len(),
            frame_next_item_ordinal: frame.next_item_ordinal,
            frame_next_run_ordinal: frame.next_run_ordinal,
            frame_run_open: frame.run_open,
            frame_current_run_ordinal: frame.current_run_ordinal,
        });
        Ok(())
    }

    fn commit_checkpoint(&mut self) -> Result<(), Flow> {
        if self.checkpoint.take().is_none() {
            return Err(invariant_flow(
                CssParserInvariantViolation::CheckpointCommitWithoutActive,
            ));
        }
        Ok(())
    }

    fn rollback_checkpoint(&mut self) -> Result<(), Flow> {
        let checkpoint = self.checkpoint.take().ok_or_else(|| {
            invariant_flow(CssParserInvariantViolation::CheckpointRollbackWithoutActive)
        })?;
        self.cursor.restore(checkpoint.cursor);
        self.occurrences.truncate(checkpoint.occurrences_len);
        self.descriptor_occurrences
            .truncate(checkpoint.descriptor_occurrences_len);
        self.diagnostics.truncate(checkpoint.diagnostics_len);
        self.recovery.truncate(checkpoint.recovery_len);
        self.unsupported.truncate(checkpoint.unsupported_len);
        self.discard.truncate(checkpoint.discard_len);
        self.component_frames
            .truncate(checkpoint.component_frames_len);
        if let Some(frame) = self.active_contexts.last_mut() {
            frame.next_item_ordinal = checkpoint.frame_next_item_ordinal;
            frame.next_run_ordinal = checkpoint.frame_next_run_ordinal;
            frame.run_open = checkpoint.frame_run_open;
            frame.current_run_ordinal = checkpoint.frame_current_run_ordinal;
        }
        Ok(())
    }

    /// Rolls back the active checkpoint before propagating a `Flow` that
    /// aborted a declaration attempt (#139 transactional stop rollback):
    /// `ResourceLimit`, `UpstreamIncomplete`, and `Invariant` must never
    /// leave a checkpoint active when they reach `run()`'s finalization.
    /// The original `flow` is preserved unless rollback itself surfaces a
    /// typed invariant failure, in which case that takes precedence.
    fn rollback_checkpoint_preserving_flow(&mut self, flow: Flow) -> Flow {
        match self.rollback_checkpoint() {
            Ok(()) => flow,
            Err(rollback_flow) => rollback_flow,
        }
    }

    // -- top-level stylesheet flow / block-content flow ---------------------

    /// Single iterative dispatch loop covering both the implicit stylesheet
    /// root (`active_contexts` empty) and every active qualified-rule
    /// context's direct block content (#167). Never recurses in Rust:
    /// entering a child context only pushes an [`ActiveContextFrame`] and
    /// lets this same loop continue servicing the new innermost scope; the
    /// root/block dispatch branch is re-decided every iteration from
    /// `active_contexts`'s current state.
    fn execute(&mut self) -> Result<(), Flow> {
        loop {
            if self.active_contexts.is_empty() {
                match self.next_semantic()? {
                    ParserPosition::SemanticToken { anchor, kind } => match kind {
                        ObservedKind::Whitespace | ObservedKind::Cdo | ObservedKind::Cdc => {
                            let end = anchor.range().end();
                            self.cursor.advance();
                            self.committed_pos = end;
                        }
                        ObservedKind::AtKeyword(decoded) => {
                            self.cursor.advance();
                            self.handle_top_level_at_rule(anchor, decoded)?;
                        }
                        _ => {
                            self.consume_top_level_qualified_rule(anchor.range().start())?;
                        }
                    },
                    ParserPosition::TrueEndOfInput => return Ok(()),
                    ParserPosition::UpstreamTokenizerTerminal => {
                        return Err(Flow::UpstreamIncomplete);
                    }
                }
            } else {
                match self.next_semantic()? {
                    ParserPosition::SemanticToken { anchor, kind } => match kind {
                        ObservedKind::Whitespace | ObservedKind::Semicolon => {
                            let end = anchor.range().end();
                            self.cursor.advance();
                            self.committed_pos = end;
                        }
                        ObservedKind::RightCurlyBracket => {
                            let end = anchor.range().end();
                            self.cursor.advance();
                            self.committed_pos = end;
                            self.close_innermost_context_authored(anchor)?;
                        }
                        ObservedKind::AtKeyword(decoded) => {
                            self.cursor.advance();
                            self.handle_nested_at_rule(anchor, decoded)?;
                        }
                        _ => {
                            let item_start = anchor.range().start();
                            if self.innermost_is_descriptor_context() {
                                self.handle_descriptor_block_item(item_start)?;
                            } else {
                                self.handle_block_item(item_start)?;
                            }
                        }
                    },
                    ParserPosition::TrueEndOfInput => return Ok(()),
                    ParserPosition::UpstreamTokenizerTerminal => {
                        return Err(Flow::UpstreamIncomplete);
                    }
                }
            }
        }
    }

    // -- context stack -------------------------------------------------------

    /// Preflights both context resources, in the fixed
    /// `PeakContextDepth`-then-`ContextRecords` precedence order, without
    /// mutating any context state. Returns the prospective achieved depth on
    /// success. Only when both preflights pass may the caller reserve a
    /// context ID, retain an active frame, or mutate the context stack
    /// (#167).
    fn preflight_context_entry(&mut self) -> Result<usize, Flow> {
        let prospective_depth = checked_resource_add(
            CssParserResourceKind::PeakContextDepth,
            self.active_contexts.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::PeakContextDepth, prospective_depth)?;
        let prospective_records = checked_resource_add(
            CssParserResourceKind::ContextRecords,
            self.pending_context_records.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::ContextRecords, prospective_records)?;
        Ok(prospective_depth)
    }

    /// Commits a structurally recognized qualified-rule block or supported
    /// group-rule block as a new active context: `block_opener` must already
    /// be consumed (cursor advanced, `committed_pos` updated) by the caller.
    /// Resource preflight happens first and gates every subsequent
    /// mutation: a refused entry allocates no ID, consumes no parent/root
    /// item ordinal, and leaves the context stack untouched (#167/#168
    /// commit-honest refusal). A `GroupRuleBlock` identity is only ever
    /// accepted with a real active parent: group contexts are never
    /// top-level in this Leaf (#168).
    fn enter_context(
        &mut self,
        header_start: usize,
        block_opener: SourceAnchor,
        identity: NewContextIdentity,
    ) -> Result<(), Flow> {
        let prospective_depth = self.preflight_context_entry()?;

        let (parent, item_ordinal, nearest_qualified_ancestor) =
            match self.active_contexts.last_mut() {
                Some(frame) => {
                    if matches!(identity, NewContextIdentity::DescriptorRuleBlock { .. }) {
                        return Err(invariant_flow(
                            CssParserInvariantViolation::DescriptorContextCannotBeNested,
                        ));
                    }
                    frame.run_open = false;
                    let ordinal = frame.next_item_ordinal;
                    frame.next_item_ordinal += 1;
                    let nearest = match frame.kind {
                        CssParserContextKind::QualifiedRuleBlock => Some(frame.id),
                        CssParserContextKind::GroupRuleBlock(_) => frame.nearest_qualified_ancestor,
                        CssParserContextKind::DescriptorRuleBlock(_) => {
                            return Err(invariant_flow(
                                CssParserInvariantViolation::DescriptorContextCannotHaveChildren,
                            ));
                        }
                    };
                    (Some(frame.id), ordinal, nearest)
                }
                None => match identity {
                    NewContextIdentity::QualifiedRuleBlock
                    | NewContextIdentity::DescriptorRuleBlock { .. } => {
                        let ordinal = self.root_next_item_ordinal;
                        self.root_next_item_ordinal += 1;
                        (None, ordinal, None)
                    }
                    NewContextIdentity::GroupRuleBlock { .. } => {
                        return Err(invariant_flow(
                            CssParserInvariantViolation::GroupContextCannotBeTopLevel,
                        ));
                    }
                },
            };

        let id = CssParserContextId::new(self.pending_context_records.len());
        self.pending_context_records.push(None);
        if prospective_depth > self.peak_context_depth {
            self.peak_context_depth = prospective_depth;
        }

        let header = self
            .source_text
            .anchor(header_start, block_opener.range().start())
            .map_err(|error| Flow::Invariant(error.into()))?;
        let body_start = block_opener.range().end();

        let (
            kind,
            at_keyword,
            decoded_at_keyword,
            descriptor_property_name,
            descriptor_decoded_property_name,
        ) = match identity {
            NewContextIdentity::QualifiedRuleBlock => (
                CssParserContextKind::QualifiedRuleBlock,
                None,
                None,
                None,
                None,
            ),
            NewContextIdentity::GroupRuleBlock {
                kind,
                at_keyword,
                decoded_at_keyword,
            } => (
                CssParserContextKind::GroupRuleBlock(kind),
                Some(at_keyword),
                Some(decoded_at_keyword),
                None,
                None,
            ),
            NewContextIdentity::DescriptorRuleBlock {
                kind,
                at_keyword,
                decoded_at_keyword,
                property_name,
                decoded_property_name,
            } => (
                CssParserContextKind::DescriptorRuleBlock(kind),
                Some(at_keyword),
                Some(decoded_at_keyword),
                property_name,
                decoded_property_name,
            ),
        };

        self.active_contexts.push(ActiveContextFrame {
            id,
            parent,
            item_ordinal: CssParserDirectItemOrdinal::new(item_ordinal),
            kind,
            at_keyword,
            decoded_at_keyword,
            descriptor_property_name,
            descriptor_decoded_property_name,
            nearest_qualified_ancestor,
            header,
            block_opener,
            body_start,
            next_item_ordinal: 0,
            next_run_ordinal: 0,
            run_open: false,
            current_run_ordinal: 0,
        });
        Ok(())
    }

    /// Builds the finalized [`CssParserContextRecord`] for a popped frame,
    /// dispatching to the constructor matching its retained kind (#168).
    fn build_context_record(
        &self,
        frame: ActiveContextFrame,
        body: SourceAnchor,
        termination: CssParserContextTermination,
    ) -> Result<CssParserContextRecord, CssParserRunError> {
        match frame.kind {
            CssParserContextKind::QualifiedRuleBlock => {
                CssParserContextRecord::new_qualified_rule_block(
                    self.source_text,
                    frame.id,
                    frame.parent,
                    frame.item_ordinal,
                    frame.nearest_qualified_ancestor,
                    frame.header,
                    frame.block_opener,
                    body,
                    termination,
                )
                .map_err(Into::into)
            }
            CssParserContextKind::GroupRuleBlock(group_kind) => {
                let at_keyword =
                    frame
                        .at_keyword
                        .ok_or(CssParserRunError::InternalInvariantFailure(
                            CssParserInvariantViolation::MissingGroupContextEvidence,
                        ))?;
                let decoded =
                    frame
                        .decoded_at_keyword
                        .ok_or(CssParserRunError::InternalInvariantFailure(
                            CssParserInvariantViolation::MissingGroupContextEvidence,
                        ))?;
                CssParserContextRecord::new_group_rule_block(
                    self.source_text,
                    frame.id,
                    frame.parent,
                    frame.item_ordinal,
                    frame.nearest_qualified_ancestor,
                    group_kind,
                    at_keyword,
                    &decoded,
                    frame.header,
                    frame.block_opener,
                    body,
                    termination,
                )
                .map_err(Into::into)
            }
            CssParserContextKind::DescriptorRuleBlock(descriptor_kind) => {
                if frame.parent.is_some() {
                    return Err(CssParserRunError::InternalInvariantFailure(
                        CssParserInvariantViolation::DescriptorContextCannotBeNested,
                    ));
                }
                let at_keyword =
                    frame
                        .at_keyword
                        .ok_or(CssParserRunError::InternalInvariantFailure(
                            CssParserInvariantViolation::MissingDescriptorContextEvidence,
                        ))?;
                let decoded =
                    frame
                        .decoded_at_keyword
                        .ok_or(CssParserRunError::InternalInvariantFailure(
                            CssParserInvariantViolation::MissingDescriptorContextEvidence,
                        ))?;
                CssParserContextRecord::new_descriptor_rule_block(
                    self.source_text,
                    frame.id,
                    frame.item_ordinal,
                    descriptor_kind,
                    at_keyword,
                    &decoded,
                    frame.descriptor_property_name,
                    frame.descriptor_decoded_property_name.as_deref(),
                    frame.header,
                    frame.block_opener,
                    body,
                    termination,
                )
                .map_err(Into::into)
            }
        }
    }

    /// Closes the innermost active context at an authored `}`: pops the
    /// active frame, finalizes its retained record with
    /// `AuthoredRightCurly` termination, and stores it into its reserved
    /// slot. The caller has already consumed `right_curly` and updated
    /// `committed_pos`.
    fn close_innermost_context_authored(&mut self, right_curly: SourceAnchor) -> Result<(), Flow> {
        let frame = self
            .active_contexts
            .pop()
            .ok_or_else(|| invariant_flow(CssParserInvariantViolation::NoActiveContextToClose))?;
        let body = self
            .source_text
            .anchor(frame.body_start, right_curly.range().start())
            .map_err(|error| Flow::Invariant(error.into()))?;
        let id_index = frame.id.index();
        let record = self
            .build_context_record(
                frame,
                body,
                CssParserContextTermination::AuthoredRightCurly { right_curly },
            )
            .map_err(Flow::Invariant)?;
        self.pending_context_records[id_index] = Some(record);
        Ok(())
    }

    /// Finalizes every still-active context, innermost first, at the exact
    /// same partial terminal, using `make_termination` to build the honest
    /// per-context termination evidence. Called only once execution has
    /// already stopped (true EOF, upstream-incomplete, or parser-resource
    /// refusal); never fabricates a closing `}`.
    fn finalize_active_contexts(
        &mut self,
        terminal: &SourceAnchor,
        make_termination: impl Fn(SourceAnchor) -> CssParserContextTermination,
    ) -> Result<(), CssParserRunError> {
        while let Some(frame) = self.active_contexts.pop() {
            let body = self
                .source_text
                .anchor(frame.body_start, terminal.range().start())?;
            let termination = make_termination(terminal.clone());
            let id_index = frame.id.index();
            let record = self.build_context_record(frame, body, termination)?;
            self.pending_context_records[id_index] = Some(record);
        }
        Ok(())
    }

    /// Qualification-aware root at-rule dispatch (#169): observes the
    /// tokenizer-decoded at-keyword and distinguishes only a candidate
    /// `font-face`, a candidate `property`, or everything else. Everything
    /// else takes the unchanged #138 whole-at-rule unsupported path.
    /// A candidate's own header is scanned structurally and boundedly; if it
    /// satisfies the approved bounded parent qualification and has a block,
    /// one descriptor context is entered through the existing context-entry
    /// resource gate. Otherwise the candidate is consumed through the same
    /// unsupported path as any other at-rule -- this never overclaims
    /// invalidity where #169 simply lacks capability (a non-qualifying
    /// `@font-face`/`@property` remains explicit unsupported evidence, never
    /// an `Invalid*` diagnostic).
    fn handle_top_level_at_rule(
        &mut self,
        at_keyword: SourceAnchor,
        decoded_at_keyword: String,
    ) -> Result<(), Flow> {
        let Some(candidate_kind) =
            CssParserDescriptorRuleKind::from_decoded_at_keyword(&decoded_at_keyword)
        else {
            return self.consume_top_level_at_rule(at_keyword);
        };

        match self.scan_descriptor_candidate_terminator(candidate_kind)? {
            DescriptorCandidateTerminator::Block {
                block_opener,
                qualification: DescriptorQualification::Qualified { property_name },
            } => {
                self.committed_pos = block_opener.range().end();
                let (property_name_anchor, decoded_property_name) = match property_name {
                    Some((anchor, decoded)) => (Some(anchor), Some(decoded)),
                    None => (None, None),
                };
                self.enter_context(
                    at_keyword.range().start(),
                    block_opener,
                    NewContextIdentity::DescriptorRuleBlock {
                        kind: candidate_kind,
                        at_keyword,
                        decoded_at_keyword,
                        property_name: property_name_anchor,
                        decoded_property_name,
                    },
                )
            }
            DescriptorCandidateTerminator::Block {
                qualification: DescriptorQualification::Unqualified,
                ..
            } => match self.consume_remainder_until_enclosing_right_curly()? {
                RemainderOutcome::EnclosingRightCurly { end, .. } => {
                    self.cursor.advance();
                    self.commit_top_level_at_rule_unsupported(at_keyword, end)
                }
                RemainderOutcome::TrueEndOfInput => {
                    let len = self.source_text.as_str().len();
                    self.commit_top_level_at_rule_unsupported(at_keyword, len)
                }
            },
            DescriptorCandidateTerminator::Semicolon { semicolon } => {
                self.commit_top_level_at_rule_unsupported(at_keyword, semicolon.range().end())
            }
            DescriptorCandidateTerminator::TrueEndOfInputWithoutBlock => {
                let len = self.source_text.as_str().len();
                self.commit_top_level_at_rule_unsupported(at_keyword, len)
            }
        }
    }

    /// Structurally scans exactly one root-level #169 descriptor-candidate
    /// at-rule's prelude, using the existing bounded component-balancing
    /// machinery, and reports how it ends: a top-level block opener (with
    /// the resolved [`DescriptorQualification`]), an authored top-level
    /// semicolon, or true end of input. Never parses a source substring:
    /// qualification is decided purely from the already observed
    /// lexical-item stream via [`DescriptorPreludeQualifier`].
    fn scan_descriptor_candidate_terminator(
        &mut self,
        candidate_kind: CssParserDescriptorRuleKind,
    ) -> Result<DescriptorCandidateTerminator, Flow> {
        self.component_frames.clear();
        let mut qualifier = DescriptorPreludeQualifier::new_for(candidate_kind);
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0 && matches!(kind, ObservedKind::Semicolon) {
                        self.cursor.advance();
                        return Ok(DescriptorCandidateTerminator::Semicolon { semicolon: anchor });
                    }
                    if depth_before == 0 && matches!(kind, ObservedKind::LeftCurlyBracket) {
                        let qualification = qualifier.finish();
                        self.cursor.advance();
                        return Ok(DescriptorCandidateTerminator::Block {
                            block_opener: anchor,
                            qualification,
                        });
                    }
                    if !matches!(kind, ObservedKind::Whitespace) {
                        qualifier.observe(depth_before, &anchor, &kind);
                    }
                    self.consume_and_balance(&kind)?;
                }
                ParserPosition::TrueEndOfInput => {
                    return Ok(DescriptorCandidateTerminator::TrueEndOfInputWithoutBlock);
                }
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    /// Commits a structurally recognized top-level at-rule as an unsupported
    /// region. `root_next_item_ordinal` only advances after that evidence is
    /// committed (#167 commit-honest refusal): a resource refusal or
    /// evidence-construction failure before the push leaves the root ordinal
    /// untouched, keeping retained top-level qualified-rule ordinals
    /// source-relative across intervening at-rules.
    fn consume_top_level_at_rule(&mut self, at_keyword: SourceAnchor) -> Result<(), Flow> {
        let AtRuleOutcome::Ended { end } = self.consume_top_level_at_rule_body()?;
        self.commit_top_level_at_rule_unsupported(at_keyword, end)
    }

    /// Shared top-level-at-rule-unsupported commit tail, used both by the
    /// unchanged #138 whole-at-rule path ([`Self::consume_top_level_at_rule`])
    /// and by #169's qualification-aware dispatch for a disqualified
    /// `font-face`/`property` candidate.
    fn commit_top_level_at_rule_unsupported(
        &mut self,
        at_keyword: SourceAnchor,
        end: usize,
    ) -> Result<(), Flow> {
        let start = at_keyword.range().start();
        let prospective = checked_resource_add(
            CssParserResourceKind::UnsupportedRegions,
            self.unsupported.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::UnsupportedRegions, prospective)?;
        let complete = self
            .source_text
            .anchor(start, end)
            .map_err(|error| Flow::Invariant(error.into()))?;
        let region = CssParserUnsupportedRegion::new_top_level_at_rule(
            self.source_text,
            complete,
            at_keyword,
        )
        .map_err(|error| Flow::Invariant(error.into()))?;
        self.unsupported.push(region);
        self.root_next_item_ordinal += 1;
        self.committed_pos = end;
        Ok(())
    }

    fn consume_top_level_at_rule_body(&mut self) -> Result<AtRuleOutcome, Flow> {
        self.component_frames.clear();
        let mut terminal_block_seen = false;
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0
                        && !terminal_block_seen
                        && matches!(kind, ObservedKind::Semicolon)
                    {
                        let end = anchor.range().end();
                        self.cursor.advance();
                        return Ok(AtRuleOutcome::Ended { end });
                    }
                    let is_top_level_curly =
                        depth_before == 0 && matches!(kind, ObservedKind::LeftCurlyBracket);
                    let event = self.consume_and_balance(&kind)?;
                    if is_top_level_curly {
                        terminal_block_seen = true;
                    }
                    if terminal_block_seen
                        && matches!(event, BalanceEvent::ClosedMatching { depth_after: 0 })
                    {
                        return Ok(AtRuleOutcome::Ended {
                            end: anchor.range().end(),
                        });
                    }
                }
                ParserPosition::TrueEndOfInput => {
                    return Ok(AtRuleOutcome::Ended {
                        end: self.source_text.as_str().len(),
                    });
                }
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    fn consume_top_level_qualified_rule(&mut self, item_start: usize) -> Result<(), Flow> {
        match self.scan_qualified_rule_prelude()? {
            PreludeOutcome::FoundBlockOpener {
                custom_property_like,
            } => {
                let left_curly = self.expect_left_curly_bracket()?;
                self.committed_pos = left_curly.range().end();
                match custom_property_like {
                    Some(prelude) => {
                        self.consume_top_level_custom_property_like_discard(item_start, prelude)
                    }
                    None => self.enter_context(
                        item_start,
                        left_curly,
                        NewContextIdentity::QualifiedRuleBlock,
                    ),
                }
            }
            PreludeOutcome::TrueEndOfInputWithoutBlock => {
                let len = self.source_text.as_str().len();
                self.commit_invalid_qualified_rule_diagnostic(item_start, len)
            }
        }
    }

    /// Consumes the `LeftCurlyBracket` that [`Self::scan_qualified_rule_prelude`]
    /// guarantees is next when it returns `FoundBlockOpener`.
    fn expect_left_curly_bracket(&mut self) -> Result<SourceAnchor, Flow> {
        match self.next_semantic()? {
            ParserPosition::SemanticToken {
                anchor,
                kind: ObservedKind::LeftCurlyBracket,
            } => {
                self.cursor.advance();
                Ok(anchor)
            }
            _ => Err(invariant_flow(
                CssParserInvariantViolation::ExpectedQualifiedRuleBlockOpener,
            )),
        }
    }

    /// #158: structurally consumes a top-level qualified rule's balanced
    /// block whose prelude matched the custom-property-like exclusion,
    /// committing exactly one [`CssParserDiscardEvidence`] covering the full
    /// candidate (prelude through the authored block) instead of entering
    /// the supported declaration context.
    fn consume_top_level_custom_property_like_discard(
        &mut self,
        item_start: usize,
        prelude: CustomPropertyLikePrelude,
    ) -> Result<(), Flow> {
        let block_end = match self.consume_remainder_until_enclosing_right_curly()? {
            RemainderOutcome::EnclosingRightCurly { end, .. } => {
                self.cursor.advance();
                end
            }
            RemainderOutcome::TrueEndOfInput => self.source_text.as_str().len(),
        };

        let prospective =
            checked_resource_add(CssParserResourceKind::DiscardRecords, self.discard.len(), 1)?;
        self.check_limit(CssParserResourceKind::DiscardRecords, prospective)?;

        let region = self
            .source_text
            .anchor(item_start, block_end)
            .map_err(|error| Flow::Invariant(error.into()))?;
        let evidence = CssParserDiscardEvidence::new(
            self.source_text,
            region,
            prelude.property_name,
            prelude.colon,
            &prelude.decoded_property_name,
            CssParserDiscardKind::TopLevelCustomPropertyLikeQualifiedRule,
        )
        .map_err(|error| Flow::Invariant(error.into()))?;

        self.discard.push(evidence);
        self.committed_pos = block_end;
        Ok(())
    }

    fn commit_invalid_qualified_rule_diagnostic(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<(), Flow> {
        let prospective = checked_resource_add(
            CssParserResourceKind::ParserDiagnostics,
            self.diagnostics.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::ParserDiagnostics, prospective)?;
        let location = self
            .source_text
            .anchor(start, end)
            .map_err(|error| Flow::Invariant(error.into()))?;
        let diagnostic = CssParserDiagnostic::new(
            self.source_text,
            CssParserDiagnosticCode::InvalidStylesheetQualifiedRule,
            location,
        )
        .map_err(|error| Flow::Invariant(error.into()))?;
        self.diagnostics.push(diagnostic);
        self.committed_pos = end;
        Ok(())
    }

    /// Scans the qualified-rule prelude, tracking only the minimum O(1)
    /// summary needed for the #158 custom-property-like exclusion: the first
    /// two top-level non-whitespace prelude values, retained only while they
    /// could still be an `Ident` (decoded `--`-prefixed) followed by a
    /// `Colon`. Values inside nested component structures never contribute.
    fn scan_qualified_rule_prelude(&mut self) -> Result<PreludeOutcome, Flow> {
        self.component_frames.clear();
        let mut non_ws_seen = 0usize;
        let mut leading_ident: Option<(SourceAnchor, String)> = None;
        let mut leading_colon: Option<SourceAnchor> = None;
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0 && matches!(kind, ObservedKind::LeftCurlyBracket) {
                        let custom_property_like = match (&leading_ident, &leading_colon) {
                            (Some((name_anchor, decoded)), Some(colon_anchor))
                                if decoded.starts_with("--") =>
                            {
                                Some(CustomPropertyLikePrelude {
                                    property_name: name_anchor.clone(),
                                    decoded_property_name: decoded.clone(),
                                    colon: colon_anchor.clone(),
                                })
                            }
                            _ => None,
                        };
                        return Ok(PreludeOutcome::FoundBlockOpener {
                            custom_property_like,
                        });
                    }
                    if depth_before == 0 && !matches!(kind, ObservedKind::Whitespace) {
                        non_ws_seen += 1;
                        match non_ws_seen {
                            1 => {
                                if let ObservedKind::Ident(name) = &kind {
                                    leading_ident = Some((anchor.clone(), name.clone()));
                                }
                            }
                            2 if leading_ident.is_some() && matches!(kind, ObservedKind::Colon) => {
                                leading_colon = Some(anchor.clone());
                            }
                            _ => {}
                        }
                    }
                    self.consume_and_balance(&kind)?;
                }
                ParserPosition::TrueEndOfInput => {
                    return Ok(PreludeOutcome::TrueEndOfInputWithoutBlock);
                }
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    // -- supported block loop -----------------------------------------------

    /// Whether the innermost active context is a #169 `DescriptorRuleBlock`:
    /// its body uses declaration-list dispatch
    /// ([`Self::handle_descriptor_block_item`]), never the ordinary
    /// declaration-vs-qualified-rule block-content transaction.
    fn innermost_is_descriptor_context(&self) -> bool {
        matches!(
            self.active_contexts.last().map(|frame| frame.kind),
            Some(CssParserContextKind::DescriptorRuleBlock(_))
        )
    }

    /// Resolves one block-content item: the declaration-vs-qualified-rule
    /// transaction (#139), now extended so a structurally recognized nested
    /// qualified rule enters a real child context instead of falling back
    /// to `NestedContentRemainder` (#167). Never reserves a context or
    /// mutates the context stack while the declaration checkpoint is
    /// active: the checkpoint has already committed or rolled back before
    /// `enter_context` runs.
    fn handle_block_item(&mut self, item_start: usize) -> Result<(), Flow> {
        self.begin_checkpoint()?;
        let outcome = match self.try_declaration() {
            Ok(outcome) => outcome,
            Err(flow) => return Err(self.rollback_checkpoint_preserving_flow(flow)),
        };
        match outcome {
            DeclarationOutcome::Recognized(parts) => {
                if let Err(flow) = self.commit_declaration(parts) {
                    return Err(self.rollback_checkpoint_preserving_flow(flow));
                }
                self.commit_checkpoint()?;
                Ok(())
            }
            DeclarationOutcome::NotRecognized => {
                self.rollback_checkpoint()?;
                match self.scan_qualified_rule_fallback()? {
                    FallbackOutcome::NestedRuleTrigger => {
                        let left_curly = self.expect_left_curly_bracket()?;
                        self.committed_pos = left_curly.range().end();
                        self.enter_context(
                            item_start,
                            left_curly,
                            NewContextIdentity::QualifiedRuleBlock,
                        )
                    }
                    FallbackOutcome::MalformedEndedAtSemicolon { semicolon } => {
                        let region_end = semicolon.range().end();
                        self.commit_malformed_recovery(
                            item_start,
                            region_end,
                            CssParserRecoveryTermination::AuthoredSemicolon { semicolon },
                        )
                    }
                    FallbackOutcome::MalformedEndedAtEnclosingBlock { right_curly } => {
                        let region_end = right_curly.range().start();
                        self.commit_malformed_recovery(
                            item_start,
                            region_end,
                            CssParserRecoveryTermination::EnclosingBlockEnd { right_curly },
                        )
                    }
                    FallbackOutcome::MalformedEndedAtTrueEof => {
                        // #159: the trailing malformed bytes reach genuine
                        // tokenizer end of input with no authored semicolon
                        // or right curly. Commit diagnostic + recovery
                        // atomically with an explicit empty EOF terminal at
                        // true source end; no delimiter is fabricated.
                        let len = self.source_text.as_str().len();
                        let terminal = self
                            .source_text
                            .anchor(len, len)
                            .map_err(|error| Flow::Invariant(error.into()))?;
                        self.commit_malformed_recovery(
                            item_start,
                            len,
                            CssParserRecoveryTermination::EndOfInput { terminal },
                        )
                    }
                }
            }
        }
    }

    fn commit_malformed_recovery(
        &mut self,
        region_start: usize,
        region_end: usize,
        termination: CssParserRecoveryTermination,
    ) -> Result<(), Flow> {
        let diagnostics_prospective = checked_resource_add(
            CssParserResourceKind::ParserDiagnostics,
            self.diagnostics.len(),
            1,
        )?;
        self.check_limit(
            CssParserResourceKind::ParserDiagnostics,
            diagnostics_prospective,
        )?;
        let recovery_prospective = checked_resource_add(
            CssParserResourceKind::RecoveryRecords,
            self.recovery.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::RecoveryRecords, recovery_prospective)?;

        let region = self
            .source_text
            .anchor(region_start, region_end)
            .map_err(|error| Flow::Invariant(error.into()))?;
        let diagnostic = CssParserDiagnostic::new(
            self.source_text,
            CssParserDiagnosticCode::InvalidBlockItem,
            region.clone(),
        )
        .map_err(|error| Flow::Invariant(error.into()))?;
        let recovery = CssParserRecoveryEvidence::new(
            self.source_text,
            region,
            CssParserRecoveryKind::MalformedBlockItem,
            termination,
        )
        .map_err(|error| Flow::Invariant(error.into()))?;

        self.diagnostics.push(diagnostic);
        self.recovery.push(recovery);
        self.committed_pos = region_end;
        Ok(())
    }

    /// Scans a candidate nested qualified-rule prelude after declaration
    /// recognition has failed and rolled back (#139/#158/#167).
    ///
    /// Carries a defensive nested custom-property-lookalike guard mirroring
    /// [`Self::scan_qualified_rule_prelude`]'s top-level exclusion, per CSS
    /// Syntax `consume a qualified rule (nested=true)`'s own custom-property
    /// safety boundary. For #167's declaration-recognition rules an
    /// `is_custom` (`--`-prefixed) property is always recognized by
    /// [`Self::try_declaration`] regardless of value complexity, so this
    /// branch is not reachable by any accepted #167 input; it exists only as
    /// defense in depth and surfaces a typed internal invariant rather than
    /// silently promoting an unreachable custom-property-shaped candidate
    /// into a child context.
    fn scan_qualified_rule_fallback(&mut self) -> Result<FallbackOutcome, Flow> {
        self.component_frames.clear();
        let mut non_ws_seen = 0usize;
        let mut leading_ident_is_custom = false;
        let mut leading_colon_seen = false;
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0 {
                        match &kind {
                            ObservedKind::LeftCurlyBracket => {
                                if leading_ident_is_custom && leading_colon_seen {
                                    return Err(invariant_flow(
                                        CssParserInvariantViolation::UnreachableNestedCustomPropertyFallback,
                                    ));
                                }
                                return Ok(FallbackOutcome::NestedRuleTrigger);
                            }
                            ObservedKind::Semicolon => {
                                self.cursor.advance();
                                return Ok(FallbackOutcome::MalformedEndedAtSemicolon {
                                    semicolon: anchor,
                                });
                            }
                            ObservedKind::RightCurlyBracket => {
                                return Ok(FallbackOutcome::MalformedEndedAtEnclosingBlock {
                                    right_curly: anchor,
                                });
                            }
                            _ => {}
                        }
                        if !matches!(kind, ObservedKind::Whitespace) {
                            non_ws_seen += 1;
                            match non_ws_seen {
                                1 => {
                                    if let ObservedKind::Ident(name) = &kind {
                                        leading_ident_is_custom = name.starts_with("--");
                                    }
                                }
                                2 => {
                                    leading_colon_seen = leading_ident_is_custom
                                        && matches!(kind, ObservedKind::Colon);
                                }
                                _ => {}
                            }
                        }
                    }
                    self.consume_and_balance(&kind)?;
                }
                ParserPosition::TrueEndOfInput => {
                    return Ok(FallbackOutcome::MalformedEndedAtTrueEof);
                }
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    /// Resolves one #169 descriptor-context block-content item:
    /// declaration-list dispatch, not the ordinary
    /// declaration-vs-qualified-rule transaction. A recognized
    /// declaration-shaped item becomes a [`CssDescriptorOccurrence`], never a
    /// [`CssDeclarationOccurrence`]. Declaration-recognition failure never
    /// falls back to a child `QualifiedRuleBlock`: `<declaration-list>`
    /// admits no such child, so a qualified-rule-shaped fragment here is
    /// simply malformed descriptor-block input, recovered with the same
    /// `InvalidBlockItem`/`MalformedBlockItem` evidence as any other
    /// malformed item.
    fn handle_descriptor_block_item(&mut self, item_start: usize) -> Result<(), Flow> {
        self.begin_checkpoint()?;
        let outcome = match self.try_declaration() {
            Ok(outcome) => outcome,
            Err(flow) => return Err(self.rollback_checkpoint_preserving_flow(flow)),
        };
        match outcome {
            DeclarationOutcome::Recognized(parts) => {
                if let Err(flow) = self.commit_descriptor_occurrence(parts) {
                    return Err(self.rollback_checkpoint_preserving_flow(flow));
                }
                self.commit_checkpoint()?;
                Ok(())
            }
            DeclarationOutcome::NotRecognized => {
                self.rollback_checkpoint()?;
                match self.scan_descriptor_malformed_item()? {
                    FallbackOutcome::NestedRuleTrigger => Err(invariant_flow(
                        CssParserInvariantViolation::UnreachableDescriptorNestedRuleTrigger,
                    )),
                    FallbackOutcome::MalformedEndedAtSemicolon { semicolon } => {
                        let region_end = semicolon.range().end();
                        self.commit_malformed_recovery(
                            item_start,
                            region_end,
                            CssParserRecoveryTermination::AuthoredSemicolon { semicolon },
                        )
                    }
                    FallbackOutcome::MalformedEndedAtEnclosingBlock { right_curly } => {
                        let region_end = right_curly.range().start();
                        self.commit_malformed_recovery(
                            item_start,
                            region_end,
                            CssParserRecoveryTermination::EnclosingBlockEnd { right_curly },
                        )
                    }
                    FallbackOutcome::MalformedEndedAtTrueEof => {
                        let len = self.source_text.as_str().len();
                        let terminal = self
                            .source_text
                            .anchor(len, len)
                            .map_err(|error| Flow::Invariant(error.into()))?;
                        self.commit_malformed_recovery(
                            item_start,
                            len,
                            CssParserRecoveryTermination::EndOfInput { terminal },
                        )
                    }
                }
            }
        }
    }

    /// Scans a malformed #169 descriptor-block item after declaration
    /// recognition has failed and rolled back, reporting the same
    /// [`FallbackOutcome`] terminal shapes as
    /// [`Self::scan_qualified_rule_fallback`] except it never produces
    /// `NestedRuleTrigger`: a top-level `{` is not a nested-rule trigger
    /// here, since `<declaration-list>` admits no child rule. Instead it is
    /// balanced through like any other component (a qualified-rule-shaped
    /// fragment is malformed content, not a context boundary), and scanning
    /// continues for the item's real terminator (`;`, the enclosing `}`, or
    /// true end of input).
    fn scan_descriptor_malformed_item(&mut self) -> Result<FallbackOutcome, Flow> {
        self.component_frames.clear();
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0 {
                        match &kind {
                            ObservedKind::Semicolon => {
                                self.cursor.advance();
                                return Ok(FallbackOutcome::MalformedEndedAtSemicolon {
                                    semicolon: anchor,
                                });
                            }
                            ObservedKind::RightCurlyBracket => {
                                return Ok(FallbackOutcome::MalformedEndedAtEnclosingBlock {
                                    right_curly: anchor,
                                });
                            }
                            _ => {}
                        }
                    }
                    self.consume_and_balance(&kind)?;
                }
                ParserPosition::TrueEndOfInput => {
                    return Ok(FallbackOutcome::MalformedEndedAtTrueEof);
                }
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    /// Nested at-keyword dispatch (#168): structurally scans exactly one
    /// at-rule, then either enters a supported `GroupRuleBlock` context or
    /// commits exactly one context-aware `NestedAtRule` unsupported direct
    /// item, resuming the same parent afterward. Supersedes #167's
    /// whole-remainder outcome for ordinary nested-at-rule production; never
    /// consumes more than one at-rule regardless of qualification.
    fn handle_nested_at_rule(
        &mut self,
        at_keyword: SourceAnchor,
        decoded_at_keyword: String,
    ) -> Result<(), Flow> {
        // #169: `<declaration-list>` automatically excludes at-rules, so a
        // descriptor context's body never qualifies a nested at-rule into a
        // group-rule context -- not even a registry member like `@media` --
        // regardless of its own prelude shape. It always becomes one
        // explicit unsupported direct item.
        let candidate_kind = if self.innermost_is_descriptor_context() {
            None
        } else {
            CssParserGroupRuleKind::from_decoded_at_keyword(&decoded_at_keyword)
        };
        match self.scan_nested_at_rule_terminator(candidate_kind)? {
            NestedAtRuleTerminator::Block {
                block_opener,
                qualifies,
            } => {
                if let (Some(kind), true) = (candidate_kind, qualifies) {
                    self.committed_pos = block_opener.range().end();
                    self.enter_context(
                        at_keyword.range().start(),
                        block_opener,
                        NewContextIdentity::GroupRuleBlock {
                            kind,
                            at_keyword,
                            decoded_at_keyword,
                        },
                    )
                } else {
                    match self.consume_remainder_until_enclosing_right_curly()? {
                        RemainderOutcome::EnclosingRightCurly { end, .. } => {
                            self.cursor.advance();
                            self.commit_nested_at_rule_unsupported(at_keyword, end)
                        }
                        RemainderOutcome::TrueEndOfInput => {
                            let len = self.source_text.as_str().len();
                            self.commit_nested_at_rule_unsupported(at_keyword, len)
                        }
                    }
                }
            }
            NestedAtRuleTerminator::Semicolon { semicolon } => {
                let end = semicolon.range().end();
                self.commit_nested_at_rule_unsupported(at_keyword, end)
            }
            NestedAtRuleTerminator::EnclosingBlockEnd { right_curly } => {
                let end = right_curly.range().start();
                self.commit_nested_at_rule_unsupported(at_keyword, end)
            }
            NestedAtRuleTerminator::TrueEndOfInputWithoutBlock => {
                let len = self.source_text.as_str().len();
                self.commit_nested_at_rule_unsupported(at_keyword, len)
            }
        }
    }

    /// Structurally scans exactly one nested at-rule's prelude, using the
    /// existing bounded component-balancing machinery, and reports how it
    /// ends: a top-level block opener (with whether the observed prelude
    /// satisfies `candidate_kind`'s bounded #168 subset, always `false` when
    /// `candidate_kind` is `None`), an authored top-level semicolon, the
    /// enclosing context's own `}` reached before this at-rule ever got a
    /// block/semicolon of its own, or true end of input. Never parses a
    /// source substring: qualification is decided purely from the already
    /// observed lexical-item stream via `PreludeQualifier`.
    fn scan_nested_at_rule_terminator(
        &mut self,
        candidate_kind: Option<CssParserGroupRuleKind>,
    ) -> Result<NestedAtRuleTerminator, Flow> {
        self.component_frames.clear();
        let mut qualifier = candidate_kind.map(PreludeQualifier::new_for);
        let mut any_top_level_seen = false;
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0 && matches!(kind, ObservedKind::Semicolon) {
                        self.cursor.advance();
                        return Ok(NestedAtRuleTerminator::Semicolon { semicolon: anchor });
                    }
                    if depth_before == 0 && matches!(kind, ObservedKind::LeftCurlyBracket) {
                        let qualifies = qualifier
                            .as_ref()
                            .is_some_and(|qualifier| qualifier.finish(any_top_level_seen));
                        self.cursor.advance();
                        return Ok(NestedAtRuleTerminator::Block {
                            block_opener: anchor,
                            qualifies,
                        });
                    }
                    if depth_before == 0 && matches!(kind, ObservedKind::RightCurlyBracket) {
                        return Ok(NestedAtRuleTerminator::EnclosingBlockEnd {
                            right_curly: anchor,
                        });
                    }
                    if !matches!(kind, ObservedKind::Whitespace) {
                        if depth_before == 0 {
                            any_top_level_seen = true;
                        }
                        if let Some(qualifier) = qualifier.as_mut() {
                            qualifier.observe(depth_before, &kind);
                        }
                    }
                    self.consume_and_balance(&kind)?;
                }
                ParserPosition::TrueEndOfInput => {
                    return Ok(NestedAtRuleTerminator::TrueEndOfInputWithoutBlock);
                }
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    /// Commits one context-aware `NestedAtRule` unsupported direct item
    /// atomically with owning-context ordering mutation (#168 commit-honest
    /// order): the `UnsupportedRegions` preflight and evidence construction
    /// happen first; only after a successful push does the owning context's
    /// direct-item ordinal advance and its currently open declaration run
    /// (if any) close.
    fn commit_nested_at_rule_unsupported(
        &mut self,
        at_keyword: SourceAnchor,
        end: usize,
    ) -> Result<(), Flow> {
        let start = at_keyword.range().start();
        let prospective = checked_resource_add(
            CssParserResourceKind::UnsupportedRegions,
            self.unsupported.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::UnsupportedRegions, prospective)?;

        let frame = self.active_contexts.last().ok_or_else(|| {
            invariant_flow(CssParserInvariantViolation::DeclarationOutsideActiveContext)
        })?;
        let context_id = frame.id;
        let item_ordinal = frame.next_item_ordinal;

        let complete = self
            .source_text
            .anchor(start, end)
            .map_err(|error| Flow::Invariant(error.into()))?;
        let evidence = CssParserUnsupportedRegion::new_nested_at_rule(
            self.source_text,
            complete,
            at_keyword,
            context_id,
            CssParserDirectItemOrdinal::new(item_ordinal),
        )
        .map_err(|error| Flow::Invariant(error.into()))?;

        self.unsupported.push(evidence);

        let frame = self
            .active_contexts
            .last_mut()
            .ok_or_else(|| invariant_flow(CssParserInvariantViolation::NoActiveContextToClose))?;
        frame.next_item_ordinal += 1;
        frame.run_open = false;
        self.committed_pos = end;
        Ok(())
    }

    fn consume_remainder_until_enclosing_right_curly(&mut self) -> Result<RemainderOutcome, Flow> {
        self.component_frames.clear();
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0 && matches!(kind, ObservedKind::RightCurlyBracket) {
                        return Ok(RemainderOutcome::EnclosingRightCurly {
                            start: anchor.range().start(),
                            end: anchor.range().end(),
                        });
                    }
                    self.consume_and_balance(&kind)?;
                }
                ParserPosition::TrueEndOfInput => return Ok(RemainderOutcome::TrueEndOfInput),
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    // -- declaration transaction ---------------------------------------------

    fn try_declaration(&mut self) -> Result<DeclarationOutcome, Flow> {
        self.charge_step()?;

        let (name_anchor, decoded_name) = match self.next_semantic()? {
            ParserPosition::SemanticToken {
                anchor,
                kind: ObservedKind::Ident(name),
            } => {
                self.cursor.advance();
                (anchor, name)
            }
            ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            ParserPosition::SemanticToken { .. } | ParserPosition::TrueEndOfInput => {
                return Ok(DeclarationOutcome::NotRecognized);
            }
        };

        let colon_anchor = loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken {
                    kind: ObservedKind::Whitespace,
                    ..
                } => {
                    self.cursor.advance();
                }
                ParserPosition::SemanticToken {
                    anchor,
                    kind: ObservedKind::Colon,
                } => {
                    self.cursor.advance();
                    break anchor;
                }
                ParserPosition::SemanticToken { .. } | ParserPosition::TrueEndOfInput => {
                    return Ok(DeclarationOutcome::NotRecognized);
                }
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        };

        let scan = self.scan_declaration_value()?;

        let is_custom = decoded_name.starts_with("--");
        let ambiguous = scan.curly_block_count >= 1 && scan.non_ws_count > 1;
        if !is_custom && ambiguous {
            return Ok(DeclarationOutcome::NotRecognized);
        }

        let priority = self.derive_priority(&scan)?;
        let value_anchor = self.derive_value_anchor(&scan, &colon_anchor, priority.as_ref())?;
        let termination = self.derive_termination(&scan)?;
        let complete_end = termination_complete_end(
            &termination,
            &colon_anchor,
            &value_anchor,
            priority.as_ref(),
        );
        let complete_anchor = self
            .source_text
            .anchor(name_anchor.range().start(), complete_end)
            .map_err(|error| Flow::Invariant(error.into()))?;

        Ok(DeclarationOutcome::Recognized(DeclarationParts {
            complete: complete_anchor,
            property_name: name_anchor,
            colon: colon_anchor,
            value: value_anchor,
            priority,
            termination,
        }))
    }

    /// Commits a recognized declaration into the innermost active context,
    /// computing its [`CssDeclarationPlacement`] from that context's current
    /// item/run counters (#167). The `DeclarationOccurrences` preflight
    /// happens before any counter mutation, so a refusal here never
    /// advances the owning context's item or run ordinal. The preflight is
    /// against the #169 aggregate cap shared with descriptor occurrences
    /// (`self.occurrences.len() + self.descriptor_occurrences.len()`), never
    /// `self.occurrences.len()` alone.
    fn commit_declaration(&mut self, parts: DeclarationParts) -> Result<(), Flow> {
        let prospective = checked_resource_add(
            CssParserResourceKind::DeclarationOccurrences,
            self.occurrences.len() + self.descriptor_occurrences.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::DeclarationOccurrences, prospective)?;

        let frame = self.active_contexts.last_mut().ok_or_else(|| {
            invariant_flow(CssParserInvariantViolation::DeclarationOutsideActiveContext)
        })?;
        let item_ordinal = frame.next_item_ordinal;
        frame.next_item_ordinal += 1;
        if !frame.run_open {
            frame.current_run_ordinal = frame.next_run_ordinal;
            frame.next_run_ordinal += 1;
            frame.run_open = true;
        }
        let run_ordinal = frame.current_run_ordinal;
        let context_id = frame.id;

        let placement = CssDeclarationPlacement::new(
            context_id,
            CssParserDirectItemOrdinal::new(item_ordinal),
            CssDeclarationRunOrdinal::new(run_ordinal),
        );

        let occurrence = CssDeclarationOccurrence::new(
            self.source_text,
            parts.complete,
            parts.property_name,
            parts.colon,
            parts.value,
            parts.priority,
            parts.termination,
            placement,
        )
        .map_err(|error| Flow::Invariant(error.into()))?;

        let end = occurrence.complete().range().end();
        self.occurrences.push(occurrence);
        self.committed_pos = end;
        Ok(())
    }

    /// Commits a recognized #169 descriptor occurrence into the innermost
    /// active `DescriptorRuleBlock` context. Follows the same commit-honest
    /// order as [`Self::commit_nested_at_rule_unsupported`], not
    /// [`Self::commit_declaration`]'s: the `DeclarationOccurrences` aggregate
    /// preflight happens first, then the occurrence is constructed and
    /// pushed, and only then does the owning context's direct-item ordinal
    /// advance -- there is no declaration-run ordinal to open, since
    /// `<declaration-list>` admits no child rule whose interleaving requires
    /// that model.
    fn commit_descriptor_occurrence(&mut self, parts: DeclarationParts) -> Result<(), Flow> {
        let prospective = checked_resource_add(
            CssParserResourceKind::DeclarationOccurrences,
            self.occurrences.len() + self.descriptor_occurrences.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::DeclarationOccurrences, prospective)?;

        let frame = self.active_contexts.last().ok_or_else(|| {
            invariant_flow(CssParserInvariantViolation::DeclarationOutsideActiveContext)
        })?;
        let context_id = frame.id;
        let item_ordinal = frame.next_item_ordinal;

        let placement =
            CssDescriptorPlacement::new(context_id, CssParserDirectItemOrdinal::new(item_ordinal));

        let occurrence = CssDescriptorOccurrence::new(
            self.source_text,
            parts.complete,
            parts.property_name,
            parts.colon,
            parts.value,
            parts.priority,
            parts.termination,
            placement,
        )
        .map_err(|error| Flow::Invariant(error.into()))?;

        let end = occurrence.complete().range().end();
        self.descriptor_occurrences.push(occurrence);

        let frame = self
            .active_contexts
            .last_mut()
            .ok_or_else(|| invariant_flow(CssParserInvariantViolation::NoActiveContextToClose))?;
        frame.next_item_ordinal += 1;
        self.committed_pos = end;
        Ok(())
    }

    fn scan_declaration_value(&mut self) -> Result<ValueScan, Flow> {
        self.component_frames.clear();
        let mut first: Option<ComponentSummary> = None;
        let mut window: [Option<ComponentSummary>; 3] = [None, None, None];
        let mut non_ws_count = 0usize;
        let mut curly_block_count = 0usize;
        let mut pending_open: Option<(SourceAnchor, ComponentFrameKind)> = None;

        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0 {
                        match kind {
                            ObservedKind::Semicolon => {
                                self.cursor.advance();
                                return Ok(ValueScan {
                                    first,
                                    window,
                                    non_ws_count,
                                    curly_block_count,
                                    terminator: ValueTerminator::Semicolon(anchor),
                                });
                            }
                            ObservedKind::RightCurlyBracket => {
                                return Ok(ValueScan {
                                    first,
                                    window,
                                    non_ws_count,
                                    curly_block_count,
                                    terminator: ValueTerminator::EnclosingRightCurly(anchor),
                                });
                            }
                            ObservedKind::Whitespace => {
                                self.cursor.advance();
                                continue;
                            }
                            _ => {}
                        }
                    }

                    if depth_before == 0 && is_opener(&kind) {
                        pending_open = Some((anchor.clone(), opener_frame_kind(&kind)?));
                    }
                    let event = self.consume_and_balance(&kind)?;
                    match event {
                        BalanceEvent::ClosedMatching { depth_after: 0 } => {
                            if let Some((open_anchor, open_kind)) = pending_open.take() {
                                let full = self
                                    .source_text
                                    .anchor(open_anchor.range().start(), anchor.range().end())
                                    .map_err(|error| Flow::Invariant(error.into()))?;
                                let simple =
                                    if matches!(open_kind, ComponentFrameKind::CurlyBracket) {
                                        curly_block_count += 1;
                                        SimpleKind::CurlyBlock
                                    } else {
                                        SimpleKind::Other
                                    };
                                push_component(
                                    &mut first,
                                    &mut window,
                                    &mut non_ws_count,
                                    ComponentSummary {
                                        anchor: full,
                                        simple,
                                    },
                                );
                            }
                        }
                        BalanceEvent::Ordinary if depth_before == 0 => {
                            let simple = match &kind {
                                ObservedKind::Delim(value) => SimpleKind::Delim(*value),
                                ObservedKind::Ident(name) => SimpleKind::Ident(name.clone()),
                                _ => SimpleKind::Other,
                            };
                            push_component(
                                &mut first,
                                &mut window,
                                &mut non_ws_count,
                                ComponentSummary { anchor, simple },
                            );
                        }
                        _ => {}
                    }
                }
                ParserPosition::TrueEndOfInput => {
                    if let Some((open_anchor, open_kind)) = pending_open.take() {
                        let end = self.source_text.as_str().len();
                        let full = self
                            .source_text
                            .anchor(open_anchor.range().start(), end)
                            .map_err(|error| Flow::Invariant(error.into()))?;
                        let simple = if matches!(open_kind, ComponentFrameKind::CurlyBracket) {
                            curly_block_count += 1;
                            SimpleKind::CurlyBlock
                        } else {
                            SimpleKind::Other
                        };
                        push_component(
                            &mut first,
                            &mut window,
                            &mut non_ws_count,
                            ComponentSummary {
                                anchor: full,
                                simple,
                            },
                        );
                    }
                    return Ok(ValueScan {
                        first,
                        window,
                        non_ws_count,
                        curly_block_count,
                        terminator: ValueTerminator::TrueEndOfInput,
                    });
                }
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    fn derive_priority(
        &self,
        scan: &ValueScan,
    ) -> Result<Option<CssDeclarationPriorityEvidence>, Flow> {
        if scan.non_ws_count < 2 {
            return Ok(None);
        }
        let last = scan.window[2].as_ref().ok_or_else(|| {
            invariant_flow(CssParserInvariantViolation::InconsistentValueScanSummary)
        })?;
        let second_last = scan.window[1].as_ref().ok_or_else(|| {
            invariant_flow(CssParserInvariantViolation::InconsistentValueScanSummary)
        })?;
        if !matches!(second_last.simple, SimpleKind::Delim('!')) {
            return Ok(None);
        }
        let decoded = match &last.simple {
            SimpleKind::Ident(name) if name.eq_ignore_ascii_case("important") => name.clone(),
            _ => return Ok(None),
        };
        let complete = self
            .source_text
            .anchor(
                second_last.anchor.range().start(),
                last.anchor.range().end(),
            )
            .map_err(|error| Flow::Invariant(error.into()))?;
        let evidence = CssDeclarationPriorityEvidence::new(
            self.source_text,
            complete,
            second_last.anchor.clone(),
            last.anchor.clone(),
            &decoded,
        )
        .map_err(|error| Flow::Invariant(error.into()))?;
        Ok(Some(evidence))
    }

    fn derive_value_anchor(
        &self,
        scan: &ValueScan,
        colon: &SourceAnchor,
        priority: Option<&CssDeclarationPriorityEvidence>,
    ) -> Result<SourceAnchor, Flow> {
        let remaining = if priority.is_some() {
            scan.non_ws_count - 2
        } else {
            scan.non_ws_count
        };
        if remaining == 0 {
            let point = colon.range().end();
            return self
                .source_text
                .anchor(point, point)
                .map_err(|error| Flow::Invariant(error.into()));
        }
        let first = scan.first.as_ref().ok_or_else(|| {
            invariant_flow(CssParserInvariantViolation::InconsistentValueScanSummary)
        })?;
        let end = if priority.is_some() {
            scan.window[0]
                .as_ref()
                .ok_or_else(|| {
                    invariant_flow(CssParserInvariantViolation::InconsistentValueScanSummary)
                })?
                .anchor
                .range()
                .end()
        } else {
            scan.window[2]
                .as_ref()
                .ok_or_else(|| {
                    invariant_flow(CssParserInvariantViolation::InconsistentValueScanSummary)
                })?
                .anchor
                .range()
                .end()
        };
        self.source_text
            .anchor(first.anchor.range().start(), end)
            .map_err(|error| Flow::Invariant(error.into()))
    }

    fn derive_termination(&self, scan: &ValueScan) -> Result<CssDeclarationTermination, Flow> {
        match &scan.terminator {
            ValueTerminator::Semicolon(anchor) => {
                Ok(CssDeclarationTermination::AuthoredSemicolon {
                    semicolon: anchor.clone(),
                })
            }
            ValueTerminator::EnclosingRightCurly(anchor) => {
                Ok(CssDeclarationTermination::OmittedBeforeRightCurly {
                    right_curly: anchor.clone(),
                })
            }
            ValueTerminator::TrueEndOfInput => {
                let len = self.source_text.as_str().len();
                let terminal = self
                    .source_text
                    .anchor(len, len)
                    .map_err(|error| Flow::Invariant(error.into()))?;
                Ok(CssDeclarationTermination::OmittedAtEndOfInput { terminal })
            }
        }
    }

    // -- result finalization -------------------------------------------------

    fn finish_complete(mut self) -> Result<CssParserRunResult, CssParserRunError> {
        let terminal = self.upstream.terminal().clone();
        self.finalize_active_contexts(&terminal, |terminal| {
            CssParserContextTermination::EndOfInput { terminal }
        })?;
        let context_records =
            finalize_context_records(std::mem::take(&mut self.pending_context_records))?;
        let coverage = coverage_for(&self.unsupported);
        let resources = self.resource_usage(context_records.len());
        CssParserRunResult::new(
            self.source_text,
            self.upstream,
            self.occurrences,
            self.descriptor_occurrences,
            self.diagnostics,
            self.recovery,
            self.unsupported,
            self.discard,
            context_records,
            terminal,
            CssParserExecutionCompletion::Complete,
            coverage,
            CssParserTermination::EndOfTokenizerInput,
            resources,
        )
    }

    fn finish_upstream_incomplete(mut self) -> Result<CssParserRunResult, CssParserRunError> {
        let terminal = self.upstream.terminal().clone();
        self.finalize_active_contexts(&terminal, |terminal| {
            CssParserContextTermination::UpstreamTokenizerIncomplete { terminal }
        })?;
        let context_records =
            finalize_context_records(std::mem::take(&mut self.pending_context_records))?;
        let coverage = coverage_for(&self.unsupported);
        let resources = self.resource_usage(context_records.len());
        CssParserRunResult::new(
            self.source_text,
            self.upstream,
            self.occurrences,
            self.descriptor_occurrences,
            self.diagnostics,
            self.recovery,
            self.unsupported,
            self.discard,
            context_records,
            terminal,
            CssParserExecutionCompletion::Incomplete,
            coverage,
            CssParserTermination::UpstreamTokenizerIncomplete,
            resources,
        )
    }

    fn finish_resource_limited(
        mut self,
        signal: ResourceLimitSignal,
    ) -> Result<CssParserRunResult, CssParserRunError> {
        let terminal = self
            .source_text
            .anchor(self.committed_pos, self.committed_pos)?;
        self.finalize_active_contexts(&terminal, |terminal| {
            CssParserContextTermination::ParserResourceLimit { terminal }
        })?;
        let context_records =
            finalize_context_records(std::mem::take(&mut self.pending_context_records))?;
        let evidence = CssParserResourceLimitEvidence::new(
            self.source_text,
            signal.kind,
            signal.limit,
            signal.attempted,
            terminal.clone(),
        )?;
        let coverage = coverage_for(&self.unsupported);
        let resources = self.resource_usage(context_records.len());
        CssParserRunResult::new(
            self.source_text,
            self.upstream,
            self.occurrences,
            self.descriptor_occurrences,
            self.diagnostics,
            self.recovery,
            self.unsupported,
            self.discard,
            context_records,
            terminal,
            CssParserExecutionCompletion::Incomplete,
            coverage,
            CssParserTermination::ParserResourceLimit(evidence),
            resources,
        )
    }

    /// `DeclarationOccurrences` usage is the #169 aggregate cap shared by
    /// ordinary declaration and descriptor occurrences, never
    /// `self.occurrences.len()` alone.
    fn resource_usage(&self, context_records_len: usize) -> CssParserResourceUsage {
        CssParserResourceUsage::new(
            self.algorithm_steps,
            self.peak_component_depth,
            self.peak_context_depth,
            self.occurrences.len() + self.descriptor_occurrences.len(),
            self.diagnostics.len(),
            self.recovery.len(),
            self.unsupported.len(),
            self.discard.len(),
            context_records_len,
        )
    }
}

/// Converts every reserved context slot into its finalized record, in
/// `CssParserContextId` allocation order (#167). Every reserved slot must be
/// finalized (by an authored `}` or by run-stop cleanup) before result
/// construction; a still-`None` slot here is a typed internal invariant
/// failure, never a panic.
fn finalize_context_records(
    pending: Vec<Option<CssParserContextRecord>>,
) -> Result<Vec<CssParserContextRecord>, CssParserRunError> {
    pending
        .into_iter()
        .enumerate()
        .map(|(index, slot)| {
            slot.ok_or(CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::UnfinalizedContextRecord { index },
            ))
        })
        .collect()
}

fn coverage_for(unsupported: &[CssParserUnsupportedRegion]) -> CssParserCoverage {
    if unsupported.is_empty() {
        CssParserCoverage::SupportedForSelectedQuestion
    } else {
        CssParserCoverage::ContainsUnsupportedContexts
    }
}

fn termination_complete_end(
    termination: &CssDeclarationTermination,
    colon: &SourceAnchor,
    value: &SourceAnchor,
    priority: Option<&CssDeclarationPriorityEvidence>,
) -> usize {
    match termination {
        CssDeclarationTermination::AuthoredSemicolon { semicolon } => semicolon.range().end(),
        CssDeclarationTermination::OmittedBeforeRightCurly { .. }
        | CssDeclarationTermination::OmittedAtEndOfInput { .. } => {
            if let Some(priority) = priority {
                priority.complete().range().end()
            } else if !value.range().is_empty() {
                value.range().end()
            } else {
                colon.range().end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::tokenizer::producer::run as run_tokenizer;
    use crate::css::tokenizer::resource::CssTokenizerLimits;
    use crate::css::tokenizer::result::CssTokenizerCompletion;
    use crate::{SourceId, SourceText};

    fn source(text: &str) -> SourceText {
        SourceText::new(SourceId::new(1), text.to_owned())
    }

    fn generous_tokenizer_limits() -> CssTokenizerLimits {
        CssTokenizerLimits::new(1 << 20, 1 << 20, 1 << 16, 1 << 16, 1 << 20, 1 << 20).unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    fn parser_limits(
        max_algorithm_steps: usize,
        max_peak_component_depth: usize,
    ) -> CssParserLimits {
        CssParserLimits::new(
            max_algorithm_steps,
            max_peak_component_depth,
            1000,
            1000,
            1000,
            1000,
            1000,
            1000,
            1000,
        )
        .unwrap()
    }

    /// #139 transaction-stop rollback: a `PeakComponentDepth` refusal
    /// surfacing from inside `scan_declaration_value` (itself reached only
    /// through `try_declaration`, called after `handle_block_item` has
    /// already begun a speculative checkpoint) must roll that checkpoint
    /// back before `Flow::ResourceLimit` propagates out of
    /// `handle_block_item`, so no active checkpoint ever reaches result
    /// finalization.
    #[test]
    fn resource_refusal_during_active_declaration_speculation_rolls_back_the_checkpoint() {
        let text = source("a{color:f(g(1));}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let limits = parser_limits(10_000, 1);
        let mut producer = Producer::new(&text, tokenizer_result, limits);

        let outcome = producer.execute();

        assert!(matches!(outcome, Err(Flow::ResourceLimit(_))));
        assert!(
            producer.checkpoint.is_none(),
            "a resource-limit refusal during active declaration speculation must not leave a checkpoint active"
        );
    }

    /// #139 transaction-stop rollback: an upstream tokenizer terminal
    /// reached from inside `scan_declaration_value` while a declaration
    /// checkpoint is active must roll that checkpoint back before
    /// `Flow::UpstreamIncomplete` propagates out of `handle_block_item`.
    #[test]
    fn upstream_incomplete_during_active_declaration_speculation_rolls_back_the_checkpoint() {
        let text = source("a{color:red;}");
        let tight_lexical_items_limit =
            CssTokenizerLimits::new(1 << 20, 1 << 20, 4, 1 << 16, 1 << 20, 1 << 20).unwrap();
        let tokenizer_result = run_tokenizer(&text, tight_lexical_items_limit).unwrap();
        assert_eq!(
            tokenizer_result.completion(),
            CssTokenizerCompletion::Incomplete
        );
        let limits = parser_limits(10_000, 1000);
        let mut producer = Producer::new(&text, tokenizer_result, limits);

        let outcome = producer.execute();

        assert!(matches!(outcome, Err(Flow::UpstreamIncomplete)));
        assert!(
            producer.checkpoint.is_none(),
            "upstream-tokenizer-incomplete during active declaration speculation must not leave a checkpoint active"
        );
    }

    /// #139 full tokenizer/source validation before parser execution: the
    /// same `SourceId` reused across genuinely different `SourceText`
    /// content must be rejected by the shared upstream-boundary validator
    /// (reused from [`super::super::result::validate_upstream_boundary`])
    /// before `Producer::execute` ever runs, even though the upstream
    /// tokenizer result is itself fully valid for the source it was
    /// actually produced from.
    #[test]
    fn run_rejects_reused_source_id_with_different_source_text_before_producer_executes() {
        let original_text = source("a{color:red;}");
        let tokenizer_result = run_tokenizer(&original_text, generous_tokenizer_limits()).unwrap();

        let different_text_same_id = SourceText::new(SourceId::new(1), "a".to_owned());

        let result = run(
            &different_text_same_id,
            tokenizer_result,
            parser_limits(10_000, 1000),
        );

        assert_eq!(
            result.unwrap_err(),
            CssParserRunError::InternalInvariantFailure(
                CssParserInvariantViolation::UpstreamUnprocessedRemainderMismatch
            )
        );
    }

    // -- #167 context-aware producer focused regressions ---------------------

    #[allow(clippy::too_many_arguments)]
    fn context_parser_limits(
        max_peak_context_depth: usize,
        max_context_records: usize,
    ) -> CssParserLimits {
        CssParserLimits::new(
            10_000,
            1000,
            max_peak_context_depth,
            1000,
            1000,
            1000,
            1000,
            1000,
            max_context_records,
        )
        .unwrap()
    }

    /// A structurally recognized nested qualified rule commits its checkpoint
    /// rollback (declaration recognition fails, falls back, no context is
    /// entered while a checkpoint is active) before `enter_context` ever
    /// mutates the context stack: at most one checkpoint is ever active, and
    /// entering the child leaves no checkpoint behind.
    #[test]
    fn entering_a_nested_qualified_rule_context_never_leaves_a_checkpoint_active() {
        let text = source("a{color:green{color:blue;}}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert!(producer.checkpoint.is_none());
        assert_eq!(producer.pending_context_records.len(), 2);
        assert!(producer.active_contexts.is_empty());
    }

    /// `PeakContextDepth` refusal is commit-honest (#167): the refused third
    /// context allocates no ID, the two already-entered ancestors remain
    /// retained, and no checkpoint is left active.
    #[test]
    fn peak_context_depth_refusal_allocates_no_id_and_retains_ancestors() {
        let text = source("a{b{c{p:v;}}}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer = Producer::new(&text, tokenizer_result, context_parser_limits(2, 1000));

        let outcome = producer.execute();

        assert!(matches!(
            outcome,
            Err(Flow::ResourceLimit(ResourceLimitSignal {
                kind: CssParserResourceKind::PeakContextDepth,
                limit: 2,
                attempted: 3,
            }))
        ));
        assert!(producer.checkpoint.is_none());
        assert_eq!(producer.pending_context_records.len(), 2);
        assert_eq!(producer.active_contexts.len(), 2);
        assert_eq!(producer.peak_context_depth, 2);
    }

    /// `ContextRecords` refusal is commit-honest (#167): a third context
    /// beyond the retained-record cap allocates no ID, and the two already-
    /// reserved slots remain (one closed, one still active).
    #[test]
    fn context_records_refusal_allocates_no_id_and_retains_reserved_slots() {
        let text = source("a{b{p:v;}c{q:w;}}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer = Producer::new(&text, tokenizer_result, context_parser_limits(1000, 2));

        let outcome = producer.execute();

        assert!(matches!(
            outcome,
            Err(Flow::ResourceLimit(ResourceLimitSignal {
                kind: CssParserResourceKind::ContextRecords,
                limit: 2,
                attempted: 3,
            }))
        ));
        assert!(producer.checkpoint.is_none());
        assert_eq!(producer.pending_context_records.len(), 2);
        assert_eq!(producer.active_contexts.len(), 1);
    }

    /// Root-scoped and nested direct-item ordinals are independent counters
    /// (#167): two top-level qualified rules are root-item 0 and 1, while
    /// each owns its own zero-based nested item ordinal space.
    #[test]
    fn root_and_nested_direct_item_ordinals_are_independent_counters() {
        let text = source("a{x{p:v;}}b{y:z;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert_eq!(producer.pending_context_records.len(), 3);
        let records: Vec<_> = producer
            .pending_context_records
            .iter()
            .map(|slot| slot.as_ref().unwrap())
            .collect();
        // `a` (root item 0), `x` (child of `a`, item 0), `b` (root item 1).
        assert!(records[0].parent().is_none());
        assert_eq!(records[0].item_ordinal().value(), 0);
        assert_eq!(records[1].parent(), Some(records[0].id()));
        assert_eq!(records[1].item_ordinal().value(), 0);
        assert!(records[2].parent().is_none());
        assert_eq!(records[2].item_ordinal().value(), 1);
    }

    // -- #169 descriptor-context focused regressions -------------------------

    /// `@font-face` with an empty prelude qualifies as a supported
    /// descriptor context, and multiple declaration-shaped items inside it
    /// become `CssDescriptorOccurrence`s, never ordinary
    /// `CssDeclarationOccurrence`s.
    #[test]
    fn font_face_with_multiple_descriptor_occurrences() {
        let text = source("@font-face{font-family:x;src:y;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert!(producer.occurrences.is_empty());
        assert_eq!(producer.descriptor_occurrences.len(), 2);
        assert_eq!(producer.pending_context_records.len(), 1);
        let record = producer.pending_context_records[0].as_ref().unwrap();
        assert_eq!(
            record.kind(),
            CssParserContextKind::DescriptorRuleBlock(CssParserDescriptorRuleKind::FontFace)
        );
        assert!(record.parent().is_none());
        assert!(record.descriptor_property_name().is_none());
        for occurrence in &producer.descriptor_occurrences {
            assert_eq!(occurrence.placement().context_id(), record.id());
        }
        assert_eq!(
            producer.descriptor_occurrences[0]
                .placement()
                .item_ordinal()
                .value(),
            0
        );
        assert_eq!(
            producer.descriptor_occurrences[1]
                .placement()
                .item_ordinal()
                .value(),
            1
        );
    }

    /// `@property --x` with a single qualifying custom-property-name
    /// qualifies as a supported descriptor context, retaining the exact
    /// authored custom-property-name anchor as parent evidence.
    #[test]
    fn property_with_qualifying_name_produces_descriptor_occurrences() {
        let text = source("@property --x{syntax:y;inherits:false;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert_eq!(producer.descriptor_occurrences.len(), 2);
        assert_eq!(producer.pending_context_records.len(), 1);
        let record = producer.pending_context_records[0].as_ref().unwrap();
        assert_eq!(
            record.kind(),
            CssParserContextKind::DescriptorRuleBlock(CssParserDescriptorRuleKind::Property)
        );
        assert_eq!(
            record
                .descriptor_property_name()
                .map(SourceAnchor::fragment),
            Some("--x")
        );
    }

    /// An unqualified `@property` prelude (not a `<custom-property-name>`)
    /// never enters a descriptor context: the whole at-rule remains
    /// structurally consumed unsupported evidence and produces no
    /// descriptor occurrences.
    #[test]
    fn unqualified_property_prelude_produces_no_descriptors() {
        let text = source("@property color{syntax:y;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert!(producer.descriptor_occurrences.is_empty());
        assert!(producer.pending_context_records.is_empty());
        assert_eq!(producer.unsupported.len(), 1);
    }

    /// The reserved exact spelling `--` never qualifies as a custom-property
    /// name.
    #[test]
    fn reserved_double_hyphen_property_name_does_not_qualify() {
        let text = source("@property --{syntax:y;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert!(producer.descriptor_occurrences.is_empty());
        assert_eq!(producer.unsupported.len(), 1);
    }

    /// Broader multi-name `@property --a, --b { ... }` syntax remains
    /// explicit unsupported capability evidence in #169, never a descriptor
    /// context.
    #[test]
    fn multi_name_property_remains_unsupported() {
        let text = source("@property --a, --b{syntax:y;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert!(producer.descriptor_occurrences.is_empty());
        assert!(producer.pending_context_records.is_empty());
        assert_eq!(producer.unsupported.len(), 1);
    }

    /// A non-empty `@font-face` prelude never qualifies as a descriptor
    /// context.
    #[test]
    fn non_empty_font_face_prelude_remains_unsupported() {
        let text = source("@font-face x{font-family:y;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert!(producer.descriptor_occurrences.is_empty());
        assert!(producer.pending_context_records.is_empty());
        assert_eq!(producer.unsupported.len(), 1);
    }

    /// Identical `name:value` spelling inside an ordinary qualified rule
    /// versus a descriptor context yields distinct occurrence types.
    #[test]
    fn same_spelling_yields_distinct_occurrence_types_by_context() {
        let text = source("a{color:red;}@font-face{color:red;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert_eq!(producer.occurrences.len(), 1);
        assert_eq!(producer.descriptor_occurrences.len(), 1);
        assert_eq!(
            producer.occurrences[0].property_name().fragment(),
            producer.descriptor_occurrences[0].name().fragment()
        );
    }

    /// Duplicate raw descriptor spelling retained in two separate descriptor
    /// contexts keeps distinct owner `ContextId`s.
    #[test]
    fn duplicate_descriptor_spelling_in_two_contexts_retains_distinct_owner_ids() {
        let text = source("@font-face{unicode-range:a;}@property --x{unicode-range:a;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert_eq!(producer.descriptor_occurrences.len(), 2);
        assert_eq!(producer.pending_context_records.len(), 2);
        assert_ne!(
            producer.descriptor_occurrences[0].placement().context_id(),
            producer.descriptor_occurrences[1].placement().context_id(),
        );
    }

    /// A malformed descriptor-shaped item recovers with the shared
    /// `InvalidBlockItem`/`MalformedBlockItem` evidence, and a later
    /// valid-shaped descriptor still commits.
    #[test]
    fn malformed_descriptor_item_recovers_and_later_descriptor_survives() {
        let text = source("@font-face{***;src:y;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert_eq!(producer.descriptor_occurrences.len(), 1);
        assert_eq!(producer.recovery.len(), 1);
        assert_eq!(producer.diagnostics.len(), 1);
    }

    /// A qualified-rule-shaped fragment inside a descriptor block is
    /// malformed content, not a child context: a top-level `{` is balanced
    /// through as ordinary malformed content (never a nested-rule trigger),
    /// so it never becomes a `QualifiedRuleBlock`, and the descriptor
    /// context stays the only retained context. The malformed region's own
    /// terminating `;` bounds it, so the later `src:y;` still recovers as a
    /// separate, valid descriptor occurrence.
    #[test]
    fn qualified_rule_shaped_fragment_inside_descriptor_block_is_not_a_child_context() {
        let text = source("@font-face{.foo{color:red;};src:y;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert_eq!(producer.pending_context_records.len(), 1);
        assert_eq!(producer.descriptor_occurrences.len(), 1);
        assert_eq!(producer.recovery.len(), 1);
    }

    /// An unsupported at-rule inside a descriptor block -- including a
    /// registry member like `@media` that would otherwise qualify as a
    /// `GroupRuleBlock` in a qualified/group context -- never becomes a
    /// group context: `<declaration-list>` automatically excludes at-rules,
    /// so it is always exactly one explicit unsupported direct item, and
    /// descriptor parsing resumes afterward.
    #[test]
    fn media_inside_descriptor_block_never_becomes_a_group_context() {
        let text = source("@font-face{@media screen{color:red;}src:y;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer =
            Producer::new(&text, tokenizer_result, context_parser_limits(1000, 1000));

        assert!(producer.execute().is_ok());

        assert_eq!(producer.pending_context_records.len(), 1);
        assert_eq!(producer.descriptor_occurrences.len(), 1);
        assert_eq!(producer.unsupported.len(), 1);
    }

    /// True EOF with a descriptor context still active retains it with
    /// honest `EndOfInput` termination and correct root-level ancestry
    /// (`parent = None`), never a fabricated closing `}`.
    #[test]
    fn true_eof_descriptor_context_retains_honest_termination_and_root_ancestry() {
        let text = source("@font-face{font-family:x");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let limits = context_parser_limits(1000, 1000);
        let result = run(&text, tokenizer_result, limits).unwrap();

        assert_eq!(result.context_records().len(), 1);
        let record = &result.context_records()[0];
        assert!(record.parent().is_none());
        assert!(matches!(
            record.termination(),
            CssParserContextTermination::EndOfInput { .. }
        ));
        assert_eq!(result.descriptor_occurrences().len(), 1);
        assert_eq!(
            result.descriptor_occurrences()[0].placement().context_id(),
            record.id()
        );
    }

    /// `PeakContextDepth`/`ContextRecords` refusal at descriptor entry is
    /// commit-honest (#167/#169): a refused descriptor context allocates no
    /// ID and leaves no partial record.
    #[test]
    fn descriptor_context_entry_resource_refusal_allocates_no_id() {
        let text = source("@font-face{font-family:x;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let mut producer = Producer::new(&text, tokenizer_result, context_parser_limits(0, 1000));

        let outcome = producer.execute();

        assert!(matches!(
            outcome,
            Err(Flow::ResourceLimit(ResourceLimitSignal {
                kind: CssParserResourceKind::PeakContextDepth,
                limit: 0,
                attempted: 1,
            }))
        ));
        assert!(producer.pending_context_records.is_empty());
        assert!(producer.descriptor_occurrences.is_empty());
    }

    /// The `DeclarationOccurrences` resource is the #169 aggregate cap
    /// shared by ordinary declarations and descriptor occurrences: one
    /// ordinary declaration followed by one descriptor occurrence exhausts a
    /// limit of one.
    #[test]
    fn declaration_occurrences_aggregate_limit_is_shared_across_descriptor_evidence() {
        let text = source("a{color:red;}@font-face{font-family:x;}");
        let tokenizer_result = run_tokenizer(&text, generous_tokenizer_limits()).unwrap();
        let limits = CssParserLimits::new(10_000, 1000, 1000, 1, 1000, 1000, 1000, 1000, 1000)
            .expect("valid parser limits");
        let mut producer = Producer::new(&text, tokenizer_result, limits);

        let outcome = producer.execute();

        assert!(matches!(
            outcome,
            Err(Flow::ResourceLimit(ResourceLimitSignal {
                kind: CssParserResourceKind::DeclarationOccurrences,
                limit: 1,
                attempted: 2,
            }))
        ));
        assert_eq!(producer.occurrences.len(), 1);
        assert!(producer.descriptor_occurrences.is_empty());
    }
}
