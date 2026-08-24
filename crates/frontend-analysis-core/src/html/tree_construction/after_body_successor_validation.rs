//! Candidate-independent TC-S2 successor validation.
//!
//! TC-S2 is the *proposed* successor theorem "Selected After-Body Uniform
//! Character-Run Handling" (Issue #353). This module is **validation only**.
//! It changes no production tree-construction behavior, and production is
//! expected to remain TC-S1 — which still refuses every `after body` character
//! run. Nothing here authorizes production placement.
//!
//! # Independent oracle boundary
//!
//! The expected meaning in this module is authored from the candidate theorem,
//! not from any production run. The boundary is structural and greppable: this
//! module imports **nothing** from [`super::driver`], [`super::session`], or
//! [`super::result`]. It therefore cannot call production `classify`, cannot
//! call `construct_html_document_shell`, and cannot project a production
//! result into an expectation.
//!
//! The only production code it uses is the already-accepted lower layer — the
//! existing batch tokenizer — and only as *evidence input*: emitted token
//! boundaries, interpreted character-run values, retained source anchors, and
//! run completion. The tokenizer is never the tree-semantic oracle.
//!
//! Two independent statements meet here and must agree:
//!
//! 1. [`CandidateSession`] — a test-only machine that implements the candidate
//!    action set over lower-layer token evidence, and
//! 2. [`candidate_gold`] — hand-authored expected observations for AB1–AB8.
//!
//! If the theorem were internally incoherent, the machine could not reproduce
//! the authored observations, and the comparison fails rather than adapting.
//!
//! # Deliberately partial model
//!
//! Per Issue #353 the model covers only the cells AB1–AB8 traverse plus the
//! `after body` character rules under validation. Every other cell is refused
//! as [`CandidateUnsupported::OutsideModelledCandidateCells`]. That is a
//! property of this oracle, not a claim about production TC-S1's wider proved
//! action set: this module deliberately does not restate that set, and it is
//! not a second HTML parser.
//!
//! # Termination without a work budget
//!
//! TC-S1's local implementation proof is "every insertion-mode transition
//! moves strictly forward". TC-S2 intentionally introduces one backward edge
//! (`AfterBody` → `InBody`), so that proof cannot carry. The replacement is
//! also structural, not numeric: within the processing of one emitted token,
//! [`CandidateSession::process`] asserts that an insertion mode is never
//! evaluated twice. A token therefore performs at most as many dispatches as
//! there are modes, and a non-whitespace `after body` character token performs
//! exactly two — because the mode it moves to consumes character tokens
//! unconditionally. No reprocess counter, retry limit, or work budget exists
//! here or is proposed for production.
//!
//! # Canonical byte authority
//!
//! Rendered GitHub text is not fixture-byte authority. AB1–AB8 are stored as
//! escaped byte literals and checked against the Issue #353 exact byte length
//! and the exact byte ranges the theorem depends on. The canonical SHA-256
//! digests are retained as documentary metadata in [`CANDIDATE_FIXTURES`] and
//! verified outside the test process: this repository has no dependency-free
//! SHA-256 implementation, and inventing general crypto code to satisfy a
//! fixture check would be an unauthorized expansion.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

// ---------------------------------------------------------------------------
// Canonical AB1–AB8 byte authority
// ---------------------------------------------------------------------------

/// One canonical candidate fixture, materialized from the escaped byte
/// sequence recorded in Issue #353.
struct CandidateFixture {
    id: &'static str,
    /// The exact canonical bytes, written as an escaped byte literal.
    bytes: &'static [u8],
    /// The canonical exact byte length from Issue #353.
    length: usize,
    /// The canonical SHA-256 digest from Issue #353, retained as documentary
    /// metadata and verified outside this test process.
    sha256: &'static str,
    /// The byte ranges the candidate theorem depends on, with their exact
    /// expected content.
    required_ranges: &'static [((usize, usize), &'static [u8])],
}

const CANDIDATE_FIXTURES: &[CandidateFixture] = &[
    CandidateFixture {
        id: "AB1",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x20",
        length: 14,
        sha256: "ed2cec78b9f9ba529023a1090fa999073b80c2634a046705035171414787ae79",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 13), b"</body>"),
            ((13, 14), b"\x20"),
        ],
    },
    CandidateFixture {
        id: "AB2",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x78",
        length: 14,
        sha256: "07944ff01b24afe1efcf1fc443c5f2c9724b4382111d41f60027e8d436e0c67b",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 13), b"</body>"),
            ((13, 14), b"\x78"),
        ],
    },
    CandidateFixture {
        id: "AB3",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x61\x3c\x2f\x62\x6f\x64\x79\x3e\x62",
        length: 15,
        sha256: "ac96b5f2f95253933899bcbc3eb9c2319428ab861d75350c27928cd76e86f46a",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 7), b"\x61"),
            ((7, 14), b"</body>"),
            ((14, 15), b"\x62"),
        ],
    },
    CandidateFixture {
        id: "AB4",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x78\x3c\x2f\x62\x6f\x64\x79\x3e\x79",
        length: 22,
        sha256: "a03b3971fe1dc4a4c3b74aa2bd5e70c4c43ddb792765e7e0e9ba7bbe1f3b44f0",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 13), b"</body>"),
            ((13, 14), b"\x78"),
            ((14, 21), b"</body>"),
            ((21, 22), b"\x79"),
        ],
    },
    CandidateFixture {
        id: "AB5",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x20\x09",
        length: 15,
        sha256: "c8d609be89130efa19fc9d64de72b9168a989209b78125cc6242922d99c6ebcf",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 13), b"</body>"),
            ((13, 15), b"\x20\x09"),
        ],
    },
    CandidateFixture {
        id: "AB6",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x20\x78",
        length: 15,
        sha256: "e3be35eeead339d0146d8dd32ffc5111b2ea2f6ce88882e775c8803fbec5bc4e",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 13), b"</body>"),
            ((13, 15), b"\x20\x78"),
        ],
    },
    CandidateFixture {
        id: "AB7",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x3c\x2f\x68\x74\x6d\x6c\x3e\x78",
        length: 21,
        sha256: "6f939c32d7e26df21ffbb225a906ce81d699cd905f44cbb63987c5ea1c7d7db7",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 13), b"</body>"),
            ((13, 20), b"</html>"),
            ((20, 21), b"\x78"),
        ],
    },
    CandidateFixture {
        id: "AB8",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x78\x79",
        length: 15,
        sha256: "eafadcc55d945917ab8f2850ee3ccabb3f82c005ed290860f2e5aa6858759701",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((6, 13), b"</body>"),
            ((13, 15), b"\x78\x79"),
        ],
    },
];

