//! Candidate-independent validation of the *proposed* successor theorem
//! "Selected AfterAfterBody Uniform Character-Run Handling" (Issue #886).
//!
//! This module is **validation only**. It changes no production
//! tree-construction behavior. Production is expected to remain unchanged and
//! still refuses every `after after body` character run as
//! `UnprovedCharacterDataPosition`. Nothing here authorizes production
//! placement or implementation.
//!
//! # Successor, not rewrite
//!
//! Issue #353 intentionally froze the predecessor boundary
//! `<body></body></html>x` -> *AfterAfterBody character data unsupported before
//! mutation*. That remains correct historical evidence for the predecessor
//! capability, and the sibling `after_body_successor_validation` module still
//! pins it. This module states a **successor** theorem. If it is later
//! accepted, a separate production-placement review decides how the predecessor
//! boundary is superseded. This module neither imports nor edits that GOLD.
//!
//! # Independent oracle boundary
//!
//! Expected meaning is authored from the candidate theorem, not from any
//! production run. The boundary is structural and greppable: this module does
//! not import the tree-construction driver, session, or durable-result modules.
//! It therefore cannot call production `classify`, cannot call the production
//! document-shell constructor, and cannot project a production result into an
//! expectation. A test below enforces that over this file's own text.
//!
//! The only production code used is the already-accepted lower layer, the batch
//! tokenizer, and only as *evidence input*: emitted token boundaries,
//! interpreted character-run values, retained source anchors, preprocessing
//! consequences, tokenizer diagnostics, and run completion. The tokenizer is
//! never the tree-semantic oracle.
//!
//! Two independent statements meet and must agree:
//!
//! 1. [`CandidateSession`], a test-only machine implementing the candidate
//!    action set over lower-layer token evidence; and
//! 2. [`candidate_gold`], hand-authored expected observations.
//!
//! If the theorem were internally incoherent, the machine could not reproduce
//! the authored observations and the comparison fails rather than adapting.
//!
//! # Deliberately partial model
//!
//! Only the cells the fixtures traverse are modelled. Every other cell is
//! refused as [`CandidateUnsupported::OutsideModelledCandidateCells`]. That is
//! a property of this oracle, not a claim about production, and this module is
//! not a second HTML parser. In particular `after body` character data is not
//! part of this successor (it belongs to the accepted TC-S2 theorem).
//!
//! # Termination without a work budget
//!
//! The selected non-whitespace token path is `AfterAfterBody -> InBody ->
//! consumed`. [`CandidateSession::process`] asserts that an insertion mode is
//! never evaluated twice for one token, and the `in body` character cell
//! consumes *every* character token (ordinary, whitespace, mixed, and U+0000).
//! Consequently the token cannot return to `AfterAfterBody`. No reprocess
//! counter, retry limit, or work budget exists here or is proposed.
//! Cross-token revisits are paid for only by later distinct consumed tokens
//! (`</body>` then `</html>`), never by the same token.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::diagnostic::{
    HtmlTokenizerDiagnosticCode as Diag, HtmlTokenizerDiagnosticContext as Context,
    HtmlTokenizerDiagnosticHandling as Handling,
};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::{
    HtmlTokenizerCapability, HtmlTokenizerCompletion, HtmlTokenizerIncompleteCause,
    HtmlTokenizerRunResult,
};

// ---------------------------------------------------------------------------
// Canonical byte authority
// ---------------------------------------------------------------------------

type ByteRange = (usize, usize);
type RequiredRange = (ByteRange, &'static [u8]);

/// One authored fixture, stored as an escaped byte literal. Rendered text is
/// never fixture-byte authority, and authored source bytes are distinct from
/// the tokenizer's interpreted character value (CR and CRLF).
struct Fixture {
    id: &'static str,
    bytes: &'static [u8],
    length: usize,
    /// Byte ranges the theorem depends on, with their exact expected content.
    required_ranges: &'static [RequiredRange],
}

const FIXTURES: &[Fixture] = &[
    // --- all-HTML-whitespace runs: delegate, mode stays AfterAfterBody ---
    Fixture {
        id: "W1",
        bytes: b"<body></body></html>\x20",
        length: 21,
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 13), b"</body>"),
            ((13, 20), b"</html>"),
            ((20, 21), b"\x20"),
        ],
    },
    Fixture {
        id: "W2",
        bytes: b"<body></body></html>\x09",
        length: 21,
        required_ranges: &[((13, 20), b"</html>"), ((20, 21), b"\x09")],
    },
    Fixture {
        id: "W3",
        bytes: b"<body></body></html>\x0a",
        length: 21,
        required_ranges: &[((13, 20), b"</html>"), ((20, 21), b"\x0a")],
    },
    Fixture {
        id: "W4",
        bytes: b"<body></body></html>\x0c",
        length: 21,
        required_ranges: &[((13, 20), b"</html>"), ((20, 21), b"\x0c")],
    },
    // Authored CR is one source byte; its interpreted value is LF.
    Fixture {
        id: "W5",
        bytes: b"<body></body></html>\x0d",
        length: 21,
        required_ranges: &[((13, 20), b"</html>"), ((20, 21), b"\x0d")],
    },
    // Authored CRLF is two source bytes; its interpreted value is one LF.
    Fixture {
        id: "W6",
        bytes: b"<body></body></html>\x0d\x0a",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"\x0d\x0a")],
    },
    // One aggregate multi-scalar whitespace run.
    Fixture {
        id: "W7",
        bytes: b"<body></body></html>\x20\x09",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"\x20\x09")],
    },
    // --- all-non-whitespace runs: one recovery, one same-token reprocess ---
    Fixture {
        id: "N1",
        bytes: b"<body></body></html>x",
        length: 21,
        required_ranges: &[((13, 20), b"</html>"), ((20, 21), b"x")],
    },
    Fixture {
        id: "N2",
        bytes: b"<body></body></html>xy",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"xy")],
    },
    // NBSP is not HTML whitespace. Two UTF-8 bytes, one scalar.
    Fixture {
        id: "N3",
        bytes: b"<body></body></html>\xc2\xa0",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"\xc2\xa0")],
    },
    // U+000B is not HTML whitespace.
    Fixture {
        id: "N4",
        bytes: b"<body></body></html>\x0b",
        length: 21,
        required_ranges: &[((13, 20), b"</html>"), ((20, 21), b"\x0b")],
    },
    // --- mixed aggregate runs: refused before mutation ---
    Fixture {
        id: "M1",
        bytes: b"<body></body></html>\x20x",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"\x20x")],
    },
    Fixture {
        id: "M2",
        bytes: b"<body></body></html>x\x20",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"x\x20")],
    },
    // CRLF then a non-whitespace scalar: three source bytes, two scalars.
    Fixture {
        id: "M3",
        bytes: b"<body></body></html>\x0d\x0ax",
        length: 23,
        required_ranges: &[((13, 20), b"</html>"), ((20, 23), b"\x0d\x0ax")],
    },
    Fixture {
        id: "M4",
        bytes: b"<body></body></html>x\x20y",
        length: 23,
        required_ranges: &[((13, 20), b"</html>"), ((20, 23), b"x\x20y")],
    },
    // --- authored U+0000 composition with #885 ---
    Fixture {
        id: "Z1",
        bytes: b"<body></body></html>\x00",
        length: 21,
        required_ranges: &[((13, 20), b"</html>"), ((20, 21), b"\x00")],
    },
    Fixture {
        id: "Z2",
        bytes: b"<body></body></html>\x00\x00",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"\x00\x00")],
    },
    Fixture {
        id: "Z3",
        bytes: b"<body></body></html>x\x00",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"x\x00")],
    },
    Fixture {
        id: "Z4",
        bytes: b"<body></body></html>\x20\x00",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"\x20\x00")],
    },
    Fixture {
        id: "Z5",
        bytes: b"<body></body></html>\x00x",
        length: 22,
        required_ranges: &[((13, 20), b"</html>"), ((20, 22), b"\x00x")],
    },
    // --- coalescing, identity, and cross-token cycles ---
    Fixture {
        id: "C1",
        bytes: b"<body>a</body></html>\x20",
        length: 22,
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 7), b"a"),
            ((7, 14), b"</body>"),
            ((14, 21), b"</html>"),
            ((21, 22), b"\x20"),
        ],
    },
    Fixture {
        id: "C2",
        bytes: b"<body>a</body></html>x",
        length: 22,
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 7), b"a"),
            ((7, 14), b"</body>"),
            ((14, 21), b"</html>"),
            ((21, 22), b"x"),
        ],
    },
    Fixture {
        id: "C3",
        bytes: b"<body>a</body></html>\x00",
        length: 22,
        required_ranges: &[((14, 21), b"</html>"), ((21, 22), b"\x00")],
    },
    Fixture {
        id: "C4",
        bytes: b"<body>a</body></html>\x20x",
        length: 23,
        required_ranges: &[((14, 21), b"</html>"), ((21, 23), b"\x20x")],
    },
    Fixture {
        id: "C5",
        bytes: b"<body></body></html>x</body></html>y",
        length: 36,
        required_ranges: &[
            ((13, 20), b"</html>"),
            ((20, 21), b"x"),
            ((21, 28), b"</body>"),
            ((28, 35), b"</html>"),
            ((35, 36), b"y"),
        ],
    },
];

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|candidate| candidate.id == id)
        .expect("canonical candidate fixture")
}

impl Fixture {
    /// The canonical bytes as source text. Panics rather than lossily
    /// converting: a fixture that is not valid UTF-8 is a stop condition.
    fn source_text(&self) -> &'static str {
        std::str::from_utf8(self.bytes).expect("canonical fixture bytes are valid UTF-8")
    }
}

// ---------------------------------------------------------------------------
// Independent candidate domain
// ---------------------------------------------------------------------------

/// Candidate insertion modes. Test-only, deliberately not the production type,
/// and deriving no ordering: the successor has a backward edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
    AfterAfterBody,
}

impl CandidateMode {
    /// Every modelled mode, to state the per-token dispatch bound as a
    /// cardinality fact rather than an invented constant.
    const ALL: [Self; 8] = [
        Self::Initial,
        Self::BeforeHtml,
        Self::BeforeHead,
        Self::InHead,
        Self::AfterHead,
        Self::InBody,
        Self::AfterBody,
        Self::AfterAfterBody,
    ];
}

/// The candidate partition of one interpreted character run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateRunClass {
    AllWhitespace,
    AllNonWhitespace,
    Mixed,
}

/// The HTML whitespace set this candidate fixes.
fn is_candidate_html_whitespace(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ')
}

/// Classifies the tokenizer's **interpreted** character run as one aggregate.
/// No source subrange is guessed at, and an empty run cannot occur because the
/// tokenizer emits no empty character token.
fn classify_run(interpreted: &str) -> CandidateRunClass {
    let mut whitespace = false;
    let mut other = false;
    for character in interpreted.chars() {
        if is_candidate_html_whitespace(character) {
            whitespace = true;
        } else {
            other = true;
        }
    }
    match (whitespace, other) {
        (true, true) => CandidateRunClass::Mixed,
        (true, false) => CandidateRunClass::AllWhitespace,
        (false, _) => CandidateRunClass::AllNonWhitespace,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateShellName {
    Html,
    Head,
    Body,
}

/// The candidate's normalization of one accepted lower-layer token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateTokenShape<'run> {
    Characters {
        range: ByteRange,
        interpreted: &'run str,
    },
    StartTag {
        name: CandidateShellName,
        range: ByteRange,
    },
    EndTag {
        name: CandidateShellName,
        range: ByteRange,
    },
    EndOfFile {
        at: usize,
    },
}

impl CandidateTokenShape<'_> {
    /// The exclusive source offset a committed processing of this token covers.
    fn committed_end(&self) -> usize {
        match self {
            Self::Characters { range, .. }
            | Self::StartTag { range, .. }
            | Self::EndTag { range, .. } => range.1,
            Self::EndOfFile { at } => *at,
        }
    }
}

/// What the candidate refuses, with exact typed meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateUnsupported {
    /// A mixed whitespace/non-whitespace `after after body` run. The successor
    /// authorizes no run splitting, so the whole aggregate token is refused.
    MixedAfterAfterBodyCharacterRun,
    /// The historical #353 predecessor refusal. A sound successor never
    /// produces it; it exists so a deliberately wrong model can.
    HistoricalAfterAfterBodyCharacterData,
    /// A cell this deliberately partial oracle does not model.
    OutsideModelledCandidateCells,
}

/// Where a candidate element node's existence comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateOrigin {
    /// The trigger token's own authored start tag, as an exact span.
    Authored(ByteRange),
    /// No authored source.
    Synthesized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateProvenance {
    AuthoredByTriggerToken,
    Synthesized,
}

/// Which emitted token caused an observation. Never an authored origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateTrigger {
    Authored {
        index: usize,
        range: ByteRange,
    },
    /// End of file has no authored extent and gets no dummy span.
    EndOfFile {
        index: usize,
    },
}

