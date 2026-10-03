//! Candidate-independent validation for Issue #882.
//!
//! This module validates the authored Data-state U+0000 correction semantics
//! through the existing selected document-tree profile, before any production
//! placement or implementation:
//!
//! ```text
//! authored Data U+0000
//!     -> tokenizer: UnexpectedNullCharacter, handling Continued,
//!                   one source-backed U+0000 character token
//!     -> selected InBody: tree parse error + ignored token
//!     -> final tree: no U+0000 text, no fabricated U+FFFD text
//! ```
//!
//! It is **test-only** and changes no production behavior. It does not
//! authorize a production correction, and it is not an independent
//! acceptance of itself.
//!
//! # Independent oracle boundary
//!
//! This module imports nothing from the production HTML tokenizer, the
//! tree-construction driver, session, or result, the null-recovery helper,
//! the tokenizer corpus, the CLI, or any browser, WPT, or html5lib output.
//! Every expected value is hand-authored from the pinned WHATWG text. The only
//! crate items used are the `SourceText` / `SourceId` anchoring primitives, so
//! that authored evidence ranges are real. Expected values are never produced
//! by running production, never derived from `ERR-003`, `PRE-010`, or
//! `RES-004`, never reconstructed by rescanning source, and never inferred
//! from interpreted lengths.
//!
//! # Pins
//!
//! WHATWG HTML `0cd32204c6d9408be0a42cb15c86145e21deab9d`, source blob
//! `05319af4659e15aafb3f5137362b7d9561db8812` (the blob identity was
//! re-derived by hashing the fetched pinned `source` file while authoring this
//! leaf, and the rules below were transcribed from that file).
//! Challenge/corroboration only, never the oracle: WPT
//! `f37e2295b34838f17c612a527f52f3056e7b6da2`, html5lib-tests
//! `c777c408b61078ea2eb4acefc2535f54dbc8b28a`.
//!
//! # Pinned obligations the gold is derived from
//!
//! - **Data state**, U+0000: `unexpected-null-character` parse error, then
//!   *emit the current input character* as a character token. No replacement.
//! - **RCDATA state**, U+0000: the same parse error, then emit **U+FFFD**.
//!   The same replacement holds for the tag-name, double-quoted and unquoted
//!   attribute-value states (and for RAWTEXT, PLAINTEXT, and Script data).
//!   The replacement is state-specific; it is not a generic "all NUL" rule.
//! - **In body**, a character token that is U+0000: parse error, ignore the
//!   token. NUL handling is therefore *spread across both stages*.
//! - **After body**, any character token that is not ASCII whitespace: parse
//!   error, switch to *in body*, reprocess the same token. So
//!   `</body>` followed by U+0000 reaches the In-body ignore rule.
//! - **Numeric Character Reference zero** (`&#0;`, `&#x0;`) is a different
//!   object: `null-character-reference`, replaced by U+FFFD, and U+FFFD is an
//!   ordinary character in body.
//! - Decoded reference output is never re-tokenized.
//!
//! # Falsified shortcuts (kept executable below)
//!
//! 1. Tokenizer NUL handling does not share one U+FFFD replacement
//!    ([`f01_nul_handling_is_state_specific`]).
//! 2. The correction is not tokenizer-only ([`f02_the_correction_is_not_tokenizer_only`]).
//! 3. One mixed aggregate `a\0b` token with one anchor is not sufficient
//!    ([`f03_a_mixed_aggregate_token_cannot_carry_the_theorem`]).
//! 4. The tree cannot recover the ignored NUL by scanning interpreted text
//!    later ([`f04_the_tree_cannot_split_an_aggregate_by_scanning`]).
//! 5. Interpreted-byte offsets are not authored source offsets
//!    ([`f05_interpreted_offsets_are_not_authored_offsets`]).
//! 6. Tokenizer unexpected-null and tree ignored-token evidence are different
//!    facts ([`f06_tokenizer_and_tree_evidence_are_different_facts`]).
//! 7. Authored U+0000 and Numeric zero are different objects
//!    ([`f07_authored_nul_and_numeric_zero_are_different`]).
//! 8. RCDATA NUL does not have to widen together with Data
//!    ([`f08_rcdata_nul_does_not_widen_with_data`]).
//! 9. No new tokenizer state is needed ([`f09_no_new_tokenizer_state_is_needed`]).
//! 10. No new insertion mode is needed ([`f10_no_new_insertion_mode_is_needed`]).
//! 11. No new provenance domain is needed ([`f11_no_new_provenance_domain_is_needed`]).
//! 12. No new resource dimension is needed ([`f12_no_new_resource_dimension_is_needed`]).
//! 13. The current #112 GOLD is not an oracle and is not rewritten here
//!     ([`f13_current_defective_gold_is_not_an_oracle`]).
//! 14. The successor correspondence cannot be rewritten indiscriminately
//!     ([`f14_successor_correspondence_is_not_one_bucket`]).
//! 15. After-after-body does not widen ([`f15_after_after_body_does_not_widen`]).
//! 16. Decoded reference output never re-enters authored-input processing
//!     ([`f16_decoded_output_is_not_authored_input`]).
//! 17. Final tree shape alone is not sufficient evidence
//!     ([`f17_final_tree_shape_alone_is_insufficient`]).
//!
//! # Findings carried to the placement review
//!
//! - **Source-backed separation is the minimal result.** Under the current
//!   one-anchor-per-character-token contract a mixed aggregate cannot give the
//!   tree exact surviving contributions without rescanning or endpoint
//!   inference. The model therefore emits ordinary runs and each authored NUL
//!   as separate source-backed character tokens. The exact helper layout is
//!   not selected.
//! - **NUL emission granularity.** Whether a homogeneous run of NULs is one
//!   token or one token per NUL is not determined by the pinned text. The
//!   model uses one token per NUL; tree-level assertions are written over
//!   scalars and contributions so they do not depend on it. Absolute
//!   emitted-token counts for dense NUL are model-specific.
//! - **Diagnostic survival is resolved by #111, not a placement choice.**
//!   `UnexpectedNullCharacter` is observation-conditioned: only
//!   `EndTagWithAttributes` and `EndTagWithTrailingSolidus` are
//!   emission-conditioned. For corrected Data-state U+0000 the handling is
//!   `Continued`; there is no replacement recovery sub-effect whose completion
//!   must precede the truth of the parse-error fact (the old replacement path
//!   reserved U+FFFD bytes first only so that `Recovered(Replaced…)` never
//!   claimed a replacement that could not happen, a rationale that does not
//!   transfer). The model lifecycle is therefore single:
//!   observation (transition commits) -> `UnexpectedNullCharacter` /
//!   `Continued` commits and its site becomes processed coverage -> the
//!   independent source-backed U+0000 retained/token effect is attempted. A
//!   later refusal leaves the diagnostic committed, commits no token and no
//!   U+FFFD, reports the unemitted NUL as an explicit abandoned region, and
//!   keeps the run incomplete. Accepted precedent: the
//!   `MissingAttributeValue` survival-with-`AbandonedInput` regression in
//!   `tokenizer/review_regression_tests.rs`.
//! - **Prior run versus NUL effect.** These are different boundaries. A
//!   pending ordinary run is emitted before the NUL semantic step begins, as
//!   required by Issue #882; if that emission is refused the NUL was never
//!   observed and no `UnexpectedNullCharacter` exists. Once the NUL has been
//!   observed and its diagnostic committed, nothing later deletes it.
//! - **Reference and tag step counts** are a model approximation (one
//!   examined unit, one transition) and are never asserted in absolute terms;
//!   absolute counts are asserted only for inputs without references or tags.
//!
//! - **Tree-side diagnostic capacity.** Each ignored NUL is a tree parse
//!   error. The live tree session has no U+0000 rule at all today, so it
//!   would insert whatever character token it receives. How tree-side
//!   ignored-token evidence is bounded for dense NUL (for example the
//!   existing 257-NUL tokenizer-diagnostic case in `tree/tests.rs`) is a
//!   placement question; this leaf adds no tree resource dimension.
//! - **Attribute-name NUL.** The pinned Standard also replaces NUL with U+FFFD
//!   in attribute names. It is untouched by the correction and not modelled.
//!
//! # Compatibility inventory (observed at base, not mutated here)
//!
//! Current GOLD that a later placement must explicitly supersede:
//! `ERR-003` (`tokenizer/validation/corpus/diagnostics.rs`: Data NUL ->
//! U+FFFD, `Recovered(ReplacedNullWithReplacementCharacter)`), `PRE-010`
//! (`.../preprocessing.rs` and its assertion in `tokenizer/validation/tests.rs`:
//! the mixed case folds NUL to U+FFFD), and `RES-004`
//! (`.../unsupported_resources.rs`: wording "before recovery mutation"; its
//! numbers -- one transition, no token, no diagnostic, zero coverage -- are
//! the same under the corrected semantics, see
//! [`g_resources_common`]). `transition_audit.rs` step counts stay valid
//! because one authored NUL is one Data transition.
//!
//! Production correspondence that a later correction must update:
//! `ds12_data_nul_behavior_is_unchanged` in
//! `data_state_named_reference_successor_production.rs`; the Data half of
//! `&#65;\0` in `data_rcdata_numeric_reference_successor_production.rs`; and
//! the product-level 257-NUL case in `tree/tests.rs` (whose tokenizer
//! diagnostic count is expected to hold but whose interaction with tree-side
//! evidence is a placement question).
//!
//! Still-valid negative controls: RCDATA NUL stays refused
//! (`in_head_title_rcdata_named_reference_successor_production.rs`, the RCDATA
//! half of the Numeric correspondence), tag-name and attribute NUL replacement
//! (the `<a\0>` and `<a x\0=y\0>` cases in `corpus/adversarial.rs`, `review_regression_tests.rs`), the CLI
//! `RcdataNullRecovery` mapping, and `\0<div>` completing with diagnostics in
//! `analysis/tests.rs` and `parser/tests.rs`.
//!
//! Historical candidate-specific evidence left untouched: the Data Named,
//! Data/RCDATA Numeric, and Title Named validation leaves, which refuse Data
//! NUL as outside their candidates and make no claim about its output.
//!
//! Sample shapes that hand-build a Data NUL as U+FFFD without exercising the
//! tokenizer (`token_tests.rs`, `token_contract_matrix_tests.rs`,
//! `tokenizer/tests.rs`) are constructor-contract samples, not behavior; the
//! contracts do not constrain the pairing. Unrelated U+FFFD behavior that must
//! stay: Numeric zero (`NullCharacterReference`), remapped numeric controls,
//! and every non-Data NUL replacement above.
//!
//! # Deliberately bounded model
//!
//! The model is a unit-driven tokenizer (Data, the selected Title RCDATA
//! feedback, and just enough tag and reference states to carry the negative
//! controls) plus a small tree over the existing selected positions. It
//! recognizes only lowercase ASCII tags, `&amp;`, and decimal/hex references to
//! 0 or ASCII graphic scalars; every other shape is an explicit
//! [`Outside`] boundary. It is not a second HTML parser and selects no
//! production state layout, cursor API, diagnostic enum, anchor encoding,
//! resource representation, or tree-side error naming.
//!
//! # Implementation-side adversarial sealing
//!
//! [`Mutation`] lets the model be broken in each way the theorem claims to
//! detect. [`s01_every_mutation_is_detected_by_its_declared_detector`] proves
//! each mutation is caught by the named theorem group, while the sound model
//! passes every group. The resource-lifecycle mutations (diagnostic suppressed on
//! token or retained refusal, NUL effect prepared atomically before the
//! diagnostic, diagnostic site left outside coverage) are killed by the hand
//! authored resource cells.

use crate::{SourceId, SourceText};
use std::panic::{self, AssertUnwindSafe};

const WHATWG_HEAD: &str = "0cd32204c6d9408be0a42cb15c86145e21deab9d";
const WHATWG_SOURCE_BLOB: &str = "05319af4659e15aafb3f5137362b7d9561db8812";
const WPT_CHALLENGE_HEAD: &str = "f37e2295b34838f17c612a527f52f3056e7b6da2";
const HTML5LIB_CHALLENGE_HEAD: &str = "c777c408b61078ea2eb4acefc2535f54dbc8b28a";

// ---------------------------------------------------------------------------
// Model configuration and test-local mutations
// ---------------------------------------------------------------------------

/// Deliberate faults, used only to prove the validator detects them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutation {
    /// Data U+0000 becomes U+FFFD (the current defect).
    NulBecomesReplacement,
    /// Data U+0000 stays U+0000 but is claimed as replacement recovery.
    NulClaimsReplacementRecovery,
    /// The U+0000 token is dropped; only the diagnostic remains.
    DropNulToken,
    /// Data U+0000 reserves three retained bytes (the U+FFFD cost).
    NulReservesReplacementBytes,
    /// Data U+0000 costs two transitions.
    NulCostsTwoTransitions,
    /// Tag-name U+0000 stays U+0000 with `Continued`.
    TagNulKeepsNul,
    /// RCDATA U+0000 follows the Data rule.
    RcdataNulUsesDataRule,
    /// Numeric zero decodes to authored-style U+0000.
    NumericZeroAsNul,
    /// The tokenizer coalesces NUL into the surrounding run; the tree keeps
    /// one whole-run anchor with the NUL scalar removed.
    AggregateWholeAnchor,
    /// The tokenizer coalesces NUL into the surrounding run; the tree splits
    /// it by interpreted offsets.
    AggregateSplitByInterpretedOffsets,
    /// The tree inserts U+0000 as text.
    TreeInsertsNul,
    /// The tree inserts a fabricated U+FFFD for the ignored NUL.
    TreeInsertsReplacement,
    /// The tree ignores the text but retains the NUL as a contribution.
    TreeRetainsNulContribution,
    /// The tree ignores the NUL without recording tree evidence.
    TreeRecordsNoEvidence,
    /// After-body character data is no longer recovered and reprocessed.
    AfterBodyNoReprocess,
    /// After-after-body character data is widened to the In-body rule.
    AfterAfterBodyWidened,
    /// A resource-limited run is reported Complete.
    UpgradeIncompleteToComplete,
    /// The diagnostic is erased when the U+0000 token emission is refused.
    SuppressDiagnosticOnTokenRefusal,
    /// The diagnostic is erased when U+0000 retention is refused.
    SuppressDiagnosticOnRetainedRefusal,
    /// Token and retained effects are prepared before the diagnostic commits,
    /// so an observed NUL leaves no diagnostic when they refuse.
    NulEffectPreparedBeforeDiagnostic,
    /// The diagnostic commits but its site is not added to processed coverage.
    DiagnosticSiteNotCovered,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Model {
    mutation: Option<Mutation>,
}

