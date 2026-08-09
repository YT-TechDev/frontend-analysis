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

use super::super::declaration::{
    CssDeclarationContext, CssDeclarationOccurrence, CssDeclarationPriorityEvidence,
    CssDeclarationTermination,
};
use super::super::token::CssTokenKind;
use super::super::tokenizer::result::CssTokenizerRunResult;
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
    validate_upstream_source_identity(source_text, &upstream_tokenizer_result)?;

    let mut producer = Producer::new(source_text, upstream_tokenizer_result, limits);
    match producer.execute() {
        Ok(()) => producer.finish_complete(),
        Err(Flow::ResourceLimit(signal)) => producer.finish_resource_limited(signal),
        Err(Flow::UpstreamIncomplete) => producer.finish_upstream_incomplete(),
        Err(Flow::Invariant(error)) => Err(error),
    }
}

/// A minimal fail-fast check ahead of the main parse loop. The final
/// `CssParserRunResult::new` call remains the authoritative, complete
/// upstream-boundary validation (prefix/remainder fragment reconciliation
/// included); this only avoids doing a full parse over evidence that could
/// never construct a valid result.
fn validate_upstream_source_identity(
    source_text: &SourceText,
    upstream: &CssTokenizerRunResult,
) -> Result<(), CssParserRunError> {
    let expected = source_text.id();
    let actual = upstream.source_id();
    if actual != expected {
        return Err(CssParserRunError::InternalInvariantFailure(
            CssParserInvariantViolation::UpstreamSourceIdentityMismatch { expected, actual },
        ));
    }
    Ok(())
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
    AtKeyword,
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
        CssTokenKind::AtKeyword(_) => ObservedKind::AtKeyword,
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

/// One resolved outcome of the declaration-vs-qualified-rule transaction.
enum DeclarationOutcome {
    Recognized(CssDeclarationOccurrence),
    NotRecognized,
}

enum AtRuleOutcome {
    Ended { end: usize },
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

enum BlockItemOutcome {
    Continue,
    ZoneEnded,
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
struct Checkpoint {
    cursor: CssParserCursor,
    occurrences_len: usize,
    diagnostics_len: usize,
    recovery_len: usize,
    unsupported_len: usize,
    discard_len: usize,
    component_frames_len: usize,
}

struct Producer<'a> {
    source_text: &'a SourceText,
    upstream: CssTokenizerRunResult,
    limits: CssParserLimits,
    cursor: CssParserCursor,
    committed_pos: usize,
    occurrences: Vec<CssDeclarationOccurrence>,
    diagnostics: Vec<CssParserDiagnostic>,
    recovery: Vec<CssParserRecoveryEvidence>,
    unsupported: Vec<CssParserUnsupportedRegion>,
    discard: Vec<CssParserDiscardEvidence>,
    algorithm_steps: usize,
    peak_component_depth: usize,
    component_frames: Vec<ComponentFrameKind>,
    checkpoint: Option<Checkpoint>,
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
            diagnostics: Vec::new(),
            recovery: Vec::new(),
            unsupported: Vec::new(),
            discard: Vec::new(),
            algorithm_steps: 0,
            peak_component_depth: 0,
            component_frames: Vec::new(),
            checkpoint: None,
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
        self.checkpoint = Some(Checkpoint {
            cursor: self.cursor.checkpoint(),
            occurrences_len: self.occurrences.len(),
            diagnostics_len: self.diagnostics.len(),
            recovery_len: self.recovery.len(),
            unsupported_len: self.unsupported.len(),
            discard_len: self.discard.len(),
            component_frames_len: self.component_frames.len(),
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
        self.diagnostics.truncate(checkpoint.diagnostics_len);
        self.recovery.truncate(checkpoint.recovery_len);
        self.unsupported.truncate(checkpoint.unsupported_len);
        self.discard.truncate(checkpoint.discard_len);
        self.component_frames
            .truncate(checkpoint.component_frames_len);
        Ok(())
    }

    // -- top-level stylesheet flow -----------------------------------------

    fn execute(&mut self) -> Result<(), Flow> {
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => match kind {
                    ObservedKind::Whitespace | ObservedKind::Cdo | ObservedKind::Cdc => {
                        let end = anchor.range().end();
                        self.cursor.advance();
                        self.committed_pos = end;
                    }
                    ObservedKind::AtKeyword => {
                        self.cursor.advance();
                        self.consume_top_level_at_rule(anchor)?;
                    }
                    _ => {
                        self.consume_top_level_qualified_rule(anchor.range().start())?;
                    }
                },
                ParserPosition::TrueEndOfInput => return Ok(()),
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    fn consume_top_level_at_rule(&mut self, at_keyword: SourceAnchor) -> Result<(), Flow> {
        let AtRuleOutcome::Ended { end } = self.consume_top_level_at_rule_body()?;
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
                    None => self.parse_block_declarations(),
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

    fn parse_block_declarations(&mut self) -> Result<(), Flow> {
        loop {
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
                        return Ok(());
                    }
                    ObservedKind::AtKeyword => {
                        let start = anchor.range().start();
                        self.begin_nested_content_remainder(start)?;
                        return Ok(());
                    }
                    _ => {
                        let item_start = anchor.range().start();
                        match self.handle_block_item(item_start)? {
                            BlockItemOutcome::Continue => {}
                            BlockItemOutcome::ZoneEnded => return Ok(()),
                        }
                    }
                },
                ParserPosition::TrueEndOfInput => return Ok(()),
                ParserPosition::UpstreamTokenizerTerminal => return Err(Flow::UpstreamIncomplete),
            }
        }
    }

    fn handle_block_item(&mut self, item_start: usize) -> Result<BlockItemOutcome, Flow> {
        self.begin_checkpoint()?;
        match self.try_declaration()? {
            DeclarationOutcome::Recognized(occurrence) => {
                self.commit_declaration(occurrence)?;
                self.commit_checkpoint()?;
                Ok(BlockItemOutcome::Continue)
            }
            DeclarationOutcome::NotRecognized => {
                self.rollback_checkpoint()?;
                match self.scan_qualified_rule_fallback()? {
                    FallbackOutcome::NestedRuleTrigger => {
                        self.begin_nested_content_remainder(item_start)?;
                        Ok(BlockItemOutcome::ZoneEnded)
                    }
                    FallbackOutcome::MalformedEndedAtSemicolon { semicolon } => {
                        let region_end = semicolon.range().end();
                        self.commit_malformed_recovery(
                            item_start,
                            region_end,
                            CssParserRecoveryTermination::AuthoredSemicolon { semicolon },
                        )?;
                        Ok(BlockItemOutcome::Continue)
                    }
                    FallbackOutcome::MalformedEndedAtEnclosingBlock { right_curly } => {
                        let region_end = right_curly.range().start();
                        self.commit_malformed_recovery(
                            item_start,
                            region_end,
                            CssParserRecoveryTermination::EnclosingBlockEnd { right_curly },
                        )?;
                        Ok(BlockItemOutcome::Continue)
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
                        )?;
                        Ok(BlockItemOutcome::Continue)
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

    fn scan_qualified_rule_fallback(&mut self) -> Result<FallbackOutcome, Flow> {
        self.component_frames.clear();
        loop {
            match self.next_semantic()? {
                ParserPosition::SemanticToken { anchor, kind } => {
                    let depth_before = self.component_frames.len();
                    if depth_before == 0 {
                        match kind {
                            ObservedKind::LeftCurlyBracket => {
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

    fn begin_nested_content_remainder(&mut self, remainder_start: usize) -> Result<(), Flow> {
        match self.consume_remainder_until_enclosing_right_curly()? {
            RemainderOutcome::EnclosingRightCurly { start, end } => {
                self.commit_unsupported_nested_remainder(remainder_start, start)?;
                self.cursor.advance();
                self.committed_pos = end;
                Ok(())
            }
            RemainderOutcome::TrueEndOfInput => {
                let len = self.source_text.as_str().len();
                self.commit_unsupported_nested_remainder(remainder_start, len)
            }
        }
    }

    fn commit_unsupported_nested_remainder(
        &mut self,
        start: usize,
        end: usize,
    ) -> Result<(), Flow> {
        let prospective = checked_resource_add(
            CssParserResourceKind::UnsupportedRegions,
            self.unsupported.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::UnsupportedRegions, prospective)?;
        let region = self
            .source_text
            .anchor(start, end)
            .map_err(|error| Flow::Invariant(error.into()))?;
        let evidence =
            CssParserUnsupportedRegion::new_nested_content_remainder(self.source_text, region)
                .map_err(|error| Flow::Invariant(error.into()))?;
        self.unsupported.push(evidence);
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

        let occurrence = CssDeclarationOccurrence::new(
            self.source_text,
            complete_anchor,
            name_anchor,
            colon_anchor,
            value_anchor,
            priority,
            termination,
            CssDeclarationContext::TopLevelQualifiedRuleLeadingDeclarationZone,
        )
        .map_err(|error| Flow::Invariant(error.into()))?;

        Ok(DeclarationOutcome::Recognized(occurrence))
    }

    fn commit_declaration(&mut self, occurrence: CssDeclarationOccurrence) -> Result<(), Flow> {
        let prospective = checked_resource_add(
            CssParserResourceKind::DeclarationOccurrences,
            self.occurrences.len(),
            1,
        )?;
        self.check_limit(CssParserResourceKind::DeclarationOccurrences, prospective)?;
        let end = occurrence.complete().range().end();
        self.occurrences.push(occurrence);
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

    fn finish_complete(self) -> Result<CssParserRunResult, CssParserRunError> {
        let terminal = self.upstream.terminal().clone();
        let coverage = coverage_for(&self.unsupported);
        let resources = self.resource_usage();
        CssParserRunResult::new(
            self.source_text,
            self.upstream,
            self.occurrences,
            self.diagnostics,
            self.recovery,
            self.unsupported,
            self.discard,
            terminal,
            CssParserExecutionCompletion::Complete,
            coverage,
            CssParserTermination::EndOfTokenizerInput,
            resources,
        )
    }

    fn finish_upstream_incomplete(self) -> Result<CssParserRunResult, CssParserRunError> {
        let terminal = self.upstream.terminal().clone();
        let coverage = coverage_for(&self.unsupported);
        let resources = self.resource_usage();
        CssParserRunResult::new(
            self.source_text,
            self.upstream,
            self.occurrences,
            self.diagnostics,
            self.recovery,
            self.unsupported,
            self.discard,
            terminal,
            CssParserExecutionCompletion::Incomplete,
            coverage,
            CssParserTermination::UpstreamTokenizerIncomplete,
            resources,
        )
    }

    fn finish_resource_limited(
        self,
        signal: ResourceLimitSignal,
    ) -> Result<CssParserRunResult, CssParserRunError> {
        let terminal = self
            .source_text
            .anchor(self.committed_pos, self.committed_pos)?;
        let evidence = CssParserResourceLimitEvidence::new(
            self.source_text,
            signal.kind,
            signal.limit,
            signal.attempted,
            terminal.clone(),
        )?;
        let coverage = coverage_for(&self.unsupported);
        let resources = self.resource_usage();
        CssParserRunResult::new(
            self.source_text,
            self.upstream,
            self.occurrences,
            self.diagnostics,
            self.recovery,
            self.unsupported,
            self.discard,
            terminal,
            CssParserExecutionCompletion::Incomplete,
            coverage,
            CssParserTermination::ParserResourceLimit(evidence),
            resources,
        )
    }

    fn resource_usage(&self) -> CssParserResourceUsage {
        CssParserResourceUsage::new(
            self.algorithm_steps,
            self.peak_component_depth,
            self.occurrences.len(),
            self.diagnostics.len(),
            self.recovery.len(),
            self.unsupported.len(),
            self.discard.len(),
        )
    }
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