impl CandidateTrigger {
    fn index(&self) -> usize {
        match self {
            Self::Authored { index, .. } | Self::EndOfFile { index } => *index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDiagnosticCode {
    MissingDoctype,
    /// The successor's own `after after body` recovery fact.
    AfterAfterBodyCharacterData,
    /// The accepted `in body` U+0000 fact. Distinct from the recovery above.
    NullCharacterInBody,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateRecovery {
    ContinuedInQuirksDocumentMode,
    SwitchedToInBodyAndReprocessedSameToken,
    IgnoredToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateDiagnostic {
    code: CandidateDiagnosticCode,
    trigger: CandidateTrigger,
    recovery: CandidateRecovery,
}

/// What one rule dispatch did with the current token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDispatchOutcome {
    Consumed,
    Reprocessed,
    Stopped,
    /// Nothing was mutated by this cell.
    Refused(CandidateUnsupported),
}

/// One evaluation of one insertion-mode rule for one emitted token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateDispatch {
    /// The candidate's actual insertion mode when the rule was selected.
    evaluated_in: CandidateMode,
    /// A rule set borrowed without mutating the actual insertion mode. This is
    /// what keeps whitespace delegation distinct from a mode transition.
    delegated_rule_set: Option<CandidateMode>,
    outcome: CandidateDispatchOutcome,
}

/// Everything one emitted token did.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateTokenRecord {
    index: usize,
    mode_before: CandidateMode,
    mode_after: CandidateMode,
    dispatches: Vec<CandidateDispatch>,
    /// Same-token reprocessing count. Not a budget: an observation.
    reprocesses: usize,
    committed_prefix_end: usize,
}

impl CandidateTokenRecord {
    fn refusal(&self) -> Option<CandidateUnsupported> {
        match self.dispatches.last()?.outcome {
            CandidateDispatchOutcome::Refused(capability) => Some(capability),
            _ => None,
        }
    }

    fn stopped(&self) -> bool {
        matches!(
            self.dispatches.last().map(|dispatch| dispatch.outcome),
            Some(CandidateDispatchOutcome::Stopped)
        )
    }
}

/// The candidate's final tree shape, projected for comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTree {
    Document(Vec<CandidateTree>),
    Element {
        name: CandidateShellName,
        origin: CandidateOrigin,
        children: Vec<CandidateTree>,
    },
    Text {
        interpreted: String,
        /// Ordered, individually retained source contributions.
        contributions: Vec<ByteRange>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateCompletion {
    Complete,
    /// The candidate stopped at exactly this capability and trigger.
    IncompleteUnsupported {
        capability: CandidateUnsupported,
        trigger: CandidateTrigger,
    },
    /// Lower-layer evidence was not complete, so the candidate cannot be.
    IncompleteLowerLayer,
}

/// The terminal candidate checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateCheckpoint {
    mode: CandidateMode,
    committed_prefix_end: usize,
    completion: CandidateCompletion,
}

/// The complete independent observation of one candidate run.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateObservation {
    tree: CandidateTree,
    diagnostics: Vec<CandidateDiagnostic>,
    /// Indexes of emitted tokens given the ignored-token disposition. Distinct
    /// from diagnostics, from text provenance, and from tokenizer evidence.
    ignored_tokens: Vec<usize>,
    tokens: Vec<CandidateTokenRecord>,
    /// Semantic creation events. Coalescing consumes none. Never a raw ID.
    identity_events: usize,
    checkpoint: CandidateCheckpoint,
}

// ---------------------------------------------------------------------------
// Test-only deliberately wrong candidates
// ---------------------------------------------------------------------------

/// A deliberately wrong candidate semantics. Each variant is one falsified
/// assumption. The validator must reject every one of them; none is a proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutation {
    /// "Every AfterAfterBody character token simply delegates to InBody."
    DelegateEveryCharacterToken,
    /// "Every AfterAfterBody character token is parse error + reprocess."
    RecoverEveryCharacterToken,
    /// "A mixed aggregate run is safely handled as one non-whitespace unit."
    AcceptMixedRunAsNonWhitespace,
    /// "Whitespace delegation changes the actual insertion mode."
    WhitespaceSwitchesActualMode,
    /// "Recovery may return the same token to the same mode."
    RecoverWithoutModeSwitch,
    /// "`xy` needs one recovery per scalar."
    RecoveryPerScalar,
    /// "`</html>\0` becomes U+0000 text after reprocessing."
    NulBecomesText,
    /// "`</html>\0` becomes fabricated U+FFFD text."
    NulBecomesReplacementText,
    /// "The recovery fact and the ignored-NUL fact are one fact."
    NulOmitsAfterAfterBodyRecovery,
    /// "The ignored-NUL disposition is optional."
    NulOmitsIgnoredDisposition,
    /// "Source endpoints may be inferred from interpreted length."
    ContributionFromInterpretedLength,
    /// "Lower-layer incomplete evidence may be reported Complete."
    UpgradeLowerLayerIncomplete,
    /// "The historical #353 refusal is still the answer."
    HistoricalPredecessorRefusal,
}

const ALL_MUTATIONS: [Mutation; 13] = [
    Mutation::DelegateEveryCharacterToken,
    Mutation::RecoverEveryCharacterToken,
    Mutation::AcceptMixedRunAsNonWhitespace,
    Mutation::WhitespaceSwitchesActualMode,
    Mutation::RecoverWithoutModeSwitch,
    Mutation::RecoveryPerScalar,
    Mutation::NulBecomesText,
    Mutation::NulBecomesReplacementText,
    Mutation::NulOmitsAfterAfterBodyRecovery,
    Mutation::NulOmitsIgnoredDisposition,
    Mutation::ContributionFromInterpretedLength,
    Mutation::UpgradeLowerLayerIncomplete,
    Mutation::HistoricalPredecessorRefusal,
];

// ---------------------------------------------------------------------------
// Independent candidate machine
// ---------------------------------------------------------------------------

/// One effect a candidate rule commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateEffect {
    RecordMissingDoctype,
    InsertShellElement {
        name: CandidateShellName,
        provenance: CandidateProvenance,
    },
    CloseHeadElement,
    /// Disposition evidence only: creates no node, no text, no identity.
    AcknowledgeShellEndTag(CandidateShellName),
    InsertCharacters,
    /// Only a deliberately wrong model ever selects this.
    InsertReplacementCharacter,
    RecordAfterAfterBodyCharacterData,
    IgnoreNullCharacter,
}

/// What a candidate rule does with the current token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateStep {
    Consume {
        effect: Option<CandidateEffect>,
        next: Option<CandidateMode>,
    },
    Reprocess {
        effect: Option<CandidateEffect>,
        next: CandidateMode,
    },
    /// Apply the selected `in body` character rule while the actual insertion
    /// mode stays unchanged. This is the whitespace delegation.
    DelegateInBodyCharacterRule,
    Stop,
}

/// The selected `in body` character cell: exactly U+0000 is ignored with its
/// own diagnostic; every other character token is inserted. Tokenizer
/// recognition already isolates an authored NUL, so an aggregate is never
/// scanned for an embedded one.
fn in_body_character_step(
    interpreted: &str,
    mutation: Option<Mutation>,
) -> Result<CandidateStep, CandidateUnsupported> {
    if interpreted == "\0" {
        let effect = match mutation {
            Some(Mutation::NulBecomesText) => CandidateEffect::InsertCharacters,
            Some(Mutation::NulBecomesReplacementText) => {
                CandidateEffect::InsertReplacementCharacter
            }
            _ => CandidateEffect::IgnoreNullCharacter,
        };
        return Ok(CandidateStep::Consume {
            effect: Some(effect),
            next: None,
        });
    }
    if interpreted.contains('\0') {
        // Unreachable through the accepted tokenizer; refuse rather than guess.
        return Err(CandidateUnsupported::OutsideModelledCandidateCells);
    }
    Ok(CandidateStep::Consume {
        effect: Some(CandidateEffect::InsertCharacters),
        next: None,
    })
}

/// Selects the candidate rule for one (mode, token) cell.
///
/// Pure: it takes no session state and mutates nothing, so a refusal is
/// structurally guaranteed to precede mutation by that cell.
fn select(
    mode: CandidateMode,
    shape: &CandidateTokenShape<'_>,
    mutation: Option<Mutation>,
) -> Result<CandidateStep, CandidateUnsupported> {
    match mode {
        CandidateMode::Initial => {
            expect_body_start_tag(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::RecordMissingDoctype),
                next: CandidateMode::BeforeHtml,
            })
        }
        CandidateMode::BeforeHtml => {
            expect_body_start_tag(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::InsertShellElement {
                    name: CandidateShellName::Html,
                    provenance: CandidateProvenance::Synthesized,
                }),
                next: CandidateMode::BeforeHead,
            })
        }
        CandidateMode::BeforeHead => {
            expect_body_start_tag(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::InsertShellElement {
                    name: CandidateShellName::Head,
                    provenance: CandidateProvenance::Synthesized,
                }),
                next: CandidateMode::InHead,
            })
        }
        CandidateMode::InHead => {
            expect_body_start_tag(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::CloseHeadElement),
                next: CandidateMode::AfterHead,
            })
        }
        CandidateMode::AfterHead => {
            expect_body_start_tag(shape)?;
            Ok(CandidateStep::Consume {
                effect: Some(CandidateEffect::InsertShellElement {
                    name: CandidateShellName::Body,
                    provenance: CandidateProvenance::AuthoredByTriggerToken,
                }),
                next: Some(CandidateMode::InBody),
            })
        }
        // `in body` consumes every character token unconditionally. This is
        // the structural fact the termination theorem rests on.
        CandidateMode::InBody => match shape {
            CandidateTokenShape::Characters { interpreted, .. } => {
                in_body_character_step(interpreted, mutation)
            }
            CandidateTokenShape::EndTag {
                name: CandidateShellName::Body,
                ..
            } => Ok(CandidateStep::Consume {
                effect: Some(CandidateEffect::AcknowledgeShellEndTag(
                    CandidateShellName::Body,
                )),
                next: Some(CandidateMode::AfterBody),
            }),
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop),
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
        // `after body` character data is the accepted TC-S2 theorem and is not
        // part of this successor.
        CandidateMode::AfterBody => match shape {
            CandidateTokenShape::EndTag {
                name: CandidateShellName::Html,
                ..
            } => Ok(CandidateStep::Consume {
                effect: Some(CandidateEffect::AcknowledgeShellEndTag(
                    CandidateShellName::Html,
                )),
                next: Some(CandidateMode::AfterAfterBody),
            }),
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop),
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
        // The successor frontier.
        CandidateMode::AfterAfterBody => match shape {
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop),
            CandidateTokenShape::Characters { interpreted, .. } => {
                select_after_after_body_characters(interpreted, mutation)
            }
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
    }
}

fn select_after_after_body_characters(
    interpreted: &str,
    mutation: Option<Mutation>,
) -> Result<CandidateStep, CandidateUnsupported> {
    let recovery = CandidateStep::Reprocess {
        effect: Some(CandidateEffect::RecordAfterAfterBodyCharacterData),
        next: CandidateMode::InBody,
    };
    match mutation {
        Some(Mutation::DelegateEveryCharacterToken) => {
            return Ok(CandidateStep::DelegateInBodyCharacterRule);
        }
        Some(Mutation::RecoverEveryCharacterToken) => return Ok(recovery),
        Some(Mutation::HistoricalPredecessorRefusal) => {
            return Err(CandidateUnsupported::HistoricalAfterAfterBodyCharacterData);
        }
        Some(Mutation::RecoverWithoutModeSwitch)
            if classify_run(interpreted) == CandidateRunClass::AllNonWhitespace =>
        {
            return Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::RecordAfterAfterBodyCharacterData),
                next: CandidateMode::AfterAfterBody,
            });
        }
        _ => {}
    }
    match classify_run(interpreted) {
        CandidateRunClass::AllWhitespace => Ok(CandidateStep::DelegateInBodyCharacterRule),
        CandidateRunClass::AllNonWhitespace => Ok(recovery),
        CandidateRunClass::Mixed if mutation == Some(Mutation::AcceptMixedRunAsNonWhitespace) => {
            Ok(recovery)
        }
        CandidateRunClass::Mixed => Err(CandidateUnsupported::MixedAfterAfterBodyCharacterRun),
    }
}

/// Every fixture opens with a `body` start tag walking the shell. Any other
/// token in the early modes is outside this deliberately partial model.
fn expect_body_start_tag(shape: &CandidateTokenShape<'_>) -> Result<(), CandidateUnsupported> {
    match shape {
        CandidateTokenShape::StartTag {
            name: CandidateShellName::Body,
            ..
        } => Ok(()),
        _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
    }
}

/// A node in the candidate's construction arena. Storage positions are private
/// working state and never become durable meaning.
#[derive(Debug, Clone)]
enum CandidateArenaKind {
    Document,
    Element {
        name: CandidateShellName,
        origin: CandidateOrigin,
    },
    Text {
        interpreted: String,
        contributions: Vec<ByteRange>,
    },
}

