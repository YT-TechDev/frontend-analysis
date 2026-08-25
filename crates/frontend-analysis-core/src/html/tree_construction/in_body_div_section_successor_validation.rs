//! Candidate-independent TC-S4 successor validation.
//!
//! TC-S4 is the proposed successor theorem "Selected In-Body Heterogeneous
//! `div`/`section` Block Closure" (Issue #361). This module is validation
//! only. It does not choose production placement or a production
//! ordinary-element model.
//!
//! The candidate consumes only the accepted batch tokenizer as lower-layer
//! evidence. It imports no production tree-construction semantics from
//! `driver`, `session`, or `result`. Expected meaning is independently stated
//! by a bounded candidate machine and hand-authored HS1-HS21 GOLD.
//!
//! The load-bearing normative authority is the #348 pinned WHATWG HTML source:
//! commit `508a037333d8a1806504303aeb489d931fabbef6`, blob
//! `68dbcb98bbe1001c6ae2531be2368c608fbafddd`. The candidate executes the
//! selected `in body` `div`/`section` start/end rules, the relevant scope
//! walk, the heterogeneous suffix-pop branch, implied end-tag generation, and
//! the `in body` EOF branch over its own private state.
//!
//! Three evidence domains are intentionally explicit here:
//! - authored evidence retains caller `SourceId` plus the exact byte range;
//! - constructed relationships use test-only semantic node identities rather
//!   than arena/storage indices;
//! - matching selected end tags and intervening recovery pops remain distinct
//!   relations tied to exact semantic identities and exact emitted triggers.
//!
//! These types are validation artifacts only. They are not proposed production
//! representations and create no public or cross-run compatibility promise.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

const PINNED_WHATWG_COMMIT: &str = "508a037333d8a1806504303aeb489d931fabbef6";
const PINNED_WHATWG_SOURCE_BLOB: &str = "68dbcb98bbe1001c6ae2531be2368c608fbafddd";

struct CandidateFixture {
    id: &'static str,
    bytes: &'static [u8],
    length: usize,
    sha256: &'static str,
    required_ranges: &'static [((usize, usize), &'static [u8])],
}

const CANDIDATE_FIXTURES: &[CandidateFixture] = &[
    CandidateFixture {
        id: "HS1",
        bytes: b"<body><section></section>",
        length: 25,
        sha256: "5dda3cc0a443ab6c73a7852af27060600a6206a657f97016f42af5bd366cfca5",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 25), b"</section>"),
            ((17, 24), b"section"),
        ],
    },
    CandidateFixture {
        id: "HS2",
        bytes: b"<body><SeCtIoN>x</sEcTiOn>",
        length: 26,
        sha256: "884c3b03d44b982fc5744073492da4c096d20aaded408afbc56f2915ed30237c",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<SeCtIoN>"),
            ((7, 14), b"SeCtIoN"),
            ((15, 16), b"x"),
            ((16, 26), b"</sEcTiOn>"),
            ((18, 25), b"sEcTiOn"),
        ],
    },
    CandidateFixture {
        id: "HS3",
        bytes: b"<body><section><section>x</section></section>",
        length: 45,
        sha256: "5761461ccbb1d593a2c478fe6e6885af49006513c3154a8eb8005579ac924129",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 24), b"<section>"),
            ((16, 23), b"section"),
            ((24, 25), b"x"),
            ((25, 35), b"</section>"),
            ((27, 34), b"section"),
            ((35, 45), b"</section>"),
            ((37, 44), b"section"),
        ],
    },
    CandidateFixture {
        id: "HS4",
        bytes: b"<body><div><section></section></div>",
        length: 36,
        sha256: "718d8a47da96e9124ba91d763ffb71e7cd7bde4288197bc14ef451d365401632",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 20), b"<section>"),
            ((12, 19), b"section"),
            ((20, 30), b"</section>"),
            ((22, 29), b"section"),
            ((30, 36), b"</div>"),
            ((32, 35), b"div"),
        ],
    },
    CandidateFixture {
        id: "HS5",
        bytes: b"<body><section><div></div></section>",
        length: 36,
        sha256: "c3bf36670d2d1ad09c7838100d5092d309a5fa339c411cae6456226a946310ea",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 20), b"<div>"),
            ((16, 19), b"div"),
            ((20, 26), b"</div>"),
            ((22, 25), b"div"),
            ((26, 36), b"</section>"),
            ((28, 35), b"section"),
        ],
    },
    CandidateFixture {
        id: "HS6",
        bytes: b"<body><section><div></section>",
        length: 30,
        sha256: "bcfeaac4e82672a78dd6e724e16c2b0782c49737ff9074a3482153fa6f3f9ed4",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 20), b"<div>"),
            ((16, 19), b"div"),
            ((20, 30), b"</section>"),
            ((22, 29), b"section"),
        ],
    },
    CandidateFixture {
        id: "HS7",
        bytes: b"<body><div><section></div>",
        length: 26,
        sha256: "e64ebe85f8adcefd7b35780e32238bb43957d9e81641bb333b9cd64319dc1171",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 20), b"<section>"),
            ((12, 19), b"section"),
            ((20, 26), b"</div>"),
            ((22, 25), b"div"),
        ],
    },
    CandidateFixture {
        id: "HS8",
        bytes: b"<body><div><section><div></section></div>",
        length: 41,
        sha256: "72a822bf2a148d9ec22d57a3644bfecd6967974b6ce5f8155fd345aec3f68ae9",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 20), b"<section>"),
            ((12, 19), b"section"),
            ((20, 25), b"<div>"),
            ((21, 24), b"div"),
            ((25, 35), b"</section>"),
            ((27, 34), b"section"),
            ((35, 41), b"</div>"),
            ((37, 40), b"div"),
        ],
    },
    CandidateFixture {
        id: "HS9",
        bytes: b"<body></section>",
        length: 16,
        sha256: "4634f216d2019c6eddb14f5d0b7b0ecda78cb91ceb7b4324edcf673a1086ef32",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 16), b"</section>"),
            ((8, 15), b"section"),
        ],
    },
    CandidateFixture {
        id: "HS10",
        bytes: b"<body><section>x",
        length: 16,
        sha256: "380012ca66d9b82ca012ae9a302d730937606be799a48b7b4e55e751472ed480",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 16), b"x"),
        ],
    },
    CandidateFixture {
        id: "HS11",
        bytes: b"<body><section id=x>",
        length: 20,
        sha256: "571a238ff786c7e5df9e12baf86def1dae534087cc2c379073c8c470ea0e023b",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 20), b"<section id=x>"),
            ((7, 14), b"section"),
            ((15, 19), b"id=x"),
        ],
    },
    CandidateFixture {
        id: "HS12",
        bytes: b"<body><section/>",
        length: 16,
        sha256: "a0cf8a789896f78f9be0600cbd07fbefc9d7839d30eaee980087ca0e2d6a8a38",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 16), b"<section/>"),
            ((7, 14), b"section"),
            ((14, 15), b"/"),
        ],
    },
    CandidateFixture {
        id: "HS13",
        bytes: b"<body></body><section>",
        length: 22,
        sha256: "60db5fc48470594c9f9cea42d9bfeeadc147fe753020c14c3d38cf7682bfde1d",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 13), b"</body>"),
            ((8, 12), b"body"),
            ((13, 22), b"<section>"),
            ((14, 21), b"section"),
        ],
    },
    CandidateFixture {
        id: "HS14",
        bytes: b"<body><section></body>",
        length: 22,
        sha256: "4d958a27b0fa855387d1e68d9e8522cfd63ef06e7480b2023ceb54c6de5fa9a0",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 22), b"</body>"),
            ((17, 21), b"body"),
        ],
    },
    CandidateFixture {
        id: "HS15",
        bytes: b"<body><p>",
        length: 9,
        sha256: "648ccd6dff0fb3d71045933acb5ea913a0c5566f1d52abb69056a073cbfc1b8c",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 9), b"<p>"),
            ((7, 8), b"p"),
        ],
    },
    CandidateFixture {
        id: "HS16",
        bytes: b"<body><span>",
        length: 12,
        sha256: "508f5a295b34346d2dc607630298946d497952bfb5d8daec67d5fd4fed301422",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 12), b"<span>"),
            ((7, 11), b"span"),
        ],
    },
    CandidateFixture {
        id: "HS17",
        bytes: b"<body><section>&amp;",
        length: 20,
        sha256: "938f8f478d6e1b6eb46960f6762131ae5b3894d77b888b89d5e1f42044cd8001",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 20), b"&amp;"),
        ],
    },
    CandidateFixture {
        id: "HS18",
        bytes: b"x<section></section>",
        length: 20,
        sha256: "814653532bf29a049730fe1c422dd658962bbda888304fa3dd12c70086c79bfe",
        required_ranges: &[
            ((0, 1), b"x"),
            ((1, 10), b"<section>"),
            ((2, 9), b"section"),
            ((10, 20), b"</section>"),
            ((12, 19), b"section"),
        ],
    },
    CandidateFixture {
        id: "HS19",
        bytes: b"<body><section></section></body>",
        length: 32,
        sha256: "bd4741e4f9b4e5799a4ffa636faae5761a17d2f9768d71ccb9664b20499a6c67",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 25), b"</section>"),
            ((17, 24), b"section"),
            ((25, 32), b"</body>"),
            ((27, 31), b"body"),
        ],
    },
    CandidateFixture {
        id: "HS20",
        bytes: b"<body><div></div>",
        length: 17,
        sha256: "44f9dbc6331c75636ef2eec39853fd6c931b1fba28272c62786ebac16cb4ba84",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 11), b"<div>"),
            ((7, 10), b"div"),
            ((11, 17), b"</div>"),
            ((13, 16), b"div"),
        ],
    },
    CandidateFixture {
        id: "HS21",
        bytes: b"<body><section></section id=x>",
        length: 30,
        sha256: "85922d5b18dd97d19f7982ea3fb29b6dab757ae40aea95337101e475be20a049",
        required_ranges: &[
            ((0, 6), b"<body>"),
            ((1, 5), b"body"),
            ((6, 15), b"<section>"),
            ((7, 14), b"section"),
            ((15, 30), b"</section id=x>"),
            ((17, 24), b"section"),
            ((25, 29), b"id=x"),
        ],
    },
];

const CANDIDATE_IDS: [&str; 21] = [
    "HS1", "HS2", "HS3", "HS4", "HS5", "HS6", "HS7", "HS8", "HS9", "HS10", "HS11", "HS12", "HS13",
    "HS14", "HS15", "HS16", "HS17", "HS18", "HS19", "HS20", "HS21",
];

fn fixture(id: &str) -> &'static CandidateFixture {
    CANDIDATE_FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .expect("canonical candidate fixture")
}

