//! Candidate-independent TC-S3 successor validation.
//!
//! TC-S3 is the proposed successor theorem "Selected In-Body No-Attribute
//! `div` Construction" (Issue #357). This module is validation only. It does
//! not choose production placement or a production ordinary-element model.
//!
//! The candidate consumes only the accepted batch tokenizer as lower-layer
//! evidence. It imports no production tree-construction semantics from
//! `driver`, `session`, or `result`. Expected meaning is independently stated
//! by a bounded candidate machine and hand-authored DV1-DV14 GOLD.
//!
//! The load-bearing normative authority is the #348 pinned WHATWG HTML source:
//! commit `508a037333d8a1806504303aeb489d931fabbef6`, blob
//! `68dbcb98bbe1001c6ae2531be2368c608fbafddd`. The candidate executes the
//! selected `in body` `div` start/end rules, the relevant scope walk, implied
//! end-tag generation, and the `in body` EOF branch over its own private state.
//!
//! Three evidence domains are intentionally explicit here:
//! - authored evidence retains caller `SourceId` plus the exact byte range;
//! - constructed relationships use test-only semantic node identities rather
//!   than arena/storage indices;
//! - matching `div` end tags are retained as closure relations from the exact
//!   semantic element identity to the exact emitted end-tag trigger.
//!
//! These types are validation artifacts only. They are not proposed production
//! representations and create no public or cross-run compatibility promise.

use crate::{SourceId, SourceText};

use super::super::token::{HtmlTagKind, HtmlTagToken, HtmlToken};
use super::super::tokenizer::producer::tokenize;
use super::super::tokenizer::resource::HtmlTokenizerLimits;
use super::super::tokenizer::result::HtmlTokenizerRunResult;

const PINNED_WHATWG_COMMIT: &str = "508a037333d8a1806504303aeb489d931fabbef6";
const PINNED_WHATWG_SOURCE_BLOB: &str = "68dbcb98bbe1001c6ae2531be2368c608fbafddd";

// ---------------------------------------------------------------------------
// Canonical DV1-DV14 byte authority
// ---------------------------------------------------------------------------

struct CandidateFixture {
    id: &'static str,
    bytes: &'static [u8],
    length: usize,
    sha256: &'static str,
    required_ranges: &'static [((usize, usize), &'static [u8])],
}

const CANDIDATE_FIXTURES: &[CandidateFixture] = &[
    CandidateFixture {
        id: "DV1",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 17,
        sha256: "44f9dbc6331c75636ef2eec39853fd6c931b1fba28272c62786ebac16cb4ba84",
        required_ranges: &[
            ((0, 6), b"<body>"), ((1, 5), b"body"), ((6, 11), b"<div>"),
            ((7, 10), b"div"), ((11, 17), b"</div>"),
        ],
    },
    CandidateFixture {
        id: "DV2",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x44\x69\x56\x3e\x78\x3c\x2f\x64\x49\x76\x3e",
        length: 18,
        sha256: "e6d7d8bdef2d9ab8d87a5c60a50b84a69b62e25ba467eef3dd7def3b02af2ea4",
        required_ranges: &[
            ((0, 6), b"<body>"), ((1, 5), b"body"), ((6, 11), b"<DiV>"),
            ((7, 10), b"DiV"), ((11, 12), b"x"), ((12, 18), b"</dIv>"),
            ((14, 17), b"dIv"),
        ],
    },
    CandidateFixture {
        id: "DV3",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x64\x69\x76\x3e\x78\x3c\x2f\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 29,
        sha256: "892f118c39733725de1fb73faace78efb3ce71f784f0a2d7b56be4184cf42e37",
        required_ranges: &[
            ((0, 6), b"<body>"), ((6, 11), b"<div>"), ((11, 16), b"<div>"),
            ((16, 17), b"x"), ((17, 23), b"</div>"), ((23, 29), b"</div>"),
        ],
    },
    CandidateFixture {
        id: "DV4",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 28,
        sha256: "0b3ed57bb102f1b262a6e8e681ed27e9ab75ae1ed6b4f519a27e7445d3a8fc81",
        required_ranges: &[
            ((0, 6), b"<body>"), ((6, 11), b"<div>"), ((11, 17), b"</div>"),
            ((17, 22), b"<div>"), ((22, 28), b"</div>"),
        ],
    },
    CandidateFixture {
        id: "DV5",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 12,
        sha256: "365a11b5e706c966789fb89d83350dbd9f26d5cfffadc53fa5fce9cfdbdd4e84",
        required_ranges: &[((0, 6), b"<body>"), ((6, 12), b"</div>")],
    },
    CandidateFixture {
        id: "DV6",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x78",
        length: 12,
        sha256: "f2552ec2bc6659b8315e7c1f2c342f2278bbc3efd613c603146382dbd209f04b",
        required_ranges: &[((0, 6), b"<body>"), ((6, 11), b"<div>"), ((11, 12), b"x")],
    },
    CandidateFixture {
        id: "DV7",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x61\x3c\x64\x69\x76\x3e\x62\x3c\x2f\x64\x69\x76\x3e\x63\x3c\x2f\x64\x69\x76\x3e",
        length: 31,
        sha256: "ddfbed7c8d6a377da9762833373c0ce1b266727fe6863e7490a9022754995b6f",
        required_ranges: &[
            ((0, 6), b"<body>"), ((6, 11), b"<div>"), ((11, 12), b"a"),
            ((12, 17), b"<div>"), ((17, 18), b"b"), ((18, 24), b"</div>"),
            ((24, 25), b"c"), ((25, 31), b"</div>"),
        ],
    },
    CandidateFixture {
        id: "DV8a",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x20\x69\x64\x3d\x78\x3e",
        length: 16,
        sha256: "2365fac0b8ea4475ec187ee0f4ecf7ef9c546e5f82c991fd57b8fc0276110496",
        required_ranges: &[((0, 6), b"<body>"), ((6, 16), b"<div id=x>"), ((11, 15), b"id=x")],
    },
    CandidateFixture {
        id: "DV8b",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x2f\x3e",
        length: 12,
        sha256: "211e3885d26f9cf03b6a15755da5738826e3b5989e2b3fab864d7f5d0dcf7620",
        required_ranges: &[((0, 6), b"<body>"), ((6, 12), b"<div/>"), ((10, 11), b"/")],
    },
    CandidateFixture {
        id: "DV9",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x2f\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e",
        length: 18,
        sha256: "5ac460c96f826b009a05235b42483980e351f4e6bc68470358bcf5afb558b173",
        required_ranges: &[((0, 6), b"<body>"), ((6, 13), b"</body>"), ((13, 18), b"<div>")],
    },
    CandidateFixture {
        id: "DV10",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x62\x6f\x64\x79\x3e",
        length: 18,
        sha256: "7ce1ef731dd9fe36fbb191fe420587c66f2bc58cb35654338209b0ade2f18b97",
        required_ranges: &[((0, 6), b"<body>"), ((6, 11), b"<div>"), ((11, 18), b"</body>")],
    },
    CandidateFixture {
        id: "DV11",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x70\x3e",
        length: 9,
        sha256: "648ccd6dff0fb3d71045933acb5ea913a0c5566f1d52abb69056a073cbfc1b8c",
        required_ranges: &[((0, 6), b"<body>"), ((6, 9), b"<p>")],
    },
    CandidateFixture {
        id: "DV12",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x26\x61\x6d\x70\x3b",
        length: 16,
        sha256: "174bb8c3b05e81890cb3a9cd0388c3c4e22aa74d25bcf0a16018af07debcb910",
        required_ranges: &[((0, 6), b"<body>"), ((6, 11), b"<div>"), ((11, 16), b"&amp;")],
    },
    CandidateFixture {
        id: "DV13",
        bytes: b"\x78\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e",
        length: 12,
        sha256: "1a155a4c794347039cc5791bc4d109e38236b5611de00ad96f5aca34470a859c",
        required_ranges: &[((0, 1), b"x"), ((1, 6), b"<div>"), ((6, 12), b"</div>")],
    },
    CandidateFixture {
        id: "DV14",
        bytes: b"\x3c\x62\x6f\x64\x79\x3e\x3c\x64\x69\x76\x3e\x3c\x2f\x64\x69\x76\x3e\x3c\x2f\x62\x6f\x64\x79\x3e",
        length: 24,
        sha256: "ec7b7eee750428e84c00289fc830b8657bdef0c7ed00bc0102ed243fb18df2cc",
        required_ranges: &[((0, 6), b"<body>"), ((6, 11), b"<div>"), ((11, 17), b"</div>"), ((17, 24), b"</body>")],
    },
];