#[derive(Debug, Clone)]
struct CandidateArenaNode {
    children: Vec<usize>,
    kind: CandidateArenaKind,
}

/// The test-only candidate construction machine.
struct CandidateSession {
    nodes: Vec<CandidateArenaNode>,
    open_elements: Vec<usize>,
    head_element: Option<usize>,
    mode: CandidateMode,
    diagnostics: Vec<CandidateDiagnostic>,
    ignored_tokens: Vec<usize>,
    identity_events: usize,
    committed_prefix_end: usize,
    processed_tokens: usize,
    mutation: Option<Mutation>,
}

impl CandidateSession {
    fn new(mutation: Option<Mutation>) -> Self {
        Self {
            nodes: vec![CandidateArenaNode {
                children: Vec::new(),
                kind: CandidateArenaKind::Document,
            }],
            open_elements: Vec::new(),
            head_element: None,
            mode: CandidateMode::Initial,
            diagnostics: Vec::new(),
            ignored_tokens: Vec::new(),
            // The Document container is one semantic creation event.
            identity_events: 1,
            committed_prefix_end: 0,
            processed_tokens: 0,
            mutation,
        }
    }

    /// Processes one emitted token to a terminal disposition.
    ///
    /// Termination is structural: an insertion mode is never evaluated twice
    /// for the same token. The assertion *is* the theorem obligation. If the
    /// candidate action set admitted a same-token cycle, this would fire
    /// instead of looping, and the successor would be falsified.
    fn process(
        &mut self,
        index: usize,
        shape: CandidateTokenShape<'_>,
        trigger: CandidateTrigger,
    ) -> CandidateTokenRecord {
        let mode_before = self.mode;
        let mut dispatches = Vec::new();
        let mut visited: Vec<CandidateMode> = Vec::new();
        let mut reprocesses = 0;

        loop {
            assert!(
                !visited.contains(&self.mode),
                "candidate theorem falsified: insertion mode {:?} was evaluated twice while \
                 processing token {index}",
                self.mode
            );
            visited.push(self.mode);

            match select(self.mode, &shape, self.mutation) {
                Err(capability) => {
                    dispatches.push(CandidateDispatch {
                        evaluated_in: self.mode,
                        delegated_rule_set: None,
                        outcome: CandidateDispatchOutcome::Refused(capability),
                    });
                    break;
                }
                Ok(CandidateStep::Stop) => {
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch {
                        evaluated_in: self.mode,
                        delegated_rule_set: None,
                        outcome: CandidateDispatchOutcome::Stopped,
                    });
                    break;
                }
                Ok(CandidateStep::Consume { effect, next }) => {
                    let evaluated_in = self.mode;
                    if let Some(effect) = effect {
                        self.apply(effect, trigger, &shape);
                    }
                    if let Some(next) = next {
                        self.mode = next;
                    }
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        delegated_rule_set: None,
                        outcome: CandidateDispatchOutcome::Consumed,
                    });
                    break;
                }
                Ok(CandidateStep::DelegateInBodyCharacterRule) => {
                    let evaluated_in = self.mode;
                    self.apply(CandidateEffect::InsertCharacters, trigger, &shape);
                    if self.mutation == Some(Mutation::WhitespaceSwitchesActualMode) {
                        self.mode = CandidateMode::InBody;
                    }
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        delegated_rule_set: Some(CandidateMode::InBody),
                        outcome: CandidateDispatchOutcome::Consumed,
                    });
                    break;
                }
                Ok(CandidateStep::Reprocess { effect, next }) => {
                    let evaluated_in = self.mode;
                    if let Some(effect) = effect {
                        self.apply(effect, trigger, &shape);
                    }
                    self.mode = next;
                    reprocesses += 1;
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        delegated_rule_set: None,
                        outcome: CandidateDispatchOutcome::Reprocessed,
                    });
                }
            }
        }

        CandidateTokenRecord {
            index,
            mode_before,
            mode_after: self.mode,
            dispatches,
            reprocesses,
            committed_prefix_end: self.committed_prefix_end,
        }
    }

    fn apply(
        &mut self,
        effect: CandidateEffect,
        trigger: CandidateTrigger,
        shape: &CandidateTokenShape<'_>,
    ) {
        match effect {
            CandidateEffect::RecordMissingDoctype => self.diagnostics.push(CandidateDiagnostic {
                code: CandidateDiagnosticCode::MissingDoctype,
                trigger,
                recovery: CandidateRecovery::ContinuedInQuirksDocumentMode,
            }),
            CandidateEffect::RecordAfterAfterBodyCharacterData => {
                let CandidateTokenShape::Characters { interpreted, .. } = shape else {
                    panic!("after after body recovery requires a character token")
                };
                let nul_run = *interpreted == "\0";
                if nul_run && self.mutation == Some(Mutation::NulOmitsAfterAfterBodyRecovery) {
                    return;
                }
                // One recovery unit per emitted token. Only the deliberately
                // wrong per-scalar model multiplies it.
                let units = if self.mutation == Some(Mutation::RecoveryPerScalar) {
                    interpreted.chars().count()
                } else {
                    1
                };
                for _ in 0..units {
                    self.diagnostics.push(CandidateDiagnostic {
                        code: CandidateDiagnosticCode::AfterAfterBodyCharacterData,
                        trigger,
                        recovery: CandidateRecovery::SwitchedToInBodyAndReprocessedSameToken,
                    });
                }
            }
            CandidateEffect::IgnoreNullCharacter => {
                if self.mutation == Some(Mutation::NulOmitsIgnoredDisposition) {
                    return;
                }
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::NullCharacterInBody,
                    trigger,
                    recovery: CandidateRecovery::IgnoredToken,
                });
                self.ignored_tokens.push(trigger.index());
            }
            CandidateEffect::InsertShellElement { name, provenance } => {
                self.insert_shell_element(name, provenance, shape);
            }
            CandidateEffect::CloseHeadElement => {
                let head = self.head_element.expect("an open head element");
                assert_eq!(
                    self.open_elements.last(),
                    Some(&head),
                    "head must be the open element when it is closed"
                );
                self.open_elements.pop();
            }
            // Acknowledgement is disposition evidence only.
            CandidateEffect::AcknowledgeShellEndTag(_) => {}
            CandidateEffect::InsertCharacters => self.insert_characters(shape, None),
            CandidateEffect::InsertReplacementCharacter => {
                self.insert_characters(shape, Some("\u{fffd}"));
            }
        }
    }

    fn insert_shell_element(
        &mut self,
        name: CandidateShellName,
        provenance: CandidateProvenance,
        shape: &CandidateTokenShape<'_>,
    ) {
        let parent = match name {
            CandidateShellName::Html => 0,
            CandidateShellName::Head | CandidateShellName::Body => *self
                .open_elements
                .last()
                .expect("an open insertion parent for a nested shell element"),
        };
        let origin = match provenance {
            CandidateProvenance::AuthoredByTriggerToken => {
                let CandidateTokenShape::StartTag { range, .. } = shape else {
                    panic!("authored insertion requires the trigger token's own start tag")
                };
                CandidateOrigin::Authored(*range)
            }
            CandidateProvenance::Synthesized => CandidateOrigin::Synthesized,
        };
        let inserted = self.nodes.len();
        self.nodes.push(CandidateArenaNode {
            children: Vec::new(),
            kind: CandidateArenaKind::Element { name, origin },
        });
        self.nodes[parent].children.push(inserted);
        self.open_elements.push(inserted);
        self.identity_events += 1;
        if name == CandidateShellName::Head {
            self.head_element = Some(inserted);
        }
    }

    /// Inserts the token's characters at the current node, which is the still
    /// open `body` (the stack is untouched by `</body>` and `</html>`),
    /// coalescing into the adjacent text node when one is the last child.
    ///
    /// Coalescing appends an ordered contribution and consumes no new identity
    /// event. The contribution is the token's own anchor, never an endpoint
    /// derived from interpreted length, and contributions are never merged.
    fn insert_characters(&mut self, shape: &CandidateTokenShape<'_>, replace_with: Option<&str>) {
        let CandidateTokenShape::Characters { range, interpreted } = shape else {
            panic!("character insertion requires a character token")
        };
        let value = replace_with.unwrap_or(interpreted);
        let contribution = if self.mutation == Some(Mutation::ContributionFromInterpretedLength) {
            (range.0, range.0 + interpreted.len())
        } else {
            *range
        };
        let parent = *self
            .open_elements
            .last()
            .expect("an open insertion target for character data");
        let adjacent_text = self.nodes[parent]
            .children
            .last()
            .copied()
            .filter(|child| matches!(self.nodes[*child].kind, CandidateArenaKind::Text { .. }));

        if let Some(text) = adjacent_text {
            let CandidateArenaKind::Text {
                interpreted: existing,
                contributions,
            } = &mut self.nodes[text].kind
            else {
                unreachable!("filtered to a text node")
            };
            existing.push_str(value);
            contributions.push(contribution);
            return;
        }

        let inserted = self.nodes.len();
        self.nodes.push(CandidateArenaNode {
            children: Vec::new(),
            kind: CandidateArenaKind::Text {
                interpreted: value.to_owned(),
                contributions: vec![contribution],
            },
        });
        self.nodes[parent].children.push(inserted);
        self.identity_events += 1;
    }

    fn commit(&mut self, shape: &CandidateTokenShape<'_>) {
        let end = shape.committed_end();
        assert!(
            end >= self.committed_prefix_end,
            "committed candidate coverage must not move backwards"
        );
        self.committed_prefix_end = end;
        self.processed_tokens += 1;
    }

    fn tree(&self) -> CandidateTree {
        self.project(0)
    }

    fn project(&self, node: usize) -> CandidateTree {
        let children = || {
            self.nodes[node]
                .children
                .iter()
                .map(|child| self.project(*child))
                .collect()
        };
        match &self.nodes[node].kind {
            CandidateArenaKind::Document => CandidateTree::Document(children()),
            CandidateArenaKind::Element { name, origin } => CandidateTree::Element {
                name: *name,
                origin: *origin,
                children: children(),
            },
            CandidateArenaKind::Text {
                interpreted,
                contributions,
            } => CandidateTree::Text {
                interpreted: interpreted.clone(),
                contributions: contributions.clone(),
            },
        }
    }
}

/// Normalizes one accepted lower-layer token into the candidate's shapes.
///
/// Pure and mutation-free, so a refusal here also precedes any mutation.
fn candidate_shape(token: &HtmlToken) -> Result<CandidateTokenShape<'_>, CandidateUnsupported> {
    match token {
        HtmlToken::Character(character) => Ok(CandidateTokenShape::Characters {
            range: span(character.source()),
            interpreted: character.interpreted(),
        }),
        HtmlToken::Tag(tag) => {
            let name = match tag.name().interpreted() {
                "html" => CandidateShellName::Html,
                "head" => CandidateShellName::Head,
                "body" => CandidateShellName::Body,
                _ => return Err(CandidateUnsupported::OutsideModelledCandidateCells),
            };
            if !tag.attributes().is_empty() || tag.self_closing_solidus().is_some() {
                return Err(CandidateUnsupported::OutsideModelledCandidateCells);
            }
            let range = span(tag.complete());
            match tag.kind() {
                HtmlTagKind::Start => Ok(CandidateTokenShape::StartTag { name, range }),
                HtmlTagKind::End => Ok(CandidateTokenShape::EndTag { name, range }),
            }
        }
        HtmlToken::EndOfFile(end_of_file) => Ok(CandidateTokenShape::EndOfFile {
            at: end_of_file.source().range().start(),
        }),
    }
}

fn span(anchor: &crate::SourceAnchor) -> ByteRange {
    (anchor.range().start(), anchor.range().end())
}

fn candidate_trigger(token: &HtmlToken, index: usize) -> CandidateTrigger {
    match token {
        HtmlToken::Character(character) => CandidateTrigger::Authored {
            index,
            range: span(character.source()),
        },
        HtmlToken::Tag(tag) => CandidateTrigger::Authored {
            index,
            range: span(tag.complete()),
        },
        HtmlToken::EndOfFile(_) => CandidateTrigger::EndOfFile { index },
    }
}