impl CandidateFixture {
    fn source_text(&self) -> &'static str {
        std::str::from_utf8(self.bytes).expect("canonical fixture bytes are valid UTF-8")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateEvidence {
    source_id: SourceId,
    range: (usize, usize),
}

fn evidence(anchor: &crate::SourceAnchor) -> CandidateEvidence {
    CandidateEvidence {
        source_id: anchor.source_id(),
        range: (anchor.range().start(), anchor.range().end()),
    }
}

fn expected_evidence(source_id: u64, range: (usize, usize)) -> CandidateEvidence {
    CandidateEvidence {
        source_id: SourceId::new(source_id),
        range,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    AfterBody,
}

impl CandidateMode {
    const ALL: [Self; 7] = [
        Self::Initial,
        Self::BeforeHtml,
        Self::BeforeHead,
        Self::InHead,
        Self::AfterHead,
        Self::InBody,
        Self::AfterBody,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateNamespace {
    Html,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateElementName {
    Html,
    Head,
    Body,
    Div,
    Section,
}

impl CandidateElementName {
    fn is_selected(self) -> bool {
        matches!(self, Self::Div | Self::Section)
    }

    fn is_scope_boundary(self) -> bool {
        matches!(self, Self::Html)
    }

    fn is_implied_end_element(self) -> bool {
        false
    }

    fn permitted_at_in_body_eof(self) -> bool {
        matches!(self, Self::Html | Self::Body)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateScripting {
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateConfiguration {
    scripting: CandidateScripting,
    fragment_parse: bool,
    template_state: bool,
    table_state: bool,
    foreign_content_state: bool,
    reentrant_state: bool,
}

impl CandidateConfiguration {
    const FIXED: Self = Self {
        scripting: CandidateScripting::Disabled,
        fragment_parse: false,
        template_state: false,
        table_state: false,
        foreign_content_state: false,
        reentrant_state: false,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateNodeId(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateOrigin {
    Authored {
        complete: CandidateEvidence,
        raw_name: CandidateEvidence,
    },
    Synthesized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTrigger {
    Authored {
        index: usize,
        evidence: CandidateEvidence,
    },
    EndOfFile {
        index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateUnsupported {
    SelectedStartTagAttribute,
    SelectedSelfClosingStartTag,
    SelectedEndTagAttribute,
    SelectedTagOutsideInBody,
    BodyCloseWithOpenSelectedElements,
    ShellTagAttribute,
    SelfClosingShellTag,
    PElement,
    GenericOrdinaryElement,
    WhitespaceSensitiveCharacterData,
    OutsideModelledCandidateCells,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDiagnosticCode {
    MissingDoctype,
    UnmatchedSelectedEndTag,
    MisnestedSelectedEndTag,
    OpenSelectedElementsAtEndOfFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateRecovery {
    ContinuedInQuirksDocumentMode {
        trigger: CandidateTrigger,
    },
    IgnoredToken {
        trigger: CandidateTrigger,
    },
    PoppedBySelectedAncestorEndTag {
        popped_identity: CandidateNodeId,
        target_identity: CandidateNodeId,
        exact_end_trigger: CandidateTrigger,
    },
    StoppedParsingWithOpenSelectedElements {
        trigger: CandidateTrigger,
    },
}

impl CandidateRecovery {
    fn trigger(&self) -> &CandidateTrigger {
        match self {
            Self::ContinuedInQuirksDocumentMode { trigger }
            | Self::IgnoredToken { trigger }
            | Self::StoppedParsingWithOpenSelectedElements { trigger } => trigger,
            Self::PoppedBySelectedAncestorEndTag {
                exact_end_trigger, ..
            } => exact_end_trigger,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateDiagnostic {
    code: CandidateDiagnosticCode,
    trigger: CandidateTrigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateClosure {
    target_identity: CandidateNodeId,
    exact_same_name_end_trigger: CandidateTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateStackEntry {
    identity: CandidateNodeId,
    name: CandidateElementName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDisposition {
    Consumed,
    Ignored,
    Reprocessed,
    Stopped,
    Refused(CandidateUnsupported),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateDispatch {
    evaluated_in: CandidateMode,
    disposition: CandidateDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateTokenRecord {
    index: usize,
    trigger: CandidateTrigger,
    mode_before: CandidateMode,
    mode_after: CandidateMode,
    dispatches: Vec<CandidateDispatch>,
    open_selected_before: Vec<CandidateStackEntry>,
    open_selected_after: Vec<CandidateStackEntry>,
    identity_count_before: usize,
    identity_count_after: usize,
    committed_prefix_end: usize,
    processed_tokens: usize,
}

impl CandidateTokenRecord {
    fn refusal(&self) -> Option<CandidateUnsupported> {
        match self.dispatches.last()?.disposition {
            CandidateDisposition::Refused(capability) => Some(capability),
            _ => None,
        }
    }

    fn stopped(&self) -> bool {
        matches!(
            self.dispatches.last().map(|dispatch| dispatch.disposition),
            Some(CandidateDisposition::Stopped)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTree {
    Document {
        id: CandidateNodeId,
        children: Vec<CandidateTree>,
    },
    Element {
        id: CandidateNodeId,
        name: CandidateElementName,
        namespace: CandidateNamespace,
        origin: CandidateOrigin,
        children: Vec<CandidateTree>,
    },
    Text {
        id: CandidateNodeId,
        interpreted: String,
        contributions: Vec<CandidateEvidence>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateCompletion {
    Complete,
    IncompleteUnsupported {
        capability: CandidateUnsupported,
        trigger: CandidateTrigger,
    },
    IncompleteLowerLayer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateCheckpoint {
    mode: CandidateMode,
    open_selected: Vec<CandidateStackEntry>,
    committed_prefix_end: usize,
    processed_tokens: usize,
    completion: CandidateCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateSemanticObservation {
    tree: CandidateTree,
    diagnostics: Vec<CandidateDiagnostic>,
    recovery: Vec<CandidateRecovery>,
    closures: Vec<CandidateClosure>,
    identity_count: usize,
    checkpoint: CandidateCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateObservation {
    semantic: CandidateSemanticObservation,
    tokens: Vec<CandidateTokenRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTokenShape<'run> {
    Characters {
        source: CandidateEvidence,
        interpreted: &'run str,
    },
    StartTag {
        name: CandidateElementName,
        complete: CandidateEvidence,
        raw_name: CandidateEvidence,
    },
    EndTag {
        name: CandidateElementName,
        complete: CandidateEvidence,
    },
    EndOfFile {
        at: usize,
    },
}

impl CandidateTokenShape<'_> {
    fn committed_end(&self) -> usize {
        match self {
            Self::Characters { source, .. } => source.range.1,
            Self::StartTag { complete, .. } | Self::EndTag { complete, .. } => complete.range.1,
            Self::EndOfFile { at } => *at,
        }
    }

    fn is_selected_tag(&self) -> bool {
        matches!(
            self,
            Self::StartTag { name, .. } | Self::EndTag { name, .. } if name.is_selected()
        )
    }
}

fn candidate_element_name(interpreted: &str) -> Option<CandidateElementName> {
    match interpreted {
        "html" => Some(CandidateElementName::Html),
        "head" => Some(CandidateElementName::Head),
        "body" => Some(CandidateElementName::Body),
        "div" => Some(CandidateElementName::Div),
        "section" => Some(CandidateElementName::Section),
        _ => None,
    }
}

fn candidate_trigger(token: &HtmlToken, index: usize) -> CandidateTrigger {
    match token {
        HtmlToken::Character(character) => CandidateTrigger::Authored {
            index,
            evidence: evidence(character.source()),
        },
        HtmlToken::Tag(tag) => CandidateTrigger::Authored {
            index,
            evidence: evidence(tag.complete()),
        },
        HtmlToken::EndOfFile(_) => CandidateTrigger::EndOfFile { index },
    }
}

fn candidate_shape(token: &HtmlToken) -> Result<CandidateTokenShape<'_>, CandidateUnsupported> {
    match token {
        HtmlToken::Character(character) => Ok(CandidateTokenShape::Characters {
            source: evidence(character.source()),
            interpreted: character.interpreted(),
        }),
        HtmlToken::Tag(tag) => {
            let interpreted = tag.name().interpreted();
            let Some(name) = candidate_element_name(interpreted) else {
                return Err(if interpreted == "p" {
                    CandidateUnsupported::PElement
                } else {
                    CandidateUnsupported::GenericOrdinaryElement
                });
            };
            if !tag.attributes().is_empty() {
                return Err(if name.is_selected() {
                    match tag.kind() {
                        HtmlTagKind::Start => CandidateUnsupported::SelectedStartTagAttribute,
                        HtmlTagKind::End => CandidateUnsupported::SelectedEndTagAttribute,
                    }
                } else {
                    CandidateUnsupported::ShellTagAttribute
                });
            }
            if tag.self_closing_solidus().is_some() {
                return Err(if name.is_selected() && tag.kind() == HtmlTagKind::Start {
                    CandidateUnsupported::SelectedSelfClosingStartTag
                } else {
                    CandidateUnsupported::SelfClosingShellTag
                });
            }
            match tag.kind() {
                HtmlTagKind::Start => Ok(CandidateTokenShape::StartTag {
                    name,
                    complete: evidence(tag.complete()),
                    raw_name: evidence(tag.name().source()),
                }),
                HtmlTagKind::End => Ok(CandidateTokenShape::EndTag {
                    name,
                    complete: evidence(tag.complete()),
                }),
            }
        }
        HtmlToken::EndOfFile(end_of_file) => Ok(CandidateTokenShape::EndOfFile {
            at: end_of_file.source().range().start(),
        }),
    }
}

fn is_html_whitespace(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\u{000c}' | '\r' | ' ')
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateSelectedEndPlan {
    target_identity: CandidateNodeId,
    target_name: CandidateElementName,
    intervening: Vec<CandidateNodeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateEffect {
    MissingDoctype,
    InsertShell {
        name: CandidateElementName,
        authored: bool,
    },
    CloseHead,
    InsertCharacters,
    InsertSelected {
        name: CandidateElementName,
    },
    CloseSelected {
        plan: CandidateSelectedEndPlan,
    },
    UnmatchedSelected {
        name: CandidateElementName,
    },
    OpenSelectedAtEof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateStep {
    Consume {
        effect: Option<CandidateEffect>,
        next: Option<CandidateMode>,
    },
    Ignore {
        effect: CandidateEffect,
    },
    Reprocess {
        effect: Option<CandidateEffect>,
        next: CandidateMode,
    },
    Stop {
        effect: Option<CandidateEffect>,
    },
}

fn reject_whitespace_sensitive(
    shape: &CandidateTokenShape<'_>,
) -> Result<(), CandidateUnsupported> {
    if let CandidateTokenShape::Characters { interpreted, .. } = shape
        && interpreted.chars().any(is_html_whitespace)
    {
        return Err(CandidateUnsupported::WhitespaceSensitiveCharacterData);
    }
    Ok(())
}

fn expect_shell_walk_trigger(shape: &CandidateTokenShape<'_>) -> Result<(), CandidateUnsupported> {
    if shape.is_selected_tag() {
        return Err(CandidateUnsupported::SelectedTagOutsideInBody);
    }
    match shape {
        CandidateTokenShape::StartTag {
            name: CandidateElementName::Body,
            ..
        }
        | CandidateTokenShape::Characters { .. } => Ok(()),
        _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
    }
}

fn prepare_selected_end(
    selected_stack: &[CandidateStackEntry],
    target_name: CandidateElementName,
) -> Option<CandidateSelectedEndPlan> {
    assert!(target_name.is_selected());
    let target_position = selected_stack
        .iter()
        .rposition(|entry| entry.name == target_name)?;
    let target = selected_stack[target_position];
    let intervening = selected_stack[target_position + 1..]
        .iter()
        .rev()
        .map(|entry| entry.identity)
        .collect();
    Some(CandidateSelectedEndPlan {
        target_identity: target.identity,
        target_name,
        intervening,
    })
}

fn select(
    mode: CandidateMode,
    selected_stack: &[CandidateStackEntry],
    preflight_selected_end: Option<CandidateSelectedEndPlan>,
    shape: &CandidateTokenShape<'_>,
) -> Result<CandidateStep, CandidateUnsupported> {
    match mode {
        CandidateMode::Initial => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::MissingDoctype),
                next: CandidateMode::BeforeHtml,
            })
        }
        CandidateMode::BeforeHtml => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::InsertShell {
                    name: CandidateElementName::Html,
                    authored: false,
                }),
                next: CandidateMode::BeforeHead,
            })
        }
        CandidateMode::BeforeHead => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::InsertShell {
                    name: CandidateElementName::Head,
                    authored: false,
                }),
                next: CandidateMode::InHead,
            })
        }
        CandidateMode::InHead => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess {
                effect: Some(CandidateEffect::CloseHead),
                next: CandidateMode::AfterHead,
            })
        }
        CandidateMode::AfterHead => {
            reject_whitespace_sensitive(shape)?;
            expect_shell_walk_trigger(shape)?;
            match shape {
                CandidateTokenShape::StartTag {
                    name: CandidateElementName::Body,
                    ..
                } => Ok(CandidateStep::Consume {
                    effect: Some(CandidateEffect::InsertShell {
                        name: CandidateElementName::Body,
                        authored: true,
                    }),
                    next: Some(CandidateMode::InBody),
                }),
                _ => Ok(CandidateStep::Reprocess {
                    effect: Some(CandidateEffect::InsertShell {
                        name: CandidateElementName::Body,
                        authored: false,
                    }),
                    next: CandidateMode::InBody,
                }),
            }
        }
        CandidateMode::InBody => match shape {
            CandidateTokenShape::Characters { .. } => Ok(CandidateStep::Consume {
                effect: Some(CandidateEffect::InsertCharacters),
                next: None,
            }),
            CandidateTokenShape::StartTag { name, .. } if name.is_selected() => {
                Ok(CandidateStep::Consume {
                    effect: Some(CandidateEffect::InsertSelected { name: *name }),
                    next: None,
                })
            }
            CandidateTokenShape::EndTag { name, .. } if name.is_selected() => {
                match preflight_selected_end {
                    Some(plan) => Ok(CandidateStep::Consume {
                        effect: Some(CandidateEffect::CloseSelected { plan }),
                        next: None,
                    }),
                    None => Ok(CandidateStep::Ignore {
                        effect: CandidateEffect::UnmatchedSelected { name: *name },
                    }),
                }
            }
            CandidateTokenShape::EndTag {
                name: CandidateElementName::Body,
                ..
            } if !selected_stack.is_empty() => {
                Err(CandidateUnsupported::BodyCloseWithOpenSelectedElements)
            }
            CandidateTokenShape::EndTag {
                name: CandidateElementName::Body,
                ..
            } => Ok(CandidateStep::Consume {
                effect: None,
                next: Some(CandidateMode::AfterBody),
            }),
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop {
                effect: (!selected_stack.is_empty()).then_some(CandidateEffect::OpenSelectedAtEof),
            }),
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
        CandidateMode::AfterBody => match shape {
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop { effect: None }),
            _ if shape.is_selected_tag() => Err(CandidateUnsupported::SelectedTagOutsideInBody),
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateArenaKind {
    Document,
    Element {
        name: CandidateElementName,
        namespace: CandidateNamespace,
        origin: CandidateOrigin,
    },
    Text {
        interpreted: String,
        contributions: Vec<CandidateEvidence>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateArenaNode {
    id: CandidateNodeId,
    children: Vec<CandidateNodeId>,
    kind: CandidateArenaKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateStorageLayout {
    leading_padding: usize,
    padding_before_each_node: usize,
    padding_after_even_identity: usize,
}

impl CandidateStorageLayout {
    const COMPACT: Self = Self {
        leading_padding: 0,
        padding_before_each_node: 0,
        padding_after_even_identity: 0,
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateStateFingerprint {
    configuration: CandidateConfiguration,
    active_formatting_elements: Vec<CandidateNodeId>,
    layout: CandidateStorageLayout,
    slots: Vec<Option<CandidateArenaNode>>,
    document: CandidateNodeId,
    tree: CandidateTree,
    open_elements: Vec<CandidateNodeId>,
    head: Option<CandidateNodeId>,
    mode: CandidateMode,
    diagnostics: Vec<CandidateDiagnostic>,
    recovery: Vec<CandidateRecovery>,
    closures: Vec<CandidateClosure>,
    identity_count: usize,
    committed_prefix_end: usize,
    processed_tokens: usize,
}

struct CandidateSession {
    configuration: CandidateConfiguration,
    active_formatting_elements: Vec<CandidateNodeId>,
    layout: CandidateStorageLayout,
    slots: Vec<Option<CandidateArenaNode>>,
    document: CandidateNodeId,
    open_elements: Vec<CandidateNodeId>,
    head: Option<CandidateNodeId>,
    mode: CandidateMode,
    diagnostics: Vec<CandidateDiagnostic>,
    recovery: Vec<CandidateRecovery>,
    closures: Vec<CandidateClosure>,
    identity_count: usize,
    committed_prefix_end: usize,
    processed_tokens: usize,
}

impl CandidateSession {
    fn new(layout: CandidateStorageLayout) -> Self {
        let mut session = Self {
            configuration: CandidateConfiguration::FIXED,
            active_formatting_elements: Vec::new(),
            layout,
            slots: Vec::new(),
            document: CandidateNodeId(0),
            open_elements: Vec::new(),
            head: None,
            mode: CandidateMode::Initial,
            diagnostics: Vec::new(),
            recovery: Vec::new(),
            closures: Vec::new(),
            identity_count: 0,
            committed_prefix_end: 0,
            processed_tokens: 0,
        };
        session.document = session.allocate(CandidateArenaKind::Document);
        session
    }

    fn allocate(&mut self, kind: CandidateArenaKind) -> CandidateNodeId {
        if self.slots.is_empty() {
            for _ in 0..self.layout.leading_padding {
                self.slots.push(None);
            }
        }
        for _ in 0..self.layout.padding_before_each_node {
            self.slots.push(None);
        }
        let id = CandidateNodeId(self.identity_count);
        self.identity_count += 1;
        self.slots.push(Some(CandidateArenaNode {
            id,
            children: Vec::new(),
            kind,
        }));
        if id.0.is_multiple_of(2) {
            for _ in 0..self.layout.padding_after_even_identity {
                self.slots.push(None);
            }
        }
        id
    }

    fn node(&self, id: CandidateNodeId) -> &CandidateArenaNode {
        self.slots
            .iter()
            .flatten()
            .find(|node| node.id == id)
            .expect("semantic node identity")
    }

    fn node_mut(&mut self, id: CandidateNodeId) -> &mut CandidateArenaNode {
        self.slots
            .iter_mut()
            .flatten()
            .find(|node| node.id == id)
            .expect("semantic node identity")
    }

    fn storage_index(&self, id: CandidateNodeId) -> usize {
        self.slots
            .iter()
            .position(|slot| slot.as_ref().is_some_and(|node| node.id == id))
            .expect("stored semantic identity")
    }

    fn element_name(&self, id: CandidateNodeId) -> CandidateElementName {
        match &self.node(id).kind {
            CandidateArenaKind::Element { name, .. } => *name,
            _ => panic!("open-elements entry must be an element"),
        }
    }

    fn element_namespace(&self, id: CandidateNodeId) -> CandidateNamespace {
        match &self.node(id).kind {
            CandidateArenaKind::Element { namespace, .. } => *namespace,
            _ => panic!("open-elements entry must be an element"),
        }
    }

    fn current_node(&self) -> Option<CandidateNodeId> {
        self.open_elements.last().copied()
    }

    fn open_selected(&self) -> Vec<CandidateStackEntry> {
        self.open_elements
            .iter()
            .filter_map(|id| {
                let name = self.element_name(*id);
                name.is_selected().then_some(CandidateStackEntry {
                    identity: *id,
                    name,
                })
            })
            .collect()
    }

    fn prepare_selected_end_in_bounded_scope(
        &self,
        target_name: CandidateElementName,
    ) -> Option<CandidateSelectedEndPlan> {
        assert!(target_name.is_selected());
        let mut intervening = Vec::new();
        for identity in self.open_elements.iter().rev() {
            let name = self.element_name(*identity);
            if name == target_name {
                let plan = CandidateSelectedEndPlan {
                    target_identity: *identity,
                    target_name,
                    intervening,
                };
                assert_eq!(
                    Some(plan.clone()),
                    prepare_selected_end(&self.open_selected(), target_name)
                );
                return Some(plan);
            }
            if name.is_scope_boundary() {
                return None;
            }
            if name.is_selected() {
                intervening.push(*identity);
            }
        }
        None
    }

    fn has_element_in_scope(&self, target: CandidateElementName) -> bool {
        for id in self.open_elements.iter().rev() {
            let name = self.element_name(*id);
            if name == target {
                return true;
            }
            if name.is_scope_boundary() {
                return false;
            }
        }
        false
    }

    fn has_p_in_button_scope(&self) -> bool {
        for id in self.open_elements.iter().rev() {
            if self.element_name(*id).is_scope_boundary() {
                return false;
            }
        }
        false
    }

    fn generate_implied_end_tags(&mut self) -> usize {
        let mut popped = 0;
        while let Some(id) = self.current_node() {
            if !self.element_name(id).is_implied_end_element() {
                break;
            }
            self.open_elements.pop();
            popped += 1;
        }
        popped
    }

    fn assert_invariant(&self) {
        assert_eq!(self.configuration, CandidateConfiguration::FIXED);
        assert_eq!(self.configuration.scripting, CandidateScripting::Disabled);
        assert!(!self.configuration.fragment_parse);
        assert!(!self.configuration.template_state);
        assert!(!self.configuration.table_state);
        assert!(!self.configuration.foreign_content_state);
        assert!(!self.configuration.reentrant_state);
        assert!(self.active_formatting_elements.is_empty());

        let names: Vec<CandidateElementName> = self
            .open_elements
            .iter()
            .map(|id| self.element_name(*id))
            .collect();
        for id in &self.open_elements {
            assert_eq!(self.element_namespace(*id), CandidateNamespace::Html);
        }
        for name in &names {
            assert!(!name.is_implied_end_element());
            assert!(*name == CandidateElementName::Html || !name.is_scope_boundary());
        }
        let valid = match names.as_slice() {
            [] | [CandidateElementName::Html] => true,
            [CandidateElementName::Html, CandidateElementName::Head] => true,
            [
                CandidateElementName::Html,
                CandidateElementName::Body,
                rest @ ..,
            ] => rest.iter().all(|name| name.is_selected()),
            _ => false,
        };
        assert!(valid, "candidate stack invariant: {names:?}");
    }

    fn fingerprint(&self) -> CandidateStateFingerprint {
        CandidateStateFingerprint {
            configuration: self.configuration,
            active_formatting_elements: self.active_formatting_elements.clone(),
            layout: self.layout,
            slots: self.slots.clone(),
            document: self.document,
            tree: self.tree(),
            open_elements: self.open_elements.clone(),
            head: self.head,
            mode: self.mode,
            diagnostics: self.diagnostics.clone(),
            recovery: self.recovery.clone(),
            closures: self.closures.clone(),
            identity_count: self.identity_count,
            committed_prefix_end: self.committed_prefix_end,
            processed_tokens: self.processed_tokens,
        }
    }

    fn process(
        &mut self,
        index: usize,
        shape: CandidateTokenShape<'_>,
        trigger: CandidateTrigger,
    ) -> CandidateTokenRecord {
        let mode_before = self.mode;
        let open_selected_before = self.open_selected();
        let identity_count_before = self.identity_count;
        let mut dispatches = Vec::new();
        let mut visited = Vec::new();

        loop {
            self.assert_invariant();
            assert!(
                !visited.contains(&self.mode),
                "same token revisited mode {:?}",
                self.mode
            );
            visited.push(self.mode);
            let before = self.fingerprint();
            let evaluated_in = self.mode;
            let selected_stack = self.open_selected();
            let preflight_selected_end = match &shape {
                CandidateTokenShape::EndTag { name, .. }
                    if self.mode == CandidateMode::InBody && name.is_selected() =>
                {
                    self.prepare_selected_end_in_bounded_scope(*name)
                }
                _ => None,
            };

            match select(self.mode, &selected_stack, preflight_selected_end, &shape) {
                Err(capability) => {
                    assert_eq!(self.fingerprint(), before, "refusal mutates nothing");
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        disposition: CandidateDisposition::Refused(capability),
                    });
                    break;
                }
                Ok(CandidateStep::Stop { effect }) => {
                    if let Some(effect) = effect {
                        self.apply(effect, &trigger, &shape);
                    }
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        disposition: CandidateDisposition::Stopped,
                    });
                    break;
                }
                Ok(CandidateStep::Consume { effect, next }) => {
                    if let Some(effect) = effect {
                        self.apply(effect, &trigger, &shape);
                    }
                    if let Some(next) = next {
                        self.mode = next;
                    }
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        disposition: CandidateDisposition::Consumed,
                    });
                    break;
                }
                Ok(CandidateStep::Ignore { effect }) => {
                    self.apply(effect, &trigger, &shape);
                    self.commit(&shape);
                    let after = self.fingerprint();
                    assert_eq!(after.tree, before.tree);
                    assert_eq!(after.open_elements, before.open_elements);
                    assert_eq!(after.mode, before.mode);
                    assert_eq!(after.closures, before.closures);
                    assert_eq!(after.identity_count, before.identity_count);
                    assert_eq!(after.diagnostics.len(), before.diagnostics.len() + 1);
                    assert_eq!(after.recovery.len(), before.recovery.len() + 1);
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        disposition: CandidateDisposition::Ignored,
                    });
                    break;
                }
                Ok(CandidateStep::Reprocess { effect, next }) => {
                    if let Some(effect) = effect {
                        self.apply(effect, &trigger, &shape);
                    }
                    self.mode = next;
                    dispatches.push(CandidateDispatch {
                        evaluated_in,
                        disposition: CandidateDisposition::Reprocessed,
                    });
                }
            }
        }

        self.assert_invariant();
        CandidateTokenRecord {
            index,
            trigger,
            mode_before,
            mode_after: self.mode,
            dispatches,
            open_selected_before,
            open_selected_after: self.open_selected(),
            identity_count_before,
            identity_count_after: self.identity_count,
            committed_prefix_end: self.committed_prefix_end,
            processed_tokens: self.processed_tokens,
        }
    }

    fn apply(
        &mut self,
        effect: CandidateEffect,
        trigger: &CandidateTrigger,
        shape: &CandidateTokenShape<'_>,
    ) {
        match effect {
            CandidateEffect::MissingDoctype => {
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::MissingDoctype,
                    trigger: trigger.clone(),
                });
                self.recovery
                    .push(CandidateRecovery::ContinuedInQuirksDocumentMode {
                        trigger: trigger.clone(),
                    });
            }
            CandidateEffect::InsertShell { name, authored } => {
                self.insert_shell(name, authored, shape);
            }
            CandidateEffect::CloseHead => {
                let head = self.head.expect("head element");
                assert_eq!(self.open_elements.last(), Some(&head));
                self.open_elements.pop();
            }
            CandidateEffect::InsertCharacters => self.insert_characters(shape),
            CandidateEffect::InsertSelected { name } => self.insert_selected(name, shape),
            CandidateEffect::CloseSelected { plan } => {
                self.close_selected(plan, trigger, shape);
            }
            CandidateEffect::UnmatchedSelected { name } => {
                assert!(name.is_selected());
                assert!(!self.has_element_in_scope(name));
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::UnmatchedSelectedEndTag,
                    trigger: trigger.clone(),
                });
                self.recovery.push(CandidateRecovery::IgnoredToken {
                    trigger: trigger.clone(),
                });
            }
            CandidateEffect::OpenSelectedAtEof => {
                assert!(
                    self.open_elements
                        .iter()
                        .map(|id| self.element_name(*id))
                        .any(|name| !name.permitted_at_in_body_eof())
                );
                assert!(matches!(trigger, CandidateTrigger::EndOfFile { .. }));
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::OpenSelectedElementsAtEndOfFile,
                    trigger: trigger.clone(),
                });
                self.recovery
                    .push(CandidateRecovery::StoppedParsingWithOpenSelectedElements {
                        trigger: trigger.clone(),
                    });
            }
        }
    }

    fn insert_shell(
        &mut self,
        name: CandidateElementName,
        authored: bool,
        shape: &CandidateTokenShape<'_>,
    ) {
        let parent = if name == CandidateElementName::Html {
            self.document
        } else {
            self.current_node().expect("shell insertion parent")
        };
        let origin = if authored {
            let CandidateTokenShape::StartTag {
                complete, raw_name, ..
            } = shape
            else {
                panic!("authored shell insertion requires a start tag")
            };
            CandidateOrigin::Authored {
                complete: complete.clone(),
                raw_name: raw_name.clone(),
            }
        } else {
            CandidateOrigin::Synthesized
        };
        let id = self.allocate(CandidateArenaKind::Element {
            name,
            namespace: CandidateNamespace::Html,
            origin,
        });
        self.node_mut(parent).children.push(id);
        self.open_elements.push(id);
        if name == CandidateElementName::Head {
            self.head = Some(id);
        }
    }

    fn insert_selected(
        &mut self,
        selected_name: CandidateElementName,
        shape: &CandidateTokenShape<'_>,
    ) {
        assert!(!self.has_p_in_button_scope());
        assert!(selected_name.is_selected());
        let CandidateTokenShape::StartTag {
            name,
            complete,
            raw_name,
        } = shape
        else {
            panic!("selected insertion requires a selected start tag")
        };
        assert_eq!(*name, selected_name);
        let parent = self.current_node().expect("selected insertion parent");
        let id = self.allocate(CandidateArenaKind::Element {
            name: selected_name,
            namespace: CandidateNamespace::Html,
            origin: CandidateOrigin::Authored {
                complete: complete.clone(),
                raw_name: raw_name.clone(),
            },
        });
        self.node_mut(parent).children.push(id);
        self.open_elements.push(id);
    }

    fn close_selected(
        &mut self,
        plan: CandidateSelectedEndPlan,
        trigger: &CandidateTrigger,
        shape: &CandidateTokenShape<'_>,
    ) {
        assert_eq!(
            prepare_selected_end(&self.open_selected(), plan.target_name),
            Some(plan.clone()),
            "selected end plan is completely pre-resolved before mutation"
        );
        assert!(self.has_element_in_scope(plan.target_name));
        let CandidateTokenShape::EndTag { name, complete } = shape else {
            panic!("selected closure requires a selected end tag")
        };
        assert_eq!(*name, plan.target_name);
        let CandidateTrigger::Authored {
            evidence: trigger_evidence,
            ..
        } = trigger
        else {
            panic!("selected closure trigger must be authored")
        };
        assert_eq!(trigger_evidence, complete);
        assert_eq!(self.generate_implied_end_tags(), 0);

        if !plan.intervening.is_empty() {
            self.diagnostics.push(CandidateDiagnostic {
                code: CandidateDiagnosticCode::MisnestedSelectedEndTag,
                trigger: trigger.clone(),
            });
        }
        for popped_identity in &plan.intervening {
            assert_eq!(self.current_node(), Some(*popped_identity));
            self.open_elements.pop();
            self.recovery
                .push(CandidateRecovery::PoppedBySelectedAncestorEndTag {
                    popped_identity: *popped_identity,
                    target_identity: plan.target_identity,
                    exact_end_trigger: trigger.clone(),
                });
        }

        assert_eq!(self.current_node(), Some(plan.target_identity));
        assert_eq!(self.element_name(plan.target_identity), plan.target_name);
        self.open_elements.pop();
        self.closures.push(CandidateClosure {
            target_identity: plan.target_identity,
            exact_same_name_end_trigger: trigger.clone(),
        });
    }

    fn insert_characters(&mut self, shape: &CandidateTokenShape<'_>) {
        let CandidateTokenShape::Characters {
            source,
            interpreted,
        } = shape
        else {
            panic!("character insertion requires a character token")
        };
        let parent = self.current_node().expect("character insertion parent");
        let adjacent = self
            .node(parent)
            .children
            .last()
            .copied()
            .filter(|id| matches!(self.node(*id).kind, CandidateArenaKind::Text { .. }));

        if let Some(id) = adjacent {
            let CandidateArenaKind::Text {
                interpreted: existing,
                contributions,
            } = &mut self.node_mut(id).kind
            else {
                unreachable!()
            };
            existing.push_str(interpreted);
            contributions.push(source.clone());
            return;
        }

        let id = self.allocate(CandidateArenaKind::Text {
            interpreted: (*interpreted).to_owned(),
            contributions: vec![source.clone()],
        });
        self.node_mut(parent).children.push(id);
    }

    fn commit(&mut self, shape: &CandidateTokenShape<'_>) {
        let end = shape.committed_end();
        assert!(end >= self.committed_prefix_end);
        self.committed_prefix_end = end;
        self.processed_tokens += 1;
    }

    fn tree(&self) -> CandidateTree {
        self.project(self.document)
    }

    fn project(&self, id: CandidateNodeId) -> CandidateTree {
        let node = self.node(id);
        let children = node
            .children
            .iter()
            .map(|child| self.project(*child))
            .collect();
        match &node.kind {
            CandidateArenaKind::Document => CandidateTree::Document { id, children },
            CandidateArenaKind::Element {
                name,
                namespace,
                origin,
            } => CandidateTree::Element {
                id,
                name: *name,
                namespace: *namespace,
                origin: origin.clone(),
                children,
            },
            CandidateArenaKind::Text {
                interpreted,
                contributions,
            } => CandidateTree::Text {
                id,
                interpreted: interpreted.clone(),
                contributions: contributions.clone(),
            },
        }
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
    tokenize(
        &SourceText::new(SourceId::new(source_id), text.to_owned()),
        limits,
    )
}

fn run_for(id: &str, source_id: u64) -> HtmlTokenizerRunResult {
    tokenize_text(fixture(id).source_text(), source_id, generous_limits())
}

fn observe_with_layout(
    run: &HtmlTokenizerRunResult,
    layout: CandidateStorageLayout,
) -> CandidateObservation {
    let mut session = CandidateSession::new(layout);
    let mut tokens = Vec::new();
    let mut refusal = None;
    let mut stopped = false;

    for (index, token) in run.tokens().iter().enumerate() {
        let trigger = candidate_trigger(token, index);
        let before_shape = session.fingerprint();
        let shape = match candidate_shape(token) {
            Ok(shape) => shape,
            Err(capability) => {
                assert_eq!(session.fingerprint(), before_shape);
                let open_selected = session.open_selected();
                tokens.push(CandidateTokenRecord {
                    index,
                    trigger: trigger.clone(),
                    mode_before: session.mode,
                    mode_after: session.mode,
                    dispatches: vec![CandidateDispatch {
                        evaluated_in: session.mode,
                        disposition: CandidateDisposition::Refused(capability),
                    }],
                    open_selected_before: open_selected.clone(),
                    open_selected_after: open_selected,
                    identity_count_before: session.identity_count,
                    identity_count_after: session.identity_count,
                    committed_prefix_end: session.committed_prefix_end,
                    processed_tokens: session.processed_tokens,
                });
                refusal = Some((capability, trigger));
                break;
            }
        };
        let record = session.process(index, shape, trigger.clone());
        if let Some(capability) = record.refusal() {
            refusal = Some((capability, trigger));
        }
        let is_refused = record.refusal().is_some();
        let is_stopped = record.stopped();
        tokens.push(record);
        if is_refused {
            break;
        }
        if is_stopped {
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
    let open_selected = session.open_selected();

    CandidateObservation {
        semantic: CandidateSemanticObservation {
            tree: session.tree(),
            diagnostics: session.diagnostics.clone(),
            recovery: session.recovery.clone(),
            closures: session.closures.clone(),
            identity_count: session.identity_count,
            checkpoint: CandidateCheckpoint {
                mode: session.mode,
                open_selected,
                committed_prefix_end: session.committed_prefix_end,
                processed_tokens: session.processed_tokens,
                completion,
            },
        },
        tokens,
    }
}

fn observe(run: &HtmlTokenizerRunResult) -> CandidateObservation {
    observe_with_layout(run, CandidateStorageLayout::COMPACT)
}

fn observe_fixture(id: &str, source_id: u64) -> CandidateObservation {
    observe(&run_for(id, source_id))
}

fn node_id(value: usize) -> CandidateNodeId {
    CandidateNodeId(value)
}

fn expected(source_id: u64, range: (usize, usize)) -> CandidateEvidence {
    expected_evidence(source_id, range)
}

fn authored_origin(
    source_id: u64,
    complete: (usize, usize),
    raw_name: (usize, usize),
) -> CandidateOrigin {
    CandidateOrigin::Authored {
        complete: expected(source_id, complete),
        raw_name: expected(source_id, raw_name),
    }
}

fn element(
    id: usize,
    name: CandidateElementName,
    origin: CandidateOrigin,
    children: Vec<CandidateTree>,
) -> CandidateTree {
    CandidateTree::Element {
        id: node_id(id),
        name,
        namespace: CandidateNamespace::Html,
        origin,
        children,
    }
}

fn text(
    id: usize,
    source_id: u64,
    interpreted: &str,
    contributions: &[(usize, usize)],
) -> CandidateTree {
    CandidateTree::Text {
        id: node_id(id),
        interpreted: interpreted.to_owned(),
        contributions: contributions
            .iter()
            .map(|range| expected(source_id, *range))
            .collect(),
    }
}

fn shell(source_id: u64, authored_body: bool, body_children: Vec<CandidateTree>) -> CandidateTree {
    CandidateTree::Document {
        id: node_id(0),
        children: vec![element(
            1,
            CandidateElementName::Html,
            CandidateOrigin::Synthesized,
            vec![
                element(
                    2,
                    CandidateElementName::Head,
                    CandidateOrigin::Synthesized,
                    vec![],
                ),
                element(
                    3,
                    CandidateElementName::Body,
                    if authored_body {
                        authored_origin(source_id, (0, 6), (1, 5))
                    } else {
                        CandidateOrigin::Synthesized
                    },
                    body_children,
                ),
            ],
        )],
    }
}

fn authored_trigger(source_id: u64, index: usize, range: (usize, usize)) -> CandidateTrigger {
    CandidateTrigger::Authored {
        index,
        evidence: expected(source_id, range),
    }
}

fn diagnostic(code: CandidateDiagnosticCode, trigger: CandidateTrigger) -> CandidateDiagnostic {
    CandidateDiagnostic { code, trigger }
}

fn missing_doctype(source_id: u64, index: usize, range: (usize, usize)) -> CandidateDiagnostic {
    diagnostic(
        CandidateDiagnosticCode::MissingDoctype,
        authored_trigger(source_id, index, range),
    )
}

fn continued_in_quirks(source_id: u64, index: usize, range: (usize, usize)) -> CandidateRecovery {
    CandidateRecovery::ContinuedInQuirksDocumentMode {
        trigger: authored_trigger(source_id, index, range),
    }
}

fn ignored_token(source_id: u64, index: usize, range: (usize, usize)) -> CandidateRecovery {
    CandidateRecovery::IgnoredToken {
        trigger: authored_trigger(source_id, index, range),
    }
}

fn recovery_pop(
    source_id: u64,
    popped_identity: usize,
    target_identity: usize,
    index: usize,
    range: (usize, usize),
) -> CandidateRecovery {
    CandidateRecovery::PoppedBySelectedAncestorEndTag {
        popped_identity: node_id(popped_identity),
        target_identity: node_id(target_identity),
        exact_end_trigger: authored_trigger(source_id, index, range),
    }
}

fn stopped_with_open_selected(index: usize) -> CandidateRecovery {
    CandidateRecovery::StoppedParsingWithOpenSelectedElements {
        trigger: CandidateTrigger::EndOfFile { index },
    }
}

fn closure(
    source_id: u64,
    target_identity: usize,
    index: usize,
    range: (usize, usize),
) -> CandidateClosure {
    CandidateClosure {
        target_identity: node_id(target_identity),
        exact_same_name_end_trigger: authored_trigger(source_id, index, range),
    }
}

fn selected_entry(identity: usize, name: CandidateElementName) -> CandidateStackEntry {
    CandidateStackEntry {
        identity: node_id(identity),
        name,
    }
}

fn checkpoint(
    mode: CandidateMode,
    open_selected: Vec<CandidateStackEntry>,
    committed_prefix_end: usize,
    processed_tokens: usize,
    completion: CandidateCompletion,
) -> CandidateCheckpoint {
    CandidateCheckpoint {
        mode,
        open_selected,
        committed_prefix_end,
        processed_tokens,
        completion,
    }
}

fn semantic(
    tree: CandidateTree,
    diagnostics: Vec<CandidateDiagnostic>,
    recovery: Vec<CandidateRecovery>,
    closures: Vec<CandidateClosure>,
    identity_count: usize,
    checkpoint: CandidateCheckpoint,
) -> CandidateSemanticObservation {
    CandidateSemanticObservation {
        tree,
        diagnostics,
        recovery,
        closures,
        identity_count,
        checkpoint,
    }
}

// Hand-authored canonical GOLD. This function never calls the candidate machine.
fn candidate_gold(id: &str, source_id: u64) -> CandidateSemanticObservation {
    let missing_body = || missing_doctype(source_id, 0, (0, 6));
    let continued_body = || continued_in_quirks(source_id, 0, (0, 6));
    match id {
        "HS1" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![closure(source_id, 4, 2, (15, 25))],
            5,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                25,
                4,
                CandidateCompletion::Complete,
            ),
        ),
        "HS2" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![text(5, source_id, "x", &[(15, 16)])],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![closure(source_id, 4, 3, (16, 26))],
            6,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                26,
                5,
                CandidateCompletion::Complete,
            ),
        ),
        "HS3" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![element(
                        5,
                        CandidateElementName::Section,
                        authored_origin(source_id, (15, 24), (16, 23)),
                        vec![text(6, source_id, "x", &[(24, 25)])],
                    )],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![
                closure(source_id, 5, 4, (25, 35)),
                closure(source_id, 4, 5, (35, 45)),
            ],
            7,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                45,
                7,
                CandidateCompletion::Complete,
            ),
        ),
        "HS4" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Div,
                    authored_origin(source_id, (6, 11), (7, 10)),
                    vec![element(
                        5,
                        CandidateElementName::Section,
                        authored_origin(source_id, (11, 20), (12, 19)),
                        vec![],
                    )],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![
                closure(source_id, 5, 3, (20, 30)),
                closure(source_id, 4, 4, (30, 36)),
            ],
            6,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                36,
                6,
                CandidateCompletion::Complete,
            ),
        ),
        "HS5" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![element(
                        5,
                        CandidateElementName::Div,
                        authored_origin(source_id, (15, 20), (16, 19)),
                        vec![],
                    )],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![
                closure(source_id, 5, 3, (20, 26)),
                closure(source_id, 4, 4, (26, 36)),
            ],
            6,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                36,
                6,
                CandidateCompletion::Complete,
            ),
        ),
        "HS6" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![element(
                        5,
                        CandidateElementName::Div,
                        authored_origin(source_id, (15, 20), (16, 19)),
                        vec![],
                    )],
                )],
            ),
            vec![
                missing_body(),
                diagnostic(
                    CandidateDiagnosticCode::MisnestedSelectedEndTag,
                    authored_trigger(source_id, 3, (20, 30)),
                ),
            ],
            vec![continued_body(), recovery_pop(source_id, 5, 4, 3, (20, 30))],
            vec![closure(source_id, 4, 3, (20, 30))],
            6,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                30,
                5,
                CandidateCompletion::Complete,
            ),
        ),
        "HS7" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Div,
                    authored_origin(source_id, (6, 11), (7, 10)),
                    vec![element(
                        5,
                        CandidateElementName::Section,
                        authored_origin(source_id, (11, 20), (12, 19)),
                        vec![],
                    )],
                )],
            ),
            vec![
                missing_body(),
                diagnostic(
                    CandidateDiagnosticCode::MisnestedSelectedEndTag,
                    authored_trigger(source_id, 3, (20, 26)),
                ),
            ],
            vec![continued_body(), recovery_pop(source_id, 5, 4, 3, (20, 26))],
            vec![closure(source_id, 4, 3, (20, 26))],
            6,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                26,
                5,
                CandidateCompletion::Complete,
            ),
        ),
        "HS8" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Div,
                    authored_origin(source_id, (6, 11), (7, 10)),
                    vec![element(
                        5,
                        CandidateElementName::Section,
                        authored_origin(source_id, (11, 20), (12, 19)),
                        vec![element(
                            6,
                            CandidateElementName::Div,
                            authored_origin(source_id, (20, 25), (21, 24)),
                            vec![],
                        )],
                    )],
                )],
            ),
            vec![
                missing_body(),
                diagnostic(
                    CandidateDiagnosticCode::MisnestedSelectedEndTag,
                    authored_trigger(source_id, 4, (25, 35)),
                ),
            ],
            vec![continued_body(), recovery_pop(source_id, 6, 5, 4, (25, 35))],
            vec![
                closure(source_id, 5, 4, (25, 35)),
                closure(source_id, 4, 5, (35, 41)),
            ],
            7,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                41,
                7,
                CandidateCompletion::Complete,
            ),
        ),
        "HS9" => semantic(
            shell(source_id, true, vec![]),
            vec![
                missing_body(),
                diagnostic(
                    CandidateDiagnosticCode::UnmatchedSelectedEndTag,
                    authored_trigger(source_id, 1, (6, 16)),
                ),
            ],
            vec![continued_body(), ignored_token(source_id, 1, (6, 16))],
            vec![],
            4,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                16,
                3,
                CandidateCompletion::Complete,
            ),
        ),
        "HS10" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![text(5, source_id, "x", &[(15, 16)])],
                )],
            ),
            vec![
                missing_body(),
                diagnostic(
                    CandidateDiagnosticCode::OpenSelectedElementsAtEndOfFile,
                    CandidateTrigger::EndOfFile { index: 3 },
                ),
            ],
            vec![continued_body(), stopped_with_open_selected(3)],
            vec![],
            6,
            checkpoint(
                CandidateMode::InBody,
                vec![selected_entry(4, CandidateElementName::Section)],
                16,
                4,
                CandidateCompletion::Complete,
            ),
        ),
        "HS11" => semantic(
            shell(source_id, true, vec![]),
            vec![missing_body()],
            vec![continued_body()],
            vec![],
            4,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                6,
                1,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::SelectedStartTagAttribute,
                    trigger: authored_trigger(source_id, 1, (6, 20)),
                },
            ),
        ),
        "HS12" => semantic(
            shell(source_id, true, vec![]),
            vec![missing_body()],
            vec![continued_body()],
            vec![],
            4,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                6,
                1,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::SelectedSelfClosingStartTag,
                    trigger: authored_trigger(source_id, 1, (6, 16)),
                },
            ),
        ),
        "HS13" => semantic(
            shell(source_id, true, vec![]),
            vec![missing_body()],
            vec![continued_body()],
            vec![],
            4,
            checkpoint(
                CandidateMode::AfterBody,
                vec![],
                13,
                2,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::SelectedTagOutsideInBody,
                    trigger: authored_trigger(source_id, 2, (13, 22)),
                },
            ),
        ),
        "HS14" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![],
            5,
            checkpoint(
                CandidateMode::InBody,
                vec![selected_entry(4, CandidateElementName::Section)],
                15,
                2,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::BodyCloseWithOpenSelectedElements,
                    trigger: authored_trigger(source_id, 2, (15, 22)),
                },
            ),
        ),
        "HS15" => semantic(
            shell(source_id, true, vec![]),
            vec![missing_body()],
            vec![continued_body()],
            vec![],
            4,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                6,
                1,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::PElement,
                    trigger: authored_trigger(source_id, 1, (6, 9)),
                },
            ),
        ),
        "HS16" => semantic(
            shell(source_id, true, vec![]),
            vec![missing_body()],
            vec![continued_body()],
            vec![],
            4,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                6,
                1,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::GenericOrdinaryElement,
                    trigger: authored_trigger(source_id, 1, (6, 12)),
                },
            ),
        ),
        "HS17" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![],
            5,
            checkpoint(
                CandidateMode::InBody,
                vec![selected_entry(4, CandidateElementName::Section)],
                15,
                2,
                CandidateCompletion::IncompleteLowerLayer,
            ),
        ),
        "HS18" => semantic(
            shell(
                source_id,
                false,
                vec![
                    text(4, source_id, "x", &[(0, 1)]),
                    element(
                        5,
                        CandidateElementName::Section,
                        authored_origin(source_id, (1, 10), (2, 9)),
                        vec![],
                    ),
                ],
            ),
            vec![missing_doctype(source_id, 0, (0, 1))],
            vec![continued_in_quirks(source_id, 0, (0, 1))],
            vec![closure(source_id, 5, 2, (10, 20))],
            6,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                20,
                4,
                CandidateCompletion::Complete,
            ),
        ),
        "HS19" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![closure(source_id, 4, 2, (15, 25))],
            5,
            checkpoint(
                CandidateMode::AfterBody,
                vec![],
                32,
                5,
                CandidateCompletion::Complete,
            ),
        ),
        "HS20" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Div,
                    authored_origin(source_id, (6, 11), (7, 10)),
                    vec![],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![closure(source_id, 4, 2, (11, 17))],
            5,
            checkpoint(
                CandidateMode::InBody,
                vec![],
                17,
                4,
                CandidateCompletion::Complete,
            ),
        ),
        "HS21" => semantic(
            shell(
                source_id,
                true,
                vec![element(
                    4,
                    CandidateElementName::Section,
                    authored_origin(source_id, (6, 15), (7, 14)),
                    vec![],
                )],
            ),
            vec![missing_body()],
            vec![continued_body()],
            vec![],
            5,
            checkpoint(
                CandidateMode::InBody,
                vec![selected_entry(4, CandidateElementName::Section)],
                15,
                2,
                CandidateCompletion::IncompleteUnsupported {
                    capability: CandidateUnsupported::SelectedEndTagAttribute,
                    trigger: authored_trigger(source_id, 2, (15, 30)),
                },
            ),
        ),
        other => panic!("no candidate GOLD for {other}"),
    }
}