const CANDIDATE_IDS: [&str; 15] = [
    "DV1", "DV2", "DV3", "DV4", "DV5", "DV6", "DV7", "DV8a", "DV8b", "DV9",
    "DV10", "DV11", "DV12", "DV13", "DV14",
];

fn fixture(id: &str) -> &'static CandidateFixture {
    CANDIDATE_FIXTURES.iter().find(|fixture| fixture.id == id).expect("fixture")
}

impl CandidateFixture {
    fn source_text(&self) -> &'static str {
        std::str::from_utf8(self.bytes).expect("canonical fixture is UTF-8")
    }
}

// ---------------------------------------------------------------------------
// Candidate evidence and semantic domain
// ---------------------------------------------------------------------------

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
    CandidateEvidence { source_id: SourceId::new(source_id), range }
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
        Self::Initial, Self::BeforeHtml, Self::BeforeHead, Self::InHead,
        Self::AfterHead, Self::InBody, Self::AfterBody,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateNamespace { Html }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateElementName { Html, Head, Body, Div }

impl CandidateElementName {
    fn is_shell(self) -> bool { !matches!(self, Self::Div) }
    fn is_scope_boundary(self) -> bool { matches!(self, Self::Html) }
    fn is_implied_end_element(self) -> bool { false }
    fn permitted_at_in_body_eof(self) -> bool { matches!(self, Self::Html | Self::Body) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateNodeId(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateOrigin {
    Authored { complete: CandidateEvidence, raw_name: CandidateEvidence },
    Synthesized,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTrigger {
    Authored { index: usize, evidence: CandidateEvidence },
    EndOfFile { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateUnsupported {
    TagWithAttributes,
    SelfClosingTag,
    DivTagOutsideInBody,
    ShellInteractionWithOpenDiv,
    WhitespaceSensitiveCharacterData,
    OutsideModelledCandidateCells,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDiagnosticCode {
    MissingDoctype,
    UnmatchedDivEndTag,
    OpenOrdinaryElementAtEndOfFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateRecovery {
    ContinuedInQuirksDocumentMode,
    IgnoredToken,
    StoppedParsingWithOpenElements,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateDiagnostic {
    code: CandidateDiagnosticCode,
    trigger: CandidateTrigger,
    recovery: CandidateRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateClosure {
    element: CandidateNodeId,
    trigger: CandidateTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateDisposition { Consumed, Ignored, Reprocessed, Stopped, Refused(CandidateUnsupported) }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateDispatch { evaluated_in: CandidateMode, disposition: CandidateDisposition }

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateTokenRecord {
    index: usize,
    trigger: CandidateTrigger,
    mode_before: CandidateMode,
    mode_after: CandidateMode,
    dispatches: Vec<CandidateDispatch>,
    open_div_depth_after: usize,
    committed_prefix_end: usize,
}

impl CandidateTokenRecord {
    fn refusal(&self) -> Option<CandidateUnsupported> {
        match self.dispatches.last()?.disposition {
            CandidateDisposition::Refused(capability) => Some(capability),
            _ => None,
        }
    }
    fn stopped(&self) -> bool {
        matches!(self.dispatches.last().map(|d| d.disposition), Some(CandidateDisposition::Stopped))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTree {
    Document { id: CandidateNodeId, children: Vec<CandidateTree> },
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
    IncompleteUnsupported { capability: CandidateUnsupported, trigger: CandidateTrigger },
    IncompleteLowerLayer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateCheckpoint {
    mode: CandidateMode,
    open_div_depth: usize,
    committed_prefix_end: usize,
    completion: CandidateCompletion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateSemanticObservation {
    tree: CandidateTree,
    diagnostics: Vec<CandidateDiagnostic>,
    closures: Vec<CandidateClosure>,
    identity_count: usize,
    checkpoint: CandidateCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateObservation {
    semantic: CandidateSemanticObservation,
    tokens: Vec<CandidateTokenRecord>,
}

// ---------------------------------------------------------------------------
// Candidate token normalization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateTokenShape<'a> {
    Characters { source: CandidateEvidence, interpreted: &'a str },
    StartTag {
        name: CandidateElementName,
        complete: CandidateEvidence,
        raw_name: CandidateEvidence,
    },
    EndTag {
        name: CandidateElementName,
        complete: CandidateEvidence,
        raw_name: CandidateEvidence,
    },
    EndOfFile { at: usize },
}

impl CandidateTokenShape<'_> {
    fn committed_end(&self) -> usize {
        match self {
            Self::Characters { source, .. } => source.range.1,
            Self::StartTag { complete, .. } | Self::EndTag { complete, .. } => complete.range.1,
            Self::EndOfFile { at } => *at,
        }
    }
    fn is_div_tag(&self) -> bool {
        matches!(self,
            Self::StartTag { name: CandidateElementName::Div, .. }
            | Self::EndTag { name: CandidateElementName::Div, .. })
    }
    fn shell_tag_name(&self) -> Option<CandidateElementName> {
        match self {
            Self::StartTag { name, .. } | Self::EndTag { name, .. } if name.is_shell() => Some(*name),
            _ => None,
        }
    }
}

fn candidate_element_name(name: &str) -> Option<CandidateElementName> {
    match name {
        "html" => Some(CandidateElementName::Html),
        "head" => Some(CandidateElementName::Head),
        "body" => Some(CandidateElementName::Body),
        "div" => Some(CandidateElementName::Div),
        _ => None,
    }
}

fn candidate_trigger(token: &HtmlToken, index: usize) -> CandidateTrigger {
    match token {
        HtmlToken::Character(character) => CandidateTrigger::Authored { index, evidence: evidence(character.source()) },
        HtmlToken::Tag(tag) => CandidateTrigger::Authored { index, evidence: evidence(tag.complete()) },
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
            let Some(name) = candidate_element_name(tag.name().interpreted()) else {
                return Err(CandidateUnsupported::OutsideModelledCandidateCells);
            };
            if !tag.attributes().is_empty() { return Err(CandidateUnsupported::TagWithAttributes); }
            if tag.self_closing_solidus().is_some() { return Err(CandidateUnsupported::SelfClosingTag); }
            match tag.kind() {
                HtmlTagKind::Start => Ok(CandidateTokenShape::StartTag {
                    name,
                    complete: evidence(tag.complete()),
                    raw_name: evidence(tag.name().source()),
                }),
                HtmlTagKind::End => Ok(CandidateTokenShape::EndTag {
                    name,
                    complete: evidence(tag.complete()),
                    raw_name: evidence(tag.name().source()),
                }),
            }
        }
        HtmlToken::EndOfFile(end) => Ok(CandidateTokenShape::EndOfFile { at: end.source().range().start() }),
    }
}

fn is_html_whitespace(c: char) -> bool { matches!(c, '\t' | '\n' | '\u{000c}' | '\r' | ' ') }

// ---------------------------------------------------------------------------
// Candidate rule table
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateEffect {
    MissingDoctype,
    InsertShell(CandidateElementName, bool),
    CloseHead,
    InsertCharacters,
    InsertDiv,
    PopDiv,
    UnmatchedDiv,
    OpenElementAtEof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CandidateStep {
    Consume { effect: Option<CandidateEffect>, next: Option<CandidateMode> },
    Ignore { effect: CandidateEffect },
    Reprocess { effect: Option<CandidateEffect>, next: CandidateMode },
    Stop { effect: Option<CandidateEffect> },
}

fn reject_whitespace_sensitive(shape: &CandidateTokenShape<'_>) -> Result<(), CandidateUnsupported> {
    if let CandidateTokenShape::Characters { interpreted, .. } = shape {
        if interpreted.chars().any(is_html_whitespace) {
            return Err(CandidateUnsupported::WhitespaceSensitiveCharacterData);
        }
    }
    Ok(())
}

fn expect_shell_walk_trigger(shape: &CandidateTokenShape<'_>) -> Result<(), CandidateUnsupported> {
    if shape.is_div_tag() { return Err(CandidateUnsupported::DivTagOutsideInBody); }
    match shape {
        CandidateTokenShape::StartTag { name: CandidateElementName::Body, .. }
        | CandidateTokenShape::Characters { .. } => Ok(()),
        _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
    }
}

fn select(
    mode: CandidateMode,
    open_div_depth: usize,
    shape: &CandidateTokenShape<'_>,
) -> Result<CandidateStep, CandidateUnsupported> {
    match mode {
        CandidateMode::Initial => {
            reject_whitespace_sensitive(shape)?; expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::MissingDoctype), next: CandidateMode::BeforeHtml })
        }
        CandidateMode::BeforeHtml => {
            reject_whitespace_sensitive(shape)?; expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::InsertShell(CandidateElementName::Html, false)), next: CandidateMode::BeforeHead })
        }
        CandidateMode::BeforeHead => {
            reject_whitespace_sensitive(shape)?; expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::InsertShell(CandidateElementName::Head, false)), next: CandidateMode::InHead })
        }
        CandidateMode::InHead => {
            reject_whitespace_sensitive(shape)?; expect_shell_walk_trigger(shape)?;
            Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::CloseHead), next: CandidateMode::AfterHead })
        }
        CandidateMode::AfterHead => {
            reject_whitespace_sensitive(shape)?; expect_shell_walk_trigger(shape)?;
            match shape {
                CandidateTokenShape::StartTag { name: CandidateElementName::Body, .. } =>
                    Ok(CandidateStep::Consume { effect: Some(CandidateEffect::InsertShell(CandidateElementName::Body, true)), next: Some(CandidateMode::InBody) }),
                _ => Ok(CandidateStep::Reprocess { effect: Some(CandidateEffect::InsertShell(CandidateElementName::Body, false)), next: CandidateMode::InBody }),
            }
        }
        CandidateMode::InBody => {
            if shape.shell_tag_name().is_some() && open_div_depth > 0 {
                return Err(CandidateUnsupported::ShellInteractionWithOpenDiv);
            }
            match shape {
                CandidateTokenShape::Characters { .. } => Ok(CandidateStep::Consume { effect: Some(CandidateEffect::InsertCharacters), next: None }),
                CandidateTokenShape::StartTag { name: CandidateElementName::Div, .. } => Ok(CandidateStep::Consume { effect: Some(CandidateEffect::InsertDiv), next: None }),
                CandidateTokenShape::EndTag { name: CandidateElementName::Div, .. } if open_div_depth > 0 => Ok(CandidateStep::Consume { effect: Some(CandidateEffect::PopDiv), next: None }),
                CandidateTokenShape::EndTag { name: CandidateElementName::Div, .. } => Ok(CandidateStep::Ignore { effect: CandidateEffect::UnmatchedDiv }),
                CandidateTokenShape::EndTag { name: CandidateElementName::Body, .. } => Ok(CandidateStep::Consume { effect: None, next: Some(CandidateMode::AfterBody) }),
                CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop { effect: (open_div_depth > 0).then_some(CandidateEffect::OpenElementAtEof) }),
                _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
            }
        }
        CandidateMode::AfterBody => match shape {
            CandidateTokenShape::EndOfFile { .. } => Ok(CandidateStep::Stop { effect: None }),
            _ if shape.is_div_tag() => Err(CandidateUnsupported::DivTagOutsideInBody),
            _ => Err(CandidateUnsupported::OutsideModelledCandidateCells),
        },
    }
}

// ---------------------------------------------------------------------------
// Independent candidate machine: semantic identities, storage-neutral relations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum CandidateArenaKind {
    Document,
    Element { name: CandidateElementName, namespace: CandidateNamespace, origin: CandidateOrigin },
    Text { interpreted: String, contributions: Vec<CandidateEvidence> },
}

#[derive(Debug, Clone)]
struct CandidateArenaNode {
    id: CandidateNodeId,
    children: Vec<CandidateNodeId>,
    kind: CandidateArenaKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateStorageLayout { padding_before_each_node: usize }
impl CandidateStorageLayout { const COMPACT: Self = Self { padding_before_each_node: 0 }; }

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateStateFingerprint {
    tree: CandidateTree,
    open_elements: Vec<CandidateNodeId>,
    mode: CandidateMode,
    diagnostics: Vec<CandidateDiagnostic>,
    closures: Vec<CandidateClosure>,
    identity_count: usize,
    committed_prefix_end: usize,
}

struct CandidateSession {
    layout: CandidateStorageLayout,
    slots: Vec<Option<CandidateArenaNode>>,
    document: CandidateNodeId,
    open_elements: Vec<CandidateNodeId>,
    head: Option<CandidateNodeId>,
    mode: CandidateMode,
    diagnostics: Vec<CandidateDiagnostic>,
    closures: Vec<CandidateClosure>,
    identity_count: usize,
    committed_prefix_end: usize,
    processed_tokens: usize,
}

impl CandidateSession {
    fn new(layout: CandidateStorageLayout) -> Self {
        let mut session = Self {
            layout,
            slots: Vec::new(),
            document: CandidateNodeId(0),
            open_elements: Vec::new(),
            head: None,
            mode: CandidateMode::Initial,
            diagnostics: Vec::new(),
            closures: Vec::new(),
            identity_count: 0,
            committed_prefix_end: 0,
            processed_tokens: 0,
        };
        session.document = session.allocate(CandidateArenaKind::Document);
        session
    }