impl Model {
    const SOUND: Self = Self { mutation: None };

    const fn mutated(mutation: Mutation) -> Self {
        Self {
            mutation: Some(mutation),
        }
    }

    fn is(self, mutation: Mutation) -> bool {
        self.mutation == Some(mutation)
    }

    fn aggregates(self) -> bool {
        self.is(Mutation::AggregateWholeAnchor)
            || self.is(Mutation::AggregateSplitByInterpretedOffsets)
    }
}

// ---------------------------------------------------------------------------
// Authored evidence and input-stream units
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct Evidence {
    source_id: SourceId,
    start: usize,
    end: usize,
    raw: String,
}

fn evidence(source: &SourceText, start: usize, end: usize) -> Evidence {
    let anchor = source.anchor(start, end).expect("candidate byte range");
    Evidence {
        source_id: anchor.source_id(),
        start: anchor.range().start(),
        end: anchor.range().end(),
        raw: anchor.fragment().to_owned(),
    }
}

/// One normalized input unit. Authored CR and CRLF become one LF unit that
/// still spans its authored bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Unit {
    ch: char,
    start: usize,
    end: usize,
}

fn normalize(text: &str) -> Vec<Unit> {
    let mut units = Vec::new();
    let mut iter = text.char_indices().peekable();
    while let Some((start, ch)) = iter.next() {
        if ch == '\r' {
            if let Some(&(next, '\n')) = iter.peek() {
                iter.next();
                units.push(Unit {
                    ch: '\n',
                    start,
                    end: next + 1,
                });
            } else {
                units.push(Unit {
                    ch: '\n',
                    start,
                    end: start + 1,
                });
            }
        } else {
            units.push(Unit {
                ch,
                start,
                end: start + ch.len_utf8(),
            });
        }
    }
    units
}

/// Independent newline normalization of a raw fragment, used to check that an
/// ordinary literal contribution's interpreted text is its authored text.
fn normalized_text(raw: &str) -> String {
    normalize(raw).iter().map(|unit| unit.ch).collect()
}

fn is_html_whitespace(scalar: char) -> bool {
    matches!(
        scalar,
        '\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}'
    )
}

/// Inputs the model does not claim: preprocessing diagnostics for controls and
/// noncharacters are a separate accepted frontier.
fn is_unmodelled_input(ch: char) -> bool {
    let code = u32::from(ch);
    let control =
        (code <= 0x1f || (0x7f..=0x9f).contains(&code)) && !is_html_whitespace(ch) && ch != '\0';
    let noncharacter = (0xfdd0..=0xfdef).contains(&code) || (code & 0xfffe) == 0xfffe;
    control || noncharacter
}

// ---------------------------------------------------------------------------
// Diagnostics (test-local vocabulary)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Code {
    UnexpectedNullCharacter,
    NullCharacterReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    Data,
    TagName,
    AttributeValueDoubleQuoted,
    AttributeValueUnquoted,
    NumericReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Handling {
    /// The parse error is recorded and processing continues unchanged.
    Continued,
    /// The specification replaces the scalar with U+FFFD.
    ReplacedWithReplacementCharacter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    code: Code,
    context: Context,
    handling: Handling,
    /// The authored site: the NUL unit, or the whole reference.
    at: Evidence,
}