fn node_count(tree: &CandidateTree) -> usize {
    match tree {
        CandidateTree::Document { children, .. } | CandidateTree::Element { children, .. } => {
            1 + children.iter().map(node_count).sum::<usize>()
        }
        CandidateTree::Text { .. } => 1,
    }
}

fn collect_tree_evidence(tree: &CandidateTree, into: &mut Vec<CandidateEvidence>) {
    match tree {
        CandidateTree::Document { children, .. } => {
            for child in children {
                collect_tree_evidence(child, into);
            }
        }
        CandidateTree::Element {
            origin, children, ..
        } => {
            if let CandidateOrigin::Authored { complete, raw_name } = origin {
                into.push(complete.clone());
                into.push(raw_name.clone());
            }
            for child in children {
                collect_tree_evidence(child, into);
            }
        }
        CandidateTree::Text { contributions, .. } => {
            into.extend(contributions.iter().cloned());
        }
    }
}

fn text_nodes(tree: &CandidateTree, into: &mut Vec<(String, Vec<(usize, usize)>)>) {
    match tree {
        CandidateTree::Document { children, .. } | CandidateTree::Element { children, .. } => {
            for child in children {
                text_nodes(child, into);
            }
        }
        CandidateTree::Text {
            interpreted,
            contributions,
            ..
        } => into.push((
            interpreted.clone(),
            contributions
                .iter()
                .map(|evidence| evidence.range)
                .collect(),
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExpectedAttribute {
    complete: (usize, usize),
    raw_name: (usize, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectedToken {
    Characters {
        range: (usize, usize),
        interpreted: String,
    },
    Tag {
        kind: HtmlTagKind,
        complete: (usize, usize),
        interpreted: String,
        raw_name: (usize, usize),
        attributes: Vec<ExpectedAttribute>,
        self_closing: Option<(usize, usize)>,
    },
    EndOfFile {
        at: usize,
    },
}

fn span(anchor: &crate::SourceAnchor) -> (usize, usize) {
    (anchor.range().start(), anchor.range().end())
}

fn observed_token(token: &HtmlToken) -> ExpectedToken {
    match token {
        HtmlToken::Character(character) => ExpectedToken::Characters {
            range: span(character.source()),
            interpreted: character.interpreted().to_owned(),
        },
        HtmlToken::Tag(tag) => ExpectedToken::Tag {
            kind: tag.kind(),
            complete: span(tag.complete()),
            interpreted: tag.name().interpreted().to_owned(),
            raw_name: span(tag.name().source()),
            attributes: tag
                .attributes()
                .iter()
                .map(|attribute| ExpectedAttribute {
                    complete: span(attribute.complete()),
                    raw_name: span(attribute.name().source()),
                })
                .collect(),
            self_closing: tag.self_closing_solidus().map(span),
        },
        HtmlToken::EndOfFile(end_of_file) => ExpectedToken::EndOfFile {
            at: end_of_file.source().range().start(),
        },
    }
}

fn tag(
    kind: HtmlTagKind,
    complete: (usize, usize),
    interpreted: &str,
    raw_name: (usize, usize),
) -> ExpectedToken {
    ExpectedToken::Tag {
        kind,
        complete,
        interpreted: interpreted.to_owned(),
        raw_name,
        attributes: vec![],
        self_closing: None,
    }
}

fn start(complete: (usize, usize), interpreted: &str, raw_name: (usize, usize)) -> ExpectedToken {
    tag(HtmlTagKind::Start, complete, interpreted, raw_name)
}

fn end(complete: (usize, usize), interpreted: &str, raw_name: (usize, usize)) -> ExpectedToken {
    tag(HtmlTagKind::End, complete, interpreted, raw_name)
}

fn characters(range: (usize, usize), interpreted: &str) -> ExpectedToken {
    ExpectedToken::Characters {
        range,
        interpreted: interpreted.to_owned(),
    }
}

fn eof(at: usize) -> ExpectedToken {
    ExpectedToken::EndOfFile { at }
}

fn exact_expected_tokens() -> Vec<(&'static str, Vec<ExpectedToken>)> {
    vec![
        (
            "HS1",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                end((15, 25), "section", (17, 24)),
                eof(25),
            ],
        ),
        (
            "HS2",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                characters((15, 16), "x"),
                end((16, 26), "section", (18, 25)),
                eof(26),
            ],
        ),
        (
            "HS3",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                start((15, 24), "section", (16, 23)),
                characters((24, 25), "x"),
                end((25, 35), "section", (27, 34)),
                end((35, 45), "section", (37, 44)),
                eof(45),
            ],
        ),
        (
            "HS4",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                start((11, 20), "section", (12, 19)),
                end((20, 30), "section", (22, 29)),
                end((30, 36), "div", (32, 35)),
                eof(36),
            ],
        ),
        (
            "HS5",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                start((15, 20), "div", (16, 19)),
                end((20, 26), "div", (22, 25)),
                end((26, 36), "section", (28, 35)),
                eof(36),
            ],
        ),
        (
            "HS6",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                start((15, 20), "div", (16, 19)),
                end((20, 30), "section", (22, 29)),
                eof(30),
            ],
        ),
        (
            "HS7",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                start((11, 20), "section", (12, 19)),
                end((20, 26), "div", (22, 25)),
                eof(26),
            ],
        ),
        (
            "HS8",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                start((11, 20), "section", (12, 19)),
                start((20, 25), "div", (21, 24)),
                end((25, 35), "section", (27, 34)),
                end((35, 41), "div", (37, 40)),
                eof(41),
            ],
        ),
        (
            "HS9",
            vec![
                start((0, 6), "body", (1, 5)),
                end((6, 16), "section", (8, 15)),
                eof(16),
            ],
        ),
        (
            "HS10",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                characters((15, 16), "x"),
                eof(16),
            ],
        ),
        (
            "HS11",
            vec![
                start((0, 6), "body", (1, 5)),
                ExpectedToken::Tag {
                    kind: HtmlTagKind::Start,
                    complete: (6, 20),
                    interpreted: "section".to_owned(),
                    raw_name: (7, 14),
                    attributes: vec![ExpectedAttribute {
                        complete: (15, 19),
                        raw_name: (15, 17),
                    }],
                    self_closing: None,
                },
                eof(20),
            ],
        ),
        (
            "HS12",
            vec![
                start((0, 6), "body", (1, 5)),
                ExpectedToken::Tag {
                    kind: HtmlTagKind::Start,
                    complete: (6, 16),
                    interpreted: "section".to_owned(),
                    raw_name: (7, 14),
                    attributes: vec![],
                    self_closing: Some((14, 15)),
                },
                eof(16),
            ],
        ),
        (
            "HS13",
            vec![
                start((0, 6), "body", (1, 5)),
                end((6, 13), "body", (8, 12)),
                start((13, 22), "section", (14, 21)),
                eof(22),
            ],
        ),
        (
            "HS14",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                end((15, 22), "body", (17, 21)),
                eof(22),
            ],
        ),
        (
            "HS15",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 9), "p", (7, 8)),
                eof(9),
            ],
        ),
        (
            "HS16",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 12), "span", (7, 11)),
                eof(12),
            ],
        ),
        (
            "HS17",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
            ],
        ),
        (
            "HS18",
            vec![
                characters((0, 1), "x"),
                start((1, 10), "section", (2, 9)),
                end((10, 20), "section", (12, 19)),
                eof(20),
            ],
        ),
        (
            "HS19",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                end((15, 25), "section", (17, 24)),
                end((25, 32), "body", (27, 31)),
                eof(32),
            ],
        ),
        (
            "HS20",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 11), "div", (7, 10)),
                end((11, 17), "div", (13, 16)),
                eof(17),
            ],
        ),
        (
            "HS21",
            vec![
                start((0, 6), "body", (1, 5)),
                start((6, 15), "section", (7, 14)),
                ExpectedToken::Tag {
                    kind: HtmlTagKind::End,
                    complete: (15, 30),
                    interpreted: "section".to_owned(),
                    raw_name: (17, 24),
                    attributes: vec![ExpectedAttribute {
                        complete: (25, 29),
                        raw_name: (25, 27),
                    }],
                    self_closing: None,
                },
                eof(30),
            ],
        ),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateValidationError {
    IdentityLifecycle,
    TokenRecord,
    StackTransition,
    Diagnostic,
    Recovery,
    Closure,
    Completion,
    Coverage,
    Provenance,
}