    fn allocate(&mut self, kind: CandidateArenaKind) -> CandidateNodeId {
        for _ in 0..self.layout.padding_before_each_node { self.slots.push(None); }
        let id = CandidateNodeId(self.identity_count);
        self.identity_count += 1;
        self.slots.push(Some(CandidateArenaNode { id, children: Vec::new(), kind }));
        id
    }

    fn node(&self, id: CandidateNodeId) -> &CandidateArenaNode {
        self.slots.iter().flatten().find(|node| node.id == id).expect("semantic node identity")
    }

    fn node_mut(&mut self, id: CandidateNodeId) -> &mut CandidateArenaNode {
        self.slots.iter_mut().flatten().find(|node| node.id == id).expect("semantic node identity")
    }

    fn storage_index(&self, id: CandidateNodeId) -> usize {
        self.slots.iter().position(|slot| slot.as_ref().is_some_and(|node| node.id == id)).expect("stored identity")
    }

    fn element_name(&self, id: CandidateNodeId) -> CandidateElementName {
        match &self.node(id).kind {
            CandidateArenaKind::Element { name, .. } => *name,
            _ => panic!("open-elements entry must be an element"),
        }
    }

    fn current_node(&self) -> Option<CandidateNodeId> { self.open_elements.last().copied() }

    fn open_div_depth(&self) -> usize {
        self.open_elements.iter().filter(|id| self.element_name(**id) == CandidateElementName::Div).count()
    }

    fn has_element_in_scope(&self, target: CandidateElementName) -> bool {
        for id in self.open_elements.iter().rev() {
            let name = self.element_name(*id);
            if name == target { return true; }
            if name.is_scope_boundary() { return false; }
        }
        false
    }

    fn has_p_in_button_scope(&self) -> bool {
        for id in self.open_elements.iter().rev() {
            if self.element_name(*id).is_scope_boundary() { return false; }
        }
        false
    }