/// Runs a candidate model over one accepted lower-layer run.
///
/// Effective completion is authored here from the candidate theorem:
/// lower-layer incompleteness of any cause is never upgraded, and a candidate
/// refusal is reported as the candidate's own typed evidence.
fn observe_with(run: &HtmlTokenizerRunResult, mutation: Option<Mutation>) -> CandidateObservation {
    let mut session = CandidateSession::new(mutation);
    let mut tokens = Vec::new();
    let mut refusal = None;
    let mut stopped = false;

    for (index, token) in run.tokens().iter().enumerate() {
        let trigger = candidate_trigger(token, index);
        let shape = match candidate_shape(token) {
            Ok(shape) => shape,
            Err(capability) => {
                tokens.push(CandidateTokenRecord {
                    index,
                    mode_before: session.mode,
                    mode_after: session.mode,
                    dispatches: vec![CandidateDispatch {
                        evaluated_in: session.mode,
                        delegated_rule_set: None,
                        outcome: CandidateDispatchOutcome::Refused(capability),
                    }],
                    reprocesses: 0,
                    committed_prefix_end: session.committed_prefix_end,
                });
                refusal = Some((capability, trigger));
                break;
            }
        };
        let record = session.process(index, shape, trigger);
        let stop = record.stopped();
        if let Some(capability) = record.refusal() {
            refusal = Some((capability, trigger));
        }
        let refused = record.refusal().is_some();
        tokens.push(record);
        if refused {
            break;
        }
        if stop {
            stopped = true;
            break;
        }
    }

    let completion = match refusal {
        Some((capability, trigger)) => CandidateCompletion::IncompleteUnsupported {
            capability,
            trigger,
        },
        None if mutation == Some(Mutation::UpgradeLowerLayerIncomplete) => {
            CandidateCompletion::Complete
        }
        None if stopped
            && session.processed_tokens == run.tokens().len()
            && !run.is_incomplete() =>
        {
            CandidateCompletion::Complete
        }
        None => CandidateCompletion::IncompleteLowerLayer,
    };

    CandidateObservation {
        tree: session.tree(),
        diagnostics: session.diagnostics.clone(),
        ignored_tokens: session.ignored_tokens.clone(),
        tokens,
        identity_events: session.identity_events,
        checkpoint: CandidateCheckpoint {
            mode: session.mode,
            committed_prefix_end: session.committed_prefix_end,
            completion,
        },
    }
}

fn observe(run: &HtmlTokenizerRunResult) -> CandidateObservation {
    observe_with(run, None)
}

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn tokenize_text(
    text: &str,
    source_id: u64,
    limits: HtmlTokenizerLimits,
) -> HtmlTokenizerRunResult {
    let source = SourceText::new(SourceId::new(source_id), text.to_owned());
    tokenize(&source, limits)
}

fn run_of(id: &str) -> HtmlTokenizerRunResult {
    tokenize_text(fixture(id).source_text(), 1, generous_limits())
}

/// Observes one canonical fixture through the accepted lower layer.
fn observe_fixture(id: &str) -> CandidateObservation {
    observe(&run_of(id))
}

// ---------------------------------------------------------------------------
// Authored candidate GOLD
// ---------------------------------------------------------------------------

fn document(children: Vec<CandidateTree>) -> CandidateTree {
    CandidateTree::Document(children)
}

fn synthesized(name: CandidateShellName, children: Vec<CandidateTree>) -> CandidateTree {
    CandidateTree::Element {
        name,
        origin: CandidateOrigin::Synthesized,
        children,
    }
}

fn authored(
    name: CandidateShellName,
    complete: ByteRange,
    children: Vec<CandidateTree>,
) -> CandidateTree {
    CandidateTree::Element {
        name,
        origin: CandidateOrigin::Authored(complete),
        children,
    }
}

fn text(interpreted: &str, contributions: &[ByteRange]) -> CandidateTree {
    CandidateTree::Text {
        interpreted: interpreted.to_owned(),
        contributions: contributions.to_vec(),
    }
}

/// Every fixture opens with the same authored `body` start tag at `[0,6)`,
/// which implies the `html` and `head` shell.
fn shell(body_children: Vec<CandidateTree>) -> CandidateTree {
    document(vec![synthesized(
        CandidateShellName::Html,
        vec![
            synthesized(CandidateShellName::Head, vec![]),
            authored(CandidateShellName::Body, (0, 6), body_children),
        ],
    )])
}

fn consumed_in(mode: CandidateMode) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        delegated_rule_set: None,
        outcome: CandidateDispatchOutcome::Consumed,
    }
}

fn reprocessed_in(mode: CandidateMode) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        delegated_rule_set: None,
        outcome: CandidateDispatchOutcome::Reprocessed,
    }
}

fn delegated_in(mode: CandidateMode, rules: CandidateMode) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        delegated_rule_set: Some(rules),
        outcome: CandidateDispatchOutcome::Consumed,
    }
}

fn stopped_in(mode: CandidateMode) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        delegated_rule_set: None,
        outcome: CandidateDispatchOutcome::Stopped,
    }
}

fn refused_in(mode: CandidateMode, capability: CandidateUnsupported) -> CandidateDispatch {
    CandidateDispatch {
        evaluated_in: mode,
        delegated_rule_set: None,
        outcome: CandidateDispatchOutcome::Refused(capability),
    }
}

/// The shared token 0 record: `<body>` walks `Initial` to `InBody`.
fn body_start_tag_record() -> CandidateTokenRecord {
    CandidateTokenRecord {
        index: 0,
        mode_before: CandidateMode::Initial,
        mode_after: CandidateMode::InBody,
        dispatches: vec![
            reprocessed_in(CandidateMode::Initial),
            reprocessed_in(CandidateMode::BeforeHtml),
            reprocessed_in(CandidateMode::BeforeHead),
            reprocessed_in(CandidateMode::InHead),
            consumed_in(CandidateMode::AfterHead),
        ],
        reprocesses: 4,
        committed_prefix_end: 6,
    }
}

/// A token consumed by a single rule dispatch.
fn simple_record(
    index: usize,
    mode_before: CandidateMode,
    mode_after: CandidateMode,
    dispatch: CandidateDispatch,
    committed_prefix_end: usize,
) -> CandidateTokenRecord {
    CandidateTokenRecord {
        index,
        mode_before,
        mode_after,
        dispatches: vec![dispatch],
        reprocesses: 0,
        committed_prefix_end,
    }
}

/// The successor recovery record: one recovery fact, one mode switch, one
/// same-token reprocess, then the selected `in body` rule consumes the token.
fn recovery_record(index: usize, committed_prefix_end: usize) -> CandidateTokenRecord {
    CandidateTokenRecord {
        index,
        mode_before: CandidateMode::AfterAfterBody,
        mode_after: CandidateMode::InBody,
        dispatches: vec![
            reprocessed_in(CandidateMode::AfterAfterBody),
            consumed_in(CandidateMode::InBody),
        ],
        reprocesses: 1,
        committed_prefix_end,
    }
}

/// `</body>` consumed in `in body`, entering `after body`.
fn body_end_record(index: usize, committed_prefix_end: usize) -> CandidateTokenRecord {
    simple_record(
        index,
        CandidateMode::InBody,
        CandidateMode::AfterBody,
        consumed_in(CandidateMode::InBody),
        committed_prefix_end,
    )
}

/// `</html>` consumed in `after body`, entering `after after body`.
fn html_end_record(index: usize, committed_prefix_end: usize) -> CandidateTokenRecord {
    simple_record(
        index,
        CandidateMode::AfterBody,
        CandidateMode::AfterAfterBody,
        consumed_in(CandidateMode::AfterBody),
        committed_prefix_end,
    )
}

/// A whitespace token delegated while the actual mode stays `after after body`.
fn delegated_record(index: usize, committed_prefix_end: usize) -> CandidateTokenRecord {
    simple_record(
        index,
        CandidateMode::AfterAfterBody,
        CandidateMode::AfterAfterBody,
        delegated_in(CandidateMode::AfterAfterBody, CandidateMode::InBody),
        committed_prefix_end,
    )
}

/// A token consumed by `in body` after the mode was already `in body`.
fn in_body_consumed_record(index: usize, committed_prefix_end: usize) -> CandidateTokenRecord {
    simple_record(
        index,
        CandidateMode::InBody,
        CandidateMode::InBody,
        consumed_in(CandidateMode::InBody),
        committed_prefix_end,
    )
}

fn eof_record(
    index: usize,
    mode: CandidateMode,
    committed_prefix_end: usize,
) -> CandidateTokenRecord {
    simple_record(index, mode, mode, stopped_in(mode), committed_prefix_end)
}

fn refused_mixed_record(index: usize, committed_prefix_end: usize) -> CandidateTokenRecord {
    simple_record(
        index,
        CandidateMode::AfterAfterBody,
        CandidateMode::AfterAfterBody,
        refused_in(
            CandidateMode::AfterAfterBody,
            CandidateUnsupported::MixedAfterAfterBodyCharacterRun,
        ),
        committed_prefix_end,
    )
}

/// Tokens 0..=2 of `<body></body></html>`.
fn plain_prefix() -> Vec<CandidateTokenRecord> {
    vec![
        body_start_tag_record(),
        body_end_record(1, 13),
        html_end_record(2, 20),
    ]
}

/// Tokens 0..=3 of `<body>a</body></html>`.
fn text_prefix() -> Vec<CandidateTokenRecord> {
    vec![
        body_start_tag_record(),
        in_body_consumed_record(1, 7),
        body_end_record(2, 14),
        html_end_record(3, 21),
    ]
}

fn chain(
    mut head: Vec<CandidateTokenRecord>,
    tail: Vec<CandidateTokenRecord>,
) -> Vec<CandidateTokenRecord> {
    head.extend(tail);
    head
}

fn missing_doctype() -> CandidateDiagnostic {
    CandidateDiagnostic {
        code: CandidateDiagnosticCode::MissingDoctype,
        trigger: CandidateTrigger::Authored {
            index: 0,
            range: (0, 6),
        },
        recovery: CandidateRecovery::ContinuedInQuirksDocumentMode,
    }
}

fn after_after_body_recovery(index: usize, range: ByteRange) -> CandidateDiagnostic {
    CandidateDiagnostic {
        code: CandidateDiagnosticCode::AfterAfterBodyCharacterData,
        trigger: CandidateTrigger::Authored { index, range },
        recovery: CandidateRecovery::SwitchedToInBodyAndReprocessedSameToken,
    }
}

fn ignored_null(index: usize, range: ByteRange) -> CandidateDiagnostic {
    CandidateDiagnostic {
        code: CandidateDiagnosticCode::NullCharacterInBody,
        trigger: CandidateTrigger::Authored { index, range },
        recovery: CandidateRecovery::IgnoredToken,
    }
}

fn complete_at(mode: CandidateMode, committed_prefix_end: usize) -> CandidateCheckpoint {
    CandidateCheckpoint {
        mode,
        committed_prefix_end,
        completion: CandidateCompletion::Complete,
    }
}

fn refused_mixed_at(
    committed_prefix_end: usize,
    trigger_index: usize,
    trigger_range: ByteRange,
) -> CandidateCheckpoint {
    CandidateCheckpoint {
        mode: CandidateMode::AfterAfterBody,
        committed_prefix_end,
        completion: CandidateCompletion::IncompleteUnsupported {
            capability: CandidateUnsupported::MixedAfterAfterBodyCharacterRun,
            trigger: CandidateTrigger::Authored {
                index: trigger_index,
                range: trigger_range,
            },
        },
    }
}

/// A whitespace run delegated at token 3 of the plain prefix, then EOF.
fn whitespace_gold(interpreted: &str, run: ByteRange) -> CandidateObservation {
    CandidateObservation {
        tree: shell(vec![text(interpreted, &[run])]),
        diagnostics: vec![missing_doctype()],
        ignored_tokens: vec![],
        tokens: chain(
            plain_prefix(),
            vec![
                delegated_record(3, run.1),
                eof_record(4, CandidateMode::AfterAfterBody, run.1),
            ],
        ),
        identity_events: 5,
        checkpoint: complete_at(CandidateMode::AfterAfterBody, run.1),
    }
}

/// A non-whitespace run recovered at token 3 of the plain prefix, then EOF.
fn recovery_gold(interpreted: &str, run: ByteRange) -> CandidateObservation {
    CandidateObservation {
        tree: shell(vec![text(interpreted, &[run])]),
        diagnostics: vec![missing_doctype(), after_after_body_recovery(3, run)],
        ignored_tokens: vec![],
        tokens: chain(
            plain_prefix(),
            vec![
                recovery_record(3, run.1),
                eof_record(4, CandidateMode::InBody, run.1),
            ],
        ),
        identity_events: 5,
        checkpoint: complete_at(CandidateMode::InBody, run.1),
    }
}

/// A mixed run refused at token 3 of the plain prefix. Nothing was mutated.
fn plain_mixed_gold(run: ByteRange) -> CandidateObservation {
    CandidateObservation {
        tree: shell(vec![]),
        diagnostics: vec![missing_doctype()],
        ignored_tokens: vec![],
        tokens: chain(plain_prefix(), vec![refused_mixed_record(3, 20)]),
        identity_events: 4,
        checkpoint: refused_mixed_at(20, 3, run),
    }
}