fn collect_identity_names(
    tree: &CandidateTree,
    identities: &mut Vec<(CandidateNodeId, Option<CandidateElementName>)>,
) -> Result<(), CandidateValidationError> {
    let (id, name, children) = match tree {
        CandidateTree::Document { id, children } => (*id, None, children.as_slice()),
        CandidateTree::Element {
            id, name, children, ..
        } => (*id, Some(*name), children.as_slice()),
        CandidateTree::Text { id, .. } => {
            if identities.iter().any(|(existing, _)| existing == id) {
                return Err(CandidateValidationError::IdentityLifecycle);
            }
            identities.push((*id, None));
            return Ok(());
        }
    };
    if identities.iter().any(|(existing, _)| *existing == id) {
        return Err(CandidateValidationError::IdentityLifecycle);
    }
    identities.push((id, name));
    for child in children {
        collect_identity_names(child, identities)?;
    }
    Ok(())
}

fn validate_tree_origins(tree: &CandidateTree) -> Result<(), CandidateValidationError> {
    match tree {
        CandidateTree::Document { children, .. } => {
            for child in children {
                validate_tree_origins(child)?;
            }
        }
        CandidateTree::Element {
            name,
            namespace,
            origin,
            children,
            ..
        } => {
            if *namespace != CandidateNamespace::Html {
                return Err(CandidateValidationError::Provenance);
            }
            if name.is_selected() && !matches!(origin, CandidateOrigin::Authored { .. }) {
                return Err(CandidateValidationError::Provenance);
            }
            for child in children {
                validate_tree_origins(child)?;
            }
        }
        CandidateTree::Text { .. } => {}
    }
    Ok(())
}