    fn generate_implied_end_tags(&mut self) -> usize {
        let mut popped = 0;
        while let Some(id) = self.current_node() {
            if !self.element_name(id).is_implied_end_element() { break; }
            self.open_elements.pop();
            popped += 1;
        }
        popped
    }

    fn assert_invariant(&self) {
        let names: Vec<CandidateElementName> = self.open_elements.iter().map(|id| self.element_name(*id)).collect();
        for name in &names {
            assert!(!name.is_implied_end_element());
            assert!(*name == CandidateElementName::Html || !name.is_scope_boundary());
        }
        let valid = match names.as_slice() {
            [] | [CandidateElementName::Html] => true,
            [CandidateElementName::Html, CandidateElementName::Head] => true,
            [CandidateElementName::Html, CandidateElementName::Body, rest @ ..] => rest.iter().all(|name| *name == CandidateElementName::Div),
            _ => false,
        };
        assert!(valid, "candidate stack invariant: {names:?}");
    }

    fn fingerprint(&self) -> CandidateStateFingerprint {
        CandidateStateFingerprint {
            tree: self.tree(),
            open_elements: self.open_elements.clone(),
            mode: self.mode,
            diagnostics: self.diagnostics.clone(),
            closures: self.closures.clone(),
            identity_count: self.identity_count,
            committed_prefix_end: self.committed_prefix_end,
        }
    }

    fn process(&mut self, index: usize, shape: CandidateTokenShape<'_>, trigger: CandidateTrigger) -> CandidateTokenRecord {
        let mode_before = self.mode;
        let mut dispatches = Vec::new();
        let mut visited = Vec::new();
        loop {
            self.assert_invariant();
            assert!(!visited.contains(&self.mode), "same token revisited mode {:?}", self.mode);
            visited.push(self.mode);
            let before = self.fingerprint();
            let evaluated_in = self.mode;
            match select(self.mode, self.open_div_depth(), &shape) {
                Err(capability) => {
                    assert_eq!(self.fingerprint(), before, "refusal mutates nothing");
                    dispatches.push(CandidateDispatch { evaluated_in, disposition: CandidateDisposition::Refused(capability) });
                    break;
                }
                Ok(CandidateStep::Stop { effect }) => {
                    if let Some(effect) = effect { self.apply(effect, &trigger, &shape); }
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch { evaluated_in, disposition: CandidateDisposition::Stopped });
                    break;
                }
                Ok(CandidateStep::Consume { effect, next }) => {
                    if let Some(effect) = effect { self.apply(effect, &trigger, &shape); }
                    if let Some(next) = next { self.mode = next; }
                    self.commit(&shape);
                    dispatches.push(CandidateDispatch { evaluated_in, disposition: CandidateDisposition::Consumed });
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
                    dispatches.push(CandidateDispatch { evaluated_in, disposition: CandidateDisposition::Ignored });
                    break;
                }
                Ok(CandidateStep::Reprocess { effect, next }) => {
                    if let Some(effect) = effect { self.apply(effect, &trigger, &shape); }
                    self.mode = next;
                    dispatches.push(CandidateDispatch { evaluated_in, disposition: CandidateDisposition::Reprocessed });
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
            open_div_depth_after: self.open_div_depth(),
            committed_prefix_end: self.committed_prefix_end,
        }
    }

    fn apply(&mut self, effect: CandidateEffect, trigger: &CandidateTrigger, shape: &CandidateTokenShape<'_>) {
        match effect {
            CandidateEffect::MissingDoctype => self.diagnostics.push(CandidateDiagnostic {
                code: CandidateDiagnosticCode::MissingDoctype,
                trigger: trigger.clone(),
                recovery: CandidateRecovery::ContinuedInQuirksDocumentMode,
            }),
            CandidateEffect::InsertShell(name, authored) => self.insert_shell(name, authored, shape),
            CandidateEffect::CloseHead => {
                let head = self.head.expect("head");
                assert_eq!(self.open_elements.last(), Some(&head));
                self.open_elements.pop();
            }
            CandidateEffect::InsertCharacters => self.insert_characters(shape),
            CandidateEffect::InsertDiv => self.insert_div(shape),
            CandidateEffect::PopDiv => self.pop_div(trigger),
            CandidateEffect::UnmatchedDiv => {
                assert!(!self.has_element_in_scope(CandidateElementName::Div));
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::UnmatchedDivEndTag,
                    trigger: trigger.clone(),
                    recovery: CandidateRecovery::IgnoredToken,
                });
            }
            CandidateEffect::OpenElementAtEof => {
                assert!(self.open_elements.iter().map(|id| self.element_name(*id)).any(|name| !name.permitted_at_in_body_eof()));
                assert!(matches!(trigger, CandidateTrigger::EndOfFile { .. }));
                self.diagnostics.push(CandidateDiagnostic {
                    code: CandidateDiagnosticCode::OpenOrdinaryElementAtEndOfFile,
                    trigger: trigger.clone(),
                    recovery: CandidateRecovery::StoppedParsingWithOpenElements,
                });
            }
        }
    }

    fn insert_shell(&mut self, name: CandidateElementName, authored: bool, shape: &CandidateTokenShape<'_>) {
        let parent = if name == CandidateElementName::Html { self.document } else { self.current_node().expect("parent") };
        let origin = if authored {
            let CandidateTokenShape::StartTag { complete, raw_name, .. } = shape else { panic!("authored shell requires start tag") };
            CandidateOrigin::Authored { complete: complete.clone(), raw_name: raw_name.clone() }
        } else { CandidateOrigin::Synthesized };
        let id = self.allocate(CandidateArenaKind::Element { name, namespace: CandidateNamespace::Html, origin });
        self.node_mut(parent).children.push(id);
        self.open_elements.push(id);
        if name == CandidateElementName::Head { self.head = Some(id); }
    }

    fn insert_div(&mut self, shape: &CandidateTokenShape<'_>) {
        assert!(!self.has_p_in_button_scope());
        let CandidateTokenShape::StartTag { name: CandidateElementName::Div, complete, raw_name } = shape else { panic!("div start") };
        let parent = self.current_node().expect("div parent");
        let id = self.allocate(CandidateArenaKind::Element {
            name: CandidateElementName::Div,
            namespace: CandidateNamespace::Html,
            origin: CandidateOrigin::Authored { complete: complete.clone(), raw_name: raw_name.clone() },
        });
        self.node_mut(parent).children.push(id);
        self.open_elements.push(id);
    }

    fn pop_div(&mut self, trigger: &CandidateTrigger) {
        assert!(self.has_element_in_scope(CandidateElementName::Div));
        assert_eq!(self.generate_implied_end_tags(), 0);
        let current = self.current_node().expect("current div");
        assert_eq!(self.element_name(current), CandidateElementName::Div);
        self.open_elements.pop();
        self.closures.push(CandidateClosure { element: current, trigger: trigger.clone() });
    }

    fn insert_characters(&mut self, shape: &CandidateTokenShape<'_>) {
        let CandidateTokenShape::Characters { source, interpreted } = shape else { panic!("characters") };
        let parent = self.current_node().expect("text parent");
        let adjacent = self.node(parent).children.last().copied().filter(|id| matches!(self.node(*id).kind, CandidateArenaKind::Text { .. }));
        if let Some(id) = adjacent {
            let CandidateArenaKind::Text { interpreted: existing, contributions } = &mut self.node_mut(id).kind else { unreachable!() };
            existing.push_str(interpreted);
            contributions.push(source.clone());
            return;
        }
        let id = self.allocate(CandidateArenaKind::Text { interpreted: (*interpreted).to_owned(), contributions: vec![source.clone()] });
        self.node_mut(parent).children.push(id);
    }

    fn commit(&mut self, shape: &CandidateTokenShape<'_>) {
        let end = shape.committed_end();
        assert!(end >= self.committed_prefix_end);
        self.committed_prefix_end = end;
        self.processed_tokens += 1;
    }