/// The authored successor candidate GOLD for one fixture.
fn candidate_gold(id: &str) -> CandidateObservation {
    match id {
        // All-whitespace: delegated `in body` rule, actual mode stays
        // `AfterAfterBody`, no recovery diagnostic, no reprocess.
        "W1" => whitespace_gold(" ", (20, 21)),
        "W2" => whitespace_gold("\t", (20, 21)),
        "W3" => whitespace_gold("\n", (20, 21)),
        "W4" => whitespace_gold("\u{000c}", (20, 21)),
        // Authored CR: one source byte, interpreted as LF.
        "W5" => whitespace_gold("\n", (20, 21)),
        // Authored CRLF: two source bytes, one interpreted LF. The exact
        // contribution is the token's own two-byte anchor.
        "W6" => whitespace_gold("\n", (20, 22)),
        "W7" => whitespace_gold(" \t", (20, 22)),
        // All-non-whitespace: one recovery, one same-token reprocess.
        "N1" => recovery_gold("x", (20, 21)),
        "N2" => recovery_gold("xy", (20, 22)),
        "N3" => recovery_gold("\u{00a0}", (20, 22)),
        "N4" => recovery_gold("\u{000b}", (20, 21)),
        // Mixed aggregate tokens: refused whole, before mutation.
        "M1" | "M2" => plain_mixed_gold((20, 22)),
        "M3" | "M4" => plain_mixed_gold((20, 23)),
        // Authored U+0000: recovery, then the accepted `in body` ignore.
        // No text, no identity, and no fabricated U+FFFD.
        "Z1" => CandidateObservation {
            tree: shell(vec![]),
            diagnostics: vec![
                missing_doctype(),
                after_after_body_recovery(3, (20, 21)),
                ignored_null(3, (20, 21)),
            ],
            ignored_tokens: vec![3],
            tokens: chain(
                plain_prefix(),
                vec![
                    recovery_record(3, 21),
                    eof_record(4, CandidateMode::InBody, 21),
                ],
            ),
            identity_events: 4,
            checkpoint: complete_at(CandidateMode::InBody, 21),
        },
        // Two NUL tokens: one recovery, because the second NUL is consumed by
        // `in body` and never revisits `after after body`.
        "Z2" => CandidateObservation {
            tree: shell(vec![]),
            diagnostics: vec![
                missing_doctype(),
                after_after_body_recovery(3, (20, 21)),
                ignored_null(3, (20, 21)),
                ignored_null(4, (21, 22)),
            ],
            ignored_tokens: vec![3, 4],
            tokens: chain(
                plain_prefix(),
                vec![
                    recovery_record(3, 21),
                    in_body_consumed_record(4, 22),
                    eof_record(5, CandidateMode::InBody, 22),
                ],
            ),
            identity_events: 4,
            checkpoint: complete_at(CandidateMode::InBody, 22),
        },
        // `x` recovers; the following NUL is already in `in body`.
        "Z3" => CandidateObservation {
            tree: shell(vec![text("x", &[(20, 21)])]),
            diagnostics: vec![
                missing_doctype(),
                after_after_body_recovery(3, (20, 21)),
                ignored_null(4, (21, 22)),
            ],
            ignored_tokens: vec![4],
            tokens: chain(
                plain_prefix(),
                vec![
                    recovery_record(3, 21),
                    in_body_consumed_record(4, 22),
                    eof_record(5, CandidateMode::InBody, 22),
                ],
            ),
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 22),
        },
        // Whitespace delegates without recovery; the NUL then recovers.
        "Z4" => CandidateObservation {
            tree: shell(vec![text(" ", &[(20, 21)])]),
            diagnostics: vec![
                missing_doctype(),
                after_after_body_recovery(4, (21, 22)),
                ignored_null(4, (21, 22)),
            ],
            ignored_tokens: vec![4],
            tokens: chain(
                plain_prefix(),
                vec![
                    delegated_record(3, 21),
                    recovery_record(4, 22),
                    eof_record(5, CandidateMode::InBody, 22),
                ],
            ),
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 22),
        },
        // The NUL recovers and is ignored; `x` is then plain `in body` text.
        "Z5" => CandidateObservation {
            tree: shell(vec![text("x", &[(21, 22)])]),
            diagnostics: vec![
                missing_doctype(),
                after_after_body_recovery(3, (20, 21)),
                ignored_null(3, (20, 21)),
            ],
            ignored_tokens: vec![3],
            tokens: chain(
                plain_prefix(),
                vec![
                    recovery_record(3, 21),
                    in_body_consumed_record(4, 22),
                    eof_record(5, CandidateMode::InBody, 22),
                ],
            ),
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 22),
        },
        // Whitespace coalesces into the existing body text node.
        "C1" => CandidateObservation {
            tree: shell(vec![text("a ", &[(6, 7), (21, 22)])]),
            diagnostics: vec![missing_doctype()],
            ignored_tokens: vec![],
            tokens: chain(
                text_prefix(),
                vec![
                    delegated_record(4, 22),
                    eof_record(5, CandidateMode::AfterAfterBody, 22),
                ],
            ),
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::AfterAfterBody, 22),
        },
        // Recovered text coalesces too, keeping both contributions.
        "C2" => CandidateObservation {
            tree: shell(vec![text("ax", &[(6, 7), (21, 22)])]),
            diagnostics: vec![missing_doctype(), after_after_body_recovery(4, (21, 22))],
            ignored_tokens: vec![],
            tokens: chain(
                text_prefix(),
                vec![
                    recovery_record(4, 22),
                    eof_record(5, CandidateMode::InBody, 22),
                ],
            ),
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 22),
        },
        // An ignored NUL leaves the earlier text exactly as it was.
        "C3" => CandidateObservation {
            tree: shell(vec![text("a", &[(6, 7)])]),
            diagnostics: vec![
                missing_doctype(),
                after_after_body_recovery(4, (21, 22)),
                ignored_null(4, (21, 22)),
            ],
            ignored_tokens: vec![4],
            tokens: chain(
                text_prefix(),
                vec![
                    recovery_record(4, 22),
                    eof_record(5, CandidateMode::InBody, 22),
                ],
            ),
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 22),
        },
        // A refused mixed run never enters the committed text prefix.
        "C4" => CandidateObservation {
            tree: shell(vec![text("a", &[(6, 7)])]),
            diagnostics: vec![missing_doctype()],
            ignored_tokens: vec![],
            tokens: chain(text_prefix(), vec![refused_mixed_record(4, 21)]),
            identity_events: 5,
            checkpoint: refused_mixed_at(21, 4, (21, 23)),
        },
        // Cross-token cycle: `after after body` is revisited only after the
        // different consumed `</body>` and `</html>` tokens.
        "C5" => CandidateObservation {
            tree: shell(vec![text("xy", &[(20, 21), (35, 36)])]),
            diagnostics: vec![
                missing_doctype(),
                after_after_body_recovery(3, (20, 21)),
                after_after_body_recovery(6, (35, 36)),
            ],
            ignored_tokens: vec![],
            tokens: chain(
                plain_prefix(),
                vec![
                    recovery_record(3, 21),
                    body_end_record(4, 28),
                    html_end_record(5, 35),
                    recovery_record(6, 36),
                    eof_record(7, CandidateMode::InBody, 36),
                ],
            ),
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 36),
        },
        other => panic!("no authored candidate GOLD for {other}"),
    }
}

// ---------------------------------------------------------------------------
// Observation helpers
// ---------------------------------------------------------------------------

fn text_nodes(tree: &CandidateTree) -> Vec<(String, Vec<ByteRange>)> {
    match tree {
        CandidateTree::Document(children) | CandidateTree::Element { children, .. } => {
            children.iter().flat_map(text_nodes).collect()
        }
        CandidateTree::Text {
            interpreted,
            contributions,
        } => vec![(interpreted.clone(), contributions.clone())],
    }
}

fn element_origins(tree: &CandidateTree, into: &mut Vec<CandidateOrigin>) {
    match tree {
        CandidateTree::Document(children) => {
            for child in children {
                element_origins(child, into);
            }
        }
        CandidateTree::Element {
            origin, children, ..
        } => {
            into.push(*origin);
            for child in children {
                element_origins(child, into);
            }
        }
        CandidateTree::Text { .. } => {}
    }
}

fn node_count(tree: &CandidateTree) -> usize {
    match tree {
        CandidateTree::Document(children) | CandidateTree::Element { children, .. } => {
            1 + children.iter().map(node_count).sum::<usize>()
        }
        CandidateTree::Text { .. } => 1,
    }
}

fn all_contributions(tree: &CandidateTree) -> Vec<ByteRange> {
    text_nodes(tree)
        .into_iter()
        .flat_map(|(_, contributions)| contributions)
        .collect()
}

fn count_code(observed: &CandidateObservation, code: CandidateDiagnosticCode) -> usize {
    observed
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == code)
        .count()
}

/// The emitted character tokens of a run, as (source range, interpreted).
fn character_runs(run: &HtmlTokenizerRunResult) -> Vec<(ByteRange, String)> {
    run.tokens()
        .iter()
        .filter_map(|token| match token {
            HtmlToken::Character(character) => {
                Some((span(character.source()), character.interpreted().to_owned()))
            }
            _ => None,
        })
        .collect()
}

/// The emitted tokenizer diagnostics, as (code, handling, location).
fn tokenizer_diagnostics(run: &HtmlTokenizerRunResult) -> Vec<(Diag, Handling, ByteRange)> {
    run.diagnostics()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.code(),
                diagnostic.handling(),
                span(diagnostic.location()),
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 1. Canonical byte authority
// ---------------------------------------------------------------------------

#[test]
fn canonical_fixture_bytes_match_their_authored_ranges() {
    let mut seen: Vec<&str> = FIXTURES.iter().map(|candidate| candidate.id).collect();
    seen.sort_unstable();
    let before = seen.len();
    seen.dedup();
    assert_eq!(seen.len(), before, "fixture ids are unique");
    assert_eq!(
        FIXTURES.len(),
        25,
        "the fixture set is exactly the authored set"
    );

    for candidate in FIXTURES {
        assert_eq!(
            candidate.bytes.len(),
            candidate.length,
            "{}: exact authored byte length",
            candidate.id
        );
        let text = candidate.source_text();
        assert_eq!(text.as_bytes(), candidate.bytes, "{}", candidate.id);

        // Every byte range the theorem depends on is verified in place, so no
        // expected span is reconstructed from prose or rendered markup.
        for ((start, end), expected) in candidate.required_ranges {
            assert!(
                *end <= candidate.length,
                "{}: required range [{start},{end}) is inside the fixture",
                candidate.id
            );
            assert_eq!(
                &candidate.bytes[*start..*end],
                *expected,
                "{}: required byte range [{start},{end})",
                candidate.id
            );
        }
    }

    // Authored CR and CRLF keep their authored byte identity.
    assert_eq!(fixture("W5").bytes[20..], *b"\r");
    assert_eq!(fixture("W6").bytes[20..], *b"\r\n");
}

// ---------------------------------------------------------------------------
// 2. Lower-layer run shape and evidence
// ---------------------------------------------------------------------------

/// The theorem assumes a specific emitted run shape. This pins it against the
/// accepted lower layer instead of assuming it, including the authored-bytes
/// versus interpreted-value distinction, token isolation of U+0000, and the
/// aggregate mixed token the refusal depends on.
#[test]
fn tokenizer_emits_the_run_shape_the_candidate_assumes() {
    type Expected = &'static [(ByteRange, &'static str)];
    let expected: [(&str, Expected); 25] = [
        ("W1", &[((20, 21), " ")]),
        ("W2", &[((20, 21), "\t")]),
        ("W3", &[((20, 21), "\n")]),
        ("W4", &[((20, 21), "\u{000c}")]),
        // CR and CRLF: authored length differs from interpreted length.
        ("W5", &[((20, 21), "\n")]),
        ("W6", &[((20, 22), "\n")]),
        ("W7", &[((20, 22), " \t")]),
        ("N1", &[((20, 21), "x")]),
        ("N2", &[((20, 22), "xy")]),
        ("N3", &[((20, 22), "\u{00a0}")]),
        ("N4", &[((20, 21), "\u{000b}")]),
        ("M1", &[((20, 22), " x")]),
        ("M2", &[((20, 22), "x ")]),
        ("M3", &[((20, 23), "\nx")]),
        ("M4", &[((20, 23), "x y")]),
        ("Z1", &[((20, 21), "\0")]),
        // Each authored NUL is its own exact token.
        ("Z2", &[((20, 21), "\0"), ((21, 22), "\0")]),
        ("Z3", &[((20, 21), "x"), ((21, 22), "\0")]),
        ("Z4", &[((20, 21), " "), ((21, 22), "\0")]),
        ("Z5", &[((20, 21), "\0"), ((21, 22), "x")]),
        ("C1", &[((6, 7), "a"), ((21, 22), " ")]),
        ("C2", &[((6, 7), "a"), ((21, 22), "x")]),
        ("C3", &[((6, 7), "a"), ((21, 22), "\0")]),
        ("C4", &[((6, 7), "a"), ((21, 23), " x")]),
        ("C5", &[((20, 21), "x"), ((35, 36), "y")]),
    ];
    for (id, runs) in expected {
        let run = run_of(id);
        assert!(
            !run.is_incomplete(),
            "{id}: the lower layer must complete for the candidate to be evaluated"
        );
        let observed: Vec<(ByteRange, String)> = character_runs(&run);
        let wanted: Vec<(ByteRange, String)> = runs
            .iter()
            .map(|(range, value)| (*range, (*value).to_owned()))
            .collect();
        assert_eq!(
            observed, wanted,
            "{id}: emitted character-run boundaries and interpreted values"
        );
        // No character token ever mixes U+0000 with another scalar.
        assert!(
            observed
                .iter()
                .all(|(_, value)| value == "\0" || !value.contains('\0')),
            "{id}: tokenizer recognition isolates an authored U+0000"
        );
    }
}