fn selected_name(
    identities: &[(CandidateNodeId, Option<CandidateElementName>)],
    identity: CandidateNodeId,
) -> Result<CandidateElementName, CandidateValidationError> {
    identities
        .iter()
        .find_map(|(candidate, name)| (*candidate == identity).then_some(*name).flatten())
        .filter(|name| name.is_selected())
        .ok_or(CandidateValidationError::IdentityLifecycle)
}

fn validate_exact_selected_end_trigger(
    run: &HtmlTokenizerRunResult,
    trigger: &CandidateTrigger,
    expected_name: CandidateElementName,
) -> Result<(), CandidateValidationError> {
    let CandidateTrigger::Authored {
        index,
        evidence: trigger_evidence,
    } = trigger
    else {
        return Err(CandidateValidationError::Closure);
    };
    let Some(HtmlToken::Tag(tag)) = run.tokens().get(*index) else {
        return Err(CandidateValidationError::Closure);
    };
    if tag.kind() != HtmlTagKind::End
        || candidate_element_name(tag.name().interpreted()) != Some(expected_name)
        || *trigger_evidence != evidence(tag.complete())
    {
        return Err(CandidateValidationError::Closure);
    }
    Ok(())
}

fn trigger_is_authored_within(trigger: &CandidateTrigger, committed_end: usize) -> bool {
    match trigger {
        CandidateTrigger::Authored { evidence, .. } => evidence.range.1 <= committed_end,
        CandidateTrigger::EndOfFile { .. } => true,
    }
}