fn fixture(id: &str) -> &'static CandidateFixture {
    CANDIDATE_FIXTURES
        .iter()
        .find(|candidate| candidate.id == id)
        .expect("canonical candidate fixture")
}

impl CandidateFixture {
    /// The canonical bytes as source text. Panics rather than lossily
    /// converting: a fixture that is not valid UTF-8 is a stop condition, not
    /// something to normalize.
    fn source_text(&self) -> &'static str {
        std::str::from_utf8(self.bytes).expect("canonical fixture bytes are valid UTF-8")
    }
}

// ---------------------------------------------------------------------------
// Independent candidate domain
// ---------------------------------------------------------------------------

/// Candidate insertion modes. Test-only, and deliberately not the production
/// mode type: TC-S2 does not inherit the strictly-forward transition promise,
/// so this type derives no ordering at all.
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
    /// Every modelled mode. Used to state the structural per-token dispatch
    /// bound as a cardinality fact rather than an invented constant.
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

/// Classifies the tokenizer's **interpreted** character run.
///
/// The classification is over the aggregate emitted run. No source subrange is
/// guessed at, and an empty run cannot occur because the tokenizer emits no
/// empty character token.
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
        range: (usize, usize),
        interpreted: &'run str,
    },
    StartTag {
        name: CandidateShellName,
        range: (usize, usize),
    },
    EndTag {
        name: CandidateShellName,
        range: (usize, usize),
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

    fn is_characters(&self) -> bool {
        matches!(self, Self::Characters { .. })
    }
}

/// What the candidate refuses, with exact typed meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateUnsupported {
    /// A mixed whitespace/non-whitespace `after body` run. TC-S2 authorizes no
    /// run splitting, so the whole aggregate run is refused.
    MixedAfterBodyCharacterRun,
    /// Character data in `after after body` stays outside TC-S2.
    AfterAfterBodyCharacterData,
    /// A character run whose handling outside `in body`/`after body` depends on
    /// a whitespace distinction the candidate does not extend there.
    WhitespaceSensitiveCharacterData,
    /// A cell this deliberately partial oracle does not model. Says nothing
    /// about production TC-S1's proved set.
    OutsideModelledCandidateCells,
}

/// Where a candidate element node's existence comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateOrigin {
    /// The trigger token's own authored start tag, as an exact span.
    Authored((usize, usize)),
    /// No authored source. The trigger token made the node necessary but is
    /// not its origin.
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
        range: (usize, usize),
    },
    /// End of file has no authored extent and gets no dummy span.
    EndOfFile {
        index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDiagnosticCode {
    MissingDoctype,
    AfterBodyCharacterData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateRecovery {
    ContinuedInQuirksDocumentMode,
    SwitchedToInBodyAndReprocessedSameToken,
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
        contributions: Vec<(usize, usize)>,
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
    tokens: Vec<CandidateTokenRecord>,
    /// Semantic creation events. Coalescing consumes none. Never a raw ID.
    identity_events: usize,
    checkpoint: CandidateCheckpoint,
}

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
    RecordAfterBodyCharacterData,
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
    /// mode stays unchanged. This is the TC-S2 whitespace delegation.
    DelegateInBodyCharacterRule,
    Stop,
}

/// Selects the candidate rule for one (mode, token) cell.
///
/// Pure: it takes no session state and mutates nothing, so a refusal is
/// structurally guaranteed to precede mutation by that cell.
fn select(
    mode: CandidateMode,
    shape: &CandidateTokenShape<'_>,
) -> Result<CandidateStep, CandidateUnsupported> {
    match mode {
        CandidateMode::Initial => {
            reject_whitespace_sensitive(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::RecordMissingDoctype),
                next: CandidateMode::BeforeHtml,
            })
        }
        CandidateMode::BeforeHtml => {
            reject_whitespace_sensitive(shape)?;
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
            reject_whitespace_sensitive(shape)?;
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
            reject_whitespace_sensitive(shape)?;
            expect_body_start_tag(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::CloseHeadElement),
                next: CandidateMode::AfterHead,
            })
        }
        CandidateMode::AfterHead => {
            reject_whitespace_sensitive(shape)?;
            expect_body_start_tag(shape)?;
            Ok(CandidateStep::Consume {
                effect: Some(CandidateEffect::InsertShellElement {
                    name: CandidateShellName::Body,
                    provenance: CandidateProvenance::AuthoredByTriggerToken,
                }),
                next: Some(CandidateMode::InBody),
            })
        }
        // Inside `in body` whitespace and non-whitespace characters are
        // inserted identically, so an aggregate run needs no splitting and no
        // whitespace refusal. This is the already accepted text rule TC-S2
        // delegates to and reprocesses into.
        CandidateMode::InBody => match shape {
            CandidateTokenShape::Characters { .. } => Ok(CandidateStep::Consume {
                effect: Some(CandidateEffect::InsertCharacters),
                next: None,
            }),
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
        // The TC-S2 frontier.
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
            CandidateTokenShape::Characters { interpreted, .. } => {
                match classify_run(interpreted) {
                    CandidateRunClass::AllWhitespace => {
                        Ok(CandidateStep::DelegateInBodyCharacterRule)
                    }
                    CandidateRunClass::AllNonWhitespace => Ok(CandidateStep::Reprocess {
                        effect: Some(CandidateEffect::RecordAfterBodyCharacterData),
                        next: CandidateMode::InBody,
                    }),
                    CandidateRunClass::Mixed => {
                        Err(CandidateUnsupported::MixedAfterBodyCharacterRun)
                    }
                }
            }
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
        CandidateMode::AfterAfterBody => match shape {
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop),
            CandidateTokenShape::Characters { .. } => {
                Err(CandidateUnsupported::AfterAfterBodyCharacterData)
            }
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
    }
}