/// Authored Data-state U+0000 is lower-layer evidence only: one continued
/// diagnostic at the exact authored byte, never a replacement.
#[test]
fn tokenizer_diagnostics_for_authored_nul_are_exact_and_only_for_nul() {
    for (id, nuls) in [
        ("Z1", vec![(20, 21)]),
        ("Z2", vec![(20, 21), (21, 22)]),
        ("Z3", vec![(21, 22)]),
        ("Z4", vec![(21, 22)]),
        ("Z5", vec![(20, 21)]),
        ("C3", vec![(21, 22)]),
    ] {
        let run = run_of(id);
        assert_eq!(
            tokenizer_diagnostics(&run),
            nuls.iter()
                .map(|range| (Diag::UnexpectedNullCharacter, Handling::Continued, *range))
                .collect::<Vec<_>>(),
            "{id}: exact authored U+0000 diagnostics"
        );
        assert!(
            run.diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.context() == Context::Data),
            "{id}: Data-state context"
        );
        assert!(
            character_runs(&run)
                .iter()
                .all(|(_, value)| !value.contains('\u{fffd}')),
            "{id}: no fabricated U+FFFD at the lower layer"
        );
    }
    for candidate in FIXTURES {
        if candidate.bytes.contains(&0) {
            continue;
        }
        // Only U+000B draws an input-stream diagnostic: a lower-layer
        // preprocessing fact about a control character, which says nothing
        // about HTML whitespace and leaves the scalar a non-whitespace run.
        let expected = if candidate.id == "N4" {
            vec![(
                Diag::ControlCharacterInInputStream,
                Handling::Continued,
                (20, 21),
            )]
        } else {
            vec![]
        };
        assert_eq!(
            tokenizer_diagnostics(&run_of(candidate.id)),
            expected,
            "{}: tokenizer diagnostics without an authored NUL",
            candidate.id
        );
    }
}

// ---------------------------------------------------------------------------
// 3. Fixtures against the authored candidate GOLD
// ---------------------------------------------------------------------------

#[test]
fn fixtures_match_the_independent_candidate_gold() {
    for candidate in FIXTURES {
        assert_eq!(
            observe_fixture(candidate.id),
            candidate_gold(candidate.id),
            "{}: independent candidate theorem",
            candidate.id
        );
    }
}

// ---------------------------------------------------------------------------
// 4. Uniform run partition
// ---------------------------------------------------------------------------

#[test]
fn uniform_run_partition_is_total_and_exclusive() {
    for (run, expected) in [
        (" ", CandidateRunClass::AllWhitespace),
        ("\t", CandidateRunClass::AllWhitespace),
        ("\n", CandidateRunClass::AllWhitespace),
        ("\u{000c}", CandidateRunClass::AllWhitespace),
        ("\r", CandidateRunClass::AllWhitespace),
        (" \t\n\u{000c}\r", CandidateRunClass::AllWhitespace),
        ("x", CandidateRunClass::AllNonWhitespace),
        ("xy", CandidateRunClass::AllNonWhitespace),
        ("\u{00a0}", CandidateRunClass::AllNonWhitespace),
        ("\u{000b}", CandidateRunClass::AllNonWhitespace),
        ("\0", CandidateRunClass::AllNonWhitespace),
        (" x", CandidateRunClass::Mixed),
        ("x ", CandidateRunClass::Mixed),
        ("x y", CandidateRunClass::Mixed),
        ("\nx", CandidateRunClass::Mixed),
        (" \0", CandidateRunClass::Mixed),
    ] {
        assert_eq!(classify_run(run), expected, "{run:?}");
    }

    for code in 0u32..=0xff {
        let character = char::from_u32(code).expect("scalar value");
        assert_eq!(
            is_candidate_html_whitespace(character),
            matches!(code, 0x09 | 0x0a | 0x0c | 0x0d | 0x20),
            "U+{code:04X}"
        );
    }
}

/// The class is read off the tokenizer's interpreted aggregate token.
#[test]
fn run_classification_uses_the_aggregate_interpreted_token() {
    for (id, expected) in [
        ("W1", CandidateRunClass::AllWhitespace),
        ("W6", CandidateRunClass::AllWhitespace),
        ("W7", CandidateRunClass::AllWhitespace),
        ("N1", CandidateRunClass::AllNonWhitespace),
        ("N2", CandidateRunClass::AllNonWhitespace),
        ("N3", CandidateRunClass::AllNonWhitespace),
        ("Z1", CandidateRunClass::AllNonWhitespace),
        ("M1", CandidateRunClass::Mixed),
        ("M3", CandidateRunClass::Mixed),
        ("M4", CandidateRunClass::Mixed),
    ] {
        let runs = character_runs(&run_of(id));
        let (_, interpreted) = runs.last().expect("a trailing character run");
        assert_eq!(classify_run(interpreted), expected, "{id}");
    }
}

// ---------------------------------------------------------------------------
// 5. Whitespace delegation without a mode mutation
// ---------------------------------------------------------------------------

#[test]
fn whitespace_runs_delegate_without_mutating_the_actual_mode() {
    for (id, index, contribution) in [
        ("W1", 3, (20, 21)),
        ("W2", 3, (20, 21)),
        ("W3", 3, (20, 21)),
        ("W4", 3, (20, 21)),
        ("W5", 3, (20, 21)),
        ("W6", 3, (20, 22)),
        ("W7", 3, (20, 22)),
        ("C1", 4, (21, 22)),
    ] {
        let observed = observe_fixture(id);
        let record = &observed.tokens[index];

        assert_eq!(record.mode_before, CandidateMode::AfterAfterBody, "{id}");
        assert_eq!(
            record.mode_after,
            CandidateMode::AfterAfterBody,
            "{id}: the actual insertion mode must not change"
        );
        assert_eq!(
            record.dispatches,
            vec![delegated_in(
                CandidateMode::AfterAfterBody,
                CandidateMode::InBody
            )],
            "{id}: one delegation to the selected in-body character rule"
        );
        assert_eq!(record.reprocesses, 0, "{id}: no same-token reprocess");
        assert_eq!(
            observed.checkpoint.mode,
            CandidateMode::AfterAfterBody,
            "{id}: the run ends in after after body"
        );
        assert_eq!(
            count_code(
                &observed,
                CandidateDiagnosticCode::AfterAfterBodyCharacterData
            ),
            0,
            "{id}: whitespace records no after-after-body recovery"
        );
        assert!(
            all_contributions(&observed.tree).contains(&contribution),
            "{id}: the exact aggregate contribution is retained"
        );
        assert_eq!(
            observed.checkpoint.completion,
            CandidateCompletion::Complete,
            "{id}"
        );
    }
}

/// CR and CRLF: the contribution is the token's own authored anchor. An
/// endpoint inferred from the interpreted one-scalar LF would be wrong.
#[test]
fn cr_and_crlf_contributions_use_authored_anchors_not_interpreted_lengths() {
    for (id, authored_len) in [("W5", 1_usize), ("W6", 2)] {
        let observed = observe_fixture(id);
        let nodes = text_nodes(&observed.tree);
        assert_eq!(nodes.len(), 1, "{id}");
        let (interpreted, contributions) = &nodes[0];
        assert_eq!(interpreted, "\n", "{id}: interpreted value is one LF");
        assert_eq!(
            contributions,
            &vec![(20, 20 + authored_len)],
            "{id}: contribution covers the authored source bytes"
        );
        assert_eq!(
            fixture(id).bytes.len() - 20,
            authored_len,
            "{id}: authored tail length"
        );
    }
    let crlf = observe_fixture("W6");
    assert_ne!(
        all_contributions(&crlf.tree),
        vec![(20, 20 + "\n".len())],
        "an interpreted-length endpoint is exactly what the validator must reject"
    );
    assert_eq!(crlf.checkpoint.committed_prefix_end, 22);
}

// ---------------------------------------------------------------------------
// 6. Non-whitespace recovery and the same-token reprocess theorem
// ---------------------------------------------------------------------------

#[test]
fn non_whitespace_runs_reprocess_the_same_token_exactly_once() {
    // (fixture, recovery token index, run range).
    for (id, index, range) in [
        ("N1", 3, (20, 21)),
        ("N2", 3, (20, 22)),
        ("N3", 3, (20, 22)),
        ("N4", 3, (20, 21)),
        ("C2", 4, (21, 22)),
    ] {
        let observed = observe_fixture(id);
        let record = &observed.tokens[index];

        assert_eq!(record.mode_before, CandidateMode::AfterAfterBody, "{id}");
        assert_eq!(record.mode_after, CandidateMode::InBody, "{id}");
        assert_eq!(record.reprocesses, 1, "{id}: exactly one reprocess");
        assert_eq!(
            record.dispatches,
            vec![
                reprocessed_in(CandidateMode::AfterAfterBody),
                consumed_in(CandidateMode::InBody),
            ],
            "{id}: one recovery dispatch then one consuming dispatch"
        );

        let recoveries: Vec<&CandidateDiagnostic> = observed
            .diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.code == CandidateDiagnosticCode::AfterAfterBodyCharacterData
            })
            .collect();
        assert_eq!(recoveries.len(), 1, "{id}: exactly one recovery fact");
        assert_eq!(
            recoveries[0].trigger,
            CandidateTrigger::Authored { index, range },
            "{id}: the fact is triggered by the emitted character token"
        );
        assert_eq!(
            recoveries[0].recovery,
            CandidateRecovery::SwitchedToInBodyAndReprocessedSameToken,
            "{id}"
        );
        assert_eq!(
            observed.checkpoint,
            complete_at(CandidateMode::InBody, range.1),
            "{id}"
        );
    }
}

/// `xy` is one recovery unit, never two, and keeps one whole-token anchor.
#[test]
fn an_aggregate_non_whitespace_run_is_one_recovery_unit() {
    let observed = observe_fixture("N2");
    assert_eq!(
        count_code(
            &observed,
            CandidateDiagnosticCode::AfterAfterBodyCharacterData
        ),
        1,
        "no per-scalar recovery multiplication"
    );
    assert_eq!(observed.tokens[3].reprocesses, 1, "no per-scalar reprocess");
    assert_eq!(
        text_nodes(&observed.tree),
        vec![("xy".to_owned(), vec![(20, 22)])],
        "one contribution covering the whole emitted token"
    );
}

/// The NBSP case is one scalar, two UTF-8 bytes. Its anchor is the token's.
#[test]
fn a_non_ascii_non_whitespace_scalar_recovers_with_its_own_anchor() {
    let observed = observe_fixture("N3");
    assert_eq!(
        text_nodes(&observed.tree),
        vec![("\u{00a0}".to_owned(), vec![(20, 22)])]
    );
    assert_eq!(observed.tokens[3].reprocesses, 1);
}

// ---------------------------------------------------------------------------
// 7. Mixed aggregate refusal
// ---------------------------------------------------------------------------

#[test]
fn a_mixed_aggregate_run_is_refused_whole_before_any_mutation() {
    for (id, index, range, prefix_end, identity, expected_text) in [
        ("M1", 3, (20, 22), 20, 4, vec![]),
        ("M2", 3, (20, 22), 20, 4, vec![]),
        ("M3", 3, (20, 23), 20, 4, vec![]),
        ("M4", 3, (20, 23), 20, 4, vec![]),
        (
            "C4",
            4,
            (21, 23),
            21,
            5,
            vec![("a".to_owned(), vec![(6, 7)])],
        ),
    ] {
        let observed = observe_fixture(id);
        assert_eq!(
            observed.checkpoint,
            refused_mixed_at(prefix_end, index, range),
            "{id}: the terminal checkpoint names the whole aggregate token"
        );

        let record = &observed.tokens[index];
        assert_eq!(
            record.dispatches,
            vec![refused_in(
                CandidateMode::AfterAfterBody,
                CandidateUnsupported::MixedAfterAfterBodyCharacterRun
            )],
            "{id}: refusal is the token's first and only dispatch"
        );
        assert_eq!(
            record.mode_before, record.mode_after,
            "{id}: no mode change"
        );
        assert_eq!(record.reprocesses, 0, "{id}: no reprocess");
        assert_eq!(record.committed_prefix_end, prefix_end, "{id}");
        assert_eq!(
            observed.tokens.len(),
            index + 1,
            "{id}: processing stops at the refused token"
        );

        assert_eq!(text_nodes(&observed.tree), expected_text, "{id}");
        assert_eq!(
            observed.identity_events, identity,
            "{id}: no identity admission for the refused run"
        );
        assert_eq!(
            count_code(
                &observed,
                CandidateDiagnosticCode::AfterAfterBodyCharacterData
            ),
            0,
            "{id}: no recovery fact for the refused run"
        );
        assert!(observed.ignored_tokens.is_empty(), "{id}");
        // No fabricated prefix or suffix anchor anywhere inside the refused
        // token, and the committed prefix never enters it.
        assert!(
            !all_contributions(&observed.tree)
                .iter()
                .any(|(start, end)| *start >= range.0 && *end <= range.1),
            "{id}: no sub-anchor inside the refused token"
        );
        assert!(observed.checkpoint.committed_prefix_end <= range.0, "{id}");
    }
}