fn validate_candidate_observation(
    run: &HtmlTokenizerRunResult,
    observation: &CandidateObservation,
) -> Result<(), CandidateValidationError> {
    let semantic = &observation.semantic;
    let mut identities = Vec::new();
    collect_identity_names(&semantic.tree, &mut identities)?;
    validate_tree_origins(&semantic.tree)?;

    let mut sorted_ids: Vec<CandidateNodeId> =
        identities.iter().map(|(identity, _)| *identity).collect();
    sorted_ids.sort();
    if sorted_ids
        != (0..semantic.identity_count)
            .map(CandidateNodeId)
            .collect::<Vec<_>>()
        || semantic.identity_count != node_count(&semantic.tree)
    {
        return Err(CandidateValidationError::IdentityLifecycle);
    }

    let mut closure_targets = Vec::new();
    for closure in &semantic.closures {
        let name = selected_name(&identities, closure.target_identity)?;
        if closure_targets.contains(&closure.target_identity) {
            return Err(CandidateValidationError::Closure);
        }
        closure_targets.push(closure.target_identity);
        validate_exact_selected_end_trigger(run, &closure.exact_same_name_end_trigger, name)?;
    }

    for recovery in &semantic.recovery {
        if let CandidateRecovery::PoppedBySelectedAncestorEndTag {
            popped_identity,
            target_identity,
            exact_end_trigger,
        } = recovery
        {
            let _ = selected_name(&identities, *popped_identity)?;
            let target_name = selected_name(&identities, *target_identity)?;
            if popped_identity == target_identity {
                return Err(CandidateValidationError::Recovery);
            }
            validate_exact_selected_end_trigger(run, exact_end_trigger, target_name)
                .map_err(|_| CandidateValidationError::Recovery)?;
        }
    }

    let mut previous_stack = Vec::new();
    let mut previous_identity_count = 1_usize;
    let mut previous_processed = 0_usize;
    let mut previous_committed = 0_usize;
    let mut accounted_closures = 0_usize;
    let mut accounted_pops = 0_usize;
    let mut accounted_ignored = 0_usize;
    let mut accounted_unmatched = 0_usize;
    let mut accounted_misnested = 0_usize;
    let mut accounted_open_eof = 0_usize;
    let mut accounted_stopped_open = 0_usize;

    for (position, record) in observation.tokens.iter().enumerate() {
        if record.index != position
            || run.tokens().get(record.index).is_none()
            || record.trigger != candidate_trigger(&run.tokens()[record.index], record.index)
            || record.open_selected_before != previous_stack
            || record.identity_count_before != previous_identity_count
            || record.identity_count_after < record.identity_count_before
            || record.processed_tokens < previous_processed
            || record.committed_prefix_end < previous_committed
        {
            return Err(CandidateValidationError::TokenRecord);
        }
        for entry in record
            .open_selected_before
            .iter()
            .chain(&record.open_selected_after)
        {
            if selected_name(&identities, entry.identity)? != entry.name {
                return Err(CandidateValidationError::IdentityLifecycle);
            }
        }

        if record.refusal().is_some() {
            if record.open_selected_after != record.open_selected_before
                || record.identity_count_after != record.identity_count_before
                || record.processed_tokens != previous_processed
                || record.committed_prefix_end != previous_committed
                || semantic
                    .closures
                    .iter()
                    .any(|closure| closure.exact_same_name_end_trigger == record.trigger)
                || semantic
                    .recovery
                    .iter()
                    .any(|recovery| recovery.trigger() == &record.trigger)
                || semantic
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.trigger == record.trigger)
            {
                return Err(CandidateValidationError::StackTransition);
            }
        } else if record.processed_tokens != previous_processed + 1 {
            return Err(CandidateValidationError::Coverage);
        }

        match &run.tokens()[record.index] {
            HtmlToken::Tag(tag)
                if tag.kind() == HtmlTagKind::End
                    && tag.attributes().is_empty()
                    && tag.self_closing_solidus().is_none()
                    && candidate_element_name(tag.name().interpreted())
                        .is_some_and(CandidateElementName::is_selected)
                    && record.refusal().is_none() =>
            {
                let end_name = candidate_element_name(tag.name().interpreted())
                    .expect("guarded selected name");
                let target_position = record
                    .open_selected_before
                    .iter()
                    .rposition(|entry| entry.name == end_name);
                let closures: Vec<&CandidateClosure> = semantic
                    .closures
                    .iter()
                    .filter(|closure| closure.exact_same_name_end_trigger == record.trigger)
                    .collect();
                let pops: Vec<&CandidateRecovery> = semantic
                    .recovery
                    .iter()
                    .filter(|recovery| {
                        matches!(
                            recovery,
                            CandidateRecovery::PoppedBySelectedAncestorEndTag {
                                exact_end_trigger,
                                ..
                            } if *exact_end_trigger == record.trigger
                        )
                    })
                    .collect();
                let ignored = semantic
                    .recovery
                    .iter()
                    .filter(|recovery| {
                        matches!(
                            recovery,
                            CandidateRecovery::IgnoredToken { trigger }
                                if *trigger == record.trigger
                        )
                    })
                    .count();
                let unmatched = semantic
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.code == CandidateDiagnosticCode::UnmatchedSelectedEndTag
                            && diagnostic.trigger == record.trigger
                    })
                    .count();
                let misnested = semantic
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.code == CandidateDiagnosticCode::MisnestedSelectedEndTag
                            && diagnostic.trigger == record.trigger
                    })
                    .count();

                match target_position {
                    None => {
                        if !closures.is_empty()
                            || !pops.is_empty()
                            || ignored != 1
                            || unmatched != 1
                            || misnested != 0
                            || record.open_selected_after != record.open_selected_before
                            || record.identity_count_after != record.identity_count_before
                            || !matches!(
                                record
                                    .dispatches
                                    .last()
                                    .map(|dispatch| dispatch.disposition),
                                Some(CandidateDisposition::Ignored)
                            )
                        {
                            return Err(CandidateValidationError::StackTransition);
                        }
                    }
                    Some(target_position) => {
                        let target = record.open_selected_before[target_position];
                        let expected_popped: Vec<CandidateNodeId> = record.open_selected_before
                            [target_position + 1..]
                            .iter()
                            .rev()
                            .map(|entry| entry.identity)
                            .collect();
                        let actual_popped: Vec<CandidateNodeId> = pops
                            .iter()
                            .map(|recovery| match recovery {
                                CandidateRecovery::PoppedBySelectedAncestorEndTag {
                                    popped_identity,
                                    target_identity,
                                    ..
                                } => {
                                    if *target_identity != target.identity {
                                        return CandidateNodeId(usize::MAX);
                                    }
                                    *popped_identity
                                }
                                _ => unreachable!(),
                            })
                            .collect();
                        if closures.len() != 1
                            || closures[0].target_identity != target.identity
                            || actual_popped != expected_popped
                            || ignored != 0
                            || unmatched != 0
                            || misnested != usize::from(!expected_popped.is_empty())
                            || record.open_selected_after.as_slice()
                                != &record.open_selected_before[..target_position]
                            || record.identity_count_after != record.identity_count_before
                            || !matches!(
                                record
                                    .dispatches
                                    .last()
                                    .map(|dispatch| dispatch.disposition),
                                Some(CandidateDisposition::Consumed)
                            )
                        {
                            return Err(CandidateValidationError::StackTransition);
                        }
                        validate_exact_selected_end_trigger(
                            run,
                            &closures[0].exact_same_name_end_trigger,
                            target.name,
                        )?;
                    }
                }

                accounted_closures += closures.len();
                accounted_pops += pops.len();
                accounted_ignored += ignored;
                accounted_unmatched += unmatched;
                accounted_misnested += misnested;
            }
            HtmlToken::EndOfFile(_) if record.refusal().is_none() => {
                let open_diagnostics = semantic
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| {
                        diagnostic.code == CandidateDiagnosticCode::OpenSelectedElementsAtEndOfFile
                            && diagnostic.trigger == record.trigger
                    })
                    .count();
                let stopped_open = semantic
                    .recovery
                    .iter()
                    .filter(|recovery| {
                        matches!(
                            recovery,
                            CandidateRecovery::StoppedParsingWithOpenSelectedElements { trigger }
                                if *trigger == record.trigger
                        )
                    })
                    .count();
                if open_diagnostics != usize::from(!record.open_selected_before.is_empty())
                    || stopped_open != usize::from(!record.open_selected_before.is_empty())
                    || record.open_selected_after != record.open_selected_before
                    || record.identity_count_after != record.identity_count_before
                {
                    return Err(CandidateValidationError::Diagnostic);
                }
                accounted_open_eof += open_diagnostics;
                accounted_stopped_open += stopped_open;
            }
            _ => {}
        }

        previous_stack = record.open_selected_after.clone();
        previous_identity_count = record.identity_count_after;
        previous_processed = record.processed_tokens;
        previous_committed = record.committed_prefix_end;
    }

    if accounted_closures != semantic.closures.len()
        || accounted_pops
            != semantic
                .recovery
                .iter()
                .filter(|recovery| {
                    matches!(
                        recovery,
                        CandidateRecovery::PoppedBySelectedAncestorEndTag { .. }
                    )
                })
                .count()
        || accounted_ignored
            != semantic
                .recovery
                .iter()
                .filter(|recovery| matches!(recovery, CandidateRecovery::IgnoredToken { .. }))
                .count()
        || accounted_unmatched
            != semantic
                .diagnostics
                .iter()
                .filter(|diagnostic| {
                    diagnostic.code == CandidateDiagnosticCode::UnmatchedSelectedEndTag
                })
                .count()
        || accounted_misnested
            != semantic
                .diagnostics
                .iter()
                .filter(|diagnostic| {
                    diagnostic.code == CandidateDiagnosticCode::MisnestedSelectedEndTag
                })
                .count()
        || accounted_open_eof
            != semantic
                .diagnostics
                .iter()
                .filter(|diagnostic| {
                    diagnostic.code == CandidateDiagnosticCode::OpenSelectedElementsAtEndOfFile
                })
                .count()
        || accounted_stopped_open
            != semantic
                .recovery
                .iter()
                .filter(|recovery| {
                    matches!(
                        recovery,
                        CandidateRecovery::StoppedParsingWithOpenSelectedElements { .. }
                    )
                })
                .count()
    {
        return Err(CandidateValidationError::Recovery);
    }

    if semantic
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == CandidateDiagnosticCode::MissingDoctype)
        .count()
        != 1
        || semantic
            .recovery
            .iter()
            .filter(|recovery| {
                matches!(
                    recovery,
                    CandidateRecovery::ContinuedInQuirksDocumentMode { .. }
                )
            })
            .count()
            != 1
    {
        return Err(CandidateValidationError::Diagnostic);
    }

    if semantic.checkpoint.open_selected != previous_stack
        || semantic.identity_count != previous_identity_count
        || semantic.checkpoint.processed_tokens != previous_processed
        || semantic.checkpoint.committed_prefix_end != previous_committed
        || semantic.checkpoint.mode
            != observation
                .tokens
                .last()
                .map_or(CandidateMode::Initial, |record| record.mode_after)
    {
        return Err(CandidateValidationError::Coverage);
    }

    match &semantic.checkpoint.completion {
        CandidateCompletion::Complete => {
            if run.is_incomplete()
                || semantic.checkpoint.processed_tokens != run.tokens().len()
                || !observation
                    .tokens
                    .last()
                    .is_some_and(CandidateTokenRecord::stopped)
            {
                return Err(CandidateValidationError::Completion);
            }
        }
        CandidateCompletion::IncompleteUnsupported {
            capability,
            trigger,
        } => {
            let Some(last) = observation.tokens.last() else {
                return Err(CandidateValidationError::Completion);
            };
            if last.refusal() != Some(*capability) || &last.trigger != trigger {
                return Err(CandidateValidationError::Completion);
            }
        }
        CandidateCompletion::IncompleteLowerLayer => {
            if !run.is_incomplete() && semantic.checkpoint.processed_tokens == run.tokens().len() {
                return Err(CandidateValidationError::Completion);
            }
        }
    }

    let committed_end = semantic.checkpoint.committed_prefix_end;
    let mut semantic_evidence = Vec::new();
    collect_tree_evidence(&semantic.tree, &mut semantic_evidence);
    if semantic_evidence
        .iter()
        .any(|evidence| evidence.range.1 > committed_end)
        || semantic
            .diagnostics
            .iter()
            .any(|diagnostic| !trigger_is_authored_within(&diagnostic.trigger, committed_end))
        || semantic
            .recovery
            .iter()
            .any(|recovery| !trigger_is_authored_within(recovery.trigger(), committed_end))
        || semantic.closures.iter().any(|closure| {
            !trigger_is_authored_within(&closure.exact_same_name_end_trigger, committed_end)
        })
    {
        return Err(CandidateValidationError::Coverage);
    }

    Ok(())
}

fn normalize_evidence(evidence: &mut CandidateEvidence) {
    evidence.source_id = SourceId::new(1);
}

fn normalize_trigger(trigger: &mut CandidateTrigger) {
    if let CandidateTrigger::Authored { evidence, .. } = trigger {
        normalize_evidence(evidence);
    }
}

fn normalize_tree_source_ids(tree: &mut CandidateTree) {
    match tree {
        CandidateTree::Document { children, .. } => {
            for child in children {
                normalize_tree_source_ids(child);
            }
        }
        CandidateTree::Element {
            origin, children, ..
        } => {
            if let CandidateOrigin::Authored { complete, raw_name } = origin {
                normalize_evidence(complete);
                normalize_evidence(raw_name);
            }
            for child in children {
                normalize_tree_source_ids(child);
            }
        }
        CandidateTree::Text { contributions, .. } => {
            for contribution in contributions {
                normalize_evidence(contribution);
            }
        }
    }
}

fn normalize_semantic_source_ids(
    mut semantic: CandidateSemanticObservation,
) -> CandidateSemanticObservation {
    normalize_tree_source_ids(&mut semantic.tree);
    for diagnostic in &mut semantic.diagnostics {
        normalize_trigger(&mut diagnostic.trigger);
    }
    for recovery in &mut semantic.recovery {
        match recovery {
            CandidateRecovery::ContinuedInQuirksDocumentMode { trigger }
            | CandidateRecovery::IgnoredToken { trigger }
            | CandidateRecovery::StoppedParsingWithOpenSelectedElements { trigger } => {
                normalize_trigger(trigger);
            }
            CandidateRecovery::PoppedBySelectedAncestorEndTag {
                exact_end_trigger, ..
            } => normalize_trigger(exact_end_trigger),
        }
    }
    for closure in &mut semantic.closures {
        normalize_trigger(&mut closure.exact_same_name_end_trigger);
    }
    if let CandidateCompletion::IncompleteUnsupported { trigger, .. } =
        &mut semantic.checkpoint.completion
    {
        normalize_trigger(trigger);
    }
    semantic
}