/// Refuses a character run whose handling in the current mode would depend on a
/// whitespace distinction the candidate does not extend outside
/// `in body`/`after body`.
fn reject_whitespace_sensitive(
    shape: &CandidateTokenShape<'_>,
) -> Result<(), CandidateUnsupported> {
    match shape {
        CandidateTokenShape::Characters { interpreted, .. }
            if interpreted.chars().any(is_candidate_html_whitespace) =>
        {
            Err(CandidateUnsupported::WhitespaceSensitiveCharacterData)
        }
        _ => Ok(()),
    }
}

/// The AB1–AB8 document prefix is uniform: a `body` start tag walks the shell.
/// Any other token in those modes is outside this deliberately partial model.
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
        contributions: Vec<(usize, usize)>,
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
    identity_events: usize,
    committed_prefix_end: usize,
    processed_tokens: usize,
}

impl CandidateSession {
    fn new() -> Self {
        Self {
            nodes: vec![CandidateArenaNode {
                children: Vec::new(),
                kind: CandidateArenaKind::Document,
            }],
            open_elements: Vec::new(),
            head_element: None,
            mode: CandidateMode::Initial,
            diagnostics: Vec::new(),
            // The Document container is one semantic creation event.
            identity_events: 1,
            committed_prefix_end: 0,
            processed_tokens: 0,
        }
    }

    /// Processes one emitted token to a terminal disposition.
    ///
    /// Termination is structural: an insertion mode is never evaluated twice
    /// for the same token. The assertion *is* the theorem obligation — if the
    /// candidate action set admitted a same-token cycle, this would fire
    /// instead of looping, and TC-S2 would be falsified.
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

            match select(self.mode, &shape) {
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
            CandidateEffect::RecordAfterBodyCharacterData => {
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::AfterBodyCharacterData,
                    trigger,
                    recovery: CandidateRecovery::SwitchedToInBodyAndReprocessedSameToken,
                });
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
            CandidateEffect::InsertCharacters => self.insert_characters(shape),
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