// ---------------------------------------------------------------------------
// Tokens and provenance
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Origin {
    /// Ordinary characters: interpreted text is the authored text after
    /// newline normalization.
    Literal,
    ResolvedNamed,
    ResolvedNumeric,
    /// Exactly one authored Data U+0000 scalar.
    AuthoredDataNull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Contribution {
    origin: Origin,
    authored: Evidence,
    interpreted: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagKind {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TagToken {
    kind: TagKind,
    name: String,
    attributes: Vec<(String, String)>,
    authored: Evidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Characters(Contribution),
    Tag(TagToken),
    EndOfFile,
}

type Projection = (Origin, usize, usize, String);

fn proj(origin: Origin, start: usize, end: usize, interpreted: &str) -> Projection {
    (origin, start, end, interpreted.to_owned())
}

fn lit(start: usize, end: usize, text: &str) -> Projection {
    proj(Origin::Literal, start, end, text)
}

fn nul(start: usize) -> Projection {
    proj(Origin::AuthoredDataNull, start, start + 1, "\0")
}

// ---------------------------------------------------------------------------
// Resource model (test-local; the typed meanings are the accepted ones)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
struct Limits {
    steps: Option<usize>,
    tokens: Option<usize>,
    diagnostics: Option<usize>,
    /// Cumulative interpreted bytes: emitted tokens plus the active builder.
    retained: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Resource {
    TransitionSteps,
    EmittedTokens,
    Diagnostics,
    RetainedInterpretedBytes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Refusal {
    resource: Resource,
    limit: usize,
    attempted: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Usage {
    steps: usize,
    tokens: usize,
    diagnostics: usize,
    retained: usize,
}

/// Boundaries the model refuses honestly instead of guessing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outside {
    /// Controls and noncharacters: a separate preprocessing frontier.
    InputPreprocessing,
    /// Any tag shape beyond the lowercase ASCII shapes the controls need.
    TagShape,
    EofInTag,
    /// A character reference inside an attribute value.
    AttributeValueReference,
    /// A reference other than `&amp;` or a decimal/hex form.
    ReferenceShape,
    /// A numeric reference value other than 0 or an ASCII graphic scalar.
    NumericValue,
    /// Authored RCDATA U+0000: the existing selected boundary.
    RcdataNul,
    /// Any RCDATA markup other than `</title>`.
    RcdataMarkup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    Outside(Outside),
    Resource(Refusal),
}

// ---------------------------------------------------------------------------
// Unit-driven tokenizer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Data,
    Rcdata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Lexed {
    source_len: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    usage: Usage,
    coverage_end: usize,
    /// Authored regions consumed or observed but never emitted (a pending run,
    /// tag, or reference, or a refused U+0000 token).
    abandoned: Vec<(usize, usize)>,
    stop: Option<Stop>,
    /// False once a reference or tag was processed: their step counts are a
    /// model approximation and are not asserted absolutely.
    steps_exact: bool,
}

impl Lexed {
    fn contributions(&self) -> Vec<&Contribution> {
        self.tokens
            .iter()
            .filter_map(|token| match token {
                Token::Characters(contribution) => Some(contribution),
                _ => None,
            })
            .collect()
    }

    fn projection(&self) -> Vec<Projection> {
        self.contributions()
            .iter()
            .map(|c| {
                (
                    c.origin,
                    c.authored.start,
                    c.authored.end,
                    c.interpreted.clone(),
                )
            })
            .collect()
    }

    fn interpreted(&self) -> String {
        self.contributions()
            .iter()
            .map(|c| c.interpreted.as_str())
            .collect()
    }

    fn diagnostic_view(&self) -> Vec<(Code, Context, Handling, usize, usize)> {
        self.diagnostics
            .iter()
            .map(|d| (d.code, d.context, d.handling, d.at.start, d.at.end))
            .collect()
    }

    fn reached_eof(&self) -> bool {
        self.tokens.last() == Some(&Token::EndOfFile)
    }

    fn count_origin(&self, origin: Origin) -> usize {
        self.contributions()
            .iter()
            .filter(|c| c.origin == origin)
            .count()
    }
}

struct Run {
    first: usize,
    text: String,
}

struct Machine {
    source: SourceText,
    units: Vec<Unit>,
    idx: usize,
    ctx: Ctx,
    model: Model,
    limits: Limits,
    usage: Usage,
    consumed_end: usize,
    coverage_end: usize,
    /// Unit index where the currently pending (unemitted) run, tag, or
    /// reference began.
    pending: Option<usize>,
    run: Option<Run>,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    abandoned: Vec<(usize, usize)>,
    steps_exact: bool,
}

fn lex(text: &str) -> Lexed {
    lex_with(Model::SOUND, text, Limits::default())
}

fn lex_with(model: Model, text: &str, limits: Limits) -> Lexed {
    let source = SourceText::new(SourceId::new(1), text.to_owned());
    let mut machine = Machine {
        units: normalize(source.as_str()),
        source,
        idx: 0,
        ctx: Ctx::Data,
        model,
        limits,
        usage: Usage::default(),
        consumed_end: 0,
        coverage_end: 0,
        pending: None,
        run: None,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
        abandoned: Vec::new(),
        steps_exact: true,
    };
    let stop = machine.run().err();
    if stop.is_some()
        && let Some(first) = machine.pending
    {
        let start = machine.units[first].start;
        machine.abandoned.push((start, machine.consumed_end));
    }
    Lexed {
        source_len: machine.source.as_str().len(),
        tokens: machine.tokens,
        diagnostics: machine.diagnostics,
        usage: machine.usage,
        coverage_end: machine.coverage_end,
        abandoned: machine.abandoned,
        stop,
        steps_exact: machine.steps_exact,
    }
}

impl Machine {
    fn check(
        resource: Resource,
        used: usize,
        add: usize,
        limit: Option<usize>,
    ) -> Result<(), Stop> {
        match limit {
            Some(limit) if used + add > limit => Err(Stop::Resource(Refusal {
                resource,
                limit,
                attempted: used + add,
            })),
            _ => Ok(()),
        }
    }

    /// Prepares every fallible effect of one semantic step before any of it is
    /// committed. Nothing is mutated here.
    fn preflight(&self, diagnostics: usize, tokens: usize, retained: usize) -> Result<(), Stop> {
        Self::check(
            Resource::Diagnostics,
            self.usage.diagnostics,
            diagnostics,
            self.limits.diagnostics,
        )?;
        Self::check(
            Resource::EmittedTokens,
            self.usage.tokens,
            tokens,
            self.limits.tokens,
        )?;
        Self::check(
            Resource::RetainedInterpretedBytes,
            self.usage.retained,
            retained,
            self.limits.retained,
        )
    }

    fn evidence_of(&self, unit: Unit) -> Evidence {
        evidence(&self.source, unit.start, unit.end)
    }

    /// A committed diagnostic is evidence that its authored site was observed
    /// and explained, so processed coverage reaches the end of that site.
    fn commit_diagnostic(&mut self, diagnostic: Diagnostic) {
        if !(self.model.is(Mutation::DiagnosticSiteNotCovered)
            && diagnostic.context == Context::Data)
        {
            self.coverage_end = self.coverage_end.max(diagnostic.at.end);
        }
        self.diagnostics.push(diagnostic);
        self.usage.diagnostics += 1;
    }

    fn commit_token(&mut self, token: Token) {
        self.tokens.push(token);
        self.usage.tokens += 1;
    }

    fn consume(&mut self, unit: Unit) {
        self.idx += 1;
        self.consumed_end = unit.end;
        self.coverage_end = self.coverage_end.max(unit.end);
    }

    /// One transition: examine the current unit (or conceptual EOF).
    fn dispatch(&mut self) -> Result<Option<Unit>, Stop> {
        let unit = self.units.get(self.idx).copied();
        if let Some(unit) = unit
            && is_unmodelled_input(unit.ch)
        {
            return Err(Stop::Outside(Outside::InputPreprocessing));
        }
        if let Some(limit) = self.limits.steps
            && self.usage.steps >= limit
        {
            return Err(Stop::Resource(Refusal {
                resource: Resource::TransitionSteps,
                limit,
                attempted: self.usage.steps + 1,
            }));
        }
        self.usage.steps += 1;
        Ok(unit)
    }

    fn next(&mut self) -> Result<Unit, Stop> {
        self.dispatch()?.ok_or(Stop::Outside(Outside::EofInTag))
    }

    fn ahead(&self, pattern: &str) -> bool {
        pattern.chars().enumerate().all(|(offset, expected)| {
            self.units.get(self.idx + offset).map(|u| u.ch) == Some(expected)
        })
    }

    fn push_run_unit(&mut self, unit: Unit) {
        if self.run.is_none() {
            self.pending = Some(self.idx);
            self.run = Some(Run {
                first: self.idx,
                text: String::new(),
            });
        }
        if let Some(run) = self.run.as_mut() {
            run.text.push(unit.ch);
        }
        self.consume(unit);
    }

    fn flush_run(&mut self) -> Result<(), Stop> {
        let Some(run) = self.run.as_ref() else {
            return Ok(());
        };
        self.preflight(0, 1, 0)?;
        let start = self.units[run.first].start;
        let text = run.text.clone();
        self.run = None;
        self.pending = None;
        let contribution = Contribution {
            origin: Origin::Literal,
            authored: evidence(&self.source, start, self.consumed_end),
            interpreted: text,
        };
        self.commit_token(Token::Characters(contribution));
        Ok(())
    }

    fn run(&mut self) -> Result<(), Stop> {
        loop {
            let Some(unit) = self.dispatch()? else {
                self.flush_run()?;
                self.preflight(0, 1, 0)?;
                self.commit_token(Token::EndOfFile);
                return Ok(());
            };
            match (self.ctx, unit.ch) {
                (Ctx::Data, '&') => {
                    self.flush_run()?;
                    self.reference(unit)?;
                }
                (_, '<') => {
                    self.flush_run()?;
                    if self.ctx == Ctx::Rcdata && !self.ahead("</title>") {
                        return Err(Stop::Outside(Outside::RcdataMarkup));
                    }
                    self.tag(unit)?;
                }
                (Ctx::Rcdata, '&') => {
                    self.flush_run()?;
                    return Err(Stop::Outside(Outside::ReferenceShape));
                }
                (Ctx::Data, '\0') => self.data_null(unit)?,
                (Ctx::Rcdata, '\0') => {
                    if self.model.is(Mutation::RcdataNulUsesDataRule) {
                        self.data_null(unit)?;
                    } else {
                        self.flush_run()?;
                        return Err(Stop::Outside(Outside::RcdataNul));
                    }
                }
                _ => {
                    self.reserve_retained(unit.ch.len_utf8())?;
                    self.push_run_unit(unit);
                }
            }
        }
    }

    fn reserve_retained(&mut self, bytes: usize) -> Result<(), Stop> {
        self.preflight(0, 0, bytes)?;
        self.usage.retained += bytes;
        Ok(())
    }

    /// Authored Data U+0000, the subject of the theorem.
    ///
    /// Pinned: `unexpected-null-character` parse error, then emit the current
    /// input character. The pending ordinary run is emitted first, so a
    /// refused prior-run emission leaves no `UnexpectedNullCharacter`.
    fn data_null(&mut self, unit: Unit) -> Result<(), Stop> {
        let model = self.model;
        if model.aggregates() {
            // Fault: the NUL is folded into the surrounding run.
            let at = self.evidence_of(unit);
            self.preflight(1, 0, 1)?;
            self.commit_diagnostic(Diagnostic {
                code: Code::UnexpectedNullCharacter,
                context: Context::Data,
                handling: Handling::Continued,
                at,
            });
            self.usage.retained += 1;
            self.push_run_unit(unit);
            return Ok(());
        }
        self.flush_run()?;
        if model.is(Mutation::NulCostsTwoTransitions) {
            self.usage.steps += 1;
        }
        let (interpreted, handling, bytes) = if model.is(Mutation::NulBecomesReplacement) {
            (
                "\u{fffd}",
                Handling::ReplacedWithReplacementCharacter,
                '\u{fffd}'.len_utf8(),
            )
        } else if model.is(Mutation::NulClaimsReplacementRecovery) {
            ("\0", Handling::ReplacedWithReplacementCharacter, 1)
        } else if model.is(Mutation::NulReservesReplacementBytes) {
            ("\0", Handling::Continued, '\u{fffd}'.len_utf8())
        } else {
            ("\0", Handling::Continued, 1)
        };
        let at = self.evidence_of(unit);
        let diagnostic = Diagnostic {
            code: Code::UnexpectedNullCharacter,
            context: Context::Data,
            handling,
            at: at.clone(),
        };
        let token = (!model.is(Mutation::DropNulToken)).then(|| {
            Token::Characters(Contribution {
                origin: Origin::AuthoredDataNull,
                authored: at,
                interpreted: interpreted.to_owned(),
            })
        });
        let token_count = usize::from(token.is_some());
        let token_bytes = if token.is_some() { bytes } else { 0 };
        if model.is(Mutation::NulEffectPreparedBeforeDiagnostic) {
            // Fault: the independent effect is prepared first.
            self.preflight(1, token_count, token_bytes)?;
            self.commit_diagnostic(diagnostic);
        } else {
            // Observation -> diagnostic commit -> independent token effect.
            self.preflight(1, 0, 0)?;
            self.commit_diagnostic(diagnostic);
            if let Err(stop) = self.preflight(0, token_count, token_bytes) {
                let suppress = match stop {
                    Stop::Resource(Refusal {
                        resource: Resource::EmittedTokens,
                        ..
                    }) => model.is(Mutation::SuppressDiagnosticOnTokenRefusal),
                    Stop::Resource(Refusal {
                        resource: Resource::RetainedInterpretedBytes,
                        ..
                    }) => model.is(Mutation::SuppressDiagnosticOnRetainedRefusal),
                    _ => false,
                };
                if suppress {
                    self.diagnostics.pop();
                    self.usage.diagnostics -= 1;
                    self.coverage_end = self.consumed_end;
                } else {
                    // The observed NUL is explained but never emitted.
                    self.abandoned.push((unit.start, unit.end));
                }
                return Err(stop);
            }
        }
        self.usage.retained += token_bytes;
        if let Some(token) = token {
            self.commit_token(token);
        }
        self.consume(unit);
        Ok(())
    }

    /// Replacement recovery for contexts that the Standard defines with U+FFFD
    /// (the existing behavior, kept as a negative control). Retained cost is
    /// reserved before the diagnostic claims the replacement.
    fn replace_null(&mut self, context: Context, unit: Unit) -> Result<char, Stop> {
        let keep = self.model.is(Mutation::TagNulKeepsNul);
        let (scalar, handling) = if keep {
            ('\0', Handling::Continued)
        } else {
            ('\u{fffd}', Handling::ReplacedWithReplacementCharacter)
        };
        self.preflight(1, 0, scalar.len_utf8())?;
        let at = self.evidence_of(unit);
        self.commit_diagnostic(Diagnostic {
            code: Code::UnexpectedNullCharacter,
            context,
            handling,
            at,
        });
        self.usage.retained += scalar.len_utf8();
        self.consume(unit);
        Ok(scalar)
    }

    /// Lowercase ASCII tags with optional double-quoted or unquoted attribute
    /// values; every other shape is an explicit boundary.
    fn tag(&mut self, lt: Unit) -> Result<(), Stop> {
        self.steps_exact = false;
        self.pending = Some(self.idx);
        self.consume(lt);
        let mut unit = self.next()?;
        let kind = if unit.ch == '/' {
            self.consume(unit);
            unit = self.next()?;
            TagKind::End
        } else {
            TagKind::Start
        };
        if !unit.ch.is_ascii_lowercase() {
            return Err(Stop::Outside(Outside::TagShape));
        }
        let mut name = String::new();
        loop {
            match unit.ch {
                'a'..='z' => {
                    self.reserve_retained(1)?;
                    name.push(unit.ch);
                    self.consume(unit);
                }
                '\0' => {
                    let scalar = self.replace_null(Context::TagName, unit)?;
                    name.push(scalar);
                }
                '>' | ' ' | '\t' | '\n' | '\u{c}' => break,
                _ => return Err(Stop::Outside(Outside::TagShape)),
            }
            unit = self.next()?;
        }
        let mut attributes = Vec::new();
        while is_html_whitespace(unit.ch) {
            if kind == TagKind::End {
                return Err(Stop::Outside(Outside::TagShape));
            }
            while is_html_whitespace(unit.ch) {
                self.consume(unit);
                unit = self.next()?;
            }
            if unit.ch == '>' {
                break;
            }
            let (attribute, last) = self.attribute(unit)?;
            attributes.push(attribute);
            unit = last;
        }
        if unit.ch != '>' {
            return Err(Stop::Outside(Outside::TagShape));
        }
        let start = self.units[self.pending.expect("tag pending")].start;
        self.preflight(0, 1, 0)?;
        self.consume(unit);
        let authored = evidence(&self.source, start, unit.end);
        self.commit_token(Token::Tag(TagToken {
            kind,
            name: name.clone(),
            attributes,
            authored,
        }));
        self.pending = None;
        if kind == TagKind::Start && name == "title" {
            self.ctx = Ctx::Rcdata;
        } else if kind == TagKind::End && name == "title" {
            self.ctx = Ctx::Data;
        }
        Ok(())
    }

    /// One `name=value` attribute; returns it with the first unit after it
    /// (a whitespace unit or `>`), not consumed.
    fn attribute(&mut self, first: Unit) -> Result<((String, String), Unit), Stop> {
        let mut unit = first;
        let mut name = String::new();
        while unit.ch.is_ascii_lowercase() {
            self.reserve_retained(1)?;
            name.push(unit.ch);
            self.consume(unit);
            unit = self.next()?;
        }
        if name.is_empty() || unit.ch != '=' {
            return Err(Stop::Outside(Outside::TagShape));
        }
        self.consume(unit);
        unit = self.next()?;
        let mut value = String::new();
        if unit.ch == '"' {
            self.consume(unit);
            loop {
                unit = self.next()?;
                match unit.ch {
                    '"' => break,
                    '&' => return Err(Stop::Outside(Outside::AttributeValueReference)),
                    '\0' => {
                        let scalar =
                            self.replace_null(Context::AttributeValueDoubleQuoted, unit)?;
                        value.push(scalar);
                    }
                    ch => {
                        self.reserve_retained(ch.len_utf8())?;
                        value.push(ch);
                        self.consume(unit);
                    }
                }
            }
            self.consume(unit);
            unit = self.next()?;
            if !(is_html_whitespace(unit.ch) || unit.ch == '>') {
                return Err(Stop::Outside(Outside::TagShape));
            }
        } else {
            if is_html_whitespace(unit.ch) || unit.ch == '>' {
                return Err(Stop::Outside(Outside::TagShape));
            }
            while !(is_html_whitespace(unit.ch) || unit.ch == '>') {
                match unit.ch {
                    '&' => return Err(Stop::Outside(Outside::AttributeValueReference)),
                    '"' | '\'' | '<' | '=' | '`' => {
                        return Err(Stop::Outside(Outside::TagShape));
                    }
                    '\0' => {
                        let scalar = self.replace_null(Context::AttributeValueUnquoted, unit)?;
                        value.push(scalar);
                    }
                    ch => {
                        self.reserve_retained(ch.len_utf8())?;
                        value.push(ch);
                        self.consume(unit);
                    }
                }
                unit = self.next()?;
            }
        }
        Ok(((name, value), unit))
    }

    /// `&amp;` and decimal/hex numeric references to 0 or an ASCII graphic
    /// scalar. Decoded output is emitted as a token and never re-enters input.
    fn reference(&mut self, amp: Unit) -> Result<(), Stop> {
        self.steps_exact = false;
        self.pending = Some(self.idx);
        self.consume(amp);
        let start = amp.start;
        if self.ahead("amp;") {
            for _ in 0..4 {
                let unit = self.next()?;
                self.consume(unit);
            }
            return self.emit_resolved(Origin::ResolvedNamed, start, "&", None);
        }
        if !self.ahead("#") {
            return Err(Stop::Outside(Outside::ReferenceShape));
        }
        let hash = self.next()?;
        self.consume(hash);
        let mut radix = 10;
        let mut unit = self.next()?;
        if matches!(unit.ch, 'x' | 'X') {
            radix = 16;
            self.consume(unit);
            unit = self.next()?;
        }
        let mut code: u32 = 0;
        let mut digits = 0usize;
        while let Some(digit) = unit.ch.to_digit(radix) {
            code = (code * radix + digit).min(0x11_0000);
            digits += 1;
            self.consume(unit);
            unit = self.next()?;
        }
        if digits == 0 || unit.ch != ';' {
            return Err(Stop::Outside(Outside::ReferenceShape));
        }
        self.consume(unit);
        let end = unit.end;
        match code {
            0 if self.model.is(Mutation::NumericZeroAsNul) => {
                let at = evidence(&self.source, start, end);
                let diagnostic = Diagnostic {
                    code: Code::UnexpectedNullCharacter,
                    context: Context::Data,
                    handling: Handling::Continued,
                    at,
                };
                self.emit_resolved(Origin::ResolvedNumeric, start, "\0", Some(diagnostic))
            }
            0 => {
                let at = evidence(&self.source, start, end);
                let diagnostic = Diagnostic {
                    code: Code::NullCharacterReference,
                    context: Context::NumericReference,
                    handling: Handling::ReplacedWithReplacementCharacter,
                    at,
                };
                self.emit_resolved(Origin::ResolvedNumeric, start, "\u{fffd}", Some(diagnostic))
            }
            0x21..=0x7e => {
                let scalar = char::from_u32(code).expect("ascii graphic");
                self.emit_resolved(Origin::ResolvedNumeric, start, &scalar.to_string(), None)
            }
            _ => Err(Stop::Outside(Outside::NumericValue)),
        }
    }

    fn emit_resolved(
        &mut self,
        origin: Origin,
        start: usize,
        interpreted: &str,
        diagnostic: Option<Diagnostic>,
    ) -> Result<(), Stop> {
        self.preflight(usize::from(diagnostic.is_some()), 1, interpreted.len())?;
        if let Some(diagnostic) = diagnostic {
            self.commit_diagnostic(diagnostic);
        }
        self.usage.retained += interpreted.len();
        let authored = evidence(&self.source, start, self.consumed_end);
        self.commit_token(Token::Characters(Contribution {
            origin,
            authored,
            interpreted: interpreted.to_owned(),
        }));
        self.pending = None;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Bounded tree over the existing selected document-tree positions
// ---------------------------------------------------------------------------

/// The existing selected positions only. No insertion mode is introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    TitleText,
    InBody,
    AfterBody,
    AfterAfterBody,
}

const EARLY_CHAIN: [Mode; 6] = [
    Mode::Initial,
    Mode::BeforeHtml,
    Mode::BeforeHead,
    Mode::InHead,
    Mode::AfterHead,
    Mode::InBody,
];

/// Tree-owned evidence, distinct from every tokenizer diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TreeEvent {
    /// After body saw a non-whitespace character token, switched to In body,
    /// and reprocessed the same token.
    AfterBodyNonWhitespaceToInBody { token: usize },
    /// In body received an exact U+0000 character token: parse error, ignore.
    InBodyIgnoredNull { token: usize, authored: Evidence },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextNode {
    text: String,
    contributions: Vec<Contribution>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Title {
    start: Evidence,
    node: Option<TextNode>,
    close: Option<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Body {
    /// `None` when the body was synthesized by the existing shell rules.
    start: Option<Evidence>,
    text_nodes: Vec<TextNode>,
    close: Option<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tree {
    shell_synthesized: bool,
    missing_doctype: bool,
    titles: Vec<Title>,
    body: Option<Body>,
    html_close: Option<Evidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TreeBoundary {
    /// Character data containing whitespace in an early or mixed position.
    WhitespaceSensitiveCharacterData,
    /// The existing After-after-body boundary: not widened by this leaf.
    CharacterDataInAfterAfterBody,
    EofInTitleText,
    OutsideModelledCells,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Unsupported {
    Outside(Outside),
    Tree(TreeBoundary),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Completion {
    Complete,
    ResourceLimit(Refusal),
    Unsupported(Unsupported),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Observation {
    lexed: Lexed,
    tree: Tree,
    mode: Mode,
    chain: Vec<Mode>,
    events: Vec<TreeEvent>,
    completion: Completion,
}

struct TreeState {
    tree: Tree,
    mode: Mode,
    chain: Vec<Mode>,
    events: Vec<TreeEvent>,
}

fn is_early(mode: Mode) -> bool {
    matches!(
        mode,
        Mode::Initial | Mode::BeforeHtml | Mode::BeforeHead | Mode::InHead | Mode::AfterHead
    )
}

impl TreeState {
    /// Existing shell rules: every early mode reprocesses forward until the
    /// body exists. A missing DOCTYPE is recorded once, leaving Initial.
    fn enter_body(&mut self, authored_start: Option<Evidence>) {
        if let Some(position) = EARLY_CHAIN.iter().position(|mode| *mode == self.mode) {
            if self.mode == Mode::Initial {
                self.tree.missing_doctype = true;
            }
            self.chain.extend_from_slice(&EARLY_CHAIN[position + 1..]);
        }
        self.tree.shell_synthesized = true;
        self.tree.body = Some(Body {
            start: authored_start,
            text_nodes: Vec::new(),
            close: None,
        });
        self.mode = Mode::InBody;
    }

    fn insert_body(&mut self, contribution: &Contribution) {
        let body = self.tree.body.as_mut().expect("body");
        match body.text_nodes.last_mut() {
            Some(node) => {
                node.text.push_str(&contribution.interpreted);
                node.contributions.push(contribution.clone());
            }
            None => body.text_nodes.push(TextNode {
                text: contribution.interpreted.clone(),
                contributions: vec![contribution.clone()],
            }),
        }
    }

    fn in_body_characters(&mut self, model: Model, index: usize, c: &Contribution) {
        if c.interpreted == "\0" {
            if model.is(Mutation::TreeInsertsNul) {
                self.insert_body(c);
            } else if model.is(Mutation::TreeInsertsReplacement) {
                let mut fabricated = c.clone();
                fabricated.interpreted = "\u{fffd}".to_owned();
                self.insert_body(&fabricated);
            } else if model.is(Mutation::TreeRetainsNulContribution) {
                let body = self.tree.body.as_mut().expect("body");
                match body.text_nodes.last_mut() {
                    Some(node) => node.contributions.push(c.clone()),
                    None => body.text_nodes.push(TextNode {
                        text: String::new(),
                        contributions: vec![c.clone()],
                    }),
                }
                self.events.push(TreeEvent::InBodyIgnoredNull {
                    token: index,
                    authored: c.authored.clone(),
                });
            } else {
                if !model.is(Mutation::TreeRecordsNoEvidence) {
                    self.events.push(TreeEvent::InBodyIgnoredNull {
                        token: index,
                        authored: c.authored.clone(),
                    });
                }
            }
            return;
        }
        if c.interpreted.contains('\0') && c.interpreted.len() > 1 {
            if model.is(Mutation::AggregateWholeAnchor) {
                let mut whole = c.clone();
                whole.interpreted = c.interpreted.replace('\0', "");
                self.insert_body(&whole);
                return;
            }
            if model.is(Mutation::AggregateSplitByInterpretedOffsets) {
                let source = SourceText::new(c.authored.source_id, {
                    // Re-anchoring needs the source; the fault rebuilds it
                    // from the aggregate's own raw bytes.
                    c.authored.raw.clone()
                });
                let mut offset = 0usize;
                for scalar in c.interpreted.chars() {
                    let len = scalar.len_utf8();
                    if scalar != '\0' {
                        let piece = Contribution {
                            origin: Origin::Literal,
                            authored: {
                                let mut at = evidence(&source, offset, offset + len);
                                at.start += c.authored.start;
                                at.end += c.authored.start;
                                at
                            },
                            interpreted: scalar.to_string(),
                        };
                        self.insert_body(&piece);
                    }
                    offset += len;
                }
                return;
            }
        }
        self.insert_body(c);
    }
}

fn process(
    state: &mut TreeState,
    model: Model,
    index: usize,
    token: &Token,
) -> Result<(), TreeBoundary> {
    loop {
        match (state.mode, token) {
            (mode, Token::Characters(c)) if is_early(mode) => {
                if c.interpreted.chars().any(is_html_whitespace) {
                    return Err(TreeBoundary::WhitespaceSensitiveCharacterData);
                }
                state.enter_body(None);
            }
            (mode, Token::Tag(tag))
                if is_early(mode) && tag.kind == TagKind::Start && tag.name == "body" =>
            {
                state.enter_body(Some(tag.authored.clone()));
                return Ok(());
            }
            (mode, Token::Tag(tag))
                if is_early(mode) && tag.kind == TagKind::Start && tag.name == "title" =>
            {
                state.tree.shell_synthesized = true;
                state.tree.titles.push(Title {
                    start: tag.authored.clone(),
                    node: None,
                    close: None,
                });
                state.mode = Mode::TitleText;
                return Ok(());
            }
            (mode, Token::EndOfFile) if is_early(mode) => return Ok(()),
            (Mode::TitleText, Token::Characters(c)) => {
                let title = state.tree.titles.last_mut().expect("title");
                match title.node.as_mut() {
                    Some(node) => {
                        node.text.push_str(&c.interpreted);
                        node.contributions.push(c.clone());
                    }
                    None => {
                        title.node = Some(TextNode {
                            text: c.interpreted.clone(),
                            contributions: vec![c.clone()],
                        });
                    }
                }
                return Ok(());
            }
            (Mode::TitleText, Token::Tag(tag))
                if tag.kind == TagKind::End && tag.name == "title" =>
            {
                state.tree.titles.last_mut().expect("title").close = Some(tag.authored.clone());
                state.mode = Mode::InHead;
                return Ok(());
            }
            (Mode::TitleText, Token::EndOfFile) => return Err(TreeBoundary::EofInTitleText),
            (Mode::InBody, Token::Characters(c)) => {
                state.in_body_characters(model, index, c);
                return Ok(());
            }
            (Mode::InBody, Token::Tag(tag)) if tag.kind == TagKind::End && tag.name == "body" => {
                state.tree.body.as_mut().expect("body").close = Some(tag.authored.clone());
                state.mode = Mode::AfterBody;
                return Ok(());
            }
            (Mode::InBody | Mode::AfterBody, Token::EndOfFile) => return Ok(()),
            (Mode::AfterBody, Token::Characters(c)) => {
                let whitespace = c
                    .interpreted
                    .chars()
                    .filter(|s| is_html_whitespace(*s))
                    .count();
                if whitespace == c.interpreted.chars().count() {
                    state.insert_body(c);
                    return Ok(());
                }
                if whitespace != 0 {
                    return Err(TreeBoundary::WhitespaceSensitiveCharacterData);
                }
                if model.is(Mutation::AfterBodyNoReprocess) {
                    state.insert_body(c);
                    return Ok(());
                }
                state
                    .events
                    .push(TreeEvent::AfterBodyNonWhitespaceToInBody { token: index });
                state.mode = Mode::InBody;
            }
            (Mode::AfterBody, Token::Tag(tag))
                if tag.kind == TagKind::End && tag.name == "html" =>
            {
                state.tree.html_close = Some(tag.authored.clone());
                state.mode = Mode::AfterAfterBody;
                return Ok(());
            }
            (Mode::AfterAfterBody, Token::EndOfFile) => return Ok(()),
            (Mode::AfterAfterBody, Token::Characters(c)) => {
                if model.is(Mutation::AfterAfterBodyWidened) {
                    state.mode = Mode::InBody;
                    continue;
                }
                let _ = c;
                return Err(TreeBoundary::CharacterDataInAfterAfterBody);
            }
            _ => return Err(TreeBoundary::OutsideModelledCells),
        }
    }
}

fn construct(model: Model, lexed: Lexed) -> Observation {
    let mut state = TreeState {
        tree: Tree {
            shell_synthesized: false,
            missing_doctype: false,
            titles: Vec::new(),
            body: None,
            html_close: None,
        },
        mode: Mode::Initial,
        chain: Vec::new(),
        events: Vec::new(),
    };
    let mut refusal = None;
    for (index, token) in lexed.tokens.iter().enumerate() {
        if let Err(boundary) = process(&mut state, model, index, token) {
            refusal = Some(boundary);
            break;
        }
    }
    let completion = match (refusal, lexed.stop) {
        (Some(boundary), _) => Completion::Unsupported(Unsupported::Tree(boundary)),
        (None, Some(Stop::Outside(outside))) => {
            Completion::Unsupported(Unsupported::Outside(outside))
        }
        (None, Some(Stop::Resource(_))) if model.is(Mutation::UpgradeIncompleteToComplete) => {
            Completion::Complete
        }
        (None, Some(Stop::Resource(refusal))) => Completion::ResourceLimit(refusal),
        (None, None) => Completion::Complete,
    };
    Observation {
        lexed,
        tree: state.tree,
        mode: state.mode,
        chain: state.chain,
        events: state.events,
        completion,
    }
}

fn observe(model: Model, text: &str) -> Observation {
    construct(model, lex_with(model, text, Limits::default()))
}

fn observe_limited(model: Model, text: &str, limits: Limits) -> Observation {
    construct(model, lex_with(model, text, limits))
}

impl Observation {
    fn body_text(&self) -> String {
        self.tree
            .body
            .as_ref()
            .map(|body| body.text_nodes.iter().map(|n| n.text.as_str()).collect())
            .unwrap_or_default()
    }

    fn body_text_node_count(&self) -> usize {
        self.tree
            .body
            .as_ref()
            .map_or(0, |body| body.text_nodes.len())
    }

    fn body_contributions(&self) -> Vec<Projection> {
        self.tree
            .body
            .iter()
            .flat_map(|body| body.text_nodes.iter())
            .flat_map(|node| node.contributions.iter())
            .map(|c| {
                (
                    c.origin,
                    c.authored.start,
                    c.authored.end,
                    c.interpreted.clone(),
                )
            })
            .collect()
    }

    fn ignored(&self) -> Vec<(usize, usize, usize)> {
        self.events
            .iter()
            .filter_map(|event| match event {
                TreeEvent::InBodyIgnoredNull { token, authored } => {
                    Some((*token, authored.start, authored.end))
                }
                TreeEvent::AfterBodyNonWhitespaceToInBody { .. } => None,
            })
            .collect()
    }

    fn after_body_recoveries(&self) -> Vec<usize> {
        self.events
            .iter()
            .filter_map(|event| match event {
                TreeEvent::AfterBodyNonWhitespaceToInBody { token } => Some(*token),
                TreeEvent::InBodyIgnoredNull { .. } => None,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Independent invariants (hand-authored properties, never production output)
// ---------------------------------------------------------------------------

/// Properties every lexer observation must satisfy, whatever the limits.
fn check_lexed(text: &str, lexed: &Lexed) {
    let len = text.len();
    assert_eq!(lexed.source_len, len);
    assert!(lexed.coverage_end <= len, "coverage past source: {text:?}");

    // Authored evidence is exact and ordered; no token mixes U+0000 with any
    // other scalar; an authored Data NUL token is exactly one source byte.
    let mut previous_end = 0usize;
    for token in &lexed.tokens {
        let authored = match token {
            Token::Characters(c) => &c.authored,
            Token::Tag(t) => &t.authored,
            Token::EndOfFile => continue,
        };
        assert!(
            authored.start >= previous_end,
            "overlapping tokens: {text:?}"
        );
        assert!(authored.end <= len && authored.start < authored.end);
        assert_eq!(authored.raw, text[authored.start..authored.end]);
        previous_end = authored.end;
        if let Token::Characters(c) = token {
            if c.interpreted.contains('\0') {
                assert_eq!(c.interpreted, "\0", "mixed NUL aggregate: {text:?}");
                assert_eq!(c.origin, Origin::AuthoredDataNull);
            }
            if c.origin == Origin::AuthoredDataNull {
                assert_eq!(c.authored.raw, "\0");
                assert_eq!(c.interpreted, "\0");
                assert_eq!(c.authored.end - c.authored.start, 1);
            }
            if c.origin == Origin::Literal {
                assert_eq!(normalized_text(&c.authored.raw), c.interpreted);
            }
        }
    }

    // A Data-context diagnostic never claims replacement; it points at one
    // authored NUL byte; and each NUL token has its diagnostic.
    let data_nul_diagnostics: Vec<&Diagnostic> = lexed
        .diagnostics
        .iter()
        .filter(|d| d.context == Context::Data)
        .collect();
    for d in &data_nul_diagnostics {
        assert_eq!(d.code, Code::UnexpectedNullCharacter);
        assert_eq!(
            d.handling,
            Handling::Continued,
            "false replacement claim: {text:?}"
        );
        assert_eq!(d.at.raw, "\0");
    }
    // Single lifecycle: every NUL token has its diagnostic at the same site;
    // at most one observed NUL lacks a token, and then the run was stopped by
    // a token/retained refusal and the NUL is an explicit abandoned region.
    for token in lexed.contributions() {
        if token.origin == Origin::AuthoredDataNull {
            assert!(
                data_nul_diagnostics
                    .iter()
                    .any(|d| (d.at.start, d.at.end) == (token.authored.start, token.authored.end)),
                "NUL token without its diagnostic: {text:?}"
            );
        }
    }
    let unemitted: Vec<&&Diagnostic> = data_nul_diagnostics
        .iter()
        .filter(|d| {
            !lexed
                .contributions()
                .iter()
                .any(|t| t.origin == Origin::AuthoredDataNull && t.authored.start == d.at.start)
        })
        .collect();
    assert!(unemitted.len() <= 1, "diagnostic/token mismatch: {text:?}");
    for d in unemitted {
        assert!(
            lexed.abandoned.contains(&(d.at.start, d.at.end)),
            "observed NUL not explained: {text:?}"
        );
        assert!(matches!(
            lexed.stop,
            Some(Stop::Resource(Refusal {
                resource: Resource::EmittedTokens | Resource::RetainedInterpretedBytes,
                ..
            }))
        ));
    }
    // An unemitted authored NUL is never silent: it carries its diagnostic.
    for &(start, end) in &lexed.abandoned {
        if end - start == 1 && text.as_bytes()[start] == 0 {
            assert!(
                data_nul_diagnostics.iter().any(|d| d.at.start == start),
                "abandoned NUL lost its diagnostic: {text:?}"
            );
        }
    }
    // Every committed diagnostic is inside processed coverage.
    for d in &lexed.diagnostics {
        assert!(
            d.at.end <= lexed.coverage_end,
            "diagnostic outside coverage: {text:?}"
        );
    }
    // No character token without retained evidence for its interpreted bytes.
    let token_bytes: usize = lexed
        .contributions()
        .iter()
        .map(|c| c.interpreted.len())
        .sum();
    assert!(
        lexed.usage.retained >= token_bytes,
        "token without retention: {text:?}"
    );

    // Coverage never outruns evidence: every processed byte is inside an
    // emitted token, a committed diagnostic site, or an explicit abandoned
    // region.
    for byte in 0..lexed.coverage_end {
        let in_token = lexed.tokens.iter().any(|token| match token {
            Token::Characters(c) => c.authored.start <= byte && byte < c.authored.end,
            Token::Tag(t) => t.authored.start <= byte && byte < t.authored.end,
            Token::EndOfFile => false,
        });
        let in_diagnostic = lexed
            .diagnostics
            .iter()
            .any(|d| d.at.start <= byte && byte < d.at.end);
        let in_abandoned = lexed.abandoned.iter().any(|(s, e)| *s <= byte && byte < *e);
        assert!(
            in_token || in_diagnostic || in_abandoned,
            "byte {byte} of {text:?} processed without evidence"
        );
    }

    // Completion meaning.
    match lexed.stop {
        None => {
            assert!(lexed.reached_eof());
            assert_eq!(lexed.coverage_end, len);
            assert!(lexed.abandoned.is_empty());
        }
        Some(_) => assert!(!lexed.reached_eof()),
    }
    if let Some(Stop::Resource(refusal)) = lexed.stop {
        assert!(refusal.attempted > refusal.limit);
    }
    assert_eq!(lexed.usage.tokens, lexed.tokens.len());
    assert_eq!(lexed.usage.diagnostics, lexed.diagnostics.len());
}

/// Properties every tree observation must satisfy.
fn check_tree(text: &str, obs: &Observation) {
    check_lexed(text, &obs.lexed);
    let tokens = &obs.lexed.tokens;

    // The final tree has no U+0000 text and no NUL contribution.
    let mut placed: Vec<&Contribution> = Vec::new();
    if let Some(body) = &obs.tree.body {
        for node in &body.text_nodes {
            assert!(!node.text.contains('\0'), "NUL text in tree: {text:?}");
            let joined: String = node
                .contributions
                .iter()
                .map(|c| c.interpreted.as_str())
                .collect();
            assert_eq!(node.text, joined, "text/contribution mismatch: {text:?}");
            assert!(!node.contributions.is_empty() && !node.text.is_empty());
            placed.extend(node.contributions.iter());
        }
    }
    for title in &obs.tree.titles {
        if let Some(node) = &title.node {
            assert!(!node.text.contains('\0'));
            placed.extend(node.contributions.iter());
        }
    }
    for c in &placed {
        assert_ne!(
            c.origin,
            Origin::AuthoredDataNull,
            "NUL contribution placed"
        );
        assert!(
            !c.authored.raw.contains('\0'),
            "NUL-bearing contribution anchor"
        );
        assert!(!c.interpreted.contains('\0'));
    }

    // Placed contributions are an ordered subsequence of token contributions.
    let mut cursor = tokens.iter();
    for c in &placed {
        assert!(
            cursor.any(|token| matches!(token, Token::Characters(t) if t == *c)),
            "placed contribution is not an ordered token contribution: {text:?}"
        );
    }
    // A fabricated U+FFFD is only legitimate when a reference resolved to it.
    for c in &placed {
        if c.interpreted.contains('\u{fffd}') {
            assert!(
                c.origin == Origin::ResolvedNumeric || c.origin == Origin::ResolvedNamed,
                "fabricated U+FFFD: {text:?}"
            );
        }
    }

    // Tree evidence: ignored-NUL events point at exact U+0000 tokens; every
    // exact NUL token the tree consumed has exactly one such event.
    for (token, start, end) in obs.ignored() {
        match &tokens[token] {
            Token::Characters(c) => {
                assert_eq!(c.interpreted, "\0");
                assert_eq!((c.authored.start, c.authored.end), (start, end));
            }
            other => panic!("ignored event on {other:?}"),
        }
    }
    if obs.completion == Completion::Complete {
        assert_eq!(
            obs.ignored().len(),
            obs.lexed.count_origin(Origin::AuthoredDataNull)
        );
        assert!(obs.lexed.reached_eof());
    }
    if matches!(obs.completion, Completion::ResourceLimit(_)) {
        assert!(!obs.lexed.reached_eof());
    }
}

// ---------------------------------------------------------------------------
// Theorem groups. Each is a plain function over a Model, so the sound model
// must pass all of them and every mutation must fail at least one.
// ---------------------------------------------------------------------------

/// T-group: tokenizer meaning of authored Data U+0000.
fn g_tokenizer_nul(m: Model) {
    let unexpected = |start: usize| {
        (
            Code::UnexpectedNullCharacter,
            Context::Data,
            Handling::Continued,
            start,
            start + 1,
        )
    };

    // `\0`
    let l = lex_with(m, "\0", Limits::default());
    assert_eq!(l.projection(), vec![nul(0)]);
    assert_eq!(l.diagnostic_view(), vec![unexpected(0)]);
    assert_eq!(l.interpreted(), "\0");
    assert_eq!(l.count_origin(Origin::AuthoredDataNull), 1);
    assert!(l.reached_eof() && l.stop.is_none());
    assert_eq!(l.coverage_end, 1);
    check_lexed("\0", &l);

    // `a\0b`: surviving ordinary data around exact U+0000 evidence.
    let l = lex_with(m, "a\0b", Limits::default());
    assert_eq!(l.projection(), vec![lit(0, 1, "a"), nul(1), lit(2, 3, "b")]);
    assert_eq!(l.diagnostic_view(), vec![unexpected(1)]);
    assert!(!l.interpreted().contains('\u{fffd}'));
    check_lexed("a\0b", &l);

    // Dense NUL: one diagnostic and one separate token per authored scalar.
    let l = lex_with(m, "\0\0\0", Limits::default());
    assert_eq!(l.projection(), vec![nul(0), nul(1), nul(2)]);
    assert_eq!(
        l.diagnostic_view(),
        vec![unexpected(0), unexpected(1), unexpected(2)]
    );
    check_lexed("\0\0\0", &l);

    // Leading and trailing ordinary data.
    let l = lex_with(m, "\0a", Limits::default());
    assert_eq!(l.projection(), vec![nul(0), lit(1, 2, "a")]);
    let l = lex_with(m, "a\0", Limits::default());
    assert_eq!(l.projection(), vec![lit(0, 1, "a"), nul(1)]);
}

/// R-group: references are distinct from authored NUL and never re-enter input.
fn g_references(m: Model) {
    let unexpected = |start: usize| {
        (
            Code::UnexpectedNullCharacter,
            Context::Data,
            Handling::Continued,
            start,
            start + 1,
        )
    };

    // Named resolution stays distinct from the adjacent authored NUL.
    let l = lex_with(m, "&amp;\0b", Limits::default());
    assert_eq!(
        l.projection(),
        vec![
            proj(Origin::ResolvedNamed, 0, 5, "&"),
            nul(5),
            lit(6, 7, "b")
        ]
    );
    assert_eq!(l.diagnostic_view(), vec![unexpected(5)]);
    check_lexed("&amp;\0b", &l);

    // Numeric resolution likewise.
    let l = lex_with(m, "&#65;\0b", Limits::default());
    assert_eq!(
        l.projection(),
        vec![
            proj(Origin::ResolvedNumeric, 0, 5, "A"),
            nul(5),
            lit(6, 7, "b")
        ]
    );
    assert_eq!(l.diagnostic_view(), vec![unexpected(5)]);

    // Numeric zero: U+FFFD + NullCharacterReference, never authored NUL.
    for (text, end) in [("&#0;", 4), ("&#x0;", 5)] {
        let l = lex_with(m, text, Limits::default());
        assert_eq!(
            l.projection(),
            vec![proj(Origin::ResolvedNumeric, 0, end, "\u{fffd}")],
            "{text}"
        );
        assert_eq!(
            l.diagnostic_view(),
            vec![(
                Code::NullCharacterReference,
                Context::NumericReference,
                Handling::ReplacedWithReplacementCharacter,
                0,
                end
            )],
            "{text}"
        );
        assert_eq!(l.count_origin(Origin::AuthoredDataNull), 0);
        assert!(!l.interpreted().contains('\0'));
        check_lexed(text, &l);
    }

    // Numeric zero immediately followed by an authored NUL: two distinct
    // objects, two distinct diagnostics, in authored order.
    let l = lex_with(m, "&#0;\0", Limits::default());
    assert_eq!(
        l.projection(),
        vec![proj(Origin::ResolvedNumeric, 0, 4, "\u{fffd}"), nul(4)]
    );
    assert_eq!(
        l.diagnostic_view(),
        vec![
            (
                Code::NullCharacterReference,
                Context::NumericReference,
                Handling::ReplacedWithReplacementCharacter,
                0,
                4
            ),
            unexpected(4)
        ]
    );
    check_lexed("&#0;\0", &l);
}

/// N-group: non-Data NUL contexts keep U+FFFD replacement.
fn g_non_data(m: Model) {
    let replaced = |context: Context, start: usize| {
        (
            Code::UnexpectedNullCharacter,
            context,
            Handling::ReplacedWithReplacementCharacter,
            start,
            start + 1,
        )
    };

    let l = lex_with(m, "<a\0>", Limits::default());
    assert_eq!(l.diagnostic_view(), vec![replaced(Context::TagName, 2)]);
    match l.tokens.first() {
        Some(Token::Tag(tag)) => {
            assert_eq!(tag.name, "a\u{fffd}");
            assert_eq!((tag.authored.start, tag.authored.end), (0, 4));
        }
        other => panic!("expected a start tag, got {other:?}"),
    }
    assert_eq!(l.count_origin(Origin::AuthoredDataNull), 0);
    check_lexed("<a\0>", &l);

    let l = lex_with(m, "<a x=\"\0\">", Limits::default());
    assert_eq!(
        l.diagnostic_view(),
        vec![replaced(Context::AttributeValueDoubleQuoted, 6)]
    );
    match l.tokens.first() {
        Some(Token::Tag(tag)) => {
            assert_eq!(
                tag.attributes,
                vec![("x".to_owned(), "\u{fffd}".to_owned())]
            );
        }
        other => panic!("expected a start tag, got {other:?}"),
    }
    check_lexed("<a x=\"\0\">", &l);

    let l = lex_with(m, "<a x=\0>", Limits::default());
    assert_eq!(
        l.diagnostic_view(),
        vec![replaced(Context::AttributeValueUnquoted, 5)]
    );
    match l.tokens.first() {
        Some(Token::Tag(tag)) => {
            assert_eq!(
                tag.attributes,
                vec![("x".to_owned(), "\u{fffd}".to_owned())]
            );
        }
        other => panic!("expected a start tag, got {other:?}"),
    }
    check_lexed("<a x=\0>", &l);

    // Selected Title/RCDATA authored NUL stays at the existing boundary: the
    // title start tag is emitted, no NUL evidence, no replacement claim, and
    // the leaf does not implement RCDATA recovery.
    let l = lex_with(m, "<title>\0</title>", Limits::default());
    assert_eq!(l.stop, Some(Stop::Outside(Outside::RcdataNul)));
    assert_eq!(l.tokens.len(), 1);
    assert!(matches!(&l.tokens[0], Token::Tag(t) if t.name == "title"));
    assert!(l.diagnostics.is_empty());
    assert_eq!(l.count_origin(Origin::AuthoredDataNull), 0);
    check_lexed("<title>\0</title>", &l);

    // The Data rule applies again after the title closes.
    let l = lex_with(m, "<title>a</title>\0", Limits::default());
    assert_eq!(l.stop, None);
    assert_eq!(l.count_origin(Origin::AuthoredDataNull), 1);
}

/// B-group: the selected tree theorem.
fn g_tree(m: Model) {
    let ignored = |token: usize, start: usize| (token, start, start + 1);

    // `\0` alone: the early shell reprocesses forward to In body, which ignores.
    let o = observe(m, "\0");
    assert_eq!(o.completion, Completion::Complete);
    assert_eq!(
        o.chain,
        vec![
            Mode::BeforeHtml,
            Mode::BeforeHead,
            Mode::InHead,
            Mode::AfterHead,
            Mode::InBody
        ]
    );
    assert!(o.tree.missing_doctype && o.tree.shell_synthesized);
    assert_eq!(o.tree.body.as_ref().map(|b| b.start.clone()), Some(None));
    assert_eq!(o.body_text(), "");
    assert_eq!(o.body_text_node_count(), 0);
    assert_eq!(o.ignored(), vec![ignored(0, 0)]);
    assert_eq!(o.mode, Mode::InBody);
    check_tree("\0", &o);

    // `<body>\0</body>`: no text node authored by the NUL.
    let text = "<body>\0</body>";
    let o = observe(m, text);
    assert_eq!(o.completion, Completion::Complete);
    assert_eq!(o.body_text_node_count(), 0);
    assert_eq!(o.body_text(), "");
    assert_eq!(o.ignored(), vec![ignored(1, 6)]);
    assert!(o.after_body_recoveries().is_empty());
    assert_eq!(o.mode, Mode::AfterBody);
    assert_eq!(
        o.tree
            .body
            .as_ref()
            .and_then(|b| b.close.as_ref())
            .map(|e| (e.start, e.end)),
        Some((7, 14))
    );
    check_tree(text, &o);

    // `<body>a\0b</body>`: final text `ab`, honest ordered contributions.
    let text = "<body>a\0b</body>";
    let o = observe(m, text);
    assert_eq!(o.completion, Completion::Complete);
    assert_eq!(o.body_text(), "ab");
    assert_eq!(o.body_text_node_count(), 1);
    assert_eq!(o.body_contributions(), vec![lit(6, 7, "a"), lit(8, 9, "b")]);
    assert_eq!(o.ignored(), vec![ignored(2, 7)]);
    check_tree(text, &o);

    // `<body></body>\0`: AfterBody recovery, then In body ignores.
    let text = "<body></body>\0";
    let o = observe(m, text);
    assert_eq!(o.completion, Completion::Complete);
    assert_eq!(o.body_text_node_count(), 0);
    assert_eq!(o.after_body_recoveries(), vec![2]);
    assert_eq!(o.ignored(), vec![ignored(2, 13)]);
    assert_eq!(
        o.events,
        vec![
            TreeEvent::AfterBodyNonWhitespaceToInBody { token: 2 },
            TreeEvent::InBodyIgnoredNull {
                token: 2,
                authored: o.events_authored(1),
            },
        ]
    );
    assert_eq!(o.mode, Mode::InBody);
    check_tree(text, &o);

    // After body: surrounding data recovers once; the NUL never re-triggers it.
    let text = "<body></body>a\0b";
    let o = observe(m, text);
    assert_eq!(o.completion, Completion::Complete);
    assert_eq!(o.body_text(), "ab");
    assert_eq!(
        o.body_contributions(),
        vec![lit(13, 14, "a"), lit(15, 16, "b")]
    );
    assert_eq!(o.after_body_recoveries(), vec![2]);
    assert_eq!(o.ignored(), vec![ignored(3, 14)]);
    check_tree(text, &o);

    // CRLF before the NUL: authored and interpreted lengths differ.
    let text = "<body>a\r\n\0b</body>";
    let o = observe(m, text);
    assert_eq!(o.body_text(), "a\nb");
    assert_eq!(
        o.body_contributions(),
        vec![lit(6, 9, "a\n"), lit(10, 11, "b")]
    );
    assert_eq!(o.ignored(), vec![ignored(2, 9)]);
    check_tree(text, &o);

    // A resolved Named reference beside the NUL.
    let text = "<body>&amp;\0b</body>";
    let o = observe(m, text);
    assert_eq!(o.body_text(), "&b");
    assert_eq!(
        o.body_contributions(),
        vec![proj(Origin::ResolvedNamed, 6, 11, "&"), lit(12, 13, "b")]
    );
    assert_eq!(o.ignored(), vec![ignored(2, 11)]);
    check_tree(text, &o);

    // Dense NUL.
    let text = "<body>\0\0</body>";
    let o = observe(m, text);
    assert_eq!(o.body_text_node_count(), 0);
    assert_eq!(o.ignored(), vec![ignored(1, 6), ignored(2, 7)]);
    check_tree(text, &o);

    // Numeric zero is U+FFFD text: the tree ignores only an exact NUL token.
    let text = "<body>&#0;</body>";
    let o = observe(m, text);
    assert_eq!(o.body_text(), "\u{fffd}");
    assert_eq!(
        o.body_contributions(),
        vec![proj(Origin::ResolvedNumeric, 6, 10, "\u{fffd}")]
    );
    assert!(o.ignored().is_empty());
    check_tree(text, &o);
}

impl Observation {
    /// The authored evidence of the `index`-th tree event, which must be an
    /// ignored NUL.
    fn events_authored(&self, index: usize) -> Evidence {
        match &self.events[index] {
            TreeEvent::InBodyIgnoredNull { authored, .. } => authored.clone(),
            other => panic!("not an ignored NUL: {other:?}"),
        }
    }
}

/// A-group: boundaries preserved, not widened.
fn g_boundaries(m: Model) {
    let text = "<body></body></html>\0";
    let o = observe(m, text);
    assert_eq!(
        o.completion,
        Completion::Unsupported(Unsupported::Tree(
            TreeBoundary::CharacterDataInAfterAfterBody
        ))
    );
    assert!(o.ignored().is_empty());
    assert_eq!(o.body_text_node_count(), 0);
    assert_eq!(o.mode, Mode::AfterAfterBody);
    // The tokenizer still produced exact NUL evidence; only the tree refuses.
    assert_eq!(o.lexed.count_origin(Origin::AuthoredDataNull), 1);
    check_lexed(text, &o.lexed);

    // Title RCDATA NUL stays Unsupported at the tokenizer boundary.
    let o = observe(m, "<title>\0</title>");
    assert_eq!(
        o.completion,
        Completion::Unsupported(Unsupported::Outside(Outside::RcdataNul))
    );
    assert!(o.tree.titles.iter().all(|t| t.node.is_none()));
}

/// S-group: transition accounting, exact only without references or tags.
fn g_steps(m: Model) {
    for (text, steps, tokens, retained) in [
        ("\0", 2, 2, 1),
        ("a\0b", 4, 4, 3),
        ("\0\0", 3, 3, 2),
        ("a\0", 3, 3, 2),
    ] {
        let l = lex_with(m, text, Limits::default());
        assert!(l.steps_exact);
        assert_eq!(l.usage.steps, steps, "steps for {text:?}");
        assert_eq!(l.usage.tokens, tokens, "tokens for {text:?}");
        assert_eq!(l.usage.retained, retained, "retained for {text:?}");
    }
    // An authored NUL costs exactly what an ordinary Data unit costs.
    assert_eq!(
        lex_with(m, "a\0b", Limits::default()).usage.steps,
        lex_with(m, "axb", Limits::default()).usage.steps
    );
}

fn steps(limit: usize) -> Limits {
    Limits {
        steps: Some(limit),
        ..Limits::default()
    }
}

fn tokens(limit: usize) -> Limits {
    Limits {
        tokens: Some(limit),
        ..Limits::default()
    }
}

fn diagnostics(limit: usize) -> Limits {
    Limits {
        diagnostics: Some(limit),
        ..Limits::default()
    }
}

fn retained(limit: usize) -> Limits {
    Limits {
        retained: Some(limit),
        ..Limits::default()
    }
}

fn refusal(resource: Resource, limit: usize, attempted: usize) -> Option<Stop> {
    Some(Stop::Resource(Refusal {
        resource,
        limit,
        attempted,
    }))
}

/// One resource cell: the stop, the committed token/diagnostic counts, the
/// processed coverage, and the abandoned regions.
#[derive(Debug, PartialEq, Eq)]
struct Cell {
    stop: Option<Stop>,
    tokens: usize,
    diagnostics: usize,
    coverage: usize,
    abandoned: Vec<(usize, usize)>,
}

fn cell(m: Model, text: &str, limits: Limits) -> Cell {
    let l = lex_with(m, text, limits);
    check_lexed(text, &l);
    Cell {
        stop: l.stop,
        tokens: l.tokens.len(),
        diagnostics: l.diagnostics.len(),
        coverage: l.coverage_end,
        abandoned: l.abandoned,
    }
}

fn expect_cell(
    m: Model,
    (text, limits): (&str, Limits),
    stop: Option<Stop>,
    tokens: usize,
    diagnostics: usize,
    coverage: usize,
    abandoned: &[(usize, usize)],
) {
    assert_eq!(
        cell(m, text, limits),
        Cell {
            stop,
            tokens,
            diagnostics,
            coverage,
            abandoned: abandoned.to_vec(),
        },
        "{text:?} with {limits:?} under {m:?}"
    );
}

type Row = (
    &'static str,
    Limits,
    Option<Stop>,
    usize,
    usize,
    usize,
    &'static [(usize, usize)],
);

/// Hand-derived resource cells for the single lifecycle: the transition
/// commits, `UnexpectedNullCharacter` / `Continued` commits and its site is
/// processed coverage, and only then is the independent retained/token effect
/// attempted. Columns: text, limits, stop, committed tokens, committed
/// diagnostics, processed coverage, abandoned (observed or pending but
/// unemitted) regions.
fn g_resources(m: Model) {
    use Resource::{Diagnostics, EmittedTokens, RetainedInterpretedBytes, TransitionSteps};
    let rows: [Row; 23] = [
        // A transition refused before the NUL is observed: nothing exists.
        ("\0", steps(0), refusal(TransitionSteps, 0, 1), 0, 0, 0, &[]),
        // The NUL is fully processed before the EOF dispatch is refused.
        ("\0", steps(1), refusal(TransitionSteps, 1, 2), 1, 1, 1, &[]),
        // A refused required diagnostic: no U+0000 token is fabricated.
        (
            "\0",
            diagnostics(0),
            refusal(Diagnostics, 0, 1),
            0,
            0,
            0,
            &[],
        ),
        // After the observation: the diagnostic survives, the token does not.
        (
            "\0",
            tokens(0),
            refusal(EmittedTokens, 0, 1),
            0,
            1,
            1,
            &[(0, 1)],
        ),
        ("\0", tokens(1), refusal(EmittedTokens, 1, 2), 1, 1, 1, &[]),
        // Attempted retained cost is one byte, never the three of U+FFFD.
        (
            "\0",
            retained(0),
            refusal(RetainedInterpretedBytes, 0, 1),
            0,
            1,
            1,
            &[(0, 1)],
        ),
        // Prior pending run versus the NUL effect: the run closes first.
        (
            "a\0",
            steps(1),
            refusal(TransitionSteps, 1, 2),
            0,
            0,
            1,
            &[(0, 1)],
        ),
        (
            "a\0",
            tokens(0),
            refusal(EmittedTokens, 0, 1),
            0,
            0,
            1,
            &[(0, 1)],
        ),
        (
            "a\0",
            diagnostics(0),
            refusal(Diagnostics, 0, 1),
            1,
            0,
            1,
            &[],
        ),
        // `a` committed, NUL observed, NUL output refused.
        (
            "a\0",
            tokens(1),
            refusal(EmittedTokens, 1, 2),
            1,
            1,
            2,
            &[(1, 2)],
        ),
        (
            "a\0",
            retained(1),
            refusal(RetainedInterpretedBytes, 1, 2),
            1,
            1,
            2,
            &[(1, 2)],
        ),
        (
            "a\0b",
            tokens(1),
            refusal(EmittedTokens, 1, 2),
            1,
            1,
            2,
            &[(1, 2)],
        ),
        (
            "a\0b",
            tokens(2),
            refusal(EmittedTokens, 2, 3),
            2,
            1,
            3,
            &[(2, 3)],
        ),
        (
            "a\0b",
            retained(2),
            refusal(RetainedInterpretedBytes, 2, 3),
            2,
            1,
            2,
            &[],
        ),
        // Dense NUL.
        (
            "\0\0\0",
            steps(3),
            refusal(TransitionSteps, 3, 4),
            3,
            3,
            3,
            &[],
        ),
        (
            "\0\0\0",
            diagnostics(2),
            refusal(Diagnostics, 2, 3),
            2,
            2,
            2,
            &[],
        ),
        (
            "\0\0\0",
            tokens(2),
            refusal(EmittedTokens, 2, 3),
            2,
            3,
            3,
            &[(2, 3)],
        ),
        (
            "\0\0\0",
            retained(2),
            refusal(RetainedInterpretedBytes, 2, 3),
            2,
            3,
            3,
            &[(2, 3)],
        ),
        // A resolved reference beside the NUL.
        (
            "&amp;\0b",
            diagnostics(0),
            refusal(Diagnostics, 0, 1),
            1,
            0,
            5,
            &[],
        ),
        (
            "&amp;\0b",
            tokens(1),
            refusal(EmittedTokens, 1, 2),
            1,
            1,
            6,
            &[(5, 6)],
        ),
        (
            "&amp;\0b",
            retained(1),
            refusal(RetainedInterpretedBytes, 1, 2),
            1,
            1,
            6,
            &[(5, 6)],
        ),
        (
            "&#65;\0b",
            tokens(1),
            refusal(EmittedTokens, 1, 2),
            1,
            1,
            6,
            &[(5, 6)],
        ),
        (
            "&#65;\0b",
            retained(1),
            refusal(RetainedInterpretedBytes, 1, 2),
            1,
            1,
            6,
            &[(5, 6)],
        ),
    ];
    for (text, limits, stop, committed_tokens, committed_diagnostics, coverage, abandoned) in rows {
        expect_cell(
            m,
            (text, limits),
            stop,
            committed_tokens,
            committed_diagnostics,
            coverage,
            abandoned,
        );
    }
    // The transition of a diagnostic-refused NUL is committed.
    assert_eq!(lex_with(m, "\0", diagnostics(0)).usage.steps, 1);
}

/// Sweep: for every input, every limit of every resource keeps
/// the invariants, only ever truncates committed evidence, and reports a typed
/// resource stop for exactly the limited resource.
fn g_sweep(m: Model) {
    let corpus = [
        "",
        "\0",
        "a\0b",
        "\0\0\0",
        "a\0",
        "\0a",
        "&amp;\0b",
        "&#65;\0b",
        "&#0;\0",
        "&#x0;",
        "<a\0>",
        "<a x=\"\0\">",
        "<a x=\0>",
        "<body>\0</body>",
        "<body>a\0b</body>",
        "<body></body>\0",
        "<body>a\r\n\0b</body>",
        "<body></body>a\0b",
        "<body></body></html>\0",
        "<title>\0</title>",
    ];
    for text in corpus {
        let full = lex_with(m, text, Limits::default());
        check_lexed(text, &full);
        let sweeps: [(Resource, usize, LimitMaker); 4] = [
            (Resource::TransitionSteps, full.usage.steps, steps),
            (Resource::EmittedTokens, full.usage.tokens, tokens),
            (Resource::Diagnostics, full.usage.diagnostics, diagnostics),
            (
                Resource::RetainedInterpretedBytes,
                full.usage.retained,
                retained,
            ),
        ];
        for (resource, used, make) in sweeps {
            for limit in 0..=used + 1 {
                let limited = lex_with(m, text, make(limit));
                check_lexed(text, &limited);
                // Committed evidence is a prefix of the unlimited evidence.
                assert!(full.tokens.starts_with(&limited.tokens), "{text:?}");
                assert!(
                    full.diagnostics.starts_with(&limited.diagnostics),
                    "{text:?}"
                );
                match limited.stop {
                    Some(Stop::Resource(r)) => {
                        assert_eq!(r.resource, resource, "{text:?} limit {limit}");
                        assert_eq!(r.limit, limit);
                    }
                    Some(Stop::Outside(_)) => assert!(full.stop.is_some()),
                    None => assert!(limit >= used, "{text:?} {resource:?} {limit}"),
                }
                // The tree never upgrades incomplete lexing to Complete.
                let observation = construct(m, limited.clone());
                check_tree(text, &observation);
                if limited.stop.is_some() {
                    assert_ne!(observation.completion, Completion::Complete);
                }
            }
        }
    }
}

/// Independent first-principles oracle for diagnostic survival, for plain
/// Data text (no references or tags). A NUL is *observed* once every
/// obligation before it has been met: all earlier tokens fit the token limit,
/// or all earlier bytes fit the retained limit. An observed NUL has its
/// `UnexpectedNullCharacter` / `Continued` diagnostic even when its own
/// token or retained byte is then refused. The expectation is read from the
/// unlimited run's token order only, never from the limited run under test.
fn g_survival(m: Model) {
    for text in ["\0", "a\0", "\0a", "a\0b", "\0\0\0", "\0a\0"] {
        let full = lex_with(m, text, Limits::default());
        let mut before_bytes = 0usize;
        let mut nul_sites: Vec<(usize, usize)> = Vec::new(); // (token index, preceding bytes)
        for (index, token) in full.tokens.iter().enumerate() {
            if let Token::Characters(c) = token {
                if c.origin == Origin::AuthoredDataNull {
                    nul_sites.push((index, before_bytes));
                }
                before_bytes += c.interpreted.len();
            }
        }
        for limit in 0..=full.tokens.len() + 1 {
            let expected = nul_sites
                .iter()
                .filter(|(index, _)| *index <= limit)
                .count();
            let limited = lex_with(m, text, tokens(limit));
            assert_eq!(
                limited.diagnostics.len(),
                expected,
                "diagnostic survival under tokens({limit}) for {text:?}"
            );
        }
        for limit in 0..=full.usage.retained + 1 {
            let expected = nul_sites
                .iter()
                .filter(|(_, before)| *before <= limit)
                .count();
            let limited = lex_with(m, text, retained(limit));
            assert_eq!(
                limited.diagnostics.len(),
                expected,
                "diagnostic survival under retained({limit}) for {text:?}"
            );
        }
    }
}

type Group = (&'static str, fn(Model));
type LimitMaker = fn(usize) -> Limits;

const GROUPS: [Group; 9] = [
    ("tokenizer_nul", g_tokenizer_nul),
    ("references", g_references),
    ("non_data", g_non_data),
    ("tree", g_tree),
    ("boundaries", g_boundaries),
    ("steps", g_steps),
    ("resources", g_resources),
    ("survival", g_survival),
    ("sweep", g_sweep),
];

fn detects(group: fn(Model), model: Model) -> bool {
    panic::catch_unwind(AssertUnwindSafe(|| group(model))).is_err()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

const SOUND: Model = Model::SOUND;

#[test]
fn p0_pins_are_recorded() {
    assert_eq!(WHATWG_HEAD.len(), 40);
    assert_eq!(WHATWG_SOURCE_BLOB.len(), 40);
    assert_eq!(WPT_CHALLENGE_HEAD.len(), 40);
    assert_eq!(HTML5LIB_CHALLENGE_HEAD.len(), 40);
}

#[test]
fn t1_the_sound_model_satisfies_every_theorem_group() {
    for (name, group) in GROUPS {
        assert!(!detects(group, SOUND), "sound model fails group {name}");
    }
}

#[test]
fn t2_authored_data_nul_is_u0000_evidence_with_continued_handling() {
    let l = lex("\0");
    assert_eq!(l.projection(), vec![nul(0)]);
    assert_eq!(l.diagnostics.len(), 1);
    let d = &l.diagnostics[0];
    assert_eq!(d.code, Code::UnexpectedNullCharacter);
    assert_eq!(d.context, Context::Data);
    assert_eq!(d.handling, Handling::Continued);
    assert_ne!(d.handling, Handling::ReplacedWithReplacementCharacter);
    assert_eq!((d.at.start, d.at.end, d.at.raw.as_str()), (0, 1, "\0"));
}

#[test]
fn t3_the_tree_ignores_exactly_the_nul_and_keeps_ordered_surviving_text() {
    let o = observe(SOUND, "<body>a\0b</body>");
    assert_eq!(o.body_text(), "ab");
    assert_eq!(o.body_contributions(), vec![lit(6, 7, "a"), lit(8, 9, "b")]);
    // The ignored NUL is explained by tokenizer + tree evidence only.
    assert_eq!(o.lexed.diagnostics.len(), 1);
    assert_eq!(o.ignored(), vec![(2, 7, 8)]);
    assert!(
        o.body_contributions()
            .iter()
            .all(|(_, s, e, _)| !(*s <= 7 && 7 < *e))
    );
}

#[test]
fn t4_after_body_recovery_precedes_the_in_body_ignore_for_a_closed_body() {
    let o = observe(SOUND, "<body></body>\0");
    assert_eq!(o.mode, Mode::InBody);
    assert!(matches!(
        o.events.as_slice(),
        [
            TreeEvent::AfterBodyNonWhitespaceToInBody { token: 2 },
            TreeEvent::InBodyIgnoredNull { token: 2, .. }
        ]
    ));
    assert_eq!(o.body_text_node_count(), 0);
}

#[test]
fn t5_the_early_shell_reaches_in_body_through_existing_modes_only() {
    let o = observe(SOUND, "\0");
    assert_eq!(o.mode, Mode::InBody);
    assert!(o.chain.iter().all(|mode| EARLY_CHAIN.contains(mode)));
    assert_eq!(o.body_text_node_count(), 0);
}

#[test]
fn f01_nul_handling_is_state_specific() {
    let data = lex("\0");
    let tag = lex("<a\0>");
    let quoted = lex("<a x=\"\0\">");
    let unquoted = lex("<a x=\0>");
    assert_eq!(data.diagnostics[0].handling, Handling::Continued);
    for tagged in [&tag, &quoted, &unquoted] {
        assert_eq!(
            tagged.diagnostics[0].handling,
            Handling::ReplacedWithReplacementCharacter
        );
        assert_ne!(tagged.diagnostics[0].context, Context::Data);
    }
    assert_eq!(data.interpreted(), "\0");
    assert!(!tag.interpreted().contains('\u{fffd}'));
    assert!(matches!(&tag.tokens[0], Token::Tag(t) if t.name == "a\u{fffd}"));
    // A single shared "NUL means U+FFFD" rule is observably wrong for Data.
    let shared = Model::mutated(Mutation::NulBecomesReplacement);
    assert!(detects(g_tokenizer_nul, shared));
}

#[test]
fn f02_the_correction_is_not_tokenizer_only() {
    // A tokenizer that is already correct still leaves U+0000 in the tree if
    // the tree inserts characters naively.
    let naive_tree = Model::mutated(Mutation::TreeInsertsNul);
    let o = observe(naive_tree, "<body>\0</body>");
    assert_eq!(o.lexed.projection()[0], nul(6));
    assert_eq!(o.body_text(), "\0");
    assert!(detects(g_tree, naive_tree));
    // And a tree that ignores correctly needs the exact-NUL token to exist.
    let o = observe(SOUND, "<body>\0</body>");
    assert_eq!(o.body_text(), "");
}

#[test]
fn f03_a_mixed_aggregate_token_cannot_carry_the_theorem() {
    let model = Model::mutated(Mutation::AggregateWholeAnchor);
    let o = observe(model, "<body>a\0b</body>");
    // The final text happens to be right ...
    assert_eq!(o.body_text(), "ab");
    // ... but the only available anchor covers the ignored NUL byte, so the
    // surviving contribution's authored evidence is not its interpreted text.
    let contributions = o.body_contributions();
    assert_eq!(contributions.len(), 1);
    let (_, start, end, interpreted) = &contributions[0];
    assert_eq!(interpreted, "ab");
    assert_eq!((*start, *end), (6, 9));
    assert!(
        o.tree.body.as_ref().unwrap().text_nodes[0].contributions[0]
            .authored
            .raw
            .contains('\0')
    );
    assert!(detects(g_tokenizer_nul, model));
    assert!(detects(g_tree, model));
}

#[test]
fn f04_the_tree_cannot_split_an_aggregate_by_scanning() {
    let model = Model::mutated(Mutation::AggregateSplitByInterpretedOffsets);
    let o = observe(model, "<body>a\r\n\0b</body>");
    let sound = observe(SOUND, "<body>a\r\n\0b</body>");
    assert_eq!(sound.body_text(), "a\nb");
    assert_ne!(o.body_contributions(), sound.body_contributions());
    // The reconstructed anchor for `b` lands on the NUL byte itself.
    let nodes = &o.tree.body.as_ref().unwrap().text_nodes;
    let last = nodes[0].contributions.last().unwrap();
    assert_eq!(last.interpreted, "b");
    assert_eq!(last.authored.raw, "\0");
    assert!(detects(g_tree, model));
}

#[test]
fn f05_interpreted_offsets_are_not_authored_offsets() {
    // CRLF normalization: interpreted `a\n\0b` places the NUL at interpreted
    // offset 2, but the authored NUL is at byte 3.
    let crlf = lex("a\r\n\0b");
    let interpreted_prefix: usize = crlf
        .contributions()
        .iter()
        .take_while(|c| c.origin != Origin::AuthoredDataNull)
        .map(|c| c.interpreted.len())
        .sum();
    assert_eq!(interpreted_prefix, 2);
    assert_eq!(crlf.contributions()[1].authored.start, 3);
    // A resolved reference: `&` (one interpreted byte) spans five authored.
    let named = lex("&amp;\0b");
    let interpreted_prefix: usize = named
        .contributions()
        .iter()
        .take_while(|c| c.origin != Origin::AuthoredDataNull)
        .map(|c| c.interpreted.len())
        .sum();
    assert_eq!(interpreted_prefix, 1);
    assert_eq!(named.contributions()[1].authored.start, 5);
}

#[test]
fn f06_tokenizer_and_tree_evidence_are_different_facts() {
    let o = observe(SOUND, "<body></body>\0");
    // One tokenizer fact, two distinct tree facts.
    assert_eq!(o.lexed.diagnostics.len(), 1);
    assert_eq!(o.events.len(), 2);
    assert_eq!(o.lexed.diagnostics[0].code, Code::UnexpectedNullCharacter);
    // Removing one leaves the other: they are not the same evidence.
    let silent_tree = observe(
        Model::mutated(Mutation::TreeRecordsNoEvidence),
        "<body>\0</body>",
    );
    assert_eq!(silent_tree.lexed.diagnostics.len(), 1);
    assert!(silent_tree.ignored().is_empty());
    assert!(detects(
        g_tree,
        Model::mutated(Mutation::TreeRecordsNoEvidence)
    ));
    // A tokenizer-only record is also not tree evidence: a tree fed a
    // NUL-free input has no ignored events.
    assert!(observe(SOUND, "<body>ab</body>").ignored().is_empty());
}

#[test]
fn f07_authored_nul_and_numeric_zero_are_different() {
    let o = observe(SOUND, "<body>&#0;\0&#x0;</body>");
    assert_eq!(o.body_text(), "\u{fffd}\u{fffd}");
    assert_eq!(o.ignored().len(), 1);
    assert_eq!(
        o.lexed
            .diagnostics
            .iter()
            .map(|d| d.code)
            .collect::<Vec<_>>(),
        vec![
            Code::NullCharacterReference,
            Code::UnexpectedNullCharacter,
            Code::NullCharacterReference
        ]
    );
    assert!(detects(
        g_references,
        Model::mutated(Mutation::NumericZeroAsNul)
    ));
}

#[test]
fn f08_rcdata_nul_does_not_widen_with_data() {
    let o = observe(SOUND, "<title>\0</title>");
    assert_eq!(
        o.completion,
        Completion::Unsupported(Unsupported::Outside(Outside::RcdataNul))
    );
    let widened = Model::mutated(Mutation::RcdataNulUsesDataRule);
    assert!(
        lex_with(widened, "<title>\0</title>", Limits::default())
            .stop
            .is_none()
    );
    assert!(detects(g_non_data, widened));
}

#[test]
fn f09_no_new_tokenizer_state_is_needed() {
    // The state set is unchanged and an authored NUL is one ordinary Data
    // transition.
    fn states(ctx: Ctx) -> u8 {
        match ctx {
            Ctx::Data => 0,
            Ctx::Rcdata => 1,
        }
    }
    assert_eq!(states(Ctx::Data) + states(Ctx::Rcdata), 1);
    let nul_run = lex("\0\0\0");
    let plain_run = lex("xyz");
    assert_eq!(nul_run.usage.steps, plain_run.usage.steps);
    assert_eq!(nul_run.usage.steps, 4);
    assert!(detects(
        g_steps,
        Model::mutated(Mutation::NulCostsTwoTransitions)
    ));
}

#[test]
fn f10_no_new_insertion_mode_is_needed() {
    fn existing(mode: Mode) -> bool {
        match mode {
            Mode::Initial
            | Mode::BeforeHtml
            | Mode::BeforeHead
            | Mode::InHead
            | Mode::AfterHead
            | Mode::TitleText
            | Mode::InBody
            | Mode::AfterBody
            | Mode::AfterAfterBody => true,
        }
    }
    for text in [
        "\0",
        "<body>\0</body>",
        "<body></body>\0",
        "<body></body>a\0b",
    ] {
        let o = observe(SOUND, text);
        assert!(o.chain.iter().copied().chain([o.mode]).all(existing));
    }
}

#[test]
fn f11_no_new_provenance_domain_is_needed() {
    // Every NUL token and diagnostic carries the same source identity and a
    // plain authored byte range as any other evidence.
    let l = lex("a\0b");
    let id = SourceId::new(1);
    for c in l.contributions() {
        assert_eq!(c.authored.source_id, id);
    }
    assert_eq!(l.diagnostics[0].at.source_id, id);
    let nul_token = l.contributions()[1];
    assert_eq!(nul_token.authored.raw, "\0");
    assert_eq!(
        (nul_token.authored.start, nul_token.authored.end),
        (l.diagnostics[0].at.start, l.diagnostics[0].at.end)
    );
}

#[test]
fn f12_no_new_resource_dimension_is_needed() {
    // Exhaustive destructure and match: adding a dimension breaks this test at
    // compile time.
    let Usage {
        steps: _,
        tokens: _,
        diagnostics: _,
        retained: _,
    } = Usage::default();
    let Limits {
        steps: _,
        tokens: _,
        diagnostics: _,
        retained: _,
    } = Limits::default();
    fn dimension(resource: Resource) -> usize {
        match resource {
            Resource::TransitionSteps => 0,
            Resource::EmittedTokens => 1,
            Resource::Diagnostics => 2,
            Resource::RetainedInterpretedBytes => 3,
        }
    }
    assert_eq!(dimension(Resource::RetainedInterpretedBytes), 3);
    // One authored NUL retains one byte; the old replacement cost was three.
    assert_eq!(lex("\0").usage.retained, 1);
    assert_ne!(lex("\0").usage.retained, '\u{fffd}'.len_utf8());
    assert!(detects(
        g_steps,
        Model::mutated(Mutation::NulReservesReplacementBytes)
    ));
    // The refusal reports one attempted byte, not the replacement's three.
    let l = lex_with(SOUND, "\0", retained(0));
    assert_eq!(l.stop, refusal(Resource::RetainedInterpretedBytes, 0, 1));
}

#[test]
fn f13_current_defective_gold_is_not_an_oracle() {
    // The current accepted tokenizer GOLD (ERR-003) freezes this defect for
    // authored Data U+0000. It is recorded here only as a negative pin.
    const CURRENT_DEFECT_INTERPRETED: &str = "\u{fffd}";
    let corrected = lex("\0");
    assert_ne!(corrected.interpreted(), CURRENT_DEFECT_INTERPRETED);
    assert_eq!(corrected.interpreted(), "\0");
    // The validator rejects exactly that defect.
    let defect = Model::mutated(Mutation::NulBecomesReplacement);
    assert_eq!(
        lex_with(defect, "\0", Limits::default()).interpreted(),
        CURRENT_DEFECT_INTERPRETED
    );
    assert!(detects(g_tokenizer_nul, defect));
    // Mixed preprocessing of PRE-010's NUL position shows the same divergence.
    assert_ne!(
        lex("\u{000c}\0").interpreted(),
        lex_with(defect, "\u{000c}\0", Limits::default()).interpreted()
    );
}

#[test]
fn f14_successor_correspondence_is_not_one_bucket() {
    // NUL-free Data Named and Numeric references are unaffected by the
    // correction; only NUL-bearing cells diverge. Indiscriminate rewriting of
    // the successor correspondence would conflate the two.
    let defect = Model::mutated(Mutation::NulBecomesReplacement);
    for text in ["&amp;b", "&#65;b", "&#0;", "&#x0;", "a<a x=y>b"] {
        assert_eq!(
            lex(text).projection(),
            lex_with(defect, text, Limits::default()).projection(),
            "{text:?}"
        );
    }
    for text in ["&amp;\0b", "&#65;\0b", "\0"] {
        assert_ne!(
            lex(text).projection(),
            lex_with(defect, text, Limits::default()).projection(),
            "{text:?}"
        );
    }
}

#[test]
fn f15_after_after_body_does_not_widen() {
    let o = observe(SOUND, "<body></body></html>\0");
    assert_eq!(
        o.completion,
        Completion::Unsupported(Unsupported::Tree(
            TreeBoundary::CharacterDataInAfterAfterBody
        ))
    );
    let widened = Model::mutated(Mutation::AfterAfterBodyWidened);
    assert_eq!(
        observe(widened, "<body></body></html>\0").completion,
        Completion::Complete
    );
    assert!(detects(g_boundaries, widened));
}

#[test]
fn f16_decoded_output_is_not_authored_input() {
    // No accepted numeric value decodes to an authored NUL, and decoded
    // U+FFFD is ordinary text that the tree never ignores.
    for code in 0..=0x7eu32 {
        let text = format!("<body>&#{code};</body>");
        let o = observe(SOUND, &text);
        if matches!(code, 0 | 0x21..=0x7e) {
            assert!(!o.body_text().contains('\0'), "{text}");
            assert!(o.ignored().is_empty(), "{text}");
            assert_eq!(o.lexed.count_origin(Origin::AuthoredDataNull), 0);
        } else {
            assert_eq!(
                o.lexed.stop,
                Some(Stop::Outside(Outside::NumericValue)),
                "{text}"
            );
        }
    }
    let o = observe(SOUND, "<body>&#0;</body>");
    assert_eq!(o.body_text(), "\u{fffd}");
}

#[test]
fn f17_final_tree_shape_alone_is_insufficient() {
    // The same final text with different evidence.
    let plain = observe(SOUND, "<body>ab</body>");
    let with_nul = observe(SOUND, "<body>a\0b</body>");
    assert_eq!(plain.body_text(), with_nul.body_text());
    assert_ne!(plain.body_contributions(), with_nul.body_contributions());
    assert!(plain.lexed.diagnostics.is_empty());
    assert_eq!(with_nul.lexed.diagnostics.len(), 1);
    assert!(plain.events.is_empty() && !with_nul.events.is_empty());
    // A model that loses tokenizer U+0000 evidence yields the same final text
    // yet is rejected.
    let lossy = Model::mutated(Mutation::DropNulToken);
    let o = observe(lossy, "<body>a\0b</body>");
    assert_eq!(o.body_text(), with_nul.body_text());
    assert!(detects(g_tokenizer_nul, lossy));
}

#[test]
fn r2_the_observed_nul_diagnostic_survives_later_token_and_retained_refusal() {
    use Resource::{EmittedTokens, RetainedInterpretedBytes};
    for (text, limits, resource, attempted) in [
        ("\0", tokens(0), EmittedTokens, 1),
        ("\0", retained(0), RetainedInterpretedBytes, 1),
        ("a\0", tokens(1), EmittedTokens, 2),
        ("a\0", retained(1), RetainedInterpretedBytes, 2),
    ] {
        let nul_at = text.find('\0').unwrap();
        let l = lex_with(SOUND, text, limits);
        assert_eq!(l.stop.map(stop_resource), Some(Some(resource)), "{text:?}");
        let Some(Stop::Resource(r)) = l.stop else {
            panic!("expected a resource stop for {text:?}");
        };
        assert_eq!(r.attempted, attempted);
        // The diagnostic is committed evidence with Continued handling ...
        assert_eq!(l.diagnostics.len(), 1, "{text:?}");
        assert_eq!(l.diagnostics[0].handling, Handling::Continued);
        assert_eq!(
            (l.diagnostics[0].at.start, l.diagnostics[0].at.end),
            (nul_at, nul_at + 1)
        );
        // ... its site is processed coverage ...
        assert_eq!(l.coverage_end, nul_at + 1);
        // ... but no U+0000 token and no replacement exist, and the NUL is
        // explicitly unemitted.
        assert_eq!(l.count_origin(Origin::AuthoredDataNull), 0);
        assert!(!l.interpreted().contains('\0') && !l.interpreted().contains('\u{fffd}'));
        assert_eq!(l.abandoned, vec![(nul_at, nul_at + 1)]);
        assert!(!l.reached_eof());
    }
    // Each lifecycle fault is killed for the intended reason.
    for fault in [
        Mutation::SuppressDiagnosticOnTokenRefusal,
        Mutation::SuppressDiagnosticOnRetainedRefusal,
        Mutation::NulEffectPreparedBeforeDiagnostic,
        Mutation::DiagnosticSiteNotCovered,
    ] {
        assert!(detects(g_resources, Model::mutated(fault)), "{fault:?}");
    }
    // Suppression is observable: the faulty model really drops the diagnostic.
    let dropped = lex_with(
        Model::mutated(Mutation::SuppressDiagnosticOnTokenRefusal),
        "\0",
        tokens(0),
    );
    assert!(dropped.diagnostics.is_empty());
    let dropped = lex_with(
        Model::mutated(Mutation::NulEffectPreparedBeforeDiagnostic),
        "\0",
        retained(0),
    );
    assert!(dropped.diagnostics.is_empty());
    let uncovered = lex_with(
        Model::mutated(Mutation::DiagnosticSiteNotCovered),
        "\0",
        tokens(0),
    );
    assert_eq!(uncovered.diagnostics.len(), 1);
    assert_eq!(uncovered.coverage_end, 0);
}

#[test]
fn r3_a_refused_prior_run_means_the_nul_was_never_observed() {
    // The pending `a` cannot close, so the NUL step never begins.
    let l = lex_with(SOUND, "a\0", tokens(0));
    assert!(l.diagnostics.is_empty());
    assert_eq!(l.usage.steps, 2);
    // Once `a` has closed, the NUL is observed and its diagnostic is stable.
    let l = lex_with(SOUND, "a\0", tokens(1));
    assert_eq!(l.diagnostics.len(), 1);
    assert_eq!(l.tokens.len(), 1);
}

fn stop_resource(stop: Stop) -> Option<Resource> {
    match stop {
        Stop::Resource(r) => Some(r.resource),
        Stop::Outside(_) => None,
    }
}

#[test]
fn r1_resource_exhaustion_is_typed_and_never_upgraded() {
    let o = observe_limited(SOUND, "<body>a\0b</body>", tokens(3));
    assert_eq!(
        o.completion,
        Completion::ResourceLimit(Refusal {
            resource: Resource::EmittedTokens,
            limit: 3,
            attempted: 4
        })
    );
    assert_eq!(o.body_text(), "a");
    assert_eq!(o.ignored(), vec![(2, 7, 8)]);
    assert!(!o.lexed.reached_eof());
    let upgraded = Model::mutated(Mutation::UpgradeIncompleteToComplete);
    assert_eq!(
        observe_limited(upgraded, "<body>a\0b</body>", tokens(3)).completion,
        Completion::Complete
    );
    assert!(detects(g_sweep, upgraded));
    // Resource exhaustion is not unsupported markup.
    assert!(!matches!(o.completion, Completion::Unsupported(_)));
}

#[test]
fn s01_every_mutation_is_detected_by_its_declared_detector() {
    let table: [(Mutation, &[&str]); 21] = [
        (Mutation::NulBecomesReplacement, &["tokenizer_nul"]),
        (Mutation::NulClaimsReplacementRecovery, &["tokenizer_nul"]),
        (Mutation::DropNulToken, &["tokenizer_nul"]),
        (Mutation::NulReservesReplacementBytes, &["steps"]),
        (Mutation::NulCostsTwoTransitions, &["steps"]),
        (Mutation::TagNulKeepsNul, &["non_data"]),
        (Mutation::RcdataNulUsesDataRule, &["non_data"]),
        (Mutation::NumericZeroAsNul, &["references"]),
        (Mutation::AggregateWholeAnchor, &["tokenizer_nul", "tree"]),
        (
            Mutation::AggregateSplitByInterpretedOffsets,
            &["tokenizer_nul", "tree"],
        ),
        (Mutation::TreeInsertsNul, &["tree"]),
        (Mutation::TreeInsertsReplacement, &["tree"]),
        (Mutation::TreeRetainsNulContribution, &["tree"]),
        (Mutation::TreeRecordsNoEvidence, &["tree"]),
        (Mutation::AfterBodyNoReprocess, &["tree"]),
        (Mutation::AfterAfterBodyWidened, &["boundaries"]),
        (Mutation::UpgradeIncompleteToComplete, &["sweep"]),
        (
            Mutation::SuppressDiagnosticOnTokenRefusal,
            &["resources", "survival"],
        ),
        (
            Mutation::SuppressDiagnosticOnRetainedRefusal,
            &["resources", "survival"],
        ),
        (Mutation::NulEffectPreparedBeforeDiagnostic, &["resources"]),
        (Mutation::DiagnosticSiteNotCovered, &["resources", "sweep"]),
    ];
    for (mutation, detectors) in table {
        let model = Model::mutated(mutation);
        for detector in detectors {
            let (_, group) = GROUPS
                .iter()
                .find(|(name, _)| name == detector)
                .expect("declared detector exists");
            assert!(
                detects(*group, model),
                "{mutation:?} escaped declared detector {detector}"
            );
        }
        // Unmutated sibling check: the same group passes for the sound model.
        for detector in detectors {
            let (_, group) = GROUPS.iter().find(|(name, _)| name == detector).unwrap();
            assert!(!detects(*group, SOUND));
        }
    }
}