fn append_duplicate_selected_node(tree: &mut CandidateTree) -> bool {
    match tree {
        CandidateTree::Document { children, .. } | CandidateTree::Element { children, .. } => {
            if let Some(selected) = children
                .iter()
                .find(|child| {
                    matches!(
                        child,
                        CandidateTree::Element { name, .. } if name.is_selected()
                    )
                })
                .cloned()
            {
                children.push(selected);
                return true;
            }
            children.iter_mut().any(append_duplicate_selected_node)
        }
        CandidateTree::Text { .. } => false,
    }
}

fn generated_candidate_sources() -> Vec<String> {
    const PIECES: [&str; 5] = ["<div>", "<section>", "</div>", "</section>", "x"];
    // Test-enumeration bound only; it is not candidate or runtime policy.
    const MAX_SEQUENCE_LENGTH: u32 = 4;

    let mut sources = Vec::new();
    for length in 0..=MAX_SEQUENCE_LENGTH {
        for code in 0..PIECES.len().pow(length) {
            let mut remaining = code;
            let mut digits = Vec::new();
            for _ in 0..length {
                digits.push(remaining % PIECES.len());
                remaining /= PIECES.len();
            }
            digits.reverse();
            let mut source = String::from("<body>");
            for digit in digits {
                source.push_str(PIECES[digit]);
            }
            sources.push(source);
        }
    }
    for suffix in [
        "<section><div><div></section><div></div>",
        "<div><section><section></div><section></section>",
        "<div><section><div><section></div></section>",
        "x<section>y<div>z</section>x",
    ] {
        sources.push(format!("<body>{suffix}"));
    }
    sources
}

#[test]
fn canonical_fixture_bytes_match_issue_361_authority() {
    assert_eq!(CANDIDATE_FIXTURES.len(), CANDIDATE_IDS.len());
    for (fixture, id) in CANDIDATE_FIXTURES.iter().zip(CANDIDATE_IDS) {
        assert_eq!(fixture.id, id);
        assert_eq!(fixture.bytes.len(), fixture.length, "{id}");
        assert_eq!(fixture.source_text().as_bytes(), fixture.bytes, "{id}");
        assert_eq!(fixture.sha256.len(), 64, "{id}");
        assert!(
            fixture
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
            "{id}"
        );
        for ((start, end), expected_bytes) in fixture.required_ranges {
            assert!(*start <= *end && *end <= fixture.length, "{id}");
            assert_eq!(
                &fixture.bytes[*start..*end],
                *expected_bytes,
                "{id} [{start},{end})"
            );
        }
    }
}

#[test]
fn pinned_authority_and_candidate_independence_are_explicit() {
    assert_eq!(
        PINNED_WHATWG_COMMIT,
        "508a037333d8a1806504303aeb489d931fabbef6"
    );
    assert_eq!(
        PINNED_WHATWG_SOURCE_BLOB,
        "68dbcb98bbe1001c6ae2531be2368c608fbafddd"
    );
    let source = include_str!("in_body_div_section_successor_validation.rs");
    let forbidden = [
        ["use super::", "driver"].concat(),
        ["use super::", "session"].concat(),
        ["use super::", "result"].concat(),
        ["construct_html_document_", "shell("].concat(),
    ];
    for pattern in forbidden {
        assert!(!source.contains(pattern.as_str()), "{pattern}");
    }
}

#[test]
fn tokenizer_emits_exact_evidence_shape_for_hs1_through_hs21() {
    let expected = exact_expected_tokens();
    assert_eq!(expected.len(), 21);
    for (id, expected_tokens) in expected {
        let run = run_for(id, 41);
        let observed: Vec<ExpectedToken> = run.tokens().iter().map(observed_token).collect();
        assert_eq!(observed, expected_tokens, "{id}");
        assert_eq!(run.is_incomplete(), id == "HS17", "{id}");
        for token in run.tokens() {
            match token {
                HtmlToken::Character(character) => {
                    assert_eq!(character.source().source_id(), SourceId::new(41));
                }
                HtmlToken::Tag(tag) => {
                    assert_eq!(tag.complete().source_id(), SourceId::new(41));
                    assert_eq!(tag.name().source().source_id(), SourceId::new(41));
                    for attribute in tag.attributes() {
                        assert_eq!(attribute.complete().source_id(), SourceId::new(41));
                    }
                }
                HtmlToken::EndOfFile(_) => {}
            }
        }
    }
}

#[test]
fn hs1_through_hs21_match_separately_hand_authored_gold() {
    for id in CANDIDATE_IDS {
        let run = run_for(id, 1);
        let observed = observe(&run);
        assert_eq!(observed.semantic, candidate_gold(id, 1), "{id}");
        assert_eq!(
            validate_candidate_observation(&run, &observed),
            Ok(()),
            "{id}"
        );
    }
}

#[test]
fn candidate_configuration_and_closed_branch_projections_are_exact() {
    assert_eq!(
        CandidateConfiguration::FIXED,
        CandidateConfiguration {
            scripting: CandidateScripting::Disabled,
            fragment_parse: false,
            template_state: false,
            table_state: false,
            foreign_content_state: false,
            reentrant_state: false,
        }
    );
    for name in [
        CandidateElementName::Html,
        CandidateElementName::Head,
        CandidateElementName::Body,
        CandidateElementName::Div,
        CandidateElementName::Section,
    ] {
        assert!(!name.is_implied_end_element());
        assert_eq!(name.is_scope_boundary(), name == CandidateElementName::Html);
        assert_eq!(
            name.is_selected(),
            matches!(
                name,
                CandidateElementName::Div | CandidateElementName::Section
            )
        );
    }
    assert_eq!(
        candidate_element_name("div"),
        Some(CandidateElementName::Div)
    );
    assert_eq!(
        candidate_element_name("section"),
        Some(CandidateElementName::Section)
    );
    for name in ["p", "span", "DIV", "SECTION", "article", "divx"] {
        assert_eq!(candidate_element_name(name), None);
    }
    assert_eq!(
        observe_fixture("HS19", 1).semantic.checkpoint.mode,
        CandidateMode::AfterBody
    );
}

#[test]
fn html_whitespace_projection_is_exact() {
    for code in 0_u32..=0xff {
        let character = char::from_u32(code).expect("ASCII-range scalar");
        assert_eq!(
            is_html_whitespace(character),
            matches!(code, 0x09 | 0x0a | 0x0c | 0x0d | 0x20)
        );
    }
}

#[test]
fn selected_starts_create_one_authored_identity_under_current_in_one_dispatch() {
    for id in CANDIDATE_IDS {
        let run = run_for(id, 1);
        let observed = observe(&run);
        for record in &observed.tokens {
            let Some(HtmlToken::Tag(tag)) = run.tokens().get(record.index) else {
                continue;
            };
            let Some(name) = candidate_element_name(tag.name().interpreted()) else {
                continue;
            };
            if tag.kind() == HtmlTagKind::Start && name.is_selected() && record.refusal().is_none()
            {
                assert_eq!(record.mode_before, CandidateMode::InBody);
                assert_eq!(record.mode_after, CandidateMode::InBody);
                assert_eq!(
                    record.dispatches,
                    vec![CandidateDispatch {
                        evaluated_in: CandidateMode::InBody,
                        disposition: CandidateDisposition::Consumed,
                    }]
                );
                assert_eq!(
                    record.open_selected_after.len(),
                    record.open_selected_before.len() + 1
                );
                assert_eq!(
                    record.identity_count_after,
                    record.identity_count_before + 1
                );
                let created = record.open_selected_after.last().expect("created selected");
                assert_eq!(created.name, name);
            }
        }
    }

    let session = CandidateSession::new(CandidateStorageLayout::COMPACT);
    assert!(!session.has_p_in_button_scope());
}

#[test]
fn current_target_and_heterogeneous_recovery_relations_are_exact() {
    for id in ["HS1", "HS3", "HS4", "HS5", "HS6", "HS7", "HS8", "HS20"] {
        let run = run_for(id, 1);
        let observed = observe(&run);
        assert_eq!(
            validate_candidate_observation(&run, &observed),
            Ok(()),
            "{id}"
        );
        assert_eq!(observed.semantic.closures, candidate_gold(id, 1).closures);
        assert_eq!(observed.semantic.recovery, candidate_gold(id, 1).recovery);
    }

    let hs6 = observe_fixture("HS6", 1);
    assert_eq!(hs6.semantic.recovery[1], recovery_pop(1, 5, 4, 3, (20, 30)));
    assert_eq!(hs6.semantic.closures, vec![closure(1, 4, 3, (20, 30))]);
    assert!(
        hs6.semantic
            .closures
            .iter()
            .all(|closure| closure.target_identity != node_id(5))
    );

    let hs7 = observe_fixture("HS7", 1);
    assert_eq!(hs7.semantic.recovery[1], recovery_pop(1, 5, 4, 3, (20, 26)));

    let multiple = tokenize_text("<body><section><div><div></section>", 1, generous_limits());
    let observed = observe(&multiple);
    let pops: Vec<CandidateNodeId> = observed
        .semantic
        .recovery
        .iter()
        .filter_map(|recovery| match recovery {
            CandidateRecovery::PoppedBySelectedAncestorEndTag {
                popped_identity, ..
            } => Some(*popped_identity),
            _ => None,
        })
        .collect();
    assert_eq!(pops, vec![node_id(6), node_id(5)]);
    assert_eq!(validate_candidate_observation(&multiple, &observed), Ok(()));
}

#[test]
fn nearest_same_name_target_is_selected_by_reverse_semantic_stack_scan() {
    let source = "<body><div><section><div><section></div>";
    let run = tokenize_text(source, 1, generous_limits());
    let observed = observe(&run);
    let closure = observed.semantic.closures.last().expect("div closure");
    assert_eq!(closure.target_identity, node_id(6));
    let popped: Vec<CandidateNodeId> = observed
        .semantic
        .recovery
        .iter()
        .filter_map(|recovery| match recovery {
            CandidateRecovery::PoppedBySelectedAncestorEndTag {
                popped_identity,
                target_identity,
                ..
            } if *target_identity == node_id(6) => Some(*popped_identity),
            _ => None,
        })
        .collect();
    assert_eq!(popped, vec![node_id(7)]);
    assert_eq!(validate_candidate_observation(&run, &observed), Ok(()));
}

#[test]
fn unmatched_selected_end_is_diagnosed_ignored_and_mutation_free() {
    let observed = observe_fixture("HS9", 1);
    assert_eq!(observed.semantic.closures, vec![]);
    assert_eq!(
        observed.tokens[1].dispatches,
        vec![CandidateDispatch {
            evaluated_in: CandidateMode::InBody,
            disposition: CandidateDisposition::Ignored,
        }]
    );
    assert_eq!(
        observed.tokens[1].open_selected_before,
        observed.tokens[1].open_selected_after
    );
    assert_eq!(
        observed.semantic.diagnostics[1],
        diagnostic(
            CandidateDiagnosticCode::UnmatchedSelectedEndTag,
            authored_trigger(1, 1, (6, 16)),
        )
    );
    assert_eq!(observed.semantic.recovery[1], ignored_token(1, 1, (6, 16)));
    assert_eq!(observed.semantic.identity_count, 4);
}

#[test]
fn text_parentage_contributions_and_append_identity_remain_exact() {
    let run = tokenize_text("<body><section>a</div>b", 1, generous_limits());
    let observed = observe(&run);
    let mut texts = Vec::new();
    text_nodes(&observed.semantic.tree, &mut texts);
    assert_eq!(texts, vec![("ab".to_owned(), vec![(15, 16), (22, 23)])]);
    assert_eq!(
        observed.semantic.identity_count,
        node_count(&observed.semantic.tree)
    );
    assert_eq!(
        observed.tokens[2].identity_count_after,
        observed.tokens[2].identity_count_before + 1
    );
    assert_eq!(
        observed.tokens[4].identity_count_after,
        observed.tokens[4].identity_count_before
    );
    assert_eq!(validate_candidate_observation(&run, &observed), Ok(()));

    let nested = tokenize_text(
        "<body><section>a<div>b</div>c</section>",
        1,
        generous_limits(),
    );
    let observed = observe(&nested);
    let mut texts = Vec::new();
    text_nodes(&observed.semantic.tree, &mut texts);
    assert_eq!(
        texts,
        vec![
            ("a".to_owned(), vec![(15, 16)]),
            ("b".to_owned(), vec![(21, 22)]),
            ("c".to_owned(), vec![(28, 29)]),
        ]
    );
    assert_eq!(validate_candidate_observation(&nested, &observed), Ok(()));
}

#[test]
fn eof_open_selected_is_complete_without_fabricated_closure_or_pop() {
    let observed = observe_fixture("HS10", 1);
    assert_eq!(observed.semantic.closures, vec![]);
    assert_eq!(
        observed.semantic.checkpoint.completion,
        CandidateCompletion::Complete
    );
    assert_eq!(
        observed.semantic.checkpoint.open_selected,
        vec![selected_entry(4, CandidateElementName::Section)]
    );
    assert_eq!(
        observed.semantic.diagnostics[1],
        diagnostic(
            CandidateDiagnosticCode::OpenSelectedElementsAtEndOfFile,
            CandidateTrigger::EndOfFile { index: 3 },
        )
    );
    assert_eq!(observed.semantic.recovery[1], stopped_with_open_selected(3));
}

fn assert_refusal_fingerprint(id: &str, expected_capability: CandidateUnsupported) {
    let run = run_for(id, 1);
    let observed = observe(&run);
    let refused = observed.tokens.last().expect("refused token record");
    assert_eq!(refused.refusal(), Some(expected_capability), "{id}");
    assert_eq!(
        refused.open_selected_before, refused.open_selected_after,
        "{id}"
    );

    let mut session = CandidateSession::new(CandidateStorageLayout::COMPACT);
    for (index, token) in run.tokens().iter().enumerate() {
        let trigger = candidate_trigger(token, index);
        if index + 1 == observed.tokens.len() {
            let before = session.fingerprint();
            match candidate_shape(token) {
                Err(capability) => assert_eq!(capability, expected_capability, "{id}"),
                Ok(shape) => {
                    let record = session.process(index, shape, trigger);
                    assert_eq!(record.refusal(), Some(expected_capability), "{id}");
                }
            }
            assert_eq!(session.fingerprint(), before, "{id}");
            return;
        }
        let shape = candidate_shape(token).expect("supported prefix token");
        let record = session.process(index, shape, trigger);
        assert!(record.refusal().is_none(), "{id}");
    }
    panic!("missing refused token for {id}");
}