    /// Inserts the token's characters, coalescing into the adjacent text node
    /// when one is already the last child of the insertion parent.
    ///
    /// Coalescing appends an ordered contribution and consumes no new identity
    /// event. Contributions are retained individually and are never merged into
    /// a reconstructed span.
    fn insert_characters(&mut self, shape: &CandidateTokenShape<'_>) {
        let CandidateTokenShape::Characters { range, interpreted } = shape else {
            panic!("character insertion requires a character token")
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
            existing.push_str(interpreted);
            contributions.push(*range);
            return;
        }

        let inserted = self.nodes.len();
        self.nodes.push(CandidateArenaNode {
            children: Vec::new(),
            kind: CandidateArenaKind::Text {
                interpreted: (*interpreted).to_owned(),
                contributions: vec![*range],
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

fn span(anchor: &crate::SourceAnchor) -> (usize, usize) {
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

/// Runs the independent candidate over one accepted lower-layer run.
///
/// Effective completion is authored here from the candidate theorem: lower-layer
/// incompleteness of any cause is never upgraded, and a candidate refusal is
/// reported as the candidate's own typed evidence.
fn observe(run: &HtmlTokenizerRunResult) -> CandidateObservation {
    let mut session = CandidateSession::new();
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
        tokens,
        identity_events: session.identity_events,
        checkpoint: CandidateCheckpoint {
            mode: session.mode,
            committed_prefix_end: session.committed_prefix_end,
            completion,
        },
    }
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

/// Observes one canonical fixture through the accepted lower layer.
fn observe_fixture(id: &str) -> CandidateObservation {
    let candidate = fixture(id);
    observe(&tokenize_text(
        candidate.source_text(),
        1,
        generous_limits(),
    ))
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
    complete: (usize, usize),
    children: Vec<CandidateTree>,
) -> CandidateTree {
    CandidateTree::Element {
        name,
        origin: CandidateOrigin::Authored(complete),
        children,
    }
}

fn text(interpreted: &str, contributions: &[(usize, usize)]) -> CandidateTree {
    CandidateTree::Text {
        interpreted: interpreted.to_owned(),
        contributions: contributions.to_vec(),
    }
}

/// Every AB fixture opens with the same authored `body` start tag at `[0,6)`,
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

/// The TC-S2 recovery record: one diagnostic, one mode switch, one same-token
/// reprocess, then the selected `in body` text rule consumes the token.
fn recovery_record(index: usize, committed_prefix_end: usize) -> CandidateTokenRecord {
    CandidateTokenRecord {
        index,
        mode_before: CandidateMode::AfterBody,
        mode_after: CandidateMode::InBody,
        dispatches: vec![
            reprocessed_in(CandidateMode::AfterBody),
            consumed_in(CandidateMode::InBody),
        ],
        reprocesses: 1,
        committed_prefix_end,
    }
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

fn after_body_character_data(index: usize, range: (usize, usize)) -> CandidateDiagnostic {
    CandidateDiagnostic {
        code: CandidateDiagnosticCode::AfterBodyCharacterData,
        trigger: CandidateTrigger::Authored { index, range },
        recovery: CandidateRecovery::SwitchedToInBodyAndReprocessedSameToken,
    }
}

fn complete_at(mode: CandidateMode, committed_prefix_end: usize) -> CandidateCheckpoint {
    CandidateCheckpoint {
        mode,
        committed_prefix_end,
        completion: CandidateCompletion::Complete,
    }
}

fn refused_at(
    mode: CandidateMode,
    committed_prefix_end: usize,
    capability: CandidateUnsupported,
    trigger_index: usize,
    trigger_range: (usize, usize),
) -> CandidateCheckpoint {
    CandidateCheckpoint {
        mode,
        committed_prefix_end,
        completion: CandidateCompletion::IncompleteUnsupported {
            capability,
            trigger: CandidateTrigger::Authored {
                index: trigger_index,
                range: trigger_range,
            },
        },
    }
}

/// The authored TC-S2 candidate GOLD for one fixture.
fn candidate_gold(id: &str) -> CandidateObservation {
    match id {
        // Whitespace-only run: delegated in-body text rule, actual mode stays
        // `AfterBody`, no diagnostic, no reprocess.
        "AB1" => CandidateObservation {
            tree: shell(vec![text(" ", &[(13, 14)])]),
            diagnostics: vec![missing_doctype()],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    13,
                ),
                simple_record(
                    2,
                    CandidateMode::AfterBody,
                    CandidateMode::AfterBody,
                    delegated_in(CandidateMode::AfterBody, CandidateMode::InBody),
                    14,
                ),
                simple_record(
                    3,
                    CandidateMode::AfterBody,
                    CandidateMode::AfterBody,
                    stopped_in(CandidateMode::AfterBody),
                    14,
                ),
            ],
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::AfterBody, 14),
        },
        // Non-whitespace run: one diagnostic, one switch, one same-token
        // reprocess, text inserted by the selected in-body rule.
        "AB2" => CandidateObservation {
            tree: shell(vec![text("x", &[(13, 14)])]),
            diagnostics: vec![missing_doctype(), after_body_character_data(2, (13, 14))],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    13,
                ),
                recovery_record(2, 14),
                simple_record(
                    3,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    14,
                ),
            ],
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 14),
        },
        // Final-tree coalescing across an action-only `</body>`.
        "AB3" => CandidateObservation {
            tree: shell(vec![text("ab", &[(6, 7), (14, 15)])]),
            diagnostics: vec![missing_doctype(), after_body_character_data(3, (14, 15))],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    consumed_in(CandidateMode::InBody),
                    7,
                ),
                simple_record(
                    2,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    14,
                ),
                recovery_record(3, 15),
                simple_record(
                    4,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    15,
                ),
            ],
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 15),
        },
        // Two bounded recovery cycles across two different character tokens.
        "AB4" => CandidateObservation {
            tree: shell(vec![text("xy", &[(13, 14), (21, 22)])]),
            diagnostics: vec![
                missing_doctype(),
                after_body_character_data(2, (13, 14)),
                after_body_character_data(4, (21, 22)),
            ],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    13,
                ),
                recovery_record(2, 14),
                simple_record(
                    3,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    21,
                ),
                recovery_record(4, 22),
                simple_record(
                    5,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    22,
                ),
            ],
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 22),
        },
        // One contiguous multi-character whitespace run, one delegated
        // dispatch, no split observation.
        "AB5" => CandidateObservation {
            tree: shell(vec![text(" \t", &[(13, 15)])]),
            diagnostics: vec![missing_doctype()],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    13,
                ),
                simple_record(
                    2,
                    CandidateMode::AfterBody,
                    CandidateMode::AfterBody,
                    delegated_in(CandidateMode::AfterBody, CandidateMode::InBody),
                    15,
                ),
                simple_record(
                    3,
                    CandidateMode::AfterBody,
                    CandidateMode::AfterBody,
                    stopped_in(CandidateMode::AfterBody),
                    15,
                ),
            ],
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::AfterBody, 15),
        },
        // Mixed run: refused before mutation, as one aggregate trigger.
        "AB6" => CandidateObservation {
            tree: shell(vec![]),
            diagnostics: vec![missing_doctype()],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    13,
                ),
                simple_record(
                    2,
                    CandidateMode::AfterBody,
                    CandidateMode::AfterBody,
                    refused_in(
                        CandidateMode::AfterBody,
                        CandidateUnsupported::MixedAfterBodyCharacterRun,
                    ),
                    13,
                ),
            ],
            identity_events: 4,
            checkpoint: refused_at(
                CandidateMode::AfterBody,
                13,
                CandidateUnsupported::MixedAfterBodyCharacterRun,
                2,
                (13, 15),
            ),
        },
        // `after after body` character data stays outside TC-S2.
        "AB7" => CandidateObservation {
            tree: shell(vec![]),
            diagnostics: vec![missing_doctype()],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    13,
                ),
                simple_record(
                    2,
                    CandidateMode::AfterBody,
                    CandidateMode::AfterAfterBody,
                    consumed_in(CandidateMode::AfterBody),
                    20,
                ),
                simple_record(
                    3,
                    CandidateMode::AfterAfterBody,
                    CandidateMode::AfterAfterBody,
                    refused_in(
                        CandidateMode::AfterAfterBody,
                        CandidateUnsupported::AfterAfterBodyCharacterData,
                    ),
                    20,
                ),
            ],
            identity_events: 4,
            checkpoint: refused_at(
                CandidateMode::AfterAfterBody,
                20,
                CandidateUnsupported::AfterAfterBodyCharacterData,
                3,
                (20, 21),
            ),
        },
        // One aggregate non-whitespace run is one recovery unit.
        "AB8" => CandidateObservation {
            tree: shell(vec![text("xy", &[(13, 15)])]),
            diagnostics: vec![missing_doctype(), after_body_character_data(2, (13, 15))],
            tokens: vec![
                body_start_tag_record(),
                simple_record(
                    1,
                    CandidateMode::InBody,
                    CandidateMode::AfterBody,
                    consumed_in(CandidateMode::InBody),
                    13,
                ),
                recovery_record(2, 15),
                simple_record(
                    3,
                    CandidateMode::InBody,
                    CandidateMode::InBody,
                    stopped_in(CandidateMode::InBody),
                    15,
                ),
            ],
            identity_events: 5,
            checkpoint: complete_at(CandidateMode::InBody, 15),
        },
        other => panic!("no authored candidate GOLD for {other}"),
    }
}

const CANDIDATE_IDS: [&str; 8] = ["AB1", "AB2", "AB3", "AB4", "AB5", "AB6", "AB7", "AB8"];

// ---------------------------------------------------------------------------
// Observation helpers
// ---------------------------------------------------------------------------

fn collect_text_nodes(tree: &CandidateTree, into: &mut Vec<(String, Vec<(usize, usize)>)>) {
    match tree {
        CandidateTree::Document(children) => {
            for child in children {
                collect_text_nodes(child, into);
            }
        }
        CandidateTree::Element { children, .. } => {
            for child in children {
                collect_text_nodes(child, into);
            }
        }
        CandidateTree::Text {
            interpreted,
            contributions,
        } => into.push((interpreted.clone(), contributions.clone())),
    }
}

fn text_nodes(tree: &CandidateTree) -> Vec<(String, Vec<(usize, usize)>)> {
    let mut collected = Vec::new();
    collect_text_nodes(tree, &mut collected);
    collected
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
        CandidateTree::Document(children) => 1 + children.iter().map(node_count).sum::<usize>(),
        CandidateTree::Element { children, .. } => {
            1 + children.iter().map(node_count).sum::<usize>()
        }
        CandidateTree::Text { .. } => 1,
    }
}

fn all_contributions(tree: &CandidateTree) -> Vec<(usize, usize)> {
    text_nodes(tree)
        .into_iter()
        .flat_map(|(_, contributions)| contributions)
        .collect()
}