    fn tree(&self) -> CandidateTree { self.project(self.document) }

    fn project(&self, id: CandidateNodeId) -> CandidateTree {
        let node = self.node(id);
        let children = node.children.iter().map(|child| self.project(*child)).collect();
        match &node.kind {
            CandidateArenaKind::Document => CandidateTree::Document { id, children },
            CandidateArenaKind::Element { name, namespace, origin } => CandidateTree::Element {
                id, name: *name, namespace: *namespace, origin: origin.clone(), children,
            },
            CandidateArenaKind::Text { interpreted, contributions } => CandidateTree::Text {
                id, interpreted: interpreted.clone(), contributions: contributions.clone(),
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Candidate execution
// ---------------------------------------------------------------------------

fn generous_limits() -> HtmlTokenizerLimits {
    HtmlTokenizerLimits::new(1_024, 8_192, 1_024, 1_024, 256, 4_096, 1_024)
}

fn tokenize_text(text: &str, source_id: u64, limits: HtmlTokenizerLimits) -> HtmlTokenizerRunResult {
    tokenize(&SourceText::new(SourceId::new(source_id), text.to_owned()), limits)
}

fn run_for(id: &str, source_id: u64) -> HtmlTokenizerRunResult {
    tokenize_text(fixture(id).source_text(), source_id, generous_limits())
}

fn observe_with_layout(run: &HtmlTokenizerRunResult, layout: CandidateStorageLayout) -> CandidateObservation {
    let mut session = CandidateSession::new(layout);
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
                    trigger: trigger.clone(),
                    mode_before: session.mode,
                    mode_after: session.mode,
                    dispatches: vec![CandidateDispatch { evaluated_in: session.mode, disposition: CandidateDisposition::Refused(capability) }],
                    open_div_depth_after: session.open_div_depth(),
                    committed_prefix_end: session.committed_prefix_end,
                });
                refusal = Some((capability, trigger));
                break;
            }
        };
        let record = session.process(index, shape, trigger.clone());
        if let Some(capability) = record.refusal() { refusal = Some((capability, trigger)); }
        let is_refused = record.refusal().is_some();
        let is_stopped = record.stopped();
        tokens.push(record);
        if is_refused { break; }
        if is_stopped { stopped = true; break; }
    }

    let completion = match refusal {
        Some((capability, trigger)) => CandidateCompletion::IncompleteUnsupported { capability, trigger },
        None if stopped && session.processed_tokens == run.tokens().len() && !run.is_incomplete() => CandidateCompletion::Complete,
        None => CandidateCompletion::IncompleteLowerLayer,
    };
    CandidateObservation {
        semantic: CandidateSemanticObservation {
            tree: session.tree(),
            diagnostics: session.diagnostics,
            closures: session.closures,
            identity_count: session.identity_count,
            checkpoint: CandidateCheckpoint {
                mode: session.mode,
                open_div_depth: session.open_div_depth(),
                committed_prefix_end: session.committed_prefix_end,
                completion,
            },
        },
        tokens,
    }
}

fn observe(run: &HtmlTokenizerRunResult) -> CandidateObservation {
    observe_with_layout(run, CandidateStorageLayout::COMPACT)
}

fn observe_fixture(id: &str, source_id: u64) -> CandidateObservation { observe(&run_for(id, source_id)) }

// ---------------------------------------------------------------------------
// Hand-authored semantic GOLD
// ---------------------------------------------------------------------------

fn id(value: usize) -> CandidateNodeId { CandidateNodeId(value) }
fn ev(source_id: u64, range: (usize, usize)) -> CandidateEvidence { expected_evidence(source_id, range) }
fn origin(source_id: u64, complete: (usize, usize), raw_name: (usize, usize)) -> CandidateOrigin {
    CandidateOrigin::Authored { complete: ev(source_id, complete), raw_name: ev(source_id, raw_name) }
}
fn element(
    node_id: usize,
    name: CandidateElementName,
    origin: CandidateOrigin,
    children: Vec<CandidateTree>,
) -> CandidateTree {
    CandidateTree::Element { id: id(node_id), name, namespace: CandidateNamespace::Html, origin, children }
}
fn text(node_id: usize, source_id: u64, interpreted: &str, contributions: &[(usize, usize)]) -> CandidateTree {
    CandidateTree::Text {
        id: id(node_id),
        interpreted: interpreted.to_owned(),
        contributions: contributions.iter().map(|range| ev(source_id, *range)).collect(),
    }
}
fn shell(source_id: u64, authored_body: bool, body_children: Vec<CandidateTree>) -> CandidateTree {
    CandidateTree::Document {
        id: id(0),
        children: vec![element(
            1,
            CandidateElementName::Html,
            CandidateOrigin::Synthesized,
            vec![
                element(2, CandidateElementName::Head, CandidateOrigin::Synthesized, vec![]),
                element(
                    3,
                    CandidateElementName::Body,
                    if authored_body { origin(source_id, (0, 6), (1, 5)) } else { CandidateOrigin::Synthesized },
                    body_children,
                ),
            ],
        )],
    }
}
fn authored_trigger(source_id: u64, index: usize, range: (usize, usize)) -> CandidateTrigger {
    CandidateTrigger::Authored { index, evidence: ev(source_id, range) }
}
fn missing_doctype(source_id: u64, index: usize, range: (usize, usize)) -> CandidateDiagnostic {
    CandidateDiagnostic { code: CandidateDiagnosticCode::MissingDoctype, trigger: authored_trigger(source_id, index, range), recovery: CandidateRecovery::ContinuedInQuirksDocumentMode }
}
fn closure(source_id: u64, element_id: usize, index: usize, range: (usize, usize)) -> CandidateClosure {
    CandidateClosure { element: id(element_id), trigger: authored_trigger(source_id, index, range) }
}
fn checkpoint(mode: CandidateMode, depth: usize, committed: usize, completion: CandidateCompletion) -> CandidateCheckpoint {
    CandidateCheckpoint { mode, open_div_depth: depth, committed_prefix_end: committed, completion }
}