#[test]
fn unsupported_boundaries_are_full_state_transactional() {
    for (id, capability, committed, processed) in [
        (
            "HS11",
            CandidateUnsupported::SelectedStartTagAttribute,
            6,
            1,
        ),
        (
            "HS12",
            CandidateUnsupported::SelectedSelfClosingStartTag,
            6,
            1,
        ),
        (
            "HS13",
            CandidateUnsupported::SelectedTagOutsideInBody,
            13,
            2,
        ),
        (
            "HS14",
            CandidateUnsupported::BodyCloseWithOpenSelectedElements,
            15,
            2,
        ),
        ("HS15", CandidateUnsupported::PElement, 6, 1),
        ("HS16", CandidateUnsupported::GenericOrdinaryElement, 6, 1),
        ("HS21", CandidateUnsupported::SelectedEndTagAttribute, 15, 2),
    ] {
        assert_refusal_fingerprint(id, capability);
        let observed = observe_fixture(id, 1);
        assert_eq!(
            observed.semantic.checkpoint.committed_prefix_end, committed,
            "{id}"
        );
        assert_eq!(
            observed.semantic.checkpoint.processed_tokens, processed,
            "{id}"
        );
        assert_eq!(observed.semantic, candidate_gold(id, 1), "{id}");
    }

    let hs21 = observe_fixture("HS21", 1);
    assert_eq!(
        hs21.semantic.checkpoint.open_selected,
        vec![selected_entry(4, CandidateElementName::Section)]
    );
    assert!(hs21.semantic.closures.is_empty());
    assert!(hs21.semantic.recovery.iter().all(|recovery| !matches!(
        recovery,
        CandidateRecovery::PoppedBySelectedAncestorEndTag { .. }
    )));
}

#[test]
fn lower_layer_incompleteness_is_monotonic_and_diagnostics_are_orthogonal() {
    let hs17 = observe_fixture("HS17", 1);
    assert_eq!(
        hs17.semantic.checkpoint.completion,
        CandidateCompletion::IncompleteLowerLayer
    );
    assert_eq!(hs17.semantic.checkpoint.committed_prefix_end, 15);
    assert!(hs17.semantic.closures.is_empty());
    assert_eq!(
        hs17.semantic.checkpoint.open_selected,
        vec![selected_entry(4, CandidateElementName::Section)]
    );

    for limits in [
        HtmlTokenizerLimits::new(4, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
        HtmlTokenizerLimits::new(1_024, 3, 1_024, 1_024, 256, 4_096, 1_024),
        HtmlTokenizerLimits::new(1_024, 0, 1_024, 1_024, 256, 4_096, 1_024),
    ] {
        let run = tokenize_text(fixture("HS1").source_text(), 1, limits);
        assert!(run.is_incomplete());
        assert_ne!(
            observe(&run).semantic.checkpoint.completion,
            CandidateCompletion::Complete
        );
    }

    for id in ["HS6", "HS9", "HS10"] {
        assert_eq!(
            observe_fixture(id, 1).semantic.checkpoint.completion,
            CandidateCompletion::Complete,
            "{id}"
        );
    }
}

#[test]
fn authored_provenance_source_identity_and_relation_domains_stay_distinct() {
    for id in CANDIDATE_IDS {
        let observed = observe_fixture(id, 73);
        let mut authored = Vec::new();
        collect_tree_evidence(&observed.semantic.tree, &mut authored);
        for diagnostic in &observed.semantic.diagnostics {
            if let CandidateTrigger::Authored { evidence, .. } = &diagnostic.trigger {
                authored.push(evidence.clone());
            }
        }
        for recovery in &observed.semantic.recovery {
            if let CandidateTrigger::Authored { evidence, .. } = recovery.trigger() {
                authored.push(evidence.clone());
            }
        }
        for closure in &observed.semantic.closures {
            if let CandidateTrigger::Authored { evidence, .. } =
                &closure.exact_same_name_end_trigger
            {
                authored.push(evidence.clone());
            }
        }
        assert!(
            authored
                .iter()
                .all(|evidence| evidence.source_id == SourceId::new(73)),
            "{id}"
        );
    }

    let hs2 = fixture("HS2");
    assert_eq!(&hs2.bytes[7..14], b"SeCtIoN");
    assert_eq!(&hs2.bytes[18..25], b"sEcTiOn");

    let hs6 = observe_fixture("HS6", 1);
    let mut tree_evidence = Vec::new();
    collect_tree_evidence(&hs6.semantic.tree, &mut tree_evidence);
    assert!(tree_evidence.iter().any(|item| item.range == (15, 20)));
    assert!(!tree_evidence.iter().any(|item| item.range == (20, 30)));
    assert!(
        hs6.semantic
            .closures
            .iter()
            .all(|closure| closure.target_identity != node_id(5))
    );
}

#[test]
fn semantic_ids_survive_multiple_storage_perturbations() {
    let layouts = [
        CandidateStorageLayout {
            leading_padding: 1,
            padding_before_each_node: 0,
            padding_after_even_identity: 0,
        },
        CandidateStorageLayout {
            leading_padding: 2,
            padding_before_each_node: 2,
            padding_after_even_identity: 0,
        },
        CandidateStorageLayout {
            leading_padding: 3,
            padding_before_each_node: 1,
            padding_after_even_identity: 2,
        },
    ];
    let run = run_for("HS8", 1);
    let baseline = observe_with_layout(&run, CandidateStorageLayout::COMPACT);
    for layout in layouts {
        let perturbed = observe_with_layout(&run, layout);
        assert_eq!(perturbed.semantic, baseline.semantic);
        assert_eq!(validate_candidate_observation(&run, &perturbed), Ok(()));

        let mut compact = CandidateSession::new(CandidateStorageLayout::COMPACT);
        let mut padded = CandidateSession::new(layout);
        for (index, token) in run.tokens().iter().take(4).enumerate() {
            let trigger = candidate_trigger(token, index);
            compact.process(
                index,
                candidate_shape(token).expect("candidate shape"),
                trigger.clone(),
            );
            padded.process(
                index,
                candidate_shape(token).expect("candidate shape"),
                trigger,
            );
        }
        assert_eq!(compact.open_selected(), padded.open_selected());
        for entry in compact.open_selected() {
            assert_ne!(
                compact.storage_index(entry.identity),
                padded.storage_index(entry.identity)
            );
        }
    }
}

#[test]
fn generated_sequences_cover_repeated_alternating_recovery_and_reopening() {
    let sources = generated_candidate_sources();
    assert_eq!(sources.len(), 785);

    for source in sources {
        let run = tokenize_text(&source, 1, generous_limits());
        assert!(!run.is_incomplete(), "{source:?}");
        let observed = observe(&run);
        assert_eq!(
            observed.semantic.checkpoint.completion,
            CandidateCompletion::Complete,
            "{source:?}"
        );
        assert_eq!(
            validate_candidate_observation(&run, &observed),
            Ok(()),
            "{source:?}"
        );
        assert_eq!(
            observe(&tokenize_text(&source, 1, generous_limits())).semantic,
            observed.semantic,
            "{source:?}"
        );
    }
}

#[test]
fn selected_processing_terminates_by_finite_structure_without_runtime_budget() {
    for source in generated_candidate_sources() {
        let run = tokenize_text(&source, 1, generous_limits());
        let observed = observe(&run);
        for record in &observed.tokens {
            let modes: Vec<CandidateMode> = record
                .dispatches
                .iter()
                .map(|dispatch| dispatch.evaluated_in)
                .collect();
            for (position, mode) in modes.iter().enumerate() {
                assert!(!modes[..position].contains(mode), "{source:?}");
            }
            assert!(modes.len() <= CandidateMode::ALL.len(), "{source:?}");
            if record.mode_before == CandidateMode::InBody {
                assert_eq!(record.dispatches.len(), 1, "{source:?}");
            }
            assert!(
                record.open_selected_after.len() <= record.open_selected_before.len() + 1,
                "{source:?}"
            );
        }
    }
}

#[test]
fn source_ids_are_propagated_and_semantics_match_modulo_source_identity() {
    for id in CANDIDATE_IDS {
        let first = observe_fixture(id, 7).semantic;
        let second = observe_fixture(id, 4_242).semantic;
        assert_eq!(first, candidate_gold(id, 7), "{id}");
        assert_eq!(second, candidate_gold(id, 4_242), "{id}");
        assert_eq!(
            normalize_semantic_source_ids(first),
            normalize_semantic_source_ids(second),
            "{id}"
        );
    }
}

#[test]
fn predecessor_shell_after_body_and_tc_s3_meaning_remain_bounded() {
    let hs18 = observe_fixture("HS18", 1);
    let CandidateTree::Document { children, .. } = &hs18.semantic.tree else {
        unreachable!()
    };
    let CandidateTree::Element {
        origin: CandidateOrigin::Synthesized,
        children,
        ..
    } = &children[0]
    else {
        panic!("synthesized html has no authored anchor")
    };
    assert!(matches!(
        children[0],
        CandidateTree::Element {
            origin: CandidateOrigin::Synthesized,
            ..
        }
    ));
    assert!(matches!(
        children[1],
        CandidateTree::Element {
            origin: CandidateOrigin::Synthesized,
            ..
        }
    ));

    assert_eq!(
        observe_fixture("HS19", 1).semantic.checkpoint.mode,
        CandidateMode::AfterBody
    );
    assert_eq!(
        observe_fixture("HS20", 1).semantic,
        candidate_gold("HS20", 1)
    );
    assert_eq!(
        observe_fixture("HS13", 1).semantic.checkpoint.completion,
        candidate_gold("HS13", 1).checkpoint.completion
    );
}

#[test]
fn candidate_validator_rejects_corrupt_relationships_and_lifecycles() {
    let run = run_for("HS1", 1);
    let baseline = observe(&run);
    assert_eq!(validate_candidate_observation(&run, &baseline), Ok(()));

    let mut duplicate_node = baseline.clone();
    assert!(append_duplicate_selected_node(
        &mut duplicate_node.semantic.tree
    ));
    assert_eq!(
        validate_candidate_observation(&run, &duplicate_node),
        Err(CandidateValidationError::IdentityLifecycle)
    );

    let mut duplicate_closure = baseline.clone();
    duplicate_closure
        .semantic
        .closures
        .push(duplicate_closure.semantic.closures[0].clone());
    assert!(validate_candidate_observation(&run, &duplicate_closure).is_err());

    let mut start_as_closure = baseline.clone();
    start_as_closure.semantic.closures[0].exact_same_name_end_trigger =
        authored_trigger(1, 1, (6, 15));
    assert!(validate_candidate_observation(&run, &start_as_closure).is_err());

    let mut wrong_range = baseline.clone();
    wrong_range.semantic.closures[0].exact_same_name_end_trigger = authored_trigger(1, 2, (15, 24));
    assert!(validate_candidate_observation(&run, &wrong_range).is_err());

    let hs4_run = run_for("HS4", 1);
    let mut wrong_name = observe(&hs4_run);
    wrong_name.semantic.closures[0].exact_same_name_end_trigger = authored_trigger(1, 4, (30, 36));
    assert!(validate_candidate_observation(&hs4_run, &wrong_name).is_err());

    let hs6_run = run_for("HS6", 1);
    let hs6 = observe(&hs6_run);

    let mut unresolved = hs6.clone();
    let CandidateRecovery::PoppedBySelectedAncestorEndTag {
        popped_identity, ..
    } = &mut unresolved.semantic.recovery[1]
    else {
        unreachable!()
    };
    *popped_identity = node_id(999);
    assert!(validate_candidate_observation(&hs6_run, &unresolved).is_err());

    let mut target_as_popped = hs6.clone();
    let CandidateRecovery::PoppedBySelectedAncestorEndTag {
        popped_identity,
        target_identity,
        ..
    } = &mut target_as_popped.semantic.recovery[1]
    else {
        unreachable!()
    };
    *popped_identity = *target_identity;
    assert!(validate_candidate_observation(&hs6_run, &target_as_popped).is_err());

    let mut wrong_recovery_range = hs6.clone();
    let CandidateRecovery::PoppedBySelectedAncestorEndTag {
        exact_end_trigger, ..
    } = &mut wrong_recovery_range.semantic.recovery[1]
    else {
        unreachable!()
    };
    *exact_end_trigger = authored_trigger(1, 3, (20, 29));
    assert!(validate_candidate_observation(&hs6_run, &wrong_recovery_range).is_err());

    let multiple_run = tokenize_text("<body><section><div><div></section>", 1, generous_limits());
    let mut wrong_order = observe(&multiple_run);
    wrong_order.semantic.recovery.swap(1, 2);
    assert!(validate_candidate_observation(&multiple_run, &wrong_order).is_err());

    let hs10_run = run_for("HS10", 1);
    let mut fabricated_eof_closure = observe(&hs10_run);
    fabricated_eof_closure
        .semantic
        .closures
        .push(CandidateClosure {
            target_identity: node_id(4),
            exact_same_name_end_trigger: CandidateTrigger::EndOfFile { index: 3 },
        });
    assert!(validate_candidate_observation(&hs10_run, &fabricated_eof_closure).is_err());
}

#[test]
fn candidate_widens_nothing_beyond_selected_div_section_cells() {
    for (source, expected_capability) in [
        (" ", CandidateUnsupported::WhitespaceSensitiveCharacterData),
        (
            "\t<body>",
            CandidateUnsupported::WhitespaceSensitiveCharacterData,
        ),
        ("<body><p>", CandidateUnsupported::PElement),
        ("<body><span>", CandidateUnsupported::GenericOrdinaryElement),
        ("<body a>", CandidateUnsupported::ShellTagAttribute),
        ("<body/>", CandidateUnsupported::SelfClosingShellTag),
        (
            "<body><section id=x>",
            CandidateUnsupported::SelectedStartTagAttribute,
        ),
        (
            "<body><section/>",
            CandidateUnsupported::SelectedSelfClosingStartTag,
        ),
        (
            "<body><section></section id=x>",
            CandidateUnsupported::SelectedEndTagAttribute,
        ),
        ("<section>", CandidateUnsupported::SelectedTagOutsideInBody),
        (
            "<body></body><section>",
            CandidateUnsupported::SelectedTagOutsideInBody,
        ),
        (
            "<body><section></body>",
            CandidateUnsupported::BodyCloseWithOpenSelectedElements,
        ),
        (
            "<body><div></body>",
            CandidateUnsupported::BodyCloseWithOpenSelectedElements,
        ),
        (
            "<body><article>",
            CandidateUnsupported::GenericOrdinaryElement,
        ),
    ] {
        let run = tokenize_text(source, 1, generous_limits());
        let observed = observe(&run);
        let CandidateCompletion::IncompleteUnsupported { capability, .. } =
            observed.semantic.checkpoint.completion
        else {
            panic!("expected candidate refusal for {source:?}")
        };
        assert_eq!(capability, expected_capability, "{source:?}");
        assert_eq!(
            observed.semantic.identity_count,
            node_count(&observed.semantic.tree)
        );
    }
}