// ---------------------------------------------------------------------------
// 1. Canonical byte authority
// ---------------------------------------------------------------------------

#[test]
fn canonical_fixture_bytes_match_the_issue_353_authority() {
    assert_eq!(
        CANDIDATE_FIXTURES.len(),
        CANDIDATE_IDS.len(),
        "the candidate fixture set is exactly AB1-AB8"
    );
    for candidate in CANDIDATE_FIXTURES {
        assert_eq!(
            candidate.bytes.len(),
            candidate.length,
            "{}: exact canonical byte length",
            candidate.id
        );
        assert_eq!(
            candidate.sha256.len(),
            64,
            "{}: canonical digest metadata is a full SHA-256 hex digest",
            candidate.id
        );
        assert!(
            candidate
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
            "{}: canonical digest metadata is lowercase hex",
            candidate.id
        );
        // The fixture must be exactly the bytes, not a lossy normalization.
        let text = candidate.source_text();
        assert_eq!(text.as_bytes(), candidate.bytes, "{}", candidate.id);
        assert_eq!(text.len(), candidate.length, "{}", candidate.id);

        // Every byte range the theorem depends on is verified in place, so no
        // expected span is ever reconstructed from prose or rendered markup.
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
}

// ---------------------------------------------------------------------------
// 2. Lower-layer run shape
// ---------------------------------------------------------------------------

/// One expected emitted character run: its exact source range and its exact
/// interpreted value.
type ExpectedRun = ((usize, usize), &'static str);

/// The candidate theorem assumes a specific emitted run shape. This pins it
/// against the accepted lower layer instead of assuming it.
#[test]
fn tokenizer_emits_the_run_shape_the_candidate_assumes() {
    let expected: [(&str, &[ExpectedRun]); 8] = [
        ("AB1", &[((13, 14), " ")]),
        ("AB2", &[((13, 14), "x")]),
        ("AB3", &[((6, 7), "a"), ((14, 15), "b")]),
        ("AB4", &[((13, 14), "x"), ((21, 22), "y")]),
        ("AB5", &[((13, 15), " \t")]),
        ("AB6", &[((13, 15), " x")]),
        ("AB7", &[((20, 21), "x")]),
        ("AB8", &[((13, 15), "xy")]),
    ];
    for (id, runs) in expected {
        let candidate = fixture(id);
        let run = tokenize_text(candidate.source_text(), 1, generous_limits());
        assert!(
            !run.is_incomplete(),
            "{id}: the lower layer must complete for the candidate to be evaluated"
        );
        let observed: Vec<((usize, usize), &str)> = run
            .tokens()
            .iter()
            .filter_map(|token| match token {
                HtmlToken::Character(character) => {
                    Some((span(character.source()), character.interpreted()))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            observed, runs,
            "{id}: emitted character-run boundaries and interpreted values"
        );
    }
}

// ---------------------------------------------------------------------------
// 3. AB1-AB8 against the authored candidate GOLD
// ---------------------------------------------------------------------------

#[test]
fn ab_cases_match_the_independent_candidate_gold() {
    for id in CANDIDATE_IDS {
        assert_eq!(
            observe_fixture(id),
            candidate_gold(id),
            "{id}: independent candidate theorem"
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
        (" x", CandidateRunClass::Mixed),
        ("x ", CandidateRunClass::Mixed),
        ("x y", CandidateRunClass::Mixed),
    ] {
        assert_eq!(classify_run(run), expected, "{run:?}");
    }

    // The candidate whitespace set is exactly the five HTML whitespace
    // characters. Nothing else in the Basic Latin and Latin-1 range joins it.
    for code in 0u32..=0xff {
        let character = char::from_u32(code).expect("scalar value");
        assert_eq!(
            is_candidate_html_whitespace(character),
            matches!(code, 0x09 | 0x0a | 0x0c | 0x0d | 0x20),
            "U+{code:04X}"
        );
    }
}

/// The classification is read off the tokenizer's interpreted run, so the
/// aggregate-run cases are classified as one unit rather than per character.
#[test]
fn run_classification_uses_the_aggregate_interpreted_run() {
    for (id, expected) in [
        ("AB1", CandidateRunClass::AllWhitespace),
        ("AB5", CandidateRunClass::AllWhitespace),
        ("AB2", CandidateRunClass::AllNonWhitespace),
        ("AB8", CandidateRunClass::AllNonWhitespace),
        ("AB6", CandidateRunClass::Mixed),
    ] {
        let candidate = fixture(id);
        let run = tokenize_text(candidate.source_text(), 1, generous_limits());
        let after_body_run = run
            .tokens()
            .iter()
            .filter_map(|token| match token {
                HtmlToken::Character(character) => Some(character),
                _ => None,
            })
            .next_back()
            .expect("a trailing character run");
        assert_eq!(
            classify_run(after_body_run.interpreted()),
            expected,
            "{id}: aggregate run classification"
        );
    }
}

// ---------------------------------------------------------------------------
// 5. Whitespace delegation without a mode mutation
// ---------------------------------------------------------------------------

#[test]
fn whitespace_after_body_runs_delegate_without_mutating_the_actual_mode() {
    for (id, contribution) in [("AB1", (13, 14)), ("AB5", (13, 15))] {
        let observed = observe_fixture(id);
        let record = &observed.tokens[2];

        assert_eq!(record.mode_before, CandidateMode::AfterBody, "{id}");
        assert_eq!(
            record.mode_after,
            CandidateMode::AfterBody,
            "{id}: the actual insertion mode must not change"
        );
        assert_eq!(
            record.dispatches.len(),
            1,
            "{id}: whitespace delegation is a single rule dispatch"
        );
        assert_eq!(
            record.dispatches[0].delegated_rule_set,
            Some(CandidateMode::InBody),
            "{id}: the selected in-body character rule is borrowed, not entered"
        );
        assert_eq!(record.reprocesses, 0, "{id}: no same-token reprocess");
        assert_eq!(
            observed.checkpoint.mode,
            CandidateMode::AfterBody,
            "{id}: the run ends in after body"
        );
        assert_eq!(
            observed
                .diagnostics
                .iter()
                .filter(
                    |diagnostic| diagnostic.code == CandidateDiagnosticCode::AfterBodyCharacterData
                )
                .count(),
            0,
            "{id}: whitespace records no after-body parse diagnostic"
        );
        assert_eq!(
            all_contributions(&observed.tree),
            vec![contribution],
            "{id}: the exact aggregate contribution"
        );
        assert_eq!(
            observed.checkpoint.completion,
            CandidateCompletion::Complete,
            "{id}"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Non-whitespace recovery and the same-token reprocess theorem
// ---------------------------------------------------------------------------

#[test]
fn non_whitespace_after_body_runs_reprocess_the_same_token_exactly_once() {
    // (fixture, recovery token index, run range).
    for (id, index, range) in [
        ("AB2", 2, (13, 14)),
        ("AB3", 3, (14, 15)),
        ("AB8", 2, (13, 15)),
    ] {
        let observed = observe_fixture(id);
        let record = &observed.tokens[index];

        assert_eq!(record.mode_before, CandidateMode::AfterBody, "{id}");
        assert_eq!(record.mode_after, CandidateMode::InBody, "{id}");
        assert_eq!(
            record.reprocesses, 1,
            "{id}: exactly one same-token reprocess"
        );
        assert_eq!(
            record.dispatches,
            vec![
                reprocessed_in(CandidateMode::AfterBody),
                consumed_in(CandidateMode::InBody),
            ],
            "{id}: one recovery dispatch then one consuming dispatch"
        );

        let recoveries: Vec<&CandidateDiagnostic> = observed
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::AfterBodyCharacterData)
            .collect();
        assert_eq!(
            recoveries.len(),
            1,
            "{id}: exactly one after-body parse diagnostic"
        );
        assert_eq!(
            recoveries[0].trigger,
            CandidateTrigger::Authored { index, range },
            "{id}: the diagnostic is triggered by the emitted character token"
        );
        assert_eq!(
            recoveries[0].recovery,
            CandidateRecovery::SwitchedToInBodyAndReprocessedSameToken,
            "{id}"
        );
        assert_eq!(
            observed.checkpoint.completion,
            CandidateCompletion::Complete,
            "{id}"
        );
    }
}

/// AB8 is the aggregate-run case: `xy` is one recovery unit, never two.
#[test]
fn an_aggregate_non_whitespace_run_is_one_recovery_unit() {
    let observed = observe_fixture("AB8");
    assert_eq!(
        observed
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::AfterBodyCharacterData)
            .count(),
        1,
        "no per-character diagnostic multiplication"
    );
    assert_eq!(
        observed.tokens[2].reprocesses, 1,
        "no per-character reprocess multiplication"
    );
    assert_eq!(
        text_nodes(&observed.tree),
        vec![("xy".to_owned(), vec![(13, 15)])],
        "one contribution covering the whole run"
    );
}

/// The replacement for TC-S1's strictly-forward proof: no mode is evaluated
/// twice for one token, and after the single backward edge the token is
/// consumed rather than returned to `after body`.
#[test]
fn same_token_dispatch_never_revisits_a_mode() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        for record in &observed.tokens {
            let modes: Vec<CandidateMode> = record
                .dispatches
                .iter()
                .map(|dispatch| dispatch.evaluated_in)
                .collect();
            for (position, mode) in modes.iter().enumerate() {
                assert!(
                    !modes[..position].contains(mode),
                    "{id}: token {} evaluated {mode:?} twice",
                    record.index
                );
            }
            assert!(
                record.dispatches.len() <= CandidateMode::ALL.len(),
                "{id}: per-token dispatches are bounded by the mode cardinality"
            );
            // Once a token has been handed to `in body`, it cannot come back
            // to `after body` before it is consumed.
            if let Some(position) = modes.iter().position(|mode| *mode == CandidateMode::InBody) {
                assert!(
                    !modes[position + 1..].contains(&CandidateMode::AfterBody),
                    "{id}: token {} returned to after body before being consumed",
                    record.index
                );
            }
        }
    }
}

/// The maximum dispatch count for one selected non-whitespace `after body`
/// character token is exactly two, and it is a semantic consequence of the
/// action set rather than a numeric cap.
#[test]
fn a_selected_after_body_character_token_performs_at_most_two_dispatches() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        let candidate = fixture(id);
        let run = tokenize_text(candidate.source_text(), 1, generous_limits());
        for record in &observed.tokens {
            if record.mode_before != CandidateMode::AfterBody {
                continue;
            }
            let Some(HtmlToken::Character(character)) = run.tokens().get(record.index) else {
                continue;
            };
            let expected = match classify_run(character.interpreted()) {
                // delegated rule evaluation
                CandidateRunClass::AllWhitespace => 1,
                // recovery dispatch + consuming dispatch
                CandidateRunClass::AllNonWhitespace => 2,
                // refusal before mutation
                CandidateRunClass::Mixed => 1,
            };
            assert_eq!(
                record.dispatches.len(),
                expected,
                "{id}: token {} dispatch count for its run class",
                record.index
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Cross-token termination
// ---------------------------------------------------------------------------

/// AB4 is the load-bearing termination case: `after body` is revisited only
/// after a *different* consumed `</body>` token, so the cycle count is bounded
/// by the finite emitted token stream and needs no work budget.
#[test]
fn cross_token_after_body_cycles_are_bounded_by_consumed_body_end_tags() {
    let observed = observe_fixture("AB4");

    let recovery_tokens: Vec<usize> = observed
        .tokens
        .iter()
        .filter(|record| record.mode_before == CandidateMode::AfterBody && record.reprocesses == 1)
        .map(|record| record.index)
        .collect();
    assert_eq!(
        recovery_tokens,
        vec![2, 4],
        "exactly two recovery cycles, on two different emitted tokens"
    );

    let entries_into_after_body = observed
        .tokens
        .iter()
        .filter(|record| {
            record.mode_before != CandidateMode::AfterBody
                && record.mode_after == CandidateMode::AfterBody
        })
        .count();
    let consumed_body_end_tags = observed
        .tokens
        .iter()
        .filter(|record| {
            record.mode_before == CandidateMode::InBody
                && record.mode_after == CandidateMode::AfterBody
        })
        .count();
    assert_eq!(
        entries_into_after_body, consumed_body_end_tags,
        "every after-body entry is paid for by a consumed body end tag"
    );
    assert_eq!(consumed_body_end_tags, 2);

    assert_eq!(
        text_nodes(&observed.tree),
        vec![("xy".to_owned(), vec![(13, 14), (21, 22)])],
        "one coalesced text node with ordered contributions"
    );
    assert_eq!(
        observed.checkpoint.completion,
        CandidateCompletion::Complete
    );
}

// ---------------------------------------------------------------------------
// 8. Mixed and after-after-body refusal
// ---------------------------------------------------------------------------

#[test]
fn a_mixed_after_body_run_is_refused_before_mutation_and_never_split() {
    let observed = observe_fixture("AB6");

    assert_eq!(
        observed.checkpoint,
        refused_at(
            CandidateMode::AfterBody,
            13,
            CandidateUnsupported::MixedAfterBodyCharacterRun,
            2,
            (13, 15),
        ),
        "the terminal checkpoint names the whole aggregate run"
    );
    let record = &observed.tokens[2];
    assert_eq!(
        record.dispatches.len(),
        1,
        "refusal precedes any dispatch work"
    );
    assert_eq!(record.mode_before, record.mode_after, "no mode mutation");
    assert_eq!(record.reprocesses, 0, "no reprocess");
    assert_eq!(
        record.committed_prefix_end, 13,
        "committed prefix ends at 13"
    );

    assert_eq!(text_nodes(&observed.tree), vec![], "no text node");
    assert_eq!(all_contributions(&observed.tree), vec![], "no contribution");
    assert_eq!(
        observed.identity_events, 4,
        "no identity event for the refused run"
    );
    assert_eq!(
        observed
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::AfterBodyCharacterData)
            .count(),
        0,
        "no after-body diagnostic is committed for the refused run"
    );
    // No fabricated sub-anchor: nothing anywhere records a strict subrange of
    // the refused aggregate run.
    assert!(
        !all_contributions(&observed.tree)
            .iter()
            .any(|(start, end)| *start >= 13 && *end <= 15),
        "no fabricated prefix or suffix anchor inside the refused run"
    );
}

#[test]
fn after_after_body_character_data_is_refused_before_mutation() {
    let observed = observe_fixture("AB7");

    assert_eq!(
        observed.tokens[2].mode_after,
        CandidateMode::AfterAfterBody,
        "the supported html end tag is committed through byte 20"
    );
    assert_eq!(observed.tokens[2].committed_prefix_end, 20);

    let refused = &observed.tokens[3];
    assert_eq!(
        refused.dispatches.len(),
        1,
        "the refusal is the token's first and only dispatch"
    );
    assert_eq!(refused.mode_before, refused.mode_after, "no mode mutation");
    assert_eq!(refused.reprocesses, 0, "no reprocess");
    assert_eq!(
        refused.committed_prefix_end, 20,
        "the committed prefix stays at byte 20"
    );

    assert_eq!(
        observed.checkpoint,
        refused_at(
            CandidateMode::AfterAfterBody,
            20,
            CandidateUnsupported::AfterAfterBodyCharacterData,
            3,
            (20, 21),
        )
    );
    assert_eq!(text_nodes(&observed.tree), vec![], "no text node for `x`");
    assert_eq!(observed.identity_events, 4, "no identity event for `x`");
    assert_eq!(
        observed.diagnostics,
        vec![missing_doctype()],
        "no diagnostic is committed for the refused character data"
    );
}

// ---------------------------------------------------------------------------
// 9. Coalescing, identity admission, provenance
// ---------------------------------------------------------------------------

#[test]
fn adjacent_text_coalescing_consumes_no_new_identity() {
    for (id, expected) in [
        ("AB3", ("ab", vec![(6usize, 7usize), (14, 15)])),
        ("AB4", ("xy", vec![(13, 14), (21, 22)])),
    ] {
        let observed = observe_fixture(id);
        assert_eq!(
            text_nodes(&observed.tree),
            vec![(expected.0.to_owned(), expected.1)],
            "{id}: one text node with ordered, individually retained contributions"
        );
        // Document + html + head + body + one text node. The second
        // contribution admits no further creation event.
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
        ("AB2", 2, (13usize, 14usize)),
        ("AB3", 3, (14, 15)),
        ("AB4", 2, (13, 14)),
        ("AB8", 2, (13, 15)),
    ] {
        let observed = observe_fixture(id);
        let diagnostic = observed
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == CandidateDiagnosticCode::AfterBodyCharacterData)
            .expect("an after-body recovery diagnostic");
        assert_eq!(
            diagnostic.trigger,
            CandidateTrigger::Authored { index, range },
            "{id}: the diagnostic names the emitted character token"
        );

        // The text keeps the character token's own anchor as a contribution,
        // and no element ever claims a character range as authored origin.
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
}

/// The authored `</body>` in AB3 and AB4 is action/disposition evidence only.
#[test]
fn an_action_only_body_end_tag_creates_no_node_and_no_contribution() {
    for (id, end_tag, expected_nodes) in [("AB3", (7usize, 14usize), 5), ("AB4", (6, 13), 5)] {
        let observed = observe_fixture(id);
        assert!(
            !all_contributions(&observed.tree).contains(&end_tag),
            "{id}: the body end tag contributes no text"
        );
        let mut origins = Vec::new();
        element_origins(&observed.tree, &mut origins);
        assert!(
            !origins.contains(&CandidateOrigin::Authored(end_tag)),
            "{id}: the body end tag is no element's authored origin"
        );
        assert_eq!(
            node_count(&observed.tree),
            expected_nodes,
            "{id}: the end tag creates no node"
        );
    }
}

// ---------------------------------------------------------------------------
// 10. Lower-layer monotonicity
// ---------------------------------------------------------------------------

/// A candidate run can only be Complete when the accepted lower layer is.
///
/// The candidate completion rule gates on `is_incomplete()`, so every
/// tokenizer incomplete cause is covered uniformly. `ResourceLimit`,
/// `InvalidConfiguration`, and `UnsupportedCapability` are producible from
/// input and are exercised here; `InternalInvariantFailure` is not producible
/// from input at this boundary and is covered by the same gate.
#[test]
fn lower_layer_incompleteness_is_never_upgraded() {
    let complete = observe_fixture("AB2");
    assert_eq!(
        complete.checkpoint.completion,
        CandidateCompletion::Complete,
        "the unrestricted AB2 run is the control"
    );

    let source = fixture("AB2").source_text();
    for (label, limits) in [
        // Source bytes exhausted mid-run.
        (
            "source bytes",
            HtmlTokenizerLimits::new(4, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
        ),
        // Emitted tokens exhausted mid-run.
        (
            "emitted tokens",
            HtmlTokenizerLimits::new(1_024, 8_192, 1, 1_024, 256, 4_096, 1_024),
        ),
        // An invalid tokenizer configuration.
        (
            "invalid configuration",
            HtmlTokenizerLimits::new(1_024, 0, 1_024, 1_024, 256, 4_096, 1_024),
        ),
    ] {
        let run = tokenize_text(source, 1, limits);
        assert!(run.is_incomplete(), "{label}: lower layer is incomplete");
        assert_ne!(
            observe(&run).checkpoint.completion,
            CandidateCompletion::Complete,
            "{label}: candidate completion is never upgraded"
        );
    }

    // A tokenizer-owned unsupported capability, reached after the candidate's
    // own after-body cell would otherwise have succeeded.
    let run = tokenize_text("<body></body>x&amp;", 1, generous_limits());
    assert!(
        run.is_incomplete(),
        "character references are the tokenizer's own capability"
    );
    assert_ne!(
        observe(&run).checkpoint.completion,
        CandidateCompletion::Complete,
        "lower-layer unsupported capability is never upgraded"
    );
}

// ---------------------------------------------------------------------------
// 11. Structural boundedness
// ---------------------------------------------------------------------------

/// No tree resource dimension, node limit, depth constant, work budget, or
/// reprocess budget is required. Every bound below is a structural consequence
/// of the selected action set and the emitted token stream.
#[test]
fn candidate_state_is_bounded_without_any_tree_limit() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id);
        let candidate = fixture(id);
        let run = tokenize_text(candidate.source_text(), 1, generous_limits());

        let character_tokens = run
            .tokens()
            .iter()
            .filter(|token| matches!(token, HtmlToken::Character(_)))
            .count();
        let admitted_character_tokens = observed
            .tokens
            .iter()
            .filter(|record| {
                record.refusal().is_none()
                    && matches!(
                        run.tokens().get(record.index),
                        Some(HtmlToken::Character(_))
                    )
            })
            .count();

        // The shell is fixed at Document + html + head + body, and each
        // admitted character token creates at most one text node.
        assert!(
            node_count(&observed.tree) <= 4 + admitted_character_tokens,
            "{id}: node count is bounded by the shell plus admitted character tokens"
        );
        assert_eq!(
            observed.identity_events,
            node_count(&observed.tree),
            "{id}: identity events equal constructed nodes; coalescing admits none"
        );
        assert!(
            observed.diagnostics.len() <= 1 + character_tokens,
            "{id}: diagnostics are linearly bounded by processed character tokens"
        );

        let dispatches: usize = observed
            .tokens
            .iter()
            .map(|record| record.dispatches.len())
            .sum();
        assert!(
            dispatches <= observed.tokens.len() * CandidateMode::ALL.len(),
            "{id}: total work is bounded by tokens times mode cardinality"
        );

        // Committed coverage is monotonic and never exceeds the source.
        let mut previous = 0;
        for record in &observed.tokens {
            assert!(
                record.committed_prefix_end >= previous,
                "{id}: committed coverage never moves backwards"
            );
            previous = record.committed_prefix_end;
        }
        assert!(
            previous <= candidate.length,
            "{id}: committed coverage stays inside the source"
        );
    }
}

// ---------------------------------------------------------------------------
// 12. Determinism
// ---------------------------------------------------------------------------

/// Semantic correspondence is stable across repeated runs and across differing
/// caller `SourceId` values. No raw identity encoding is asserted anywhere.
#[test]
fn candidate_semantics_are_deterministic_across_source_ids() {
    for id in CANDIDATE_IDS {
        let candidate = fixture(id);
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
                "{id}: semantic correspondence for SourceId {source_id}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 13. Inherited refusal boundary
// ---------------------------------------------------------------------------

/// TC-S2 widens exactly one thing: uniform character runs immediately in
/// `after body`. Everything the candidate does not model stays refused before
/// mutation, so the successor cannot silently absorb neighbouring cells.
#[test]
fn the_candidate_widens_nothing_beyond_after_body_uniform_runs() {
    for (source, expected) in [
        // Whitespace stays refused outside `in body`/`after body`.
        (" ", CandidateUnsupported::WhitespaceSensitiveCharacterData),
        (
            "\t<body>",
            CandidateUnsupported::WhitespaceSensitiveCharacterData,
        ),
        // Shapes and cells outside this deliberately partial oracle.
        ("<p>", CandidateUnsupported::OutsideModelledCandidateCells),
        (
            "<body a>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<body/>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<html>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<body><body>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
        (
            "<body></body><body>",
            CandidateUnsupported::OutsideModelledCandidateCells,
        ),
    ] {
        let run = tokenize_text(source, 1, generous_limits());
        assert!(!run.is_incomplete(), "{source:?}: lower layer completes");
        let observed = observe(&run);
        let CandidateCompletion::IncompleteUnsupported { capability, .. } =
            observed.checkpoint.completion
        else {
            panic!("{source:?}: expected an explicit candidate refusal")
        };
        assert_eq!(capability, expected, "{source:?}");

        // Refusal precedes mutation *by the refusing cell*: the refused
        // dispatch is terminal for its token, and the refused token advances
        // committed coverage by nothing, so the terminal checkpoint stays the
        // last valid committed prefix.
        //
        // An earlier dispatch of the same token may already have committed a
        // supported effect — `<html>` is reprocessed out of `Initial` before
        // the model refuses it in `before html`. That is checkpoint honesty,
        // not a partial mutation of the refused cell. AB6 and AB7 assert the
        // stronger first-dispatch property separately.
        let (last, earlier) = observed
            .tokens
            .split_last()
            .expect("at least one token record");
        assert!(last.refusal().is_some(), "{source:?}");
        assert!(
            matches!(
                last.dispatches.last().map(|dispatch| dispatch.outcome),
                Some(CandidateDispatchOutcome::Refused(_))
            ),
            "{source:?}: the refusal is the token's terminal dispatch"
        );
        let committed_before = earlier
            .last()
            .map_or(0, |record| record.committed_prefix_end);
        assert_eq!(
            last.committed_prefix_end, committed_before,
            "{source:?}: a refused token commits no coverage"
        );
    }
}