fn candidate_gold(id_value: &str, source_id: u64) -> CandidateSemanticObservation {
    let missing_body = || missing_doctype(source_id, 0, (0, 6));
    let complete = CandidateCompletion::Complete;
    match id_value {
        "DV1" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![])]),
            diagnostics: vec![missing_body()],
            closures: vec![closure(source_id, 4, 2, (11, 17))],
            identity_count: 5,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 17, complete),
        },
        "DV2" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![text(5, source_id, "x", &[(11, 12)])])]),
            diagnostics: vec![missing_body()],
            closures: vec![closure(source_id, 4, 3, (12, 18))],
            identity_count: 6,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 18, complete),
        },
        "DV3" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![
                element(5, CandidateElementName::Div, origin(source_id, (11, 16), (12, 15)), vec![text(6, source_id, "x", &[(16, 17)])]),
            ])]),
            diagnostics: vec![missing_body()],
            closures: vec![closure(source_id, 5, 4, (17, 23)), closure(source_id, 4, 5, (23, 29))],
            identity_count: 7,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 29, complete),
        },
        "DV4" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![
                element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![]),
                element(5, CandidateElementName::Div, origin(source_id, (17, 22), (18, 21)), vec![]),
            ]),
            diagnostics: vec![missing_body()],
            closures: vec![closure(source_id, 4, 2, (11, 17)), closure(source_id, 5, 4, (22, 28))],
            identity_count: 6,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 28, complete),
        },
        "DV5" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![]),
            diagnostics: vec![
                missing_body(),
                CandidateDiagnostic { code: CandidateDiagnosticCode::UnmatchedDivEndTag, trigger: authored_trigger(source_id, 1, (6, 12)), recovery: CandidateRecovery::IgnoredToken },
            ],
            closures: vec![], identity_count: 4,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 12, complete),
        },
        "DV6" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![text(5, source_id, "x", &[(11, 12)])])]),
            diagnostics: vec![
                missing_body(),
                CandidateDiagnostic { code: CandidateDiagnosticCode::OpenOrdinaryElementAtEndOfFile, trigger: CandidateTrigger::EndOfFile { index: 3 }, recovery: CandidateRecovery::StoppedParsingWithOpenElements },
            ],
            closures: vec![], identity_count: 6,
            checkpoint: checkpoint(CandidateMode::InBody, 1, 12, complete),
        },
        "DV7" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![
                text(5, source_id, "a", &[(11, 12)]),
                element(6, CandidateElementName::Div, origin(source_id, (12, 17), (13, 16)), vec![text(7, source_id, "b", &[(17, 18)])]),
                text(8, source_id, "c", &[(24, 25)]),
            ])]),
            diagnostics: vec![missing_body()],
            closures: vec![closure(source_id, 6, 5, (18, 24)), closure(source_id, 4, 7, (25, 31))],
            identity_count: 9,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 31, complete),
        },
        "DV8a" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![]), diagnostics: vec![missing_body()], closures: vec![], identity_count: 4,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 6, CandidateCompletion::IncompleteUnsupported { capability: CandidateUnsupported::TagWithAttributes, trigger: authored_trigger(source_id, 1, (6, 16)) }),
        },
        "DV8b" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![]), diagnostics: vec![missing_body()], closures: vec![], identity_count: 4,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 6, CandidateCompletion::IncompleteUnsupported { capability: CandidateUnsupported::SelfClosingTag, trigger: authored_trigger(source_id, 1, (6, 12)) }),
        },
        "DV9" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![]), diagnostics: vec![missing_body()], closures: vec![], identity_count: 4,
            checkpoint: checkpoint(CandidateMode::AfterBody, 0, 13, CandidateCompletion::IncompleteUnsupported { capability: CandidateUnsupported::DivTagOutsideInBody, trigger: authored_trigger(source_id, 2, (13, 18)) }),
        },
        "DV10" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![])]),
            diagnostics: vec![missing_body()], closures: vec![], identity_count: 5,
            checkpoint: checkpoint(CandidateMode::InBody, 1, 11, CandidateCompletion::IncompleteUnsupported { capability: CandidateUnsupported::ShellInteractionWithOpenDiv, trigger: authored_trigger(source_id, 2, (11, 18)) }),
        },
        "DV11" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![]), diagnostics: vec![missing_body()], closures: vec![], identity_count: 4,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 6, CandidateCompletion::IncompleteUnsupported { capability: CandidateUnsupported::OutsideModelledCandidateCells, trigger: authored_trigger(source_id, 1, (6, 9)) }),
        },
        "DV12" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![])]),
            diagnostics: vec![missing_body()], closures: vec![], identity_count: 5,
            checkpoint: checkpoint(CandidateMode::InBody, 1, 11, CandidateCompletion::IncompleteLowerLayer),
        },
        "DV13" => CandidateSemanticObservation {
            tree: shell(source_id, false, vec![
                text(4, source_id, "x", &[(0, 1)]),
                element(5, CandidateElementName::Div, origin(source_id, (1, 6), (2, 5)), vec![]),
            ]),
            diagnostics: vec![missing_doctype(source_id, 0, (0, 1))],
            closures: vec![closure(source_id, 5, 2, (6, 12))],
            identity_count: 6,
            checkpoint: checkpoint(CandidateMode::InBody, 0, 12, complete),
        },
        "DV14" => CandidateSemanticObservation {
            tree: shell(source_id, true, vec![element(4, CandidateElementName::Div, origin(source_id, (6, 11), (7, 10)), vec![])]),
            diagnostics: vec![missing_body()], closures: vec![closure(source_id, 4, 2, (11, 17))], identity_count: 5,
            checkpoint: checkpoint(CandidateMode::AfterBody, 0, 24, complete),
        },
        other => panic!("no GOLD for {other}"),
    }
}

// ---------------------------------------------------------------------------
// Observation helpers
// ---------------------------------------------------------------------------

fn node_count(tree: &CandidateTree) -> usize {
    match tree {
        CandidateTree::Document { children, .. } | CandidateTree::Element { children, .. } => 1 + children.iter().map(node_count).sum::<usize>(),
        CandidateTree::Text { .. } => 1,
    }
}
fn collect_ids(tree: &CandidateTree, ids: &mut Vec<CandidateNodeId>) {
    match tree {
        CandidateTree::Document { id, children } | CandidateTree::Element { id, children, .. } => {
            ids.push(*id); for child in children { collect_ids(child, ids); }
        }
        CandidateTree::Text { id, .. } => ids.push(*id),
    }
}
fn collect_evidence(tree: &CandidateTree, into: &mut Vec<CandidateEvidence>) {
    match tree {
        CandidateTree::Document { children, .. } => for child in children { collect_evidence(child, into); },
        CandidateTree::Element { origin, children, .. } => {
            if let CandidateOrigin::Authored { complete, raw_name } = origin { into.push(complete.clone()); into.push(raw_name.clone()); }
            for child in children { collect_evidence(child, into); }
        }
        CandidateTree::Text { contributions, .. } => into.extend(contributions.iter().cloned()),
    }
}
fn text_nodes(tree: &CandidateTree, into: &mut Vec<(String, Vec<(usize, usize)>)>) {
    match tree {
        CandidateTree::Document { children, .. } | CandidateTree::Element { children, .. } => for child in children { text_nodes(child, into); },
        CandidateTree::Text { interpreted, contributions, .. } => into.push((interpreted.clone(), contributions.iter().map(|e| e.range).collect())),
    }
}
fn depth_before(observation: &CandidateObservation, position: usize) -> usize {
    if position == 0 { 0 } else { observation.tokens[position - 1].open_div_depth_after }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn canonical_fixture_bytes_match_issue_357_authority() {
    assert_eq!(CANDIDATE_FIXTURES.len(), CANDIDATE_IDS.len());
    for (fixture, id) in CANDIDATE_FIXTURES.iter().zip(CANDIDATE_IDS) {
        assert_eq!(fixture.id, id);
        assert_eq!(fixture.bytes.len(), fixture.length, "{id}");
        assert_eq!(fixture.source_text().as_bytes(), fixture.bytes, "{id}");
        assert_eq!(fixture.sha256.len(), 64, "{id}");
        assert!(fixture.sha256.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()), "{id}");
        for ((start, end), expected) in fixture.required_ranges {
            assert!(*start <= *end && *end <= fixture.length, "{id}");
            assert_eq!(&fixture.bytes[*start..*end], *expected, "{id} [{start},{end})");
        }
    }
}

#[test]
fn pinned_authority_and_candidate_independence_are_explicit() {
    assert_eq!(PINNED_WHATWG_COMMIT, "508a037333d8a1806504303aeb489d931fabbef6");
    assert_eq!(PINNED_WHATWG_SOURCE_BLOB, "68dbcb98bbe1001c6ae2531be2368c608fbafddd");
    let source = include_str!("in_body_div_successor_validation.rs");
    assert!(!source.contains("use super::driver"));
    assert!(!source.contains("use super::session"));
    assert!(!source.contains("use super::result"));
    assert!(!source.contains("construct_html_document_shell("));
}

#[test]
fn tokenizer_emits_candidate_evidence_with_exact_identity_and_ranges() {
    for id_value in CANDIDATE_IDS {
        let run = run_for(id_value, 41);
        assert_eq!(run.is_incomplete(), id_value == "DV12", "{id_value}");
        for (index, token) in run.tokens().iter().enumerate() {
            let trigger = candidate_trigger(token, index);
            if let CandidateTrigger::Authored { evidence, .. } = trigger {
                assert_eq!(evidence.source_id, SourceId::new(41), "{id_value}");
            }
            match token {
                HtmlToken::Character(character) => assert_eq!(character.source().source_id(), SourceId::new(41)),
                HtmlToken::Tag(tag) => {
                    assert_eq!(tag.complete().source_id(), SourceId::new(41));
                    assert_eq!(tag.name().source().source_id(), SourceId::new(41));
                }
                HtmlToken::EndOfFile(_) => {}
            }
        }
    }
}