// ---------------------------------------------------------------------------
// 8. Composition with #885: authored U+0000
// ---------------------------------------------------------------------------

/// `</html>\0` carries four distinct facts. None may stand in for another.
#[test]
fn authored_nul_composes_with_885_as_four_distinct_facts() {
    let run = run_of("Z1");
    let observed = observe(&run);
    assert_eq!(observed, candidate_gold("Z1"));

    // 1. Tokenizer: UnexpectedNullCharacter / Continued + one exact token.
    assert_eq!(
        tokenizer_diagnostics(&run),
        vec![(Diag::UnexpectedNullCharacter, Handling::Continued, (20, 21))]
    );
    assert_eq!(character_runs(&run), vec![((20, 21), "\0".to_owned())]);

    // 2. AfterAfterBody recovery, with the same token reprocessed once.
    let recovery = observed
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.code == CandidateDiagnosticCode::AfterAfterBodyCharacterData
        })
        .collect::<Vec<_>>();
    assert_eq!(recovery.len(), 1);
    assert_eq!(
        observed.tokens[3].mode_before,
        CandidateMode::AfterAfterBody
    );
    assert_eq!(observed.tokens[3].mode_after, CandidateMode::InBody);
    assert_eq!(observed.tokens[3].reprocesses, 1);

    // 3. The accepted `in body` ignored-NUL fact and disposition.
    let ignored = observed
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::NullCharacterInBody)
        .collect::<Vec<_>>();
    assert_eq!(ignored.len(), 1);
    assert_eq!(ignored[0].recovery, CandidateRecovery::IgnoredToken);
    assert_eq!(observed.ignored_tokens, vec![3]);

    // The recovery and the ignore are different codes on the same token.
    assert_ne!(recovery[0].code, ignored[0].code);
    assert_ne!(recovery[0].recovery, ignored[0].recovery);
    assert_eq!(recovery[0].trigger, ignored[0].trigger);

    // 4. Text provenance: nothing. No text, no contribution, no identity,
    // no U+0000, and no fabricated U+FFFD.
    assert_eq!(text_nodes(&observed.tree), vec![]);
    assert_eq!(all_contributions(&observed.tree), vec![]);
    assert_eq!(observed.identity_events, node_count(&observed.tree));
    assert_eq!(observed.identity_events, 4);
}