#[test]
fn dv_cases_match_hand_authored_semantic_gold() {
    for id_value in CANDIDATE_IDS {
        assert_eq!(observe_fixture(id_value, 1).semantic, candidate_gold(id_value, 1), "{id_value}");
    }
}

#[test]
fn candidate_invariants_and_pinned_branch_projections_are_exact() {
    for name in [CandidateElementName::Html, CandidateElementName::Head, CandidateElementName::Body, CandidateElementName::Div] {
        assert!(!name.is_implied_end_element());
        assert_eq!(name.is_scope_boundary(), name == CandidateElementName::Html);
        assert_eq!(name.permitted_at_in_body_eof(), matches!(name, CandidateElementName::Html | CandidateElementName::Body));
    }
    assert_eq!(candidate_element_name("div"), Some(CandidateElementName::Div));
    for name in ["p", "span", "section", "DIV", "divx"] { assert_eq!(candidate_element_name(name), None); }
    for code in 0u32..=0xff {
        let c = char::from_u32(code).expect("scalar");
        assert_eq!(is_html_whitespace(c), matches!(code, 0x09 | 0x0a | 0x0c | 0x0d | 0x20));
    }
}

#[test]
fn accepted_div_starts_move_s_k_to_s_k_plus_one_in_one_dispatch() {
    for id_value in CANDIDATE_IDS {
        let run = run_for(id_value, 1);
        let observed = observe(&run);
        for (position, record) in observed.tokens.iter().enumerate() {
            let Some(HtmlToken::Tag(tag)) = run.tokens().get(record.index) else { continue; };
            if tag.kind() == HtmlTagKind::Start && tag.name().interpreted() == "div" && record.refusal().is_none() {
                assert_eq!(record.mode_before, CandidateMode::InBody);
                assert_eq!(record.mode_after, CandidateMode::InBody);
                assert_eq!(record.dispatches, vec![CandidateDispatch { evaluated_in: CandidateMode::InBody, disposition: CandidateDisposition::Consumed }]);
                assert_eq!(record.open_div_depth_after, depth_before(&observed, position) + 1);
            }
        }
    }
}

#[test]
fn matching_div_ends_record_exact_semantic_closure_relations() {
    for id_value in ["DV1", "DV2", "DV3", "DV4", "DV7", "DV13", "DV14"] {
        let observed = observe_fixture(id_value, 1);
        let gold = candidate_gold(id_value, 1);
        assert_eq!(observed.semantic.closures, gold.closures, "{id_value}");
        for closure in &observed.semantic.closures {
            let CandidateTrigger::Authored { index, evidence } = &closure.trigger else { panic!("closure must be authored") };
            let HtmlToken::Tag(tag) = &run_for(id_value, 1).tokens()[*index] else { panic!("closure token") };
            assert_eq!(tag.kind(), HtmlTagKind::End);
            assert_eq!(tag.name().interpreted(), "div");
            assert_eq!(*evidence, evidence(tag.complete()));
            assert!(closure.element.0 < observed.semantic.identity_count);
        }
    }
}

#[test]
fn stray_div_end_is_diagnosed_ignored_and_creates_no_closure() {
    let observed = observe_fixture("DV5", 1);
    assert_eq!(observed.semantic.closures, vec![]);
    assert_eq!(observed.tokens[1].dispatches, vec![CandidateDispatch { evaluated_in: CandidateMode::InBody, disposition: CandidateDisposition::Ignored }]);
    assert_eq!(observed.semantic.identity_count, 4);
    assert_eq!(observed.semantic.diagnostics[1].trigger, authored_trigger(1, 1, (6, 12)));
}

#[test]
fn character_data_keeps_parent_sensitive_exact_contributions_and_coalescing_identity() {
    let observed = observe_fixture("DV7", 1);
    let mut texts = Vec::new();
    text_nodes(&observed.semantic.tree, &mut texts);
    assert_eq!(texts, vec![
        ("a".to_owned(), vec![(11, 12)]),
        ("b".to_owned(), vec![(17, 18)]),
        ("c".to_owned(), vec![(24, 25)]),
    ]);
    let coalesced = observe(&tokenize_text("<body>a</div>b", 1, generous_limits()));
    let mut texts = Vec::new(); text_nodes(&coalesced.semantic.tree, &mut texts);
    assert_eq!(texts, vec![("ab".to_owned(), vec![(6, 7), (13, 14)])]);
    assert_eq!(coalesced.semantic.identity_count, node_count(&coalesced.semantic.tree));
}

#[test]
fn open_div_at_eof_diagnoses_once_without_fabricated_closure() {
    let observed = observe_fixture("DV6", 1);
    assert_eq!(observed.semantic.closures, vec![]);
    assert_eq!(observed.semantic.checkpoint.completion, CandidateCompletion::Complete);
    assert_eq!(observed.semantic.checkpoint.open_div_depth, 1);
    assert_eq!(observed.semantic.diagnostics.iter().filter(|d| d.code == CandidateDiagnosticCode::OpenOrdinaryElementAtEndOfFile).count(), 1);
    assert!(matches!(observed.semantic.diagnostics[1].trigger, CandidateTrigger::EndOfFile { index: 3 }));
}

#[test]
fn dv_refusals_are_transactional_and_pin_exact_trigger_evidence() {
    for (id_value, capability, index, range, committed, depth) in [
        ("DV8a", CandidateUnsupported::TagWithAttributes, 1, (6, 16), 6, 0),
        ("DV8b", CandidateUnsupported::SelfClosingTag, 1, (6, 12), 6, 0),
        ("DV9", CandidateUnsupported::DivTagOutsideInBody, 2, (13, 18), 13, 0),
        ("DV10", CandidateUnsupported::ShellInteractionWithOpenDiv, 2, (11, 18), 11, 1),
        ("DV11", CandidateUnsupported::OutsideModelledCandidateCells, 1, (6, 9), 6, 0),
    ] {
        let observed = observe_fixture(id_value, 1);
        let last = observed.tokens.last().expect("refused record");
        assert_eq!(last.refusal(), Some(capability), "{id_value}");
        assert_eq!(last.mode_before, last.mode_after, "{id_value}");
        assert_eq!(last.committed_prefix_end, committed, "{id_value}");
        assert_eq!(last.open_div_depth_after, depth, "{id_value}");
        assert_eq!(last.trigger, authored_trigger(1, index, range), "{id_value}");
        assert_eq!(observed.semantic.identity_count, node_count(&observed.semantic.tree), "{id_value}");
        assert!(observed.semantic.closures.iter().all(|closure| closure.trigger != last.trigger), "{id_value}");
    }
}

#[test]
fn candidate_provenance_retains_source_identity_and_keeps_domains_distinct() {
    for id_value in CANDIDATE_IDS {
        let observed = observe_fixture(id_value, 73);
        let mut authored = Vec::new();
        collect_evidence(&observed.semantic.tree, &mut authored);
        for diagnostic in &observed.semantic.diagnostics {
            if let CandidateTrigger::Authored { evidence, .. } = &diagnostic.trigger { authored.push(evidence.clone()); }
        }
        for closure in &observed.semantic.closures {
            if let CandidateTrigger::Authored { evidence, .. } = &closure.trigger { authored.push(evidence.clone()); }
        }
        assert!(authored.iter().all(|evidence| evidence.source_id == SourceId::new(73)), "{id_value}");
        for closure in &observed.semantic.closures {
            let CandidateTrigger::Authored { evidence, .. } = &closure.trigger else { unreachable!() };
            let mut origins = Vec::new(); collect_evidence(&observed.semantic.tree, &mut origins);
            assert!(!origins.iter().any(|origin| origin.range == evidence.range), "{id_value}: end tag is not node origin/text contribution");
        }
    }
}

#[test]
fn semantic_identities_are_contiguous_and_relationships_are_not_storage_indices() {
    for id_value in CANDIDATE_IDS {
        let observed = observe_fixture(id_value, 1);
        let mut ids = Vec::new(); collect_ids(&observed.semantic.tree, &mut ids);
        ids.sort_by_key(|id| id.0);
        assert_eq!(ids, (0..observed.semantic.identity_count).map(CandidateNodeId).collect::<Vec<_>>(), "{id_value}");
        assert_eq!(observed.semantic.identity_count, node_count(&observed.semantic.tree));
    }

    let run = run_for("DV3", 1);
    let baseline = observe_with_layout(&run, CandidateStorageLayout::COMPACT);
    for padding in [1usize, 2, 3, 7] {
        assert_eq!(observe_with_layout(&run, CandidateStorageLayout { padding_before_each_node: padding }).semantic, baseline.semantic);
    }

    let mut compact = CandidateSession::new(CandidateStorageLayout::COMPACT);
    let mut padded = CandidateSession::new(CandidateStorageLayout { padding_before_each_node: 3 });
    for (index, token) in run.tokens().iter().take(3).enumerate() {
        let trigger = candidate_trigger(token, index);
        compact.process(index, candidate_shape(token).expect("shape"), trigger.clone());
        padded.process(index, candidate_shape(token).expect("shape"), trigger);
    }
    assert_eq!(compact.open_elements, padded.open_elements, "semantic stack identities are layout-independent");
    for semantic_id in &compact.open_elements {
        assert_ne!(compact.storage_index(*semantic_id), padded.storage_index(*semantic_id), "physical storage moves while semantic identity stays fixed");
    }
}

#[test]
fn lower_layer_incompleteness_is_never_upgraded_and_diagnostics_are_orthogonal() {
    let dv12 = observe_fixture("DV12", 1);
    assert_eq!(dv12.semantic.checkpoint.completion, CandidateCompletion::IncompleteLowerLayer);
    assert_eq!(dv12.semantic.checkpoint.committed_prefix_end, 11);
    assert_eq!(dv12.semantic.closures, vec![]);
    for limits in [
        HtmlTokenizerLimits::new(4, 8_192, 1_024, 1_024, 256, 4_096, 1_024),
        HtmlTokenizerLimits::new(1_024, 3, 1_024, 1_024, 256, 4_096, 1_024),
        HtmlTokenizerLimits::new(1_024, 0, 1_024, 1_024, 256, 4_096, 1_024),
    ] {
        let run = tokenize_text(fixture("DV1").source_text(), 1, limits);
        assert!(run.is_incomplete());
        assert_ne!(observe(&run).semantic.checkpoint.completion, CandidateCompletion::Complete);
    }
    assert_eq!(observe_fixture("DV5", 1).semantic.checkpoint.completion, CandidateCompletion::Complete);
    assert_eq!(observe_fixture("DV6", 1).semantic.checkpoint.completion, CandidateCompletion::Complete);
}

#[test]
fn selected_cells_terminate_structurally_without_runtime_budget() {
    for id_value in CANDIDATE_IDS {
        let observed = observe_fixture(id_value, 1);
        for record in &observed.tokens {
            let modes: Vec<_> = record.dispatches.iter().map(|dispatch| dispatch.evaluated_in).collect();
            for (position, mode) in modes.iter().enumerate() { assert!(!modes[..position].contains(mode), "{id_value}"); }
            assert!(modes.len() <= CandidateMode::ALL.len(), "{id_value}");
            if record.mode_before == CandidateMode::InBody { assert_eq!(record.dispatches.len(), 1, "{id_value}"); }
        }
    }
}

fn generated_candidate_sources() -> Vec<String> {
    const PIECES: [&str; 3] = ["<div>", "</div>", "x"];
    const MAX_SEQUENCE_LENGTH: u32 = 4;
    const MAX_BALANCED_DEPTH: usize = 8;
    let mut sources = Vec::new();
    for length in 0..=MAX_SEQUENCE_LENGTH {
        for code in 0..PIECES.len().pow(length) {
            let mut remaining = code;
            let mut digits = Vec::new();
            for _ in 0..length { digits.push(remaining % PIECES.len()); remaining /= PIECES.len(); }
            digits.reverse();
            let mut source = String::from("<body>");
            for digit in digits { source.push_str(PIECES[digit]); }
            sources.push(source);
        }
    }
    for depth in 0..=MAX_BALANCED_DEPTH {
        let mut source = String::from("<body>");
        for _ in 0..depth { source.push_str("<div>"); }
        source.push('x');
        for _ in 0..depth { source.push_str("</div>"); }
        sources.push(source);
    }
    sources
}

#[test]
fn generated_sequences_uphold_stack_closure_identity_and_completion_theorems() {
    let sources = generated_candidate_sources();
    assert_eq!(sources.len(), 130);
    for source in sources {
        let run = tokenize_text(&source, 1, generous_limits());
        assert!(!run.is_incomplete(), "{source:?}");
        let observed = observe(&run);
        assert_eq!(observed.semantic.checkpoint.completion, CandidateCompletion::Complete, "{source:?}");
        assert_eq!(observed.semantic.identity_count, node_count(&observed.semantic.tree), "{source:?}");
        assert!(observed.semantic.closures.iter().all(|closure| closure.element.0 < observed.semantic.identity_count), "{source:?}");
        assert_eq!(observe(&tokenize_text(&source, 1, generous_limits())).semantic, observed.semantic, "{source:?}");
    }
}

#[test]
fn candidate_semantics_and_authored_provenance_are_deterministic_across_source_ids() {
    for id_value in CANDIDATE_IDS {
        for source_id in [1_u64, 7, 4_242, u64::from(u32::MAX)] {
            assert_eq!(observe_fixture(id_value, source_id).semantic, candidate_gold(id_value, source_id), "{id_value} SourceId {source_id}");
        }
    }
}

#[test]
fn candidate_widens_nothing_beyond_selected_in_body_no_attribute_div_cells() {
    for (source, expected) in [
        (" ", CandidateUnsupported::WhitespaceSensitiveCharacterData),
        ("\t<body>", CandidateUnsupported::WhitespaceSensitiveCharacterData),
        ("<body><p>", CandidateUnsupported::OutsideModelledCandidateCells),
        ("<body><span>", CandidateUnsupported::OutsideModelledCandidateCells),
        ("<body a>", CandidateUnsupported::TagWithAttributes),
        ("<body/>", CandidateUnsupported::SelfClosingTag),
        ("<body><div id=x>", CandidateUnsupported::TagWithAttributes),
        ("<body><div/>", CandidateUnsupported::SelfClosingTag),
        ("<div>", CandidateUnsupported::DivTagOutsideInBody),
        ("</div>", CandidateUnsupported::DivTagOutsideInBody),
        ("<body></body><div>", CandidateUnsupported::DivTagOutsideInBody),
        ("<body><div></body>", CandidateUnsupported::ShellInteractionWithOpenDiv),
        ("<body><div><body>", CandidateUnsupported::ShellInteractionWithOpenDiv),
        ("<body><div></html>", CandidateUnsupported::ShellInteractionWithOpenDiv),
    ] {
        let run = tokenize_text(source, 1, generous_limits());
        assert!(!run.is_incomplete(), "{source:?}");
        let observed = observe(&run);
        let CandidateCompletion::IncompleteUnsupported { capability, .. } = observed.semantic.checkpoint.completion else { panic!("expected refusal for {source:?}") };
        assert_eq!(capability, expected, "{source:?}");
        assert_eq!(observed.semantic.identity_count, node_count(&observed.semantic.tree));
    }
}

#[test]
fn historical_body_p_boundary_remains_exactly_unsupported_at_six_to_nine() {
    let observed = observe_fixture("DV11", 1);
    assert_eq!(observed.semantic, candidate_gold("DV11", 1));
    let CandidateCompletion::IncompleteUnsupported { trigger, .. } = observed.semantic.checkpoint.completion else { unreachable!() };
    assert_eq!(trigger, authored_trigger(1, 1, (6, 9)));
}