/// The ignore keeps surviving neighbours exact and per-token.
#[test]
fn nul_neighbours_keep_exact_per_token_semantics() {
    // Two NUL tokens: one recovery, two ignores, never a second recovery.
    let two = observe_fixture("Z2");
    assert_eq!(
        count_code(&two, CandidateDiagnosticCode::AfterAfterBodyCharacterData),
        1
    );
    assert_eq!(
        count_code(&two, CandidateDiagnosticCode::NullCharacterInBody),
        2
    );
    assert_eq!(two.ignored_tokens, vec![3, 4]);
    assert_eq!(
        two.tokens[4].dispatches,
        vec![consumed_in(CandidateMode::InBody)]
    );
    assert_eq!(two.tokens[4].reprocesses, 0);

    // `x` then NUL: the NUL never returns to after after body.
    let after_text = observe_fixture("Z3");
    assert_eq!(
        text_nodes(&after_text.tree),
        vec![("x".to_owned(), vec![(20, 21)])]
    );
    assert_eq!(after_text.ignored_tokens, vec![4]);

    // Whitespace then NUL: only the NUL recovers.
    let after_whitespace = observe_fixture("Z4");
    assert_eq!(
        count_code(
            &after_whitespace,
            CandidateDiagnosticCode::AfterAfterBodyCharacterData
        ),
        1
    );
    assert_eq!(
        text_nodes(&after_whitespace.tree),
        vec![(" ".to_owned(), vec![(20, 21)])]
    );

    // NUL then `x`: the surviving text keeps only its own anchor.
    let before_text = observe_fixture("Z5");
    assert_eq!(
        text_nodes(&before_text.tree),
        vec![("x".to_owned(), vec![(21, 22)])]
    );

    // Ignored NUL after real text leaves that text untouched.
    let coalesced = observe_fixture("C3");
    assert_eq!(
        text_nodes(&coalesced.tree),
        vec![("a".to_owned(), vec![(6, 7)])]
    );
    for id in ["Z1", "Z2", "Z3", "Z4", "Z5", "C3"] {
        let observed = observe_fixture(id);
        for (interpreted, _) in text_nodes(&observed.tree) {
            assert!(
                !interpreted.contains('\0') && !interpreted.contains('\u{fffd}'),
                "{id}: no U+0000 and no fabricated U+FFFD text"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 9. Termination
// ---------------------------------------------------------------------------

/// No mode is evaluated twice for one token, and after the single backward edge
/// the token is consumed rather than returned to `after after body`.
#[test]
fn same_token_dispatch_never_revisits_a_mode() {
    for candidate in FIXTURES {
        let observed = observe_fixture(candidate.id);
        for record in &observed.tokens {
            let modes: Vec<CandidateMode> = record
                .dispatches
                .iter()
                .map(|dispatch| dispatch.evaluated_in)
                .collect();
            for (position, mode) in modes.iter().enumerate() {
                assert!(
                    !modes[..position].contains(mode),
                    "{}: token {} evaluated {mode:?} twice",
                    candidate.id,
                    record.index
                );
            }
            assert!(
                record.dispatches.len() <= CandidateMode::ALL.len(),
                "{}: per-token dispatches are bounded by the mode cardinality",
                candidate.id
            );
            if let Some(position) = modes.iter().position(|mode| *mode == CandidateMode::InBody) {
                assert!(
                    !modes[position + 1..].contains(&CandidateMode::AfterAfterBody),
                    "{}: token {} returned to after after body before consumption",
                    candidate.id,
                    record.index
                );
            }
        }
    }
}

/// The structural reason: the `in body` character cell is a consuming cell for
/// every character class, so a recovered token can never be re-reprocessed.
#[test]
fn the_in_body_character_cell_consumes_every_character_token() {
    for interpreted in [
        "x", "xy", " ", "\t", "\n", "\u{000c}", "\u{00a0}", "\u{000b}", "\0", " x", "x ", "x y",
        "\nx",
    ] {
        let shape = CandidateTokenShape::Characters {
            range: (0, interpreted.len()),
            interpreted,
        };
        match select(CandidateMode::InBody, &shape, None) {
            Ok(CandidateStep::Consume { next: None, .. }) => {}
            other => {
                panic!("in body must consume {interpreted:?} without a mode change: {other:?}")
            }
        }
    }
}

/// A selected token costs at most two dispatches, a consequence of the action
/// set rather than a numeric cap.
#[test]
fn a_selected_after_after_body_character_token_performs_a_fixed_dispatch_shape() {
    for candidate in FIXTURES {
        let observed = observe_fixture(candidate.id);
        let run = run_of(candidate.id);
        for record in &observed.tokens {
            if record.mode_before != CandidateMode::AfterAfterBody {
                continue;
            }
            let Some(HtmlToken::Character(character)) = run.tokens().get(record.index) else {
                continue;
            };
            let expected = match classify_run(character.interpreted()) {
                CandidateRunClass::AllWhitespace => 1,
                CandidateRunClass::AllNonWhitespace => 2,
                CandidateRunClass::Mixed => 1,
            };
            assert_eq!(
                record.dispatches.len(),
                expected,
                "{}: token {} dispatch count for its run class",
                candidate.id,
                record.index
            );
        }
    }
}

/// C5 is the load-bearing cross-token case: `after after body` is entered a
/// second time only after the different consumed `</body>` and `</html>`
/// tokens, so the cycle count is bounded by the finite emitted token stream.
#[test]
fn cross_token_cycles_are_paid_for_by_distinct_consumed_tokens() {
    let observed = observe_fixture("C5");

    let recovery_tokens: Vec<usize> = observed
        .tokens
        .iter()
        .filter(|record| {
            record.mode_before == CandidateMode::AfterAfterBody && record.reprocesses == 1
        })
        .map(|record| record.index)
        .collect();
    assert_eq!(recovery_tokens, vec![3, 6], "two recoveries, two tokens");

    let entries = observed
        .tokens
        .iter()
        .filter(|record| {
            record.mode_before != CandidateMode::AfterAfterBody
                && record.mode_after == CandidateMode::AfterAfterBody
        })
        .count();
    let consumed_html_end_tags = observed
        .tokens
        .iter()
        .filter(|record| {
            record.mode_before == CandidateMode::AfterBody
                && record.mode_after == CandidateMode::AfterAfterBody
        })
        .count();
    assert_eq!(
        entries, consumed_html_end_tags,
        "every re-entry is paid for by a consumed html end tag"
    );
    assert_eq!(entries, 2);
    assert_eq!(
        text_nodes(&observed.tree),
        vec![("xy".to_owned(), vec![(20, 21), (35, 36)])]
    );
    assert_eq!(
        observed.checkpoint.completion,
        CandidateCompletion::Complete
    );
}

// ---------------------------------------------------------------------------
// 10. Coalescing, identity admission, provenance
// ---------------------------------------------------------------------------

#[test]
fn coalescing_consumes_no_new_identity_and_keeps_ordered_contributions() {
    for (id, interpreted, contributions) in [
        ("C1", "a ", vec![(6usize, 7usize), (21, 22)]),
        ("C2", "ax", vec![(6, 7), (21, 22)]),
        ("C5", "xy", vec![(20, 21), (35, 36)]),
    ] {
        let observed = observe_fixture(id);
        assert_eq!(
            text_nodes(&observed.tree),
            vec![(interpreted.to_owned(), contributions)],
            "{id}: one text node with ordered, individually retained contributions"
        );
        // Document + html + head + body + one text node.
        assert_eq!(
            observed.identity_events,
            node_count(&observed.tree),
            "{id}: one creation event per constructed node, none for appending"
        );
        assert_eq!(observed.identity_events, 5, "{id}");
    }
}

/// A diagnostic trigger, a token disposition, and an authored origin are three
/// different things, and the candidate must not substitute one for another.
#[test]
fn recovery_diagnostics_are_never_substituted_as_the_text_origin() {
    for (id, index, range) in [
        ("N1", 3, (20usize, 21usize)),
        ("N2", 3, (20, 22)),
        ("N3", 3, (20, 22)),
        ("C2", 4, (21, 22)),
    ] {
        let observed = observe_fixture(id);
        let diagnostic = observed
            .diagnostics
            .iter()
            .find(|diagnostic| {
                diagnostic.code == CandidateDiagnosticCode::AfterAfterBodyCharacterData
            })
            .expect("an after-after-body recovery fact");
        assert_eq!(
            diagnostic.trigger,
            CandidateTrigger::Authored { index, range },
            "{id}: the fact names the emitted character token"
        );
        assert!(
            all_contributions(&observed.tree).contains(&range),
            "{id}: the exact emitted character anchor is the text contribution"
        );
        let mut origins = Vec::new();
        element_origins(&observed.tree, &mut origins);
        assert_eq!(
            origins,
            vec![
                CandidateOrigin::Synthesized,
                CandidateOrigin::Synthesized,
                CandidateOrigin::Authored((0, 6)),
            ],
            "{id}: only the authored body start tag is an element origin"
        );
    }

    // The end tags that reached `after after body` are action-only evidence.
    for (id, end_tags) in [
        ("N1", [(6usize, 13usize), (13, 20)]),
        ("C2", [(7, 14), (14, 21)]),
    ] {
        let observed = observe_fixture(id);
        let mut origins = Vec::new();
        element_origins(&observed.tree, &mut origins);
        for end_tag in end_tags {
            assert!(
                !all_contributions(&observed.tree).contains(&end_tag),
                "{id}: an end tag contributes no text"
            );
            assert!(
                !origins.contains(&CandidateOrigin::Authored(end_tag)),
                "{id}: an end tag is no element's authored origin"
            );
        }
        assert_eq!(
            node_count(&observed.tree),
            5,
            "{id}: end tags create no node"
        );
    }
}

// ---------------------------------------------------------------------------
// 11. Lower-layer monotonicity
// ---------------------------------------------------------------------------

/// Limit scenarios that make the accepted lower layer incomplete, with the
/// exact cause each is expected to produce.
fn incomplete_scenarios() -> Vec<(&'static str, HtmlTokenizerRunResult, &'static str)> {
    let n1 = fixture("N1").source_text();
    let z1 = fixture("Z1").source_text();
    let w1 = fixture("W1").source_text();
    vec![
        (
            "source bytes",
            tokenize_text(
                n1,
                1,
                HtmlTokenizerLimits::new(4, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
            ),
            "resource",
        ),
        (
            "source bytes cut inside the selected tail",
            tokenize_text(
                z1,
                1,
                HtmlTokenizerLimits::new(20, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
            ),
            "resource",
        ),
        (
            "emitted tokens before the selected tail",
            tokenize_text(
                n1,
                1,
                HtmlTokenizerLimits::new(1_024, 8_192, 3, 1_024, 256, 4_096, 1_024),
            ),
            "resource",
        ),
        (
            "emitted tokens before EOF",
            tokenize_text(
                w1,
                1,
                HtmlTokenizerLimits::new(1_024, 8_192, 4, 1_024, 256, 4_096, 1_024),
            ),
            "resource",
        ),
        (
            "tokenizer diagnostics",
            tokenize_text(
                z1,
                1,
                HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 0, 256, 4_096, 1_024),
            ),
            "resource",
        ),
        (
            "invalid configuration",
            tokenize_text(
                n1,
                1,
                HtmlTokenizerLimits::new(1_024, 0, 1_024, 1_024, 256, 4_096, 1_024),
            ),
            "configuration",
        ),
        (
            "markup declaration after the selected tail",
            tokenize_text("<body></body></html>x<!xx>", 1, generous_limits()),
            "unsupported",
        ),
        (
            "DOCTYPE after the selected tail",
            tokenize_text("<body></body></html><!DOCTYPE html>", 1, generous_limits()),
            "unsupported",
        ),
        (
            "comment after the selected tail",
            tokenize_text("<body></body></html><!--x-->", 1, generous_limits()),
            "unsupported",
        ),
        (
            "processing instruction after the selected tail",
            tokenize_text("<body></body></html><?x>", 1, generous_limits()),
            "unsupported",
        ),
    ]
}

/// A candidate run can only be Complete when the accepted lower layer is.
///
/// `ResourceLimit`, `InvalidConfiguration`, and `UnsupportedCapability` are
/// producible from input and exercised here with their exact cause.
/// `InternalInvariantFailure` is not producible from input at this boundary;
/// the same `is_incomplete()` gate covers it uniformly.
#[test]
fn lower_layer_incompleteness_is_never_upgraded() {
    assert_eq!(
        observe_fixture("N1").checkpoint.completion,
        CandidateCompletion::Complete,
        "the unrestricted N1 run is the control"
    );

    let mut causes = Vec::new();
    for (label, run, cause) in incomplete_scenarios() {
        assert!(run.is_incomplete(), "{label}: lower layer is incomplete");
        let observed_cause = match run.completion() {
            HtmlTokenizerCompletion::Incomplete(HtmlTokenizerIncompleteCause::ResourceLimit(_)) => {
                "resource"
            }
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::InvalidConfiguration(_),
            ) => "configuration",
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::UnsupportedCapability(_),
            ) => "unsupported",
            HtmlTokenizerCompletion::Incomplete(
                HtmlTokenizerIncompleteCause::InternalInvariantFailure(_),
            ) => "internal",
            HtmlTokenizerCompletion::Complete => "complete",
        };
        assert_eq!(observed_cause, cause, "{label}: exact lower-layer cause");
        causes.push(observed_cause);

        let observed = observe(&run);
        assert_ne!(
            observed.checkpoint.completion,
            CandidateCompletion::Complete,
            "{label}: candidate completion is never upgraded"
        );
        // The candidate never invents the stop that the lower layer owns.
        assert!(
            !matches!(
                observed.checkpoint.completion,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::MixedAfterAfterBodyCharacterRun,
                    ..
                }
            ),
            "{label}: no mixed-run refusal is fabricated for lower-layer evidence"
        );
    }
    for cause in ["resource", "configuration", "unsupported"] {
        assert!(causes.contains(&cause), "cause {cause} is exercised");
    }
    assert!(
        !causes.contains(&"internal"),
        "InternalInvariantFailure is not producible from input here"
    );
}

/// The lower layer's own unsupported capabilities stay the lower layer's.
#[test]
fn lower_layer_markup_capabilities_are_not_absorbed_by_the_successor() {
    for source in [
        "<body></body></html><!xx>",
        "<body></body></html><!DOCTYPE html>",
        "<body></body></html><!--x-->",
        "<body></body></html><?x>",
    ] {
        let run = tokenize_text(source, 1, generous_limits());
        assert!(run.is_incomplete(), "{source:?}");
        let HtmlTokenizerCompletion::Incomplete(
            HtmlTokenizerIncompleteCause::UnsupportedCapability(unsupported),
        ) = run.completion()
        else {
            panic!("{source:?}: expected a lower-layer unsupported capability")
        };
        assert!(
            matches!(
                unsupported.capability(),
                HtmlTokenizerCapability::MarkupDeclaration
                    | HtmlTokenizerCapability::ProcessingInstruction
                    | HtmlTokenizerCapability::BogusCommentRecovery
            ),
            "{source:?}: {:?}",
            unsupported.capability()
        );
        assert_ne!(
            observe(&run).checkpoint.completion,
            CandidateCompletion::Complete,
            "{source:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// 12. Widening negative controls
// ---------------------------------------------------------------------------

/// The successor widens exactly one thing: uniform character runs immediately
/// in `after after body`. Tags and everything else stay refused before
/// mutation, so the successor cannot silently absorb neighbouring cells.
#[test]
fn the_candidate_widens_nothing_beyond_after_after_body_uniform_runs() {
    for source in [
        "<body></body></html><p>",
        "<body></body></html></p>",
        "<body></body></html><html>",
        "<body></body></html><body>",
        "<body></body></html></html>",
        "<body></body></html><body a>",
    ] {
        let run = tokenize_text(source, 1, generous_limits());
        assert!(!run.is_incomplete(), "{source:?}: lower layer completes");
        let observed = observe(&run);
        let CandidateCompletion::IncompleteUnsupported { capability, .. } =
            observed.checkpoint.completion
        else {
            panic!("{source:?}: expected an explicit candidate refusal")
        };
        assert_eq!(
            capability,
            CandidateUnsupported::OutsideModelledCandidateCells,
            "{source:?}"
        );
        let (last, earlier) = observed.tokens.split_last().expect("a token record");
        assert!(last.refusal().is_some(), "{source:?}");
        assert_eq!(
            last.committed_prefix_end,
            earlier
                .last()
                .map_or(0, |record| record.committed_prefix_end),
            "{source:?}: a refused token commits no coverage"
        );
        assert_eq!(
            last.mode_before, last.mode_after,
            "{source:?}: no mode change"
        );
        assert_eq!(last.reprocesses, 0, "{source:?}");
        assert_eq!(
            count_code(
                &observed,
                CandidateDiagnosticCode::AfterAfterBodyCharacterData
            ),
            0
        );
        assert_eq!(text_nodes(&observed.tree), vec![], "{source:?}");
    }
}

// ---------------------------------------------------------------------------
// 13. Determinism
// ---------------------------------------------------------------------------

/// Semantic correspondence is stable across repeated runs and across differing
/// caller `SourceId` values. No raw identity encoding is asserted anywhere.
#[test]
fn candidate_semantics_are_deterministic_across_source_ids() {
    for candidate in FIXTURES {
        let baseline = observe(&tokenize_text(
            candidate.source_text(),
            1,
            generous_limits(),
        ));
        for source_id in [1_u64, 7, 4_242, u64::from(u32::MAX)] {
            let repeated = observe(&tokenize_text(
                candidate.source_text(),
                source_id,
                generous_limits(),
            ));
            assert_eq!(
                repeated, baseline,
                "{}: semantic correspondence for SourceId {source_id}",
                candidate.id
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 14. Structural boundedness
// ---------------------------------------------------------------------------

/// No tree resource dimension, node limit, depth constant, work budget, or
/// reprocess budget is required. Every bound is a structural consequence of the
/// selected action set and the emitted token stream.
#[test]
fn candidate_state_is_bounded_without_any_tree_limit() {
    for candidate in FIXTURES {
        let observed = observe_fixture(candidate.id);
        let run = run_of(candidate.id);

        let character_tokens = run
            .tokens()
            .iter()
            .filter(|token| matches!(token, HtmlToken::Character(_)))
            .count();
        assert!(
            node_count(&observed.tree) <= 4 + character_tokens,
            "{}: nodes are bounded by the shell plus character tokens",
            candidate.id
        );
        assert_eq!(
            observed.identity_events,
            node_count(&observed.tree),
            "{}: identity events equal constructed nodes",
            candidate.id
        );
        // At most one recovery and one ignore fact per character token, plus
        // the single missing-doctype fact.
        assert!(
            observed.diagnostics.len() <= 1 + 2 * character_tokens,
            "{}: diagnostics are linearly bounded by character tokens",
            candidate.id
        );
        let dispatches: usize = observed
            .tokens
            .iter()
            .map(|record| record.dispatches.len())
            .sum();
        assert!(
            dispatches <= observed.tokens.len() * CandidateMode::ALL.len(),
            "{}: total work is bounded by tokens times mode cardinality",
            candidate.id
        );
        let mut previous = 0;
        for record in &observed.tokens {
            assert!(
                record.committed_prefix_end >= previous,
                "{}: committed coverage never moves backwards",
                candidate.id
            );
            previous = record.committed_prefix_end;
        }
        assert!(previous <= candidate.length, "{}", candidate.id);
    }
}

// ---------------------------------------------------------------------------
// 15. Mutation falsification
// ---------------------------------------------------------------------------

/// Whether the validator rejects a deliberately wrong model: a gold mismatch,
/// a structural-termination panic, or an upgraded lower-layer completion.
fn validator_rejects(mutation: Mutation) -> bool {
    for candidate in FIXTURES {
        let run = run_of(candidate.id);
        let outcome = catch_unwind(AssertUnwindSafe(|| observe_with(&run, Some(mutation))));
        match outcome {
            Err(_) => return true,
            Ok(observed) if observed != candidate_gold(candidate.id) => return true,
            Ok(_) => {}
        }
    }
    for (_, run, _) in incomplete_scenarios() {
        let outcome = catch_unwind(AssertUnwindSafe(|| observe_with(&run, Some(mutation))));
        match outcome {
            Err(_) => return true,
            Ok(observed) if observed.checkpoint.completion == CandidateCompletion::Complete => {
                return true;
            }
            Ok(_) => {}
        }
    }
    false
}

#[test]
fn the_sound_model_is_not_rejected_by_its_own_validator() {
    for candidate in FIXTURES {
        let run = run_of(candidate.id);
        assert_eq!(observe_with(&run, None), candidate_gold(candidate.id));
    }
    for (label, run, _) in incomplete_scenarios() {
        assert_ne!(
            observe_with(&run, None).checkpoint.completion,
            CandidateCompletion::Complete,
            "{label}"
        );
    }
}

#[test]
fn every_deliberately_wrong_model_is_rejected() {
    for mutation in ALL_MUTATIONS {
        assert!(
            validator_rejects(mutation),
            "{mutation:?}: the validator failed to detect a falsified assumption"
        );
    }
}

/// The same-token cycle is rejected structurally, by a panic, not by a counter.
#[test]
fn a_same_token_cycle_is_a_structural_failure_not_a_counted_retry() {
    let run = run_of("N1");
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        observe_with(&run, Some(Mutation::RecoverWithoutModeSwitch))
    }));
    assert!(
        outcome.is_err(),
        "returning the token to after after body must violate the structural theorem"
    );
}

// ---------------------------------------------------------------------------
// 16. Independence and predecessor boundaries
// ---------------------------------------------------------------------------

/// The independence boundary is checked over this file's own text. Needles are
/// assembled at run time so this test never matches itself.
#[test]
fn this_module_imports_no_production_tree_construction_code() {
    let own = include_str!("after_after_body_character_successor_validation.rs");
    let forbidden = [
        ["super", "driver"],
        ["super", "session"],
        ["super", "result"],
        ["crate::html::tree_construction", "driver"],
        ["crate::html::tree_construction", "session"],
        ["crate::html::tree_construction", "result"],
    ]
    .map(|[head, tail]| format!("{head}::{tail}"));
    for needle in forbidden {
        assert!(
            !own.lines()
                .filter(|line| line.trim_start().starts_with("use "))
                .any(|line| line.contains(&needle)),
            "an import of {needle} would let production derive its own oracle"
        );
    }
    for needle in [
        ["construct_html_document", "_shell"].concat(),
        ["Unproved", "CharacterDataPosition"].concat(),
        ["HtmlTree", "Capability"].concat(),
    ] {
        assert!(
            !own.lines().any(|line| {
                let line = line.trim_start();
                !line.starts_with("//") && line.contains(&needle)
            }),
            "{needle} must not appear in executable test code"
        );
    }
}

/// The historical #353 predecessor boundary is still pinned by its own module.
/// This is a tripwire over that file's text, not a restatement of its result.
#[test]
fn the_historical_353_predecessor_boundary_remains_pinned_in_its_own_module() {
    let predecessor = include_str!("after_body_successor_validation.rs");
    for needle in [
        "\"AB7\" => CandidateObservation",
        "CandidateUnsupported::AfterAfterBodyCharacterData",
        "fn after_after_body_character_data_is_refused_before_mutation()",
    ] {
        assert!(
            predecessor.contains(needle),
            "predecessor evidence must stay historical: missing {needle}"
        );
    }
}
